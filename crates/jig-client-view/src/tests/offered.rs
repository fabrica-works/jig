#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary public tree proofs inspect nodes and decoded meanings"
)]
use super::cards::{CLIENT, contains, words};
use super::cards_objects::{loaded, task_doc};
use crate::{Act, Builder, JigBindings, Role, cards, decode};
use jig_client::{action::Act as Action, contract::*};
use skein_lib::{List, Wall};

const KINDS: [ActKind; 25] = [
    ActKind::Write,
    ActKind::Answer,
    ActKind::Take,
    ActKind::HandBack,
    ActKind::Accept,
    ActKind::Reject,
    ActKind::PassUp,
    ActKind::Release,
    ActKind::LeaveHeld,
    ActKind::Amend,
    ActKind::AmendRelease,
    ActKind::Stop,
    ActKind::Cancel,
    ActKind::Close,
    ActKind::TakeOver,
    ActKind::SetGoal,
    ActKind::Prioritise,
    ActKind::MarkSeen,
    ActKind::EditNote,
    ActKind::DeleteNote,
    ActKind::Adopt,
    ActKind::ChangePolicy,
    ActKind::SetRoles,
    ActKind::MakeService,
    ActKind::SetPool,
];
#[test]
fn every_closed_offer_control_and_whole_refusal_fits_the_public_card_tree_with_two_paged_rows() {
    assert_eq!(u32::try_from(KINDS.len()).unwrap(), MAX_OFFERED_ACTS);
    let w = words();
    let mut limits = crate::Limits::of_words(&CLIENT, &w).unwrap();
    limits.nodes = crate::card_nodes(&CLIENT).unwrap() + 1;
    limits.text_bytes = limits.nodes * limits.text_per_node;
    for refused in [false, true] {
        let mut value = task_doc(&KINDS);
        if refused {
            let reason = HoldReason::Tries {
                failures: {
                    let mut failures = List::with_capacity(CLIENT.rows);
                    for attempt in [0, u64::MAX] {
                        failures
                            .push(AttemptFailure {
                                attempt,
                                class: FailureClass::HostLost,
                                reason: super::cards::text(b"whole failure"),
                                at: Wall::from_nanos(u64::MAX),
                            })
                            .unwrap();
                    }
                    failures
                },
            };
            for index in 0..value.acts.len() {
                let entry = value.acts.get_mut(index).unwrap();
                entry.admission = Admission::Refused(Refusal::Moved {
                    now: Revision { value: u64::MAX },
                    what: Moved::Hold { reason: Some(CurrentHold::Document(Box::new(reason.clone()))) },
                });
            }
        }
        let before = value.clone();
        let (domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(value.clone())));
        let mut bindings = JigBindings::new(limits.bindings, CLIENT.text);
        let mut builder = Builder::new(&limits);
        builder.open(Role::Page);
        cards::card(&mut builder, &domain, id, &cards::Place::Elsewhere, &w, &mut bindings);
        builder.close();
        let tree = builder.finish();
        super::cards_access::check(&tree);
        assert!(tree.len() <= limits.nodes);
        let mut total = 0_u64;
        let mut decoded = Vec::new();
        for node in tree.nodes() {
            assert!(node.text_bytes() <= limits.text_per_node);
            total += u64::from(node.text_bytes());
            if node.role == Role::Button
                && let Some(binding) = node.binding
            {
                if let Some(Action::Intend { object, act }) = decode(&mut bindings, binding.token, Act::Press) {
                    assert_eq!(object, id.token());
                    decoded.push(act);
                }
            }
        }
        assert!(total <= u64::from(limits.text_bytes));
        if refused {
            assert!(decoded.is_empty());
            assert!(contains(&tree, b"whole failure"));
            assert!(contains(&tree, b"18446744073709551615"));
            let revisions = tree
                .nodes()
                .iter()
                .filter(|node| {
                    node.text.iter().any(|run| skein_lib::bytes::find(&run.text, b"18446744073709551615").is_some())
                })
                .count();
            assert!(revisions >= usize::try_from(MAX_OFFERED_ACTS).unwrap());
        } else {
            assert_eq!(decoded, KINDS);
        }
        assert_eq!(value, before);
        assert_eq!(CLIENT.rows, 2);
    }
}
#[test]
fn pending_proposal_offer_capacity_is_independent_while_funding_rows_stay_bounded() {
    let mut value = OwnerProposal {
        key: ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX },
        project: u32::MAX,
        reason: DocumentText::complete(super::cards::text(b"whole reason")),
        action: OwnerProposalAction::Release {
            task: OwnerTaskLink::Stored { number: u64::MAX, status: None },
            hold: None,
        },
        as_holder: true,
        waiting: OwnerProposalWaiting { holder: OwnerHistoryHolder::Task(u64::MAX), since: Wall::from_nanos(u64::MAX) },
        funding: List::with_capacity(CLIENT.rows),
        acts: List::with_capacity(MAX_OFFERED_ACTS),
    };
    let w = words();
    let original = value.clone();
    let requirement = crate::owner_proposal::required(&value, &CLIENT, &w).unwrap();
    assert!(requirement.nodes <= crate::proposal_nodes(&CLIENT).unwrap());
    value.acts = List::with_capacity(MAX_OFFERED_ACTS + 1);
    assert!(crate::owner_proposal::required(&value, &CLIENT, &w).is_none());
    value.acts = original.acts.clone();
    value.funding = List::with_capacity(CLIENT.rows + 1);
    assert!(crate::owner_proposal::required(&value, &CLIENT, &w).is_none());
    assert_eq!(original.acts.capacity(), MAX_OFFERED_ACTS);
}
