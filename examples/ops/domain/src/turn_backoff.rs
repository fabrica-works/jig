//! The engine hub's turn retry owner (protocol/channel.md, 5.2).
use crate::domain::Work;
use crate::{Domain, Limits, Output, Write};
use jig_domain_host as host;
use skein_lib::{Decision, Duration, Env, Queue, Token};

pub(crate) fn busy(domain: &mut Domain, env: &Env<Limits>, run: Token, attempt: Token, turn: u32) {
    if !domain.host.has_turn(run, attempt, turn) {
        return;
    }
    let span = Duration::from_nanos(domain.turn_rng.between(50_000_000, 100_000_000));
    domain.turn_backoffs.arm((run, attempt, turn), env.now.saturating_add(span)).expect("hub retained turn bound");
}

pub(crate) fn fire(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    work: &mut Queue<Work>,
) {
    let Some((run, attempt, number)) = domain.turn_backoffs.expire(env.now) else { return };
    let Some(turn) = domain.host.turn(run, attempt, number) else { return };
    let mut out = Queue::with_capacity(host::max_out(&env.limits.host));
    out.push(host::Request::Up(host::LinkUp::Turn { run, attempt, turn }));
    crate::route::host_out(domain, env, decision, &mut out, work);
}
