//! Own the root's turn and answer metadata (hosts.md, 6.4).
use crate::{Domain, Work};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_tasks as tasks;
use skein_lib::{Queue, Token};
#[derive(Debug)]
pub(super) enum Payload {
    Busy,
    Turn {
        task: u64,
        attempt: u64,
        turn: u32,
        cumulative: u64,
        read: Option<u64>,
        transcript: Box<[u8]>,
        opens_window: bool,
        refused: Box<[u64]>,
    },
    Answer {
        task: u64,
        attempt: u64,
        cumulative: u64,
        end: crate::link::Ending,
        read: Option<u64>,
        refused: Box<[u64]>,
    },
    SettledCall(core::SettledCall),
    InboxWord(tasks::Word),
}
pub(super) fn receive(domain: &mut Domain, work: &mut Queue<Work>, channel: Token, value: Payload) {
    let (run, attempt, turn) = match &value {
        Payload::Turn { task, attempt, turn, .. } => (*task, *attempt, Some(*turn)),
        Payload::Answer { task, attempt, .. } => (*task, *attempt, None),
        Payload::Busy | Payload::SettledCall(_) | Payload::InboxWord(_) => unreachable!("turn or answer metadata"),
    };
    let name = crate::payload(domain, value);
    let run = Token::new(run);
    let attempt = Token::new(attempt);
    let event = match turn {
        Some(turn) => fleet::Event::Turn { channel, run, attempt, turn, body: name },
        None => fleet::Event::Answer { channel, run, attempt, answer: fleet::Answer::Ended, payload: name },
    };
    work.push(Work::Core(core::Event::Fleet(event)));
}
