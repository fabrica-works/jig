const OTHER_PENDING: jig_client::Key = jig_client::Key { asker: jig_client::Asker::JIG, number: 999 };
use crate::Request;
use crate::objects::{Objects, Source};
use crate::tests::fixtures::{AnsweredByEngine, Ended, Fixture, ReadPage, Reported, Watched};
use jig_client::contract::{
    Activity, ActivityLine, ActivityPage, Block, Call, Cursor, Message, ObjectKey, ProvisionalTurnName, Read,
    RunBeginning, RunText, Speaker, Text, Turn, TurnName, TurnRead, Usage,
};
use jig_client::thread::{CallName, Fold, Thread};
use skein_lib::{List, Wall};
fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), 100).unwrap()
}
fn cursor(bytes: &[u8]) -> Cursor {
    Cursor::new(Box::from(bytes), 1024).expect("bounded opaque cursor")
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
        cut: false,
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
            turn: TurnRead::Decoded(Turn {
                messages,
                used: Usage { input: 1, output: 2, cached: 0, cache_written: 0 },
                spent: 3,
                read_through: None,
                cut: false,
            }),
        },
    }
}
#[test]
fn calls_fold_on_the_page_and_engine_objects_share_one_live_card() {
    let mut c = Thread::new(9, &jig_client::thread::Limits { lines: 3, bytes: 65536, text: 4096 });
    assert!(c.moved(2).is_some());
    let mut rows = List::with_capacity(2);
    rows.push(calls(2, 2)).unwrap();
    rows.push(calls(1, 1)).unwrap();
    assert_eq!(c.accept(ActivityPage { cut: false, rows, before: None, after: None }), None);
    let call = CallName { attempt: 1, turn: 2, call: 7 };
    assert_eq!(c.fold(call), Fold::Folded);
    assert!(c.set_fold(call, Fold::Open, &[]));
    assert_eq!(c.fold(call), Fold::Open);
    assert!(c.set_fold(call, Fold::Folded, &[]));
    assert_eq!(c.fold(call), Fold::Folded);
    assert!(!c.set_fold(CallName { attempt: 8, ..call }, Fold::Open, &[]));
    let mut objects = Objects::new(3, 2);
    let source = Source::Window { number: 2 };
    assert_eq!(objects.replace(source, c.names(3, &[]).as_slice()), 0);
    let id = objects.find(ObjectKey::Task(44)).unwrap();
    assert_eq!(objects.replace(source, c.names(3, &[]).as_slice()), 0);
    assert_eq!(objects.len(), 1);
    assert_eq!(objects.find(ObjectKey::Task(44)), Some(id));
    assert!(objects.named(ObjectKey::Task(44), source));
}

#[test]
fn provisional_turns_append_by_offset_and_commit_only_replaces_the_same_name() {
    let (mut c, run) = native_run(4);
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(stream(1, 0, 3, b"one", false))));
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(stream(1, 3, 5, b"++", false))));
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"one++");
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(stream(1, 3, 5, b"++", false))));
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"one++");
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(stream(1, 7, 8, b"x", false))));
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(stream(2, 0, 3, b"two", false))));
    assert_eq!(c.conversation().live.len(), 2);
    assert_eq!(c.conversation().live.get(1).unwrap().text.bytes(), b"two");
    assert_eq!(
        match c.client.text(run) {
            Some(text) => text.through,
            None => 0,
        },
        3
    );
    assert!(c.ui.conversation.moved(1).is_some());
    let mut rows = List::with_capacity(1);
    rows.push(calls(1, 1)).unwrap();
    assert_eq!(c.ui.conversation.accept(ActivityPage { cut: false, rows, before: None, after: None }), None);
    assert_eq!(c.conversation().live.len(), 1);
    assert_eq!(c.conversation().live.get(0).unwrap().name.number, 2);
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(stream(1, 0, 3, b"old", false))));
    assert_eq!(c.conversation().live.len(), 1);
    c.ui.conversation.run_finished = true;
    c.ui.conversation.end_run();
    assert!(c.conversation().live.is_empty());
    assert_eq!(c.first_turn(), None);
    routed_conversation();
}
#[test]
fn provisional_text_keeps_its_cut_mark_and_recovery_replaces_uncommitted_text() {
    let (mut c, run) = native_run(2);
    c.ui.text_limit = 4;
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(stream(1, 0, 3, b"abc", false))));
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(stream(1, 3, 6, b"def", false))));
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"cdef");
    assert!(c.conversation().live.get(0).unwrap().cut);
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(stream(1, 10, 12, b"xy", true))));
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"xy");
    assert_eq!(
        match c.client.text(run) {
            Some(text) => text.through,
            None => 0,
        },
        12
    );
    c.reopen(run);
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"xy");
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(stream(1, 0, 3, b"new", false))));
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"new");
    assert!(!c.conversation().live.get(0).unwrap().cut);
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(stream(2, 0, 1, b"x", false))));
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(stream(3, 0, 1, b"y", false))));
    assert_eq!(c.conversation().live.len(), 2);
    c.ui.conversation.end_run();
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(stream(3, 0, 1, b"y", false))));
    assert_eq!(c.first_turn(), Some(ProvisionalTurnName { retry: 0, attempt: 1, number: 3 }));
}

