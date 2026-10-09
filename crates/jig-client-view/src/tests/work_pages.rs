#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary fixtures script public events and query semantic nodes"
)]
use super::{
    cards::{contains, document, drive, start, text, words},
    pages::{build, deliver, named_binding},
};
use crate::{JigBindings, Role, Tree, words::Words};
use jig_client::{
    Domain, Event, Request,
    action::Act,
    address::{Address, Place, ProjectSection, TaskSection},
    contract::*,
};
use skein_lib::{List, Wall};
pub(super) fn list<T>(values: impl IntoIterator<Item = T>) -> List<T> {
    let values: Vec<_> = values.into_iter().collect();
    let mut rows = List::with_capacity(u32::try_from(values.len()).unwrap());
    for value in values {
        rows.push(value).ok().unwrap();
    }
    rows
}
pub(super) fn part() -> Part {
    Part::Panel { connector: 2, task: 8 }
}
fn counts() -> PhaseCounts {
    PhaseCounts { waiting: 0, running: 0, held: 0, closing: 0, done: 0, failed: 0, cancelled: 0 }
}
pub(super) fn chip(task: u64) -> Chip {
    let mut chip = super::cards_objects::task_doc(&[]).chip;
    chip.task = task;
    chip
}
pub(super) fn entered(address: Address) -> (Domain, Vec<Request>) {
    let (mut d, f) = start();
    let mut frame = document();
    frame.projects.get_mut(0).unwrap().goals = list([Charter { number: 2, name: text(b"Guide") }]);
    frame.projects.get_mut(0).unwrap().pool.left = 20;
    deliver(&mut d, f, Delivery::Frame(Box::new(frame)));
    let out = drive(&mut d, Event::Went { place: Place::Jig(address) });
    (d, out)
}
pub(super) fn task(section: TaskSection, executor: Executor) -> (Domain, Vec<Request>) {
    let (mut d, out) = entered(Address::Task { task: 8, section });
    let watch = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Task { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let mut doc = super::cards_objects::task_doc(&[ActKind::Cancel]);
    doc.chip.executor = executor;
    doc.resources = list([part()]);
    doc.parts = list([part()]);
    doc.inputs = list([chip(9)]);
    doc.dependencies = list([chip(9)]);
    let mut outputs = deliver(&mut d, watch, Delivery::Task(Box::new(doc)));
    outputs.extend(out);
    (d, outputs)
}
fn acts_are_named(tree: &Tree) {
    assert!(
        tree.nodes().iter().filter(|n| n.binding.is_some()).all(|n| n.name.as_ref().is_some_and(|n| !n.is_empty()))
    );
}
fn slot(tree: &Tree, bindings: &JigBindings) {
    let node = tree.nodes().iter().find(|n| n.role == Role::Slot).unwrap();
    let reference = node.slot.unwrap();
    assert_eq!(bindings.part(reference.token), Some(&part()));
}
fn task_sections(words: &Words) {
    for executor in [
        Executor::Agent { charter: 2 },
        Executor::Procedure { connector: 2, kind: 1 },
        Executor::Party { party: 1 },
        Executor::Role { project: 4, role: 1 },
    ] {
        for section in [TaskSection::Overview, TaskSection::Activity, TaskSection::Budget, TaskSection::Panels] {
            let (d, _) = task(section, executor);
            let (tree, bindings) = build(&d, 3, words);
            assert!(contains(&tree, b"Task 8"));
            assert!(contains(&tree, b"held") || contains(&tree, b"Held"));
            assert!(contains(&tree, b"Goal"));
            assert!(contains(&tree, b"Priority 1"));
            acts_are_named(&tree);
            if section == TaskSection::Overview {
                assert!(contains(&tree, b"whole task specification"));
                slot(&tree, &bindings);
            }
            if section == TaskSection::Panels {
                slot(&tree, &bindings);
            }
            if section == TaskSection::Activity && executor == (Executor::Procedure { connector: 2, kind: 1 }) {
                slot(&tree, &bindings);
            }
            if section == TaskSection::Budget {
                assert!(contains(&tree, b"Total budget: 20"));
                assert!(contains(&tree, b"Left: 20"));
            }
        }
    }
}
fn plan_folds(words: &Words) {
    let (mut d, out) = task(TaskSection::Plan, Executor::Agent { charter: 2 });
    let tree_watch = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Tree { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let node = |task, parent| TreeNode {
        chip: chip(task),
        parent,
        dependencies: list([]),
        proposals: list([]),
        spend: 1,
        folded: counts(),
        held_below: 1,
    };
    deliver(
        &mut d,
        tree_watch,
        Delivery::Tree(Box::new(jig_client::contract::Tree {
            task: 8,
            nodes: list([node(9, None), node(10, Some(9))]),
            folded: PhaseCounts { done: 3, ..counts() },
            held_below: 2,
        })),
    );
    let (tree, mut bindings) = build(&d, 3, words);
    assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::TreeItem).count(), 2);
    assert!(contains(&tree, b"Held below: 2"));
    let fold = crate::decode(&mut bindings, named_binding(&tree, b"Fold subtree"), crate::Act::Press).unwrap();
    drive(&mut d, Event::Act { act: fold });
    let (folded, mut bindings) = build(&d, 3, words);
    assert_eq!(folded.nodes().iter().filter(|n| n.role == Role::TreeItem).count(), 1);
    assert!(contains(&folded, b"Held below: 1"));
    let open = crate::decode(&mut bindings, named_binding(&folded, b"Open subtree"), crate::Act::Press).unwrap();
    drive(&mut d, Event::Act { act: open });
    assert_eq!(build(&d, 3, words).0.nodes().iter().filter(|n| n.role == Role::TreeItem).count(), 2);
    for filter in [
        jig_client::pages::task::PlanFilter::Held,
        jig_client::pages::task::PlanFilter::Live,
        jig_client::pages::task::PlanFilter::All,
    ] {
        drive(&mut d, Event::Act { act: Act::FilterPlan { filter } });
        acts_are_named(&build(&d, 3, words).0);
    }
    drive(&mut d, Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Plan }) });
    assert!(d.task_page().plan.nodes.is_empty());
}
pub(super) fn project_doc() -> Project {
    let doc = super::cards_objects::task_doc(&[]);
    Project {
        period: 3,
        spend_lines: list([TaskSpend { chip: chip(8), spent: 5 }]),
        number: 4,
        name: text(b"Workshop"),
        policy: Policy {
            ceiling: doc.authority.clone(),
            period: 3,
            spend: 20,
            roles: list([RolePolicy { number: 1, authority: doc.authority, allotment: 10, decisions: list([2]) }]),
            requirements: list([part()]),
            deciders: list([Decider { kind: 2, role: 1 }]),
        },
        roles: list([Holding { party: 1, role: 1 }]),
        spend: doc.budget,
        pools: list([Pool { party: 1, period: 3, budget: doc.budget }]),
        resources: list([part()]),
        services: list([Party { number: 2, kind: PartyKind::Service, name: text(b"Notifier") }]),
        recurring: list([Recurring {
            number: 1,
            spec: text(b"Daily work"),
            period: 3,
            budget: 10,
            last: list([8]),
            current_period: 3,
            began: Some(Wall::EPOCH),
            next: Some(Wall::EPOCH),
        }]),
        parts: list([part()]),
        acts: list([Offered { act: ActKind::SetGoal, admission: Admission::Allowed }]),
    }
}
pub(super) fn project(section: ProjectSection) -> (Domain, Vec<Request>) {
    let (mut d, out) = entered(Address::Project { project: 4, section });
    let read = out
        .iter()
        .find_map(|r| match r {
            Request::Read { read, what: Read::Project { .. } } => Some(*read),
            _ => None,
        })
        .unwrap();
    let mut outputs =
        drive(&mut d, Event::Read { read, result: ReadResult::Value(ReadValue::Project(Box::new(project_doc()))) });
    outputs.extend(out);
    (d, outputs)
}
fn history(words: &Words) {
    let (mut d, out) = task(TaskSection::History, Executor::Agent { charter: 2 });
    let mut read = out
        .iter()
        .find_map(|r| match r {
            Request::Read { read, what: Read::History { .. } } => Some(*read),
            _ => None,
        })
        .unwrap();
    let doc = super::cards_objects::task_doc(&[]);
    let changes = [
        HistoryChange::Made { batch: 7 },
        HistoryChange::Amended { before: text(b"old instruction"), after: text(b"new instruction") },
        HistoryChange::Held(doc.hold.unwrap()),
        HistoryChange::Released,
        HistoryChange::Cancelled { reason: text(b"cancel history") },
        HistoryChange::Moved { from: Actor::Party(1), to: Actor::Party(2), reason: text(b"move history") },
    ];
    for (page, changes) in changes.chunks(2).enumerate() {
        drive(
            &mut d,
            Event::Read {
                read,
                result: ReadResult::Value(ReadValue::History(Box::new(HistoryPage {
                    rows: list(changes.iter().enumerate().map(|(index, change)| HistoryLine {
                        position: u64::try_from(page.checked_mul(2).unwrap().checked_add(index).unwrap()).unwrap(),
                        task: 8,
                        by: Actor::Party(1),
                        at: Wall::EPOCH,
                        change: change.clone(),
                    })),
                    before: Some(Cursor { value: text(b"before") }),
                    after: Some(Cursor { value: text(b"after") }),
                }))),
            },
        );
        let (tree, mut bindings) = build(&d, 3, words);
        let labels: &[&[u8]] = match page {
            0 => &[b"old instruction", b"new instruction"],
            1 => &[b"Released", b"Budget"],
            2 => &[b"cancel history", b"move history"],
            _ => unreachable!(),
        };
        for label in labels {
            assert!(contains(&tree, label), "{label:?}");
        }
        for label in [b"Read earlier".as_slice(), b"Read later"] {
            assert!(contains(&tree, label));
        }
        assert_eq!(tree.nodes().iter().filter_map(|n| n.key).filter(|key| key.kind == 21).count(), 2);
        assert_eq!(
            crate::decode(&mut bindings, named_binding(&tree, b"This task and its subtree"), crate::Act::Press),
            Some(Act::HistoryScope { subtree: true })
        );
        let more = crate::decode(&mut bindings, named_binding(&tree, b"Read later"), crate::Act::Press).unwrap();
        let out = drive(&mut d, Event::Act { act: more });
        read = out
            .iter()
            .find_map(|r| match r {
                Request::Read { read, what: Read::History { .. } } => Some(*read),
                _ => None,
            })
            .unwrap();
    }
}
pub(super) fn project_rows(d: &mut Domain, out: &[Request]) {
    for r in out {
        match r {
            Request::Open { watch, subject: Subject::Goals { .. } } => {
                let doc = super::cards_objects::task_doc(&[]);
                deliver(
                    d,
                    *watch,
                    Delivery::Goals(Box::new(Goals {
                        project: 4,
                        rows: list([GoalLine {
                            chip: chip(8),
                            priority: 2,
                            progress: counts(),
                            budget: doc.budget,
                            deadline: Some(Wall::EPOCH),
                            held_below: 1,
                            changed: Wall::EPOCH,
                        }]),
                        more: true,
                    })),
                );
            }
            Request::Open { watch, subject: Subject::Tasks { filter, .. } } => {
                deliver(
                    d,
                    *watch,
                    Delivery::Tasks(Box::new(Tasks {
                        project: 4,
                        filter: *filter,
                        rows: list([TaskLine {
                            chip: chip(8),
                            goal: Some(8),
                            requester: Actor::Party(1),
                            changed: Wall::EPOCH,
                            asked_by_me: true,
                        }]),
                        more: true,
                    })),
                );
            }
            Request::Read { read, what: Read::Notes { .. } } => {
                drive(
                    d,
                    Event::Read {
                        read: *read,
                        result: ReadResult::Value(ReadValue::Notes(Box::new(NotePage {
                            rows: list([Note {
                                number: 1,
                                revision: Revision { value: 7 },
                                scope: NoteScope::Project(4),
                                description: text(b"Useful correction"),
                                body: text(b"whole note body"),
                                by: Actor::Party(1),
                                tasks: list([8]),
                                changed: Wall::EPOCH,
                                acts: list([Offered { act: ActKind::EditNote, admission: Admission::Allowed }]),
                            }]),
                            before: Some(Cursor { value: text(b"before") }),
                            after: Some(Cursor { value: text(b"after") }),
                        }))),
                    },
                );
            }
            _ => {}
        }
    }
}
fn project_sections(words: &Words) {
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
        let (mut d, out) = project(section);
        project_rows(&mut d, &out);
        let (tree, bindings) = build(&d, 3, words);
        assert!(contains(&tree, b"Workshop"));
        acts_are_named(&tree);
        match section {
            ProjectSection::Overview
            | ProjectSection::Settings
            | ProjectSection::Resources
            | ProjectSection::Policy => slot(&tree, &bindings),
            ProjectSection::Goals => {
                assert!(tree.nodes().iter().any(|n| n.name.as_deref() == Some(b"Goal specification")));
            }
            ProjectSection::Tasks => {
                assert!(contains(&tree, b"Tasks I asked for"));
                assert!(contains(&tree, b"Goal 8"));
            }
            ProjectSection::Notes => {
                assert!(tree.nodes().iter().any(|n| n.name.as_deref() == Some(b"Search notes by description")));
                for label in [b"whole note body".as_slice(), b"Useful correction", b"Read earlier", b"Read later"] {
                    assert!(contains(&tree, label));
                }
            }
            ProjectSection::Roles => assert!(contains(&tree, b"Notifier")),
            ProjectSection::Spend => assert!(contains(&tree, b"Period 3")),
            ProjectSection::Recurring => assert!(contains(&tree, b"Daily work")),
        }
    }
}
fn goal_form(words: &Words) {
    let (mut d, _) = project(ProjectSection::Goals);
    let form =
        jig_client::pages::project::goals::GoalForm { project: 4, charter: 2, budget: 30, deadline: None, priority: 1 };
    drive(&mut d, Event::Act { act: Act::GoalForm { form } });
    let (tree, mut bindings) = build(&d, 3, words);
    assert!(contains(&tree, b"this will be proposed"));
    let action = crate::decode(
        &mut bindings,
        named_binding(&tree, b"Goal budget"),
        crate::Act::Input { value: Box::from(b"40".as_slice()) },
    )
    .unwrap();
    match action {
        Act::GoalForm { form: changed } => {
            assert_eq!(changed, jig_client::pages::project::goals::GoalForm { budget: 40, ..form });
        }
        _ => panic!("typed goal field"),
    }
    let at = crate::confirmation::amendment::deadline(b"2026-10-09 17:00").unwrap();
    let action = crate::decode(
        &mut bindings,
        named_binding(&tree, b"Goal deadline (UTC, YYYY-MM-DD HH:MM)"),
        crate::Act::Input { value: Box::from(b"2026-10-09 17:00".as_slice()) },
    )
    .unwrap();
    assert_eq!(
        action,
        Act::GoalForm { form: jig_client::pages::project::goals::GoalForm { deadline: Some(at), ..form } }
    );
    drive(&mut d, Event::Act { act: action });
    let (dated, _) = build(&d, 3, words);
    let field = dated
        .nodes()
        .iter()
        .find(|node| node.name.as_deref() == Some(b"Goal deadline (UTC, YYYY-MM-DD HH:MM)"))
        .unwrap();
    assert_eq!(field.value.as_ref().unwrap().text.as_ref(), b"2026-10-09 17:00");
    assert!(
        crate::decode(
            &mut bindings,
            named_binding(&tree, b"Goal priority"),
            crate::Act::Input { value: Box::from(b"bad".as_slice()) }
        )
        .is_none()
    );
}
fn deployment(words: &Words) {
    let (mut d, out) = entered(Address::Deployment);
    let watch = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Deployment } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let value = Deployment {
        hosts: list([
            Host { number: 1, connected: true, grace: None, used: 1, free: 2, runs: list([chip(8)]), refused: None },
            Host {
                number: 2,
                connected: false,
                grace: Some(Wall::EPOCH),
                used: 0,
                free: 0,
                runs: list([]),
                refused: Some(text(b"No capacity")),
            },
        ]),
        accounts: list([Account {
            number: 1,
            usable: false,
            reason: Some(text(b"Cooling down")),
            runs: list([chip(8)]),
            waiting: list([chip(9)]),
        }]),
        capacity: list([
            Capacity { project: None, live: 3, limit: 4 },
            Capacity { project: Some(4), live: 2, limit: 3 },
        ]),
        proposals: 2,
        refusals: list([Refusal::Limit { which: Limit::LiveTasks { project: 4, limit: 3 } }]),
        parts: list([part()]),
    };
    deliver(&mut d, watch, Delivery::Deployment(Box::new(value)));
    let (tree, bindings) = build(&d, 3, words);
    for label in [
        b"Connected".as_slice(),
        b"Lost",
        b"Grace holds until",
        b"Slots used: 1",
        b"Slots free: 2",
        b"Cooling down",
        b"Live tasks: 3",
        b"Task limit: 4",
        b"Pending proposals",
    ] {
        assert!(contains(&tree, label), "{label:?}");
    }
    slot(&tree, &bindings);
    acts_are_named(&tree);
}
#[test]
fn work_pages_show_executor_sections_folds_project_settings_and_deployment_capacity() {
    let words = words();
    task_sections(&words);
    plan_folds(&words);
    history(&words);
    project_sections(&words);
    recurring_times_use_supplied_boundaries_without_inferred_cadence(&words);
    goal_form(&words);
    deployment(&words);
}

