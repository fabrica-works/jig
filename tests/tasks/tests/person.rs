use jig_core_tasks::{self as tasks, Contract, Event, Executor, Party, PersonAddress, Phase, TaskResult, Verdict};
use jig_tasks_world::{LIMITS, Reply, World, task};

fn response(world: &mut World, event: impl FnOnce(skein_lib::ReplyTo) -> Event) -> Reply {
    let reply_to = world.to();
    world.send(event(reply_to));
    world.replies.last_key_value().expect("terminal reply").1.clone()
}

fn notified(world: &World, task: u64) -> &tasks::ActivityLine {
    world
        .records
        .values()
        .find_map(|record| match record {
            tasks::Stored::Activity(line) if line.task == task && matches!(line.line, tasks::Line::Notified { .. }) => {
                Some(line)
            }
            tasks::Stored::Activity(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::PersonProposal(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Live(_)
            | tasks::Stored::Ended(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Pool(_) => None,
        })
        .expect("immutable outward notification")
}

#[test]
fn a_person_task_is_taken_handed_back_and_answered() {
    let mut world = World::new(220, LIMITS);
    let mut new = task(1, &[]);
    new.executor = Executor::Person(PersonAddress::Role(7));
    new.contract = Contract::Verdict {
        choices: Box::new([Verdict { code: 1, words: 8, followups: 0 }, Verdict { code: 2, words: 8, followups: 0 }]),
    };
    assert_eq!(world.make(Party::Person(9), vec![new]), Reply::Made(vec![1]));
    assert_eq!(world.record(1).phase, Phase::Active(tasks::Active::Due));
    assert!(!world.activations.contains(&1), "a person task does not claim a worker");

    assert_eq!(response(&mut world, |reply_to| Event::TakePerson { reply_to, task: 1, person: 10 }), Reply::Done);
    assert_eq!(world.record(1).taken_by, Some(10));
    assert!(matches!(
        response(&mut world, |reply_to| Event::TakePerson { reply_to, task: 1, person: 11 }),
        Reply::Refused(_)
    ));
    world.restart();
    assert_eq!(world.record(1).taken_by, Some(10));
    assert_eq!(response(&mut world, |reply_to| Event::HandBackPerson { reply_to, task: 1, person: 10 }), Reply::Done);
    assert_eq!(world.record(1).taken_by, None);
    assert_eq!(response(&mut world, |reply_to| Event::TakePerson { reply_to, task: 1, person: 11 }), Reply::Done);
    assert!(matches!(
        response(&mut world, |reply_to| Event::AnswerPerson {
            reply_to,
            task: 1,
            person: 11,
            result: TaskResult::Verdict { code: 3, words: Box::new([]) },
        }),
        Reply::Refused(_)
    ));
    assert_eq!(
        response(&mut world, |reply_to| Event::AnswerPerson {
            reply_to,
            task: 1,
            person: 11,
            result: TaskResult::Verdict { code: 2, words: Box::new([1]) },
        }),
        Reply::Done
    );
    world.settle(1);
    assert!(matches!(world.results[&1], tasks::Ending::Done(TaskResult::Verdict { code: 2, .. })));
}

#[test]
fn a_direct_person_task_accepts_only_its_addressees_answer() {
    let mut limits = LIMITS;
    limits.person_stall = skein_lib::Duration::from_secs(1);
    let mut world = World::new(221, limits);
    let mut new = task(1, &[]);
    new.executor = Executor::Person(PersonAddress::Person(10));
    assert_eq!(world.make(Party::Person(9), vec![new]), Reply::Made(vec![1]));
    world.elapse(skein_lib::Duration::from_secs(1));
    let notice = world.record(1).person_wait.expect("answerable wait").notice.expect("personal requester was notified");
    let tasks::Line::Notified { recipient, word } = &notified(&world, 1).line else { panic!() };
    assert_eq!(*recipient, Party::Person(9));
    assert_eq!(word.number, notice.message);
    assert_eq!(word.kind, tasks::MessageKind::PersonStalled { task: 1 });
    assert_eq!(notified(&world, 1).position, notice.activity);
    assert!(matches!(
        response(&mut world, |reply_to| Event::AnswerPerson {
            reply_to,
            task: 1,
            person: 11,
            result: TaskResult::Report { words: Box::new([1]) },
        }),
        Reply::Refused(_)
    ));
    assert_eq!(
        response(&mut world, |reply_to| Event::AnswerPerson {
            reply_to,
            task: 1,
            person: 10,
            result: TaskResult::Report { words: Box::new([1]) },
        }),
        Reply::Done
    );
    world.settle(1);
    assert!(matches!(world.results[&1], tasks::Ending::Done(TaskResult::Report { .. })));
    world.restart();
    assert_eq!(notified(&world, 1).at, notice.at, "a person can read the notice after the waiting entry clears");
}

#[test]
fn a_persons_wait_starts_after_dependencies_and_notifies_its_requester_once_across_restart() {
    let mut limits = LIMITS;
    limits.person_stall = skein_lib::Duration::from_secs(10);
    let mut world = World::new(224, limits);
    assert_eq!(world.make(Party::Person(9), vec![task(1, &[])]), Reply::Made(vec![1]));
    let mut person = task(2, &[3]);
    person.executor = Executor::Person(PersonAddress::Role(7));
    assert_eq!(world.make(Party::Task(1), vec![person, task(3, &[])]), Reply::Made(vec![2, 3]));
    assert_eq!(world.record(2).person_wait, None);
    world.elapse(skein_lib::Duration::from_secs(5));
    world.claim_fresh(3);
    world.finish(3);
    world.settle(3);
    let wait = world.record(2).person_wait.expect("person becomes answerable after its dependency");
    assert_eq!(wait.since, skein_lib::Wall::from_nanos(5_000_000_000));
    assert_eq!(wait.deadline, skein_lib::Wall::from_nanos(15_000_000_000));
    assert_eq!(wait.stall, limits.person_stall);
    assert_eq!(wait.notice, None);
    world.send(Event::PersonStallNotice { task: 2, message: 100 });
    assert_eq!(world.record(2).person_wait, Some(wait), "a premature root notice cannot shorten the wait");
    world.elapse(skein_lib::Duration::from_secs(9));
    world.restart();
    assert_eq!(response(&mut world, |reply_to| Event::TakePerson { reply_to, task: 2, person: 10 }), Reply::Done);
    assert_eq!(response(&mut world, |reply_to| Event::HandBackPerson { reply_to, task: 2, person: 10 }), Reply::Done);
    assert_eq!(world.record(2).person_wait, Some(wait), "claims do not restart the question");
    world.send(Event::Hold { task: 2, why: tasks::Hold::StoppedBy { party: 9 } });
    world.elapse(skein_lib::Duration::from_secs(1));
    assert_eq!(world.record(2).person_wait, Some(wait), "held tasks do not send a stall notice");
    world.restart();
    assert_eq!(
        response(&mut world, |reply_to| Event::Control {
            reply_to,
            by: Party::Task(1),
            task: 2,
            control: tasks::Control::Release,
        }),
        Reply::Done
    );
    world.elapse(skein_lib::Duration::ZERO);
    let noticed = world.record(2).person_wait.expect("waiting facts retained");
    let notice = noticed.notice.expect("deadline issued a notice");
    assert_eq!(notice.at, wait.deadline);
    let delivered: Vec<_> = world
        .record(1)
        .inbox
        .iter()
        .filter(|word| word.kind == tasks::MessageKind::PersonStalled { task: 2 })
        .collect();
    assert_eq!(delivered.len(), 1);
    assert_eq!(delivered[0].number, notice.message);
    let tasks::Line::Notified { recipient, word } = &notified(&world, 2).line else { panic!() };
    assert_eq!(*recipient, Party::Task(1));
    assert_eq!(word, delivered[0]);
    assert_eq!(notified(&world, 2).position, notice.activity);
    assert!(
        world.records.values().any(|record| matches!(record,
            tasks::Stored::Activity(tasks::ActivityLine { task: 1, line: tasks::Line::Heard(news), .. })
                if news.number == notice.message
        )),
        "requester activity retains the admitted news"
    );
    world.restart();
    world.elapse(skein_lib::Duration::from_secs(1));
    assert_eq!(world.record(2).person_wait, Some(noticed));
    assert_eq!(
        world.record(1).inbox.iter().filter(|word| word.kind == tasks::MessageKind::PersonStalled { task: 2 }).count(),
        1
    );
}

#[test]
fn person_stall_news_and_the_result_keep_room_before_ordinary_words() {
    let mut limits = LIMITS;
    limits.inbox_messages = 2;
    limits.person_stall = skein_lib::Duration::from_secs(1);
    let mut world = World::new(225, limits);
    let mut parent = task(1, &[]);
    parent.wake.news = tasks::WakeRule::Never;
    assert_eq!(world.make(Party::Person(9), vec![parent]), Reply::Made(vec![1]));
    world.claim_fresh(1);
    assert_eq!(world.terminal(1, tasks::End::Parked), Reply::Acknowledged(tasks::Accepted::New));
    let mut person = task(2, &[]);
    person.executor = Executor::Person(PersonAddress::Person(10));
    assert_eq!(world.make(Party::Task(1), vec![person]), Reply::Made(vec![2]));
    let got = response(&mut world, |reply_to| Event::Message {
        reply_to,
        project: 1,
        task: 1,
        word: tasks::Word {
            number: 50,
            from: Party::Person(9),
            kind: tasks::MessageKind::Words,
            words: Box::new([1]),
            at: skein_lib::Wall::EPOCH,
            hits: 1,
            eligible: true,
        },
    });
    assert!(matches!(got, Reply::Refused(tasks::Problem { why: tasks::Refusal::Busy, .. })));
    world.elapse(skein_lib::Duration::from_secs(1));
    assert_eq!(
        world.record(1).phase,
        Phase::Active(tasks::Active::Idle),
        "stall news respects its requester's wake policy"
    );
    assert_eq!(
        response(&mut world, |reply_to| Event::AnswerPerson {
            reply_to,
            task: 2,
            person: 10,
            result: TaskResult::Report { words: Box::new([1]) }
        }),
        Reply::Done
    );
    world.settle(2);
    let inbox = &world.record(1).inbox;
    assert_eq!(inbox.len(), 2);
    assert!(inbox.iter().any(|word| word.kind == tasks::MessageKind::PersonStalled { task: 2 }));
    assert!(inbox.iter().any(|word| word.kind == tasks::MessageKind::Result(tasks::ResultKind::Report)));
}