fn activity_read(requests: &[Request]) -> jig_client::Handle {
    for request in requests {
        if let Request::Client {
            asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Activity { .. }) },
        } = request
        {
            return *read;
        }
    }
    unreachable!("activity read requested")
}
fn routed_conversation() {
    use super::spine::{LIMITS, delivered, drive, frame, open_token, start};
    use crate::{
        Event, Request,
        address::{Address, Place, TaskSection},
    };
    use jig_client::contract::{Delivery, ReadResult, ReadValue, Runtime, WatchEvent};
    use skein_lib::Time;
    let mut domain = Fixture::new(&LIMITS);
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
    document.chip.executor = jig_client::contract::Executor::Agent { charter: 1 };
    document.context.as_mut().expect("supplied task context").runtime =
        Runtime::Running { attempt: 1, since: Wall::EPOCH, tool: None };
    document.activity = 1;
    let requests = drive(
        &mut domain,
        Time::ZERO,
        Watched { watch: task_watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(document.clone()))) },
    );
    let run_watch = open_token(&requests);
    let read = activity_read(&requests);
    let first = page(&[1], Some(b"older"), None);
    drive(&mut domain, Time::ZERO, ReadPage { read, result: ReadResult::Value(ReadValue::Activity(Box::new(first))) });
    assert_eq!(domain.turn_rows().rows.len(), 1);
    let mut live_call = stream(1, 0, 3, b"one", false);
    live_call.call = Some(jig_client::contract::LiveCall::Decoded(engine_call()));
    let _requests = drive(
        &mut domain,
        Time::ZERO,
        Watched { watch: run_watch, event: WatchEvent::Delivered(Delivery::Run(Box::new(live_call))) },
    );
    domain.reopen(run_watch);
    let continued = domain.down.as_slice().contains(&jig_client::Down::Continue {
        watch: domain.watch_token(run_watch).unwrap(),
        subject: jig_client::contract::Subject::Run {
            task: 8,
            turn: Some(ProvisionalTurnName { retry: 0, attempt: 1, number: 1 }),
            offset: 3,
        },
    });
    assert!(continued);
    assert!(domain.objects().find(ObjectKey::Task(44)).is_some());
    document.activity = 2;
    let requests = drive(
        &mut domain,
        Time::ZERO,
        Watched { watch: task_watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(document.clone()))) },
    );
    let read = activity_read(&requests);
    for request in &requests {
        if let Request::Client { asked: jig_client::Event::Watch { .. } } = request {
            unreachable!("one run watch follows each page");
        }
    }
    drive(
        &mut domain,
        Time::ZERO,
        Watched {
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
        ReadPage {
            read,
            result: ReadResult::Value(ReadValue::Activity(Box::new(ActivityPage {
                cut: false,
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
    domain: &mut Fixture,
    mut document: jig_client::contract::Task,
    task_watch: jig_client::Handle,
) {
    use super::spine::drive;
    use crate::{Event, Request, action::Act};
    use jig_client::contract::{Delivery, ReadResult, ReadValue, Runtime, WatchEvent};
    use skein_lib::Time;
    let name = CallName { attempt: 1, turn: 1, call: 7 };
    drive(domain, Time::ZERO, Event::Act { action: Act::FoldCall { name, open: true } });
    assert_eq!(domain.conversation().fold(name), Fold::Open);
    let requests = drive(
        domain,
        Time::ZERO,
        Event::Act {
            action: Act::More { window: skein_lib::Token::new(1), end: jig_client::thread::window::End::Before },
        },
    );
    let read = activity_read(&requests);
    let mut continued = false;
    for request in &requests {
        if let Request::Client {
            asked:
                jig_client::Event::Read { read: jig_client::Readable::Jig(Read::Activity { before: Some(_), .. }), .. },
        } = request
        {
            continued = true;
        }
    }
    assert!(continued);
    drive(
        domain,
        Time::ZERO,
        ReadPage { read, result: ReadResult::Value(ReadValue::Activity(Box::new(page(&[0], None, Some(b"newer"))))) },
    );
    document.context.as_mut().expect("supplied task context").runtime = Runtime::Idle;
    drive(
        domain,
        Time::ZERO,
        Watched { watch: task_watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(document))) },
    );
    assert!(domain.conversation().live.is_empty());
}

#[test]
fn routed_words_join_the_original_request_and_clear_the_composer_only_once() {
    use super::spine::{LIMITS, drain, drive};
    use crate::{
        Event,
        address::{Address, Place, TaskSection},
        fields::Purpose,
    };
    use jig_client::contract::*;
    use skein_lib::{Queue, Time};
    let (mut domain, _) = super::objects::live();
    drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Activity }) },
    );
    let id = domain.objects.name(ObjectKey::Task(8), Source::Window { number: 4 }).unwrap();
    let mut document = super::confirmation::task();
    document.acts = List::with_capacity(1);
    document.acts.push(Offered { effect: None, act: ActKind::Message, admission: Admission::Allowed }).unwrap();
    assert!(domain.objects.put(ObjectKey::Task(8), crate::objects::Body::Task(Box::new(document))));
    domain.objects.get_mut(id).unwrap().project = Some(1);
    let submitted = submit_words(&mut domain, b"first");
    let field = Purpose::Composer { task: 8 };
    let writes = domain.fields.get(field).unwrap().writes;
    assert_eq!(domain.conversation().words.rows.get(0).unwrap().state, jig_client::thread::Words::BeingSent);
    assert!(domain.conversation.words.begin(
        jig_client::Key { asker: jig_client::Asker::JIG, number: 999 },
        None,
        text(b"other pending")
    ));
    let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
    assert_eq!(
        crate::intents::send_card(
            &mut domain.ui,
            ObjectKey::Task(8),
            ActKind::Message,
            Box::new(Ask::Message { project: 1, task: 8, words: text(b"changed") }),
            &mut out,
            &domain.client
        ),
        Some(domain.intent(submitted))
    );
    assert!(drain(out).is_empty());
    assert_eq!(domain.conversation().words.rows.get(0).unwrap().words, text(b"first"));
    domain.conversation.words.refused(OTHER_PENDING);
    let answer = Answer::Done(Outcome::Owner(OwnerOutcome::Said { task: 8, message: 4 }));
    drive(&mut domain, Time::ZERO, AnsweredByEngine { request: submitted, answer: answer.clone() });
    assert_eq!(domain.conversation().words.rows.get(0).unwrap().state, jig_client::thread::Words::Sent { message: 4 });
    assert_eq!(domain.fields.get(field).unwrap().text.bytes(), b"");
    assert_eq!(domain.fields.get(field).unwrap().writes, writes + 1);
    drive(&mut domain, Time::ZERO, AnsweredByEngine { request: submitted, answer });
    assert_eq!(domain.fields.get(field).unwrap().writes, writes + 1);
    domain.intents.reclaim();
    routed_later_words(&mut domain);
}
fn submit_words(domain: &mut Fixture, words: &[u8]) -> jig_client::Key {
    use super::spine::drive;
    use crate::{Event, Request, action::Act, fields::Purpose};
    use skein_lib::Time;
    let field = Purpose::Composer { task: 8 };
    drive(domain, Time::ZERO, Event::Act { action: Act::Edit { field, text: text(words) } });
    drive(domain, Time::ZERO, Event::Act { action: Act::Submit { field } });
    for request in &drive(domain, Time::ZERO, Event::Act { action: Act::Confirm }) {
        if let Request::Client { asked: jig_client::Event::Send { key: request, .. } } = request {
            return *request;
        }
    }
    unreachable!("confirmed words send once")
}
fn routed_later_words(domain: &mut Fixture) {
    use super::spine::drive;
    use crate::{Event, action::Act, fields::Purpose};
    use jig_client::contract::*;
    use skein_lib::Time;
    let request = submit_words(domain, b"second");
    let field = Purpose::Composer { task: 8 };
    drive(domain, Time::ZERO, Event::Act { action: Act::Edit { field, text: text(b"later") } });
    let writes = domain.fields.get(field).unwrap().writes;
    drive(
        domain,
        Time::ZERO,
        AnsweredByEngine { request, answer: Answer::Done(Outcome::Owner(OwnerOutcome::Said { task: 8, message: 5 })) },
    );
    assert_eq!(domain.fields.get(field).unwrap().text.bytes(), b"later");
    assert_eq!(domain.fields.get(field).unwrap().writes, writes);
    assert!(card_words(domain).is_empty());
    let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
    assert_eq!(
        domain.objects().get(id).unwrap().card,
        crate::objects::Card::Refused { refusal: Refusal::Limit { which: Limit::ConversationWords { limit: 2 } } }
    );
    domain.conversation.words.read(5);
    assert_eq!(domain.conversation().words.rows.get(0).unwrap().state, jig_client::thread::Words::Read { message: 4 });
    assert_eq!(domain.conversation().words.rows.get(1).unwrap().state, jig_client::thread::Words::Read { message: 5 });
    routed_pending_restore(domain);
}
fn card_words(domain: &mut Fixture) -> Box<[Request]> {
    domain.drive(
        skein_lib::Time::ZERO,
        crate::tests::fixtures::CardAsk {
            object: ObjectKey::Task(8),
            act: jig_client::contract::ActKind::Message,
            ask: jig_client::contract::Ask::Message { project: 1, task: 8, words: text(b"program") },
        },
    )
}
fn routed_pending_restore(domain: &mut Fixture) {
    use super::spine::drive;
    use crate::{
        Event, Request,
        address::{Address, Place, TaskSection},
        intents::Progress,
    };
    use skein_lib::Time;
    let mut sent = None;
    for request in &card_words(domain) {
        if let Request::Client { asked: jig_client::Event::Send { key: request, .. } } = request {
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
    assert_eq!(
        domain.conversation().words.rows.len(),
        1,
        "page {:?}; confirmed {:?}; intents {:?}; task {}",
        domain.page,
        domain.confirmed,
        domain.intents,
        domain.conversation.task
    );
    let projected = domain.conversation();
    let row = projected.words.rows.get(0).unwrap();
    assert_eq!(
        row.request,
        Some(jig_client::Key { asker: jig_client::Asker::JIG, number: domain.intent(sent).raw() ^ domain.seed })
    );
    assert_eq!(row.words, text(b"program"));
    assert_eq!(row.state, jig_client::thread::Words::BeingSent);
    drive(domain, Time::ZERO, Reported { request: sent, progress: Progress::Unknown });
    assert!(domain.conversation().words.rows.is_empty());
    assert_eq!(domain.fields.get(crate::fields::Purpose::Composer { task: 8 }).unwrap().text.bytes(), b"later");
}

#[test]
fn calls_share_a_chip_watch_and_deliveries_keep_the_same_task_card_live() {
    use super::spine::{drive, open_token};
    use crate::{
        Event, Request,
        address::{Address, Place, TaskSection},
    };
    use jig_client::contract::*;
    use skein_lib::Time;
    let (mut domain, inbox) = super::objects::live();
    drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Activity }) },
    );
    drive(&mut domain, Time::ZERO, Ended { watch: inbox, end: WatchEnd::Closed });
    domain.reclaim();
    let mut rows = List::with_capacity(2);
    rows.push(calls(2, 2)).unwrap();
    rows.push(calls(1, 1)).unwrap();
    let page = ActivityPage { cut: false, rows, before: None, after: None };
    let read = domain.request_read(Source::Window { number: 1 }, Read::Activity { task: 8, before: None }).unwrap();
    let requests = drive(
        &mut domain,
        Time::ZERO,
        ReadPage { read, result: ReadResult::Value(ReadValue::Activity(Box::new(page))) },
    );
    let watch = open_token(&requests);
    let mut count = 0_u32;
    for request in &requests {
        if let Request::Client {
            asked: jig_client::Event::Watch { subject: jig_client::Watchable::Jig(Subject::Chips { tasks }), .. },
        } = request
        {
            assert_eq!(tasks.as_slice(), &[44]);
            count += 1;
        }
    }
    assert_eq!(count, 1);
    let id = domain.objects().find(ObjectKey::Task(44)).unwrap();
    for phase in [Phase::Running, Phase::Held] {
        let mut chips = List::with_capacity(1);
        let mut chip = super::objects::chip(44);
        chip.phase = phase;
        *chip.details = super::objects::phase_details(phase);
        chips.push(chip).unwrap();
        let requests =
            drive(&mut domain, Time::ZERO, Watched { watch, event: WatchEvent::Delivered(Delivery::Chips(chips)) });
        for request in &requests {
            if let Request::Client { asked: jig_client::Event::Watch { .. } } = request {
                unreachable!("repeated names share their watch");
            }
        }
        assert_eq!(domain.objects().find(ObjectKey::Task(44)), Some(id));
        if let Some(crate::objects::Body::Chip(chip)) = &domain.objects().get(id).unwrap().body {
            assert_eq!(chip.phase, phase);
        } else {
            unreachable!("chips update the shared task facts");
        }
    }
    let watch = changed_chip_membership(&mut domain, watch);
    drive(&mut domain, Time::ZERO, Ended { watch, end: WatchEnd::Unknown });
    domain.reclaim();
    let requests = drive(&mut domain, Time::ZERO, crate::tests::fixtures::Tick);
    for request in &requests {
        if let Request::Client { asked: jig_client::Event::Watch { .. } } = request {
            unreachable!("a final unavailable subject waits for different names");
        }
    }
}
fn changed_chip_membership(domain: &mut Fixture, old: jig_client::Handle) -> jig_client::Handle {
    use super::spine::drive;
    use crate::Request;
    use jig_client::contract::*;
    use skein_lib::Time;
    let mut rows = List::with_capacity(2);
    for position in [4_u32, 3] {
        let mut row = calls(u64::from(position), position);
        if let Activity::Turn { turn: TurnRead::Decoded(turn), .. } = &mut row.event
            && let Block::Call(call) = turn.messages.get_mut(0).unwrap().blocks.get_mut(0).unwrap()
        {
            call.objects.clear();
            call.objects.push(ObjectKey::Task(55)).unwrap();
        }
        rows.push(row).unwrap();
    }
    let what = domain.conversation.moved(4).unwrap();
    let read = domain.request_read(Source::Window { number: 1 }, what).unwrap();
    let requests = drive(
        domain,
        Time::ZERO,
        ReadPage {
            read,
            result: ReadResult::Value(ReadValue::Activity(Box::new(ActivityPage {
                cut: false,
                rows,
                before: None,
                after: None,
            }))),
        },
    );
    let mut closed = false;
    for request in &requests {
        if let Request::Client { asked: jig_client::Event::Unwatch { handle: watch } } = request {
            closed = *watch == old;
        }
    }
    assert!(closed);
    drive(domain, Time::ZERO, Ended { watch: old, end: WatchEnd::Closed });
    domain.reclaim();
    drive(domain, Time::ZERO, crate::tests::fixtures::Tick);
    let watch = domain.conversation().chips_watch.unwrap();
    assert_ne!(watch, old);
    assert_eq!(domain.conversation().chips.as_slice(), &[55]);
    watch
}

