#![expect(
    clippy::wildcard_enum_match_arm,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary tests select scripted requests"
)]
use super::cards::{CLIENT, contains, document, drive, start, text, words};
use crate::{Builder, JigBindings, Role, Tree};
use jig_client::{
    Domain, Event, Request,
    address::{Address, Place},
    contract::*,
    pages::Page,
};
use skein_lib::{List, Token};
pub(super) fn build(d: &Domain, kind: u8, words: &crate::words::Words) -> (Tree, JigBindings) {
    let mut limits = super::cards::limits();
    limits.nodes = 2000;
    limits.bindings = 512;
    limits.text_bytes = 200_000;
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(512, CLIENT.text);
    b.open(Role::Page);
    match kind {
        0 => crate::pages::inbox(&mut b, d, words, &mut bindings),
        1 => crate::pages::conversations(&mut b, d, words, &mut bindings),
        3 => {
            crate::pages::page(&mut b, d, words, &mut bindings);
        }
        _ => crate::pages::not_the_work(&mut b, d, words, &mut bindings),
    }
    b.close();
    (b.finish(), bindings)
}
pub(super) fn deliver(d: &mut Domain, token: Token, document: Delivery) -> Vec<Request> {
    drive(d, Event::Watch { watch: token, event: WatchEvent::Delivered(document) })
}
pub(super) fn named_binding(tree: &Tree, name: &[u8]) -> Token {
    tree.nodes().iter().find(|n| n.name.as_deref() == Some(name) && n.binding.is_some()).unwrap().binding.unwrap().token
}
fn special_pages(words: &crate::words::Words) {
    let (mut d, f) = start();
    assert!(contains(&build(&d, 2, words).0, b"Connecting"));
    deliver(&mut d, f, Delivery::Frame(Box::new(document())));
    drive(&mut d, Event::Went { place: Place::Missing });
    assert!(contains(&build(&d, 2, words).0, b"Page missing"));
    drive(&mut d, Event::SignedOut);
    assert_eq!(d.page(), &Page::SigningIn);
    let mut d = Domain::new(&CLIENT, 1);
    d.provider(jig_client::signin::Provider { number: 3, name: text(b"Organization") }).unwrap();
    drive(
        &mut d,
        Event::Start {
            address: Address::Inbox { project: None },
            kept: List::with_capacity(0),
            offset: jig_client::boundary::Offset { minutes: 0 },
        },
    );
    drive(&mut d, Event::SignedOut);
    let (tree, mut bindings) = build(&d, 2, words);
    assert_eq!(
        crate::decode(&mut bindings, named_binding(&tree, b"Sign in with Organization"), crate::Act::Press),
        Some(jig_client::action::Act::SignIn { provider: 3 })
    );
    let (mut d, f) = start();
    let mut frame = document();
    frame.versions = VersionRange { first: 2, last: 2 };
    deliver(&mut d, f, Delivery::Frame(Box::new(frame)));
    assert!(contains(&build(&d, 2, words).0, b"update or reload"));
    let mut d = Domain::new(&CLIENT, 9);
    let mut kept = List::with_capacity(1);
    kept.push(jig_client::store::Record::Draft {
        version: jig_client::store::STORE_VERSION,
        party: 2,
        task: 1,
        words: text(b"private"),
        touched: 1,
    })
    .unwrap();
    let out = drive(
        &mut d,
        Event::Start {
            address: Address::Inbox { project: None },
            kept,
            offset: jig_client::boundary::Offset { minutes: 0 },
        },
    );
    let f = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Frame } => Some(*watch),
            _ => None,
        })
        .unwrap();
    deliver(&mut d, f, Delivery::Frame(Box::new(document())));
    assert_eq!(d.page(), &Page::SomeoneElse);
    let tree = build(&d, 2, words).0;
    assert!(contains(&tree, b"Someone else's work"));
    assert!(!contains(&tree, b"private"));
}
fn inbox_edges(words: &crate::words::Words) {
    let (mut d, f) = start();
    let out = deliver(&mut d, f, Delivery::Frame(Box::new(document())));
    let page = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Inbox { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let mut entries = List::with_capacity(2);
    for n in 1..=2 {
        entries
            .push(InboxEntry {
                reply: None,
                lineage: List::with_capacity(0),
                goal: Some(7),
                recipient: Some(Recipient::Role { project: 4, role: 1 }),
                stall: Some(skein_lib::Wall::from_nanos(100)),
                object: ObjectKey::Task(n),
                project: 4,
                group: InboxGroup::Questions,
                position: n,
                at: skein_lib::Wall::EPOCH,
            })
            .unwrap();
    }
    deliver(
        &mut d,
        page,
        Delivery::Inbox(Box::new(Inbox {
            entries,
            needs: Needs { questions: u32::MAX, proposals: 1, escalations: 0, person_tasks: 0 },
            updates: Count { value: 10, limited: true },
            results_seen: 0,
            replies_seen: List::with_capacity(0),
            more: true,
        })),
    );
    let (tree, mut bindings) = build(&d, 0, words);
    for label in [
        b"4294967296 need you".as_slice(),
        b"More than 10 updates",
        b"For the person on call",
        b"Passes up at",
        b"Read later",
    ] {
        assert!(contains(&tree, label));
    }
    assert_eq!(
        crate::decode(&mut bindings, named_binding(&tree, b"Read later"), crate::Act::Press),
        Some(jig_client::action::Act::More {
            window: Token::new(jig_client::pages::conversations::INBOX_WINDOW),
            end: jig_client::action::End::After
        })
    );
    assert_eq!(
        crate::decode(&mut bindings, named_binding(&tree, b"Goal 7"), crate::Act::Press),
        Some(jig_client::action::Act::FilterInbox {
            filter: jig_client::pages::inbox::Filter { project: None, kind: None, goal: Some(7) }
        })
    );
}
fn inbox_committed_read_position(words: &crate::words::Words) {
    let (mut d, f) = start();
    let out = deliver(&mut d, f, Delivery::Frame(Box::new(document())));
    let page = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Inbox { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let mut rows = List::with_capacity(1);
    rows.push(InboxEntry {
        reply: None,
        lineage: List::with_capacity(0),
        goal: None,
        recipient: None,
        stall: None,
        object: ObjectKey::Result(1),
        project: 4,
        group: InboxGroup::Results,
        position: 4,
        at: skein_lib::Wall::EPOCH,
    })
    .unwrap();
    let value = Inbox {
        entries: rows.clone(),
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        updates: Count { value: 1, limited: false },
        results_seen: 0,
        replies_seen: List::with_capacity(0),
        more: false,
    };
    deliver(&mut d, page, Delivery::Inbox(Box::new(value.clone())));
    assert!(contains(&build(&d, 0, words).0, b"New"));
    drive(&mut d, Event::Act { act: jig_client::action::Act::OpenUpdates });
    assert!(contains(&build(&d, 0, words).0, b"New"));
    let mut committed = value;
    committed.results_seen = 4;
    deliver(&mut d, page, Delivery::Inbox(Box::new(committed)));
    assert!(!contains(&build(&d, 0, words).0, b"New"));
    let filter = jig_client::pages::inbox::Filter { project: None, kind: None, goal: None };
    let out = drive(&mut d, Event::Act { act: jig_client::action::Act::FilterInbox { filter } });
    let read = out
        .iter()
        .find_map(|r| match r {
            Request::Read { read, what: Read::Inbox { .. } } => Some(*read),
            _ => None,
        })
        .unwrap();
    drive(
        &mut d,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Inbox(Box::new(InboxPage {
                rows,
                before: Some(Cursor { value: text(b"earlier") }),
                after: Some(Cursor { value: text(b"later") }),
            }))),
        },
    );
    let (tree, mut bindings) = build(&d, 0, words);
    assert!(contains(&tree, b"Read earlier"));
    assert!(contains(&tree, b"Read later"));
    let act = crate::decode(&mut bindings, named_binding(&tree, b"Read earlier"), crate::Act::Press).unwrap();
    let out = drive(&mut d, Event::Act { act });
    assert!(out.iter().any(|r| match r {
        Request::Read { what: Read::Inbox { after: Some(cursor), .. }, .. } => cursor.value.bytes() == b"earlier",
        _ => false,
    }));
}

