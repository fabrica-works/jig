//! Wrapped vocabulary: protocol decodes answers; routing only unwraps owners.
use crate::{
    Address, Place, first,
    store::{Key, Record},
};
use alloc::boxed::Box;
use jig_client::contract::Ask as JigAsk;
use skein_lib::List;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    Jig(Box<JigAsk>),
    First(first::Ask),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answered {
    Jig(jig_client::contract::Answer),
    First(first::Answer),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    SignIn,
    Jig(jig_client_view::action::Act),
    First(first::Act),
}
#[derive(Debug)]
pub enum Event {
    Start { address: Address, kept: List<Record>, offset: jig_client_view::boundary::Offset },
    Went { place: Place },
    Act { action: Action },
    Client { event: jig_client::Event },
    Jig(jig_client_view::Event),
    First(first::Event),
    SignedOut,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    PartsFull,
    InvalidRecord,
    UnknownConnector,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Save { record: Record },
    Erase { key: Key },
    Client { request: jig_client::Request },
    First(first::Out),
    Show { address: Address, replace: bool },
    SignIn { provider: u16 },
    SignOut,
    Fact { fact: Fact },
}
