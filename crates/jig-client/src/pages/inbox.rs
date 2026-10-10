//! Inbox context and filters, without a second copy of an object's card.
//! Whole live documents replace their source; historical pages carry opaque
//! cursors. Needs sort oldest first and updates newest first. Reading results
//! asks the keyed table to mark the newest displayed position once per opening.
//! See client/domain/pages.md, 3, and contract.md, 3.2 and 6–7.
use crate::{
    address::{Address, ProjectSection},
    contract::{
        Count, Cursor, InboxDocument, InboxEntry, InboxGroup, InboxPage, InboxReadGroup, InboxRow, Needs, ObjectKey,
        OwnerInbox, OwnerInboxPage, OwnerInboxSelection, Read,
    },
};
use alloc::boxed::Box;
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
    pub rows: List<InboxRow>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
    pub more: bool,
    pub reading: Option<skein_lib::Token>,
    /// Original accepted query, including its independent opaque continuation.
    pub query: Option<Read>,
    /// Whole original retained context; never replaced with current requester facts.
    pub selection: Option<List<OwnerInboxSelection>>,
}
impl InboxWindow {
    fn new(rows: u32) -> Self {
        Self {
            cut: false,
            rows: List::with_capacity(rows),
            before: None,
            after: None,
            more: false,
            reading: None,
            query: None,
            selection: None,
        }
    }
    // Read rows past the watch remain; duplicate snapshots never resurrect a removed live key.
    fn past(&mut self, live: &List<InboxRow>) {
        if self.rows.is_empty() {
            return;
        }
        let capacity = self.rows.capacity();
        let rows = core::mem::replace(&mut self.rows, List::with_capacity(capacity));
        for row in rows.into_boxed() {
            let mut watched = false;
            for current in live {
                if current.identity() == row.identity() {
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
        self.query = None;
        self.selection = None;
    }
}
/// Context rows merge the live watch and both read groups by object key.
/// A watched row wins; each source is bounded by one document's row limit.
#[derive(Debug)]
pub struct Inbox {
    pub rows: List<InboxRow>,
    pub filter: Filter,
    /// Whole latest source; exact owner aggregates never become four counters.
    pub document: Option<InboxDocument>,
    /// Live owner goal membership awaits a separately supplied source context.
    pub goal_context_pending: bool,
    pub needs: Needs,
    pub updates: Count,
    pub live: List<InboxRow>,
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
            rows: List::with_capacity(rows.checked_mul(4).expect("inbox source rows fit")),
            filter: Filter { project: None, kind: None, goal: None },
            document: None,
            goal_context_pending: false,
            needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
            updates: Count { value: 0, limited: false },
            live: List::with_capacity(rows.checked_mul(2).expect("two live groups fit")),
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
    pub fn includes(&self, entry: &InboxRow) -> bool {
        let membership = match entry {
            InboxRow::Entry(value) => self.filter.goal.is_none() || self.filter.goal == value.goal,
            // Absence of source ancestry is never asserted to be non-membership.
            // This incomplete live path is explicitly exposed to the view.
            InboxRow::Owner(_) => self.filter.goal.is_none(),
        };
        (self.filter.project.is_none() || self.filter.project == entry.project())
            && (self.filter.kind.is_none() || self.filter.kind == Some(entry.group()))
            && membership
    }
    /// Leaving a page discards every source and its pending cursor.
    pub(crate) fn clear(&mut self) {
        self.live.clear();
        self.document = None;
        self.goal_context_pending = false;
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
        self.live.clear();
        for entry in entries.into_boxed() {
            self.live.push(InboxRow::Entry(Box::new(entry))).expect("whole live source fits");
        }
        self.live_more = more;
        self.needs_window.past(&self.live);
        self.updates_window.past(&self.live);
        self.merge();
    }
    /// A read terminal moves only its group's rows and cursor.
    pub(crate) fn page(&mut self, group: InboxReadGroup, page: InboxPage) {
        let window = self.window_mut(group);
        window.selection = None;
        window.cut = page.cut;
        window.rows.clear();
        for row in page.rows.into_boxed() {
            window.rows.push(InboxRow::Entry(Box::new(row))).expect("whole historical source fits");
        }
        window.before = page.before;
        window.after = page.after;
        window.more = window.after.is_some();
        match group {
            InboxReadGroup::Needs => self.needs_window.past(&self.live),
            InboxReadGroup::Updates => self.updates_window.past(&self.live),
        }
        self.merge();
    }
    /// Whole exact owner delivery; both allocated groups remain in the envelope.
    pub(crate) fn owner(&mut self, value: Box<OwnerInbox>) {
        self.results_seen = value.results_seen;
        self.marked_results = self.marked_results.max(value.results_seen);
        self.live_more = value.next.is_some();
        self.live.clear();
        for entries in [&value.needs, &value.updates] {
            for entry in entries {
                self.live.push(InboxRow::Owner(Box::new(entry.clone()))).expect("both bounded live groups fit");
            }
        }
        self.document = Some(InboxDocument::Owner(value));
        self.needs_window.past(&self.live);
        self.updates_window.past(&self.live);
        self.merge();
    }
    /// The source selected these rows using the original query; no guessed goal
    /// or ancestry is required or used to re-filter a bounded historical page.
    pub(crate) fn owner_page(&mut self, group: InboxReadGroup, query: Read, page: OwnerInboxPage) {
        let window = self.window_mut(group);
        window.selection = page.selection;
        window.cut = page.cut;
        window.rows.clear();
        for row in page.rows.into_boxed() {
            window.rows.push(InboxRow::Owner(Box::new(row))).expect("whole owner page fits");
        }
        window.before = page.before;
        window.after = page.after;
        window.more = window.after.is_some();
        window.query = Some(query);
        match group {
            InboxReadGroup::Needs => self.needs_window.past(&self.live),
            InboxReadGroup::Updates => self.updates_window.past(&self.live),
        }
        self.merge();
    }
    fn merge(&mut self) {
        self.rows.clear();
        self.goal_context_pending = false;
        for entry in &self.live {
            match entry {
                InboxRow::Owner(_) => self.goal_context_pending |= self.filter.goal.is_some(),
                InboxRow::Entry(_) => {}
            }
            if self.includes(entry) {
                self.rows.push(entry.clone()).expect("bounded live groups fit");
            }
        }
        for window in [&self.needs_window, &self.updates_window] {
            for entry in &window.rows {
                let included = match entry {
                    InboxRow::Entry(_) => self.includes(entry),
                    InboxRow::Owner(_) => query_filter(window.query.as_ref()) == Some(self.filter),
                };
                if !included {
                    continue;
                }
                let mut known = false;
                for current in &self.rows {
                    if current.identity() == entry.identity() {
                        known = true;
                        break;
                    }
                }
                if !known {
                    self.rows.push(entry.clone()).expect("four bounded inbox sources fit");
                }
            }
        }
        // Adjacent swaps preserve supplied order at unknown-time boundaries.
        for _pass in 0..self.rows.len() {
            for second in 1..self.rows.len() {
                let first = second.checked_sub(1).expect("row has predecessor");
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
fn earlier(a: &InboxRow, b: &InboxRow) -> bool {
    let a_needs = needs(a.group());
    let b_needs = needs(b.group());
    if a_needs != b_needs {
        return a_needs;
    }
    match a.at() {
        Some(a_at) => match b.at() {
            Some(b_at) => {
                if a_at != b_at {
                    return if a_needs { a_at < b_at } else { a_at > b_at };
                }
                // Existing supplied rows retain their original tie ordering;
                // exact owner ties retain the owner's source order.
                match a {
                    InboxRow::Entry(a) => match b {
                        InboxRow::Entry(b) => a.object < b.object,
                        InboxRow::Owner(_) => false,
                    },
                    InboxRow::Owner(_) => false,
                }
            }
            None => false,
        },
        None => false,
    }
}
/// Original source filter, independent of owner row fields.
#[must_use]
pub const fn query_filter(query: Option<&Read>) -> Option<Filter> {
    match query {
        Some(Read::Inbox { project, kind, goal, .. }) => Some(Filter { project: *project, kind: *kind, goal: *goal }),
        Some(
            Read::Frame
            | Read::Task { .. }
            | Read::Proposal { .. }
            | Read::Escalation { .. }
            | Read::Question { .. }
            | Read::PersonTask { .. }
            | Read::Result { .. }
            | Read::Ended { .. }
            | Read::History { .. }
            | Read::Activity { .. }
            | Read::Conversations { .. }
            | Read::Goals { .. }
            | Read::Spend { .. }
            | Read::Project { .. }
            | Read::Notes { .. }
            | Read::Deployment,
        )
        | None => None,
    }
}
/// Whole Inbox storage, projections and merge scratch including independent windows.
#[must_use]
pub fn worst_case(limits: &crate::Limits) -> Option<u64> {
    let rows = limits.rows.checked_mul(4)?;
    List::<InboxRow>::worst_case(rows)?
        .checked_mul(3)?
        .checked_add(u64::from(rows).checked_mul(limits.document_bytes)?.checked_mul(3)?)?
        .checked_add(limits.document_bytes.checked_mul(3)?)
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
