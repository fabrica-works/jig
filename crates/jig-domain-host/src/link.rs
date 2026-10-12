//! The engine link's vocabulary (domain/hosts.md, sections 2 and 6.1).
use crate::{Answer, Assignment, Grant, Hosting, Turn};
use alloc::boxed::Box;
use skein_lib::{Duration, ReplyTo, Token};

#[derive(PartialEq, Eq, Debug)]
pub enum LinkDown {
    Assign { reply_to: ReplyTo, assignment: Assignment },
    Message { run: Token, attempt: Token, name: Token, sender: Box<[u8]>, words: Box<[u8]> },
    Relayed { run: Token, attempt: Token, call: Token, answer: Relayed },
    TurnKept { run: Token, attempt: Token, turn: u32 },
    Acknowledge { run: Token, attempt: Token },
    Cancel { run: Token, attempt: Token },
    Grant { run: Token, attempt: Token, grant: Grant },
}

#[derive(PartialEq, Eq, Debug)]
pub enum LinkUp {
    Turn {
        run: Token,
        attempt: Token,
        turn: Turn,
    },
    Call {
        run: Token,
        attempt: Token,
        call: Box<[u8]>,
        delivery: Token,
        tool: Box<[u8]>,
        writes: bool,
        input: Box<[u8]>,
        deadline: Duration,
    },
    Answer {
        reply_to: ReplyTo,
        run: Token,
        attempt: Token,
        answer: Answer,
    },
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Relayed {
    Answer { error: bool, text: Box<[u8]> },
    Busy,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ModelBound {
    pub name: Box<[u8]>,
    pub window: u32,
    pub output: u32,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Bounds {
    pub models: Box<[ModelBound]>,
    pub turns: u32,
    pub spend: u64,
    pub time: Duration,
    pub waiting: Duration,
    pub checks: Duration,
    pub message_bytes: u32,
    pub inbox: u32,
    pub unacknowledged: u32,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Hello {
    pub slots: u32,
    pub stop_bound: Duration,
    pub workstreams: Box<[u64]>,
    pub hosting: Box<[Hosting]>,
    pub bounds: Bounds,
}
