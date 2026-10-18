//! The scripted engine commits priority-only amendments together (contract.md, 3.4).
use jig_client_domain::contract::{self as c, Admission, Answer, ObjectKey, Operation, ReadValue, Refusal};
use skein_lib::{List, Wall};

pub(super) fn only(operations: &List<Operation>) -> bool {
    !operations.is_empty()
        && operations.iter().all(|op| {
            matches!(op,
        Operation::Amend { revision: None, change, .. }
            if change.addressed.is_none() && change.spec.is_none() && change.wake.is_none()
                && change.dependencies.is_none() && change.authority.is_none() && change.instructions.is_none()
                && matches!(change.tracked, Some(c::Tracking::Tracked { .. })))
        })
}
impl<A: crate::Application> crate::Engine<A> {
    pub(super) fn priorities(
        &mut self,
        party: u64,
        project: u32,
        operations: &List<Operation>,
        wall: Wall,
    ) -> Result<(Answer, Vec<crate::Change>), crate::Invalid> {
        let offers = self
            .truth
            .owner_projects
            .get(&project)
            .map(|p| &p.acts)
            .or_else(|| self.truth.projects.get(&project).map(|p| &p.acts));
        let admission = offers.and_then(|rows| rows.iter().find(|offer| offer.act == c::ActKind::Reorder));
        let refusal = if self.truth.visible(party, project) {
            match admission.map(|offer| &offer.admission) {
                Some(Admission::Allowed) => None,
                Some(Admission::Refused(why)) => Some(why.clone()),
                Some(Admission::Proposed { .. } | Admission::Bounded { .. }) | None => Some(Refusal::NotOffered),
            }
        } else {
            Some(Refusal::Unknown)
        };
        if let Some(refusal) = refusal {
            return Ok((Answer::Refused { refusal, at: None }, vec![]));
        }
        let mut next = self.truth.clone();
        let mut changes = Vec::new();
        for (at, op) in operations.iter().enumerate() {
            let at = u32::try_from(at).expect("operations bounded");
            let Operation::Amend { project: named, task, change, .. } = op else { unreachable!() };
            let Some(ReadValue::Task(before)) = next.objects.get(&ObjectKey::Task(*task)) else {
                return Ok((Answer::Refused { refusal: Refusal::Unknown, at: Some(at) }, vec![]));
            };
            if *named != project || before.project != project || before.priority.is_none() {
                return Ok((Answer::Refused { refusal: Refusal::NotOffered, at: Some(at) }, vec![]));
            }
            let mut task = before.clone();
            let Some(c::Tracking::Tracked { priority }) = change.tracked else { unreachable!() };
            task.priority = Some(priority);
            task.changed = wall;
            let change =
                crate::Change::Object { project, key: ObjectKey::Task(task.chip.task), value: ReadValue::Task(task) };
            change.validate(&self.bounds)?;
            next.apply(&change)?;
            changes.push(change);
        }
        self.truth = next;
        Ok((
            Answer::Done(c::Outcome::Owner(c::OwnerOutcome::Compound {
                project,
                operations: operations.len(),
                first_message: self.next_commit,
            })),
            changes,
        ))
    }
}
