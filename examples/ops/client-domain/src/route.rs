//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides, family addresses and application vocabulary.
use crate::observability;
use crate::{
    Action, Address, Ask, Domain, Event, Fact, JigOut, Limits, Place, Request, infrastructure,
    owner::Owner,
    store::{Key, Record},
};
use skein_lib::{Env, List, Map, Queue, RequestProgress};
pub(crate) fn jig_env(env: &Env<Limits>) -> Env<jig_client_view::Limits> {
    Env { now: env.now, wall: env.wall, limits: env.limits.jig }
}
fn infrastructure_env(env: &Env<Limits>) -> Env<infrastructure::Limits> {
    Env { now: env.now, wall: env.wall, limits: env.limits.infrastructure }
}
pub(crate) fn jig(root: &mut Domain, env: &Env<Limits>, event: jig_client_view::Event, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(jig_client_view::max_out(&env.limits.jig));
    jig_client_view::step(&mut root.jig, &jig_env(env), event, &mut child);
    route_jig(root, env, &mut child, out);
}
pub(crate) fn infrastructure(
    root: &mut Domain,
    env: &Env<Limits>,
    event: infrastructure::Event,
    out: &mut Queue<Request>,
) {
    let mut child = Queue::with_capacity(infrastructure::max_out(&env.limits.infrastructure));
    infrastructure::step(&mut root.infrastructure, &infrastructure_env(env), event, &mut child);
    route_infrastructure(root, env, &mut child, out);
}
pub(crate) fn observability(
    root: &mut Domain,
    env: &Env<Limits>,
    event: observability::Event,
    out: &mut Queue<Request>,
) {
    let mut child = Queue::with_capacity(observability::max_out(&env.limits.observability));
    observability::step(
        &mut root.observability,
        &Env { now: env.now, wall: env.wall, limits: env.limits.observability },
        event,
        &mut child,
    );
    route_observability(root, env, &mut child, out);
}
pub(crate) fn event(root: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Start { address, kept, offset } => {
            let mut scopes = Map::<u64, u32>::with_capacity(env.limits.table.requests);
            let mut jig_kept = List::with_capacity(env.limits.jig.records);
            for record in kept.into_boxed() {
                match record {
                    Record::Jig(record) => {
                        if jig_kept.push(record).is_err() {
                            out.push(Request::Fact { fact: Fact::InvalidRecord });
                        }
                    }
                    Record::Infrastructure(record) => {
                        infrastructure(root, env, infrastructure::Event::Restore { record }, out);
                    }
                    Record::Observability(record) => {
                        observability(root, env, observability::Event::Restore { record }, out);
                    }
                    Record::Table { version, record } => {
                        let size = crate::domain::ask_size(&env.limits, &record.request);
                        let scope = record.scope;
                        if version != crate::store::VERSION
                            || root.table.restore(Owner::Kept { scope }, size, record).is_err()
                        {
                            out.push(Request::Fact { fact: Fact::InvalidRecord });
                        } else {
                            let requests = scopes
                                .get(&scope)
                                .copied()
                                .unwrap_or(0)
                                .checked_add(1)
                                .expect("restored request count is bounded");
                            scopes.insert(scope, requests).expect("one scope per admitted restored request");
                        }
                    }
                }
            }
            let jig_address = match address.clone() {
                Address::Jig(address) => address,
                Address::Infrastructure(_) | Address::Observability(_) => {
                    jig_client_view::address::Address::Inbox { project: None }
                }
            };
            jig(root, env, jig_client_view::Event::Start { address: jig_address, kept: jig_kept, offset }, out);
            for (scope, requests) in &scopes {
                jig(root, env, jig_client_view::Event::Kept { scope: *scope, requests: *requests }, out);
            }
            match address {
                Address::Jig(_) => {
                    infrastructure(root, env, infrastructure::Event::Left, out);
                    observability(root, env, observability::Event::Left, out);
                }
                Address::Infrastructure(address) => {
                    went(root, env, Place::Address(Address::Infrastructure(address)), out);
                }
                Address::Observability(address) => {
                    went(root, env, Place::Address(Address::Observability(address)), out);
                }
            }
        }
        Event::Went { place } => went(root, env, place, out),
        Event::Act { action } => match action {
            Action::Jig(act) => jig(root, env, jig_client_view::Event::Act { act }, out),
            Action::Infrastructure(act) => infrastructure(root, env, infrastructure::Event::Act { act }, out),
            Action::Observability(act) => observability(root, env, observability::Event::Act { act }, out),
        },
        Event::Answer { attempt, envelope, answer } => {
            crate::domain::answered(root, env, attempt, envelope, answer, out);
        }
        Event::Jig(event) => jig(root, env, event, out),
        Event::Infrastructure(event) => infrastructure(root, env, event, out),
        Event::Observability(event) => observability(root, env, event, out),
        Event::SignedOut => jig(root, env, jig_client_view::Event::SignedOut, out),
    }
}
fn went(root: &mut Domain, env: &Env<Limits>, place: Place, out: &mut Queue<Request>) {
    match place {
        Place::Address(Address::Jig(address)) => {
            infrastructure(root, env, infrastructure::Event::Left, out);
            observability(root, env, observability::Event::Left, out);
            jig(root, env, jig_client_view::Event::Went { place: jig_client_view::address::Place::Jig(address) }, out);
        }
        Place::Address(Address::Infrastructure(address)) => {
            observability(root, env, observability::Event::Left, out);
            jig(root, env, jig_client_view::Event::Went { place: jig_client_view::address::Place::Elsewhere }, out);
            infrastructure(root, env, infrastructure::Event::Went { address }, out);
        }
        Place::Address(Address::Observability(address)) => {
            infrastructure(root, env, infrastructure::Event::Left, out);
            jig(root, env, jig_client_view::Event::Went { place: jig_client_view::address::Place::Elsewhere }, out);
            observability(root, env, observability::Event::Went { address }, out);
        }
        Place::Missing => {
            infrastructure(root, env, infrastructure::Event::Left, out);
            observability(root, env, observability::Event::Left, out);
            jig(root, env, jig_client_view::Event::Went { place: jig_client_view::address::Place::Missing }, out);
        }
    }
}
pub(crate) fn route_jig(
    root: &mut Domain,
    env: &Env<Limits>,
    child: &mut Queue<jig_client_view::Request>,
    out: &mut Queue<Request>,
) {
    for _ in 0..child.len() {
        match child.pop().expect("counted jig output") {
            jig_client_view::Request::Send { request, scope, ask } => {
                crate::domain::ask(root, env, Owner::Jig(request), scope, Ask::Jig(ask), out);
            }
            jig_client_view::Request::Read { read, what } => out.push(Request::Jig(JigOut::Read { read, what })),
            jig_client_view::Request::Open { watch, subject } => {
                out.push(Request::Jig(JigOut::Open { watch, subject }));
            }
            jig_client_view::Request::Continue { watch, subject } => {
                out.push(Request::Jig(JigOut::Continue { watch, subject }));
            }
            jig_client_view::Request::Close { watch } => out.push(Request::Jig(JigOut::Close { watch })),
            jig_client_view::Request::Show { address, replace } => {
                out.push(Request::Show { address: Address::Jig(address), replace });
            }
            jig_client_view::Request::Save { record } => out.push(Request::Save { record: Record::Jig(record) }),
            jig_client_view::Request::Erase { key } => out.push(Request::Erase { key: Key::Jig(key) }),
            jig_client_view::Request::SignOut => out.push(Request::SignOut),
            jig_client_view::Request::SignIn { provider, back } => {
                out.push(Request::SignIn { provider, back: Address::Jig(back) });
            }
            jig_client_view::Request::ShowPart { part, generation, scope } => {
                crate::parts::show(root, env, part, generation, scope, out);
            }
            jig_client_view::Request::PartsLeft { generation } => {
                infrastructure(root, env, infrastructure::Event::PartsLeft { generation }, out);
                observability(root, env, observability::Event::PartsLeft { generation }, out);
            }
            jig_client_view::Request::Link { link } => {
                infrastructure(root, env, infrastructure::Event::Link { link }, out);
                observability(root, env, observability::Event::Link { link }, out);
                root.table.link(match link {
                    jig_client_view::link::Link::Live => true,
                    jig_client_view::link::Link::Starting | jig_client_view::link::Link::Offline => false,
                });
            }
            jig_client_view::Request::Confirmed { party } => {
                infrastructure(root, env, infrastructure::Event::Confirmed { party }, out);
                observability(root, env, observability::Event::Confirmed { party }, out);
                root.table.confirmed(party);
            }
            jig_client_view::Request::Retention { retention } => root.table.retention(retention),
            jig_client_view::Request::FrameIs { asked, frame } => {
                infrastructure(root, env, infrastructure::Event::FrameIs { asked, frame: frame.clone() }, out);
                observability(root, env, observability::Event::FrameIs { asked, frame }, out);
            }
            jig_client_view::Request::Answered { asked, answer } => {
                out.push(Request::Jig(JigOut::Answered { asked, answer }));
            }
            jig_client_view::Request::Unknown { asked } => out.push(Request::Jig(JigOut::Unknown { asked })),
            jig_client_view::Request::Fact { fact } => out.push(Request::Fact { fact: Fact::Jig(fact) }),
        }
    }
}
pub(crate) fn route_infrastructure(
    root: &mut Domain,
    env: &Env<Limits>,
    child: &mut Queue<infrastructure::Request>,
    out: &mut Queue<Request>,
) {
    for _ in 0..child.len() {
        match child.pop().expect("counted infrastructure output") {
            infrastructure::Request::Frame { asked } => jig(root, env, jig_client_view::Event::Frame { asked }, out),
            infrastructure::Request::Show { address } => {
                out.push(Request::Show { address: Address::Infrastructure(address), replace: false });
            }
            infrastructure::Request::Save { record } => {
                out.push(Request::Save { record: Record::Infrastructure(record) });
            }
            infrastructure::Request::Send { request, scope, ask } => {
                crate::domain::ask(root, env, Owner::Infrastructure(request), scope, Ask::Infrastructure(ask), out);
            }
            infrastructure::Request::Out(value) => match value {
                infrastructure::Out::Fact { fact: infrastructure::Fact::SignedOut } => {
                    jig(root, env, jig_client_view::Event::SignedOut, out);
                }
                infrastructure::Out::Read { .. }
                | infrastructure::Out::Open { .. }
                | infrastructure::Out::Close { .. }
                | infrastructure::Out::Fact {
                    fact:
                        infrastructure::Fact::Full
                        | infrastructure::Fact::InvalidDocument
                        | infrastructure::Fact::ReadNotAllowed
                        | infrastructure::Fact::WatchTerminal { .. },
                } => {
                    out.push(Request::Infrastructure(value));
                }
            },
            infrastructure::Request::ShowObjects { objects } => {
                jig(root, env, jig_client_view::Event::ShowObjects { objects }, out);
            }
            infrastructure::Request::ObjectsLeft => jig(root, env, jig_client_view::Event::ObjectsLeft, out),
        }
    }
}
pub(crate) fn route_observability(
    root: &mut Domain,
    env: &Env<Limits>,
    child: &mut Queue<observability::Request>,
    out: &mut Queue<Request>,
) {
    for _ in 0..child.len() {
        match child.pop().expect("counted observability output") {
            observability::Request::Frame { asked } => jig(root, env, jig_client_view::Event::Frame { asked }, out),
            observability::Request::Show { address } => {
                out.push(Request::Show { address: Address::Observability(address), replace: false });
            }
            observability::Request::Save { record } => {
                out.push(Request::Save { record: Record::Observability(record) });
            }
            observability::Request::Out(value) => match value {
                observability::Out::Fact { fact: observability::Fact::SignedOut } => {
                    jig(root, env, jig_client_view::Event::SignedOut, out);
                }
                observability::Out::Read { .. }
                | observability::Out::Open { .. }
                | observability::Out::Close { .. }
                | observability::Out::Fact {
                    fact:
                        observability::Fact::Full
                        | observability::Fact::InvalidDocument
                        | observability::Fact::ReadNotAllowed
                        | observability::Fact::WatchTerminal { .. },
                } => {
                    out.push(Request::Observability(value));
                }
            },
            observability::Request::ShowObjects { objects } => {
                jig(root, env, jig_client_view::Event::ShowObjects { objects }, out);
            }
            observability::Request::ObjectsLeft => jig(root, env, jig_client_view::Event::ObjectsLeft, out),
            observability::Request::ShowSections { sections } => {
                jig(root, env, jig_client_view::Event::ShowSections { sections }, out);
            }
        }
    }
}
pub(crate) fn progress(
    root: &mut Domain,
    env: &Env<Limits>,
    owner: Owner,
    progress: RequestProgress,
    out: &mut Queue<Request>,
) {
    match owner {
        Owner::Jig(request) => {
            let progress = match progress {
                RequestProgress::InFlight => jig_client_view::intents::Progress::InFlight,
                RequestProgress::Retrying => jig_client_view::intents::Progress::Retrying,
                RequestProgress::Parked => jig_client_view::intents::Progress::Parked,
                RequestProgress::Unknown => jig_client_view::intents::Progress::Unknown,
            };
            jig(root, env, jig_client_view::Event::Progress { request, progress }, out);
        }
        Owner::Infrastructure(request) => {
            infrastructure(root, env, infrastructure::Event::Progress { request, progress }, out);
        }
        Owner::Observability(_) => out.push(Request::Fact { fact: Fact::InvalidAnswer }),
        Owner::Kept { scope } => match progress {
            RequestProgress::Unknown => jig(root, env, jig_client_view::Event::KeptGone { scope }, out),
            RequestProgress::InFlight | RequestProgress::Retrying | RequestProgress::Parked => {}
        },
    }
}
