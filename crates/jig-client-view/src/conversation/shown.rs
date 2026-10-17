//! Literal cards drawn by the conversation (view.md, 4 and 9).
//! Task-page proposals remain beside the log unless that exact shared key
//! already has a card in a supplied turn, live call or decision.
use crate::conversation::Conversation;
use jig_client::contract::{Activity, ArchivedTurn, Block, LiveCall, ObjectKey, Turn, TurnRead};

impl Conversation {
    /// Whether a shown conversation line or live call shows this object (view.md, 4.1).
    #[must_use]
    pub fn shows(&self, key: ObjectKey) -> bool {
        for line in &self.lines.rows {
            match &line.event {
                Activity::Turn { turn: TurnRead::Decoded(turn), .. }
                | Activity::Owner { decoded: Some(ArchivedTurn { turn: TurnRead::Decoded(turn), .. }), .. } => {
                    if turn_shows(turn, key) {
                        return true;
                    }
                }
                Activity::Decision { object, .. } => {
                    if *object == key {
                        return true;
                    }
                }
                Activity::Result { result } => {
                    if ObjectKey::Result(result.task.task) == key {
                        return true;
                    }
                }
                Activity::Turn { turn: TurnRead::Undecodable { .. }, .. }
                | Activity::Owner { decoded: Some(ArchivedTurn { turn: TurnRead::Undecodable { .. }, .. }), .. }
                | Activity::Owner { decoded: None, .. }
                | Activity::Words { .. }
                | Activity::RunBegan { .. }
                | Activity::RunEnded { .. }
                | Activity::News { .. } => {}
            }
        }
        for live in &self.live {
            match &live.call {
                Some(LiveCall::Decoded(call)) => {
                    if call.objects.as_slice().contains(&key) {
                        return true;
                    }
                }
                Some(LiveCall::Raw(_)) | None => {}
            }
        }
        false
    }
}
fn turn_shows(turn: &Turn, key: ObjectKey) -> bool {
    for message in &turn.messages {
        for block in &message.blocks {
            match block {
                Block::Call(call) => {
                    if call.objects.as_slice().contains(&key) {
                        return true;
                    }
                }
                Block::Text(_) | Block::Result { .. } | Block::Opaque { .. } => {}
            }
        }
    }
    false
}
