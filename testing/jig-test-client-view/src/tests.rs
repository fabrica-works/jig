#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary fixtures collect outputs and query public accessibility trees"
)]
mod connector_values;
mod family_terminals;
mod world_limits;
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
            connector: jig_test_client_domain::connector_values::ConnectorLimits {
                rows: 0,
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
                party: FrameParty {
                    number: 7,
                    kind: PartyKind::Person,
                    login: DocumentText::complete(text(b"fixture-login")),
                    name: DocumentText::complete(text(b"Pat")),
                },
                projects: List::with_capacity(0),
                needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
                needs_total: 0,
                results_seen: 0,
                charters: List::with_capacity(0),
                cut: false,
                updates: Count { value: 0, limited: false },

                retention: Duration::from_secs(1000),
                deployment_role: Some(HeldRole::Policy { role: 1 }),
            }))),
        }),
    );
    root
}
fn words() -> jig_client_view::words::Words {
    static WORDS: std::sync::OnceLock<jig_client_view::words::Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| crate::connector_values_words::english(limits().jig.text).unwrap()).clone()
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
fn owner_authority() -> OwnerAuthorityDocument {
    OwnerAuthorityDocument {
        authority: OwnerAuthority {
            tools: 0,
            grants: List::with_capacity(0),
            delegation: OwnerDelegation { kinds: List::with_capacity(0), tasks: 1, depth: 1 },
            spend: 1,
            deadline: None,
            notes: 0,
            note_resources: List::with_capacity(0),
        },
        cut: false,
    }
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
            title: DocumentText::complete(text(&[b't'; 32])),
            executor: Executor::Procedure { connector: jig_test_client_domain::FIRST, kind: 1 },
            phase: Phase::Running,
            details: Box::new(ChipPhase::Procedure {
                panel: Part::Panel { connector: jig_test_client_domain::FIRST, task: 9 },
            }),
        },
        goal: None,
        priority: Some(1),
        lineage_chips: list([]),
        inputs: list([]),
        project: 3,
        requester: OwnerRequester::Person(7),
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
        wake: OwnerWakePolicy {
            words: WakeRule::Never,
            notices: WakeRule::Never,
            news: WakeRule::Never,
            results: ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        subscriptions: Subscriptions { entries: List::with_capacity(0) },
        result_contract: TaskContractDocument::Report { words: 32 },
        dependencies: list([]),
        references: list([]),
        cut: false,
        resources: list([]),
        budget: OwnerBudget {
            numbers: OwnerNumbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
            left: 1,
            funder: Funder::Pool { project: 1, person: 7, period: 3 },
            deadline: None,
        },
        authority: owner_authority(),
        hold: None,
        proposals: list([]),
        activity: 0,
        changed: Wall::EPOCH,
        acts: list([]),
        result: None,
        delegates: list([]),

        context: Some(Box::new(TaskContext {
            escalation: None,
            parts: list([]),
            closing: None,
            spend: list([]),
            runtime: Runtime::Procedure { panel: Part::Panel { connector: jig_test_client_domain::FIRST, task: 9 } },
            impact: Impact {
                tasks: PhaseCounts {
                    ended: 0,
                    waiting: 0,
                    running: 0,
                    held: 0,
                    closing: 0,
                    done: 0,
                    failed: 0,
                    cancelled: 0,
                },
                live_runs: 0,
                proposals: 0,
                effects_pending: 0,
                effects_settling: 0,
                takeover_budget: 0,
            },
            writer_slots: list([]),
        })),
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

#[test]
fn raw_owner_effect_keys_keep_full_scope_and_numbers_separate_from_equal_core_makers() {
    let reference = u64::MAX;
    let mut keys = Vec::new();
    for scope in [family::Scope::Project(0), family::Scope::Project(u32::MAX), family::Scope::Deployment] {
        let key = crate::family_key(&family::Subject { scope, kind: family::Kind::OwnerEffect(reference) });
        assert_eq!(key.kind, 20);
        assert_eq!(key.b, reference);
        assert_eq!(
            key.a,
            match scope {
                family::Scope::Project(project) => u64::from(project),
                family::Scope::Deployment => u64::MAX,
            }
        );
        assert!(!keys.contains(&key));
        keys.push(key);
    }
    for proposer in [Proposer::Task(reference), Proposer::Party(reference)] {
        let key = crate::family_key(&family::Subject {
            scope: family::Scope::Project(0),
            kind: family::Kind::Effect(ProposalKey { proposer, number: reference }),
        });
        assert!(!keys.contains(&key));
        keys.push(key);
    }
    assert_eq!(keys.len(), 5);
}

#[test]
fn complete_application_object_uses_its_own_zero_value_form_and_exact_terminal_fragment() {
    use jig_test_client_domain::connector_values::{
        ApplicationObject, ApplicationOutcome, ClientDocument, ConnectorAnswer, ConnectorDocument,
    };
    let mut root = ready();
    let opened =
        drive(&mut root, Event::Went { place: Place::Address(Address::First(first::Address { project: 3, item: 5 })) });
    let subject = family::Subject { scope: family::Scope::Project(3), kind: family::Kind::Item(5) };
    drive(
        &mut root,
        Event::First(first::Event::Read {
            read: family_read(&opened),
            result: first::ReadResult::Value(family::Document::Connector(ConnectorDocument {
                subject,
                value: Box::new(ClientDocument::Object {
                    object: 5,
                    now: ApplicationObject { revision: 17, value: u64::MAX },
                }),
            })),
        }),
    );
    let id = root.first().find(subject).unwrap();
    let mut view = view();
    patches(&mut view, &root);
    assert!(contains(&view, b"18446744073709551615"));
    let field = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == Role::TextField && node.name.as_deref() == Some(b"Parameters".as_slice()))
        .unwrap()
        .id;
    let changed = decode(&mut view, field, Act::Input { value: Box::from(*b"0") }).unwrap();
    assert_eq!(
        changed,
        jig_test_client_domain::Action::First(first::Act::Parameters {
            object: id.token(),
            revision: Revision { value: 17 },
            value: 0
        })
    );
    drive(&mut root, Event::Act { action: changed });
    patches(&mut view, &root);
    let apply = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Apply parameters".as_slice()))
        .unwrap()
        .id;
    drive(&mut root, Event::Act { action: decode(&mut view, apply, Act::Press).unwrap() });
    let mut sends = Queue::with_capacity(jig_test_client_domain::max_out(&limits()));
    jig_test_client_domain::take(&mut root, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }, &mut sends);
    let mut attempt = None;
    while let Some(send) = sends.pop() {
        if let Request::Send { attempt: number, ask: jig_test_client_domain::Ask::First(ask), .. } = send {
            assert_eq!(
                ask,
                family::Ask::ChangeObject { project: 3, object: 5, revision: Revision { value: 17 }, value: 0 }
            );
            attempt = Some(number);
        }
    }
    let answer = first::Answer::Connector(ConnectorAnswer {
        value: ApplicationOutcome::Changed { object: 5, now: ApplicationObject { revision: 18, value: 0 } },
    });
    drive(
        &mut root,
        Event::Answer {
            attempt: attempt.unwrap(),
            envelope: skein_lib::RequestEnvelope::Final,
            answer: Some(jig_test_client_domain::Answered::First(answer.clone())),
        },
    );
    patches(&mut view, &root);
    assert!(
        view.tree()
            .nodes()
            .iter()
            .any(|node| node.role == Role::Status && node.name.as_deref() == Some(b"Application outcome".as_slice()))
    );
    assert!(contains(&view, b"Object changed"));
    assert!(contains(&view, b"Object: 5"));
    assert!(contains(&view, b"Object revision: 18"));
    assert!(contains(&view, b"Value: 0"));
    assert_eq!(root.first().object(id).unwrap().card, first::Card::Done { answer });
    assert!(view.tree().len() <= view.limits().tree.nodes);
    assert!(patches(&mut view, &root).is_empty());
}

