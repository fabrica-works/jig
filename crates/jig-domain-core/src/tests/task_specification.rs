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
    let task = task_from_row(&core, &limits, 1, &row).unwrap();
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
    let task = task_from_row(&core, &limits, 1, &row).unwrap();
    assert!(task.spec.cut, "only parameters exceed the spec list bound");
    assert_eq!(task.spec.parameters.len(), 2);
    assert_eq!(task.spec.parameters.get(0), Some(&Parameter::Number { name: 7, value: 8 }));
    assert_eq!(task.spec.parameters.get(1), Some(&Parameter::Number { name: 9, value: 10 }));
    assert!(task.spec.inputs.is_empty());
    assert!(!task.spec.words.cut);

    row.spec.parameters = Box::new([]);
    row.spec.inputs = Box::new([u64::MAX, 1, 3]);
    let task = task_from_row(&core, &limits, 1, &row).unwrap();
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
    let task = task_from_row(&core, &limits, 1, &row).unwrap();
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
