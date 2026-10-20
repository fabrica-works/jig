#![expect(clippy::manual_map, reason = "step code translates optional facts without closures")]
//! Accounts watch decoding and shared deployment source facts (contract.md, 5.1).
use super::{Limits, List, Reader, Wall, c, piece_problem, too_large, w};
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
        w::OwnerAccountRefusal::Configured => c::OwnerAccountRefusal::Configured,
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
pub(crate) fn owner_account_source(value: &w::OwnerAccountSource, limits: &Limits) -> c::OwnerAccountSource {
    c::OwnerAccountSource {
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
    }
}
pub(crate) fn owner_account_line(
    value: &w::OwnerAccountLine,
    limits: &Limits,
) -> Result<c::OwnerAccountLine, w::Problem> {
    Ok(c::OwnerAccountLine {
        number: value.number(),
        place: value.place(),
        uses: owner_account_uses(value.uses().as_ref()),
        plan: value.plan(),
        provider: *value.provider(),
        display: crate::translate::facts::document_text(value.display(), limits)?,
        holder: owner_account_holder(value.holder()),
        sources: owner_account_source_piece(value.sources(), limits)?,
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
    Ok(c::OwnerAccounts {
        accounts: owner_account_line_piece(value.accounts(), limits)?,
        signing: owner_account_sign_in_piece(value.signing(), limits)?,
        last_end: match value.last_end() {
            Some(row) => Some(c::OwnerAccountLastEnd { source: row.source(), end: owner_account_end(row.end()) }),
            None => None,
        },
        usable: value.usable(),
        settled: value.settled(),
        cut: value.cut(),
    })
}
pub(crate) fn owner_account_source_piece(
    bytes: &[u8],
    limits: &Limits,
) -> Result<List<c::OwnerAccountSource>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(limits.client.rows);
    let mut sum = 0_u64;
    for _ in 0..=limits.client.rows {
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
        let row = w::OwnerAccountSource::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = owner_account_source(&row, limits);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}
pub(crate) fn owner_account_line_piece(bytes: &[u8], limits: &Limits) -> Result<List<c::OwnerAccountLine>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(limits.client.rows);
    let mut sum = 0_u64;
    for _ in 0..=limits.client.rows {
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
        let row = w::OwnerAccountLine::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = owner_account_line(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}
pub(crate) fn owner_account_sign_in_piece(
    bytes: &[u8],
    limits: &Limits,
) -> Result<List<c::OwnerAccountSignIn>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(limits.client.rows);
    let mut sum = 0_u64;
    for _ in 0..=limits.client.rows {
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
        let row = w::OwnerAccountSignIn::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = owner_account_sign_in(&row);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}
