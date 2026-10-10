//! Concrete bounded windows sharing edges and one sliding rule.
//! Reading beyond a full window drops the opposite end and keeps the supplied
//! reverse cursor; live rows enter only at an end the window still reaches.
//! See client/domain/client.md, 7. Row owners supply cursors for re-reading drops.
use crate::{
    action::End,
    contract::{ActivityLine, ConversationLine, Cursor, HistoryRow, ObjectKey},
};
use skein_lib::List;

/// An end of the list or an opaque continuation for more rows.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Edge {
    /// The window reaches this end of the list.
    End,
    /// More rows can be read from this cursor.
    More(Cursor),
}
/// What can be read at both ends of a window.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Edges {
    pub before: Edge,
    pub after: Edge,
}
impl Edges {
    /// Whether the window reaches the end where live rows are being added.
    #[must_use]
    pub fn reaches(&self, end: End) -> bool {
        let edge = match end {
            End::Before => &self.before,
            End::After => &self.after,
        };
        match edge {
            Edge::End => true,
            Edge::More(_) => false,
        }
    }
}
/// The ranges retained by a slide, shared by all concrete row kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Slide {
    old_from: u32,
    old_count: u32,
    new_from: u32,
    new_count: u32,
    dropped: u32,
}
fn plan(old: u32, new: u32, capacity: u32, end: End) -> Slide {
    let dropped = old.saturating_add(new).saturating_sub(capacity);
    match end {
        End::After => {
            let old_from = dropped.min(old);
            let old_count = old.saturating_sub(old_from);
            let new_from = dropped.saturating_sub(old);
            let new_count = new.saturating_sub(new_from);
            Slide { old_from, old_count, new_from, new_count, dropped }
        }
        End::Before => {
            let new_count = new.min(capacity);
            let old_count = old.min(capacity.saturating_sub(new_count));
            Slide { old_from: 0, old_count, new_from: 0, new_count, dropped }
        }
    }
}
fn edges(current: Edges, incoming: Edges, dropped: u32, empty: bool, end: End) -> Edges {
    match end {
        End::After => {
            Edges { before: if dropped > 0 || empty { incoming.before } else { current.before }, after: incoming.after }
        }
        End::Before => {
            Edges { before: incoming.before, after: if dropped > 0 || empty { incoming.after } else { current.after } }
        }
    }
}
/// Object names on a page.
#[derive(Debug)]
pub struct ObjectWindow {
    pub rows: List<ObjectKey>,
    pub edges: Edges,
}
impl ObjectWindow {
    /// Allocates a fixed row capacity and starts at both list ends.
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self { rows: List::with_capacity(capacity), edges: Edges { before: Edge::End, after: Edge::End } }
    }
    /// Slides from one end, returning the number dropped from the other.
    pub fn slide(&mut self, incoming: List<ObjectKey>, incoming_edges: Edges, end: End) -> u32 {
        let capacity = self.rows.capacity();
        let empty = self.rows.is_empty();
        let slide = plan(self.rows.len(), incoming.len(), capacity, end);
        let mut rows = List::with_capacity(capacity);
        let old = core::mem::replace(&mut self.rows, List::with_capacity(0));
        match end {
            End::After => {
                move_objectwindow(old, slide.old_from, slide.old_count, &mut rows);
                move_objectwindow(incoming, slide.new_from, slide.new_count, &mut rows);
            }
            End::Before => {
                move_objectwindow(incoming, slide.new_from, slide.new_count, &mut rows);
                move_objectwindow(old, slide.old_from, slide.old_count, &mut rows);
            }
        }
        self.rows = rows;
        let current = core::mem::replace(&mut self.edges, Edges { before: Edge::End, after: Edge::End });
        self.edges = edges(current, incoming_edges, slide.dropped, empty, end);
        slide.dropped
    }
    /// Adds live rows only if this window reaches their end.
    pub fn live(&mut self, incoming: List<ObjectKey>, incoming_edges: Edges, end: End) -> bool {
        if !self.edges.reaches(end) {
            return false;
        }
        let dropped = self.slide(incoming, incoming_edges, end);
        if dropped > 0 {
            assert!(self.rows.len() == self.rows.capacity(), "a dropping slide fills its window");
        }
        true
    }
}
fn move_objectwindow(rows: List<ObjectKey>, start: u32, count: u32, out: &mut List<ObjectKey>) {
    let until = start.saturating_add(count);
    for (index, row) in rows.into_boxed().into_iter().enumerate() {
        let index = u32::try_from(index).expect("a bounded list index fits u32");
        if index >= start && index < until {
            out.push(row).expect("the slide plan fits the window");
        }
    }
}

