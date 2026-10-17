//! General operation targets and project template overrides on the wire.
use super::asks;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v8 as w;
use skein_lib::{List, bytes};

pub(super) fn answer(value: &c::AnswerOn, limits: &w::Limits) -> Result<w::AnswerOn, w::Problem> {
    Ok(match value {
        c::AnswerOn::Question { task, message, words } => w::AnswerOn::Question(w::AnswerQuestion::new(
            limits,
            w::AnswerQuestionParts { task: *task, message: *message, words: bytes::copy_of(words.bytes()) },
        )?),
        c::AnswerOn::Person { task, revision, answer } => w::AnswerOn::Person(w::AnswerPerson::new(
            limits,
            w::AnswerPersonParts {
                task: *task,
                revision: asks::wire_revision(*revision, limits)?,
                answer: asks::wire_person_answer(answer, limits)?,
            },
        )?),
    })
}
pub(super) fn decide(value: &c::DecideOn, limits: &w::Limits) -> Result<w::DecideOn, w::Problem> {
    Ok(match value {
        c::DecideOn::Proposal { proposal, decision } => w::DecideOn::Proposal(w::DecideProposal::new(
            limits,
            w::DecideProposalParts {
                proposal: asks::wire_proposal_key(proposal, limits)?,
                decision: asks::wire_decision(decision, limits)?,
            },
        )?),
        c::DecideOn::Escalation { task, revision, decision } => w::DecideOn::Escalation(w::DecideEscalation::new(
            limits,
            w::DecideEscalationParts {
                task: *task,
                revision: asks::wire_revision(*revision, limits)?,
                decision: asks::wire_decision(decision, limits)?,
            },
        )?),
    })
}
pub(super) fn templates(values: &List<c::TemplateOverride>, limits: &w::Limits) -> Result<w::PolicyEdit, w::Problem> {
    let mut overrides = List::with_capacity(values.len());
    for v in values {
        overrides
            .push(w::TemplateOverride::new(
                limits,
                w::TemplateOverrideParts {
                    template: v.template,
                    executor: match v.executor {
                        Some(value) => Some(executor(value, limits)?),
                        None => None,
                    },
                    budget: match v.budget {
                        Some(value) => Some(budget(value, limits)?),
                        None => None,
                    },
                    deadline: match v.deadline {
                        Some(value) => Some(deadline(value, limits)?),
                        None => None,
                    },
                    priority: v.priority,
                },
            )?)
            .expect("same capacity");
    }
    Ok(w::PolicyEdit::Templates(w::PolicyEditTemplates::new(limits, w::PolicyEditTemplatesParts { overrides })?))
}
fn executor(value: c::TemplateExecutor, limits: &w::Limits) -> Result<w::TemplateExecutor, w::Problem> {
    Ok(match value {
        c::TemplateExecutor::Charter(value) => {
            w::TemplateExecutor::Charter(w::TemplateCharter::new(limits, w::TemplateCharterParts { value })?)
        }
        c::TemplateExecutor::Chosen => w::TemplateExecutor::Chosen,
    })
}
fn budget(value: c::TemplateBudget, limits: &w::Limits) -> Result<w::TemplateBudget, w::Problem> {
    Ok(match value {
        c::TemplateBudget::Amount(value) => {
            w::TemplateBudget::Amount(w::TemplateAmount::new(limits, w::TemplateAmountParts { value })?)
        }
        c::TemplateBudget::AsAllowed { cap } => {
            w::TemplateBudget::AsAllowed(w::TemplateAsAllowed::new(limits, w::TemplateAsAllowedParts { cap })?)
        }
        c::TemplateBudget::Given => w::TemplateBudget::Given,
    })
}
fn deadline(value: c::TemplateDeadline, limits: &w::Limits) -> Result<w::TemplateDeadline, w::Problem> {
    Ok(match value {
        c::TemplateDeadline::None => w::TemplateDeadline::None,
        c::TemplateDeadline::Span(value) => {
            w::TemplateDeadline::Span(w::TemplateSpan::new(limits, w::TemplateSpanParts { value })?)
        }
        c::TemplateDeadline::Given => w::TemplateDeadline::Given,
    })
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(super) fn restore_templates(values: &List<w::TemplateOverride>) -> c::PolicyEdit {
    let mut overrides = List::with_capacity(values.len());
    for v in values {
        overrides
            .push(c::TemplateOverride {
                template: v.template(),
                executor: match v.executor() {
                    Some(value) => Some(restore_executor(value)),
                    None => None,
                },
                budget: match v.budget() {
                    Some(value) => Some(restore_budget(value)),
                    None => None,
                },
                deadline: match v.deadline() {
                    Some(value) => Some(restore_deadline(value)),
                    None => None,
                },
                priority: *v.priority(),
            })
            .expect("same capacity");
    }
    c::PolicyEdit::Templates { overrides }
}
fn restore_executor(value: &w::TemplateExecutor) -> c::TemplateExecutor {
    match value {
        w::TemplateExecutor::Charter(v) => c::TemplateExecutor::Charter(v.value()),
        w::TemplateExecutor::Chosen => c::TemplateExecutor::Chosen,
    }
}
fn restore_budget(value: &w::TemplateBudget) -> c::TemplateBudget {
    match value {
        w::TemplateBudget::Amount(v) => c::TemplateBudget::Amount(v.value()),
        w::TemplateBudget::AsAllowed(v) => c::TemplateBudget::AsAllowed { cap: *v.cap() },
        w::TemplateBudget::Given => c::TemplateBudget::Given,
    }
}
fn restore_deadline(value: &w::TemplateDeadline) -> c::TemplateDeadline {
    match value {
        w::TemplateDeadline::None => c::TemplateDeadline::None,
        w::TemplateDeadline::Span(v) => c::TemplateDeadline::Span(v.value()),
        w::TemplateDeadline::Given => c::TemplateDeadline::Given,
    }
}

pub(super) fn tracking(value: c::Tracking, limits: &w::Limits) -> Result<w::Tracking, w::Problem> {
    Ok(match value {
        c::Tracking::Tracked { priority } => {
            w::Tracking::Tracked(w::TrackingTracked::new(limits, w::TrackingTrackedParts { priority })?)
        }
        c::Tracking::Untracked => w::Tracking::Untracked,
    })
}
pub(super) fn restore_tracking(value: &w::Tracking) -> c::Tracking {
    match value {
        w::Tracking::Tracked(v) => c::Tracking::Tracked { priority: v.priority() },
        w::Tracking::Untracked => c::Tracking::Untracked,
    }
}
