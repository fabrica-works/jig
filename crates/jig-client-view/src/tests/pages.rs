#![expect(
    clippy::wildcard_enum_match_arm,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary tests select scripted requests"
)]
use super::cards::{CLIENT, contains, document, drive, start, text, words};
use crate::JigBindings;
use crate::{
    Domain, Event, Request,
    address::{Address, Place},
    page::Page,
};
use jig_client::contract::*;
use skein_lib::{List, Token};
use skein_ui::{Builder, Role, Tree};
pub(super) fn build(d: &Domain, kind: u8, words: &crate::words::Words) -> (Tree, JigBindings) {
    let mut limits = super::cards::limits();
    limits.nodes = 2000;
    limits.depth = 32;
    limits.bindings = 512;
    limits.text_bytes = 200_000;
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(512, CLIENT.text);
    b.open(Role::Page);
    match kind {
        0 | 1 | 3 => {
            crate::pages::main(&mut b, d, words, &mut bindings);
        }
        _ => {
            crate::pages::not_the_work(&mut b, d, words, &mut bindings);
        }
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
pub(super) fn special_pages(words: &crate::words::Words) {
    let (mut d, f) = start();
    assert!(contains(&build(&d, 2, words).0, b"Connecting"));
    deliver(&mut d, f, Delivery::Frame(Box::new(document())));
    drive(&mut d, Event::Went { place: Place::Missing });
    assert!(contains(&build(&d, 2, words).0, b"Page missing"));
    drive(&mut d, Event::SignedOut);
    assert_eq!(d.page(), &Page::SigningIn);
    let mut d = Domain::new(&CLIENT, 1);
    d.provider(crate::signin::Provider { number: 3, name: text(b"Organization") }).unwrap();
    drive(
        &mut d,
        Event::Start {
            address: Address::Inbox { project: None },
            kept: List::with_capacity(0),
            offset: crate::boundary::Offset { minutes: 0 },
        },
    );
    drive(&mut d, Event::SignedOut);
    let (tree, mut bindings) = build(&d, 2, words);
    assert_eq!(
        crate::decode(&mut bindings, named_binding(&tree, b"Sign in with Organization"), skein_ui::Act::Press),
        Some(crate::action::Act::SignIn { provider: 3 })
    );
    let (mut d, f) = start();
    drive(&mut d, Event::Hello { served: false });
    assert!(deliver(&mut d, f, Delivery::Frame(Box::new(document()))).is_empty());
    assert!(contains(&build(&d, 2, words).0, b"update or reload"));
    let mut d = Domain::new(&CLIENT, 9);
    let mut kept = List::with_capacity(1);
    kept.push(crate::store::Record::Draft {
        version: crate::store::STORE_VERSION,
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
            offset: crate::boundary::Offset { minutes: 0 },
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
    for request in &out {
        if let Request::Read { read, what: Read::Inbox { .. } } = request {
            drive(
                &mut d,
                Event::Read {
                    read: *read,
                    result: ReadResult::Value(ReadValue::Inbox(Box::new(InboxPage {
                        cut: false,
                        rows: List::with_capacity(0),
                        before: None,
                        after: Some(Cursor::new(Box::from(b"next".as_slice()), 1024).expect("bounded opaque cursor")),
                    }))),
                },
            );
        }
    }
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
        crate::decode(&mut bindings, named_binding(&tree, b"Read later"), skein_ui::Act::Press),
        Some(crate::action::Act::More {
            window: Token::new(crate::page::conversations::INBOX_WINDOW),
            end: crate::action::End::After
        })
    );
    assert_eq!(
        crate::decode(&mut bindings, named_binding(&tree, b"Goal 7"), skein_ui::Act::Press),
        Some(crate::action::Act::FilterInbox {
            filter: crate::page::inbox::Filter { project: None, kind: None, goal: Some(7) }
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
    drive(&mut d, Event::Act { act: crate::action::Act::OpenUpdates });
    assert!(contains(&build(&d, 0, words).0, b"New"));
    let mut committed = value;
    committed.results_seen = 4;
    deliver(&mut d, page, Delivery::Inbox(Box::new(committed)));
    assert!(!contains(&build(&d, 0, words).0, b"New"));
    let filter = crate::page::inbox::Filter { project: None, kind: None, goal: None };
    drive(&mut d, Event::Act { act: crate::action::Act::FilterInbox { filter } });
    let read = out
        .iter()
        .find_map(|r| match r {
            Request::Read { read, what: Read::Inbox { group: InboxReadGroup::Updates, .. } } => Some(*read),
            _ => None,
        })
        .unwrap();
    drive(
        &mut d,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Inbox(Box::new(InboxPage {
                cut: false,
                rows,
                before: Some(Cursor::new(Box::from(b"earlier".as_slice()), 1024).expect("bounded opaque cursor")),
                after: Some(Cursor::new(Box::from(b"later".as_slice()), 1024).expect("bounded opaque cursor")),
            }))),
        },
    );
    let (tree, mut bindings) = build(&d, 0, words);
    assert!(contains(&tree, b"Read earlier"));
    assert!(contains(&tree, b"Read later"));
    let act = crate::decode(&mut bindings, named_binding(&tree, b"Read earlier"), skein_ui::Act::Press).unwrap();
    let out = drive(&mut d, Event::Act { act });
    assert!(out.iter().any(|r| match r {
        Request::Read { what: Read::Inbox { after: Some(cursor), .. }, .. } => cursor.value.bytes() == b"earlier",
        _ => false,
    }));
}

fn conversations_domain() -> Domain {
    let (mut d, f) = start();
    let mut frame = document();
    frame.charters.get_mut(0).unwrap().chats = List::with_capacity(1);
    frame
        .charters
        .get_mut(0)
        .unwrap()
        .chats
        .push(Charter { number: 2, name: Some(DocumentText::complete(text(b"guide"))) })
        .unwrap();
    frame.projects.get_mut(0).unwrap().chat_budget = 20;
    deliver(&mut d, f, Delivery::Frame(Box::new(frame)));
    drive(&mut d, Event::Went { place: Place::Jig(Address::Conversations { project: Some(4) }) });
    d
}
fn conversations_form(words: &crate::words::Words) {
    let mut d = conversations_domain();
    let (tree, mut bindings) = build(&d, 1, words);
    for label in [b"Conversations".as_slice(), b"First words", b"Charter 2", b"Budget 20", b"End of the list"] {
        assert!(contains(&tree, label) || tree.nodes().iter().any(|n| n.name.as_deref() == Some(label)));
    }
    assert_eq!(
        crate::decode(&mut bindings, named_binding(&tree, b"Start a conversation"), skein_ui::Act::Press),
        Some(crate::action::Act::StartChat { project: 4 })
    );
    let edit = crate::decode(
        &mut bindings,
        named_binding(&tree, b"First words"),
        skein_ui::Act::Input { value: b"Hello".to_vec().into_boxed_slice() },
    )
    .unwrap();
    drive(&mut d, Event::Act { act: edit });
    let out = drive(&mut d, Event::Act { act: crate::action::Act::StartChat { project: 4 } });
    let request = out
        .iter()
        .find_map(|r| match r {
            Request::Send { request, .. } => Some(*request),
            _ => None,
        })
        .unwrap();
    let waiting = build(&d, 1, words).0;
    assert!(waiting.nodes().iter().any(|n| n.states.as_slice().contains(&skein_ui::State::Busy)));
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
        (
            Runtime::Closing(Closing {
                delegates: 1,
                stage: None,
                ending: None,
                connectors: Some({
                    let mut rows = List::with_capacity(2);
                    rows.push(1).unwrap();
                    rows.push(2).unwrap();
                    rows
                }),
            }),
            b"Closing: 1 delegates".as_slice(),
        ),
        (Runtime::Held { was: None }, b"Held".as_slice()),
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
                cached_replies: 0,
                runtime: Some(runtime.clone()),
                delegates: 2,
                budget: OwnerBudget {
                    numbers: OwnerNumbers { budget: 20, spent: 5, spent_below: 1, reserved: 2 },
                    left: 12,
                    funder: Funder::Pool { project: 4, person: 1, period: 3 },
                    deadline: None,
                },
                chip: Chip {
                    task,
                    title: DocumentText::complete(text(b"Help")),
                    executor: Executor::Agent { charter: 2 },
                    phase: if task == 1 { Phase::Done } else { Phase::Idle },
                    details: Box::new(ChipPhase::Starting),
                },
                project: 4,
                first_words: DocumentText::complete(text(b"Help with work")),
                unread: Some(Count { value: 2, limited: task == 2 }),
                seen: 1,
                last_reply: 3,
                changed: Some(skein_lib::Wall::EPOCH),
            })
            .unwrap();
        }
        drive(
            &mut d,
            Event::Read {
                read: token,
                result: ReadResult::Value(ReadValue::Conversations(Box::new(ConversationPage {
                    cut: false,
                    rows,
                    before: Some(Cursor::new(Box::from(b"before".as_slice()), 1024).expect("bounded opaque cursor")),
                    after: Some(Cursor::new(Box::from(b"after".as_slice()), 1024).expect("bounded opaque cursor")),
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
            crate::decode(&mut bindings, named_binding(&tree, b"Task 2: Help"), skein_ui::Act::Press),
            Some(crate::action::Act::OpenChat { task: 2 })
        );
    }
}

#[test]
fn pages_show_non_work_states_inbox_edges_and_the_offered_conversation_form() {
    let words = words();
    special_pages(words);
    inbox_edges(words);
    inbox_committed_read_position(words);
    conversations_form(words);
    conversation_states(words);
}

#[test]
fn inbox_group_controls_decode_independent_windows() {
    let words = words();
    let (mut d, frame) = start();
    let out = deliver(&mut d, frame, Delivery::Frame(Box::new(document())));
    for request in &out {
        if let Request::Read { read, what: Read::Inbox { group, .. } } = request {
            drive(
                &mut d,
                Event::Read {
                    read: *read,
                    result: ReadResult::Value(ReadValue::Inbox(Box::new(InboxPage {
                        cut: false,
                        rows: List::with_capacity(0),
                        before: None,
                        after: Some(
                            Cursor::new(
                                Box::from(if *group == InboxReadGroup::Needs {
                                    b"needs".as_slice()
                                } else {
                                    b"updates".as_slice()
                                }),
                                1024,
                            )
                            .expect("bounded opaque cursor"),
                        ),
                    }))),
                },
            );
        }
    }
    let (tree, mut bindings) = build(&d, 0, words);
    let mut acts = Vec::new();
    for node in tree.nodes() {
        if node.name.as_deref() == Some(b"Read later".as_slice()) {
            acts.push(crate::decode(&mut bindings, node.binding.unwrap().token, skein_ui::Act::Press).unwrap());
        }
    }
    assert_eq!(
        acts.as_slice(),
        &[
            crate::action::Act::More {
                window: Token::new(crate::page::conversations::INBOX_WINDOW),
                end: crate::action::End::After
            },
            crate::action::Act::More {
                window: Token::new(crate::page::conversations::INBOX_UPDATES_WINDOW),
                end: crate::action::End::After
            }
        ]
    );
    let more = drive(&mut d, Event::Act { act: acts[1].clone() });
    let reads: Vec<_> = more
        .iter()
        .filter_map(|request| match request {
            Request::Read { what: Read::Inbox { group, after, .. }, .. } => Some((*group, after.clone().unwrap())),
            _ => None,
        })
        .collect();
    assert_eq!(reads.len(), 1);
    assert_eq!(reads[0].0, InboxReadGroup::Updates);
    assert_eq!(reads[0].1.value.bytes(), b"updates");
    assert_eq!(d.inbox().needs_window.after.as_ref().unwrap().value.bytes(), b"needs");
}

fn whole_page(d: &Domain) -> Tree {
    let mut limits = super::cards::limits();
    limits.nodes = 2000;
    limits.text_bytes = 200_000;
    limits.bindings = 512;
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(limits.bindings, CLIENT.text);
    b.open(Role::Page);
    assert!(crate::jig(
        &mut b,
        d,
        skein_ui::SlotRef { owner: (crate::Owner::Application { number: 1 }).opaque(), token: Token::new(1) },
        words(),
        &mut bindings
    ));
    b.close();
    b.finish()
}

#[test]
fn each_of_jigs_pages_keeps_the_outline() {
    use crate::address::{ProjectSection, TaskSection};
    let rows = [
        ("inbox", Place::Jig(Address::Inbox { project: None })),
        ("conversations", Place::Jig(Address::Conversations { project: None })),
        ("task overview", Place::Jig(Address::Task { task: 8, section: TaskSection::Overview })),
        ("project overview", Place::Jig(Address::Project { project: 4, section: ProjectSection::Overview })),
        ("deployment", Place::Jig(Address::Deployment)),
        ("page missing", Place::Missing),
    ];
    let (mut d, watch) = start();
    assert_eq!(skein_ui::outline::check(&whole_page(&d)), None, "starting");
    deliver(&mut d, watch, Delivery::Frame(Box::new(document())));
    for (label, place) in rows {
        let out = drive(&mut d, Event::Went { place });
        for request in out {
            if let Request::Open { watch, subject: Subject::Task { .. } } = request {
                deliver(&mut d, watch, Delivery::Task(Box::new(super::cards_objects::task_doc(&[]))));
            }
        }
        assert_eq!(skein_ui::outline::check(&whole_page(&d)), None, "{label}");
    }
    drive(&mut d, Event::SignedOut);
    assert_eq!(skein_ui::outline::check(&whole_page(&d)), None, "signing in");
}

fn domain_with_an_owed_card() -> Domain {
    let (mut d, watch) = start();
    deliver(&mut d, watch, Delivery::Frame(Box::new(document())));
    let mut objects = List::with_capacity(1);
    objects.push(ObjectKey::Task(8)).unwrap();
    let out = drive(&mut d, Event::ShowObjects { objects });
    for request in out {
        if let Request::Read { read, what: Read::Task { .. } } = request {
            drive(
                &mut d,
                Event::Read {
                    read,
                    result: ReadResult::Value(ReadValue::Task(Box::new(super::cards_objects::task_doc(&[
                        ActKind::Cancel,
                    ])))),
                },
            );
        }
    }
    let id = d.objects().find(ObjectKey::Task(8)).unwrap();
    drive(&mut d, Event::Act { act: crate::action::Act::Intend { object: id.token(), act: ActKind::Cancel } });
    assert_eq!(
        d.objects().get(id).unwrap().card,
        crate::objects::Card::Deciding { request: None, progress: crate::intents::Progress::Confirming }
    );
    drive(&mut d, Event::Act { act: crate::action::Act::Confirm });
    drive(&mut d, Event::ObjectsLeft);
    d
}

#[test]
fn the_owed_cards_sit_in_their_aside_after_the_main() {
    let d = domain_with_an_owed_card();
    let tree = whole_page(&d);
    let main = tree.nodes().iter().position(|node| node.role == Role::Main).unwrap();
    let owed =
        tree.nodes().iter().position(|node| node.region == Some(crate::regions::JigRegion::Owed.region())).unwrap();
    assert_eq!(tree.nodes()[owed].role, Role::Complementary);
    assert_eq!(main + usize::try_from(tree.nodes()[main].size).unwrap(), owed);
    assert!(tree.nodes()[owed].size > 1);
    assert_eq!(skein_ui::outline::check(&tree), None);
}

#[test]
fn the_inbox_header_shows_with_either_group() {
    let (d, _) = super::work_pages::entered(Address::Inbox { project: None });
    for group in [InboxReadGroup::Needs, InboxReadGroup::Updates] {
        let mut b = Builder::new(&super::cards::limits());
        let mut bindings = JigBindings::new(64, CLIENT.text);
        b.open(Role::Page);
        assert!(crate::sections::inbox::header(&mut b, &d, words(), &mut bindings));
        assert!(crate::sections::section(
            &mut b,
            &d,
            words(),
            &mut bindings,
            crate::page::Section::Inbox { group, project: None }
        ));
        b.close();
        let tree = b.finish();
        assert_eq!(
            tree.nodes().iter().filter(|n| n.region == Some(crate::regions::JigRegion::InboxHeader.region())).count(),
            1,
            "{group:?}"
        );
        assert!(contains(&tree, b"0 need you"), "{group:?}");
    }
    let (other, _) = super::work_pages::entered(Address::Conversations { project: None });
    let mut b = Builder::new(&super::cards::limits());
    let mut bindings = JigBindings::new(64, CLIENT.text);
    assert!(!crate::sections::inbox::header(&mut b, &other, words(), &mut bindings));
    assert_eq!(b.finish().len(), 0);
}
#[test]
fn the_new_chat_form_is_a_form_inside_the_conversations() {
    let d = conversations_domain();
    let (tree, _) = build(&d, 1, words());
    let nodes = tree.nodes();
    let section =
        nodes.iter().position(|n| n.region == Some(crate::regions::JigRegion::Conversations.region())).unwrap();
    let form = nodes.iter().position(|n| n.region == Some(crate::regions::JigRegion::NewChat.region())).unwrap();
    assert_eq!(nodes[form].role, Role::Form);
    assert!(form > section && form < section + usize::try_from(nodes[section].size).unwrap());
    let field =
        nodes.iter().position(|n| n.role == Role::TextArea && n.name.as_deref() == Some(b"First words")).unwrap();
    assert!(field > form && field < form + usize::try_from(nodes[form].size).unwrap());
}
#[test]
fn the_deployment_section_holds_its_four_named_regions() {
    use crate::regions::JigRegion;
    let d = super::work_pages::deployment_domain();
    let (tree, _) = build(&d, 3, words());
    let nodes = tree.nodes();
    let section = nodes.iter().position(|n| n.region == Some(JigRegion::Deployment.region())).unwrap();
    for (region, name) in [
        (JigRegion::Hosts, b"jig-hosts".as_slice()),
        (JigRegion::Accounts, b"jig-accounts".as_slice()),
        (JigRegion::Connectors, b"jig-connectors".as_slice()),
        (JigRegion::Capacity, b"jig-capacity".as_slice()),
    ] {
        let index = nodes.iter().position(|n| n.region == Some(region.region())).unwrap();
        assert_eq!(nodes[index].role, Role::Region, "{name:?}");
        assert_eq!(region.name(), name);
        assert!(index > section && index < section + usize::try_from(nodes[section].size).unwrap());
        assert!(nodes[index].name.as_ref().is_some_and(|name| !name.is_empty()));
    }
}

#[test]
fn a_missing_section_says_so_in_its_own_region() {
    let (mut domain, frame) = start();
    deliver(&mut domain, frame, Delivery::Frame(Box::new(document())));
    drive(&mut domain, Event::Went { place: Place::Elsewhere });
    let section = crate::page::Section::Task { task: 8, section: crate::address::TaskSection::Plan };
    let mut sections = List::with_capacity(1);
    sections.push(section).unwrap();
    let opened = drive(&mut domain, Event::ShowSections { sections });
    let watch = opened
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, .. } => Some(*watch),
            _ => None,
        })
        .unwrap();
    drive(&mut domain, Event::Ended { watch, end: WatchEnd::NotAllowed });
    let mut builder = Builder::new(&super::cards::limits());
    let mut bindings = JigBindings::new(256, CLIENT.text);
    assert!(crate::sections::section(&mut builder, &domain, words(), &mut bindings, section));
    let tree = builder.finish();
    assert_eq!(tree.nodes()[0].region, Some(crate::regions::JigRegion::TaskPlan.region()));
    assert_eq!(tree.nodes()[0].size, tree.len());
    assert!(contains(&tree, b"Page missing"));
}
