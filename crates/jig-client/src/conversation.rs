//! A task's committed conversation, newest first, in one sliding window.
//! Keeps opaque read cursors and page-owned call folds; knows no transcript
//! encoding, application values or engine internals. `moved` reads a new tail
//! only while the window reaches it; `older` and `newer` follow its cursors.
//! `accept` consumes one requested page and may ask for another new-tail page.
//! The owner routes reads and keeps returned object names live in its working set.
//! See client/domain/client.md, 7 and 10, and objects.md, 10.
//!
//! | State | Event | Next | Emits |
//! |---|---|---|---|
//! | idle, at newest | activity moved | latest read | activity read |
//! | idle | scroll to a cursor | edge read | activity read |
//! | latest read | page before known tail | latest read or idle | continuation or names |
//! | edge read | page | idle | names |
//! | any read | failed | idle | nothing |
use crate::{
    action::End,
    contract::{Activity, ActivityLine, ActivityPage, Block, Cursor, ObjectKey, Read, TurnName},
    objects::{Objects, Source},
    windows::{Edge, Edges, TurnWindow},
};
use skein_lib::{List, Map};

/// The page's presentation of one call, independent of committed turn facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Fold {
    /// One summary line.
    Folded,
    /// Input and outcome details are visible.
    Open,
}
/// A call is named within a turn, so calls in different attempts stay distinct.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CallName {
    pub attempt: u32,
    pub turn: u32,
    pub call: u64,
}
/// The one activity read currently requested by this conversation.
#[derive(Debug)]
pub enum Reading {
    /// The newest pages, stopping at the already held committed position.
    Latest { known: u64 },
    /// Older activity at the end of the displayed window.
    Older,
    /// Activity nearer the present, after scrolling into history.
    Newer,
}
/// Committed lines and page state for one task conversation.
#[derive(Debug)]
pub struct Conversation {
    pub task: u64,
    pub lines: TurnWindow,
    /// The newest position announced by the task document.
    pub committed: u64,
    /// The newest position this conversation has read.
    pub loaded: u64,
    pub reading: Option<Reading>,
    /// Whole new-tail pages waiting to meet the held committed position.
    pub fresh: TurnWindow,
    /// The displayed calls whose details the person opened.
    pub opened: Map<CallName, ()>,
}
impl Conversation {
    /// Allocates the activity window and equally bounded staging and fold state.
    #[must_use]
    pub fn new(task: u64, rows: u32) -> Self {
        Self {
            task,
            lines: TurnWindow::new(rows),
            committed: 0,
            loaded: 0,
            reading: None,
            fresh: TurnWindow::new(rows),
            opened: Map::with_capacity(rows),
        }
    }
    /// Reads a newer committed tail once, leaving a historical window in place.
    pub fn moved(&mut self, committed: u64) -> Option<Read> {
        self.committed = self.committed.max(committed);
        if self.reading.is_some() || !self.lines.edges.reaches(End::Before) || self.committed <= self.loaded {
            return None;
        }
        self.fresh.rows.clear();
        self.fresh.edges = Edges { before: Edge::End, after: Edge::End };
        self.reading = Some(Reading::Latest { known: self.loaded });
        Some(Read::Activity { task: self.task, before: None })
    }
    /// Reads the preceding activity page from the window's opaque older cursor.
    pub fn older(&mut self) -> Option<Read> {
        if self.reading.is_some() {
            return None;
        }
        let cursor = match &self.lines.edges.after {
            Edge::End => return None,
            Edge::More(cursor) => cursor.clone(),
        };
        self.reading = Some(Reading::Older);
        Some(Read::Activity { task: self.task, before: Some(cursor) })
    }
    /// Re-reads rows dropped at the present end while scrolling into history.
    pub fn newer(&mut self) -> Option<Read> {
        if self.reading.is_some() {
            return None;
        }
        let cursor = match &self.lines.edges.before {
            Edge::End => return self.moved(self.committed),
            Edge::More(cursor) => cursor.clone(),
        };
        self.reading = Some(Reading::Newer);
        Some(Read::Activity { task: self.task, before: Some(cursor) })
    }
    /// Takes a requested page; repeated rows never duplicate a committed line.
    /// The owner has validated its row and payload bounds before this call.
    pub fn accept(&mut self, page: ActivityPage) -> Option<Read> {
        let reading = self.reading.take()?;
        let edges = page_edges(page.after, page.before);
        match reading {
            Reading::Latest { known } => {
                let available = self.fresh.rows.capacity().saturating_sub(self.fresh.rows.len());
                let mut rows = List::with_capacity(self.lines.rows.capacity());
                let mut reached = false;
                for row in page.rows.into_boxed() {
                    if row.position <= known {
                        reached = true;
                    } else if rows.push(row).is_err() {
                        break;
                    }
                }
                order(&mut rows);
                let fits = rows.len() <= available;
                if fits {
                    let dropped = self.fresh.slide(rows, edges, End::After);
                    assert!(dropped == 0, "whole refresh pages fit staging room");
                } else {
                    // Keep the previous page's cursor: a cursor after this page
                    // would skip its rows if only part of it fitted the window.
                    reached = false;
                }
                if fits && !reached && self.fresh.rows.len() < self.fresh.rows.capacity() {
                    match &self.fresh.edges.after {
                        Edge::More(cursor) => {
                            let cursor = cursor.clone();
                            self.reading = Some(Reading::Latest { known });
                            return Some(Read::Activity { task: self.task, before: Some(cursor) });
                        }
                        Edge::End => {}
                    }
                }
                if let Some(row) = self.fresh.rows.get(0) {
                    self.loaded = self.loaded.max(row.position);
                }
                let rows = core::mem::replace(&mut self.fresh.rows, List::with_capacity(self.lines.rows.capacity()));
                let edges = core::mem::replace(&mut self.fresh.edges, Edges { before: Edge::End, after: Edge::End });
                if !reached {
                    self.lines.rows.clear();
                    self.lines.edges = Edges { before: Edge::End, after: Edge::End };
                }
                let dropped = self.lines.slide(rows, edges, End::Before);
                if dropped > 0 {
                    assert!(self.lines.rows.len() == self.lines.rows.capacity(), "a dropping slide fills its window");
                }
            }
            Reading::Older | Reading::Newer => {
                let end = match reading {
                    Reading::Older => End::After,
                    Reading::Newer => End::Before,
                    Reading::Latest { .. } => unreachable!("latest handled above"),
                };
                let mut rows = List::with_capacity(self.lines.rows.capacity());
                for row in page.rows.into_boxed() {
                    let mut held = false;
                    for old in &self.lines.rows {
                        if old.position == row.position {
                            held = true;
                            break;
                        }
                    }
                    if !held && rows.push(row).is_err() {
                        break;
                    }
                }
                order(&mut rows);
                let dropped = self.lines.slide(rows, edges, end);
                if dropped > 0 {
                    assert!(self.lines.rows.len() == self.lines.rows.capacity(), "a dropping slide fills its window");
                }
            }
        }
        self.prune_folds();
        self.moved(self.committed)
    }
    /// Allows a failed or refused read to be retried without inventing a page.
    pub fn failed(&mut self) {
        self.reading = None;
        self.fresh.rows.clear();
    }
    /// Default folds are implicit; only opened calls consume page fold room.
    #[must_use]
    pub fn fold(&self, name: CallName) -> Fold {
        if self.opened.contains_key(&name) { Fold::Open } else { Fold::Folded }
    }
    /// Opens or closes one displayed call, refusing unknown calls or full fold room.
    pub fn set_fold(&mut self, name: CallName, fold: Fold) -> bool {
        if !has_call(&self.lines.rows, name) {
            return false;
        }
        match fold {
            Fold::Folded => {
                self.opened.remove(&name);
                true
            }
            Fold::Open => self.opened.insert(name, ()).is_ok(),
        }
    }
    fn prune_folds(&mut self) {
        let mut gone = List::with_capacity(self.opened.capacity());
        for (name, ()) in &self.opened {
            if !has_call(&self.lines.rows, *name) {
                gone.push(*name).expect("removed folds fit fold capacity");
            }
        }
        for name in gone.into_boxed() {
            self.opened.remove(&name);
        }
    }
    /// Names cards produced by calls and heard results, deduplicated by object key.
    #[must_use]
    pub fn names(&self, capacity: u32) -> List<ObjectKey> {
        let mut names = List::with_capacity(capacity);
        for line in &self.lines.rows {
            match &line.event {
                Activity::Turn { turn, .. } => {
                    for message in &turn.messages {
                        for block in &message.blocks {
                            match block {
                                Block::Call(call) => {
                                    for key in &call.objects {
                                        add_name(&mut names, *key);
                                    }
                                }
                                Block::Text(_) | Block::Result { .. } => {}
                            }
                        }
                    }
                }
                Activity::Result { result } => add_name(&mut names, ObjectKey::Result(result.task.task)),
                Activity::Decision { object, .. } => add_name(&mut names, *object),
                Activity::Words { .. }
                | Activity::RunBegan { .. }
                | Activity::RunEnded { .. }
                | Activity::News { .. } => {}
            }
        }
        names
    }
    /// Replaces this window's working-set source without duplicating shared cards.
    pub fn name_objects(&self, objects: &mut Objects, source: Source, capacity: u32) -> u32 {
        let names = self.names(capacity);
        objects.replace(source, names.as_slice())
    }
}
fn add_name(names: &mut List<ObjectKey>, key: ObjectKey) {
    if !names.as_slice().contains(&key) {
        match names.push(key) {
            Ok(()) | Err(_) => {}
        }
    }
}
fn page_edges(newer: Option<Cursor>, older: Option<Cursor>) -> Edges {
    Edges {
        before: match newer {
            Some(cursor) => Edge::More(cursor),
            None => Edge::End,
        },
        after: match older {
            Some(cursor) => Edge::More(cursor),
            None => Edge::End,
        },
    }
}
fn order(rows: &mut List<ActivityLine>) {
    for first in 0..rows.len() {
        for second in first.saturating_add(1)..rows.len() {
            let a = rows.get(first).expect("bounded row exists");
            let b = rows.get(second).expect("bounded row exists");
            if b.position > a.position {
                let b = b.clone();
                let a = core::mem::replace(rows.get_mut(first).expect("bounded row exists"), b);
                *rows.get_mut(second).expect("bounded row exists") = a;
            }
        }
    }
}
fn has_call(rows: &List<ActivityLine>, name: CallName) -> bool {
    for row in rows {
        match &row.event {
            Activity::Turn { name: TurnName { attempt, number }, turn } => {
                if *attempt == name.attempt && *number == name.turn {
                    for message in &turn.messages {
                        for block in &message.blocks {
                            match block {
                                Block::Call(call) => {
                                    if call.number == name.call {
                                        return true;
                                    }
                                }
                                Block::Text(_) | Block::Result { .. } => {}
                            }
                        }
                    }
                }
            }
            Activity::Words { .. }
            | Activity::RunBegan { .. }
            | Activity::RunEnded { .. }
            | Activity::Result { .. }
            | Activity::News { .. }
            | Activity::Decision { .. } => {}
        }
    }
    false
}
/// Conversation fold/staging allocations beyond the shared activity window.
#[must_use]
pub fn worst_case(limits: &crate::Limits) -> Option<u64> {
    Map::<CallName, ()>::worst_case(limits.rows)?
        .checked_add(List::<CallName>::worst_case(limits.rows)?)?
        .checked_add(List::<ActivityLine>::worst_case(limits.rows)?.checked_mul(3)?)?
        .checked_add(u64::from(limits.rows).checked_mul(limits.document_bytes)?.checked_mul(4)?)
}
