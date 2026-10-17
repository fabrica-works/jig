//! General operation targets and template overrides, translated into people records.
use super::{Untranslatable, asks};
use alloc::boxed::Box;
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

/// Decode the pieces before anything reaches the core.
pub(super) fn compound(value: w::Compound, limits: &w::Limits) -> Result<p::Ask, Untranslatable> {
    let path = w::Path::CompoundOperations;
    let mut reader = skein_lib::Reader::new(value.operations());
    let mut decoded = List::with_capacity(asks::count(value.operations(), path)?);
    while reader.remaining() != 0 {
        let Ok(Some(bytes)) = skein_codec::piece::split(&mut reader) else {
            return Err(Untranslatable { path });
        };
        let Ok(operation) = w::Operation::decode(limits, &mut skein_lib::Reader::new(&bytes)) else {
            return Err(Untranslatable { path });
        };
        decoded.push(operation).expect("pieces bounded by bytes");
    }
    let mut operations = List::with_capacity(decoded.len());
    for operation in decoded.into_boxed() {
        operations.push(operation_to_people(operation, value.project(), limits)?).expect("same capacity");
    }
    Ok(p::Ask::Compound { project: value.project(), operations })
}
fn operation_to_people(value: w::Operation, project: u32, limits: &w::Limits) -> Result<p::Operation, Untranslatable> {
    if operation_project(&value) != project {
        return Err(Untranslatable { path: w::Path::CompoundOperations });
    }
    Ok(match value {
        w::Operation::Message(v) => {
            let v = v.into_parts();
            p::Operation::Message { task: v.task, words: v.words, release: false }
        }
        w::Operation::Amend(v) => {
            let v = v.into_parts();
            p::Operation::Amend {
                task: v.task,
                revision: match v.revision {
                    Some(v) => v.value(),
                    None => 0,
                },
                amendment: Box::new(asks::amendment(v.change, v.reason, limits)?),
                release: None,
            }
        }
        w::Operation::Cancel(v) => {
            let v = v.into_parts();
            p::Operation::Cancel { task: v.task, reason: v.reason, grace: v.grace }
        }
        w::Operation::Release(v) => {
            let v = v.into_parts();
            p::Operation::Release { task: v.task, revision: v.revision.value(), again: v.again }
        }
        w::Operation::Stop(v) => p::Operation::Stop { task: v.task() },
        w::Operation::MoveTask(v) => {
            let v = v.into_parts();
            p::Operation::Move { task: v.task, reason: v.reason }
        }
    })
}
fn operation_project(value: &w::Operation) -> u32 {
    match value {
        w::Operation::Message(v) => v.project(),
        w::Operation::Amend(v) => v.project(),
        w::Operation::Cancel(v) => v.project(),
        w::Operation::Release(v) => v.project(),
        w::Operation::Stop(v) => v.project(),
        w::Operation::MoveTask(v) => v.project(),
    }
}

/// Project mismatch is a typed refusal before the core sees any operation.
pub(crate) fn refused(record: &w::Ask, limits: &w::Limits) -> Result<Option<w::KeyedAnswer>, w::Problem> {
    let w::Ask::Compound(value) = record else {
        return Ok(None);
    };
    let mut reader = skein_lib::Reader::new(value.operations());
    let mut at = 0_u32;
    while let Some(bytes) = wire_piece(&mut reader)? {
        let operation = w::Operation::decode(limits, &mut skein_lib::Reader::new(&bytes))?;
        let project = operation_project(&operation);
        if project != value.project() {
            let refusal = w::Refusal::Malformed(w::RefusalMalformed::new(
                limits,
                w::RefusalMalformedParts { at, why: w::Malformed::ProjectMismatch },
            )?);
            let answer = w::Final::Refused(w::Refused::new(limits, w::RefusedParts { refusal, at: Some(at) })?);
            let mut out = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("wire size fits"));
            answer.encode(&mut out).expect("measured answer");
            return match super::answers::owned(w::Owner::Jig, jig_client_protocol_wire::VERSION, out.finish(), limits) {
                Ok(value) => Ok(Some(value)),
                Err(problem) => Err(problem),
            };
        }
        at = at.checked_add(1).ok_or(super::problem(skein_codec::Reason::Bound))?;
    }
    Ok(None)
}

fn wire_piece(reader: &mut skein_lib::Reader<'_>) -> Result<Option<Box<[u8]>>, w::Problem> {
    match skein_codec::piece::split(reader) {
        Ok(value) => Ok(value),
        Err(_) => Err(super::problem(skein_codec::Reason::Malformed)),
    }
}