#[test]
fn owner_said_receipts_join_only_the_pending_task_and_preserve_later_composer_edits() {
    use super::spine::{LIMITS, drive};
    use crate::{
        Event,
        action::Act,
        address::{Address, Place, TaskSection},
        fields::Purpose,
    };
    use jig_client::contract::*;
    use skein_lib::Time;
    for later in [false, true] {
        {
            for actual_task in [8, 9] {
                let (mut domain, _) = super::objects::live();
                drive(
                    &mut domain,
                    Time::ZERO,
                    Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Activity }) },
                );
                let id = domain.objects.name(ObjectKey::Task(8), Source::Window { number: 4 }).unwrap();
                let mut document = super::confirmation::task();
                document.acts = List::with_capacity(1);
                document
                    .acts
                    .push(Offered { effect: None, act: ActKind::Message, admission: Admission::Allowed })
                    .unwrap();
                assert!(domain.objects.put(ObjectKey::Task(8), crate::objects::Body::Task(Box::new(document))));
                domain.objects.get_mut(id).unwrap().project = Some(1);
                let request = submit_words(&mut domain, b"submitted");
                let field = Purpose::Composer { task: 8 };
                if later {
                    drive(&mut domain, Time::ZERO, Event::Act { action: Act::Edit { field, text: text(b"later") } });
                }
                let writes = domain.fields.get(field).unwrap().writes;
                let answer = Answer::Done(Outcome::Owner(OwnerOutcome::Said { task: actual_task, message: u64::MAX }));
                drive(&mut domain, Time::ZERO, AnsweredByEngine { request, answer: answer.clone() });
                if actual_task == 8 {
                    assert_eq!(
                        domain.conversation().words.rows.get(0).unwrap().state,
                        jig_client::thread::Words::Sent { message: u64::MAX }
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
                assert_eq!(domain.objects().get(id).unwrap().answer.as_ref(), Some(&answer));
                let final_writes = domain.fields.get(field).unwrap().writes;
                assert!(drive(&mut domain, Time::ZERO, AnsweredByEngine { request, answer }).is_empty());
                assert_eq!(domain.fields.get(field).unwrap().writes, final_writes);
                assert_eq!(domain.conversation().words.rows.capacity(), LIMITS.conversation_words);
            }
        }
    }
}

