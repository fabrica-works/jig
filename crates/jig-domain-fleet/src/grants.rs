//! Grant names retained with attempts (domain/engine.md, sections 8 and 12).
//! Values stay in their owners' protocol tables.

use crate::attempt::{Attempt, State, Where};
use crate::domain::Domain;
use crate::{Grant, GrantName, Limits, Request};
use skein_lib::{Duration, Env, List, Queue, Time, Token};

#[derive(Debug)]
pub(crate) struct Kept {
    name: GrantName,
    current: Option<(u64, Time)>,
}

pub(crate) fn keep(
    domain: &mut Domain,
    env: &Env<Limits>,
    run: Token,
    attempt: Token,
    holds: &[GrantName],
    grants: &[Grant],
    out: &mut Queue<Request>,
) {
    let Some(&id) = domain.names.get(&(run, attempt)) else { return };
    let attempt = domain.attempts.get_mut(id).expect("admitted attempt");
    attempt.grants = entries(holds, grants, env.now, env.limits.grants);
    resend(attempt, &domain.channels, env.now, out);
}

fn entries(holds: &[GrantName], grants: &[Grant], now: Time, most: u32) -> List<Kept> {
    let mut kept = List::with_capacity(most);
    for name in holds {
        add(&mut kept, *name);
    }
    for grant in grants {
        let name = GrantName { table: grant.table, account: grant.account };
        if holds.is_empty() {
            add(&mut kept, name);
        }
        for index in 0..kept.len() {
            let entry = kept.get_mut(index).expect("grant index");
            if entry.name == name {
                newer(entry, *grant, now);
            }
        }
    }
    kept
}

fn add(kept: &mut List<Kept>, name: GrantName) {
    for entry in &*kept {
        if entry.name == name {
            return;
        }
    }
    kept.push(Kept { name, current: None }).expect("validated grant count");
}

fn newer(kept: &mut Kept, grant: Grant, now: Time) -> bool {
    if let Some((generation, _)) = kept.current
        && grant.generation <= generation
    {
        return false;
    }
    kept.current = Some((grant.generation, now.saturating_add(grant.valid)));
    true
}

pub(crate) fn renew(domain: &mut Domain, env: &Env<Limits>, grant: Grant, out: &mut Queue<Request>) {
    for (_, &id) in &domain.names {
        let attempt = domain.attempts.get_mut(id).expect("a named attempt is tracked");
        match attempt.state {
            State::Waiting { .. } | State::Adopted { .. } | State::Claimed { .. } => {}
            State::Cancelled { .. }
            | State::Handed { .. }
            | State::Acknowledged { .. }
            | State::Stray { .. }
            | State::Kept { .. }
            | State::Fenced { .. }
            | State::Closed => continue,
        }
        let mut changed = false;
        for index in 0..attempt.grants.len() {
            let kept = attempt.grants.get_mut(index).expect("grant index");
            if kept.name == (GrantName { table: grant.table, account: grant.account }) {
                changed = newer(kept, grant, env.now);
            }
        }
        if changed
            && grant.valid > Duration::ZERO
            && let State::Claimed { at: Where::On(channel), .. } = attempt.state
        {
            let channel = domain.channels.get(channel).expect("a claimed channel is connected").token;
            out.push(Request::Grant { channel, run: attempt.run, attempt: attempt.token, grant });
        }
    }
}

pub(crate) fn resend(
    attempt: &Attempt,
    channels: &skein_lib::Slab<crate::channel::Channel>,
    now: Time,
    out: &mut Queue<Request>,
) {
    let State::Claimed { at: Where::On(channel), .. } = attempt.state else { return };
    let channel = channels.get(channel).expect("a claimed channel is connected").token;
    for kept in &attempt.grants {
        if let Some((generation, lapses)) = kept.current
            && lapses > now
        {
            out.push(Request::Grant {
                channel,
                run: attempt.run,
                attempt: attempt.token,
                grant: Grant {
                    table: kept.name.table,
                    account: kept.name.account,
                    generation,
                    valid: lapses.saturating_since(now),
                },
            });
        }
    }
}
