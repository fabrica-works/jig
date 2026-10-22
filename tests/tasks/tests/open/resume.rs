//! Open resumes as Finite; only the next finish differs (domain/tasks.md, 5.5–5.8, 6.7 and 7–9).
use super::*;
use jig_domain_tasks::{Class, EscalationDecision, EscalationHolder, Operation, ReplyHint};
use skein_lib::{List, ReplyTo};

const CHAT: u64 = 1;
const DEPENDENCY: u64 = 2;
const PERSON: u64 = 1;
const FIRST_ATTEMPT: u64 = 1;
const NEXT_ATTEMPT: u64 = 2;
const DEPENDENCY_ATTEMPT: u64 = 3;
const ENTRY: u64 = 100;
const OPEN: Flags = Flags { conversation: true, ended_by: EndedBy::Requester, ..Flags::PLAIN };
const FINITE: Flags = Flags { conversation: true, ..Flags::PLAIN };
// The making activates once; a resume to Due delivers the second preparation.
const RESUMED: u32 = 2;
// Recheck delivers the second preparation; restart delivers the third.
const RESTORED: u32 = 3;
const POOL_BUDGET: u64 = 1_000;
const PERIOD_BUDGET: u64 = 10_000;

#[derive(Clone, Copy)]
struct Resume {
    phase: Active,
    tries: Tries,
    refusals: u32,
    activations: u32,
}

const DUE: Resume = Resume { phase: Active::Due, tries: Tries::NONE, refusals: 0, activations: RESUMED };
const IDLE: Resume = Resume { phase: Active::Idle, activations: 1, ..DUE };

enum Finish {
    Parks,
    Ends,
}

fn unchanged_reply(w: &World, replies: &[ReplyHint]) {
    assert_eq!(w.record(CHAT).last_reply, 1);
    assert_eq!(w.record(CHAT).replies.as_ref(), replies);
}

fn resumed(w: &World, expected: Resume, replies: &[ReplyHint]) {
    assert_eq!(w.record(CHAT).phase, Phase::Active(expected.phase));
    assert_eq!(w.record(CHAT).tries, expected.tries);
    assert_eq!(w.record(CHAT).refusals, expected.refusals);
    assert_eq!(w.activations(CHAT), expected.activations);
    assert!(!w.results.contains_key(&CHAT));
    assert!(!w.records.contains_key(&Key::Ended(CHAT)));
    assert!(!w.records.values().any(|row| matches!(row, Stored::Result(row) if row.task == CHAT)));
    committed_turn(w);
    unchanged_reply(w, replies);
}

fn twins(resume: impl Fn(&mut World), expected: Resume, wake: fn(&mut World)) {
    for (flags, finish) in [(OPEN, Finish::Parks), (FINITE, Finish::Ends)] {
        let mut w = claimed(flags);
        commit_turn(&mut w, FIRST_ATTEMPT, None);
        let replies = w.record(CHAT).replies.clone();
        let (turn_key, turn) = w
            .records
            .iter()
            .find(|(_, row)| matches!(row, Stored::Activity(row) if row.task == CHAT && matches!(row.line, Line::Turn { .. })))
            .map(|(key, row)| (*key, row.clone()))
            .expect("committed reply activity");
        resume(&mut w);
        resumed(&w, expected, &replies);
        assert_eq!(w.record(CHAT).flags, flags);
        assert_eq!(w.records[&turn_key], turn);
        wake(&mut w);
        let activations = w.activations(CHAT);
        w.claim(CHAT, NEXT_ATTEMPT);
        // Consume notices delivered by the resume before checking the raw finish.
        if let Some(read) = w.record(CHAT).inbox.last().map(|word| word.number) {
            commit_turn(&mut w, NEXT_ATTEMPT, Some(read));
        }
        let replies = w.record(CHAT).replies.clone();
        let last_reply = w.record(CHAT).last_reply;
        assert_eq!(w.terminal(CHAT, raw_finish()), Reply::Acknowledged(Accepted::New));
        match finish {
            Finish::Parks => {
                assert_eq!(w.record(CHAT).phase, Phase::Active(Active::Idle));
                assert_eq!(w.record(CHAT).tries, Tries::NONE);
                assert_eq!(w.record(CHAT).refusals, 0);
                assert_eq!(w.record(CHAT).last_answer, Some(NEXT_ATTEMPT));
                assert_eq!(w.record(CHAT).last_reply, last_reply);
                assert_eq!(w.record(CHAT).replies, replies);
                assert!(!w.results.contains_key(&CHAT));
                assert!(!w.records.contains_key(&Key::Ended(CHAT)));
                assert!(!w.records.values().any(|row| matches!(row, Stored::Result(row) if row.task == CHAT)));
                run_ended(&w, NEXT_ATTEMPT, EndKind::Parked);
            }
            Finish::Ends => {
                let ending = Ending::Done(TaskResult::Report { words: b"done".as_slice().into() });
                w.settle(CHAT);
                assert_eq!(w.results[&CHAT], ending);
                let Stored::Ended(row) = &w.records[&Key::Ended(CHAT)] else { panic!("finite ended record") };
                assert_eq!(row.phase, Phase::Ended(ending));
                assert_eq!(row.tries, expected.tries);
                assert_eq!(row.refusals, expected.refusals);
                assert_eq!(row.last_reply, last_reply);
                assert_eq!(row.replies, replies);
            }
        }
        assert_eq!(w.activations(CHAT), activations);
        assert_eq!(w.records[&turn_key], turn);
    }
}

