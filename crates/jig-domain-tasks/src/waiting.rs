//! The live inbox and pending decisions in message-number order (tasks.md, 8.2 and 9).
use crate::Word;
use crate::domain::{Domain, record};

pub(crate) fn next(domain: &Domain, task: u64, after: Option<u64>) -> Option<Word> {
    let task_record = record(domain, task)?;
    let mut next = None;
    for word in &task_record.inbox {
        consider(&mut next, word, after);
    }
    for word in crate::proposals::waiting_for(domain, task) {
        consider(&mut next, &word, after);
    }
    for word in crate::escalation::waiting_for(domain, task) {
        consider(&mut next, &word, after);
    }
    next
}

fn consider(next: &mut Option<Word>, word: &Word, after: Option<u64>) {
    if let Some(after) = after
        && word.number <= after
    {
        return;
    }
    if let Some(current) = next
        && current.number <= word.number
    {
        return;
    }
    *next = Some(word.clone());
}
