use super::{
    objects,
    spine::{LIMITS, drive},
};
use crate::{
    Domain, Event, Request,
    address::{Address, Place},
    objects::Body,
};
use jig_client_domain::contract::*;
use skein_lib::{Env, List, Queue, Time, Token, Wall};

fn source(maker: Proposer, part: OwnerPart) -> OwnerProposal {
    OwnerProposal {
        key: ProposalKey { proposer: maker, number: 7 },
        project: 41,
        reason: DocumentText { bytes: objects::text(UTF8_TEXT), cut: true },
        action: OwnerProposalAction::Effect {
            part,
            authority: objects::owner_authority(),
            attempt: u64::MAX,
            completion: u32::MAX,
            position: 0,
            purpose: u64::MAX,
        },
        as_holder: false,
        waiting: OwnerProposalWaiting { holder: OwnerHistoryHolder::Person(7), since: Wall::EPOCH },
        funding: List::with_capacity(2),
        acts: List::with_capacity(2),
    }
}
fn native(number: u64) -> OwnerPart {
    OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Effect, key: OwnerPartKey::Number(number) }
}
fn shown(domain: &mut Domain, key: ProposalKey) -> (Token, Token) {
    drive(domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
    let mut names = List::with_capacity(1);
    names.push(ObjectKey::Proposal(key)).unwrap();
    let output = drive(domain, Time::ZERO, Event::ShowObjects { objects: names });
    let mut read = None;
    let mut watch = None;
    for request in &output {
        if let Request::Read { read: token, what: Read::Proposal { proposal } } = request
            && *proposal == key
        {
            assert!(read.replace(*token).is_none());
        }
        if let Request::Open { watch: token, subject: Subject::Inbox { .. } } = request {
            assert!(watch.replace(*token).is_none());
        }
    }
    (read.unwrap(), watch.unwrap())
}
fn parts(output: &[Request]) -> Box<[Request]> {
    let mut parts = List::with_capacity(LIMITS.parts);
    for request in output {
        if let Request::ShowPart { .. } = request {
            parts.push(request.clone()).unwrap();
        }
    }
    parts.into_boxed()
}
fn exact(domain: &Domain, source: &OwnerProposal) {
    let object = domain.objects().get(domain.objects().find(ObjectKey::Proposal(source.key)).unwrap()).unwrap();
    assert_eq!(object.body, Some(Body::OwnerProposal(Box::new(source.clone()))));
    assert_eq!(object.project, Some(source.project));
}

#[test]
fn application_native_effect_parts_preserve_raw_numbers_scope_whole_facts_and_their_generation_without_maker_inference()
{
    for maker in [Proposer::Task(8), Proposer::Party(8)] {
        let (mut domain, _) = objects::live();
        let mut value = source(maker, native(u64::MAX));
        let (read, watch) = shown(&mut domain, value.key);
        let output = drive(
            &mut domain,
            Time::ZERO,
            Event::Read { read, result: ReadResult::Value(ReadValue::OwnerProposal(Box::new(value.clone()))) },
        );
        assert_eq!(
            parts(&output).as_ref(),
            &[Request::ShowPart {
                part: Part::OwnerEffect { connector: u16::MAX, proposal: u64::MAX },
                generation: 1,
                scope: PartScope::Project(41),
            }]
        );
        exact(&domain, &value);
        value.action = OwnerProposalAction::Effect {
            part: native(0),
            authority: objects::owner_authority(),
            attempt: 0,
            completion: 0,
            position: u32::MAX,
            purpose: 19,
        };
        let output = drive(
            &mut domain,
            Time::ZERO,
            Event::Watch { watch, event: WatchEvent::Delivered(Delivery::OwnerProposal(Box::new(value.clone()))) },
        );
        assert_eq!(
            parts(&output).as_ref(),
            &[Request::ShowPart {
                part: Part::OwnerEffect { connector: u16::MAX, proposal: 0 },
                generation: 1,
                scope: PartScope::Project(41),
            }]
        );
        exact(&domain, &value);
        assert!(
            drive(
                &mut domain,
                Time::ZERO,
                Event::Read {
                    read,
                    result: ReadResult::Value(ReadValue::OwnerProposal(Box::new(source(maker, native(u64::MAX)))))
                }
            )
            .is_empty()
        );
        exact(&domain, &value);
        let left = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Inbox { project: None }) });
        assert!(left.contains(&Request::PartsLeft { generation: 1 }));
        assert!(left.contains(&Request::Close { watch }));
        let late = drive(
            &mut domain,
            Time::ZERO,
            Event::Watch { watch, event: WatchEvent::Delivered(Delivery::OwnerProposal(Box::new(value.clone()))) },
        );
        assert!(parts(&late).is_empty());
        assert!(domain.objects().find(ObjectKey::Proposal(value.key)).is_none());
    }
}

