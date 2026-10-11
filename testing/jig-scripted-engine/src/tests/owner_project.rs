use super::{frame, script, text};
use crate::{Bounds, Engine, Invalid, Script, owner_project};
use jig_client::contract::*;
use skein_lib::{List, Time, Token};
fn source() -> OwnerProject {
    OwnerProject {
        number: 1,
        acts: List::with_capacity(2),
        resources: List::with_capacity(2),
        policy: OwnerProjectPolicy {
            ceiling: OwnerAuthorityDocument::complete(OwnerAuthority {
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
    let mut bounds = Bounds { rows: 2, document_bytes: bytes, ..Bounds::default() };
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
            login: DocumentText { bytes: text(&[255, 0, 254, 0]), cut: true },
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
    let mut bounds = Bounds { rows: 2, text: 4, document_bytes: u64::MAX, ..Bounds::default() };
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
    assert_eq!(original.members.get(0).unwrap().login.bytes(), &[255, 0, 254, 0]);
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
