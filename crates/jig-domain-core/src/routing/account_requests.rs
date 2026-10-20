//! A party's account request between people and the accounts child (domain/people.md, 5.1;
//! domain/accounts.md, 3.3 and section 5): people admitted it from its holder, the child decides.

use jig_domain_accounts as accounts;
use jig_domain_people as people;

/// The account's holder: the asking person's own, or the deployment's.
pub(super) const fn holder(holder: people::AccountHolder, person: u64) -> accounts::Holder {
    match holder {
        people::AccountHolder::Own => accounts::Holder::Party(person),
        people::AccountHolder::Deployment => accounts::Holder::Deployment,
    }
}

/// The child's answer as the party's outcome. A sign-in's subject is checked at its end, a fact,
/// so the subject's refusals (another holder's, or not well formed) are never an answer here.
pub(super) fn outcome(answer: accounts::Answer) -> people::Outcome {
    let refusal = match answer {
        accounts::Answer::Started { source, address } => {
            return people::Outcome::AccountSignInStarted { source, address };
        }
        accounts::Answer::SignedOut { confirmed } => return people::Outcome::AccountSignedOut { confirmed },
        accounts::Answer::Refused { refusal } => refusal,
        accounts::Answer::KeyKept { source } => return people::Outcome::AccountKeyKept { source },
        accounts::Answer::Done => return people::Outcome::AccountChanged,
        accounts::Answer::Removed { confirmed } => return people::Outcome::AccountRemoved { confirmed },
    };
    let refusal = match refusal {
        // Account sign-in ended before its visit, not wanted yet, or closing: the key may ask again.
        accounts::Refusal::Closing
        | accounts::Refusal::NotStarted
        | accounts::Refusal::NotKept { why: accounts::KeyFailure::NotStored } => {
            return people::Outcome::Again(people::Again::NotReady);
        }
        accounts::Refusal::Limit { bound } => people::Refusal::Limit { which: limit(bound) },
        accounts::Refusal::Numbers => people::Refusal::Limit { which: people::LimitKind::Arithmetic },
        // A provider or source kind that does not offer the requested change.
        accounts::Refusal::Provider { .. }
        | accounts::Refusal::Kind { .. }
        | accounts::Refusal::NotLimited
        | accounts::Refusal::Uses => people::Refusal::NotOffered,
        accounts::Refusal::NotKept { why: accounts::KeyFailure::TooLong } => {
            people::Refusal::Limit { which: people::LimitKind::Words }
        }
        accounts::Refusal::Unknown => people::Refusal::Unknown,
        accounts::Refusal::Held { .. } | accounts::Refusal::Subject => {
            unreachable!("a subject is refused at its sign-in's end")
        }
    };
    people::Outcome::Refused(refusal)
}

const fn limit(bound: accounts::Bound) -> people::LimitKind {
    match bound {
        accounts::Bound::Accounts => people::LimitKind::Accounts,
        accounts::Bound::HolderAccounts => people::LimitKind::HolderAccounts,
        accounts::Bound::Sources => people::LimitKind::Sources,
        accounts::Bound::SignIns => people::LimitKind::SignIns,
    }
}

/// Convert the holder's account change and check the provider's uses rule. LLM key
/// changes need the carried envelope and are dispatched by `key_routes` instead.
pub(crate) fn change(
    core: &crate::Core,
    holder: accounts::Holder,
    change: people::AccountChange,
) -> Result<accounts::Change, people::Refusal> {
    Ok(match change {
        people::AccountChange::AddKey { .. } | people::AccountChange::ReplaceKey { .. } => {
            return Err(people::Refusal::NotOffered);
        }
        people::AccountChange::RemoveKey { source } => accounts::Change::RemoveKey { source },
        people::AccountChange::Resume { source } => accounts::Change::Resume { source },
        people::AccountChange::Remove { account } => accounts::Change::Remove { account },
        people::AccountChange::MoveAccount { account, place } => accounts::Change::MoveAccount { account, place },
        people::AccountChange::MoveSource { account, source, place } => {
            accounts::Change::MoveSource { account, source, place }
        }
        people::AccountChange::SetUses { account, uses } => {
            let row = core.accounts.account(account).ok_or(people::Refusal::Unknown)?;
            if row.holder() != holder {
                return Err(people::Refusal::Unknown);
            }
            let uses = match uses {
                None => None,
                Some(uses) => {
                    let mut converted =
                        skein_lib::List::with_capacity(u32::try_from(uses.len()).expect("admitted account uses"));
                    for value in &*uses {
                        let value = match *value {
                            people::Use::Party(party) => {
                                payer(core, holder, row.plan(), party)?;
                                accounts::Use::Party(party)
                            }
                            people::Use::Project(project) => {
                                let members = core.people.project_members(project).ok_or(people::Refusal::Unknown)?;
                                for member in members {
                                    payer(core, holder, row.plan(), member.person)?;
                                }
                                accounts::Use::Project(project)
                            }
                        };
                        converted.push(value).expect("one converted account use");
                    }
                    Some(converted.into_boxed())
                }
            };
            accounts::Change::SetUses { account, uses }
        }
    })
}
fn payer(core: &crate::Core, holder: accounts::Holder, plan: bool, party: u64) -> Result<(), people::Refusal> {
    let kind = core.people.kind(party).ok_or(people::Refusal::Unknown)?;
    if plan && holder != accounts::Holder::Party(party) && kind != people::Kind::Service {
        return Err(people::Refusal::Provider);
    }
    Ok(())
}
