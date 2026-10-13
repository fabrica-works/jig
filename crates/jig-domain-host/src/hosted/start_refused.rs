//! A refused start frees its slot without keeping an unstarted agent (hosts.md, 8).
use super::{Hosted, State, conclude, tail};
use crate::{Answer, Domain, Ending, Invalid, Limits, LinkUp, Refusal, Request};
use alloc::boxed::Box;
use core::mem;
use skein_lib::{Env, Id, Queue, Token};

pub(crate) fn start_refused(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    why: Box<[u8]>,
    out: &mut Queue<Request>,
) {
    let id = Id::<Hosted>::from_token(owner);
    let entry = domain.hosted.get(id).expect("a start lives until its terminal");
    match &entry.state {
        // A cancellation already won: keep its established terminal path.
        State::Unwanted { .. } => {
            super::gone(domain, env, owner, why, out);
            return;
        }
        State::Starting { .. } => {}
        State::Preparing { .. }
        | State::Cancelling { .. }
        | State::Active { .. }
        | State::Waiting { .. }
        | State::Stopping { .. }
        | State::Saving { .. }
        | State::Closed => unreachable!("only a pending start can be refused"),
    }
    let entry = domain.hosted.get_mut(id).expect("pending start");
    let State::Starting { workspace, held } = mem::replace(&mut entry.state, State::Closed) else {
        unreachable!("checked the pending start above");
    };
    crate::refusals::ending(held, &mut entry.refusals);
    if let Some(workspace) = workspace {
        out.push(Request::Release { workspace });
    }
    let answer = Answer {
        turns: 0,
        spent: 0,
        read: None,
        refused: entry.refusals.take(),
        ending: Ending::Refused(Refusal::Invalid(Invalid::Start { why: tail(why, &env.limits) })),
    };
    out.push(Request::Up(LinkUp::Answer { run: entry.run, attempt: entry.attempt, answer }));
    conclude(domain, id, out);
}
