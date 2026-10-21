//! Open and finite runs after the same committed turn (domain/tasks.md, 5.8).
use jig_domain_tasks::{
    Accepted, Active, Amendment, Closing, Control, End, EndKind, EndedBy, Ending, Event, Executor, Flags, Hold,
    InvalidResult, Key, Line, MessageKind, Models, New, Party, PersonAddress, Phase, Problem, Refusal, Stage, Stored,
    TaskResult, Tries, Was, Word,
};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::Duration;

fn claimed(flags: Flags) -> World {
    let mut chat = task(1, &[]);
    chat.flags = flags;
    claimed_task(chat)
}

fn claimed_task(chat: New) -> World {
    let mut w = World::new(327, LIMITS);
    assert_eq!(w.make(Party::Person(1), vec![chat]), Reply::Made(vec![1]));
    w.claim(1, 1);
    w
}

fn commit_turn(w: &mut World, attempt: u64, read: Option<u64>) {
    let reply_to = w.to();
    let call = reply_to.into_token();
    w.send(Event::Turn {
        reply_to: skein_lib::ReplyTo::new(call),
        task: 1,
        attempt,
        turn: 1,
        read,
        refused: Box::new([]),
        offered: read,
        cumulative: 0,
    });
    assert_eq!(w.replies[&call.raw()], Reply::Turn(Accepted::New));
}

fn raw_finish() -> End {
    End::Finished { result: TaskResult::Report { words: b"done".as_slice().into() }, cancel_delegates: false }
}

fn replied(flags: Flags) -> World {
    let mut w = claimed(flags);
    commit_turn(&mut w, 1, None);
    w.terminal(
        1,
        End::Finished {
            result: TaskResult::Verdict { fields: Box::new([]), code: 0, words: Box::new([]) },
            cancel_delegates: false,
        },
    );
    w
}

fn committed_turn(w: &World) {
    assert_eq!(w.record(1).last_reply, 1);
    assert_eq!((w.record(1).attempt, w.record(1).turn), (1, 1));
    assert!(w.records.values().any(|row| matches!(row,
        Stored::Activity(row) if row.task == 1 && row.line == Line::Turn {
            reply: 1, attempt: 1, turn: 1, read: None,
        }
    )));
}

fn past_backoff(w: &mut World) {
    let retries = w.env.limits.retries;
    let longest = [retries.transient, retries.permanent, retries.run, retries.agent, retries.lost, retries.invalid]
        .into_iter()
        .map(|retry| retry.max)
        .max()
        .expect("retry classes");
    w.elapse(longest.saturating_add(Duration::from_nanos(1)));
}

fn say(w: &mut World, number: u64) {
    let reply_to = w.to();
    w.send(Event::Message {
        reply_to,
        project: 1,
        task: 1,
        word: Word {
            number,
            from: Party::Person(1),
            kind: MessageKind::Words,
            words: b"again".as_slice().into(),
            at: w.env.wall,
            hits: 1,
            eligible: true,
        },
    });
}

#[test]
fn an_open_chats_invalid_finish_after_its_reply_parks_once() {
    let mut w = replied(Flags { conversation: true, ended_by: EndedBy::Requester, ..Flags::PLAIN });
    committed_turn(&w);
    assert_eq!(w.record(1).tries, Tries::NONE);
    assert_eq!(w.record(1).invalid_result, None);
    assert_eq!(w.record(1).phase, Phase::Active(Active::Idle));
    assert_eq!(w.activations(1), 1);
    past_backoff(&mut w);
    assert_eq!(w.record(1).phase, Phase::Active(Active::Idle));
    assert_eq!(w.activations(1), 1);
    say(&mut w, 1);
    assert_eq!(w.record(1).phase, Phase::Active(Active::Due));
    assert_eq!(w.activations(1), 2);
    say(&mut w, 2);
    assert_eq!(w.activations(1), 2);
}

#[test]
fn a_finite_tasks_invalid_finish_after_its_reply_retries() {
    let mut w = replied(Flags::PLAIN);
    committed_turn(&w);
    assert_eq!(w.record(1).tries.invalid, 1);
    assert_eq!(w.record(1).invalid_result, Some(InvalidResult::Form));
    assert!(matches!(w.record(1).phase, Phase::Active(Active::BackingOff { .. })));
    assert_eq!(w.activations(1), 1);
    past_backoff(&mut w);
    assert_eq!(w.record(1).phase, Phase::Active(Active::Due));
    assert_eq!(w.activations(1), 2);
    assert_eq!(w.contexts[&1].invalid_result, Some(InvalidResult::Form));
}

