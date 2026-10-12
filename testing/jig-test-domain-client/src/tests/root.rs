#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "test fixtures collect observed root outputs and locate their variants"
)]
use crate::{Address, Answered, Ask, Domain, Event, JigOut, Limits, Request, first, new, step, take};
use jig_domain_client::{
    boundary::Offset,
    contract::{
        self, Answer, Delivery, DocumentText, Frame, FrameParty, HeldRole, Needs, Outcome, PartyKind, VersionRange,
        WatchEnd, WatchEvent,
    },
    limits::Backoff,
    link::Link,
};
use skein_lib::{Duration, Env, List, Queue, RequestEnvelope, RequestLimits, Time, Token, Wall};
const JIG: jig_domain_client::Limits = jig_domain_client::Limits {
    watches: 4,
    notices: 2,
    notice_life: Duration::from_secs(5),
    linger: Duration::from_secs(5),
    save_delay: Duration::from_secs(1),
    intents: 2,
    fields: 4,
    providers: 2,
    objects: 2,
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
    backoff: Backoff { first: Duration::from_nanos(10), most: Duration::from_nanos(80) },
    heartbeat: Duration::from_nanos(100),
};
fn limits() -> Limits {
    Limits {
        jig: JIG,
        first: first::Limits {
            connector: crate::connector_values::ConnectorLimits {
                rows: 0,
                paths: 0,
                segments: 0,
                segment_bytes: 0,
                bytes: 4096,
            },
            parts: 10,
            text: 32,
            objects: 12,
            watches: 24,
            reads: 24,
            requests: 4,
            rows: 2,
            retry: Duration::from_nanos(10),
        },
        second_parts: 8,
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
fn env() -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }
}
fn drive(root: &mut Domain, event: Event) -> Vec<Request> {
    let mut out = Queue::with_capacity(crate::max_out(&limits()));
    step(root, &env(), event, &mut out);
    out_iter(out)
}
fn out_iter(mut out: Queue<Request>) -> Vec<Request> {
    let mut rows = Vec::new();
    while let Some(row) = out.pop() {
        rows.push(row);
    }
    rows
}
fn taken(root: &mut Domain) -> Vec<Request> {
    let mut out = Queue::with_capacity(crate::max_out(&limits()));
    take(root, &env(), &mut out);
    out_iter(out)
}
fn start(root: &mut Domain, kept: List<crate::store::Record>) -> Token {
    let rows = drive(
        root,
        Event::Start {
            address: Address::Jig(jig_domain_client::address::Address::Inbox { project: None }),
            kept,
            offset: Offset { minutes: 0 },
        },
    );
    rows.iter()
        .find_map(|row| match row {
            Request::Jig(JigOut::Open { watch, subject: contract::Subject::Frame }) => Some(*watch),
            _ => None,
        })
        .unwrap()
}
fn confirm(root: &mut Domain, watch: Token, party: u64) {
    drive(root, Event::Jig(jig_domain_client::Event::Hello { versions: VersionRange { first: 1, last: 1 } }));
    let frame = Frame {
        party: FrameParty {
            number: party,
            kind: PartyKind::Person,
            login: DocumentText::complete(contract::Text::new(Box::from(*b"fixture-login"), 32).unwrap()),
            name: DocumentText::complete(contract::Text::new(Box::from(*b"Pat"), 32).unwrap()),
        },
        projects: List::with_capacity(0),
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        results_seen: 0,
        charters: List::with_capacity(0),
        cut: false,
        updates: contract::Count { value: 0, limited: false },

        retention: Duration::from_secs(1000),
        deployment_role: Some(HeldRole::Policy { role: 1 }),
    };
    drive(
        root,
        Event::Jig(jig_domain_client::Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Frame(Box::new(frame))),
        }),
    );
}
fn ask(root: &mut Domain, asked: Token) -> Vec<Request> {
    drive(
        root,
        Event::Jig(jig_domain_client::Event::Ask {
            ask: jig_domain_client::intents::ByValue {
                asked,
                scope: 7,
                ask: Box::new(contract::Ask::Stop { project: 1, task: 9 }),
            },
        }),
    )
}
#[test]
fn keyed_requests_leave_only_through_the_table_after_their_records() {
    let mut root = new(&limits(), 7);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    assert!(ask(&mut root, Token::new(42)).is_empty());
    let rows = taken(&mut root);
    assert_eq!(rows.len(), 2);
    let Request::Save { record: crate::store::Record::Table { record, .. } } = &rows[0] else {
        unreachable!("save precedes send");
    };
    assert!(record.first_sent.is_some());
    let Request::Send { ask: Ask::Jig(ask), .. } = &rows[1] else {
        unreachable!("one jig request was sent");
    };
    assert_eq!(**ask, contract::Ask::Stop { project: 1, task: 9 });
    assert!(taken(&mut root).is_empty());
}
#[test]
fn a_final_answer_reaches_the_child_that_asked_unread_and_once() {
    let mut root = new(&limits(), 8);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    ask(&mut root, Token::new(42));
    let rows = taken(&mut root);
    let attempt = rows
        .iter()
        .find_map(|r| match r {
            Request::Send { attempt, .. } => Some(*attempt),
            _ => None,
        })
        .unwrap();
    let answer = Answer::Done(Outcome::Changed);
    let rows = drive(
        &mut root,
        Event::Answer { attempt, envelope: RequestEnvelope::Final, answer: Some(Answered::Jig(answer.clone())) },
    );
    assert_eq!(rows, vec![Request::Jig(JigOut::Answered { asked: Token::new(42), answer: answer.clone() })]);
    assert!(
        drive(
            &mut root,
            Event::Answer { attempt, envelope: RequestEnvelope::Final, answer: Some(Answered::Jig(answer)) }
        )
        .is_empty()
    );
    let Request::Erase { key: crate::store::Key::Table(_) } = &taken(&mut root)[0] else {
        unreachable!("final erases its request record");
    };
}
#[test]
fn a_reload_parks_kept_requests_until_jig_confirms_their_party() {
    let mut old = new(&limits(), 9);
    let watch = start(&mut old, List::with_capacity(0));
    confirm(&mut old, watch, 7);
    ask(&mut old, Token::new(42));
    let rows = taken(&mut old);
    let record = rows
        .iter()
        .filter_map(|r| match r {
            Request::Save { record: crate::store::Record::Table { version, record } } => {
                Some(crate::store::Record::Table { version: *version, record: record.clone() })
            }
            _ => None,
        })
        .next_back()
        .unwrap();
    let mut kept = List::with_capacity(1);
    kept.push(record).unwrap();
    let mut root = new(&limits(), 10);
    let watch = start(&mut root, kept);
    assert!(taken(&mut root).is_empty());
    confirm(&mut root, watch, 8);
    assert!(taken(&mut root).is_empty());
    confirm(&mut root, watch, 7);
    let sends = taken(&mut root);
    let attempt = sends
        .iter()
        .find_map(|r| match r {
            Request::Send { attempt, .. } => Some(*attempt),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        drive(
            &mut root,
            Event::Answer {
                attempt,
                envelope: RequestEnvelope::Final,
                answer: Some(Answered::Jig(Answer::Done(Outcome::Changed)))
            }
        ),
        vec![Request::Fact { fact: crate::Fact::KeptAnswered }]
    );
}
#[test]
fn link_and_confirmation_reach_both_children_and_park_the_table() {
    let mut root = new(&limits(), 11);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    assert_eq!(root.first().link(), Link::Live);
    assert_eq!(root.second().link(), Link::Live);
    assert_eq!(root.first().party(), Some(7));
    assert_eq!(root.second().party(), Some(7));
    ask(&mut root, Token::new(42));
    drive(&mut root, Event::Jig(jig_domain_client::Event::Ended { watch, end: WatchEnd::Dropped }));
    assert_eq!(root.first().link(), Link::Offline);
    assert_eq!(root.second().link(), Link::Offline);
    for row in taken(&mut root) {
        if let Request::Send { .. } = row {
            unreachable!("offline requests are parked");
        }
    }
    drive(&mut root, Event::SignedOut);
    assert_eq!(root.first().party(), None);
    assert_eq!(root.second().party(), None);
}

#[test]
fn a_current_signed_out_envelope_tells_jig_and_both_siblings() {
    let mut root = new(&limits(), 12);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    ask(&mut root, Token::new(42));
    let rows = taken(&mut root);
    let Request::Send { attempt, .. } = &rows[1] else {
        unreachable!("one request is in flight");
    };
    drive(&mut root, Event::Answer { attempt: *attempt, envelope: RequestEnvelope::SignedOut, answer: None });
    assert_eq!(root.first().party(), None);
    assert_eq!(root.second().party(), None);
    for row in taken(&mut root) {
        if let Request::Send { .. } = row {
            unreachable!("signed out requests remain parked");
        }
    }
}

fn ready(seed: u64) -> Domain {
    let mut root = new(&limits(), seed);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    root
}
fn family_read(rows: &[Request]) -> Token {
    rows.iter()
        .find_map(|row| match row {
            Request::First(first::Out::Read { read, .. }) => Some(*read),
            _ => None,
        })
        .unwrap()
}
fn family_watch(rows: &[Request]) -> Token {
    rows.iter()
        .find_map(|row| match row {
            Request::First(first::Out::Open { watch, .. }) => Some(*watch),
            _ => None,
        })
        .unwrap()
}
fn item(revision: u64) -> crate::family::Document {
    let mut proposals = List::with_capacity(1);
    proposals.push(contract::ProposalKey { proposer: contract::Proposer::Task(9), number: 2 }).unwrap();
    crate::family::Document::Item(crate::family::Item {
        project: 3,
        key: 5,
        revision: contract::Revision { value: revision },
        state: crate::family::ItemState::Ready,
        value: 13,
        task: 9,
        proposals,
        touchable: true,
    })
}
fn family_page(root: &mut Domain) -> Vec<Request> {
    drive(root, Event::Went { place: crate::Place::Address(Address::First(first::Address { project: 3, item: 5 })) })
}
#[test]
fn a_jig_page_routes_every_part_kind_and_forgets_its_generation() {
    let mut root = ready(21);
    let opened = drive(
        &mut root,
        Event::Went { place: crate::Place::Address(Address::Jig(jig_domain_client::address::Address::Deployment)) },
    );
    let watch = opened
        .iter()
        .find_map(|row| match row {
            Request::Jig(JigOut::Open { watch, subject: contract::Subject::Deployment }) => Some(*watch),
            _ => None,
        })
        .unwrap();
    let mut parts = List::with_capacity(10);
    for part in [
        contract::Part::Resource { connector: crate::FIRST, path: resource_path(b"name") },
        contract::Part::Kind { connector: crate::FIRST, kind: 4 },
        contract::Part::Panel { connector: crate::FIRST, task: 9 },
        contract::Part::Result { connector: crate::FIRST, task: 9 },
        contract::Part::Effect {
            connector: crate::FIRST,
            proposal: contract::ProposalKey { proposer: contract::Proposer::Task(9), number: 2 },
        },
        contract::Part::Hold { connector: crate::FIRST, task: 9 },
        contract::Part::Project { connector: crate::FIRST, project: 3 },
        contract::Part::Deployment { connector: crate::FIRST },
        contract::Part::Resource { connector: crate::SECOND, path: resource_path(b"other") },
        contract::Part::Kind { connector: crate::SECOND, kind: 6 },
    ] {
        parts.push(part).unwrap();
    }
    let expected = parts.clone();
    let rows = drive(
        &mut root,
        Event::Jig(jig_domain_client::Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Deployment(Box::new(contract::Deployment {
                hosts: List::with_capacity(0),
                accounts: List::with_capacity(0),
                capacity: List::with_capacity(0),
                proposals: 0,
                refusals: List::with_capacity(0),
                parts,
            }))),
        }),
    );
    let generation = root.jig().generation();
    for part in &expected.as_slice()[..8] {
        assert!(root.first().contains(part, generation));
    }
    assert!(root.second().contains(&expected.as_slice()[8]));
    assert!(root.second().contains(&expected.as_slice()[9]));
    assert_eq!(
        rows.iter()
            .filter(|row| match row {
                Request::First(first::Out::Read { .. }) => true,
                _ => false,
            })
            .count(),
        6
    );
    assert_eq!(root.first().part(&expected.as_slice()[2]).unwrap().value, first::Value::Loading);
    let read = family_read(&rows);
    drive(&mut root, Event::First(first::Event::Read { read, result: first::ReadResult::Unknown }));
    assert_eq!(root.first().part(&expected.as_slice()[2]).unwrap().value, first::Value::Unknown);
    let rows = drive(
        &mut root,
        Event::Went {
            place: crate::Place::Address(Address::Jig(jig_domain_client::address::Address::Inbox { project: None })),
        },
    );
    for part in &expected.as_slice()[..8] {
        assert!(!root.first().contains(part, generation));
    }
    assert!(!root.second().contains(&expected.as_slice()[8]));
    assert!(!root.second().contains(&expected.as_slice()[9]));
    assert_eq!(
        rows.iter()
            .filter(|row| match row {
                Request::First(first::Out::Close { .. }) => true,
                _ => false,
            })
            .count(),
        6
    );
    drive(&mut root, Event::First(first::Event::Read { read, result: first::ReadResult::Value(item(1)) }));
    assert!(root.first().part(&expected.as_slice()[2]).is_none());
}
#[test]
fn a_family_page_hands_its_task_and_proposal_to_jigs_working_set() {
    let mut root = ready(22);
    let rows_initial = family_page(&mut root);
    let read = family_read(&rows_initial);
    let rows = drive(&mut root, Event::First(first::Event::Read { read, result: first::ReadResult::Value(item(1)) }));
    assert!(root.jig().objects().find(contract::ObjectKey::Task(9)).is_some());
    assert!(
        root.jig()
            .objects()
            .find(contract::ObjectKey::Proposal(contract::ProposalKey {
                proposer: contract::Proposer::Task(9),
                number: 2
            }))
            .is_some()
    );
    assert!(rows.iter().any(|row| match row {
        Request::Jig(JigOut::Open { subject: contract::Subject::Chips { .. }, .. }) => true,
        _ => false,
    }));
    assert!(rows.iter().any(|row| match row {
        Request::Jig(JigOut::Open { subject: contract::Subject::Inbox { .. }, .. }) => true,
        _ => false,
    }));
    let watch = family_watch(&rows_initial);
    let rows =
        drive(&mut root, Event::First(first::Event::Watch { watch, event: first::WatchEvent::Delivered(item(1)) }));
    assert!(!rows.iter().any(|row| match row {
        Request::Jig(JigOut::Open { .. }) => true,
        _ => false,
    }));
}
#[test]
fn family_touch_refuses_a_moved_revision_and_final_answers_stay_with_the_family() {
    let mut root = ready(23);
    let rows = family_page(&mut root);
    drive(
        &mut root,
        Event::First(first::Event::Read { read: family_read(&rows), result: first::ReadResult::Value(item(2)) }),
    );
    let id = root.first().page().unwrap();
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::Touch {
                object: id.token(),
                revision: contract::Revision { value: 1 },
            }),
        },
    );
    assert_eq!(
        root.first().object(id).unwrap().card,
        first::Card::Refused { answer: first::Answer::Moved { item: 5, revision: contract::Revision { value: 2 } } }
    );
    assert!(taken(&mut root).is_empty());
    assert!(
        drive(
            &mut root,
            Event::Act {
                action: crate::Action::First(first::Act::Touch {
                    object: id.token(),
                    revision: contract::Revision { value: 2 }
                })
            }
        )
        .is_empty()
    );
    let rows = taken(&mut root);
    let Request::Save { record: crate::store::Record::Table { .. } } = &rows[0] else { panic!("record first") };
    let Request::Send { attempt, ask: Ask::First(ask), .. } = &rows[1] else { panic!("family request") };
    assert_eq!(
        *ask,
        first::Ask::Touch { project: 3, item: 5, revision: contract::Revision { value: 2 }, parameters: None }
    );
    let answer = first::Answer::Touched { item: 5, revision: contract::Revision { value: 3 } };
    drive(
        &mut root,
        Event::Answer {
            attempt: *attempt,
            envelope: RequestEnvelope::Final,
            answer: Some(Answered::First(answer.clone())),
        },
    );
    assert_eq!(root.first().object(id).unwrap().card, first::Card::Done { answer });
}
#[test]
fn panel_parameters_are_the_familys_revision_naming_request() {
    let mut root = ready(24);
    let part = contract::Part::Panel { connector: crate::FIRST, task: 9 };
    let rows = drive(
        &mut root,
        Event::First(first::Event::Show { part: part.clone(), generation: 4, scope: crate::family::Scope::Project(3) }),
    );
    let subject =
        crate::family::Subject { scope: crate::family::Scope::Project(3), kind: crate::family::Kind::Panel(9) };
    drive(
        &mut root,
        Event::First(first::Event::Read {
            read: family_read(&rows),
            result: first::ReadResult::Value(crate::family::Document::Part(crate::family::PartDocument {
                subject,
                revision: contract::Revision { value: 4 },
                value: 13,
                parameters: Some(crate::family::Parameters {
                    item: 5,
                    revision: contract::Revision { value: 2 },
                    value: 13,
                    amendable: true,
                }),
            })),
        }),
    );
    let object = root.first().find(subject).unwrap().token();
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::Parameters {
                object,
                revision: contract::Revision { value: 2 },
                value: 21,
            }),
        },
    );
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::Touch { object, revision: contract::Revision { value: 2 } }),
        },
    );
    let rows = taken(&mut root);
    let Request::Send { ask: Ask::First(ask), .. } = &rows[1] else { panic!("family parameters request") };
    assert_eq!(
        *ask,
        first::Ask::Touch { project: 3, item: 5, revision: contract::Revision { value: 2 }, parameters: Some(21) }
    );
}