fn already_due(_: &mut World) {}

fn wake_idle(w: &mut World) {
    let message = w.record(CHAT).last_message + 1;
    assert_eq!(say(w, message), Reply::Done);
}

#[test]
fn an_open_task_resumes_from_the_retry_alarm_and_still_parks() {
    for (end, tries, refusals) in
        [(End::Failed(Class::Transient), Tries { transient: 1, ..Tries::NONE }, 0), (End::Refused, Tries::NONE, 1)]
    {
        twins(
            |w| {
                assert_eq!(w.terminal(CHAT, end.clone()), Reply::Acknowledged(Accepted::New));
                assert!(matches!(w.record(CHAT).phase, Phase::Active(Active::BackingOff { .. })));
                assert_eq!(w.activations(CHAT), 1);
                past_backoff(w);
            },
            Resume { tries, refusals, ..DUE },
            already_due,
        );
    }
}

#[test]
fn an_open_task_resumes_from_restore_and_still_parks() {
    // Recheck makes Due without a try; restore reissues its preparation.
    twins(restore_due, Resume { activations: RESTORED, ..DUE }, already_due);
    twins(restore_preparing, Resume { activations: RESTORED, ..DUE }, already_due);
}

fn restore_due(w: &mut World) {
    recheck(w);
    w.restart();
}

fn restore_preparing(w: &mut World) {
    recheck(w);
    let reply_to = w.to();
    w.send(Event::Prepare { reply_to, task: CHAT });
    w.restart();
}

fn park(w: &mut World) {
    assert_eq!(w.terminal(CHAT, End::Parked), Reply::Acknowledged(Accepted::New));
}

fn hold(w: &mut World) {
    w.send(Event::Hold { task: CHAT, why: Hold::Stopped });
}

fn release(w: &mut World) {
    assert_eq!(control(w, Control::Release { again: None, message: None }), Reply::Done);
}

#[test]
fn an_open_task_resumes_from_escalation_and_still_parks() {
    for (before_hold, expected, wake) in [
        (park as fn(&mut World), IDLE, wake_idle as fn(&mut World)),
        (recheck, Resume { activations: RESTORED, ..DUE }, already_due),
    ] {
        twins(|w| escalation(w, before_hold), expected, wake);
    }
}

fn recheck(w: &mut World) {
    assert_eq!(w.terminal(CHAT, End::Recheck), Reply::Acknowledged(Accepted::New));
}

fn escalation(w: &mut World, before_hold: fn(&mut World)) {
    before_hold(w);
    hold(w);
    let revision = w.record(CHAT).escalation.revision();
    w.send(Event::RoutedEscalation { task: CHAT, revision, holder: EscalationHolder::Person(PERSON), entry: ENTRY });
    let reply_to = w.to();
    w.stage(Event::DecideEscalation {
        reply_to,
        task: CHAT,
        revision,
        by: PERSON,
        entry: None,
        decision: EscalationDecision::Release,
    });
    // As in kept.rs: crash after durability, before the root consumes the decision.
    w.durable();
    w.restart();
}

fn amendment(dependencies: &[u64], release: Option<u64>) -> Amendment {
    Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        release,
        spec: None,
        wake: None,
        dependencies: Some(dependencies.into()),
        authority: None,
        llm: None,
        reason: Box::new([]),
    }
}