#[test]
fn a_batchs_requester_authority_takes_one_shape() {
    const PARENT: u64 = 1;
    const MEMBER: u64 = 2;
    const PERSON: Party = Party::Person(7);
    const AGENT: Executor = Executor::Agent { charter: 1 };
    const OPEN: Flags = Flags { conversation: true, ended_by: EndedBy::Requester, ..Flags::PLAIN };
    let accepted = Reply::Made(vec![MEMBER]);
    let refused = Reply::Refused(Problem::new(Some(MEMBER), Refusal::Contract));
    for (name, flags, executor, creator, expected) in [
        ("person's agent conversation", OPEN, AGENT, PERSON, accepted.clone()),
        ("person executor", OPEN, Executor::Person(PersonAddress::Person(7)), PERSON, refused.clone()),
        ("procedure executor", OPEN, Executor::Procedure { connector: 1, code: 1 }, PERSON, refused.clone()),
        ("no conversation", Flags { conversation: false, ..OPEN }, AGENT, PERSON, refused.clone()),
        ("task creator", OPEN, AGENT, Party::Task(PARENT), refused.clone()),
        ("deployment creator", OPEN, AGENT, Party::Deployment { project: 1 }, refused.clone()),
        ("zero person", OPEN, AGENT, Party::Person(0), refused),
        ("result authority control", Flags::PLAIN, AGENT, PERSON, accepted),
    ] {
        let mut w = World::new(328, LIMITS);
        w.make(PERSON, vec![task(PARENT, &[])]);
        let before = w.live();
        let records = w.records.clone();
        let mut member = task(MEMBER, &[]);
        member.flags = flags;
        member.executor = executor;
        assert_eq!(w.make(creator, vec![member]), expected, "{name}");
        match expected {
            Reply::Made(_) => {
                assert_eq!(w.live(), before + 1, "{name}");
                assert_eq!(w.record(MEMBER).flags.ended_by, flags.ended_by, "{name}");
                assert_eq!(w.record(MEMBER).executor, executor, "{name}");
                assert_eq!(w.record(MEMBER).requester, creator, "{name}");
            }
            Reply::Refused(_) => {
                assert_eq!(w.live(), before, "{name}");
                assert_eq!(w.records, records, "{name}");
            }
            Reply::Done | Reply::Acknowledged(_) | Reply::Turn(_) | Reply::WriterWaiting(_) => {
                panic!("batch outcome")
            }
        }
    }
}

#[test]
fn an_open_task_held_from_active_keeps_open_and_rewraps_the_hold() {
    const CHAT: u64 = 1;
    const FIRST_ATTEMPT: u64 = 1;
    const NEXT_ATTEMPT: u64 = 2;
    let mut w = claimed(Flags { conversation: true, ended_by: EndedBy::Requester, ..Flags::PLAIN });
    commit_turn(&mut w, FIRST_ATTEMPT, None);
    let replies = w.record(CHAT).replies.clone();
    w.send(Event::Hold { task: CHAT, why: Hold::Stopped });
    assert_eq!(
        w.record(CHAT).phase,
        Phase::Held { was: Was::Active(Active::Running { attempt: FIRST_ATTEMPT }), why: Hold::Stopped, source: None }
    );
    assert_eq!(w.terminal(CHAT, raw_finish()), Reply::Acknowledged(Accepted::New));
    assert_eq!(w.record(CHAT).phase, Phase::Held { was: Was::Active(Active::Idle), why: Hold::Stopped, source: None });
    assert_eq!(w.record(CHAT).tries, Tries::NONE);
    assert_eq!(w.record(CHAT).last_answer, Some(FIRST_ATTEMPT));
    assert_eq!(w.record(CHAT).flags.ended_by, EndedBy::Requester);
    assert!(w.results.is_empty());
    assert_eq!(w.activations(CHAT), 1);
    assert_eq!(control(&mut w, Control::Release { again: None, message: None }), Reply::Done);
    assert_eq!(w.record(CHAT).phase, Phase::Active(Active::Due));
    assert_eq!(w.activations(CHAT), 2);
    assert_eq!(w.record(CHAT).tries, Tries::NONE);
    assert!(w.results.is_empty());
    committed_turn(&w);
    assert_eq!(w.record(CHAT).replies, replies);
    w.claim(CHAT, NEXT_ATTEMPT);
    assert_eq!(w.terminal(CHAT, raw_finish()), Reply::Acknowledged(Accepted::New));
    assert_eq!(w.record(CHAT).phase, Phase::Active(Active::Idle));
    assert_eq!(w.record(CHAT).tries, Tries::NONE);
    assert_eq!(w.record(CHAT).last_answer, Some(NEXT_ATTEMPT));
    assert_eq!(w.activations(CHAT), 2);
    assert_eq!(w.record(CHAT).last_reply, 1);
    assert_eq!(w.record(CHAT).replies, replies);
    assert!(w.results.is_empty());
}

fn control(w: &mut World, control: Control) -> Reply {
    let reply_to = w.to();
    let call = reply_to.into_token().raw();
    w.send(Event::Control {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(call)),
        by: Party::Person(1),
        task: 1,
        control,
    });
    w.replies[&call].clone()
}

fn run_ended(w: &World, attempt: u64, end: EndKind) {
    assert!(w.records.values().any(|row| matches!(row,
        Stored::Activity(row) if row.task == 1 && row.line == Line::RunEnded { attempt, end, spent: 0 }
    )));
}

