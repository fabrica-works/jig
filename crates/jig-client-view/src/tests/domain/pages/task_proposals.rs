//! The Task document supplies proposal identities independently of transcript calls.
use super::*;
use crate::objects::{Body, Card, Left};
use crate::tests::fixtures::{Ended, Fixture, ReadPage, Watched};

fn opened() -> (Fixture, jig_client::Handle) {
    let (mut domain, _) = super::super::objects::live();
    let out = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::task(8)) });
    (domain, open_token(&out))
}
fn task(keys: &[ProposalKey], activity: u64) -> Task {
    let mut task = super::super::confirmation::task();
    task.context = None;
    task.activity = activity;
    task.proposals = List::with_capacity(2);
    for key in keys {
        task.proposals.push(*key).unwrap();
    }
    task
}
fn deliver(domain: &mut Fixture, watch: jig_client::Handle, value: Task) -> Box<[Request]> {
    drive(domain, Time::ZERO, Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(value))) })
}
fn proposal_read(out: &[Request], key: ProposalKey) -> jig_client::Handle {
    let mut found = None;
    for row in out {
        if let Request::Client {
            asked:
                jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Proposal { proposal }) },
        } = row
            && *proposal == key
        {
            assert!(found.replace(*read).is_none(), "one read per supplied proposal");
        }
    }
    found.expect("supplied proposal is read")
}
fn proposal(key: ProposalKey) -> Proposal {
    let mut value = super::super::objects::proposal();
    value.key = key;
    value.proposer = key.proposer;
    value
}
fn receive(domain: &mut Fixture, read: jig_client::Handle, value: Proposal) {
    drive(domain, Time::ZERO, ReadPage { read, result: ReadResult::Value(ReadValue::Proposal(Box::new(value))) });
}
#[test]
fn task_watch_names_literal_proposals_once_and_refreshes_whole_shared_cards_when_activity_moves() {
    for proposer in [Proposer::Task(8), Proposer::Party(8)] {
        let key = ProposalKey { proposer, number: 1 };
        let (mut domain, watch) = opened();
        let output = deliver(&mut domain, watch, task(&[key, key], 11));
        let read = proposal_read(&output, key);
        let id = domain.objects().find(ObjectKey::Proposal(key)).unwrap();
        assert_eq!(domain.objects().len(), 2);
        assert_eq!(domain.task_page().proposals.as_slice(), &[key]);
        assert_eq!(domain.conversation().names(2).as_slice(), &[]);
        assert!(deliver(&mut domain, watch, task(&[key], 11)).is_empty());
        let value = proposal(key);
        receive(&mut domain, read, value.clone());
        assert_eq!(domain.objects().get(id).unwrap().body, Some(Body::Proposal(Box::new(value))));
        assert_eq!(domain.objects().get(id).unwrap().card, Card::Open);
        assert!(deliver(&mut domain, watch, task(&[key], 11)).is_empty());
        let changed = deliver(&mut domain, watch, task(&[key], 12));
        let read = proposal_read(&changed, key);
        assert_eq!(domain.objects().find(ObjectKey::Proposal(key)), Some(id));
        let mut moved = proposal(key);
        moved.why = text(b"whole new reason");
        receive(&mut domain, read, moved.clone());
        assert_eq!(domain.objects().get(id).unwrap().body, Some(Body::Proposal(Box::new(moved))));
        assert_eq!(domain.objects().get(id).unwrap().card, Card::Open);
    }
}
#[test]
fn a_task_proposal_removed_from_its_last_watch_reads_its_actual_ending_and_keeps_its_departing_card() {
    let key = ProposalKey { proposer: Proposer::Task(8), number: 1 };
    let (mut domain, watch) = opened();
    let out = deliver(&mut domain, watch, task(&[key], 11));
    let read = proposal_read(&out, key);
    receive(&mut domain, read, proposal(key));
    let id = domain.objects().find(ObjectKey::Proposal(key)).unwrap();
    let removed = deliver(&mut domain, watch, task(&[], 12));
    let read = proposal_read(&removed, key);
    let Card::Leaving { how: Left::Looking, .. } = domain.objects().get(id).unwrap().card else {
        panic!("last-source removal retains a final read");
    };
    assert_eq!(domain.task_page().proposals.as_slice(), &[key]);
    let mut ended = proposal(key);
    ended.ending =
        Some(Ending::Decided { by: 9, party: None, how: Some(Decision::Accepted), at: skein_lib::Wall::EPOCH });
    receive(&mut domain, read, ended.clone());
    assert_eq!(domain.objects().get(id).unwrap().body, Some(Body::Proposal(Box::new(ended))));
    let Card::Leaving { how: Left::Ended { .. }, .. } = domain.objects().get(id).unwrap().card else {
        panic!("actual supplied ending determines the departing card");
    };
    assert_eq!(domain.task_page().proposals.as_slice(), &[key]);
    drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
    assert!(domain.task_page().proposals.is_empty());
    assert!(deliver(&mut domain, watch, task(&[key], 13)).is_empty());
    assert!(domain.task_page().proposals.is_empty());
}

