#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary tests collect text, query trees and select scripted requests"
)]

use crate::{
    Act, Builder, JigBindings, JigMeaning, Owner, Role, SlotRef, Tree, decode,
    phrases::{
        Hole, PhraseLimits, Phrasebook, english,
        jig::{JigPhrase, shapes},
    },
    words::Words,
};
use jig_client::{
    Domain, Event, Request,
    action::Act as Action,
    address::Address,
    boundary::Offset,
    contract::*,
    intents::Progress,
    limits::Backoff,
    objects::{Card, Left, Source},
};
use skein_lib::{Duration, Env, List, Queue, Time, Token, Wall};

const CLIENT: jig_client::Limits = jig_client::Limits {
    watches: 8,
    notices: 2,
    notice_life: Duration::from_secs(5),
    linger: Duration::from_secs(5),
    save_delay: Duration::from_secs(1),
    intents: 2,
    fields: 8,
    providers: 2,
    objects: 3,
    sources: 8,
    reads: 8,
    parts: 8,
    delegates: 2,
    document_bytes: 65536,
    records: 2,
    drafts: 1,
    projects: 2,
    roles: 2,
    charters: 2,
    text: 128,
    rows: 2,
    conversation_words: 2,
    navigation_entries: 8,
    updates_count: 99,
    backoff: Backoff { first: Duration::from_nanos(10), most: Duration::from_nanos(80) },
    heartbeat: Duration::from_secs(100),
};
fn limits() -> crate::Limits {
    crate::Limits {
        nodes: 256,
        depth: 10,
        text_per_node: 1024,
        text_bytes: 16384,
        drawing_bytes: 0,
        styles_per_node: 2,
        patches: 1536,
        bindings: 64,
        slots: 8,
        markdown_depth: 3,
    }
}
fn words() -> Words {
    Words { book: english::english(CLIENT.text).unwrap(), language: 0, offset_minutes: 60 }
}
fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), CLIENT.text).unwrap()
}
fn drive(d: &mut Domain, event: Event) -> Vec<Request> {
    let mut q = Queue::with_capacity(jig_client::max_out(&CLIENT));
    jig_client::step(d, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: CLIENT }, event, &mut q);
    let mut out = Vec::new();
    while let Some(r) = q.pop() {
        out.push(r);
    }
    out
}
fn document() -> Frame {
    let mut projects = List::with_capacity(2);
    projects
        .push(ProjectRole {
            project: 4,
            name: text(b"One"),
            roles: List::with_capacity(0),
            chats: List::with_capacity(0),
            goals: List::with_capacity(0),
            pool: Budget { total: 0, own: 0, ended: 0, reserved: 0, left: 0, funder: Actor::Party(1) },
            chat_budget: 0,
        })
        .unwrap();
    Frame {
        party: Party { number: 1, kind: PartyKind::Person, name: text(b"Pat") },
        projects,
        needs: Needs { questions: 1, proposals: 2, escalations: 3, person_tasks: 4 },
        updates: Count { value: 120, limited: false },
        versions: VersionRange { first: 1, last: 1 },
        retention: Duration::from_secs(1000),
        deployment_role: Some(1),
    }
}
fn start() -> (Domain, Token) {
    let mut d = Domain::new(&CLIENT, 1);
    let out = drive(
        &mut d,
        Event::Start {
            address: Address::Inbox { project: None },
            kept: List::with_capacity(0),
            offset: Offset { minutes: 60 },
        },
    );
    let token = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, .. } => Some(*watch),
            _ => None,
        })
        .unwrap();
    (d, token)
}
fn flat(tree: &Tree) -> Vec<u8> {
    tree.nodes().iter().flat_map(|n| n.text.as_slice().iter().flat_map(|r| r.text.iter().copied())).collect()
}
fn contains(tree: &Tree, text: &[u8]) -> bool {
    flat(tree).windows(text.len()).any(|w| w == text)
}

