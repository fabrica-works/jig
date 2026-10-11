//! Exact offered acts, holder identities and supported draft bounds (contract.md, 5.3).
use super::{owner, refusals};
use jig_client::{Limits, contract as client};
use jig_core::acts;
use skein_lib::List;

#[must_use]
pub fn act(value: acts::Act) -> client::ActKind {
    match value {
        acts::Act::Write => client::ActKind::Write,
        acts::Act::Answer => client::ActKind::Answer,
        acts::Act::Take => client::ActKind::Take,
        acts::Act::HandBack => client::ActKind::HandBack,
        acts::Act::Accept => client::ActKind::Accept,
        acts::Act::Reject => client::ActKind::Reject,
        acts::Act::PassUp => client::ActKind::PassUp,
        acts::Act::Release => client::ActKind::Release,
        acts::Act::LeaveHeld => client::ActKind::LeaveHeld,
        acts::Act::Amend => client::ActKind::Amend,
        acts::Act::AmendRelease => client::ActKind::AmendRelease,
        acts::Act::Stop => client::ActKind::Stop,
        acts::Act::Cancel => client::ActKind::Cancel,
        acts::Act::Close => client::ActKind::Close,
        acts::Act::TakeOver => client::ActKind::TakeOver,
        acts::Act::SetGoal => client::ActKind::SetGoal,
        acts::Act::Prioritise => client::ActKind::Prioritise,
        acts::Act::MarkSeen => client::ActKind::MarkSeen,
        acts::Act::EditNote => client::ActKind::EditNote,
        acts::Act::DeleteNote => client::ActKind::DeleteNote,
        acts::Act::Adopt => client::ActKind::Adopt,
        acts::Act::ChangePolicy => client::ActKind::ChangePolicy,
        acts::Act::SetRoles => client::ActKind::SetRoles,
        acts::Act::MakeService => client::ActKind::MakeService,
        acts::Act::SetPool => client::ActKind::SetPool,
    }
}

/// # Errors
/// Refuses overflowing or pending owner facts and any whole offer allocation above bounds.
pub fn recipient(value: &acts::Recipient, limits: &Limits) -> Result<client::Recipient, owner::Invalid> {
    Ok(match value {
        acts::Recipient::Task(number) => client::Recipient::Task(*number),
        acts::Recipient::Party(number) => client::Recipient::Party(*number),
        acts::Recipient::Role { project, role } => client::Recipient::Role { project: *project, role: *role },
        acts::Recipient::Roles { project, roles } => {
            client::Recipient::Roles { project: *project, roles: owner::list(roles, limits, |number| Ok(*number))? }
        }
    })
}

/// # Errors
/// Refuses overflowing or pending owner facts and any whole offer allocation above bounds.
pub fn bounds(value: &acts::Bounds, limits: &Limits) -> Result<client::Bounds, owner::Invalid> {
    Ok(match value {
        acts::Bounds::Goal { pool_left, charters, procedures, latest, budget, maximum_budget, maximum_deadline } => {
            client::Bounds::Goal {
                pool_left: *pool_left,
                charters: owner::list(charters, limits, |number| Ok(*number))?,
                procedures: owner::list(procedures, limits, |value| {
                    Ok(client::GoalProcedure { connector: value.connector, code: value.code })
                })?,
                latest: *latest,
                budget: *budget,
                maximum_budget: *maximum_budget,
                maximum_deadline: *maximum_deadline,
            }
        }
        acts::Bounds::Amend { budget_left, grants, implications, authority, ceiling } => client::Bounds::Amend {
            budget_left: *budget_left,
            grants: owner::list(grants, limits, |grant| {
                Ok(client::OwnerGrant {
                    connector: grant.connector,
                    kind: grant.kind,
                    pattern: owner::pattern(&grant.segments, &grant.last, limits)?,
                })
            })?,
            implications: owner::list(implications, limits, |pair| {
                Ok(client::OwnerKindImplication { connector: pair.connector, kind: pair.kind, implies: pair.implies })
            })?,
            authority: Box::new(owner::authority(authority, limits)?),
            ceiling: Box::new(owner::authority(ceiling, limits)?),
        },
    })
}

/// # Errors
/// Refuses overflowing or pending owner facts and any whole offer allocation above bounds.
pub fn admission(value: &acts::Admission, limits: &Limits) -> Result<client::Admission, owner::Invalid> {
    Ok(match value {
        acts::Admission::Allowed => client::Admission::Allowed,
        acts::Admission::Proposed { to } => client::Admission::Proposed { to: recipient(to, limits)? },
        acts::Admission::Refused(value) => client::Admission::Refused(refusals::refusal(*value)),
        acts::Admission::Bounded { within, beyond } => client::Admission::Bounded {
            within: bounds(within, limits)?,
            beyond: match beyond {
                acts::Beyond::Proposed { to } => client::Beyond::Proposed { to: recipient(to, limits)? },
                acts::Beyond::Refused(value) => client::Beyond::Refused(refusals::refusal(*value)),
            },
        },
    })
}

