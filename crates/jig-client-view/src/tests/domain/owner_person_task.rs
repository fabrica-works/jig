#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary public-boundary proofs inspect emitted asks and retained documents"
)]
use super::{
    objects::{chip, live, person, text},
    spine::drive,
};
use crate::tests::fixtures::refusal_answer;
use crate::tests::fixtures::{AnsweredByEngine, Fixture, ReadPage};
use crate::{
    Event, Request,
    action::Act,
    confirm::Missing,
    fields::Purpose,
    objects::{Body, Card, Left, Object, Source},
};
use jig_client::contract::*;
use skein_lib::{Duration, Id, List, Time, Wall};
fn values<T>(items: impl IntoIterator<Item = T>) -> List<T> {
    let items: Vec<_> = items.into_iter().collect();
    let mut rows = List::with_capacity(u32::try_from(items.len()).unwrap());
    for item in items {
        rows.push(item).ok().unwrap();
    }
    rows
}
fn value(contract: TaskContractDocument) -> OwnerPersonTask {
    let mut task = chip(8);
    task.details = Box::new(ChipPhase::Owner {
        phase: OwnerChipPhase::Person { address: PersonAddress::Role(u32::MAX), taken_by: None, since: None },
        details: Box::new(super::chip_facts::details()),
    });
    task.phase = Phase::Active;
    OwnerPersonTask {
        task,
        revision: Revision { value: 91 },
        requester: OwnerRequester::Task(u64::MAX),
        spec: person().spec,
        address: PersonAddress::Role(u32::MAX),
        taken_by: None,
        since: None,
        stall: None,
        deadline: None,
        contract,
        results: List::with_capacity(3),
        cut: false,
        acts: values([
            Offered { effect: None, act: ActKind::Answer, admission: Admission::Allowed },
            Offered { effect: None, act: ActKind::Take, admission: Admission::Allowed },
            Offered { effect: None, act: ActKind::HandBack, admission: Admission::Allowed },
        ]),
    }
}
fn installed(doc: OwnerPersonTask) -> (Fixture, Id<Object>) {
    let (mut d, _) = live();
    let key = ObjectKey::PersonTask(doc.task.task);
    let id = d.objects.name(key, Source::Window { number: 4 }).unwrap();
    assert!(d.objects.put(key, Body::OwnerPersonTask(Box::new(doc))));
    (d, id)
}
fn act(d: &mut Fixture, a: Act) -> Box<[Request]> {
    drive(d, Time::ZERO, Event::Act { action: a })
}
fn open(d: &mut Fixture, id: Id<Object>, kind: ActKind) {
    assert!(act(d, Act::Intend { object: id.token(), act: kind }).is_empty());
}
fn edit(d: &mut Fixture, p: Purpose, b: &[u8]) {
    assert!(act(d, Act::Edit { field: p, text: text(b) }).is_empty());
}
fn one(rows: &[Request]) -> &Ask {
    assert_eq!(rows.len(), 1);
    let Request::Client { asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. } } = &rows[0] else {
        panic!("one send expected")
    };
    ask
}
fn reread(d: &mut Fixture, doc: OwnerPersonTask) {
    let read = d.request_read(Source::Window { number: 4 }, Read::PersonTask { task: 8 }).unwrap();
    drive(d, Time::ZERO, ReadPage { read, result: ReadResult::Value(ReadValue::OwnerPersonTask(Box::new(doc))) });
}
#[test]
fn person_owner_carrier_keeps_full_width_addresses_requesters_wait_options_and_independent_loss() {
    for address in [PersonAddress::Party(u64::MAX), PersonAddress::Role(u32::MAX)] {
        for requester in [
            OwnerRequester::Task(u64::MAX),
            OwnerRequester::Person(u64::MAX),
            OwnerRequester::Deployment { project: u32::MAX },
        ] {
            for mask in 0_u8..16 {
                let mut doc = value(TaskContractDocument::Verdict { choices: List::with_capacity(2), cut: true });
                doc.task.task = u64::MAX;
                doc.task.title.cut = true;
                doc.revision = Revision { value: u64::MAX };
                doc.spec.cut = true;
                doc.spec.specification.words.cut = true;
                doc.address = address;
                doc.requester = requester;
                doc.taken_by = if mask & 1 != 0 { Some(u64::MAX) } else { None };
                doc.since = if mask & 2 != 0 { Some(Wall::EPOCH) } else { None };
                doc.stall = if mask & 4 != 0 { Some(Duration::from_nanos(u64::MAX)) } else { None };
                doc.deadline = if mask & 8 != 0 { Some(Wall::from_nanos(u64::MAX)) } else { None };
                doc.cut = true;
                let expected = doc.clone();
                let (d, id) = installed(doc);
                let projected = d.objects().get(id).unwrap();
                let body = projected.body.as_ref().unwrap();
                assert_eq!(body.key(), ObjectKey::PersonTask(u64::MAX));
                assert_eq!(body.ending(), None);
                assert_eq!(body.terminal(), None);
                assert_eq!(crate::objects::person_task::owner_facts(&d.objects().get(id).unwrap()), Some(&expected));
                assert_eq!(d.objects().get(id).unwrap().project, Some(u32::MAX));
                assert_eq!(expected.results.capacity(), 3);
            }
        }
    }
}
#[test]
fn person_owner_approval_copies_opaque_state_and_distinguishes_optional_empty_words() {
    for state in [b"".as_slice(), b"a/b", &[255, 0, 128]] {
        for words in [None, Some(b"".as_slice()), Some(b"why".as_slice())] {
            let doc = value(TaskContractDocument::Approval {
                state: state.into(),
                part: OwnerPart {
                    connector: u16::MAX,
                    kind: OwnerPartKind::Outbox,
                    key: OwnerPartKey::Number(u64::MAX),
                },
            });
            let (mut d, id) = installed(doc);
            open(&mut d, id, ActKind::Answer);
            if let Some(words) = words {
                edit(&mut d, Purpose::Reason, words);
            }
            assert_eq!(
                one(&act(&mut d, Act::Confirm)),
                &Ask::Answer {
                    project: u32::MAX,
                    on: AnswerOn::Person {
                        task: 8,
                        revision: Revision { value: 91 },
                        answer: PersonAnswer::Approved { state: state.into(), words: words.map(text) }
                    }
                }
            );
        }
    }
}
#[test]
fn person_owner_numeric_verdict_checks_word_bounds_fields_and_cut_without_inventing_choices() {
    let choices = values([
        VerdictChoice { code: u32::MAX, words: 3, followups: u32::MAX, fields: List::with_capacity(2) },
        VerdictChoice {
            code: 2,
            words: 3,
            followups: 7,
            fields: values([FieldRule { name: Box::from(b"field".as_slice()), max: u32::MAX }]),
        },
    ]);
    let doc = value(TaskContractDocument::Verdict { choices, cut: true });
    for (code, reason, allowed) in [
        (u32::MAX, None, true),
        (u32::MAX, Some(b"".as_slice()), true),
        (u32::MAX, Some(b"abc".as_slice()), true),
        (u32::MAX, Some(b"abcd".as_slice()), false),
        (2, None, false),
        (3, None, false),
    ] {
        let (mut d, id) = installed(doc.clone());
        open(&mut d, id, ActKind::Answer);
        act(&mut d, Act::Choose { option: code });
        if let Some(reason) = reason {
            edit(&mut d, Purpose::Reason, reason);
        }
        let rows = act(&mut d, Act::Confirm);
        if allowed {
            assert_eq!(
                one(&rows),
                &Ask::Answer {
                    project: u32::MAX,
                    on: AnswerOn::Person {
                        task: 8,
                        revision: Revision { value: 91 },
                        answer: PersonAnswer::Choice { option: u32::MAX, reason: text(reason.unwrap_or(b"")) }
                    }
                }
            );
        } else {
            assert!(rows.is_empty());
            assert!(d.confirmation().unwrap().missing.is_some());
        }
    }
}
#[test]
fn person_owner_report_accepts_supplied_empty_and_exact_bound_while_change_retains_cannot() {
    for (bytes, allowed) in [(b"".as_slice(), true), (b"abc".as_slice(), true), (b"abcd".as_slice(), false)] {
        let (mut d, id) = installed(value(TaskContractDocument::Report { words: 3 }));
        open(&mut d, id, ActKind::Answer);
        edit(&mut d, Purpose::Answer, bytes);
        let rows = act(&mut d, Act::Confirm);
        if allowed {
            assert_eq!(
                one(&rows),
                &Ask::Answer {
                    project: u32::MAX,
                    on: AnswerOn::Person {
                        task: 8,
                        revision: Revision { value: 91 },
                        answer: PersonAnswer::Words(text(bytes))
                    }
                }
            );
        } else {
            assert!(rows.is_empty());
            assert_eq!(d.fields().get(Purpose::Answer).unwrap().text.bytes(), bytes);
        }
    }
    let (mut d, id) = installed(value(TaskContractDocument::Change { connector: u16::MAX, kind: u16::MAX, words: 3 }));
    open(&mut d, id, ActKind::Answer);
    edit(&mut d, Purpose::Answer, b"abc");
    assert!(act(&mut d, Act::Confirm).is_empty());
    act(&mut d, Act::CannotDo);
    edit(&mut d, Purpose::Reason, b"cannot");
    assert_eq!(
        one(&act(&mut d, Act::Confirm)),
        &Ask::Answer {
            project: u32::MAX,
            on: AnswerOn::Person {
                task: 8,
                revision: Revision { value: 91 },
                answer: PersonAnswer::Cannot { reason: text(b"cannot") }
            }
        }
    );
}
#[test]
fn person_owner_role_actions_follow_offers_and_missing_metadata_never_uses_page_context() {
    for kind in [ActKind::Take, ActKind::HandBack] {
        let doc = value(TaskContractDocument::Report { words: 3 });
        let (mut d, id) = installed(doc.clone());
        open(&mut d, id, kind);
        let expected = if kind == ActKind::Take {
            Ask::Take { project: u32::MAX, task: 8 }
        } else {
            Ask::HandBack { project: u32::MAX, task: 8 }
        };
        assert_eq!(one(&act(&mut d, Act::Confirm)), &expected);
        let (mut d, id) = installed(doc.clone());
        open(&mut d, id, kind);
        let mut current = doc.clone();
        current.acts = List::with_capacity(3);
        reread(&mut d, current);
        assert!(act(&mut d, Act::Confirm).is_empty());
        assert_eq!(d.confirmation().unwrap().missing, Some(Missing::Refused { why: Refusal::NotOffered }));
        let mut partial = doc.clone();
        partial.task.details = Box::new(ChipPhase::Starting);
        let (mut d, id) = installed(partial);
        d.objects.get_mut(id).unwrap().project = Some(7);
        open(&mut d, id, kind);
        assert!(act(&mut d, Act::Confirm).is_empty());
        assert_eq!(d.confirmation().unwrap().missing, Some(Missing::Context));
    }
}
#[test]
fn person_owner_changed_revision_fences_old_selection_and_moved_refusal_preserves_new_state() {
    let doc = value(TaskContractDocument::Verdict {
        choices: values([VerdictChoice { code: 7, words: 3, followups: 0, fields: List::with_capacity(0) }]),
        cut: false,
    });
    let (mut d, id) = installed(doc.clone());
    open(&mut d, id, ActKind::Answer);
    act(&mut d, Act::Choose { option: 7 });
    let mut newer = doc.clone();
    newer.revision = Revision { value: 92 };
    reread(&mut d, newer.clone());
    assert!(act(&mut d, Act::Confirm).is_empty());
    assert!(d.confirmation().unwrap().changed);
    act(&mut d, Act::Dismiss);
    open(&mut d, id, ActKind::Answer);
    assert_eq!(d.confirmation().unwrap().choices.option, None);
    act(&mut d, Act::Choose { option: 7 });
    let rows = act(&mut d, Act::Confirm);
    assert_eq!(
        one(&rows),
        &Ask::Answer {
            project: u32::MAX,
            on: AnswerOn::Person {
                task: 8,
                revision: Revision { value: 92 },
                answer: PersonAnswer::Choice { option: 7, reason: text(b"") }
            }
        }
    );
    let Request::Client { asked: jig_client::Event::Send { key: request, .. } } = rows[0] else {
        panic!("one request")
    };
    let why = Refusal::Moved { now: Revision { value: 93 }, what: Moved::PersonTask };
    drive(&mut d, Time::ZERO, AnsweredByEngine { request, answer: refusal_answer(why.clone()) });
    assert_eq!(d.objects().get(id).unwrap().card, Card::Refused { refusal: why.clone() });
    assert_eq!(d.objects().get(id).unwrap().answer, Some(refusal_answer(why)));
    assert_eq!(crate::objects::person_task::owner_facts(&d.objects().get(id).unwrap()), Some(&newer));
}
#[test]
fn person_owner_original_read_identity_and_late_answer_remain_separate_from_current_body() {
    let doc = value(TaskContractDocument::Report { words: 3 });
    let (mut d, id) = installed(doc.clone());
    let mut wrong = doc.clone();
    wrong.task.task = 9;
    reread(&mut d, wrong);
    assert_eq!(crate::objects::person_task::owner_facts(&d.objects().get(id).unwrap()), Some(&doc));
    open(&mut d, id, ActKind::Answer);
    edit(&mut d, Purpose::Answer, b"abc");
    let rows = act(&mut d, Act::Confirm);
    let Request::Client { asked: jig_client::Event::Send { key: request, .. } } = rows[0] else {
        panic!("one request")
    };
    let mut newer = doc.clone();
    newer.revision = Revision { value: 100 };
    assert!(d.objects.put(ObjectKey::PersonTask(8), Body::OwnerPersonTask(Box::new(newer.clone()))));
    let outcome = Outcome::Changed;
    drive(&mut d, Time::ZERO, AnsweredByEngine { request, answer: Answer::Done(outcome.clone()) });
    assert_eq!(crate::objects::person_task::owner_facts(&d.objects().get(id).unwrap()), Some(&newer));
    assert_eq!(d.objects().get(id).unwrap().answer, Some(Answer::Done(outcome.clone())));
    assert_eq!(
        d.objects().get(id).unwrap().card,
        Card::Leaving { how: Left::Answered { outcome }, until: Time::from_nanos(5_000_000_000) }
    );
    assert_eq!(d.objects().get(id).unwrap().body.as_ref().unwrap().ending(), None);
    assert_eq!(crate::objects::person_task::revision(&d.objects().get(id).unwrap()), Some(Revision { value: 100 }));
    assert!(crate::objects::person_task::facts(&d.objects().get(id).unwrap()).is_none());
}

use jig_client::contract::AnswerOn;
