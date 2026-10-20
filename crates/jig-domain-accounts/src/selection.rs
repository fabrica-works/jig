//! Read-only ordered selection (accounts.md, 3.4 and 4.3). The core supplies the people join;
//! this child owns account/source identity and order. No credential or second ledger is held.
use crate::{Account, Domain, Holder, Kind, State, Use};
use skein_lib::Time;

/// Coverage of one account for the payer of this check, recomputed by the core.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Coverage {
    pub account: u32,
    pub serves: bool,
}
/// Source availability on this candidate host, including its holder rule. A program requires
/// positive login evidence; a direct source is available unless explicitly excluded. Only a
/// program's host-local unavailability permits the next source; global holds always win.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HostSource {
    pub source: u32,
    pub usable: bool,
}
/// A limited old account excludes its holder's other accounts of that provider on re-choice.
/// A still-serving preferred account wins before this fresh-choice exclusion.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Excluded {
    pub holder: Holder,
    pub provider: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WaitWhy {
    Source(State),
    Refreshing,
    Host,
    Empty,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Selection {
    Chosen { account: u32, source: u32 },
    Wait { account: u32, source: Option<u32>, why: WaitWhy },
    NoAccount,
}

impl Domain {
    /// Choose for one provider on one candidate host. `preferred` is the kept or checked named
    /// account; when still eligible it wins even while unusable. Other accounts are considered
    /// in payer/explicit/deployment order, preserving each holder's order, with number ties.
    /// Inputs are bounded by this child's account/source capacities. Duplicate evidence fails
    /// closed. The query allocates nothing. Work is bounded by accounts squared plus host facts
    /// squared plus sources times host facts, all within the configured capacities.
    #[must_use]
    #[expect(clippy::too_many_arguments, reason = "one concrete read-only selection check")]
    pub fn select(
        &self,
        payer: Use,
        provider: u32,
        preferred: Option<u32>,
        excluded: Option<Excluded>,
        coverage: &[Coverage],
        host: &[HostSource],
        now: Time,
    ) -> Selection {
        if coverage.len() > usize::try_from(self.accounts.capacity()).expect("account capacity fits")
            || host.len() > usize::try_from(self.sources.capacity()).expect("source capacity fits")
        {
            return Selection::NoAccount;
        }
        for (at, proof) in coverage.iter().enumerate() {
            for earlier in coverage.iter().take(at) {
                if earlier.account == proof.account {
                    return Selection::NoAccount;
                }
            }
        }
        for (at, proof) in host.iter().enumerate() {
            for earlier in host.iter().take(at) {
                if earlier.source == proof.source {
                    return Selection::NoAccount;
                }
            }
        }
        if let Some(number) = preferred
            && let Some(account) = self.accounts.get(&number)
            && eligible(number, account, provider, None, coverage)
        {
            return self.select_source(number, account, host, now);
        }
        let mut after = None;
        let mut waiting = Selection::NoAccount;
        for _ in 0..self.accounts.len() {
            let mut next = None;
            for (number, account) in &self.accounts {
                if !eligible(*number, account, provider, excluded, coverage) {
                    continue;
                }
                let key = (rank(payer, account), account.holder(), account.place(), *number);
                if after >= Some(key) {
                    continue;
                }
                match next {
                    Some((best, _)) if best <= key => {}
                    Some(_) | None => next = Some((key, account)),
                }
            }
            let Some((key, account)) = next else { break };
            after = Some(key);
            let selected = self.select_source(key.3, account, host, now);
            match selected {
                Selection::Chosen { .. } => return selected,
                Selection::Wait { .. } => {
                    if waiting == Selection::NoAccount {
                        waiting = selected;
                    }
                }
                Selection::NoAccount => {}
            }
        }
        waiting
    }

    fn select_source(&self, number: u32, account: &Account, host: &[HostSource], now: Time) -> Selection {
        let mut first_local = None;
        for (_, source) in account.sources() {
            let fact = self.source(*source).expect("account's source exists");
            let globally_held = match fact.state {
                State::Limited { .. }
                | State::Refused
                | State::NeedsSignIn
                | State::Misconfigured
                | State::Unloaded { .. }
                | State::Cancelled
                | State::Closing => true,
                State::Asking | State::Granted | State::Later { .. } => false,
            };
            if globally_held {
                return Selection::Wait { account: number, source: Some(*source), why: WaitWhy::Source(fact.state) };
            }
            let on_host = host_allows(host, *source, fact.kind);
            let usable = match fact.kind {
                Kind::None => fact.usable,
                Kind::SignIn | Kind::Key => self.source_grant(*source, now).is_some(),
                Kind::Program => on_host,
            };
            if usable {
                if !on_host {
                    return Selection::Wait { account: number, source: Some(*source), why: WaitWhy::Host };
                }
                return Selection::Chosen { account: number, source: *source };
            }
            match fact.kind {
                Kind::Program => {
                    if first_local.is_none() {
                        first_local = Some(*source);
                    }
                }
                Kind::SignIn | Kind::Key | Kind::None => {
                    return Selection::Wait { account: number, source: Some(*source), why: WaitWhy::Refreshing };
                }
            }
        }
        Selection::Wait {
            account: number,
            source: first_local,
            why: match first_local {
                Some(_) => WaitWhy::Host,
                None => WaitWhy::Empty,
            },
        }
    }
}

fn host_allows(host: &[HostSource], source: u32, kind: Kind) -> bool {
    let mut allowed = kind != Kind::Program;
    for proof in host {
        if proof.source == source {
            allowed = proof.usable;
        }
    }
    allowed
}

fn eligible(number: u32, account: &Account, provider: u32, excluded: Option<Excluded>, coverage: &[Coverage]) -> bool {
    if account.provider() != Some(provider) {
        return false;
    }
    if let Some(excluded) = excluded
        && excluded.holder == account.holder()
        && excluded.provider == provider
    {
        return false;
    }
    for proof in coverage {
        if proof.account == number {
            return proof.serves;
        }
    }
    false
}
fn rank(payer: Use, account: &Account) -> u8 {
    let own = match payer {
        Use::Party(party) => account.holder() == Holder::Party(party),
        Use::Project(_) => account.plan() && account.uses().is_none() && account.holder() != Holder::Deployment,
    };
    if own {
        return 0;
    }
    match account.uses() {
        Some(_) => 1,
        None => 2,
    }
}
