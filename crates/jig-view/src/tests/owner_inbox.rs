#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary public proofs inspect checked trees and scripted requests"
)]
use super::cards::{CLIENT, contains, document, drive, start};
use crate::{
    Builder, JigBindings, Role, Tree,
    phrases::{Hole, Invalid, PhraseLimits, Phrasebook, Segment},
    words::Words,
};
use jig_domain_client::{Domain, Event, Request, contract::*};
use skein_lib::{List, bytes::copy_of};
fn words(selected: bool) -> Words {
    let english = crate::phrases::english::language();
    let mut second = english.clone();
    for phrase in &mut second.phrases {
        for form in &mut phrase.forms {
            for segment in &mut form.segments {
                if let Segment::Text(bytes) = segment {
                    *bytes = copy_of(b"second ");
                }
            }
        }
    }
    Words {
        book: Phrasebook::load(
            crate::phrases::jig::shapes(CLIENT.text),
            Box::new([english, second]),
            &PhraseLimits {
                languages: 2,
                phrases: crate::owner_words::phrase_count(),
                holes: 4,
                segments: 5000,
                text_bytes: 100_000,
                hole_bytes: CLIENT.text,
            },
        )
        .unwrap(),
        language: u16::from(selected),
        offset_minutes: 0,
    }
}
fn row(object: OwnerInboxObject) -> OwnerInboxEntry {
    OwnerInboxEntry {
        reply: None,
        task: OwnerTaskLink::Stored { number: u64::MAX, status: None },
        project: None,
        object,
        activity: None,
        at: None,
        unread: None,
        cursor: OwnerInboxCursor::Number { number: 0 },
    }
}
fn whole(rows: impl IntoIterator<Item = OwnerInboxEntry>, limited: bool) -> OwnerInbox {
    let mut needs = List::with_capacity(CLIENT.rows);
    let mut updates = List::with_capacity(CLIENT.rows);
    for row in rows {
        if jig_domain_client::pages::inbox::needs(row.group()) {
            needs.push(row).unwrap();
        } else {
            updates.push(row).unwrap();
        }
    }
    OwnerInbox {
        party: 1,
        needs,
        updates,
        next: None,
        cached_needs: u32::MAX,
        cached_updates: Count { value: 17, limited },
        results_seen: u64::MAX,
        updates_time_complete: false,
        selection: None,
    }
}
fn domain(value: OwnerInbox) -> Domain {
    let (mut client, frame) = start();
    let out = drive(
        &mut client,
        Event::Watch { watch: frame, event: WatchEvent::Delivered(Delivery::Frame(Box::new(document()))) },
    );
    let watch = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Inbox { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    drive(&mut client, Event::Watch { watch, event: WatchEvent::Delivered(Delivery::OwnerInbox(Box::new(value))) });
    client
}
fn build(client: &Domain, w: &Words) -> Tree {
    let mut limits = crate::Limits::of_words(&CLIENT, w).unwrap();
    limits.nodes = crate::inbox_nodes(&CLIENT, 0).unwrap().checked_add(1).unwrap();
    limits.bindings = limits.nodes;
    limits.slots = limits.nodes;
    limits.text_bytes = limits.nodes.checked_mul(limits.text_per_node).unwrap();
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(limits.bindings, CLIENT.text);
    b.open(Role::Page);
    crate::pages::inbox(&mut b, client, w, &mut bindings);
    b.close();
    let tree = b.finish();
    assert!(tree.len() <= limits.nodes);
    let mut total = 0_u64;
    for node in tree.nodes() {
        assert!(node.text_bytes() <= limits.text_per_node);
        total = total.checked_add(u64::from(node.text_bytes())).unwrap();
    }
    assert!(total <= u64::from(limits.text_bytes));
    tree
}
#[test]
fn owner_inbox_public_six_objects_equal_makers_full_width_keys_and_missing_facts_are_checked() {
    let w = words(false);
    let cases = [
        (OwnerInboxObject::Question { message: u64::MAX }, b"Question message: 18446744073709551615".as_slice(), 50),
        (
            OwnerInboxObject::Proposal { proposer: Proposer::Task(u64::MAX), number: u64::MAX },
            b"Task proposer: 18446744073709551615 / proposal: 18446744073709551615",
            51,
        ),
        (
            OwnerInboxObject::Proposal { proposer: Proposer::Party(u64::MAX), number: u64::MAX },
            b"Party proposer: 18446744073709551615 / proposal: 18446744073709551615",
            52,
        ),
        (OwnerInboxObject::Escalation { revision: u64::MAX }, b"Escalation revision: 18446744073709551615", 53),
        (OwnerInboxObject::PersonTask, b"Person task", 54),
        (OwnerInboxObject::Reply { message: u64::MAX }, b"Reply message: 18446744073709551615", 55),
        (OwnerInboxObject::Result { position: u64::MAX }, b"Result position: 18446744073709551615", 56),
    ];
    for (object, label, kind) in cases {
        let value = whole([row(object)], false);
        let unchanged = value.clone();
        let client = domain(value);
        let tree = build(&client, &w);
        assert!(contains(&tree, label));
        assert!(contains(&tree, b"4294967295 need you"));
        assert!(contains(&tree, b"17 updates"));
        assert!(!contains(&tree, b"More than 17"));
        assert!(!contains(&tree, b"New update"));
        assert!(!contains(&tree, b"Activity position:"));
        assert!(!contains(&tree, b"Waiting for"));
        assert!(contains(&tree, b"18446744073709551615"));
        assert!(tree.nodes().iter().any(|node| if let Some(key) = node.key {
            key.kind == kind && key.a == u64::MAX
        } else {
            false
        }));
        let Some(InboxDocument::Owner(source)) = &client.inbox().document else {
            panic!("whole owner envelope");
        };
        assert_eq!(**source, unchanged);
    }
}
#[test]
fn owner_inbox_second_checked_language_rejects_wrong_holes_without_fallback() {
    let w = words(true);
    let mut value = row(OwnerInboxObject::Result { position: u64::MAX });
    value.project = Some(0);
    value.activity = Some(0);
    value.unread = Some(true);
    let client = domain(whole([value], true));
    let tree = build(&client, &w);
    assert!(contains(&tree, b"second 18446744073709551615"));
    assert!(!contains(&tree, b"Result position:"));
    assert!(!contains(&tree, b"Stored task"));
    assert!(!contains(&tree, b"More than"));
    let id = crate::owner_inbox_words::InboxPhrase::Result.id();
    assert_eq!(w.book.say(1, id, &[Hole::Number(u64::MAX)]).unwrap().text.as_ref(), b"second 18446744073709551615");
    assert_eq!(w.book.say(1, id, &[]).err(), Some(Invalid::Values { id }));
    assert!(w.book.capacity(id).unwrap() >= u32::try_from(b"second 18446744073709551615".len()).unwrap());
}
#[test]
fn owner_inbox_both_full_groups_two_read_windows_counts_source_cuts_fit_the_whole_page_bound() {
    let w = words(false);
    let values = [
        row(OwnerInboxObject::Question { message: 1 }),
        row(OwnerInboxObject::Escalation { revision: 2 }),
        row(OwnerInboxObject::Reply { message: 3 }),
        row(OwnerInboxObject::Result { position: 4 }),
    ];
    let mut client = domain(whole(values, true));
    let needs = client.inbox().needs_window.reading.unwrap();
    let updates = client.inbox().updates_window.reading.unwrap();
    for (token, entries, after) in [
        (needs, [row(OwnerInboxObject::Question { message: 5 }), row(OwnerInboxObject::PersonTask)], &[255, 0][..]),
        (
            updates,
            [row(OwnerInboxObject::Reply { message: 6 }), row(OwnerInboxObject::Result { position: 7 })],
            &[254, 0][..],
        ),
    ] {
        let mut rows = List::with_capacity(CLIENT.rows);
        for entry in entries {
            rows.push(entry).unwrap();
        }
        drive(
            &mut client,
            Event::Read {
                read: token,
                result: ReadResult::Value(ReadValue::OwnerInbox(Box::new(OwnerInboxPage {
                    selection: None,
                    cut: true,
                    rows,
                    before: Some(Cursor { value: Text::new(copy_of(after), CLIENT.text).unwrap() }),
                    after: Some(Cursor { value: Text::new(copy_of(after), CLIENT.text).unwrap() }),
                }))),
            },
        );
    }
    let tree = build(&client, &w);
    assert_eq!(client.inbox().rows.len(), 8);
    assert!(client.inbox().needs_window.cut);
    assert!(client.inbox().updates_window.cut);
    assert_ne!(client.inbox().needs_window.after, client.inbox().updates_window.after);
    assert!(contains(&tree, b"More than 17 updates"));
    assert!(contains(&tree, b"Question message: 1"));
    assert!(contains(&tree, b"Escalation revision: 2"));
    assert!(contains(&tree, b"Reply message: 3"));
    assert!(contains(&tree, b"Result position: 4"));
    assert!(crate::inbox_nodes(&jig_domain_client::Limits { rows: u32::MAX, ..CLIENT }, 0).is_none());
}