#[test]
fn a_genuine_watch_ending_keeps_a_shown_waiting_proposal_until_its_supplied_withdrawal() {
    for end in [WatchEnd::Unknown, WatchEnd::SubjectEnded] {
        let key = ProposalKey { proposer: Proposer::Task(8), number: 1 };
        let (mut domain, watch) = opened();
        let output = deliver(&mut domain, watch, task(&[key], 11));
        receive(&mut domain, proposal_read(&output, key), proposal(key));
        let id = domain.objects().find(ObjectKey::Proposal(key)).unwrap();
        assert!(domain.objects().get(id).unwrap().shown);
        let output = drive(&mut domain, Time::ZERO, Ended { watch, end });
        let read = proposal_read(&output, key);
        assert_eq!(domain.objects().find(ObjectKey::Proposal(key)), Some(id));
        assert!(domain.objects().get(id).unwrap().sources.is_empty());
        let Card::Leaving { how: Left::Looking, .. } = domain.objects().get(id).unwrap().card else {
            panic!("genuine source ending retains the shown proposal for its final read");
        };
        assert!(drive(&mut domain, Time::ZERO, Ended { watch, end }).is_empty());
        let supplied = OwnerHistorical {
            key: ObjectKey::Proposal(key),
            value: OwnerHistoricalValue::Proposal(OwnerProposalHistory {
                action: OwnerRetainedAction::Batch(List::with_capacity(0)),
                number: key.number,
                proposer: 8,
                as_holder: false,
                state: OwnerProposalHistoryState::Withdrawn,
            }),
            at: None,
            by: None,
        };
        drive(
            &mut domain,
            Time::ZERO,
            ReadPage { read, result: ReadResult::Value(ReadValue::Historical(Box::new(supplied.clone()))) },
        );
        assert_eq!(domain.objects().get(id).unwrap().body, Some(Body::Historical(Box::new(supplied.clone()))));
        let Card::Leaving { how: Left::Historical { .. }, .. } = domain.objects().get(id).unwrap().card else {
            panic!("the supplied withdrawal explains the retained proposal");
        };
        drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
        assert!(!domain.objects().get(id).unwrap().shown);
        assert!(drive(&mut domain, Time::ZERO, Ended { watch, end }).is_empty());
    }
    let key = ProposalKey { proposer: Proposer::Task(8), number: 1 };
    let (mut domain, watch) = opened();
    let output = deliver(&mut domain, watch, task(&[key], 11));
    receive(&mut domain, proposal_read(&output, key), proposal(key));
    let output = drive(&mut domain, Time::ZERO, Ended { watch, end: WatchEnd::NotAllowed });
    assert!(domain.objects().find(ObjectKey::Proposal(key)).is_none());
    for request in &output {
        if let Request::Client {
            asked: jig_client::Event::Read { read: jig_client::Readable::Jig(Read::Proposal { .. }), .. },
        } = request
        {
            panic!("privacy retirement must not read the hidden proposal");
        }
    }
}
#[test]
fn task_proposal_registration_keeps_the_header_then_refuses_excess_distinct_makers_without_guessing_identity() {
    let task_key = ProposalKey { proposer: Proposer::Task(8), number: 1 };
    let party_key = ProposalKey { proposer: Proposer::Party(8), number: 1 };
    let (mut domain, watch) = opened();
    let out = deliver(&mut domain, watch, task(&[task_key, party_key], 11));
    proposal_read(&out, task_key);
    assert!(domain.task_page().header.is_some());
    assert_eq!(domain.task_page().proposals.as_slice(), &[task_key]);
    assert!(domain.objects().find(ObjectKey::Proposal(party_key)).is_none());
    assert!(domain.objects().find(ObjectKey::Task(8)).is_some());
    let Page::Work { excess: 1, refused: Some(Limit::Objects), .. } = domain.page() else {
        panic!("one excess object is refused in place");
    };
}
