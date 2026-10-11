//! Shared proposals are rendered from supplied Task-page names, not invented calls.
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary proofs drive public events and inspect semantic trees"
)]
use super::{
    cards::{CLIENT, contains, drive, text, words},
    pages::deliver,
    work_pages::{entered, list, part},
};
use crate::{Builder, JigBindings, Role, Tree};
use jig_client::{
    Domain, Event, Request,
    address::{Address, Place, TaskSection},
    contract::*,
};
use skein_lib::{Token, Wall};
fn key(proposer: Proposer) -> ProposalKey {
    ProposalKey { proposer, number: 1 }
}
fn task(keys: &[ProposalKey], activity: u64) -> Task {
    let mut doc = super::cards_objects::task_doc(&[]);
    doc.chip.executor = Executor::Agent { charter: 2 };
    doc.chip.phase = Phase::Running;
    *doc.chip.details = ChipPhase::Running { attempt: 2, since: Wall::EPOCH, tool: None };
    doc.context = None;
    doc.hold = None;
    doc.proposals = list(keys.iter().copied());
    doc.activity = activity;
    doc
}
fn opened(keys: &[ProposalKey]) -> (Domain, Token, Vec<Request>) {
    let (mut d, out) = entered(Address::Task { task: 8, section: TaskSection::Activity });
    let watch = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Task { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let out = deliver(&mut d, watch, Delivery::Task(Box::new(task(keys, 11))));
    (d, watch, out)
}
fn proposal(key: ProposalKey) -> Proposal {
    Proposal {
        key,
        project: 4,
        proposer: key.proposer,
        why: text(b"whole shared reason"),
        action: ProposalAction::Effect { part: part() },
        needs: super::cards_objects::authority(),
        funding: Budget { total: 20, own: 0, ended: 0, reserved: 0, left: 20, funder: Actor::Party(1) },
        waiting: Recipient::Party(1),
        trail: list([]),
        since: Wall::EPOCH,
        acts: list([Offered { act: ActKind::Accept, admission: Admission::Allowed }]),
        ending: None,
    }
}
fn token(out: &[Request], key: ProposalKey) -> Token {
    out.iter()
        .find_map(|r| match r {
            Request::Read { read, what: Read::Proposal { proposal } } if *proposal == key => Some(*read),
            _ => None,
        })
        .unwrap()
}
fn receive(d: &mut Domain, out: &[Request], value: Proposal) {
    drive(
        d,
        Event::Read { read: token(out, value.key), result: ReadResult::Value(ReadValue::Proposal(Box::new(value))) },
    );
}
fn build(d: &Domain) -> Tree {
    let w = words();
    let nodes = crate::task_nodes(&CLIENT, 0)
        .unwrap()
        .checked_add(crate::confirmation_nodes(&CLIENT).unwrap())
        .unwrap()
        .checked_add(crate::notices_nodes(&CLIENT).unwrap())
        .unwrap()
        .checked_add(1)
        .unwrap();
    let mut limits = super::cards::limits();
    limits.nodes = nodes;
    limits.depth = 34;
    limits.bindings = nodes;
    limits.slots = nodes;
    limits.text_bytes = nodes.checked_mul(limits.text_per_node).unwrap();
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(nodes, CLIENT.text);
    b.open(Role::Page);
    crate::pages::task(&mut b, d, &w, &mut bindings);
    crate::pages::after_page(&mut b, d, &w, &mut bindings);
    b.close();
    let tree = b.finish();
    assert!(tree.len() <= nodes);
    assert!(crate::worst_case(&limits).is_some());
    assert!(crate::jig_bindings_worst_case(nodes, &CLIENT).is_some());
    super::cards_access::check(&tree);
    tree
}
fn cards(tree: &Tree, key: ProposalKey) -> usize {
    let wanted = crate::pages::object_key(ObjectKey::Proposal(key));
    tree.nodes().iter().filter(|n| n.key == Some(wanted)).count()
}
#[test]
fn a_task_activity_renders_whole_shared_proposals_without_any_decoded_call() {
    let first = key(Proposer::Task(8));
    let second = key(Proposer::Party(8));
    let (mut d, _, out) = opened(&[first, second]);
    for k in [first, second] {
        receive(&mut d, &out, proposal(k));
    }
    assert!(d.conversation().names(CLIENT.objects).is_empty());
    let tree = build(&d);
    assert_eq!(cards(&tree, first), 1);
    assert_eq!(cards(&tree, second), 1);
    assert!(contains(&tree, b"whole shared reason"));
    assert!(contains(&tree, b"Accept"));
    assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::Slot).count(), 2);
    let mut huge = CLIENT;
    huge.objects = u32::MAX;
    assert!(crate::task_nodes(&huge, 0).is_none());
    assert!(crate::task_nodes(&CLIENT, u32::MAX).is_none());
}
fn turn(objects: [ObjectKey; 1]) -> Turn {
    Turn {
        messages: list([Message {
            speaker: Speaker::Assistant,
            blocks: list([Block::Call(Call {
                number: 9,
                tool: text(b"Propose"),
                input: text(b"opaque input"),
                outcome: None,
                failed: false,
                objects: list(objects),
                parts: list([]),
            })]),
        }]),
        used: Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
        spent: 0,
        read_through: None,
        cut: false,
    }
}
#[test]
fn task_proposals_share_exact_cards_with_committed_and_live_calls_without_conflating_makers() {
    for live in [false, true] {
        let first = key(Proposer::Task(8));
        let second = key(Proposer::Party(8));
        let (mut d, watch, out) = opened(&[first, second]);
        for k in [first, second] {
            receive(&mut d, &out, proposal(k));
        }
        let mut doc = task(&[first, second], 12);
        doc.context = super::cards_objects::task_doc(&[]).context;
        doc.context.as_mut().unwrap().runtime = Runtime::Running { attempt: 2, since: Wall::EPOCH, tool: None };
        deliver(&mut d, watch, Delivery::Task(Box::new(doc)));
        if live {
            let run = out
                .iter()
                .find_map(|r| match r {
                    Request::Open { watch, subject: Subject::Run { .. } } => Some(*watch),
                    _ => None,
                })
                .unwrap();
            let value = turn([ObjectKey::Proposal(first)]);
            let Block::Call(call) = value.messages.get(0).unwrap().blocks.get(0).unwrap() else {
                panic!("supplied call");
            };
            deliver(
                &mut d,
                run,
                Delivery::Run(Box::new(RunText {
                    attempt: 2,
                    began: None,
                    turn: ProvisionalTurnName { retry: 0, attempt: 2, number: 1 },
                    from: 0,
                    through: 0,
                    text: text(b""),
                    cut: false,
                    call: Some(LiveCall::Decoded(call.clone())),
                })),
            );
        } else {
            let read = out
                .iter()
                .find_map(|r| match r {
                    Request::Read { read, what: Read::Activity { .. } } => Some(*read),
                    _ => None,
                })
                .unwrap();
            drive(
                &mut d,
                Event::Read {
                    read,
                    result: ReadResult::Value(ReadValue::Activity(Box::new(ActivityPage {
                        cut: false,
                        rows: list([ActivityLine {
                            position: 11,
                            at: Wall::EPOCH,
                            event: Activity::Turn {
                                name: TurnName { attempt: 2, number: 1 },
                                turn: turn([ObjectKey::Proposal(first)]),
                            },
                        }]),
                        before: None,
                        after: None,
                    }))),
                },
            );
        }
        let tree = build(&d);
        assert_eq!(cards(&tree, first), 1);
        assert_eq!(cards(&tree, second), 1);
    }
}
#[test]
fn a_departing_task_proposal_shows_its_supplied_terminal_decision_until_page_departure() {
    let k = key(Proposer::Task(8));
    let (mut d, watch, out) = opened(&[k]);
    receive(&mut d, &out, proposal(k));
    let out = deliver(&mut d, watch, Delivery::Task(Box::new(task(&[], 12))));
    let mut ended = proposal(k);
    ended.ending = Some(Ending::Decided { by: 91, party: None, how: Some(Decision::Accepted), at: Wall::EPOCH });
    receive(&mut d, &out, ended);
    let tree = build(&d);
    assert_eq!(cards(&tree, k), 1);
    assert!(contains(&tree, b"Accepted"));
    assert!(contains(&tree, b"91"));
    assert!(!tree.nodes().iter().any(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Accept")));
    drive(&mut d, Event::Went { place: Place::Elsewhere });
    assert!(d.task_page().proposals.is_empty());
}

#[test]
fn a_task_activity_renders_the_native_whole_pending_proposal_with_its_supplied_funding() {
    let k = key(Proposer::Task(8));
    let (mut d, _, out) = opened(&[k]);
    let native = OwnerProposal {
        key: k,
        project: 4,
        reason: DocumentText::complete(text(b"native whole shared reason")),
        action: OwnerProposalAction::Batch(list([])),
        as_holder: true,
        waiting: OwnerProposalWaiting { holder: OwnerHistoryHolder::Person(1), since: Wall::EPOCH },
        funding: list([OwnerFundingOffer {
            source: Funder::Pool { project: 4, person: 1, period: 3 },
            needed: 150,
            left: Some(250),
            after: Some(100),
        }]),
        acts: list([Offered { act: ActKind::Accept, admission: Admission::Allowed }]),
    };
    drive(
        &mut d,
        Event::Read { read: token(&out, k), result: ReadResult::Value(ReadValue::OwnerProposal(Box::new(native))) },
    );
    let tree = build(&d);
    assert_eq!(cards(&tree, k), 1);
    assert!(contains(&tree, b"native whole shared reason"));
    assert!(contains(&tree, b"150"));
    assert!(contains(&tree, b"250"));
    assert!(contains(&tree, b"100"));
    assert!(contains(&tree, b"Accept"));
}
