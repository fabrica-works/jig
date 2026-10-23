//! Whole program facts, admitted against the document's shared byte budget.
use super::documents::{charge, piece_count};
use super::piece_problem;
use crate::Limits;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v15 as w;
use skein_lib::{List, Reader, bytes};
fn copy(value: &[u8], room: &mut u64) -> Result<Box<[u8]>, w::Problem> {
    charge(room, u64::try_from(value.len()).ok())?;
    Ok(bytes::copy_of(value))
}
fn failure(value: &w::OwnerProgramFailure) -> c::OwnerProgramFailure {
    match value {
        w::OwnerProgramFailure::Missing => c::OwnerProgramFailure::Missing,
        w::OwnerProgramFailure::Deadline => c::OwnerProgramFailure::Deadline,
        w::OwnerProgramFailure::Unreadable => c::OwnerProgramFailure::Unreadable,
        w::OwnerProgramFailure::Exited => c::OwnerProgramFailure::Exited,
        w::OwnerProgramFailure::Unconfigured => c::OwnerProgramFailure::Unconfigured,
        w::OwnerProgramFailure::Unknown => c::OwnerProgramFailure::Unknown,
        w::OwnerProgramFailure::Closed => c::OwnerProgramFailure::Closed,
    }
}
fn state(value: &w::OwnerProgramState, room: &mut u64) -> Result<c::OwnerProgramState, w::Problem> {
    Ok(match value {
        w::OwnerProgramState::Usable => c::OwnerProgramState::Usable,
        w::OwnerProgramState::NotSignedIn => c::OwnerProgramState::NotSignedIn,
        w::OwnerProgramState::SignedInOtherwise(row) => {
            if !c::valid_program_method(row.method()) {
                return Err(piece_problem(skein_codec::Reason::Malformed));
            }
            c::OwnerProgramState::SignedInOtherwise { method: copy(row.method(), room)? }
        }
        w::OwnerProgramState::Failed(row) => {
            c::OwnerProgramState::Failed { reason: failure(row.reason()), detail: copy(row.detail(), room)? }
        }
    })
}
fn login(value: &w::OwnerProgramLogin, room: &mut u64) -> Result<c::OwnerProgramLogin, w::Problem> {
    if !c::valid_program_name(value.name()) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    let state = match value.state() {
        Some(value) => Some(state(value, room)?),
        None => None,
    };
    let sign_out = match value.sign_out() {
        Some(w::OwnerProgramSignOut::SignedOut) => Some(c::OwnerProgramSignOut::SignedOut),
        Some(w::OwnerProgramSignOut::Failed(value)) => Some(c::OwnerProgramSignOut::Failed {
            reason: failure(value.reason()),
            detail: copy(value.detail(), room)?,
        }),
        None => None,
    };
    Ok(c::OwnerProgramLogin {
        name: copy(value.name(), room)?,
        state,
        sign_out,
        cleanup: value.cleanup(),
        cut: value.cut(),
    })
}
pub(crate) fn logins(value: &[u8], limits: &Limits, room: &mut u64) -> Result<List<c::OwnerProgramLogin>, w::Problem> {
    let count = piece_count(value)?;
    charge(room, List::<c::OwnerProgramLogin>::worst_case(count))?;
    let mut reader = Reader::new(value);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let Ok(Some(item)) = skein_codec::piece::split(&mut reader) else {
            return Err(piece_problem(skein_codec::Reason::Malformed));
        };
        let value = w::OwnerProgramLogin::decode(&limits.wire, &mut Reader::new(&item))?;
        rows.push(login(&value, room)?).expect("counted login pieces");
    }
    Ok(rows)
}
#[expect(clippy::disallowed_methods, reason = "protocol validates configured UTF-8 names before allocation")]
pub(crate) fn name(value: &[u8], room: &mut u64) -> Result<Box<[u8]>, w::Problem> {
    if core::str::from_utf8(value).is_err() {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    copy(value, room)
}
pub(crate) fn hosts(value: &[u8], limits: &Limits, room: &mut u64) -> Result<List<c::OwnerProgramHost>, w::Problem> {
    let count = piece_count(value)?;
    charge(room, List::<c::OwnerProgramHost>::worst_case(count))?;
    let mut reader = Reader::new(value);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let Ok(Some(item)) = skein_codec::piece::split(&mut reader) else {
            return Err(piece_problem(skein_codec::Reason::Malformed));
        };
        let row = w::OwnerProgramHost::decode(&limits.wire, &mut Reader::new(&item))?;
        rows.push(c::OwnerProgramHost {
            name: name(row.name(), room)?,
            channel: row.channel(),
            connected: row.connected(),
            awaiting_logins: row.awaiting_logins(),
            programs: logins(row.programs(), limits, room)?,
            cut: row.cut(),
        })
        .expect("counted host pieces");
    }
    Ok(rows)
}
