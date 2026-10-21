#![expect(clippy::manual_map, reason = "step code translates optional facts without closures")]
//! Accounts watch decoding and shared deployment source facts (contract.md, 5.1).
use super::{Limits, List, Reader, Wall, c, piece_problem, too_large, w};
use alloc::boxed::Box;
pub(crate) fn owner_account_use(value: &w::OwnerAccountUse) -> c::OwnerAccountUse {
    match value {
        w::OwnerAccountUse::Party(row) => c::OwnerAccountUse::Party(row.party()),
        w::OwnerAccountUse::Project(row) => c::OwnerAccountUse::Project(row.project()),
    }
}
pub(crate) fn owner_account_holder(value: &w::OwnerAccountHolder) -> c::OwnerAccountHolder {
    match value {
        w::OwnerAccountHolder::Deployment => c::OwnerAccountHolder::Deployment,
        w::OwnerAccountHolder::Party(row) => c::OwnerAccountHolder::Party(row.party()),
    }
}
pub(crate) fn owner_account_kind(value: &w::OwnerAccountKind) -> c::OwnerAccountKind {
    match value {
        w::OwnerAccountKind::SignIn => c::OwnerAccountKind::SignIn,
        w::OwnerAccountKind::Key => c::OwnerAccountKind::Key,
        w::OwnerAccountKind::Program => c::OwnerAccountKind::Program,
        w::OwnerAccountKind::None => c::OwnerAccountKind::None,
    }
}
pub(crate) fn owner_account_bound(value: &w::OwnerAccountBound) -> c::OwnerAccountBound {
    match value {
        w::OwnerAccountBound::Accounts => c::OwnerAccountBound::Accounts,
        w::OwnerAccountBound::HolderAccounts => c::OwnerAccountBound::HolderAccounts,
        w::OwnerAccountBound::Sources => c::OwnerAccountBound::Sources,
        w::OwnerAccountBound::SignIns => c::OwnerAccountBound::SignIns,
    }
}
pub(crate) fn owner_account_key_failure(value: &w::OwnerAccountKeyFailure) -> c::OwnerAccountKeyFailure {
    match value {
        w::OwnerAccountKeyFailure::TooLong => c::OwnerAccountKeyFailure::TooLong,
        w::OwnerAccountKeyFailure::NotStored => c::OwnerAccountKeyFailure::NotStored,
    }
}
pub(crate) fn owner_account_refusal(value: &w::OwnerAccountRefusal) -> c::OwnerAccountRefusal {
    match value {
        w::OwnerAccountRefusal::NotLimited => c::OwnerAccountRefusal::NotLimited,
        w::OwnerAccountRefusal::Uses => c::OwnerAccountRefusal::Uses,
        w::OwnerAccountRefusal::NotKept(row) => {
            c::OwnerAccountRefusal::NotKept { why: owner_account_key_failure(row.why()) }
        }
        w::OwnerAccountRefusal::Limit(row) => c::OwnerAccountRefusal::Limit { bound: owner_account_bound(row.bound()) },
        w::OwnerAccountRefusal::Held(row) => c::OwnerAccountRefusal::Held { provider: row.provider() },
        w::OwnerAccountRefusal::Provider(row) => c::OwnerAccountRefusal::Provider { provider: row.provider() },
        w::OwnerAccountRefusal::Subject => c::OwnerAccountRefusal::Subject,
        w::OwnerAccountRefusal::Unknown => c::OwnerAccountRefusal::Unknown,
        w::OwnerAccountRefusal::Numbers => c::OwnerAccountRefusal::Numbers,
        w::OwnerAccountRefusal::Kind(row) => c::OwnerAccountRefusal::Kind { kind: owner_account_kind(row.kind()) },
        w::OwnerAccountRefusal::Closing => c::OwnerAccountRefusal::Closing,
        w::OwnerAccountRefusal::NotStarted => c::OwnerAccountRefusal::NotStarted,
    }
}
pub(crate) fn owner_account_failure(value: &w::OwnerAccountFailure) -> c::OwnerAccountFailure {
    match value {
        w::OwnerAccountFailure::Later(row) => c::OwnerAccountFailure::Later { after: *row.after() },
        w::OwnerAccountFailure::SignInAgain => c::OwnerAccountFailure::SignInAgain,
        w::OwnerAccountFailure::MissingScope(row) => c::OwnerAccountFailure::MissingScope { scope: row.scope() },
        w::OwnerAccountFailure::Misconfigured => c::OwnerAccountFailure::Misconfigured,
        w::OwnerAccountFailure::Unloaded(row) => {
            c::OwnerAccountFailure::Unloaded { at: owner_account_place(row.at()), why: owner_account_unread(row.why()) }
        }
        w::OwnerAccountFailure::Cancelled => c::OwnerAccountFailure::Cancelled,
    }
}
pub(crate) fn owner_account_end(value: &w::OwnerAccountEnd) -> c::OwnerAccountEnd {
    match value {
        w::OwnerAccountEnd::Made(row) => c::OwnerAccountEnd::Made { account: row.account(), declined: row.declined() },
        w::OwnerAccountEnd::Renewed(row) => {
            c::OwnerAccountEnd::Renewed { account: row.account(), declined: row.declined() }
        }
        w::OwnerAccountEnd::Refused(row) => {
            c::OwnerAccountEnd::Refused { refusal: owner_account_refusal(row.refusal()) }
        }
        w::OwnerAccountEnd::Failed(row) => c::OwnerAccountEnd::Failed { failure: owner_account_failure(row.failure()) },
    }
}
fn owner_account_place(value: &w::OwnerAccountPlace) -> c::OwnerAccountPlace {
    match value {
        w::OwnerAccountPlace::Directory => c::OwnerAccountPlace::Directory,
        w::OwnerAccountPlace::File => c::OwnerAccountPlace::File,
    }
}
fn owner_account_unread(value: &w::OwnerAccountUnread) -> c::OwnerAccountUnread {
    match value {
        w::OwnerAccountUnread::Unreadable => c::OwnerAccountUnread::Unreadable,
        w::OwnerAccountUnread::Refused => c::OwnerAccountUnread::Refused,
        w::OwnerAccountUnread::Failed => c::OwnerAccountUnread::Failed,
        w::OwnerAccountUnread::Stalled => c::OwnerAccountUnread::Stalled,
        w::OwnerAccountUnread::TooLarge => c::OwnerAccountUnread::TooLarge,
    }
}
pub(crate) fn owner_account_uses(value: Option<&List<w::OwnerAccountUse>>) -> Option<List<c::OwnerAccountUse>> {
    let value = value?;
    let mut uses = List::with_capacity(value.len());
    for item in value {
        uses.push(owner_account_use(item)).expect("supplied uses fit");
    }
    Some(uses)
}
pub(crate) fn owner_account_source(
    value: &w::OwnerAccountSource,
    limits: &Limits,
    room: &mut u64,
) -> Result<c::OwnerAccountSource, w::Problem> {
    let name = match value.name() {
        Some(name) => {
            if !c::valid_program_name(name) {
                return Err(piece_problem(skein_codec::Reason::Malformed));
            }
            charge(room, u64::try_from(name.len()).ok())?;
            Some(skein_lib::bytes::copy_of(name))
        }
        None => None,
    };
    Ok(c::OwnerAccountSource {
        name,
        place: value.place(),
        number: value.number(),
        kind: owner_account_kind(value.kind()),
        state: super::owner_account_state_fields(value.state(), limits),
        usable: value.usable(),
        expiring: value.expiring(),
        attention: value.attention(),
        spent_until: match value.spent_until() {
            Some(at) => Some(Wall::from_nanos(*at)),
            None => None,
        },
    })
}
/// Charges an account's own heap, its uses and its display text, to the document's room.
pub(crate) fn charge_account(
    uses: Option<&List<w::OwnerAccountUse>>,
    display: &w::DocumentText,
    room: &mut u64,
) -> Result<(), w::Problem> {
    let uses = match uses {
        Some(value) => List::<c::OwnerAccountUse>::worst_case(value.len()),
        None => Some(0),
    };
    charge(room, uses)?;
    charge(room, Some(u64::try_from(display.bytes().len()).expect("text fits u64")))
}
pub(crate) fn owner_account_line(
    value: &w::OwnerAccountLine,
    limits: &Limits,
    room: &mut u64,
) -> Result<c::OwnerAccountLine, w::Problem> {
    charge_account(value.uses().as_ref(), value.display(), room)?;
    Ok(c::OwnerAccountLine {
        number: value.number(),
        place: value.place(),
        uses: owner_account_uses(value.uses().as_ref()),
        plan: value.plan(),
        provider: *value.provider(),
        provider_name: match value.provider_name() {
            Some(name) => Some(crate::translate::facts::document_text(name, limits)?),
            None => None,
        },
        manage_usage: checked_manage_usage(value.manage_usage().as_deref(), w::Path::OwnerAccountLineManageUsage)?,
        display: crate::translate::facts::document_text(value.display(), limits)?,
        holder: owner_account_holder(value.holder()),
        sources: owner_account_source_piece(value.sources(), limits, room)?,
        cut: value.cut(),
    })
}
pub(crate) fn owner_account_sign_in(value: &w::OwnerAccountSignIn) -> c::OwnerAccountSignIn {
    c::OwnerAccountSignIn {
        source: value.source(),
        provider: value.provider(),
        holder: owner_account_holder(value.holder()),
    }
}
pub(crate) fn owner_accounts(value: w::OwnerAccounts, limits: &Limits) -> Result<c::OwnerAccounts, w::Problem> {
    let mut room = document_room(limits);
    charge(&mut room, Some(u64::try_from(size_of::<c::OwnerAccounts>()).expect("a record's size fits u64")))?;
    Ok(c::OwnerAccounts {
        readiness: super::catalogue::readiness(value.readiness(), limits, &mut room)?,
        readiness_cut: value.readiness_cut(),
        program_hosts: super::super::programs::hosts(value.program_hosts(), limits, &mut room)?,
        accounts: owner_account_line_piece(value.accounts(), limits, &mut room)?,
        signing: owner_account_sign_in_piece(value.signing(), limits, &mut room)?,
        last_end: match value.last_end() {
            Some(row) => Some(c::OwnerAccountLastEnd { source: row.source(), end: owner_account_end(row.end()) }),
            None => None,
        },
        usable: value.usable(),
        settled: value.settled(),
        cut: value.cut(),
    })
}
/// The bytes a decoded account list may hold, with the record that holds it. The client bounds a
/// document by its bytes and account counts (client/domain/client.md, 14), and its domain prices
/// each document it holds at twice its byte limit (`jig_client_domain::limits`).
pub(crate) fn document_room(limits: &Limits) -> u64 {
    limits.client.document_bytes.saturating_mul(2)
}
/// Takes `bytes` from the document's room; a document they would pass is too large.
pub(crate) fn charge(room: &mut u64, bytes: Option<u64>) -> Result<(), w::Problem> {
    let Some(bytes) = bytes else {
        return Err(too_large());
    };
    match room.checked_sub(bytes) {
        Some(left) => {
            *room = left;
            Ok(())
        }
        None => Err(too_large()),
    }
}
/// The pieces a list holds, each its four length bytes and then its record.
pub(crate) fn piece_count(bytes: &[u8]) -> Result<u32, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut count = 0_u32;
    for _ in 0..=reader.remaining() / 4 {
        if reader.is_empty() {
            return Ok(count);
        }
        let Some(length) = reader.u32() else {
            return Err(piece_problem(skein_codec::Reason::Malformed));
        };
        if reader.skip(length).is_none() {
            return Err(piece_problem(skein_codec::Reason::Malformed));
        }
        count = count.checked_add(1).expect("pieces are fewer than their bytes");
    }
    unreachable!("each piece takes at least its four length bytes")
}
pub(crate) fn owner_account_source_piece(
    bytes: &[u8],
    limits: &Limits,
    room: &mut u64,
) -> Result<List<c::OwnerAccountSource>, w::Problem> {
    let count = piece_count(bytes)?;
    if count > limits.client.accounts.sources {
        return Err(too_large());
    }
    charge(room, List::<c::OwnerAccountSource>::worst_case(count))?;
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let Ok(Some(item)) = skein_codec::piece::split(&mut reader) else { unreachable!("counted pieces split") };
        let row = w::OwnerAccountSource::decode(&limits.wire, &mut Reader::new(&item))?;
        rows.push(owner_account_source(&row, limits, room)?).expect("counted pieces fit");
    }
    Ok(rows)
}
pub(crate) fn owner_account_line_piece(
    bytes: &[u8],
    limits: &Limits,
    room: &mut u64,
) -> Result<List<c::OwnerAccountLine>, w::Problem> {
    let count = piece_count(bytes)?;
    if count > limits.client.accounts.accounts {
        return Err(too_large());
    }
    charge(room, List::<c::OwnerAccountLine>::worst_case(count))?;
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let Ok(Some(item)) = skein_codec::piece::split(&mut reader) else { unreachable!("counted pieces split") };
        let row = w::OwnerAccountLine::decode(&limits.wire, &mut Reader::new(&item))?;
        rows.push(owner_account_line(&row, limits, room)?).expect("counted pieces fit");
    }
    Ok(rows)
}
pub(crate) fn owner_account_sign_in_piece(
    bytes: &[u8],
    limits: &Limits,
    room: &mut u64,
) -> Result<List<c::OwnerAccountSignIn>, w::Problem> {
    let count = piece_count(bytes)?;
    if count > limits.client.accounts.sign_ins {
        return Err(too_large());
    }
    charge(room, List::<c::OwnerAccountSignIn>::worst_case(count))?;
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let Ok(Some(item)) = skein_codec::piece::split(&mut reader) else { unreachable!("counted pieces split") };
        let row = w::OwnerAccountSignIn::decode(&limits.wire, &mut Reader::new(&item))?;
        rows.push(owner_account_sign_in(&row)).expect("counted pieces fit");
    }
    Ok(rows)
}

/// Both account document forms reject targets outside the contract's renderable set.
pub(crate) fn checked_manage_usage(value: Option<&[u8]>, path: w::Path) -> Result<Option<Box<[u8]>>, w::Problem> {
    match value {
        Some(address) => {
            if !c::valid_manage_usage(address) {
                return Err(w::Problem { path, reason: skein_codec::Reason::Malformed });
            }
            Ok(Some(skein_lib::bytes::copy_of(address)))
        }
        None => Ok(None),
    }
}
