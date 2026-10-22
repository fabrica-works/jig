//! What a task waits on, from its live subtree and the entries in it.
use super::*;
use fixtures::{make_batch, restore};

fn pair() -> (Core, Limits) {
    let (mut core, limits) = tree_core();
    let parent = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
    make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([parent]));
    let child = new_row(&core, 2, tasks::Funder::Task(1), None);
    make_batch(&mut core, &limits, tasks::Party::Task(1), Box::new([child]));
    (core, limits)
}

fn question(number: u64, at: u64) -> tasks::Word {
    tasks::Word {
        number,
        from: tasks::Party::Task(2),
        kind: tasks::MessageKind::Question,
        words: Box::from(&b"Question?"[..]),
        at: Wall::from_nanos(at),
        hits: 1,
        eligible: true,
    }
}

#[test]
fn a_task_with_live_delegates_and_no_entries_waits_on_its_delegates() {
    let (core, limits) = pair();
    assert_eq!(core.waits_on(&limits, 1), WaitsOn::Delegates);
}

#[test]
fn a_question_to_the_requester_names_that_party_and_one_entry() {
    let (mut core, limits) = pair();
    let mut parent = core.tasks.task(1).unwrap().clone();
    parent.inbox = Box::new([question(3, 10)]);
    parent.last_message = 3;
    parent.question_lines = Box::new([tasks::QuestionLine { message: 3, line: parent.activity, answer: None }]);
    let child = core.tasks.task(2).unwrap().clone();
    restore(&mut core, &limits, Box::new([parent, child]));
    let position = core.tasks.task(1).unwrap().activity;
    assert!(core.remember_question(&tasks::ActivityLine {
        task: 1,
        position,
        at: Wall::from_nanos(10),
        line: tasks::Line::Words {
            message: 3,
            by: tasks::Party::Task(2),
            kind: tasks::MessageKind::Question,
            words: b"Question?".as_slice().into()
        }
    }));
    assert_eq!(
        core.waits_on(&limits, 1),
        WaitsOn::Party {
            whom: people::Whom::Person(1),
            entries: 1,
            task: 1,
            kind: people::EntryKind::Question { message: 3 },
        }
    );
}

#[test]
fn the_oldest_of_two_waiting_entries_names_whom() {
    let (mut core, limits) = pair();
    let mut parent = core.tasks.task(1).unwrap().clone();
    parent.inbox = Box::new([question(3, 10)]);
    parent.last_message = 3;
    parent.question_lines = Box::new([tasks::QuestionLine { message: 3, line: parent.activity, answer: None }]);
    let mut child = core.tasks.task(2).unwrap().clone();
    child.revisions.hold = 1;
    child.escalation = tasks::Escalation::Waiting {
        revision: 1,
        holder: tasks::EscalationHolder::Person(2),
        entry: 4,
        since: Wall::from_nanos(9),
    };
    child.phase =
        tasks::Phase::Held { why: tasks::Hold::Budget, source: None, was: tasks::Was::Active(tasks::Active::Due) };
    restore(&mut core, &limits, Box::new([parent, child]));
    let position = core.tasks.task(1).unwrap().activity;
    assert!(core.remember_question(&tasks::ActivityLine {
        task: 1,
        position,
        at: Wall::from_nanos(10),
        line: tasks::Line::Words {
            message: 3,
            by: tasks::Party::Task(2),
            kind: tasks::MessageKind::Question,
            words: b"Question?".as_slice().into()
        }
    }));
    assert_eq!(
        core.waits_on(&limits, 1),
        WaitsOn::Party {
            whom: people::Whom::Person(2),
            entries: 2,
            task: 2,
            kind: people::EntryKind::Escalation { revision: 1 },
        }
    );
}

