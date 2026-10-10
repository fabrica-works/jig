#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary fixtures collect outputs and query public accessibility trees"
)]
mod connector_values;
use crate::{Limits, View, build, decode, words};
use jig_client::contract::*;
use jig_client_view::{Act, NodeId, Owner, Patch, Role};
use jig_test_client_domain::{Address, Domain, Event, JigOut, Place, Request, family, first};
use skein_lib::{Duration, Env, List, Queue, RequestLimits, Time, Token, Wall};
fn limits() -> jig_test_client_domain::Limits {
    jig_test_client_domain::Limits {
        jig: jig_client::Limits {
            watches: 8,
            notices: 2,
            notice_life: Duration::from_secs(5),
            linger: Duration::from_secs(5),
            save_delay: Duration::from_secs(1),
            intents: 2,
            fields: 4,
            providers: 2,
            objects: 3,
            sources: 8,
            reads: 8,
            parts: 8,
            delegates: 2,
            document_bytes: 1024,
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
            backoff: jig_client::limits::Backoff { first: Duration::from_nanos(10), most: Duration::from_nanos(80) },
            heartbeat: Duration::from_secs(100),
        },
        first: first::Limits {
            parts: 8,
            objects: 8,
            reads: 16,
            watches: 16,
            requests: 4,
            rows: 2,
            text: 32,
            retry: Duration::from_nanos(10),
        },
        second_parts: 8,
        table: RequestLimits {
            requests: 4,
            bytes: 65536,
            out: 16,
            first: Duration::from_nanos(10),
            most: Duration::from_nanos(80),
            margin: Duration::from_secs(1),
        },
    }
}
fn drive(root: &mut Domain, event: Event) -> Vec<Request> {
    let mut out = Queue::with_capacity(jig_test_client_domain::max_out(&limits()));
    jig_test_client_domain::step(root, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }, event, &mut out);
    let mut rows = Vec::new();
    while let Some(row) = out.pop() {
        rows.push(row);
    }
    rows
}
fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), limits().jig.text).unwrap()
}
fn list<T>(rows: impl IntoIterator<Item = T>) -> List<T> {
    let rows: Vec<_> = rows.into_iter().collect();
    let mut out = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for row in rows {
        out.push(row).ok().unwrap();
    }
    out
}
fn ready() -> Domain {
    let mut root = jig_test_client_domain::new(&limits(), 1);
    let rows = drive(
        &mut root,
        Event::Start {
            address: Address::Jig(jig_client::address::Address::Inbox { project: None }),
            kept: List::with_capacity(0),
            offset: jig_client::boundary::Offset { minutes: 0 },
        },
    );
    let watch = rows
        .iter()
        .find_map(|row| match row {
            Request::Jig(JigOut::Open { watch, subject: Subject::Frame }) => Some(*watch),
            _ => None,
        })
        .unwrap();
    drive(&mut root, Event::Jig(jig_client::Event::Hello { versions: VersionRange { first: 1, last: 1 } }));
    drive(
        &mut root,
        Event::Jig(jig_client::Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Frame(Box::new(Frame {
                party: Party { number: 7, kind: PartyKind::Person, name: text(b"Pat") },
                projects: List::with_capacity(0),
                needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
                updates: Count { value: 0, limited: false },

                retention: Duration::from_secs(1000),
                deployment_role: Some(1),
            }))),
        }),
    );
    root
}
fn words() -> jig_client_view::words::Words {
    static WORDS: std::sync::OnceLock<jig_client_view::words::Words> = std::sync::OnceLock::new();
    WORDS
        .get_or_init(|| jig_client_view::words::Words {
            book: jig_client_view::phrases::english::english(limits().jig.text).unwrap(),
            language: 0,
            offset_minutes: 0,
        })
        .clone()
}
fn family_words() -> words::Words {
    static WORDS: std::sync::OnceLock<words::Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| words::Words::english(limits().first.text).unwrap()).clone()
}
fn view() -> View {
    let jig = words();
    let first = family_words();
    let bounds = Limits::for_world(&limits(), &jig, &first, 1024, 256).unwrap();
    View::new(&bounds, &limits(), jig, first, first::Address { project: 3, item: 5 })
}
fn patches(view: &mut View, root: &Domain) -> Vec<Patch> {
    let mut out = Queue::with_capacity(view.limits().tree.patches);
    build(view, root, &mut out);
    let mut rows = Vec::new();
    while let Some(row) = out.pop() {
        rows.push(row);
    }
    rows
}
fn family_read(rows: &[Request]) -> Token {
    rows.iter()
        .find_map(|row| match row {
            Request::First(first::Out::Read { read, .. }) => Some(*read),
            _ => None,
        })
        .unwrap()
}
fn family_document(root: &mut Domain, proposal_count: u64) -> Vec<Request> {
    let rows =
        drive(root, Event::Went { place: Place::Address(Address::First(first::Address { project: 3, item: 5 })) });
    let proposals = list(
        (2..2_u64.checked_add(proposal_count).unwrap())
            .map(|number| ProposalKey { proposer: Proposer::Task(9), number }),
    );
    drive(
        root,
        Event::First(first::Event::Read {
            read: family_read(&rows),
            result: first::ReadResult::Value(family::Document::Item(family::Item {
                project: 3,
                key: 5,
                revision: Revision { value: 1 },
                state: family::ItemState::Ready,
                value: u64::MAX,
                task: 9,
                proposals,
                touchable: true,
            })),
        }),
    )
}
fn authority() -> Authority {
    Authority {
        budget: 1,
        grants: list([]),
        task_kinds: list([]),
        depth: 1,
        tasks: 1,
        deadline: None,
        tools: list([]),
        note_scopes: list([]),
    }
}
fn budget() -> Budget {
    Budget { total: 1, own: 0, ended: 0, reserved: 0, left: 1, funder: Actor::Party(7) }
}
fn task() -> Task {
    Task {
        chip: Chip {
            task: 9,
            title: text(&[b't'; 32]),
            executor: Executor::Procedure { connector: jig_test_client_domain::FIRST, kind: 1 },
            phase: Phase::Running,
            details: Box::new(ChipPhase::Procedure {
                panel: Part::Panel { connector: jig_test_client_domain::FIRST, task: 9 },
            }),
        },
        goal: false,
        priority: Some(1),
        lineage_chips: list([]),
        inputs: list([]),
        project: 3,
        requester: Actor::Party(7),
        lineage: list([]),
        spec: SpecificationDocument::complete(Specification {
            words: text(b"spec"),
            parameters: list([]),
            inputs: list([]),
        }),
        spec_revision: Revision { value: 1 },
        person_revision: None,
        taken_by: None,
        instructions: Some(DocumentText::complete(text(b"instructions"))),
        wake: WakePolicy { words: false, results: false, topics: list([]), subscriptions: list([]) },
        result_contract: ResultContract::Report,
        dependencies: list([]),
        references: list([]),
        resources: list([]),
        budget: budget(),
        authority: authority(),
        hold: None,
        escalation: None,
        proposals: list([]),
        activity: 0,
        deadline: None,
        changed: Wall::EPOCH,
        acts: list([]),
        parts: list([]),
        closing: None,
        spend: list([]),
        result: None,
        runtime: Runtime::Procedure { panel: Part::Panel { connector: jig_test_client_domain::FIRST, task: 9 } },
        impact: Impact {
            tasks: PhaseCounts { waiting: 0, running: 0, held: 0, closing: 0, done: 0, failed: 0, cancelled: 0 },
            live_runs: 0,
            proposals: 0,
            effects_pending: 0,
            effects_settling: 0,
            takeover_budget: 0,
        },
        delegates: list([]),
        writer_slots: list([]),
    }
}
fn proposal(key: ProposalKey) -> Proposal {
    Proposal {
        key,
        project: 3,
        proposer: key.proposer,
        why: text(&[b'w'; 32]),
        action: ProposalAction::Effect {
            part: Part::Effect { connector: jig_test_client_domain::FIRST, proposal: key },
        },
        needs: authority(),
        funding: budget(),
        waiting: Recipient::Roles { project: 3, roles: list([1, 2]) },
        trail: list([]),
        since: Wall::EPOCH,
        acts: list([]),
        ending: None,
    }
}
fn load_cards(root: &mut Domain, rows: Vec<Request>) {
    for row in rows {
        if let Request::Jig(JigOut::Read { read, what }) = row {
            let value = match what {
                Read::Task { .. } => ReadValue::Task(Box::new(task())),
                Read::Proposal { proposal: key } => ReadValue::Proposal(Box::new(proposal(key))),
                _ => continue,
            };
            let parts = drive(root, Event::Jig(jig_client::Event::Read { read, result: ReadResult::Value(value) }));
            for row in parts {
                if let Request::First(first::Out::Read { read, subject }) = row {
                    let parameters = match subject.kind {
                        family::Kind::Panel(_) => Some(family::Parameters {
                            item: 5,
                            revision: Revision { value: 1 },
                            value: 13,
                            amendable: true,
                        }),
                        _ => None,
                    };
                    drive(
                        root,
                        Event::First(first::Event::Read {
                            read,
                            result: first::ReadResult::Value(family::Document::Part(family::PartDocument {
                                subject,
                                revision: Revision { value: 1 },
                                value: 13,
                                parameters,
                            })),
                        }),
                    );
                }
            }
        }
    }
}
fn named(view: &View, role: Role, name: &[u8]) -> NodeId {
    view.tree().nodes().iter().find(|node| node.role == role && node.name.as_deref() == Some(name)).unwrap().id
}
fn contains(view: &View, text: &[u8]) -> bool {
    view.tree()
        .nodes()
        .iter()
        .any(|node| node.text.as_slice().iter().any(|run| run.text.windows(text.len()).any(|bytes| bytes == text)))
}
#[test]
fn a_family_page_holding_jig_cards_holding_parts_fits_and_diffs_to_nothing() {
    let mut root = ready();
    let rows = family_document(&mut root, 2);
    load_cards(&mut root, rows);
    let mut view = view();
    assert!(!patches(&mut view, &root).is_empty());
    assert!(contains(&view, b"Task 9"));
    assert!(contains(&view, b"Family part revision 1"));
    assert!(contains(&view, b"Waiting at policy roles"));
    assert!(view.tree().len() <= view.limits().tree.nodes);
    assert!(!view.tree().nodes().iter().any(|node| node.states.as_slice().contains(&jig_client_view::State::Busy)));
    assert_eq!(
        view.tree()
            .nodes()
            .iter()
            .flat_map(|node| node.text.as_slice())
            .filter(|run| run.text.as_ref() == b"Family part revision 1")
            .count(),
        3
    );
    assert!(patches(&mut view, &root).is_empty());
}
#[test]
fn the_drawing_has_a_named_readable_fallback_and_bounded_owner_bytes() {
    let mut root = ready();
    family_document(&mut root, 0);
    let mut view = view();
    patches(&mut view, &root);
    let id = named(&view, Role::Drawing, b"Item value");
    let drawing = view.tree().find(id).unwrap().drawing.as_ref().unwrap();
    assert_eq!(drawing.kind, crate::drawing::KIND);
    assert_eq!(&*drawing.data, &u64::MAX.to_le_bytes());
    assert_eq!(drawing.data.len(), usize::try_from(crate::drawing::BYTES).unwrap());
    named(&view, Role::Status, b"Value: 18446744073709551615");
    assert!(contains(&view, b"Value: 18446744073709551615"));
    let jig = words();
    let first = family_words();
    let mut bounds = limits();
    bounds.first.rows = u32::MAX;
    assert!(Limits::of(&bounds, &jig, &first).is_none());
}
#[test]
fn buttons_and_parameter_fields_decode_to_their_owner_and_departed_nodes_are_stale() {
    let mut root = ready();
    let rows = family_document(&mut root, 1);
    load_cards(&mut root, rows);
    let mut view = view();
    patches(&mut view, &root);
    let button = named(&view, Role::Button, b"Touch");
    let object = root.first().page().unwrap().token();
    assert_eq!(
        decode(&mut view, button, Act::Press),
        Some(crate::Action::First(first::Act::Touch { object, revision: Revision { value: 1 } }))
    );
    let field = named(&view, Role::TextField, b"Parameters");
    let subject = family::Subject { scope: family::Scope::Project(3), kind: family::Kind::Panel(9) };
    let panel = root.first().find(subject).unwrap().token();
    assert_eq!(
        decode(&mut view, field, Act::Input { value: b"21".as_slice().into() }),
        Some(crate::Action::First(first::Act::Parameters {
            object: panel,
            revision: Revision { value: 1 },
            value: 21
        }))
    );
    assert_eq!(decode(&mut view, field, Act::Input { value: b"bad".as_slice().into() }), None);
    let nav = named(&view, Role::Link, b"Family");
    assert_eq!(
        decode(&mut view, nav, Act::Press),
        Some(crate::Action::First(first::Act::Go { address: first::Address { project: 3, item: 5 } }))
    );
    let jig_node = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.binding.is_some_and(|binding| binding.owner == Owner::Jig) && node.role == Role::Link)
        .unwrap()
        .id;
    let Some(crate::Action::Jig(_)) = decode(&mut view, jig_node, Act::Press) else {
        panic!("jig owns its navigation action")
    };
    let action = decode(&mut view, field, Act::Input { value: b"21".as_slice().into() }).unwrap();
    drive(&mut root, Event::Act { action });
    patches(&mut view, &root);
    assert_eq!(view.tree().find(field).unwrap().value.as_ref().unwrap().text.as_ref(), b"21");
    let apply = named(&view, Role::Button, b"Apply parameters");
    let action = decode(&mut view, apply, Act::Press).unwrap();
    drive(&mut root, Event::Act { action });
    patches(&mut view, &root);
    assert!(view.tree().find(field).unwrap().states.as_slice().contains(&jig_client_view::State::Disabled));
    assert_eq!(view.tree().find(field).unwrap().binding, None);
    assert_eq!(decode(&mut view, field, Act::Input { value: b"22".as_slice().into() }), None);
    drive(&mut root, Event::Went { place: Place::Address(Address::Jig(jig_client::address::Address::Deployment)) });
    patches(&mut view, &root);
    assert_eq!(decode(&mut view, button, Act::Press), None);
    assert_eq!(decode(&mut view, field, Act::Input { value: b"21".as_slice().into() }), None);
}

