//! Retained rows and scoped opaque cursors; contract.md, 6.
use crate::{Bounds, Engine, Invalid, Up, payload};
use jig_client::contract::{
    ActivityLine, ActivityPage, ConversationLine, ConversationPage, Cursor, GoalLine, GoalPage, HistoryLine,
    HistoryPage, InboxEntry, InboxPage, Note, NotePage, NoteScope, ObjectKey, ObjectPage, Read, ReadNone, ReadResult,
    ReadValue, Result, SpendPage, SpendPeriod, Text,
};
use skein_lib::{List, Time, Token};

/// An authoritative retained row collection, not a protocol page.
/// Every row is checked as a whole payload; the protocol chooses bounded pages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rows {
    Ended(Vec<Result>),
    History(Vec<HistoryLine>),
    Activity(Vec<ActivityLine>),
    Inbox(Vec<InboxEntry>),
    Conversations(Vec<ConversationLine>),
    Goals(Vec<GoalLine>),
    Spend(Vec<SpendPeriod>),
    Notes(Vec<Note>),
}
/// A party's page source. The read has no cursor and supplies its exact filters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub party: u64,
    pub read: Read,
    pub rows: Rows,
}
impl Source {
    pub(crate) fn validate(&self, bounds: &Bounds) -> core::result::Result<(), Invalid> {
        let (read, cursor) = plain(self.read.clone());
        if cursor.is_some() || !payload::fits(&read, bounds) || !self.rows.matches(&read) {
            return Err(Invalid::Identity);
        }
        self.rows.validate(bounds)
    }
}
impl Rows {
    pub(crate) fn validate(&self, bounds: &Bounds) -> core::result::Result<(), Invalid> {
        let fits = match self {
            Self::Ended(rows) => rows.iter().all(|row| payload::fits(row, bounds)),
            Self::History(rows) => rows.iter().all(|row| payload::fits(row, bounds)),
            Self::Activity(rows) => rows.iter().all(|row| payload::fits(row, bounds)),
            Self::Inbox(rows) => rows.iter().all(|row| payload::fits(row, bounds)),
            Self::Conversations(rows) => rows.iter().all(|row| payload::fits(row, bounds)),
            Self::Goals(rows) => rows.iter().all(|row| payload::fits(row, bounds)),
            Self::Spend(rows) => rows.iter().all(|row| payload::fits(row, bounds)),
            Self::Notes(rows) => rows.iter().all(|row| payload::fits(row, bounds)),
        };
        if fits { Ok(()) } else { Err(Invalid::Payload) }
    }
    fn matches(&self, read: &Read) -> bool {
        matches!(
            (self, read),
            (Self::Ended(_), Read::Ended { .. })
                | (Self::History(_), Read::History { .. })
                | (Self::Activity(_), Read::Activity { .. })
                | (Self::Inbox(_), Read::Inbox { .. } | Read::InboxGroup { .. })
                | (Self::Conversations(_), Read::Conversations { .. })
                | (Self::Goals(_), Read::Goals { .. })
                | (Self::Spend(_), Read::Spend { .. })
                | (Self::Notes(_), Read::Notes { .. })
        )
    }
    fn len(&self) -> usize {
        match self {
            Self::Ended(rows) => rows.len(),
            Self::History(rows) => rows.len(),
            Self::Activity(rows) => rows.len(),
            Self::Inbox(rows) => rows.len(),
            Self::Conversations(rows) => rows.len(),
            Self::Goals(rows) => rows.len(),
            Self::Spend(rows) => rows.len(),
            Self::Notes(rows) => rows.len(),
        }
    }
    fn page(&self, start: usize, end: usize, before: Option<Cursor>, after: Option<Cursor>) -> ReadValue {
        match self {
            Self::Ended(rows) => {
                ReadValue::Ended(Box::new(ObjectPage { rows: list(&rows[start..end]), before, after }))
            }
            Self::History(rows) => {
                ReadValue::History(Box::new(HistoryPage { rows: list(&rows[start..end]), before, after }))
            }
            Self::Activity(rows) => {
                ReadValue::Activity(Box::new(ActivityPage { rows: list(&rows[start..end]), before, after }))
            }
            Self::Inbox(rows) => ReadValue::Inbox(Box::new(InboxPage { rows: list(&rows[start..end]), before, after })),
            Self::Conversations(rows) => {
                ReadValue::Conversations(Box::new(ConversationPage { rows: list(&rows[start..end]), before, after }))
            }
            Self::Goals(rows) => ReadValue::Goals(Box::new(GoalPage { rows: list(&rows[start..end]), before, after })),
            Self::Spend(rows) => ReadValue::Spend(Box::new(SpendPage { rows: list(&rows[start..end]), before, after })),
            Self::Notes(rows) => ReadValue::Notes(Box::new(NotePage { rows: list(&rows[start..end]), before, after })),
        }
    }
}
#[derive(Debug)]
pub(crate) struct Position {
    party: u64,
    read: Read,
    index: usize,
}
fn cursor(number: u64, bounds: &Bounds) -> Cursor {
    Cursor {
        value: Text::new(number.to_be_bytes().to_vec().into_boxed_slice(), bounds.text).expect("cursor bound checked"),
    }
}
pub(crate) fn list<T: Clone>(rows: &[T]) -> List<T> {
    let mut list = List::with_capacity(u32::try_from(rows.len()).expect("bounded rows fit u32"));
    for row in rows {
        assert!(list.push(row.clone()).is_ok(), "reserved page capacity");
    }
    list
}
pub(crate) fn plain(mut read: Read) -> (Read, Option<Cursor>) {
    let position = match &mut read {
        Read::Ended { after, .. }
        | Read::History { after, .. }
        | Read::InboxGroup { after, .. }
        | Read::Inbox { after, .. }
        | Read::Conversations { after, .. }
        | Read::Goals { after, .. }
        | Read::Spend { after, .. }
        | Read::Notes { after, .. } => after.take(),
        Read::Activity { before, .. } => before.take(),
        Read::Frame
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Project { .. }
        | Read::Deployment => None,
    };
    (read, position)
}
impl Engine {
    /// Captures one read at the latest numbered commit. A token receives one terminal.
    pub fn read(&mut self, now: Time, sign_in: u64, token: Token, read: Read) -> core::result::Result<(), Invalid> {
        if !payload::fits(&read, &self.bounds) {
            return Err(Invalid::Payload);
        }
        if !self.read_tokens.insert(token) {
            return Err(Invalid::Identity);
        }
        let terminal = if let Some(party) = self.truth.sign_ins.get(&sign_in).copied() {
            self.read_value(party, read).map_or_else(ReadResult::None, ReadResult::Value)
        } else {
            ReadResult::None(ReadNone::SignedOut)
        };
        self.queue(now, self.next_commit - 1, Up::Read { token, terminal });
        Ok(())
    }
    pub(crate) fn read_value(&mut self, party: u64, read: Read) -> core::result::Result<ReadValue, ReadNone> {
        let (query, position) = plain(read);
        if !self.read_visible(party, &query) {
            return Err(ReadNone::Unknown);
        }
        let object = match &query {
            Read::Frame => {
                return self
                    .checked_read(self.truth.frames.get(&party).cloned().map(|v| ReadValue::Frame(Box::new(v))));
            }
            Read::Task { task } => Some(ObjectKey::Task(*task)),
            Read::Proposal { proposal } => Some(ObjectKey::Proposal(*proposal)),
            Read::Escalation { task } => Some(ObjectKey::Escalation(*task)),
            Read::Question { task, message } => Some(ObjectKey::Question { task: *task, message: *message }),
            Read::PersonTask { task } => Some(ObjectKey::PersonTask(*task)),
            Read::Result { task } => Some(ObjectKey::Result(*task)),
            Read::Project { project } => {
                return self
                    .checked_read(self.truth.projects.get(project).cloned().map(|v| ReadValue::Project(Box::new(v))));
            }
            Read::Deployment => {
                return self.checked_read(self.truth.deployment.clone().map(|v| ReadValue::Deployment(Box::new(v))));
            }
            Read::Ended { .. }
            | Read::History { .. }
            | Read::Activity { .. }
            | Read::InboxGroup { .. }
            | Read::Inbox { .. }
            | Read::Conversations { .. }
            | Read::Goals { .. }
            | Read::Spend { .. }
            | Read::Notes { .. } => None,
        };
        if let Some(key) = object {
            return self.checked_read(self.truth.objects.get(&key).cloned());
        }
        let rows = self
            .truth
            .sources
            .iter()
            .find(|source| source.party == party && source.read == query)
            .map_or_else(|| empty(&query), |source| source.rows.clone());
        let backward = matches!(query, Read::Activity { .. });
        let index = if let Some(cursor) = position {
            let bytes: [u8; 8] = cursor.value.bytes().try_into().map_err(|_| ReadNone::Gone)?;
            let saved = self.positions.get(&u64::from_be_bytes(bytes)).ok_or(ReadNone::Gone)?;
            if saved.party != party || saved.read != query {
                return Err(ReadNone::Gone);
            }
            saved.index.min(rows.len())
        } else if backward {
            rows.len()
        } else {
            0
        };
        let maximum = usize::try_from(self.bounds.rows).expect("u32 fits usize");
        let mut count = maximum.min(if backward { index } else { rows.len() - index });
        loop {
            let (start, end) = if backward { (index - count, index) } else { (index, index + count) };
            let before = (start != 0).then(|| cursor(self.next_position, &self.bounds));
            let after = (end != rows.len()).then(|| cursor(self.next_position + 1, &self.bounds));
            let page = rows.page(start, end, before.clone(), after.clone());
            if payload::fits(&page, &self.bounds) {
                if before.is_some() {
                    self.positions.insert(self.next_position, Position { party, read: query.clone(), index: start });
                }
                if after.is_some() {
                    self.positions.insert(self.next_position + 1, Position { party, read: query, index: end });
                }
                self.next_position += 2;
                return Ok(page);
            }
            if count == 0 {
                return Err(ReadNone::Busy);
            }
            count -= 1;
            if count == 0 && rows.len() != 0 {
                return Err(ReadNone::Busy);
            }
        }
    }
    fn checked_read(&self, value: Option<ReadValue>) -> core::result::Result<ReadValue, ReadNone> {
        let value = value.ok_or(ReadNone::Unknown)?;
        if payload::fits(&value, &self.bounds) { Ok(value) } else { Err(ReadNone::Busy) }
    }
    pub(crate) fn read_visible(&self, party: u64, read: &Read) -> bool {
        let key = match read {
            Read::Task { task }
            | Read::Ended { task, .. }
            | Read::History { task, .. }
            | Read::Activity { task, .. } => Some(ObjectKey::Task(*task)),
            Read::Proposal { proposal } => Some(ObjectKey::Proposal(*proposal)),
            Read::Escalation { task } => Some(ObjectKey::Escalation(*task)),
            Read::Question { task, message } => Some(ObjectKey::Question { task: *task, message: *message }),
            Read::PersonTask { task } => Some(ObjectKey::PersonTask(*task)),
            Read::Result { task } => Some(ObjectKey::Result(*task)),
            Read::Project { project } | Read::Goals { project, .. } | Read::Spend { project, .. } => {
                return self.truth.visible(party, *project);
            }
            Read::InboxGroup { project, .. } | Read::Inbox { project, .. } | Read::Conversations { project, .. } => {
                return project.is_none_or(|p| self.truth.visible(party, p));
            }
            Read::Notes { scope, .. } => {
                return match scope {
                    NoteScope::Deployment => self.truth.frames.get(&party).is_some_and(|f| f.deployment_role.is_some()),
                    NoteScope::Project(project) | NoteScope::Resource { project, .. } => {
                        self.truth.visible(party, *project)
                    }
                    NoteScope::Goal(task) => {
                        self.truth.project(ObjectKey::Task(*task)).is_some_and(|p| self.truth.visible(party, p))
                    }
                };
            }
            Read::Frame => return self.truth.frames.contains_key(&party),
            Read::Deployment => return self.truth.frames.get(&party).is_some_and(|f| f.deployment_role.is_some()),
        };
        key.and_then(|key| self.truth.project(key)).is_some_and(|project| self.truth.visible(party, project))
    }
}
fn empty(read: &Read) -> Rows {
    match read {
        Read::Ended { .. } => Rows::Ended(vec![]),
        Read::History { .. } => Rows::History(vec![]),
        Read::Activity { .. } => Rows::Activity(vec![]),
        Read::Inbox { .. } | Read::InboxGroup { .. } => Rows::Inbox(vec![]),
        Read::Conversations { .. } => Rows::Conversations(vec![]),
        Read::Goals { .. } => Rows::Goals(vec![]),
        Read::Spend { .. } => Rows::Spend(vec![]),
        Read::Notes { .. } => Rows::Notes(vec![]),
        Read::Frame
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Project { .. }
        | Read::Deployment => unreachable!("documents do not request a row source"),
    }
}
