use super::*;
use skein_lib::List;
pub const PROJECT: u32 = 7;
pub const CHARTER: u32 = 7;
pub const ROLE: u32 = 2;
pub fn create_ask() -> c::Ask {
    let mut tasks = List::with_capacity(2);
    tasks.push(new_task(c::OwnerExecutor::Agent { charter: CHARTER }, c::NewContract::Report { words: 17 })).unwrap();
    tasks
        .push(new_task(
            c::OwnerExecutor::Person(c::PersonAddress::Role(ROLE)),
            c::NewContract::Change { connector: 5, kind: 6, words: 18 },
        ))
        .unwrap();
    c::Ask::Create { project: PROJECT, tasks }
}
pub fn new_task(executor: c::OwnerExecutor, contract: c::NewContract) -> c::NewTask {
    c::NewTask {
        executor,
        spec: c::Specification {
            words: store_text(),
            parameters: list([c::Parameter::Number { name: 5, value: 6 }]),
            inputs: list([u64::MAX]),
        },
        contract,
        authority: read_value_owner_authority(),
        budget: c::NewBudget::AsAllowed { cap: Some(99) },
        wake: Some(read_value_owner_wake_policy()),
        dependencies: list([c::TaskRef::Member(1), c::TaskRef::Task(u64::MAX)]),
        named: list([c::OwnerGrant {
            connector: 5,
            kind: 6,
            pattern: c::OwnerPattern {
                segments: list([b"raw".as_slice().into()]),
                last: c::OwnerPatternEnd::Open(b"tail".as_slice().into()),
            },
        }]),
        tracked: Some(13),
        label: 14,
        flags: c::TaskFlags { conversation: false, listed: true, retention: c::Retention::ForgottenOnceRead },
    }
}
