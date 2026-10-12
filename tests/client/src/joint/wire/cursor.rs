//! Opaque continuations cross reload as bytes, carrying the whole owner cursor.
//! The protocol layer interprets the bytes; the client only echoes them
//! (client/domain/contract.md, 6). No live cursor table owns continuation state.
use jig_domain_client::contract::{Cursor, Text};
use jig_domain_core::{CallKey, CoreKey, Key, ProjectionKey, documents, reads};
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::Wall;

/// A complete continuation failed its configured byte bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TooLarge;

/// # Errors
/// Refuses malformed or oversized opaque cursor bytes and a cursor belonging to another exact query.
pub fn encode(value: &reads::Cursor, limit: u32) -> Result<Cursor, TooLarge> {
    let mut writer = Writer { bytes: Vec::new(), limit: usize::try_from(limit).map_err(|_| TooLarge)? };
    writer.byte(1).ok_or(TooLarge)?;
    put_cursor(&mut writer, value).ok_or(TooLarge)?;
    let value = Text::new(writer.bytes.into_boxed_slice(), limit).map_err(|_| TooLarge)?;
    Ok(Cursor { value })
}

/// Rejects malformed, over-bound and trailing data before admitting a cursor.
/// The engine still checks that the decoded query and position match the read.
#[must_use]
pub fn decode(value: &Cursor, limit: u32) -> Option<reads::Cursor> {
    if !value.value.fits(limit) {
        return None;
    }
    let mut reader = Reader { bytes: value.value.bytes() };
    if reader.byte()? != 1 {
        return None;
    }
    let cursor = take_cursor(&mut reader)?;
    if !reader.bytes.is_empty() {
        return None;
    }
    Some(cursor)
}

/// Preserve a Tree owner's continuation under the original task and window.
/// It is not a retained read cursor and never creates a window-follow action.
/// # Errors
/// Refuses the complete identity and cursor when they exceed the byte bound.
pub fn encode_tree(value: &documents::Cursor, task: u64, window: u32, limit: u32) -> Result<Cursor, TooLarge> {
    let mut writer = Writer { bytes: Vec::new(), limit: usize::try_from(limit).map_err(|_| TooLarge)? };
    writer.byte(2).ok_or(TooLarge)?;
    writer.u64(task).ok_or(TooLarge)?;
    writer.u32(window).ok_or(TooLarge)?;
    put_document_cursor(&mut writer, value).ok_or(TooLarge)?;
    Ok(Cursor { value: Text::new(writer.bytes.into_boxed_slice(), limit).map_err(|_| TooLarge)? })
}

/// Echoed continuations keep their complete original Tree identity.
#[must_use]
pub fn decode_tree(value: &Cursor, task: u64, window: u32, limit: u32) -> Option<documents::Cursor> {
    if !value.value.fits(limit) {
        return None;
    }
    let mut reader = Reader { bytes: value.value.bytes() };
    if reader.byte()? != 2 || reader.u64()? != task || reader.u32()? != window {
        return None;
    }
    let value = take_document_cursor(&mut reader)?;
    if !reader.bytes.is_empty() {
        return None;
    }
    Some(value)
}

/// The read owner admits positions; this boundary preserves the exact query namespace.
#[must_use]
pub fn belongs_to(value: &reads::Cursor, expected: &reads::Scope) -> bool {
    match value {
        reads::Cursor::Live { scope, .. } | reads::Cursor::Store { scope, .. } => scope == expected,
        reads::Cursor::ConversationsBefore { .. }
        | reads::Cursor::ConversationsLiveBefore { .. }
        | reads::Cursor::ConversationsStart => expected == &reads::Scope::Conversations,
    }
}

