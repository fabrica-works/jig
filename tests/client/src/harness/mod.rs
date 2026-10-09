//! Shell duties and the reusable application/front boundaries (testing.md, 4).
pub mod referee;
mod world;
use jig_client_view::{Act, NodeId, Patch, Tree};
use skein_lib::{Env, Queue, Time, Wall};
use std::fmt::Debug;
pub use world::*;
/// An application root and view. The shell reads only requests and trees.
pub trait App: Sized {
    type Root;
    type View;
    type Limits: Clone;
    type Event;
    type Request;
    type Record: Clone + Debug;
    type Key: Clone + Eq + Debug;
    type Address: Clone + Eq + Debug;
    fn limits() -> Self::Limits;
    fn new(seed: u64) -> (Self::Root, Self::View);
    fn start(address: Self::Address, kept: Vec<Self::Record>, offset: i32) -> Self::Event;
    fn went(address: Self::Address) -> Self::Event;
    fn step(root: &mut Self::Root, env: &Env<Self::Limits>, event: Self::Event, out: &mut Queue<Self::Request>);
    fn fire(root: &mut Self::Root, env: &Env<Self::Limits>, out: &mut Queue<Self::Request>);
    fn take(root: &mut Self::Root, env: &Env<Self::Limits>, out: &mut Queue<Self::Request>);
    fn next_deadline(root: &Self::Root) -> Option<Time>;
    fn build(view: &mut Self::View, root: &Self::Root, patches: &mut Queue<Patch>);
    fn tree(view: &Self::View) -> &Tree;
    fn act(view: &mut Self::View, node: NodeId, act: Act) -> Option<Self::Event>;
    fn duty(request: Self::Request) -> Duty<Self>;
    fn record_key(record: &Self::Record) -> Self::Key;
    fn unsent(request: Self::Request) -> Option<Self::Event>;
    fn max_out(limits: &Self::Limits) -> u32;
    fn patch_limit(view: &Self::View) -> u32;
    fn worst_case(limits: &Self::Limits) -> u64;
    fn seen(tree: &Tree) -> Vec<Seen>;
}
/// One persistent shell record change.
#[derive(Debug)]
pub enum StoreWrite<R, K> {
    Save(R),
    Erase(K),
}
/// The shell obligation represented by a root request.
pub enum Duty<A: App> {
    Store(StoreWrite<A::Record, A::Key>),
    Wire(A::Request),
    Show { address: A::Address, replace: bool },
    SignIn { provider: u16, back: A::Address },
    Fact,
}
/// Engine boundary; token routing and transport epochs belong to the front.
pub trait Front<A: App> {
    fn bind(&mut self, client: ClientId, party: u64);
    fn wire(&mut self, client: ClientId, now: Time, wall: Wall, request: A::Request);
    fn poll(&mut self, now: Time, wall: Wall) -> Vec<(ClientId, A::Event)>;
    fn next_due(&self) -> Option<Time>;
    fn observed(&mut self) -> Vec<Observed>;
    fn fault(&mut self, now: Time, fault: FrontFault);
    fn reload(&mut self, client: ClientId, now: Time);
    fn received(&mut self, client: ClientId, event: &A::Event);
    fn sign_in(&mut self, client: ClientId, now: Time, wall: Wall);
}
/// A card’s externally visible progress or departure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardSeen {
    Waiting,
    Deciding,
    Done { commit: u64 },
    Left { reason: bool },
}
/// Facts read from a visible accessibility tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Seen {
    Card { key: u64, state: CardSeen },
    Words { client: ClientId, words: Vec<u8> },
    Link { live: bool },
}
/// Durable truth reported by the engine boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Observed {
    Durable { commit: u64, at: Time },
    Made { party: u64, key: [u8; 16], commit: u64 },
    Kept { task: u64, message: u64 },
}
/// A fault the engine boundary can inject.
#[derive(Clone, Debug)]
pub enum FrontFault {
    Busy,
    NotReady,
    Restart { offline: skein_lib::Duration },
    Slow { span: skein_lib::Duration },
    DropWatch,
    NoAnswer,
    SignedOut,
    Versions,
}
