use super::{
    confirmation::{change, reread, task},
    objects::{chip, live, text},
    pages::project,
    spine::{LIMITS, delivered, drain, drive, frame},
};
use crate::tests::fixtures::refusal_answer;
use crate::tests::fixtures::{AnsweredByEngine, Fixture, Reported, Served, Watched};
use crate::{
    Event, Limits, Request,
    action::Act,
    address::{Address, Place},
    confirm::Missing,
    fields::Purpose,
    intents::Progress,
    objects::{Body, Object, Source},
    store::{Record, StoreKey},
};
use jig_client::contract::*;
use skein_lib::{Env, Id, List, Queue, Time, Wall};
fn install(domain: &mut Fixture, document: Task) -> Id<Object> {
    let key = ObjectKey::Task(document.chip.task);
    let id = domain.objects.name(key, Source::Window { number: 4 }).unwrap();
    assert!(domain.objects.put(key, Body::Task(Box::new(document))));
    id
}
fn intend(domain: &mut Fixture, id: Id<Object>, act: ActKind) {
    assert!(drive(domain, Time::ZERO, Event::Act { action: Act::Intend { object: id.token(), act } }).is_empty());
}
fn edit(domain: &mut Fixture, task: u64, words: &[u8]) -> Box<[Request]> {
    drive(domain, Time::ZERO, Event::Act { action: Act::Edit { field: Purpose::Composer { task }, text: text(words) } })
}
fn only_send(rows: &[Request]) -> (jig_client::Key, &Ask) {
    let Request::Client { asked: jig_client::Event::Send { key: request, request: jig_client::Ask::Jig(ask), .. } } =
        &rows[0]
    else {
        panic!("one send")
    };
    assert_eq!(rows.len(), 1);
    (*request, ask)
}
fn steering_document(act: ActKind) -> Task {
    let mut document = task();
    document.acts = List::with_capacity(2);
    document.acts.push(Offered { effect: None, act, admission: Admission::Allowed }).unwrap();
    if act == ActKind::Amend {
        document.acts.push(Offered { effect: None, act: ActKind::Release, admission: Admission::Allowed }).unwrap();
    }
    document
}
#[test]
fn steering_sends_one_ask_with_each_choice_and_amend_release_is_atomic() {
    for act in [ActKind::Message, ActKind::Stop, ActKind::Cancel, ActKind::Move, ActKind::Amend, ActKind::Release] {
        let (mut domain, _) = live();
        let mut document = steering_document(act);
        if act == ActKind::Release {
            let TaskHold::Document(hold) = document.hold.as_mut().unwrap() else { panic!("supplied legacy hold") };
            hold.reason = HoldReason::EffectUncertain { part: Part::Kind { connector: 1, kind: 2 } };
        }
        let id = install(&mut domain, document);
        if act == ActKind::Message {
            assert!(edit(&mut domain, 8, b"wake up").is_empty());
            assert!(
                drive(
                    &mut domain,
                    Time::ZERO,
                    Event::Act { action: Act::Submit { field: Purpose::Composer { task: 8 } } }
                )
                .is_empty()
            );
        } else if act == ActKind::Amend {
            intend_amend_release(&mut domain, id);
        } else {
            intend(&mut domain, id, act);
        }
        let expected = prepare_steering(&mut domain, act);
        let rows = drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm });
        assert_eq!(only_send(&rows).1, &expected);
        assert!(drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm }).is_empty());
    }
}
fn prepare_steering(domain: &mut Fixture, act: ActKind) -> Ask {
    match act {
        ActKind::Message => {
            drive(domain, Time::ZERO, Event::Act { action: Act::WriteRelease { release: true } });
            crate::tests::fixtures::releasing(
                Ask::Message { project: 1, task: 8, words: text(b"wake up") },
                Some(Revision { value: 3 }),
            )
        }
        ActKind::Stop => Ask::Stop { project: 1, task: 8 },
        ActKind::Cancel | ActKind::Move => {
            drive(domain, Time::ZERO, Event::Act { action: Act::Edit { field: Purpose::Reason, text: text(b"why") } });
            if act == ActKind::Cancel {
                Ask::Cancel { project: 1, task: 8, reason: text(b"why"), grace: None }
            } else {
                Ask::Move { project: 1, task: 8, reason: text(b"why") }
            }
        }
        ActKind::Amend => {
            drive(domain, Time::ZERO, Event::Act { action: Act::Edit { field: Purpose::Reason, text: text(b"") } });
            drive(domain, Time::ZERO, Event::Act { action: Act::Change { change: Box::new(change()) } });
            crate::tests::fixtures::releasing(
                Ask::Amend {
                    reason: text(b""),
                    project: 1,
                    task: 8,
                    revision: Some(Revision { value: 9 }),
                    change: Box::new(change()),
                },
                Some(Revision { value: 3 }),
            )
        }
        ActKind::Release => {
            assert_eq!(domain.confirmation().unwrap().missing, Some(Missing::Again));
            drive(domain, Time::ZERO, Event::Act { action: Act::Again { again: false } });
            Ask::Release { project: 1, task: 8, revision: Revision { value: 3 }, again: Some(false) }
        }
        ActKind::Reorder
        | ActKind::Withdraw
        | ActKind::Answer
        | ActKind::Take
        | ActKind::HandBack
        | ActKind::Accept
        | ActKind::Reject
        | ActKind::PassUp
        | ActKind::LeaveHeld
        | ActKind::Seen
        | ActKind::EditNote
        | ActKind::DeleteNote
        | ActKind::Adopt
        | ActKind::Rename
        | ActKind::ChangePolicy
        | ActKind::SetRoles
        | ActKind::MakeService
        | ActKind::SetPool => unreachable!(),
    }
}
#[test]
fn closing_names_each_live_delegate_once_and_chat_about_sends_only_the_named_first_words() {
    let (mut domain, _) = live();
    let mut me = frame(7);
    me.projects = List::with_capacity(1);
    me.projects.push(project(1, true)).unwrap();
    let watch = domain.frame_watch.unwrap();
    delivered(&mut domain, Time::ZERO, watch, me);
    let mut document = steering_document(ActKind::Cancel);
    document.flags.conversation = true;
    document.delegates = List::with_capacity(2);
    document.delegates.push(LiveDelegate { chip: chip(10), takeover_budget: 6 }).unwrap();
    document.delegates.push(LiveDelegate { chip: chip(11), takeover_budget: 7 }).unwrap();
    let id = install(&mut domain, document);
    assert!(drive(&mut domain, Time::ZERO, Event::Act { action: Act::CloseChat { object: id.token() } }).is_empty());
    assert!(domain.confirmation().unwrap().dangerous, "closing includes cancellation");
    assert_eq!(domain.confirmation().unwrap().missing, Some(Missing::Delegates));
    for task in [10, 10, 99] {
        drive(&mut domain, Time::ZERO, Event::Act { action: Act::Dispose { task, take_over: false } });
    }
    assert!(drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm }).is_empty());
    drive(&mut domain, Time::ZERO, Event::Act { action: Act::Dispose { task: 99, take_over: true } });
    // Reopen to discard the invalid selection, then choose both live identities.
    drive(&mut domain, Time::ZERO, Event::Act { action: Act::Dismiss });
    assert!(drive(&mut domain, Time::ZERO, Event::Act { action: Act::CloseChat { object: id.token() } }).is_empty());
    drive(&mut domain, Time::ZERO, Event::Act { action: Act::Dispose { task: 10, take_over: false } });
    drive(&mut domain, Time::ZERO, Event::Act { action: Act::Dispose { task: 11, take_over: true } });
    let mut operations = List::with_capacity(3);
    operations.push(Operation::Cancel { project: 1, task: 10, reason: text(b""), grace: None }).unwrap();
    operations.push(Operation::Move { project: 1, task: 11, reason: text(b"") }).unwrap();
    operations.push(Operation::Cancel { project: 1, task: 8, reason: text(b""), grace: None }).unwrap();
    assert_eq!(
        only_send(&drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm })).1,
        &Ask::Compound { project: 1, operations }
    );
    let (mut domain, _) = live();
    let id = install(&mut domain, task());
    let mut current = frame(7);
    current.projects = List::with_capacity(1);
    current.projects.push(project(1, true)).unwrap();
    let watch = domain.frame_watch.unwrap();
    delivered(&mut domain, Time::ZERO, watch, current);
    let rows = drive(&mut domain, Time::ZERO, Event::Act { action: Act::ChatAbout { object: id.token() } });
    assert_eq!(only_send(&rows).1, &crate::tests::fixtures::chat_request(1, text(b"title")));
}
fn custom(
    domain: &mut Fixture,
    limits: Limits,
    now: Time,
    event: impl crate::tests::fixtures::Input,
) -> Box<[Request]> {
    domain.ui.limits = limits;
    domain.drive(now, event)
}
#[test]
fn drafts_survive_pages_and_restart_evict_oldest_untouched_and_coalesce_only_current_records() {
    check_drafts_survive_pages_and_restart_evict_oldest_untouched_and_coalesce_only_current_records();
}
fn check_drafts_survive_pages_and_restart_evict_oldest_untouched_and_coalesce_only_current_records() {
    let limits = Limits { drafts: 2, records: 3, ..LIMITS };
    let mut domain = Fixture::new(&limits);
    let first = custom(
        &mut domain,
        limits,
        Time::ZERO,
        Event::Start { seed: 8, place: Place::Jig(Address::Inbox { project: None }), records: List::with_capacity(0) },
    );
    let Request::Client { asked: jig_client::Event::Watch { handle: watch, .. } } = first[0] else { panic!("frame") };
    custom(&mut domain, limits, Time::ZERO, Served { served: true });
    custom(
        &mut domain,
        limits,
        Time::ZERO,
        Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(frame(7)))) },
    );
    edit(&mut domain, 8, b"first");
    edit(&mut domain, 9, b"older");
    edit(&mut domain, 8, b"newest");
    let erased = edit(&mut domain, 10, b"third");
    assert_eq!(
        erased.as_ref(),
        &[Request::Erase { records: crate::tests::fixtures::one(StoreKey::Draft { task: 9 }) }]
    );
    assert!(domain.drafts().get(9).is_none());
    assert!(domain.fields().get(Purpose::Composer { task: 9 }).is_none());
    assert_eq!(domain.drafts().get(8).unwrap().words, text(b"newest"));
    let mut saves = Queue::with_capacity(crate::max_out(&limits));
    for _ in 0_u32..3 {
        crate::fire(
            &mut domain.ui,
            &Env { now: Time::from_nanos(1_000_000_000), wall: Wall::EPOCH, limits },
            &mut saves,
            &domain.client,
        );
    }
    let saves = drain(saves);
    assert_eq!(saves.len(), 2);
    for save in &saves {
        let Request::Save { records } = save else { panic!("only coalesced drafts") };
        let [Record::Draft { task, words, .. }] = records.as_slice() else { panic!("one coalesced draft") };
        assert_ne!(*task, 9);
        if *task == 8 {
            assert_eq!(*words, text(b"newest"));
        }
    }
    custom(&mut domain, limits, Time::from_nanos(1_000_000_000), Event::Went { place: Place::Jig(Address::task(8)) });
    let writes = domain.fields().get(Purpose::Composer { task: 8 }).unwrap().writes;
    assert!(writes > 0);
    custom(&mut domain, limits, Time::from_nanos(1_000_000_000), Event::Went { place: Place::Elsewhere });
    assert_eq!(domain.drafts().get(8).unwrap().words, text(b"newest"));
    let kept = domain.records().clone();
    let mut restarted = Fixture::new(&limits);
    let rows = custom(
        &mut restarted,
        limits,
        Time::ZERO,
        Event::Start { seed: 8, place: Place::Jig(Address::task(8)), records: kept },
    );
    assert!(restarted.fields().get(Purpose::Composer { task: 8 }).is_none());
    let Request::Client { asked: jig_client::Event::Watch { handle: watch, .. } } = rows[0] else { panic!("frame") };
    custom(&mut restarted, limits, Time::ZERO, Served { served: true });
    custom(
        &mut restarted,
        limits,
        Time::ZERO,
        Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(frame(7)))) },
    );
    let field = restarted.fields().get(Purpose::Composer { task: 8 }).unwrap();
    assert_eq!(field.text, text(b"newest"));
    assert_eq!(field.writes, 1);
    let rows = edit(&mut restarted, 12, b"fourth");
    assert_eq!(rows.as_ref(), &[Request::Erase { records: crate::tests::fixtures::one(StoreKey::Draft { task: 8 }) }]);
    assert!(restarted.drafts().get(10).is_some());
}
#[test]
fn a_words_answer_clears_only_the_submitted_touch_and_refused_or_unknown_words_keep_the_draft() {
    for case in 0_u32..5 {
        let (mut domain, _) = live();
        install(&mut domain, steering_document(ActKind::Message));
        assert!(edit(&mut domain, 8, b"words").is_empty());
        drive(&mut domain, Time::ZERO, Event::Act { action: Act::Submit { field: Purpose::Composer { task: 8 } } });
        let rows = drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm });
        let request = only_send(&rows).0;
        if case == 1 {
            edit(&mut domain, 8, b"later");
        }
        if case == 2 {
            edit(&mut domain, 8, b"words");
        }
        let answer = Answer::Done(Outcome::Owner(OwnerOutcome::Said { task: 8, message: 3 }));
        let rows = if case == 4 {
            drive(&mut domain, Time::ZERO, Reported { request, progress: Progress::Unknown })
        } else if case == 3 {
            drive(
                &mut domain,
                Time::ZERO,
                AnsweredByEngine {
                    request,
                    answer: refusal_answer(Refusal::Limit { which: Limit::Inbox { task: 8, limit: 1 } }),
                },
            )
        } else {
            drive(&mut domain, Time::ZERO, AnsweredByEngine { request, answer: answer.clone() })
        };
        if case == 0 {
            assert_eq!(
                rows.as_ref(),
                &[Request::Erase { records: crate::tests::fixtures::one(StoreKey::Draft { task: 8 }) }]
            );
            assert!(domain.drafts().get(8).is_none());
            assert!(domain.fields().get(Purpose::Composer { task: 8 }).unwrap().text.bytes().is_empty());
            assert_eq!(domain.fields().get(Purpose::Composer { task: 8 }).unwrap().writes, 1);
        } else {
            assert!(rows.is_empty());
            assert!(domain.drafts().get(8).is_some());
            assert_eq!(domain.fields().get(Purpose::Composer { task: 8 }).unwrap().writes, 0);
        }
        assert!(drive(&mut domain, Time::ZERO, AnsweredByEngine { request, answer }).is_empty());
    }
}