struct Writer {
    bytes: Vec<u8>,
    limit: usize,
}
impl Writer {
    fn put(&mut self, bytes: &[u8]) -> Option<()> {
        if self.bytes.len().checked_add(bytes.len())? > self.limit {
            return None;
        }
        self.bytes.extend_from_slice(bytes);
        Some(())
    }
    fn byte(&mut self, value: u8) -> Option<()> {
        self.put(&[value])
    }
    fn u16(&mut self, value: u16) -> Option<()> {
        self.put(&value.to_le_bytes())
    }
    fn u32(&mut self, value: u32) -> Option<()> {
        self.put(&value.to_le_bytes())
    }
    fn u64(&mut self, value: u64) -> Option<()> {
        self.put(&value.to_le_bytes())
    }
    fn boolean(&mut self, value: bool) -> Option<()> {
        self.byte(u8::from(value))
    }
    fn blob(&mut self, value: &[u8]) -> Option<()> {
        self.u32(u32::try_from(value.len()).ok()?)?;
        self.put(value)
    }
}
struct Reader<'a> {
    bytes: &'a [u8],
}
impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let value = self.bytes.get(..N)?.try_into().ok()?;
        self.bytes = self.bytes.get(N..)?;
        Some(value)
    }
    fn byte(&mut self) -> Option<u8> {
        Some(self.take::<1>()?[0])
    }
    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take()?))
    }
    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take()?))
    }
    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take()?))
    }
    fn boolean(&mut self) -> Option<bool> {
        match self.byte()? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
    fn blob(&mut self) -> Option<Box<[u8]>> {
        let count = usize::try_from(self.u32()?).ok()?;
        let value = self.bytes.get(..count)?.to_vec().into_boxed_slice();
        self.bytes = self.bytes.get(count..)?;
        Some(value)
    }
}
#[expect(
    clippy::ref_option,
    reason = "the ordinary codec borrows every optional owner field through one typed callback"
)]
fn put_option<T>(w: &mut Writer, value: &Option<T>, put: fn(&mut Writer, &T) -> Option<()>) -> Option<()> {
    match value {
        None => w.byte(0),
        Some(value) => {
            w.byte(1)?;
            put(w, value)
        }
    }
}
#[expect(
    clippy::option_option,
    reason = "outer absence means malformed bytes; inner absence is a valid supplied field"
)]
fn take_option<T>(r: &mut Reader<'_>, take: fn(&mut Reader<'_>) -> Option<T>) -> Option<Option<T>> {
    match r.byte()? {
        0 => Some(None),
        1 => Some(Some(take(r)?)),
        _ => None,
    }
}
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "uniform borrowed callbacks encode optional fields without changing ownership"
)]
fn put_u32(w: &mut Writer, value: &u32) -> Option<()> {
    w.u32(*value)
}
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "uniform borrowed callbacks encode optional fields without changing ownership"
)]
fn put_u64(w: &mut Writer, value: &u64) -> Option<()> {
    w.u64(*value)
}
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "uniform borrowed callbacks encode optional fields without changing ownership"
)]
fn put_wall(w: &mut Writer, value: &Wall) -> Option<()> {
    w.u64(value.as_nanos())
}
fn take_wall(r: &mut Reader<'_>) -> Option<Wall> {
    Some(Wall::from_nanos(r.u64()?))
}
fn put_blob(w: &mut Writer, value: &[u8]) -> Option<()> {
    w.blob(value)
}
fn take_blob(r: &mut Reader<'_>) -> Option<Box<[u8]>> {
    r.blob()
}

fn put_cursor(w: &mut Writer, value: &reads::Cursor) -> Option<()> {
    match value {
        reads::Cursor::Live { scope, after, store_after } => {
            w.byte(0)?;
            put_scope(w, scope)?;
            put_document_cursor(w, after)?;
            put_option(w, store_after, put_position)?;
        }
        reads::Cursor::Store { scope, after } => {
            w.byte(1)?;
            put_scope(w, scope)?;
            put_option(w, after, put_position)?;
        }
        reads::Cursor::ConversationsBefore { before } => {
            w.byte(2)?;
            put_position(w, before)?;
        }
        reads::Cursor::ConversationsLiveBefore { before } => {
            w.byte(3)?;
            put_option(w, before, put_document_cursor)?;
        }
        reads::Cursor::ConversationsStart => w.byte(4)?,
    }
    Some(())
}
fn take_cursor(r: &mut Reader<'_>) -> Option<reads::Cursor> {
    Some(match r.byte()? {
        0 => reads::Cursor::Live {
            scope: take_scope(r)?,
            after: take_document_cursor(r)?,
            store_after: take_option(r, take_position)?,
        },
        1 => reads::Cursor::Store { scope: take_scope(r)?, after: take_option(r, take_position)? },
        2 => reads::Cursor::ConversationsBefore { before: take_position(r)? },
        3 => reads::Cursor::ConversationsLiveBefore { before: take_option(r, take_document_cursor)? },
        4 => reads::Cursor::ConversationsStart,
        _ => return None,
    })
}

