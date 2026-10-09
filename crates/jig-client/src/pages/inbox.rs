//! Inbox context and filters, without a second copy of an object's card.
//! Whole live documents replace their source; historical pages carry opaque
//! cursors. Needs sort oldest first and updates newest first. Reading results
//! asks the keyed table to mark the newest displayed position once per opening.
//! See client/domain/pages.md, 3, and contract.md, 3.2 and 6–7.
use crate::{
    address::{Address, ProjectSection},
    contract::{Count, Cursor, InboxEntry, InboxGroup, Needs, ObjectKey},
};
use skein_lib::List;
/// An inbox filter, shared by live rows and historical reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Filter {
    pub project: Option<u32>,
    pub kind: Option<InboxGroup>,
    pub goal: Option<u64>,
}
/// Context rows name cards in the one working set.
#[derive(Debug)]
pub struct Inbox {
    pub rows: List<InboxEntry>,
    pub filter: Filter,
    pub needs: Needs,
    pub updates: Count,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
    pub more: bool,
    pub refusal: Option<crate::contract::Refusal>,
    pub reading: Option<skein_lib::Token>,
    pub opening_updates: bool,
    pub marked_results: u64,
}
impl Inbox {
    /// Allocates the page's bounded context window.
    #[must_use]
    pub fn new(rows: u32) -> Self {
        Self {
            rows: List::with_capacity(rows),
            filter: Filter { project: None, kind: None, goal: None },
            needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
            updates: Count { value: 0, limited: false },
            before: None,
            after: None,
            more: false,
            refusal: None,
            reading: None,
            opening_updates: false,
            marked_results: 0,
        }
    }
    /// A filtered row belongs here only while its shared object still waits.
    #[must_use]
    pub fn includes(&self, entry: &InboxEntry) -> bool {
        (self.filter.project.is_none() || self.filter.project == Some(entry.project))
            && (self.filter.kind.is_none() || self.filter.kind == Some(entry.group))
            && (self.filter.goal.is_none() || self.filter.goal == entry.goal)
    }
    /// A bounded whole replacement, ordered without changing card state.
    pub(crate) fn replace(&mut self, entries: List<InboxEntry>) {
        self.rows.clear();
        for entry in entries.into_boxed() {
            if self.includes(&entry) && self.rows.push(entry).is_err() {
                break;
            }
        }
        for first in 0..self.rows.len() {
            for second in first.checked_add(1).expect("row index fits")..self.rows.len() {
                let a = self.rows.get(first).expect("row exists");
                let b = self.rows.get(second).expect("row exists");
                if earlier(b, a) {
                    let b = b.clone();
                    let a = core::mem::replace(self.rows.get_mut(first).expect("row exists"), b);
                    *self.rows.get_mut(second).expect("row exists") = a;
                }
            }
        }
    }
}
/// Whether an inbox kind holds work up.
#[must_use]
pub const fn needs(group: InboxGroup) -> bool {
    match group {
        InboxGroup::Questions | InboxGroup::Proposals | InboxGroup::Escalations | InboxGroup::PersonTasks => true,
        InboxGroup::Results | InboxGroup::Replies => false,
    }
}
fn earlier(a: &InboxEntry, b: &InboxEntry) -> bool {
    let a_needs = needs(a.group);
    let b_needs = needs(b.group);
    if a_needs != b_needs {
        return a_needs;
    }
    if a.at != b.at {
        return if a_needs { a.at < b.at } else { a.at > b.at };
    }
    a.object < b.object
}
/// The task address behind every card or inbox entry.
#[must_use]
pub const fn address(key: ObjectKey) -> Address {
    let task = match key {
        ObjectKey::Task(task)
        | ObjectKey::Escalation(task)
        | ObjectKey::PersonTask(task)
        | ObjectKey::Result(task)
        | ObjectKey::Question { task, .. } => task,
        ObjectKey::Proposal(proposal) => proposal.proposer,
        ObjectKey::Note { project, .. } => return Address::Project { project, section: ProjectSection::Notes },
    };
    Address::task(task)
}
