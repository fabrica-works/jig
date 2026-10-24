//! Configured bounds hold a run at once (domain/tasks.md, 5.5 and 5.8).
use jig_domain_tasks::{
    Accepted, Active, Bound, Cause, Class, Control, End, EndKind, EndedBy, Event, Exceeded, Flags, Hold, Line,
    MessageKind, Party, Phase, RetryNotice, Stored, Tries, Was, Word,
};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::Duration;

const TASK: u64 = 1;
const REQUESTER: Party = Party::Person(1);
const FIRST: u64 = 1;
const NEXT: u64 = 2;
const MESSAGE: u64 = 1;
const SPENT: u64 = 3;
const BOUND: Exceeded = Exceeded { bound: Bound::Reasoning, value: 2048 };
const HELD: Phase = Phase::Held { was: Was::Active(Active::Due), why: Hold::Exceeded(BOUND), source: None };

fn claimed(flags: Flags) -> World {
    let mut w = World::new(337, LIMITS);
    let mut new = task(TASK, &[]);
    new.flags = flags;
    w.make(REQUESTER, vec![new]);
    w.claim(TASK, FIRST);
    w
}

fn release(w: &mut World) -> Reply {
    let reply_to = w.to();
    let call = reply_to.into_token().raw();
    w.send(Event::Control {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(call)),
        by: REQUESTER,
        task: TASK,
        control: Control::Release { again: None, message: None },
    });
    w.replies[&call].clone()
}

fn past_backoff(w: &mut World) {
    w.elapse(LIMITS.retries.transient.max.saturating_add(Duration::from_nanos(1)));
}

#[test]
fn a_run_that_meets_a_bound_is_held_at_once_without_a_try() {
    let mut w = claimed(Flags::PLAIN);
    assert_eq!(w.terminal(TASK, End::Exceeded(BOUND)), Reply::Acknowledged(Accepted::New));
    assert_eq!(w.record(TASK).phase, HELD);
    assert_eq!(w.record(TASK).tries, Tries::NONE);
    assert_eq!(w.record(TASK).refusals, 0);
    assert_eq!(w.record(TASK).retry_notice, None);
    past_backoff(&mut w);
    assert_eq!(w.record(TASK).phase, HELD);
    assert_eq!(w.activations(TASK), 1);
    assert!(!w.activations.contains(&TASK));
}

#[test]
fn a_bound_hold_clears_an_earlier_retry_notice_and_keeps_committed_work() {
    let mut w = claimed(Flags::PLAIN);
    w.terminal(TASK, End::Failed(Class::Transient));
    past_backoff(&mut w);
    assert!(matches!(w.record(TASK).retry_notice, Some(RetryNotice::Failed { .. })));
    w.claim(TASK, NEXT);
    let tries = w.record(TASK).tries;
    let reply_to = w.to();
    w.send(Event::Message {
        reply_to,
        project: 1,
        task: TASK,
        word: Word {
            number: MESSAGE,
            from: REQUESTER,
            kind: MessageKind::Words,
            words: b"hello".as_slice().into(),
            at: w.env.wall,
            hits: 1,
            eligible: true,
        },
    });
    let reply_to = w.to();
    w.send(Event::Turn {
        reply_to,
        task: TASK,
        attempt: NEXT,
        turn: 1,
        read: Some(MESSAGE),
        refused: Box::new([]),
        offered: Some(MESSAGE),
        outstanding: Box::new([]),
        cumulative: SPENT,
    });
    let replies = w.record(TASK).replies.clone();
    assert_eq!(
        w.terminal_cause(TASK, End::Exceeded(BOUND), Cause::Priced { cumulative: SPENT }),
        Reply::Acknowledged(Accepted::New)
    );
    let record = w.record(TASK);
    assert_eq!(record.phase, HELD);
    assert_eq!(record.tries, tries);
    assert_eq!(record.retry_notice, None);
    assert_eq!(record.numbers.spent, SPENT);
    assert_eq!(record.last_message, MESSAGE);
    assert_eq!(record.last_answer, Some(NEXT));
    assert_eq!(record.last_reply, 1);
    assert_eq!(record.replies, replies);
    assert!(w.records.values().any(|row| matches!(row, Stored::Activity(row)
    if row.task == TASK && row.line == Line::RunEnded {
        attempt: NEXT, end: EndKind::Failed { class: Class::Run }, spent: SPENT,
    })));
    past_backoff(&mut w);
    assert_eq!(w.record(TASK).phase, HELD);
    assert_eq!(w.activations(TASK), 2);
}

#[test]
fn a_released_bound_hold_runs_once_and_holds_again_on_the_same_bound() {
    let mut w = claimed(Flags::PLAIN);
    w.terminal(TASK, End::Exceeded(BOUND));
    assert_eq!(release(&mut w), Reply::Done);
    assert_eq!(w.record(TASK).phase, Phase::Active(Active::Due));
    assert_eq!(w.record(TASK).tries, Tries::NONE);
    w.claim(TASK, NEXT);
    assert_eq!(w.terminal(TASK, End::Exceeded(BOUND)), Reply::Acknowledged(Accepted::New));
    assert_eq!(w.record(TASK).phase, HELD);
    assert_eq!(w.record(TASK).tries, Tries::NONE);
    past_backoff(&mut w);
    assert_eq!(w.activations(TASK), 2);
    assert!(!w.activations.contains(&TASK));
}

#[test]
fn an_open_chat_held_on_a_bound_keeps_its_requester_ending() {
    let mut w = claimed(Flags { conversation: true, ended_by: EndedBy::Requester, ..Flags::PLAIN });
    w.terminal(TASK, End::Exceeded(BOUND));
    assert_eq!(w.record(TASK).phase, HELD);
    assert_eq!(w.record(TASK).flags.ended_by, EndedBy::Requester);
    assert_eq!(w.record(TASK).tries, Tries::NONE);
    assert!(w.results.is_empty());
    assert_eq!(w.activations(TASK), 1);
    release(&mut w);
    assert_eq!(w.record(TASK).flags.ended_by, EndedBy::Requester);
}

#[test]
fn a_bound_hold_survives_a_restart() {
    let mut w = claimed(Flags::PLAIN);
    w.terminal(TASK, End::Exceeded(BOUND));
    let before = w.records.clone();
    w.restart();
    assert_eq!(w.records, before);
    assert_eq!(w.record(TASK).phase, HELD);
    assert!(w.activations.is_empty());
    past_backoff(&mut w);
    assert_eq!(w.record(TASK).phase, HELD);
    assert_eq!(w.activations(TASK), 1);
}

#[test]
fn a_bound_hold_keeps_the_terminals_saved_resources() {
    let mut w = claimed(Flags::PLAIN);
    let saved = Box::new([jig_domain_tasks::SavedResource { connector: 1, path: Box::new([Box::from([3])]) }]);
    let reply_to = w.to();
    let call = reply_to.into_token().raw();
    w.send(Event::Activation {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(call)),
        task: TASK,
        attempt: FIRST,
        cause: Cause::Priced { cumulative: SPENT },
        end: End::Exceeded(BOUND),
        saved: Some(saved.clone()),
    });
    assert_eq!(w.replies[&call], Reply::Acknowledged(Accepted::New));
    assert_eq!(w.record(TASK).phase, HELD);
    assert_eq!(w.record(TASK).saved.as_ref(), saved.as_ref());
    assert_eq!(w.record(TASK).numbers.spent, SPENT);
}