fn put_scope(w: &mut Writer, value: &reads::Scope) -> Option<()> {
    match value {
        reads::Scope::Task { task } => {
            w.byte(0)?;
            w.u64(*task)?;
        }
        reads::Scope::Proposal { proposer, proposal } => {
            w.byte(1)?;
            put_proposer(w, proposer)?;
            w.u64(*proposal)?;
        }
        reads::Scope::Escalation { task } => {
            w.byte(2)?;
            w.u64(*task)?;
        }
        reads::Scope::Question { task, message } => {
            w.byte(3)?;
            w.u64(*task)?;
            w.u64(*message)?;
        }
        reads::Scope::PersonTask { task } => {
            w.byte(4)?;
            w.u64(*task)?;
        }
        reads::Scope::Result { task } => {
            w.byte(5)?;
            w.u64(*task)?;
        }
        reads::Scope::Reply { task, activity } => {
            w.byte(6)?;
            w.u64(*task)?;
            w.u64(*activity)?;
        }
        reads::Scope::Turn { task, attempt, turn } => {
            w.byte(7)?;
            w.u64(*task)?;
            w.u64(*attempt)?;
            w.u32(*turn)?;
        }
        reads::Scope::EndedDelegates { task } => {
            w.byte(8)?;
            w.u64(*task)?;
        }
        reads::Scope::History { task, subtree } => {
            w.byte(9)?;
            w.u64(*task)?;
            w.boolean(*subtree)?;
        }
        reads::Scope::Activity { task } => {
            w.byte(10)?;
            w.u64(*task)?;
        }
        reads::Scope::Inbox { group, filter } => {
            w.byte(11)?;
            put_group(w, group)?;
            put_filter(w, filter)?;
        }
        reads::Scope::Conversations => {
            w.byte(12)?;
        }
        reads::Scope::Goals { project } => {
            w.byte(13)?;
            w.u32(*project)?;
        }
        reads::Scope::Notes { project } => {
            w.byte(14)?;
            w.u32(*project)?;
        }
        reads::Scope::ScopedNotes { scope, description } => {
            w.byte(15)?;
            put_note_scope(w, scope)?;
            put_option(w, description, |w, value| w.blob(value))?;
        }
        reads::Scope::Spend { project } => {
            w.byte(16)?;
            w.u32(*project)?;
        }
    }
    Some(())
}
fn take_scope(r: &mut Reader<'_>) -> Option<reads::Scope> {
    Some(match r.byte()? {
        0 => reads::Scope::Task { task: r.u64()? },
        1 => reads::Scope::Proposal { proposer: take_proposer(r)?, proposal: r.u64()? },
        2 => reads::Scope::Escalation { task: r.u64()? },
        3 => reads::Scope::Question { task: r.u64()?, message: r.u64()? },
        4 => reads::Scope::PersonTask { task: r.u64()? },
        5 => reads::Scope::Result { task: r.u64()? },
        6 => reads::Scope::Reply { task: r.u64()?, activity: r.u64()? },
        7 => reads::Scope::Turn { task: r.u64()?, attempt: r.u64()?, turn: r.u32()? },
        8 => reads::Scope::EndedDelegates { task: r.u64()? },
        9 => reads::Scope::History { task: r.u64()?, subtree: r.boolean()? },
        10 => reads::Scope::Activity { task: r.u64()? },
        11 => reads::Scope::Inbox { group: take_group(r)?, filter: take_filter(r)? },
        12 => reads::Scope::Conversations,
        13 => reads::Scope::Goals { project: r.u32()? },
        14 => reads::Scope::Notes { project: r.u32()? },
        15 => reads::Scope::ScopedNotes { scope: take_note_scope(r)?, description: take_option(r, take_blob)? },
        16 => reads::Scope::Spend { project: r.u32()? },
        _ => return None,
    })
}

fn put_position(w: &mut Writer, value: &reads::Position) -> Option<()> {
    match value {
        reads::Position::Owner(value) => {
            w.byte(0)?;
            put_key(w, value)?;
        }
        reads::Position::Inbox { at, task, object, key } => {
            w.byte(1)?;
            put_option(w, at, put_wall)?;
            w.u64(*task)?;
            put_inbox(w, object)?;
            put_key(w, key)?;
        }
    }
    Some(())
}
fn take_position(r: &mut Reader<'_>) -> Option<reads::Position> {
    Some(match r.byte()? {
        0 => reads::Position::Owner(take_key(r)?),
        1 => reads::Position::Inbox {
            at: take_option(r, take_wall)?,
            task: r.u64()?,
            object: take_inbox(r)?,
            key: take_key(r)?,
        },
        _ => return None,
    })
}

fn put_document_cursor(w: &mut Writer, value: &documents::Cursor) -> Option<()> {
    match value {
        documents::Cursor::Number { number } => {
            w.byte(0)?;
            w.u64(*number)?;
        }
        documents::Cursor::Inbox { task, object, at } => {
            w.byte(1)?;
            w.u64(*task)?;
            put_inbox(w, object)?;
            put_option(w, at, put_wall)?;
        }
    }
    Some(())
}
fn take_document_cursor(r: &mut Reader<'_>) -> Option<documents::Cursor> {
    Some(match r.byte()? {
        0 => documents::Cursor::Number { number: r.u64()? },
        1 => documents::Cursor::Inbox { task: r.u64()?, object: take_inbox(r)?, at: take_option(r, take_wall)? },
        _ => return None,
    })
}

