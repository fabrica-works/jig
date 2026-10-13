//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Box, Limits, Wall, c, object_key_fields, owner_activity_line_fields, owner_escalation_decision_fields,
    owner_milestone_line_fields, owner_person_proposal_fields, owner_person_task_fields,
    owner_proposal_decision_fields, owner_proposal_history_fields, question_fields, result_fields, task_fields, w,
};
pub(crate) fn owner_escalation_decision_record_fields(
    value: &w::OwnerEscalationDecisionRecord,
    limits: &Limits,
) -> Result<c::OwnerEscalationDecisionRecord, w::Problem> {
    Ok(c::OwnerEscalationDecisionRecord {
        project: value.project(),
        requester: value.requester(),
        task: value.task(),
        revision: value.revision(),
        by: value.by(),
        at: Wall::from_nanos(value.at()),
        decision: owner_escalation_decision_fields(value.decision(), limits)?,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_historical_value_fields(
    value: &w::OwnerHistoricalValue,
    limits: &Limits,
) -> Result<c::OwnerHistoricalValue, w::Problem> {
    Ok(match value {
        w::OwnerHistoricalValue::Task(row) => c::OwnerHistoricalValue::Task {
            document: Box::new(task_fields(row.document(), limits)?),
            result: Box::new(result_fields(row.result(), limits)?),
        },
        w::OwnerHistoricalValue::Proposal(row) => {
            c::OwnerHistoricalValue::Proposal(owner_proposal_history_fields(row.value(), limits)?)
        }
        w::OwnerHistoricalValue::PersonProposal(row) => {
            let opt0 = match row.decision() {
                Some(item) => Some(owner_proposal_decision_fields(item, limits)),
                None => None,
            };
            c::OwnerHistoricalValue::PersonProposal {
                proposal: Box::new(owner_person_proposal_fields(row.proposal(), limits)?),
                decision: opt0,
            }
        }
        w::OwnerHistoricalValue::Escalation(row) => {
            let opt0 = match row.hold() {
                Some(item) => Some(owner_milestone_line_fields(item, limits)?),
                None => None,
            };
            c::OwnerHistoricalValue::Escalation {
                task: Box::new(task_fields(row.task(), limits)?),
                decision: owner_escalation_decision_record_fields(row.decision(), limits)?,
                hold: opt0,
            }
        }
        w::OwnerHistoricalValue::Question(row) => {
            let opt0 = match row.answer() {
                Some(item) => Some(owner_activity_line_fields(item, limits)?),
                None => None,
            };
            let opt1 = match row.ending() {
                Some(item) => Some(Box::new(result_fields(item, limits)?)),
                None => None,
            };
            c::OwnerHistoricalValue::Question {
                question: question_fields(row.question(), limits)?,
                answer: opt0,
                ending: opt1,
            }
        }
        w::OwnerHistoricalValue::PersonTask(row) => c::OwnerHistoricalValue::PersonTask {
            document: Box::new(owner_person_task_fields(row.document(), limits)?),
            result: Box::new(result_fields(row.result(), limits)?),
        },
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_historical_fields(
    value: &w::OwnerHistorical,
    limits: &Limits,
) -> Result<c::OwnerHistorical, w::Problem> {
    let opt0 = match value.at() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt1 = match value.by() {
        Some(item) => Some(crate::translate::facts::owner_requester(item, limits)),
        None => None,
    };
    Ok(c::OwnerHistorical {
        key: object_key_fields(value.key(), limits),
        value: owner_historical_value_fields(value.value(), limits)?,
        at: opt0,
        by: opt1,
    })
}