/// Conversations in the party list.
#[derive(Debug)]
pub struct ConversationWindow {
    /// A retained read reported page loss; independent from its row edges.
    pub cut: bool,
    pub rows: List<ConversationLine>,
    pub edges: Edges,
    /// Original opaque owner continuation supplied by an admitted live snapshot.
    pub owner_after: Option<Cursor>,
}
impl ConversationWindow {
    /// Allocates a fixed row capacity and starts at both list ends.
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self {
            cut: false,
            rows: List::with_capacity(capacity),
            edges: Edges { before: Edge::End, after: Edge::End },
            owner_after: None,
        }
    }
    /// Merges by task key before sliding; a retained row wins over a read overlap.
    pub fn slide(&mut self, incoming: List<ConversationLine>, incoming_edges: Edges, end: End) -> u32 {
        let key_capacity = self.rows.len().checked_add(incoming.len()).expect("admitted row counts fit");
        let mut keys = List::with_capacity(key_capacity);
        let old_count = conversation_count(&self.rows);
        match end {
            End::After => conversation_after_keys(&self.rows, &incoming, &mut keys),
            End::Before => {
                conversation_keys(&self.rows, &mut keys);
                conversation_keys(&incoming, &mut keys);
            }
        }
        self.merge(incoming, incoming_edges, end, keys, old_count)
    }
    /// The supplied watch continuation starts a source prefix before retained past.
    pub(crate) fn replay(&mut self, incoming: List<ConversationLine>, incoming_edges: Edges) -> u32 {
        let key_capacity = self.rows.len().checked_add(incoming.len()).expect("admitted row counts fit");
        let mut keys = List::with_capacity(key_capacity);
        for row in &self.rows {
            if crate::pages::conversations::live(row.chip.phase) {
                conversation_key(&mut keys, row.chip.task);
            }
        }
        conversation_keys(&incoming, &mut keys);
        conversation_keys(&self.rows, &mut keys);
        let old_count = conversation_count(&self.rows);
        self.merge(incoming, incoming_edges, End::After, keys, old_count)
    }
    fn merge(
        &mut self,
        incoming: List<ConversationLine>,
        incoming_edges: Edges,
        end: End,
        keys: List<u64>,
        old_count: u32,
    ) -> u32 {
        let capacity = self.rows.capacity();
        let empty = self.rows.is_empty();
        let new_count = keys.len().checked_sub(old_count).expect("keys only append");
        let slide = plan(old_count, new_count, capacity, end);
        let mut rows = List::with_capacity(capacity);
        let old = core::mem::replace(&mut self.rows, List::with_capacity(0));
        let new_from = old_count.checked_add(slide.new_from).expect("bounded key range fits");
        match end {
            End::After => {
                let count = keys.len().min(capacity);
                let start = keys.len().saturating_sub(count);
                move_conversationwindow(old, &keys, start, count, &mut rows);
                move_conversationwindow(incoming, &keys, start, count, &mut rows);
                order_conversationwindow(&mut rows, &keys, start);
            }
            End::Before => {
                move_conversationwindow(incoming, &keys, new_from, slide.new_count, &mut rows);
                move_conversationwindow(old, &keys, slide.old_from, slide.old_count, &mut rows);
            }
        }
        self.rows = rows;
        let current = core::mem::replace(&mut self.edges, Edges { before: Edge::End, after: Edge::End });
        self.edges = edges(current, incoming_edges, slide.dropped, empty, end);
        slide.dropped
    }
    /// Adds live rows only if this window reaches their end.
    pub fn live(&mut self, incoming: List<ConversationLine>, incoming_edges: Edges, end: End) -> bool {
        if !self.edges.reaches(end) {
            return false;
        }
        let dropped = self.slide(incoming, incoming_edges, end);
        if dropped > 0 {
            assert!(self.rows.len() == self.rows.capacity(), "a dropping slide fills its window");
        }
        true
    }
}
fn conversation_keys(rows: &List<ConversationLine>, keys: &mut List<u64>) {
    for row in rows {
        if !keys.as_slice().contains(&row.chip.task) {
            keys.push(row.chip.task).expect("one key per admitted row fits");
        }
    }
}
fn conversation_count(rows: &List<ConversationLine>) -> u32 {
    let mut count = 0_u32;
    for index in 0..rows.len() {
        let key = rows.get(index).expect("row exists").chip.task;
        let mut present = false;
        for earlier in 0..index {
            if rows.get(earlier).expect("row exists").chip.task == key {
                present = true;
                break;
            }
        }
        if !present {
            count = count.checked_add(1).expect("a distinct key count fits the bounded rows");
        }
    }
    count
}
fn conversation_named(rows: &List<ConversationLine>, key: u64) -> bool {
    for row in rows {
        if row.chip.task == key {
            return true;
        }
    }
    false
}
fn conversation_key(keys: &mut List<u64>, key: u64) {
    if !keys.as_slice().contains(&key) {
        keys.push(key).expect("the distinct union fits the admitted key capacity");
    }
}
/// A replay may introduce source rows before a retained overlap. The supplied
/// page's keys keep their order; current rows still supply the whole facts.
/// Retained gaps remain beside their old preceding source anchor, without an
/// inferred clock or numeric ordering. The bounded After slide selects its tail.
fn conversation_after_keys(old: &List<ConversationLine>, incoming: &List<ConversationLine>, keys: &mut List<u64>) {
    for row in old {
        if conversation_named(incoming, row.chip.task) {
            break;
        }
        conversation_key(keys, row.chip.task);
    }
    for row in incoming {
        conversation_key(keys, row.chip.task);
        let mut following = false;
        for held in old {
            if held.chip.task == row.chip.task {
                following = true;
            } else if following {
                if conversation_named(incoming, held.chip.task) {
                    break;
                }
                conversation_key(keys, held.chip.task);
            }
        }
    }
    conversation_keys(old, keys);
}
fn order_conversationwindow(rows: &mut List<ConversationLine>, keys: &List<u64>, start: u32) {
    for first in 0..rows.len() {
        let key = keys.get(start.checked_add(first).expect("selected key range fits")).expect("key exists");
        for second in first..rows.len() {
            if rows.get(second).expect("row exists").chip.task == *key {
                if second != first {
                    let row = rows.get(second).expect("row exists").clone();
                    let row = core::mem::replace(rows.get_mut(first).expect("row exists"), row);
                    *rows.get_mut(second).expect("row exists") = row;
                }
                break;
            }
        }
    }
}
fn move_conversationwindow(
    rows: List<ConversationLine>,
    keys: &List<u64>,
    start: u32,
    count: u32,
    out: &mut List<ConversationLine>,
) {
    let until = start.saturating_add(count);
    for row in rows.into_boxed() {
        let mut selected = false;
        for index in start..until {
            if keys.get(index) == Some(&row.chip.task) {
                selected = true;
                break;
            }
        }
        let mut present = false;
        for held in &*out {
            if held.chip.task == row.chip.task {
                present = true;
                break;
            }
        }
        if selected && !present {
            out.push(row).expect("the distinct key slide fits the window");
        }
    }
}

