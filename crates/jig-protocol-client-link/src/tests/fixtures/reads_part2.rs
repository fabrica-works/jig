use super::*;
use skein_lib::List;
pub fn read_wire_revision() -> w::Revision {
    let lim = w::CEILINGS;
    w::Revision::new(&lim, w::RevisionParts { value: 7 }).unwrap()
}

pub fn read_wire_phase() -> w::Phase {
    w::Phase::Waiting
}

pub fn read_wire_steering() -> w::Steering {
    let lim = w::CEILINGS;
    w::Steering::new(
        &lim,
        w::SteeringParts {
            addressed: None,
            person_revision: None,
            taken_by: None,
            revision: read_wire_revision(),
            spec: read_wire_specification_document(),
            instructions: None,
            wake: read_wire_owner_wake_policy(),
            dependencies: List::with_capacity(0),
            authority: read_wire_owner_authority_document(),
            budget: read_wire_budget(),
        },
    )
    .unwrap()
}

pub fn read_wire_specification_document() -> w::SpecificationDocument {
    let lim = w::CEILINGS;
    w::SpecificationDocument::new(
        &lim,
        w::SpecificationDocumentParts { specification: read_wire_document_specification(), cut: false },
    )
    .unwrap()
}

pub fn read_wire_document_specification() -> w::DocumentSpecification {
    let lim = w::CEILINGS;
    w::DocumentSpecification::new(
        &lim,
        w::DocumentSpecificationParts {
            words: read_wire_document_text(),
            parameters: List::with_capacity(0),
            inputs: List::with_capacity(0),
        },
    )
    .unwrap()
}

pub fn read_wire_owner_wake_policy() -> w::OwnerWakePolicy {
    let lim = w::CEILINGS;
    w::OwnerWakePolicy::new(
        &lim,
        w::OwnerWakePolicyParts {
            words: read_wire_wake_rule(),
            notices: read_wire_wake_rule(),
            news: read_wire_wake_rule(),
            results: read_wire_results_wake(),
            questions: false,
            answers: false,
            timers: false,
        },
    )
    .unwrap()
}

pub fn read_wire_wake_rule() -> w::WakeRule {
    w::WakeRule::Never
}

pub fn read_wire_results_wake() -> w::ResultsWake {
    w::ResultsWake::Never
}

pub fn read_wire_owner_authority_document() -> w::OwnerAuthorityDocument {
    let lim = w::CEILINGS;
    w::OwnerAuthorityDocument::new(
        &lim,
        w::OwnerAuthorityDocumentParts { authority: read_wire_owner_authority(), cut: false },
    )
    .unwrap()
}

pub fn read_wire_owner_authority() -> w::OwnerAuthority {
    let lim = w::CEILINGS;
    w::OwnerAuthority::new(
        &lim,
        w::OwnerAuthorityParts {
            tools: 7,
            grants: Box::from([]),
            delegation: read_wire_owner_delegation(),
            spend: 7,
            deadline: None,
            notes: 7,
            note_resources: Box::from([]),
        },
    )
    .unwrap()
}

pub fn read_wire_owner_delegation() -> w::OwnerDelegation {
    let lim = w::CEILINGS;
    w::OwnerDelegation::new(&lim, w::OwnerDelegationParts { kinds: List::with_capacity(0), tasks: 7, depth: 7 })
        .unwrap()
}

pub fn read_wire_impact() -> w::Impact {
    let lim = w::CEILINGS;
    w::Impact::new(
        &lim,
        w::ImpactParts {
            tasks: read_wire_phase_counts(),
            live_runs: 7,
            proposals: 7,
            effects_pending: 7,
            effects_settling: 7,
            takeover_budget: 7,
        },
    )
    .unwrap()
}

pub fn read_wire_phase_counts() -> w::PhaseCounts {
    let lim = w::CEILINGS;
    w::PhaseCounts::new(
        &lim,
        w::PhaseCountsParts { ended: 7, waiting: 7, running: 7, held: 7, closing: 7, done: 7, failed: 7, cancelled: 7 },
    )
    .unwrap()
}

pub fn read_wire_owner_escalation() -> w::OwnerEscalation {
    let lim = w::CEILINGS;
    w::OwnerEscalation::new(&lim, w::OwnerEscalationParts { acts: Box::from([]), task: task_record() }).unwrap()
}

pub fn read_wire_question() -> w::Question {
    let lim = w::CEILINGS;
    w::Question::new(
        &lim,
        w::QuestionParts {
            task: chip_record(),
            from: None,
            message: 7,
            words: Box::from([]),
            since: 7,
            acts: Box::from([]),
            ending: None,
        },
    )
    .unwrap()
}

pub fn read_wire_person_task() -> w::PersonTask {
    let lim = w::CEILINGS;
    w::PersonTask::new(
        &lim,
        w::PersonTaskParts {
            task: chip_record(),
            revision: read_wire_revision(),
            requester: read_wire_actor(),
            spec: read_wire_specification_document(),
            contract: read_wire_person_contract(),
            inputs: Box::from([]),
            addressed: read_wire_recipient(),
            taken: None,
            since: None,
            stall: None,
            deadline: None,
            acts: Box::from([]),
            ending: None,
        },
    )
    .unwrap()
}
