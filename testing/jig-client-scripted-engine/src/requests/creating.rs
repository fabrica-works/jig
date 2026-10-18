//! Creation rules are checked against the creating offer, independent of templates.
use jig_client_domain::contract::{self as c, Admission, Ask, Bounds, GoalExecutor, NewTask};
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
    pub(super) fn batch_admission(offer: &Admission, tasks: &List<NewTask>) -> Admission {
        if tasks.is_empty() {
            return Admission::Refused(c::Refusal::NotOffered);
        }
        let mut result = Admission::Allowed;
        for task in tasks {
            let executor = match task.executor {
                c::OwnerExecutor::Agent { charter } => GoalExecutor::Charter(charter),
                c::OwnerExecutor::Procedure { connector, code } => GoalExecutor::Procedure { connector, code },
                c::OwnerExecutor::Person(_) => return Admission::Refused(c::Refusal::NotOffered),
            };
            let judged = c::admission::goal(offer, executor, task.authority.spend, task.authority.deadline);
            match judged {
                Admission::Allowed => {}
                Admission::Proposed { .. } => result = judged,
                Admission::Refused(_) | Admission::Bounded { .. } => return judged,
            }
        }
        result
    }
}