fn put_inbox(w: &mut Writer, value: &documents::InboxObject) -> Option<()> {
    match value {
        documents::InboxObject::Question { message } => {
            w.byte(0)?;
            w.u64(*message)?;
        }
        documents::InboxObject::Proposal { proposer, number } => {
            w.byte(1)?;
            put_proposer(w, proposer)?;
            w.u64(*number)?;
        }
        documents::InboxObject::Escalation { revision } => {
            w.byte(2)?;
            w.u64(*revision)?;
        }
        documents::InboxObject::PersonTask => {
            w.byte(3)?;
        }
        documents::InboxObject::Reply { message } => {
            w.byte(4)?;
            w.u64(*message)?;
        }
        documents::InboxObject::Result { position } => {
            w.byte(5)?;
            w.u64(*position)?;
        }
    }
    Some(())
}
fn take_inbox(r: &mut Reader<'_>) -> Option<documents::InboxObject> {
    Some(match r.byte()? {
        0 => documents::InboxObject::Question { message: r.u64()? },
        1 => documents::InboxObject::Proposal { proposer: take_proposer(r)?, number: r.u64()? },
        2 => documents::InboxObject::Escalation { revision: r.u64()? },
        3 => documents::InboxObject::PersonTask,
        4 => documents::InboxObject::Reply { message: r.u64()? },
        5 => documents::InboxObject::Result { position: r.u64()? },
        _ => return None,
    })
}

fn put_proposer(w: &mut Writer, value: &people::Proposer) -> Option<()> {
    match value {
        people::Proposer::Task(value) => {
            w.byte(0)?;
            w.u64(*value)?;
        }
        people::Proposer::Party(value) => {
            w.byte(1)?;
            w.u64(*value)?;
        }
    }
    Some(())
}
fn take_proposer(r: &mut Reader<'_>) -> Option<people::Proposer> {
    Some(match r.byte()? {
        0 => people::Proposer::Task(r.u64()?),
        1 => people::Proposer::Party(r.u64()?),
        _ => return None,
    })
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "uniform borrowed callbacks encode optional fields without changing ownership"
)]
fn put_group(w: &mut Writer, value: &reads::Group) -> Option<()> {
    match value {
        reads::Group::Needs => {
            w.byte(0)?;
        }
        reads::Group::Updates => {
            w.byte(1)?;
        }
        reads::Group::Proposals => {
            w.byte(2)?;
        }
        reads::Group::Escalations => {
            w.byte(3)?;
        }
        reads::Group::Questions => {
            w.byte(4)?;
        }
        reads::Group::PersonTasks => {
            w.byte(5)?;
        }
        reads::Group::Results => {
            w.byte(6)?;
        }
        reads::Group::Replies => {
            w.byte(7)?;
        }
    }
    Some(())
}
fn take_group(r: &mut Reader<'_>) -> Option<reads::Group> {
    Some(match r.byte()? {
        0 => reads::Group::Needs,
        1 => reads::Group::Updates,
        2 => reads::Group::Proposals,
        3 => reads::Group::Escalations,
        4 => reads::Group::Questions,
        5 => reads::Group::PersonTasks,
        6 => reads::Group::Results,
        7 => reads::Group::Replies,
        _ => return None,
    })
}

fn put_note_scope(w: &mut Writer, value: &notes::Scope) -> Option<()> {
    match value {
        notes::Scope::Deployment => {
            w.byte(0)?;
        }
        notes::Scope::Project { project } => {
            w.byte(1)?;
            w.u32(*project)?;
        }
        notes::Scope::Goal { project, goal } => {
            w.byte(2)?;
            w.u32(*project)?;
            w.u64(*goal)?;
        }
        notes::Scope::Resources { project, connector, pattern } => {
            w.byte(3)?;
            w.u32(*project)?;
            w.u16(*connector)?;
            put_pattern(w, pattern)?;
        }
    }
    Some(())
}
fn take_note_scope(r: &mut Reader<'_>) -> Option<notes::Scope> {
    Some(match r.byte()? {
        0 => notes::Scope::Deployment,
        1 => notes::Scope::Project { project: r.u32()? },
        2 => notes::Scope::Goal { project: r.u32()?, goal: r.u64()? },
        3 => notes::Scope::Resources { project: r.u32()?, connector: r.u16()?, pattern: take_pattern(r)? },
        _ => return None,
    })
}

fn put_last(w: &mut Writer, value: &notes::Last) -> Option<()> {
    match value {
        notes::Last::Exact(value) => {
            w.byte(0)?;
            put_blob(w, value)?;
        }
        notes::Last::Open(value) => {
            w.byte(1)?;
            put_blob(w, value)?;
        }
    }
    Some(())
}
fn take_last(r: &mut Reader<'_>) -> Option<notes::Last> {
    Some(match r.byte()? {
        0 => notes::Last::Exact(take_blob(r)?),
        1 => notes::Last::Open(take_blob(r)?),
        _ => return None,
    })
}

