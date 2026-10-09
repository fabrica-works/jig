//! Page policies exercised through the client boundary.
use super::spine::{LIMITS, delivered, drive, frame, open_token, start};
use crate::{
    Domain, Event, Request,
    address::{Address, Place, ProjectSection},
    contract::*,
    link::Link,
    pages::{
        Page,
        navigation::{Destination, Navigation},
    },
};
use skein_lib::{List, Time};
fn text(value: &[u8]) -> Text {
    Text::new(Box::from(value), LIMITS.text).unwrap()
}
pub(super) fn project(number: u32, chats: bool) -> ProjectRole {
    let mut charters = List::with_capacity(1);
    if chats {
        charters.push(Charter { number: 1, name: text(b"guide") }).unwrap();
    }
    ProjectRole {
        project: number,
        name: text(b"shop"),
        roles: List::with_capacity(0),
        chats: charters,
        chat_budget: 10,
        goals: List::with_capacity(0),
        pool: Budget { total: 20, own: 0, ended: 0, reserved: 0, left: 20, funder: Actor::Party(7) },
    }
}
fn the_frame_counts_needs_exactly_and_caps_updates_on_every_page() {
    let mut domain = Domain::new(&LIMITS, 1);
    let f = start(&mut domain, List::with_capacity(0));
    let mut value = frame(7);
    value.needs = Needs { questions: u32::MAX, proposals: 2, escalations: 3, person_tasks: 4 };
    value.updates = Count { value: 100, limited: false };
    delivered(&mut domain, Time::ZERO, f, value.clone());
    let summary = domain.frame_summary();
    assert_eq!(summary.needs, u64::from(u32::MAX) + 9);
    assert_eq!(summary.updates, Count { value: 99, limited: true });
    assert_eq!(summary.party, Some(7));
    assert_eq!(summary.link, Link::Live);
    drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
    assert_eq!(domain.frame_summary().needs, summary.needs);
    value.updates = Count { value: 99, limited: false };
    delivered(&mut domain, Time::ZERO, f, value.clone());
    assert_eq!(domain.frame_summary().updates, Count { value: 99, limited: false });
    value.updates.limited = true;
    delivered(&mut domain, Time::ZERO, f, value);
    assert!(domain.frame_summary().updates.limited);
    drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Project { project: 1, section: ProjectSection::Overview }) },
    );
    assert_eq!(domain.frame_summary().project, Some(1));
    assert_eq!(Address::task(42), Address::Task { task: 42, section: crate::address::TaskSection::Overview });
}
fn navigation_keeps_the_application_order_and_leaves_out_inapplicable_pages() {
    let mut domain = Domain::new(&LIMITS, 2);
    let mut navigation = Navigation::new(8);
    assert_eq!(navigation.home, Destination::Jig(Address::Inbox { project: None }));
    let own = Destination::Application { owner: 1, number: 3 };
    let chat = Destination::Jig(Address::Conversations { project: None });
    let deployment = Destination::Jig(Address::Deployment);
    navigation.home = own;
    for destination in [own, chat, deployment] {
        navigation.entries.push(destination).unwrap();
    }
    domain.configure_navigation(navigation).unwrap();
    let f = start(&mut domain, List::with_capacity(0));
    let mut value = frame(7);
    value.deployment_role = None;
    delivered(&mut domain, Time::ZERO, f, value.clone());
    assert_eq!(domain.navigation().entries.as_slice(), &[own, chat, deployment]);
    assert!(domain.navigation().offered(own, domain.frame().document.as_ref()));
    assert!(!domain.navigation().offered(chat, domain.frame().document.as_ref()));
    assert!(!domain.navigation().offered(deployment, domain.frame().document.as_ref()));
    value.projects = List::with_capacity(2);
    value.projects.push(project(1, false)).unwrap();
    value.projects.push(project(2, true)).unwrap();
    value.deployment_role = Some(1);
    delivered(&mut domain, Time::ZERO, f, value);
    assert!(domain.navigation().offered(chat, domain.frame().document.as_ref()));
    assert!(
        !domain
            .navigation()
            .offered(Destination::Jig(Address::Conversations { project: Some(1) }), domain.frame().document.as_ref())
    );
    assert!(domain.navigation().offered(deployment, domain.frame().document.as_ref()));
    assert!(domain.configure_navigation(Navigation::new(0)).is_err());
    let mut fresh = Domain::new(&LIMITS, 3);
    assert!(fresh.configure_navigation(Navigation::new(9)).is_err());
    assert_eq!(own.place(), Place::Elsewhere);
}
fn non_work_pages_follow_their_causes_and_missing_tasks_do_not_reveal_visibility() {
    let mut domain = Domain::new(&LIMITS, 4);
    let f = start(&mut domain, List::with_capacity(0));
    assert_eq!(domain.page(), &Page::Starting);
    delivered(&mut domain, Time::ZERO, f, frame(7));
    drive(&mut domain, Time::ZERO, Event::Went { place: Place::Missing });
    assert_eq!(domain.page(), &Page::Missing);
    drive(&mut domain, Time::ZERO, Event::SignedOut);
    assert_eq!(domain.page(), &Page::SigningIn);
    for end in [WatchEnd::Unknown, WatchEnd::NotAllowed] {
        let mut domain = Domain::new(&LIMITS, 5);
        let f = start(&mut domain, List::with_capacity(0));
        delivered(&mut domain, Time::ZERO, f, frame(7));
        let opened = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::task(42)) });
        drive(&mut domain, Time::ZERO, Event::Ended { watch: open_token(&opened), end });
        assert_eq!(domain.page(), &Page::Missing);
    }
    let mut old = Domain::new(&LIMITS, 6);
    let f = start(&mut old, List::with_capacity(0));
    let mut value = frame(7);
    value.versions = VersionRange { first: 2, last: 2 };
    delivered(&mut old, Time::ZERO, f, value);
    assert_eq!(old.page(), &Page::OutOfDate);
    let mut other = Domain::new(&LIMITS, 7);
    let mut kept = List::with_capacity(1);
    kept.push(crate::store::Record::Draft {
        version: crate::store::STORE_VERSION,
        party: 8,
        task: 42,
        words: text(b"secret"),
        touched: 1,
    })
    .unwrap();
    let f = start(&mut other, kept);
    delivered(&mut other, Time::ZERO, f, frame(7));
    assert_eq!(other.page(), &Page::SomeoneElse);
}
fn deployment_without_a_role_is_missing_and_never_watched() {
    let mut domain = Domain::new(&LIMITS, 8);
    let f = start(&mut domain, List::with_capacity(0));
    let mut value = frame(7);
    value.deployment_role = None;
    delivered(&mut domain, Time::ZERO, f, value);
    let out = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Deployment) });
    assert_eq!(domain.page(), &Page::Missing);
    for request in &out {
        if let Request::Open { subject, .. } = request {
            assert_ne!(*subject, Subject::Deployment);
        }
    }
}

