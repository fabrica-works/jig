use super::{
    confirmation::task,
    objects::{live, person, proposal},
    spine::{LIMITS, drain, drive},
};
use crate::tests::fixtures::{AnsweredByEngine, Fixture, ReadPage, Reported};
use crate::{
    Event, Request,
    action::Act,
    intents::{Progress, send_card},
    notices::kept::Kind,
    objects::{Body, Card, Left, Object, Source, answer::recovery},
};
use jig_client::contract::*;
use skein_lib::{Id, Queue, Time, Token, Wall};
mod person_refresh;
fn install(domain: &mut Fixture, body: Body) -> Id<Object> {
    let key = body.key();
    let id = domain.objects.name(key, Source::Window { number: 4 }).unwrap();
    assert!(domain.objects.put(key, body));
    id
}
fn send(domain: &mut Fixture, key: ObjectKey) -> Token {
    let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
    let token = send_card(
        &mut domain.ui,
        key,
        ActKind::Stop,
        Box::new(Ask::Stop { project: 1, task: 8 }),
        &mut out,
        &domain.client,
    )
    .unwrap();
    assert_eq!(drain(out).len(), 1);
    token
}
fn answered(domain: &mut Fixture, request: Token, answer: Answer) -> Box<[Request]> {
    drive(domain, Time::ZERO, AnsweredByEngine { request, answer })
}
fn read_token(rows: &[Request]) -> jig_client::Handle {
    for row in rows {
        if let Request::Client { asked: jig_client::Event::Read { handle: read, .. } } = row {
            return *read;
        }
    }
    unreachable!("refresh read emitted")
}
#[test]
fn durable_outcomes_remain_on_open_cards_and_decided_first_leaves_with_all_facts() {
    for outcome in [
        Outcome::Made { what: Made::Words { task: 8, message: 3 } },
        Outcome::Proposed { proposal: proposal().key },
        Outcome::Changed,
    ] {
        let (mut domain, _) = live();
        let id = install(&mut domain, Body::Task(Box::new(task())));
        let request = send(&mut domain, ObjectKey::Task(8));
        let answer = Answer::Done(outcome);
        assert!(answered(&mut domain, request, answer.clone()).is_empty());
        assert_eq!(domain.objects().get(id).unwrap().card, Card::Open);
        assert_eq!(domain.objects().get(id).unwrap().answer, Some(answer.clone()));
        assert!(answered(&mut domain, request, Answer::Refused(Refusal::Unknown)).is_empty());
        assert_eq!(domain.objects().get(id).unwrap().answer, Some(answer));
    }
    let (mut domain, _) = live();
    let id = install(&mut domain, Body::PersonTask(Box::new(person())));
    let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
    let request = send_card(
        &mut domain.ui,
        ObjectKey::PersonTask(8),
        ActKind::Take,
        Box::new(Ask::Take { project: 1, task: 8 }),
        &mut out,
        &domain.client,
    )
    .unwrap();
    answered(&mut domain, request, Answer::Done(Outcome::Changed));
    assert_eq!(domain.objects().get(id).unwrap().card, Card::Open);
    let (mut domain, _) = live();
    let id = install(&mut domain, Body::Proposal(Box::new(proposal())));
    let request = send(&mut domain, ObjectKey::Proposal(proposal().key));
    let outcome = Outcome::DecidedFirst { by: super::spine::party(9), how: Decision::Accepted, at: Wall::EPOCH };
    answered(&mut domain, request, Answer::Done(outcome.clone()));
    assert_eq!(
        domain.objects().get(id).unwrap().card,
        Card::Leaving { how: Left::Answered { outcome }, until: Time::from_nanos(5_000_000_000) }
    );
}
#[test]
fn refusals_keep_every_engine_fact_and_moved_reads_the_current_object_without_losing_the_refusal() {
    let refusals = [
        Refusal::Role { held: Some(HeldRole::Policy { role: 2 }) },
        Refusal::Role { held: None },
        Refusal::Disposal { task: 81, issue: DisposalIssue::Repeated },
        Refusal::Taken { by: 71 },
        Refusal::NotWaiting,
        Refusal::NoFurther,
        Refusal::Authority { lacking: Lacking::Deadline { asked: None, latest: None }, proposable: None },
        Refusal::Limit { which: Limit::Admission { which: AdmissionLimit::InboxFull } },
        Refusal::Moved {
            now: Revision { value: 12 },
            what: Moved::Hold { reason: Some(CurrentHold::Reason(HoldCategory::Uncertain { entry: 123 })) },
        },
        Refusal::Ended { how: Ending::Decided { by: 71, party: None, how: None, at: Wall::EPOCH } },
        Refusal::Ended {
            how: Ending::Cancelled { by: Actor::ProjectDeployment { project: 1 }, reason: None, at: Some(Wall::EPOCH) },
        },
        Refusal::Ended { how: Ending::Deleted },
        Refusal::Authority { lacking: Lacking::Budget { needed: 40, left: 12 }, proposable: None },
        Refusal::Limit { which: Limit::LiveTasks { project: 1, limit: 2 } },
        Refusal::Moved { now: Revision { value: 12 }, what: Moved::Spec },
        Refusal::Ended { how: Ending::Done },
        Refusal::Unknown,
        Refusal::NotOffered,
        Refusal::KeyConflict,
    ];
    for refusal in refusals {
        let (mut domain, _) = live();
        let id = install(&mut domain, Body::Task(Box::new(task())));
        let request = send(&mut domain, ObjectKey::Task(8));
        let rows = answered(&mut domain, request, Answer::Refused(refusal.clone()));
        let object = domain.objects().get(id).unwrap();
        assert_eq!(object.card, Card::Refused { refusal: refusal.clone() });
        assert_eq!(object.answer, Some(Answer::Refused(refusal.clone())));
        match refusal {
            Refusal::Moved { .. } => {
                assert!(recovery(&object).look_again);
                assert!(
                    drive(&mut domain, Time::ZERO, Event::Act { action: Act::LookAgain { object: id.token() } })
                        .is_empty()
                );
                let mut current = task();
                current.spec_revision = Revision { value: 12 };
                drive(
                    &mut domain,
                    Time::ZERO,
                    ReadPage {
                        read: read_token(&rows),
                        result: ReadResult::Value(ReadValue::Task(Box::new(current.clone()))),
                    },
                );
                assert_eq!(domain.objects().get(id).unwrap().body, Some(Body::Task(Box::new(current))));
                assert_eq!(domain.objects().get(id).unwrap().card, Card::Refused { refusal });
            }
            Refusal::Authority { .. } | Refusal::Limit { .. } => {
                assert!(recovery(&object).change_ask);
                assert!(rows.is_empty());
            }
            Refusal::Disposal { .. }
            | Refusal::Taken { .. }
            | Refusal::NotWaiting
            | Refusal::NoFurther
            | Refusal::Role { .. }
            | Refusal::Ended { .. }
            | Refusal::Unknown
            | Refusal::NotOffered
            | Refusal::KeyConflict => {
                assert!(!recovery(&object).look_again && !recovery(&object).change_ask);
                assert!(rows.is_empty());
            }
        }
    }
}
#[test]
fn retention_expiry_retires_once_keeps_unknown_distinct_and_inspection_never_resends() {
    let (mut domain, _) = live();
    let id = install(&mut domain, Body::Task(Box::new(task())));
    let key = ObjectKey::Task(8);
    let request = send(&mut domain, key);
    assert!(drive(&mut domain, Time::ZERO, Reported { request, progress: Progress::Unknown }).is_empty());
    assert_eq!(
        domain.objects().get(id).unwrap().card,
        Card::Unknown { act: ActKind::Stop, look: Read::Task { task: 8 } }
    );
    assert!(domain.objects().get(id).unwrap().answer.is_none());
    assert!(answered(&mut domain, request, Answer::Done(Outcome::Changed)).is_empty());
    let rows = drive(&mut domain, Time::ZERO, Event::Act { action: Act::LookAgain { object: id.token() } });
    assert_eq!(rows.len(), 1);
    drive(
        &mut domain,
        Time::ZERO,
        ReadPage { read: read_token(&rows), result: ReadResult::None(ReadNone::Unreachable) },
    );
    assert!(recovery(&domain.objects().get(id).unwrap()).look_again);

    let rows = drive(&mut domain, Time::ZERO, Event::Act { action: Act::LookAgain { object: id.token() } });
    drive(
        &mut domain,
        Time::ZERO,
        ReadPage { read: read_token(&rows), result: ReadResult::Value(ReadValue::Task(Box::new(task()))) },
    );
    assert_eq!(domain.objects().get(id).unwrap().card, Card::Open);
    domain.intents.reclaim();
}
#[test]
fn full_requests_refuse_without_queueing_and_hidden_answers_and_unknowns_have_notices() {
    let (mut domain, _) = live();
    let id = install(&mut domain, Body::Task(Box::new(task())));
    for number in 1..=LIMITS.intents {
        let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
        domain
            .intents
            .ask(
                7,
                crate::intents::Origin::Card { object: ObjectKey::Task(u64::from(number)), act: ActKind::Stop },
                Box::new(Ask::Stop { project: 1, task: 8 }),
                &mut out,
            )
            .unwrap();
    }
    let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
    assert!(
        send_card(
            &mut domain.ui,
            ObjectKey::Task(8),
            ActKind::Stop,
            Box::new(Ask::Stop { project: 1, task: 8 }),
            &mut out,
            &domain.client
        )
        .is_none()
    );
    assert_eq!(domain.objects().get(id).unwrap().card, Card::NotSent { why: crate::intents::NotSent::Limit });
    assert_eq!(drain(out).as_ref(), &[]);
    for unknown in [false, true] {
        let (mut domain, _) = live();
        install(&mut domain, Body::Task(Box::new(task())));
        let request = send(&mut domain, ObjectKey::Task(8));
        drive(&mut domain, Time::ZERO, Event::Went { place: crate::address::Place::Elsewhere });
        if unknown {
            drive(&mut domain, Time::ZERO, Reported { request, progress: Progress::Unknown });
        } else {
            answered(&mut domain, request, Answer::Done(Outcome::Changed));
        }
        let (notice, ()) = domain.notices().ids().iter().next().unwrap();
        let expected = if unknown {
            Kind::Unknown { object: ObjectKey::Task(8), act: ActKind::Stop }
        } else {
            Kind::Answer { object: ObjectKey::Task(8), answer: Answer::Done(Outcome::Changed) }
        };
        assert_eq!(domain.notices().get(*notice).unwrap().kind, expected);
        assert!(domain.objects().find(ObjectKey::Task(8)).is_none());
    }
}