#[test]
fn closing_family_watches_keep_their_terminal_obligation_and_stale_tokens_do_nothing() {
    let mut root = ready(25);
    let rows = family_page(&mut root);
    let watch = family_watch(&rows);
    let read = family_read(&rows);
    drive(
        &mut root,
        Event::Went {
            place: crate::Place::Address(Address::Jig(jig_domain_client::address::Address::Inbox { project: None })),
        },
    );
    drive(&mut root, Event::First(first::Event::Watch { watch, event: first::WatchEvent::Delivered(item(2)) }));
    assert!(root.first().page().is_none());
    assert!(drive(&mut root, Event::First(first::Event::Ended { watch, end: first::WatchEnd::Closed })).is_empty());
    assert!(drive(&mut root, Event::First(first::Event::Ended { watch, end: first::WatchEnd::Closed })).is_empty());
    drive(&mut root, Event::First(first::Event::Read { read, result: first::ReadResult::Value(item(2)) }));
    drive(&mut root, Event::First(first::Event::Read { read, result: first::ReadResult::Value(item(2)) }));
    let rows = family_page(&mut root);
    let current = family_watch(&rows);
    assert_ne!(watch, current);
    drive(&mut root, Event::First(first::Event::Ended { watch: current, end: first::WatchEnd::Dropped }));
    assert_eq!(crate::next_deadline(&root), Some(Time::ZERO.saturating_add(Duration::from_nanos(10))));
    let mut out = Queue::with_capacity(crate::max_out(&limits()));
    let mut later = env();
    later.now = Time::ZERO.saturating_add(Duration::from_nanos(10));
    crate::fire(&mut root, &later, &mut out);
    assert_eq!(family_watch(&out_iter(out)), current);
}