fn entry(task: u64, group: InboxGroup, at: u64) -> InboxEntry {
    InboxEntry {
        reply: None,
        object: ObjectKey::Task(task),
        project: 1,
        group,
        position: task,
        at: skein_lib::Wall::from_nanos(at),
        lineage: List::with_capacity(0),
        goal: Some(9),
        recipient: Some(Recipient::Role { project: 1, role: 2 }),
        stall: Some(skein_lib::Wall::from_nanos(100)),
    }
}
fn inbox(entries: List<InboxEntry>) -> Inbox {
    Inbox {
        entries,
        needs: Needs { questions: 0, proposals: 2, escalations: 0, person_tasks: 0 },
        updates: Count { value: 100, limited: true },
        results_seen: 0,
        replies_seen: List::with_capacity(0),
        more: false,
    }
}
fn live_inbox(seed: u64) -> (Domain, skein_lib::Token) {
    let mut domain = Domain::new(&LIMITS, seed);
    let f = start(&mut domain, List::with_capacity(0));
    let page = open_token(&delivered(&mut domain, Time::ZERO, f, frame(7)));
    (domain, page)
}
fn inbox_delivery(domain: &mut Domain, page: skein_lib::Token, entries: List<InboxEntry>) -> Box<[Request]> {
    drive(
        domain,
        Time::ZERO,
        Event::Watch { watch: page, event: WatchEvent::Delivered(Delivery::Inbox(Box::new(inbox(entries)))) },
    )
}
fn list_entry(task: u64, phase: Phase, changed: u64) -> ConversationLine {
    ConversationLine {
        chip: Chip {
            task,
            title: text(b"held checkout"),
            executor: Executor::Agent { charter: 1 },
            phase,
            details: Box::new(super::objects::phase_details(phase)),
        },
        project: 1,
        first_words: text(b"help checkout"),
        unread: Count { value: 1, limited: false },
        seen: 2,
        last_reply: 3,
        changed: skein_lib::Wall::from_nanos(changed),
        runtime: Runtime::WaitingForWords { attempt: 1, since: skein_lib::Wall::EPOCH },
        delegates: 2,
        budget: Budget { total: 20, own: 5, ended: 1, reserved: 2, left: 12, funder: Actor::Party(7) },
    }
}
fn inbox_orders_both_groups_filters_context_and_removes_role_entries_for_every_holder() {
    for group in [InboxGroup::Proposals, InboxGroup::Results] {
        let (mut domain, page) = live_inbox(20);
        let mut rows = List::with_capacity(2);
        rows.push(entry(8, group, 20)).unwrap();
        rows.push(entry(7, group, 10)).unwrap();
        inbox_delivery(&mut domain, page, rows);
        let first = if crate::pages::inbox::needs(group) { 7 } else { 8 };
        assert_eq!(domain.inbox().rows.get(0).unwrap().object, ObjectKey::Task(first));
        assert_eq!(domain.inbox().rows.get(0).unwrap().recipient, Some(Recipient::Role { project: 1, role: 2 }));
        assert_eq!(domain.inbox().updates, Count { value: 100, limited: true });
        assert_eq!(crate::pages::inbox::address(ObjectKey::Task(first)), Address::task(first));
        for filter in [
            crate::pages::inbox::Filter { project: Some(2), kind: None, goal: None },
            crate::pages::inbox::Filter { project: None, kind: Some(InboxGroup::Questions), goal: None },
            crate::pages::inbox::Filter { project: None, kind: None, goal: Some(10) },
        ] {
            let out = drive(&mut domain, Time::ZERO, Event::Act { act: crate::action::Act::FilterInbox { filter } });
            assert!(domain.inbox().rows.is_empty());
            for request in &out {
                if let Request::Read { what, .. } = request {
                    assert_eq!(
                        *what,
                        Read::Inbox { project: filter.project, kind: filter.kind, goal: filter.goal, after: None }
                    );
                }
            }
        }
    }
    for party in [7, 8] {
        let mut domain = Domain::new(&LIMITS, party);
        let f = start(&mut domain, List::with_capacity(0));
        let page = open_token(&delivered(&mut domain, Time::ZERO, f, frame(party)));
        let mut rows = List::with_capacity(1);
        rows.push(entry(7, InboxGroup::Proposals, 10)).unwrap();
        inbox_delivery(&mut domain, page, rows);
        inbox_delivery(&mut domain, page, List::with_capacity(0));
        assert!(domain.inbox().rows.is_empty());
    }
}
fn opening_updates_sends_one_mark_seen_for_the_newest_result_displayed() {
    let (mut domain, page) = live_inbox(21);
    drive(&mut domain, Time::ZERO, Event::Act { act: crate::action::Act::OpenUpdates });
    let mut rows = List::with_capacity(2);
    rows.push(entry(7, InboxGroup::Results, 10)).unwrap();
    rows.push(entry(8, InboxGroup::Results, 20)).unwrap();
    let out = inbox_delivery(&mut domain, page, rows.clone());
    let mut count = 0_u32;
    for request in &out {
        if let Request::Send { ask, .. } = request {
            assert_eq!(**ask, Ask::MarkSeen { seen: Seen::Results { through: 8 } });
            count = count.checked_add(1).expect("bounded emitted requests");
        }
    }
    assert_eq!(count, 1);
    let out = inbox_delivery(&mut domain, page, rows);
    for request in &out {
        if let Request::Send { .. } = request {
            panic!("passive delivery does not mark read again");
        }
    }
    assert!(drive(&mut domain, Time::ZERO, Event::Act { act: crate::action::Act::OpenUpdates }).is_empty());
}
fn conversations_show_live_before_ended_and_read_more_using_the_opaque_cursor() {
    let (mut domain, _) = live_inbox(22);
    let entered =
        drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Conversations { project: None }) });
    let mut read = None;
    for request in &entered {
        if let Request::Read { read: token, what: Read::Conversations { .. } } = request {
            read = Some(*token);
        }
    }
    let mut rows = List::with_capacity(2);
    rows.push(list_entry(7, Phase::Done, 30)).unwrap();
    rows.push(list_entry(8, Phase::Idle, 10)).unwrap();
    let cursor = Cursor { value: text(b"next") };
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read {
            read: read.unwrap(),
            result: ReadResult::Value(ReadValue::Conversations(Box::new(ConversationPage {
                rows,
                before: None,
                after: Some(cursor.clone()),
            }))),
        },
    );
    assert_eq!(domain.conversation_rows().rows.get(0).unwrap().chip.task, 8);
    assert_eq!(domain.conversation_rows().rows.get(0).unwrap().delegates, 2);
    assert_eq!(domain.conversation_rows().rows.get(0).unwrap().budget.own, 5);
    domain.reclaim();
    let out = drive(
        &mut domain,
        Time::ZERO,
        Event::Act {
            act: crate::action::Act::More {
                window: skein_lib::Token::new(crate::pages::conversations::CONVERSATIONS_WINDOW),
                end: crate::action::End::After,
            },
        },
    );
    let mut found = false;
    for request in &out {
        if let Request::Read { what, .. } = request {
            assert_eq!(*what, Read::Conversations { project: None, after: Some(cursor.clone()) });
            found = true;
        }
    }
    assert!(found);
    let out = drive(&mut domain, Time::ZERO, Event::Act { act: crate::action::Act::OpenChat { task: 8 } });
    let mut mark = false;
    for request in &out {
        if let Request::Send { ask, .. } = request {
            assert_eq!(**ask, Ask::MarkSeen { seen: Seen::Replies { chat: 8, through: 3 } });
            mark = true;
        }
    }
    assert!(mark);
}
fn starting_a_chat_joins_refuses_and_opens_only_after_its_durable_answer() {
    let mut about = Domain::new(&LIMITS, 24);
    let f = start(&mut about, List::with_capacity(0));
    let mut value = frame(7);
    value.projects = List::with_capacity(1);
    value.projects.push(project(1, true)).unwrap();
    let page = open_token(&delivered(&mut about, Time::ZERO, f, value));
    let mut rows = List::with_capacity(1);
    rows.push(entry(8, InboxGroup::Escalations, 10)).unwrap();
    inbox_delivery(&mut about, page, rows);
    assert!(about.objects.put(ObjectKey::Task(8), crate::objects::Body::Chip(list_entry(8, Phase::Held, 10).chip)));
    let mut out = skein_lib::Queue::with_capacity(crate::max_out(&LIMITS));
    crate::pages::conversations::about(&mut about, &super::spine::env(Time::ZERO), ObjectKey::Task(8), &mut out);
    let requests = super::spine::drain(out);
    assert_eq!(requests.len(), 1);
    let Request::Send { ask, .. } = &requests[0] else {
        panic!("chat about the held task");
    };
    assert_eq!(**ask, Ask::StartChat { project: 1, words: text(b"held checkout") });
    let mut domain = Domain::new(&LIMITS, 23);
    let f = start(&mut domain, List::with_capacity(0));
    let mut value = frame(7);
    value.projects = List::with_capacity(1);
    value.projects.push(project(1, true)).unwrap();
    delivered(&mut domain, Time::ZERO, f, value);
    drive(
        &mut domain,
        Time::ZERO,
        Event::Act { act: crate::action::Act::Edit { field: crate::fields::Purpose::NewChat, text: text(b"help") } },
    );
    for refused in [true, false] {
        let out = drive(&mut domain, Time::ZERO, Event::Act { act: crate::action::Act::StartChat { project: 1 } });
        let mut token = None;
        for request in &out {
            if let Request::Send { request: sent, ask, .. } = request {
                assert_eq!(**ask, Ask::StartChat { project: 1, words: text(b"help") });
                token = Some(*sent);
            }
        }
        assert!(
            drive(&mut domain, Time::ZERO, Event::Act { act: crate::action::Act::StartChat { project: 1 } }).is_empty()
        );
        assert_eq!(
            domain.page(),
            &Page::Work {
                address: Address::Inbox { project: None },
                generation: 0,
                behind: false,
                excess: 0,
                refused: None
            }
        );
        let answer = if refused {
            Answer::Refused(Refusal::Authority { lacking: Lacking::Budget { needed: 10, left: 2 }, proposable: None })
        } else {
            Answer::Done(Outcome::Made { what: Made::Chat { task: 42 } })
        };
        drive(&mut domain, Time::ZERO, Event::Answered { request: token.unwrap(), answer });
        if refused {
            assert!(domain.chats().refusal.is_some());
        } else {
            let Page::Work { address, .. } = domain.page() else {
                panic!("chat opened");
            };
            assert_eq!(*address, Address::Task { task: 42, section: crate::address::TaskSection::Activity });
            assert!(domain.fields().get(crate::fields::Purpose::NewChat).unwrap().text.bytes().is_empty());
        }
        domain.reclaim();
    }
}

