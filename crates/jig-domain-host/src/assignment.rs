//! Admission bounds for a hosted run (domain/hosts.md, section 6.2).
//! Workspace items belong to the application and stay opaque here.

use crate::boundary::{Invalid, RunAssignment};
use crate::limits::Limits;

/// Whether `assignment` fits `limits`, and what about it does not.
pub(crate) fn check(assignment: &RunAssignment, limits: &Limits) -> Result<(), Invalid> {
    if len(&assignment.charter) > limits.charter_bytes {
        return Err(Invalid::Charter);
    }
    if u64::try_from(assignment.grants.len()).expect("a length fits") > u64::from(limits.accounts) {
        return Err(Invalid::Grants);
    }
    for (index, grant) in assignment.grants.iter().enumerate() {
        for other in assignment.grants.iter().skip(index.saturating_add(1)) {
            if grant.table == other.table && grant.account == other.account {
                return Err(Invalid::Grants);
            }
        }
    }
    Ok(())
}

/// The length of `bytes`.
pub(crate) fn len(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).expect("a length fits in a u64")
}

/// A start carries at most one inbox of individually bounded messages.
pub(crate) fn messages_fit(messages: &[crate::StartMessage], limits: &Limits) -> bool {
    if u64::try_from(messages.len()).expect("a length fits") > u64::from(limits.inbox) {
        return false;
    }
    for message in messages {
        match len(&message.sender).checked_add(len(&message.words)) {
            Some(bytes) if bytes <= limits.event_bytes => {}
            Some(_) | None => return false,
        }
    }
    true
}
