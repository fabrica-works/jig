//! Open and finite runs after the same committed turn (domain/tasks.md, 5.8).
use jig_domain_tasks::{
    Accepted, Active, End, EndedBy, Event, Executor, Flags, InvalidResult, Line, MessageKind, Party, PersonAddress,
    Phase, Problem, Refusal, Stored, TaskResult, Tries, Word,
};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::Duration;

fn replied(flags: Flags) -> World {
    let mut w = World::new(327, LIMITS);
    let mut chat = task(1, &[]);
    chat.flags = flags;
    assert_eq!(w.make(Party::Person(1), vec![chat]), Reply::Made(vec![1]));
    w.claim(1, 1);
    let reply_to = w.to();
    let call = reply_to.into_token();
    w.send(Event::Turn {
        reply_to: skein_lib::ReplyTo::new(call),
        task: 1,
        attempt: 1,
        turn: 1,
        read: None,
        refused: Box::new([]),
        offered: None,
        cumulative: 0,
    });
    assert_eq!(w.replies[&call.raw()], Reply::Turn(Accepted::New));
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