fn task_sections_and_activity_sources_follow_the_executor() {
    use crate::{address::TaskSection, pages::task};
    for executor in [
        Executor::Agent { charter: 1 },
        Executor::Procedure { connector: 2, kind: 3 },
        Executor::Party { party: 7 },
        Executor::Role { project: 1, role: 1 },
    ] {
        let mut document = super::confirmation::task();
        document.chip.executor = executor;
        let sections = task::sections(&document);
        for section in [TaskSection::Overview, TaskSection::Activity, TaskSection::History, TaskSection::Budget] {
            assert!(task::offered(&sections, section));
        }
        assert!(!task::offered(&sections, TaskSection::Plan));
        let mut domain = Domain::new(&LIMITS, 22);
        let f = start(&mut domain, List::with_capacity(0));
        delivered(&mut domain, Time::ZERO, f, frame(7));
        let opened = drive(
            &mut domain,
            Time::ZERO,
            Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Activity }) },
        );
        let watch = open_token(&opened);
        let requests = drive(
            &mut domain,
            Time::ZERO,
            Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(document))) },
        );
        assert!(domain.task_page().header.is_some());
        match executor {
            Executor::Agent { .. } => {
                assert_eq!(requests.len(), 2);
                let Request::Open { subject: Subject::Run { task: 8, turn: None, offset: 0 }, .. } = requests[0] else {
                    panic!("agent run watch");
                };
                let Request::Read { what: Read::Activity { task: 8, before: None }, .. } = requests[1] else {
                    panic!("agent activity read");
                };
            }
            Executor::Procedure { connector, .. } => {
                assert_eq!(
                    requests.as_ref(),
                    &[Request::ShowPart { part: Part::Panel { connector, task: 8 }, generation: 1 }]
                );
            }
            Executor::Party { .. } | Executor::Role { .. } => {
                let Request::Read { what: Read::PersonTask { task: 8 }, .. } = requests[0] else {
                    panic!("person task card read");
                };
                assert_eq!(requests.len(), 1);
            }
        }
    }
}
fn tree_node(number: u64, phase: Phase, held: u32) -> TreeNode {
    let mut chip = super::objects::chip(number);
    chip.phase = phase;
    *chip.details = super::objects::phase_details(phase);
    TreeNode {
        chip,
        parent: Some(8),
        dependencies: List::with_capacity(0),
        proposals: List::with_capacity(0),
        spend: 0,
        folded: empty_phase_counts(),
        held_below: held,
    }
}
fn task_plan_keeps_folded_held_counts_and_filters_without_losing_rows() {
    use crate::pages::task::{self, PlanFilter};
    let mut limits = LIMITS;
    limits.rows = 2;
    let mut domain = Domain::new(&limits, 22);
    let mut nodes = List::with_capacity(3);
    nodes.push(tree_node(9, Phase::Done, 3)).unwrap();
    nodes.push(tree_node(10, Phase::Running, 0)).unwrap();
    nodes.push(tree_node(11, Phase::Held, 0)).unwrap();
    task::tree(&mut domain, Tree { task: 8, nodes, folded: empty_phase_counts(), held_below: 4 });
    let plan = &domain.task_page().plan;
    assert_eq!(plan.nodes.len(), 2);
    assert_eq!(plan.folded.held, 1);
    assert_eq!(plan.held_below, 4);
    task::filter(&mut domain, PlanFilter::Held);
    let plan = &domain.task_page().plan;
    assert!(plan.visible(&plan.nodes.as_slice()[0]));
    assert!(!plan.visible(&plan.nodes.as_slice()[1]));
    task::filter(&mut domain, PlanFilter::Live);
    let plan = &domain.task_page().plan;
    assert!(!plan.visible(&plan.nodes.as_slice()[0]));
    assert!(plan.visible(&plan.nodes.as_slice()[1]));
    task::filter(&mut domain, PlanFilter::All);
    assert_eq!(domain.task_page().plan.nodes.len(), 2);
    let mut domain = Domain::new(&LIMITS, 22);
    let f = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    let opened = drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: crate::address::TaskSection::Plan }) },
    );
    let watch = open_token(&opened);
    let tree_open = drive(
        &mut domain,
        Time::ZERO,
        Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(super::confirmation::task()))) },
    );
    let tree_watch = open_token(&tree_open);
    let mut nodes = List::with_capacity(1);
    nodes.push(tree_node(9, Phase::Done, 3)).unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        Event::Watch {
            watch: tree_watch,
            event: WatchEvent::Delivered(Delivery::Tree(Box::new(Tree {
                task: 8,
                nodes,
                folded: empty_phase_counts(),
                held_below: 3,
            }))),
        },
    );
    let asked = drive(&mut domain, Time::ZERO, Event::Act { act: crate::action::Act::OpenEnded { task: 9 } });
    let Request::Read { read, what: Read::Ended { task: 9, after: None } } = asked[0] else {
        panic!("ended subtree read");
    };
    let cursor = Cursor { value: text(b"opaque subtree") };
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Ended(Box::new(ObjectPage {
                rows: List::with_capacity(0),
                before: None,
                after: Some(cursor.clone()),
            }))),
        },
    );
    let more = drive(
        &mut domain,
        Time::ZERO,
        Event::Act {
            act: crate::action::Act::More {
                window: skein_lib::Token::new(task::ENDED_WINDOW),
                end: crate::action::End::After,
            },
        },
    );
    let Request::Read { what, .. } = &more[0] else {
        panic!("ended subtree continuation");
    };
    assert_eq!(*what, Read::Ended { task: 9, after: Some(cursor) });
}
fn task_history_pages_and_scope_changes_discard_old_scope_reads() {
    use crate::{
        action::{Act, End},
        address::TaskSection,
        pages::task::HISTORY_WINDOW,
    };
    let mut domain = Domain::new(&LIMITS, 22);
    let f = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    let opened = drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::History }) },
    );
    let Request::Read { read: own, .. } = opened[3] else {
        panic!("own history read");
    };
    let requests = drive(&mut domain, Time::ZERO, Event::Act { act: Act::HistoryScope { subtree: true } });
    let Request::Read { read, what: Read::History { subtree: true, .. } } = requests[0] else {
        panic!("subtree read");
    };
    let cursor = Cursor { value: text(b"opaque history edge") };
    let mut rows = List::with_capacity(1);
    rows.push(HistoryLine {
        position: 1,
        task: 9,
        by: Actor::Party(7),
        at: skein_lib::Wall::EPOCH,
        change: HistoryChange::Released,
    })
    .unwrap();
    let page = HistoryPage { rows, before: None, after: Some(cursor.clone()) };
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read: own, result: ReadResult::Value(ReadValue::History(Box::new(page.clone()))) },
    );
    assert!(domain.history_rows().rows.is_empty());
    drive(&mut domain, Time::ZERO, Event::Read { read, result: ReadResult::Value(ReadValue::History(Box::new(page))) });
    assert_eq!(domain.history_rows().rows.len(), 1);
    let more = drive(
        &mut domain,
        Time::ZERO,
        Event::Act { act: Act::More { window: skein_lib::Token::new(HISTORY_WINDOW), end: End::After } },
    );
    let Request::Read { what, .. } = &more[0] else {
        panic!("next history page");
    };
    assert_eq!(*what, Read::History { task: 8, subtree: true, after: Some(cursor) });
}
fn task_parts_follow_the_section_and_leave_with_the_page_generation() {
    use crate::address::TaskSection;
    let mut domain = Domain::new(&LIMITS, 22);
    let f = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    let opened = drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Overview }) },
    );
    let watch = open_token(&opened);
    let mut document = super::confirmation::task();
    document.resources = List::with_capacity(1);
    let resource = Part::Resource { connector: 2, path: text(b"resource") };
    document.resources.push(resource.clone()).unwrap();
    document.parts = List::with_capacity(1);
    document.parts.push(Part::Panel { connector: 3, task: 8 }).unwrap();
    let requests = drive(
        &mut domain,
        Time::ZERO,
        Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(document))) },
    );
    assert_eq!(requests.as_ref(), &[Request::ShowPart { part: resource, generation: 1 }]);
    let left = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
    assert!(left.contains(&Request::PartsLeft { generation: 1 }));
    assert!(domain.task_page().header.is_none());
}

