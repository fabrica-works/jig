//! Task wait facts are already joined and translated to wall time by the core.
use jig_client_protocol_wire::v13 as w;
use jig_domain_core::documents as d;

pub(super) fn waiting(value: Option<d::TaskRunWait>, limits: &w::Limits) -> Result<Option<w::TaskRunWait>, w::Problem> {
    let Some(value) = value else { return Ok(None) };
    Ok(Some(match value {
        d::TaskRunWait::Credential => w::TaskRunWait::Credential,
        d::TaskRunWait::Host => w::TaskRunWait::Host,
        d::TaskRunWait::Refused(value) => w::TaskRunWait::Refused(w::TaskRunWaitRefused::new(
            limits,
            w::TaskRunWaitRefusedParts {
                value: match value {
                    d::TaskWaitRefused::Hello { kind, at, why } => w::TaskWaitRefused::Hello(w::TaskWaitHello::new(
                        limits,
                        w::TaskWaitHelloParts {
                            kind: super::deployment::host_kind(kind),
                            at: at.as_nanos(),
                            why: super::deployment::hello(why, limits)?,
                        },
                    )?),
                    d::TaskWaitRefused::Program(value) => w::TaskWaitRefused::Program(w::TaskWaitRefusedProgram::new(
                        limits,
                        w::TaskWaitRefusedProgramParts {
                            value: match value {
                                d::OpenProgram::Model => w::TaskWaitProgram::Model,
                                d::OpenProgram::Source { account, source } => {
                                    w::TaskWaitProgram::Source(w::TaskWaitProgramSource::new(
                                        limits,
                                        w::TaskWaitProgramSourceParts { account, source },
                                    )?)
                                }
                            },
                        },
                    )?),
                },
            },
        )?),
        d::TaskRunWait::Account { account, source, why } => w::TaskRunWait::Account(w::TaskWaitAccount::new(
            limits,
            w::TaskWaitAccountParts {
                account,
                source,
                why: match why {
                    d::TaskWaitWhy::Refreshing => w::TaskWaitWhy::Refreshing,
                    d::TaskWaitWhy::Host => w::TaskWaitWhy::Host,
                    d::TaskWaitWhy::Empty => w::TaskWaitWhy::Empty,
                    d::TaskWaitWhy::Invalid(why) => w::TaskWaitWhy::Invalid(w::TaskWaitInvalid::new(
                        limits,
                        w::TaskWaitInvalidParts {
                            why: match why {
                                jig_domain_tasks::AccountWhy::Unknown => w::AccountWhy::Unknown,
                                jig_domain_tasks::AccountWhy::Provider => w::AccountWhy::Provider,
                                jig_domain_tasks::AccountWhy::Uses => w::AccountWhy::Uses,
                                jig_domain_tasks::AccountWhy::Limited => w::AccountWhy::Limited,
                            },
                        },
                    )?),
                    d::TaskWaitWhy::Source(state) => w::TaskWaitWhy::Source(w::TaskWaitSource::new(
                        limits,
                        w::TaskWaitSourceParts { state: source_state(state, limits)? },
                    )?),
                },
            },
        )?),
    }))
}
fn source_state(value: d::TaskSourceState, limits: &w::Limits) -> Result<w::TaskSourceState, w::Problem> {
    use jig_domain_accounts::{Place, Unread};
    Ok(match value {
        d::TaskSourceState::Asking => w::TaskSourceState::Asking,
        d::TaskSourceState::Granted => w::TaskSourceState::Granted,
        d::TaskSourceState::Later { until } => w::TaskSourceState::Later(w::TaskSourceStateUntil::new(
            limits,
            w::TaskSourceStateUntilParts { until: wall(until) },
        )?),
        d::TaskSourceState::Limited { until } => w::TaskSourceState::Limited(w::TaskSourceStateUntil::new(
            limits,
            w::TaskSourceStateUntilParts { until: wall(until) },
        )?),
        d::TaskSourceState::Refused => w::TaskSourceState::Refused,
        d::TaskSourceState::NeedsSignIn => w::TaskSourceState::NeedsSignIn,
        d::TaskSourceState::Misconfigured => w::TaskSourceState::Misconfigured,
        d::TaskSourceState::Cancelled => w::TaskSourceState::Cancelled,
        d::TaskSourceState::Closing => w::TaskSourceState::Closing,
        d::TaskSourceState::Unloaded { at, why } => w::TaskSourceState::Unloaded(w::TaskSourceStateUnloaded::new(
            limits,
            w::TaskSourceStateUnloadedParts {
                at: match at {
                    Place::Directory => w::TaskSourcePlace::Directory,
                    Place::File => w::TaskSourcePlace::File,
                },
                why: match why {
                    Unread::Unreadable => w::TaskSourceUnread::Unreadable,
                    Unread::Refused => w::TaskSourceUnread::Refused,
                    Unread::Failed => w::TaskSourceUnread::Failed,
                    Unread::Stalled => w::TaskSourceUnread::Stalled,
                    Unread::TooLarge => w::TaskSourceUnread::TooLarge,
                },
            },
        )?),
    })
}
#[expect(clippy::manual_map, reason = "step code translates optional wall times without closures")]
fn wall(value: Option<skein_lib::Wall>) -> Option<u64> {
    match value {
        Some(value) => Some(value.as_nanos()),
        None => None,
    }
}
