//! Field-for-field document values (client/domain/contract.md, 5–7).
use super::{
    Limits, List, Reader, Wall, c, chip_piece, owner_task_link_fields, part_piece, piece_problem, text, too_large, w,
};
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn host_fields(value: &w::Host, limits: &Limits) -> Result<c::Host, w::Problem> {
    let opt0 = match value.grace() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt1 = match value.refused() {
        Some(item) => Some(text(item, limits)?),
        None => None,
    };
    Ok(c::Host {
        number: value.number(),
        connected: value.connected(),
        grace: opt0,
        used: value.used(),
        free: value.free(),
        runs: chip_piece(value.runs(), limits, limits.client.rows)?,
        refused: opt1,
    })
}

pub(crate) fn host_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::Host>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client.document_bytes {
            return Err(too_large());
        }
        let row = w::Host::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = host_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn account_fields(value: &w::Account, limits: &Limits) -> Result<c::Account, w::Problem> {
    let opt0 = match value.reason() {
        Some(item) => Some(text(item, limits)?),
        None => None,
    };
    Ok(c::Account {
        number: value.number(),
        usable: value.usable(),
        reason: opt0,
        runs: chip_piece(value.runs(), limits, limits.client.rows)?,
        waiting: chip_piece(value.waiting(), limits, limits.client.rows)?,
    })
}

pub(crate) fn account_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::Account>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client.document_bytes {
            return Err(too_large());
        }
        let row = w::Account::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = account_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn capacity_fields(value: &w::Capacity, _limits: &Limits) -> c::Capacity {
    let opt0 = match value.project() {
        Some(item) => Some(*item),
        None => None,
    };
    c::Capacity { project: opt0, live: value.live(), limit: value.limit() }
}

pub(crate) fn refusal_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::Refusal>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client.document_bytes {
            return Err(too_large());
        }
        let row = w::Refusal::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = crate::translate::answers::refusal(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn deployment_fields(value: &w::Deployment, limits: &Limits) -> Result<c::Deployment, w::Problem> {
    if value.capacity().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.capacity().len());
    for item in value.capacity() {
        list0.push(capacity_fields(item, limits)).expect("reserved supplied list");
    }
    Ok(c::Deployment {
        hosts: host_piece(value.hosts(), limits, limits.client.rows)?,
        accounts: account_piece(value.accounts(), limits, limits.client.rows)?,
        capacity: list0,
        proposals: value.proposals(),
        refusals: refusal_piece(value.refusals(), limits, limits.client.rows)?,
        parts: part_piece(value.parts(), limits, limits.client.rows)?,
    })
}

pub(crate) fn store_fill_fields(value: &w::StoreFill, _limits: &Limits) -> c::StoreFill {
    c::StoreFill { bytes: value.bytes(), mark: value.mark() }
}

pub(crate) fn owner_host_kind_fields(value: &w::OwnerHostKind, _limits: &Limits) -> c::OwnerHostKind {
    match value {
        w::OwnerHostKind::Worker => c::OwnerHostKind::Worker,
        w::OwnerHostKind::Engine => c::OwnerHostKind::Engine,
    }
}