#[test]
fn an_unknown_family_request_inspects_before_another_touch() {
    let mut root = ready(26);
    let rows = family_page(&mut root);
    drive(
        &mut root,
        Event::First(first::Event::Read { read: family_read(&rows), result: first::ReadResult::Value(item(2)) }),
    );
    let object = root.first().page().unwrap();
    let action =
        crate::Action::First(first::Act::Touch { object: object.token(), revision: contract::Revision { value: 2 } });
    drive(&mut root, Event::Act { action: action.clone() });
    let first::Card::Deciding { request, .. } = root.first().object(object).unwrap().card else {
        panic!("the family owns its deciding request");
    };
    let rows = drive(
        &mut root,
        Event::First(first::Event::Progress { request, progress: skein_lib::RequestProgress::Unknown }),
    );
    assert_eq!(root.first().object(object).unwrap().card, first::Card::Unknown);
    assert_eq!(root.first().object(object).unwrap().value, first::Value::Loading);
    assert!(drive(&mut root, Event::Act { action }).is_empty());
    drive(
        &mut root,
        Event::First(first::Event::Read { read: family_read(&rows), result: first::ReadResult::Value(item(3)) }),
    );
    assert_eq!(root.first().object(object).unwrap().card, first::Card::Open);
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::Touch {
                object: object.token(),
                revision: contract::Revision { value: 2 },
            }),
        },
    );
    assert_eq!(
        root.first().object(object).unwrap().card,
        first::Card::Refused { answer: first::Answer::Moved { item: 5, revision: contract::Revision { value: 3 } } }
    );
}

