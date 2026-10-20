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
        accounts::Answer::KeyKept { .. } | accounts::Answer::Done | accounts::Answer::Removed { .. } => {
            unreachable!("account change asks are routed in w1b")
        }
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
        // A provider offering no sign-in, or a configured source that no sign-out serves.
        accounts::Refusal::Provider { .. } | accounts::Refusal::Kind { .. } => people::Refusal::NotOffered,
        accounts::Refusal::Configured | accounts::Refusal::NotLimited | accounts::Refusal::Uses => {
            people::Refusal::NotOffered
        }
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