fn put_key(w: &mut Writer, value: &Key) -> Option<()> {
    match value {
        Key::Core(value) => {
            w.byte(0)?;
            put_core_key(w, value)?;
        }
        Key::Tasks(value) => {
            w.byte(1)?;
            put_task_key(w, value)?;
        }
        Key::People(value) => {
            w.byte(2)?;
            put_people_key(w, value)?;
        }
        Key::Notes(value) => {
            w.byte(3)?;
            put_note_key(w, value)?;
        }
    }
    Some(())
}
fn take_key(r: &mut Reader<'_>) -> Option<Key> {
    Some(match r.byte()? {
        0 => Key::Core(take_core_key(r)?),
        1 => Key::Tasks(take_task_key(r)?),
        2 => Key::People(take_people_key(r)?),
        3 => Key::Notes(take_note_key(r)?),
        _ => return None,
    })
}

fn put_core_key(w: &mut Writer, value: &CoreKey) -> Option<()> {
    match value {
        CoreKey::Projection(value) => {
            w.byte(1)?;
            put_projection_key(w, value)?;
        }
        CoreKey::Deployment => {
            w.byte(0)?;
        }
        CoreKey::RunProof(value) => {
            w.byte(2)?;
            w.u64(*value)?;
        }
        CoreKey::Call(value) => {
            w.byte(3)?;
            put_call_key(w, value)?;
        }
        CoreKey::Turn { task, attempt, turn } => {
            w.byte(4)?;
            w.u64(*task)?;
            w.u64(*attempt)?;
            w.u32(*turn)?;
        }
        CoreKey::Terminal { task, attempt } => {
            w.byte(5)?;
            w.u64(*task)?;
            w.u64(*attempt)?;
        }
        CoreKey::ProposalDecision { proposer, proposal } => {
            w.byte(6)?;
            put_proposer(w, proposer)?;
            w.u64(*proposal)?;
        }
        CoreKey::EscalationDecision { task, revision } => {
            w.byte(7)?;
            w.u64(*task)?;
            w.u64(*revision)?;
        }
    }
    Some(())
}
fn take_core_key(r: &mut Reader<'_>) -> Option<CoreKey> {
    Some(match r.byte()? {
        0 => CoreKey::Deployment,
        1 => CoreKey::Projection(take_projection_key(r)?),
        2 => CoreKey::RunProof(r.u64()?),
        3 => CoreKey::Call(take_call_key(r)?),
        4 => CoreKey::Turn { task: r.u64()?, attempt: r.u64()?, turn: r.u32()? },
        5 => CoreKey::Terminal { task: r.u64()?, attempt: r.u64()? },
        6 => CoreKey::ProposalDecision { proposer: take_proposer(r)?, proposal: r.u64()? },
        7 => CoreKey::EscalationDecision { task: r.u64()?, revision: r.u64()? },
        _ => return None,
    })
}

fn put_projection_key(w: &mut Writer, value: &ProjectionKey) -> Option<()> {
    match value {
        ProjectionKey::Goal(value) => {
            w.byte(0)?;
            w.u64(*value)?;
        }
        ProjectionKey::Task { goal, task } => {
            w.byte(1)?;
            w.u64(*goal)?;
            w.u64(*task)?;
        }
        ProjectionKey::Lifecycle { goal, task, position } => {
            w.byte(2)?;
            w.u64(*goal)?;
            w.u64(*task)?;
            w.u64(*position)?;
        }
        ProjectionKey::Revision { goal, task, revision } => {
            w.byte(3)?;
            w.u64(*goal)?;
            w.u64(*task)?;
            w.u64(*revision)?;
        }
    }
    Some(())
}
fn take_projection_key(r: &mut Reader<'_>) -> Option<ProjectionKey> {
    Some(match r.byte()? {
        0 => ProjectionKey::Goal(r.u64()?),
        1 => ProjectionKey::Task { goal: r.u64()?, task: r.u64()? },
        2 => ProjectionKey::Lifecycle { goal: r.u64()?, task: r.u64()?, position: r.u64()? },
        3 => ProjectionKey::Revision { goal: r.u64()?, task: r.u64()?, revision: r.u64()? },
        _ => return None,
    })
}

