#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "fixtures inspect public tree nodes and observed boundary outputs"
)]
use crate::{Limits, View, build, words};
use jig_client::contract::*;
use jig_ops_client_domain::{self as root, Address, Domain, Event, JigOut, Request, infrastructure, observability};
use skein_lib::{Duration, Env, List, Queue, RequestLimits, Time, Wall};
const JIG: jig_client::Limits = jig_client::Limits {
    watches: 4,
    notices: 2,
    notice_life: Duration::from_secs(5),
    linger: Duration::from_secs(5),
    save_delay: Duration::from_secs(1),
    intents: 2,
    fields: 4,
    providers: 2,
    objects: 3,
    sources: 8,
    reads: 4,
    parts: 10,
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
    backoff: jig_client::limits::Backoff { first: Duration::from_nanos(10), most: Duration::from_nanos(80) },
    heartbeat: Duration::from_nanos(100),
};
fn limits() -> root::Limits {
    root::Limits {
        jig: JIG,
        infrastructure: infrastructure::Limits {
            parts: 10,
            text: 32,
            objects: 12,
            watches: 24,
            reads: 24,
            requests: 4,
            rows: 2,
            retry: Duration::from_nanos(10),
            document_bytes: 65536,
        },
        observability: observability::Limits {
            parts: 10,
            objects: 12,
            reads: 24,
            watches: 24,
            rows: 2,
            text: 32,
            retry: Duration::from_nanos(10),
            document_bytes: 65536,
        },
        table: RequestLimits {
            requests: 4,
            bytes: 262_144,
            out: 16,
            first: Duration::from_nanos(10),
            most: Duration::from_nanos(80),
            margin: Duration::from_secs(1),
        },
    }
}

