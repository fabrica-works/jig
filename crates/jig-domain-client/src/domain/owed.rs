//! Cards with nowhere else to show on the page (client/domain/view.md, 4.1).
use crate::{
    Domain,
    address::TaskSection,
    contract::ObjectKey,
    objects::{Card, Object},
    pages::Section,
};
use skein_lib::Id;
impl Domain {
    /// Whether the page still owes this object's card.
    #[must_use]
    pub fn owes(&self, id: Id<Object>) -> bool {
        let Some(object) = self.objects().get(id) else {
            return false;
        };
        if !object.shown {
            return false;
        }
        match object.card {
            Card::Deciding { .. } | Card::Leaving { .. } | Card::Unknown { .. } => {}
            Card::Open | Card::Refused { .. } => return false,
        }
        if let Some(task) = self.shown_task()
            && self.shows(Section::Task { task, section: TaskSection::Activity })
            && self.conversation().shows(object.key)
        {
            return false;
        }
        if object.sources.is_empty() {
            return true;
        }
        let Some(task) = self.shown_task() else {
            return false;
        };
        if !self.shows(Section::Task { task, section: TaskSection::Plan }) {
            return false;
        }
        let key = match object.key {
            ObjectKey::Proposal(key) => key,
            ObjectKey::Task(_)
            | ObjectKey::Escalation { .. }
            | ObjectKey::Question { .. }
            | ObjectKey::PersonTask(_)
            | ObjectKey::Result(_)
            | ObjectKey::Note { .. } => return false,
        };
        let Some(tree) = &self.task_page().plan.owner else {
            return false;
        };
        for row in &tree.rows {
            if row.proposals.as_slice().contains(&key) {
                return false;
            }
        }
        true
    }
}
