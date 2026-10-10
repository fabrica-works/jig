//! Concrete executor facts remain available in live and standalone retained chips.
use super::*;

#[test]
fn retained_chips_preserve_each_executor_identity_and_full_role_project_context() {
    let (core, limits) = make_tree();
    let mut row = core.tasks.task(2).expect("owner row").clone();
    row.project = u32::MAX;
    row.phase = tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: Box::new([]) }));
    row.ended_at = Some(Wall::from_nanos(77));
    for (executor, kind) in [
        (tasks::Executor::Agent { charter: u32::MAX }, ExecutorKind::Agent),
        (tasks::Executor::Procedure { connector: u16::MAX, code: u32::MAX }, ExecutorKind::Procedure),
        (tasks::Executor::Person(tasks::PersonAddress::Person(u64::MAX)), ExecutorKind::Person),
        (tasks::Executor::Person(tasks::PersonAddress::Role(u32::MAX)), ExecutorKind::Person),
    ] {
        row.executor = executor;
        let chip = chip_from_row(&limits, &row).expect("retained chip");
        assert_eq!(chip.executor, kind);
        assert_eq!(chip.details.executor, executor);
        assert_eq!(chip.details.project, u32::MAX);
        assert_eq!(chip.details.observation, None, "a retained row supplies no host observation");
        assert_eq!(chip.details.ended_at, row.ended_at);
        let ended = ended_delegate_from_row(&limits, &row).expect("retained delegate");
        assert_eq!(ended.chip.details.executor, executor);
        assert_eq!(ended.chip.details.project, u32::MAX);
    }
}

#[test]
fn live_task_tree_and_named_chips_keep_the_executors_actual_owner_identity() {
    let (core, limits) = make_tree();
    let mut names = List::with_capacity(limits.documents.chips);
    names.push(2).expect("one named chip");
    let Document::Chips(chips) = make(&core, &limits, 1, &Subject::Chips { tasks: names }, &window()).expect("chips")
    else {
        panic!("named chips")
    };
    let row = core.tasks.task(2).expect("actual admitted row");
    let chip = chips.get(0).expect("named chip");
    assert_eq!(chip.details.executor, tasks::Executor::Agent { charter: 1 });
    assert_eq!(chip.details.project, 1);
    let Document::Task(task) = make(&core, &limits, 1, &Subject::Task { task: 2 }, &window()).expect("task") else {
        panic!("task document")
    };
    assert_eq!(task.chip.details.executor, row.executor);
    assert_eq!(task.chip.details.project, task.project);
    let Document::Tree(tree) = make(&core, &limits, 1, &Subject::Tree { task: 1 }, &window()).expect("tree") else {
        panic!("tree document")
    };
    assert_eq!(tree.root.details.executor, core.tasks.task(1).expect("root owner").executor);
    for child in &tree.rows {
        let owner = core.tasks.task(child.chip.number).expect("child owner");
        assert_eq!(child.chip.details.executor, owner.executor);
        assert_eq!(child.chip.details.project, owner.project);
    }
}
