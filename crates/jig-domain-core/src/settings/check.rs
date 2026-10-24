//! One whole-set check, also used after restoring stored changes (engine.md, 17.1).
use super::{Current, Failure, Fault, Setting, Value};
use crate::{Limits, Settings};
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;

/// Check the maps in place: no third materialised set of values is constructed.
pub fn check(current: &Current, settings: &Settings, connectors: &[u16], limits: &Limits) -> Result<(), Failure> {
    check_set(current, true, settings, connectors, limits)?;
    check_set(current, false, settings, connectors, limits)
}

// Both passes borrow the same two maps. The flag selects one view; no values
// or borrowed generic wrapper are materialised.
fn value(current: &Current, defaults: bool, setting: Setting) -> Option<&Value> {
    if defaults {
        match current.defaults.get(&setting) {
            Some(default) => Some(&default.value),
            None => None,
        }
    } else {
        current.value(setting)
    }
}
fn charter(current: &Current, defaults: bool, number: u32) -> Option<&crate::Charter> {
    match value(current, defaults, Setting::Charter(number)) {
        Some(Value::Charter(value)) => Some(value),
        Some(Value::Template(_) | Value::Policy(_) | Value::Rules(_)) | None => None,
    }
}
fn template(current: &Current, defaults: bool, number: u32) -> Option<&crate::Template> {
    match value(current, defaults, Setting::Template(number)) {
        Some(Value::Template(value)) => Some(value),
        Some(Value::Charter(_) | Value::Policy(_) | Value::Rules(_)) | None => None,
    }
}
fn rules(current: &Current, defaults: bool) -> &authority::Rules {
    match value(current, defaults, Setting::Rules) {
        Some(Value::Rules(value)) => value,
        Some(Value::Charter(_) | Value::Template(_) | Value::Policy(_)) | None => unreachable!("rules checked first"),
    }
}
fn offered(current: &Current, defaults: bool) -> skein_lib::List<Setting> {
    if !defaults {
        return current.offered();
    }
    let mut names = skein_lib::List::with_capacity(current.defaults.len());
    for (&setting, _) in &current.defaults {
        names.push(setting).expect("default names");
    }
    names
}
fn check_set(
    current: &Current,
    defaults: bool,
    settings: &Settings,
    connectors: &[u16],
    limits: &Limits,
) -> Result<(), Failure> {
    let mut counts = [0_u32; 3];
    for (&setting, _) in &current.defaults {
        count(setting, &mut counts, limits)?;
    }
    counts = [0; 3];
    for (&setting, _) in &current.changes {
        count(setting, &mut counts, limits)?;
    }
    counts = [0; 3];
    let names = offered(current, defaults);
    for &setting in &names {
        count(setting, &mut counts, limits)?;
    }
    let rules = match value(current, defaults, Setting::Rules) {
        Some(Value::Rules(rules)) => rules,
        Some(Value::Charter(_) | Value::Template(_) | Value::Policy(_)) | None => {
            return Err(Failure { setting: Setting::Rules, fault: Fault::Rules });
        }
    };
    if !authority::valid_rules(rules, &limits.authority) {
        return Err(Failure { setting: Setting::Rules, fault: Fault::Rules });
    }
    if !authority_refs(current, defaults, &rules.ceiling, None, connectors, settings)
        || !requirement_refs(&rules.requirements, connectors)
    {
        return Err(Failure { setting: Setting::Rules, fault: Fault::Reference });
    }
    for pair in rules.implies.entries() {
        if !connectors.contains(&pair.connector) {
            return Err(Failure { setting: Setting::Rules, fault: Fault::Reference });
        }
    }
    // Hidden defaults still obey their own byte bounds; removals do not remove that cost.
    for (&setting, default) in &current.defaults {
        shape(setting, &default.value, limits)?;
    }
    for (&setting, change) in &current.changes {
        if let Some(value) = &change.value {
            shape(setting, value, limits)?;
        }
    }
    for &setting in &names {
        let fault = match value(current, defaults, setting).expect("offered setting") {
            Value::Charter(charter) => charter_fault(charter, settings, limits),
            Value::Template(template) => {
                template_fault(current, defaults, template, template.executor, settings, connectors, limits)
            }
            Value::Policy(policy) => policy_fault(current, defaults, policy, settings, connectors, limits),
            Value::Rules(_) => None,
        };
        if let Some(fault) = fault {
            return Err(Failure { setting, fault });
        }
    }
    Ok(())
}
pub(super) fn shape(setting: Setting, value: &Value, limits: &Limits) -> Result<(), Failure> {
    let valid = match value {
        Value::Charter(value) => {
            setting == Setting::Charter(value.number)
                && value.number != 0
                && match crate::memory::policy_owned_bytes(&value.policy) {
                    Some(bytes) => bytes <= u64::from(limits.run_bytes),
                    None => false,
                }
        }
        Value::Template(value) => {
            setting == Setting::Template(value.number)
                && value.number != 0
                && value.name.len() <= usize::try_from(limits.documents.text_bytes).expect("text bound")
                && tasks::valid_contract(&limits.tasks, &value.contract)
        }
        Value::Policy(value) => {
            let numbered = match setting {
                Setting::Policy(number) => number != 0,
                Setting::Charter(_) | Setting::Template(_) | Setting::Rules => false,
            };
            numbered
                && value.templates.len() <= usize::try_from(limits.templates).expect("template override bound")
                && match crate::policy::snapshot(&value.policy, &value.permissions, &value.templates) {
                    Some(value) => match people::policy_bytes(&value) {
                        Some(bytes) => bytes <= limits.policy_bytes,
                        None => false,
                    },
                    None => false,
                }
        }
        Value::Rules(value) => setting == Setting::Rules && authority::valid_rules(value, &limits.authority),
    };
    if valid { Ok(()) } else { Err(Failure { setting, fault: Fault::Shape }) }
}
fn charter_fault(charter: &crate::Charter, settings: &Settings, limits: &Limits) -> Option<Fault> {
    let policy = &charter.policy;
    if !model_valid(&policy.model, settings) {
        return Some(Fault::Reference);
    }
    for (at, model) in policy.alternatives.iter().enumerate() {
        if !model_valid(model, settings) {
            return Some(Fault::Reference);
        }
        for earlier in policy.alternatives.iter().take(at) {
            if earlier.name == model.name {
                return Some(Fault::Shape);
            }
        }
    }
    if policy.alternatives.len() > usize::try_from(tasks::MODELS).expect("models bound") {
        return Some(Fault::Count);
    }
    if policy.turns <= policy.reserve_turns || policy.time <= policy.reserve_time {
        return Some(Fault::Budget);
    }
    if match crate::run::reserve_floor(policy) {
        Some(floor) => charter.spend < floor,
        None => true,
    } {
        return Some(Fault::Budget);
    }
    if crate::memory::policy_owned_bytes(policy)? > u64::from(limits.run_bytes) {
        return Some(Fault::Shape);
    }
    None
}
fn template_fault(
    current: &Current,
    defaults: bool,
    template: &crate::Template,
    executor: people::TemplateExecutor,
    settings: &Settings,
    connectors: &[u16],
    limits: &Limits,
) -> Option<Fault> {
    match &template.contract {
        tasks::Contract::Change { connector, .. } => {
            if !connectors.contains(connector) {
                return Some(Fault::Reference);
            }
        }
        tasks::Contract::Approval { part, .. } => {
            if !connectors.contains(&part.connector) {
                return Some(Fault::Reference);
            }
        }
        tasks::Contract::Report { .. } | tasks::Contract::Verdict { .. } => {}
    }
    if let Some(grace) = template.grace
        && grace > limits.tasks.grace
    {
        return Some(Fault::Grace);
    }
    if let Some(choice) = &template.llm
        && !choice_valid(&settings.models, choice)
    {
        return Some(Fault::Reference);
    }
    if template.flags.ended_by == tasks::EndedBy::Requester
        && (!template.flags.conversation || executor == people::TemplateExecutor::Chosen)
    {
        return Some(Fault::Shape);
    }
    match executor {
        people::TemplateExecutor::Charter(number) => {
            let Some(charter) = charter(current, defaults, number) else { return Some(Fault::Reference) };
            if let Some(choice) = &template.llm
                && !crate::llm::takes(&settings.models, &charter.policy, choice)
            {
                return Some(Fault::Reference);
            }
        }
        people::TemplateExecutor::Chosen => {
            if let Some(choice) = &template.llm
                && !choice_valid(&settings.models, choice)
            {
                return Some(Fault::Reference);
            }
        }
    }
    None
}
fn choice_valid(models: &[crate::Model], choice: &tasks::LlmChoice) -> bool {
    if choice.subagents.len() > usize::try_from(tasks::MODELS).expect("model bound") {
        return false;
    }
    if !effort_valid(models, choice.main) {
        return false;
    }
    for (at, chosen) in choice.subagents.iter().enumerate() {
        for earlier in choice.subagents.iter().take(at) {
            if earlier.model == chosen.model {
                return false;
            }
        }
        if !effort_valid(models, *chosen) {
            return false;
        }
    }
    true
}
fn policy_fault(
    current: &Current,
    defaults: bool,
    configured: &crate::ConfiguredPolicy,
    settings: &Settings,
    connectors: &[u16],
    limits: &Limits,
) -> Option<Fault> {
    let policy = &configured.policy;
    if let Some(reason) = authority::policy_refusal(rules(current, defaults), &limits.authority, policy) {
        return Some(match reason {
            authority::PolicyRefusal::AboveRules => Fault::Rules,
            authority::PolicyRefusal::Oversized | authority::PolicyRefusal::InvalidRole => Fault::Shape,
            authority::PolicyRefusal::Full => Fault::Count,
        });
    }
    let mut owner = false;
    for role in &policy.roles {
        if role.number == people::Role::Owner.number() {
            owner = role.requests.allows(authority::RequestKind::Create);
        }
        if !authority_refs(current, defaults, &role.authority, Some(policy), connectors, settings) {
            return Some(Fault::Reference);
        }
    }
    if !owner {
        return Some(Fault::Owner);
    }
    if !authority_refs(current, defaults, &policy.ceiling, Some(policy), connectors, settings)
        || !requirement_refs(&policy.requirements, connectors)
    {
        return Some(Fault::Reference);
    }
    for grant in &policy.projections {
        if !connectors.contains(&grant.connector) {
            return Some(Fault::Reference);
        }
    }
    for (at, permission) in configured.permissions.iter().enumerate() {
        if !connectors.contains(&permission.connector) {
            return Some(Fault::Reference);
        }
        let mut found = false;
        for role in &policy.roles {
            found |= role.number == permission.role;
        }
        if !found {
            return Some(Fault::Reference);
        }
        for earlier in configured.permissions.iter().take(at) {
            if earlier.connector == permission.connector && earlier.permission == permission.permission {
                return Some(Fault::Shape);
            }
        }
    }
    for (at, entry) in configured.templates.iter().enumerate() {
        for earlier in configured.templates.iter().take(at) {
            if earlier.template == entry.template {
                return Some(Fault::Shape);
            }
        }
        let Some(template) = template(current, defaults, entry.template) else { return Some(Fault::Reference) };
        let executor = entry.executor.unwrap_or(template.executor);
        if let Some(fault) = template_fault(current, defaults, template, executor, settings, connectors, limits) {
            return Some(fault);
        }
    }
    None
}
fn authority_refs(
    current: &Current,
    defaults: bool,
    value: &authority::Authority,
    policy: Option<&authority::Policy>,
    connectors: &[u16],
    settings: &Settings,
) -> bool {
    let count = u32::try_from(settings.models.len()).expect("configured model count");
    if !value.models.is_all() && value.models.bits().checked_shr(count).unwrap_or(0) != 0 {
        return false;
    }
    for grant in &value.grants {
        if !connectors.contains(&grant.connector) {
            return false;
        }
    }
    for scope in &value.note_resources {
        if !connectors.contains(&scope.connector) {
            return false;
        }
    }
    for executor in &value.delegation.kinds {
        match executor {
            authority::Executor::Charter(number) => {
                if charter(current, defaults, *number).is_none() {
                    return false;
                }
            }
            authority::Executor::Role(number) => {
                if let Some(policy) = policy {
                    let mut found = false;
                    for role in &policy.roles {
                        found |= role.number == *number;
                    }
                    if !found {
                        return false;
                    }
                }
            }
            authority::Executor::Procedure(_) => {}
        }
    }
    true
}
fn requirement_refs(requirements: &[authority::Requirement], connectors: &[u16]) -> bool {
    for requirement in requirements {
        if !connectors.contains(&requirement.connector) || !connectors.contains(&requirement.judge.connector) {
            return false;
        }
    }
    true
}