#[test]
fn family_card_keys_are_public_subjects_across_page_and_fragment_positions() {
    let mut root = ready();
    let rows = family_document(&mut root, 2);
    load_cards(&mut root, rows);
    let mut view = view();
    patches(&mut view, &root);
    for kind in [family::Kind::Item(5), family::Kind::Panel(9)] {
        let subject = family::Subject { scope: family::Scope::Project(3), kind };
        let key = crate::family_key(&subject);
        assert!(
            view.tree().nodes().iter().any(|node| node.key == Some(key)),
            "subject {subject:?} is rendered independently of its slab position"
        );
    }
    assert_ne!(
        crate::family_key(&family::Subject { scope: family::Scope::Project(3), kind: family::Kind::Panel(9) }),
        crate::family_key(&family::Subject { scope: family::Scope::Project(4), kind: family::Kind::Panel(9) })
    );
}

#[test]
fn a_family_entrance_refusal_shows_its_budget_and_keeps_the_object_present() {
    let mut root = ready();
    family_document(&mut root, 0);
    let id = root.first().page().unwrap();
    drive(
        &mut root,
        Event::Act {
            action: jig_test_client_domain::Action::First(first::Act::Touch {
                object: id.token(),
                revision: Revision { value: 1 },
            }),
        },
    );
    let mut out = Queue::with_capacity(jig_test_client_domain::max_out(&limits()));
    jig_test_client_domain::take(&mut root, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }, &mut out);
    let mut attempt = None;
    while let Some(request) = out.pop() {
        if let Request::Send { attempt: sent, .. } = request {
            attempt = Some(sent);
        }
    }
    let refusal = EntranceRefusal::new(Refusal::Authority {
        lacking: Lacking::Budget { needed: 19, left: 3 },
        proposable: Some(false),
    })
    .unwrap();
    drive(
        &mut root,
        Event::Answer {
            attempt: attempt.unwrap(),
            envelope: skein_lib::RequestEnvelope::Final,
            answer: Some(jig_test_client_domain::Answered::First(first::Answer::Refused(refusal))),
        },
    );
    let mut view = view();
    patches(&mut view, &root);
    assert!(contains(&view, b"19"));
    assert!(contains(&view, b"3"));
    assert!(contains(&view, b"Nobody in this project may accept it"));
    assert!(!contains(&view, b"Gone"));
    assert!(root.first().object(id).is_some());
    assert!(view.tree().len() <= view.limits().tree.nodes);
    assert!(patches(&mut view, &root).is_empty());
}