fn drive(root: &mut Domain, event: Event) -> Vec<Request> {
    let mut out = Queue::with_capacity(root::max_out(&limits()));
    root::step(root, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }, event, &mut out);
    let mut rows = Vec::new();
    while let Some(r) = out.pop() {
        rows.push(r);
    }
    rows
}
fn ready() -> Domain {
    let mut root = root::new(&limits(), 5);
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
        .find_map(|r| match r {
            Request::Jig(JigOut::Open { watch, subject: Subject::Frame }) => Some(*watch),
            _ => None,
        })
        .unwrap();
    let frame = Frame {
        party: Party { number: 7, kind: PartyKind::Person, name: Text::new(Box::from(*b"Pat"), 32).unwrap() },
        projects: List::with_capacity(0),
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        updates: Count { value: 0, limited: false },
        versions: VersionRange { first: 1, last: 1 },
        retention: Duration::from_secs(1000),
        deployment_role: Some(1),
    };
    drive(
        &mut root,
        Event::Jig(jig_client::Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Frame(Box::new(frame))) }),
    );
    root
}
fn view() -> View {
    let jig = jig_client_view::words::Words {
        book: jig_client_view::phrases::english::english(32).unwrap(),
        language: 0,
        offset_minutes: 0,
    };
    let obs = words::Words::english(1, 32).unwrap();
    let infra = words::Words::english(2, 32).unwrap();
    let v = Limits::of(&limits(), &jig, &obs, &infra).unwrap();
    View::new(&v, &limits(), jig, obs, infra, 1)
}
fn incident() -> observability::documents::Incident {
    use observability::documents::*;
    let mut timeline = List::with_capacity(2);
    let before = Health {
        observed: Wall::from_nanos(36_000_000_000_000),
        healthy_replicas: 3,
        errors: 2,
        error_rate_percent: 2,
    };
    timeline.push(Point { at: before.observed, health: Some(before) }).unwrap();
    Incident {
        project: 1,
        number: 12,
        revision: Revision { value: 2 },
        watch: 888,
        raised: before.observed,
        service: Service { environment: Box::from(*b"production"), name: Box::from(*b"checkout") },
        severity: 2,
        state: State::Alerting,
        timeline,
        cut: false,
        health: Some(Health {
            observed: Wall::from_nanos(36_120_000_000_000),
            healthy_replicas: 2,
            errors: 30,
            error_rate_percent: 30,
        }),
        lead: None,
        proposals: List::with_capacity(2),
    }
}
#[test]
fn deployment_keys_are_distinct_from_both_zero_and_maximum_project_identifiers() {
    let infrastructure = [
        infrastructure::Scope::Project(0),
        infrastructure::Scope::Project(u32::MAX),
        infrastructure::Scope::Deployment,
    ]
    .map(|scope| {
        crate::infrastructure::key(&infrastructure::Subject { scope, kind: infrastructure::Kind::Outbox(17) })
    });
    let observability =
        [observability::Scope::Project(0), observability::Scope::Project(u32::MAX), observability::Scope::Deployment]
            .map(|scope| {
                crate::observability::key(&observability::Subject { scope, kind: observability::Kind::Outbox(17) })
            });
    for keys in [infrastructure, observability] {
        assert_ne!(keys[0], keys[1]);
        assert_ne!(keys[0], keys[2]);
        assert_ne!(keys[1], keys[2]);
        assert_eq!(keys[0].a, 0);
        assert_eq!(keys[1].a, u64::from(u32::MAX));
        assert_eq!(keys[2].a, u64::MAX);
        assert!(keys.iter().all(|key| key.b == 17 && key.kind == 6));
    }
}
#[test]
fn an_incident_drawing_has_a_readable_fallback_and_missing_lead_is_explicit() {
    let mut root = ready();
    let requests = drive(
        &mut root,
        Event::Went {
            place: root::Place::Address(Address::Observability(observability::Address::Incident {
                project: 1,
                incident: 12,
            })),
        },
    );
    let read = requests
        .iter()
        .find_map(|r| match r {
            Request::Observability(observability::Out::Read { read, .. }) => Some(*read),
            _ => None,
        })
        .unwrap();
    drive(
        &mut root,
        Event::Observability(observability::Event::Read {
            read,
            result: observability::ReadResult::Value(Box::new(observability::Document::Incident(incident()))),
        }),
    );
    let mut view = view();
    let mut patches = Queue::with_capacity(view.limits().tree.patches);
    build(&mut view, &root, &mut patches);
    assert!(
        view.tree()
            .nodes()
            .iter()
            .any(|n| n.role == jig_client_view::Role::Drawing && n.name.as_deref() == Some(b"Error rate".as_slice()))
    );
    let text: Vec<u8> =
        view.tree().nodes().iter().flat_map(|n| n.text.iter().flat_map(|r| r.text.iter().copied())).collect();
    assert!(String::from_utf8(text).unwrap().contains("Error rate 30 % Since 10:02, Before: 2 %"));
    assert!(view.tree().nodes().iter().any(|n| n.role == jig_client_view::Role::Status
        && n.name.as_deref() == Some(b"No lead task has been supplied".as_slice())));
    while patches.pop().is_some() {}
    build(&mut view, &root, &mut patches);
    assert!(patches.is_empty());
}
#[test]
fn family_keys_keep_scope_and_original_owner_identity() {
    use observability::documents::{Kind, Scope, Subject};
    let a = crate::observability::key(&Subject { scope: Scope::Project(1), kind: Kind::Incident(12) });
    let b = crate::observability::key(&Subject { scope: Scope::Project(2), kind: Kind::Incident(12) });
    assert_ne!(a, b);
    assert_eq!(a.b, 12);
    assert_ne!(
        crate::observability::key(&Subject {
            scope: Scope::Project(1),
            kind: Kind::Effect(ProposalKey { proposer: 12, number: 3 })
        }),
        crate::observability::key(&Subject { scope: Scope::Project(1), kind: Kind::Outbox(3) })
    );
}

fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), 32).unwrap()
}
fn list<T>(rows: impl IntoIterator<Item = T>) -> List<T> {
    let rows: Vec<_> = rows.into_iter().collect();
    let mut out = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for row in rows {
        out.push(row).ok().unwrap();
    }
    out
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
            executor: Executor::Procedure { connector: 2, kind: 1 },
            phase: Phase::Running,
            details: Box::new(ChipPhase::Procedure { panel: Part::Panel { connector: 2, task: 9 } }),
        },
        goal: false,
        priority: Some(1),
        lineage_chips: list([]),
        inputs: list([]),
        project: 1,
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
        instructions: text(b"instructions"),
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
        runtime: Runtime::Procedure { panel: Part::Panel { connector: 2, task: 9 } },
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
        project: 1,
        proposer: Actor::Task(9),
        why: text(&[b'w'; 32]),
        action: ProposalAction::Effect { part: Part::Effect { connector: 2, proposal: key } },
        needs: authority(),
        funding: budget(),
        waiting: Recipient::Roles { project: 1, roles: list([1, 2]) },
        trail: list([]),
        since: Wall::EPOCH,
        acts: list([]),
        ending: None,
    }
}

