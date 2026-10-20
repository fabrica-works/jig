use super::{frame, script, text};
use crate::{Bounds, Engine, Invalid, Script, owner_project};
use jig_client_domain::contract::*;
use skein_lib::{List, Time, Token};
fn source() -> OwnerProject {
    OwnerProject {
        number: 1,
        acts: List::with_capacity(2),
        resources: List::with_capacity(2),
        policy: OwnerProjectPolicy {
            templates: skein_lib::List::with_capacity(0),
            ceiling: OwnerAuthorityDocument::complete(OwnerAuthority {
                models: None,
                tools: u64::MAX,
                grants: List::with_capacity(0),
                delegation: OwnerDelegation { kinds: List::with_capacity(0), tasks: u32::MAX, depth: u32::MAX },
                spend: u64::MAX,
                deadline: None,
                notes: u8::MAX,
                note_resources: List::with_capacity(0),
            }),
            period_spend: u64::MAX,
            escalation_role: None,
            projections: List::with_capacity(2),
            roles: List::with_capacity(2),
            requirements: List::with_capacity(2),
            permissions: None,
            permissions_cut: false,
            cut: true,
        },
        members: List::with_capacity(2),
        current_period: None,
        pools: List::with_capacity(2),
        configured_period: 0,
        configured_pools: List::with_capacity(2),
        configured_pools_cut: false,
        spend: List::with_capacity(2),
        recurring: List::with_capacity(2),
        cut: true,
    }
}
#[test]
fn owner_project_kit_prices_every_empty_reserved_list_at_exact_bytes_and_refuses_one_under_unchanged() {
    let value = source();
    let bytes = core::mem::size_of::<OwnerProject>() as u64
        + List::<Offered>::worst_case(2).unwrap()
        + List::<OwnerProjectResource>::worst_case(2).unwrap()
        + List::<OwnerGrant>::worst_case(2).unwrap()
        + List::<OwnerProjectPolicyRole>::worst_case(2).unwrap()
        + List::<PolicyRequirement>::worst_case(2).unwrap()
        + List::<OwnerProjectMember>::worst_case(2).unwrap()
        + List::<PoolLedger>::worst_case(2).unwrap()
        + List::<PoolLedger>::worst_case(2).unwrap()
        + List::<OwnerProjectSpend>::worst_case(2).unwrap()
        + List::<Recurring>::worst_case(2).unwrap();
    let mut bounds = Bounds { rows: 2, segments: 2, document_bytes: bytes, ..Bounds::default() };
    assert_eq!(owner_project::bytes(&value, &bounds), Some(bytes));
    assert!(owner_project::fits(&value, &bounds));
    assert_eq!(owner_project::worst_case(&bounds), Some(bytes));
    let before = value.clone();
    bounds.document_bytes -= 1;
    assert!(!owner_project::fits(&value, &bounds));
    assert_eq!(value, before);
    bounds.document_bytes = u64::MAX;
    bounds.rows = 1;
    assert!(!owner_project::fits(&value, &bounds));
    assert_eq!(value, before);
}
#[test]
fn owner_project_kit_checks_final_member_text_and_whole_segment_and_nested_reserved_capacities() {
    let mut value = source();
    value
        .members
        .push(OwnerProjectMember {
            party: u64::MAX,
            kind: PartyKind::Service,
            role: HeldRole::Policy { role: u32::MAX },
            login: DocumentText { bytes: text(UTF8_TEXTS[0]), cut: true },
            name: DocumentText::complete(text(b"")),
        })
        .unwrap();
    let mut segments = List::with_capacity(2);
    segments.push(Box::from([255, 0, b'/', 254])).unwrap();
    value
        .resources
        .push(OwnerProjectResource {
            part: OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Resource, key: OwnerPartKey::Path(segments) },
            role: OwnerProjectResourceRole::Unavailable,
        })
        .unwrap();
    let mut bounds =
        Bounds { rows: 2, authority: 2, segments: 2, text: 4, document_bytes: u64::MAX, ..Bounds::default() };
    assert!(owner_project::fits(&value, &bounds));
    bounds.text = 3;
    assert!(!owner_project::fits(&value, &bounds));
    bounds.text = 4;
    let original = value.clone();
    value.policy.ceiling.authority.note_resources = List::with_capacity(3);
    assert!(!owner_project::fits(&value, &bounds));
    value = original.clone();
    value.recurring = List::with_capacity(3);
    assert!(!owner_project::fits(&value, &bounds));
    value = original.clone();
    value.resources.get_mut(0).unwrap().part.key = OwnerPartKey::Path(List::with_capacity(3));
    assert!(!owner_project::fits(&value, &bounds));
    assert_eq!(original.members.get(0).unwrap().login.bytes(), UTF8_TEXTS[0]);
}
#[test]
fn owner_project_kit_original_public_read_keeps_whole_source_and_rejects_unadmitted_truth_before_delivery() {
    let mut setup = script();
    let value = source();
    setup.truth.owner_projects.insert(1, value.clone());
    let whole_bytes = owner_project::bytes(&value, &setup.bounds).unwrap();
    let envelope = whole_bytes + core::mem::size_of::<ReadValue>() as u64;
    setup.bounds.document_bytes = envelope;
    let mut engine = Engine::new(setup, 9123).unwrap();
    engine.read(Time::ZERO, 11, Token::new(1), Read::Project { project: 1 }).unwrap();
    let out = engine.poll(Time::ZERO);
    assert!(out.iter().any(|up|matches!(up,crate::Up::Read {terminal:ReadResult::Value(ReadValue::OwnerProject(received)),..} if received.as_ref()==&value)));
    assert_eq!(engine.truth().owner_projects.get(&1), Some(&value));
    assert!(engine.truth().projects.is_empty());
    let mut short = script();
    short.truth.owner_projects.insert(1, value.clone());
    short.bounds.document_bytes = envelope - 1;
    let mut refused = Engine::new(short, 9126).unwrap();
    refused.read(Time::ZERO, 11, Token::new(2), Read::Project { project: 1 }).unwrap();
    assert!(
        refused
            .poll(Time::ZERO)
            .iter()
            .any(|up| matches!(up, crate::Up::Read { terminal: ReadResult::None(ReadNone::Busy), .. }))
    );
    assert_eq!(refused.truth().owner_projects.get(&1), Some(&value));
    let mut setup = Script::default();
    setup.truth.frames.insert(1, frame(1));
    setup.truth.owner_projects.insert(1, value.clone());
    setup.bounds.rows = 1;
    assert!(matches!(Engine::new(setup, 9124), Err(Invalid::Payload)));
    let mut setup = script();
    setup.truth.owner_projects.insert(0, value);
    assert!(matches!(Engine::new(setup, 9125), Err(Invalid::Identity)));
}
#[test]
fn owner_project_kit_distinguishes_missing_empty_and_full_width_permission_capacities_and_configured_pool_loss() {
    let mut value = source();
    let mut bounds = Bounds { rows: 2, segments: 2, text: 4, document_bytes: u64::MAX, ..Bounds::default() };
    let absent = owner_project::bytes(&value, &bounds).unwrap();
    value.policy.permissions = Some(List::with_capacity(2));
    assert_eq!(owner_project::bytes(&value, &bounds), Some(absent + List::<PolicyPermission>::worst_case(2).unwrap()));
    value
        .policy
        .permissions
        .as_mut()
        .unwrap()
        .push(PolicyPermission { connector: u16::MAX, permission: u16::MAX, role: u32::MAX })
        .unwrap();
    value.policy.permissions_cut = true;
    value.configured_period = u64::MAX;
    value.configured_pools_cut = true;
    let whole = value.clone();
    let bytes = owner_project::bytes(&value, &bounds).unwrap();
    bounds.document_bytes = bytes;
    assert!(owner_project::fits(&value, &bounds));
    bounds.document_bytes -= 1;
    assert!(!owner_project::fits(&value, &bounds));
    assert_eq!(value, whole);
    bounds.document_bytes = u64::MAX;
    value.policy.permissions = Some(List::with_capacity(3));
    assert!(!owner_project::fits(&value, &bounds));
    value = whole;
    value.configured_pools = List::with_capacity(3);
    assert!(!owner_project::fits(&value, &bounds));
}