#[test]
fn family_capacities_refuse_overflow_and_resource_words_past_the_owner_bound() {
    let mut bounds = limits();
    bounds.first.objects = u32::MAX;
    assert!(crate::worst_case(&bounds).is_none());
    bounds = limits();
    bounds.first.retry = Duration::ZERO;
    assert!(crate::worst_case(&bounds).is_none());
    let mut root = ready(27);
    let part = contract::Part::Resource { connector: crate::FIRST, path: resource_path(&[b'x'; 33]) };
    let rows = drive(
        &mut root,
        Event::First(first::Event::Show { part: part.clone(), generation: 4, scope: crate::family::Scope::Deployment }),
    );
    assert_eq!(rows, vec![Request::First(first::Out::Fact { fact: first::Fact::Full })]);
    assert!(!root.first().contains(&part, 4));
    let mut side = crate::second::Side::new(1, 1, 32, 1);
    assert!(!side.show(part, 4));
    let part = contract::Part::Kind { connector: crate::SECOND, kind: 1 };
    assert!(side.show(part.clone(), 4));
    assert!(side.show(part, 4));
    assert!(!side.show(contract::Part::Kind { connector: crate::SECOND, kind: 2 }, 4));
}

fn restored_requests(count: u32) -> List<crate::store::Record> {
    let mut old = new(&limits(), 30);
    let watch = start(&mut old, List::with_capacity(0));
    confirm(&mut old, watch, 7);
    let mut kept = List::with_capacity(count);
    for n in 0..count {
        drive(
            &mut old,
            Event::Jig(jig_domain_client::Event::Ask {
                ask: jig_domain_client::intents::ByValue {
                    asked: Token::new(u64::from(n)),
                    scope: 7,
                    ask: Box::new(contract::Ask::Stop { project: 1, task: 9 + u64::from(n) }),
                },
            }),
        );
        for row in taken(&mut old) {
            if let Request::Save { record: crate::store::Record::Table { version, record } } = row {
                kept.push(crate::store::Record::Table { version, record }).unwrap();
            }
        }
    }
    kept
}

