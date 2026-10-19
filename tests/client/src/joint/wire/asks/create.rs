//! The direct joint world's independent batch translation.
use super::{authority, client as c, people as p, request_arguments, specification};
use skein_lib::List;
pub(super) fn ask(project: u32, values: &List<c::NewTask>) -> p::Ask {
    p::Ask::Create { project, batch: values.iter().map(member).collect() }
}
fn member(v: &c::NewTask) -> p::Member {
    let spec = specification(&v.spec);
    p::Member {
        executor: match v.executor {
            c::OwnerExecutor::Agent { charter } => p::TaskExecutor::Agent { charter },
            c::OwnerExecutor::Procedure { connector, code } => p::TaskExecutor::Procedure { connector, code },
            c::OwnerExecutor::Person(address) => p::TaskExecutor::Person(match address {
                c::PersonAddress::Party(v) => p::PersonAddress::Person(v),
                c::PersonAddress::Role(v) => p::PersonAddress::Role(v),
            }),
        },
        words: spec.words,
        parameters: spec.parameters,
        inputs: spec.inputs,
        authority: authority(&v.authority),
        deadline: match v.deadline {
            c::NewDeadline::Named => p::Deadline::Named,
        },
        budget: match v.budget {
            c::NewBudget::Named => p::Budget::Named,
            c::NewBudget::AsAllowed { cap } => p::Budget::AsAllowed { cap },
        },
        contract: match &v.contract {
            c::NewContract::Report { words } => p::Contract::Report { words: *words },
            c::NewContract::Change { connector, kind, words } => {
                p::Contract::Change { connector: *connector, kind: *kind, words: *words }
            }
            c::NewContract::Verdict { choices } => p::Contract::Verdict {
                choices: choices
                    .iter()
                    .map(|v| p::Verdict {
                        code: v.code,
                        words: v.words,
                        followups: v.followups,
                        fields: v.fields.iter().map(|f| p::FieldRule { name: f.name.clone(), max: f.max }).collect(),
                    })
                    .collect(),
            },
        },
        wake: match v.wake {
            Some(wake) => request_arguments::wake(wake),
            None => p::WakePolicy {
                words: p::WakeRule::Immediate,
                notices: p::WakeRule::Immediate,
                news: p::WakeRule::Immediate,
                results: p::ResultsWake::LastOrFailure,
                questions: true,
                answers: true,
                timers: true,
            },
        },
        dependencies: v
            .dependencies
            .iter()
            .map(|v| match v {
                c::TaskRef::Member(v) => p::Dependency::Batch(*v),
                c::TaskRef::Task(v) => p::Dependency::Existing(*v),
            })
            .collect(),
        named: v.named.iter().map(super::grant).collect(),
        tracked: v.tracked,
        label: v.label,
        flags: p::Flags {
            conversation: v.flags.conversation,
            listed: v.flags.listed,
            retention: match v.flags.retention {
                c::Retention::Kept => p::Retention::Kept,
                c::Retention::ForgottenOnceRead => p::Retention::ForgottenOnceRead,
            },
        },
    }
}

pub(super) fn propose(project: u32, action: &c::ProposeAction, reason: &c::Text) -> Result<p::Ask, super::Invalid> {
    let action = match action {
        c::ProposeAction::Batch { tasks } => p::Action::Batch(tasks.iter().map(member).collect()),
        c::ProposeAction::Amendment { task, revision, change } => {
            let ask = c::Ask::Amend {
                project,
                task: *task,
                revision: Some(*revision),
                change: change.clone(),
                reason: reason.clone(),
            };
            let p::Ask::Amend { task, revision, amendment, .. } = super::ask(&ask)? else {
                unreachable!("amend translation")
            };
            p::Action::Amend { task, revision, amendment: Box::new(amendment) }
        }
    };
    Ok(p::Ask::Propose { project, action, reason: reason.bytes().into() })
}
