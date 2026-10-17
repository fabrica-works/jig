#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary tests inspect checked sayings and public tree events"
)]
use super::cards::{CLIENT, contains, document, drive, start, words};
use crate::tests::fixtures::{AnsweredByEngine, Fixture, Watched};
use crate::words::Words;
use crate::{JigBindings, owner_tree as view};
use skein_ui::phrases::{PhraseLimits, Phrasebook, Segment};
use skein_ui::{Builder, Role, Tree};

use crate::{
    Event, Request,
    action::Act as ClientAct,
    address::{Address, Place, TaskSection},
};
use jig_client::contract::*;
use skein_lib::{List, Wall, bytes::copy_of};
fn chip(n: u64) -> Chip {
    let mut c = super::cards_objects::task_doc(&[]).chip;
    c.task = n;
    c.title.cut = true;
    c
}
fn counts() -> OwnerPhaseCounts {
    OwnerPhaseCounts {
        waiting: 1,
        idle: 2,
        starting: 3,
        running: 4,
        backing_off: 5,
        closing: 6,
        held: 7,
        done: 8,
        failed: 9,
        cancelled: u32::MAX,
    }
}
fn source() -> OwnerTree {
    let mut rows = List::with_capacity(2);
    let mut proposals = List::with_capacity(2);
    proposals.push(ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX }).unwrap();
    let mut dependencies = List::with_capacity(2);
    dependencies.push(OwnerTaskLink::Stored { number: u64::MAX, status: None }).unwrap();
    rows.push(OwnerTreeRow {
        chip: chip(9),
        parent: 8,
        depth: u32::MAX,
        proposals,
        spent: u64::MAX,
        dependencies,
        cut: true,
    })
    .unwrap();
    let mut folded = List::with_capacity(2);
    folded.push(OwnerTreeFold { parent: 0, phases: counts(), held_below: 11 }).unwrap();
    folded
        .push(OwnerTreeFold {
            parent: u64::MAX,
            phases: OwnerPhaseCounts { idle: u32::MAX, ..counts() },
            held_below: 17,
        })
        .unwrap();
    OwnerTree {
        root: chip(8),
        rows,
        folded,
        held_below: 19,
        next: Some(Cursor::new(copy_of(UTF8_TEXTS[0]), 128).unwrap()),
    }
}
fn domain(value: &OwnerTree) -> Fixture {
    domain_and_watch(value).0
}
fn domain_and_watch(value: &OwnerTree) -> (Fixture, jig_client::Handle) {
    let (mut d, frame) = start();
    drive(&mut d, Watched { watch: frame, event: WatchEvent::Delivered(Delivery::Me(Box::new(document()))) });
    let asks = drive(&mut d, Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Plan }) });
    let task =
        asks.iter()
            .find_map(|r| match r {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Task { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    let asks = drive(
        &mut d,
        Watched {
            watch: task,
            event: WatchEvent::Delivered(Delivery::Task(Box::new(super::cards_objects::task_doc(&[])))),
        },
    );
    let watch =
        asks.iter()
            .find_map(|r| match r {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Tree { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    drive(&mut d, Watched { watch, event: WatchEvent::Delivered(Delivery::OwnerTree(Box::new(value.clone()))) });
    (d, watch)
}
fn build(value: &OwnerTree, d: &Fixture, w: &Words, nodes: Option<u32>) -> Tree {
    let f = view::required(value, &CLIENT, w).unwrap();
    let mut l = crate::limits::of_words(&CLIENT, w).unwrap();
    l.nodes = nodes.unwrap_or(f.nodes);
    l.depth = f.depth;
    l.text_bytes = f.text_bytes;
    l.text_per_node = f.text_per_node;
    let mut builder = Builder::new(&l);
    let mut bindings = JigBindings::new(f.bindings, CLIENT.text);
    view::render(&mut builder, value, &d.task_page().plan, &d.view(), w, &mut bindings);
    builder.finish()
}
#[test]
fn owner_tree_view_names_root_parent_depth_spend_loss_proposal_and_all_ten_counts_per_parent() {
    let value = source();
    let d = domain(&value);
    let w = words();
    let tree = build(&value, &d, w, None);
    for expected in [
        b"Root task: 8".as_slice(),
        b"Parent task: 8",
        b"Depth from root: 4294967295",
        b"Subtree row spend:",
        b"Some dependencies are outside this row",
        b"Waiting: 1",
        b"Idle: 2",
        b"Starting: 3",
        b"Running: 4",
        b"Backing off: 5",
        b"Closing: 6",
        b"Held: 7",
        b"Done: 8",
        b"Failed: 9",
        b"Cancelled: 4294967295",
        b"Held below: 11",
        b"Held below: 17",
        b"Held below: 19",
        b"More descendants remain",
        b"18446744073709551615",
    ] {
        assert!(contains(&tree, expected), "missing {}", core::str::from_utf8(expected).unwrap());
    }
    assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::TreeItem).count(), 2);
    assert_eq!(d.task_page().plan.owner, Some(&value));
}
#[test]
fn owner_tree_view_preserves_all_link_metadata_without_fabricated_stored_chip_or_time() {
    let mut value = source();
    let w = words();
    for (link, expected) in [
        (OwnerTaskLink::Live(chip(17)), b"Live task".as_slice()),
        (OwnerTaskLink::Retained { chip: chip(17), at: None }, b"Ended time unavailable"),
        (OwnerTaskLink::Retained { chip: chip(17), at: Some(Wall::EPOCH) }, b"Retained ended task"),
        (OwnerTaskLink::Stored { number: u64::MAX, status: None }, b"Stored status unavailable"),
        (OwnerTaskLink::Stored { number: u64::MAX, status: Some(OwnerTaskStatus::Done) }, b"Done"),
        (OwnerTaskLink::Stored { number: u64::MAX, status: Some(OwnerTaskStatus::Failed) }, b"Failed"),
        (OwnerTaskLink::Stored { number: u64::MAX, status: Some(OwnerTaskStatus::Cancelled) }, b"Cancelled"),
    ] {
        value.rows.get_mut(0).unwrap().dependencies.clear();
        value.rows.get_mut(0).unwrap().dependencies.push(link).unwrap();
        let d = domain(&value);
        let tree = build(&value, &d, w, None);
        assert!(contains(&tree, expected), "missing {}", core::str::from_utf8(expected).unwrap());
        assert_eq!(d.task_page().plan.owner, Some(&value));
    }
}
#[test]
fn owner_tree_view_uses_checked_selected_language_and_finite_node_text_scratch_envelopes() {
    let mut value = source();
    let first = value.rows.get_mut(0).unwrap();
    first.proposals.push(ProposalKey { proposer: Proposer::Task(u64::MAX), number: u64::MAX }).unwrap();
    first.dependencies.push(OwnerTaskLink::Retained { chip: chip(17), at: Some(Wall::EPOCH) }).unwrap();
    let mut second = first.clone();
    second.chip.task = 10;
    second.parent = 9;
    second.depth = 2;
    value.rows.push(second).unwrap();
    let mut metadata = super::owner_chip::details();
    metadata.closing_connectors = Some(List::with_capacity(CLIENT.rows));
    *value.root.details = ChipPhase::Owner {
        phase: OwnerChipPhase::Active(RunStage::Running { attempt: u64::MAX }),
        details: Box::new(metadata),
    };
    value.root.phase = Phase::Running;
    let d = domain(&value);
    let english = crate::phrases::english::language();
    let mut selected = english.clone();
    for phrase in &mut selected.phrases {
        if (8300..8340).contains(&phrase.id.number) {
            for form in &mut phrase.forms {
                for segment in &mut form.segments {
                    if let Segment::Text(bytes) = segment {
                        *bytes = copy_of(b"selected tree ");
                    }
                }
            }
        }
    }
    let w = Words {
        book: Phrasebook::load(
            crate::phrases::jig::shapes(256),
            Box::new([english, selected]),
            &PhraseLimits {
                languages: 2,
                phrases: crate::owner_words::phrase_count(),
                holes: 4,
                segments: 5000,
                text_bytes: 100_000,
                hole_bytes: 256,
            },
        )
        .unwrap(),
        language: 1,
        offset_minutes: 0,
    };
    let tree = build(&value, &d, &w, None);
    assert!(contains(&tree, b"selected tree "));
    assert!(!contains(&tree, b"Plan subtree"));
    let f = view::required(&value, &CLIENT, &w).unwrap();
    assert!(tree.len() <= f.nodes);
    assert_eq!(f.scratch, view::scratch(CLIENT.rows).unwrap());
    assert!(crate::task_nodes(&CLIENT, 0).unwrap() > f.nodes);
    let exact = tree.len();
    assert_eq!(build(&value, &d, &w, Some(exact)).len(), exact);
    let fail = std::panic::catch_unwind(|| build(&value, &d, &w, Some(exact.checked_sub(1).unwrap())));
    let _panic = fail.unwrap_err();
    let mut bytes = 0_u32;
    for node in tree.nodes() {
        assert!(node.value.is_none() && node.external.is_none() && node.drawing.is_none());
        if let Some(name) = &node.name {
            bytes += u32::try_from(name.len()).unwrap();
        }
        for run in &node.text {
            bytes += u32::try_from(run.text.len()).unwrap();
        }
    }
    let mut cap = crate::limits::of_words(&CLIENT, &w).unwrap();
    cap.nodes = exact;
    cap.depth = f.depth;
    cap.text_bytes = bytes;
    cap.text_per_node = f.text_per_node;
    let render = |cap: skein_ui::Limits| {
        let mut b = Builder::new(&cap);
        let mut bindings = JigBindings::new(f.bindings, CLIENT.text);
        view::render(&mut b, &value, &d.task_page().plan, &d.view(), &w, &mut bindings);
        b.finish()
    };
    assert_eq!(render(cap).len(), exact);
    cap.text_bytes = bytes.checked_sub(1).unwrap();
    let _panic = std::panic::catch_unwind(|| render(cap)).unwrap_err();
    let mut invalid = value.clone();
    invalid.rows = List::with_capacity(CLIENT.rows + 1);
    assert!(view::required(&invalid, &CLIENT, &w).is_none());
}
#[test]
fn owner_tree_local_filters_folds_cycles_and_empty_loss_continuations_remain_bounded_and_independent() {
    let mut value = source();
    let w = words();
    value.rows.get_mut(0).unwrap().parent = 9;
    let d = domain(&value);
    let tree = build(&value, &d, w, None);
    assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::TreeItem).count(), 2);
    for next in [None, Some(Cursor::new(copy_of(b""), 128).unwrap())] {
        value.rows = List::with_capacity(2);
        value.folded = List::with_capacity(2);
        value.next = next.clone();
        let d = domain(&value);
        let tree = build(&value, &d, w, None);
        assert!(contains(&tree, if next.is_some() { b"More descendants remain" } else { b"no continuation" }));
        assert!(!contains(&tree, b"Some dependencies are outside this row"));
        assert!(contains(&tree, b"Held below: 19"));
        assert_eq!(d.task_page().plan.owner, Some(&value));
    }
}

#[test]
fn owner_tree_departure_keeps_a_separately_named_deciding_card_until_its_answer_once() {
    let mut value = source();
    let key = value.rows.get(0).unwrap().proposals.get(0).copied().unwrap();
    let (mut d, tree_watch) = domain_and_watch(&value);
    let mut objects = List::with_capacity(1);
    objects.push(ObjectKey::Proposal(key)).unwrap();
    let asks = drive(&mut d, Event::Named { objects, sections: List::with_capacity(0) });
    let proposal_watch =
        asks.iter()
            .find_map(|ask| match ask {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Inbox { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    let mut acts = List::with_capacity(1);
    acts.push(Offered { act: ActKind::Accept, admission: Admission::Allowed }).unwrap();
    let proposal = OwnerProposal {
        key,
        project: 4,
        reason: DocumentText::complete(super::cards::text(b"supplied pending reason")),
        action: OwnerProposalAction::Release { task: OwnerTaskLink::Live(chip(9)), hold: None },
        as_holder: true,
        waiting: OwnerProposalWaiting { holder: OwnerHistoryHolder::Task(9), since: Wall::EPOCH },
        funding: List::with_capacity(0),
        acts,
    };
    drive(
        &mut d,
        Watched { watch: proposal_watch, event: WatchEvent::Delivered(Delivery::OwnerProposal(Box::new(proposal))) },
    );
    let id = d.objects().find(ObjectKey::Proposal(key)).unwrap();
    drive(&mut d, Event::Act { action: ClientAct::Intend { object: id.token(), act: ActKind::Accept } });
    let sent = drive(&mut d, Event::Act { action: ClientAct::Confirm });
    let request = sent
        .iter()
        .find_map(|ask| match ask {
            Request::Client { asked: jig_client::Event::Send { key: request, .. } } => Some(*request),
            _ => None,
        })
        .unwrap();
    let card_key = crate::pages::object_key(ObjectKey::Proposal(key));
    let before = deciding_tree_page(&d);
    assert_eq!(before.nodes().iter().filter(|node| node.key == Some(card_key)).count(), 1);
    let row = d.objects().find(ObjectKey::Task(9)).unwrap();
    drive(&mut d, Event::Act { action: ClientAct::Fold { object: row.token(), open: false } });
    assert_eq!(
        deciding_tree_page(&d).nodes().iter().filter(|node| node.key == Some(card_key)).count(),
        0,
        "person fold stays hidden"
    );
    drive(&mut d, Event::Act { action: ClientAct::Fold { object: row.token(), open: true } });
    value.rows.get_mut(0).unwrap().proposals.clear();
    drive(
        &mut d,
        Watched { watch: tree_watch, event: WatchEvent::Delivered(Delivery::OwnerTree(Box::new(value.clone()))) },
    );
    assert!(!d.objects().get(id).unwrap().sources.is_empty());
    assert!(d.task_page().plan.owner.as_ref().unwrap().rows.get(0).unwrap().proposals.is_empty());
    let pending = deciding_tree_page(&d);
    assert_eq!(pending.nodes().iter().filter(|node| node.key == Some(card_key)).count(), 1);
    assert!(contains(&pending, b"Being decided"));
    assert!(contains(&pending, b"supplied pending reason"));
    drive(
        &mut d,
        AnsweredByEngine {
            request,
            answer: Answer::Done(Outcome::Owner(OwnerOutcome::ProposalDecided {
                proposal: key,
                by: 1,
                at: Wall::EPOCH,
                choice: OwnerProposalChoice::Accepted,
            })),
        },
    );
    let answered = deciding_tree_page(&d);
    assert_eq!(answered.nodes().iter().filter(|node| node.key == Some(card_key)).count(), 1);
    assert!(contains(&answered, b"Accepted"));
    assert!(contains(&answered, b"supplied pending reason"));
}

const UTF8_TEXTS: [&[u8]; 1] = [b"~\0/"];

fn deciding_tree_page(domain: &Fixture) -> Tree {
    let w = words();
    let mut cap = crate::limits::of_words(&CLIENT, w).unwrap();
    cap.nodes = cap.nodes.max(4096);
    cap.text_bytes = cap.nodes.checked_mul(cap.text_per_node).unwrap();
    let mut b = Builder::new(&cap);
    let mut bindings = JigBindings::new(cap.bindings, CLIENT.text);
    b.open(Role::Page);
    view::render(
        &mut b,
        domain.task_page().plan.owner.as_ref().unwrap(),
        &domain.task_page().plan,
        &domain.view(),
        w,
        &mut bindings,
    );
    crate::pages::after_page(&mut b, &domain.view(), w, &mut bindings);
    b.close();
    b.finish()
}

#[test]
fn a_tree_keeps_a_waiting_card_held_by_another_source_after_its_row_leaves() {
    let mut value = source();
    let key = ObjectKey::Proposal(*value.rows.get(0).unwrap().proposals.get(0).unwrap());
    let (mut domain, watch) = domain_and_watch(&value);
    domain.ui.objects.name(key, crate::objects::Source::Window { number: 23 }).unwrap();
    let card = crate::pages::object_key(key);
    assert_eq!(deciding_tree_page(&domain).nodes().iter().filter(|node| node.key == Some(card)).count(), 1);
    value.rows.get_mut(0).unwrap().proposals.clear();
    drive(&mut domain, Watched { watch, event: WatchEvent::Delivered(Delivery::OwnerTree(Box::new(value))) });
    assert_eq!(deciding_tree_page(&domain).nodes().iter().filter(|node| node.key == Some(card)).count(), 1);
}

#[test]
fn a_waiting_tree_card_is_still_owed_when_the_task_stops_offering_its_plan() {
    let value = source();
    let key = ObjectKey::Proposal(*value.rows.get(0).unwrap().proposals.get(0).unwrap());
    let (mut domain, _) = domain_and_watch(&value);
    let task = domain
        .ui
        .handles
        .watches
        .iter()
        .find_map(|(handle, held)| (held.subject == Subject::Task { task: 8 }).then_some(*handle))
        .unwrap();
    let mut finished = super::cards_objects::task_doc(&[]);
    finished.delegates.clear();
    finished.context = None;
    drive(&mut domain, Watched { watch: task, event: WatchEvent::Delivered(Delivery::Task(Box::new(finished))) });
    assert!(!domain.shows(crate::page::Section::Task { task: 8, section: TaskSection::Plan }));
    let id = domain.objects().find(key).unwrap();
    assert!(domain.owes(id));
}

#[test]
fn a_task_end_keeps_its_other_documents_visible_while_the_final_header_is_read() {
    let value = source();
    let key = ObjectKey::Proposal(*value.rows.get(0).unwrap().proposals.get(0).unwrap());
    let (mut domain, _) = domain_and_watch(&value);
    let task = domain
        .ui
        .handles
        .watches
        .iter()
        .find_map(|(handle, held)| (held.subject == Subject::Task { task: 8 }).then_some(*handle))
        .unwrap();
    drive(&mut domain, crate::tests::fixtures::Ended { watch: task, end: WatchEnd::SubjectEnded });
    let w = words();
    let cap = super::cards::limits();
    let mut b = Builder::new(&cap);
    let mut bindings = JigBindings::new(cap.bindings, CLIENT.text);
    b.open(Role::Page);
    crate::pages::main(&mut b, &domain.view(), w, &mut bindings);
    crate::pages::after_page(&mut b, &domain.view(), w, &mut bindings);
    b.close();
    let tree = b.finish();
    assert_eq!(tree.nodes().iter().filter(|node| node.key == Some(crate::pages::object_key(key))).count(), 1);
}
