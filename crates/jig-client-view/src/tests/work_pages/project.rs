use super::*;
pub(crate) fn project_doc() -> Project {
    Project {
        period: 3,
        spend_lines: list([TaskSpend { chip: chip(8), spent: 5 }]),
        number: 4,
        name: text(b"Workshop"),
        policy: Policy {
            ceiling: super::super::cards_objects::authority(),
            period: 3,
            spend: 20,
            roles: list([RolePolicy {
                number: 1,
                authority: super::super::cards_objects::authority(),
                allotment: 10,
                decisions: list([2]),
            }]),
            requirements: list([part()]),
            deciders: list([Decider { kind: 2, role: 1 }]),
        },
        roles: list([Holding { party: 1, role: HeldRole::Policy { role: 1 } }]),
        spend: Budget { total: 20, own: 0, ended: 0, reserved: 0, left: 20, funder: Actor::Party(1) },
        pools: list([Pool {
            party: 1,
            period: 3,
            budget: Budget { total: 20, own: 0, ended: 0, reserved: 0, left: 20, funder: Actor::Party(1) },
        }]),
        resources: list([part()]),
        services: list([Party { number: 2, kind: PartyKind::Service, name: text(b"Notifier") }]),
        recurring: list([Recurring {
            chip: chip(7),
            key: 1,
            batch: list([recurring_task(1, b"Daily work"), recurring_task(2, b"Then follow up")]),
            budget: 10,
            overlap: RecurringOverlap::Wait,
            last_period: 3,
            pending_period: Some(4),
            last_batch: list([8, 10]),
            cut: false,
            current_period: 3,
            began: Some(Wall::EPOCH),
            next: Some(Wall::EPOCH),
        }]),
        parts: list([part()]),
        acts: list([Offered { effect: None, act: ActKind::Reorder, admission: Admission::Allowed }]),
    }
}