#[test]
fn outbox_family_keys_keep_scope_entry_and_effect_proposal_distinct() {
    let entry = u64::MAX;
    let subjects = [
        family::Subject { scope: family::Scope::Project(0), kind: family::Kind::Outbox(entry) },
        family::Subject { scope: family::Scope::Project(3), kind: family::Kind::Outbox(entry) },
        family::Subject { scope: family::Scope::Project(4), kind: family::Kind::Outbox(entry) },
        family::Subject { scope: family::Scope::Deployment, kind: family::Kind::Outbox(entry) },
        family::Subject {
            scope: family::Scope::Project(3),
            kind: family::Kind::Effect(ProposalKey { proposer: Proposer::Task(entry), number: entry }),
        },
    ];
    let mut root = ready();
    let opened =
        drive(&mut root, Event::Went { place: Place::Address(Address::Jig(jig_client::address::Address::Deployment)) });
    let watch = opened
        .iter()
        .find_map(|row| match row {
            Request::Jig(JigOut::Open { watch, subject: Subject::Deployment }) => Some(*watch),
            _ => None,
        })
        .unwrap();
    let mut parts = List::with_capacity(3);
    for scope in [PartScope::Project(3), PartScope::Project(4), PartScope::Deployment] {
        parts.push(Part::Outbox { connector: jig_test_client_domain::FIRST, scope, entry }).unwrap();
    }
    drive(&mut root, Event::Jig(jig_client::Event::Hello { versions: VersionRange { first: 1, last: 1 } }));
    drive(
        &mut root,
        Event::Jig(jig_client::Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Deployment(Box::new(Deployment {
                hosts: list([]),
                accounts: list([]),
                capacity: list([]),
                proposals: 0,
                refusals: list([]),
                parts,
            }))),
        }),
    );
    let mut view = view();
    patches(&mut view, &root);
    let shown = view
        .tree()
        .nodes()
        .iter()
        .flat_map(|node| node.text.iter())
        .filter(|run| run.text.as_ref() == b"Outbox entry 18446744073709551615")
        .count();
    assert_eq!(shown, 3);
    for subject in &subjects[1..4] {
        assert!(view.tree().nodes().iter().any(|node| node.key == Some(crate::family_key(subject))));
    }
    for (index, subject) in subjects.iter().enumerate() {
        let key = crate::family_key(subject);
        for previous in &subjects[..index] {
            assert_ne!(key, crate::family_key(previous));
        }
        if let family::Kind::Outbox(_) = subject.kind {
            assert_eq!(key.b, entry);
        }
    }
}

