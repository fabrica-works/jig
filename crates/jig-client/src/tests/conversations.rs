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
fn engine_call() -> Call {
    let mut objects = List::with_capacity(2);
    objects.push(ObjectKey::Task(44)).unwrap();
    Call {
        number: 7,
        tool: text(b"delegate"),
        input: text(b"words"),
        outcome: Some(text(b"made")),
        failed: false,
        objects,
        parts: List::with_capacity(0),
    }
}
fn calls(position: u64, number: u32) -> ActivityLine {
    let mut blocks = List::with_capacity(2);
    blocks.push(Block::Call(engine_call())).unwrap();
    let mut messages = List::with_capacity(1);
    messages.push(Message { speaker: Speaker::Assistant, blocks }).unwrap();
    ActivityLine {
        position,
        at: Wall::EPOCH,
        event: Activity::Turn {
            name: TurnName { attempt: 1, number },
            turn: Turn {
                messages,
                used: Usage { input: 1, output: 2, cached: 0, cache_written: 0 },
                spent: 3,
                read_through: None,
                cut: false,
            },
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
        began: Some(Wall::EPOCH),
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
    let initial = delivered(&mut domain, Time::ZERO, f, frame(7));
    super::pages::finish_initial_inbox_reads(&mut domain, &initial);
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
    let mut live_call = stream(1, 0, 3, b"one", false);
    live_call.call = Some(crate::contract::LiveCall::Decoded(engine_call()));
    let requests = drive(
        &mut domain,
        Time::ZERO,
        Event::Watch { watch: run_watch, event: WatchEvent::Delivered(Delivery::Run(Box::new(live_call))) },
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
    assert!(domain.objects.find(ObjectKey::Task(44)).is_some());
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

#[test]
fn words_keep_the_submitted_snapshot_and_restore_sent_messages_until_read() {
    use crate::conversation::{KeptWords, Words};
    use skein_lib::Token;
    let mut words = KeptWords::new(2);
    let a = Token::new(11);
    let b = Token::new(12);
    assert!(words.begin(a, None, text(b"first")));
    assert!(words.begin(a, None, text(b"later edit")));
    assert_eq!(words.rows.get(0).unwrap().words, text(b"first"));
    assert_eq!(words.rows.get(0).unwrap().state, Words::BeingSent);
    assert!(words.begin(b, None, text(b"second")));
    assert!(!words.begin(Token::new(13), None, text(b"full")));
    words.read(1);
    assert!(words.sent(a, 1));
    assert!(words.sent(b, 2));
    assert_eq!(words.rows.get(0).unwrap().state, Words::Read { message: 1 });
    assert_eq!(words.rows.get(1).unwrap().state, Words::Sent { message: 2 });
    words.read(2);
    assert_eq!(words.rows.get(1).unwrap().state, Words::Read { message: 2 });
    assert!(words.admits());
    assert_eq!(words.rows.len(), 0);
    words.bind(Some(7));
    assert!(words.begin(a, None, text(b"private pending")));
    words.bind(Some(8));
    assert!(words.rows.is_empty());
    let mut joined = KeptWords::new(2);
    assert!(joined.begin(a, None, text(b"original")));
    assert!(joined.restore(super::spine::party(7), 3, text(b"activity copy")));
    assert!(joined.sent(a, 3));
    assert_eq!(joined.rows.len(), 1);
    assert_eq!(joined.rows.get(0).unwrap().words, text(b"original"));
    let mut reload = Conversation::new(8, 3);
    assert!(reload.moved(2).is_some());
    assert_eq!(reload.accept(words_page()), None);
    assert_eq!(reload.words.rows.len(), 2);
    assert_eq!(reload.words.rows.get(0).unwrap().request, None);
    assert_eq!(reload.words.rows.get(0).unwrap().state, Words::Sent { message: 2 });
    assert!(reload.moved(3).is_some());
    let mut committed = calls(3, 1);
    if let Activity::Turn { turn, .. } = &mut committed.event {
        turn.read_through = Some(3);
    }
    let mut rows = List::with_capacity(1);
    rows.push(committed).unwrap();
    assert_eq!(reload.accept(ActivityPage { rows, before: None, after: None }), None);
    assert_eq!(reload.words.rows.get(0).unwrap().state, Words::Read { message: 2 });
    assert_eq!(reload.words.rows.get(1).unwrap().state, Words::Read { message: 1 });
    assert_eq!(reload.words.rows.get(1).unwrap().words, text(b"first"));
}
fn words_page() -> ActivityPage {
    let mut rows = List::with_capacity(2);
    for (message, words) in [(2, b"second".as_slice()), (1, b"first".as_slice())] {
        rows.push(ActivityLine {
            position: message,
            at: Wall::EPOCH,
            event: Activity::Words { by: super::spine::party(7), message, words: text(words) },
        })
        .unwrap();
    }
    ActivityPage { rows, before: None, after: None }
}

#[test]
fn routed_words_join_the_original_request_and_clear_the_composer_only_once() {
    use super::spine::{LIMITS, drain, drive};
    use crate::{
        Event,
        address::{Address, Place, TaskSection},
        contract::*,
        fields::Purpose,
    };
    use skein_lib::{Queue, Time, Token};
    let (mut domain, _) = super::objects::live();
    drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Activity }) },
    );
    let id =
        domain.objects.name(ObjectKey::Task(8), Source::Window { generation: domain.generation, number: 4 }).unwrap();
    let mut document = super::confirmation::task();
    document.acts = List::with_capacity(1);
    document.acts.push(Offered { act: ActKind::Write, admission: Admission::Allowed }).unwrap();
    assert!(domain.objects.put(ObjectKey::Task(8), crate::objects::Body::Task(Box::new(document))));
    domain.objects.get_mut(id).unwrap().project = Some(1);
    let submitted = submit_words(&mut domain, b"first");
    let field = Purpose::Composer { task: 8 };
    let writes = domain.fields.get(field).unwrap().writes;
    assert_eq!(domain.conversation().words.rows.get(0).unwrap().state, crate::conversation::Words::BeingSent);
    assert!(domain.conversation.words.begin(Token::new(999), None, text(b"other pending")));
    let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
    assert_eq!(
        crate::intents::send_card(
            &mut domain,
            ObjectKey::Task(8),
            ActKind::Write,
            Box::new(Ask::Write { project: 1, task: 8, words: text(b"changed"), release: false }),
            &mut out
        ),
        Some(submitted)
    );
    assert!(drain(out).is_empty());
    assert_eq!(domain.conversation().words.rows.get(0).unwrap().words, text(b"first"));
    domain.conversation.words.refused(Token::new(999));
    let answer = Answer::Done(Outcome::Made { what: Made::Words { task: 8, message: 4 } });
    drive(&mut domain, Time::ZERO, Event::Answered { request: submitted, answer: answer.clone() });
    assert_eq!(domain.conversation().words.rows.get(0).unwrap().state, crate::conversation::Words::Sent { message: 4 });
    assert_eq!(domain.fields.get(field).unwrap().text.bytes(), b"");
    assert_eq!(domain.fields.get(field).unwrap().writes, writes + 1);
    drive(&mut domain, Time::ZERO, Event::Answered { request: submitted, answer });
    assert_eq!(domain.fields.get(field).unwrap().writes, writes + 1);
    domain.intents.reclaim();
    routed_later_words(&mut domain);
}
fn submit_words(domain: &mut crate::Domain, words: &[u8]) -> skein_lib::Token {
    use super::spine::drive;
    use crate::{Event, Request, action::Act, fields::Purpose};
    use skein_lib::Time;
    let field = Purpose::Composer { task: 8 };
    drive(domain, Time::ZERO, Event::Act { act: Act::Edit { field, text: text(words) } });
    drive(domain, Time::ZERO, Event::Act { act: Act::Submit { field } });
    for request in &drive(domain, Time::ZERO, Event::Act { act: Act::Confirm }) {
        if let Request::Send { request, .. } = request {
            return *request;
        }
    }
    unreachable!("confirmed words send once")
}
fn routed_later_words(domain: &mut crate::Domain) {
    use super::spine::drive;
    use crate::{Event, action::Act, contract::*, fields::Purpose};
    use skein_lib::Time;
    let request = submit_words(domain, b"second");
    let field = Purpose::Composer { task: 8 };
    drive(domain, Time::ZERO, Event::Act { act: Act::Edit { field, text: text(b"later") } });
    let writes = domain.fields.get(field).unwrap().writes;
    drive(
        domain,
        Time::ZERO,
        Event::Answered { request, answer: Answer::Done(Outcome::Made { what: Made::Words { task: 8, message: 5 } }) },
    );
    assert_eq!(domain.fields.get(field).unwrap().text.bytes(), b"later");
    assert_eq!(domain.fields.get(field).unwrap().writes, writes);
    let refused = program_words(domain);
    assert_eq!(
        refused.as_ref(),
        &[crate::Request::Answered {
            asked: skein_lib::Token::new(77),
            answer: Answer::Refused(Refusal::Limit { which: Limit::ConversationWords { limit: 2 } })
        }]
    );
    domain.conversation.words.read(5);
    assert_eq!(domain.conversation().words.rows.get(0).unwrap().state, crate::conversation::Words::Read { message: 4 });
    assert_eq!(domain.conversation().words.rows.get(1).unwrap().state, crate::conversation::Words::Read { message: 5 });
    routed_pending_restore(domain);
}
fn program_words(domain: &mut crate::Domain) -> Box<[crate::Request]> {
    super::spine::drive(
        domain,
        skein_lib::Time::ZERO,
        crate::Event::Ask {
            ask: crate::intents::ByValue {
                asked: skein_lib::Token::new(77),
                scope: 7,
                ask: Box::new(crate::contract::Ask::Write {
                    project: 1,
                    task: 8,
                    words: text(b"program"),
                    release: false,
                }),
            },
        },
    )
}
fn routed_pending_restore(domain: &mut crate::Domain) {
    use super::spine::drive;
    use crate::{
        Event, Request,
        address::{Address, Place, TaskSection},
        intents::Progress,
    };
    use skein_lib::Time;
    let mut sent = None;
    for request in &program_words(domain) {
        if let Request::Send { request, .. } = request {
            sent = Some(*request);
        }
    }
    let sent = sent.unwrap();
    drive(domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Inbox { project: None }) });
    assert!(domain.conversation().words.rows.is_empty());
    drive(
        domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Activity }) },
    );
    assert_eq!(domain.conversation().words.rows.len(), 1);
    let row = domain.conversation().words.rows.get(0).unwrap();
    assert_eq!(row.request, Some(sent));
    assert_eq!(row.words, text(b"program"));
    assert_eq!(row.state, crate::conversation::Words::BeingSent);
    drive(domain, Time::ZERO, Event::Progress { request: sent, progress: Progress::Unknown });
    assert!(domain.conversation().words.rows.is_empty());
    assert_eq!(domain.fields.get(crate::fields::Purpose::Composer { task: 8 }).unwrap().text.bytes(), b"later");
}

