//! Inbox context and filters, without a second copy of an object's card.
//! Whole live documents replace their source; historical pages carry opaque
//! cursors. Needs sort oldest first and updates newest first. Reading results
//! asks the keyed table to mark the newest displayed position once per opening.
//! See client/domain/pages.md, 3, and contract.md, 3.2 and 6–7.
use crate::{
    address::{Address, ProjectSection},
    contract::{Count, Cursor, InboxEntry, InboxGroup, InboxPage, InboxReadGroup, Needs, ObjectKey},
};
use skein_lib::List;
/// An inbox filter, shared by live rows and historical reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Filter {
    pub project: Option<u32>,
    pub kind: Option<InboxGroup>,
    pub goal: Option<u64>,
}
/// One read group's bounded window and independent continuation.
#[derive(Debug)]
pub struct InboxWindow {
    pub cut: bool,
    pub rows: List<InboxEntry>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
    pub more: bool,
    pub reading: Option<skein_lib::Token>,
}
impl InboxWindow {
    fn new(rows: u32) -> Self {
        Self { cut: false, rows: List::with_capacity(rows), before: None, after: None, more: false, reading: None }
    }
    // Read rows past the watch remain; duplicate snapshots never resurrect a removed live key.
    fn past(&mut self, live: &List<InboxEntry>) {
        if self.rows.is_empty() {
            return;
        }
        let capacity = self.rows.capacity();
        let rows = core::mem::replace(&mut self.rows, List::with_capacity(capacity));
        for row in rows.into_boxed() {
            let mut watched = false;
            for current in live {
                if current.object == row.object {
                    watched = true;
                    break;
                }
            }
            if !watched {
                self.rows.push(row).expect("retained read rows fit their original capacity");
            }
        }
    }
    fn clear(&mut self) {
        self.rows.clear();
        self.cut = false;
        self.before = None;
        self.after = None;
        self.more = false;
        self.reading = None;
    }
}
/// Context rows merge the live watch and both read groups by object key.
/// A watched row wins; each source is bounded by one document's row limit.
#[derive(Debug)]
pub struct Inbox {
    pub rows: List<InboxEntry>,
    pub filter: Filter,
    pub needs: Needs,
    pub updates: Count,
    pub live: List<InboxEntry>,
    /// The shared live watch has rows beyond its window; read cursors remain independent.
    pub live_more: bool,
    pub needs_window: InboxWindow,
    pub updates_window: InboxWindow,
    pub refusal: Option<crate::contract::Refusal>,
    pub opening_updates: bool,
    pub marked_results: u64,
    /// The committed read position; pending mark-seen requests do not advance it.
    pub results_seen: u64,
}
impl Inbox {
    /// Allocates the page's bounded context window.
    #[must_use]
    pub fn new(rows: u32) -> Self {
        Self {
            rows: List::with_capacity(rows.checked_mul(3).expect("inbox source rows fit")),
            filter: Filter { project: None, kind: None, goal: None },
            needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
            updates: Count { value: 0, limited: false },
            live: List::with_capacity(rows),
            live_more: false,
            needs_window: InboxWindow::new(rows),
            updates_window: InboxWindow::new(rows),
            refusal: None,
            opening_updates: false,
            marked_results: 0,
            results_seen: 0,
        }
    }
    /// A filtered row belongs here only while its shared object still waits.
    #[must_use]
    pub fn includes(&self, entry: &InboxEntry) -> bool {
        (self.filter.project.is_none() || self.filter.project == Some(entry.project))
            && (self.filter.kind.is_none() || self.filter.kind == Some(entry.group))
            && (self.filter.goal.is_none() || self.filter.goal == entry.goal)
    }
    /// Leaving a page discards every source and its pending cursor.
    pub(crate) fn clear(&mut self) {
        self.live.clear();
        self.live_more = false;
        self.needs_window.clear();
        self.updates_window.clear();
        self.rows.clear();
    }
    /// Changing filters retains the current live source and starts each query anew.
    pub(crate) fn filter(&mut self, filter: Filter) {
        self.filter = filter;
        self.needs_window.clear();
        self.updates_window.clear();
        self.merge();
    }
    /// Read window for exactly one parent group.
    #[must_use]
    pub const fn window(&self, group: InboxReadGroup) -> &InboxWindow {
        match group {
            InboxReadGroup::Needs => &self.needs_window,
            InboxReadGroup::Updates => &self.updates_window,
        }
    }
    pub(crate) const fn window_mut(&mut self, group: InboxReadGroup) -> &mut InboxWindow {
        match group {
            InboxReadGroup::Needs => &mut self.needs_window,
            InboxReadGroup::Updates => &mut self.updates_window,
        }
    }
    /// The whole live watch replaces its own source, preserving read windows.
    pub(crate) fn replace(&mut self, entries: List<InboxEntry>, more: bool) {
        self.live = entries;
        self.live_more = more;
        self.needs_window.past(&self.live);
        self.updates_window.past(&self.live);
        self.merge();
    }
    /// A read terminal moves only its group's rows and cursor.
    pub(crate) fn page(&mut self, group: InboxReadGroup, page: InboxPage) {
        let window = self.window_mut(group);
        window.cut = page.cut;
        window.rows = page.rows;
        window.before = page.before;
        window.after = page.after;
        window.more = window.after.is_some();
        match group {
            InboxReadGroup::Needs => self.needs_window.past(&self.live),
            InboxReadGroup::Updates => self.updates_window.past(&self.live),
        }
        self.merge();
    }
    fn merge(&mut self) {
        self.rows.clear();
        for entries in [&self.live, &self.needs_window.rows, &self.updates_window.rows] {
            for entry in entries {
                if !self.includes(entry) {
                    continue;
                }
                let mut known = false;
                for current in &self.rows {
                    if current.object == entry.object {
                        known = true;
                        break;
                    }
                }
                if !known {
                    self.rows.push(entry.clone()).expect("three bounded inbox sources fit");
                }
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
/// The known address behind an entry; a party maker supplies no task address.
#[must_use]
pub const fn address(key: ObjectKey) -> Option<Address> {
    let task = match key {
        ObjectKey::Task(task)
        | ObjectKey::Escalation(task)
        | ObjectKey::PersonTask(task)
        | ObjectKey::Result(task)
        | ObjectKey::Question { task, .. } => task,
        ObjectKey::Proposal(proposal) => match proposal.proposer {
            crate::contract::Proposer::Task(task) => task,
            crate::contract::Proposer::Party(_) => return None,
        },
        ObjectKey::Note { project, .. } => return Some(Address::Project { project, section: ProjectSection::Notes }),
    };
    Some(Address::task(task))
}
