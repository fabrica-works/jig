use super::*;
#[test]
fn cancellation_waits_for_its_reason_before_it_can_be_confirmed() {
    let (mut domain, _) = live();
    let mut document = task();
    document.acts = offered(ActKind::Cancel, Admission::Allowed);
    let object = install(&mut domain, Body::Task(Box::new(document)));
    intend(&mut domain, object, ActKind::Cancel);
    assert_eq!(domain.confirmation().unwrap().missing, Some(Missing::Words { field: Purpose::Reason }));
    assert!(drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm }).is_empty());
    edit(&mut domain, Purpose::Reason, b"Done chatting");
    assert!(domain.confirmation().unwrap().missing.is_none());
}