#[test]
fn calls_share_a_chip_watch_and_deliveries_keep_the_same_task_card_live() {
    use super::spine::{LIMITS, drive, open_token};
    use crate::{
        Event, Request,
        address::{Address, Place, TaskSection},
        contract::*,
    };
    use skein_lib::{Time, Token};
    let (mut domain, inbox) = super::objects::live();
    drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Activity }) },
    );
    drive(&mut domain, Time::ZERO, Event::Ended { watch: inbox, end: WatchEnd::Closed });
    domain.reclaim();
    let mut rows = List::with_capacity(2);
    rows.push(calls(2, 2)).unwrap();
    rows.push(calls(1, 1)).unwrap();
    let page = ActivityPage { rows, before: None, after: None };
    let mut out = skein_lib::Queue::with_capacity(crate::max_out(&LIMITS));
    let read = domain
        .reads
        .ask(
            domain.generation,
            Source::Window { generation: domain.generation, number: 1 },
            Read::Activity { task: 8, before: None },
            &mut out,
        )
        .unwrap()
        .token();
    let requests = drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read, result: ReadResult::Value(ReadValue::Activity(Box::new(page))) },
    );
    let watch = open_token(&requests);
    let mut count = 0_u32;
    for request in &requests {
        if let Request::Open { subject: Subject::Chips { tasks }, .. } = request {
            assert_eq!(tasks.as_slice(), &[44]);
            count += 1;
        }
    }
    assert_eq!(count, 1);
    let id = domain.objects.find(ObjectKey::Task(44)).unwrap();
    for phase in [Phase::Running, Phase::Held] {
        let mut chips = List::with_capacity(1);
        let mut chip = super::objects::chip(44);
        chip.phase = phase;
        *chip.details = super::objects::phase_details(phase);
        chips.push(chip).unwrap();
        let requests = drive(
            &mut domain,
            Time::ZERO,
            Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Chips(chips)) },
        );
        for request in &requests {
            if let Request::Open { .. } = request {
                unreachable!("repeated names share their watch");
            }
        }
        assert_eq!(domain.objects.find(ObjectKey::Task(44)), Some(id));
        if let Some(crate::objects::Body::Chip(chip)) = &domain.objects.get(id).unwrap().body {
            assert_eq!(chip.phase, phase);
        } else {
            unreachable!("chips update the shared task facts");
        }
    }
    let watch = changed_chip_membership(&mut domain, watch);
    drive(&mut domain, Time::ZERO, Event::Ended { watch, end: WatchEnd::Unknown });
    domain.reclaim();
    let requests = drive(&mut domain, Time::ZERO, Event::Frame { asked: Token::new(99) });
    for request in &requests {
        if let Request::Open { .. } = request {
            unreachable!("a final unavailable subject waits for different names");
        }
    }
}
fn changed_chip_membership(domain: &mut crate::Domain, old: skein_lib::Token) -> skein_lib::Token {
    use super::spine::{LIMITS, drive};
    use crate::{Event, Request, contract::*};
    use skein_lib::{Queue, Time, Token};
    let mut rows = List::with_capacity(2);
    for position in [4_u32, 3] {
        let mut row = calls(u64::from(position), position);
        if let Activity::Turn { turn, .. } = &mut row.event
            && let Block::Call(call) = turn.messages.get_mut(0).unwrap().blocks.get_mut(0).unwrap()
        {
            call.objects.clear();
            call.objects.push(ObjectKey::Task(55)).unwrap();
        }
        rows.push(row).unwrap();
    }
    let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
    let read = domain
        .reads
        .ask(
            domain.generation,
            Source::Window { generation: domain.generation, number: 1 },
            domain.conversation.moved(4).unwrap(),
            &mut out,
        )
        .unwrap()
        .token();
    let requests = drive(
        domain,
        Time::ZERO,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Activity(Box::new(ActivityPage { rows, before: None, after: None }))),
        },
    );
    let mut closed = false;
    for request in &requests {
        if let Request::Open { .. } = request {
            unreachable!("the old watch must end first");
        }
        if let Request::Close { watch } = request {
            closed = *watch == old;
        }
    }
    assert!(closed);
    drive(domain, Time::ZERO, Event::Ended { watch: old, end: WatchEnd::Closed });
    domain.reclaim();
    drive(domain, Time::ZERO, Event::Frame { asked: Token::new(98) });
    let watch = domain.conversation().chips_watch.unwrap();
    assert_ne!(watch.token(), old);
    assert_eq!(domain.conversation().chips.as_slice(), &[55]);
    watch.token()
}