#[test]
fn frame_builds_statuses_counts_navigation_and_number_submission_in_every_link_state() {
    let (mut d, watch) = start();
    let words = words();
    let mut bindings = JigBindings::new(64, CLIENT.text);
    for expected in [b"Connecting".as_slice(), b"Live", b"Offline"] {
        if expected == b"Live" {
            drive(&mut d, Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Frame(Box::new(document()))) });
        }
        if expected == b"Offline" {
            drive(&mut d, Event::Ended { watch, end: WatchEnd::Dropped });
        }
        bindings.begin();
        let mut b = Builder::new(&limits());
        b.open(Role::Page);
        crate::frame::frame(
            &mut b,
            &d,
            SlotRef { owner: Owner::Other { child: 1 }, token: Token::new(1) },
            &words,
            &mut bindings,
        );
        b.close();
        let tree = b.finish();
        assert!(tree.nodes().iter().any(|n| n.role == Role::Status && n.name.as_deref() == Some(expected)));
        if expected != b"Connecting" {
            assert!(contains(&tree, b"10 need you"));
            assert!(contains(&tree, b"More than 99 updates not read"));
            assert!(contains(&tree, b"Signed in as Pat"));
        }
        let field = tree.nodes().iter().find(|n| n.role == Role::TextField).unwrap().binding.unwrap();
        assert_eq!(decode(&mut bindings, field.token, Act::Input { value: b"123".as_slice().into() }), None);
        assert_eq!(decode(&mut bindings, field.token, Act::Submit), Some(Action::Go { address: Address::task(123) }));
        assert_eq!(decode(&mut bindings, field.token, Act::Input { value: b"x".as_slice().into() }), None);
        assert_eq!(decode(&mut bindings, field.token, Act::Submit), None);
    }
}

