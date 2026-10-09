use crate::{
    contract::{
        Activity, ActivityLine, ActivityPage, Block, Call, Cursor, Message, ObjectKey, Read, RunBeginning, RunText,
        Speaker, Text, Turn, TurnName, Usage,
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

fn stream(number: u32, from: u32, through: u32, bytes: &[u8], cut: bool) -> RunText {
    RunText {
        attempt: 1,
        began: Wall::EPOCH,
        turn: TurnName { attempt: 1, number },
        from,
        through,
        text: text(bytes),
        cut,
        call: None,
    }
}
#[test]
fn provisional_turns_append_by_offset_and_commit_only_replaces_the_same_name() {
    let mut c = Conversation::new(9, 4);
    assert!(c.stream(stream(1, 0, 3, b"one", false), 10));
    assert!(c.stream(stream(1, 3, 5, b"++", false), 10));
    assert_eq!(c.live.get(0).unwrap().text.bytes(), b"one++");
    assert!(c.stream(stream(1, 3, 5, b"++", false), 10));
    assert_eq!(c.live.get(0).unwrap().text.bytes(), b"one++");
    assert!(!c.stream(stream(1, 7, 8, b"x", false), 10));
    assert!(c.stream(stream(2, 0, 3, b"two", false), 10));
    assert_eq!(c.live.len(), 2);
    assert_eq!(c.live.get(0).unwrap().text.bytes(), b"one++");
    assert_eq!(c.offset(), 3);
    assert!(c.moved(1).is_some());
    let mut rows = List::with_capacity(1);
    rows.push(calls(1, 1)).unwrap();
    assert_eq!(c.accept(ActivityPage { rows, before: None, after: None }), None);
    assert_eq!(c.live.len(), 1);
    assert_eq!(c.live.get(0).unwrap().name.number, 2);
    assert!(c.stream(stream(1, 0, 3, b"old", false), 10));
    assert_eq!(c.live.len(), 1);
    c.end_run();
    assert!(c.live.is_empty());
    assert_eq!(c.current, None);
    routed_conversation();
}
#[test]
fn provisional_text_keeps_its_cut_mark_and_recovery_replaces_uncommitted_text() {
    let mut c = Conversation::new(9, 2);
    assert!(c.stream(stream(1, 0, 3, b"abc", false), 4));
    assert!(c.stream(stream(1, 3, 6, b"def", false), 4));
    assert_eq!(c.live.get(0).unwrap().text.bytes(), b"cdef");
    assert!(c.live.get(0).unwrap().cut);
    assert!(c.stream(stream(1, 10, 12, b"xy", true), 4));
    assert_eq!(c.live.get(0).unwrap().text.bytes(), b"xy");
    assert_eq!(c.offset(), 12);
    c.interrupted();
    assert_eq!(c.live.get(0).unwrap().text.bytes(), b"xy");
    assert!(c.stream(stream(1, 0, 3, b"new", false), 4));
    assert_eq!(c.live.get(0).unwrap().text.bytes(), b"new");
    assert!(!c.live.get(0).unwrap().cut);
    assert!(c.stream(stream(2, 0, 1, b"x", false), 4));
    assert!(!c.stream(stream(3, 0, 1, b"y", false), 4));
    assert_eq!(c.live.len(), 2);
    c.end_run();
    assert!(c.stream(stream(3, 0, 1, b"y", false), 4));
    assert_eq!(c.current, Some(TurnName { attempt: 1, number: 3 }));
}

fn activity_read(requests: &[crate::Request]) -> skein_lib::Token {
    for request in requests {
        if let crate::Request::Read { read, what: Read::Activity { .. } } = request {
            return *read;
        }
    }
    unreachable!("activity read requested")
}
fn routed_conversation() {
    use super::spine::{LIMITS, delivered, drive, frame, open_token, start};
    use crate::{
        Domain, Event, Request,
        address::{Address, Place, TaskSection},
        contract::{Delivery, ReadResult, ReadValue, Runtime, WatchEvent},
    };
    use skein_lib::Time;
    let mut domain = Domain::new(&LIMITS, 21);
    let f = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    let opened = drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Activity }) },
    );
    let task_watch = open_token(&opened);
    let mut document = super::confirmation::task();
    document.chip.executor = crate::contract::Executor::Agent { charter: 1 };
    document.runtime = Runtime::Running { attempt: 1, since: Wall::EPOCH, tool: None };
    document.activity = 1;
    let requests = drive(
        &mut domain,
        Time::ZERO,
        Event::Watch { watch: task_watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(document.clone()))) },
    );
    let run_watch = open_token(&requests);
    let read = activity_read(&requests);
    let first = page(&[1], Some(b"older"), None);
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read, result: ReadResult::Value(ReadValue::Activity(Box::new(first))) },
    );
    assert_eq!(domain.turn_rows().rows.len(), 1);
    let requests = drive(
        &mut domain,
        Time::ZERO,
        Event::Watch {
            watch: run_watch,
            event: WatchEvent::Delivered(Delivery::Run(Box::new(stream(1, 0, 3, b"one", false)))),
        },
    );
    let mut continued = false;
    for request in &requests {
        if let Request::Continue {
            watch,
            subject: crate::contract::Subject::Run { turn: Some(TurnName { attempt: 1, number: 1 }), offset: 3, .. },
        } = request
        {
            continued = *watch == run_watch;
        }
    }
    assert!(continued);
    document.activity = 2;
    let requests = drive(
        &mut domain,
        Time::ZERO,
        Event::Watch { watch: task_watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(document.clone()))) },
    );
    let read = activity_read(&requests);
    for request in &requests {
        if let Request::Open { .. } = request {
            unreachable!("one run watch follows each page");
        }
    }
    drive(
        &mut domain,
        Time::ZERO,
        Event::Watch {
            watch: run_watch,
            event: WatchEvent::Delivered(Delivery::Run(Box::new(stream(2, 0, 3, b"two", false)))),
        },
    );
    let mut rows = List::with_capacity(2);
    rows.push(calls(2, 1)).unwrap();
    rows.push(row(1)).unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Activity(Box::new(ActivityPage {
                rows,
                before: Some(cursor(b"older")),
                after: None,
            }))),
        },
    );
    assert_eq!(domain.conversation().live.len(), 1);
    assert_eq!(domain.conversation().live.get(0).unwrap().name.number, 2);
    routed_scroll_and_end(&mut domain, document, task_watch);
}
fn routed_scroll_and_end(
    domain: &mut crate::Domain,
    mut document: crate::contract::Task,
    task_watch: skein_lib::Token,
) {
    use super::spine::drive;
    use crate::{
        Event, Request,
        action::Act,
        contract::{Delivery, ReadResult, ReadValue, Runtime, WatchEvent},
    };
    use skein_lib::{Time, Token};
    let name = CallName { attempt: 1, turn: 1, call: 7 };
    drive(domain, Time::ZERO, Event::Act { act: Act::FoldCall { name, open: true } });
    assert_eq!(domain.conversation().fold(name), Fold::Open);
    let requests = drive(
        domain,
        Time::ZERO,
        Event::Act { act: Act::More { window: Token::new(1), end: crate::action::End::Before } },
    );
    let read = activity_read(&requests);
    let mut continued = false;
    for request in &requests {
        if let Request::Read { what: Read::Activity { before: Some(_), .. }, .. } = request {
            continued = true;
        }
    }
    assert!(continued);
    drive(
        domain,
        Time::ZERO,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Activity(Box::new(page(&[0], None, Some(b"newer"))))),
        },
    );
    document.runtime = Runtime::Idle;
    drive(
        domain,
        Time::ZERO,
        Event::Watch { watch: task_watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(document))) },
    );
    assert!(domain.conversation().live.is_empty());
}
