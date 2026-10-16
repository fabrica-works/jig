//! Whole owner cursors, with keys carried by the store codec (client.md, 6).
use alloc::boxed::Box;
use jig_domain_core::{Key, documents, reads};
use jig_domain_notes as notes;
use jig_domain_people as people;
use skein_lib::{List, Wall};

pub fn encode(value: &reads::Cursor, limit: u32) -> Option<Box<[u8]>> {
    let mut out = Writer { bytes: skein_lib::Intake::with_capacity(limit) };
    out.byte(1)?;
    put_cursor(&mut out, value)?;
    out.bytes.meet(skein_lib::stream::Read::Fill(out.bytes.len()))
}
pub(crate) fn encode_tree(value: &documents::Cursor, task: u64, window: u32, limit: u32) -> Option<Box<[u8]>> {
    let mut out = Writer { bytes: skein_lib::Intake::with_capacity(limit) };
    out.byte(2)?;
    out.u64(task)?;
    out.u32(window)?;
    put_document_cursor(&mut out, value)?;
    out.bytes.meet(skein_lib::stream::Read::Fill(out.bytes.len()))
}
pub fn decode(bytes: &[u8]) -> Option<reads::Cursor> {
    if bytes.len() > usize::try_from(super::w::CEILINGS.cursor_value).ok()? {
        return None;
    }
    let mut r = skein_lib::Reader::new(bytes);
    if r.u8()? != 1 {
        return None;
    }
    let value = take_cursor(&mut r)?;
    if !r.is_empty() {
        return None;
    }
    Some(value)
}
struct Writer {
    bytes: skein_lib::Intake,
}
impl Writer {
    fn byte(&mut self, value: u8) -> Option<()> {
        self.bytes.append(&[value]).ok()
    }
    fn u16(&mut self, value: u16) -> Option<()> {
        self.bytes.append(&value.to_le_bytes()).ok()
    }
    fn u32(&mut self, value: u32) -> Option<()> {
        self.bytes.append(&value.to_le_bytes()).ok()
    }
    fn u64(&mut self, value: u64) -> Option<()> {
        self.bytes.append(&value.to_le_bytes()).ok()
    }
    fn boolean(&mut self, value: bool) -> Option<()> {
        self.byte(u8::from(value))
    }
    fn blob(&mut self, value: &[u8]) -> Option<()> {
        self.u32(u32::try_from(value.len()).ok()?)?;
        self.bytes.append(value).ok()
    }
    fn optional_blob(&mut self, value: Option<&[u8]>) -> Option<()> {
        match value {
            None => self.byte(0),
            Some(value) => {
                self.byte(1)?;
                self.blob(value)
            }
        }
    }
    fn optional_u32(&mut self, value: Option<u32>) -> Option<()> {
        match value {
            None => self.byte(0),
            Some(value) => {
                self.byte(1)?;
                self.u32(value)
            }
        }
    }
    fn optional_u64(&mut self, value: Option<u64>) -> Option<()> {
        match value {
            None => self.byte(0),
            Some(value) => {
                self.byte(1)?;
                self.u64(value)
            }
        }
    }
    fn optional_wall(&mut self, value: Option<Wall>) -> Option<()> {
        match value {
            None => self.byte(0),
            Some(value) => {
                self.byte(1)?;
                self.u64(value.as_nanos())
            }
        }
    }
}
fn u16(r: &mut skein_lib::Reader<'_>) -> Option<u16> {
    Some(u16::from_le_bytes(r.bytes(2)?.try_into().ok()?))
}
fn u32(r: &mut skein_lib::Reader<'_>) -> Option<u32> {
    Some(u32::from_le_bytes(r.bytes(4)?.try_into().ok()?))
}
fn u64(r: &mut skein_lib::Reader<'_>) -> Option<u64> {
    Some(u64::from_le_bytes(r.bytes(8)?.try_into().ok()?))
}
fn boolean(r: &mut skein_lib::Reader<'_>) -> Option<bool> {
    match r.u8()? {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}
fn blob(r: &mut skein_lib::Reader<'_>) -> Option<Box<[u8]>> {
    let length = u32(r)?;
    Some(skein_lib::bytes::copy_of(r.bytes(length)?))
}
fn put_blob(w: &mut Writer, value: &[u8]) -> Option<()> {
    w.blob(value)
}
#[expect(clippy::option_option, reason = "outer absence is malformed; inner absence is an owner's absent field")]
fn optional_blob(r: &mut skein_lib::Reader<'_>) -> Option<Option<Box<[u8]>>> {
    match r.u8()? {
        0 => Some(None),
        1 => Some(Some(blob(r)?)),
        _ => None,
    }
}
#[expect(clippy::option_option, reason = "outer absence is malformed; inner absence is an owner's absent field")]
fn optional_wall(r: &mut skein_lib::Reader<'_>) -> Option<Option<Wall>> {
    match r.u8()? {
        0 => Some(None),
        1 => Some(Some(Wall::from_nanos(u64(r)?))),
        _ => None,
    }
}
#[expect(clippy::option_option, reason = "outer absence is malformed; inner absence is an owner's absent field")]
fn optional_u32(r: &mut skein_lib::Reader<'_>) -> Option<Option<u32>> {
    match r.u8()? {
        0 => Some(None),
        1 => Some(Some(u32(r)?)),
        _ => None,
    }
}
#[expect(clippy::option_option, reason = "outer absence is malformed; inner absence is an owner's absent field")]
fn optional_u64(r: &mut skein_lib::Reader<'_>) -> Option<Option<u64>> {
    match r.u8()? {
        0 => Some(None),
        1 => Some(Some(u64(r)?)),
        _ => None,
    }
}
fn put_cursor(w: &mut Writer, value: &reads::Cursor) -> Option<()> {
    match value {
        reads::Cursor::Live { scope, after, store_after } => {
            w.byte(0)?;
            put_scope(w, scope)?;
            put_document_cursor(w, after)?;
            match store_after {
                Some(value) => {
                    w.byte(1)?;
                    put_position(w, value)?;
                }
                None => w.byte(0)?,
            }
        }
        reads::Cursor::Store { scope, after } => {
            w.byte(1)?;
            put_scope(w, scope)?;
            match after {
                Some(value) => {
                    w.byte(1)?;
                    put_position(w, value)?;
                }
                None => w.byte(0)?,
            }
        }
        reads::Cursor::ConversationsBefore { before } => {
            w.byte(2)?;
            put_position(w, before)?;
        }
        reads::Cursor::ConversationsLiveBefore { before } => {
            w.byte(3)?;
            match before {
                Some(value) => {
                    w.byte(1)?;
                    put_document_cursor(w, value)?;
                }
                None => w.byte(0)?,
            }
        }
        reads::Cursor::ConversationsStart => w.byte(4)?,
    }
    Some(())
}
fn take_cursor(r: &mut skein_lib::Reader<'_>) -> Option<reads::Cursor> {
    Some(match r.u8()? {
        0 => {
            let scope = take_scope(r)?;
            let after = take_document_cursor(r)?;
            let store_after = match r.u8()? {
                0 => None,
                1 => Some(take_position(r)?),
                _ => return None,
            };
            reads::Cursor::Live { scope, after, store_after }
        }
        1 => {
            let scope = take_scope(r)?;
            let after = match r.u8()? {
                0 => None,
                1 => Some(take_position(r)?),
                _ => return None,
            };
            reads::Cursor::Store { scope, after }
        }
        2 => reads::Cursor::ConversationsBefore { before: take_position(r)? },
        3 => {
            let before = match r.u8()? {
                0 => None,
                1 => Some(take_document_cursor(r)?),
                _ => return None,
            };
            reads::Cursor::ConversationsLiveBefore { before }
        }
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
            put_group(w, *group)?;
            put_filter(w, filter)?;
        }
        reads::Scope::Conversations => {
            w.byte(12)?;
        }
        reads::Scope::Tasks { .. } => return None,
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
            w.optional_blob(description.as_deref())?;
        }
        reads::Scope::Spend { project } => {
            w.byte(16)?;
            w.u32(*project)?;
        }
    }
    Some(())
}
fn take_scope(r: &mut skein_lib::Reader<'_>) -> Option<reads::Scope> {
    Some(match r.u8()? {
        0 => reads::Scope::Task { task: u64(r)? },
        1 => reads::Scope::Proposal { proposer: take_proposer(r)?, proposal: u64(r)? },
        2 => reads::Scope::Escalation { task: u64(r)? },
        3 => reads::Scope::Question { task: u64(r)?, message: u64(r)? },
        4 => reads::Scope::PersonTask { task: u64(r)? },
        5 => reads::Scope::Result { task: u64(r)? },
        6 => reads::Scope::Reply { task: u64(r)?, activity: u64(r)? },
        7 => reads::Scope::Turn { task: u64(r)?, attempt: u64(r)?, turn: u32(r)? },
        8 => reads::Scope::EndedDelegates { task: u64(r)? },
        9 => reads::Scope::History { task: u64(r)?, subtree: boolean(r)? },
        10 => reads::Scope::Activity { task: u64(r)? },
        11 => reads::Scope::Inbox { group: take_group(r)?, filter: take_filter(r)? },
        12 => reads::Scope::Conversations,
        13 => reads::Scope::Goals { project: u32(r)? },
        14 => reads::Scope::Notes { project: u32(r)? },
        15 => reads::Scope::ScopedNotes { scope: take_note_scope(r)?, description: optional_blob(r)? },
        16 => reads::Scope::Spend { project: u32(r)? },
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
            w.optional_wall(*at)?;
            w.u64(*task)?;
            put_inbox(w, object)?;
            put_key(w, key)?;
        }
    }
    Some(())
}
fn take_position(r: &mut skein_lib::Reader<'_>) -> Option<reads::Position> {
    Some(match r.u8()? {
        0 => reads::Position::Owner(take_key(r)?),
        1 => reads::Position::Inbox { at: optional_wall(r)?, task: u64(r)?, object: take_inbox(r)?, key: take_key(r)? },
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
            w.optional_wall(*at)?;
        }
    }
    Some(())
}
fn take_document_cursor(r: &mut skein_lib::Reader<'_>) -> Option<documents::Cursor> {
    Some(match r.u8()? {
        0 => documents::Cursor::Number { number: u64(r)? },
        1 => documents::Cursor::Inbox { task: u64(r)?, object: take_inbox(r)?, at: optional_wall(r)? },
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
fn take_inbox(r: &mut skein_lib::Reader<'_>) -> Option<documents::InboxObject> {
    Some(match r.u8()? {
        0 => documents::InboxObject::Question { message: u64(r)? },
        1 => documents::InboxObject::Proposal { proposer: take_proposer(r)?, number: u64(r)? },
        2 => documents::InboxObject::Escalation { revision: u64(r)? },
        3 => documents::InboxObject::PersonTask,
        4 => documents::InboxObject::Reply { message: u64(r)? },
        5 => documents::InboxObject::Result { position: u64(r)? },
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
fn take_proposer(r: &mut skein_lib::Reader<'_>) -> Option<people::Proposer> {
    Some(match r.u8()? {
        0 => people::Proposer::Task(u64(r)?),
        1 => people::Proposer::Party(u64(r)?),
        _ => return None,
    })
}

fn put_group(w: &mut Writer, value: reads::Group) -> Option<()> {
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
fn take_group(r: &mut skein_lib::Reader<'_>) -> Option<reads::Group> {
    Some(match r.u8()? {
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
fn take_note_scope(r: &mut skein_lib::Reader<'_>) -> Option<notes::Scope> {
    Some(match r.u8()? {
        0 => notes::Scope::Deployment,
        1 => notes::Scope::Project { project: u32(r)? },
        2 => notes::Scope::Goal { project: u32(r)?, goal: u64(r)? },
        3 => notes::Scope::Resources { project: u32(r)?, connector: u16(r)?, pattern: take_pattern(r)? },
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
fn take_last(r: &mut skein_lib::Reader<'_>) -> Option<notes::Last> {
    Some(match r.u8()? {
        0 => notes::Last::Exact(blob(r)?),
        1 => notes::Last::Open(blob(r)?),
        _ => return None,
    })
}

fn put_key(w: &mut Writer, value: &Key) -> Option<()> {
    let bytes = jig_protocol_store::keys::encode(value, w.bytes.room())?;
    w.blob(&bytes)
}
fn take_key(r: &mut skein_lib::Reader<'_>) -> Option<Key> {
    let length = u32(r)?;
    jig_protocol_store::keys::decode(r.bytes(length)?)
}
fn put_filter(w: &mut Writer, value: &reads::Filter) -> Option<()> {
    w.optional_u32(value.project)?;
    w.optional_u64(value.goal)
}
fn take_filter(r: &mut skein_lib::Reader<'_>) -> Option<reads::Filter> {
    Some(reads::Filter { project: optional_u32(r)?, goal: optional_u64(r)? })
}
fn put_pattern(w: &mut Writer, value: &notes::Pattern) -> Option<()> {
    w.u32(u32::try_from(value.segments.len()).ok()?)?;
    for segment in &value.segments {
        w.blob(segment)?;
    }
    put_last(w, &value.last)
}
fn take_pattern(r: &mut skein_lib::Reader<'_>) -> Option<notes::Pattern> {
    let count = u32(r)?;
    if count > r.remaining().checked_div(4)? {
        return None;
    }
    let mut segments = List::with_capacity(count);
    for _ in 0..count {
        segments.push(blob(r)?).ok()?;
    }
    Some(notes::Pattern { segments: segments.into_boxed(), last: take_last(r)? })
}
