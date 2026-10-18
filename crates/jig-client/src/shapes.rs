//! Template recipes over the public contract (client/domain/client.md, 19).
use crate::contract::{
    Ask, Beyond, CreateBounds, Creating, Me, NewBudget, NewContract, NewTask, Operation, OwnerExecutor, ProposeAction,
    Recipient, Refusal, Specification, TaskContractDocument, Template, TemplateBudget, TemplateDeadline,
    TemplateExecutor, Text,
};
use skein_lib::{List, Wall};
pub const CHAT: u32 = 1;
pub const GOAL: u32 = 2;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotOffered {
    Project,
    Template,
    Executor,
    Ceiling,
    Creating,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    Cancel,
    Move,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Delegate {
    pub task: u64,
    pub fate: Fate,
}
#[derive(Debug)]
pub struct GoalDraft {
    pub words: Text,
    pub executor: OwnerExecutor,
    pub budget: u64,
    pub deadline: Option<Wall>,
    pub priority: Option<u32>,
}
#[derive(Debug)]
pub enum Judged {
    Create(Ask),
    Propose { ask: Ask, to: Recipient },
    Refused(Refusal),
    NotOffered(NotOffered),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fall {
    Within,
    Proposable(Recipient),
    Refused(Refusal),
    NotOffered(NotOffered),
}
#[must_use]
pub fn template(me: &Me, project: u32, number: u32) -> Option<&Template> {
    for membership in &me.projects {
        if membership.project == project {
            for template in &membership.templates {
                if template.number == number {
                    return Some(template);
                }
            }
        }
    }
    None
}
fn creating(me: &Me, project: u32) -> Result<&Creating, NotOffered> {
    for membership in &me.projects {
        if membership.project == project {
            return membership.creating.as_ref().ok_or(NotOffered::Creating);
        }
    }
    Err(NotOffered::Project)
}
pub fn chat(me: &Me, project: u32, words: Text, now: Wall) -> Result<Ask, NotOffered> {
    let creating = creating(me, project)?;
    let template = template(me, project, CHAT).ok_or(NotOffered::Template)?;
    let task = fill(template, creating, words, None, now)?;
    Ok(Ask::Create { project, tasks: single(task) })
}
pub fn close_chat(me: &Me, project: u32, chat: u64, delegates: &[Delegate], reason: Text) -> Result<Ask, NotOffered> {
    let mut present = false;
    for membership in &me.projects {
        if membership.project == project {
            present = true;
        }
    }
    if !present {
        return Err(NotOffered::Project);
    }
    let template = template(me, project, CHAT).ok_or(NotOffered::Template)?;
    let count = u32::try_from(delegates.len()).expect("validated delegates fit u32");
    let mut operations = List::with_capacity(count.saturating_add(1));
    for delegate in delegates {
        let operation = match delegate.fate {
            Fate::Cancel => Operation::Cancel { project, task: delegate.task, reason: reason.clone(), grace: None },
            Fate::Move => Operation::Move { project, task: delegate.task, reason: reason.clone() },
        };
        operations.push(operation).expect("one operation per delegate");
    }
    operations.push(Operation::Cancel { project, task: chat, reason, grace: template.grace }).expect("chat last");
    Ok(Ask::Compound { project, operations })
}
#[must_use]
pub fn set_goal(me: &Me, project: u32, draft: GoalDraft, reason: Text, now: Wall) -> Judged {
    let creating = match creating(me, project) {
        Ok(value) => value,
        Err(why) => return Judged::NotOffered(why),
    };
    let Some(template) = template(me, project, GOAL) else {
        return Judged::NotOffered(NotOffered::Template);
    };
    let task = match fill(template, creating, draft.words.clone(), Some(&draft), now) {
        Ok(value) => value,
        Err(why) => return Judged::NotOffered(why),
    };
    let filled = GoalDraft {
        words: draft.words,
        executor: task.executor,
        budget: task.authority.spend,
        deadline: task.authority.deadline,
        priority: task.tracked,
    };
    match fall(creating, &filled) {
        Fall::Within => Judged::Create(Ask::Create { project, tasks: single(task) }),
        Fall::Proposable(to) => Judged::Propose {
            ask: Ask::Propose { project, action: ProposeAction::Batch { tasks: single(task) }, reason },
            to,
        },
        Fall::Refused(why) => Judged::Refused(why),
        Fall::NotOffered(why) => Judged::NotOffered(why),
    }
}
fn single(task: NewTask) -> List<NewTask> {
    let mut tasks = List::with_capacity(1);
    tasks.push(task).expect("one template task");
    tasks
}
fn fill(
    template: &Template,
    creating: &Creating,
    words: Text,
    draft: Option<&GoalDraft>,
    now: Wall,
) -> Result<NewTask, NotOffered> {
    let executor = match template.executor {
        TemplateExecutor::Charter(charter) => {
            let executor = OwnerExecutor::Agent { charter };
            if let Some(draft) = draft
                && draft.executor != executor
            {
                return Err(NotOffered::Executor);
            }
            executor
        }
        TemplateExecutor::Chosen => draft.ok_or(NotOffered::Template)?.executor,
    };
    let contract = match &template.contract {
        TaskContractDocument::Report { words } => NewContract::Report { words: *words },
        TaskContractDocument::Change { connector, kind, words } => {
            NewContract::Change { connector: *connector, kind: *kind, words: *words }
        }
        TaskContractDocument::Verdict { choices, cut } => {
            if *cut {
                return Err(NotOffered::Template);
            }
            NewContract::Verdict { choices: choices.clone() }
        }
        TaskContractDocument::Approval { .. } => return Err(NotOffered::Template),
    };
    if template.shallower == 0 {
        return Err(NotOffered::Template);
    }
    let mut authority = creating.within.authority.whole().ok_or(NotOffered::Creating)?;
    authority.delegation.depth = authority.delegation.depth.saturating_sub(template.shallower.saturating_sub(1));
    let budget = match template.budget {
        TemplateBudget::Amount(amount) => {
            authority.spend = amount;
            NewBudget::Named
        }
        TemplateBudget::AsAllowed { cap } => {
            authority.spend = 0;
            NewBudget::AsAllowed { cap }
        }
        TemplateBudget::Given => {
            authority.spend = draft.ok_or(NotOffered::Template)?.budget;
            NewBudget::Named
        }
    };
    authority.deadline = match template.deadline {
        TemplateDeadline::None => None,
        TemplateDeadline::Span(span) => Some(Wall::from_nanos(now.as_nanos().saturating_add(span.as_nanos()))),
        TemplateDeadline::Given => draft.ok_or(NotOffered::Template)?.deadline,
    };
    Ok(NewTask {
        executor,
        spec: Specification { words, parameters: List::with_capacity(0), inputs: List::with_capacity(0) },
        contract,
        authority,
        budget,
        wake: None,
        dependencies: List::with_capacity(0),
        named: List::with_capacity(0),
        tracked: priority(template.tracked, draft),
        label: template.label,
        flags: template.flags,
    })
}
#[must_use]
pub fn fall(creating: &Creating, draft: &GoalDraft) -> Fall {
    let within = &creating.within;
    if !listed(within, &draft.executor) {
        return Fall::NotOffered(NotOffered::Executor);
    }
    if draft.budget > within.maximum_budget || !no_later(draft.deadline, within.maximum_deadline) {
        return Fall::NotOffered(NotOffered::Ceiling);
    }
    if draft.budget <= within.budget && no_later(draft.deadline, within.latest) {
        return Fall::Within;
    }
    match &creating.beyond {
        Beyond::Proposed { to } => Fall::Proposable(to.clone()),
        Beyond::Refused(why) => Fall::Refused(why.clone()),
    }
}
fn listed(within: &CreateBounds, executor: &OwnerExecutor) -> bool {
    match executor {
        OwnerExecutor::Agent { charter } => {
            for offered in &within.charters {
                if offered == charter {
                    return true;
                }
            }
        }
        OwnerExecutor::Procedure { connector, code } => {
            for offered in &within.procedures {
                if offered.connector == *connector && offered.code == *code {
                    return true;
                }
            }
        }
        OwnerExecutor::Person(_) => {}
    }
    false
}
const fn no_later(deadline: Option<Wall>, bound: Option<Wall>) -> bool {
    match (deadline, bound) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(at), Some(latest)) => at.as_nanos() <= latest.as_nanos(),
    }
}

fn priority(default: Option<u32>, draft: Option<&GoalDraft>) -> Option<u32> {
    let default = default?;
    Some(match draft {
        Some(draft) => draft.priority.unwrap_or(default),
        None => default,
    })
}
