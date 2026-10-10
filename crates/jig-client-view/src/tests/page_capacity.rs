#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary maximum fixtures script public sources and inspect node budgets"
)]
use super::{
    cards::{CLIENT, document, drive, start, words},
    pages::deliver,
    work_pages::{chip, list, part, project_doc, recurring_task},
};
use crate::{Builder, JigBindings, Owner, Role, SlotRef, Tree, words::Words};
use jig_client::{
    Domain, Event, Request,
    action::Act,
    address::{Address, Place, ProjectSection, TaskSection},
    contract::*,
};
use skein_lib::List;
use skein_lib::{Token, Wall};
fn full() -> Text {
    let mut bytes = Vec::new();
    bytes.resize(usize::try_from(CLIENT.text).unwrap(), b'x');
    Text::new(bytes.into_boxed_slice(), CLIENT.text).unwrap()
}
fn full_chip(number: u64) -> Chip {
    let mut value = chip(number);
    value.title = full();
    value
}
fn parts() -> List<Part> {
    list([part(), Part::Panel { connector: 3, task: 8 }])
}
fn authority() -> Authority {
    Authority {
        budget: u64::MAX,
        grants: list([
            Grant { connector: 2, kind: 1, pattern: full() },
            Grant { connector: 3, kind: 2, pattern: full() },
        ]),
        task_kinds: list([Executor::Agent { charter: 2 }, Executor::Procedure { connector: 3, kind: 1 }]),
        depth: 2,
        tasks: 2,
        deadline: Some(Wall::from_nanos(u64::MAX)),
        tools: list([1, 2]),
        note_scopes: list([
            NoteScope::Resource { project: 4, connector: 2, pattern: full() },
            NoteScope::Resource { project: 4, connector: 3, pattern: full() },
        ]),
    }
}
fn entered(address: Address) -> (Domain, Vec<Request>) {
    let (mut d, watch) = start();
    let mut frame = document();
    let mut project = frame.projects.get(0).unwrap().clone();
    project.name = full();
    project.roles = list([1, 2]);
    project.chats = list([Charter { number: 1, name: full() }, Charter { number: 2, name: full() }]);
    project.goals = project.chats.clone();
    let mut second = project.clone();
    second.project = 5;
    frame.projects = list([project, second]);
    frame.party.name = full();
    deliver(&mut d, watch, Delivery::Frame(Box::new(frame)));
    let out = drive(&mut d, Event::Went { place: Place::Jig(address) });
    (d, out)
}
fn render(d: &Domain, words: &Words) -> Tree {
    let limits = crate::Limits::of_words(&CLIENT, words).unwrap();
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(limits.bindings, CLIENT.text);
    b.open(Role::Page);
    assert!(crate::jig(
        &mut b,
        d,
        SlotRef { owner: Owner::Other { child: 1 }, token: Token::new(1) },
        words,
        &mut bindings
    ));
    b.close();
    let tree = b.finish();
    assert!(u32::try_from(tree.nodes().len()).unwrap() <= limits.nodes);
    assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::Heading { level: 1 }).count(), 1);
    assert!(
        tree.nodes()
            .iter()
            .filter(|n| n.binding.is_some())
            .all(|n| n.name.as_ref().is_some_and(|name| !name.is_empty()))
    );
    tree
}
fn row(position: u64, event: Activity) -> ActivityLine {
    ActivityLine { position, at: Wall::EPOCH, event }
}
fn full_call(number: u64) -> Call {
    Call {
        number,
        tool: full(),
        input: full(),
        outcome: Some(full()),
        failed: false,
        objects: list([ObjectKey::Task(8), ObjectKey::Task(9)]),
        parts: parts(),
    }
}
fn full_turn() -> Turn {
    Turn {
        messages: list([
            Message { speaker: Speaker::Assistant, blocks: list([Block::Text(full()), Block::Call(full_call(1))]) },
            Message { speaker: Speaker::Assistant, blocks: list([Block::Text(full()), Block::Call(full_call(2))]) },
        ]),
        used: Usage { input: u64::MAX, output: u64::MAX, cached: u64::MAX, cache_written: u64::MAX },
        spent: u64::MAX,
        read_through: None,
        cut: true,
    }
}
fn task_pages(words: &Words) {
    for section in [
        TaskSection::Overview,
        TaskSection::Activity,
        TaskSection::Plan,
        TaskSection::History,
        TaskSection::Budget,
        TaskSection::Panels,
    ] {
        let (mut d, out) = entered(Address::Task { task: 8, section });
        let watch = out
            .iter()
            .find_map(|r| match r {
                Request::Open { watch, subject: Subject::Task { .. } } => Some(*watch),
                _ => None,
            })
            .unwrap();
        let mut doc = super::cards_objects::task_doc(&[ActKind::Cancel, ActKind::Amend]);
        doc.chip = full_chip(8);
        doc.chip.executor = Executor::Agent { charter: 2 };
        doc.spec = specification(full());
        doc.instructions = full();
        doc.authority = authority();
        doc.lineage_chips = list([full_chip(1), full_chip(2)]);
        doc.inputs = list([full_chip(9), full_chip(10)]);
        doc.dependencies = doc.inputs.clone();
        doc.references = doc.inputs.clone();
        doc.resources = parts();
        doc.parts = parts();
        doc.writer_slots = list([
            WriterSlot { resource: part(), task: 9, attempt: Some(1) },
            WriterSlot { resource: part(), task: 10, attempt: Some(2) },
        ]);
        doc.wake.topics = list([1, 2]);
        doc.wake.subscriptions = parts();
        doc.spend = list([
            SpendLine { task: 9, attempt: Some(1), spent: u64::MAX },
            SpendLine { task: 10, attempt: None, spent: u64::MAX },
        ]);
        doc.activity = 2;
        doc.runtime = Runtime::Running { attempt: 1, since: Wall::EPOCH, tool: Some(full()) };
        let mut sources = deliver(&mut d, watch, Delivery::Task(Box::new(doc)));
        sources.extend(out);
        for request in sources {
            match request {
                Request::Open { watch, subject: Subject::Tree { .. } } => fill_tree(&mut d, watch),
                Request::Read { read, what: Read::Activity { .. } } => fill_activity(&mut d, read),
                Request::Read { read, what: Read::History { .. } } => fill_history(&mut d, read),
                _ => {}
            }
        }
        let tree = render(&d, words);
        if section == TaskSection::Plan {
            assert_eq!(tree.nodes().iter().filter(|node| node.role == Role::TreeItem).count(), 2);
        }
        if section == TaskSection::Activity {
            assert!(tree.nodes().iter().any(|node| node.role == Role::Log));
        }
    }
}
fn project_pages(words: &Words) {
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
        doc.roles = list([Holding { party: 1, role: 1 }, Holding { party: 2, role: 2 }]);
        doc.services = list([
            Party { number: 1, kind: PartyKind::Service, name: full() },
            Party { number: 2, kind: PartyKind::Service, name: full() },
        ]);
        let recurring = Recurring {
            chip: full_chip(7),
            key: u32::MAX,
            batch: list([recurring_task(1, full().bytes()), recurring_task(2, full().bytes())]),
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
        doc.spend_lines = list([
            TaskSpend { chip: full_chip(8), spent: u64::MAX },
            TaskSpend { chip: full_chip(9), spent: u64::MAX },
        ]);
        for request in out {
            match request {
                Request::Read { read, what: Read::Project { .. } } => {
                    drive(
                        &mut d,
                        Event::Read { read, result: ReadResult::Value(ReadValue::Project(Box::new(doc.clone()))) },
                    );
                }
                Request::Open { watch, subject: Subject::Goals { .. } } => fill_goals(&mut d, watch, &doc),
                Request::Read { read, what: Read::Goals { .. } } => fill_ended_goals(&mut d, read, &doc),
                Request::Open { watch, subject: Subject::Inbox { .. } } => fill_inbox(&mut d, watch),
                Request::Open { watch, subject: Subject::Tasks { filter, .. } } => fill_tasks(&mut d, watch, filter),
                Request::Read { read, what: Read::Notes { .. } } => fill_notes(&mut d, read),
                Request::Read { read, what: Read::Spend { .. } } => fill_spend(&mut d, read, &doc),
                _ => {}
            }
        }
        if section == ProjectSection::Goals {
            drive(
                &mut d,
                Event::Act {
                    act: Act::GoalForm {
                        form: jig_client::pages::project::goals::GoalForm {
                            project: 4,
                            executor: GoalExecutor::Charter(1),
                            budget: u64::MAX,
                            deadline: Some(Wall::from_nanos(u64::MAX)),
                            priority: u32::MAX,
                        },
                    },
                },
            );
            drive(&mut d, Event::Act { act: Act::Edit { field: jig_client::fields::Purpose::NewGoal, text: full() } });
        }
        render(&d, words);
    }
}
fn goal_offers(doc: &mut Project) {
    doc.acts = list([Offered {
        act: ActKind::SetGoal,
        admission: Admission::Bounded {
            within: Bounds::Goal {
                pool_left: u64::MAX,
                charters: list([1, 2]),
                procedures: list([
                    GoalProcedure { connector: u16::MAX, code: u32::MAX },
                    GoalProcedure { connector: u16::MAX, code: u32::MAX.checked_sub(1).unwrap() },
                ]),
                latest: None,
                budget: 0,
                maximum_budget: u64::MAX,
                maximum_deadline: None,
            },
            beyond: Beyond::Proposed { to: Recipient::Roles { project: 4, roles: list([1, 2]) } },
        },
    }]);
}
fn other_pages(words: &Words) {
    deployment(words);
    conversations(words);
    inbox(words);
    not_the_work(words);
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
    let host = |number| Host {
        number,
        connected: false,
        grace: Some(Wall::from_nanos(u64::MAX)),
        used: u32::MAX,
        free: u32::MAX,
        runs: list([full_chip(8), full_chip(9)]),
        refused: Some(full()),
    };
    let account = |number| Account {
        number,
        usable: false,
        reason: Some(full()),
        runs: list([full_chip(8), full_chip(9)]),
        waiting: list([full_chip(8), full_chip(9)]),
    };
    deliver(
        &mut d,
        watch,
        Delivery::Deployment(Box::new(Deployment {
            hosts: list([host(1), host(2)]),
            accounts: list([account(1), account(2)]),
            capacity: list([
                Capacity { project: None, live: u32::MAX, limit: u32::MAX },
                Capacity { project: Some(4), live: u32::MAX, limit: u32::MAX },
            ]),
            proposals: u32::MAX,
            refusals: list([Refusal::Limit { which: Limit::Objects }, Refusal::Limit { which: Limit::Reads }]),
            parts: parts(),
        })),
    );
    render(&d, words);
}
fn conversations(words: &Words) {
    let (mut d, out) = entered(Address::Conversations { project: None });
    let watch = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Conversations } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let line = |number| ConversationLine {
        runtime: Runtime::Waiting { dependencies: list([8, 9]), hold: Some(part()) },
        delegates: u32::MAX,
        budget: project_doc().spend,
        chip: full_chip(number),
        project: 4,
        first_words: full(),
        unread: Count { value: u32::MAX, limited: true },
        seen: 0,
        last_reply: u64::MAX,
        changed: Wall::EPOCH,
    };
    deliver(
        &mut d,
        watch,
        Delivery::Conversations(Box::new(Conversations { rows: list([line(8), line(9)]), more: true })),
    );
    render(&d, words);
}
fn inbox(words: &Words) {
    let (mut d, out) = entered(Address::Inbox { project: None });
    let watch = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Inbox { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let entry = |number| InboxEntry {
        object: ObjectKey::Task(number),
        group: InboxGroup::PersonTasks,
        project: 4,
        lineage: list([full_chip(1), full_chip(2)]),
        goal: Some(8),
        recipient: Some(Recipient::Role { project: 4, role: 1 }),
        stall: Some(Wall::from_nanos(u64::MAX)),
        reply: None,
        position: number,
        at: Wall::EPOCH,
    };
    deliver(
        &mut d,
        watch,
        Delivery::Inbox(Box::new(Inbox {
            entries: list([entry(8), entry(9)]),
            needs: Needs { questions: u32::MAX, proposals: u32::MAX, escalations: u32::MAX, person_tasks: u32::MAX },
            updates: Count { value: u32::MAX, limited: true },
            more: true,
            results_seen: 0,
            replies_seen: list([ReplyPosition { chat: 8, through: 0 }, ReplyPosition { chat: 9, through: 0 }]),
        })),
    );
    for request in &out {
        if let Request::Read { read, what: Read::Inbox { group, .. } } = request {
            let mut first = entry(if *group == InboxReadGroup::Needs { 10 } else { 12 });
            let mut second = entry(if *group == InboxReadGroup::Needs { 11 } else { 13 });
            if *group == InboxReadGroup::Updates {
                first.group = InboxGroup::Results;
                second.group = InboxGroup::Results;
            }
            drive(
                &mut d,
                Event::Read {
                    read: *read,
                    result: ReadResult::Value(ReadValue::Inbox(Box::new(InboxPage {
                        rows: list([first, second]),
                        before: Some(Cursor { value: full() }),
                        after: Some(Cursor { value: full() }),
                    }))),
                },
            );
        }
    }
    assert_eq!(d.inbox().rows.len(), 6);
    render(&d, words);
}
fn not_the_work(words: &Words) {
    let mut d = Domain::new(&CLIENT, 1);
    for number in [1, 2] {
        d.provider(jig_client::signin::Provider { number, name: full() }).unwrap();
    }
    drive(
        &mut d,
        Event::Start {
            address: Address::Inbox { project: None },
            kept: list([]),
            offset: jig_client::boundary::Offset { minutes: 0 },
        },
    );
    drive(&mut d, Event::SignedOut);
    render(&d, words);
    let (d, _) = start();
    render(&d, words);
}
#[test]
fn every_page_with_full_windows_nested_lists_and_maximum_text_fits_its_composed_capacity() {
    let words = words();
    task_pages(&words);
    project_pages(&words);
    other_pages(&words);
}
#[test]
#[expect(
    clippy::disallowed_macros,
    clippy::print_stdout,
    reason = "reports configured capacity evidence for the composition handoff"
)]
fn capacity_composes_siblings_fragments_and_application_pages_with_checked_overflow() {
    let words = words();
    let limits = crate::Limits::of_words(&CLIENT, &words).unwrap();
    let largest = crate::page_nodes(&CLIENT, 0).unwrap();
    assert_eq!(
        limits.nodes,
        largest
            .checked_add(crate::frame_nodes(&CLIENT).unwrap())
            .unwrap()
            .checked_add(crate::confirmation_nodes(&CLIENT).unwrap())
            .unwrap()
            .checked_add(crate::notices_nodes(&CLIENT).unwrap())
            .unwrap()
            .checked_add(1)
            .unwrap()
    );
    let extra = crate::Limits::compose(&CLIENT, &words, largest.checked_add(7).unwrap(), 9, 2).unwrap();
    assert_eq!(extra.nodes, limits.nodes.checked_add(16).unwrap());
    assert_eq!(extra.depth, limits.depth.checked_add(2).unwrap());
    assert_eq!(crate::page_nodes(&CLIENT, 9), largest.checked_add(9));
    assert!(crate::Limits::compose(&CLIENT, &words, u32::MAX, 0, 0).is_none());
    assert!(crate::page_nodes(&CLIENT, u32::MAX).is_none());
    let mut huge = CLIENT;
    huge.rows = u32::MAX;
    assert!(crate::page_nodes(&huge, 0).is_none());
    assert!(crate::worst_case(&limits).is_some());
    assert!(crate::jig_bindings_worst_case(limits.bindings, &CLIENT).is_some());
    let (mut d, _) = start();
    drive(&mut d, Event::Went { place: Place::Elsewhere });
    let mut b = Builder::new(&extra);
    let mut bindings = JigBindings::new(limits.bindings, CLIENT.text);
    b.open(Role::Page);
    assert!(!crate::jig(
        &mut b,
        &d,
        SlotRef { owner: Owner::Other { child: 1 }, token: Token::new(1) },
        &words,
        &mut bindings
    ));
    b.open(Role::Heading { level: 1 });
    b.text(crate::Run::Plain, b"Application page");
    b.close();
    crate::after_page(&mut b, &d, &words, &mut bindings);
    b.close();
    let slots = b.slots(0);
    assert_eq!(slots.len(), 1);
    let mut fragment = Builder::new(&extra);
    fragment.open(Role::Region);
    fragment.name(b"Application navigation");
    fragment.open(Role::Button);
    fragment.name(b"Home");
    fragment.close();
    fragment.close();
    b.splice(slots[0].id, fragment);
    let tree = b.finish();
    assert_eq!(tree.nodes().iter().filter(|node| node.role == Role::Heading { level: 1 }).count(), 1);
    println!(
        "fixture page nodes={largest}, composed nodes={}, depth={}, text/node={}, text bytes={}, bindings={}, slots={}, patches={}",
        limits.nodes,
        limits.depth,
        limits.text_per_node,
        limits.text_bytes,
        limits.bindings,
        limits.slots,
        limits.patches
    );
}

