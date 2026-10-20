//! Configured defaults for client shapes of work (configuration.md, 3.1).
use crate::{Limits, Path, Problems, What, check, read};
use alloc::boxed::Box;
use jig_domain_core::{Charter, Model, Template};
use jig_domain_people::{TemplateBudget, TemplateDeadline, TemplateExecutor};
use jig_domain_tasks as tasks;
use skein_json::{Document, document::Kind};
use skein_lib::{Duration, List};

pub(crate) fn templates(
    document: &Document,
    at: u32,
    path: &Path,
    charters: &[Charter],
    models: &[Model],
    limits: &Limits,
    problems: &mut Problems,
) -> Option<Box<[Template]>> {
    let items = report_list(read::list(document, at, limits.core.templates), path, problems)?;
    let mut templates = List::<Template>::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        if let Some(template) = template(document, *at, &path, models, limits, problems) {
            for previous in &templates {
                if previous.number == template.number {
                    problems.push(path.member(b"number"), What::Unique);
                    break;
                }
            }
            match template.executor {
                TemplateExecutor::Charter(number) => {
                    let mut configured = None;
                    for charter in charters {
                        if charter.number == number {
                            configured = Some(charter);
                        }
                    }
                    match (configured, &template.llm) {
                        (None, _) => problems.push(path.member(b"executor"), What::Refers),
                        (Some(charter), Some(choice)) => {
                            charters_models(charter, choice, models, &path.member(b"llm"), problems);
                        }
                        (Some(_), None) => {}
                    }
                }
                TemplateExecutor::Chosen => {}
            }
            templates.push(template).expect("bounded templates");
        }
    }
    let templates = templates.into_boxed();
    Some(templates)
}

