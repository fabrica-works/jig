//! Permanent people outcomes and their refusal facts (protocol/store.md, 5.1).
use super::{domain, wire};
use skein_lib::{Token, Wall};
mod facts;
use facts::{
    decode_again, decode_escalation_choice, decode_proposal_choice, decode_refusal, encode_again,
    encode_escalation_choice, encode_proposal_choice, encode_refusal,
};
pub(super) fn encode_outcome(value: &domain::Outcome, limits: &wire::Limits) -> Result<wire::Outcome, wire::Problem> {
    Ok(match value {
        domain::Outcome::Created { first, last, given } => encode_created(*first, *last, *given, limits)?,
        domain::Outcome::Closing { chat, cancelled, kept } => {
            wire::Outcome::Closing(encode_closing(*chat, *cancelled, *kept, limits)?)
        }
        domain::Outcome::MarkedSeen { seen } => wire::Outcome::MarkedSeen(marked_seen(seen, limits)?),
        domain::Outcome::Applied { connector, token } => wire::Outcome::Applied(wire::OutcomeApplied::new(
            limits,
            wire::OutcomeAppliedParts { connector: *connector, token: token.raw() },
        )?),
        domain::Outcome::Again(value) => wire::Outcome::Again(wire::OutcomeAgain::new(
            limits,
            wire::OutcomeAgainParts { value: encode_again(*value, limits) },
        )?),
        domain::Outcome::SignedOut => wire::Outcome::SignedOut,
        domain::Outcome::NoteEdited { name } => wire::Outcome::NoteEdited(wire::OutcomeNoteEdited::new(
            limits,
            wire::OutcomeNoteEditedParts { name: *name },
        )?),
        domain::Outcome::ServiceMade { person } => wire::Outcome::ServiceMade(wire::OutcomeServiceMade::new(
            limits,
            wire::OutcomeServiceMadeParts { person: *person },
        )?),
        domain::Outcome::Adopted { project } => {
            wire::Outcome::Adopted(wire::OutcomeAdopted::new(limits, wire::OutcomeAdoptedParts { project: *project })?)
        }
        domain::Outcome::GoalStarted { task } => wire::Outcome::GoalStarted(wire::OutcomeGoalStarted::new(
            limits,
            wire::OutcomeGoalStartedParts { task: *task },
        )?),
        domain::Outcome::GoalProposed { proposal } => wire::Outcome::GoalProposed(wire::OutcomeGoalProposed::new(
            limits,
            wire::OutcomeGoalProposedParts { proposal: *proposal },
        )?),
        domain::Outcome::Stopped { task } => {
            wire::Outcome::Stopped(wire::OutcomeStopped::new(limits, wire::OutcomeStoppedParts { task: *task })?)
        }
        domain::Outcome::Cancelled { task } => {
            wire::Outcome::Cancelled(wire::OutcomeCancelled::new(limits, wire::OutcomeCancelledParts { task: *task })?)
        }
        domain::Outcome::Released { task } => {
            wire::Outcome::Released(wire::OutcomeReleased::new(limits, wire::OutcomeReleasedParts { task: *task })?)
        }
        domain::Outcome::PersonTaken { task } => wire::Outcome::PersonTaken(wire::OutcomePersonTaken::new(
            limits,
            wire::OutcomePersonTakenParts { task: *task },
        )?),
        domain::Outcome::PersonHandedBack { task } => wire::Outcome::PersonHandedBack(
            wire::OutcomePersonHandedBack::new(limits, wire::OutcomePersonHandedBackParts { task: *task })?,
        ),
        domain::Outcome::PersonAnswered { task } => wire::Outcome::PersonAnswered(wire::OutcomePersonAnswered::new(
            limits,
            wire::OutcomePersonAnsweredParts { task: *task },
        )?),
        domain::Outcome::Moved { task } => {
            wire::Outcome::Moved(wire::OutcomeMoved::new(limits, wire::OutcomeMovedParts { task: *task })?)
        }
        domain::Outcome::ProposalDecided { proposer, proposal, by, at, choice } => {
            wire::Outcome::ProposalDecided(encode_proposal_decided(proposer, *proposal, *by, *at, *choice, limits)?)
        }
        domain::Outcome::Said { task, message, released } => {
            wire::Outcome::Said(encode_said(*task, *message, *released, limits)?)
        }
        domain::Outcome::QuestionAnswered { task, question, message } => {
            wire::Outcome::QuestionAnswered(encode_question_answered(*task, *question, *message, limits)?)
        }
        domain::Outcome::Prioritised { project } => wire::Outcome::Prioritised(wire::OutcomePrioritised::new(
            limits,
            wire::OutcomePrioritisedParts { project: *project },
        )?),
        domain::Outcome::Amended { task, released } => wire::Outcome::Amended(wire::OutcomeAmended::new(
            limits,
            wire::OutcomeAmendedParts { task: *task, released: *released },
        )?),
        domain::Outcome::AmendProposed { task, proposal, released } => {
            wire::Outcome::AmendProposed(encode_amend_proposed(*task, *proposal, *released, limits)?)
        }
        domain::Outcome::RolesSet { project } => wire::Outcome::RolesSet(wire::OutcomeRolesSet::new(
            limits,
            wire::OutcomeRolesSetParts { project: *project },
        )?),
        domain::Outcome::PolicyChanged { project } => wire::Outcome::PolicyChanged(wire::OutcomePolicyChanged::new(
            limits,
            wire::OutcomePolicyChangedParts { project: *project },
        )?),
        domain::Outcome::PoolSet { project, person } => wire::Outcome::PoolSet(wire::OutcomePoolSet::new(
            limits,
            wire::OutcomePoolSetParts { project: *project, person: *person },
        )?),
        domain::Outcome::EscalationDecided { task, revision, by, at, choice } => {
            wire::Outcome::EscalationDecided(encode_escalation_decided(*task, *revision, *by, *at, *choice, limits)?)
        }
        domain::Outcome::Started { task } => {
            wire::Outcome::Started(wire::OutcomeStarted::new(limits, wire::OutcomeStartedParts { task: *task })?)
        }
        domain::Outcome::Refused(value) => wire::Outcome::Refused(wire::OutcomeRefused::new(
            limits,
            wire::OutcomeRefusedParts { value: encode_refusal(value, limits)? },
        )?),
    })
}
pub(super) fn decode_outcome(value: wire::Outcome) -> domain::Outcome {
    match value {
        wire::Outcome::Created(value) => decode_created(value),
        wire::Outcome::Closing(value) => decode_closing(value),
        wire::Outcome::MarkedSeen(value) => decode_marked_seen(value),
        wire::Outcome::Applied(value) => {
            let wire::OutcomeAppliedParts { connector, token } = value.into_parts();
            domain::Outcome::Applied { connector, token: Token::new(token) }
        }
        wire::Outcome::Again(value) => {
            let wire::OutcomeAgainParts { value } = value.into_parts();
            domain::Outcome::Again(decode_again(value))
        }
        wire::Outcome::SignedOut => domain::Outcome::SignedOut,
        wire::Outcome::NoteEdited(value) => {
            let wire::OutcomeNoteEditedParts { name } = value.into_parts();
            domain::Outcome::NoteEdited { name }
        }
        wire::Outcome::ServiceMade(value) => {
            let wire::OutcomeServiceMadeParts { person } = value.into_parts();
            domain::Outcome::ServiceMade { person }
        }
        wire::Outcome::Adopted(value) => {
            let wire::OutcomeAdoptedParts { project } = value.into_parts();
            domain::Outcome::Adopted { project }
        }
        wire::Outcome::GoalStarted(value) => {
            let wire::OutcomeGoalStartedParts { task } = value.into_parts();
            domain::Outcome::GoalStarted { task }
        }
        wire::Outcome::GoalProposed(value) => {
            let wire::OutcomeGoalProposedParts { proposal } = value.into_parts();
            domain::Outcome::GoalProposed { proposal }
        }
        wire::Outcome::Stopped(value) => {
            let wire::OutcomeStoppedParts { task } = value.into_parts();
            domain::Outcome::Stopped { task }
        }
        wire::Outcome::Cancelled(value) => {
            let wire::OutcomeCancelledParts { task } = value.into_parts();
            domain::Outcome::Cancelled { task }
        }
        wire::Outcome::Released(value) => {
            let wire::OutcomeReleasedParts { task } = value.into_parts();
            domain::Outcome::Released { task }
        }
        wire::Outcome::PersonTaken(value) => {
            let wire::OutcomePersonTakenParts { task } = value.into_parts();
            domain::Outcome::PersonTaken { task }
        }
        wire::Outcome::PersonHandedBack(value) => {
            let wire::OutcomePersonHandedBackParts { task } = value.into_parts();
            domain::Outcome::PersonHandedBack { task }
        }
        wire::Outcome::PersonAnswered(value) => {
            let wire::OutcomePersonAnsweredParts { task } = value.into_parts();
            domain::Outcome::PersonAnswered { task }
        }
        wire::Outcome::Moved(value) => {
            let wire::OutcomeMovedParts { task } = value.into_parts();
            domain::Outcome::Moved { task }
        }
        wire::Outcome::ProposalDecided(value) => decode_proposal_decided(value),
        wire::Outcome::Said(value) => {
            let wire::OutcomeSaidParts { task, message, released } = value.into_parts();
            domain::Outcome::Said { task, message, released }
        }
        wire::Outcome::QuestionAnswered(value) => decode_question_answered(value),
        wire::Outcome::Prioritised(value) => {
            let wire::OutcomePrioritisedParts { project } = value.into_parts();
            domain::Outcome::Prioritised { project }
        }
        wire::Outcome::Amended(value) => {
            let wire::OutcomeAmendedParts { task, released } = value.into_parts();
            domain::Outcome::Amended { task, released }
        }
        wire::Outcome::AmendProposed(value) => decode_amend_proposed(value),
        wire::Outcome::RolesSet(value) => {
            let wire::OutcomeRolesSetParts { project } = value.into_parts();
            domain::Outcome::RolesSet { project }
        }
        wire::Outcome::PolicyChanged(value) => {
            let wire::OutcomePolicyChangedParts { project } = value.into_parts();
            domain::Outcome::PolicyChanged { project }
        }
        wire::Outcome::PoolSet(value) => {
            let wire::OutcomePoolSetParts { project, person } = value.into_parts();
            domain::Outcome::PoolSet { project, person }
        }
        wire::Outcome::EscalationDecided(value) => decode_escalation_decided(value),
        wire::Outcome::Started(value) => {
            let wire::OutcomeStartedParts { task } = value.into_parts();
            domain::Outcome::Started { task }
        }
        wire::Outcome::Refused(value) => {
            let wire::OutcomeRefusedParts { value } = value.into_parts();
            domain::Outcome::Refused(decode_refusal(value))
        }
    }
}
fn encode_closing(
    chat: u64,
    cancelled: u32,
    kept: u32,
    limits: &wire::Limits,
) -> Result<wire::OutcomeClosing, wire::Problem> {
    wire::OutcomeClosing::new(limits, wire::OutcomeClosingParts { chat, cancelled, kept })
}
fn decode_closing(value: wire::OutcomeClosing) -> domain::Outcome {
    let wire::OutcomeClosingParts { chat, cancelled, kept } = value.into_parts();
    domain::Outcome::Closing { chat, cancelled, kept }
}
fn encode_proposal_decided(
    proposer: &domain::Proposer,
    proposal: u64,
    by: u64,
    at: Wall,
    choice: domain::ProposalChoice,
    limits: &wire::Limits,
) -> Result<wire::OutcomeProposalDecided, wire::Problem> {
    wire::OutcomeProposalDecided::new(
        limits,
        wire::OutcomeProposalDecidedParts {
            proposer: super::asks::encode_proposer(proposer, limits)?,
            proposal,
            by,
            at: at.as_nanos(),
            choice: encode_proposal_choice(choice, limits),
        },
    )
}
fn decode_proposal_decided(value: wire::OutcomeProposalDecided) -> domain::Outcome {
    let wire::OutcomeProposalDecidedParts { proposer, proposal, by, at, choice } = value.into_parts();
    domain::Outcome::ProposalDecided {
        proposer: super::asks::decode_proposer(proposer),
        proposal,
        by,
        at: Wall::from_nanos(at),
        choice: decode_proposal_choice(choice),
    }
}
fn encode_escalation_decided(
    task: u64,
    revision: u64,
    by: u64,
    at: Wall,
    choice: domain::EscalationChoice,
    limits: &wire::Limits,
) -> Result<wire::OutcomeEscalationDecided, wire::Problem> {
    wire::OutcomeEscalationDecided::new(
        limits,
        wire::OutcomeEscalationDecidedParts {
            task,
            revision,
            by,
            at: at.as_nanos(),
            choice: encode_escalation_choice(choice, limits),
        },
    )
}
fn decode_escalation_decided(value: wire::OutcomeEscalationDecided) -> domain::Outcome {
    let wire::OutcomeEscalationDecidedParts { task, revision, by, at, choice } = value.into_parts();
    domain::Outcome::EscalationDecided {
        task,
        revision,
        by,
        at: Wall::from_nanos(at),
        choice: decode_escalation_choice(choice),
    }
}
fn encode_question_answered(
    task: u64,
    question: u64,
    message: u64,
    limits: &wire::Limits,
) -> Result<wire::OutcomeQuestionAnswered, wire::Problem> {
    wire::OutcomeQuestionAnswered::new(limits, wire::OutcomeQuestionAnsweredParts { task, question, message })
}
fn decode_question_answered(value: wire::OutcomeQuestionAnswered) -> domain::Outcome {
    let wire::OutcomeQuestionAnsweredParts { task, question, message } = value.into_parts();
    domain::Outcome::QuestionAnswered { task, question, message }
}
fn encode_amend_proposed(
    task: u64,
    proposal: u64,
    released: bool,
    limits: &wire::Limits,
) -> Result<wire::OutcomeAmendProposed, wire::Problem> {
    wire::OutcomeAmendProposed::new(limits, wire::OutcomeAmendProposedParts { task, proposal, released })
}
fn decode_amend_proposed(value: wire::OutcomeAmendProposed) -> domain::Outcome {
    let wire::OutcomeAmendProposedParts { task, proposal, released } = value.into_parts();
    domain::Outcome::AmendProposed { task, proposal, released }
}