fn put_people_key(w: &mut Writer, value: &people::Key) -> Option<()> {
    match value {
        people::Key::Person(value) => {
            w.byte(0)?;
            w.u64(*value)?;
        }
        people::Key::ReadResults { party } => {
            w.byte(1)?;
            w.u64(*party)?;
        }
        people::Key::ReadChat { party, task } => {
            w.byte(2)?;
            w.u64(*party)?;
            w.u64(*task)?;
        }
        people::Key::SignIn(value) => {
            w.byte(3)?;
            w.u64(*value)?;
        }
        people::Key::Roles(value) => {
            w.byte(4)?;
            w.u32(*value)?;
        }
        people::Key::Policy(value) => {
            w.byte(5)?;
            w.u32(*value)?;
        }
        people::Key::Answer(value) => {
            w.byte(6)?;
            put_request_key(w, value)?;
        }
    }
    Some(())
}
fn take_people_key(r: &mut Reader<'_>) -> Option<people::Key> {
    Some(match r.byte()? {
        0 => people::Key::Person(r.u64()?),
        1 => people::Key::ReadResults { party: r.u64()? },
        2 => people::Key::ReadChat { party: r.u64()?, task: r.u64()? },
        3 => people::Key::SignIn(r.u64()?),
        4 => people::Key::Roles(r.u32()?),
        5 => people::Key::Policy(r.u32()?),
        6 => people::Key::Answer(take_request_key(r)?),
        _ => return None,
    })
}

fn put_note_key(w: &mut Writer, value: &notes::Key) -> Option<()> {
    match value {
        notes::Key::Entry { name } => {
            w.byte(0)?;
            w.u64(*name)?;
        }
        notes::Key::Line { scope, name } => {
            w.byte(1)?;
            put_note_scope(w, scope)?;
            w.u64(*name)?;
        }
    }
    Some(())
}
fn take_note_key(r: &mut Reader<'_>) -> Option<notes::Key> {
    Some(match r.byte()? {
        0 => notes::Key::Entry { name: r.u64()? },
        1 => notes::Key::Line { scope: take_note_scope(r)?, name: r.u64()? },
        _ => return None,
    })
}

fn put_funder(w: &mut Writer, value: &tasks::Funder) -> Option<()> {
    match value {
        tasks::Funder::Task(value) => {
            w.byte(0)?;
            w.u64(*value)?;
        }
        tasks::Funder::Pool { project, person, period } => {
            w.byte(1)?;
            w.u32(*project)?;
            w.u64(*person)?;
            w.u64(*period)?;
        }
        tasks::Funder::Recurring { project, task, period } => {
            w.byte(2)?;
            w.u32(*project)?;
            w.u64(*task)?;
            w.u64(*period)?;
        }
        tasks::Funder::Period { project, period } => {
            w.byte(3)?;
            w.u32(*project)?;
            w.u64(*period)?;
        }
    }
    Some(())
}
fn take_funder(r: &mut Reader<'_>) -> Option<tasks::Funder> {
    Some(match r.byte()? {
        0 => tasks::Funder::Task(r.u64()?),
        1 => tasks::Funder::Pool { project: r.u32()?, person: r.u64()?, period: r.u64()? },
        2 => tasks::Funder::Recurring { project: r.u32()?, task: r.u64()?, period: r.u64()? },
        3 => tasks::Funder::Period { project: r.u32()?, period: r.u64()? },
        _ => return None,
    })
}

fn put_task_key(w: &mut Writer, value: &tasks::Key) -> Option<()> {
    match value {
        tasks::Key::Activity { task, position } => {
            w.byte(9)?;
            w.u64(*task)?;
            w.u64(*position)?;
        }
        tasks::Key::PersonProposal(value) => {
            w.byte(5)?;
            w.u64(*value)?;
        }
        tasks::Key::History { task, revision } => {
            w.byte(7)?;
            w.u64(*task)?;
            w.u64(*revision)?;
        }
        tasks::Key::Milestone { task, position } => {
            w.byte(8)?;
            w.u64(*task)?;
            w.u64(*position)?;
        }
        tasks::Key::Live(value) => {
            w.byte(0)?;
            w.u64(*value)?;
        }
        tasks::Key::Ended(value) => {
            w.byte(6)?;
            w.u64(*value)?;
        }
        tasks::Key::Stub(value) => {
            w.byte(1)?;
            w.u64(*value)?;
        }
        tasks::Key::Ledger(value) => {
            w.byte(2)?;
            put_funder(w, value)?;
        }
        tasks::Key::Writer(value) => {
            w.byte(3)?;
            w.u64(*value)?;
        }
        tasks::Key::Pool(value) => {
            w.byte(4)?;
            w.u64(*value)?;
        }
    }
    Some(())
}
fn take_task_key(r: &mut Reader<'_>) -> Option<tasks::Key> {
    Some(match r.byte()? {
        9 => tasks::Key::Activity { task: r.u64()?, position: r.u64()? },
        5 => tasks::Key::PersonProposal(r.u64()?),
        7 => tasks::Key::History { task: r.u64()?, revision: r.u64()? },
        8 => tasks::Key::Milestone { task: r.u64()?, position: r.u64()? },
        0 => tasks::Key::Live(r.u64()?),
        6 => tasks::Key::Ended(r.u64()?),
        1 => tasks::Key::Stub(r.u64()?),
        2 => tasks::Key::Ledger(take_funder(r)?),
        3 => tasks::Key::Writer(r.u64()?),
        4 => tasks::Key::Pool(r.u64()?),
        _ => return None,
    })
}