#[test]
fn amend_release_recalls_the_hold_independently_and_requires_review_when_either_revision_moves() {
    for which in 0_u8..3 {
        let (mut domain, _) = live();
        let mut document = steering_document(ActKind::Amend);
        let id = install(&mut domain, document.clone());
        intend_amend_release(&mut domain, id);
        assert_eq!(domain.confirmation().unwrap().revision, Some(Revision { value: 9 }));
        assert_eq!(domain.confirmation().unwrap().release_revision, Some(Revision { value: 3 }));
        drive(&mut domain, Time::ZERO, Event::Act { action: Act::Edit { field: Purpose::Reason, text: text(b"") } });
        drive(&mut domain, Time::ZERO, Event::Act { action: Act::Change { change: Box::new(change()) } });
        match which {
            0 => {
                let TaskHold::Document(hold) = document.hold.as_mut().unwrap() else { panic!("supplied legacy hold") };
                hold.revision = Revision { value: 4 };
            }
            1 => document.spec_revision = Revision { value: 10 },
            2 => document.hold = None,
            _ => unreachable!(),
        }
        reread(&mut domain, Body::Task(Box::new(document.clone())), Read::Task { task: 8 });
        assert!(drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm }).is_empty());
        assert_eq!(
            domain.confirmation().unwrap().missing,
            match which {
                0 => Some(Missing::ChangedHold { now: Some(Revision { value: 4 }) }),
                1 => Some(Missing::Changed { now: Some(Revision { value: 10 }) }),
                2 => Some(Missing::ChangedHold { now: None }),
                _ => unreachable!(),
            }
        );
        // Further draft editing does not acknowledge a newly displayed hold.
        drive(&mut domain, Time::ZERO, Event::Act { action: Act::Edit { field: Purpose::Reason, text: text(b"") } });
        drive(&mut domain, Time::ZERO, Event::Act { action: Act::Change { change: Box::new(change()) } });
        assert!(drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm }).is_empty());
        if which < 2 {
            drive(&mut domain, Time::ZERO, Event::Act { action: Act::Dismiss });
            intend_amend_release(&mut domain, id);
            drive(
                &mut domain,
                Time::ZERO,
                Event::Act { action: Act::Edit { field: Purpose::Reason, text: text(b"") } },
            );
            drive(&mut domain, Time::ZERO, Event::Act { action: Act::Change { change: Box::new(change()) } });
            let rows = drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm });
            assert_eq!(
                only_send(&rows).1,
                &crate::tests::fixtures::releasing(
                    Ask::Amend {
                        reason: text(b""),
                        project: 1,
                        task: 8,
                        revision: Some(document.spec_revision),
                        change: Box::new(change()),
                    },
                    Some(document.hold.as_ref().unwrap().revision())
                )
            );
        }
    }
}

fn intend_amend_release(domain: &mut Fixture, id: Id<Object>) {
    assert!(drive(domain, Time::ZERO, Event::Act { action: Act::AmendRelease { object: id.token() } }).is_empty());
}