#[test]
fn raw_live_calls_keep_optional_wall_literals_cut_and_full_identity_without_made_values() {
    use jig_client::contract::{CurrentCall, LiveCall};
    let (mut c, run) = native_run(4);
    let name = ProvisionalTurnName { attempt: u64::MAX, number: u32::MAX - 1, retry: 0 };
    let mut value = stream(name.number, 0, 3, b"one", false);
    value.attempt = name.attempt;
    value.turn = name;
    value.began = None;
    value.call = Some(LiveCall::Raw(CurrentCall {
        identity: current_identity(UTF8_TEXTS[0]),
        began: None,
        tool: text(UTF8_TEXTS[1]),
        input: text(UTF8_TEXTS[2]),
        cut: true,
    }));
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(value.clone())));
    let projected = c.conversation();
    let held = projected.live.get(0).unwrap();
    assert_eq!(held.began, None);
    assert_eq!(held.call, value.call);
    assert!(c.conversation().names(4).is_empty());
    assert!(c.conversation().parts(4).is_empty());
    let call = current_identity(UTF8_TEXTS[0]);
    assert!(fold_current(&mut c, call.clone(), Fold::Open));
    assert!(!fold_current(
        &mut c,
        jig_client::contract::CallIdentity { owner: skein_lib::Token::new(1), ..call.clone() },
        Fold::Open
    ));
    assert!(!fold_call(&mut c, CallName { attempt: name.attempt, turn: name.number, call: u64::MAX }, Fold::Open));
    value.began = Some(Wall::EPOCH);
    value.call = Some(LiveCall::Raw(CurrentCall {
        identity: current_identity(UTF8_TEXTS[0]),
        began: None,
        tool: text(UTF8_TEXTS[1]),
        input: text(b"new literal"),
        cut: false,
    }));
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(value.clone())));
    assert_eq!(c.conversation().live.get(0).unwrap().began, Some(Wall::EPOCH));
    assert_eq!(c.conversation().live.get(0).unwrap().call, value.call);
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"one");
    value.from = 1;
    value.through = 5;
    value.text = text(b"ne++");
    value.began = None;
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(value.clone())));
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"one++");
    assert_eq!(c.conversation().live.get(0).unwrap().began, None);
    assert_eq!(c.conversation().live.get(0).unwrap().call, value.call);
    let later = ProvisionalTurnName { number: u32::MAX, ..name };
    value.turn = later;
    value.from = 0;
    value.through = 4;
    value.text = text(b"next");
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(value)));
    assert_eq!(c.conversation().live.len(), 2);
    assert_eq!(c.conversation().live.get(1).unwrap().text.bytes(), b"next");
    assert!(c.ui.conversation.moved(1).is_some());
    let mut committed = calls(1, name.number);
    if let Activity::Turn { name: committed_name, .. } = &mut committed.event {
        *committed_name = name.turn();
    }
    let mut rows = List::with_capacity(1);
    rows.push(committed).unwrap();
    assert_eq!(c.ui.conversation.accept(ActivityPage { cut: false, rows, before: None, after: None }), None);
    assert_eq!(c.conversation().live.len(), 1);
    assert_eq!(c.conversation().live.get(0).unwrap().name, later);
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"next");
    c.ui.conversation.run_finished = true;
    c.ui.conversation.end_run();
    assert!(c.conversation().live.is_empty());
    let (reloaded, _) = native_run(4);
    assert!(reloaded.conversation().live.is_empty(), "provisional facts are read again after reload");
}