#[test]
fn a_proposal_below_says_the_party_is_waited_on_for_that_proposal() {
    let (mut core, limits) = pair();
    let parent = core.tasks.task(1).unwrap().clone();
    let mut child = core.tasks.task(2).unwrap().clone();
    child.revision = 1;
    child.proposal = Some(Box::new(tasks::Proposal {
        number: 4,
        proposer: 2,
        project: 1,
        action: tasks::ProposalAction::Release { task: 1 },
        reason: Box::new([]),
        as_holder: false,
        state: tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Person(1), since: Wall::EPOCH },
    }));
    restore(&mut core, &limits, Box::new([parent, child]));
    assert_eq!(
        core.waits_on(&limits, 1),
        WaitsOn::Party {
            whom: people::Whom::Person(1),
            entries: 1,
            task: 2,
            kind: people::EntryKind::Proposal { number: 4 },
        }
    );
}

#[test]
fn a_person_task_below_names_its_role() {
    let (mut core, limits) = pair();
    let parent = core.tasks.task(1).unwrap().clone();
    let mut child = core.tasks.task(2).unwrap().clone();
    child.executor = tasks::Executor::Person(tasks::PersonAddress::Role(1));
    child.person_wait = Some(tasks::PersonWait {
        since: Wall::EPOCH,
        stall: Duration::from_secs(1),
        deadline: Wall::from_nanos(1_000_000_000),
        notice: None,
    });
    restore(&mut core, &limits, Box::new([parent, child]));
    assert_eq!(
        core.waits_on(&limits, 1),
        WaitsOn::Party {
            whom: people::Whom::Role { project: 1, role: 1 },
            entries: 1,
            task: 2,
            kind: people::EntryKind::PersonTask,
        }
    );
}

#[test]
fn an_idle_task_with_nothing_below_waits_on_news() {
    let (mut core, limits) = pair();
    let mut child = core.tasks.task(2).unwrap().clone();
    child.phase = tasks::Phase::Active(tasks::Active::Idle);
    restore(&mut core, &limits, Box::new([child]));
    assert_eq!(core.waits_on(&limits, 2), WaitsOn::News);
}

#[test]
fn a_working_task_with_live_delegates_waits_on_nothing() {
    let (mut core, limits) = pair();
    let mut parent = core.tasks.task(1).unwrap().clone();
    parent.phase = tasks::Phase::Active(tasks::Active::Running { attempt: 1 });
    parent.attempt = 1;
    parent.run_since = Some(Wall::EPOCH);
    let child = core.tasks.task(2).unwrap().clone();
    restore(&mut core, &limits, Box::new([parent, child]));
    assert_eq!(core.waits_on(&limits, 1), WaitsOn::Nothing);
}

#[test]
fn a_task_not_live_waits_on_nothing() {
    let (core, limits) = pair();
    assert_eq!(core.waits_on(&limits, 99), WaitsOn::Nothing);
}

#[test]
fn a_kept_idle_row_waits_on_dependencies_before_delegates_and_cuts_oldest_first() {
    let (mut core, mut limits) = tree_core();
    let parent = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
    make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([parent]));
    let older = new_row(&core, 3, tasks::Funder::Period { project: 1, period: 1 }, None);
    fixtures::make_batch_at(&mut core, &limits, tasks::Party::Person(1), Box::new([older]), Wall::from_nanos(10));
    let newer = new_row(&core, 2, tasks::Funder::Task(1), None);
    fixtures::make_batch_at(&mut core, &limits, tasks::Party::Task(1), Box::new([newer]), Wall::from_nanos(20));
    limits.documents.lines = 1;
    let mut row = core.tasks.task(1).unwrap().clone();
    row.phase = tasks::Phase::Active(tasks::Active::Idle);
    row.dependencies = Box::new([2, 3]);
    row.waiting_on = Box::new([2, 3]);
    let WaitsOn::Dependencies { pending, count } = crate::waits::from_row(&core, &limits, &row) else { panic!("kept") };
    assert_eq!(pending.as_slice(), &[3]);
    assert_eq!(count, 2);
    row.phase = tasks::Phase::Active(tasks::Active::Running { attempt: 1 });
    assert_eq!(crate::waits::from_row(&core, &limits, &row), WaitsOn::Nothing);
}
