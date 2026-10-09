//! Concrete bounded windows sharing edges and one sliding rule.
//! Reading beyond a full window drops the opposite end and keeps the supplied
//! reverse cursor; live rows enter only at an end the window still reaches.
//! See client/domain/client.md, 7. Row owners supply cursors for re-reading drops.
use crate::{
    action::End,
    contract::{ActivityLine, ConversationLine, Cursor, HistoryLine, ObjectKey},
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
    pub rows: List<ConversationLine>,
    pub edges: Edges,
}
impl ConversationWindow {
    /// Allocates a fixed row capacity and starts at both list ends.
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self { rows: List::with_capacity(capacity), edges: Edges { before: Edge::End, after: Edge::End } }
    }
    /// Slides from one end, returning the number dropped from the other.
    pub fn slide(&mut self, incoming: List<ConversationLine>, incoming_edges: Edges, end: End) -> u32 {
        let capacity = self.rows.capacity();
        let empty = self.rows.is_empty();
        let slide = plan(self.rows.len(), incoming.len(), capacity, end);
        let mut rows = List::with_capacity(capacity);
        let old = core::mem::replace(&mut self.rows, List::with_capacity(0));
        match end {
            End::After => {
                move_conversationwindow(old, slide.old_from, slide.old_count, &mut rows);
                move_conversationwindow(incoming, slide.new_from, slide.new_count, &mut rows);
            }
            End::Before => {
                move_conversationwindow(incoming, slide.new_from, slide.new_count, &mut rows);
                move_conversationwindow(old, slide.old_from, slide.old_count, &mut rows);
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
fn move_conversationwindow(rows: List<ConversationLine>, start: u32, count: u32, out: &mut List<ConversationLine>) {
    let until = start.saturating_add(count);
    for (index, row) in rows.into_boxed().into_iter().enumerate() {
        let index = u32::try_from(index).expect("a bounded list index fits u32");
        if index >= start && index < until {
            out.push(row).expect("the slide plan fits the window");
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
    pub rows: List<HistoryLine>,
    pub edges: Edges,
}
impl HistoryWindow {
    /// Allocates a fixed row capacity and starts at both list ends.
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self { rows: List::with_capacity(capacity), edges: Edges { before: Edge::End, after: Edge::End } }
    }
    /// Slides from one end, returning the number dropped from the other.
    pub fn slide(&mut self, incoming: List<HistoryLine>, incoming_edges: Edges, end: End) -> u32 {
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
    pub fn live(&mut self, incoming: List<HistoryLine>, incoming_edges: Edges, end: End) -> bool {
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
fn move_historywindow(rows: List<HistoryLine>, start: u32, count: u32, out: &mut List<HistoryLine>) {
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
    total = total.checked_add(List::<HistoryLine>::worst_case(limits.rows)?)?;
    total.checked_mul(2)?.checked_add(u64::from(limits.rows).checked_mul(limits.document_bytes)?.checked_mul(6)?)
}
