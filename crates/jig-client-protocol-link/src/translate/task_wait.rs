//! Preserve the task's next-run wait and its unknown deadlines.
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v10 as w;

pub(super) fn waiting(value: Option<&w::TaskRunWait>) -> Option<c::TaskRunWait> {
    Some(match value? {
        w::TaskRunWait::Credential => c::TaskRunWait::Credential,
        w::TaskRunWait::Host => c::TaskRunWait::Host,
        w::TaskRunWait::Account(row) => c::TaskRunWait::Account {
            account: *row.account(),
            source: *row.source(),
            why: match row.why() {
                w::TaskWaitWhy::Refreshing => c::TaskWaitWhy::Refreshing,
                w::TaskWaitWhy::Host => c::TaskWaitWhy::Host,
                w::TaskWaitWhy::Empty => c::TaskWaitWhy::Empty,
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
fn wall(value: Option<u64>) -> Option<skein_lib::Wall> {
    match value {
        Some(value) => Some(skein_lib::Wall::from_nanos(value)),
        None => None,
    }
}