#[test]
fn restored_foreign_scope_counts_are_routed_without_root_page_policy() {
    let mut root = new(&limits(), 31);
    let watch = start(&mut root, restored_requests(2));
    confirm(&mut root, watch, 8);
    assert_eq!(root.jig().page(), &jig_domain_client::pages::Page::SomeoneElse);
    assert_eq!(root.first().party(), Some(8));
    assert!(taken(&mut root).is_empty());
    confirm(&mut root, watch, 7);
    assert_eq!(
        taken(&mut root)
            .iter()
            .filter(|r| match r {
                Request::Send { .. } => true,
                _ => false,
            })
            .count(),
        2
    );
}

#[test]
fn restored_own_scope_sends_and_retires_each_final_request_once() {
    let mut root = new(&limits(), 32);
    let watch = start(&mut root, restored_requests(2));
    confirm(&mut root, watch, 7);
    assert!(work_page(root.jig().page()));
    let sends: Vec<_> = taken(&mut root)
        .into_iter()
        .filter_map(|row| match row {
            Request::Send { attempt, .. } => Some(attempt),
            _ => None,
        })
        .collect();
    assert_eq!(sends.len(), 2);
    for attempt in sends {
        let out = drive(
            &mut root,
            Event::Answer {
                attempt,
                envelope: RequestEnvelope::Final,
                answer: Some(Answered::Jig(Answer::Done(Outcome::Changed))),
            },
        );
        assert!(out.contains(&Request::Fact { fact: crate::Fact::KeptAnswered }));
    }
    confirm(&mut root, watch, 8);
    assert!(work_page(root.jig().page()));
}

