//! Concrete test-application adapter; application vocabulary is read only here.
#![allow(clippy::wildcard_imports, reason = "world adapters translate the complete owner vocabulary")]
use crate::harness::*;
use jig_domain_client::{boundary::Offset, contract::*};
use jig_test_domain_client::{self as domain, Address, Event, JigOut, Request, first};
use jig_test_view as view;
use jig_view::{Act, NodeId, Patch, Tree};
use skein_lib::{Duration, Env, List, Queue, RequestEnvelope, RequestLimits, Time};
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};
/// The test application’s client root and view adapter.
pub struct Test;
/// Returns the small focused-world capacities.
#[must_use]
pub fn limits() -> jig_test_domain_client::Limits {
    jig_test_domain_client::Limits {
        jig: jig_domain_client::Limits {
            watches: 8,
            notices: 2,
            notice_life: Duration::from_secs(5),
            linger: Duration::from_secs(5),
            save_delay: Duration::from_millis(10),
            intents: 2,
            fields: 4,
            providers: 2,
            objects: 3,
            sources: 8,
            reads: 8,
            parts: 8,
            delegates: 2,
            document_bytes: 65536,
            records: 2,
            kept: 4,
            drafts: 1,
            projects: 2,
            roles: 2,
            charters: 2,
            text: 32,
            rows: 2,
            conversation_words: 2,
            navigation_entries: 8,
            updates_count: 99,
            backoff: jig_domain_client::limits::Backoff {
                first: Duration::from_millis(10),
                most: Duration::from_millis(80),
            },
            heartbeat: Duration::from_millis(100),
        },
        first: first::Limits {
            connector: jig_test_domain_client::connector_values::ConnectorLimits {
                rows: 4,
                paths: 0,
                segments: 0,
                segment_bytes: 0,
                bytes: 4096,
            },
            parts: 8,
            objects: 8,
            reads: 16,
            watches: 16,
            requests: 4,
            rows: 2,
            text: 32,
            retry: Duration::from_millis(10),
        },
        second_parts: 8,
        table: RequestLimits {
            requests: 4,
            bytes: 65536,
            out: 16,
            first: Duration::from_millis(10),
            most: Duration::from_millis(80),
            margin: Duration::from_secs(1),
        },
    }
}

/// Creates a bounded fixture list with room for all supplied rows.
#[must_use]
pub fn list<T>(rows: impl IntoIterator<Item = T>) -> List<T> {
    let rows: Vec<_> = rows.into_iter().collect();
    let mut out = List::with_capacity(u32::try_from(rows.len()).expect("fixture rows fit"));
    for row in rows {
        assert!(out.push(row).is_ok(), "fixture capacity");
    }
    out
}

