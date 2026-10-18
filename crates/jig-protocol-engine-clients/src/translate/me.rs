//! Served template and creating facts (protocol/configuration.md, 3.1; contract.md, 5.1).
use super::{objects, pieces, problem, task, w};
use alloc::boxed::Box;
use jig_domain_core::acts;
use jig_domain_people as p;
use skein_lib::{List, bytes};
fn executor(value: p::TemplateExecutor, limits: &w::Limits) -> Result<w::TemplateExecutor, w::Problem> {
    Ok(match value {
        p::TemplateExecutor::Charter(value) => {
            w::TemplateExecutor::Charter(w::TemplateCharter::new(limits, w::TemplateCharterParts { value })?)
        }
        p::TemplateExecutor::Chosen => w::TemplateExecutor::Chosen,
    })
}
fn budget(value: p::TemplateBudget, limits: &w::Limits) -> Result<w::TemplateBudget, w::Problem> {
    Ok(match value {
        p::TemplateBudget::Amount(value) => {
            w::TemplateBudget::Amount(w::TemplateAmount::new(limits, w::TemplateAmountParts { value })?)
        }
        p::TemplateBudget::AsAllowed { cap } => {
            w::TemplateBudget::AsAllowed(w::TemplateAsAllowed::new(limits, w::TemplateAsAllowedParts { cap })?)
        }
        p::TemplateBudget::Given => w::TemplateBudget::Given,
    })
}
fn deadline(value: p::TemplateDeadline, limits: &w::Limits) -> Result<w::TemplateDeadline, w::Problem> {
    Ok(match value {
        p::TemplateDeadline::None => w::TemplateDeadline::None,
        p::TemplateDeadline::Span(value) => {
            w::TemplateDeadline::Span(w::TemplateSpan::new(limits, w::TemplateSpanParts { value })?)
        }
        p::TemplateDeadline::Given => w::TemplateDeadline::Given,
    })
}

pub fn templates(values: &List<jig_domain_core::Template>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(w::Template::new(
            limits,
            w::TemplateParts {
                number: value.number,
                name: w::DocumentText::new(
                    limits,
                    w::DocumentTextParts { bytes: bytes::copy_of(&value.name), cut: false },
                )?,
                executor: executor(value.executor, limits)?,
                contract: objects::goal::contract(&value.contract, limits)?,
                shallower: value.shallower,
                budget: budget(value.budget, limits)?,
                deadline: deadline(value.deadline, limits)?,
                tracked: value.tracked,
                flags: flags(value.flags, limits)?,
                label: value.label,
                grace: value.grace,
            },
        )?)
        .expect("one row per template");
    }
    let mut encoded = List::with_capacity(rows.len());
    for row in &rows {
        let mut out = skein_lib::Writer::new(usize::try_from(row.measure()).expect("record measure fits"));
        if row.encode(&mut out).is_err() {
            return Err(problem(skein_codec::Reason::Bound));
        }
        encoded.push(out.finish()).expect("same row count");
    }
    pieces(&encoded, limits.me_project_templates, w::Path::MeProjectTemplates)
}

pub(super) fn flags(value: jig_domain_tasks::Flags, limits: &w::Limits) -> Result<w::TaskFlags, w::Problem> {
    w::TaskFlags::new(
        limits,
        w::TaskFlagsParts {
            conversation: value.conversation,
            listed: value.listed,
            retention: match value.retention {
                jig_domain_tasks::Retention::Kept => w::Retention::Kept,
                jig_domain_tasks::Retention::ForgottenOnceRead => w::Retention::ForgottenOnceRead,
            },
        },
    )
}
pub fn creating(value: &acts::Creating, limits: &w::Limits) -> Result<w::CreatingBounds, w::Problem> {
    let within = match task::bounds(&value.within, limits)? {
        w::Bounds::Create(value) => value,
        w::Bounds::Amend(_) => unreachable!("creating has creating bounds"),
    };
    let beyond = match &value.beyond {
        acts::Beyond::Proposed { to } => w::Beyond::Proposed(w::BeyondProposed::new(
            limits,
            w::BeyondProposedParts { to: task::recipient(to, limits)? },
        )?),
        acts::Beyond::Refused(why) => w::Beyond::Refused(w::BeyondRefused::new(
            limits,
            w::BeyondRefusedParts { value: super::refusals::refusal(*why, limits)? },
        )?),
    };
    w::CreatingBounds::new(limits, w::CreatingBoundsParts { within, beyond })
}
pub fn overrides(
    values: &List<p::TemplateOverride>,
    limits: &w::Limits,
) -> Result<List<w::TemplateOverride>, w::Problem> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(w::TemplateOverride::new(
            limits,
            w::TemplateOverrideParts {
                template: value.template,
                executor: match value.executor {
                    Some(v) => Some(executor(v, limits)?),
                    None => None,
                },
                budget: match value.budget {
                    Some(v) => Some(budget(v, limits)?),
                    None => None,
                },
                deadline: match value.deadline {
                    Some(v) => Some(deadline(v, limits)?),
                    None => None,
                },
                priority: value.priority,
            },
        )?)
        .expect("same capacity");
    }
    Ok(rows)
}
