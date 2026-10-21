//! Placement protects kept personal logins (accounts.md, 3.4; engine.md, section 8).
use crate::Core;
use jig_domain_accounts::{self as accounts, Holder, HostSource};
use jig_domain_fleet::{HostKind, LoginState, PlacementConstraint};
use skein_lib::List;

/// One bounded fact per host, independent of any run or payer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HostRule {
    Open,
    Person(u64),
    Several,
    Unknown,
}
impl HostRule {
    pub(crate) fn allows(self, holder: Holder) -> bool {
        match self {
            Self::Open => true,
            Self::Person(person) => holder == Holder::Party(person),
            Self::Several | Self::Unknown => false,
        }
    }
}
impl Core {
    pub(crate) fn host_fact_capacity(&self) -> u32 {
        let mut capacity = 0_u32;
        for (_, account) in self.accounts.accounts_by_number() {
            capacity = capacity.checked_add(account.sources().len()).expect("bounded live sources");
        }
        capacity
    }

    pub(crate) fn host_rule(&self, placement: PlacementConstraint) -> HostRule {
        if placement.kind == HostKind::Engine || self.people.persons() <= 1 {
            return HostRule::Open;
        }
        let Some(logins) = self.fleet.program_logins(placement.channel) else { return HostRule::Unknown };
        let mut person = None;
        let mut several = false;
        for login in logins {
            if !login.state.kept() {
                continue;
            }
            match accounts::programs::issued(&self.accounts, login.name.as_bytes()) {
                None => return HostRule::Unknown,
                Some((_, Holder::Deployment)) => {}
                Some((_, Holder::Party(holder))) => match person {
                    None => person = Some(holder),
                    Some(first) => several |= first != holder,
                },
            }
        }
        if several {
            HostRule::Several
        } else {
            match person {
                Some(person) => HostRule::Person(person),
                None => HostRule::Open,
            }
        }
    }

    /// At most one fact per live source, including explicit exclusions for direct sources.
    pub(crate) fn host_facts(&self, placement: PlacementConstraint, rule: HostRule, out: &mut List<HostSource>) {
        out.clear();
        if placement.kind == HostKind::Worker
            && let Some(logins) = self.fleet.program_logins(placement.channel)
        {
            for login in logins {
                if let Some((source, holder)) = accounts::programs::issued(&self.accounts, login.name.as_bytes())
                    && self.accounts.source(source).is_some()
                {
                    out.push(HostSource { source, usable: login.state == LoginState::Usable && rule.allows(holder) })
                        .expect("one fact per source");
                }
            }
        }
        for (_, account) in self.accounts.accounts_by_number() {
            if rule.allows(account.holder()) {
                continue;
            }
            for (_, source) in account.sources() {
                let mut present = false;
                for fact in &*out {
                    present |= fact.source == *source;
                }
                if !present {
                    out.push(HostSource { source: *source, usable: false }).expect("one fact per source");
                }
            }
        }
    }
}