pub(crate) fn owner_attempt_state_fields(value: &w::OwnerAttemptState, _limits: &Limits) -> c::OwnerAttemptState {
    match value {
        w::OwnerAttemptState::Waiting => c::OwnerAttemptState::Waiting,
        w::OwnerAttemptState::Adopted => c::OwnerAttemptState::Adopted,
        w::OwnerAttemptState::Claimed => c::OwnerAttemptState::Claimed,
        w::OwnerAttemptState::Cancelled => c::OwnerAttemptState::Cancelled,
        w::OwnerAttemptState::Handed => c::OwnerAttemptState::Handed,
        w::OwnerAttemptState::Acknowledged => c::OwnerAttemptState::Acknowledged,
        w::OwnerAttemptState::Stray => c::OwnerAttemptState::Stray,
        w::OwnerAttemptState::Kept => c::OwnerAttemptState::Kept,
        w::OwnerAttemptState::Fenced => c::OwnerAttemptState::Fenced,
        w::OwnerAttemptState::Closed => c::OwnerAttemptState::Closed,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_run_location_fields(value: &w::OwnerRunLocation, limits: &Limits) -> c::OwnerRunLocation {
    match value {
        w::OwnerRunLocation::None => c::OwnerRunLocation::None,
        w::OwnerRunLocation::Connected(row) => {
            c::OwnerRunLocation::Connected { channel: row.channel(), kind: owner_host_kind_fields(row.kind(), limits) }
        }
        w::OwnerRunLocation::Adrift(row) => {
            let opt0 = match row.channel() {
                Some(item) => Some(*item),
                None => None,
            };
            let opt1 = match row.until() {
                Some(item) => Some(Wall::from_nanos(*item)),
                None => None,
            };
            let opt2 = match row.remaining() {
                Some(item) => Some(*item),
                None => None,
            };
            c::OwnerRunLocation::Adrift { channel: opt0, until: opt1, remaining: opt2 }
        }
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_host_run_fields(value: &w::OwnerHostRun, limits: &Limits) -> Result<c::OwnerHostRun, w::Problem> {
    let opt0 = match value.grace_until() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::OwnerHostRun {
        task: owner_task_link_fields(value.task(), limits)?,
        attempt: value.attempt(),
        kept: value.kept(),
        listed: value.listed(),
        state: owner_attempt_state_fields(value.state(), limits),
        location: owner_run_location_fields(value.location(), limits),
        grace_until: opt0,
    })
}

pub(crate) fn owner_host_run_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::OwnerHostRun>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client.document_bytes {
            return Err(too_large());
        }
        let row = w::OwnerHostRun::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = owner_host_run_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_host_fields(
    value: &w::OwnerHost,
    limits: &Limits,
    room: &mut u64,
) -> Result<c::OwnerHost, w::Problem> {
    let opt0 = match value.grace_until() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt1 = match value.grace_remaining() {
        Some(item) => Some(*item),
        None => None,
    };
    let name = match value.name() {
        Some(name) => Some(crate::translate::programs::name(name, room)?),
        None => None,
    };
    Ok(c::OwnerHost {
        name,
        awaiting_logins: value.awaiting_logins(),
        rule: match value.rule() {
            w::OwnerHostRule::Open => c::HostRule::Open,
            w::OwnerHostRule::Person(person) => c::HostRule::Person(person.party()),
            w::OwnerHostRule::Several => c::HostRule::Several,
            w::OwnerHostRule::Unknown => c::HostRule::Unknown,
        },
        programs: crate::translate::programs::logins(value.programs(), limits, room)?,
        channel: value.channel(),
        kind: owner_host_kind_fields(value.kind(), limits),
        slots: value.slots(),
        used: value.used(),
        free: value.free(),
        draining: value.draining(),
        connected: value.connected(),
        grace_until: opt0,
        grace_remaining: opt1,
        runs: owner_host_run_piece(value.runs(), limits, limits.client.rows)?,
        cut: value.cut(),
    })
}

pub(crate) fn owner_host_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
    room: &mut u64,
) -> Result<List<c::OwnerHost>, w::Problem> {
    let count = super::piece_count(bytes)?;
    if count > bound {
        return Err(too_large());
    }
    super::charge(room, List::<c::OwnerHost>::worst_case(bound))?;
    let mut rows = List::with_capacity(bound);
    let mut reader = Reader::new(bytes);
    for _ in 0..count {
        let Ok(Some(item)) = skein_codec::piece::split(&mut reader) else {
            return Err(piece_problem(skein_codec::Reason::Malformed));
        };
        let row = w::OwnerHost::decode(&limits.wire, &mut Reader::new(&item))?;
        rows.push(owner_host_fields(&row, limits, room)?).expect("counted host pieces");
    }
    Ok(rows)
}

pub(crate) fn owner_account_place_fields(value: &w::OwnerAccountPlace, _limits: &Limits) -> c::OwnerAccountPlace {
    match value {
        w::OwnerAccountPlace::Directory => c::OwnerAccountPlace::Directory,
        w::OwnerAccountPlace::File => c::OwnerAccountPlace::File,
    }
}

pub(crate) fn owner_account_unread_fields(value: &w::OwnerAccountUnread, _limits: &Limits) -> c::OwnerAccountUnread {
    match value {
        w::OwnerAccountUnread::Unreadable => c::OwnerAccountUnread::Unreadable,
        w::OwnerAccountUnread::Refused => c::OwnerAccountUnread::Refused,
        w::OwnerAccountUnread::Failed => c::OwnerAccountUnread::Failed,
        w::OwnerAccountUnread::Stalled => c::OwnerAccountUnread::Stalled,
        w::OwnerAccountUnread::TooLarge => c::OwnerAccountUnread::TooLarge,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_account_state_fields(value: &w::OwnerAccountState, limits: &Limits) -> c::OwnerAccountState {
    match value {
        w::OwnerAccountState::Asking => c::OwnerAccountState::Asking,
        w::OwnerAccountState::Granted => c::OwnerAccountState::Granted,
        w::OwnerAccountState::Later(row) => {
            let opt0 = match row.until() {
                Some(item) => Some(Wall::from_nanos(*item)),
                None => None,
            };
            c::OwnerAccountState::Later { until: opt0 }
        }
        w::OwnerAccountState::SignInAgain => c::OwnerAccountState::SignInAgain,
        w::OwnerAccountState::Misconfigured => c::OwnerAccountState::Misconfigured,
        w::OwnerAccountState::Unloaded(row) => c::OwnerAccountState::Unloaded {
            at: owner_account_place_fields(row.at(), limits),
            why: owner_account_unread_fields(row.why(), limits),
        },
        w::OwnerAccountState::Cancelled => c::OwnerAccountState::Cancelled,
        w::OwnerAccountState::Closing => c::OwnerAccountState::Closing,
    }
}
