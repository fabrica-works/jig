//! General operation targets and template overrides, translated into people records.
use super::{Untranslatable, asks};
use jig_client_protocol_wire::v8 as w;
use jig_domain_people as p;
use skein_lib::List;

pub(super) fn answer(project: u32, on: w::AnswerOn) -> p::Ask {
    match on {
        w::AnswerOn::Question(value) => {
            let v = value.into_parts();
            p::Ask::AnswerQuestion { project, task: v.task, question: v.message, words: v.words }
        }
        w::AnswerOn::Person(value) => {
            let v = value.into_parts();
            p::Ask::AnswerPerson {
                project,
                task: v.task,
                revision: v.revision.value(),
                result: asks::person_answer(v.answer),
            }
        }
    }
}
pub(super) fn decide(project: u32, on: w::DecideOn) -> Result<p::Ask, Untranslatable> {
    Ok(match on {
        w::DecideOn::Proposal(value) => {
            let v = value.into_parts();
            p::Ask::DecideProposal {
                project,
                proposer: asks::proposer(v.proposal.proposer()),
                proposal: v.proposal.number(),
                decision: asks::proposal_decision(v.decision)?,
            }
        }
        w::DecideOn::Escalation(value) => {
            let v = value.into_parts();
            p::Ask::DecideEscalation {
                project,
                task: v.task,
                revision: v.revision.value(),
                decision: asks::escalation_decision(v.decision)?,
            }
        }
    })
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(super) fn templates(value: w::PolicyEditTemplates) -> p::PolicyChange {
    let values = value.into_parts().overrides;
    let mut overrides = List::with_capacity(values.capacity());
    for value in values.into_boxed() {
        let v = value.into_parts();
        overrides
            .push(p::TemplateOverride {
                template: v.template,
                executor: match v.executor {
                    Some(value) => Some(executor(value)),
                    None => None,
                },
                budget: match v.budget {
                    Some(value) => Some(budget(value)),
                    None => None,
                },
                deadline: match v.deadline {
                    Some(value) => Some(deadline(value)),
                    None => None,
                },
                priority: v.priority,
            })
            .expect("same capacity");
    }
    p::PolicyChange::Templates { overrides: overrides.into_boxed() }
}
fn executor(value: w::TemplateExecutor) -> p::TemplateExecutor {
    match value {
        w::TemplateExecutor::Charter(v) => p::TemplateExecutor::Charter(v.value()),
        w::TemplateExecutor::Chosen => p::TemplateExecutor::Chosen,
    }
}
fn budget(value: w::TemplateBudget) -> p::TemplateBudget {
    match value {
        w::TemplateBudget::Amount(v) => p::TemplateBudget::Amount(v.value()),
        w::TemplateBudget::AsAllowed(v) => p::TemplateBudget::AsAllowed { cap: *v.cap() },
        w::TemplateBudget::Given => p::TemplateBudget::Given,
    }
}
fn deadline(value: w::TemplateDeadline) -> p::TemplateDeadline {
    match value {
        w::TemplateDeadline::None => p::TemplateDeadline::None,
        w::TemplateDeadline::Span(v) => p::TemplateDeadline::Span(v.value()),
        w::TemplateDeadline::Given => p::TemplateDeadline::Given,
    }
}

pub(super) fn tracking(value: w::Tracking) -> p::Tracking {
    match value {
        w::Tracking::Tracked(v) => p::Tracking::Tracked { priority: v.priority() },
        w::Tracking::Untracked => p::Tracking::Untracked,
    }
}