fn report(value: Result<u64, What>, path: &Path, problems: &mut Problems) -> Option<u64> {
    match value {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.clone(), what);
            None
        }
    }
}
fn report_list(value: Result<List<u32>, What>, path: &Path, problems: &mut Problems) -> Option<List<u32>> {
    match value {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.clone(), what);
            None
        }
    }
}
fn template(
    document: &Document,
    at: u32,
    path: &Path,
    models: &[Model],
    limits: &Limits,
    problems: &mut Problems,
) -> Option<Template> {
    let mut object = check::object(document, at, path, problems)?;
    let number = check::count(&mut object, document, b"number", path, problems);
    let name = match object.required(document, b"name", path, problems) {
        Some(at) => match read::text(document, at, limits.core.documents.text_bytes) {
            Ok(name) => Some(name),
            Err(what) => {
                problems.push(path.member(b"name"), what);
                None
            }
        },
        None => None,
    };
    let executor = match object.required(document, b"executor", path, problems) {
        Some(at) => executor(document, at, &path.member(b"executor"), problems),
        None => None,
    };
    let contract = match object.required(document, b"contract", path, problems) {
        Some(at) => contract(document, at, &path.member(b"contract"), &limits.core.tasks, problems),
        None => None,
    };
    let shallower = check::count(&mut object, document, b"shallower", path, problems);
    let budget = match object.required(document, b"budget", path, problems) {
        Some(at) => budget(document, at, &path.member(b"budget"), problems),
        None => None,
    };
    let deadline = match object.required(document, b"deadline", path, problems) {
        Some(at) => deadline(document, at, &path.member(b"deadline"), problems),
        None => None,
    };
    let tracked = match object.required(document, b"tracked", path, problems) {
        Some(at) => priority(document, at, &path.member(b"tracked"), problems),
        None => Err(What::Missing),
    };
    let conversation = check::boolean(&mut object, document, b"conversation", path, problems);
    let listed = check::boolean(&mut object, document, b"listed", path, problems);
    let retention = check::text(&mut object, document, b"retention", path, problems);
    let label = check::count(&mut object, document, b"label", path, problems);
    let grace = match object.required(document, b"grace_ms", path, problems) {
        Some(at) => grace(document, at, &path.member(b"grace_ms"), problems),
        None => Err(What::Missing),
    };
    let llm = match object.required(document, b"llm", path, problems) {
        Some(at) => choice(document, at, &path.member(b"llm"), models, problems),
        None => Err(What::Missing),
    };
    object.finish(document, path, problems);
    let retention = match retention?.as_ref() {
        b"kept" => tasks::Retention::Kept,
        b"forgotten_once_read" => tasks::Retention::ForgottenOnceRead,
        _ => {
            problems.push(path.member(b"retention"), What::Form);
            return None;
        }
    };
    let template = Template {
        number: number?,
        name: name?,
        executor: executor?,
        contract: contract?,
        shallower: shallower?,
        budget: budget?,
        deadline: deadline?,
        tracked: tracked.ok()?,
        flags: tasks::Flags { conversation: conversation?, listed: listed?, retention },
        label: label?,
        grace: grace.ok()?,
        llm: llm.ok()?,
    };
    if template.number == 0 {
        problems.push(path.member(b"number"), What::Bound);
    }
    if let Some(grace) = template.grace
        && grace > limits.core.tasks.grace
    {
        problems.push(path.member(b"grace_ms"), What::Grace { template: template.number });
    }
    Some(template)
}
fn null(document: &Document, at: u32) -> bool {
    match document.token(at) {
        Some(token) => token.kind == Kind::Null,
        None => false,
    }
}
fn named(document: &Document, at: u32, name: &[u8]) -> bool {
    match read::text(document, at, 32) {
        Ok(text) => text.as_ref() == name,
        Err(_) => false,
    }
}
fn form_object(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<read::Object> {
    if let Some(token) = document.token(at)
        && token.kind == Kind::String
    {
        problems.push(path.clone(), What::Form);
        return None;
    }
    check::object(document, at, path, problems)
}
pub(crate) fn executor(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<TemplateExecutor> {
    if named(document, at, b"chosen") {
        return Some(TemplateExecutor::Chosen);
    }
    let mut object = form_object(document, at, path, problems)?;
    let number = if object.member(document, b"charter").is_some() {
        check::count(&mut object, document, b"charter", path, problems)
    } else {
        problems.push(path.clone(), What::Form);
        None
    };
    object.finish(document, path, problems);
    Some(TemplateExecutor::Charter(number?))
}
pub(crate) fn budget(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<TemplateBudget> {
    if named(document, at, b"given") {
        return Some(TemplateBudget::Given);
    }
    let mut object = form_object(document, at, path, problems)?;
    let value = if let Some(at) = object.member(document, b"amount") {
        Some(TemplateBudget::Amount(report(read::integer(document, at, u64::MAX), &path.member(b"amount"), problems)?))
    } else if let Some(at) = object.member(document, b"as_allowed") {
        if null(document, at) {
            Some(TemplateBudget::AsAllowed { cap: None })
        } else {
            Some(TemplateBudget::AsAllowed {
                cap: Some(report(read::integer(document, at, u64::MAX), &path.member(b"as_allowed"), problems)?),
            })
        }
    } else {
        problems.push(path.clone(), What::Form);
        None
    };
    object.finish(document, path, problems);
    value
}
pub(crate) fn deadline(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<TemplateDeadline> {
    if named(document, at, b"given") {
        return Some(TemplateDeadline::Given);
    }
    if named(document, at, b"none") {
        return Some(TemplateDeadline::None);
    }
    let mut object = form_object(document, at, path, problems)?;
    let span = if object.member(document, b"span_ms").is_some() {
        check::duration(&mut object, document, b"span_ms", path, problems)
    } else {
        problems.push(path.clone(), What::Form);
        None
    };
    object.finish(document, path, problems);
    Some(TemplateDeadline::Span(span?))
}
fn grace(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Result<Option<Duration>, What> {
    if null(document, at) {
        return Ok(None);
    }
    match read::duration(document, at) {
        Ok(span) => Ok(Some(span)),
        Err(what) => {
            problems.push(path.clone(), what);
            Err(what)
        }
    }
}
fn priority(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Result<Option<u32>, What> {
    if null(document, at) {
        return Ok(None);
    }
    let mut object = check::object(document, at, path, problems).ok_or(What::Kind)?;
    let value = check::count(&mut object, document, b"priority", path, problems);
    object.finish(document, path, problems);
    Ok(Some(value.ok_or(What::Bound)?))
}

/// A template's LLM choice: `null` for its charter's default, or a main model and the models its
/// run's sub-agents may use, each named once (configuration.md, 3.1; domain/tasks.md, 3).
fn choice(
    document: &Document,
    at: u32,
    path: &Path,
    models: &[Model],
    problems: &mut Problems,
) -> Result<Option<tasks::LlmChoice>, What> {
    if null(document, at) {
        return Ok(None);
    }
    let mut object = check::object(document, at, path, problems).ok_or(What::Kind)?;
    let main = match object.required(document, b"main", path, problems) {
        Some(at) => model_effort(document, at, &path.member(b"main"), models, problems),
        None => None,
    };
    let items = check::list(&mut object, document, b"sub_agents", tasks::MODELS, path, problems);
    object.finish(document, path, problems);
    let items = items.ok_or(What::Kind)?;
    let mut subagents = List::<tasks::ModelEffort>::with_capacity(items.len());
    let mut whole = true;
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path.member(b"sub_agents"), index);
        let Some(chosen) = model_effort(document, *at, &path, models, problems) else {
            whole = false;
            continue;
        };
        let mut repeated = false;
        for earlier in &subagents {
            repeated |= earlier.model == chosen.model;
        }
        if repeated {
            problems.push(path.member(b"model"), What::Unique);
            whole = false;
        } else {
            subagents.push(chosen).expect("sub-agents bounded by the list");
        }
    }
    match main {
        Some(main) if whole => Ok(Some(tasks::LlmChoice { main, subagents: subagents.into_boxed() })),
        Some(_) | None => Err(What::Refers),
    }
}
/// One model a template's choice names, with an effort it takes: `null` only for a model that
/// declares none (configuration.md, 3.1 and 8).
fn model_effort(
    document: &Document,
    at: u32,
    path: &Path,
    models: &[Model],
    problems: &mut Problems,
) -> Option<tasks::ModelEffort> {
    let mut object = check::object(document, at, path, problems)?;
    let name = check::text(&mut object, document, b"model", path, problems);
    let effort = match object.required(document, b"effort", path, problems) {
        Some(at) if null(document, at) => Some(None),
        Some(at) => match read::text(document, at, jig_domain_core::EFFORT_BYTES) {
            Ok(effort) => Some(Some(effort)),
            Err(what) => {
                problems.push(path.member(b"effort"), what);
                None
            }
        },
        None => None,
    };
    object.finish(document, path, problems);
    let name = name?;
    let mut configured = None;
    for (index, model) in models.iter().enumerate() {
        if model.name == name {
            configured = Some((index, model));
            break;
        }
    }
    let Some((index, model)) = configured else {
        problems.push(path.member(b"model"), What::Refers);
        return None;
    };
    let effort = match effort? {
        Some(effort) => {
            let mut taken = None;
            for (at, declared) in model.efforts.iter().enumerate() {
                if *declared == effort {
                    taken = Some(at);
                    break;
                }
            }
            match taken {
                Some(at) => Some(u8::try_from(at).expect("efforts bounded by `EFFORTS`")),
                None => {
                    problems.push(path.member(b"effort"), What::Refers);
                    return None;
                }
            }
        }
        None if model.efforts.is_empty() => None,
        None => {
            problems.push(path.member(b"effort"), What::Refers);
            return None;
        }
    };
    Some(tasks::ModelEffort { model: u8::try_from(index).expect("models bounded by `MODELS`"), effort })
}
/// Refuse each model a template's choice names that its charter does not: its main model or one
/// of its alternatives (configuration.md, 3 and 3.1).
fn charters_models(
    charter: &Charter,
    choice: &tasks::LlmChoice,
    models: &[Model],
    path: &Path,
    problems: &mut Problems,
) {
    if !among(charter, models, choice.main) {
        problems.push(path.member(b"main").member(b"model"), What::Refers);
    }
    for (index, chosen) in choice.subagents.iter().enumerate() {
        if !among(charter, models, *chosen) {
            problems.push(check::index(&path.member(b"sub_agents"), index).member(b"model"), What::Refers);
        }
    }
}
fn among(charter: &Charter, models: &[Model], chosen: tasks::ModelEffort) -> bool {
    let Some(model) = models.get(usize::from(chosen.model)) else {
        return false;
    };
    let mut listed = charter.policy.model.name == model.name;
    for alternative in &charter.policy.alternatives {
        listed |= alternative.name == model.name;
    }
    listed
}

fn contract(
    document: &Document,
    at: u32,
    path: &Path,
    limits: &tasks::Limits,
    problems: &mut Problems,
) -> Option<tasks::Contract> {
    let mut object = check::object(document, at, path, problems)?;
    let value = if let Some(at) = object.member(document, b"report") {
        let path = path.member(b"report");
        let mut report = check::object(document, at, &path, problems)?;
        let words = check::count(&mut report, document, b"words", &path, problems);
        report.finish(document, &path, problems);
        Some(tasks::Contract::Report { words: words? })
    } else if let Some(at) = object.member(document, b"change") {
        change(document, at, &path.member(b"change"), problems)
    } else if let Some(at) = object.member(document, b"verdict") {
        verdict(document, at, &path.member(b"verdict"), limits, problems)
    } else {
        problems.push(path.clone(), What::Form);
        return None;
    };
    object.finish(document, path, problems);
    let value = value?;
    if !tasks::valid_contract(limits, &value) {
        problems.push(path.clone(), What::Bound);
    }
    Some(value)
}
fn change(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<tasks::Contract> {
    let mut object = check::object(document, at, path, problems)?;
    let connector = check::short(&mut object, document, b"connector", path, problems);
    let kind = check::short(&mut object, document, b"kind", path, problems);
    let words = check::count(&mut object, document, b"words", path, problems);
    object.finish(document, path, problems);
    Some(tasks::Contract::Change { connector: connector?, kind: kind?, words: words? })
}
fn verdict(
    document: &Document,
    at: u32,
    path: &Path,
    limits: &tasks::Limits,
    problems: &mut Problems,
) -> Option<tasks::Contract> {
    let mut object = check::object(document, at, path, problems)?;
    let items = check::list(&mut object, document, b"choices", limits.contract_choices, path, problems);
    object.finish(document, path, problems);
    let items = items?;
    let mut choices = List::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path.member(b"choices"), index);
        let mut object = check::object(document, *at, &path, problems)?;
        let code = check::count(&mut object, document, b"code", &path, problems);
        let words = check::count(&mut object, document, b"words", &path, problems);
        let followups = check::count(&mut object, document, b"followups", &path, problems);
        let fields = match object.required(document, b"fields", &path, problems) {
            Some(at) => fields(document, at, &path.member(b"fields"), problems),
            None => None,
        };
        object.finish(document, &path, problems);
        choices
            .push(tasks::Verdict { code: code?, words: words?, followups: followups?, fields: fields? })
            .expect("bounded choices");
    }
    Some(tasks::Contract::Verdict { choices: choices.into_boxed() })
}
fn fields(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Box<[tasks::FieldRule]>> {
    let items = report_list(read::list(document, at, document.len()), path, problems)?;
    let mut fields = List::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        let mut object = check::object(document, *at, &path, problems)?;
        let name = check::text(&mut object, document, b"name", &path, problems);
        let max = check::count(&mut object, document, b"max", &path, problems);
        object.finish(document, &path, problems);
        fields.push(tasks::FieldRule { name: name?, max: max? }).expect("bounded fields");
    }
    Some(fields.into_boxed())
}

pub(crate) fn overrides(
    document: &Document,
    at: u32,
    path: &Path,
    limit: u32,
    problems: &mut Problems,
) -> Option<Box<[jig_domain_people::TemplateOverride]>> {
    let items = report_list(read::list(document, at, limit), path, problems)?;
    let mut entries = List::<jig_domain_people::TemplateOverride>::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        let mut object = check::object(document, *at, &path, problems)?;
        let template = check::count(&mut object, document, b"template", &path, problems);
        if let Some(number) = template {
            for previous in &entries {
                if previous.template == number {
                    problems.push(path.member(b"template"), What::Unique);
                    break;
                }
            }
        }
        let at = object.required(document, b"executor", &path, problems)?;
        let executor =
            if null(document, at) { None } else { Some(executor(document, at, &path.member(b"executor"), problems)?) };
        let at = object.required(document, b"budget", &path, problems)?;
        let budget =
            if null(document, at) { None } else { Some(budget(document, at, &path.member(b"budget"), problems)?) };
        let at = object.required(document, b"deadline", &path, problems)?;
        let deadline =
            if null(document, at) { None } else { Some(deadline(document, at, &path.member(b"deadline"), problems)?) };
        let priority = check::optional(&mut object, document, b"priority", &path, problems).ok()?;
        let priority = match priority {
            Some(value) => match u32::try_from(value) {
                Ok(value) => Some(value),
                Err(_) => {
                    problems.push(path.member(b"priority"), What::Bound);
                    return None;
                }
            },
            None => None,
        };
        object.finish(document, &path, problems);
        entries
            .push(jig_domain_people::TemplateOverride { template: template?, executor, budget, deadline, priority })
            .expect("bounded overrides");
    }
    Some(entries.into_boxed())
}
