//! Ordered hand-out continuations; the routing answers each exactly once.
use jig_domain_host as host;
use skein_lib::Token;
use smith_host_domain as smith;

#[derive(Debug)]
pub(crate) enum Pending {
    Turn { owner: Token },
    Call { owner: Token, call: Token },
    Result { owner: Token, turns: u32, spent: u64, read: Option<Token>, finish: Finish },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Finish {
    Ended,
    Parked,
    Failed(host::RunFailure),
    Refused,
}

impl Finish {
    pub(crate) fn of(result: &smith::RunResult) -> Self {
        match result {
            smith::RunResult::Accepted { .. } => Self::Ended,
            smith::RunResult::Parked => Self::Parked,
            smith::RunResult::Failed { failure } => Self::Failed(crate::translate::failure(*failure)),
            smith::RunResult::Refused { .. } => Self::Refused,
        }
    }
}
