//! Single scopes use exact keys and retained positions (store.md, 7.3).
use super::{Failure, Spent, empty, joins};
use crate::{Get, Row, Store, gets};
use alloc::boxed::Box;
use jig_domain_core::{self as core, reads};
use jig_domain_people::Proposer;
use jig_domain_tasks as tasks;
use skein_lib::Token;

pub(super) fn record(store: &mut Store, key: &core::Key) -> Result<Option<core::Record>, Failure> {
    let row = gets::row(store, Token::new(0), Get::Jig(key.clone())).or(Err(Failure::Corrupt))?;
    let Some(row) = row else { return Ok(None) };
    match row {
        Row::Jig(record) => Ok(Some(record)),
        Row::Application { .. } => Err(Failure::Corrupt),
    }
}

pub(super) fn task(
    store: &mut Store,
    range: &reads::Range,
    task: u64,
    spent: &mut Spent,
) -> Result<reads::Loaded, Failure> {
    if spent.rows == 0 {
        return Ok(empty());
    }
    for key in [jig_domain_tasks::Key::Live(task), jig_domain_tasks::Key::Ended(task)] {
        let key = core::Key::Tasks(key);
        let Some(value) = record(store, &key)? else { continue };
        let position = reads::Position::Owner(key);
        if let Some(after) = &range.after
            && position <= *after
        {
            return Ok(empty());
        }
        let Some(owner) = joins::as_task(&value) else { return Err(Failure::Corrupt) };
        let owner = owner.clone();
        let related = core::Key::Core(core::CoreKey::Terminal { task, attempt: owner.attempt });
        let row = joins::row(store, value, Some(&owner), Some(related), spent)?;
        return Ok(reads::Loaded::Page { rows: Box::new([row]), next: None });
    }
    Ok(empty())
}

fn page(
    store: &mut Store,
    range: &reads::Range,
    value: core::Record,
    owner: Option<&tasks::TaskRecord>,
    related: Option<core::Key>,
    spent: &mut Spent,
) -> Result<reads::Loaded, Failure> {
    if spent.rows == 0 {
        return Ok(empty());
    }
    if let Some(after) = &range.after {
        let position = reads::Position::Owner(value.key());
        if range.backwards {
            if position >= *after {
                return Ok(empty());
            }
        } else if position <= *after {
            return Ok(empty());
        }
    }
    let row = joins::row(store, value, owner, related, spent)?;
    Ok(reads::Loaded::Page { rows: Box::new([row]), next: None })
}

pub(super) fn proposal(
    store: &mut Store,
    range: &reads::Range,
    proposer: Proposer,
    proposal: u64,
    spent: &mut Spent,
) -> Result<reads::Loaded, Failure> {
    match proposer {
        Proposer::Party(party) => {
            if party != range.party {
                return Ok(empty());
            }
            for key in [tasks::Key::PersonProposal(proposal), tasks::Key::DecidedProposal(proposal)] {
                let Some(value) = record(store, &core::Key::Tasks(key))? else { continue };
                let related = core::Key::Core(core::CoreKey::ProposalDecision { proposer, proposal });
                return page(store, range, value, None, Some(related), spent);
            }
            Ok(empty())
        }
        Proposer::Task(task) => {
            let Some(owner) = joins::owner(store, task, spent)? else { return Ok(empty()) };
            for kept in &owner.proposal_rows {
                if kept.proposal == proposal {
                    let key = core::Key::Tasks(tasks::Key::History { task, revision: kept.revision });
                    let Some(value) = record(store, &key)? else { return Ok(empty()) };
                    return page(store, range, value, Some(&owner), None, spent);
                }
            }
            Ok(reads::Loaded::Gone)
        }
    }
}

pub(super) fn escalation(
    store: &mut Store,
    range: &reads::Range,
    task: u64,
    spent: &mut Spent,
) -> Result<reads::Loaded, Failure> {
    let Some(owner) = joins::owner(store, task, spent)? else { return Ok(empty()) };
    let key = core::Key::Core(core::CoreKey::EscalationDecision { task, revision: owner.escalation.revision() });
    let Some(value) = record(store, &key)? else { return Ok(empty()) };
    let related = core::Key::Tasks(tasks::Key::Milestone { task, position: owner.hold });
    page(store, range, value, Some(&owner), Some(related), spent)
}

pub(super) fn question(
    store: &mut Store,
    range: &reads::Range,
    task: u64,
    message: u64,
    spent: &mut Spent,
) -> Result<reads::Loaded, Failure> {
    let Some(owner) = joins::owner(store, task, spent)? else { return Ok(empty()) };
    for kept in &owner.question_lines {
        if kept.message == message {
            let key = core::Key::Tasks(tasks::Key::Activity { task, position: kept.line });
            let Some(value) = record(store, &key)? else { return Ok(empty()) };
            let mut related = None;
            if let Some(answer) = kept.answer {
                related = Some(core::Key::Tasks(tasks::Key::Activity { task: answer.task, position: answer.position }));
            }
            return page(store, range, value, Some(&owner), related, spent);
        }
    }
    Ok(reads::Loaded::Gone)
}

pub(super) fn reply(
    store: &mut Store,
    range: &reads::Range,
    task: u64,
    activity: u64,
    spent: &mut Spent,
) -> Result<reads::Loaded, Failure> {
    let key = core::Key::Tasks(tasks::Key::Activity { task, position: activity });
    let Some(value) = record(store, &key)? else { return Ok(empty()) };
    let Some(owner) = joins::owner(store, task, spent)? else { return Ok(empty()) };
    // A turn reply's transcript evidence joins here in jig 17.
    page(store, range, value, Some(&owner), None, spent)
}

pub(super) fn turn() -> reads::Loaded {
    // The transcript handler replaces this empty answer in jig 17.
    empty()
}
