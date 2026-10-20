//! Current people/account coverage (accounts.md, 4.2; providers.md, 2.6 and 3.4).
//! One predicate serves named-account admission, selection and the account document.
use jig_domain_accounts as accounts;
use jig_domain_people as people;
use jig_domain_tasks as tasks;

pub(crate) const fn payer(value: tasks::Payer) -> accounts::Use {
    match value {
        tasks::Payer::Party(party) => accounts::Use::Party(party),
        tasks::Payer::Project(project) => accounts::Use::Project(project),
    }
}

/// Explicit uses replace defaults, including an empty list. Membership is read now, so a
/// project becoming shared cannot keep using a person's plan through an old `SetUses`.
pub(crate) fn serves(people: &people::Domain, account: &accounts::Account, payer: accounts::Use) -> bool {
    let known = match payer {
        accounts::Use::Party(party) => people.kind(party).is_some(),
        accounts::Use::Project(project) => people.project_members(project).is_some(),
    };
    if !known {
        return false;
    }
    let explicit = match account.uses() {
        None => false,
        Some(uses) => {
            let mut named = false;
            for value in uses {
                if *value == payer {
                    named = true;
                }
            }
            if !named {
                return false;
            }
            true
        }
    };
    if !account.plan() {
        return explicit
            || match account.holder() {
                accounts::Holder::Deployment => true,
                accounts::Holder::Party(holder) => payer == accounts::Use::Party(holder),
            };
    }
    let holder = match account.holder() {
        accounts::Holder::Party(holder) => holder,
        accounts::Holder::Deployment => return false,
    };
    if people.kind(holder) != Some(people::Kind::Person) {
        return false;
    }
    match payer {
        accounts::Use::Party(party) => {
            party == holder || (explicit && people.kind(party) == Some(people::Kind::Service))
        }
        accounts::Use::Project(project) => {
            let Some(members) = people.project_members(project) else { return false };
            if members.is_empty() {
                return false;
            }
            for member in members {
                if member.person != holder && !(explicit && people.kind(member.person) == Some(people::Kind::Service)) {
                    return false;
                }
            }
            true
        }
    }
}

impl crate::Core {
    /// Bounded secret-free proofs for the account child. These are transient check data, never
    /// retained as another ledger or trusted after people or uses change.
    #[must_use]
    pub fn account_coverage(&self, value: tasks::Payer) -> skein_lib::List<accounts::Coverage> {
        let mut result = skein_lib::List::with_capacity(self.accounts.accounts());
        for (number, account) in self.accounts.accounts_by_number() {
            result
                .push(accounts::Coverage { account: *number, serves: serves(&self.people, account, payer(value)) })
                .expect("one proof per account");
        }
        result
    }
}
