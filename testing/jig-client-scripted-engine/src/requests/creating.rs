//! Creation rules are checked against the creating offer, independent of templates.
use jig_client_domain::contract::{self as c, Admission, Ask, Bounds, NewTask};
use skein_lib::List;
impl<A: crate::Application> crate::Engine<A> {
    pub(super) fn creating_admission(&self, party: u64, ask: &Ask) -> Option<Admission> {
        let (Ask::Create { project, .. } | Ask::Propose { project, action: c::ProposeAction::Batch { .. }, .. }) = ask
        else {
            return None;
        };
        Some(
            match self
                .truth
                .frames
                .get(&party)
                .and_then(|me| me.projects.iter().find(|p| p.project == *project))
                .and_then(|p| p.creating.as_ref())
            {
                Some(creating) => Admission::Bounded {
                    within: Bounds::Create(creating.within.clone()),
                    beyond: creating.beyond.clone(),
                },
                None => Admission::Refused(c::Refusal::NotOffered),
            },
        )
    }
    pub(super) fn batch_admission(offer: &Admission, tasks: &List<NewTask>, decision: skein_lib::Wall) -> Admission {
        if tasks.is_empty() {
            return Admission::Refused(c::Refusal::NotOffered);
        }
        let mut result = Admission::Allowed;
        for task in tasks {
            let deadline = deadline(task.deadline, task.authority.deadline, decision);
            let judged = c::admission::creation(offer, task.executor, task.authority.spend, deadline);
            match judged {
                Admission::Allowed => {}
                Admission::Proposed { .. } => result = judged,
                Admission::Refused(_) | Admission::Bounded { .. } => return judged,
            }
        }
        result
    }
    pub(super) fn accepting_admission(&self, party: u64, ask: &Ask, decision: skein_lib::Wall) -> Option<Admission> {
        let Ask::Decide { project, on: c::DecideOn::Proposal { proposal, decision: c::Decision::Accepted } } = ask
        else {
            return None;
        };
        let value = self.truth.objects.get(&c::ObjectKey::Proposal(*proposal))?;
        let tasks = if let c::ReadValue::Proposal(value) = value {
            let c::ProposalAction::Batch { tasks, .. } = &value.action else { return None };
            tasks
        } else if let c::ReadValue::OwnerProposal(value) = value {
            let c::OwnerProposalAction::Batch(tasks) = &value.action else { return None };
            tasks
        } else {
            return None;
        };
        let creating =
            self.truth.frames.get(&party)?.projects.iter().find(|row| row.project == *project)?.creating.as_ref()?;
        for task in tasks {
            if let c::NewDeadline::Span(_) = task.deadline {
                let asked = deadline(task.deadline, task.authority.authority.deadline, decision);
                if creating.within.latest.is_some_and(|latest| asked.is_some_and(|at| at > latest)) {
                    return Some(Admission::Refused(c::Refusal::Authority {
                        lacking: c::Lacking::Deadline { asked, latest: creating.within.latest },
                        proposable: Some(false),
                    }));
                }
                if creating.within.maximum_deadline.is_some_and(|latest| asked.is_some_and(|at| at > latest)) {
                    return Some(Admission::Refused(c::Refusal::NotOffered));
                }
            }
        }
        None
    }
}

fn deadline(
    selector: c::NewDeadline,
    named: Option<skein_lib::Wall>,
    decision: skein_lib::Wall,
) -> Option<skein_lib::Wall> {
    match selector {
        c::NewDeadline::Named => named,
        c::NewDeadline::Span(span) => {
            Some(skein_lib::Wall::from_nanos(decision.as_nanos().saturating_add(span.as_nanos())))
        }
    }
}
