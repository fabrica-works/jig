//! Keep: the child routing shape and exhaustive connector variants.
//! Replace: Ops connector sides, family addresses and application vocabulary.
use crate::{
    Address, Place, infrastructure,
    store::{Key, Record},
};
use alloc::boxed::Box;
use jig_client::contract::Ask as JigAsk;
use skein_lib::{List, Token};
/// One keyed request, tagged with the child that owns its meaning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    Jig(Box<JigAsk>),
    Infrastructure(infrastructure::Ask),
}
/// A decoded terminal tagged with its owning child.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answered {
    Jig(jig_client::contract::Answer),
    Infrastructure(infrastructure::Answer),
}
/// A view act routed exhaustively to its child.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Jig(jig_client_view::action::Act),
    Infrastructure(infrastructure::Act),
    Observability(crate::observability::Act),
}
/// One external fact or act for the root to route.
#[derive(Debug)]
pub enum Event {
    Start { address: Address, kept: List<Record>, offset: jig_client_view::boundary::Offset },
    Went { place: Place },
    Act { action: Action },
    Client { event: jig_client::Event },
    Jig(jig_client_view::Event),
    Infrastructure(infrastructure::Event),
    Observability(crate::observability::Event),
    SignedOut,
}
/// An unkeyed jig shell obligation passed through the root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JigOut {
    Answered { asked: Token, answer: jig_client::contract::Answer },
    Unknown { asked: Token },
}
/// Bounded routing and child facts for the shell to observe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    Jig(jig_client_view::facts::Fact),
    PartsFull,
    InvalidRecord,
    UnknownConnector,
}
/// One shell obligation from the client or a child.
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Save { record: Record },
    Erase { key: Key },
    Client { request: jig_client::Request },
    Jig(JigOut),
    Infrastructure(infrastructure::Out),
    Observability(crate::observability::Out),
    Show { address: Address, replace: bool },
    SignIn { provider: u16, back: Address },
    SignOut,
    Fact { fact: Fact },
}