/// Committed activity including words, run boundaries and turns.
#[derive(Debug)]
pub struct TurnWindow {
    pub rows: List<ActivityLine>,
    pub edges: Edges,
}
impl TurnWindow {
    /// Allocates a fixed row capacity and starts at both list ends.
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self { rows: List::with_capacity(capacity), edges: Edges { before: Edge::End, after: Edge::End } }
    }
    /// Slides from one end, returning the number dropped from the other.
    pub fn slide(&mut self, incoming: List<ActivityLine>, incoming_edges: Edges, end: End) -> u32 {
        let capacity = self.rows.capacity();
        let empty = self.rows.is_empty();
        let slide = plan(self.rows.len(), incoming.len(), capacity, end);
        let mut rows = List::with_capacity(capacity);
        let old = core::mem::replace(&mut self.rows, List::with_capacity(0));
        match end {
            End::After => {
                move_turnwindow(old, slide.old_from, slide.old_count, &mut rows);
                move_turnwindow(incoming, slide.new_from, slide.new_count, &mut rows);
            }
            End::Before => {
                move_turnwindow(incoming, slide.new_from, slide.new_count, &mut rows);
                move_turnwindow(old, slide.old_from, slide.old_count, &mut rows);
            }
        }
        self.rows = rows;
        let current = core::mem::replace(&mut self.edges, Edges { before: Edge::End, after: Edge::End });
        self.edges = edges(current, incoming_edges, slide.dropped, empty, end);
        slide.dropped
    }
    /// Adds live rows only if this window reaches their end.
    pub fn live(&mut self, incoming: List<ActivityLine>, incoming_edges: Edges, end: End) -> bool {
        if !self.edges.reaches(end) {
            return false;
        }
        let dropped = self.slide(incoming, incoming_edges, end);
        if dropped > 0 {
            assert!(self.rows.len() == self.rows.capacity(), "a dropping slide fills its window");
        }
        true
    }
}
fn move_turnwindow(rows: List<ActivityLine>, start: u32, count: u32, out: &mut List<ActivityLine>) {
    let until = start.saturating_add(count);
    for (index, row) in rows.into_boxed().into_iter().enumerate() {
        let index = u32::try_from(index).expect("a bounded list index fits u32");
        if index >= start && index < until {
            out.push(row).expect("the slide plan fits the window");
        }
    }
}

