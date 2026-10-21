//! Open and finite runs after the same committed turn (domain/tasks.md, 5.8).
use jig_domain_tasks::{
    Accepted, Active, End, EndedBy, Event, Flags, InvalidResult, Line, MessageKind, Party, Phase, Stored, TaskResult,
    Tries, Word,
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