#[test]
fn retention_unknown_retires_the_last_foreign_scope_and_clears_its_warning() {
    let mut root = new(&limits(), 33);
    let watch = start(&mut root, restored_requests(1));
    confirm(&mut root, watch, 8);
    assert_eq!(root.jig().page(), &jig_domain_client::pages::Page::SomeoneElse);
    let mut future = env();
    future.now = Time::from_nanos(1_002_000_000_000);
    future.wall = Wall::from_nanos(1_002_000_000_000);
    let mut out = Queue::with_capacity(crate::max_out(&limits()));
    // Only the table expiry is relevant; the frame remains a public confirmed fact.
    root.table.fire(future.now, future.wall);
    take(&mut root, &future, &mut out);
    assert!(work_page(root.jig().page()));
}

fn work_page(page: &jig_domain_client::pages::Page) -> bool {
    match page {
        jig_domain_client::pages::Page::Work { .. } => true,
        jig_domain_client::pages::Page::Starting
        | jig_domain_client::pages::Page::Elsewhere
        | jig_domain_client::pages::Page::SigningIn
        | jig_domain_client::pages::Page::Missing
        | jig_domain_client::pages::Page::OutOfDate
        | jig_domain_client::pages::Page::SomeoneElse => false,
    }
}

#[test]
fn a_raw_entrance_refusal_finishes_the_original_family_request_once_with_its_facts() {
    let mut root = ready(29);
    let rows = family_page(&mut root);
    drive(
        &mut root,
        Event::First(first::Event::Read { read: family_read(&rows), result: first::ReadResult::Value(item(2)) }),
    );
    let id = root.first().page().unwrap();
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::Touch {
                object: id.token(),
                revision: contract::Revision { value: 2 },
            }),
        },
    );
    let rows = taken(&mut root);
    let Request::Save { .. } = &rows[0] else { panic!("retention precedes send") };
    let Request::Send { attempt, ask: Ask::First(_), .. } = &rows[1] else { panic!("family request") };
    let answer = first::Answer::Refused(
        contract::EntranceRefusal::new(contract::Refusal::Authority {
            lacking: contract::Lacking::Budget { needed: 19, left: 3 },
            proposable: Some(false),
        })
        .unwrap(),
    );
    drive(
        &mut root,
        Event::Answer {
            attempt: *attempt,
            envelope: RequestEnvelope::Final,
            answer: Some(Answered::First(answer.clone())),
        },
    );
    assert_eq!(root.first().object(id).unwrap().card, first::Card::Refused { answer: answer.clone() });
    let shown = crate::shown(&root);
    drive(
        &mut root,
        Event::Answer {
            attempt: *attempt,
            envelope: RequestEnvelope::Final,
            answer: Some(Answered::First(first::Answer::Gone)),
        },
    );
    assert_eq!(root.first().object(id).unwrap().card, first::Card::Refused { answer });
    assert_eq!(crate::shown(&root), shown);
    assert!(taken(&mut root).iter().all(|request| match request {
        Request::Send { .. } => false,
        _ => true,
    }));
}

