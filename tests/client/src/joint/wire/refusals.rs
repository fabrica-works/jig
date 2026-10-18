//! Exact sparse entrance facts (client/domain/contract.md, 4–5).
//! Missing explanations, actors' names and numerical bounds remain absent.
use jig_client_domain::contract as client;
use jig_domain_people as people;

pub fn refusal(value: people::Refusal) -> client::Refusal {
    match value {
        people::Refusal::Role { held } => client::Refusal::Role { held: held.map(super::owner::held_role) },
        people::Refusal::Authority { lacking, proposable } => client::Refusal::Authority {
            lacking: lack(lacking),
            proposable: match proposable {
                Some(people::Accepter::Role(_)) => Some(true),
                Some(people::Accepter::Nobody) => Some(false),
                None => None,
            },
        },
        people::Refusal::Limit { which } => {
            client::Refusal::Limit { which: client::Limit::Admission { which: limit(which) } }
        }
        people::Refusal::Moved { now } => moved(now),
        people::Refusal::Ended { how } => client::Refusal::Ended { how: ending(how) },
        people::Refusal::Taken { by } => client::Refusal::Taken { by },
        people::Refusal::NotWellFormed { .. } => unreachable!("no v8 ask leads to NotWellFormed"),
        people::Refusal::NotWaiting => client::Refusal::NotWaiting,
        people::Refusal::NoFurther => client::Refusal::NoFurther,
        people::Refusal::Unknown => client::Refusal::Unknown,
        people::Refusal::NotOffered | people::Refusal::NoEscalationHolder => client::Refusal::NotOffered,
        people::Refusal::KeyConflict => client::Refusal::KeyConflict,
    }
}

#[must_use]
pub fn lack(value: people::Lack) -> client::Lacking {
    match value {
        people::Lack::Approver => client::Lacking::Approver,
        people::Lack::Budget { needs, left } => client::Lacking::Budget { needed: needs, left },
        people::Lack::Grant { kind } => client::Lacking::GrantKind { kind },
        people::Lack::Funding => client::Lacking::Funding,
        people::Lack::TaskKind { .. } => client::Lacking::TaskKind { executor: None },
        people::Lack::Depth { most } => client::Lacking::Depth { needed: None, allowed: most },
        people::Lack::Deadline => client::Lacking::Deadline { asked: None, latest: None },
        people::Lack::Tasks { left } => client::Lacking::Tasks { needed: None, left },
        people::Lack::Tool => client::Lacking::Tool,
        people::Lack::Scope => client::Lacking::Scope,
        people::Lack::Reference => client::Lacking::Reference,
        people::Lack::ResourceAccess => client::Lacking::ResourceAccess,
        people::Lack::ResourceUnavailable => client::Lacking::ResourceUnavailable,
        people::Lack::Shape => client::Lacking::Shape,
        people::Lack::Again { required } => client::Lacking::Again { required },
        people::Lack::Account => client::Lacking::Account,
        people::Lack::Writer => client::Lacking::Writer,
        people::Lack::Required { connector, kind } => client::Lacking::Required { connector, kind },
        people::Lack::Failed { connector, kind } => client::Lacking::Failed { connector, kind },
        people::Lack::Unguarded { connector, kind } => client::Lacking::Unguarded { connector, kind },
    }
}