fn contains(view: &View, words: &[u8]) -> bool {
    view.tree()
        .nodes()
        .iter()
        .flat_map(|n| n.text.iter())
        .any(|r| r.text.windows(words.len()).any(|slice| slice == words))
}
#[test]
fn an_incident_with_a_proposal_and_an_infrastructure_panel_fills_both_slot_passes() {
    let mut root = ready();
    let requests = drive(
        &mut root,
        Event::Went {
            place: root::Place::Address(Address::Observability(observability::Address::Incident {
                project: 1,
                incident: 12,
            })),
        },
    );
    let read = requests
        .iter()
        .find_map(|r| match r {
            Request::Observability(observability::Out::Read { read, .. }) => Some(*read),
            _ => None,
        })
        .unwrap();
    let mut facts = incident();
    facts.lead = Some(9);
    facts.proposals.push(ProposalKey { proposer: 9, number: 1 }).unwrap();
    let requests = drive(
        &mut root,
        Event::Observability(observability::Event::Read {
            read,
            result: observability::ReadResult::Value(Box::new(observability::Document::Incident(facts))),
        }),
    );
    for r in requests {
        if let Request::Jig(JigOut::Read { read, what }) = r {
            let value = match what {
                Read::Task { .. } => ReadValue::Task(Box::new(task())),
                Read::Proposal { proposal: key } => ReadValue::Proposal(Box::new(proposal(key))),
                _ => continue,
            };
            let parts =
                drive(&mut root, Event::Jig(jig_client::Event::Read { read, result: ReadResult::Value(value) }));
            for r in parts {
                if let Request::Infrastructure(infrastructure::Out::Read { read, subject }) = r {
                    use infrastructure::documents::{Panel, PartDocument, Phase, Procedure, Service};
                    let doc = PartDocument {
                        subject,
                        revision: Revision { value: 1 },
                        panel: Some(Panel {
                            task: 9,
                            procedure: Procedure::Remediate {
                                service: Service {
                                    environment: Box::from(*b"production"),
                                    name: Box::from(*b"checkout"),
                                },
                                operation: 17,
                                deadline: Wall::EPOCH,
                            },
                            phase: Phase::WaitingCondition,
                        }),
                        description: None,
                        effect: None,
                        resources: List::with_capacity(2),
                        effects: List::with_capacity(2),
                        drift: List::with_capacity(2),
                        environments: List::with_capacity(2),
                        pools: List::with_capacity(2),
                        backend: None,
                        made: None,
                        request_limit: None,
                        staged_limit: None,
                    };
                    drive(
                        &mut root,
                        Event::Infrastructure(infrastructure::Event::Read {
                            read,
                            result: infrastructure::ReadResult::Value(Box::new(infrastructure::Document::Part(
                                Box::new(doc),
                            ))),
                        }),
                    );
                }
            }
        }
    }
    let mut view = view();
    let mut patches = Queue::with_capacity(view.limits().tree.patches);
    build(&mut view, &root, &mut patches);
    assert!(contains(&view, b"Task 9"));
    assert!(contains(&view, b"Waiting at policy roles"));
    assert!(view.tree().nodes().iter().any(|n| n.role == jig_client_view::Role::Region
        && n.name.as_deref() == Some(b"Infrastructure procedure".as_slice())));
    assert!(contains(&view, b"Waiting for condition"));
    assert!(view.tree().len() <= view.limits().tree.nodes);
    while patches.pop().is_some() {}
    build(&mut view, &root, &mut patches);
    assert!(patches.is_empty());
}
#[test]
fn moving_an_environment_end_names_the_end_revision_and_waits_for_the_final_envelope() {
    use infrastructure::documents::{Creation, End, Environment, Purpose};
    let mut root = ready();
    let requests = drive(
        &mut root,
        Event::Went {
            place: root::Place::Address(Address::Infrastructure(infrastructure::Address::Environment {
                project: 1,
                environment: 7,
            })),
        },
    );
    let read = requests
        .iter()
        .find_map(|r| match r {
            Request::Infrastructure(infrastructure::Out::Read { read, .. }) => Some(*read),
            _ => None,
        })
        .unwrap();
    let document = Environment {
        project: 1,
        number: 7,
        pool: 3,
        name: Box::from(*b"staging-7"),
        revision: Revision { value: 99 },
        created_by: Creation { deployment: [1; 16], task: 9, origin: Purpose::Procedure { purpose: 2 }, purpose: 2 },
        ready_at: None,
        ready: Some(false),
        cost: Some(40),
        observed_until: Some(Wall::from_nanos(123_000_000_000)),
        end: Some(End { revision: Revision { value: 5 }, until: Wall::from_nanos(600_000_000_000) }),
    };
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Read {
            read,
            result: infrastructure::ReadResult::Value(Box::new(infrastructure::Document::Environment(
                document.clone(),
            ))),
        }),
    );
    let mut view = view();
    let mut patches = Queue::with_capacity(view.limits().tree.patches);
    while patches.pop().is_some() {}
    build(&mut view, &root, &mut patches);
    assert!(contains(&view, b"Not ready"));
    assert!(contains(&view, b"Observed backend lease:"));
    let field = view
        .tree()
        .nodes()
        .iter()
        .find(|n| {
            n.role == jig_client_view::Role::TextField
                && n.name.as_deref() == Some(b"New end (seconds since epoch)".as_slice())
        })
        .unwrap()
        .id;
    let action = crate::decode(&mut view, field, jig_client_view::Act::Input { value: Box::from(*b"900") }).unwrap();
    drive(&mut root, Event::Act { action });
    while patches.pop().is_some() {}
    build(&mut view, &root, &mut patches);
    let button = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == jig_client_view::Role::Button && n.name.as_deref() == Some(b"Move end".as_slice()))
        .unwrap()
        .id;
    let action = crate::decode(&mut view, button, jig_client_view::Act::Press).unwrap();
    assert!(drive(&mut root, Event::Act { action }).is_empty());
    let mut out = Queue::with_capacity(root::max_out(&limits()));
    root::take(&mut root, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }, &mut out);
    let Some(Request::Save { record: root::store::Record::Table { .. } }) = out.pop() else {
        panic!("save precedes send");
    };
    let request = out.pop().unwrap();
    let Request::Send {
        attempt,
        ask: root::Ask::Infrastructure(infrastructure::Ask::MoveEnd { revision, until, .. }),
        ..
    } = request
    else {
        panic!("keyed move leaves through table")
    };
    assert_eq!(revision.value, 5);
    assert_eq!(until.as_secs(), 900);
    drive(
        &mut root,
        Event::Answer {
            attempt,
            envelope: skein_lib::RequestEnvelope::Again,
            answer: Some(root::Answered::Infrastructure(infrastructure::Answer::MovedEnd {
                environment: 7,
                revision: Revision { value: 6 },
                until,
            })),
        },
    );
    while patches.pop().is_some() {}
    build(&mut view, &root, &mut patches);
    assert!(!contains(&view, b"End moved"));
}
#[test]
fn extreme_capacity_and_wall_overflow_are_refused_before_building() {
    let jig = jig_client_view::words::Words {
        book: jig_client_view::phrases::english::english(32).unwrap(),
        language: 0,
        offset_minutes: 0,
    };
    let obs = words::Words::english(1, 32).unwrap();
    let infra = words::Words::english(2, 32).unwrap();
    let mut limits = limits();
    limits.observability.rows = u32::MAX;
    assert!(Limits::of(&limits, &jig, &obs, &infra).is_none());
    assert!(crate::time::wall(u64::MAX).is_none());
}

