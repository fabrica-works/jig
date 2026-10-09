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
fn project(number: u32, chats: bool) -> ProjectRole {
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
#[test]
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
#[test]
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
#[test]
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
    kept.push(crate::store::Record::Draft { version: 1, party: 8, task: 42, words: text(b"secret") }).unwrap();
    let f = start(&mut other, kept);
    delivered(&mut other, Time::ZERO, f, frame(7));
    assert_eq!(other.page(), &Page::SomeoneElse);
}
#[test]
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
        chip: Chip { task, title: text(b"held checkout"), executor: Executor::Agent { charter: 1 }, phase },
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
#[test]
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
#[test]
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
            count += 1;
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
#[test]
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
#[test]
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
            Answer::Refused(Refusal::Authority { lacking: Lacking::Budget { needed: 10, left: 2 } })
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

#[test]
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
#[test]
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
#[test]
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
#[test]
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