fn current_identity(id: &[u8]) -> jig_client::contract::CallIdentity {
    jig_client::contract::CallIdentity {
        owner: skein_lib::Token::new(u64::MAX),
        sequence: u32::MAX,
        retry: u32::MAX,
        id: text(id),
    }
}

#[test]
fn opaque_current_calls_replace_by_full_owner_completion_and_id_without_numeric_matching() {
    use jig_client::contract::{CallIdentity, CurrentCall, LiveCall};
    let (mut c, run) = native_run(4);
    let identity = current_identity(UTF8_TEXTS[3]);
    let mut value = stream(1, 0, 3, b"one", false);
    value.began = None;
    let raw = CurrentCall {
        identity: identity.clone(),
        tool: text(UTF8_TEXTS[4]),
        input: text(b"first"),
        cut: false,
        began: Some(Wall::EPOCH),
    };
    value.call = Some(LiveCall::Raw(raw.clone()));
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(value.clone())));
    assert!(fold_current(&mut c, identity.clone(), Fold::Open));
    assert_eq!(c.ui.conversation.current_fold(&identity), Fold::Open);
    value.call = Some(LiveCall::Raw(CurrentCall { input: text(b"replacement"), cut: true, ..raw.clone() }));
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(value.clone())));
    assert_eq!(c.conversation().live.len(), 1);
    assert_eq!(c.conversation().live.get(0).unwrap().call, value.call);
    assert_eq!(c.ui.conversation.current_fold(&identity), Fold::Open);
    assert_eq!(c.conversation().live.get(0).unwrap().began, None, "call first-seen is independent of turn wall");
    for different in [
        CallIdentity { owner: skein_lib::Token::new(1), ..identity.clone() },
        CallIdentity { sequence: 1, ..identity.clone() },
        CallIdentity { retry: 1, ..identity.clone() },
        CallIdentity { id: text(UTF8_TEXTS[5]), ..identity.clone() },
    ] {
        value.call = Some(LiveCall::Raw(CurrentCall { identity: different.clone(), began: None, ..raw.clone() }));
        feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(value.clone())));
        assert_eq!(c.conversation().live.get(0).unwrap().call, value.call);
        assert_eq!(c.ui.conversation.current_fold(&identity), Fold::Folded);
        assert_eq!(c.ui.conversation.current_fold(&different), Fold::Folded);
        assert!(!fold_current(&mut c, identity.clone(), Fold::Open));
        assert!(fold_current(&mut c, different.clone(), Fold::Open));
        assert_eq!(c.ui.conversation.current_opened.len(), 1);
        assert!(fold_current(&mut c, different, Fold::Folded));
        assert!(c.ui.conversation.current_opened.is_empty());
    }
    assert!(c.conversation().names(4).is_empty());
    assert!(c.conversation().parts(4).is_empty());
    assert!(!fold_call(&mut c, CallName { attempt: 1, turn: 1, call: 7 }, Fold::Open));
    assert!(c.ui.conversation.moved(1).is_some());
    let mut rows = List::with_capacity(1);
    rows.push(calls(1, 1)).unwrap();
    assert_eq!(c.ui.conversation.accept(ActivityPage { cut: false, rows, before: None, after: None }), None);
    assert!(c.conversation().live.is_empty());
    assert!(c.ui.conversation.current_opened.is_empty());
    assert!(fold_call(&mut c, CallName { attempt: 1, turn: 1, call: 7 }, Fold::Open));
}