#[test]
fn an_environment_without_an_end_has_no_revision_or_move_form() {
    use infrastructure::documents::{Creation, Environment, Purpose};
    let mut root = ready();
    let requests = drive(
        &mut root,
        Event::Went {
            place: root::Place::Address(Address::Infrastructure(infrastructure::Address::Environment {
                project: 1,
                environment: 7,
            })),
        },
    );
    let read = requests
        .iter()
        .find_map(|r| match r {
            Request::Infrastructure(infrastructure::Out::Read { read, .. }) => Some(*read),
            _ => None,
        })
        .unwrap();
    let document = Environment {
        project: 1,
        number: 7,
        pool: 3,
        name: Box::from(*b"staging-7"),
        revision: Revision { value: 99 },
        created_by: Creation { deployment: [1; 16], task: 9, origin: Purpose::Procedure { purpose: 2 }, purpose: 2 },
        ready_at: None,
        ready: None,
        cost: None,
        observed_until: None,
        end: None,
    };
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Read {
            read,
            result: infrastructure::ReadResult::Value(Box::new(infrastructure::Document::Environment(document))),
        }),
    );
    let mut view = view();
    let mut patches = Queue::with_capacity(view.limits().tree.patches);
    build(&mut view, &root, &mut patches);
    assert!(!view.tree().nodes().iter().any(|n| n.role == jig_client_view::Role::TextField
        && n.name.as_deref() == Some(b"New end (seconds since epoch)".as_slice())));
    assert!(!view.tree().nodes().iter().any(|n| n.name.as_deref() == Some(b"Move end".as_slice())));
    assert!(!contains(&view, b"Cost:"));
    assert!(!contains(&view, b"Revision"));
    assert!(!contains(&view, b"Not ready"));
}

