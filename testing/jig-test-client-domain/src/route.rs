//! Exhaustive wrapping and synchronous hand-offs. The root never interprets
//! child answers or application documents (client/domain/root.md, section 5).
use crate::{
    Action, Address, Ask, Domain, Event, Fact, JigOut, Limits, Place, Request, first,
    owner::Owner,
    store::{Key, Record},
};
use skein_lib::{Env, List, Queue, RequestProgress};
pub(crate) fn jig_env(env: &Env<Limits>) -> Env<jig_client::Limits> {
    Env { now: env.now, wall: env.wall, limits: env.limits.jig }
}
fn first_env(env: &Env<Limits>) -> Env<first::Limits> {
    Env { now: env.now, wall: env.wall, limits: env.limits.first }
}
pub(crate) fn jig(root: &mut Domain, env: &Env<Limits>, event: jig_client::Event, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(jig_client::max_out(&env.limits.jig));
    jig_client::step(&mut root.jig, &jig_env(env), event, &mut child);
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
                        let size = crate::domain::ask_size(&env.limits, &record.request);
                        if version != crate::store::VERSION || root.table.restore(Owner::Kept, size, record).is_err() {
                            out.push(Request::Fact { fact: Fact::InvalidRecord });
                        }
                    }
                }
            }
            let jig_address = match address {
                Address::Jig(address) => address,
                Address::First(_) => jig_client::address::Address::Inbox { project: None },
            };
            jig(root, env, jig_client::Event::Start { address: jig_address, kept: jig_kept, offset }, out);
            match address {
                Address::Jig(_) => first(root, env, first::Event::Left, out),
                Address::First(address) => went(root, env, Place::Address(Address::First(address)), out),
            }
        }
        Event::Went { place } => went(root, env, place, out),
        Event::Act { action } => match action {
            Action::Jig(act) => jig(root, env, jig_client::Event::Act { act }, out),
            Action::First(act) => first(root, env, first::Event::Act { act }, out),
        },
        Event::Answer { attempt, envelope, answer } => {
            crate::domain::answered(root, env, attempt, envelope, answer, out);
        }
        Event::Jig(event) => jig(root, env, event, out),
        Event::First(event) => first(root, env, event, out),
        Event::SignedOut => jig(root, env, jig_client::Event::SignedOut, out),
    }
}
fn went(root: &mut Domain, env: &Env<Limits>, place: Place, out: &mut Queue<Request>) {
    match place {
        Place::Address(Address::Jig(address)) => {
            first(root, env, first::Event::Left, out);
            jig(root, env, jig_client::Event::Went { place: jig_client::address::Place::Jig(address) }, out);
        }
        Place::Address(Address::First(address)) => {
            jig(root, env, jig_client::Event::Went { place: jig_client::address::Place::Elsewhere }, out);
            first(root, env, first::Event::Went { address }, out);
        }
        Place::Missing => {
            first(root, env, first::Event::Left, out);
            jig(root, env, jig_client::Event::Went { place: jig_client::address::Place::Missing }, out);
        }
    }
}
pub(crate) fn route_jig(
    root: &mut Domain,
    env: &Env<Limits>,
    child: &mut Queue<jig_client::Request>,
    out: &mut Queue<Request>,
) {
    for _ in 0..child.len() {
        match child.pop().expect("counted jig output") {
            jig_client::Request::Send { request, scope, ask } => {
                crate::domain::ask(root, env, Owner::Jig(request), scope, Ask::Jig(ask), out);
            }
            jig_client::Request::Read { read, what } => out.push(Request::Jig(JigOut::Read { read, what })),
            jig_client::Request::Open { watch, subject } => out.push(Request::Jig(JigOut::Open { watch, subject })),
            jig_client::Request::Continue { watch, subject } => {
                out.push(Request::Jig(JigOut::Continue { watch, subject }));
            }
            jig_client::Request::Close { watch } => out.push(Request::Jig(JigOut::Close { watch })),
            jig_client::Request::Show { address, replace } => {
                out.push(Request::Show { address: Address::Jig(address), replace });
            }
            jig_client::Request::Save { record } => out.push(Request::Save { record: Record::Jig(record) }),
            jig_client::Request::Erase { key } => out.push(Request::Erase { key: Key::Jig(key) }),
            jig_client::Request::SignIn { provider, back } => {
                out.push(Request::SignIn { provider, back: Address::Jig(back) });
            }
            jig_client::Request::ShowPart { part, generation, scope } => {
                crate::parts::show(root, env, part, generation, scope, out);
            }
            jig_client::Request::PartsLeft { generation } => {
                first(root, env, first::Event::PartsLeft { generation }, out);
                root.second.left(generation);
            }
            jig_client::Request::Link { link } => {
                first(root, env, first::Event::Link { link }, out);
                root.second.link_is(link);
                root.table.link(match link {
                    jig_client::link::Link::Live => true,
                    jig_client::link::Link::Starting | jig_client::link::Link::Offline => false,
                });
            }
            jig_client::Request::Confirmed { party } => {
                first(root, env, first::Event::Confirmed { party }, out);
                root.second.confirmed(party);
                root.table.confirmed(party);
            }
            jig_client::Request::Retention { retention } => root.table.retention(retention),
            jig_client::Request::FrameIs { asked, frame } => {
                first(root, env, first::Event::FrameIs { asked, frame }, out);
            }
            jig_client::Request::Answered { asked, answer } => {
                out.push(Request::Jig(JigOut::Answered { asked, answer }));
            }
            jig_client::Request::Unknown { asked } => out.push(Request::Jig(JigOut::Unknown { asked })),
            jig_client::Request::Fact { fact } => out.push(Request::Fact { fact: Fact::Jig(fact) }),
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
            first::Request::Frame { asked } => jig(root, env, jig_client::Event::Frame { asked }, out),
            first::Request::Show { address } => {
                out.push(Request::Show { address: Address::First(address), replace: false });
            }
            first::Request::Save { record } => out.push(Request::Save { record: Record::First(record) }),
            first::Request::Send { request, scope, ask } => {
                crate::domain::ask(root, env, Owner::First(request), scope, Ask::First(ask), out);
            }
            first::Request::Out(value) => match value {
                first::Out::Fact { fact: first::Fact::SignedOut } => jig(root, env, jig_client::Event::SignedOut, out),
                first::Out::Read { .. }
                | first::Out::Open { .. }
                | first::Out::Close { .. }
                | first::Out::Fact { fact: first::Fact::Full | first::Fact::InvalidDocument } => {
                    out.push(Request::First(value));
                }
            },
            first::Request::ShowObjects { objects } => jig(root, env, jig_client::Event::ShowObjects { objects }, out),
            first::Request::ObjectsLeft => jig(root, env, jig_client::Event::ObjectsLeft, out),
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
                RequestProgress::InFlight => jig_client::intents::Progress::InFlight,
                RequestProgress::Retrying => jig_client::intents::Progress::Retrying,
                RequestProgress::Parked => jig_client::intents::Progress::Parked,
                RequestProgress::Unknown => jig_client::intents::Progress::Unknown,
            };
            jig(root, env, jig_client::Event::Progress { request, progress }, out);
        }
        Owner::First(request) => first(root, env, first::Event::Progress { request, progress }, out),
        Owner::Kept => {}
    }
}
