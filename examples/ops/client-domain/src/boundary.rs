//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides, family addresses and application vocabulary.
use crate::{
    Address, Place, infrastructure,
    store::{Key, Record},
};
use alloc::boxed::Box;
use jig_client_domain::contract::{Ask as JigAsk, Read, Subject};
use skein_lib::{List, RequestEnvelope, RequestKey, Token};
/// One keyed request, tagged with the child that owns its meaning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    Jig(Box<JigAsk>),
    Infrastructure(infrastructure::Ask),
}
/// A decoded terminal tagged with its owning child.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answered {
    Jig(jig_client_domain::contract::Answer),
    Infrastructure(infrastructure::Answer),
}
/// A view act routed exhaustively to its child.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Jig(jig_domain_client::action::Act),
    Infrastructure(infrastructure::Act),
    Observability(crate::observability::Act),
}
/// One external fact or act for the root to route.
#[derive(Debug)]
pub enum Event {
    Start { address: Address, kept: List<Record>, offset: jig_domain_client::boundary::Offset },
    Went { place: Place },
    Act { action: Action },
    Answer { attempt: Token, envelope: RequestEnvelope, answer: Option<Answered> },
    Jig(jig_domain_client::Event),
    Infrastructure(infrastructure::Event),
    Observability(crate::observability::Event),
    SignedOut,
}
/// An unkeyed jig shell obligation passed through the root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JigOut {
    Read { read: Token, what: Read },
    Open { watch: Token, subject: Subject },
    Continue { watch: Token, subject: Subject },
    Close { watch: Token },
    Answered { asked: Token, answer: jig_client_domain::contract::Answer },
    Unknown { asked: Token },
}
/// Bounded routing and child facts for the shell to observe.
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
/// One shell obligation; keyed sends come only from the table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Save { record: Record },
    Erase { key: Key },
    Send { attempt: Token, key: RequestKey, ask: Ask },
    Jig(JigOut),
    Infrastructure(infrastructure::Out),
    Observability(crate::observability::Out),
    Show { address: Address, replace: bool },
    SignIn { provider: u16, back: Address },
    SignOut,
    Fact { fact: Fact },
}