fn put_filter(w: &mut Writer, value: &reads::Filter) -> Option<()> {
    put_option(w, &value.project, put_u32)?;
    put_option(w, &value.goal, put_u64)
}
fn take_u32(r: &mut Reader<'_>) -> Option<u32> {
    r.u32()
}
fn take_u64(r: &mut Reader<'_>) -> Option<u64> {
    r.u64()
}
fn take_filter(r: &mut Reader<'_>) -> Option<reads::Filter> {
    Some(reads::Filter { project: take_option(r, take_u32)?, goal: take_option(r, take_u64)? })
}
fn put_call_key(w: &mut Writer, value: &CallKey) -> Option<()> {
    w.u64(value.task)?;
    w.u64(value.attempt)?;
    w.u32(value.completion)?;
    w.u32(value.position)
}
fn take_call_key(r: &mut Reader<'_>) -> Option<CallKey> {
    Some(CallKey { task: r.u64()?, attempt: r.u64()?, completion: r.u32()?, position: r.u32()? })
}
fn put_request_key(w: &mut Writer, value: &people::RequestKey) -> Option<()> {
    w.u64(value.person)?;
    w.put(&value.key)
}
fn take_request_key(r: &mut Reader<'_>) -> Option<people::RequestKey> {
    Some(people::RequestKey { person: r.u64()?, key: r.take()? })
}
fn put_pattern(w: &mut Writer, value: &notes::Pattern) -> Option<()> {
    w.u32(u32::try_from(value.segments.len()).ok()?)?;
    for segment in &value.segments {
        w.blob(segment)?;
    }
    put_last(w, &value.last)
}
fn take_pattern(r: &mut Reader<'_>) -> Option<notes::Pattern> {
    let count = usize::try_from(r.u32()?).ok()?;
    // Each segment has at least its four-byte length. Validate before allocation.
    if count > r.bytes.len() / 4 {
        return None;
    }
    let mut segments = Vec::with_capacity(count);
    for _ in 0..count {
        segments.push(r.blob()?);
    }
    Some(notes::Pattern { segments: segments.into_boxed_slice(), last: take_last(r)? })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversation_owner_both_end_cursors_survive_reload_whole_without_reversing_or_default_positions() {
        let position = reads::Position::Owner(Key::Core(CoreKey::Terminal { task: u64::MAX, attempt: 0 }));
        for source in [
            reads::Cursor::ConversationsBefore { before: position },
            reads::Cursor::ConversationsLiveBefore { before: None },
            reads::Cursor::ConversationsLiveBefore { before: Some(documents::Cursor::Number { number: 0 }) },
            reads::Cursor::ConversationsLiveBefore { before: Some(documents::Cursor::Number { number: u64::MAX }) },
            reads::Cursor::ConversationsStart,
            reads::Cursor::Store { scope: reads::Scope::Conversations, after: None },
        ] {
            let encoded = encode(&source, 200).expect("whole owner continuation");
            let length = u32::try_from(encoded.value.bytes().len()).expect("bounded cursor");
            assert_eq!(encode(&source, length).expect("exact bound"), encoded);
            assert_eq!(encode(&source, length - 1), Err(TooLarge));
            let saved = Cursor { value: Text::new(encoded.value.into_bytes(), length).expect("saved whole bytes") };
            assert_eq!(decode(&saved, length), Some(source.clone()));
            assert!(belongs_to(&source, &reads::Scope::Conversations));
            assert!(!belongs_to(&source, &reads::Scope::Goals { project: 0 }));
            let request = jig_domain_client::contract::Read::Conversations { after: Some(saved) };
            assert_eq!(
                super::super::reads::read(&request, length).expect("original query"),
                reads::Read::Conversations { from: Some(source) }
            );
        }
        assert_ne!(
            encode(&reads::Cursor::ConversationsStart, 200).expect("start"),
            encode(&reads::Cursor::Store { scope: reads::Scope::Conversations, after: None }, 200)
                .expect("stored start")
        );
        assert_ne!(
            encode(&reads::Cursor::ConversationsLiveBefore { before: None }, 200).expect("absent"),
            encode(
                &reads::Cursor::ConversationsLiveBefore { before: Some(documents::Cursor::Number { number: 0 }) },
                200
            )
            .expect("zero")
        );
    }

    #[test]
    fn opaque_note_cursors_keep_literal_segments_terminal_bytes_and_whole_bounds_across_reload() {
        let scope = notes::Scope::Resources {
            project: u32::MAX,
            connector: u16::MAX,
            pattern: notes::Pattern {
                segments: vec![b"a/b".to_vec().into_boxed_slice(), Box::new([]), vec![0xff].into_boxed_slice()]
                    .into_boxed_slice(),
                last: notes::Last::Exact(b"end/".to_vec().into_boxed_slice()),
            },
        };
        let cursor = reads::Cursor::Store {
            scope: reads::Scope::ScopedNotes { scope: scope.clone(), description: Some(Box::new([])) },
            after: Some(reads::Position::Owner(Key::Notes(notes::Key::Line { scope, name: u64::MAX }))),
        };
        let encoded = encode(&cursor, 1024).expect("complete cursor fits");
        let length = u32::try_from(encoded.value.bytes().len()).expect("bounded length");
        assert_eq!(encode(&cursor, length).expect("exact bound"), encoded);
        assert_eq!(encode(&cursor, length - 1), Err(TooLarge));
        // Only saved bytes survive. A newly made reader reconstructs the complete owner cursor.
        let saved = encoded.value.into_bytes();
        let restored = Cursor { value: Text::new(saved, length).expect("saved whole bytes") };
        assert_eq!(decode(&restored, length), Some(cursor));
        assert_eq!(decode(&restored, length - 1), None);
    }

    #[test]
    fn cursor_decoder_refuses_truncation_trailing_bytes_unknown_tags_and_oversized_segment_counts() {
        let expected = reads::Cursor::Store { scope: reads::Scope::Conversations, after: None };
        let bytes = vec![1, 1, 12, 0];
        let cursor = Cursor { value: Text::new(bytes.clone().into_boxed_slice(), 64).expect("short cursor") };
        assert_eq!(decode(&cursor, 64), Some(expected.clone()));
        assert_eq!(encode(&expected, 64).expect("whole cursor").value.bytes(), bytes);
        for length in 0..bytes.len() {
            let cut =
                Cursor { value: Text::new(bytes[..length].to_vec().into_boxed_slice(), 64).expect("short bytes") };
            assert_eq!(decode(&cut, 64), None);
        }
        for malformed in [vec![1, 1, 12, 0, 0], vec![2, 1, 12, 0], vec![1, 2], vec![1, 1, 255], vec![1, 1, 12, 2]] {
            let value = Cursor { value: Text::new(malformed.into_boxed_slice(), 64).expect("bounded malformed bytes") };
            assert_eq!(decode(&value, 64), None);
        }
        let mut huge_count = vec![1, 1, 15, 3];
        huge_count.extend_from_slice(&1_u32.to_le_bytes());
        huge_count.extend_from_slice(&2_u16.to_le_bytes());
        huge_count.extend_from_slice(&u32::MAX.to_le_bytes());
        let value = Cursor { value: Text::new(huge_count.into_boxed_slice(), 64).expect("bounded count prefix") };
        assert_eq!(decode(&value, 64), None);
    }

    #[test]
    fn cursors_distinguish_same_number_makers_parent_groups_filters_and_absent_wall_positions() {
        let task = reads::Cursor::Live {
            scope: reads::Scope::Inbox {
                group: reads::Group::Proposals,
                filter: reads::Filter { project: Some(u32::MAX), goal: Some(u64::MAX) },
            },
            after: documents::Cursor::Inbox {
                task: 7,
                object: documents::InboxObject::Proposal { proposer: people::Proposer::Task(9), number: 11 },
                at: None,
            },
            store_after: Some(reads::Position::Inbox {
                at: Some(Wall::from_nanos(u64::MAX)),
                task: 7,
                object: documents::InboxObject::Proposal { proposer: people::Proposer::Party(9), number: 11 },
                key: Key::Core(CoreKey::ProposalDecision { proposer: people::Proposer::Party(9), proposal: 11 }),
            }),
        };
        let mut other = task.clone();
        let reads::Cursor::Live { scope, after, .. } = &mut other else {
            panic!("live cursor");
        };
        *scope = reads::Scope::Inbox {
            group: reads::Group::Needs,
            filter: reads::Filter { project: Some(u32::MAX), goal: None },
        };
        *after = documents::Cursor::Inbox {
            task: 7,
            object: documents::InboxObject::Proposal { proposer: people::Proposer::Party(9), number: 11 },
            at: Some(Wall::EPOCH),
        };
        let a = encode(&task, 1024).expect("bounded cursor");
        let b = encode(&other, 1024).expect("bounded cursor");
        assert_ne!(a, b);
        assert_eq!(decode(&a, 1024), Some(task));
        assert_eq!(decode(&b, 1024), Some(other));
    }
}
