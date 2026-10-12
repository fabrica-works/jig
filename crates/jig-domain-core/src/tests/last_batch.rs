//! Project reads the exact retained batch rather than current delegate membership.
use super::*;

#[test]
fn project_reads_the_whole_last_created_batch_after_ending_and_refuses_a_partial_list() {
    let (mut core, mut limits) = tree_core();
    let (authenticated, auth_limits) = watch_core();
    core.people = authenticated.people;
    limits.people = auth_limits.people;
    let mut parent = new_row(&core, 100, tasks::Funder::Period { project: 1, period: 1 }, None);
    parent.executor = tasks::Executor::Procedure { connector: RECURRING, code: 1 };
    parent.numbers = tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 };
    parent.authority.delegation.kinds = Box::new([tasks::AuthorityExecutor::Charter(1)]);
    let mut first = new_row(&core, 1, tasks::Funder::Task(100), None);
    first.authority.delegation.kinds = Box::new([]);
    let mut second = first.clone();
    second.number = 2;
    parent.recurring = Some(Box::new(tasks::RecurringTemplate {
        key: 1,
        batch: Box::new([first, second]),
        overlap: tasks::RecurringOverlap::Skip,
    }));
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(1)),
            creator: tasks::Party::Deployment { project: 1 },
            batch: Box::new([parent]),
        },
    );
    send_task(&mut core, &limits, tasks::Event::TickRecurring { task: 100, period: 1 });
    send_task(&mut core, &limits, tasks::Event::RecurringBatch { task: 100, period: 1, numbers: Box::new([101, 102]) });
    assert_eq!(core.tasks.task(100).unwrap().recurring.as_ref().unwrap().last_batch.as_ref(), [101, 102]);
    for child in [101, 102] {
        send_task(
            &mut core,
            &limits,
            tasks::Event::Control {
                reply_to: ReplyTo::new(Token::new(1)),
                by: tasks::Party::Task(100),
                task: child,
                control: tasks::Control::Cancel { reason: Box::new([]) },
            },
        );
        send_task(&mut core, &limits, tasks::Event::EffectsSettled { task: child });
        send_task(&mut core, &limits, tasks::Event::Settled { task: child });
        assert!(core.tasks.task(child).is_none());
    }
    assert!(core.tasks.task(100).unwrap().delegates.is_empty());
    let Document::Project(project) = make(&core, &limits, 1, &Subject::Project { project: 1 }, &window()).unwrap()
    else {
        panic!("project")
    };
    let line = project.recurring.get(0).unwrap();
    assert_eq!(line.last_batch.len(), 2);
    assert_eq!(line.last_batch.get(0), Some(&101));
    assert_eq!(line.last_batch.get(1), Some(&102));
    limits.documents.lines = 1;
    assert_eq!(make(&core, &limits, 1, &Subject::Project { project: 1 }, &window()), Err(Missing::TooLarge));
}
