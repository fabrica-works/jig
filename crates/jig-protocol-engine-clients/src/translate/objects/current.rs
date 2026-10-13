#![expect(clippy::manual_map, reason = "step code matches optional owner facts without closures")]
//! Whole single-read owner documents (protocol/client.md, 3 and 6).
use super::super::{chips, owner, task, w};
use alloc::boxed::Box;
use jig_domain_core::documents as d;
use skein_lib::{List, Writer, bytes};
pub fn result(value: &d::ResultDoc, limits: &w::Limits) -> Result<w::ResultDocument, w::Problem> {
    let mut facts = List::with_capacity(4);
    facts
        .push(w::Fact::Ending(w::EndingRow::new(
            limits,
            w::EndingRowParts {
                parent: None,
                value: w::EndingFields::Task(w::EndingTaskFields::new(limits, w::EndingTaskFieldsParts { value: 1 })?),
            },
        )?))
        .expect("one ending root");
    chips::append_ending(&value.ending, Some(0), &mut facts, limits)?;
    let ending = w::Ending::new(
        limits,
        w::EndingParts { rows: chips::fact_pieces(&facts, limits.ending_rows, w::Path::EndingRows)? },
    )?;
    let result = match &value.ending {
        d::Ending::Done(item) => Some(result_value(item, limits)?),
        d::Ending::Failed(_) => None,
        d::Ending::Cancelled { result, .. } => match result {
            Some(item) => Some(result_value(item, limits)?),
            None => None,
        },
    };
    w::ResultDocument::new(
        limits,
        w::ResultDocumentParts {
            task: chips::chip(&value.task, limits)?,
            ending,
            value: result,
            at: match value.at {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            details: Some(details(value, limits)?),
            acts: task::offered(&value.acts, limits, limits.result_document_acts, w::Path::ResultDocumentActs)?,
        },
    )
}
pub fn details(value: &d::ResultDoc, limits: &w::Limits) -> Result<w::ResultDetails, w::Problem> {
    let mut delegates = List::with_capacity(value.delegates.len());
    for row in &value.delegates {
        delegates.push(task::link(row, limits)?).expect("supplied rows fit");
    }
    let delegates = task::links_pieces(&delegates, limits.result_details_delegates, w::Path::ResultDetailsDelegates)?;
    let followups = match &value.followups {
        Some(rows) => {
            let mut links = List::with_capacity(rows.len());
            for row in rows {
                links.push(task::link(row, limits)?).expect("supplied rows fit");
            }
            let links =
                task::links_pieces(&links, limits.result_details_followups_some, w::Path::ResultDetailsFollowups)?;
            Some(links)
        }
        None => None,
    };
    w::ResultDetails::new(
        limits,
        w::ResultDetailsParts {
            contract: owner::contract(&value.contract, limits)?,
            requester: owner::requester(value.requester, limits)?,
            executor: owner::executor(value.executor, limits)?,
            chosen_by: value.chosen_by,
            delegates,
            followups,
            proposal: match &value.proposal {
                Some(item) => Some(task::proposal(item, limits)?),
                None => None,
            },
            cut: value.cut,
        },
    )
}
pub fn result_value(value: &d::ResultValue, limits: &w::Limits) -> Result<w::ResultValue, w::Problem> {
    let mut facts = List::with_capacity(1);
    facts
        .push(w::Fact::ResultValue(w::ResultValueRow::new(
            limits,
            w::ResultValueRowParts { parent: None, value: chips::result_fields(value, limits)? },
        )?))
        .expect("one result fact");
    w::ResultValue::new(
        limits,
        w::ResultValueParts { rows: chips::fact_pieces(&facts, limits.result_value_rows, w::Path::ResultValueRows)? },
    )
}
pub fn person_task(value: &d::PersonTask, limits: &w::Limits) -> Result<w::OwnerPersonTask, w::Problem> {
    let mut results = List::with_capacity(value.results.len());
    for row in &value.results {
        results
            .push(w::OwnerShownResult::new(
                limits,
                w::OwnerShownResultParts {
                    task: task::link(&row.task, limits)?,
                    result: match &row.result {
                        Some(item) => Some(result(item, limits)?),
                        None => None,
                    },
                },
            )?)
            .expect("supplied rows fit");
    }
    let results =
        owner_shown_result_pieces(&results, limits.owner_person_task_results, w::Path::OwnerPersonTaskResults)?;

    w::OwnerPersonTask::new(
        limits,
        w::OwnerPersonTaskParts {
            task: chips::chip(&value.task, limits)?,
            revision: w::Revision::new(limits, w::RevisionParts { value: value.revision })?,
            requester: owner::requester(value.requester, limits)?,
            spec: owner::specification(&value.spec, limits)?,
            address: chips::address(value.address, limits)?,
            taken_by: value.taken_by,
            since: match value.since {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            stall: value.stall,
            deadline: match value.deadline {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            contract: owner::contract(&value.contract, limits)?,
            results,
            cut: value.cut,
            acts: task::offered(&value.acts, limits, limits.owner_person_task_acts, w::Path::OwnerPersonTaskActs)?,
        },
    )
}
pub fn question(value: &d::Question, limits: &w::Limits) -> Result<w::Question, w::Problem> {
    w::Question::new(
        limits,
        w::QuestionParts {
            task: chips::chip(&value.task, limits)?,
            from: Some(owner::requester(value.from, limits)?),
            message: value.message,
            words: bytes::copy_of(&value.words),
            since: value.at.as_nanos(),
            acts: task::offered(&value.acts, limits, limits.question_acts, w::Path::QuestionActs)?,
            ending: None,
        },
    )
}
pub fn escalation(value: &d::Escalation, limits: &w::Limits) -> Result<w::OwnerEscalation, w::Problem> {
    w::OwnerEscalation::new(
        limits,
        w::OwnerEscalationParts {
            task: task::document(&value.task, limits)?,
            acts: task::offered(&value.acts, limits, limits.owner_escalation_acts, w::Path::OwnerEscalationActs)?,
        },
    )
}
pub(super) fn owner_shown_result_pieces(
    rows: &List<w::OwnerShownResult>,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
