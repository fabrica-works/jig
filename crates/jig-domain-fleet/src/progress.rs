//! Best-effort live observations (domain/hosts.md, section 6.5). The current
//! claim and hosting channel fence them; none changes placement or scheduling.

use crate::facts::Fact;
use crate::{Domain, Limits, Request};
use alloc::boxed::Box;
use skein_lib::{Env, Queue, Token, Wall};

/// What the host last reported, independent of assignment/adoption time.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Activity {
    Preparing,
    Starting,
    Active,
    Writing,
    /// Whole opaque call identity and whole declared tool name, never truncated.
    Tool {
        call: Box<[u8]>,
        tool: Box<[u8]>,
    },
    /// Waiting for another event while the attempt still occupies its host slot.
    Waiting,
    Ending,
}

/// A decoded report, translated by the protocol below the core.
#[derive(PartialEq, Eq, Debug)]
pub enum Report {
    /// The agent actually started; receipt time is not the host's start clock.
    Started,
    Activity(Activity),
}

/// A volatile report retained beside its attempt. These are engine receipt
/// times, not durable run timing or proof that a best-effort report is current.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Observation {
    pub channel: Token,
    pub activity: Activity,
    /// First receipt of this unchanged activity on this channel.
    pub since: Wall,
    /// Most recent receipt, even when the activity did not change.
    pub reported_at: Wall,
    /// First actual-start report received on this channel, if any.
    pub started: Option<Wall>,
}

pub(crate) fn received(
    domain: &mut Domain,
    env: &Env<Limits>,
    channel: Token,
    run: Token,
    attempt: Token,
    report: Report,
    out: &mut Queue<Request>,
) {
    let Some(id) = crate::call::owned(domain, channel, run, attempt) else {
        domain.facts.push(Fact::Dropped);
        return;
    };
    let (activity, started) = match report {
        Report::Started => {
            let entry = domain.attempts.get_mut(id).expect("current owned claim");
            if let Some(previous) = &mut entry.progress
                && previous.channel == channel
            {
                if previous.started.is_none() {
                    previous.started = Some(env.wall);
                }
                match previous.activity {
                    Activity::Preparing | Activity::Starting => {
                        previous.activity = Activity::Active;
                        previous.since = env.wall;
                    }
                    Activity::Active
                    | Activity::Writing
                    | Activity::Tool { .. }
                    | Activity::Waiting
                    | Activity::Ending => {}
                }
                previous.reported_at = env.wall;
                out.push(Request::Observed { run, attempt });
                return;
            }
            (Activity::Active, true)
        }
        Report::Activity(activity) => (activity, false),
    };
    let fits = match &activity {
        Activity::Tool { call, tool } => {
            env.limits.progress_bytes > 0
                && u64::try_from(call.len()).expect("length fits") <= env.limits.call_name_bytes
                && u64::try_from(tool.len()).expect("length fits") <= env.limits.progress_bytes
        }
        Activity::Preparing
        | Activity::Starting
        | Activity::Active
        | Activity::Writing
        | Activity::Waiting
        | Activity::Ending => true,
    };
    if !fits {
        domain.facts.push(Fact::Dropped);
        return;
    }
    let entry = domain.attempts.get_mut(id).expect("current owned claim");
    let mut since = env.wall;
    let mut first_start = None;
    if let Some(previous) = &entry.progress
        && previous.channel == channel
    {
        if previous.activity == activity {
            since = previous.since;
        }
        first_start = previous.started;
    }
    if started && first_start.is_none() {
        first_start = Some(env.wall);
    }
    entry.progress = Some(Observation { channel, activity, since, reported_at: env.wall, started: first_start });
    out.push(Request::Observed { run, attempt });
}

pub(crate) fn observed(domain: &Domain, run: Token, attempt: Token) -> Option<&Observation> {
    let id = *domain.names.get(&(run, attempt))?;
    let entry = domain.attempts.get(id)?;
    let observation = entry.progress.as_ref()?;
    crate::call::owned(domain, observation.channel, run, attempt)?;
    Some(observation)
}
