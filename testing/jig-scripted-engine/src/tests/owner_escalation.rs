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
    task.acts = list(vec![Offered { act: ActKind::Stop, admission: Admission::Allowed }]);
    let value = ReadValue::OwnerEscalation(Box::new(OwnerEscalation {
        task: Box::new(task),
        acts: list(vec![Offered { act: ActKind::Release, admission: Admission::Allowed }]),
    }));
    setup.truth.objects.insert(ObjectKey::Escalation(8), value.clone());
    setup.truth.object_projects.insert(ObjectKey::Escalation(8), 1);
    let engine = Engine::new(setup.clone(), 1).unwrap();
    assert_eq!(engine.truth.objects[&ObjectKey::Escalation(8)], value);
    setup.truth.object_projects.insert(ObjectKey::Escalation(8), 2);
    assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Identity));
}