fn count(setting: Setting, counts: &mut [u32; 3], limits: &Limits) -> Result<(), Failure> {
    let (slot, bound) = match setting {
        Setting::Charter(_) => (0, limits.tasks.charters),
        Setting::Template(_) => (1, limits.templates),
        Setting::Policy(_) => (2, limits.authority.projects),
        Setting::Rules => return Ok(()),
    };
    let count = counts.get_mut(slot).expect("three numbered kinds");
    *count = count.saturating_add(1);
    if *count > bound { Err(Failure { setting, fault: Fault::Count }) } else { Ok(()) }
}

fn model_valid(model: &crate::Model, settings: &Settings) -> bool {
    match crate::charters::configured(settings, model) {
        Some(index) => settings.models.get(usize::from(index)) == Some(model),
        None => false,
    }
}
fn effort_valid(models: &[crate::Model], chosen: tasks::ModelEffort) -> bool {
    let Some(model) = models.get(usize::from(chosen.model)) else { return false };
    match chosen.effort {
        Some(effort) => usize::from(effort) < model.efforts.len(),
        None => model.efforts.is_empty(),
    }
}

/// Check a streamed change before retaining its value under a per-kind heap bound.
pub(super) fn admits_change(current: &Current, setting: Setting, limits: &Limits) -> Result<(), Failure> {
    let mut counts = [0_u32; 3];
    for (&held, _) in &current.changes {
        count(held, &mut counts, limits)?;
    }
    count(setting, &mut counts, limits)
}
