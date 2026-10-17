//! A supplied amendment reason is independent of the six selected changes.
use super::*;

fn opened(act: ActKind) -> (Domain, Id<Object>) {
    let (mut domain, _) = live();
    let mut row = task();
    row.acts = offered(act, Admission::Allowed);
    let id = install(&mut domain, Body::Task(Box::new(row)));
    intend(&mut domain, id, act);
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Change { change: Box::new(change()) } });
    (domain, id)
}

#[test]
fn amendment_requires_person_reason_presence_and_accepts_explicit_empty() {
    let (mut full, _) = opened(ActKind::Amend);
    for field in [Purpose::NewChat, Purpose::Answer, Purpose::Instructions, Purpose::NewGoal] {
        edit(&mut full, field, b"");
    }
    let refused =
        drive(&mut full, Time::ZERO, Event::Act { act: Act::Edit { field: Purpose::Reason, text: text(b"") } });
    assert!(refused.as_ref().contains(&Request::Fact { fact: crate::facts::Fact::Refused { which: Limit::Fields } }));
    assert_eq!(full.confirmation().unwrap().reason_input, crate::confirm::ReasonInput::Missing);
    assert!(full.fields().get(Purpose::Reason).is_none());
    let (mut domain, id) = opened(ActKind::Amend);
    assert_eq!(domain.confirmation().unwrap().missing, Some(Missing::Words { field: Purpose::Reason }));
    assert!(acts::ask(domain.objects().get(id).unwrap(), domain.confirmation().unwrap(), domain.fields()).is_none());
    assert!(drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm }).is_empty());
    edit(&mut domain, Purpose::Instructions, b"unrelated");
    let too_long = Text::new(Box::from(UTF8_TEXTS[0]), 33).unwrap();
    let refused =
        drive(&mut domain, Time::ZERO, Event::Act { act: Act::Edit { field: Purpose::Reason, text: too_long } });
    assert!(
        refused
            .as_ref()
            .contains(&Request::Fact { fact: crate::facts::Fact::Refused { which: Limit::Words { limit: 32 } } })
    );
    assert_eq!(domain.confirmation().unwrap().reason_input, crate::confirm::ReasonInput::Missing);
    assert!(domain.fields().get(Purpose::Reason).is_none());
    edit(&mut domain, Purpose::Reason, b"");
    assert_eq!(domain.confirmation().unwrap().reason_input, crate::confirm::ReasonInput::Supplied);
    assert!(domain.confirmation().unwrap().missing.is_none());
    let out = drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm });
    assert_eq!(
        sent(&out),
        &Ask::Amend {
            project: 1,
            task: 8,
            revision: Revision { value: 9 },
            change: change(),
            release: None,
            reason: text(b"")
        }
    );
    assert!(drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm }).is_empty());
}

#[test]
fn amendment_reason_receipt_resets_without_losing_domain_clear_write_counts() {
    let (mut domain, id) = opened(ActKind::Amend);
    edit(&mut domain, Purpose::Reason, b"previous reason");
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Dismiss });
    assert_eq!(domain.fields().get(Purpose::Reason).unwrap().text.bytes(), b"");
    assert_eq!(domain.fields().get(Purpose::Reason).unwrap().writes, 1);
    intend(&mut domain, id, ActKind::Amend);
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Change { change: Box::new(change()) } });
    assert_eq!(domain.confirmation().unwrap().reason_input, crate::confirm::ReasonInput::Missing);
    assert_eq!(domain.confirmation().unwrap().missing, Some(Missing::Words { field: Purpose::Reason }));
    assert!(drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm }).is_empty());
    edit(&mut domain, Purpose::Reason, b"");
    assert!(domain.confirmation().unwrap().missing.is_none());
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Dismiss });
    assert_eq!(domain.fields().get(Purpose::Reason).unwrap().writes, 2);
    intend(&mut domain, id, ActKind::Amend);
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Change { change: Box::new(change()) } });
    assert_eq!(domain.confirmation().unwrap().reason_input, crate::confirm::ReasonInput::Missing);
}

#[test]
fn amendment_release_keeps_opaque_reason_delta_and_independent_revisions_in_one_send() {
    let (mut domain, _) = opened(ActKind::AmendRelease);
    let bytes = UTF8_TEXTS[1];
    edit(&mut domain, Purpose::Reason, bytes);
    let selected = domain.confirmation().unwrap().choices.change.clone().unwrap();
    assert_eq!(selected, change());
    let out = drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm });
    assert_eq!(
        sent(&out),
        &Ask::Amend {
            project: 1,
            task: 8,
            revision: Revision { value: 9 },
            change: selected,
            release: Some(Revision { value: 3 }),
            reason: text(bytes)
        }
    );
    assert!(domain.fields().get(Purpose::Reason).unwrap().text.bytes().is_empty());
    assert_eq!(domain.fields().get(Purpose::Reason).unwrap().writes, 1);
}

#[test]
fn a_late_amendment_answer_preserves_another_forms_supplied_reason_until_page_switch() {
    let (mut domain, _) = opened(ActKind::Amend);
    edit(&mut domain, Purpose::Reason, b"old reason");
    let out = drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm });
    let Request::Send { request, .. } = out[0] else { panic!("one amendment send") };
    let mut other = task();
    other.chip.task = 9;
    other.acts = offered(ActKind::Amend, Admission::Allowed);
    let id = install(&mut domain, Body::Task(Box::new(other)));
    intend(&mut domain, id, ActKind::Amend);
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Change { change: Box::new(change()) } });
    edit(&mut domain, Purpose::Reason, b"new reason");
    let writes = domain.fields().get(Purpose::Reason).unwrap().writes;
    drive(&mut domain, Time::ZERO, Event::Answered { request, answer: Answer::Refused(Refusal::NotOffered) });
    assert_eq!(domain.confirmation().unwrap().object, id);
    assert_eq!(domain.confirmation().unwrap().reason_input, crate::confirm::ReasonInput::Supplied);
    assert_eq!(domain.fields().get(Purpose::Reason).unwrap().text.bytes(), b"new reason");
    assert_eq!(domain.fields().get(Purpose::Reason).unwrap().writes, writes);
    drive(
        &mut domain,
        Time::ZERO,
        Event::Act { act: Act::Go { address: crate::address::Address::Inbox { project: None } } },
    );
    assert!(domain.confirmation().is_none());
    assert!(domain.fields().get(Purpose::Reason).unwrap().text.bytes().is_empty());
    assert_eq!(domain.fields().get(Purpose::Reason).unwrap().writes, writes.checked_add(1).unwrap());
}

const UTF8_TEXTS: [&[u8]; 2] = [&[b'x'; 33], &[b'x'; 32]];