#[test]
fn native_effect_registration_refuses_other_kinds_and_paths_and_preserves_the_separate_maker_effect_key() {
    let mut invalid = List::with_capacity(7);
    invalid.push(OwnerPart { key: OwnerPartKey::Path(List::with_capacity(0)), ..native(99) }).unwrap();
    for kind in [
        OwnerPartKind::Resource,
        OwnerPartKind::Pool,
        OwnerPartKind::Result,
        OwnerPartKind::Procedure,
        OwnerPartKind::Outbox,
        OwnerPartKind::Hold,
    ] {
        invalid.push(OwnerPart { kind, ..native(99) }).unwrap();
    }
    for part in invalid.into_boxed() {
        let (mut domain, _) = objects::live();
        let value = source(Proposer::Task(8), part);
        let (read, _) = shown(&mut domain, value.key);
        let output = drive(
            &mut domain,
            Time::ZERO,
            Event::Read { read, result: ReadResult::Value(ReadValue::OwnerProposal(Box::new(value.clone()))) },
        );
        assert!(parts(&output).is_empty());
        exact(&domain, &value);
    }
    let (mut domain, _) = objects::live();
    let mut value = objects::proposal();
    let maker_part = Part::Effect { connector: 2, proposal: ProposalKey { proposer: Proposer::Party(99), number: 17 } };
    value.action = ProposalAction::Effect { part: maker_part.clone() };
    let (read, _) = shown(&mut domain, value.key);
    let output = drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read, result: ReadResult::Value(ReadValue::Proposal(Box::new(value.clone()))) },
    );
    assert_eq!(
        parts(&output).as_ref(),
        &[Request::ShowPart { part: maker_part, generation: 1, scope: PartScope::Project(1) }]
    );
    let object = domain.objects().get(domain.objects().find(ObjectKey::Proposal(value.key)).unwrap()).unwrap();
    assert_eq!(object.body, Some(Body::Proposal(Box::new(value))));
}

#[test]
fn native_effect_parts_refuse_full_capacity_while_retaining_the_whole_pending_document_and_offers() {
    let (mut domain, _) = objects::live();
    let mut value = source(Proposer::Party(8), native(11));
    value.acts.push(Offered { act: ActKind::Accept, admission: Admission::Allowed }).unwrap();
    let (read, _) = shown(&mut domain, value.key);
    let limits = crate::Limits { parts: 0, ..LIMITS };
    let mut output = Queue::with_capacity(crate::max_out(&limits));
    crate::step(
        &mut domain,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        Event::Read { read, result: ReadResult::Value(ReadValue::OwnerProposal(Box::new(value.clone()))) },
        &mut output,
    );
    while let Some(request) = output.pop() {
        if let Request::ShowPart { .. } = request {
            panic!("full capacity cannot emit a part");
        }
    }
    exact(&domain, &value);
    let (notice, ()) = domain.notices().ids().iter().next().unwrap();
    assert_eq!(domain.notices().get(*notice).unwrap().kind, crate::notices::Kind::Refused(Limit::Objects));
}