#[test]
fn outbox_entries_route_as_scoped_numeric_subjects_independently_of_effect_proposals() {
    use crate::family::{Kind, Scope, Subject};
    let mut root = ready(121);
    let opened = drive(
        &mut root,
        Event::Went { place: crate::Place::Address(Address::Jig(jig_domain_client::address::Address::Deployment)) },
    );
    let watch = opened
        .iter()
        .find_map(|request| match request {
            Request::Jig(JigOut::Open { watch, subject: contract::Subject::Deployment }) => Some(*watch),
            _ => None,
        })
        .unwrap();
    let entry = u64::MAX;
    let proposal = contract::ProposalKey { proposer: contract::Proposer::Task(entry), number: entry };
    let parts = [
        contract::Part::Outbox { connector: crate::FIRST, scope: contract::PartScope::Project(3), entry },
        contract::Part::Outbox { connector: crate::FIRST, scope: contract::PartScope::Project(4), entry },
        contract::Part::Outbox { connector: crate::FIRST, scope: contract::PartScope::Deployment, entry },
        contract::Part::Effect { connector: crate::FIRST, proposal },
    ];
    let mut retained = List::with_capacity(4);
    for part in &parts {
        retained.push(part.clone()).unwrap();
    }
    let rows = drive(
        &mut root,
        Event::Jig(jig_domain_client::Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Deployment(Box::new(contract::Deployment {
                hosts: List::with_capacity(0),
                accounts: List::with_capacity(0),
                capacity: List::with_capacity(0),
                proposals: 0,
                refusals: List::with_capacity(0),
                parts: retained,
            }))),
        }),
    );
    let expected = [
        Subject { scope: Scope::Project(3), kind: Kind::Outbox(entry) },
        Subject { scope: Scope::Project(4), kind: Kind::Outbox(entry) },
        Subject { scope: Scope::Deployment, kind: Kind::Outbox(entry) },
        Subject { scope: Scope::Deployment, kind: Kind::Effect(proposal) },
    ];
    for (part, subject) in parts.iter().zip(expected) {
        assert_eq!(root.first().part(part).unwrap().subject, subject);
        assert!(rows.iter().any(|row| match row {
            Request::First(first::Out::Read { subject: requested, .. }) => *requested == subject,
            _ => false,
        }));
        assert!(root.first().contains(part, root.jig().generation()));
    }
    let generation = root.jig().generation();
    drive(
        &mut root,
        Event::Went {
            place: crate::Place::Address(Address::Jig(jig_domain_client::address::Address::Inbox { project: None })),
        },
    );
    for part in &parts {
        assert!(!root.first().contains(part, generation));
    }
}

