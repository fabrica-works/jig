#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary proofs inspect bounded source values and scripted requests"
)]
//! Exact owner Inbox proofs independent of existing supplied fixture projections.
use super::spine::{LIMITS, delivered, drive, frame, open_token, start};
use crate::{
    Domain, Event, Request,
    page::inbox::{Filter, Inbox as PageInbox},
};
use jig_client::contract::*;
use skein_lib::{List, Time, Token, Wall};
fn list<T>(values: impl IntoIterator<Item = T>) -> List<T> {
    let values: Vec<_> = values.into_iter().collect();
    let mut result = List::with_capacity(u32::try_from(values.len()).unwrap());
    for value in values {
        result.push(value).ok().unwrap();
    }
    result
}
fn entry(task: u64, object: OwnerInboxObject) -> OwnerInboxEntry {
    OwnerInboxEntry {
        reply: None,
        task: OwnerTaskLink::Stored { number: task, status: None },
        project: None,
        object,
        activity: None,
        at: None,
        unread: None,
        cursor: OwnerInboxCursor::Number { number: u64::MAX },
    }
}
fn whole(needs: List<OwnerInboxEntry>, updates: List<OwnerInboxEntry>) -> OwnerInbox {
    OwnerInbox {
        party: u64::MAX,
        needs,
        updates,
        next: Some(OwnerInboxCursor::Number { number: 0 }),
        cached_needs: u32::MAX,
        cached_updates: Count { value: 17, limited: true },
        results_seen: u64::MAX,
        updates_time_complete: false,
        selection: None,
    }
}
#[test]
fn owner_inbox_six_raw_objects_full_width_makers_options_and_independent_cursors_survive() {
    let objects = [
        OwnerInboxObject::Question { message: u64::MAX },
        OwnerInboxObject::Proposal { proposer: Proposer::Task(u64::MAX), number: 7 },
        OwnerInboxObject::Proposal { proposer: Proposer::Party(u64::MAX), number: 7 },
        OwnerInboxObject::Escalation { revision: u64::MAX },
        OwnerInboxObject::PersonTask,
        OwnerInboxObject::Reply { message: u64::MAX },
        OwnerInboxObject::Result { position: u64::MAX },
    ];
    let mut ids = Vec::new();
    for object in objects {
        let row = entry(u64::MAX, object);
        assert_eq!(row.task_number(), u64::MAX);
        assert_eq!(row.task, OwnerTaskLink::Stored { number: u64::MAX, status: None });
        assert_eq!(row.object, object);
        let wrapped = InboxRow::Owner(Box::new(row.clone()));
        assert!(!ids.contains(&wrapped.identity()));
        ids.push(wrapped.identity());
        let mut zero = row.clone();
        zero.project = Some(0);
        zero.activity = Some(0);
        zero.at = Some(Wall::EPOCH);
        zero.unread = Some(false);
        zero.cursor =
            OwnerInboxCursor::Inbox { task: 0, object: OwnerInboxObject::PersonTask, at: Some(Wall::from_nanos(1)) };
        assert_ne!(row, zero);
        assert_eq!(wrapped.identity(), InboxRow::Owner(Box::new(zero)).identity());
        assert_eq!(row.project, None);
        assert_eq!(row.activity, None);
        assert_eq!(row.at, None);
        assert_eq!(row.unread, None);
    }
    let a = entry(0, OwnerInboxObject::Escalation { revision: 1 });
    let b = entry(0, OwnerInboxObject::Escalation { revision: 2 });
    assert_eq!(a.card_key(), b.card_key());
    assert_ne!(InboxRow::Owner(Box::new(a)).identity(), InboxRow::Owner(Box::new(b)).identity());
}
#[test]
fn owner_inbox_whole_groups_aggregate_counts_live_winner_and_unknown_time_source_order_are_independent() {
    let mut page = PageInbox::new(3);
    let a = entry(1, OwnerInboxObject::Question { message: 7 });
    let mut b = entry(2, OwnerInboxObject::Escalation { revision: 9 });
    b.at = Some(Wall::from_nanos(5));
    let result = entry(3, OwnerInboxObject::Result { position: 42 });
    let source = whole(list([a.clone(), b.clone()]), list([result.clone()]));
    let unchanged = source.clone();
    page.owner(Box::new(source));
    assert_eq!(
        page.rows.as_slice(),
        &[
            InboxRow::Owner(Box::new(a.clone())),
            InboxRow::Owner(Box::new(b.clone())),
            InboxRow::Owner(Box::new(result))
        ]
    );
    let Some(InboxDocument::Owner(value)) = &page.document else {
        panic!("whole source");
    };
    assert_eq!(**value, unchanged);
    assert_eq!(page.results_seen, u64::MAX);
    assert!(page.live_more);
    let query = Read::Inbox {
        group: InboxReadGroup::Needs,
        project: None,
        kind: None,
        goal: None,
        after: Some(Cursor::new(Box::from(UTF8_TEXTS[0]), 2).unwrap()),
    };
    let mut old = a.clone();
    old.activity = Some(99);
    page.owner_page(
        InboxReadGroup::Needs,
        query.clone(),
        OwnerInboxPage {
            selection: None,
            cut: true,
            rows: list([old]),
            before: None,
            after: Some(Cursor::new(Box::from(UTF8_TEXTS[2]), 2).unwrap()),
        },
    );
    assert!(page.needs_window.rows.is_empty());
    assert_eq!(page.needs_window.query, Some(query));
    assert!(page.needs_window.cut);
    assert!(page.needs_window.after.is_some());
    assert_eq!(page.rows.get(0), Some(&InboxRow::Owner(Box::new(a))));
    for limited in [false, true] {
        let mut source = unchanged.clone();
        source.cached_updates.limited = limited;
        page.owner(Box::new(source.clone()));
        let Some(InboxDocument::Owner(got)) = &page.document else {
            panic!("whole");
        };
        assert_eq!(**got, source);
    }
}
fn begin() -> (Domain, Token, Box<[Request]>) {
    let mut domain = Domain::new(&LIMITS);
    let f = start(&mut domain, List::with_capacity(0));
    let out = delivered(&mut domain, Time::ZERO, f, frame(7));
    (domain, open_token(&out), out)
}
#[test]
fn owner_inbox_read_group_original_goal_query_cut_and_opaque_cursor_are_preserved_before_consumption() {
    let (mut domain, _, out) = begin();
    super::pages::finish_initial_inbox_reads(&mut domain, &out);
    let filter = Filter { project: Some(1), kind: Some(InboxGroup::Questions), goal: Some(77) };
    let out = drive(&mut domain, Time::ZERO, Event::Act { act: crate::action::Act::FilterInbox { filter } });
    let (read, query) = out
        .iter()
        .find_map(|r| match r {
            Request::Read { read, what: what @ Read::Inbox { .. } } => Some((*read, what.clone())),
            _ => None,
        })
        .unwrap();
    let mut row = entry(3, OwnerInboxObject::Question { message: 5 });
    row.project = Some(1);
    let original = row.clone();
    let cursor = Cursor::new(Box::from(UTF8_TEXTS[1]), 3).unwrap();
    let wrong = OwnerInboxPage {
        selection: None,
        cut: true,
        rows: list([entry(3, OwnerInboxObject::Result { position: 9 })]),
        before: None,
        after: Some(cursor.clone()),
    };
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read, result: ReadResult::Value(ReadValue::OwnerInbox(Box::new(wrong))) },
    );
    assert_eq!(domain.inbox().needs_window.reading, Some(read));
    assert!(domain.inbox().needs_window.rows.is_empty());
    let context = list([OwnerInboxSelection { cursor: row.cursor, goals: list([3, 77]) }]);
    let page = OwnerInboxPage {
        selection: Some(context.clone()),
        cut: true,
        rows: list([row]),
        before: None,
        after: Some(cursor.clone()),
    };
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read, result: ReadResult::Value(ReadValue::OwnerInbox(Box::new(page))) },
    );
    assert_eq!(domain.inbox().rows.get(0), Some(&InboxRow::Owner(Box::new(original))));
    assert_eq!(domain.inbox().needs_window.query, Some(query));
    assert_eq!(domain.inbox().needs_window.selection, Some(context));
    assert!(domain.inbox().needs_window.cut);
    assert_eq!(domain.inbox().needs_window.after, Some(cursor));
    assert_eq!(domain.inbox().needs_window.reading, None);
    assert!(domain.inbox().updates_window.rows.is_empty());
}
#[test]
fn owner_inbox_live_goal_context_is_explicitly_pending_and_bounded_read_membership_is_not_invented() {
    let mut page = PageInbox::new(2);
    page.filter(Filter { project: None, kind: None, goal: Some(77) });
    page.owner(Box::new(whole(list([entry(3, OwnerInboxObject::Question { message: 5 })]), List::with_capacity(2))));
    assert!(page.goal_context_pending);
    assert!(page.rows.is_empty());
    let query = Read::Inbox { group: InboxReadGroup::Needs, project: None, kind: None, goal: Some(77), after: None };
    let row = entry(3, OwnerInboxObject::Question { message: 5 });
    page.owner_page(
        InboxReadGroup::Needs,
        query,
        OwnerInboxPage { selection: None, cut: false, rows: list([row.clone()]), before: None, after: None },
    );
    assert_eq!(page.rows.get(0), Some(&InboxRow::Owner(Box::new(row))));
    assert!(page.goal_context_pending);
}
#[test]
fn owner_inbox_reply_keeps_preview_unread_runtime_options_and_activity_distinct_from_mark_seen() {
    let (mut domain, watch, out) = begin();
    super::pages::finish_initial_inbox_reads(&mut domain, &out);
    let line = ConversationLine {
        chip: super::objects::chip(8),
        project: 1,
        first_words: DocumentText::complete(super::objects::text(b"full preview")),
        changed: None,
        runtime: None,
        unread: None,
        last_reply: 9,
        seen: 4,
        delegates: 0,
        cached_replies: u32::MAX,
        budget: OwnerBudget {
            numbers: OwnerNumbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
            left: 0,
            funder: Funder::Period { project: 1, period: 0 },
            deadline: None,
        },
    };
    let mut row = entry(8, OwnerInboxObject::Reply { message: u64::MAX });
    row.reply = Some(Box::new(line.clone()));
    row.project = Some(1);
    row.activity = Some(0);
    let mut value = whole(List::with_capacity(0), list([row.clone()]));
    value.party = 7;
    value.results_seen = 0;
    drive(
        &mut domain,
        Time::ZERO,
        Event::Watch { watch, event: WatchEvent::Delivered(Delivery::OwnerInbox(Box::new(value))) },
    );
    let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
    let reply = domain.objects().get(id).unwrap().reply.as_ref().unwrap();
    assert_eq!(reply.first_unread, 5);
    assert_eq!(reply.first_words, line.first_words);
    assert_eq!(domain.inbox().rows.get(0), Some(&InboxRow::Owner(Box::new(row))));
    let out = drive(&mut domain, Time::ZERO, Event::Act { act: crate::action::Act::OpenChat { task: 8 } });
    assert!(out.iter().any(|r| match r {
        Request::Send { ask, .. } => **ask == Ask::MarkSeen { seen: Seen::Replies { chat: 8, through: 9 } },
        _ => false,
    }));
}
#[test]
fn owner_inbox_superseded_query_terminal_retires_without_replacing_the_new_filter_source() {
    let (mut domain, _, out) = begin();
    super::pages::finish_initial_inbox_reads(&mut domain, &out);
    let old = Filter { project: Some(1), kind: Some(InboxGroup::Questions), goal: Some(77) };
    let out = drive(&mut domain, Time::ZERO, Event::Act { act: crate::action::Act::FilterInbox { filter: old } });
    let (token, query) = out
        .iter()
        .find_map(|request| match request {
            Request::Read { read, what: what @ Read::Inbox { .. } } => Some((*read, what.clone())),
            _ => None,
        })
        .unwrap();
    let new = Filter { goal: Some(88), ..old };
    drive(&mut domain, Time::ZERO, Event::Act { act: crate::action::Act::FilterInbox { filter: new } });
    let current = domain.inbox().needs_window.reading.unwrap();
    assert_ne!(token, current);
    let page = OwnerInboxPage {
        selection: None,
        cut: true,
        rows: list([entry(3, OwnerInboxObject::Question { message: 5 })]),
        before: None,
        after: Some(Cursor::new(Box::from(b"obsolete".as_slice()), 1024).expect("bounded opaque cursor")),
    };
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerInbox(Box::new(page))) },
    );
    assert!(!domain.reads.pending(&query));
    assert_eq!(domain.inbox().filter, new);
    assert_eq!(domain.inbox().needs_window.reading, Some(current));
    assert!(domain.inbox().needs_window.rows.is_empty());
    assert!(!domain.inbox().needs_window.cut);
    assert_eq!(domain.inbox().needs_window.query, None);
}

