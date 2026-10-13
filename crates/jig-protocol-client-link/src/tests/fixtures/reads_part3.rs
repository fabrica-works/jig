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
    let lim = w::CEILINGS;
    w::OwnerPersonTask::new(
        &lim,
        w::OwnerPersonTaskParts {
            task: chip_record(),
            revision: read_wire_revision(),
            requester: read_wire_owner_requester(),
            spec: read_wire_specification_document(),
            address: read_wire_person_address(),
            taken_by: None,
            since: None,
            stall: None,
            deadline: None,
            contract: read_wire_task_contract_document(),
            results: Box::from([]),
            cut: false,
            acts: Box::from([]),
        },
    )
    .unwrap()
}

pub fn read_wire_owner_requester() -> w::OwnerRequester {
    w::OwnerRequester::Task(read_wire_owner_requester_task())
}

pub fn read_wire_owner_requester_task() -> w::OwnerRequesterTask {
    let lim = w::CEILINGS;
    w::OwnerRequesterTask::new(&lim, w::OwnerRequesterTaskParts { value: 7 }).unwrap()
}

pub fn read_wire_person_address() -> w::PersonAddress {
    w::PersonAddress::Party(read_wire_person_address_party())
}

pub fn read_wire_person_address_party() -> w::PersonAddressParty {
    let lim = w::CEILINGS;
    w::PersonAddressParty::new(&lim, w::PersonAddressPartyParts { value: 7 }).unwrap()
}

pub fn read_wire_task_contract_document() -> w::TaskContractDocument {
    w::TaskContractDocument::Approval(read_wire_task_contract_document_approval())
}

pub fn read_wire_task_contract_document_approval() -> w::TaskContractDocumentApproval {
    let lim = w::CEILINGS;
    w::TaskContractDocumentApproval::new(
        &lim,
        w::TaskContractDocumentApprovalParts { state: Box::from([]), part: read_wire_owner_part() },
    )
    .unwrap()
}

pub fn read_wire_owner_part() -> w::OwnerPart {
    let lim = w::CEILINGS;
    w::OwnerPart::new(
        &lim,
        w::OwnerPartParts { connector: 7, kind: read_wire_owner_part_kind(), key: read_wire_owner_part_key() },
    )
    .unwrap()
}

pub fn read_wire_owner_part_kind() -> w::OwnerPartKind {
    w::OwnerPartKind::Resource
}

pub fn read_wire_owner_part_key() -> w::OwnerPartKey {
    w::OwnerPartKey::Path(read_wire_owner_part_key_path())
}

pub fn read_wire_owner_part_key_path() -> w::OwnerPartKeyPath {
    let lim = w::CEILINGS;
    w::OwnerPartKeyPath::new(&lim, w::OwnerPartKeyPathParts { value: List::with_capacity(0) }).unwrap()
}

pub fn read_wire_result() -> w::ResultDocument {
    let lim = w::CEILINGS;
    w::ResultDocument::new(
        &lim,
        w::ResultDocumentParts {
            task: chip_record(),
            ending: read_ending_record(),
            value: None,
            at: None,
            details: None,
            acts: Box::from([]),
        },
    )
    .unwrap()
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
