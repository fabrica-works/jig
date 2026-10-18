use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

#[test]
fn owner_escalation_prices_whole_boxed_task_and_separate_reserved_offers_and_refuses_without_clipping() {
    let bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    let mut task = task();
    task.context = None;
    task.resources = List::with_capacity(2);
    let mut path = List::with_capacity(2);
    path.push(Box::<[u8]>::from([255_u8; 32])).unwrap();
    task.resources.push(TaskResource::Part(Part::Resource { connector: u16::MAX, path })).unwrap();
    let row = OwnerEscalation { task: Box::new(task.clone()), acts: List::with_capacity(2) };
    let independent = u64::try_from(size_of::<Task>()).unwrap()
        + task.heap(&bounds).unwrap()
        + List::<Offered>::worst_case(2).unwrap();
    assert_eq!(row.heap(&bounds), Some(independent));
    let value = ReadValue::OwnerEscalation(Box::new(row.clone()));
    let exact = u64::try_from(size_of::<ReadValue>()).unwrap()
        + u64::try_from(size_of::<OwnerEscalation>()).unwrap()
        + independent;
    assert!(payload::fits(&value, &Bounds { document_bytes: exact, ..bounds }));
    assert!(!payload::fits(&value, &Bounds { document_bytes: exact - 1, ..bounds }));
    assert!(!payload::fits(&value, &Bounds { rows: 1, ..bounds }));
    assert!(!payload::fits(&value, &Bounds { text: 31, ..bounds }));
    assert_eq!(value, ReadValue::OwnerEscalation(Box::new(row)));
    assert_eq!(task.context, None);
}

#[test]
fn owner_escalation_script_preserves_supplied_identity_scope_and_outer_offer_without_synthetic_terminal() {
    let mut setup = script();
    let mut task = task();
    task.context = None;
    task.acts = list(vec![Offered { effect: None, act: ActKind::Stop, admission: Admission::Allowed }]);
    let value = ReadValue::OwnerEscalation(Box::new(OwnerEscalation {
        task: Box::new(task),
        acts: list(vec![Offered { effect: None, act: ActKind::Release, admission: Admission::Allowed }]),
    }));
    setup.truth.objects.insert(ObjectKey::Escalation(8), value.clone());
    setup.truth.object_projects.insert(ObjectKey::Escalation(8), 1);
    let engine = Engine::new(setup.clone(), 1).unwrap();
    assert_eq!(engine.truth.objects[&ObjectKey::Escalation(8)], value);
    // The embedded project is authoritative even if an obsolete fallback differs.
    setup.truth.object_projects.insert(ObjectKey::Escalation(8), 2);
    assert!(Engine::new(setup.clone(), 1).is_ok());
    assert_eq!(
        crate::Change::Object { project: 2, key: ObjectKey::Escalation(8), value: value.clone() }
            .validate(&Bounds::default()),
        Err(Invalid::Identity)
    );
    setup.truth.objects.remove(&ObjectKey::Escalation(8));
    setup.truth.objects.insert(ObjectKey::Escalation(99), value);
    assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Identity));
}

fn reserved_offers(kinds: &[ActKind], capacity: u32) -> List<Offered> {
    let mut offers = List::with_capacity(capacity);
    for act in kinds {
        offers.push(Offered { effect: None, act: *act, admission: Admission::Allowed }).unwrap();
    }
    offers
}
fn whole_read_bytes(row: &OwnerEscalation, bounds: &Bounds) -> u64 {
    u64::try_from(size_of::<ReadValue>()).unwrap()
        + u64::try_from(size_of::<OwnerEscalation>()).unwrap()
        + u64::try_from(size_of::<Task>()).unwrap()
        + row.task.as_ref().heap(bounds).unwrap()
        + crate::offered::heap(&row.acts, bounds).unwrap()
}