fn fill_tree(d: &mut Domain, watch: Token) {
    let node = |number, parent| TreeNode {
        chip: full_chip(number),
        parent,
        dependencies: list([1, 2]),
        proposals: list([ProposalKey { proposer: 8, number: 1 }, ProposalKey { proposer: 8, number: 2 }]),
        spend: u64::MAX,
        folded: PhaseCounts { waiting: 2, running: 2, held: 2, closing: 2, done: 2, failed: 2, cancelled: 2 },
        held_below: 2,
    };
    deliver(
        d,
        watch,
        Delivery::Tree(Box::new(jig_client::contract::Tree {
            task: 8,
            nodes: list([node(9, None), node(10, Some(9))]),
            folded: PhaseCounts { waiting: 2, running: 2, held: 2, closing: 2, done: 2, failed: 2, cancelled: 2 },
            held_below: 2,
        })),
    );
}
fn fill_activity(d: &mut Domain, read: Token) {
    drive(
        d,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Activity(Box::new(ActivityPage {
                rows: list([
                    row(2, Activity::Turn { name: TurnName { attempt: 1, number: 2 }, turn: full_turn() }),
                    row(1, Activity::Turn { name: TurnName { attempt: 1, number: 1 }, turn: full_turn() }),
                ]),
                before: Some(Cursor { value: full() }),
                after: Some(Cursor { value: full() }),
            }))),
        },
    );
    for number in [1, 2] {
        for call in [1, 2] {
            drive(
                d,
                Event::Act {
                    act: Act::FoldCall {
                        name: jig_client::conversation::CallName { attempt: 1, turn: number, call },
                        open: true,
                    },
                },
            );
        }
    }
}
fn fill_history(d: &mut Domain, read: Token) {
    drive(
        d,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::History(Box::new(HistoryPage {
                rows: list([
                    HistoryLine {
                        position: 2,
                        task: 8,
                        by: Actor::Party(1),
                        at: Wall::EPOCH,
                        change: HistoryChange::Amended { before: full(), after: full() },
                    },
                    HistoryLine {
                        position: 1,
                        task: 9,
                        by: Actor::Party(1),
                        at: Wall::EPOCH,
                        change: HistoryChange::Amended { before: full(), after: full() },
                    },
                ]),
                before: Some(Cursor { value: full() }),
                after: Some(Cursor { value: full() }),
            }))),
        },
    );
}