#[test]
fn every_owner_outcome_remains_whole_on_its_original_card_after_duplicate_terminals() {
    use OwnerOutcome as O;
    let outcomes = [
        O::Closing { chat: 101, cancelled: 2, kept: 3 },
        O::MarkedSeen { seen: Seen::Results { through: 103 } },
        O::MarkedSeen { seen: Seen::Replies { chat: 104, through: 105 } },
        O::NoteEdited { name: 106 },
        O::ServiceMade { person: 107 },
        O::Adopted { project: 108 },
        O::GoalStarted { task: 109 },
        O::GoalProposed { proposal: 110 },
        O::Stopped { task: 111 },
        O::Cancelled { task: 112 },
        O::Released { task: 113 },
        O::PersonTaken { task: 114 },
        O::PersonHandedBack { task: 115 },
        O::PersonAnswered { task: 116 },
        O::Moved { task: 117 },
        O::ProposalDecided {
            proposal: ProposalKey { proposer: Proposer::Party(118), number: 119 },
            by: 120,
            at: Wall::from_nanos(121),
            choice: OwnerProposalChoice::Stale,
        },
        O::Said { task: 122, message: 123, released: false },
        O::QuestionAnswered { task: 124, question: 125, message: 126 },
        O::Prioritised { project: 127 },
        O::Amended { task: 128, released: true },
        O::AmendProposed { task: 129, proposal: 130, released: false },
        O::RolesSet { project: 131 },
        O::PolicyChanged { project: 132 },
        O::PoolSet { project: 133, person: 134 },
        O::EscalationDecided {
            task: 135,
            revision: Revision { value: 136 },
            by: 137,
            at: Wall::from_nanos(138),
            choice: OwnerEscalationChoice::Passed,
        },
        O::Started { task: 139 },
    ];
    for value in outcomes {
        let (mut domain, _) = live();
        let id = install(&mut domain, Body::Task(Box::new(task())));
        let request = send(&mut domain, ObjectKey::Task(8));
        let answer = Answer::Done(Outcome::Owner(value));
        assert!(answered(&mut domain, request, answer.clone()).is_empty());
        assert_eq!(domain.objects().get(id).unwrap().card, Card::Open);
        assert_eq!(domain.objects().get(id).unwrap().answer.as_ref(), Some(&answer));
        assert!(answered(&mut domain, request, Answer::Refused(Refusal::Unknown)).is_empty());
        assert_eq!(domain.objects().get(id).unwrap().answer, Some(answer));
    }
}