// Independently sum the inline carrier and every empty ordinary reservation.
fn reserved_project_bytes(offer_slots: u32) -> u64 {
    u64::try_from(core::mem::size_of::<OwnerProject>()).unwrap()
        + List::<Offered>::worst_case(offer_slots).unwrap()
        + List::<OwnerProjectResource>::worst_case(2).unwrap()
        + List::<OwnerGrant>::worst_case(2).unwrap()
        + List::<OwnerProjectPolicyRole>::worst_case(2).unwrap()
        + List::<PolicyRequirement>::worst_case(2).unwrap()
        + List::<OwnerProjectMember>::worst_case(2).unwrap()
        + List::<PoolLedger>::worst_case(2).unwrap() * 2
        + List::<OwnerProjectSpend>::worst_case(2).unwrap()
        + List::<Recurring>::worst_case(2).unwrap()
}
fn project_offers(populated: u32, capacity: u32) -> List<Offered> {
    let kinds = [ActKind::Reorder, ActKind::ChangePolicy, ActKind::SetRoles, ActKind::MakeService, ActKind::SetPool];
    let mut offers = List::with_capacity(capacity);
    for index in 0..populated {
        offers
            .push(Offered {
                effect: None,
                act: kinds[usize::try_from(index % 5).unwrap()],
                admission: Admission::Allowed,
            })
            .unwrap();
    }
    offers
}
#[test]
fn owner_project_closed_offer_reservations_price_empty_partial_and_twenty_five_at_exact_whole_read_bytes() {
    for populated in [0, 1, MAX_OFFERED_ACTS] {
        let mut value = source();
        value.acts = project_offers(populated, MAX_OFFERED_ACTS);
        let original = value.clone();
        let bytes = reserved_project_bytes(MAX_OFFERED_ACTS);
        let mut bounds = Bounds { rows: 2, segments: 2, document_bytes: bytes, ..Bounds::default() };
        assert_eq!(owner_project::bytes(&value, &bounds), Some(bytes));
        assert!(owner_project::fits(&value, &bounds));
        bounds.document_bytes -= 1;
        assert!(!owner_project::fits(&value, &bounds));
        assert_eq!(value, original);
        let envelope = bytes + u64::try_from(core::mem::size_of::<ReadValue>()).unwrap();
        let mut setup = script();
        setup.bounds.rows = 2;
        setup.bounds.document_bytes = envelope;
        setup.truth.owner_projects.insert(1, value.clone());
        let mut engine = Engine::new(setup, 9130).unwrap();
        engine.read(Time::ZERO, 11, Token::new(3), Read::Project { project: 1 }).unwrap();
        let outputs = engine.poll(Time::ZERO);
        assert_eq!(
            outputs,
            vec![crate::Up::Read {
                token: Token::new(3),
                terminal: ReadResult::Value(ReadValue::OwnerProject(Box::new(value.clone())))
            }]
        );
        assert_eq!(engine.truth().owner_projects.get(&1), Some(&original));
        let mut short = script();
        short.bounds.rows = 2;
        short.bounds.document_bytes = envelope - 1;
        short.truth.owner_projects.insert(1, value.clone());
        let mut engine = Engine::new(short, 9131).unwrap();
        engine.read(Time::ZERO, 11, Token::new(4), Read::Project { project: 1 }).unwrap();
        assert_eq!(
            engine.poll(Time::ZERO),
            vec![crate::Up::Read { token: Token::new(4), terminal: ReadResult::None(ReadNone::Busy) }]
        );
        assert_eq!(engine.truth().owner_projects.get(&1), Some(&original));
        value.acts = project_offers(populated, MAX_OFFERED_ACTS + 1);
        let oversized = value.clone();
        bounds.document_bytes = u64::MAX;
        assert!(!owner_project::fits(&value, &bounds));
        assert_eq!(value, oversized);
        let mut setup = script();
        setup.bounds.rows = 2;
        setup.truth.owner_projects.insert(1, value);
        assert!(matches!(Engine::new(setup, 9132), Err(Invalid::Payload)));
    }
}
#[test]
fn owner_project_last_of_twenty_five_offers_keeps_recursive_rows_and_source_order_independent() {
    let mut value = source();
    value.acts = project_offers(MAX_OFFERED_ACTS, MAX_OFFERED_ACTS);
    let last = MAX_OFFERED_ACTS - 1;
    let mut charters = List::with_capacity(2);
    charters.push(u32::MAX).unwrap();
    charters.push(0).unwrap();
    let mut roles = List::with_capacity(2);
    roles.push(u32::MAX).unwrap();
    roles.push(u32::MAX).unwrap();
    value.acts.get_mut(last).unwrap().act = ActKind::Reorder;
    value.acts.get_mut(last).unwrap().admission = Admission::Bounded {
        within: jig_client_domain::contract::Bounds::Create(jig_client_domain::contract::CreateBounds {
            authority: Box::new(crate::tests::owner_authority()),
            pool_left: u64::MAX,
            charters,
            procedures: List::with_capacity(2),
            latest: Some(skein_lib::Wall::EPOCH),
            budget: 0,
            maximum_budget: u64::MAX,
            maximum_deadline: None,
        }),
        beyond: Beyond::Proposed { to: Recipient::Roles { project: u32::MAX, roles } },
    };
    let original = value.clone();
    let bytes = reserved_project_bytes(MAX_OFFERED_ACTS)
        + List::<u32>::worst_case(2).unwrap() * 2
        + List::<CreateProcedure>::worst_case(2).unwrap()
        + u64::try_from(core::mem::size_of::<OwnerAuthorityDocument>()).unwrap();
    let mut bounds = Bounds { rows: 2, segments: 2, document_bytes: bytes, ..Bounds::default() };
    assert_eq!(owner_project::bytes(&value, &bounds), Some(bytes));
    assert!(owner_project::fits(&value, &bounds));
    bounds.document_bytes -= 1;
    assert!(!owner_project::fits(&value, &bounds));
    bounds.document_bytes = u64::MAX;
    let Admission::Bounded { beyond: Beyond::Proposed { to: Recipient::Roles { roles, .. } }, .. } =
        &mut value.acts.get_mut(last).unwrap().admission
    else {
        panic!("whole last admission")
    };
    *roles = List::with_capacity(3);
    let malformed = value.clone();
    assert!(!owner_project::fits(&value, &bounds));
    assert_eq!(value, malformed);
    assert_eq!(original.acts.capacity(), 25);
    assert_eq!(original.acts.len(), 25);
    assert_eq!(original.acts.get(last).unwrap().act, ActKind::Reorder);
    value = original.clone();
    value.members = List::with_capacity(3);
    assert!(!owner_project::fits(&value, &bounds));
}

const UTF8_TEXTS: [&[u8]; 1] = [b"~\0!\0"];
