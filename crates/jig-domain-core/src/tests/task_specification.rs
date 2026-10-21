//! Specification source facts through the public task document constructor.
use super::*;

const UTF8_WORDS: &[u8] = b"~\0^\x01";

#[test]
fn task_specification_keeps_source_inputs_and_separate_page_loss() {
    let (core, mut limits) = make_tree();
    limits.documents.lines = 2;
    let mut row = core.tasks.task(2).unwrap().clone();
    row.spec.inputs = Box::new([1, u64::MAX]);
    row.dependencies = Box::new([91, 92, 93]);
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    assert!(task.cut, "the dependency list omits its third row");
    assert!(!task.spec.cut, "the entire specification still fits");
    assert_eq!(task.spec.words, task.words);
    assert_eq!(task.spec.parameters, task.parameters);
    assert_eq!(task.spec.inputs.len(), 2);
    assert_eq!(task.spec.inputs.get(0), Some(&1));
    assert_eq!(task.spec.inputs.get(1), Some(&u64::MAX));
    let TaskLink::Live(input) = task.inputs.get(0).unwrap() else { panic!("live input") };
    assert_eq!(input.number, 1);
    assert_eq!(task.inputs.get(1), Some(&TaskLink::Stored { number: u64::MAX, status: None }));
    assert_eq!(task.dependencies.len(), 2);
    assert_eq!(core.tasks.task(2).unwrap().spec.inputs.len(), 0, "projection does not mutate the owner");
}

#[test]
fn task_specification_marks_parameter_and_input_list_loss_independently() {
    let (core, mut limits) = make_tree();
    limits.documents.lines = 2;
    let mut row = core.tasks.task(2).unwrap().clone();
    row.spec.parameters = Box::new([
        tasks::Parameter::Number { name: 7, value: 8 },
        tasks::Parameter::Number { name: 9, value: 10 },
        tasks::Parameter::Number { name: 11, value: 12 },
    ]);
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    assert!(task.spec.cut, "only parameters exceed the spec list bound");
    assert_eq!(task.spec.parameters.len(), 2);
    assert_eq!(task.spec.parameters.get(0), Some(&Parameter::Number { name: 7, value: 8 }));
    assert_eq!(task.spec.parameters.get(1), Some(&Parameter::Number { name: 9, value: 10 }));
    assert!(task.spec.inputs.is_empty());
    assert!(!task.spec.words.cut);

    row.spec.parameters = Box::new([]);
    row.spec.inputs = Box::new([u64::MAX, 1, 3]);
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    assert!(task.spec.cut, "only inputs exceed the spec list bound");
    assert!(task.spec.parameters.is_empty());
    assert_eq!(task.spec.inputs.len(), 2);
    assert_eq!(task.spec.inputs.get(0), Some(&u64::MAX));
    assert_eq!(task.spec.inputs.get(1), Some(&1));
    assert!(!task.spec.words.cut);
}

#[test]
fn task_specification_preserves_parameter_identity_and_local_text_loss() {
    let (core, mut limits) = make_tree();
    limits.documents.lines = 3;
    limits.documents.text_bytes = 3;
    let mut row = core.tasks.task(2).unwrap().clone();
    row.spec.words = Box::from(&b"abcdef"[..]);
    row.spec.parameters = Box::new([
        tasks::Parameter::Bytes { name: u32::MAX, value: Box::from(UTF8_WORDS) },
        tasks::Parameter::Number { name: 2, value: u64::MAX },
        tasks::Parameter::Resource { name: 3, connector: u16::MAX, resource: u64::MAX },
    ]);
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    assert!(!task.spec.cut, "text loss does not omit specification list rows");
    assert!(!task.cut, "text loss is local to each text");
    assert_eq!(task.spec.words, Text { bytes: Box::from(&b"abc"[..]), cut: true });
    assert_eq!(
        task.spec.parameters.get(0),
        Some(&Parameter::Bytes { name: u32::MAX, value: Text { bytes: Box::from(&UTF8_WORDS[..3]), cut: true } })
    );
    assert_eq!(task.spec.parameters.get(1), Some(&Parameter::Number { name: 2, value: u64::MAX }));
    assert_eq!(
        task.spec.parameters.get(2),
        Some(&Parameter::Resource {
            name: 3,
            part: Part { connector: u16::MAX, kind: PartKind::Resource, key: PartKey::Number(u64::MAX) }
        })
    );
    assert_eq!(task.spec.parameters, task.parameters);
}