#[test]
fn a_common_entrance_refusal_prints_supplied_budget_facts_without_claiming_gone() {
    use infrastructure::documents::{Creation, End, Environment, Purpose};
    let mut root = ready();
    let requests = drive(
        &mut root,
        Event::Went {
            place: root::Place::Address(Address::Infrastructure(infrastructure::Address::Environment {
                project: 1,
                environment: 7,
            })),
        },
    );
    let read = requests
        .iter()
        .find_map(|r| match r {
            Request::Infrastructure(infrastructure::Out::Read { read, .. }) => Some(*read),
            _ => None,
        })
        .unwrap();
    let document = Environment {
        project: 1,
        number: 7,
        pool: 3,
        name: Box::from(*b"staging-7"),
        revision: Revision { value: 99 },
        created_by: Creation { deployment: [1; 16], task: 9, origin: Purpose::Procedure { purpose: 2 }, purpose: 2 },
        ready_at: None,
        ready: Some(false),
        cost: Some(40),
        observed_until: Some(Wall::from_nanos(123_000_000_000)),
        end: Some(End { revision: Revision { value: 5 }, until: Wall::from_nanos(600_000_000_000) }),
    };
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Read {
            read,
            result: infrastructure::ReadResult::Value(Box::new(infrastructure::Document::Environment(
                document.clone(),
            ))),
        }),
    );
    let object = root.infrastructure().page().unwrap().token();
    let revision = Revision { value: 5 };
    drive(
        &mut root,
        Event::Act {
            action: root::Action::Infrastructure(infrastructure::Act::Until {
                object,
                revision,
                value: Wall::from_nanos(900_000_000_000),
            }),
        },
    );
    drive(
        &mut root,
        Event::Act { action: root::Action::Infrastructure(infrastructure::Act::MoveEnd { object, revision }) },
    );
    let mut out = Queue::with_capacity(root::max_out(&limits()));
    root::take(&mut root, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }, &mut out);
    let mut attempt = None;
    while let Some(request) = out.pop() {
        if let Request::Send { attempt: token, .. } = request {
            attempt = Some(token);
        }
    }
    let refusal =
        EntranceRefusal::new(Refusal::Authority { lacking: Lacking::Budget { needed: 10, left: 3 }, proposable: None })
            .unwrap();
    drive(
        &mut root,
        Event::Answer {
            attempt: attempt.unwrap(),
            envelope: skein_lib::RequestEnvelope::Final,
            answer: Some(root::Answered::Infrastructure(infrastructure::Answer::EntranceRefused(refusal))),
        },
    );
    let mut view = view();
    let mut patches = Queue::with_capacity(view.limits().tree.patches);
    build(&mut view, &root, &mut patches);
    assert!(contains(&view, b"Your pool has 3 left; this needs 10"));
    assert!(!contains(&view, b"Gone"));
    assert!(!contains(&view, b"End moved"));
    assert!(view.tree().len() <= view.limits().tree.nodes);
}

