//! One malformed archive among committed turns, decoded by the joint translation.
use super::{chat_ready, text};
use crate::{Scripted, Test, World, joint::wire::activity, test::list};
use jig_domain_client::contract::*;
use skein_lib::Wall;

fn readable(number: u32, words: &[u8]) -> ActivityLine {
    ActivityLine {
        position: u64::from(number),
        at: Wall::EPOCH,
        event: Activity::Turn {
            name: TurnName { attempt: 1, number },
            turn: TurnRead::Decoded(Turn {
                messages: list([Message { speaker: Speaker::Assistant, blocks: list([Block::Text(text(words))]) }]),
                used: Usage { input: 1, output: 1, cached: 0, cache_written: 0 },
                spent: 1,
                read_through: None,
                cut: false,
            }),
        },
    }
}
/// Starts a conversation with a malformed turn between two readable ones.
#[must_use]
pub fn world(seed: u64) -> World<Test, Scripted> {
    let mut world = chat_ready(seed);
    let limits = limits().jig;
    let source = jig_domain_tasks::ActivityLine {
        task: 9,
        position: 2,
        at: Wall::EPOCH,
        line: jig_domain_tasks::Line::Turn { reply: 2, attempt: 1, turn: 2, read: None },
    };
    let source = activity::line(9, &source, &limits).expect("source line");
    let archive = jig_domain_core::TurnRecord {
        opens_window: false,
        task: 9,
        attempt: 1,
        turn: 2,
        spent: 2,
        read: None,
        at: Wall::EPOCH,
        transcript: Box::from([0_u8, 3, 255]),
    };
    let codec = jig_protocol_transcript::Limits {
        smith: smith_transcript::v3::CEILINGS,
        bytes: 128,
        turn: TurnLimits { messages: 2, blocks: 2, text: 32 },
    };
    let unread = activity::with_archive(&source, &archive, &codec, &limits).expect("undecodable turn retained");
    world
        .front()
        .engine
        .act(
            skein_lib::Time::ZERO,
            Wall::EPOCH,
            jig_scripted_engine::Change::Source(jig_scripted_engine::Source {
                party: 7,
                read: Read::Activity { task: 9, before: None },
                rows: jig_scripted_engine::Rows::Activity(vec![
                    readable(3, b"After the malformed turn"),
                    unread,
                    readable(1, b"Before the malformed turn"),
                ]),
            }),
        )
        .expect("whole conversation fits");
    world
}

/// Three rows keep both neighbouring turns in the same bounded window.
#[must_use]
pub fn limits() -> jig_test_domain_client::Limits {
    let mut limits = crate::test::limits();
    limits.jig.rows = 3;
    limits
}
/// Builds the root and view with the story's receiving bounds.
#[must_use]
pub fn client(seed: u64) -> (jig_test_domain_client::Domain, jig_test_view::View<'static>) {
    crate::test::new(&limits(), seed)
}
