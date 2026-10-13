#![expect(clippy::disallowed_methods, reason = "ordinary whole page proofs inspect supplied tree facts")]
use super::{cards, owner_history as fixture};
use crate::{Builder, JigBindings, Key, Owner, Role, Tree, words::Words};
use jig_domain_client::{
    Domain, Event, Request,
    address::{Address, TaskSection},
    boundary::Offset,
    contract::*,
};
use skein_lib::{Env, List, Queue, Time, Token, Wall};
fn drive(domain: &mut Domain, event: Event) -> List<Request> {
    let limits = fixture::client_limits();
    let mut out = Queue::with_capacity(jig_domain_client::max_out(&limits));
    jig_domain_client::step(domain, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits }, event, &mut out);
    let mut values = List::with_capacity(jig_domain_client::max_out(&limits));
    while let Some(request) = out.pop() {
        values.push(request).unwrap();
    }
    values
}
fn history_read(values: &[Request]) -> Token {
    for value in values {
        if let Request::Read { read, what: Read::History { .. } } = value {
            return *read;
        }
    }
    panic!("whole source history request");
}
fn history_page() -> HistoryPage {
    let mut before = fixture::state();
    before.instructions = None;
    let mut after = before.clone();
    after.instructions = Some(DocumentText::complete(cards::text(b"")));
    after.cut = true;
    let mut batch = List::with_capacity(3);
    for _ in 0_u32..3 {
        batch.push(super::owner_history::task()).unwrap();
    }
    let mut value = fixture::line();
    value.revision = u64::MAX;
    value.by = OwnerRequester::Deployment { project: u32::MAX };
    value.amendment = Some((Box::new(before), Box::new(after)));
    value.proposal = Some(fixture::proposal(OwnerRetainedAction::Batch(batch)));
    let mut rows = List::with_capacity(3);
    rows.push(HistoryRow::Change(Box::new(value))).unwrap();
    rows.push(HistoryRow::Milestone(Box::new(OwnerMilestoneLine {
        task: u64::MAX,
        position: u64::MAX,
        at: Some(Wall::EPOCH),
        phase: OwnerMilestonePhase::Held {
            before: OwnerPriorPhase::Closing(ClosingStage::Run { attempt: u64::MAX }),
            why: HoldCategory::Uncertain { entry: u64::MAX },
            source: Some(fixture::part()),
            ending: Some(OwnerChipEnding::Done(ResultValue::Approved {
                state: Box::from([255_u8; 8]),
                words: Some(DocumentText { bytes: cards::text(b""), cut: true }),
            })),
        },
    })))
    .unwrap();
    rows.push(HistoryRow::Line(Box::new(HistoryLine {
        task: u64::MAX,
        position: u64::MAX,
        by: Actor::Party(7),
        at: Wall::from_nanos(u64::MAX),
        change: HistoryChange::Made { batch: 99 },
    })))
    .unwrap();
    HistoryPage {
        cut: true,
        rows,
        before: None,
        after: Some(Cursor::new(Box::from(b"opaque".as_slice()), 1024).expect("bounded opaque cursor")),
    }
}
fn populated() -> Domain {
    let mut domain = Domain::new(&fixture::client_limits(), 208);
    let out = drive(
        &mut domain,
        Event::Start {
            address: Address::Task { task: 8, section: TaskSection::History },
            kept: List::with_capacity(0),
            offset: Offset { minutes: 0 },
        },
    );
    let Request::Open { watch, subject: Subject::Frame } = out.as_slice()[0] else { panic!("explicit frame watch") };
    drive(&mut domain, Event::Hello { versions: VersionRange { first: 5, last: 5 } });
    let mut frame = cards::document();
    frame.party.login = DocumentText::complete(cards::text(b"login"));
    let opened =
        drive(&mut domain, Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Frame(Box::new(frame))) });
    let _original_read = history_read(opened.as_slice());
    let task_watch = opened
        .as_slice()
        .iter()
        .find_map(|request| {
            if let Request::Open { watch, subject: Subject::Task { task: 8 }, .. } = request {
                Some(*watch)
            } else {
                None
            }
        })
        .expect("actual task header watch");
    let mut task = super::cards_objects::task_doc(&[]);
    task.instructions = Some(DocumentText::complete(cards::text(b"now")));
    drive(
        &mut domain,
        Event::Watch { watch: task_watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(task))) },
    );
    let read = history_read(
        drive(&mut domain, Event::Act { act: jig_domain_client::action::Act::HistoryScope { subtree: true } })
            .as_slice(),
    );
    drive(&mut domain, Event::Read { read, result: ReadResult::Value(ReadValue::History(Box::new(history_page()))) });
    domain
}
fn build(domain: &Domain, words: &Words) -> (Tree, JigBindings) {
    let client = fixture::client_limits();
    let limits = crate::Limits::of_words(&client, words).unwrap();
    let mut bindings = JigBindings::new(limits.bindings, client.text);
    let mut b = Builder::new(&limits);
    b.open(Role::Page);
    crate::pages::page(&mut b, domain, words, &mut bindings);
    b.close();
    let tree = b.finish();
    assert!(tree.len() <= limits.nodes);
    assert!(tree.nodes().iter().all(|node| node.text_bytes() <= limits.text_per_node));
    (tree, bindings)
}
#[test]
fn whole_history_page_keeps_three_distinct_keys_stored_overrides_and_raw_slots_in_selected_languages() {
    let domain = populated();
    assert_eq!(domain.history_rows().rows.as_slice(), history_page().rows.as_slice());
    for selected in [false, true] {
        let words = fixture::words(selected);
        let (tree, bindings) = build(&domain, &words);
        for kind in [21, 41, 42] {
            let key = Key { owner: Owner::Jig, kind, a: u64::MAX, b: u64::MAX };
            assert_eq!(tree.nodes().iter().filter(|node| node.key == Some(key)).count(), 1);
            if kind != 21 {
                let index = tree.nodes().iter().position(|node| node.key == Some(key)).unwrap();
                let region = &tree.nodes()[index + 1];
                assert_eq!(region.role, Role::Region);
                assert!(region.name.as_ref().is_some_and(|name| !name.is_empty()));
            }
        }
        let mut slot_count = 0_u32;
        for node in tree.nodes() {
            if let Some(slot) = node.slot {
                slot_count += 1;
                assert_eq!(bindings.owner_part(slot.token), Some(&fixture::part()));
                assert!(bindings.part(slot.token).is_none());
            }
        }
        assert_eq!(slot_count, 7);
        assert!(!cards::contains(&tree, b"now"), "stored snapshots do not borrow the current override");
        assert!(cards::contains(&tree, b"a/b"));
        assert!(cards::contains(&tree, b"\\xFF\\x00"));
        assert!(cards::contains(&tree, b"\\xFF\\xFF\\xFF\\xFF\\xFF\\xFF\\xFF\\xFF"));
        if selected {
            assert!(cards::contains(&tree, b"selected history: "));
        } else {
            assert!(cards::contains(&tree, b"Stored"));
            assert!(cards::contains(&tree, b"This page is incomplete"));
            assert!(cards::contains(&tree, b"99"));
        }
    }
}
