use super::*;
use jig_domain_people as p;
pub fn create_ask() -> (w::Ask, p::Ask) {
    let (wire, people) = authority_ask();
    let w::Ask::ChangePolicy(wire) = wire else { panic!("policy fixture") };
    let w::PolicyEdit::Role(wire) = wire.into_parts().edit else { panic!("role fixture") };
    let authority = wire.into_parts().value.into_parts().authority;
    let p::Ask::ChangePolicy { change: p::PolicyChange::Role(expected_role), .. } = people else {
        panic!("authority fixture")
    };
    let mut tasks = List::with_capacity(2);
    let mut batch = List::with_capacity(2);
    for role in [false, true] {
        let (task, member) = member(role, authority.clone(), expected_role.authority.clone());
        tasks.push(task).unwrap();
        batch.push(member).unwrap();
    }
    let mut size = 0_usize;
    for task in &tasks {
        size = size.checked_add(usize::try_from(task.measure()).unwrap()).unwrap().checked_add(4).unwrap();
    }
    let mut bytes = Writer::new(size);
    for task in tasks.into_boxed() {
        bytes.put(&task.measure().to_be_bytes()).unwrap();
        task.encode(&mut bytes).unwrap();
    }
    (
        w::Ask::Create(w::Create::new(&super::wire(), w::CreateParts { project: 3, tasks: bytes.finish() }).unwrap()),
        p::Ask::Create { project: 3, batch: batch.into_boxed() },
    )
}
fn member(role: bool, authority: w::OwnerAuthority, expected: p::Authority) -> (w::NewTask, p::Member) {
    let l = wire();
    let dependencies = list([
        w::TaskRef::Member(w::TaskRefMember::new(&l, w::TaskRefMemberParts { value: 1 }).unwrap()),
        w::TaskRef::Task(w::TaskRefTask::new(&l, w::TaskRefTaskParts { value: u64::MAX }).unwrap()),
    ]);
    let wake = w::OwnerWakePolicy::new(
        &l,
        w::OwnerWakePolicyParts {
            words: w::WakeRule::Immediate,
            notices: w::WakeRule::Never,
            news: w::WakeRule::Never,
            results: w::ResultsWake::LastOrFailure,
            questions: true,
            answers: false,
            timers: true,
        },
    )
    .unwrap();
    let spec = w::Specification::new(
        &l,
        w::SpecificationParts {
            words: b"new words".as_slice().into(),
            parameters: list([w::Parameter::Number(
                w::ParameterNumber::new(&l, w::ParameterNumberParts { name: 9, value: 12 }).unwrap(),
            )]),
            inputs: list([77]),
        },
    )
    .unwrap();
    let field = w::FieldRule::new(&l, w::FieldRuleParts { name: b"field".as_slice().into(), max: 13 }).unwrap();
    let choice =
        w::VerdictChoice::new(&l, w::VerdictChoiceParts { code: 5, words: 16, followups: 2, fields: list([field]) })
            .unwrap();
    let executor = if role {
        w::OwnerExecutor::Person(
            w::OwnerExecutorPerson::new(
                &l,
                w::OwnerExecutorPersonParts {
                    value: w::PersonAddress::Role(
                        w::PersonAddressRole::new(&l, w::PersonAddressRoleParts { value: 2 }).unwrap(),
                    ),
                },
            )
            .unwrap(),
        )
    } else {
        w::OwnerExecutor::Agent(w::OwnerExecutorAgent::new(&l, w::OwnerExecutorAgentParts { charter: 7 }).unwrap())
    };
    let task = w::NewTask::new(
        &l,
        w::NewTaskParts {
            executor,
            spec,
            contract: w::NewContract::Verdict(
                w::ContractVerdict::new(&l, w::ContractVerdictParts { choices: list([choice]) }).unwrap(),
            ),
            authority: authority.clone(),
            budget: w::NewBudget::AsAllowed(
                w::BudgetAsAllowed::new(&l, w::BudgetAsAllowedParts { cap: Some(42) }).unwrap(),
            ),
            wake,
            dependencies,
            named: authority.grants().into(),
            tracked: Some(14),
            label: 15,
            flags: w::TaskFlags::new(
                &l,
                w::TaskFlagsParts { conversation: false, listed: true, retention: w::Retention::ForgottenOnceRead },
            )
            .unwrap(),
        },
    )
    .unwrap();
    (task, expected_member(role, expected))
}
fn expected_member(role: bool, expected: p::Authority) -> p::Member {
    p::Member {
        executor: if role {
            p::TaskExecutor::Person(p::PersonAddress::Role(2))
        } else {
            p::TaskExecutor::Agent { charter: 7 }
        },
        words: b"new words".as_slice().into(),
        parameters: Box::new([p::Parameter::Number { name: 9, value: 12 }]),
        inputs: Box::new([77]),
        contract: p::Contract::Verdict {
            choices: Box::new([p::Verdict {
                code: 5,
                words: 16,
                followups: 2,
                fields: Box::new([p::FieldRule { name: b"field".as_slice().into(), max: 13 }]),
            }]),
        },
        authority: expected.clone(),
        budget: p::Budget::AsAllowed { cap: Some(42) },
        wake: p::WakePolicy {
            words: p::WakeRule::Immediate,
            notices: p::WakeRule::Never,
            news: p::WakeRule::Never,
            results: p::ResultsWake::LastOrFailure,
            questions: true,
            answers: false,
            timers: true,
        },
        dependencies: Box::new([p::Dependency::Batch(1), p::Dependency::Existing(u64::MAX)]),
        named: expected.grants,
        tracked: Some(14),
        label: 15,
        flags: p::Flags { conversation: false, listed: true, retention: p::Retention::ForgottenOnceRead },
    }
}
fn list<T, const N: usize>(values: [T; N]) -> List<T> {
    let mut list = List::with_capacity(u32::try_from(N).unwrap());
    for value in values {
        assert!(list.push(value).is_ok());
    }
    list
}