/// # Errors
/// Refuses overflowing or pending owner facts and any whole offer allocation above bounds.
pub fn offered(value: &List<acts::Offered>, limits: &Limits) -> Result<List<client::Offered>, owner::Invalid> {
    if value.capacity() > client::MAX_OFFERED_ACTS {
        return Err(owner::Invalid::OfferCapacity { allocated: value.capacity(), maximum: client::MAX_OFFERED_ACTS });
    }
    let mut result = List::with_capacity(value.capacity());
    for value in value {
        result
            .push(client::Offered { act: act(value.act), admission: admission(&value.admission, limits)? })
            .map_err(|_| owner::Invalid::TooLarge)?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_core_people as people;
    use skein_lib::Wall;

    #[test]
    fn goal_offer_keeps_whole_closed_executor_choices_and_independent_direct_hard_and_pool_bounds() {
        let limits = Limits { rows: 4, ..crate::test::limits().jig };
        let mut charters = List::with_capacity(4);
        charters.push(u32::MAX).expect("supplied charter");
        let mut procedures = List::with_capacity(4);
        procedures.push(acts::GoalProcedure { connector: u16::MAX, code: u32::MAX }).expect("supplied procedure");
        let mut roles = List::with_capacity(4);
        roles.push(0).expect("supplied zero role");
        roles.push(u32::MAX).expect("supplied full role");
        let source = acts::Admission::Bounded {
            within: acts::Bounds::Goal {
                pool_left: 3,
                charters,
                procedures,
                latest: None,
                budget: 5,
                maximum_budget: u64::MAX,
                maximum_deadline: Some(Wall::EPOCH),
            },
            beyond: acts::Beyond::Proposed { to: acts::Recipient::Roles { project: u32::MAX, roles } },
        };
        let before = source.clone();
        let copied = admission(&source, &limits).expect("whole bounded offer");
        let client::Admission::Bounded {
            within:
                client::Bounds::Goal { pool_left, charters, procedures, latest, budget, maximum_budget, maximum_deadline },
            beyond: client::Beyond::Proposed { to: client::Recipient::Roles { project, roles } },
        } = copied
        else {
            panic!("actual bounded goal and collective holder")
        };
        assert_eq!((pool_left, budget, maximum_budget), (3, 5, u64::MAX));
        assert_eq!((latest, maximum_deadline), (None, Some(Wall::EPOCH)));
        assert_eq!(charters.capacity(), 4);
        assert_eq!(charters.as_slice(), &[u32::MAX]);
        assert_eq!(procedures.capacity(), 4);
        assert_eq!(procedures.as_slice(), &[client::GoalProcedure { connector: u16::MAX, code: u32::MAX }]);
        assert_eq!(project, u32::MAX);
        assert_eq!(roles.capacity(), 4);
        assert_eq!(roles.as_slice(), &[0, u32::MAX]);
        assert_eq!(admission(&source, &Limits { rows: 3, ..limits }), Err(owner::Invalid::TooLarge));
        assert_eq!(source, before);
    }

    #[test]
    fn offers_keep_allowed_proposed_and_exact_sparse_refused_judgments_without_holder_or_reason_inference() {
        let limits = crate::test::limits().jig;
        for (source, expected) in [
            (acts::Recipient::Task(u64::MAX), client::Recipient::Task(u64::MAX)),
            (acts::Recipient::Party(0), client::Recipient::Party(0)),
            (
                acts::Recipient::Role { project: u32::MAX, role: u32::MAX },
                client::Recipient::Role { project: u32::MAX, role: u32::MAX },
            ),
        ] {
            assert_eq!(
                admission(&acts::Admission::Proposed { to: source }, &limits),
                Ok(client::Admission::Proposed { to: expected })
            );
        }
        assert_eq!(admission(&acts::Admission::Allowed, &limits), Ok(client::Admission::Allowed));
        assert_eq!(
            admission(&acts::Admission::Refused(people::Refusal::Role { held: None }), &limits),
            Ok(client::Admission::Refused(client::Refusal::Role { held: None }))
        );
        assert_eq!(
            admission(&acts::Admission::Refused(people::Refusal::KeyConflict), &limits),
            Ok(client::Admission::Refused(client::Refusal::KeyConflict))
        );
    }
    #[test]
    fn amendment_offer_preserves_reserved_relation_storage_and_independent_whole_authority_loss_before_receiver_admission()
     {
        use jig_core::documents;
        use jig_core_tasks as tasks;
        let authority = documents::AuthorityLine {
            tools: tasks::Tools(u64::MAX),
            grants: List::with_capacity(3),
            executors: List::with_capacity(3),
            tasks: u32::MAX,
            depth: u32::MAX,
            ceiling: tasks::Budget { spend: u64::MAX, deadline: None },
            notes: tasks::Scopes(u8::MAX),
            note_resources: List::with_capacity(3),
            cut: true,
        };
        let mut ceiling = authority.clone();
        ceiling.cut = false;
        ceiling.ceiling.deadline = Some(Wall::EPOCH);
        let source = acts::Bounds::Amend {
            budget_left: u64::MAX,
            grants: List::with_capacity(3),
            implications: List::with_capacity(4),
            authority: Box::new(authority),
            ceiling: Box::new(ceiling),
        };
        let before = source.clone();
        let limits = Limits { rows: 4, ..crate::test::limits().jig };
        let actual = bounds(&source, &limits).expect("whole supplied bounded amendment");
        let client::Bounds::Amend { budget_left, grants, implications, authority, ceiling } = actual else {
            panic!("actual amendment bound")
        };
        assert_eq!(budget_left, u64::MAX);
        assert_eq!(grants.capacity(), 3);
        assert_eq!(implications.capacity(), 4);
        assert!(implications.is_empty(), "reserved empty complete relation is retained, not synthesized");
        assert!(authority.cut);
        assert!(!ceiling.cut);
        assert_eq!(authority.authority.tools, u64::MAX);
        assert_eq!(authority.authority.notes, u8::MAX);
        assert_eq!(authority.authority.deadline, None);
        assert_eq!(ceiling.authority.deadline, Some(Wall::EPOCH));
        assert_eq!(authority.authority.grants.capacity(), 3);
        assert_eq!(authority.authority.delegation.kinds.capacity(), 3);
        assert_eq!(authority.authority.note_resources.capacity(), 3);
        assert_eq!(bounds(&source, &Limits { rows: 3, ..limits }), Err(owner::Invalid::TooLarge));
        assert_eq!(source, before);
    }
}
