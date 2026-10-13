use super::*;
use skein_lib::List;
pub fn read_wire_person_contract() -> w::PersonContract {
    w::PersonContract::Choice(read_wire_person_contract_choice())
}

pub fn read_wire_person_contract_choice() -> w::PersonContractChoice {
    let lim = w::CEILINGS;
    w::PersonContractChoice::new(&lim, w::PersonContractChoiceParts { options: Box::from([]) }).unwrap()
}

pub fn read_wire_owner_person_task() -> w::OwnerPersonTask {
    jig_test_client_records::read_wire_owner_person_task()
}

pub fn read_wire_result() -> w::ResultDocument {
    jig_test_client_records::read_wire_result()
}

pub fn read_wire_project() -> w::Project {
    let lim = w::CEILINGS;
    w::Project::new(
        &lim,
        w::ProjectParts {
            period: 7,
            spend_lines: Box::from([]),
            number: 7,
            name: Box::from([]),
            policy: read_wire_policy(),
            roles: List::with_capacity(0),
            spend: read_wire_budget(),
            pools: List::with_capacity(0),
            resources: Box::from([]),
            services: List::with_capacity(0),
            recurring: Box::from([]),
            parts: Box::from([]),
            acts: Box::from([]),
        },
    )
    .unwrap()
}

pub fn read_wire_policy() -> w::Policy {
    let lim = w::CEILINGS;
    w::Policy::new(
        &lim,
        w::PolicyParts {
            ceiling: read_wire_authority(),
            period: 7,
            spend: 7,
            roles: Box::from([]),
            requirements: Box::from([]),
            deciders: List::with_capacity(0),
        },
    )
    .unwrap()
}
