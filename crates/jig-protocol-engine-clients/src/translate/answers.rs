//! Durable keyed answers (protocol/client.md, section 5; contract.md, 4).
use super::refusals;
use alloc::boxed::Box;
use jig_client_protocol_wire::v8 as w;
use jig_domain_people as p;
use skein_lib::Writer;
pub(crate) fn answer(reply: &p::Reply, limits: &w::Limits) -> Result<w::Answer, w::Problem> {
    let final_answer = match reply {
        p::Reply::Outcome(p::Outcome::Again(why)) | p::Reply::Again(why) => return Ok(again(*why)),
        p::Reply::Outcome(p::Outcome::SignedOut) | p::Reply::SignedOut => return Ok(w::Answer::SignedOut),
        p::Reply::Outcome(p::Outcome::Refused(why)) | p::Reply::Refused(why) => {
            w::Final::Refused(w::Refused::new(limits, w::RefusedParts { refusal: refusals::refusal(*why, limits)? })?)
        }
        p::Reply::Outcome(p::Outcome::ServiceMade { person }) => {
            let what = w::Made::Service(w::MadeService::new(
                limits,
                w::MadeServiceParts { party: *person, credential: skein_lib::Secret::new(Box::new([])) },
            )?);
            let outcome = w::Outcome::Made(w::OutcomeMade::new(limits, w::OutcomeMadeParts { what })?);
            w::Final::Done(w::Done::new(limits, w::DoneParts { outcome })?)
        }
        p::Reply::Outcome(value) => {
            let value = owner(value, limits)?;
            let outcome = w::Outcome::Owner(w::OutcomeOwner::new(limits, w::OutcomeOwnerParts { value })?);
            w::Final::Done(w::Done::new(limits, w::DoneParts { outcome })?)
        }
        p::Reply::SignedIn { .. } => return Err(problem()),
    };
    let mut out = Writer::new(usize::try_from(final_answer.measure()).expect("wire size fits"));
    final_answer.encode(&mut out).expect("measured final answer");
    owned(w::Owner::Jig, jig_client_protocol_wire::VERSION, out.finish(), limits)
}
pub(crate) fn service(party: u64, credential: skein_lib::Secret, limits: &w::Limits) -> Result<w::Answer, w::Problem> {
    let what = w::Made::Service(w::MadeService::new(limits, w::MadeServiceParts { party, credential })?);
    let outcome = w::Outcome::Made(w::OutcomeMade::new(limits, w::OutcomeMadeParts { what })?);
    let answer = w::Final::Done(w::Done::new(limits, w::DoneParts { outcome })?);
    let mut out = Writer::new(usize::try_from(answer.measure()).expect("wire size fits"));
    answer.encode(&mut out).expect("measured final answer");
    owned(w::Owner::Jig, jig_client_protocol_wire::VERSION, out.finish(), limits)
}
pub(crate) fn owned(
    owner: w::Owner,
    version: u32,
    record: Box<[u8]>,
    limits: &w::Limits,
) -> Result<w::Answer, w::Problem> {
    Ok(w::Answer::FinalAnswer(w::Owned::new(limits, w::OwnedParts { owner, version, record })?))
}
fn again(value: p::Again) -> w::Answer {
    match value {
        p::Again::Busy => w::Answer::Busy,
        p::Again::NotReady => w::Answer::NotReady,
    }
}
fn problem() -> w::Problem {
    w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Malformed }
}
#[expect(clippy::too_many_lines, reason = "one total match names every people outcome")]
fn owner(value: &p::Outcome, limits: &w::Limits) -> Result<w::OwnerOutcome, w::Problem> {
    Ok(match value {
        p::Outcome::Closing { chat, cancelled, kept } => w::OwnerOutcome::Closing(w::OwnerOutcomeClosing::new(
            limits,
            w::OwnerOutcomeClosingParts { chat: *chat, cancelled: *cancelled, kept: *kept },
        )?),
        p::Outcome::NoteEdited { name } => w::OwnerOutcome::NoteEdited(w::OwnerOutcomeNoteEdited::new(
            limits,
            w::OwnerOutcomeNoteEditedParts { name: *name },
        )?),
        p::Outcome::ServiceMade { person } => w::OwnerOutcome::ServiceMade(w::OwnerOutcomeServiceMade::new(
            limits,
            w::OwnerOutcomeServiceMadeParts { person: *person },
        )?),
        p::Outcome::Adopted { project } => w::OwnerOutcome::Adopted(w::OwnerOutcomeAdopted::new(
            limits,
            w::OwnerOutcomeAdoptedParts { project: *project },
        )?),
        p::Outcome::GoalStarted { task } => w::OwnerOutcome::GoalStarted(w::OwnerOutcomeGoalStarted::new(
            limits,
            w::OwnerOutcomeGoalStartedParts { task: *task },
        )?),
        p::Outcome::GoalProposed { proposal } => w::OwnerOutcome::GoalProposed(w::OwnerOutcomeGoalProposed::new(
            limits,
            w::OwnerOutcomeGoalProposedParts { proposal: *proposal },
        )?),
        p::Outcome::Stopped { task } => {
            w::OwnerOutcome::Stopped(w::OwnerOutcomeStopped::new(limits, w::OwnerOutcomeStoppedParts { task: *task })?)
        }
        p::Outcome::Cancelled { task } => w::OwnerOutcome::Cancelled(w::OwnerOutcomeCancelled::new(
            limits,
            w::OwnerOutcomeCancelledParts { task: *task },
        )?),
        p::Outcome::Released { task } => w::OwnerOutcome::Released(w::OwnerOutcomeReleased::new(
            limits,
            w::OwnerOutcomeReleasedParts { task: *task },
        )?),
        p::Outcome::PersonTaken { task } => w::OwnerOutcome::PersonTaken(w::OwnerOutcomePersonTaken::new(
            limits,
            w::OwnerOutcomePersonTakenParts { task: *task },
        )?),
        p::Outcome::PersonHandedBack { task } => w::OwnerOutcome::PersonHandedBack(
            w::OwnerOutcomePersonHandedBack::new(limits, w::OwnerOutcomePersonHandedBackParts { task: *task })?,
        ),
        p::Outcome::PersonAnswered { task } => w::OwnerOutcome::PersonAnswered(w::OwnerOutcomePersonAnswered::new(
            limits,
            w::OwnerOutcomePersonAnsweredParts { task: *task },
        )?),
        p::Outcome::Moved { task } => {
            w::OwnerOutcome::Moved(w::OwnerOutcomeMoved::new(limits, w::OwnerOutcomeMovedParts { task: *task })?)
        }
        p::Outcome::Said { task, message, released } => w::OwnerOutcome::Said(w::OwnerOutcomeSaid::new(
            limits,
            w::OwnerOutcomeSaidParts { task: *task, message: *message, released: *released },
        )?),
        p::Outcome::QuestionAnswered { task, question, message } => {
            w::OwnerOutcome::QuestionAnswered(w::OwnerOutcomeQuestionAnswered::new(
                limits,
                w::OwnerOutcomeQuestionAnsweredParts { task: *task, question: *question, message: *message },
            )?)
        }
        p::Outcome::Prioritised { project } => w::OwnerOutcome::Prioritised(w::OwnerOutcomePrioritised::new(
            limits,
            w::OwnerOutcomePrioritisedParts { project: *project },
        )?),
        p::Outcome::Amended { task, released } => w::OwnerOutcome::Amended(w::OwnerOutcomeAmended::new(
            limits,
            w::OwnerOutcomeAmendedParts { task: *task, released: *released },
        )?),
        p::Outcome::AmendProposed { task, proposal, released } => {
            w::OwnerOutcome::AmendProposed(w::OwnerOutcomeAmendProposed::new(
                limits,
                w::OwnerOutcomeAmendProposedParts { task: *task, proposal: *proposal, released: *released },
            )?)
        }
        p::Outcome::RolesSet { project } => w::OwnerOutcome::RolesSet(w::OwnerOutcomeRolesSet::new(
            limits,
            w::OwnerOutcomeRolesSetParts { project: *project },
        )?),
        p::Outcome::PolicyChanged { project } => w::OwnerOutcome::PolicyChanged(w::OwnerOutcomePolicyChanged::new(
            limits,
            w::OwnerOutcomePolicyChangedParts { project: *project },
        )?),
        p::Outcome::PoolSet { project, person } => w::OwnerOutcome::PoolSet(w::OwnerOutcomePoolSet::new(
            limits,
            w::OwnerOutcomePoolSetParts { project: *project, person: *person },
        )?),
        p::Outcome::Started { task } => {
            w::OwnerOutcome::Started(w::OwnerOutcomeStarted::new(limits, w::OwnerOutcomeStartedParts { task: *task })?)
        }
        p::Outcome::MarkedSeen { seen } => w::OwnerOutcome::MarkedSeen(w::OwnerOutcomeMarkedSeen::new(
            limits,
            w::OwnerOutcomeMarkedSeenParts { seen: self::seen(*seen, limits)? },
        )?),
        p::Outcome::ProposalDecided { proposer, proposal, by, at, choice } => {
            let proposer = match proposer {
                p::Proposer::Task(value) => {
                    w::Proposer::Task(w::ProposerTask::new(limits, w::ProposerTaskParts { value: *value })?)
                }
                p::Proposer::Party(value) => {
                    w::Proposer::Party(w::ProposerParty::new(limits, w::ProposerPartyParts { value: *value })?)
                }
            };
            let proposal = w::ProposalKey::new(limits, w::ProposalKeyParts { proposer, number: *proposal })?;
            w::OwnerOutcome::ProposalDecided(w::OwnerOutcomeProposalDecided::new(
                limits,
                w::OwnerOutcomeProposalDecidedParts {
                    proposal,
                    by: *by,
                    at: at.as_nanos(),
                    choice: proposal_choice(*choice),
                },
            )?)
        }
        p::Outcome::EscalationDecided { task, revision, by, at, choice } => {
            let revision = w::Revision::new(limits, w::RevisionParts { value: *revision })?;
            w::OwnerOutcome::EscalationDecided(w::OwnerOutcomeEscalationDecided::new(
                limits,
                w::OwnerOutcomeEscalationDecidedParts {
                    task: *task,
                    revision,
                    by: *by,
                    at: at.as_nanos(),
                    choice: escalation_choice(*choice),
                },
            )?)
        }
        p::Outcome::Applied { .. } | p::Outcome::Again(_) | p::Outcome::SignedOut | p::Outcome::Refused(_) => {
            return Err(problem());
        }
    })
}
fn seen(value: p::Seen, limits: &w::Limits) -> Result<w::Seen, w::Problem> {
    Ok(match value {
        p::Seen::Results { through } => w::Seen::Results(w::SeenResults::new(limits, w::SeenResultsParts { through })?),
        p::Seen::Chat { task, through } => {
            w::Seen::Replies(w::SeenReplies::new(limits, w::SeenRepliesParts { chat: task, through })?)
        }
    })
}
fn proposal_choice(value: p::ProposalChoice) -> w::OwnerProposalChoice {
    match value {
        p::ProposalChoice::Accepted => w::OwnerProposalChoice::Accepted,
        p::ProposalChoice::Rejected => w::OwnerProposalChoice::Rejected,
        p::ProposalChoice::Passed => w::OwnerProposalChoice::Passed,
        p::ProposalChoice::Withdrawn => w::OwnerProposalChoice::Withdrawn,
        p::ProposalChoice::Stale => w::OwnerProposalChoice::Stale,
    }
}
fn escalation_choice(value: p::EscalationChoice) -> w::OwnerEscalationChoice {
    match value {
        p::EscalationChoice::Released => w::OwnerEscalationChoice::Released,
        p::EscalationChoice::Rejected => w::OwnerEscalationChoice::Rejected,
        p::EscalationChoice::Passed => w::OwnerEscalationChoice::Passed,
    }
}
