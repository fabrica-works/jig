//! Presentation of committed owner results (client/domain/contract.md, 4;
//! client/domain/view.md, 7). Reads supplied facts only: no request, current
//! object, name lookup or explanation is used to fill a missing fact.
use crate::owner_outcome_words;
use crate::owner_outcome_words::OutcomePhrase as P;
use crate::words::Words;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use jig_client::contract::{
    OwnerEscalationChoice, OwnerOutcome, OwnerProposalChoice, ProposalKey, Proposer, Revision, SeenThrough,
};
use skein_lib::Wall;

/// A finite fragment envelope reusable by an owner's enclosing view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutcomeFragment {
    pub nodes: u32,
    pub text_bytes: u32,
    pub text_per_node: u32,
    pub depth: u32,
}
/// Maximum fragment over all committed kinds and every loaded language.
/// One region, one named status with its text, and at most five fact rows.
#[must_use]
pub fn required(words: &Words) -> Option<OutcomeFragment> {
    let mut capacity = 0_u32;
    for phrase in owner_outcome_words::ALL {
        capacity = capacity.max(owner_outcome_words::capacity(words, phrase));
    }
    Some(OutcomeFragment { nodes: 8, text_bytes: capacity.checked_mul(7)?, text_per_node: capacity, depth: 3 })
}
fn header(value: &OwnerOutcome) -> P {
    match value {
        OwnerOutcome::Compound { .. } => P::Compound,
        OwnerOutcome::Closing { .. } => P::Closing,
        OwnerOutcome::MarkedSeen { .. } => P::MarkedSeen,
        OwnerOutcome::NoteEdited { .. } => P::NoteEdited,
        OwnerOutcome::ServiceMade { .. } => P::ServiceMade,
        OwnerOutcome::Adopted { .. } => P::Adopted,
        OwnerOutcome::GoalStarted { .. } => P::GoalStarted,
        OwnerOutcome::GoalProposed { .. } => P::GoalProposed,
        OwnerOutcome::Stopped { .. } => P::Stopped,
        OwnerOutcome::Cancelled { .. } => P::Cancelled,
        OwnerOutcome::Released { .. } => P::Released,
        OwnerOutcome::PersonTaken { .. } => P::PersonTaken,
        OwnerOutcome::PersonHandedBack { .. } => P::PersonHandedBack,
        OwnerOutcome::PersonAnswered { .. } => P::PersonAnswered,
        OwnerOutcome::Moved { .. } => P::Moved,
        OwnerOutcome::ProposalDecided { .. } => P::ProposalDecided,
        OwnerOutcome::Said { .. } => P::Said,
        OwnerOutcome::QuestionAnswered { .. } => P::QuestionAnswered,
        OwnerOutcome::Prioritised { .. } => P::Prioritised,
        OwnerOutcome::Amended { .. } => P::Amended,
        OwnerOutcome::AmendProposed { .. } => P::AmendProposed,
        OwnerOutcome::RolesSet { .. } => P::RolesSet,
        OwnerOutcome::PolicyChanged { .. } => P::PolicyChanged,
        OwnerOutcome::PoolSet { .. } => P::PoolSet,
        OwnerOutcome::EscalationDecided { .. } => P::EscalationDecided,
        OwnerOutcome::Started { .. } => P::Started,
    }
}
fn text(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    let saying = owner_outcome_words::say(words, phrase, holes);
    b.text(Run::Plain, &saying);
}
fn number(b: &mut Builder, words: &Words, phrase: P, value: u64) {
    text(b, words, phrase, &[Hole::Number(value)]);
}
fn seen(b: &mut Builder, value: &SeenThrough, words: &Words) {
    match value {
        SeenThrough::Results { through } => number(b, words, P::ResultsThrough, *through),
        SeenThrough::Conversation { task: chat, through } => {
            number(b, words, P::Chat, *chat);
            number(b, words, P::RepliesThrough, *through);
        }
    }
}
fn proposal(b: &mut Builder, key: &ProposalKey, by: u64, at: Wall, choice: OwnerProposalChoice, words: &Words) {
    match key.proposer {
        Proposer::Task(task) => number(b, words, P::TaskProposer, task),
        Proposer::Party(party) => number(b, words, P::PartyProposer, party),
    }
    number(b, words, P::ProposalNumber, key.number);
    number(b, words, P::By, by);
    text(b, words, P::At, &[Hole::Wall(words.adjusted_wall(at))]);
    let phrase = match choice {
        OwnerProposalChoice::Accepted => P::Accepted,
        OwnerProposalChoice::Rejected => P::Rejected,
        OwnerProposalChoice::Passed => P::Passed,
        OwnerProposalChoice::Withdrawn => P::Withdrawn,
        OwnerProposalChoice::Stale => P::Stale,
    };
    text(b, words, phrase, &[]);
}
fn escalation(
    b: &mut Builder,
    task: u64,
    revision: Revision,
    by: u64,
    at: Wall,
    choice: OwnerEscalationChoice,
    words: &Words,
) {
    number(b, words, P::Task, task);
    number(b, words, P::Revision, revision.value);
    number(b, words, P::By, by);
    text(b, words, P::At, &[Hole::Wall(words.adjusted_wall(at))]);
    let phrase = match choice {
        OwnerEscalationChoice::Released => P::ReleaseChoice,
        OwnerEscalationChoice::Rejected => P::Rejected,
        OwnerEscalationChoice::Passed => P::Passed,
    };
    text(b, words, phrase, &[]);
}
fn details(b: &mut Builder, value: &OwnerOutcome, words: &Words) {
    match value {
        OwnerOutcome::Closing { chat, cancelled, kept } => {
            number(b, words, P::Chat, *chat);
            number(b, words, P::CancelledCount, u64::from(*cancelled));
            number(b, words, P::KeptCount, u64::from(*kept));
        }
        OwnerOutcome::MarkedSeen { seen: position } => seen(b, position, words),
        OwnerOutcome::NoteEdited { name } => number(b, words, P::Note, *name),
        OwnerOutcome::ServiceMade { person } => number(b, words, P::Person, *person),
        OwnerOutcome::Adopted { project }
        | OwnerOutcome::Prioritised { project }
        | OwnerOutcome::RolesSet { project }
        | OwnerOutcome::PolicyChanged { project } => number(b, words, P::Project, u64::from(*project)),
        OwnerOutcome::GoalStarted { task }
        | OwnerOutcome::Stopped { task }
        | OwnerOutcome::Cancelled { task }
        | OwnerOutcome::Released { task }
        | OwnerOutcome::PersonTaken { task }
        | OwnerOutcome::PersonHandedBack { task }
        | OwnerOutcome::PersonAnswered { task }
        | OwnerOutcome::Moved { task }
        | OwnerOutcome::Started { task } => number(b, words, P::Task, *task),
        OwnerOutcome::GoalProposed { proposal } => number(b, words, P::ProposalNumber, *proposal),
        OwnerOutcome::ProposalDecided { proposal: key, by, at, choice } => proposal(b, key, *by, *at, *choice, words),
        OwnerOutcome::Compound { project, operations, first_message } => {
            number(b, words, P::Project, u64::from(*project));
            number(b, words, P::Operations, u64::from(*operations));
            number(b, words, P::Message, *first_message);
        }
        OwnerOutcome::Said { task, message } => {
            number(b, words, P::Task, *task);
            number(b, words, P::Message, *message);
        }
        OwnerOutcome::QuestionAnswered { task, question, message } => {
            number(b, words, P::Task, *task);
            number(b, words, P::Question, *question);
            number(b, words, P::Message, *message);
        }
        OwnerOutcome::Amended { task } => {
            number(b, words, P::Task, *task);
        }
        OwnerOutcome::AmendProposed { task, proposal } => {
            number(b, words, P::Task, *task);
            number(b, words, P::ProposalNumber, *proposal);
        }
        OwnerOutcome::PoolSet { project, person } => {
            number(b, words, P::Project, u64::from(*project));
            number(b, words, P::Person, *person);
        }
        OwnerOutcome::EscalationDecided { task, revision, by, at, choice } => {
            escalation(b, *task, *revision, *by, *at, *choice, words);
        }
    }
}
/// Build one bounded fragment using only the committed value and selected Words.
pub fn render(b: &mut Builder, value: &OwnerOutcome, words: &Words) {
    b.open(Role::Region);
    let title = owner_outcome_words::say(words, header(value), &[]);
    b.open(Role::Status);
    b.name(&title);
    b.text(Run::Plain, &title);
    b.close();
    details(b, value, words);
    b.close();
}