fn empty_phase_counts() -> PhaseCounts {
    PhaseCounts { waiting: 0, running: 0, held: 0, closing: 0, done: 0, failed: 0, cancelled: 0 }
}

fn project_document() -> Project {
    let mut acts = List::with_capacity(1);
    let mut charters = List::with_capacity(1);
    charters.push(1).unwrap();
    acts.push(Offered {
        act: ActKind::SetGoal,
        admission: Admission::Bounded {
            within: Bounds::Goal {
                pool_left: 20,
                charters,
                latest: None,
                budget: 20,
                maximum_budget: 100,
                maximum_deadline: None,
            },
            beyond: Beyond::Proposed { to: Recipient::Party(9) },
        },
    })
    .unwrap();
    Project {
        number: 1,
        period: 1,
        spend_lines: List::with_capacity(0),
        name: text(b"shop"),
        policy: Policy {
            ceiling: super::objects::authority(),
            period: 1,
            spend: 0,
            roles: List::with_capacity(0),
            requirements: List::with_capacity(0),
            deciders: List::with_capacity(0),
        },
        roles: List::with_capacity(0),
        spend: project(1, false).pool,
        pools: List::with_capacity(0),
        resources: List::with_capacity(0),
        services: List::with_capacity(0),
        recurring: List::with_capacity(0),
        parts: List::with_capacity(0),
        acts,
    }
}
fn project_open(section: ProjectSection) -> (Domain, Box<[Request]>) {
    let mut domain = Domain::new(&LIMITS, 22);
    let f = start(&mut domain, List::with_capacity(0));
    let mut value = frame(7);
    let mut role = project(1, true);
    role.goals = List::with_capacity(1);
    role.goals.push(Charter { number: 1, name: text(b"lead") }).unwrap();
    value.projects = List::with_capacity(1);
    value.projects.push(role).unwrap();
    delivered(&mut domain, Time::ZERO, f, value);
    let opened =
        drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Project { project: 1, section }) });
    (domain, opened)
}
fn project_read(requests: &[Request]) -> skein_lib::Token {
    for request in requests {
        if let Request::Read { read, what: Read::Project { project: 1 } } = request {
            return *read;
        }
    }
    panic!("project read")
}
fn goal_line(number: u64, priority: u32, phase: Phase, changed: u64) -> GoalLine {
    let mut chip = super::objects::chip(number);
    chip.phase = phase;
    *chip.details = super::objects::phase_details(phase);
    GoalLine {
        chip,
        priority,
        progress: empty_phase_counts(),
        budget: project(1, false).pool,
        deadline: None,
        held_below: 2,
        changed: skein_lib::Wall::from_nanos(changed),
    }
}
fn project_board_orders_live_waiting_and_ended_goals_and_names_task_cards() {
    use crate::{
        action::{Act, End},
        pages::project::GOALS_WINDOW,
    };
    let (mut domain, opened) = project_open(ProjectSection::Goals);
    let project_read = project_read(&opened);
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read: project_read, result: ReadResult::Value(ReadValue::Project(Box::new(project_document()))) },
    );
    let watch = open_token(&opened);
    let mut rows = List::with_capacity(2);
    rows.push(goal_line(9, 20, Phase::Waiting, 1)).unwrap();
    rows.push(goal_line(8, 1, Phase::Running, 2)).unwrap();
    let facts = drive(
        &mut domain,
        Time::ZERO,
        Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Goals(Box::new(Goals { project: 1, rows, more: true }))),
        },
    );
    assert_eq!(domain.project_page().goals.as_slice()[0].chip.task, 8);
    assert_eq!(domain.project_page().goals.as_slice()[1].chip.task, 9);
    assert!(domain.project_page().goals_more);
    assert!(domain.objects().find(ObjectKey::Task(8)).is_some());
    let Request::Read { read, what: Read::Task { task: 8 } } = facts[0] else {
        panic!("goal task facts read");
    };
    let mut task = super::confirmation::task();
    task.goal = true;
    task.priority = Some(1);
    task.acts = List::with_capacity(1);
    task.acts.push(Offered { act: ActKind::Prioritise, admission: Admission::Allowed }).unwrap();
    drive(&mut domain, Time::ZERO, Event::Read { read, result: ReadResult::Value(ReadValue::Task(Box::new(task))) });
    let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
    assert_eq!(crate::acts::offered(domain.objects().get(id).unwrap(), ActKind::Prioritise), Admission::Allowed);

    let mut rows = List::with_capacity(2);
    rows.push(goal_line(8, 1, Phase::Running, 2)).unwrap();
    rows.push(goal_line(9, 2, Phase::Running, 1)).unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Goals(Box::new(Goals { project: 1, rows, more: false }))),
        },
    );
    assert_eq!(domain.project_page().goals.as_slice()[0].chip.task, 9);
    let Request::Read { read, .. } = opened[3] else { panic!("ended goals read") };
    let mut rows = List::with_capacity(2);
    rows.push(goal_line(8, 1, Phase::Done, 1)).unwrap();
    rows.push(goal_line(9, 2, Phase::Done, 2)).unwrap();
    let cursor = Cursor { value: text(b"ended goals edge") };
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Goals(Box::new(GoalPage {
                rows,
                before: None,
                after: Some(cursor.clone()),
            }))),
        },
    );
    assert_eq!(domain.project_page().ended.as_slice()[0].chip.task, 9);
    domain.reclaim();
    let more = drive(
        &mut domain,
        Time::ZERO,
        Event::Act { act: Act::More { window: skein_lib::Token::new(GOALS_WINDOW), end: End::After } },
    );
    let Request::Read { what, .. } = &more[0] else { panic!("ended goals continuation") };
    assert_eq!(*what, Read::Goals { project: 1, ended: true, after: Some(cursor) });
    project_task_filters_and_spend_pages();
}
fn a_goal_past_the_pool_is_confirmed_as_proposed_and_sent_once() {
    use crate::{action::Act, fields::Purpose, pages::project::goals::GoalForm};
    let (mut domain, opened) = project_open(ProjectSection::Goals);
    let read = project_read(&opened);
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read, result: ReadResult::Value(ReadValue::Project(Box::new(project_document()))) },
    );
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Edit { field: Purpose::NewGoal, text: text(b"new goal") } });
    let form = GoalForm { project: 1, charter: 1, budget: 21, deadline: None, priority: 2 };
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::GoalForm { form } });
    let opened = drive(&mut domain, Time::ZERO, Event::Act { act: Act::Submit { field: Purpose::NewGoal } });
    assert!(opened.is_empty());
    let confirmation = domain.project_page().form.confirmation.as_ref().unwrap();
    assert!(confirmation.proposed);
    assert_eq!(confirmation.pool_left, 20);
    assert!(domain.confirmation().is_none());
    let mut task = super::confirmation::task();
    task.acts = List::with_capacity(1);
    task.acts.push(Offered { act: ActKind::Write, admission: Admission::Allowed }).unwrap();
    let id =
        domain.objects.name(ObjectKey::Task(8), crate::objects::Source::Window { generation: 1, number: 99 }).unwrap();
    assert!(domain.objects.put(ObjectKey::Task(8), crate::objects::Body::Task(Box::new(task))));
    crate::confirm::open(&mut domain, &super::spine::env(Time::ZERO), id, ActKind::Write);
    assert!(domain.project_page().form.confirmation.is_none());
    assert!(domain.confirmation().is_some());
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Submit { field: Purpose::NewGoal } });
    assert!(domain.project_page().form.confirmation.is_some());
    assert!(domain.confirmation().is_none());
    let sent = drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm });
    let Request::Send { request, ask, .. } = &sent[0] else { panic!("one goal intent") };
    assert_eq!(
        **ask,
        Ask::SetGoal {
            project: 1,
            spec: text(b"new goal"),
            executor: Executor::Agent { charter: 1 },
            budget: 21,
            priority: 2,
            deadline: None
        }
    );
    assert_eq!(sent.len(), 1);
    assert!(drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm }).is_empty());
    let request = *request;
    drive(&mut domain, Time::ZERO, Event::Answered { request, answer: Answer::Refused(Refusal::NotOffered) });
    assert_eq!(domain.project_page().form.refusal, Some(Refusal::NotOffered));
    assert_eq!(domain.fields().get(Purpose::NewGoal).unwrap().text, text(b"new goal"));
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Submit { field: Purpose::NewGoal } });
    let sent = drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm });
    let Request::Send { request, .. } = sent[0] else { panic!("retry goal") };
    drive(
        &mut domain,
        Time::ZERO,
        Event::Act { act: Act::Edit { field: Purpose::NewGoal, text: text(b"later words") } },
    );
    drive(
        &mut domain,
        Time::ZERO,
        Event::Answered { request, answer: Answer::Done(Outcome::Made { what: Made::Goal { task: 42 } }) },
    );
    assert_eq!(domain.address, Address::task(42));
    assert_eq!(domain.fields().get(Purpose::NewGoal).unwrap().text, text(b"later words"));
}
#[test]
fn bounded_goal_facts_control_charter_deadline_budget_and_refusal_without_frame_inference() {
    use crate::{action::Act, fields::Purpose, pages::project::goals::GoalForm};
    use skein_lib::Wall;
    let (mut domain, opened) = project_open(ProjectSection::Goals);
    let read = project_read(&opened);
    let mut doc = project_document();
    let latest = Wall::from_nanos(100);
    doc.acts.get_mut(0).unwrap().admission = Admission::Bounded {
        within: Bounds::Goal {
            pool_left: 20,
            charters: {
                let mut rows = List::with_capacity(1);
                rows.push(99).unwrap();
                rows
            },
            latest: Some(latest),
            budget: 3,
            maximum_budget: 100,
            maximum_deadline: None,
        },
        beyond: Beyond::Refused(Refusal::Role { project: 1, role: 2 }),
    };
    drive(&mut domain, Time::ZERO, Event::Read { read, result: ReadResult::Value(ReadValue::Project(Box::new(doc))) });
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Edit { field: Purpose::NewGoal, text: text(b"bounded") } });
    for (charter, budget, deadline, allowed) in [
        (99, 3, Some(latest), true),
        (99, 4, Some(latest), false),
        (99, 101, Some(latest), false),
        (1, 3, Some(latest), false),
        (99, 3, Some(Wall::from_nanos(101)), false),
        (99, 3, None, false),
    ] {
        let form = GoalForm { project: 1, charter, budget, deadline, priority: 1 };
        drive(&mut domain, Time::ZERO, Event::Act { act: Act::GoalForm { form } });
        drive(&mut domain, Time::ZERO, Event::Act { act: Act::Submit { field: Purpose::NewGoal } });
        assert_eq!(domain.project_page().form.confirmation.is_some(), allowed);
        if allowed {
            let open = domain.project_page().form.confirmation.as_ref().unwrap();
            assert_eq!(open.pool_left, 20);
            assert!(!open.proposed);
        } else {
            assert_eq!(
                domain.project_page().form.refusal,
                Some(if charter == 1 || budget > 100 {
                    Refusal::NotOffered
                } else {
                    Refusal::Role { project: 1, role: 2 }
                })
            );
            assert!(drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm }).is_empty());
        }
    }
    let form = GoalForm { project: 1, charter: 99, budget: 3, deadline: Some(latest), priority: 1 };
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::GoalForm { form } });
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Submit { field: Purpose::NewGoal } });
    assert!(domain.project_page().form.confirmation.is_some());
    let mut moved = domain.project_page().document.as_ref().unwrap().clone();
    match &mut moved.acts.get_mut(0).unwrap().admission {
        Admission::Bounded { within: Bounds::Goal { maximum_deadline, .. }, .. } => {
            *maximum_deadline = Some(Wall::from_nanos(99));
        }
        Admission::Allowed | Admission::Proposed { .. } | Admission::Bounded { .. } | Admission::Refused(_) => {
            panic!("bounded goal")
        }
    }
    let mut out = skein_lib::Queue::with_capacity(crate::max_out(&LIMITS));
    crate::pages::project::document(&mut domain, &super::spine::env(Time::ZERO), moved, &mut out);
    assert!(domain.project_page().form.confirmation.is_none());
    assert_eq!(domain.project_page().form.refusal, Some(Refusal::NotOffered));
    assert!(drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm }).is_empty());
    let mut refused = domain.project_page().document.as_ref().unwrap().clone();
    refused.acts.get_mut(0).unwrap().admission = Admission::Refused(Refusal::Role { project: 1, role: 2 });
    crate::pages::project::document(&mut domain, &super::spine::env(Time::ZERO), refused, &mut out);
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Submit { field: Purpose::NewGoal } });
    assert_eq!(domain.project_page().form.refusal, Some(Refusal::Role { project: 1, role: 2 }));
    let nobody = Refusal::Authority { lacking: Lacking::Approver, proposable: Some(false) };
    let mut refused = domain.project_page().document.as_ref().unwrap().clone();
    refused.acts.get_mut(0).unwrap().admission = Admission::Bounded {
        within: Bounds::Goal {
            pool_left: 20,
            charters: {
                let mut rows = List::with_capacity(1);
                rows.push(99).unwrap();
                rows
            },
            latest: Some(latest),
            budget: 3,
            maximum_budget: 100,
            maximum_deadline: None,
        },
        beyond: Beyond::Refused(nobody.clone()),
    };
    crate::pages::project::document(&mut domain, &super::spine::env(Time::ZERO), refused, &mut out);
    let form = GoalForm { project: 1, charter: 99, budget: 4, deadline: Some(latest), priority: 1 };
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::GoalForm { form } });
    let emitted = drive(&mut domain, Time::ZERO, Event::Act { act: Act::Submit { field: Purpose::NewGoal } });
    assert!(emitted.is_empty());
    assert_eq!(domain.project_page().form.refusal, Some(nobody));
    assert!(domain.project_page().form.confirmation.is_none());
    assert!(drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm }).is_empty());
}