fn fill_goals(d: &mut Domain, watch: Token, doc: &Project) {
    let goal = |number| GoalLine {
        changed: Wall::EPOCH,
        chip: full_chip(number),
        priority: u32::MAX,
        progress: PhaseCounts { waiting: 2, running: 2, held: 2, closing: 2, done: 2, failed: 2, cancelled: 2 },
        budget: doc.spend,
        deadline: Some(Wall::from_nanos(u64::MAX)),
        held_below: 2,
    };
    deliver(d, watch, Delivery::Goals(Box::new(Goals { project: 4, rows: list([goal(8), goal(9)]), more: true })));
}
fn fill_ended_goals(d: &mut Domain, read: Token, doc: &Project) {
    let goal = |number| GoalLine {
        changed: Wall::EPOCH,
        chip: full_chip(number),
        priority: u32::MAX,
        progress: PhaseCounts { waiting: 2, running: 2, held: 2, closing: 2, done: 2, failed: 2, cancelled: 2 },
        budget: doc.spend,
        deadline: Some(Wall::from_nanos(u64::MAX)),
        held_below: 2,
    };
    drive(
        d,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Goals(Box::new(GoalPage {
                rows: list([goal(10), goal(11)]),
                before: Some(Cursor { value: full() }),
                after: Some(Cursor { value: full() }),
            }))),
        },
    );
}
fn fill_inbox(d: &mut Domain, watch: Token) {
    let entry = |number| InboxEntry {
        object: ObjectKey::Task(number),
        group: InboxGroup::PersonTasks,
        project: 4,
        lineage: list([full_chip(1), full_chip(2)]),
        goal: Some(8),
        recipient: Some(Recipient::Role { project: 4, role: 1 }),
        stall: Some(Wall::from_nanos(u64::MAX)),
        reply: None,
        position: number,
        at: Wall::EPOCH,
    };
    deliver(
        d,
        watch,
        Delivery::Inbox(Box::new(Inbox {
            entries: list([entry(8), entry(9)]),
            needs: Needs { questions: u32::MAX, proposals: u32::MAX, escalations: u32::MAX, person_tasks: u32::MAX },
            updates: Count { value: u32::MAX, limited: true },
            more: true,
            results_seen: 0,
            replies_seen: list([ReplyPosition { chat: 8, through: 0 }, ReplyPosition { chat: 9, through: 0 }]),
        })),
    );
}
fn fill_tasks(d: &mut Domain, watch: Token, filter: TaskFilter) {
    let row = |number| TaskLine {
        asked_by_me: true,
        chip: full_chip(number),
        goal: Some(8),
        requester: Actor::Party(1),
        changed: Wall::EPOCH,
    };
    deliver(
        d,
        watch,
        Delivery::Tasks(Box::new(Tasks { project: 4, filter, rows: list([row(8), row(9)]), more: true })),
    );
}
fn fill_notes(d: &mut Domain, read: Token) {
    let note = |number| Note {
        number,
        revision: Revision { value: 7 },
        scope: NoteScope::Resource { project: 4, connector: 2, pattern: full() },
        description: DocumentText::complete(full()),
        body: DocumentText::complete(full()),
        by: Actor::Party(1),
        tasks: list([8, 9]),
        changed: Wall::EPOCH,
        acts: list([
            Offered { act: ActKind::EditNote, admission: Admission::Allowed },
            Offered { act: ActKind::DeleteNote, admission: Admission::Allowed },
        ]),
        cut: false,
    };
    drive(
        d,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Notes(Box::new(NotePage {
                rows: list([note(1), note(2)]),
                before: Some(Cursor { value: full() }),
                after: Some(Cursor { value: full() }),
            }))),
        },
    );
}
fn fill_spend(d: &mut Domain, read: Token, doc: &Project) {
    let period =
        SpendPeriod { period: u64::MAX, budget: doc.spend, pools: doc.pools.clone(), tasks: doc.spend_lines.clone() };
    drive(
        d,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Spend(Box::new(SpendPage {
                rows: list([period.clone(), period]),
                before: Some(Cursor { value: full() }),
                after: Some(Cursor { value: full() }),
            }))),
        },
    );
}

fn specification(words: Text) -> SpecificationDocument {
    SpecificationDocument::complete(Specification {
        words,
        parameters: List::with_capacity(0),
        inputs: List::with_capacity(0),
    })
}