#[test]
fn owner_historical_proposal_choices_keep_the_named_winner_and_clock_without_explanations() {
    for maker in [Proposer::Task(u64::MAX), Proposer::Party(u64::MAX)] {
        for choice in [
            OwnerProposalChoice::Accepted,
            OwnerProposalChoice::Rejected,
            OwnerProposalChoice::Passed,
            OwnerProposalChoice::Withdrawn,
            OwnerProposalChoice::Stale,
        ] {
            let (mut domain, _) = live();
            let mut value = proposal();
            value.key = ProposalKey { proposer: maker, number: 41 };
            let key = value.key;
            let id = install(&mut domain, Body::Proposal(Box::new(value)));
            let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
            let request = send_card(
                &mut domain.ui,
                ObjectKey::Proposal(key),
                ActKind::Accept,
                Box::new(Ask::DecideProposal { project: 1, proposal: key, decision: Decision::Accepted }),
                &mut out,
                &domain.client,
            )
            .unwrap();
            assert_eq!(drain(out).len(), 1);
            let outcome = Outcome::Owner(OwnerOutcome::ProposalDecided {
                proposal: key,
                by: 71,
                at: Wall::from_nanos(91),
                choice,
            });
            answered(&mut domain, request, Answer::Done(outcome.clone()));
            assert_eq!(
                domain.objects().get(id).unwrap().card,
                Card::Leaving {
                    how: Left::Answered { outcome: outcome.clone() },
                    until: Time::from_nanos(5_000_000_000)
                }
            );
            assert_eq!(domain.objects().get(id).unwrap().answer, Some(Answer::Done(outcome)));
        }
    }
    for value in [
        OwnerOutcome::GoalProposed { proposal: u64::MAX },
        OwnerOutcome::AmendProposed { task: 8, proposal: u64::MAX, released: true },
    ] {
        assert!(!crate::objects::stops_waiting(ObjectKey::Proposal(proposal().key), &Outcome::Owner(value)));
    }
}