#[test]
fn unsupported_individual_deployment_parts_say_cannot_show_without_a_project_or_unknown_value() {
    let mut root = ready();
    let part = Part::Outbox { connector: jig_test_client_domain::FIRST, scope: PartScope::Deployment, entry: u64::MAX };
    let opened = drive(
        &mut root,
        Event::First(first::Event::Show { part: part.clone(), generation: 1, scope: family::Scope::Deployment }),
    );
    let read = family_read(&opened);
    let watch = opened
        .iter()
        .find_map(|request| match request {
            Request::First(first::Out::Open { watch, .. }) => Some(*watch),
            _ => None,
        })
        .unwrap();
    drive(&mut root, Event::First(first::Event::Read { read, result: first::ReadResult::Unsupported }));
    drive(&mut root, Event::First(first::Event::Ended { watch, end: first::WatchEnd::Unsupported }));
    let subject = family::Subject { scope: family::Scope::Deployment, kind: family::Kind::Outbox(u64::MAX) };
    assert_eq!(root.first().object(root.first().find(subject).unwrap()).unwrap().value, first::Value::Unsupported);
    let jig = words();
    let family_words = words::Words::english(limits().first.text).unwrap();
    let bounds = Limits::of(&limits(), &jig, &family_words).unwrap();
    let mut builder = jig_client_view::Builder::new(&bounds.tree);
    let mut bindings = crate::bindings::Bindings::new(bounds.tree.bindings, limits().first.text);
    builder.open(Role::Page);
    crate::family::part(&mut builder, root.first(), &part, 1, &family_words, &jig, &mut bindings);
    builder.close();
    let tree = builder.finish();
    assert!(
        tree.nodes()
            .iter()
            .any(|node| node.role == Role::Status && node.name.as_deref() == Some(b"Cannot show this part".as_slice()))
    );
    assert!(
        !tree
            .nodes()
            .iter()
            .any(|node| node.role == Role::Status && node.name.as_deref() == Some(b"Unknown family object".as_slice()))
    );
    assert!(!tree.nodes().iter().any(|node| node.role == Role::TextField));
    assert!(tree.nodes().iter().filter_map(|node| node.key).all(|key| key.kind != 10 || key.a != 0));
}
