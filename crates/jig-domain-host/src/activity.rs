//! Content-free best-effort activity (domain/hosts.md, section 6.5).
use crate::{Domain, LinkUp, Phase, Request};
use skein_lib::{Duration, Queue, Token};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Operation {
    Checks,
    Completion,
    Command,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CountKind {
    Admitted,
    Ended,
    LlmStarted,
    LlmRetried,
    LlmFinished,
    ToolStarted,
    ToolFinished,
    CheckStarted,
    CheckFinished,
    TextArrived,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Activity {
    Phase(Phase),
    Long { operation: Operation, span: Duration },
    LongDone,
    Count { kind: CountKind, elapsed: Duration, count: u64 },
}

pub(crate) fn reported(domain: &mut Domain, owner: Token, activity: Activity) {
    if let Some(hosting) = live(domain, owner) {
        domain.facts.activity(LinkUp::Activity { run: hosting.run, attempt: hosting.attempt, activity });
    }
}

pub(crate) fn live(domain: &Domain, owner: Token) -> Option<crate::Hosting> {
    let hosting = domain.hosting(owner)?;
    match hosting.phase {
        Phase::Active | Phase::Waiting => Some(hosting),
        Phase::Preparing | Phase::Starting | Phase::Ending | Phase::Answered => None,
    }
}

pub(crate) fn rejected(domain: &Domain, owner: Token, account: u32, generation: u64, out: &mut Queue<Request>) {
    if let Some(hosting) = live(domain, owner) {
        out.push(Request::Up(LinkUp::Rejected { run: hosting.run, attempt: hosting.attempt, account, generation }));
    }
}

pub(crate) fn exhausted(domain: &Domain, owner: Token, account: u32, retry_after: Duration, out: &mut Queue<Request>) {
    if let Some(hosting) = live(domain, owner) {
        out.push(Request::Up(LinkUp::Exhausted { run: hosting.run, attempt: hosting.attempt, account, retry_after }));
    }
}
