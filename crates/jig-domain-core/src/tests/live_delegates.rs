//! Actual task admission, charging, moving, document bounds and watched delivery.
use super::*;

#[test]
fn live_delegate_documents_keep_every_identity_and_the_owner_takeover_reservation() {
    let (mut core, mut limits) = make_tree();
    let mut grandchild = new_row(&core, 5, tasks::Funder::Task(2), None);
    grandchild.numbers.budget = 3;
    grandchild.authority.budget.spend = 3;
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(1)),
            creator: tasks::Party::Task(2),
            batch: Box::new([grandchild]),
        },
    );
    send_task(&mut core, &limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(1)), task: 2 });
    send_task(
        &mut core,
        &limits,
        tasks::Event::Claim {
            account: None,
            reply_to: ReplyTo::new(Token::new(1)),
            task: 2,
            attempt: 1,
            resumed: false,
            budget: 6,
            writes: Box::new([]),
        },
    );
    send_task(
        &mut core,
        &limits,
        tasks::Event::Turn {
            refused: Box::new([]),
            reply_to: ReplyTo::new(Token::new(1)),
            task: 2,
            attempt: 1,
            turn: 1,
            read: None,
            offered: None,
            outstanding: Box::new([]),
            cumulative: 2,
        },
    );
    let row = core.tasks.task(2).unwrap();
    assert_eq!(row.numbers.spent, 2);
    assert_eq!(row.numbers.reserved, 7);
    assert_eq!(core.tasks.takeover_budget(2), Some(8));
    let Document::Task(doc) = make(&core, &limits, 1, &Subject::Task { task: 1 }, &window(), Wall::EPOCH).unwrap()
    else {
        panic!("task")
    };
    assert_eq!(doc.delegates.len(), 3);
    assert_eq!(doc.delegates.get(0).unwrap().chip.number, 2);
    assert_eq!(doc.delegates.get(1).unwrap().chip.number, 3);
    assert_eq!(doc.delegates.get(2).unwrap().chip.number, 4);
    assert_eq!(doc.delegates.get(0).unwrap().takeover_budget, 8);
    limits.documents.lines = 2;
    assert_eq!(make(&core, &limits, 1, &Subject::Task { task: 1 }, &window(), Wall::EPOCH), Err(Missing::TooLarge));
    limits.documents.lines = 8;
    send_task(
        &mut core,
        &limits,
        tasks::Event::Move {
            reply_to: ReplyTo::new(Token::new(1)),
            task: 2,
            person: 1,
            period: 1,
            pool_budget: 500,
            period_budget: 1000,
            reason: Box::new([]),
        },
    );
    let moved = core.tasks.task(2).unwrap();
    assert_eq!(moved.requester, tasks::Party::Person(1));
    assert_eq!(moved.numbers.budget, 8);
    assert_eq!(
        core.tasks.funding(tasks::Funder::Pool { project: 1, person: 1, period: 1 }).unwrap().numbers.reserved,
        8
    );
    let Document::Task(doc) = make(&core, &limits, 1, &Subject::Task { task: 1 }, &window(), Wall::EPOCH).unwrap()
    else {
        panic!("task")
    };
    assert_eq!(doc.delegates.len(), 2);
    assert_eq!(doc.delegates.get(0).unwrap().chip.number, 3);
    assert_eq!(doc.delegates.get(1).unwrap().chip.number, 4);
}

#[test]
fn a_delegate_change_refreshes_its_requesters_task_document() {
    let (mut core, mut limits) = make_tree();
    let (authenticated, auth_limits) = watch_core();
    core.people = authenticated.people;
    limits.people = auth_limits.people;
    finish_watch_outputs(&mut core, &limits);
    watch_outputs(
        &mut core,
        &limits,
        Event::Watch { watcher: Token::new(30), sign_in: 10, subject: Watched::Task { task: 1 }, window: window() },
    );
    watch_outputs(&mut core, &limits, Event::Delivered { watcher: Token::new(30) });
    watch_outputs(&mut core, &limits, Event::Tasks(tasks::Event::Hold { task: 2, why: tasks::Hold::Budget }));
    let out = finish_watch_outputs(&mut core, &limits);
    let mut delivered = false;
    for request in &out {
        let Request::Held(held) = request else { continue };
        let Held::Delivery { watcher, document, .. } = held.as_ref() else { continue };
        if *watcher == Token::new(30) {
            let Document::Task(task) = document.as_ref() else { panic!("task") };
            assert_eq!(
                task.delegates.get(0).unwrap().chip.phase,
                PhaseLine::Held { reason: tasks::Hold::Budget, revision: core.tasks.task(2).unwrap().revisions.hold }
            );
            delivered = true;
        }
    }
    assert!(delivered);
}