fn project_notes_keep_revisions_and_settings_deployment_keep_facts_and_parts() {
    use crate::{action::Act, objects::Body};
    let (mut domain, opened) = project_open(ProjectSection::Notes);
    let Request::Read { read, .. } = opened[2] else { panic!("notes read") };
    let mut acts = List::with_capacity(2);
    acts.push(Offered { act: ActKind::EditNote, admission: Admission::Allowed }).unwrap();
    acts.push(Offered { act: ActKind::DeleteNote, admission: Admission::Allowed }).unwrap();
    let note = Note {
        number: 3,
        revision: Revision { value: 1 },
        scope: NoteScope::Project(1),
        description: text(b"description"),
        body: text(b"body"),
        by: Actor::Party(7),
        tasks: List::with_capacity(0),
        changed: skein_lib::Wall::EPOCH,
        acts,
    };
    let mut rows = List::with_capacity(1);
    rows.push(note.clone()).unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Notes(Box::new(NotePage { rows, before: None, after: None }))),
        },
    );
    let id = domain.project_page().notes.as_slice()[0];
    assert_eq!(crate::acts::offered(domain.objects().get(id).unwrap(), ActKind::EditNote), Admission::Allowed);
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Intend { object: id.token(), act: ActKind::DeleteNote } });
    assert_eq!(domain.confirmation().unwrap().revision, Some(Revision { value: 1 }));
    let sent = drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm });
    let Request::Send { request, .. } = sent[0] else { panic!("delete note request") };
    let refresh = drive(
        &mut domain,
        Time::ZERO,
        Event::Answered {
            request,
            answer: Answer::Refused(Refusal::Moved { now: Revision { value: 2 }, what: Moved::Note }),
        },
    );
    let Request::Read { read, .. } = refresh[0] else { panic!("moved note read") };
    let mut newer = note;
    newer.revision = Revision { value: 2 };
    newer.body = text(b"new body");
    let mut rows = List::with_capacity(1);
    rows.push(newer).unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Notes(Box::new(NotePage { rows, before: None, after: None }))),
        },
    );
    let Some(Body::Note { document, .. }) = &domain.objects().get(id).unwrap().body else { panic!("updated note") };
    assert_eq!(document.revision, Revision { value: 2 });
    assert_eq!(document.body, text(b"new body"));
    assert_eq!(domain.project_page().notes.as_slice()[0], id);
    domain.reclaim();
    let moved = drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Project { project: 1, section: ProjectSection::Settings }) },
    );
    let read = project_read(&moved);
    let mut doc = project_document();
    let resource = Part::Resource { connector: 1, path: text(b"resource") };
    doc.resources = List::with_capacity(1);
    doc.resources.push(resource.clone()).unwrap();
    let shown = drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read, result: ReadResult::Value(ReadValue::Project(Box::new(doc))) },
    );
    assert_eq!(shown.as_ref(), &[Request::ShowPart { part: resource, generation: 2 }]);
    assert_eq!(domain.project_page().document.as_ref().unwrap().period, 1);
    let opened = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Deployment) });
    let watch = open_token(&opened);
    let part = Part::Deployment { connector: 1 };
    let mut parts = List::with_capacity(1);
    parts.push(part.clone()).unwrap();
    let doc = Deployment {
        hosts: List::with_capacity(0),
        accounts: List::with_capacity(0),
        capacity: List::with_capacity(0),
        proposals: 3,
        refusals: List::with_capacity(0),
        parts,
    };
    let shown = drive(
        &mut domain,
        Time::ZERO,
        Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Deployment(Box::new(doc))) },
    );
    assert_eq!(shown.as_ref(), &[Request::ShowPart { part, generation: 3 }]);
    assert_eq!(domain.deployment_page().document.as_ref().unwrap().proposals, 3);
}

