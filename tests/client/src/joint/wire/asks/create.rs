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
        wake: request_arguments::wake(v.wake),
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