#[test]
fn whole_run_state_keeps_call_only_and_postcommit_facts_without_a_synthetic_turn() {
    use jig_client::contract::{CurrentCall, LiveCall, RunState};
    let (mut c, run) = native_run(4);
    let identity = current_identity(UTF8_TEXTS[6]);
    let call = CurrentCall {
        identity: identity.clone(),
        tool: text(UTF8_TEXTS[4]),
        input: text(b"literal/\x00"),
        cut: true,
        began: Some(Wall::from_nanos(71)),
    };
    let mut state = RunState { task: 9, attempt: Some(1), committed_turn: 0, writing: None, call: Some(call.clone()) };
    feed_run(&mut c, run, jig_client::contract::Delivery::RunState(Box::new(state.clone())));
    assert_eq!(c.conversation().current_run.cloned(), Some(state.clone()));
    assert!(c.conversation().live.is_empty());
    assert_eq!(c.first_turn(), None);
    assert_eq!(
        match c.client.text(run) {
            Some(text) => text.through,
            None => 0,
        },
        0
    );
    assert!(fold_current(&mut c, identity.clone(), Fold::Open));
    assert!(!fold_current(&mut c, current_identity(b"different"), Fold::Open));
    assert!(c.conversation().names(4).is_empty());
    assert!(c.conversation().parts(4).is_empty());
    let mut writing = stream(1, 0, 3, b"one", false);
    writing.call = Some(LiveCall::Raw(call.clone()));
    state.writing = Some(writing.clone());
    state.committed_turn = 1;
    feed_run(&mut c, run, jig_client::contract::Delivery::RunState(Box::new(state.clone())));
    assert_eq!(c.conversation().live.len(), 1, "owner progress alone cannot hide unread committed text");
    assert_eq!(c.conversation().live.get(0).unwrap().name, writing.turn);
    assert!(c.ui.conversation.moved(1).is_some());
    let mut rows = List::with_capacity(1);
    rows.push(calls(1, 1)).unwrap();
    assert_eq!(c.ui.conversation.accept(ActivityPage { cut: false, rows, before: None, after: None }), None);
    assert!(c.conversation().live.is_empty(), "actual activity replaces only matching provisional text");
    assert_eq!(c.conversation().current_run.cloned(), Some(state.clone()), "a committed turn is not a tool-end report");
    assert_eq!(c.ui.conversation.current_fold(&identity), Fold::Open);
    state.writing = None;
    feed_run(&mut c, run, jig_client::contract::Delivery::RunState(Box::new(state.clone())));
    assert_eq!(c.conversation().current_run.cloned().as_ref().unwrap().call, Some(call));
    assert!(fold_call(&mut c, CallName { attempt: 1, turn: 1, call: 7 }, Fold::Open));
    assert_eq!(
        c.ui.conversation.current_fold(&identity),
        Fold::Open,
        "raw identity and transcript number remain separate"
    );
    state.attempt = Some(2);
    state.call = None;
    feed_run(&mut c, run, jig_client::contract::Delivery::RunState(Box::new(state)));
    assert!(!fold_current(&mut c, identity, Fold::Open));
    assert!(c.ui.conversation.current_opened.is_empty());
    let ended = RunState { task: 9, attempt: None, committed_turn: 1, writing: None, call: None };
    feed_run(&mut c, run, jig_client::contract::Delivery::RunState(Box::new(ended.clone())));
    assert_eq!(c.conversation().current_run.cloned(), Some(ended), "an absent attempt ends provisional state");
    assert_eq!(c.first_turn(), None);
    assert!(c.conversation().live.is_empty());
    c.ui.conversation.run_finished = true;
    c.ui.conversation.end_run();
    assert_eq!(c.conversation().current_run.cloned(), None);
}

