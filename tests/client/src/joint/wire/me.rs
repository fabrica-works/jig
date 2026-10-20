//! The served template and creating records in the joint world's map.
use super::{historical_goal, offers, owner, refusals};
use jig_client::DomainLimits as Limits;
use jig_client_domain::contract as c;
use jig_domain_core::{Template, acts};
use jig_domain_people as p;
use jig_domain_tasks as t;
use skein_lib::List;
pub(super) fn templates(values: &List<Template>, limits: &Limits) -> Result<List<c::Template>, owner::Invalid> {
    if values.len() > limits.rows {
        return Err(owner::Invalid::TooLarge);
    }
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(template(value, limits)?).expect("same length");
    }
    Ok(rows)
}
fn template(value: &Template, limits: &Limits) -> Result<c::Template, owner::Invalid> {
    Ok(c::Template {
        llm: None,
        number: value.number,
        name: owner::text(&jig_domain_core::documents::Text { bytes: value.name.clone(), cut: false }, limits)?,
        executor: match value.executor {
            p::TemplateExecutor::Charter(v) => c::TemplateExecutor::Charter(v),
            p::TemplateExecutor::Chosen => c::TemplateExecutor::Chosen,
        },
        contract: historical_goal::contract(&value.contract, limits)?,
        shallower: value.shallower,
        budget: match value.budget {
            p::TemplateBudget::Amount(v) => c::TemplateBudget::Amount(v),
            p::TemplateBudget::AsAllowed { cap } => c::TemplateBudget::AsAllowed { cap },
            p::TemplateBudget::Given => c::TemplateBudget::Given,
        },
        deadline: match value.deadline {
            p::TemplateDeadline::None => c::TemplateDeadline::None,
            p::TemplateDeadline::Span(v) => c::TemplateDeadline::Span(v),
            p::TemplateDeadline::Given => c::TemplateDeadline::Given,
        },
        tracked: value.tracked,
        flags: c::TaskFlags {
            conversation: value.flags.conversation,
            listed: value.flags.listed,
            retention: match value.flags.retention {
                t::Retention::Kept => c::Retention::Kept,
                t::Retention::ForgottenOnceRead => c::Retention::ForgottenOnceRead,
            },
        },
        label: value.label,
        grace: value.grace,
    })
}
pub(super) fn creating(value: &acts::Creating, limits: &Limits) -> Result<c::Creating, owner::Invalid> {
    let mut value = value.clone();
    match &mut value.within {
        acts::Bounds::Create { charters, procedures, .. } => {
            *charters = entries(charters);
            *procedures = entries(procedures);
        }
        acts::Bounds::Amend { .. } => panic!("creating bound"),
    }
    match &mut value.beyond {
        acts::Beyond::Proposed { to: acts::Recipient::Roles { roles, .. } } => *roles = entries(roles),
        acts::Beyond::Proposed { .. } | acts::Beyond::Refused(_) => (),
    }
    let c::Bounds::Create(within) = offers::bounds(&value.within, limits)? else { panic!("creating bound") };
    let beyond = match &value.beyond {
        acts::Beyond::Proposed { to } => c::Beyond::Proposed { to: offers::recipient(to, limits)? },
        acts::Beyond::Refused(why) => c::Beyond::Refused(refusals::refusal(*why)),
    };
    Ok(c::Creating { within, beyond })
}

pub(super) fn overrides(
    values: &List<p::TemplateOverride>,
    limits: &Limits,
) -> Result<List<c::TemplateOverride>, owner::Invalid> {
    owner::list(&entries(values), limits, |value| {
        Ok(c::TemplateOverride {
            template: value.template,
            executor: value.executor.map(|value| match value {
                p::TemplateExecutor::Charter(v) => c::TemplateExecutor::Charter(v),
                p::TemplateExecutor::Chosen => c::TemplateExecutor::Chosen,
            }),
            budget: value.budget.map(|value| match value {
                p::TemplateBudget::Amount(v) => c::TemplateBudget::Amount(v),
                p::TemplateBudget::AsAllowed { cap } => c::TemplateBudget::AsAllowed { cap },
                p::TemplateBudget::Given => c::TemplateBudget::Given,
            }),
            deadline: value.deadline.map(|value| match value {
                p::TemplateDeadline::None => c::TemplateDeadline::None,
                p::TemplateDeadline::Span(v) => c::TemplateDeadline::Span(v),
                p::TemplateDeadline::Given => c::TemplateDeadline::Given,
            }),
            priority: value.priority,
        })
    })
}

fn entries<T: Clone>(values: &List<T>) -> List<T> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(value.clone()).ok().expect("same entries");
    }
    rows
}

#[cfg(test)]
pub(super) fn empty_authority() -> jig_domain_core::documents::AuthorityLine {
    jig_domain_core::documents::AuthorityLine {
        tools: t::Tools(0),
        grants: List::with_capacity(0),
        executors: List::with_capacity(0),
        tasks: 0,
        depth: 0,
        ceiling: t::Budget { spend: 0, deadline: None },
        notes: t::Scopes(0),
        note_resources: List::with_capacity(0),
    }
}
