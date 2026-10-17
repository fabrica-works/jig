//! Exhaustive wrapping and synchronous hand-offs. The root never interprets
//! child answers or application documents (client/domain/root.md, section 5).
use crate::{
    Action, Address, Ask, Domain, Event, Fact, JigOut, Limits, Place, Request, first,
    owner::Owner,
    store::{Key, Record},
};
use skein_lib::{Env, List, Map, Queue, RequestProgress};
pub(crate) fn jig_env(env: &Env<Limits>) -> Env<jig_client_view::Limits> {
    Env { now: env.now, wall: env.wall, limits: env.limits.jig }
}
fn first_env(env: &Env<Limits>) -> Env<first::Limits> {
    Env { now: env.now, wall: env.wall, limits: env.limits.first }
}
pub(crate) fn jig(root: &mut Domain, env: &Env<Limits>, event: jig_client_view::Event, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(jig_client_view::max_out(&env.limits.jig));
    jig_client_view::step(&mut root.jig, &jig_env(env), event, &mut child);
    route_jig(root, env, &mut child, out);
}
pub(crate) fn first(root: &mut Domain, env: &Env<Limits>, event: first::Event, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(first::max_out(&env.limits.first));
    first::step(&mut root.first, &first_env(env), event, &mut child);
    route_first(root, env, &mut child, out);
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
                    Record::First(record) => first(root, env, first::Event::Restore { record }, out),
                    Record::Table { version, record } => {
                        let size = crate::domain::ask_size(&env.limits, &record.request.request);
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
            let jig_address = match address {
                Address::Jig(address) => address,
                Address::First(_) => jig_client_view::address::Address::Inbox { project: None },
            };
            jig(root, env, jig_client_view::Event::Start { address: jig_address, kept: jig_kept, offset }, out);
            for (scope, requests) in &scopes {
                jig(root, env, jig_client_view::Event::Kept { scope: *scope, requests: *requests }, out);
            }
            match address {
                Address::Jig(_) => first(root, env, first::Event::Left, out),
                Address::First(address) => went(root, env, Place::Address(Address::First(address)), out),
            }
        }
        Event::Went { place } => went(root, env, place, out),
        Event::Act { action } => match action {
            Action::Jig(act) => jig(root, env, jig_client_view::Event::Act { act }, out),
            Action::First(act) => first(root, env, first::Event::Act { act }, out),
        },
        Event::Answer { attempt, envelope, answer } => {
            crate::domain::answered(root, env, attempt, envelope, answer, out);
        }
        Event::Jig(event) => jig(root, env, event, out),
        Event::First(event) => first(root, env, event, out),
        Event::SignedOut => jig(root, env, jig_client_view::Event::SignedOut, out),
    }
}
fn went(root: &mut Domain, env: &Env<Limits>, place: Place, out: &mut Queue<Request>) {
    match place {
        Place::Address(Address::Jig(address)) => {
            first(root, env, first::Event::Left, out);
            jig(root, env, jig_client_view::Event::Went { place: jig_client_view::address::Place::Jig(address) }, out);
        }
        Place::Address(Address::First(address)) => {
            jig(root, env, jig_client_view::Event::Went { place: jig_client_view::address::Place::Elsewhere }, out);
            first(root, env, first::Event::Went { address }, out);
        }
        Place::Missing => {
            first(root, env, first::Event::Left, out);
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
                first(root, env, first::Event::PartsLeft { generation }, out);
                root.second.left(generation);
            }
            jig_client_view::Request::Link { link } => {
                first(root, env, first::Event::Link { link }, out);
                root.second.link_is(link);
                root.table.link(match link {
                    jig_client_view::link::Link::Live => true,
                    jig_client_view::link::Link::Starting | jig_client_view::link::Link::Offline => false,
                });
            }
            jig_client_view::Request::Confirmed { party } => {
                first(root, env, first::Event::Confirmed { party }, out);
                root.second.confirmed(party);
                root.table.confirmed(party);
            }
            jig_client_view::Request::Retention { retention } => root.table.retention(retention),
            jig_client_view::Request::FrameIs { asked, frame } => {
                first(root, env, first::Event::FrameIs { asked, frame }, out);
            }
            jig_client_view::Request::Answered { asked, answer } => {
                out.push(Request::Jig(JigOut::Answered { asked, answer }));
            }
            jig_client_view::Request::Unknown { asked } => out.push(Request::Jig(JigOut::Unknown { asked })),
            jig_client_view::Request::Fact { fact } => out.push(Request::Fact { fact: Fact::Jig(fact) }),
        }
    }
}
pub(crate) fn route_first(
    root: &mut Domain,
    env: &Env<Limits>,
    child: &mut Queue<first::Request>,
    out: &mut Queue<Request>,
) {
    for _ in 0..child.len() {
        match child.pop().expect("counted first output") {
            first::Request::Frame { asked } => jig(root, env, jig_client_view::Event::Frame { asked }, out),
            first::Request::Show { address } => {
                out.push(Request::Show { address: Address::First(address), replace: false });
            }
            first::Request::Save { record } => out.push(Request::Save { record: Record::First(record) }),
            first::Request::Send { request, scope, ask } => {
                crate::domain::ask(root, env, Owner::First(request), scope, Ask::First(ask), out);
            }
            first::Request::Out(value) => match value {
                first::Out::Fact { fact: first::Fact::SignedOut } => {
                    jig(root, env, jig_client_view::Event::SignedOut, out);
                }
                first::Out::Read { .. }
                | first::Out::Open { .. }
                | first::Out::Close { .. }
                | first::Out::Fact { fact: first::Fact::Full | first::Fact::InvalidDocument } => {
                    out.push(Request::First(value));
                }
            },
            first::Request::ShowObjects { objects } => {
                jig(root, env, jig_client_view::Event::ShowObjects { objects }, out);
            }
            first::Request::ShowSections { sections } => {
                jig(root, env, jig_client_view::Event::ShowSections { sections }, out);
            }
            first::Request::ObjectsLeft => jig(root, env, jig_client_view::Event::ObjectsLeft, out),
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
        Owner::First(request) => first(root, env, first::Event::Progress { request, progress }, out),
        Owner::Kept { scope } => match progress {
            RequestProgress::Unknown => jig(root, env, jig_client_view::Event::KeptGone { scope }, out),
            RequestProgress::InFlight | RequestProgress::Retrying | RequestProgress::Parked => {}
        },
    }
}