#[test]
fn owner_escalation_reserved_closed_offers_are_independent_of_task_rows_at_whole_read_byte_boundaries() {
    let bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    for populated in [false, true] {
        let mut row = OwnerEscalation {
            task: Box::new(task()),
            acts: reserved_offers(if populated { &[ActKind::Release] } else { &[] }, MAX_OFFERED_ACTS),
        };
        row.task.context = None;
        row.task.acts = reserved_offers(&[ActKind::Stop], MAX_OFFERED_ACTS);
        let original = row.clone();
        let exact = whole_read_bytes(&row, &bounds);
        let value = ReadValue::OwnerEscalation(Box::new(row.clone()));
        assert_eq!(
            row.heap(&bounds).unwrap(),
            exact
                - u64::try_from(size_of::<ReadValue>()).unwrap()
                - u64::try_from(size_of::<OwnerEscalation>()).unwrap()
        );
        assert_eq!(crate::offered::heap(&row.acts, &bounds), List::<Offered>::worst_case(MAX_OFFERED_ACTS));
        assert!(payload::fits(&value, &Bounds { document_bytes: exact, ..bounds }));
        assert!(!payload::fits(&value, &Bounds { document_bytes: exact - 1, ..bounds }));
        let base = OwnerEscalation { acts: reserved_offers(&[], 2), ..row.clone() };
        assert_eq!(
            row.heap(&bounds).unwrap() - base.heap(&bounds).unwrap(),
            List::<Offered>::worst_case(MAX_OFFERED_ACTS).unwrap() - List::<Offered>::worst_case(2).unwrap()
        );
        for outer in [false, true] {
            let mut oversized = row.clone();
            if outer {
                oversized.acts = reserved_offers(&[], MAX_OFFERED_ACTS + 1);
            } else {
                oversized.task.acts = reserved_offers(&[], MAX_OFFERED_ACTS + 1);
            }
            let untouched = oversized.clone();
            assert!(!payload::fits(&ReadValue::OwnerEscalation(Box::new(oversized.clone())), &bounds));
            assert_eq!(oversized, untouched);
        }
        row.task.dependencies = List::with_capacity(3);
        assert!(!payload::fits(&ReadValue::OwnerEscalation(Box::new(row.clone())), &bounds));
        assert_eq!(original.acts.capacity(), MAX_OFFERED_ACTS);
        assert_eq!(original.task.acts.capacity(), MAX_OFFERED_ACTS);
        assert_eq!(original.task.context, None);
        assert_eq!(value, ReadValue::OwnerEscalation(Box::new(original)));
    }
}

#[test]
fn owner_escalation_normal_read_preserves_offer_order_recursive_admissions_and_both_reserved_allocations() {
    let kinds = [ActKind::Release, ActKind::LeaveHeld, ActKind::PassUp, ActKind::Amend, ActKind::Cancel];
    let mut row = OwnerEscalation { task: Box::new(task()), acts: reserved_offers(&kinds, MAX_OFFERED_ACTS) };
    row.task.context = None;
    row.task.acts = reserved_offers(&[ActKind::Stop], MAX_OFFERED_ACTS);
    row.acts.get_mut(1).unwrap().admission =
        Admission::Proposed { to: Recipient::Roles { project: u32::MAX, roles: list(vec![u32::MAX, 0]) } };
    row.acts.get_mut(3).unwrap().admission = Admission::Bounded {
        within: jig_client_domain::contract::Bounds::Amend {
            budget_left: u64::MAX,
            implications: List::with_capacity(2),
            grants: List::with_capacity(2),
            authority: Box::new(row.task.authority.clone()),
            ceiling: Box::new(row.task.authority.clone()),
        },
        beyond: Beyond::Refused(Refusal::NotOffered),
    };
    let mut setup = script();
    setup.bounds = Bounds { rows: 2, authority: 2, text: 32, document_bytes: u64::MAX, ..setup.bounds };
    let authority_heap = row.task.authority.heap(&setup.bounds).unwrap();
    let nested = List::<u32>::worst_case(2).unwrap()
        + List::<OwnerKindImplication>::worst_case(2).unwrap()
        + List::<OwnerGrant>::worst_case(2).unwrap()
        + 2 * (u64::try_from(size_of::<OwnerAuthorityDocument>()).unwrap() + authority_heap);
    assert_eq!(
        crate::offered::heap(&row.acts, &setup.bounds),
        Some(List::<Offered>::worst_case(MAX_OFFERED_ACTS).unwrap() + nested)
    );
    let exact = whole_read_bytes(&row, &setup.bounds);
    let original = row.clone();
    let value = ReadValue::OwnerEscalation(Box::new(row.clone()));
    assert!(payload::fits(&value, &Bounds { document_bytes: exact, ..setup.bounds }));
    assert!(!payload::fits(&value, &Bounds { document_bytes: exact - 1, ..setup.bounds }));
    setup.truth.objects.insert(ObjectKey::Escalation(8), value.clone());
    let mut engine = Engine::new(setup.clone(), 17).unwrap();
    engine.read(Time::ZERO, 11, Token::new(1), Read::Escalation { task: 8 }).unwrap();
    let outputs = engine.poll(Time::ZERO);
    let received = outputs
        .iter()
        .find_map(|up| match up {
            Up::Read { token, terminal: ReadResult::Value(ReadValue::OwnerEscalation(document)) }
                if *token == Token::new(1) =>
            {
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
    assert_eq!(received.as_ref(), &original);
    assert_eq!(received.acts.capacity(), MAX_OFFERED_ACTS);
    assert_eq!(received.task.acts.capacity(), MAX_OFFERED_ACTS);
    assert_eq!(received.acts.iter().map(|offer| offer.act).collect::<Vec<_>>(), kinds);
    let Admission::Bounded { within: jig_client_domain::contract::Bounds::Amend { implications, .. }, .. } =
        &mut row.acts.get_mut(3).unwrap().admission
    else {
        panic!("supplied amendment");
    };
    *implications = List::with_capacity(3);
    let oversized = row.clone();
    assert!(!payload::fits(&ReadValue::OwnerEscalation(Box::new(row.clone())), &setup.bounds));
    assert_eq!(row, oversized);
    assert_eq!(value, ReadValue::OwnerEscalation(Box::new(original)));
}