#[test]
fn task_dependencies_show_added_provenance_pending_and_stub_status() {
    const DEPENDENCY: u64 = 7;
    let (mut core, limits) = make_tree();
    let mut row = core.tasks.task(2).unwrap().clone();
    let parent = core.tasks.task(1).unwrap().clone();
    fixtures::restore(&mut core, &limits, Box::new([parent, row.clone()]));
    row.phase = tasks::Phase::Active(tasks::Active::Idle);
    row.dependencies = Box::new([1, DEPENDENCY]);
    row.waiting_on = Box::new([1]);
    let added = tasks::Added { task: DEPENDENCY, at: Wall::from_nanos(11), by: tasks::Party::Person(1) };
    row.added = Box::new([added.clone()]);
    send_task(
        &mut core,
        &limits,
        tasks::Event::Restore {
            record: tasks::Stored::Stub(tasks::Stub {
                task: DEPENDENCY,
                phase: tasks::Status::Done,
                result: Token::new(DEPENDENCY),
            }),
        },
    );
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    assert_eq!(task.dependency_facts.get(0).unwrap().number, 1);
    assert!(task.dependency_facts.get(0).unwrap().pending);
    assert!(task.dependency_facts.get(0).unwrap().keeps);
    assert_eq!(task.dependency_facts.get(1).unwrap().added, Some(added));
    assert_eq!(task.dependency_facts.get(1).unwrap().status, Some(tasks::Status::Done));
    assert!(!task.dependency_facts.get(1).unwrap().pending);
    row.waiting_on = Box::new([]);
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    assert!(!task.dependency_facts.get(0).unwrap().keeps);
}

#[test]
fn a_kept_person_card_offers_no_answer_and_keeps_its_take() {
    let (core, limits) = make_tree();
    let mut row = core.tasks.task(2).unwrap().clone();
    row.executor = tasks::Executor::Person(tasks::PersonAddress::Role(people::Role::Owner.number()));
    row.phase = tasks::Phase::Active(tasks::Active::Idle);
    row.dependencies = Box::new([1]);
    row.waiting_on = Box::new([1]);
    let card = person_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    let mut take = false;
    for offer in card.acts.as_slice() {
        assert_ne!(offer.act, crate::acts::Act::Answer);
        if offer.act == crate::acts::Act::Take {
            take = true;
        }
    }
    assert!(take);
}

#[test]
fn dependency_refusals_keep_the_problem_names_at_the_people_boundary() {
    let (core, _) = make_tree();
    for (why, expected) in [
        (tasks::Refusal::Circle { cut: true }, people::Refusal::Circle { tasks: Box::new([7, 5, 9]), cut: true }),
        (tasks::Refusal::Kept, people::Refusal::Kept { pending: Box::new([7, 5, 9]) }),
        (
            tasks::Refusal::DependencyEnded { how: tasks::Status::Failed },
            people::Refusal::Dependency { dependency: 7, how: people::DependencyStatus::Failed },
        ),
    ] {
        let problem = tasks::Problem::naming(Some(2), why, &[7, 5, 9]);
        assert_eq!(routing::refusals::person_problem_refusal(&core, problem), people::Outcome::Refused(expected));
    }
}

#[test]
fn a_circle_hold_projection_cuts_the_chips_without_losing_the_source_cut() {
    let (core, mut limits) = make_tree();
    limits.documents.lines = 2;
    let mut row = core.tasks.task(2).unwrap().clone();
    row.revisions.hold = 1;
    row.phase = tasks::Phase::Held {
        was: tasks::Was::Waiting,
        why: tasks::Hold::Circle,
        source: Some(Box::new(tasks::HoldSource::Circle(tasks::Circle { tasks: Box::new([9, 7, 5]), cut: false }))),
    };
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    let circle = task.hold.unwrap().circle.unwrap();
    assert_eq!(circle.tasks.as_ref(), &[9, 7]);
    assert!(circle.cut);
}