fn recurring_times_use_supplied_boundaries_without_inferred_cadence(words: &Words) {
    for next in [None, Some(Wall::from_nanos(3_600_000_000_000))] {
        for began in [None, Some(Wall::from_nanos(1_800_000_000_000))] {
            let (mut d, out) = entered(Address::Project { project: 4, section: ProjectSection::Recurring });
            let read = out
                .iter()
                .find_map(|request| match request {
                    Request::Read { read, what: Read::Project { .. } } => Some(*read),
                    _ => None,
                })
                .unwrap();
            let mut doc = project_doc();
            let recurring = doc.recurring.get_mut(0).unwrap();
            recurring.next = next;
            recurring.current_period = 9;
            recurring.began = began;
            drive(&mut d, Event::Read { read, result: ReadResult::Value(ReadValue::Project(Box::new(doc))) });
            let (tree, _) = build(&d, 3, words);
            assert!(contains(&tree, b"Daily work"));
            assert!(contains(&tree, b"Period 3"));
            assert!(contains(&tree, b"Current period 9"));
            assert_eq!(contains(&tree, b"Period began at 01:30"), began.is_some());
            assert_eq!(contains(&tree, b"Runs at the next period"), next.is_none());
            assert_eq!(contains(&tree, b"Next runs at 02:00"), next.is_some());
            assert!(!contains(&tree, b"Deadline"));
        }
    }
}