#[must_use]
pub fn limit(value: people::LimitKind) -> client::AdmissionLimit {
    match value {
        people::LimitKind::Words => client::AdmissionLimit::Words,
        people::LimitKind::Identity => client::AdmissionLimit::Identity,
        people::LimitKind::Roles => client::AdmissionLimit::Roles,
        people::LimitKind::Arithmetic => client::AdmissionLimit::Arithmetic,
        people::LimitKind::Duplicate => client::AdmissionLimit::Duplicate,
        people::LimitKind::Empty => client::AdmissionLimit::Empty,
        people::LimitKind::Batch => client::AdmissionLimit::Batch,
        people::LimitKind::ProjectLiveTasks | people::LimitKind::Goals => client::AdmissionLimit::ProjectLiveTasks,
        people::LimitKind::LiveTasks => client::AdmissionLimit::LiveTasks,
        people::LimitKind::Tree => client::AdmissionLimit::Tree,
        people::LimitKind::Depth => client::AdmissionLimit::Depth,
        people::LimitKind::Delegates => client::AdmissionLimit::Delegates,
        people::LimitKind::Subscription => client::AdmissionLimit::Subscription,
        people::LimitKind::Dependencies => client::AdmissionLimit::Dependencies,
        people::LimitKind::Cycle => client::AdmissionLimit::Cycle,
        people::LimitKind::Operations => client::AdmissionLimit::Operations,
        people::LimitKind::Grace => client::AdmissionLimit::Grace,
        people::LimitKind::Spec => client::AdmissionLimit::Spec,
        people::LimitKind::Contract => client::AdmissionLimit::Contract,
        people::LimitKind::Authority => client::AdmissionLimit::Authority,
        people::LimitKind::Inputs => client::AdmissionLimit::Inputs,
        people::LimitKind::Attempts => client::AdmissionLimit::Attempts,
        people::LimitKind::LiveDelegates => client::AdmissionLimit::LiveDelegates,
        people::LimitKind::Restore => client::AdmissionLimit::Restore,
        people::LimitKind::Read => client::AdmissionLimit::Read,
        people::LimitKind::Turn => client::AdmissionLimit::Turn,
        people::LimitKind::HoldKind => client::AdmissionLimit::HoldKind,
        people::LimitKind::HoldTaken => client::AdmissionLimit::HoldTaken,
        people::LimitKind::Holds => client::AdmissionLimit::Holds,
        people::LimitKind::InboxFull => client::AdmissionLimit::InboxFull,
        people::LimitKind::Notes => client::AdmissionLimit::Notes,
        people::LimitKind::Revision => client::AdmissionLimit::Revision,
        people::LimitKind::Question => client::AdmissionLimit::Question,
        people::LimitKind::Priority => client::AdmissionLimit::Priority,
        people::LimitKind::Policy => client::AdmissionLimit::Policy,
        people::LimitKind::Resource => client::AdmissionLimit::Resource,
        people::LimitKind::Service => client::AdmissionLimit::Service,
    }
}

#[must_use]
pub fn moved(value: people::Moved) -> client::Refusal {
    let (revision, what) = match value {
        people::Moved::Hold { revision, reason } => {
            (revision, client::Moved::Hold { reason: Some(client::CurrentHold::Reason(hold(reason))) })
        }
        people::Moved::Spec { revision } => (revision, client::Moved::Spec),
        people::Moved::Person { revision } => (revision, client::Moved::PersonTask),
        people::Moved::Note { revision } => (u64::from(revision), client::Moved::Note),
    };
    client::Refusal::Moved { now: client::Revision { value: revision }, what }
}

#[must_use]
pub fn hold(value: people::HoldReason) -> client::HoldCategory {
    match value {
        people::HoldReason::Failures { class } => client::HoldCategory::Failures { class: failure(class) },
        people::HoldReason::Stalled => client::HoldCategory::Stalled,
        people::HoldReason::Dependency { task } => client::HoldCategory::Dependency { task },
        people::HoldReason::Stopped => client::HoldCategory::Stopped,
        people::HoldReason::StoppedBy { party } => client::HoldCategory::StoppedBy { party },
        people::HoldReason::Drift => client::HoldCategory::Drift,
        people::HoldReason::Effects => client::HoldCategory::Effects,
        people::HoldReason::EffectFailed => client::HoldCategory::EffectFailed,
        people::HoldReason::Uncertain { entry } => client::HoldCategory::Uncertain { entry },
        people::HoldReason::Budget => client::HoldCategory::Budget,
        people::HoldReason::Deadline => client::HoldCategory::Deadline,
        people::HoldReason::HoldsWaited => client::HoldCategory::HoldsWaited,
        people::HoldReason::ResourceUnavailable => client::HoldCategory::ResourceUnavailable,
        people::HoldReason::ResourceAccess => client::HoldCategory::ResourceAccess,
        people::HoldReason::PoolLost { pool } => client::HoldCategory::PoolLost { pool },
        people::HoldReason::Procedure => client::HoldCategory::Procedure,
    }
}

fn failure(value: people::FailureClass) -> client::HoldFailure {
    match value {
        people::FailureClass::Transient => client::HoldFailure::Transient,
        people::FailureClass::Permanent => client::HoldFailure::Permanent,
        people::FailureClass::Run => client::HoldFailure::Run,
        people::FailureClass::Agent => client::HoldFailure::Agent,
        people::FailureClass::Lost => client::HoldFailure::Lost,
        people::FailureClass::Invalid => client::HoldFailure::Invalid,
    }
}

