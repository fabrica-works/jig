//! Literal cards drawn by the conversation (view.md, 4 and 9).
//! Task-page proposals remain beside the log unless that exact shared key
//! already has a card in a supplied turn, live call or decision.
use jig_client::{
    contract::{Activity, Block, LiveCall, ObjectKey, Turn},
    conversation::Conversation,
};

pub(crate) fn shows_object(state: &Conversation, key: ObjectKey) -> bool {
    for line in &state.lines.rows {
        match &line.event {
            Activity::Turn { turn, .. }
            | Activity::Owner { decoded: Some(jig_client::contract::ArchivedTurn { turn, .. }), .. } => {
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
            Activity::Owner { decoded: None, .. }
            | Activity::Words { .. }
            | Activity::RunBegan { .. }
            | Activity::RunEnded { .. }
            | Activity::News { .. } => {}
        }
    }
    for live in &state.live {
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