fn project_task_filters_and_spend_pages() {
    use crate::{
        action::{Act, End},
        pages::project::SPEND_WINDOW,
    };
    let (mut domain, opened) = project_open(ProjectSection::Tasks);
    let watch = open_token(&opened);
    let filter = domain.project_page().task_filter;
    let mut rows = List::with_capacity(2);
    rows.push(TaskLine {
        chip: tree_node(8, Phase::Running, 0).chip,
        goal: Some(10),
        requester: Actor::Party(7),
        asked_by_me: true,
        changed: skein_lib::Wall::from_nanos(1),
    })
    .unwrap();
    rows.push(TaskLine {
        chip: tree_node(9, Phase::Running, 0).chip,
        goal: Some(10),
        requester: Actor::Party(8),
        asked_by_me: false,
        changed: skein_lib::Wall::from_nanos(2),
    })
    .unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Tasks(Box::new(Tasks {
                project: 1,
                filter,
                rows: rows.clone(),
                more: true,
            }))),
        },
    );
    assert_eq!(domain.project_page().tasks.as_slice()[0].chip.task, 9);
    assert!(domain.project_page().tasks_more);
    let filter = TaskFilter {
        phase: Some(Phase::Running),
        executor: Some(Executor::Party { party: 7 }),
        goal: Some(10),
        asked_by_me: true,
    };
    let opened = drive(&mut domain, Time::ZERO, Event::Act { act: Act::FilterTasks { filter } });
    let watch = open_token(&opened);
    drive(
        &mut domain,
        Time::ZERO,
        Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Tasks(Box::new(Tasks { project: 1, filter, rows, more: false }))),
        },
    );
    assert_eq!(domain.project_page().tasks.len(), 1);
    assert!(!domain.project_page().tasks_more);
    assert_eq!(domain.project_page().tasks.as_slice()[0].chip.task, 8);
    let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
    assert_eq!(domain.objects().get(id).unwrap().project, Some(1));
    let (mut domain, opened) = project_open(ProjectSection::Spend);
    let Request::Read { read, what: Read::Spend { project: 1, .. } } = opened[2] else { panic!("past spend read") };
    let mut rows = List::with_capacity(1);
    rows.push(SpendPeriod {
        period: 1,
        budget: project(1, false).pool,
        pools: List::with_capacity(0),
        tasks: List::with_capacity(0),
    })
    .unwrap();
    let cursor = Cursor { value: text(b"past period edge") };
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Spend(Box::new(SpendPage {
                rows,
                before: None,
                after: Some(cursor.clone()),
            }))),
        },
    );
    assert_eq!(domain.project_page().spend.as_slice()[0].period, 1);
    let more = drive(
        &mut domain,
        Time::ZERO,
        Event::Act { act: Act::More { window: skein_lib::Token::new(SPEND_WINDOW), end: End::After } },
    );
    let Request::Read { what, .. } = &more[0] else { panic!("past spend continuation") };
    assert_eq!(*what, Read::Spend { project: 1, after: Some(cursor) });
}

