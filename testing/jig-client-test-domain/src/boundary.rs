//! Wrapped vocabulary: protocol decodes answers; routing only unwraps owners.
use crate::{
    Address, Place, first,
    store::{Key, Record},
};
use alloc::boxed::Box;
use jig_client_domain::contract::{Ask as JigAsk, Read, Subject};
use skein_lib::{List, RequestEnvelope, RequestKey, Token};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    Jig(Box<JigAsk>),
    First(first::Ask),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answered {
    Jig(jig_client_domain::contract::Answer),
    First(first::Answer),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Jig(jig_domain_client::action::Act),
    First(first::Act),
}
#[derive(Debug)]
pub enum Event {
    Start { address: Address, kept: List<Record>, offset: jig_domain_client::boundary::Offset },
    Went { place: Place },
    Act { action: Action },
    Answer { attempt: Token, envelope: RequestEnvelope, answer: Option<Answered> },
    Jig(jig_domain_client::Event),
    First(first::Event),
    SignedOut,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JigOut {
    Read { read: Token, what: Read },
    Open { watch: Token, subject: Subject },
    Continue { watch: Token, subject: Subject },
    Close { watch: Token },
    Answered { asked: Token, answer: jig_client_domain::contract::Answer },
    Unknown { asked: Token },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    Jig(jig_domain_client::facts::Fact),
    TableFull,
    PartsFull,
    InvalidRecord,
    InvalidAnswer,
    UnknownConnector,
    KeptAnswered,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Save { record: Record },
    Erase { key: Key },
    Send { attempt: Token, key: RequestKey, ask: Ask },
    Jig(JigOut),
    First(first::Out),
    Show { address: Address, replace: bool },
    SignIn { provider: u16, back: Address },
    SignOut,
    Fact { fact: Fact },
}