#[test]
fn owner_said_receipts_join_only_the_pending_task_and_preserve_later_composer_edits() {
    use super::spine::{LIMITS, drive};
    use crate::{
        Event,
        action::Act,
        address::{Address, Place, TaskSection},
        contract::*,
        fields::Purpose,
    };
    use skein_lib::Time;
    for later in [false, true] {
        for released in [false, true] {
            for actual_task in [8, 9] {
                let (mut domain, _) = super::objects::live();
                drive(
                    &mut domain,
                    Time::ZERO,
                    Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Activity }) },
                );
                let id = domain
                    .objects
                    .name(ObjectKey::Task(8), Source::Window { generation: domain.generation, number: 4 })
                    .unwrap();
                let mut document = super::confirmation::task();
                document.acts = List::with_capacity(1);
                document.acts.push(Offered { act: ActKind::Write, admission: Admission::Allowed }).unwrap();
                assert!(domain.objects.put(ObjectKey::Task(8), crate::objects::Body::Task(Box::new(document))));
                domain.objects.get_mut(id).unwrap().project = Some(1);
                let request = submit_words(&mut domain, b"submitted");
                let field = Purpose::Composer { task: 8 };
                if later {
                    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Edit { field, text: text(b"later") } });
                }
                let writes = domain.fields.get(field).unwrap().writes;
                let answer =
                    Answer::Done(Outcome::Owner(OwnerOutcome::Said { task: actual_task, message: u64::MAX, released }));
                drive(&mut domain, Time::ZERO, Event::Answered { request, answer: answer.clone() });
                if actual_task == 8 {
                    assert_eq!(
                        domain.conversation().words.rows.get(0).unwrap().state,
                        crate::conversation::Words::Sent { message: u64::MAX }
                    );
                } else {
                    assert!(domain.conversation().words.rows.is_empty());
                }
                let cleared = actual_task == 8 && !later;
                assert_eq!(domain.fields.get(field).unwrap().writes, writes + u64::from(cleared));
                assert_eq!(domain.drafts.get(8).is_none(), cleared);
                assert_eq!(
                    domain.fields.get(field).unwrap().text.bytes(),
                    if cleared {
                        b"".as_slice()
                    } else if later {
                        b"later".as_slice()
                    } else {
                        b"submitted".as_slice()
                    }
                );
                assert_eq!(domain.objects.get(id).unwrap().answer.as_ref(), Some(&answer));
                let final_writes = domain.fields.get(field).unwrap().writes;
                assert!(drive(&mut domain, Time::ZERO, Event::Answered { request, answer }).is_empty());
                assert_eq!(domain.fields.get(field).unwrap().writes, final_writes);
                assert_eq!(domain.conversation().words.rows.capacity(), LIMITS.conversation_words);
            }
        }
    }
}

