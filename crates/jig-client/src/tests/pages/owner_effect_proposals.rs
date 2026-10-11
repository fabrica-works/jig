//! Native proposal page reads and watches register only supplied connector parts.
use super::*;
use crate::objects::{Body, Card};
use skein_lib::{Env, Queue, Token, Wall};

fn proposal(proposer: Proposer, effect: u64) -> OwnerProposal {
    let mut acts = List::with_capacity(1);
    acts.push(Offered { act: ActKind::Accept, admission: Admission::Allowed }).unwrap();
    OwnerProposal {
        key: ProposalKey { proposer, number: 7 },
        project: 41,
        reason: DocumentText { bytes: text(&[255, 0, 17]), cut: true },
        action: OwnerProposalAction::Effect {
            part: OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Effect, key: OwnerPartKey::Number(effect) },
            authority: super::super::objects::owner_authority(),
            attempt: u64::MAX,
            completion: u32::MAX,
            position: 0,
            purpose: 19,
        },
        as_holder: false,
        waiting: OwnerProposalWaiting { holder: OwnerHistoryHolder::Person(7), since: Wall::EPOCH },
        funding: List::with_capacity(2),
        acts,
    }
}
fn named(domain: &mut Domain, watch: Token, key: ProposalKey) -> Token {
    let mut entries = List::with_capacity(1);
    let mut row = entry(8, InboxGroup::Proposals, 0);
    row.object = ObjectKey::Proposal(key);
    entries.push(row).unwrap();
    let output = inbox_delivery(domain, watch, entries);
    let mut found = None;
    for request in &output {
        if let Request::Read { read, what: Read::Proposal { proposal } } = request
            && *proposal == key
        {
            assert!(found.replace(*read).is_none());
        }
    }
    found.unwrap()
}
fn parts(output: &[Request]) -> Box<[Request]> {
    let mut result = List::with_capacity(LIMITS.parts);
    for request in output {
        if let Request::ShowPart { .. } = request {
            result.push(request.clone()).unwrap();
        }
    }
    result.into_boxed()
}
fn exact(domain: &Domain, value: &OwnerProposal) {
    let id = domain.objects().find(ObjectKey::Proposal(value.key)).unwrap();
    let object = domain.objects().get(id).unwrap();
    assert_eq!(object.body, Some(Body::OwnerProposal(Box::new(value.clone()))));
    assert_eq!(object.project, Some(value.project));
    assert_eq!(object.card, Card::Open);
}
fn supplied(generation: u32, effect: u64) -> Request {
    Request::ShowPart {
        part: Part::OwnerEffect { connector: u16::MAX, proposal: effect },
        generation,
        scope: PartScope::Project(41),
    }
}

#[test]
fn native_proposal_page_reads_and_watches_keep_whole_facts_scope_and_identity_without_late_resurrection() {
    for proposer in [Proposer::Task(8), Proposer::Party(8)] {
        let (mut domain, watch) = live_inbox(10);
        let original = proposal(proposer, u64::MAX);
        let read = named(&mut domain, watch, original.key);
        let event =
            Event::Read { read, result: ReadResult::Value(ReadValue::OwnerProposal(Box::new(original.clone()))) };
        let output = drive(&mut domain, Time::ZERO, event.clone());
        assert_eq!(parts(&output).as_ref(), &[supplied(0, u64::MAX)]);
        exact(&domain, &original);
        let id = domain.objects().find(ObjectKey::Proposal(original.key)).unwrap();
        let mut changed = proposal(proposer, 0);
        changed.reason = DocumentText::complete(text(b"new supplied facts"));
        changed.as_holder = true;
        let delivery =
            Event::Watch { watch, event: WatchEvent::Delivered(Delivery::OwnerProposal(Box::new(changed.clone()))) };
        let output = drive(&mut domain, Time::ZERO, delivery.clone());
        assert_eq!(parts(&output).as_ref(), &[supplied(0, 0)]);
        assert_eq!(domain.objects().find(ObjectKey::Proposal(changed.key)), Some(id));
        exact(&domain, &changed);
        assert!(drive(&mut domain, Time::ZERO, event.clone()).is_empty());
        exact(&domain, &changed);
        let left = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Deployment) });
        assert!(left.contains(&Request::PartsLeft { generation: 0 }));
        assert!(left.contains(&Request::Close { watch }));
        assert!(domain.objects().find(ObjectKey::Proposal(original.key)).is_none());
        assert!(drive(&mut domain, Time::ZERO, delivery).is_empty());
        assert!(drive(&mut domain, Time::ZERO, event).is_empty());
        assert!(domain.objects().find(ObjectKey::Proposal(original.key)).is_none());
    }
}

#[test]
fn native_proposal_page_reads_and_watches_refuse_part_capacity_while_preserving_whole_offered_documents() {
    for watch_delivery in [false, true] {
        let (mut domain, watch) = live_inbox(10);
        let value = proposal(Proposer::Party(8), 17);
        let read = named(&mut domain, watch, value.key);
        let event = if watch_delivery {
            Event::Watch { watch, event: WatchEvent::Delivered(Delivery::OwnerProposal(Box::new(value.clone()))) }
        } else {
            Event::Read { read, result: ReadResult::Value(ReadValue::OwnerProposal(Box::new(value.clone()))) }
        };
        let limits = crate::Limits { parts: 0, ..LIMITS };
        let mut output = Queue::with_capacity(crate::max_out(&limits));
        crate::step(&mut domain, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits }, event, &mut output);
        while let Some(request) = output.pop() {
            if let Request::ShowPart { .. } = request {
                panic!("full part capacity cannot emit a connector part");
            }
        }
        exact(&domain, &value);
        let (notice, ()) = domain.notices().ids().iter().next().unwrap();
        assert_eq!(domain.notices().get(*notice).unwrap().kind, crate::notices::Kind::Refused(Limit::Objects));
    }
}
