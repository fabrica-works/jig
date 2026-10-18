use super::*;
use crate::tests::proposed_task::full_template;
pub(super) fn project_pages(words: &Words) {
    for section in [
        ProjectSection::Overview,
        ProjectSection::Goals,
        ProjectSection::Tasks,
        ProjectSection::Notes,
        ProjectSection::Settings,
        ProjectSection::Resources,
        ProjectSection::Policy,
        ProjectSection::Roles,
        ProjectSection::Spend,
        ProjectSection::Recurring,
    ] {
        let (mut d, out) = entered(Address::Project { project: 4, section });
        let doc = full_project(section);
        for request in out {
            match request {
                Request::Client {
                    asked:
                        jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Project { .. }) },
                } => {
                    drive(
                        &mut d,
                        ReadPage { read, result: ReadResult::Value(ReadValue::Project(Box::new(doc.clone()))) },
                    );
                }
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject:
                                jig_client::Watchable::Jig(Subject::Tasks {
                                    query: TaskQuery { tracked: Some(true), .. },
                                    ..
                                }),
                        },
                } => fill_goals(&mut d, watch, &doc),
                Request::Client {
                    asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Tasks { .. }) },
                } => fill_ended_goals(&mut d, read, &doc),
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Inbox { .. }),
                        },
                } => fill_inbox(&mut d, watch),
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Tasks { query: filter, .. }),
                        },
                } => fill_tasks(&mut d, watch, filter),
                Request::Client {
                    asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Notes { .. }) },
                } => fill_notes(&mut d, read),
                Request::Client {
                    asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Spend { .. }) },
                } => fill_spend(&mut d, read, &doc),
                _ => {}
            }
        }
        if section == ProjectSection::Goals {
            drive(
                &mut d,
                Event::Act {
                    action: Act::GoalForm {
                        form: crate::page::project::goals::GoalForm {
                            project: 4,
                            executor: OwnerExecutor::Agent { charter: 1 },
                            budget: u64::MAX,
                            deadline: Some(Wall::from_nanos(u64::MAX)),
                            priority: u32::MAX,
                        },
                    },
                },
            );
            drive(&mut d, Event::Act { action: Act::Edit { field: crate::fields::Purpose::NewGoal, text: full() } });
        }
        render(&d, words);
    }
}

fn full_project(section: ProjectSection) -> Project {
    let mut doc = project_doc();
    doc.name = full();
    if section == ProjectSection::Goals {
        goal_offers(&mut doc);
    }
    doc.resources = parts();
    doc.parts = parts();
    doc.policy.ceiling = authority();
    doc.policy.roles = list([
        RolePolicy { number: 1, authority: authority(), allotment: u64::MAX, decisions: list([1, 2]) },
        RolePolicy { number: 2, authority: authority(), allotment: u64::MAX, decisions: list([1, 2]) },
    ]);
    doc.policy.requirements = parts();
    doc.policy.deciders = list([Decider { kind: 1, role: 1 }, Decider { kind: 2, role: 2 }]);
    doc.roles = list([
        Holding { party: 1, role: HeldRole::Policy { role: 1 } },
        Holding { party: 2, role: HeldRole::Policy { role: 2 } },
    ]);
    doc.services = list([
        Party { number: 1, kind: PartyKind::Service, name: full() },
        Party { number: 2, kind: PartyKind::Service, name: full() },
    ]);
    let recurring = Recurring {
        chip: full_chip(7),
        key: u32::MAX,
        batch: list([full_template(1), full_template(2)]),
        budget: u64::MAX,
        overlap: RecurringOverlap::Wait,
        last_period: u64::MAX,
        pending_period: Some(u64::MAX),
        last_batch: list([8, 9]),
        cut: true,
        current_period: u64::MAX,
        began: Some(Wall::from_nanos(u64::MAX)),
        next: Some(Wall::from_nanos(u64::MAX)),
    };
    doc.recurring = list([recurring.clone(), Recurring { key: 2, ..recurring }]);
    doc.pools = list([*doc.pools.get(0).unwrap(), *doc.pools.get(0).unwrap()]);
    doc.spend_lines =
        list([TaskSpend { chip: full_chip(8), spent: u64::MAX }, TaskSpend { chip: full_chip(9), spent: u64::MAX }]);
    doc
}
