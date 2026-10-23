//! Preserve the task's next-run wait and its unknown deadlines.
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v16 as w;

pub(super) fn waiting(value: Option<&w::TaskRunWait>, limits: &crate::Limits) -> Option<c::TaskRunWait> {
    Some(match value? {
        w::TaskRunWait::Credential => c::TaskRunWait::Credential,
        w::TaskRunWait::Host => c::TaskRunWait::Host,
        w::TaskRunWait::Refused(value) => c::TaskRunWait::Refused(match value.value() {
            w::TaskWaitRefused::Hello(row) => c::TaskWaitRefused::Hello {
                kind: super::documents::owner_host_kind_fields(row.kind(), limits),
                at: skein_lib::Wall::from_nanos(row.at()),
                why: super::documents::owner_hello_refusal_fields(row.why(), limits),
            },
            w::TaskWaitRefused::Program(value) => c::TaskWaitRefused::Program(match value.value() {
                w::TaskWaitProgram::Model => c::OpenProgram::Model,
                w::TaskWaitProgram::Source(row) => {
                    c::OpenProgram::Source { account: row.account(), source: row.source() }
                }
            }),
        }),
        w::TaskRunWait::Account(row) => c::TaskRunWait::Account {
            account: *row.account(),
            source: *row.source(),
            why: match row.why() {
                w::TaskWaitWhy::Refreshing => c::TaskWaitWhy::Refreshing,
                w::TaskWaitWhy::Host => c::TaskWaitWhy::Host,
                w::TaskWaitWhy::Empty => c::TaskWaitWhy::Empty,
                w::TaskWaitWhy::Invalid(row) => c::TaskWaitWhy::Invalid(match row.why() {
                    w::AccountWhy::Unknown => c::AccountWhy::Unknown,
                    w::AccountWhy::Provider => c::AccountWhy::Provider,
                    w::AccountWhy::Uses => c::AccountWhy::Uses,
                    w::AccountWhy::Limited => c::AccountWhy::Limited,
                }),
                w::TaskWaitWhy::Source(row) => c::TaskWaitWhy::Source(source_state(row.state())),
            },
        },
    })
}
fn source_state(value: &w::TaskSourceState) -> c::TaskSourceState {
    match value {
        w::TaskSourceState::Asking => c::TaskSourceState::Asking,
        w::TaskSourceState::Granted => c::TaskSourceState::Granted,
        w::TaskSourceState::Later(row) => c::TaskSourceState::Later { until: wall(*row.until()) },
        w::TaskSourceState::Limited(row) => c::TaskSourceState::Limited { until: wall(*row.until()) },
        w::TaskSourceState::Refused => c::TaskSourceState::Refused,
        w::TaskSourceState::NeedsSignIn => c::TaskSourceState::NeedsSignIn,
        w::TaskSourceState::Misconfigured => c::TaskSourceState::Misconfigured,
        w::TaskSourceState::Cancelled => c::TaskSourceState::Cancelled,
        w::TaskSourceState::Closing => c::TaskSourceState::Closing,
        w::TaskSourceState::Unloaded(row) => c::TaskSourceState::Unloaded {
            at: match row.at() {
                w::TaskSourcePlace::Directory => c::OwnerAccountPlace::Directory,
                w::TaskSourcePlace::File => c::OwnerAccountPlace::File,
            },
            why: match row.why() {
                w::TaskSourceUnread::Unreadable => c::OwnerAccountUnread::Unreadable,
                w::TaskSourceUnread::Refused => c::OwnerAccountUnread::Refused,
                w::TaskSourceUnread::Failed => c::OwnerAccountUnread::Failed,
                w::TaskSourceUnread::Stalled => c::OwnerAccountUnread::Stalled,
                w::TaskSourceUnread::TooLarge => c::OwnerAccountUnread::TooLarge,
            },
        },
    }
}
#[expect(clippy::manual_map, reason = "step code translates optional wall times without function pointers")]
fn wall(value: Option<u64>) -> Option<skein_lib::Wall> {
    match value {
        Some(value) => Some(skein_lib::Wall::from_nanos(value)),
        None => None,
    }
}
