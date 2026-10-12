//! Positions for single reads, retained with the task (protocol/store.md, 15.7).
use crate::{
    Change, Domain, History, Limits, MessageKind, Party, Proposal, ProposalState, Request, Stored, TaskRecord, Word,
};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue};

/// A question this task heard and the positions of its original lines.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct QuestionLine {
    pub message: u64,
    pub line: u64,
    pub answer: Option<AnswerLine>,
}

/// The task and position where a question's first answer was heard.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AnswerLine {
    pub task: u64,
    pub position: u64,
}

/// The history revision retaining one decided task proposal.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ProposalRow {
    pub proposal: u64,
    pub revision: u64,
}

pub(crate) fn message(task: &mut TaskRecord, word: &Word, bound: u32) {
    if word.kind == MessageKind::Question {
        let mut kept = List::with_capacity(bound);
        kept.push(QuestionLine { message: word.number, line: task.activity, answer: None }).expect("one question fits");
        for line in &task.question_lines {
            if kept.room() == 0 {
                break;
            }
            kept.push(*line).expect("bounded question positions");
        }
        task.question_lines = kept.into_boxed();
    }
    match word.kind {
        MessageKind::Answer { question } => match word.from {
            Party::Person(_) => {
                let _first = answer(task, question, AnswerLine { task: task.number, position: task.activity });
            }
            Party::Task(_) | Party::Deployment { .. } => {}
        },
        MessageKind::Words
        | MessageKind::Question
        | MessageKind::EffectNotMade { .. }
        | MessageKind::Escalation { .. }
        | MessageKind::Proposal { .. }
        | MessageKind::ProposalDecision { .. }
        | MessageKind::Amendment { .. }
        | MessageKind::Notice { .. }
        | MessageKind::Timer { .. }
        | MessageKind::News { .. }
        | MessageKind::PersonStalled { .. }
        | MessageKind::Result(_) => {}
    }
}

pub(crate) fn proposal(task: &mut TaskRecord, number: u64, bound: u32) {
    let mut kept = List::with_capacity(bound);
    kept.push(ProposalRow { proposal: number, revision: task.revision }).expect("one proposal fits");
    for row in &task.proposal_rows {
        if kept.room() == 0 {
            break;
        }
        if row.proposal != number {
            kept.push(*row).expect("bounded proposal positions");
        }
    }
    task.proposal_rows = kept.into_boxed();
}

pub(crate) fn valid(task: &TaskRecord, bound: u32) -> bool {
    let bound = usize::try_from(bound).expect("u32 fits usize");
    if task.hold > task.milestone || task.question_lines.len() > bound || task.proposal_rows.len() > bound {
        return false;
    }
    let mut previous = u64::MAX;
    for (index, line) in task.question_lines.iter().enumerate() {
        if line.message == 0
            || line.message > task.last_message
            || line.line == 0
            || line.line > task.activity
            || (index != 0 && line.line >= previous)
        {
            return false;
        }
        if let Some(answer) = line.answer
            && (answer.task == 0
                || answer.position == 0
                || (answer.task == task.number && (answer.position <= line.line || answer.position > task.activity)))
        {
            return false;
        }
        for old in task.question_lines.iter().take(index) {
            if old.message == line.message {
                return false;
            }
        }
        previous = line.line;
    }
    previous = u64::MAX;
    for (index, row) in task.proposal_rows.iter().enumerate() {
        if row.proposal == 0
            || row.revision == 0
            || row.revision > task.revision
            || (index != 0 && row.revision >= previous)
        {
            return false;
        }
        for old in task.proposal_rows.iter().take(index) {
            if old.proposal == row.proposal {
                return false;
            }
        }
        previous = row.revision;
    }
    true
}

pub(crate) fn history(
    domain: &mut Domain,
    env: &Env<Limits>,
    proposal: Proposal,
    by: Party,
    change: Change,
    out: &mut Queue<Request>,
) {
    let task = crate::domain::task_mut(domain, proposal.proposer).expect("proposal task live");
    task.record.revision = task.record.revision.checked_add(1).expect("proposal revision preflighted");
    let revision = task.record.revision;
    match proposal.state {
        ProposalState::Accepted { .. } | ProposalState::Rejected { .. } | ProposalState::Withdrawn => {
            crate::positions::proposal(&mut task.record, proposal.number, env.limits.inbox_messages);
        }
        ProposalState::Pending { .. } => {}
    }
    crate::activity::append(&mut task.record, env.wall, crate::Line::Decided { revision }, out);
    out.push(Request::Save {
        record: Stored::History(History {
            task: proposal.proposer,
            revision: task.record.revision,
            by,
            change,
            reason: proposal.reason.clone(),
            proposal: Some(Box::new(proposal)),
            at: env.wall,
            amendment: None,
        }),
    });
}

fn answer(task: &mut TaskRecord, question: u64, heard: AnswerLine) -> bool {
    for line in &mut task.question_lines {
        if line.message == question && line.answer.is_none() {
            line.answer = Some(heard);
            return true;
        }
    }
    false
}

pub(crate) fn sent_answer(
    domain: &mut Domain,
    env: &Env<Limits>,
    sender: Party,
    question: u64,
    heard: AnswerLine,
    out: &mut Queue<Request>,
) {
    let number = match sender {
        Party::Task(number) => number,
        Party::Person(_) | Party::Deployment { .. } => return,
    };
    let task = crate::domain::task_mut(domain, number).expect("answering task is live");
    if answer(&mut task.record, question, heard) {
        crate::domain::publish(domain, env, number, out);
    }
}