/// Task or subtree history lines.
#[derive(Debug)]
pub struct HistoryWindow {
    /// A retained read reported page loss; independent from its row edges.
    pub cut: bool,
    pub rows: List<HistoryRow>,
    pub edges: Edges,
}
impl HistoryWindow {
    /// Allocates a fixed row capacity and starts at both list ends.
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self { cut: false, rows: List::with_capacity(capacity), edges: Edges { before: Edge::End, after: Edge::End } }
    }
    /// Slides from one end, returning the number dropped from the other.
    pub fn slide(&mut self, incoming: List<HistoryRow>, incoming_edges: Edges, end: End) -> u32 {
        let capacity = self.rows.capacity();
        let empty = self.rows.is_empty();
        let slide = plan(self.rows.len(), incoming.len(), capacity, end);
        let mut rows = List::with_capacity(capacity);
        let old = core::mem::replace(&mut self.rows, List::with_capacity(0));
        match end {
            End::After => {
                move_historywindow(old, slide.old_from, slide.old_count, &mut rows);
                move_historywindow(incoming, slide.new_from, slide.new_count, &mut rows);
            }
            End::Before => {
                move_historywindow(incoming, slide.new_from, slide.new_count, &mut rows);
                move_historywindow(old, slide.old_from, slide.old_count, &mut rows);
            }
        }
        self.rows = rows;
        let current = core::mem::replace(&mut self.edges, Edges { before: Edge::End, after: Edge::End });
        self.edges = edges(current, incoming_edges, slide.dropped, empty, end);
        slide.dropped
    }
    /// Adds live rows only if this window reaches their end.
    pub fn live(&mut self, incoming: List<HistoryRow>, incoming_edges: Edges, end: End) -> bool {
        if !self.edges.reaches(end) {
            return false;
        }
        let dropped = self.slide(incoming, incoming_edges, end);
        if dropped > 0 {
            assert!(self.rows.len() == self.rows.capacity(), "a dropping slide fills its window");
        }
        true
    }
}
fn move_historywindow(rows: List<HistoryRow>, start: u32, count: u32, out: &mut List<HistoryRow>) {
    let until = start.saturating_add(count);
    for (index, row) in rows.into_boxed().into_iter().enumerate() {
        let index = u32::try_from(index).expect("a bounded list index fits u32");
        if index >= start && index < until {
            out.push(row).expect("the slide plan fits the window");
        }
    }
}

/// Concrete row containers and the maximum row payloads while sliding.
#[must_use]
pub fn worst_case(limits: &crate::Limits) -> Option<u64> {
    let mut total = List::<ObjectKey>::worst_case(limits.rows)?;
    total = total.checked_add(List::<ConversationLine>::worst_case(limits.rows)?)?;
    total = total.checked_add(List::<ActivityLine>::worst_case(limits.rows)?)?;
    total = total.checked_add(List::<HistoryRow>::worst_case(limits.rows)?)?;
    total
        .checked_mul(2)?
        .checked_add(List::<u64>::worst_case(limits.rows.checked_mul(2)?)?)?
        .checked_add(u64::from(limits.text))?
        .checked_add(u64::from(limits.rows).checked_mul(limits.document_bytes)?.checked_mul(6)?)
}