fn encode_said(
    task: u64,
    message: u64,
    released: bool,
    limits: &wire::Limits,
) -> Result<wire::OutcomeSaid, wire::Problem> {
    wire::OutcomeSaid::new(limits, wire::OutcomeSaidParts { task, message, released })
}

fn marked_seen(seen: &domain::Seen, limits: &wire::Limits) -> Result<wire::OutcomeMarkedSeen, wire::Problem> {
    wire::OutcomeMarkedSeen::new(limits, wire::OutcomeMarkedSeenParts { seen: super::asks::encode_seen(seen, limits)? })
}

fn encode_created(
    first: u64,
    last: u64,
    given: Option<domain::Given>,
    limits: &wire::Limits,
) -> Result<wire::Outcome, wire::Problem> {
    Ok(wire::Outcome::Created(wire::OutcomeCreated::new(
        limits,
        wire::OutcomeCreatedParts {
            first,
            last,
            given: match given {
                Some(domain::Given { task, amount }) => {
                    Some(wire::Given::new(limits, wire::GivenParts { task, amount })?)
                }
                None => None,
            },
        },
    )?))
}
fn decode_created(value: wire::OutcomeCreated) -> domain::Outcome {
    let wire::OutcomeCreatedParts { first, last, given } = value.into_parts();
    let mut amount = None;
    if let Some(value) = given {
        let wire::GivenParts { task, amount: value } = value.into_parts();
        amount = Some(domain::Given { task, amount: value });
    }
    domain::Outcome::Created { first, last, given: amount }
}

fn decode_marked_seen(value: wire::OutcomeMarkedSeen) -> domain::Outcome {
    let wire::OutcomeMarkedSeenParts { seen } = value.into_parts();
    domain::Outcome::MarkedSeen { seen: super::asks::decode_seen(seen) }
}