// One process per page family keeps startup costs within this session's share;
// each scenario above still allocates its own state and retains every assertion.
#[test]
fn frame_navigation_and_non_work_pages_follow_owner_facts() {
    the_frame_counts_needs_exactly_and_caps_updates_on_every_page();
    navigation_keeps_the_application_order_and_leaves_out_inapplicable_pages();
    non_work_pages_follow_their_causes_and_missing_tasks_do_not_reveal_visibility();
    deployment_without_a_role_is_missing_and_never_watched();
}
#[test]
fn inbox_and_conversation_pages_preserve_context_order_and_request_lifetimes() {
    inbox_orders_both_groups_filters_context_and_removes_role_entries_for_every_holder();
    opening_updates_sends_one_mark_seen_for_the_newest_result_displayed();
    conversations_show_live_before_ended_and_read_more_using_the_opaque_cursor();
    starting_a_chat_joins_refuses_and_opens_only_after_its_durable_answer();
}
#[test]
fn task_pages_preserve_executor_sections_tree_history_and_part_lifetimes() {
    task_sections_and_activity_sources_follow_the_executor();
    task_plan_keeps_folded_held_counts_and_filters_without_losing_rows();
    task_history_pages_and_scope_changes_discard_old_scope_reads();
    task_parts_follow_the_section_and_leave_with_the_page_generation();
}
#[test]
fn project_and_deployment_pages_preserve_facts_confirmations_and_revision_checks() {
    project_board_orders_live_waiting_and_ended_goals_and_names_task_cards();
    a_goal_past_the_pool_is_confirmed_as_proposed_and_sent_once();
    project_notes_keep_revisions_and_settings_deployment_keep_facts_and_parts();
}

