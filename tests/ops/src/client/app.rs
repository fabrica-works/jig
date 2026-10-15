//! Ops root/view shell adapter; observations come from public requests and trees.
#![allow(clippy::wildcard_imports, reason = "the adapter translates complete owner vocabularies")]
use jig_client_world::harness::Seen;
use jig_client_world::{harness::*, observe};
use jig_domain_client::contract::Outcome;
use jig_domain_client::{boundary::Offset, contract::*};
use jig_ops_domain_client::{
    self as domain, Action, Address, Event, JigOut, Request, infrastructure as infra, observability as obs,
};
use jig_ops_domain_connectors::{Connector, number};
use jig_ops_view as view;
use jig_view::Owner;
use skein_ui::Act;
use skein_ui::{NodeId, Patch, Role, Tree};

use skein_lib::{Duration, Env, Queue, RequestEnvelope, RequestLimits, Time};

use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};

/// Reference Ops client composed over the common shell world.
pub struct Ops;
/// Small explicit story capacities, also used to admit scripted whole documents.
#[must_use]
pub fn limits() -> domain::Limits {
    domain::Limits {
        jig: jig_domain_client::Limits {
            watches: 8,
            notices: 2,
            notice_life: Duration::from_secs(5),
            // A finite fixture keeps the same before/after-linger behavior at a short scale.
            linger: Duration::from_millis(250),
            save_delay: Duration::from_millis(10),
            intents: 2,
            fields: 4,
            providers: 1,
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
        observability: obs::Limits {
            parts: 8,
            objects: 12,
            reads: 16,
            watches: 16,
            rows: 2,
            text: 32,
            retry: Duration::from_millis(10),
            document_bytes: 65536,
        },
        infrastructure: infra::Limits {
            parts: 8,
            objects: 12,
            reads: 16,
            watches: 16,
            requests: 4,
            rows: 2,
            text: 32,
            retry: Duration::from_millis(10),
            document_bytes: 65536,
        },
        table: RequestLimits {
            requests: 4,
            bytes: 262_144,
            out: 16,
            first: Duration::from_millis(10),
            most: Duration::from_millis(80),
            margin: Duration::from_secs(1),
        },
    }
}
type Books = (jig_view::words::Words, view::words::Words, view::words::Words);
type BookCache = Mutex<BTreeMap<(u32, u32, u32), Books>>;
fn phrasebooks(limits: &domain::Limits) -> Books {
    static BOOKS: OnceLock<BookCache> = OnceLock::new();
    let mut books = BOOKS.get_or_init(|| Mutex::new(BTreeMap::new())).lock().expect("fixture books lock");
    books
        .entry((limits.jig.text, limits.observability.text, limits.infrastructure.text))
        .or_insert_with(|| {
            (
                jig_view::words::Words {
                    book: jig_view::phrases::english::english(limits.jig.text).expect("English fits"),
                    language: 0,
                    offset_minutes: 0,
                },
                view::words::Words::english(number(Connector::Observability), limits.observability.text)
                    .expect("incident English fits"),
                view::words::Words::english(number(Connector::Infrastructure), limits.infrastructure.text)
                    .expect("environment English fits"),
            )
        })
        .clone()
}
// Story storage is declared independently of the shipped composition envelope.
// Builder checks catch a fixture growing beyond these bounds.
fn view_bounds(
    limits: &domain::Limits,
    jig: &jig_view::words::Words,
    obs: &view::words::Words,
    infra: &view::words::Words,
) -> view::Limits {
    let mut bounds = view::Limits::of(limits, jig, obs, infra).expect("composed view fits");
    assert!(bounds.tree.nodes >= 1024, "world profile fits the composition envelope");
    bounds.tree.nodes = 1024;
    bounds.tree.bindings = 1024;
    bounds.tree.slots = 256;
    bounds.tree.patches = 5 * bounds.tree.nodes;
    bounds.tree.text_bytes = bounds.tree.nodes.checked_mul(bounds.tree.text_per_node).expect("world text fits");
    view::worst_case(&bounds, limits, jig, obs, infra).expect("world profile fits");
    bounds
}
// The ordinary world uses one immutable limits/English profile. Keep all
// composition and heap checks in its initializer before admitting clients.
fn fixture_profile() -> (view::Limits, u64) {
    static PROFILE: OnceLock<(view::Limits, u64)> = OnceLock::new();
    *PROFILE.get_or_init(|| {
        let limits = limits();
        let (jig, obs, infra) = phrasebooks(&limits);
        let bounds = view_bounds(&limits, &jig, &obs, &infra);
        let heap = domain::worst_case(&limits).expect("root bound")
            + 2 * view::worst_case(&bounds, &limits, &jig, &obs, &infra).expect("shell model projection bound");
        (bounds, heap)
    })
}
impl App for Ops {
    type Root = domain::Domain;
    type View = view::View;
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
        let limits = limits();
        let (jig, obs, infra) = phrasebooks(&limits);
        let bounds = fixture_profile().0;
        let mut root = domain::new(&limits, seed);
        domain::configure_arrangement(&mut root, view::arrangement()).expect("ops arrangement before start");
        domain::provider(
            &mut root,
            jig_domain_client::signin::Provider { number: 1, name: jig_client_world::stories::text(b"Ops") },
        )
        .expect("provider fits");
        (root, view::View::new(&bounds, &limits, jig, obs, infra, 1))
    }
    fn start(address: Address, kept: Vec<Self::Record>, offset: i32) -> Event {
        Event::Start { address, kept: super::stories::list(kept), offset: Offset { minutes: offset } }
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
            Request::Show { address, replace } => Duty::Show { address, replace },
            Request::SignIn { provider, back } => Duty::SignIn { provider, back },
            Request::SignOut => Duty::SignOut,
            Request::Send { .. }
            | Request::Jig(
                JigOut::Read { .. } | JigOut::Open { .. } | JigOut::Continue { .. } | JigOut::Close { .. },
            )
            | Request::Observability(obs::Out::Read { .. } | obs::Out::Open { .. } | obs::Out::Close { .. })
            | Request::Infrastructure(infra::Out::Read { .. } | infra::Out::Open { .. } | infra::Out::Close { .. }) => {
                Duty::Wire(request)
            }
            Request::Fact { .. }
            | Request::Jig(JigOut::Answered { .. } | JigOut::Unknown { .. })
            | Request::Observability(obs::Out::Fact { .. })
            | Request::Infrastructure(infra::Out::Fact { .. }) => Duty::Fact,
        }
    }
    fn record_key(record: &Self::Record) -> Self::Key {
        match record {
            domain::store::Record::Jig(record) => domain::store::Key::Jig(record.key()),
            domain::store::Record::Observability(obs::Record::Place { .. }) => {
                domain::store::Key::Observability(obs::Key::Place)
            }
            domain::store::Record::Infrastructure(infra::Record::Place { .. }) => {
                domain::store::Key::Infrastructure(infra::Key::Place)
            }
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
            Request::Observability(obs::Out::Read { read, .. }) => {
                Some(Event::Observability(obs::Event::Read { read, result: obs::ReadResult::Busy }))
            }
            Request::Observability(obs::Out::Open { watch, .. } | obs::Out::Close { watch }) => {
                Some(Event::Observability(obs::Event::Ended { watch, end: obs::WatchEnd::Dropped }))
            }
            Request::Infrastructure(infra::Out::Read { read, .. }) => {
                Some(Event::Infrastructure(infra::Event::Read { read, result: infra::ReadResult::Busy }))
            }
            Request::Infrastructure(infra::Out::Open { watch, .. } | infra::Out::Close { watch }) => {
                Some(Event::Infrastructure(infra::Event::Ended { watch, end: infra::WatchEnd::Dropped }))
            }
            Request::Save { .. }
            | Request::Erase { .. }
            | Request::Show { .. }
            | Request::SignOut
            | Request::SignIn { .. }
            | Request::Fact { .. }
            | Request::Jig(JigOut::Answered { .. } | JigOut::Unknown { .. })
            | Request::Observability(obs::Out::Fact { .. })
            | Request::Infrastructure(infra::Out::Fact { .. }) => None,
        }
    }
    fn max_out(limits: &Self::Limits) -> u32 {
        domain::max_out(limits)
    }
    fn view_limits(view: &Self::View) -> skein_ui::Limits {
        view.limits().tree
    }
    fn patch_limit(view: &Self::View) -> u32 {
        view.limits().tree.patches
    }
    fn worst_case(limits: &Self::Limits) -> u64 {
        if *limits == self::limits() {
            return fixture_profile().1;
        }
        let (jig, obs, infra) = phrasebooks(limits);
        let bounds = view_bounds(limits, &jig, &obs, &infra);
        domain::worst_case(limits).expect("root bound")
            + 2 * view::worst_case(&bounds, limits, &jig, &obs, &infra).expect("shell model projection bound")
    }
    fn window_changed(event: &Event) -> bool {
        match event {
            Event::Act { action: Action::Jig(act) } | Event::Jig(jig_domain_client::Event::Act { act }) => {
                observe::jig_window_changed(act)
            }
            Event::Act {
                action: Action::Observability(obs::Act::Go { .. }) | Action::Infrastructure(infra::Act::Go { .. }),
            } => true,
            Event::Start { .. }
            | Event::Went { .. }
            | Event::Act { .. }
            | Event::Answer { .. }
            | Event::Jig(_)
            | Event::Observability(_)
            | Event::Infrastructure(_)
            | Event::SignedOut => false,
        }
    }
    fn seen(tree: &Tree) -> Vec<Seen> {
        super::stories::seen(tree)
    }
    fn sent(request: &Request) -> Option<Sent> {
        let Request::Send { attempt, key, ask } = request else { return None };
        Some(match ask {
            domain::Ask::Jig(ask) => observe::jig_sent(*attempt, *key, ask),
            domain::Ask::Infrastructure(infra::Ask::NativeMoveEnd { object, revision, .. }) => Sent {
                attempt: *attempt,
                key: *key,
                card: Some(CardKey::of(skein_ui::Key {
                    owner: (jig_view::Owner::Application { number: 2 }).opaque(),
                    kind: 12,
                    a: 0,
                    b: object.raw(),
                })),
                revision: Some((0, revision.value)),
                words: None,
            },
            domain::Ask::Infrastructure(infra::Ask::MoveEnd { project, environment, revision, .. }) => Sent {
                attempt: *attempt,
                key: *key,
                card: Some(environment_key(*project, *environment)),
                revision: Some((0, revision.value)),
                words: None,
            },
        })
    }
    fn received(event: &Event) -> Vec<Receipt> {
        match event {
            Event::Jig(event) => observe::jig_received(event),
            Event::Answer { attempt, envelope, answer } => {
                if *envelope == RequestEnvelope::SignedOut {
                    vec![Receipt::Parked]
                } else {
                    let words = match answer {
                        Some(domain::Answered::Jig(Answer::Done(Outcome::Made {
                            what: Made::Words { task, message },
                        }))) => Some((*task, *message)),
                        Some(domain::Answered::Jig(_) | domain::Answered::Infrastructure(_)) | None => None,
                    };
                    vec![Receipt::Answer {
                        attempt: *attempt,
                        final_answer: *envelope == RequestEnvelope::Final,
                        words,
                    }]
                }
            }
            Event::Infrastructure(
                infra::Event::Read { result: infra::ReadResult::Value(doc), .. }
                | infra::Event::Watch { event: infra::WatchEvent::Delivered(doc), .. },
            ) => {
                if let infra::Document::Environment(env) = doc.as_ref()
                    && let Some(end) = env.end
                {
                    vec![Receipt::Card {
                        key: environment_key(env.project, env.number),
                        revisions: vec![(0, end.revision.value)],
                    }]
                } else {
                    vec![]
                }
            }
            Event::SignedOut => vec![Receipt::Parked],
            Event::Start { .. }
            | Event::Went { .. }
            | Event::Act { .. }
            | Event::Observability(_)
            | Event::Infrastructure(_) => vec![],
        }
    }
    fn card(tree: &Tree, node: NodeId) -> Option<CardKey> {
        let at = tree.nodes().iter().position(|n| n.id == node)?;
        let family = tree
            .nodes()
            .iter()
            .enumerate()
            .take(at + 1)
            .filter(|(i, n)| *i + usize::try_from(n.size).expect("subtree") > at)
            .filter_map(|(_, n)| {
                n.key.filter(|k| {
                    matches!(Owner::from_opaque(k.owner), Some(Owner::Application { number: connector }) if connector == number(Connector::Infrastructure))
                        && matches!(k.kind, 2 | 12)
                        && n.role == Role::Item
                })
            })
            .next_back();
        family.map(CardKey::of).or_else(|| observe::jig_card(tree, node))
    }
}
#[must_use]
pub(super) fn environment_key(project: u32, environment: u64) -> CardKey {
    CardKey::of(
        view::infrastructure::key(&infra::Subject {
            scope: infra::Scope::Project(project),
            kind: infra::Kind::Environment(environment),
        })
        .expect("legacy numeric environment key"),
    )
}
