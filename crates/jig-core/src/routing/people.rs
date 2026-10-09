//! Authenticated party events routed through the people child
//! (`domain/people.md`, section 5.1).

use super::{Box, Event, Limits, Now, ReplyTo, Request, Requests, tag_people};
use crate::Core;
use skein_lib::{Env, Queue};

use jig_core_people as people;

pub(super) fn route_people(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    event: people::Event,
) -> Requests {
    let mut child = Queue::with_capacity(people::max_out(&env.limits.people));
    people::step(&mut core.people, &Env { now: env.now, wall: env.wall, limits: env.limits.people }, event, &mut child);
    tag_people(core, env, child, work)
}

pub(super) fn sign_in(
    core: &mut Core,
    work: &mut Queue<Event>,
    reply_to: ReplyTo,
    identity: people::Identity,
) -> Requests {
    let mut out = Queue::with_capacity(2);
    if core.counters.deployment().people == u64::MAX || core.counters.deployment().sign_ins == u64::MAX {
        out.push(Request::Now(Box::new(Now::SignInRefused { to: reply_to })));
    } else {
        let person = crate::fresh(&mut core.counters, crate::Family::Person).expect("person counter available");
        let sign_in = crate::fresh(&mut core.counters, crate::Family::SignIn).expect("sign-in counter available");
        core.signing_in = Some(sign_in);
        let kind = if identity.key.provider == core.settings.deployment_provider {
            people::Kind::Service
        } else {
            people::Kind::Person
        };
        work.push(Event::People(people::Event::SignedIn { reply_to, person, sign_in, identity, kind }));
    }
    out.push(Request::Decided);
    Requests::Out(out)
}
