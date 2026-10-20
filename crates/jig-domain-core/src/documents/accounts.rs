//! The party's accounts (`client/domain/contract.md`, 5.1; `domain/accounts.md`, sections 3 and 5).
use super::deployment::{AccountState, DocumentClock, account_state, wall_deadline};
use super::inbox_deployment::line_bound;
use super::{DocumentLimits, Missing, Text, Window};
use crate::{Core, Limits};
use alloc::boxed::Box;
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
    pub place: u32,
    pub uses: Option<List<accounts::Use>>,
    pub plan: bool,
    pub provider: Option<u32>,
    pub provider_name: Option<Text>,
    pub manage_usage: Option<Box<[u8]>>,
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
/// Secret-free facts of the party's accounts, including configured provider terms.
/// Program per-host state and checks await T12. Uses name explicit payers,
/// or leave the default; `usable` follows those uses for this party;
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

#[expect(clippy::manual_map, reason = "step code has no closures (programming-model.md, section 10)")]
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
    let views = core.accounts.account_views();
    let mut after = None;
    for _ in 0..views.len() {
        let Some((place, number)) = next_account(core, &views, accounts::Holder::Party(party), after) else { break };
        after = Some((place, number));
        let account = core.accounts.account(number).expect("a selected account");
        if accounts.room() == 0 {
            cut = true;
            continue;
        }
        let sources = source_lines(core, limits, account, clock);
        accounts
            .push(AccountLine {
                number,
                place: account.place(),
                uses: uses(account),
                plan: account.plan(),
                provider: Some(account.provider()),
                provider_name: provider_name(core, account, limits.documents.text_bytes),
                manage_usage: manage_usage(core, account),
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

/// A usable account serves its explicit party payers, or its default holder/deployment uses.
fn serves_work(core: &Core, party: u64) -> bool {
    for fact in &core.accounts.account_views() {
        let account = core.accounts.account(fact.account).expect("an account view names its account");
        if !fact.usable {
            continue;
        }
        if crate::coverage::serves(&core.people, account, accounts::Use::Party(party)) {
            return true;
        }
    }
    false
}

pub(super) fn worst_case(limits: &DocumentLimits) -> Option<u64> {
    let account_heap = List::<SourceLine>::worst_case(limits.lines)?
        .checked_add(u64::from(limits.text_bytes).checked_mul(2)?)?
        .checked_add(u64::from(accounts::MANAGE_USAGE))?
        .checked_add(List::<accounts::Use>::worst_case(accounts::USES)?)?;
    u64::try_from(size_of::<Accounts>())
        .ok()?
        .checked_add(List::<AccountLine>::worst_case(limits.lines)?)?
        .checked_add(u64::from(limits.lines).checked_mul(account_heap)?)?
        .checked_add(List::<SignInLine>::worst_case(limits.lines)?)
}

/// Copy an account's explicit uses without changing the default/empty distinction.
pub(super) fn uses(account: &accounts::Account) -> Option<List<accounts::Use>> {
    let uses = account.uses()?;
    let mut result = List::with_capacity(accounts::USES);
    for payer in uses {
        result.push(*payer).expect("the child bounds uses");
    }
    Some(result)
}

/// The next holder account by place, with its number breaking ties deterministically.
pub(super) fn next_account(
    core: &Core,
    views: &List<accounts::Fact>,
    holder: accounts::Holder,
    after: Option<(u32, u32)>,
) -> Option<(u32, u32)> {
    let mut next = None;
    for fact in views {
        let account = core.accounts.account(fact.account).expect("an account view names its account");
        let position = (account.place(), fact.account);
        if account.holder() != holder
            || match after {
                Some(old) => position <= old,
                None => false,
            }
        {
            continue;
        }
        if match next {
            Some(old) => position < old,
            None => true,
        } {
            next = Some(position);
        }
    }
    next
}

/// Public configuration is independent of the holder's private account document.
pub(super) fn provider_name(core: &Core, account: &accounts::Account, bytes: u32) -> Option<Text> {
    let provider = core.settings.llm_providers.get(account.provider())?;
    Some(super::small::text(&provider.name, bytes))
}
pub(super) fn manage_usage(core: &Core, account: &accounts::Account) -> Option<Box<[u8]>> {
    core.settings.llm_providers.get(account.provider())?.manage_usage.clone()
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_accounts_document_is_priced() {
        let limits = crate::tests::limits();
        let accounts = super::worst_case(&limits.documents).expect("accounts bound");
        assert!(accounts <= crate::documents::document_worst_case(&limits.documents).expect("document bound"));
    }
}