#[test]
fn raw_live_calls_keep_optional_wall_literals_cut_and_full_identity_without_made_values() {
    use crate::contract::{CurrentCall, LiveCall};
    let mut c = Conversation::new(9, 4);
    let name = TurnName { attempt: u64::MAX, number: u32::MAX - 1 };
    let mut value = stream(name.number, 0, 3, b"one", false);
    value.attempt = name.attempt;
    value.turn = name;
    value.began = None;
    value.call = Some(LiveCall::Raw(CurrentCall {
        number: u64::MAX,
        tool: text(b"raw/tool\xff"),
        input: text(b"a/b\x00\xfe"),
        cut: true,
    }));
    assert!(c.stream(value.clone(), 100));
    let held = c.live.get(0).unwrap();
    assert_eq!(held.began, None);
    assert_eq!(held.call, value.call);
    assert!(c.names(4).is_empty());
    assert!(c.parts(4).is_empty());
    let call = CallName { attempt: name.attempt, turn: name.number, call: u64::MAX };
    assert!(c.set_fold(call, Fold::Open));
    assert!(!c.set_fold(CallName { attempt: 1, ..call }, Fold::Open));
    value.began = Some(Wall::EPOCH);
    value.call = Some(LiveCall::Raw(CurrentCall {
        number: u64::MAX,
        tool: text(b"raw/tool\xff"),
        input: text(b"new literal"),
        cut: false,
    }));
    assert!(c.stream(value.clone(), 100));
    assert_eq!(c.live.get(0).unwrap().began, Some(Wall::EPOCH));
    assert_eq!(c.live.get(0).unwrap().call, value.call);
    assert_eq!(c.live.get(0).unwrap().text.bytes(), b"one");
    value.from = 1;
    value.through = 5;
    value.text = text(b"ne++");
    value.began = None;
    assert!(c.stream(value.clone(), 100));
    assert_eq!(c.live.get(0).unwrap().text.bytes(), b"one++");
    assert_eq!(c.live.get(0).unwrap().began, None);
    assert_eq!(c.live.get(0).unwrap().call, value.call);
    let later = TurnName { number: u32::MAX, ..name };
    value.turn = later;
    value.from = 0;
    value.through = 4;
    value.text = text(b"next");
    assert!(c.stream(value, 100));
    assert_eq!(c.live.len(), 2);
    assert_eq!(c.live.get(0).unwrap().text.bytes(), b"one++");
    assert!(c.moved(1).is_some());
    let mut committed = calls(1, name.number);
    if let Activity::Turn { name: committed_name, .. } = &mut committed.event {
        *committed_name = name;
    }
    let mut rows = List::with_capacity(1);
    rows.push(committed).unwrap();
    assert_eq!(c.accept(ActivityPage { rows, before: None, after: None }), None);
    assert_eq!(c.live.len(), 1);
    assert_eq!(c.live.get(0).unwrap().name, later);
    assert_eq!(c.live.get(0).unwrap().text.bytes(), b"next");
    c.end_run();
    assert!(c.live.is_empty());
    let reloaded = Conversation::new(9, 4);
    assert!(reloaded.live.is_empty(), "provisional facts are read again after reload");
}