pub fn create_refusals() -> [(&'static str, p::Refusal, w::Refusal); 5] {
    let l = wire();
    [
        (
            "two budgets",
            p::Refusal::NotWellFormed { at: 1, why: p::Malformed::AsAllowedTwice },
            w::Refusal::Malformed(
                w::RefusalMalformed::new(&l, w::RefusalMalformedParts { at: 1, why: w::Malformed::TwoAsAllowed })
                    .unwrap(),
            ),
        ),
        (
            "person conversation",
            p::Refusal::NotWellFormed { at: 2, why: p::Malformed::Conversation },
            w::Refusal::Malformed(
                w::RefusalMalformed::new(
                    &l,
                    w::RefusalMalformedParts { at: 2, why: w::Malformed::ConversationNotAgent },
                )
                .unwrap(),
            ),
        ),
        (
            "cycle",
            p::Refusal::NotWellFormed { at: 3, why: p::Malformed::Cycle },
            w::Refusal::Malformed(
                w::RefusalMalformed::new(&l, w::RefusalMalformedParts { at: 3, why: w::Malformed::Cycle }).unwrap(),
            ),
        ),
        (
            "proposed AsAllowed",
            p::Refusal::NotWellFormed { at: 4, why: p::Malformed::ProposedAsAllowed },
            w::Refusal::Malformed(
                w::RefusalMalformed::new(&l, w::RefusalMalformedParts { at: 4, why: w::Malformed::ProposedAsAllowed })
                    .unwrap(),
            ),
        ),
        (
            "goals",
            p::Refusal::Limit { which: p::LimitKind::Goals },
            w::Refusal::Limit(
                w::RefusalLimit::new(
                    &l,
                    w::RefusalLimitParts {
                        which: w::Limit::Admission(
                            w::LimitAdmission::new(&l, w::LimitAdmissionParts { which: w::AdmissionLimit::Goals })
                                .unwrap(),
                        ),
                    },
                )
                .unwrap(),
            ),
        ),
    ]
}
pub fn created_outcome() -> w::Outcome {
    let l = wire();
    w::Outcome::Owner(
        w::OutcomeOwner::new(
            &l,
            w::OutcomeOwnerParts {
                value: w::OwnerOutcome::Created(
                    w::OwnerOutcomeCreated::new(
                        &l,
                        w::OwnerOutcomeCreatedParts {
                            first: 7,
                            last: 8,
                            given: Some(
                                w::CreatedGiven::new(&l, w::CreatedGivenParts { task: 8, amount: 42 }).unwrap(),
                            ),
                        },
                    )
                    .unwrap(),
                ),
            },
        )
        .unwrap(),
    )
}