#[test]
fn whole_current_call_replacement_copies_each_supplied_current_clock_without_a_history_ledger() {
    use jig_client::contract::{CurrentCall, RunState};
    let (mut c, run) = native_run(4);
    let a = CurrentCall {
        identity: current_identity(UTF8_TEXTS[7]),
        tool: text(b"tool"),
        input: text(b"input"),
        cut: false,
        began: Some(Wall::from_nanos(3)),
    };
    let mut value = RunState { task: 9, attempt: Some(1), committed_turn: 0, writing: None, call: Some(a.clone()) };
    feed_run(&mut c, run, jig_client::contract::Delivery::RunState(Box::new(value.clone())));
    assert!(fold_current(&mut c, a.identity.clone(), Fold::Open));
    value.call.as_mut().unwrap().input = text(b"replacement");
    feed_run(&mut c, run, jig_client::contract::Delivery::RunState(Box::new(value.clone())));
    assert_eq!(c.conversation().current_run.cloned(), Some(value.clone()));
    assert_eq!(c.ui.conversation.current_fold(&a.identity), Fold::Open);
    value.call.as_mut().unwrap().identity.retry = 1;
    value.call.as_mut().unwrap().began = Some(Wall::from_nanos(5));
    feed_run(&mut c, run, jig_client::contract::Delivery::RunState(Box::new(value.clone())));
    assert_eq!(c.conversation().current_run.cloned(), Some(value));
    assert_eq!(c.ui.conversation.current_fold(&a.identity), Fold::Folded);
    let returned = CurrentCall { began: Some(Wall::from_nanos(9)), ..a.clone() };
    let value = RunState { task: 9, attempt: Some(1), committed_turn: 0, writing: None, call: Some(returned) };
    feed_run(&mut c, run, jig_client::contract::Delivery::RunState(Box::new(value.clone())));
    assert_eq!(
        c.conversation().current_run.cloned(),
        Some(value),
        "A to B to A copies the supplied restarted current clock"
    );
    assert_eq!(c.ui.conversation.current_fold(&a.identity), Fold::Folded);
    assert!(c.first_turn().is_none());
    assert!(c.conversation().live.is_empty());
}