#[test]
fn shared_card_frame_says_each_progress_refusal_and_departure_with_context_only_by_place() {
    let (mut d, frame_watch) = start();
    let source = Source::Window { generation: 0, number: 1 };
    // The public object collection is read-only in Domain; build states through its shared entry point.
    let mut objects = jig_client::objects::Objects::new(2, 8);
    let id = objects.name(ObjectKey::Task(7), source).unwrap();
    let words = words();
    let mut bindings = JigBindings::new(64, 128);
    let states = [
        Card::Open,
        Card::Deciding { request: None, progress: Progress::Confirming },
        Card::Deciding { request: Some(Token::new(1)), progress: Progress::InFlight },
        Card::Deciding { request: Some(Token::new(1)), progress: Progress::Retrying },
        Card::Deciding { request: Some(Token::new(1)), progress: Progress::Parked },
        Card::Deciding { request: Some(Token::new(1)), progress: Progress::Unknown },
        Card::Refused { refusal: Refusal::Authority { lacking: Lacking::Budget { needed: 40, left: 12 } } },
        Card::Unknown { act: ActKind::Cancel, look: Read::Task { task: 7 } },
        Card::Leaving {
            how: Left::Answered {
                outcome: Outcome::DecidedFirst {
                    by: Party { number: 2, kind: PartyKind::Person, name: text(b"Lee") },
                    how: Decision::Rejected { reason: text(b"already decided") },
                    at: Wall::EPOCH,
                },
            },
            until: Time::ZERO,
        },
        Card::Leaving {
            how: Left::Ended { how: Ending::Cancelled { by: Actor::Party(3), reason: text(b"no longer needed") } },
            until: Time::ZERO,
        },
        Card::Leaving { how: Left::Looking, until: Time::ZERO },
        Card::Leaving { how: Left::Unknown, until: Time::ZERO },
    ];
    for state in states {
        let mut b = Builder::new(&limits());
        b.open(Role::Page);
        crate::cards::state(&mut b, &state, id, &words, &mut bindings);
        b.close();
        let tree = b.finish();
        assert!(tree.nodes().iter().any(|n| n.role == Role::Status));
        if let Card::Refused { .. } = state {
            assert!(contains(&tree, b"Your pool has 12 left; this needs 40"));
        }
        if let Card::Unknown { .. } = state {
            assert!(
                tree.nodes()
                    .iter()
                    .any(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Inspect current object".as_slice()))
            );
        }
    }
    card_context(&mut d, frame_watch, &words, &mut bindings);
    // Refused by the domain after its handle has left; a retained tree token still decodes the handle.
    let binding = bindings.insert(JigMeaning::Action(Action::LookAgain { object: id.token() }));
    let action = decode(&mut bindings, binding.token, Act::Press).unwrap();
    drive(&mut d, Event::Act { act: action });
}

fn card_context(d: &mut Domain, frame_watch: Token, words: &Words, bindings: &mut JigBindings) {
    let out = drive(
        d,
        Event::Watch { watch: frame_watch, event: WatchEvent::Delivered(Delivery::Frame(Box::new(document()))) },
    );
    let inbox_watch = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Inbox { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let mut entries = List::with_capacity(1);
    entries
        .push(InboxEntry {
            reply: None,
            lineage: List::with_capacity(0),
            goal: None,
            recipient: None,
            stall: None,
            object: ObjectKey::Question { task: 7, message: 1 },
            project: 4,
            group: InboxGroup::Questions,
            position: 1,
            at: Wall::EPOCH,
        })
        .unwrap();
    drive(
        d,
        Event::Watch {
            watch: inbox_watch,
            event: WatchEvent::Delivered(Delivery::Inbox(Box::new(Inbox {
                entries,
                needs: document().needs,
                updates: Count { value: 0, limited: false },
                results_seen: 0,
                replies_seen: List::with_capacity(0),
                more: false,
            }))),
        },
    );
    let shared = d.objects().find(ObjectKey::Question { task: 7, message: 1 }).unwrap();
    let mut lineage = List::with_capacity(1);
    lineage.push(6).unwrap();
    for place in [
        crate::cards::Place::Inbox { lineage },
        crate::cards::Place::Conversation,
        crate::cards::Place::TaskPage,
        crate::cards::Place::Elsewhere,
    ] {
        let mut b = Builder::new(&limits());
        b.open(Role::Page);
        assert!(crate::cards::open(&mut b, d, shared, &place, b"Question", words, bindings));
        crate::cards::close(&mut b);
        b.close();
        let tree = b.finish();
        assert!(
            tree.nodes().iter().any(|n| n.role == Role::Region && n.name.as_deref() == Some(b"Question".as_slice()))
        );
        assert!(tree.nodes().iter().any(|n| n.role == Role::Item && n.key.is_some()));
        assert_eq!(
            contains(&tree, b"In project 4"),
            match place {
                crate::cards::Place::Inbox { .. } | crate::cards::Place::Elsewhere => true,
                crate::cards::Place::Conversation | crate::cards::Place::TaskPage => false,
            }
        );
        assert_eq!(
            contains(&tree, b"Task 6"),
            match place {
                crate::cards::Place::Inbox { .. } => true,
                crate::cards::Place::Conversation | crate::cards::Place::TaskPage | crate::cards::Place::Elsewhere =>
                    false,
            }
        );
    }
}

#[test]
fn every_binding_kind_decodes_and_old_tokens_are_retired() {
    let mut bindings = JigBindings::new(64, 4);
    let object = Token::new(7);
    let actions = [
        Action::Go { address: Address::task(1) },
        Action::Submit { field: jig_client::fields::Purpose::Answer },
        Action::Intend { object, act: ActKind::Cancel },
        Action::Choose { option: 2 },
        Action::Confirm,
        Action::Dismiss,
        Action::Fold { object, open: true },
        Action::Filter { group: InboxGroup::Questions, project: None },
        Action::More { window: object, end: jig_client::action::End::Before },
        Action::SignIn { provider: 1 },
    ];
    for action in actions {
        let token = bindings.insert(JigMeaning::Action(action.clone())).token;
        assert_eq!(decode(&mut bindings, token, Act::Press), Some(action));
        assert_eq!(decode(&mut bindings, token, Act::Input { value: Box::new([]) }), None);
    }
    let field = jig_client::fields::Purpose::Answer;
    let token = bindings.insert(JigMeaning::Field(field)).token;
    assert_eq!(
        decode(&mut bindings, token, Act::Input { value: b"yes".as_slice().into() }),
        Some(Action::Edit { field, text: text(b"yes") })
    );
    assert_eq!(decode(&mut bindings, token, Act::Submit), Some(Action::Submit { field }));
    assert_eq!(decode(&mut bindings, token, Act::Input { value: b"large".as_slice().into() }), None);
    bindings.begin();
    assert_eq!(decode(&mut bindings, token, Act::Submit), None);
    assert_ne!(bindings.insert(JigMeaning::Field(field)).token, token);
}

#[test]
fn jig_phrasebook_requires_every_sentence_and_says_wall_times_in_person_offset() {
    let mut language = english::language();
    let missing = language.phrases.last().unwrap().id;
    let size = language.phrases.len() - 1;
    language.phrases = language.phrases.into_vec().into_iter().take(size).collect::<Vec<_>>().into_boxed_slice();
    assert_eq!(
        Phrasebook::load(
            shapes(128),
            Box::new([language]),
            &PhraseLimits { languages: 1, phrases: 1000, holes: 3, segments: 1000, text_bytes: 10000, hole_bytes: 128 }
        )
        .unwrap_err(),
        crate::phrases::Invalid::Missing { id: missing }
    );
    let words = words();
    let mut b = Builder::new(&limits());
    b.open(Role::Page);
    words.wall(&mut b, Wall::EPOCH);
    words.waiting(&mut b, Wall::from_nanos(240_000_000_000), Wall::EPOCH);
    words.amount(&mut b, u64::MAX);
    b.close();
    let tree = b.finish();
    assert!(contains(&tree, b"At 01:00"));
    assert!(contains(&tree, b"Waiting 4 min"));
    assert!(contains(&tree, b"18446744073709551615"));
    assert_eq!(words.say(JigPhrase::Live, &[]).as_ref(), b"Live");
    assert_eq!(words.say(JigPhrase::Amount, &[Hole::Amount(12)]).as_ref(), b"12");
}