fn cancelled(w: &World, ending: &Ending, at: skein_lib::Wall, partial: Option<TaskResult>) {
    assert_eq!(
        ending,
        &Ending::Cancelled { by: Party::Person(1), at, reason: b"stop".as_slice().into(), result: partial }
    );
    assert_eq!(w.results.len(), 1);
    let Stored::Ended(row) = &w.records[&Key::Ended(1)] else { panic!("ended task record") };
    assert_eq!(row.phase, Phase::Ended(ending.clone()));
    assert_eq!(row.tries, Tries::NONE);
    assert_eq!(w.live(), 0);
}

#[test]
fn an_open_raw_finish_while_narrowing_goes_due() {
    const CHAT: u64 = 1;
    const ATTEMPT: u64 = 1;
    let mut chat = task(CHAT, &[]);
    chat.flags = Flags { conversation: true, ended_by: EndedBy::Requester, ..Flags::PLAIN };
    chat.authority.models = Models::Named(0b11);
    let mut w = claimed_task(chat);
    commit_turn(&mut w, ATTEMPT, None);
    let replies = w.record(CHAT).replies.clone();
    let mut authority = w.record(CHAT).authority.clone();
    authority.models = Models::Named(0b01);
    let reply_to = w.to();
    let call = reply_to.into_token().raw();
    w.send(Event::Amend {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(call)),
        funder: None,
        by: Party::Person(1),
        task: CHAT,
        message: 1,
        stop_run: true,
        amendment: Amendment {
            tracked: None,
            addressed: None,
            instructions: None,
            release: None,
            spec: None,
            wake: None,
            dependencies: None,
            authority: Some(authority),
            llm: None,
            reason: b"narrow".as_slice().into(),
        },
    });
    assert_eq!(w.replies[&call], Reply::Done);
    assert!(w.record(CHAT).narrowing);
    assert!(w.stops.contains(&(CHAT, ATTEMPT)));
    assert_eq!(w.terminal(CHAT, raw_finish()), Reply::Acknowledged(Accepted::New));
    assert_eq!(w.record(CHAT).phase, Phase::Active(Active::Due));
    assert_eq!(w.record(CHAT).authority.models, Models::Named(0b01));
    assert!(!w.record(CHAT).narrowing);
    assert_eq!(w.record(CHAT).tries, Tries::NONE);
    assert_eq!(w.record(CHAT).last_answer, Some(ATTEMPT));
    assert_eq!(w.record(CHAT).replies, replies);
    committed_turn(&w);
    assert!(w.results.is_empty());
    assert!(!w.records.contains_key(&Key::Ended(CHAT)));
    assert_eq!(w.activations(CHAT), 2);
    run_ended(&w, ATTEMPT, EndKind::Cancelled);
}

#[test]
fn an_open_task_held_from_closing_drops_open() {
    const CHAT: u64 = 1;
    const ATTEMPT: u64 = 1;
    let mut w = claimed(Flags { conversation: true, ended_by: EndedBy::Requester, ..Flags::PLAIN });
    commit_turn(&mut w, ATTEMPT, None);
    let at = w.env.wall;
    assert_eq!(
        control(&mut w, Control::Cancel { reason: b"stop".as_slice().into(), grace: None, message: None }),
        Reply::Done
    );
    assert!(matches!(w.record(CHAT).phase, Phase::Closing(Closing { stage: Stage::Run { attempt: ATTEMPT }, .. })));
    w.send(Event::Hold { task: CHAT, why: Hold::Stopped });
    assert!(matches!(w.record(CHAT).phase, Phase::Held { was: Was::Closing(_), .. }));
    assert_eq!(w.terminal(CHAT, raw_finish()), Reply::Acknowledged(Accepted::New));
    let partial = TaskResult::Report { words: b"done".as_slice().into() };
    let ending = Ending::Cancelled {
        by: Party::Person(1),
        at,
        reason: b"stop".as_slice().into(),
        result: Some(partial.clone()),
    };
    assert_eq!(
        w.record(CHAT).phase,
        Phase::Held {
            was: Was::Closing(Closing { stage: Stage::Delegates, ending: ending.clone() }),
            why: Hold::Stopped,
            source: None,
        }
    );
    assert_eq!(w.record(CHAT).tries, Tries::NONE);
    assert_eq!(w.record(CHAT).last_answer, Some(ATTEMPT));
    committed_turn(&w);
    assert!(w.results.is_empty());
    assert_eq!(w.activations(CHAT), 1);
    assert_eq!(control(&mut w, Control::Release { again: None, message: None }), Reply::Done);
    w.settle(CHAT);
    cancelled(&w, &w.results[&CHAT], at, Some(partial));
    let Stored::Ended(row) = &w.records[&Key::Ended(CHAT)] else { panic!("ended task record") };
    assert_eq!(row.last_reply, 1);
    run_ended(&w, ATTEMPT, EndKind::Cancelled);
}