#[test]
fn mandatory_empty_retry_replaces_same_turn_and_stale_retries_cannot_restore_old_bytes() {
    let (mut c, run) = native_run(2);
    let old = stream(2, 0, 3, b"old", false);
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(old.clone())));
    let mut retry = stream(2, 0, 0, b"", false);
    retry.turn.retry = u32::MAX;
    retry.began = None;
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(retry.clone())));
    assert_eq!(c.conversation().live.len(), 1);
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"");
    assert_eq!(c.conversation().live.get(0).unwrap().name, retry.turn);
    assert_eq!(c.first_turn(), Some(retry.turn));
    assert_eq!(
        match c.client.text(run) {
            Some(text) => text.through,
            None => 0,
        },
        0
    );
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(old.clone())));
    assert_eq!(old.text.bytes(), b"old", "captured earlier delivery stays immutable");
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"");
    retry.text = text(b"new");
    retry.through = 3;
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(retry.clone())));
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"new");
    assert_eq!(
        match c.client.text(run) {
            Some(text) => text.through,
            None => 0,
        },
        3
    );
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(retry.clone())));
    assert_eq!(
        c.conversation().live.get(0).unwrap().text.bytes(),
        b"new",
        "exact retry/offset duplicate is absorbed once"
    );
    let mut invalid = retry.clone();
    invalid.attempt = 2;
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(invalid)));
    assert_eq!(c.first_turn(), Some(retry.turn));
    assert!(c.ui.conversation.moved(1).is_some());
    let mut rows = List::with_capacity(1);
    rows.push(calls(1, 2)).unwrap();
    assert_eq!(c.ui.conversation.accept(ActivityPage { cut: false, rows, before: None, after: None }), None);
    assert!(c.conversation().live.is_empty(), "committed attempt/turn replaces all provisional retries of that turn");
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(retry)));
    assert!(c.conversation().live.is_empty(), "a retry number cannot evade the committed turn fence");
}

#[test]
fn explicit_cut_and_reopened_origin_replace_retry_text_while_valid_suffixes_deduplicate() {
    let (mut c, run) = native_run(3);
    let mut value = stream(1, 0, 6, b"abcdef", false);
    value.turn.retry = 7;
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(value.clone())));
    value.from = 3;
    value.through = 8;
    value.text = text(b"deXYZ");
    value.cut = true;
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(value.clone())));
    assert_eq!(
        c.conversation().live.get(0).unwrap().text.bytes(),
        b"deXYZ",
        "cut never joins a retained prefix across a lost origin"
    );
    assert!(c.conversation().live.get(0).unwrap().cut);
    value.from = 5;
    value.through = 10;
    value.text = text(b"XYZ++");
    value.cut = false;
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(value.clone())));
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"deXYZ++");
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(value.clone())));
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"deXYZ++");
    let mut stale = value.clone();
    stale.from = 3;
    stale.through = 8;
    stale.text = text(b"stale");
    stale.cut = true;
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(stale)));
    assert_eq!(
        c.conversation().live.get(0).unwrap().text.bytes(),
        b"deXYZ++",
        "older captured cut document cannot move a retained position backward"
    );
    c.reopen(run);
    assert_eq!(
        c.conversation().live.get(0).unwrap().text.bytes(),
        b"deXYZ++",
        "brief interruption keeps the visible text"
    );
    value.from = 0;
    value.through = 4;
    value.text = text(b"boot");
    feed_run(&mut c, run, jig_client::contract::Delivery::Run(Box::new(value.clone())));
    assert_eq!(c.conversation().live.get(0).unwrap().text.bytes(), b"boot");
    assert!(!c.conversation().live.get(0).unwrap().cut);
    assert_eq!(c.first_turn(), Some(value.turn));
    assert_eq!(
        match c.client.text(run) {
            Some(text) => text.through,
            None => 0,
        },
        4
    );
}

const UTF8_TEXTS: [&[u8]; 8] =
    [b"id/~", b"raw/tool~", b"a/b\x00!", b"same/id~\x00", b"tool/!", b"same/id~\x01", b"opaque/~\x00", b"same/~"];

mod native;
use native::{feed_run, fold_call, fold_current, native_run, stream};