fn confirmed_accept(domain: &mut Domain, object: Token, value: &OwnerProposal) -> Token {
    assert!(
        drive(domain, Time::ZERO, Event::Act { act: crate::action::Act::Intend { object, act: ActKind::Accept } })
            .is_empty()
    );
    assert!(domain.confirmation().is_some());
    let sent = drive(domain, Time::ZERO, Event::Act { act: crate::action::Act::Confirm });
    assert_eq!(sent.len(), 1);
    let mut request = None;
    for row in &sent {
        if let Request::Send { request: token, ask, .. } = row {
            assert_eq!(
                ask.as_ref(),
                &Ask::DecideProposal { project: value.project, proposal: value.key, decision: Decision::Accepted }
            );
            assert!(request.replace(*token).is_none());
        } else {
            panic!("confirmation emits only its exact supplied decision");
        }
    }
    let request = request.unwrap();
    assert!(drive(domain, Time::ZERO, Event::Act { act: crate::action::Act::Confirm }).is_empty());
    request
}

#[test]
fn native_effect_card_refresh_registers_only_shown_current_facts_within_capacity_and_never_resends() {
    for (shown_card, part_capacity) in [(true, LIMITS.parts), (true, 0), (false, LIMITS.parts)] {
        let (mut domain, _) = objects::live();
        let mut value = source(Proposer::Party(8), native(11));
        value.acts.push(Offered { act: ActKind::Accept, admission: Admission::Allowed }).unwrap();
        let (read, _) = shown(&mut domain, value.key);
        drive(
            &mut domain,
            Time::ZERO,
            Event::Read { read, result: ReadResult::Value(ReadValue::OwnerProposal(Box::new(value.clone()))) },
        );
        let id = domain.objects().find(ObjectKey::Proposal(value.key)).unwrap();
        let request = confirmed_accept(&mut domain, id.token(), &value);
        assert!(
            drive(&mut domain, Time::ZERO, Event::Progress { request, progress: crate::intents::Progress::Unknown })
                .is_empty()
        );
        let output =
            drive(&mut domain, Time::ZERO, Event::Act { act: crate::action::Act::LookAgain { object: id.token() } });
        let mut current_read = None;
        for request in &output {
            if let Request::Read { read, what: Read::Proposal { proposal } } = request {
                assert_eq!(*proposal, value.key);
                assert!(current_read.replace(*read).is_none());
            } else {
                panic!("inspection only reads, never resends the decision");
            }
        }
        if !shown_card {
            exact(&domain, &value);
            let left =
                drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Inbox { project: None }) });
            assert!(left.contains(&Request::PartsLeft { generation: 1 }));
            assert!(domain.objects().find(ObjectKey::Proposal(value.key)).is_none());
        }
        value.action = OwnerProposalAction::Effect {
            part: native(17),
            authority: objects::owner_authority(),
            attempt: 1,
            completion: 2,
            position: 3,
            purpose: 19,
        };
        let limits = crate::Limits { parts: part_capacity, ..LIMITS };
        let mut output = Queue::with_capacity(crate::max_out(&limits));
        let event = Event::Read {
            read: current_read.unwrap(),
            result: ReadResult::Value(ReadValue::OwnerProposal(Box::new(value.clone()))),
        };
        crate::step(&mut domain, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits }, event.clone(), &mut output);
        let output = super::spine::drain(output);
        if shown_card && part_capacity > 0 {
            assert_eq!(
                parts(&output).as_ref(),
                &[Request::ShowPart {
                    part: Part::OwnerEffect { connector: u16::MAX, proposal: 17 },
                    generation: 1,
                    scope: PartScope::Project(41)
                }]
            );
        } else {
            assert!(parts(&output).is_empty());
        }
        if shown_card && part_capacity == 0 {
            let (notice, ()) = domain.notices().ids().iter().next().unwrap();
            assert_eq!(domain.notices().get(*notice).unwrap().kind, crate::notices::Kind::Refused(Limit::Objects));
        }
        if shown_card {
            exact(&domain, &value);
            assert_eq!(domain.objects().get(id).unwrap().card, crate::objects::Card::Open);
        } else {
            assert!(output.is_empty());
            assert!(domain.objects().find(ObjectKey::Proposal(value.key)).is_none());
        }
        assert!(drive(&mut domain, Time::ZERO, event).is_empty());
        if shown_card {
            exact(&domain, &value);
        } else {
            assert!(domain.objects().find(ObjectKey::Proposal(value.key)).is_none());
        }
    }
}

const UTF8_TEXT: &[u8] = b"~\0\x11";