#[test]
fn owner_inbox_live_supplied_cursor_roots_control_membership_without_reinterpreting_historical_context() {
    let mut page = PageInbox::new(2);
    let mut a = entry(3, OwnerInboxObject::Question { message: 5 });
    a.cursor = OwnerInboxCursor::Inbox { task: 3, object: a.object, at: None };
    let mut b = entry(4, OwnerInboxObject::Escalation { revision: u64::MAX });
    b.cursor = OwnerInboxCursor::Number { number: 4 };
    let mut source = whole(list([a.clone(), b.clone()]), List::with_capacity(2));
    source.selection = Some(list([
        OwnerInboxSelection { cursor: a.cursor, goals: list([3, 77, 88, u64::MAX]) },
        OwnerInboxSelection { cursor: b.cursor, goals: List::with_capacity(4) },
    ]));
    for goal in [3, 77, 88, u64::MAX] {
        page.filter(Filter { project: None, kind: None, goal: Some(goal) });
        page.owner(Box::new(source.clone()));
        assert!(!page.goal_context_pending);
        assert_eq!(page.rows.as_slice(), &[InboxRow::Owner(Box::new(a.clone()))]);
    }
    page.filter(Filter { project: None, kind: None, goal: Some(4) });
    assert!(!page.goal_context_pending);
    assert!(page.rows.is_empty()); // b's task number supplies no inferred membership.
    page.filter(Filter { project: None, kind: None, goal: Some(77) });
    let query = Read::Inbox { group: InboxReadGroup::Needs, project: None, kind: None, goal: Some(77), after: None };
    let mut stored = entry(9, OwnerInboxObject::Question { message: 1 });
    stored.cursor = OwnerInboxCursor::Number { number: 9 };
    let retained = list([OwnerInboxSelection { cursor: stored.cursor, goals: list([9, 77]) }]);
    page.owner_page(
        InboxReadGroup::Needs,
        query.clone(),
        OwnerInboxPage {
            selection: Some(retained.clone()),
            cut: true,
            rows: list([stored.clone()]),
            before: None,
            after: None,
        },
    );
    assert_eq!(page.rows.len(), 2);
    assert_eq!(page.needs_window.selection, Some(retained.clone()));
    let mut conflicting = whole(list([stored.clone()]), List::with_capacity(2));
    conflicting.selection = Some(list([OwnerInboxSelection { cursor: stored.cursor, goals: list([9, 88]) }]));
    page.owner(Box::new(conflicting));
    assert!(!page.goal_context_pending);
    assert!(page.rows.is_empty()); // Current watch is an actual negative, not a guessed one.
    assert!(page.needs_window.rows.is_empty());
    assert_eq!(page.needs_window.query, Some(query));
    assert_eq!(page.needs_window.selection, Some(retained));
    source.selection = Some(list([OwnerInboxSelection {
        cursor: OwnerInboxCursor::Inbox { task: 3, object: a.object, at: Some(Wall::EPOCH) },
        goals: list([77]),
    }]));
    page.owner(Box::new(source));
    assert!(page.goal_context_pending); // Equal raw identity does not replace the original cursor association.
    assert!(page.rows.is_empty());
    page.owner(Box::new(whole(List::with_capacity(2), List::with_capacity(2))));
    assert!(!page.goal_context_pending);
    assert!(page.rows.is_empty());
}

const UTF8_TEXTS: [&[u8]; 3] = [b"~\0", b"~\0!", b"!\0"];
