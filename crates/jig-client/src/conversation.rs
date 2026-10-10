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
    contract::{
        Activity, ActivityLine, ActivityPage, Block, CallIdentity, Cursor, LiveCall, ObjectKey, Part,
        ProvisionalTurnName, Read, RunState, RunText, Text, TurnName,
    },
    objects::{Objects, Source},
    windows::{Edge, Edges, TurnWindow},
};
use skein_lib::{List, Map, Wall, Writer};
pub(crate) mod chips;
pub(crate) mod routing;
pub(crate) mod sending;
pub mod words;
pub use routing::ACTIVITY_WINDOW;
pub use words::{KeptWords, Sent, Words};

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
    pub attempt: u64,
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
/// One turn's text is provisional until its committed turn is read.
#[derive(Debug)]
pub struct LiveTurn {
    pub name: ProvisionalTurnName,
    pub began: Option<Wall>,
    pub text: Text,
    pub offset: u32,
    pub cut: bool,
    pub call: Option<LiveCall>,
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
    /// Opaque current-call folds are equality-only and independent of transcript numbers.
    pub current_opened: List<CallIdentity>,
    /// Whole run document, independent of retained provisional and committed turns.
    pub current_run: Option<RunState>,
    /// Earlier provisional turns survive deliveries of a later turn.
    pub live: List<LiveTurn>,
    /// The name and offset to request on the run watch.
    pub current: Option<ProvisionalTurnName>,
    /// The current watch cursor survives replacement by a committed turn.
    pub stream_offset: u32,
    /// The latest committed turn read, fencing stale stream deliveries.
    pub committed_turn: Option<TurnName>,
    /// An interrupted connection replaces its provisional text on recovery.
    pub replacing: bool,
    /// The task ended its run; unread committed activity is taken before cleanup.
    pub run_finished: bool,
    /// Pending snapshots and durable words retained until the agent read them.
    pub words: KeptWords,
    /// One live chips source for tasks named by calls in this window.
    pub chips_watch: Option<skein_lib::Id<crate::watches::Watch>>,
    /// Task membership shared by all displayed engine calls.
    pub chips: List<u64>,
    /// Membership is waiting for the old watch terminal or available watch room.
    pub chips_pending: bool,
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
            current_opened: List::with_capacity(rows),
            current_run: None,
            live: List::with_capacity(rows),
            current: None,
            stream_offset: 0,
            committed_turn: None,
            replacing: false,
            run_finished: false,
            words: KeptWords::new(rows),
            chips_watch: None,
            chips: List::with_capacity(rows),
            chips_pending: false,
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
        self.words.observe(&page);
        for row in &page.rows {
            match &row.event {
                Activity::Turn { name, .. } => self.committed(*name),
                Activity::Words { .. }
                | Activity::RunBegan { .. }
                | Activity::RunEnded { .. }
                | Activity::Result { .. }
                | Activity::News { .. }
                | Activity::Decision { .. } => {}
            }
        }
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
                if known > 0 && fits && !reached && self.fresh.rows.len() < self.fresh.rows.capacity() {
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
    /// Apply the owner's whole current run without inventing a provisional name.
    /// Owner committed progress never hides text before its activity is read.
    pub fn run_state(&mut self, value: RunState, text_limit: u32) -> bool {
        if value.task != self.task {
            return false;
        }
        if value.attempt.is_none() && (value.writing.is_some() || value.call.is_some()) {
            return false;
        }
        if let Some(held) = &self.current_run
            && let Some(previous) = held.attempt
            && let Some(incoming) = value.attempt
            && incoming < previous
        {
            return true;
        }
        if let Some(writing) = &value.writing {
            if value.attempt != Some(writing.attempt) {
                return false;
            }
            if !self.stream(writing.clone(), text_limit) {
                return false;
            }
        }
        if value.attempt.is_none() {
            self.end_run();
        }
        self.current_run = Some(value);
        self.prune_folds();
        true
    }
    /// Appends one valid offset delivery, retaining other provisional turn names.
    /// A gap needs an explicit cut; overlap and duplicate text are absorbed once.
    pub fn stream(&mut self, value: RunText, text_limit: u32) -> bool {
        if value.attempt != value.turn.attempt
            || value.through.checked_sub(value.from) != u32::try_from(value.text.bytes().len()).ok()
        {
            return false;
        }
        if let Some(run) = &self.current_run
            && let Some(attempt) = run.attempt
            && value.attempt < attempt
        {
            return true;
        }
        if let Some(committed) = self.committed_turn
            && !later(value.turn.turn(), committed)
        {
            return true;
        }
        if self.replacing {
            self.live.clear();
            self.current = None;
            self.stream_offset = 0;
            self.replacing = false;
        }
        let mut index = None;
        for at in 0..self.live.len() {
            let held = self.live.get(at).expect("live row exists").name;
            if held.turn() == value.turn.turn() {
                if value.turn.retry < held.retry {
                    return true;
                }
                index = Some(at);
                break;
            }
        }
        match index {
            Some(at) => {
                let live = self.live.get_mut(at).expect("live row exists");
                if value.turn.retry > live.name.retry {
                    *live = LiveTurn {
                        name: value.turn,
                        began: value.began,
                        text: tail(&[], value.text.bytes(), text_limit),
                        offset: value.through,
                        cut: value.cut || value.from > 0 || !value.text.fits(text_limit),
                        call: value.call,
                    };
                } else if value.cut && value.through >= live.offset {
                    live.text = tail(&[], value.text.bytes(), text_limit);
                    live.offset = value.through;
                    live.began = value.began;
                    live.call = value.call;
                    live.cut = true;
                } else {
                    if value.through <= live.offset {
                        if value.through == live.offset {
                            live.began = value.began;
                            live.call = value.call;
                        }
                        self.prune_folds();
                        return true;
                    }
                    if value.from > live.offset && !value.cut {
                        return false;
                    }
                    let skip = live.offset.saturating_sub(value.from);
                    let skip = usize::try_from(skip).expect("offset fits usize");
                    let Some(text) = value.text.bytes().get(skip..) else {
                        return false;
                    };
                    let old = if value.from > live.offset { &[] } else { live.text.bytes() };
                    let combined = old.len().saturating_add(text.len());
                    live.text = tail(old, text, text_limit);
                    live.offset = value.through;
                    live.cut =
                        live.cut || value.cut || combined > usize::try_from(text_limit).expect("text bound fits usize");
                    live.began = value.began;
                    live.call = value.call;
                }
            }
            None => {
                if self.live.len() == self.live.capacity() {
                    return false;
                }
                let cut = value.cut || value.from > 0 || !value.text.fits(text_limit);
                let text = tail(&[], value.text.bytes(), text_limit);
                let live = LiveTurn {
                    name: value.turn,
                    began: value.began,
                    text,
                    offset: value.through,
                    cut,
                    call: value.call,
                };
                self.live.push(live).expect("live turn room checked");
            }
        }
        match self.current {
            Some(current) => {
                if value.turn >= current {
                    self.current = Some(value.turn);
                    self.stream_offset = value.through;
                }
            }
            None => {
                self.current = Some(value.turn);
                self.stream_offset = value.through;
            }
        }
        self.prune_folds();
        true
    }
    /// Keeps text during a brief offline interval, replacing it on the next delivery.
    pub fn interrupted(&mut self) {
        self.replacing = true;
    }
    /// Drops provisional text that a completed run never committed.
    pub fn end_run(&mut self) {
        self.live.clear();
        self.current_run = None;
        self.current = None;
        self.stream_offset = 0;
        self.replacing = false;
        self.prune_folds();
    }
    /// The stream cursor follows the current name, never another turn's offset.
    #[must_use]
    pub const fn offset(&self) -> u32 {
        self.stream_offset
    }
    fn committed(&mut self, name: TurnName) {
        match self.committed_turn {
            Some(old) => {
                if later(name, old) {
                    self.committed_turn = Some(name);
                }
            }
            None => self.committed_turn = Some(name),
        }
        let mut kept = List::with_capacity(self.live.capacity());
        let live = core::mem::replace(&mut self.live, List::with_capacity(0));
        for turn in live.into_boxed() {
            if turn.name.turn() != name {
                kept.push(turn).expect("removal keeps bounded live turns");
            }
        }
        self.live = kept;
    }
    /// Connector-owned parts carried by calls, results or news, requested by the root.
    #[must_use]
    pub fn parts(&self, capacity: u32) -> List<Part> {
        let mut parts = List::with_capacity(capacity);
        for line in &self.lines.rows {
            match &line.event {
                Activity::Turn { turn, .. } => {
                    for message in &turn.messages {
                        for block in &message.blocks {
                            match block {
                                Block::Call(call) => {
                                    for part in &call.parts {
                                        add_part(&mut parts, part);
                                    }
                                }
                                Block::Text(_) | Block::Result { .. } | Block::Opaque { .. } => {}
                            }
                        }
                    }
                }
                Activity::News { part, .. } => {
                    if let Some(part) = part {
                        add_part(&mut parts, part);
                    }
                }
                Activity::Result { result } => {
                    if let Some(value) = &result.value {
                        match value {
                            crate::contract::ResultValue::Connector { part, .. } => add_part(&mut parts, part),
                            crate::contract::ResultValue::Report(_)
                            | crate::contract::ResultValue::Verdict { .. }
                            | crate::contract::ResultValue::Change { .. }
                            | crate::contract::ResultValue::Choice { .. }
                            | crate::contract::ResultValue::Approved { .. }
                            | crate::contract::ResultValue::Failure(_)
                            | crate::contract::ResultValue::Words(_) => {}
                        }
                    }
                }
                Activity::Words { .. }
                | Activity::RunBegan { .. }
                | Activity::RunEnded { .. }
                | Activity::Decision { .. } => {}
            }
        }
        for live in &self.live {
            if let Some(call) = &live.call {
                match call {
                    LiveCall::Decoded(call) => {
                        for part in &call.parts {
                            add_part(&mut parts, part);
                        }
                    }
                    LiveCall::Raw(_) => {}
                }
            }
        }
        parts
    }
    /// Default folds are implicit; only opened calls consume page fold room.
    #[must_use]
    pub fn fold(&self, name: CallName) -> Fold {
        if self.opened.contains_key(&name) { Fold::Open } else { Fold::Folded }
    }
    fn displayed_call(&self, name: CallName) -> bool {
        if has_call(&self.lines.rows, name) {
            return true;
        }
        for live in &self.live {
            if live.name.attempt == name.attempt
                && live.name.number == name.turn
                && let Some(LiveCall::Decoded(call)) = &live.call
                && call.number == name.call
            {
                return true;
            }
        }
        false
    }
    /// Opens or closes one displayed call, refusing unknown calls or full fold room.
    pub fn set_fold(&mut self, name: CallName, fold: Fold) -> bool {
        if !self.displayed_call(name) {
            return false;
        }
        match fold {
            Fold::Folded => {
                self.opened.remove(&name);
                true
            }
            Fold::Open => {
                if !self.opened.contains_key(&name)
                    && match self.opened.len().checked_add(self.current_opened.len()) {
                        Some(total) => total >= self.opened.capacity(),
                        None => true,
                    }
                {
                    return false;
                }
                self.opened.insert(name, ()).is_ok()
            }
        }
    }
    /// Presentation state for a raw call is keyed by its supplied opaque triple.
    #[must_use]
    pub fn current_fold(&self, identity: &CallIdentity) -> Fold {
        if self.current_opened.as_slice().contains(identity) { Fold::Open } else { Fold::Folded }
    }
    fn displayed_current_call(&self, identity: &CallIdentity) -> bool {
        if let Some(run) = &self.current_run {
            return match &run.call {
                Some(call) => &call.identity == identity,
                None => false,
            };
        }
        for live in &self.live {
            if let Some(LiveCall::Raw(call)) = &live.call
                && &call.identity == identity
            {
                return true;
            }
        }
        false
    }
    /// Opening an unknown identity or exceeding the bounded fold table is refused.
    pub fn set_current_fold(&mut self, identity: CallIdentity, fold: Fold) -> bool {
        if !self.displayed_current_call(&identity) {
            return false;
        }
        self.prune_folds();
        match fold {
            Fold::Open => {
                if self.current_opened.as_slice().contains(&identity) {
                    return true;
                }
                if match self.opened.len().checked_add(self.current_opened.len()) {
                    Some(total) => total >= self.current_opened.capacity(),
                    None => true,
                } {
                    return false;
                }
                self.current_opened.push(identity).is_ok()
            }
            Fold::Folded => {
                let mut kept = List::with_capacity(self.current_opened.capacity());
                for name in &self.current_opened {
                    if name != &identity {
                        kept.push(name.clone()).expect("folds remain bounded");
                    }
                }
                self.current_opened = kept;
                true
            }
        }
    }
    fn prune_folds(&mut self) {
        let mut gone = List::with_capacity(self.opened.capacity());
        for (name, ()) in &self.opened {
            if !self.displayed_call(*name) {
                gone.push(*name).expect("removed folds fit fold capacity");
            }
        }
        for name in gone.into_boxed() {
            self.opened.remove(&name);
        }
        let mut kept = List::with_capacity(self.current_opened.capacity());
        for identity in &self.current_opened {
            if self.displayed_current_call(identity) {
                kept.push(identity.clone()).expect("remaining current folds fit");
            }
        }
        self.current_opened = kept;
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
                                Block::Text(_) | Block::Result { .. } | Block::Opaque { .. } => {}
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
        for live in &self.live {
            if let Some(call) = &live.call {
                match call {
                    LiveCall::Decoded(call) => {
                        for key in &call.objects {
                            add_name(&mut names, *key);
                        }
                    }
                    LiveCall::Raw(_) => {}
                }
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
                                Block::Text(_) | Block::Result { .. } | Block::Opaque { .. } => {}
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
    words::worst_case(limits)?
        .checked_add(List::<LiveTurn>::worst_case(limits.rows)?)?
        .checked_mul(2)?
        .checked_add(
            u64::from(limits.rows)
                .checked_mul(u64::from(limits.text).checked_add(limits.document_bytes)?)?
                .checked_mul(2)?,
        )?
        .checked_add(u64::from(limits.text).checked_mul(2)?)?
        .checked_add(Map::<CallName, ()>::worst_case(limits.rows)?)?
        .checked_add(List::<CallName>::worst_case(limits.rows)?)?
        .checked_add(List::<CallIdentity>::worst_case(limits.rows)?.checked_mul(2)?)?
        .checked_add(limits.document_bytes.checked_mul(2)?)?
        .checked_add(u64::from(limits.rows).checked_mul(u64::from(limits.text))?.checked_mul(2)?)?
        .checked_add(List::<u64>::worst_case(limits.rows)?.checked_mul(2)?)?
        .checked_add(List::<ActivityLine>::worst_case(limits.rows)?.checked_mul(3)?)?
        .checked_add(u64::from(limits.rows).checked_mul(limits.document_bytes)?.checked_mul(4)?)
}

fn later(name: TurnName, other: TurnName) -> bool {
    name.attempt > other.attempt || (name.attempt == other.attempt && name.number > other.number)
}
fn tail(old: &[u8], incoming: &[u8], limit: u32) -> Text {
    let length = old.len().checked_add(incoming.len()).expect("validated text lengths fit usize");
    let kept = length.min(usize::try_from(limit).expect("text limit fits usize"));
    let dropped = length.saturating_sub(kept);
    let old_from = dropped.min(old.len());
    let new_from = dropped.saturating_sub(old.len());
    let mut writer = Writer::new(kept);
    writer.put(old.get(old_from..).expect("old tail starts within text")).expect("old tail fits allocation");
    writer.put(incoming.get(new_from..).expect("new tail starts within text")).expect("new tail fits allocation");
    Text::new(writer.finish(), limit).expect("tail respects its byte bound")
}
fn add_part(parts: &mut List<Part>, part: &Part) {
    if !parts.as_slice().contains(part) {
        match parts.push(part.clone()) {
            Ok(()) | Err(_) => {}
        }
    }
}
