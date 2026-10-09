use crate::{
    contract::{
        Activity, ActivityLine, ActivityPage, Block, Call, Cursor, Message, ObjectKey, Read, RunBeginning, Speaker,
        Text, Turn, TurnName, Usage,
    },
    conversation::{CallName, Conversation, Fold},
    objects::{Objects, Source},
    windows::Edge,
};
use skein_lib::{List, Wall};
fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), 100).unwrap()
}
fn cursor(bytes: &[u8]) -> Cursor {
    Cursor { value: text(bytes) }
}
fn row(position: u64) -> ActivityLine {
    ActivityLine {
        position,
        at: Wall::EPOCH,
        event: Activity::RunBegan { attempt: 1, how: RunBeginning::Fresh { tail: 3 }, why: text(b"fresh") },
    }
}
fn page(positions: &[u64], older: Option<&[u8]>, newer: Option<&[u8]>) -> ActivityPage {
    let mut rows = List::with_capacity(10);
    for position in positions {
        rows.push(row(*position)).unwrap();
    }
    ActivityPage {
        rows,
        before: match older {
            Some(bytes) => {
                let next = cursor(bytes);
                Some(next)
            }
            None => None,
        },
        after: match newer {
            Some(bytes) => {
                let next = cursor(bytes);
                Some(next)
            }
            None => None,
        },
    }
}
fn positions(c: &Conversation) -> Box<[u64]> {
    let mut positions = List::with_capacity(c.lines.rows.capacity());
    for row in &c.lines.rows {
        positions.push(row.position).unwrap();
    }
    positions.into_boxed()
}
#[test]
fn activity_reads_newest_then_older_and_keeps_reverse_cursor_when_sliding() {
    let mut c = Conversation::new(9, 3);
    assert_eq!(c.moved(6), Some(Read::Activity { task: 9, before: None }));
    assert_eq!(c.moved(6), None);
    assert_eq!(c.accept(page(&[6, 5, 4], Some(b"older"), None)), None);
    assert_eq!(positions(&c).as_ref(), &[6, 5, 4]);
    assert_eq!(c.older(), Some(Read::Activity { task: 9, before: Some(cursor(b"older")) }));
    assert_eq!(c.accept(page(&[3, 2, 1], None, Some(b"newer"))), None);
    assert_eq!(positions(&c).as_ref(), &[3, 2, 1]);
    assert_eq!(c.lines.edges.before, Edge::More(cursor(b"newer")));
    assert_eq!(c.older(), None);
    assert_eq!(c.moved(7), None);
    assert_eq!(c.newer(), Some(Read::Activity { task: 9, before: Some(cursor(b"newer")) }));
    assert_eq!(c.accept(page(&[6, 5, 4], Some(b"older"), None)), Some(Read::Activity { task: 9, before: None }));
    assert_eq!(c.accept(page(&[7, 6, 5], Some(b"older"), None)), None);
    assert_eq!(positions(&c).as_ref(), &[7, 6, 5]);
}
#[test]
fn activity_document_movement_reads_only_new_lines_and_bridges_small_pages() {
    let mut c = Conversation::new(9, 4);
    assert!(c.moved(2).is_some());
    assert_eq!(c.accept(page(&[2, 1], None, None)), None);
    assert_eq!(c.moved(2), None);
    assert!(c.moved(5).is_some());
    assert_eq!(
        c.accept(page(&[5, 4], Some(b"tail"), None)),
        Some(Read::Activity { task: 9, before: Some(cursor(b"tail")) })
    );
    assert_eq!(c.accept(page(&[3, 2], Some(b"past"), Some(b"reverse"))), None);
    assert_eq!(positions(&c).as_ref(), &[5, 4, 3, 2]);
    assert_eq!(c.loaded, 5);
    assert_eq!(c.moved(5), None);
    assert!(c.moved(6).is_some());
    c.failed();
    assert!(c.moved(6).is_some());
    assert_eq!(
        c.accept(page(&[11, 10], Some(b"remaining"), None)),
        Some(Read::Activity { task: 9, before: Some(cursor(b"remaining")) })
    );
    assert_eq!(c.accept(page(&[9, 8, 7], Some(b"overflow"), Some(b"back"))), None);
    assert_eq!(positions(&c).as_ref(), &[11, 10]);
    assert_eq!(c.lines.edges.after, Edge::More(cursor(b"remaining")));
}
fn calls(position: u64, number: u32) -> ActivityLine {
    let mut objects = List::with_capacity(2);
    objects.push(ObjectKey::Task(44)).unwrap();
    let mut blocks = List::with_capacity(2);
    blocks
        .push(Block::Call(Call {
            number: 7,
            tool: text(b"delegate"),
            input: text(b"words"),
            outcome: Some(text(b"made")),
            failed: false,
            objects,
            parts: List::with_capacity(0),
        }))
        .unwrap();
    let mut messages = List::with_capacity(1);
    messages.push(Message { speaker: Speaker::Assistant, blocks }).unwrap();
    ActivityLine {
        position,
        at: Wall::EPOCH,
        event: Activity::Turn {
            name: TurnName { attempt: 1, number },
            turn: Turn { messages, used: Usage { input: 1, output: 2, cached: 0 }, spent: 3, read_through: None },
        },
    }
}
#[test]
fn calls_fold_on_the_page_and_engine_objects_share_one_live_card() {
    let mut c = Conversation::new(9, 3);
    assert!(c.moved(2).is_some());
    let mut rows = List::with_capacity(2);
    rows.push(calls(2, 2)).unwrap();
    rows.push(calls(1, 1)).unwrap();
    assert_eq!(c.accept(ActivityPage { rows, before: None, after: None }), None);
    let call = CallName { attempt: 1, turn: 2, call: 7 };
    assert_eq!(c.fold(call), Fold::Folded);
    assert!(c.set_fold(call, Fold::Open));
    assert_eq!(c.fold(call), Fold::Open);
    assert!(c.set_fold(call, Fold::Folded));
    assert_eq!(c.fold(call), Fold::Folded);
    assert!(!c.set_fold(CallName { attempt: 8, ..call }, Fold::Open));
    let mut objects = Objects::new(3, 2);
    let source = Source::Window { generation: 1, number: 2 };
    assert_eq!(c.name_objects(&mut objects, source, 3), 0);
    let id = objects.find(ObjectKey::Task(44)).unwrap();
    assert_eq!(c.name_objects(&mut objects, source, 3), 0);
    assert_eq!(objects.len(), 1);
    assert_eq!(objects.find(ObjectKey::Task(44)), Some(id));
    assert!(objects.named(ObjectKey::Task(44), source));
}
