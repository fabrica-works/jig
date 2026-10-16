//! The root's one shape, client/domain/root.md, section 3. State is exactly
//! the three children and their typed request table; no token is inspected.
use crate::{
    Ask, Event, Fact, Limits, Request, first,
    owner::Owner,
    route,
    store::{Held, Key, Record},
};
use skein_lib::{Env, Queue, RequestAnswered, RequestOut, RequestProgress, RequestTable, Rng, Time};
#[derive(Debug)]
pub struct Domain {
    pub(crate) jig: jig_domain_client::Domain,
    pub(crate) first: first::Side,
    pub(crate) second: crate::second::Side,
    pub(crate) table: RequestTable<Owner, Held>,
}
#[must_use]
pub fn new(limits: &Limits, seed: u64) -> Domain {
    assert!(crate::worst_case(limits).is_some(), "root capacities are checked");
    let mut rng = Rng::new(seed);
    Domain {
        jig: jig_domain_client::Domain::new(&limits.jig, rng.next_u64()),
        first: first::Side::new(&limits.first, rng.next_u64()),
        second: crate::second::Side::new(limits.second_parts, limits.first.rows, limits.first.text, rng.next_u64()),
        table: RequestTable::new(&limits.table, rng.next_u64()),
    }
}
impl Domain {
    #[must_use]
    pub const fn jig(&self) -> &jig_domain_client::Domain {
        &self.jig
    }
    #[must_use]
    pub const fn first(&self) -> &first::Side {
        &self.first
    }
    #[must_use]
    pub const fn second(&self) -> &crate::second::Side {
        &self.second
    }
}
pub fn step(root: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    route::event(root, env, event, out);
}
pub fn fire(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(jig_domain_client::max_out(&env.limits.jig));
    jig_domain_client::fire(&mut root.jig, &route::jig_env(env), &mut child);
    route::route_jig(root, env, &mut child, out);
    let mut child = Queue::with_capacity(first::max_out(&env.limits.first));
    first::fire(&mut root.first, &Env { now: env.now, wall: env.wall, limits: env.limits.first }, &mut child);
    route::route_first(root, env, &mut child, out);
    root.table.fire(env.now, env.wall);
}
#[must_use]
pub fn next_deadline(root: &Domain) -> Option<Time> {
    let mut next = jig_domain_client::next_deadline(&root.jig);
    for deadline in [first::next_deadline(&root.first), root.table.next_deadline()].into_iter().flatten() {
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
                    out.push(Request::Send { attempt, key, ask: request.request.clone() });
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
    jig_domain_client::reclaim(&mut root.jig);
    first::reclaim(&mut root.first);
}
#[must_use]
pub fn shown(root: &Domain) -> u64 {
    root.jig
        .shown()
        .checked_add(first::shown(&root.first))
        .expect("first shown count fits")
        .checked_add(root.second.shown())
        .expect("second shown count fits")
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
                    route::jig(root, env, jig_domain_client::Event::Answered { request, answer }, out);
                }
                Some(crate::Answered::First(_)) | None => out.push(Request::Fact { fact: Fact::InvalidAnswer }),
            },
            Owner::First(request) => match answer {
                Some(crate::Answered::First(answer)) => {
                    route::first(root, env, first::Event::Answered { request, answer }, out);
                }
                Some(crate::Answered::Jig(_)) | None => out.push(Request::Fact { fact: Fact::InvalidAnswer }),
            },
            Owner::Kept { scope } => {
                out.push(Request::Fact { fact: Fact::KeptAnswered });
                route::jig(root, env, jig_domain_client::Event::KeptGone { scope }, out);
            }
        },
        RequestAnswered::Pending => match envelope {
            skein_lib::RequestEnvelope::SignedOut => route::jig(root, env, jig_domain_client::Event::SignedOut, out),
            skein_lib::RequestEnvelope::Final
            | skein_lib::RequestEnvelope::Again
            | skein_lib::RequestEnvelope::Lost => {}
        },
        RequestAnswered::Stale => {}
    }
}
pub(crate) fn ask(root: &mut Domain, env: &Env<Limits>, owner: Owner, scope: u64, ask: Ask, out: &mut Queue<Request>) {
    let size = ask_size(&env.limits, &ask);
    let key = match owner {
        Owner::Jig(token) => jig_client_domain::Key { asker: jig_client_domain::Asker::JIG, number: token.raw() },
        Owner::First(token) => {
            jig_client_domain::Key { asker: jig_client_domain::Asker(u32::from(crate::FIRST)), number: token.raw() }
        }
        Owner::Kept { .. } => unreachable!("restored requests do not ask"),
    };
    if root.table.ask(owner, scope, size, Held { key, request: ask }).is_err() {
        out.push(Request::Fact { fact: Fact::TableFull });
        route::progress(root, env, owner, RequestProgress::Unknown, out);
    }
}
pub(crate) fn ask_size(limits: &Limits, ask: &Ask) -> u64 {
    match ask {
        Ask::Jig(_) => limits.jig.document_bytes,
        Ask::First(_) => u64::try_from(size_of::<first::Ask>()).expect("ask size fits"),
    }
}

/// Installs application-configured provider words before Start (client.md, 12).
pub fn provider(
    root: &mut Domain,
    provider: jig_domain_client::signin::Provider,
) -> Result<(), jig_domain_client::signin::Provider> {
    root.jig.provider(provider)
}

/// Installs the application's arrangement before Start; jig validates it.
pub fn configure_arrangement(
    root: &mut Domain,
    arrangement: jig_domain_client::pages::Arrangement,
) -> Result<(), jig_domain_client::pages::Arrangement> {
    root.jig.configure_arrangement(arrangement)
}
