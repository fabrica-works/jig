use super::*;
pub const COMPOUND_PROJECT: u32 = 3;
const COMPOUND_TASK: u64 = 8;
pub fn compound_operations() -> [Operation; 2] {
    let limits = limits();
    [
        Operation::Message(
            Message::new(
                &limits,
                MessageParts { project: COMPOUND_PROJECT, task: COMPOUND_TASK, words: b"words".as_slice().into() },
            )
            .unwrap(),
        ),
        Operation::Release(
            Release::new(
                &limits,
                ReleaseParts {
                    project: COMPOUND_PROJECT,
                    task: COMPOUND_TASK,
                    revision: Revision::new(&limits, RevisionParts { value: 4 }).unwrap(),
                    again: None,
                },
            )
            .unwrap(),
        ),
    ]
}
