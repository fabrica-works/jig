//! Offers and paged rows have independent admission and allocation bounds.
use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

fn offered(kinds: &[ActKind], capacity: u32) -> List<Offered> {
    let mut result = List::with_capacity(capacity);
    for act in kinds {
        result.push(Offered { act: *act, admission: Admission::Allowed }).unwrap();
    }
    result
}
fn seven() -> Task {
    let mut value = task();
    value.acts = offered(
        &[
            ActKind::Write,
            ActKind::Amend,
            ActKind::TakeOver,
            ActKind::Close,
            ActKind::Stop,
            ActKind::Cancel,
            ActKind::MarkSeen,
        ],
        7,
    );
    value
}
fn exact_packets(value: &Task, bounds: &Bounds) {
    let allocation = u64::try_from(size_of::<Task>()).unwrap() + value.heap(bounds).unwrap();
    let read = ReadValue::Task(Box::new(value.clone()));
    let delivery = Delivery::Task(Box::new(value.clone()));
    let original = (read.clone(), delivery.clone());
    assert_eq!(read.heap(bounds), Some(allocation));
    assert_eq!(delivery.heap(bounds), Some(allocation));
    let mut exact = *bounds;
    exact.document_bytes = u64::try_from(size_of::<ReadValue>()).unwrap() + allocation;
    assert!(payload::fits(&read, &exact));
    exact.document_bytes -= 1;
    assert!(!payload::fits(&read, &exact));
    exact.document_bytes = u64::try_from(size_of::<Delivery>()).unwrap() + allocation;
    assert!(payload::fits(&delivery, &exact));
    exact.document_bytes -= 1;
    assert!(!payload::fits(&delivery, &exact));
    assert_eq!((read, delivery), original);
}
#[test]
fn seven_offers_cross_normal_task_read_and_delivery_whole_with_two_paged_rows() {
    let value = seven();
    let before = value.clone();
    let mut setup = script();
    setup.bounds = Bounds { rows: 2, text: 32, ..setup.bounds };
    exact_packets(&value, &setup.bounds);
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(value.clone())));
    let mut engine = Engine::new(setup, 17).unwrap();
    engine.read(Time::ZERO, 11, Token::new(1), Read::Task { task: 8 }).unwrap();
    engine.open(Time::ZERO, 11, Token::new(2), Subject::Task { task: 8 }).unwrap();
    let outputs = engine.poll(Time::ZERO);
    assert!(outputs.contains(&Up::Read {
        token: Token::new(1),
        terminal: ReadResult::Value(ReadValue::Task(Box::new(value.clone())))
    }));
    assert!(outputs.contains(&Up::Watch {
        token: Token::new(2),
        event: WatchEvent::Delivered(Delivery::Task(Box::new(value.clone())))
    }));
    // List equality compares its entries; reserved capacity is a separate fact.
    let read = outputs
        .iter()
        .find_map(|output| match output {
            Up::Read { token, terminal: ReadResult::Value(ReadValue::Task(document)) } if *token == Token::new(1) => {
                Some(document)
            }
            Up::Answer { .. }
            | Up::Read { .. }
            | Up::Watch { .. }
            | Up::ApplicationAnswer { .. }
            | Up::ApplicationRead { .. }
            | Up::App { .. }
            | Up::Ended { .. } => None,
        })
        .unwrap();
    let delivery = outputs
        .iter()
        .find_map(|output| match output {
            Up::Watch { token, event: WatchEvent::Delivered(Delivery::Task(document)) } if *token == Token::new(2) => {
                Some(document)
            }
            Up::Answer { .. }
            | Up::Read { .. }
            | Up::Watch { .. }
            | Up::ApplicationAnswer { .. }
            | Up::ApplicationRead { .. }
            | Up::App { .. }
            | Up::Ended { .. } => None,
        })
        .unwrap();
    assert_eq!(read.acts.capacity(), 7);
    assert_eq!(delivery.acts.capacity(), 7);
    assert_eq!(value, before);
    assert_eq!(value.acts.capacity(), 7);
    assert_eq!(
        value.acts.iter().map(|v| v.act).collect::<Vec<_>>(),
        vec![
            ActKind::Write,
            ActKind::Amend,
            ActKind::TakeOver,
            ActKind::Close,
            ActKind::Stop,
            ActKind::Cancel,
            ActKind::MarkSeen,
        ]
    );
}
#[test]
fn reserved_closed_offer_ceiling_is_priced_and_one_more_is_refused_whole() {
    let bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    for populated in [false, true] {
        let mut value = seven();
        value.acts = offered(if populated { &[ActKind::Write] } else { &[] }, MAX_OFFERED_ACTS);
        let before = value.clone();
        assert_eq!(crate::offered::heap(&value.acts, &bounds), List::<Offered>::worst_case(MAX_OFFERED_ACTS));
        exact_packets(&value, &bounds);
        value.acts = offered(if populated { &[ActKind::Write] } else { &[] }, MAX_OFFERED_ACTS + 1);
        let oversized = value.clone();
        assert!(!payload::fits(&ReadValue::Task(Box::new(value.clone())), &bounds));
        assert!(!payload::fits(&Delivery::Task(Box::new(value.clone())), &bounds));
        assert_eq!(value, oversized);
        assert_eq!(before.acts.capacity(), MAX_OFFERED_ACTS);
    }
}
fn authority_full() -> OwnerAuthorityDocument {
    let pattern = OwnerPattern {
        segments: list(vec![vec![255, 0, 128].into_boxed_slice(), vec![0, 128, 255].into_boxed_slice()]),
        last: OwnerPatternEnd::Open(vec![128, 255, 0].into_boxed_slice()),
    };
    OwnerAuthorityDocument::complete(OwnerAuthority {
        tools: u64::MAX,
        grants: list(vec![OwnerGrant { connector: u16::MAX, kind: u16::MAX, pattern: pattern.clone() }; 2]),
        delegation: OwnerDelegation {
            kinds: list(vec![OwnerAuthorityExecutor::Charter(u32::MAX), OwnerAuthorityExecutor::Role(u32::MAX)]),
            tasks: u32::MAX,
            depth: u32::MAX,
        },
        spend: u64::MAX,
        deadline: Some(Wall::from_nanos(u64::MAX)),
        notes: u8::MAX,
        note_resources: list(vec![OwnerResourceScope { connector: u16::MAX, pattern }; 2]),
    })
}
fn nested() -> List<Offered> {
    let authority = authority_full();
    list(vec![
        Offered {
            act: ActKind::SetGoal,
            admission: Admission::Bounded {
                within: jig_client_domain::contract::Bounds::Goal {
                    pool_left: u64::MAX,
                    charters: list(vec![u32::MAX, 0]),
                    procedures: list(vec![GoalProcedure { connector: u16::MAX, code: u32::MAX }; 2]),
                    latest: Some(Wall::from_nanos(u64::MAX)),
                    budget: u64::MAX,
                    maximum_budget: u64::MAX,
                    maximum_deadline: None,
                },
                beyond: Beyond::Proposed { to: Recipient::Roles { project: u32::MAX, roles: list(vec![u32::MAX, 0]) } },
            },
        },
        Offered {
            act: ActKind::Amend,
            admission: Admission::Bounded {
                within: jig_client_domain::contract::Bounds::Amend {
                    budget_left: u64::MAX,
                    grants: authority.authority.grants.clone(),
                    implications: list(vec![
                        OwnerKindImplication { connector: u16::MAX, kind: u16::MAX, implies: 0 };
                        2
                    ]),
                    authority: Box::new(authority.clone()),
                    ceiling: Box::new(authority),
                },
                beyond: Beyond::Refused(Refusal::Authority {
                    lacking: Lacking::Budget { needed: u64::MAX, left: 0 },
                    proposable: Some(false),
                }),
            },
        },
        Offered { act: ActKind::TakeOver, admission: Admission::Proposed { to: Recipient::Task(u64::MAX) } },
        Offered { act: ActKind::Cancel, admission: Admission::Proposed { to: Recipient::Party(u64::MAX) } },
        Offered {
            act: ActKind::Close,
            admission: Admission::Proposed { to: Recipient::Role { project: u32::MAX, role: u32::MAX } },
        },
        Offered { act: ActKind::MarkSeen, admission: Admission::Refused(Refusal::NotWaiting) },
        Offered { act: ActKind::Write, admission: Admission::Allowed },
    ])
}
#[test]
fn recursive_goal_and_amend_offers_keep_full_width_holders_rows_and_opaque_bytes_at_exact_packet_bounds() {
    let bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    let mut value = seven();
    value.acts = nested();
    let before = value.clone();
    // Independent allocation arithmetic, including both authority boxes and all raw pattern bytes.
    let pattern = List::<Box<[u8]>>::worst_case(2).unwrap() + 9;
    let grants = List::<OwnerGrant>::worst_case(2).unwrap() + 2 * pattern;
    let authority = grants
        + List::<OwnerAuthorityExecutor>::worst_case(2).unwrap()
        + List::<OwnerResourceScope>::worst_case(2).unwrap()
        + 2 * pattern;
    let goal = List::<u32>::worst_case(2).unwrap() * 2 + List::<GoalProcedure>::worst_case(2).unwrap();
    let amend = grants
        + List::<OwnerKindImplication>::worst_case(2).unwrap()
        + 2 * (u64::try_from(size_of::<OwnerAuthorityDocument>()).unwrap() + authority);
    assert_eq!(
        crate::offered::heap(&value.acts, &bounds),
        Some(List::<Offered>::worst_case(7).unwrap() + goal + amend)
    );
    exact_packets(&value, &bounds);
    assert_eq!(value, before);
}
#[test]
fn nested_offer_lists_and_unused_paged_rows_keep_their_original_independent_bounds() {
    let bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    for dimension in 0..6 {
        let mut value = seven();
        value.acts = nested();
        match dimension {
            0 => {
                let Admission::Bounded { within: jig_client_domain::contract::Bounds::Goal { charters, .. }, .. } =
                    &mut value.acts.get_mut(0).unwrap().admission
                else {
                    panic!("goal")
                };
                *charters = List::with_capacity(3);
            }
            1 => {
                let Admission::Bounded { beyond: Beyond::Proposed { to: Recipient::Roles { roles, .. } }, .. } =
                    &mut value.acts.get_mut(0).unwrap().admission
                else {
                    panic!("roles")
                };
                *roles = List::with_capacity(3);
            }
            2 => {
                let Admission::Bounded {
                    within: jig_client_domain::contract::Bounds::Amend { implications, .. }, ..
                } = &mut value.acts.get_mut(1).unwrap().admission
                else {
                    panic!("amend")
                };
                *implications = List::with_capacity(3);
            }
            3 => {
                let Admission::Bounded { within: jig_client_domain::contract::Bounds::Amend { grants, .. }, .. } =
                    &mut value.acts.get_mut(1).unwrap().admission
                else {
                    panic!("grants")
                };
                *grants = List::with_capacity(3);
            }
            4 => value.lineage = List::with_capacity(3),
            5 => value.dependencies = List::with_capacity(3),
            _ => unreachable!(),
        }
        let before = value.clone();
        assert!(!payload::fits(&ReadValue::Task(Box::new(value.clone())), &bounds));
        assert!(!payload::fits(&Delivery::Task(Box::new(value.clone())), &bounds));
        assert_eq!(value, before);
    }
}
