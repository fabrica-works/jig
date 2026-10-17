use super::*;
use skein_lib::List;
pub fn create_ask() -> c::Ask {
    let mut tasks = List::with_capacity(2);
    tasks.push(new_task(c::OwnerExecutor::Agent { charter: 7 }, c::NewContract::Report { words: 17 })).unwrap();
    tasks
        .push(new_task(
            c::OwnerExecutor::Person(c::PersonAddress::Role(2)),
            c::NewContract::Change { connector: 5, kind: 6, words: 18 },
        ))
        .unwrap();
    c::Ask::Create { project: 7, tasks }
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
        wake: read_value_owner_wake_policy(),
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

pub fn list<T, const N: usize>(values: [T; N]) -> List<T> {
    let mut result = List::with_capacity(u32::try_from(N).unwrap());
    for value in values {
        assert!(result.push(value).is_ok());
    }
    result
}
