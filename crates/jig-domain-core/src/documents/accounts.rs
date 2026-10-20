//! The party's accounts (`client/domain/contract.md`, 5.1; `domain/accounts.md`, sections 3 and 5).
use super::deployment::{AccountState, DocumentClock, account_state, wall_deadline};
use super::inbox_deployment::line_bound;
use super::{DocumentLimits, Missing, Text, Window};
use crate::{Core, Limits};
use jig_domain_accounts as accounts;
use skein_lib::{List, Wall};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SourceLine {
    pub place: u32,
    pub number: u32,
    pub kind: accounts::Kind,
    pub state: AccountState,
    pub usable: bool,
    pub expiring: bool,
    pub attention: bool,
    pub spent_until: Option<Wall>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AccountLine {
    pub number: u32,
    pub provider: Option<u32>,
    pub display: Text,
    pub holder: accounts::Holder,
    pub sources: List<SourceLine>,
    pub cut: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SignInLine {
    pub source: u32,
    pub provider: u32,
    pub holder: accounts::Holder,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LastEnd {
    pub source: u32,
    pub end: accounts::End,
}
/// Secret-free facts of the party's accounts. The providers' terms facts await
/// configuration's plan/manage-usage facts and provider names (r2 OP1); uses and
/// account order await T8, and program per-host state and checks await T12.
/// `usable` is the staged approximation in `serves_work`, replaced by T8's uses;
/// `settled` currently counts only sign-ins the party asked for.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Accounts {
    pub accounts: List<AccountLine>,
    pub signing: List<SignInLine>,
    pub last_end: Option<LastEnd>,
    pub usable: bool,
    pub settled: bool,
    pub cut: bool,
}

pub(super) fn source_lines(
    core: &Core,
    limits: &Limits,
    account: &accounts::Account,
    clock: Option<DocumentClock>,
) -> List<SourceLine> {
    let mut sources = List::with_capacity(limits.documents.lines);
    for (place, number) in account.sources().iter().take(line_bound(limits)) {
        let fact = core.accounts.source(*number).expect("an account's sources are kept");
        let spent_until = match fact.spent_until {
            Some(at) => wall_deadline(clock, at),
            None => None,
        };
        sources
            .push(SourceLine {
                place: *place,
                number: *number,
                kind: fact.kind,
                state: account_state(fact.state, clock),
                usable: fact.usable,
                expiring: fact.expiring,
                attention: fact.attention,
                spent_until,
            })
            .expect("bounded source lines");
    }
    sources
}

pub(super) fn make(
    core: &Core,
    limits: &Limits,
    party: u64,
    owner: u64,
    window: &Window,
    clock: Option<DocumentClock>,
) -> Result<Accounts, Missing> {
    core.people.identity(party).ok_or(Missing::Unknown)?;
    if super::document_worst_case(&limits.documents).is_none() || window.rows > limits.documents.rows {
        return Err(Missing::TooLarge);
    }
    if owner != party {
        return Err(Missing::NotAllowed);
    }
    let mut accounts = List::with_capacity(limits.documents.lines);
    let mut cut = false;
    for fact in &core.accounts.account_views() {
        let account = core.accounts.account(fact.account).expect("an account view names its account");
        if account.holder() != accounts::Holder::Party(party) {
            continue;
        }
        if accounts.room() == 0 {
            cut = true;
            continue;
        }
        let sources = source_lines(core, limits, account, clock);
        accounts
            .push(AccountLine {
                number: fact.account,
                provider: account.provider(),
                display: super::small::text(account.display(), limits.documents.text_bytes),
                holder: account.holder(),
                cut: sources.len() < account.sources().len(),
                sources,
            })
            .expect("bounded account lines");
    }
    let mut signing = List::with_capacity(limits.documents.lines);
    let mut settled = true;
    for pending in &core.accounts.pending() {
        if pending.by != party {
            continue;
        }
        settled = false;
        if signing.room() == 0 {
            cut = true;
            continue;
        }
        signing
            .push(SignInLine { source: pending.source, provider: pending.provider, holder: pending.holder })
            .expect("bounded sign-in lines");
    }
    let last_end = match core.accounts.last_end(party) {
        Some(ended) => Some(LastEnd { source: ended.source, end: ended.end }),
        None => None,
    };
    Ok(Accounts { accounts, signing, last_end, usable: serves_work(core, party), settled, cut })
}

/// Staged until T8 supplies uses: own accounts and the deployment's serve this party's work.
fn serves_work(core: &Core, party: u64) -> bool {
    for fact in &core.accounts.account_views() {
        let account = core.accounts.account(fact.account).expect("an account view names its account");
        if fact.usable
            && (account.holder() == accounts::Holder::Party(party) || account.holder() == accounts::Holder::Deployment)
        {
            return true;
        }
    }
    false
}

pub(super) fn worst_case(limits: &DocumentLimits) -> Option<u64> {
    let account_heap = List::<SourceLine>::worst_case(limits.lines)?.checked_add(u64::from(limits.text_bytes))?;
    u64::try_from(size_of::<Accounts>())
        .ok()?
        .checked_add(List::<AccountLine>::worst_case(limits.lines)?)?
        .checked_add(u64::from(limits.lines).checked_mul(account_heap)?)?
        .checked_add(List::<SignInLine>::worst_case(limits.lines)?)
}
