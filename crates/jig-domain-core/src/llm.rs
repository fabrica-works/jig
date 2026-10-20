//! A task's LLM choice at the root: the default it records at Create (domain/tasks.md, section 3)
//! and the models a run of the task uses (domain/engine.md, 7.1).

use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::List;

use crate::{Core, Model, RunPolicy, Settings};

impl Core {
    /// Give each agent task of `batch` that names no LLM choice its charter's default, recorded as
    /// given (domain/tasks.md, section 3; owner's decision D5), and the members of a recurring
    /// task's template too, which tasks makes each occurrence from. A task whose charter is
    /// configured but whose authority allows none of its models is refused, naming the charter's
    /// first model; a task of an unknown charter is left to tasks, which refuses its executor.
    #[expect(clippy::result_large_err, reason = "the refusal keeps tasks' exact problem for the creator")]
    pub(crate) fn resolve_llm(&self, batch: &mut [tasks::New]) -> Result<(), tasks::Problem> {
        for new in batch {
            let number = new.number;
            self.default_llm(new, number)?;
            if let Some(template) = &mut new.recurring {
                for member in &mut template.batch {
                    self.default_llm(member, number)?;
                }
            }
        }
        Ok(())
    }

    #[expect(clippy::result_large_err, reason = "the refusal keeps tasks' exact problem for the creator")]
    fn default_llm(&self, new: &mut tasks::New, number: u64) -> Result<(), tasks::Problem> {
        let charter = match new.executor {
            tasks::Executor::Agent { charter } => charter,
            tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => return Ok(()),
        };
        if new.llm.is_some() {
            return Ok(());
        }
        // T10 narrows the default to the charter's models whose LLM provider has an account
        // usable now and serving the task's payer (domain/accounts.md, section 4.3).
        new.llm = self.tasks.default_llm(charter, new.authority.models);
        if new.llm.is_some() {
            return Ok(());
        }
        for configured in self.tasks.charters() {
            if configured.number == charter
                && let Some(first) = configured.models.first()
            {
                let why = tasks::Refusal::Llm { model: *first, why: tasks::LlmFault::Authority };
                return Err(tasks::Problem::new(Some(number), why));
            }
        }
        Ok(())
    }

    /// The policy a run of `task` uses: its charter's, running its LLM choice's models, each the
    /// charter's model of that configured name with its chosen effort's name (domain/engine.md,
    /// 7.1). A task with no choice, or one configuration no longer holds since it was checked,
    /// its models reordered or taken from the charter, runs its charter's own.
    pub(crate) fn chosen_policy(&self, task: u64) -> RunPolicy {
        let policy = &self.charter_of(task).policy;
        let chosen = match &self.tasks.task(task).expect("live agent task").llm {
            Some(choice) => chosen(&self.settings, policy, choice),
            None => None,
        };
        match chosen {
            Some(chosen) => chosen,
            None => policy.clone(),
        }
    }
}

/// A party's LLM choice as tasks keeps it, models and efforts by index in configuration's order
/// both: tasks checks it when it makes or amends the task (domain/tasks.md, section 3).
pub(crate) fn choice(value: Option<&people::LlmChoice>) -> Option<tasks::LlmChoice> {
    let value = value?;
    let mut subagents = List::with_capacity(u32::try_from(value.subagents.len()).expect("people bounds sub-agents"));
    for subagent in &value.subagents {
        subagents.push(model_effort(*subagent)).expect("same number of sub-agents");
    }
    Some(tasks::LlmChoice { account: None, main: model_effort(value.main), subagents: subagents.into_boxed() })
}

fn model_effort(value: people::ModelEffort) -> tasks::ModelEffort {
    tasks::ModelEffort { model: value.model, effort: value.effort }
}

fn chosen(settings: &Settings, policy: &RunPolicy, choice: &tasks::LlmChoice) -> Option<RunPolicy> {
    let main = model(settings, policy, choice.main)?;
    let mut alternatives = List::with_capacity(u32::try_from(choice.subagents.len()).ok()?);
    for subagent in &choice.subagents {
        alternatives.push(model(settings, policy, *subagent)?).ok()?;
    }
    let mut chosen = policy.clone();
    chosen.model = main;
    chosen.alternatives = alternatives.into_boxed();
    Some(chosen)
}

fn model(settings: &Settings, policy: &RunPolicy, chosen: tasks::ModelEffort) -> Option<Model> {
    let name = &settings.models.get(usize::from(chosen.model))?.name;
    let mut model = charter_model(policy, name)?.clone();
    model.effort = match chosen.effort {
        Some(effort) => Some(model.efforts.get(usize::from(effort))?.clone()),
        None if model.efforts.is_empty() => None,
        None => return None,
    };
    Some(model)
}

#[expect(clippy::manual_find, reason = "the strict subset iterates without a closure")]
fn charter_model<'policy>(policy: &'policy RunPolicy, name: &[u8]) -> Option<&'policy Model> {
    if policy.model.name.as_ref() == name {
        return Some(&policy.model);
    }
    for model in &policy.alternatives {
        if model.name.as_ref() == name {
            return Some(model);
        }
    }
    None
}
