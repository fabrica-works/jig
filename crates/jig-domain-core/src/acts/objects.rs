//! Read-only object judgments shared with authenticated requests.
use super::{Act, Admission, allowed, bounds, standing};
use crate::{Core, Limits, documents};
use jig_domain_people as people;
use jig_domain_tasks as tasks;

mod decisions;
mod task;
pub(crate) use decisions::{escalation, proposal};
pub(crate) use task::{close, person, question, write, write_in};

pub(super) fn admit(core: &Core, limits: &Limits, party: u64, act: Act, object: &documents::Subject) -> Admission {
    match object {
        documents::Subject::Task { task: number } => task::admit(core, limits, party, act, *number),
        documents::Subject::PersonTask { task: number } => allowed(person(core, party, act, *number, None)),
        documents::Subject::Question { task: number, message } => {
            allowed(question(core, limits, party, act, *number, *message))
        }
        documents::Subject::Escalation { task } => match act {
            Act::Amend | Act::Cancel => task::admit(core, limits, party, act, *task),
            Act::Message
            | Act::Answer
            | Act::Take
            | Act::HandBack
            | Act::Withdraw
            | Act::Accept
            | Act::Reject
            | Act::PassUp
            | Act::Release
            | Act::LeaveHeld
            | Act::Stop
            | Act::Close
            | Act::Move
            | Act::SetGoal
            | Act::Prioritise
            | Act::Seen
            | Act::EditNote
            | Act::DeleteNote
            | Act::Adopt
            | Act::ChangePolicy
            | Act::SetRoles
            | Act::MakeService
            | Act::SetPool => allowed(escalation(core, limits, party, act, *task)),
        },
        documents::Subject::Proposal { proposer, number } => {
            allowed(proposal(core, limits, party, act, *proposer, *number))
        }
        documents::Subject::Project { project } => project_admission(core, limits, party, act, *project),
        documents::Subject::Result { task } => {
            let Some(row) = core.tasks.task(*task) else { return Admission::Refused(people::Refusal::Unknown) };
            if act == Act::Seen && core.people.role(party, row.project).is_some() {
                Admission::Allowed
            } else {
                Admission::Refused(people::Refusal::NotOffered)
            }
        }
        documents::Subject::Inbox { party: owner } | documents::Subject::Frame { party: owner } => {
            if act == Act::Seen && *owner == party {
                Admission::Allowed
            } else {
                Admission::Refused(people::Refusal::NotOffered)
            }
        }
        documents::Subject::Query { .. }
        | documents::Subject::Note { .. }
        | documents::Subject::Application { .. }
        | documents::Subject::Tree { .. }
        | documents::Subject::Chips { .. }
        | documents::Subject::Run { .. }
        | documents::Subject::Deployment => Admission::Refused(people::Refusal::NotOffered),
    }
}

fn project_admission(core: &Core, limits: &Limits, party: u64, act: Act, project: u32) -> Admission {
    let held = core.people.role(party, project);
    if let Err(why) = standing::role(core, project, held, act) {
        return Admission::Refused(why);
    }
    match act {
        Act::SetGoal => bounds::goal(core, limits, party, project, held),
        Act::Prioritise => allowed(core.priorities_allowed(project, held, 0, limits.tasks.tasks)),
        Act::Adopt | Act::ChangePolicy | Act::SetRoles | Act::MakeService | Act::SetPool => {
            allowed(core.role_allowed(party, project, limits.tasks.tree_tasks))
        }
        Act::Seen => Admission::Allowed,
        Act::Message
        | Act::Answer
        | Act::Take
        | Act::HandBack
        | Act::Withdraw
        | Act::Accept
        | Act::Reject
        | Act::PassUp
        | Act::Release
        | Act::LeaveHeld
        | Act::Amend
        | Act::Stop
        | Act::Cancel
        | Act::Close
        | Act::Move
        | Act::EditNote
        | Act::DeleteNote => Admission::Refused(people::Refusal::NotOffered),
    }
}

pub(super) fn mutable(row: &tasks::TaskRecord) -> Result<(), people::Refusal> {
    mutable_phase(&row.phase)
}
pub(crate) fn mutable_phase(phase: &tasks::Phase) -> Result<(), people::Refusal> {
    match phase {
        tasks::Phase::Closing(closing) => Err(crate::routing::refusals::ended(&closing.ending)),
        tasks::Phase::Ended(ending) => Err(crate::routing::refusals::ended(ending)),
        tasks::Phase::Held { was: tasks::Was::Closing(closing), .. } => {
            Err(crate::routing::refusals::ended(&closing.ending))
        }
        tasks::Phase::Held { was: tasks::Was::Waiting | tasks::Was::Active(_), .. }
        | tasks::Phase::Waiting
        | tasks::Phase::Active(_) => Ok(()),
    }
}