#[test]
fn effect_family_keys_keep_equal_task_and_party_makers_in_distinct_public_items() {
    let keys = [
        ProposalKey { proposer: Proposer::Task(u64::MAX), number: u64::MAX },
        ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX },
    ];
    let mut root = ready();
    let opened =
        drive(&mut root, Event::Went { place: Place::Address(Address::Jig(jig_client::address::Address::Deployment)) });
    let watch = opened
        .iter()
        .find_map(|row| match row {
            Request::Jig(JigOut::Open { watch, subject: Subject::Deployment }) => Some(*watch),
            _ => None,
        })
        .unwrap();
    let parts = list(keys.map(|proposal| Part::Effect { connector: jig_test_client_domain::FIRST, proposal }));
    let reads = drive(
        &mut root,
        Event::Jig(jig_client::Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Deployment(Box::new(Deployment {
                hosts: list([]),
                accounts: list([]),
                capacity: list([]),
                proposals: 0,
                refusals: list([]),
                parts,
            }))),
        }),
    );
    let subjects: Vec<_> = reads
        .iter()
        .filter_map(|row| match row {
            Request::First(first::Out::Read { subject, .. }) => Some(*subject),
            _ => None,
        })
        .collect();
    assert_eq!(subjects.len(), 2);
    for (index, subject) in subjects.iter().enumerate() {
        assert_eq!(subject.kind, family::Kind::Effect(keys[index]));
    }
    let mut view = view();
    patches(&mut view, &root);
    let public: Vec<_> = subjects.iter().map(crate::family_key).collect();
    assert_ne!(public[0], public[1]);
    for key in public {
        assert_eq!((key.a, key.b), (u64::MAX, u64::MAX));
        assert!(view.tree().nodes().iter().any(|node| node.key == Some(key)));
    }
    assert!(view.tree().len() <= view.limits().tree.nodes);
    assert!(patches(&mut view, &root).is_empty());
}
