//! Exhaustive wrapping and synchronous hand-offs. The root never interprets
//! child answers or application documents (client/domain/root.md, section 5).
use crate::{
    Action, Address, Domain, Event, Fact, JigOut, Limits, Place, Request, first,
    store::{Key, Record},
};
use skein_lib::{Env, List, Queue};
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
            let mut jig_kept = List::with_capacity(env.limits.jig.records);
            let mut bytes = List::with_capacity(env.limits.client.domain.requests.requests);
            let mut records = List::with_capacity(env.limits.client.domain.requests.requests);
            for record in kept.into_boxed() {
                match record {
                    Record::Jig(record) => {
                        if jig_kept.push(record).is_err() {
                            out.push(Request::Fact { fact: Fact::InvalidRecord });
                        }
                    }
                    Record::First(record) => first(root, env, first::Event::Restore { record }, out),
                    Record::Client(record) => {
                        bytes.push(record).expect("bounded client records");
                    }
                    Record::Unlinked(record) => {
                        records.push(record).expect("bounded unlinked records");
                    }
                }
            }
            let jig_address = match address {
                Address::Jig(address) => address,
                Address::First(_) => jig_client_view::address::Address::Inbox { project: None },
            };
            let mut child = Queue::with_capacity(jig_client_view::max_out(&env.limits.jig));
            jig_client_view::step(
                &mut root.jig,
                &jig_env(env),
                jig_client_view::Event::Start { address: jig_address, kept: jig_kept, offset },
                &mut child,
            );
            let event = if root.unlinked {
                jig_client::Event::StartUnlinked { kept: records }
            } else {
                jig_client::Event::Start { kept: bytes }
            };
            client(root, env, event, out);
            route_jig(root, env, &mut child, out);
            told(root, env, jig_client::Told::Link { link: root.client.link().state }, out);
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
        Event::Client { event } => client(root, env, event, out),
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
        let request = child.pop().expect("counted jig output");
        if let Some(event) = jig_client_view::bridge::asked(&request, root.salt, env.limits.jig.watches) {
            client(root, env, event, out);
            if let jig_client_view::Request::Close { watch } = request {
                let mut closed = Queue::with_capacity(jig_client_view::max_out(&env.limits.jig));
                jig_client_view::bridge::closed(&mut root.jig, &jig_env(env), watch, &mut closed);
                route_jig(root, env, &mut closed, out);
            }
            continue;
        }
        match request {
            jig_client_view::Request::Send { .. }
            | jig_client_view::Request::Read { .. }
            | jig_client_view::Request::Open { .. }
            | jig_client_view::Request::Close { .. }
            | jig_client_view::Request::SignOut => unreachable!("the bridge takes client work"),
            jig_client_view::Request::Show { address, replace } => {
                out.push(Request::Show { address: Address::Jig(address), replace });
            }
            jig_client_view::Request::Save { record } => out.push(Request::Save { record: Record::Jig(record) }),
            jig_client_view::Request::Erase { key } => out.push(Request::Erase { key: Key::Jig(key) }),
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
            jig_client_view::Request::Continue { .. }
            | jig_client_view::Request::Link { .. }
            | jig_client_view::Request::Retention { .. } => {}
            jig_client_view::Request::Confirmed { party } => {
                first(root, env, first::Event::Confirmed { party }, out);
                root.second.confirmed(party);
            }
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
            first::Request::Send { request, ask, .. } => {
                crate::family::client::send(root, env, request, &ask, out);
            }
            first::Request::Out(first::Out::Fact { fact: first::Fact::SignedOut }) => {
                jig(root, env, jig_client_view::Event::SignedOut, out);
            }
            first::Request::Out(value) => crate::family::client::asked(root, env, value, out),
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
pub(crate) fn client(root: &mut Domain, env: &Env<Limits>, event: jig_client::Event, out: &mut Queue<Request>) {
    let mut child = Queue::with_capacity(jig_client::max_out(&env.limits.client));
    root.client.step(env.now, env.wall, event, &mut child);
    route_client(root, env, &mut child, out);
}
pub(crate) fn route_client(
    root: &mut Domain,
    env: &Env<Limits>,
    child: &mut Queue<jig_client::Request>,
    out: &mut Queue<Request>,
) {
    while let Some(request) = child.pop() {
        match request {
            jig_client::Request::Told { told: value } => told(root, env, value, out),
            jig_client::Request::Save { records } => {
                for record in records.into_boxed() {
                    out.push(Request::Save { record: Record::Client(record) });
                }
            }
            jig_client::Request::Erase { records } => {
                for key in records.into_boxed() {
                    out.push(Request::Erase { key: Key::Client(key) });
                }
            }
            jig_client::Request::Down { down: jig_client::Down::Save { record } } => {
                out.push(Request::Save { record: Record::Unlinked(record) });
            }
            jig_client::Request::Down { down: jig_client::Down::Erase { key } } => {
                out.push(Request::Erase { key: Key::Unlinked(key) });
            }
            request @ (jig_client::Request::Exchange { .. }
            | jig_client::Request::OpenStream { .. }
            | jig_client::Request::CloseStream
            | jig_client::Request::Down { .. }
            | jig_client::Request::SignOut) => out.push(Request::Client { request }),
        }
    }
}
pub(crate) fn told(root: &mut Domain, env: &Env<Limits>, told: jig_client::Told, out: &mut Queue<Request>) {
    use jig_client::Told;
    let asker = match &told {
        Told::Link { link } => {
            let link = match link {
                jig_client::Link::Live => jig_client_view::link::Link::Live,
                jig_client::Link::Starting => jig_client_view::link::Link::Starting,
                jig_client::Link::Offline | jig_client::Link::SignedOut | jig_client::Link::OutOfDate => {
                    jig_client_view::link::Link::Offline
                }
            };
            first(root, env, first::Event::Link { link }, out);
            root.second.link_is(link);
            jig_client::Asker::JIG
        }
        Told::Dropped { .. } => jig_client::Asker::JIG,
        Told::Kept { key, .. } | Told::Pending { key } | Told::Answered { key, .. } => key.asker,
        Told::Changed { handle } | Told::Text { handle } | Told::Ended { handle, .. } | Told::Page { handle, .. } => {
            handle.asker
        }
    };
    match jig_client_view::Owner::of_asker(asker) {
        Some(jig_client_view::Owner::Jig) => {
            let mut child = Queue::with_capacity(jig_client_view::max_out(&env.limits.jig));
            jig_client_view::bridge::told(&mut root.jig, &jig_env(env), &root.client, told, root.salt, &mut child);
            route_jig(root, env, &mut child, out);
        }
        Some(jig_client_view::Owner::Application { number }) => match crate::Connector::of(number) {
            Some(crate::Connector::First) => crate::family::client::told(root, env, told, out),
            Some(crate::Connector::Second) | None => {}
        },
        None => {}
    }
}
