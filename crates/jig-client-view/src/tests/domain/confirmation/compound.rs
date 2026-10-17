//! A composed confirmation retains both independently offered admissions.
use super::*;
#[test]
fn a_composed_amendment_rechecks_the_current_release_offer() {
    let (mut domain, _) = live();
    let mut row = task();
    row.acts = offered(ActKind::Amend, Admission::Allowed);
    let id = install(&mut domain, Body::Task(Box::new(row.clone())));
    intend_amend_release(&mut domain, id);
    edit(&mut domain, Purpose::Reason, b"");
    drive(&mut domain, Time::ZERO, Event::Act { action: Act::Change { change: Box::new(change()) } });
    assert_eq!(domain.confirmation().unwrap().missing, None);
    row.acts.get_mut(1).unwrap().admission = Admission::Refused(Refusal::NotWaiting);
    reread(&mut domain, Body::Task(Box::new(row)), Read::Task { task: 8 });
    assert_eq!(domain.confirmation().unwrap().missing, Some(Missing::Refused { why: Refusal::NotWaiting }));
    assert!(drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm }).is_empty());
}