fn read_named(requests: &[Request], expected: &Read) -> skein_lib::Token {
    for request in requests {
        if let Request::Read { read, what } = request
            && what == expected
        {
            return *read;
        }
    }
    panic!("expected the named read");
}
fn reply_entry(task: u64, seen: u64, last: u64) -> InboxEntry {
    let mut row = entry(task, InboxGroup::Replies, last);
    let mut line = list_entry(task, Phase::Idle, last);
    line.seen = seen;
    line.last_reply = last;
    line.unread = Count { value: u32::try_from(last.saturating_sub(seen)).unwrap(), limited: false };
    row.reply = Some(Box::new(line));
    row
}
#[test]
fn inbox_reply_receipts_keep_whole_chat_facts_shared_task_documents_and_seen_requests() {
    use crate::{action::Act, objects::Body, pages::inbox::Filter};
    let (mut domain, page) = live_inbox(45);
    let mut rows = List::with_capacity(1);
    rows.push(reply_entry(8, 4, 7)).unwrap();
    let out = inbox_delivery(&mut domain, page, rows);
    let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
    let object = domain.objects().get(id).unwrap();
    assert_eq!(object.project, Some(1));
    assert_eq!(object.reply.as_ref().unwrap().first_words.bytes(), b"help checkout");
    assert_eq!(object.reply.as_ref().unwrap().first_unread, 5);
    let Some(Body::Chip(chip)) = &object.body else {
        panic!("whole committed chat chip");
    };
    assert_eq!(chip.title.bytes(), b"held checkout");
    assert_eq!(object.sources.len(), 1);
    assert!(domain.conversation_rows().rows.is_empty());

    let mut task = super::confirmation::task();
    task.chip.task = 8;
    task.chip.title = text(b"current task title");
    task.spec = text(b"whole task specification");
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read {
            read: read_named(&out, &Read::Task { task: 8 }),
            result: ReadResult::Value(ReadValue::Task(Box::new(task.clone()))),
        },
    );
    domain.reclaim();
    let filter = Filter { project: Some(1), kind: Some(InboxGroup::Replies), goal: None };
    let out = drive(&mut domain, Time::ZERO, Event::Act { act: Act::FilterInbox { filter } });
    let what = Read::Inbox { project: filter.project, kind: filter.kind, goal: filter.goal, after: None };
    let mut rows = List::with_capacity(1);
    rows.push(reply_entry(8, 6, 9)).unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read {
            read: read_named(&out, &what),
            result: ReadResult::Value(ReadValue::Inbox(Box::new(InboxPage {
                rows,
                before: Some(Cursor { value: text(b"back") }),
                after: None,
            }))),
        },
    );
    let object = domain.objects().get(id).unwrap();
    assert_eq!(object.reply.as_ref().unwrap().first_unread, 7);
    assert_eq!(object.sources.len(), 1, "watch and read share the displayed row owner");
    let Some(Body::Task(kept)) = &object.body else {
        panic!("task document retained");
    };
    assert_eq!(**kept, task, "reply receipts do not replace the task's facts");
    let out = drive(&mut domain, Time::ZERO, Event::Act { act: Act::OpenChat { task: 8 } });
    let mut marked = false;
    for request in &out {
        if let Request::Send { ask, .. } = request {
            assert_eq!(**ask, Ask::MarkSeen { seen: Seen::Replies { chat: 8, through: 9 } });
            marked = true;
        }
    }
    assert!(marked, "an inbox-only reply marks its committed last position");
    assert!(domain.objects().get(id).is_none(), "leaving drops unretained page objects");

    let (mut domain, page) = live_inbox(46);
    let mut bad = reply_entry(8, 1, 2);
    bad.reply.as_mut().unwrap().chip.task = 9;
    let mut rows = List::with_capacity(1);
    rows.push(bad).unwrap();
    inbox_delivery(&mut domain, page, rows);
    assert!(domain.objects().get(domain.objects().find(ObjectKey::Task(8)).unwrap()).unwrap().reply.is_none());
    assert!(domain.objects().find(ObjectKey::Task(9)).is_none(), "mismatched facts cannot name another chat");
}
fn live_conversation_page(seed: u64) -> (Domain, skein_lib::Token, skein_lib::Token) {
    let (mut domain, _) = live_inbox(seed);
    let entered =
        drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Conversations { project: None }) });
    let watch = open_token(&entered);
    let first_read = read_named(&entered, &Read::Conversations { project: None, after: None });
    (domain, watch, first_read)
}
#[test]
fn conversation_reply_windows_replace_sources_drop_rows_and_clear_committed_seen_facts() {
    use crate::{
        action::{Act, End},
        objects::{Body, Source},
    };
    let (mut domain, watch, first_read) = live_conversation_page(47);
    let mut rows = List::with_capacity(2);
    rows.push(list_entry(8, Phase::Idle, 10)).unwrap();
    rows.push(list_entry(9, Phase::Idle, 9)).unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Conversations(Box::new(Conversations {
                rows: rows.clone(),
                more: true,
            }))),
        },
    );
    let eight = domain.objects().find(ObjectKey::Task(8)).unwrap();
    let nine = domain.objects().find(ObjectKey::Task(9)).unwrap();
    assert_eq!(domain.objects().get(eight).unwrap().reply.as_ref().unwrap().first_unread, 3);
    assert!(domain.objects().get(eight).unwrap().sources.contains(&Source::Watch(skein_lib::Id::from_token(watch))));
    let mut task = super::confirmation::task();
    task.chip.task = 9;
    task.spec = text(b"retained full document");
    assert!(domain.objects.put(ObjectKey::Task(9), Body::Task(Box::new(task.clone()))));
    rows.get_mut(1).unwrap().seen = 3;
    rows.get_mut(1).unwrap().unread.value = 0;
    drive(
        &mut domain,
        Time::ZERO,
        Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Conversations(Box::new(Conversations {
                rows: rows.clone(),
                more: true,
            }))),
        },
    );
    assert!(domain.objects().get(nine).unwrap().reply.is_none(), "committed seen facts remove stale unread metadata");
    let cursor = Cursor { value: text(b"older") };
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read {
            read: first_read,
            result: ReadResult::Value(ReadValue::Conversations(Box::new(ConversationPage {
                rows,
                before: None,
                after: Some(cursor.clone()),
            }))),
        },
    );
    assert_eq!(domain.objects().get(eight).unwrap().sources.len(), 1);
    domain.reclaim();
    let out = drive(
        &mut domain,
        Time::ZERO,
        Event::Act {
            act: Act::More {
                window: skein_lib::Token::new(crate::pages::conversations::CONVERSATIONS_WINDOW),
                end: End::After,
            },
        },
    );
    let mut rows = List::with_capacity(1);
    rows.push(list_entry(10, Phase::Done, 8)).unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read {
            read: read_named(&out, &Read::Conversations { project: None, after: Some(cursor) }),
            result: ReadResult::Value(ReadValue::Conversations(Box::new(ConversationPage {
                rows,
                before: Some(Cursor { value: text(b"back") }),
                after: None,
            }))),
        },
    );
    assert!(domain.objects().find(ObjectKey::Task(8)).is_none(), "a dropped row has no obsolete watch name");
    assert_eq!(domain.objects().get(nine).unwrap().sources.len(), 1);
    let Some(Body::Task(kept)) = &domain.objects().get(nine).unwrap().body else {
        panic!("task retained");
    };
    assert_eq!(**kept, task);
    let ten = domain.objects().find(ObjectKey::Task(10)).unwrap();
    assert_eq!(domain.objects().get(ten).unwrap().reply.as_ref().unwrap().first_words.bytes(), b"help checkout");
    let out = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
    assert!(domain.objects().find(ObjectKey::Task(9)).is_none());
    assert!(domain.objects().find(ObjectKey::Task(10)).is_none());
    let mut closed = false;
    for request in &out {
        if let Request::Close { watch: old } = request {
            closed |= *old == watch;
        }
    }
    assert!(closed);
}