#[test]
fn actual_application_object_form_keeps_explicit_zero_observed_revision_and_original_final_child() {
    use crate::connector_values::{
        ApplicationObject, ApplicationOutcome, ClientDocument, ConnectorAnswer, ConnectorDocument,
    };
    use crate::family::{Document, Kind, Scope, Subject};
    let mut root = ready(91);
    let opened = family_page(&mut root);
    let subject = Subject { scope: Scope::Project(3), kind: Kind::Item(5) };
    let value = Document::Connector(ConnectorDocument {
        subject,
        value: Box::new(ClientDocument::Object { object: 5, now: ApplicationObject { revision: 17, value: 64 } }),
    });
    drive(
        &mut root,
        Event::First(first::Event::Read {
            read: family_read(&opened),
            result: first::ReadResult::Value(value.clone()),
        }),
    );
    let id = root.first().find(subject).unwrap();
    assert_eq!(root.first().object(id).unwrap().value, first::Value::Known(value));
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::Parameters {
                object: id.token(),
                revision: contract::Revision { value: 17 },
                value: 0,
            }),
        },
    );
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::Touch {
                object: id.token(),
                revision: contract::Revision { value: 17 },
            }),
        },
    );
    let sent = taken(&mut root);
    let (attempt, ask) = sent
        .iter()
        .find_map(|request| match request {
            Request::Send { attempt, ask: Ask::First(ask), .. } => Some((*attempt, ask)),
            _ => None,
        })
        .expect("one keyed complete application request");
    assert_eq!(
        ask,
        &crate::family::Ask::ChangeObject {
            project: 3,
            object: 5,
            revision: contract::Revision { value: 17 },
            value: 0
        }
    );
    let answer = crate::family::Answer::Connector(ConnectorAnswer {
        value: ApplicationOutcome::Changed { object: 5, now: ApplicationObject { revision: 18, value: 0 } },
    });
    drive(
        &mut root,
        Event::Answer { attempt, envelope: RequestEnvelope::Final, answer: Some(Answered::First(answer.clone())) },
    );
    assert_eq!(root.first().object(id).unwrap().card, first::Card::Done { answer: answer.clone() });
    drive(
        &mut root,
        Event::Answer {
            attempt,
            envelope: RequestEnvelope::Final,
            answer: Some(Answered::First(crate::family::Answer::Connector(ConnectorAnswer {
                value: ApplicationOutcome::Role { held: u32::MAX },
            }))),
        },
    );
    assert_eq!(root.first().object(id).unwrap().card, first::Card::Done { answer });
}

#[test]
fn whole_owner_parts_require_original_inner_identity_and_keep_unversioned_snapshots_without_forms() {
    use crate::connector_values::{ClientDocument, ConnectorDocument};
    use crate::family::{Document, Kind, Scope, Subject};
    let mut root = ready(92);
    let part = contract::Part::Panel { connector: crate::FIRST, task: 9 };
    let opened = drive(&mut root, Event::First(first::Event::Show { part, generation: 1, scope: Scope::Project(3) }));
    let subject = Subject { scope: Scope::Project(3), kind: Kind::Panel(9) };
    let watch = family_watch(&opened);
    for bad in [
        ClientDocument::Panel {
            task: 10,
            procedure: None,
            result: None,
            resources: List::with_capacity(0),
            effects: List::with_capacity(0),
        },
        ClientDocument::Panel {
            task: 9,
            procedure: None,
            result: None,
            resources: List::with_capacity(1),
            effects: List::with_capacity(0),
        },
    ] {
        let outputs = drive(
            &mut root,
            Event::First(first::Event::Watch {
                watch,
                event: first::WatchEvent::Delivered(Document::Connector(ConnectorDocument {
                    subject,
                    value: Box::new(bad),
                })),
            }),
        );
        let mut invalid = false;
        for request in &outputs {
            if let Request::First(first::Out::Fact { fact: first::Fact::InvalidDocument }) = request {
                invalid = true;
            }
        }
        assert!(invalid);
        assert_eq!(root.first().object(root.first().find(subject).unwrap()).unwrap().value, first::Value::Loading);
    }
    for result in [None, Some(0), Some(u16::MAX)] {
        let document = Document::Connector(ConnectorDocument {
            subject,
            value: Box::new(ClientDocument::Panel {
                task: 9,
                procedure: None,
                result,
                resources: List::with_capacity(0),
                effects: List::with_capacity(0),
            }),
        });
        assert_eq!(document.revision(), None);
        drive(
            &mut root,
            Event::First(first::Event::Watch { watch, event: first::WatchEvent::Delivered(document.clone()) }),
        );
        let id = root.first().find(subject).unwrap();
        assert_eq!(root.first().object(id).unwrap().value, first::Value::Known(document));
        drive(
            &mut root,
            Event::Act {
                action: crate::Action::First(first::Act::Touch {
                    object: id.token(),
                    revision: contract::Revision { value: 0 },
                }),
            },
        );
        for request in taken(&mut root) {
            if let Request::Send { ask: Ask::First(_), .. } = request {
                panic!("an unversioned snapshot must not submit an Object form");
            }
        }
    }
}

fn resource_path(bytes: &[u8]) -> List<Box<[u8]>> {
    let mut rows = List::with_capacity(1);
    rows.push(Box::from(bytes)).unwrap();
    rows
}
