use super::fixtures::{self, PROPOSAL, SUBSCRIPTION, TARGET};
use jig_domain_tasks::{MessageKind, NoticeState, SubscriptionKind};

#[test]
fn an_escalation_reads_with_its_task_and_hold_revision() {
    let word = fixtures::word(MessageKind::Escalation { task: TARGET, revision: 2 });
    assert_eq!(crate::words(&word).as_ref(), b"Escalation: task 3 is held at hold revision 2:\nHello");
}
#[test]
fn a_held_notice_reads_with_its_hold_revision() {
    let word = fixtures::word(MessageKind::Notice {
        subscription: SUBSCRIPTION,
        target: TARGET,
        state: NoticeState::Held { revision: 2 },
    });
    assert_eq!(crate::words(&word).as_ref(), b"Task 3 is held at hold revision 2:\nHello");
}
const DECISIONS: [(bool, &[u8]); 2] =
    [(true, b"Proposal accepted 2:\nHello".as_slice()), (false, b"Proposal rejected 2:\nHello".as_slice())];
#[test]
fn a_proposal_decision_reads_as_inbox_words_did() {
    for (accepted, expected) in DECISIONS {
        let word = fixtures::word(MessageKind::ProposalDecision { proposal: PROPOSAL, accepted });
        assert_eq!(crate::words(&word).as_ref(), expected, "accepted: {accepted}");
    }
}
#[test]
fn a_subscribe_call_may_ask_to_hear_a_tasks_holds() {
    match crate::decode(b"subscribe", br#"{"kind":"task","target":3,"held":true}"#, &fixtures::LIMITS) {
        crate::Decoded::Action(jig_domain_core::NamedAction::Subscribe { kind }) => {
            assert_eq!(kind, SubscriptionKind::Task { target: TARGET, held: true, result: true });
        }
        other @ (crate::Decoded::Action(_)
        | crate::Decoded::Delegate(_)
        | crate::Decoded::Problem(_)
        | crate::Decoded::NotMine) => panic!("unexpected subscription {other:?}"),
    }
}
