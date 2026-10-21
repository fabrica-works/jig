//! The served template and creating records in the joint world's map.
use super::{historical_goal, offers, owner, refusals};
use jig_client::DomainLimits as Limits;
use jig_client_domain::contract as c;
use jig_domain_core::{Template, acts};
use jig_domain_people as p;
use jig_domain_tasks as t;
use skein_lib::List;
pub(super) fn templates(
    values: &List<Template>,
    limits: &Limits,
    models: &[jig_domain_core::Model],
) -> Result<List<c::Template>, owner::Invalid> {
    if values.len() > limits.rows {
        return Err(owner::Invalid::TooLarge);
    }
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(template(value, limits, models)?).expect("same length");
    }
    Ok(rows)
}
fn template(
    value: &Template,
    limits: &Limits,
    models: &[jig_domain_core::Model],
) -> Result<c::Template, owner::Invalid> {
    Ok(c::Template {
        llm: super::llm::choice(value.llm.as_ref(), models, limits)?,
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
            ended_by: match value.flags.ended_by {
                t::EndedBy::Result => c::EndedBy::Result,
                t::EndedBy::Requester => c::EndedBy::Requester,
            },
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
pub(super) fn creating(
    value: &acts::Creating,
    limits: &Limits,
    models: &[jig_domain_core::Model],
) -> Result<c::Creating, owner::Invalid> {
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
    let c::Bounds::Create(within) = offers::bounds(&value.within, limits, models)? else { panic!("creating bound") };
    let beyond = match &value.beyond {
        acts::Beyond::Proposed { to } => c::Beyond::Proposed { to: offers::recipient(to, limits)? },
        acts::Beyond::Refused(why) => c::Beyond::Refused(refusals::refusal(why.clone())?),
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
        models: t::Models::ALL,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_carries_its_named_choice() {
        let mut source = jig_core_world::world::config(8455).core.settings.templates[0].clone();
        source.llm = Some(t::LlmChoice {
            account: Some(17),
            main: t::ModelEffort { model: 0, effort: Some(1) },
            subagents: Box::new([t::ModelEffort { model: 0, effort: None }]),
        });
        let limits = super::super::deployment::tests::limits();
        let actual = template(&source, &limits, &[jig_core_world::world::scripted_model()]).expect("whole template");
        let text = |value: &[u8]| c::Text::new(value.into(), limits.text).expect("fixture name");
        assert_eq!(
            actual.llm,
            Some(c::LlmChoice {
                account: Some(17),
                main: c::ModelEffort { model: text(b"scripted"), effort: Some(text(b"high")) },
                subagents: crate::test::list([c::ModelEffort { model: text(b"scripted"), effort: None }]),
            })
        );
    }
}
