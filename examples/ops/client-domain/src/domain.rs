//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides, family addresses and application vocabulary.
use crate::{
    Ask, Event, Fact, Limits, Request, infrastructure,
    owner::Owner,
    route,
    store::{Key, Record},
};
use skein_lib::{Env, Queue, RequestAnswered, RequestOut, RequestProgress, RequestTable, Rng, Time};
/// The three children and their shared durable request table.
#[derive(Debug)]
pub struct Domain {
    pub(crate) jig: jig_client::Domain,
    pub(crate) infrastructure: infrastructure::Side,
    pub(crate) observability: crate::observability::Side,
    pub(crate) table: RequestTable<Owner, Ask>,
}
#[must_use]
pub fn new(limits: &Limits, seed: u64) -> Domain {
    assert!(crate::worst_case(limits).is_some(), "root capacities are checked");
    let mut rng = Rng::new(seed);
    Domain {
        jig: jig_client::Domain::new(&limits.jig, rng.next_u64()),
        infrastructure: infrastructure::Side::new(&limits.infrastructure, rng.next_u64()),
        observability: crate::observability::Side::new(&limits.observability, rng.next_u64()),
        table: RequestTable::new(&limits.table, rng.next_u64()),
    }
}
impl Domain {
    #[must_use]
    pub const fn jig(&self) -> &jig_client::Domain {
        &self.jig
    }
    #[must_use]
    pub const fn infrastructure(&self) -> &infrastructure::Side {
        &self.infrastructure
    }
    #[must_use]
    pub const fn observability(&self) -> &crate::observability::Side {
        &self.observability
    }
}
pub fn step(root: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    route::event(root, env, event, out);
}
pub fn fire(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(jig_client::max_out(&env.limits.jig));
    jig_client::fire(&mut root.jig, &route::jig_env(env), &mut child);
    route::route_jig(root, env, &mut child, out);
    let mut child = Queue::with_capacity(infrastructure::max_out(&env.limits.infrastructure));
    infrastructure::fire(
        &mut root.infrastructure,
        &Env { now: env.now, wall: env.wall, limits: env.limits.infrastructure },
        &mut child,
    );
    route::route_infrastructure(root, env, &mut child, out);
    let mut child = Queue::with_capacity(crate::observability::max_out(&env.limits.observability));
    crate::observability::fire(
        &mut root.observability,
        &Env { now: env.now, wall: env.wall, limits: env.limits.observability },
        &mut child,
    );
    route::route_observability(root, env, &mut child, out);
    root.table.fire(env.now, env.wall);
}
#[must_use]
pub fn next_deadline(root: &Domain) -> Option<Time> {
    let mut next = jig_client::next_deadline(&root.jig);
    for deadline in [
        infrastructure::next_deadline(&root.infrastructure),
        crate::observability::next_deadline(&root.observability),
        root.table.next_deadline(),
    ]
    .into_iter()
    .flatten()
    {
        next = match next {
            Some(old) => Some(old.min(deadline)),
            None => Some(deadline),
        };
    }
    next
}
/// Iterate's table phase: emit stores before sends, report progress, reclaim.
pub fn take(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    {
        let mut cursor = root.table.take(env.now, env.wall);
        for _ in 0..env.limits.table.out {
            match cursor.next_out() {
                Some(RequestOut::Save(record)) => out.push(Request::Save {
                    record: Record::Table { version: crate::store::VERSION, record: record.clone() },
                }),
                Some(RequestOut::Erase(key)) => out.push(Request::Erase { key: Key::Table(key) }),
                Some(RequestOut::Send { attempt, key, request }) => {
                    out.push(Request::Send { attempt, key, ask: request.clone() });
                }
                None => break,
            }
        }
    }
    for _ in 0..env.limits.table.requests {
        let Some((owner, progress)) = root.table.progress() else {
            break;
        };
        route::progress(root, env, owner, progress, out);
    }
    root.table.reclaim();
    jig_client::reclaim(&mut root.jig);
    infrastructure::reclaim(&mut root.infrastructure);
    crate::observability::reclaim(&mut root.observability);
}
#[must_use]
pub fn shown(root: &Domain) -> u64 {
    root.jig
        .shown()
        .checked_add(infrastructure::shown(&root.infrastructure))
        .expect("infrastructure shown count fits")
        .checked_add(crate::observability::shown(&root.observability))
        .expect("observability shown count fits")
}
pub(crate) fn answered(
    root: &mut Domain,
    env: &Env<Limits>,
    attempt: skein_lib::Token,
    envelope: skein_lib::RequestEnvelope,
    answer: Option<crate::Answered>,
    out: &mut Queue<Request>,
) {
    match root.table.answered(env.now, attempt, envelope) {
        RequestAnswered::Final(owner) => match owner {
            Owner::Jig(request) => match answer {
                Some(crate::Answered::Jig(answer)) => {
                    route::jig(root, env, jig_client::Event::Answered { request, answer }, out);
                }
                Some(crate::Answered::Infrastructure(_)) | None => {
                    out.push(Request::Fact { fact: Fact::InvalidAnswer });
                }
            },
            Owner::Infrastructure(request) => match answer {
                Some(crate::Answered::Infrastructure(answer)) => {
                    route::infrastructure(root, env, infrastructure::Event::Answered { request, answer }, out);
                }
                Some(crate::Answered::Jig(_)) | None => out.push(Request::Fact { fact: Fact::InvalidAnswer }),
            },
            Owner::Observability(_) => out.push(Request::Fact { fact: Fact::InvalidAnswer }),
            Owner::Kept { scope } => {
                out.push(Request::Fact { fact: Fact::KeptAnswered });
                route::jig(root, env, jig_client::Event::KeptGone { scope }, out);
            }
        },
        RequestAnswered::Pending => match envelope {
            skein_lib::RequestEnvelope::SignedOut => route::jig(root, env, jig_client::Event::SignedOut, out),
            skein_lib::RequestEnvelope::Final
            | skein_lib::RequestEnvelope::Again
            | skein_lib::RequestEnvelope::Lost => {}
        },
        RequestAnswered::Stale => {}
    }
}
pub(crate) fn ask(root: &mut Domain, env: &Env<Limits>, owner: Owner, scope: u64, ask: Ask, out: &mut Queue<Request>) {
    let size = ask_size(&env.limits, &ask);
    if root.table.ask(owner, scope, size, ask).is_err() {
        out.push(Request::Fact { fact: Fact::TableFull });
        route::progress(root, env, owner, RequestProgress::Unknown, out);
    }
}
pub(crate) fn ask_size(limits: &Limits, ask: &Ask) -> u64 {
    match ask {
        Ask::Jig(_) => limits.jig.document_bytes,
        Ask::Infrastructure(ask) => {
            let base = u64::try_from(size_of::<infrastructure::Ask>()).expect("ask size fits");
            match ask {
                infrastructure::Ask::MoveEnd { .. } => base,
                infrastructure::Ask::NativeMoveEnd { environment, .. } => base
                    .saturating_add(u64::try_from(environment.pool.len()).expect("name size fits"))
                    .saturating_add(u64::try_from(environment.name.len()).expect("name size fits")),
            }
        }
    }
}

/// Installs application-configured provider words before Start (client.md, 12).
pub fn provider(root: &mut Domain, provider: jig_client::signin::Provider) -> Result<(), jig_client::signin::Provider> {
    root.jig.provider(provider)
}

/// Installs the application's navigation before Start; the jig child validates its bound.
pub fn configure_navigation(
    root: &mut Domain,
    navigation: jig_client::pages::navigation::Navigation,
) -> Result<(), jig_client::pages::navigation::Navigation> {
    root.jig.configure_navigation(navigation)
}