#[test]
fn actual_outbox_entries_render_from_their_explicit_scope_and_keep_proposal_keys_distinct() {
    for scope in [PartScope::Project(0), PartScope::Project(u32::MAX), PartScope::Project(91), PartScope::Deployment] {
        let mut root = ready();
        let rows = drive(
            &mut root,
            Event::Went { place: root::Place::Address(Address::Jig(jig_client::address::Address::Deployment)) },
        );
        let watch = rows
            .iter()
            .find_map(|request| match request {
                Request::Jig(JigOut::Open { watch, subject: Subject::Deployment }) => Some(*watch),
                _ => None,
            })
            .unwrap();
        let mut parts = List::with_capacity(1);
        parts.push(Part::Outbox { connector: 2, scope, entry: 17 }).unwrap();
        let rows = drive(
            &mut root,
            Event::Jig(jig_client::Event::Watch {
                watch,
                event: WatchEvent::Delivered(Delivery::Deployment(Box::new(Deployment {
                    hosts: List::with_capacity(0),
                    accounts: List::with_capacity(0),
                    capacity: List::with_capacity(0),
                    proposals: 0,
                    refusals: List::with_capacity(0),
                    parts,
                }))),
            }),
        );
        let (read, subject) = rows
            .iter()
            .find_map(|request| match request {
                Request::Infrastructure(infrastructure::Out::Read { read, subject }) => Some((*read, *subject)),
                _ => None,
            })
            .unwrap();
        let expected_scope = match scope {
            PartScope::Project(project) => infrastructure::Scope::Project(project),
            PartScope::Deployment => infrastructure::Scope::Deployment,
        };
        assert_eq!(subject, infrastructure::Subject { scope: expected_scope, kind: infrastructure::Kind::Outbox(17) });
        drive(
            &mut root,
            Event::Infrastructure(infrastructure::Event::Read {
                read,
                result: infrastructure::ReadResult::Value(Box::new(outbox_document(subject))),
            }),
        );
        let mut view = view();
        let mut patches = Queue::with_capacity(view.limits().tree.patches);
        build(&mut view, &root, &mut patches);
        let value = match scope {
            PartScope::Project(project) => u64::from(project),
            PartScope::Deployment => u64::MAX,
        };
        let expected_key =
            jig_client_view::Key { owner: jig_client_view::Owner::Side { connector: 2 }, kind: 6, a: value, b: 17 };
        assert!(view.tree().nodes().iter().any(|node| node.key == Some(expected_key)));
        assert!(contains(&view, b"Outbox entry 17"));
        assert!(contains(&view, b"Sent"));
        assert!(!contains(&view, b"Maximum price:"));
        let proposal_key = crate::infrastructure::key(&infrastructure::Subject {
            scope: expected_scope,
            kind: infrastructure::Kind::Effect(ProposalKey { proposer: 17, number: 17 }),
        });
        assert_ne!(expected_key, proposal_key);
    }
}

fn outbox_document(subject: infrastructure::Subject) -> infrastructure::Document {
    use infrastructure::documents::{Creation, Effect, EffectAction, EffectState, PartDocument, Purpose, Service};
    let mut effects = List::with_capacity(1);
    effects
        .push(Effect {
            entry: 17,
            task: 29,
            project: Some(91),
            origin: Creation { deployment: [4; 16], task: 29, origin: Purpose::Procedure { purpose: 2 }, purpose: 2 },
            description: None,
            state: EffectState::Sent,
            attempt: 2,
            deadline: Wall::EPOCH,
            action: EffectAction::Restart {
                service: Service { environment: Box::from(*b"production"), name: Box::from(*b"checkout") },
                operation: 31,
            },
        })
        .unwrap();
    infrastructure::Document::Part(Box::new(PartDocument {
        subject,
        revision: Revision { value: 7 },
        panel: None,
        description: None,
        effect: None,
        resources: List::with_capacity(0),
        effects,
        drift: List::with_capacity(0),
        environments: List::with_capacity(0),
        pools: List::with_capacity(0),
        backend: None,
        made: None,
        request_limit: None,
        staged_limit: None,
    }))
}