fn amend(w: &mut World, task: u64, dependencies: &[u64]) {
    let reply_to = w.to();
    let call = reply_to.into_token();
    let message = w.record(task).last_message + 1;
    w.send(Event::Amend {
        places: Box::new([]),
        reply_to: ReplyTo::new(call),
        by: Party::Person(PERSON),
        task,
        message,
        stop_run: false,
        amendment: amendment(dependencies, None),
        funder: None,
    });
    assert_eq!(w.replies[&call.raw()], Reply::Done);
}

fn dependency_done(w: &mut World) {
    w.claim(DEPENDENCY, DEPENDENCY_ATTEMPT);
    w.finish(DEPENDENCY);
    w.settle(DEPENDENCY);
}

#[test]
fn an_open_task_resumes_from_the_met_wake_and_still_parks() {
    twins(met, DUE, already_due);
}

fn met(w: &mut World) {
    assert_eq!(w.make(Party::Person(PERSON), vec![task(DEPENDENCY, &[])]), Reply::Made(vec![DEPENDENCY]));
    amend(w, CHAT, &[DEPENDENCY]);
    park(w);
    dependency_done(w);
    let notice = w.record(CHAT).inbox.iter().find(|word| matches!(word.kind, MessageKind::Met { .. })).expect("Met");
    assert_eq!(notice.kind, MessageKind::Met { dependency: DEPENDENCY, removed: false });
    assert_eq!(notice.from, Party::Task(DEPENDENCY));
    assert_eq!(notice.hits, 1);
    assert!(notice.words.is_empty());
    assert!(w.record(CHAT).waiting_on.is_empty());
}

#[test]
fn an_open_task_resumes_from_the_circle_lift_and_still_parks() {
    twins(circle, IDLE, dependency_done);
}

fn circle(w: &mut World) {
    park(w);
    hold(w);
    assert_eq!(w.make(Party::Person(PERSON), vec![task(DEPENDENCY, &[CHAT])]), Reply::Made(vec![DEPENDENCY]));
    amend(w, CHAT, &[DEPENDENCY]);
    release(w);
    assert!(matches!(w.record(CHAT).phase, Phase::Held { was: Was::Active(Active::Idle), why: Hold::Circle, .. }));
    amend(w, DEPENDENCY, &[]);
    assert_eq!(w.record(CHAT).waiting_on.as_ref(), [DEPENDENCY]);
}

fn compound(w: &mut World, operations: Vec<Operation>) {
    let bound = u32::try_from(operations.len()).expect("fixture operations");
    let mut staged = List::with_capacity(bound);
    for operation in operations {
        staged.push(operation).expect("bounded fixture");
    }
    let reply_to = w.to();
    let call = reply_to.into_token();
    w.send(Event::Compound {
        reply_to: ReplyTo::new(call),
        person: PERSON,
        project: 1,
        operations: staged,
        period: 0,
        pool_budget: POOL_BUDGET,
        period_budget: PERIOD_BUDGET,
    });
    assert_eq!(w.replies[&call.raw()], Reply::Done);
}

#[test]
fn an_open_task_resumes_from_staged_release_and_still_parks() {
    twins(staged_release, DUE, already_due);
    twins(staged_amendment_release, DUE, already_due);
}

fn staged_release(w: &mut World) {
    park(w);
    hold(w);
    let message = w.record(CHAT).last_message + 1;
    compound(
        w,
        vec![
            Operation::Release { task: CHAT, again: None, message: None },
            Operation::Amend { task: CHAT, message, stop_run: false, amendment: Box::new(amendment(&[], None)) },
        ],
    );
}

fn staged_amendment_release(w: &mut World) {
    park(w);
    hold(w);
    let message = w.record(CHAT).last_message + 1;
    let release = Some(w.record(CHAT).revisions.hold);
    compound(
        w,
        vec![
            Operation::Amend { task: CHAT, message, stop_run: false, amendment: Box::new(amendment(&[], release)) },
            Operation::Message {
                task: CHAT,
                word: Word {
                    number: message + 1,
                    from: Party::Person(PERSON),
                    kind: MessageKind::Words,
                    words: b"again".as_slice().into(),
                    at: w.env.wall,
                    hits: 1,
                    eligible: true,
                },
            },
        ],
    );
}
