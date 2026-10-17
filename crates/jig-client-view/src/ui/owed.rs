//! Cards with nowhere else to show on the page (client/domain/view.md, 4.1).
use crate::{Ui, address::TaskSection, objects::Object, page::Section};
use jig_client::contract::ObjectKey;
use skein_lib::Id;
impl Ui {
    /// Whether the page still owes this object's card.
    #[must_use]
    pub fn owes(&self, id: Id<Object>, lent: &jig_client::Client) -> bool {
        let Some(object) = self.objects().get(id) else {
            return false;
        };
        if !object.shown
            || (self.page == crate::page::Page::Elsewhere
                && object.sources.contains(&crate::objects::Source::Window { number: crate::application::WINDOW }))
        {
            return false;
        }
        if let Some(task) = self.shown_task()
            && self.shows(Section::Task { task, section: TaskSection::Activity }, lent)
            && self.borrow(lent).conversation().shows(object.key)
        {
            return false;
        }
        if object.sources.is_empty() {
            return true;
        }
        let Some(task) = self.shown_task() else {
            return false;
        };
        let plan = Section::Task { task, section: TaskSection::Plan };
        if !self.sections().contains(&plan) {
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
        if !self.shows(plan, lent) {
            return true;
        }
        let Some(tree) = self.borrow(lent).task_page().plan.owner else {
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