#[must_use]
pub fn ending(value: people::Ending) -> client::Ending {
    match value {
        people::Ending::Done => client::Ending::Done,
        people::Ending::Failed => client::Ending::Failed { reason: None },
        people::Ending::Cancelled { by, at } => client::Ending::Cancelled {
            by: match by {
                people::Party::Person(number) => client::Actor::Party(number),
                people::Party::Task(number) => client::Actor::Task(number),
                people::Party::Deployment { project } => client::Actor::ProjectDeployment { project },
            },
            reason: None,
            at: Some(at),
        },
        people::Ending::Decided { by, at } => client::Ending::Decided { by, party: None, how: None, at },
        people::Ending::Deleted => client::Ending::Deleted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::Wall;

    #[test]
    fn sparse_refusals_preserve_absent_facts_and_full_width_supplied_identities() {
        let cases = [
            (people::Refusal::Role { held: None }, client::Refusal::Role { held: None }),
            (
                people::Refusal::Role { held: Some(people::Role::Policy { role: u32::MAX }) },
                client::Refusal::Role { held: Some(client::HeldRole::Policy { role: u32::MAX }) },
            ),
            (people::Refusal::Taken { by: u64::MAX }, client::Refusal::Taken { by: u64::MAX }),
            (people::Refusal::NotWaiting, client::Refusal::NotWaiting),
            (people::Refusal::NoFurther, client::Refusal::NoFurther),
            (people::Refusal::Unknown, client::Refusal::Unknown),
            (people::Refusal::NotOffered, client::Refusal::NotOffered),
            (people::Refusal::KeyConflict, client::Refusal::KeyConflict),
            (people::Refusal::NoEscalationHolder, client::Refusal::NotOffered),
        ];
        for (source, expected) in cases {
            let actual = refusal(source);
            assert_eq!(actual, expected);
            assert!(client::EntranceRefusal::new(actual).is_ok());
        }
        for (proposable, boolean) in
            [(None, None), (Some(people::Accepter::Nobody), Some(false)), (Some(people::Accepter::Role(0)), Some(true))]
        {
            let source = people::Refusal::Authority { lacking: people::Lack::TaskKind { executor: None }, proposable };
            assert_eq!(
                refusal(source),
                client::Refusal::Authority {
                    lacking: client::Lacking::TaskKind { executor: None },
                    proposable: boolean
                }
            );
        }
        assert_eq!(
            lack(people::Lack::Budget { needs: u64::MAX, left: 0 }),
            client::Lacking::Budget { needed: u64::MAX, left: 0 }
        );
        assert_eq!(
            lack(people::Lack::Depth { most: u32::MAX }),
            client::Lacking::Depth { needed: None, allowed: u32::MAX }
        );
        assert_eq!(lack(people::Lack::Deadline), client::Lacking::Deadline { asked: None, latest: None });
        assert_eq!(
            lack(people::Lack::Tasks { left: u32::MAX }),
            client::Lacking::Tasks { needed: None, left: u32::MAX }
        );
        assert_eq!(ending(people::Ending::Failed), client::Ending::Failed { reason: None });
        assert_eq!(
            ending(people::Ending::Decided { by: u64::MAX, at: Wall::EPOCH }),
            client::Ending::Decided { by: u64::MAX, party: None, how: None, at: Wall::EPOCH }
        );
        for (by, actor) in [
            (people::Party::Person(u64::MAX), client::Actor::Party(u64::MAX)),
            (people::Party::Task(u64::MAX), client::Actor::Task(u64::MAX)),
            (people::Party::Deployment { project: u32::MAX }, client::Actor::ProjectDeployment { project: u32::MAX }),
        ] {
            assert_eq!(
                ending(people::Ending::Cancelled { by, at: Wall::EPOCH }),
                client::Ending::Cancelled { by: actor, reason: None, at: Some(Wall::EPOCH) }
            );
        }
    }

    #[test]
    fn independent_moved_revisions_keep_sparse_hold_categories_and_numbered_references() {
        for (source, revision, what) in [
            (people::Moved::Spec { revision: u64::MAX }, u64::MAX, client::Moved::Spec),
            (people::Moved::Person { revision: 17 }, 17, client::Moved::PersonTask),
            (people::Moved::Note { revision: u32::MAX }, u64::from(u32::MAX), client::Moved::Note),
        ] {
            assert_eq!(moved(source), client::Refusal::Moved { now: client::Revision { value: revision }, what });
        }
        for (source, expected) in [
            (people::HoldReason::Stalled, client::HoldCategory::Stalled),
            (people::HoldReason::Dependency { task: u64::MAX }, client::HoldCategory::Dependency { task: u64::MAX }),
            (people::HoldReason::Stopped, client::HoldCategory::Stopped),
            (people::HoldReason::StoppedBy { party: u64::MAX }, client::HoldCategory::StoppedBy { party: u64::MAX }),
            (people::HoldReason::Drift, client::HoldCategory::Drift),
            (people::HoldReason::Effects, client::HoldCategory::Effects),
            (people::HoldReason::EffectFailed, client::HoldCategory::EffectFailed),
            (people::HoldReason::Uncertain { entry: u64::MAX }, client::HoldCategory::Uncertain { entry: u64::MAX }),
            (people::HoldReason::Budget, client::HoldCategory::Budget),
            (people::HoldReason::Deadline, client::HoldCategory::Deadline),
            (people::HoldReason::HoldsWaited, client::HoldCategory::HoldsWaited),
            (people::HoldReason::ResourceUnavailable, client::HoldCategory::ResourceUnavailable),
            (people::HoldReason::ResourceAccess, client::HoldCategory::ResourceAccess),
            (people::HoldReason::PoolLost { pool: u64::MAX }, client::HoldCategory::PoolLost { pool: u64::MAX }),
            (people::HoldReason::Procedure, client::HoldCategory::Procedure),
        ] {
            let actual = moved(people::Moved::Hold { revision: 31, reason: source });
            assert_eq!(
                actual,
                client::Refusal::Moved {
                    now: client::Revision { value: 31 },
                    what: client::Moved::Hold { reason: Some(client::CurrentHold::Reason(expected)) },
                }
            );
            assert!(client::EntranceRefusal::new(actual).is_ok());
        }
        for (source, expected) in [
            (people::FailureClass::Transient, client::HoldFailure::Transient),
            (people::FailureClass::Permanent, client::HoldFailure::Permanent),
            (people::FailureClass::Run, client::HoldFailure::Run),
            (people::FailureClass::Agent, client::HoldFailure::Agent),
            (people::FailureClass::Lost, client::HoldFailure::Lost),
            (people::FailureClass::Invalid, client::HoldFailure::Invalid),
        ] {
            assert_eq!(
                hold(people::HoldReason::Failures { class: source }),
                client::HoldCategory::Failures { class: expected }
            );
        }
    }

    #[test]
    fn every_entrance_admission_category_remains_a_category_without_a_guessed_bound() {
        use client::AdmissionLimit as To;
        use people::LimitKind as From;
        let cases = [
            (From::Words, To::Words),
            (From::Identity, To::Identity),
            (From::Roles, To::Roles),
            (From::Arithmetic, To::Arithmetic),
            (From::Duplicate, To::Duplicate),
            (From::Empty, To::Empty),
            (From::Batch, To::Batch),
            (From::ProjectLiveTasks, To::ProjectLiveTasks),
            (From::LiveTasks, To::LiveTasks),
            (From::Tree, To::Tree),
            (From::Depth, To::Depth),
            (From::Delegates, To::Delegates),
            (From::Subscription, To::Subscription),
            (From::Dependencies, To::Dependencies),
            (From::Cycle, To::Cycle),
            (From::Spec, To::Spec),
            (From::Contract, To::Contract),
            (From::Authority, To::Authority),
            (From::Inputs, To::Inputs),
            (From::Attempts, To::Attempts),
            (From::LiveDelegates, To::LiveDelegates),
            (From::Restore, To::Restore),
            (From::Read, To::Read),
            (From::Turn, To::Turn),
            (From::HoldKind, To::HoldKind),
            (From::HoldTaken, To::HoldTaken),
            (From::Holds, To::Holds),
            (From::InboxFull, To::InboxFull),
            (From::Notes, To::Notes),
            (From::Revision, To::Revision),
            (From::Question, To::Question),
            (From::Priority, To::Priority),
            (From::Policy, To::Policy),
            (From::Resource, To::Resource),
            (From::Service, To::Service),
        ];
        assert_eq!(cases.len(), 35);
        for (source, expected) in cases {
            let actual = refusal(people::Refusal::Limit { which: source });
            assert_eq!(actual, client::Refusal::Limit { which: client::Limit::Admission { which: expected } });
            assert!(client::EntranceRefusal::new(actual).is_ok());
        }
    }
}