fn conversations_form(words: &crate::words::Words) {
    let (mut d, f) = start();
    let mut frame = document();
    frame.projects.get_mut(0).unwrap().chats = List::with_capacity(1);
    frame.projects.get_mut(0).unwrap().chats.push(Charter { number: 2, name: text(b"guide") }).unwrap();
    frame.projects.get_mut(0).unwrap().chat_budget = 20;
    deliver(&mut d, f, Delivery::Frame(Box::new(frame)));
    drive(&mut d, Event::Went { place: Place::Jig(Address::Conversations { project: Some(4) }) });
    let (tree, mut bindings) = build(&d, 1, words);
    for label in [b"Conversations".as_slice(), b"First words", b"Charter 2", b"Budget 20", b"End of the list"] {
        assert!(contains(&tree, label) || tree.nodes().iter().any(|n| n.name.as_deref() == Some(label)));
    }
    assert_eq!(
        crate::decode(&mut bindings, named_binding(&tree, b"Start a conversation"), crate::Act::Press),
        Some(jig_client::action::Act::StartChat { project: 4 })
    );
    let edit = crate::decode(
        &mut bindings,
        named_binding(&tree, b"First words"),
        crate::Act::Input { value: b"Hello".to_vec().into_boxed_slice() },
    )
    .unwrap();
    drive(&mut d, Event::Act { act: edit });
    let out = drive(&mut d, Event::Act { act: jig_client::action::Act::StartChat { project: 4 } });
    let request = out
        .iter()
        .find_map(|r| match r {
            Request::Send { request, .. } => Some(*request),
            _ => None,
        })
        .unwrap();
    let waiting = build(&d, 1, words).0;
    assert!(waiting.nodes().iter().any(|n| n.states.as_slice().contains(&crate::State::Busy)));
    drive(&mut d, Event::Answered { request, answer: Answer::Refused(Refusal::NotOffered) });
    assert!(contains(&build(&d, 1, words).0, b"This act is not offered"));
    assert!(
        tree.nodes()
            .iter()
            .filter(|n| n.binding.is_some())
            .all(|n| n.name.as_ref().is_some_and(|name| !name.is_empty()))
    );
}
fn conversation_states(words: &crate::words::Words) {
    let cases = [
        (Runtime::Idle, b"Waiting for news".as_slice()),
        (Runtime::Starting, b"Starting".as_slice()),
        (Runtime::Running { attempt: 1, since: skein_lib::Wall::EPOCH, tool: None }, b"Writing".as_slice()),
        (
            Runtime::Running { attempt: 1, since: skein_lib::Wall::EPOCH, tool: Some(text(b"inspect")) },
            b"Using tool inspect".as_slice(),
        ),
        (Runtime::WaitingForWords { attempt: 1, since: skein_lib::Wall::EPOCH }, b"Waiting for your words".as_slice()),
        (
            Runtime::BackingOff {
                at: skein_lib::Wall::EPOCH,
                attempt: 1,
                tries: 2,
                class: FailureClass::Temporary,
                reason: text(b"later"),
            },
            b"Retrying at".as_slice(),
        ),
        (Runtime::Closing(Closing { delegates: 1, effects: 2 }), b"Closing: 1 delegates".as_slice()),
        (Runtime::Held, b"Held".as_slice()),
        (Runtime::Ended, b"Ended".as_slice()),
    ];
    for (runtime, expected) in cases {
        let (mut d, f) = start();
        deliver(&mut d, f, Delivery::Frame(Box::new(document())));
        let out = drive(&mut d, Event::Went { place: Place::Jig(Address::Conversations { project: None }) });
        let token = out
            .iter()
            .find_map(|r| match r {
                Request::Read { read, what: Read::Conversations { .. } } => Some(*read),
                _ => None,
            })
            .unwrap();
        let mut rows = List::with_capacity(2);
        for task in [1, 2] {
            rows.push(ConversationLine {
                runtime: runtime.clone(),
                delegates: 2,
                budget: Budget { total: 20, own: 5, ended: 1, reserved: 2, left: 12, funder: Actor::Party(1) },
                chip: Chip {
                    task,
                    title: text(b"Help"),
                    executor: Executor::Agent { charter: 2 },
                    phase: if task == 1 { Phase::Done } else { Phase::Idle },
                    details: Box::new(ChipPhase::Starting),
                },
                project: 4,
                first_words: text(b"Help with work"),
                unread: Count { value: 2, limited: task == 2 },
                seen: 1,
                last_reply: 3,
                changed: skein_lib::Wall::EPOCH,
            })
            .unwrap();
        }
        drive(
            &mut d,
            Event::Read {
                read: token,
                result: ReadResult::Value(ReadValue::Conversations(Box::new(ConversationPage {
                    rows,
                    before: Some(Cursor { value: text(b"before") }),
                    after: Some(Cursor { value: text(b"after") }),
                }))),
            },
        );
        let (tree, mut bindings) = build(&d, 1, words);
        for label in [expected, b"Live delegates: 2", b"More than 2 updates", b"Read earlier", b"Read later"] {
            assert!(contains(&tree, label), "{label:?}");
        }
        let keyed: Vec<_> = tree.nodes().iter().filter_map(|n| n.key.filter(|key| key.kind == 20)).collect();
        assert_eq!(keyed[0].a, 2);
        assert_eq!(keyed[1].a, 1);
        assert_eq!(
            crate::decode(&mut bindings, named_binding(&tree, b"Task 2: Help"), crate::Act::Press),
            Some(jig_client::action::Act::OpenChat { task: 2 })
        );
    }
}

#[test]
fn pages_show_non_work_states_inbox_edges_and_the_offered_conversation_form() {
    let words = words();
    special_pages(&words);
    inbox_edges(&words);
    inbox_committed_read_position(&words);
    conversations_form(&words);
    conversation_states(&words);
}
