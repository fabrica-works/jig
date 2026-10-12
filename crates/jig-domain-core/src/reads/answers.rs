//! A question joins its answer where it was heard (protocol/store.md, 7.3).
use jig_domain_tasks as tasks;

pub(super) fn matches(owner: &tasks::TaskRecord, message: u64, answer: &tasks::ActivityLine) -> bool {
    for line in &owner.question_lines {
        if line.message == message {
            return line.answer == Some(tasks::AnswerLine { task: answer.task, position: answer.position });
        }
    }
    false
}
