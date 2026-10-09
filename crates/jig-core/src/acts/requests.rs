//! Current object judgments at the one authenticated keyed entrance.
use super::{Act, Admission, admit, standing};
use crate::{Core, Limits, documents};
use jig_core_people as people;

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive keyed entrance preserves the historical and live dispatch precedence"
)]
pub(crate) fn request(
    core: &Core,
    limits: &Limits,
    person: u64,
    project: u32,
    role: Option<people::Role>,
    ask: &people::Ask,
) -> Result<(), people::Refusal> {
    let (act, subject) = match ask {
        people::Ask::Application { .. } => return Ok(()),
        people::Ask::StartChat { .. } => return standing::role(core, project, role, Act::SetGoal),
        people::Ask::Say { task, .. } => (Act::Write, documents::Subject::Task { task: *task }),
        people::Ask::AnswerQuestion { task, question, .. } => {
            (Act::Answer, documents::Subject::Question { task: *task, message: *question })
        }
        people::Ask::TakePerson { task, .. } => (Act::Take, documents::Subject::PersonTask { task: *task }),
        people::Ask::HandBackPerson { task, .. } => (Act::HandBack, documents::Subject::PersonTask { task: *task }),
        people::Ask::AnswerPerson { task, .. } => (Act::Answer, documents::Subject::PersonTask { task: *task }),
        people::Ask::DecideProposal { proposer, proposal, decision, .. } => {
            let act = match decision {
                people::ProposalDecision::Accept => Act::Accept,
                people::ProposalDecision::Reject { .. } => Act::Reject,
                people::ProposalDecision::Pass => Act::PassUp,
            };
            let proposer = if core.tasks.person_proposal(*proposer, *proposal).is_some() {
                documents::Proposer::Party(*proposer)
            } else {
                documents::Proposer::Task(*proposer)
            };
            (act, documents::Subject::Proposal { proposer, number: *proposal })
        }
        people::Ask::DecideEscalation { task, decision, .. } => {
            let act = match decision {
                people::EscalationDecision::Release => Act::Release,
                people::EscalationDecision::Reject { .. } => Act::LeaveHeld,
                people::EscalationDecision::Pass => Act::PassUp,
            };
            (act, documents::Subject::Escalation { task: *task })
        }
        people::Ask::Release { task, .. } => (Act::Release, documents::Subject::Task { task: *task }),
        people::Ask::Amend { task, release, .. } => {
            let held = match core.tasks.task(*task) {
                Some(row) => match row.phase {
                    jig_core_tasks::Phase::Held { .. } => true,
                    jig_core_tasks::Phase::Waiting
                    | jig_core_tasks::Phase::Active(_)
                    | jig_core_tasks::Phase::Closing(_)
                    | jig_core_tasks::Phase::Ended(_) => false,
                },
                None => false,
            };
            (
                if release.is_some() && held { Act::AmendRelease } else { Act::Amend },
                documents::Subject::Task { task: *task },
            )
        }
        people::Ask::Stop { task, .. } => (Act::Stop, documents::Subject::Task { task: *task }),
        people::Ask::Cancel { task, .. } => (Act::Cancel, documents::Subject::Task { task: *task }),
        people::Ask::Close { chat, .. } => (Act::Close, documents::Subject::Task { task: *chat }),
        people::Ask::Move { task, .. } => (Act::TakeOver, documents::Subject::Task { task: *task }),
        people::Ask::SetGoal { .. } => (Act::SetGoal, documents::Subject::Project { project }),
        people::Ask::Prioritise { .. } => (Act::Prioritise, documents::Subject::Goals { project }),
        people::Ask::MarkSeen { seen } => match seen {
            people::Seen::Results { .. } => (Act::MarkSeen, documents::Subject::Inbox { party: person }),
            people::Seen::Chat { task, .. } => (Act::MarkSeen, documents::Subject::Task { task: *task }),
        },
        people::Ask::EditNote { scope, change, .. } => {
            return super::note(
                core,
                person,
                project,
                scope,
                match &**change {
                    people::NoteChange::Correct { .. } => Act::EditNote,
                    people::NoteChange::Delete { .. } => Act::DeleteNote,
                },
            );
        }
        people::Ask::Adopt { .. } => (Act::Adopt, documents::Subject::Project { project }),
        people::Ask::ChangePolicy { .. } => (Act::ChangePolicy, documents::Subject::Project { project }),
        people::Ask::SetRoles { .. } => (Act::SetRoles, documents::Subject::Project { project }),
        people::Ask::MakeService { .. } => (Act::MakeService, documents::Subject::Project { project }),
        people::Ask::SetPool { .. } => (Act::SetPool, documents::Subject::Project { project }),
    };
    match ask {
        // The person-task entrance compares the named revision before phase
        // eligibility, and preserves the historical ended-task lookup.
        people::Ask::AnswerPerson { .. } => return Ok(()),
        people::Ask::DecideProposal { proposer, proposal, .. } => {
            if core.tasks.person_proposal(*proposer, *proposal).is_none()
                && core.tasks.proposal(*proposer, *proposal).is_none()
            {
                return Ok(());
            }
        }
        people::Ask::DecideEscalation { task, revision, .. } => {
            let Some(context) = core.tasks.escalation(*task) else {
                return Ok(());
            };
            match context.escalation {
                jig_core_tasks::Escalation::Waiting { revision: now, .. } if now == *revision => {}
                jig_core_tasks::Escalation::Waiting { .. }
                | jig_core_tasks::Escalation::Unheld { .. }
                | jig_core_tasks::Escalation::Routing { .. }
                | jig_core_tasks::Escalation::Rejected { .. } => return Ok(()),
            }
        }
        people::Ask::Application { .. }
        | people::Ask::StartChat { .. }
        | people::Ask::Say { .. }
        | people::Ask::AnswerQuestion { .. }
        | people::Ask::TakePerson { .. }
        | people::Ask::HandBackPerson { .. }
        | people::Ask::Release { .. }
        | people::Ask::Amend { .. }
        | people::Ask::Stop { .. }
        | people::Ask::Cancel { .. }
        | people::Ask::Close { .. }
        | people::Ask::Move { .. }
        | people::Ask::SetGoal { .. }
        | people::Ask::Prioritise { .. }
        | people::Ask::MarkSeen { .. }
        | people::Ask::EditNote { .. }
        | people::Ask::Adopt { .. }
        | people::Ask::ChangePolicy { .. }
        | people::Ask::SetRoles { .. }
        | people::Ask::MakeService { .. }
        | people::Ask::SetPool { .. } => {}
    }
    match &subject {
        documents::Subject::PersonTask { task } => {
            if core.tasks.task(*task).is_none() {
                return Ok(());
            }
        }
        documents::Subject::Task { task } if act == Act::MarkSeen && core.tasks.task(*task).is_none() => return Ok(()),
        documents::Subject::Task { .. }
        | documents::Subject::Application { .. }
        | documents::Subject::Frame { .. }
        | documents::Subject::Tree { .. }
        | documents::Subject::Chips { .. }
        | documents::Subject::Inbox { .. }
        | documents::Subject::Goals { .. }
        | documents::Subject::Conversations { .. }
        | documents::Subject::Run { .. }
        | documents::Subject::Deployment
        | documents::Subject::Proposal { .. }
        | documents::Subject::Escalation { .. }
        | documents::Subject::Question { .. }
        | documents::Subject::Result { .. }
        | documents::Subject::Project { .. }
        | documents::Subject::Note { .. } => {}
    }
    match admit(core, limits, person, act, &subject) {
        Admission::Allowed | Admission::Proposed { .. } | Admission::Bounded { .. } => Ok(()),
        // The child emits the same refusal with its exact observed delegate
        // identities. Preserve that capacity observation at its owner.
        Admission::Refused(people::Refusal::Limit { which: people::LimitKind::LiveDelegates })
            if act == Act::Answer =>
        {
            Ok(())
        }
        Admission::Refused(why) => Err(why),
    }
}
