//! Small requester trees for party reads (protocol/store.md, 7.3).
use super::{core_rows, ended_task};
use alloc::boxed::Box;
use jig_domain_core::{self as core, Record};
use jig_domain_tasks as tasks;

pub const GOAL: u64 = 31;
pub const CHILD: u64 = 32;
pub const GRANDCHILD: u64 = 33;

pub fn read(party: u64, scope: core::reads::Scope) -> core::reads::Range {
    core::reads::Range { party, scope, after: None, backwards: false, rows: 8, bytes: 65_536 }
}

pub fn ended_of(task: u64, requester: tasks::Party) -> Record {
    let Record::Tasks(tasks::Stored::Ended(mut row)) = ended_task(task) else { panic!("ended fixture") };
    row.requester = requester;
    row.attempt = 1;
    row.dependencies = Box::new([]);
    row.spec.inputs = Box::new([]);
    row.references = Box::new([]);
    row.delegates = Box::new([]);
    row.result_followups = Box::new([]);
    Record::Tasks(tasks::Stored::Ended(row))
}

pub fn terminal(task: u64, attempt: u64) -> Record {
    let mut row = core_rows::terminal();
    row.task = task;
    row.attempt = attempt;
    Record::Core(core::CoreRecord::Terminal(row))
}

pub fn with_links(record: Record, dependencies: &[u64]) -> Record {
    let Record::Tasks(tasks::Stored::Ended(mut row)) = record else { panic!("ended fixture") };
    row.dependencies = Box::from(dependencies);
    row.spec.inputs = Box::from(dependencies);
    row.references = Box::from(dependencies);
    row.delegates = Box::from(dependencies);
    row.result_followups = Box::from(dependencies);
    Record::Tasks(tasks::Stored::Ended(row))
}

pub fn person_proposal(number: u64, decided: bool) -> Record {
    let mut row = super::task_person_proposal();
    row.number = number;
    row.proposer = super::ALICE;
    if decided {
        row.state = tasks::PersonProposalState::Accepted { by: tasks::Party::Person(super::ALICE) };
        Record::Tasks(tasks::Stored::DecidedProposal(Box::new(row)))
    } else {
        Record::Tasks(tasks::Stored::PersonProposal(Box::new(row)))
    }
}

pub fn proposal_decision(party: u64, number: u64) -> Record {
    let mut row = core_rows::proposal();
    row.proposer = jig_domain_people::Proposer::Party(party);
    row.proposal = number;
    row.project = 3;
    Record::Core(core::CoreRecord::ProposalDecision(row))
}

pub fn history_with_proposal(task: u64, revision: u64, proposal: u64) -> Record {
    let mut row = super::task_history();
    row.task = task;
    row.revision = revision;
    let mut kept = super::task_proposal();
    kept.proposer = task;
    kept.number = proposal;
    row.proposal = Some(Box::new(kept));
    Record::Tasks(tasks::Stored::History(row))
}

pub fn escalation_decision(task: u64, revision: u64) -> Record {
    let mut row = core_rows::escalation();
    row.task = task;
    row.revision = revision;
    Record::Core(core::CoreRecord::EscalationDecision(row))
}

pub fn hold_milestone(task: u64, position: u64) -> Record {
    Record::Tasks(tasks::Stored::Milestone(tasks::Milestone {
        task,
        position,
        phase: super::task_phase(),
        at: skein_lib::Wall::from_nanos(19),
    }))
}

fn words_line(task: u64, position: u64, message: u64, kind: tasks::MessageKind) -> Record {
    Record::Tasks(tasks::Stored::Activity(tasks::ActivityLine {
        task,
        position,
        at: skein_lib::Wall::from_nanos(19),
        line: tasks::Line::Words {
            message,
            by: tasks::Party::Person(super::ALICE),
            kind,
            words: Box::from(b"words".as_slice()),
        },
    }))
}

pub fn question_line(task: u64, position: u64, message: u64) -> Record {
    words_line(task, position, message, tasks::MessageKind::Question)
}

pub fn answer_line(task: u64, position: u64, question: u64) -> Record {
    words_line(task, position, 7, tasks::MessageKind::Answer { question })
}

pub fn notified_line(task: u64, position: u64, person: u64) -> Record {
    Record::Tasks(tasks::Stored::Activity(tasks::ActivityLine {
        task,
        position,
        at: skein_lib::Wall::from_nanos(19),
        line: tasks::Line::Notified { recipient: tasks::Party::Person(person), word: super::task_word(7) },
    }))
}

fn retained_task(record: Record) -> Box<tasks::TaskRecord> {
    let Record::Tasks(tasks::Stored::Ended(row)) = record else { panic!("ended fixture") };
    row
}

pub fn with_question_lines(record: Record, lines: &[tasks::QuestionLine]) -> Record {
    let mut row = retained_task(record);
    row.question_lines = Box::from(lines);
    Record::Tasks(tasks::Stored::Ended(row))
}

pub fn with_hold(record: Record, position: u64, revision: u64) -> Record {
    let mut row = retained_task(record);
    row.hold = position;
    row.escalation = tasks::Escalation::Unheld { revision };
    Record::Tasks(tasks::Stored::Ended(row))
}

pub fn with_proposal_rows(record: Record, rows: &[tasks::ProposalRow]) -> Record {
    let mut row = retained_task(record);
    row.proposal_rows = Box::from(rows);
    Record::Tasks(tasks::Stored::Ended(row))
}

pub fn activity_line(task: u64, position: u64) -> Record {
    words_line(task, position, position, tasks::MessageKind::Words)
}

pub fn ledger_period(project: u32, period: u64, closed: bool) -> Record {
    let row = tasks::FundingRecord {
        period: None,
        funder: tasks::Funder::Period { project, period },
        parent: None,
        numbers: tasks::Numbers { budget: 10, spent: 0, spent_below: 2, reserved: 0 },
        made: 1,
        closed,
    };
    if closed { Record::Tasks(tasks::Stored::ClosedLedger(row)) } else { Record::Tasks(tasks::Stored::Ledger(row)) }
}

pub fn notes_entry_in(scope: &jig_domain_notes::Scope, name: u64, description: &[u8]) -> (Record, Record) {
    let mut entry = super::note_entry();
    entry.scope = scope.clone();
    entry.name = name;
    entry.description = Box::from(description);
    let mut line = super::note_line();
    line.scope = scope.clone();
    line.name = name;
    line.description = Box::from(description);
    (Record::Notes(jig_domain_notes::Record::Entry(entry)), Record::Notes(jig_domain_notes::Record::Line(line)))
}