// Checked immutable books and envelopes are shared by every equal fixture.
type Books = (jig_view::words::Words, view::words::Words);
fn phrasebooks(limits: &domain::Limits) -> &'static Books {
    static BOOKS: OnceLock<Mutex<BTreeMap<(u32, u32), &'static Books>>> = OnceLock::new();
    let mut books = BOOKS.get_or_init(|| Mutex::new(BTreeMap::new())).lock().expect("fixture books lock");
    books.entry((limits.jig.text, limits.first.text)).or_insert_with(|| {
        Box::leak(Box::new((
            view::connector_values_words::english(limits.jig.text).expect("composed application English fits"),
            view::words::Words::english(limits.first.text).expect("family English fits"),
        )))
    })
}
struct Fixture {
    books: &'static Books,
    bounds: view::Limits,
    worst_case: u64,
}
fn fixture(limits: &domain::Limits) -> &'static Fixture {
    static FIXTURES: OnceLock<Mutex<Vec<(domain::Limits, &'static Fixture)>>> = OnceLock::new();
    let mut fixtures = FIXTURES.get_or_init(|| Mutex::new(Vec::new())).lock().expect("fixture envelopes lock");
    if let Some((_, fixture)) = fixtures.iter().find(|(key, _)| key == limits) {
        return fixture;
    }
    let books = phrasebooks(limits);
    let bounds = view::Limits::for_world(limits, &books.0, &books.1, 1024, 256).expect("view bounds fit");
    let worst_case = domain::worst_case(limits).expect("root worst case")
        // The shell model is bounded by another renderer envelope.
        + 2 * view::worst_case(&bounds, limits, &books.0, &books.1).expect("view worst case");
    let fixture = Box::leak(Box::new(Fixture { books, bounds, worst_case }));
    fixtures.push((*limits, fixture));
    fixture
}
/// Constructs the test application's root and view with one declared bound.
#[must_use]
pub fn new(limits: &domain::Limits, seed: u64) -> (domain::Domain, view::View<'static>) {
    let fixture = fixture(limits);
    let (words, family) = fixture.books;
    let mut root = domain::new(limits, seed);
    domain::provider(&mut root, jig_domain_client::signin::Provider { number: 1, name: crate::stories::text(b"Test") })
        .expect("provider fits");
    domain::provider(
        &mut root,
        jig_domain_client::signin::Provider { number: 2, name: crate::stories::text(b"Other") },
    )
    .expect("second provider fits");
    (root, view::View::new(&fixture.bounds, limits, words, family, first::Address { project: 1, item: 1 }))
}
impl App for Test {
    type Root = domain::Domain;
    type View = view::View<'static>;
    type Limits = domain::Limits;
    type Event = Event;
    type Request = Request;
    type Record = domain::store::Record;
    type Key = domain::store::Key;
    type Address = Address;
    fn limits() -> Self::Limits {
        limits()
    }
    fn new(seed: u64) -> (Self::Root, Self::View) {
        new(&limits(), seed)
    }
    fn start(address: Address, kept: Vec<Self::Record>, offset: i32) -> Event {
        Event::Start { address, kept: list(kept), offset: Offset { minutes: offset } }
    }
    fn went(address: Address) -> Event {
        Event::Went { place: domain::Place::Address(address) }
    }
    fn step(root: &mut Self::Root, env: &Env<Self::Limits>, event: Event, out: &mut Queue<Request>) {
        domain::step(root, env, event, out);
    }
    fn fire(root: &mut Self::Root, env: &Env<Self::Limits>, out: &mut Queue<Request>) {
        domain::fire(root, env, out);
    }
    fn take(root: &mut Self::Root, env: &Env<Self::Limits>, out: &mut Queue<Request>) {
        domain::take(root, env, out);
    }
    fn next_deadline(root: &Self::Root) -> Option<Time> {
        domain::next_deadline(root)
    }
    fn build(view: &mut Self::View, root: &Self::Root, patches: &mut Queue<Patch>) {
        view::build(view, root, patches);
    }
    fn tree(view: &Self::View) -> &Tree {
        view.tree()
    }
    fn act(view: &mut Self::View, node: NodeId, act: Act) -> Option<Event> {
        view::decode(view, node, act).map(|action| Event::Act { action })
    }
    fn duty(request: Request) -> Duty<Self> {
        match request {
            Request::Save { record } => Duty::Store(StoreWrite::Save(record)),
            Request::Erase { key } => Duty::Store(StoreWrite::Erase(key)),
            Request::Send { .. }
            | Request::Jig(
                JigOut::Read { .. } | JigOut::Open { .. } | JigOut::Continue { .. } | JigOut::Close { .. },
            )
            | Request::First(first::Out::Read { .. } | first::Out::Open { .. } | first::Out::Close { .. }) => {
                Duty::Wire(request)
            }
            Request::Show { address, replace } => Duty::Show { address, replace },
            Request::SignIn { provider, back } => Duty::SignIn { provider, back },
            Request::SignOut => Duty::SignOut,
            Request::Jig(JigOut::Answered { .. } | JigOut::Unknown { .. })
            | Request::First(first::Out::Fact { .. })
            | Request::Fact { .. } => Duty::Fact,
        }
    }
    fn record_key(record: &Self::Record) -> Self::Key {
        match record {
            domain::store::Record::Jig(record) => domain::store::Key::Jig(record.key()),
            domain::store::Record::First(first::Record::Place { .. }) => domain::store::Key::First(first::Key::Place),
            domain::store::Record::Table { record, .. } => domain::store::Key::Table(record.key),
        }
    }
    fn unsent(request: Request) -> Option<Event> {
        match request {
            Request::Send { attempt, .. } => {
                Some(Event::Answer { attempt, envelope: RequestEnvelope::Lost, answer: None })
            }
            Request::Jig(JigOut::Read { read, .. }) => {
                Some(Event::Jig(jig_domain_client::Event::Read { read, result: ReadResult::None(ReadNone::Busy) }))
            }
            Request::Jig(JigOut::Open { watch, .. } | JigOut::Continue { watch, .. } | JigOut::Close { watch }) => {
                Some(Event::Jig(jig_domain_client::Event::Ended { watch, end: WatchEnd::Dropped }))
            }
            Request::First(first::Out::Read { read, .. }) => {
                Some(Event::First(first::Event::Read { read, result: first::ReadResult::Busy }))
            }
            Request::First(first::Out::Open { watch, .. } | first::Out::Close { watch }) => {
                Some(Event::First(first::Event::Ended { watch, end: first::WatchEnd::Dropped }))
            }
            Request::Save { .. }
            | Request::Erase { .. }
            | Request::Show { .. }
            | Request::SignOut
            | Request::SignIn { .. }
            | Request::Fact { .. }
            | Request::Jig(JigOut::Answered { .. } | JigOut::Unknown { .. })
            | Request::First(first::Out::Fact { .. }) => None,
        }
    }
    fn max_out(limits: &Self::Limits) -> u32 {
        domain::max_out(limits)
    }
    fn patch_limit(view: &Self::View) -> u32 {
        view.limits().tree.patches
    }
    fn worst_case(limits: &Self::Limits) -> u64 {
        fixture(limits).worst_case
    }
    fn window_changed(event: &Self::Event) -> bool {
        crate::observe::window_changed(event)
    }
    fn seen(tree: &Tree) -> Vec<crate::harness::Seen> {
        crate::observe::seen(tree)
    }
    fn sent(request: &Request) -> Option<Sent> {
        crate::observe::sent(request)
    }
    fn received(event: &Event) -> Vec<Receipt> {
        crate::observe::received(event)
    }
    fn card(tree: &Tree, node: NodeId) -> Option<CardKey> {
        crate::observe::card(tree, node)
    }
}
