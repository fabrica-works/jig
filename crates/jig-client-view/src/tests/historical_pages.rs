#![expect(
    clippy::disallowed_methods,
    reason = "ordinary proofs inspect complete public trees and checked language data"
)]
use super::cards::CLIENT;
use crate::historical_pages_words::{self as sayings, HistoricalPhrase as P};
use crate::words::Words;
use crate::{JigBindings, historical_pages as view};
use skein_ui::Limits;
use skein_ui::phrases::{Hole, Invalid, PhraseLimits, Phrasebook, Segment};
use skein_ui::{Builder, Tree};

use crate::Domain;
use jig_client::contract::*;
use skein_lib::{List, Wall, bytes::copy_of};
fn doc(bytes: &[u8], cut: bool) -> DocumentText {
    DocumentText { bytes: Text::new(copy_of(bytes), CLIENT.text).unwrap(), cut }
}
fn words() -> Words {
    let mut language = crate::phrases::english::language();
    language.wall = skein_ui::phrases::WallFormat::Nanos;
    Words {
        book: Phrasebook::load(crate::phrases::jig::shapes(CLIENT.text), Box::new([language]), &phrase_limits())
            .unwrap(),
        language: 0,
        offset_minutes: 0,
    }
}
fn phrase_limits() -> PhraseLimits {
    PhraseLimits {
        languages: 2,
        phrases: crate::owner_words::phrase_count(),
        holes: 4,
        segments: 5000,
        text_bytes: 100_000,
        hole_bytes: 256,
    }
}
fn chip() -> Chip {
    let mut chip = super::cards_objects::task_doc(&[]).chip;
    chip.task = u64::MAX;
    chip.title = doc(b"", true);
    chip.phase = Phase::Ended;
    let mut metadata = super::owner_chip::details();
    metadata.resources = List::with_capacity(CLIENT.rows);
    metadata.closing_connectors = Some(List::with_capacity(CLIENT.rows));
    *chip.details = ChipPhase::Owner { phase: OwnerChipPhase::Ended, details: Box::new(metadata) };
    chip
}
fn ended(value: OwnerChipEnding, at: Option<Wall>) -> OwnerEndedPage {
    let mut rows = List::with_capacity(CLIENT.rows);
    rows.push(OwnerEndedDelegate { chip: chip(), ended: value, at }).unwrap();
    OwnerEndedPage { rows, before: None, after: None, cut: true }
}
fn funding(funder: Funder, parent: Option<Funder>, period: Option<OwnerPeriodTimes>, closed: bool) -> OwnerSpendPage {
    let mut rows = List::with_capacity(CLIENT.rows);
    rows.push(OwnerFundingRecord {
        period,
        funder,
        parent,
        numbers: OwnerNumbers { budget: 1, spent: 7, spent_below: 11, reserved: u64::MAX },
        made: u32::MAX,
        closed,
    })
    .unwrap();
    OwnerSpendPage { rows, before: None, after: None, cut: true }
}
fn limits(f: view::HistoricalFragment) -> Limits {
    Limits {
        nodes: f.nodes,
        depth: f.depth,
        text_per_node: f.text_per_node,
        text_bytes: f.text_bytes,
        drawing_bytes: 0,
        styles_per_node: 2,
        patches: 0,
        bindings: f.bindings,
        slots: f.slots,
        markdown_depth: 2,
        scene_marks: 0,
        scene_bytes: 0,
    }
}
fn ended_tree(page: &OwnerEndedPage, w: &Words, cap: Option<Limits>) -> Tree {
    let f = view::ended_required(page, &CLIENT, w).unwrap();
    let cap = cap.unwrap_or(limits(f));
    let mut b = Builder::new(&cap);
    let mut bindings = JigBindings::new(cap.bindings, CLIENT.text);
    view::ended_render(&mut b, &Domain::new(&CLIENT, 1), page, w, &mut bindings);
    b.finish()
}
fn spend_tree(page: &OwnerSpendPage, w: &Words, cap: Option<Limits>) -> Tree {
    let f = view::spend_required(page, &CLIENT, w).unwrap();
    let mut b = Builder::new(&cap.unwrap_or(limits(f)));
    view::spend_render(&mut b, page, w);
    b.finish()
}
fn has(tree: &Tree, bytes: &[u8]) -> bool {
    tree.nodes().iter().any(|n| {
        n.name.as_deref() == Some(bytes) || n.text.iter().any(|r| r.text.windows(bytes.len()).any(|part| part == bytes))
    })
}
fn caption(tree: &Tree, w: &Words, p: P, holes: &[Hole]) {
    assert!(has(tree, &w.book.say(w.language, p.id(), holes).unwrap().text));
}
#[test]
fn historical_ended_view_keeps_all_owner_causes_requesters_optional_times_loss_and_empty_title_identity() {
    let w = words();
    for by in [
        OwnerRequester::Task(u64::MAX),
        OwnerRequester::Person(u64::MAX),
        OwnerRequester::Deployment { project: u32::MAX },
    ] {
        for at in [None, Some(Wall::EPOCH), Some(Wall::from_nanos(u64::MAX))] {
            for ending in [
                OwnerChipEnding::Done(ResultValue::Words(doc(b"actual result", true))),
                OwnerChipEnding::Failed(doc(b"actual failure", true)),
                OwnerChipEnding::Cancelled {
                    reason: doc(b"actual cancellation", true),
                    result: Some(ResultValue::Words(doc(b"actual result", true))),
                    by,
                    at: Wall::from_nanos(u64::MAX),
                },
            ] {
                let page = ended(ending.clone(), at);
                let untouched = page.clone();
                let tree = ended_tree(&page, &w, None);
                super::cards_access::check(&tree);
                caption(&tree, &w, P::EndedRow, &[Hole::Number(u64::MAX)]);
                match at {
                    Some(at) => caption(&tree, &w, P::At, &[Hole::Wall(at)]),
                    None => caption(&tree, &w, P::AtAbsent, &[]),
                }
                match ending {
                    OwnerChipEnding::Done(_) => assert!(has(&tree, b"actual result")),
                    OwnerChipEnding::Failed(_) => assert!(has(&tree, b"actual failure")),
                    OwnerChipEnding::Cancelled { .. } => {
                        assert!(has(&tree, b"actual cancellation"));
                        assert!(has(&tree, b"actual result"));
                        let requester = match by {
                            OwnerRequester::Task(_) => b"Cancellation requester task: 18446744073709551615".as_slice(),
                            OwnerRequester::Person(_) => {
                                b"Cancellation requester person: 18446744073709551615".as_slice()
                            }
                            OwnerRequester::Deployment { .. } => {
                                b"Cancellation requester deployment project: 4294967295".as_slice()
                            }
                        };
                        assert!(has(&tree, requester));
                        assert!(has(&tree, b"Cancelled at: 18446744073709551615"));
                    }
                }
                assert!(tree.nodes().iter().all(|n| n.key.is_none()));
                assert_eq!(page, untouched);
                assert_eq!(page.rows.capacity(), CLIENT.rows);
            }
        }
    }
    let without = ended(
        OwnerChipEnding::Cancelled {
            reason: doc(b"reason without result", false),
            result: None,
            by: OwnerRequester::Person(9),
            at: Wall::EPOCH,
        },
        None,
    );
    let tree = ended_tree(&without, &w, None);
    assert!(has(&tree, b"reason without result"));
    assert!(!has(&tree, b"actual result"));
}
#[test]
fn historical_funding_view_preserves_all_source_parent_coordinates_four_counters_period_absence_and_closed_state() {
    let w = words();
    let kinds = [
        Funder::Task(u64::MAX),
        Funder::Pool { project: u32::MAX, person: u64::MAX, period: u64::MAX },
        Funder::Recurring { project: u32::MAX, task: u64::MAX, period: u64::MAX },
        Funder::Period { project: u32::MAX, period: u64::MAX },
    ];
    for funder in kinds {
        for parent in [None, Some(kinds[0]), Some(kinds[1]), Some(kinds[2]), Some(kinds[3])] {
            for period in [
                None,
                Some(OwnerPeriodTimes { began: Wall::EPOCH, next: None }),
                Some(OwnerPeriodTimes { began: Wall::EPOCH, next: Some(Wall::EPOCH) }),
                Some(OwnerPeriodTimes { began: Wall::from_nanos(u64::MAX), next: Some(Wall::from_nanos(u64::MAX)) }),
            ] {
                for closed in [false, true] {
                    let page = funding(funder, parent, period, closed);
                    let original = page.clone();
                    let tree = spend_tree(&page, &w, None);
                    super::cards_access::check(&tree);
                    for (p, n) in [(P::Budget, 1), (P::Spent, 7), (P::SpentBelow, 11), (P::Reserved, u64::MAX)] {
                        caption(&tree, &w, p, &[Hole::Amount(n)]);
                    }
                    caption(&tree, &w, P::Made, &[Hole::Number(u64::from(u32::MAX))]);
                    caption(&tree, &w, if closed { P::Closed } else { P::Open }, &[]);
                    match period {
                        None => {
                            caption(&tree, &w, P::PeriodAbsent, &[]);
                            assert!(!has(&tree, b"Period began:"));
                        }
                        Some(period) => {
                            caption(&tree, &w, P::Began, &[Hole::Wall(period.began)]);
                            match period.next {
                                None => caption(&tree, &w, P::NextAbsent, &[]),
                                Some(next) => caption(&tree, &w, P::Next, &[Hole::Wall(next)]),
                            }
                        }
                    }
                    if parent.is_none() {
                        caption(&tree, &w, P::ParentAbsent, &[]);
                    }
                    assert!(!has(&tree, b"Left:"));
                    assert!(!has(&tree, b"Pools"));
                    assert!(tree.nodes().iter().all(|n| n.key.is_none()));
                    assert_eq!(page, original);
                }
            }
        }
    }
}
#[test]
fn historical_checked_selected_language_has_no_fallback_and_rejects_a_missing_number_counter_hole() {
    let english = crate::phrases::english::language();
    let mut selected = english.clone();
    for phrase in &mut selected.phrases {
        if (9500..9600).contains(&phrase.id.number) {
            for form in &mut phrase.forms {
                for segment in &mut form.segments {
                    if let Segment::Text(bytes) = segment {
                        *bytes = copy_of(b"selected history ");
                    }
                }
            }
        }
    }
    let w = Words {
        book: Phrasebook::load(
            crate::phrases::jig::shapes(CLIENT.text),
            Box::new([english, selected]),
            &phrase_limits(),
        )
        .unwrap(),
        language: 1,
        offset_minutes: 0,
    };
    let page = ended(OwnerChipEnding::Failed(doc(b"source failure", true)), None);
    let tree = ended_tree(&page, &w, None);
    assert!(has(&tree, b"selected history "));
    assert!(!has(&tree, b"Ended delegates"));
    assert!(!has(&tree, b"Ending time not supplied"));
    let page = funding(Funder::Period { project: 1, period: 9 }, None, None, false);
    let tree = spend_tree(&page, &w, None);
    assert!(has(&tree, b"selected history "));
    assert!(!has(&tree, b"Period times not supplied"));
    let mut broken = sayings::language();
    for phrase in &mut broken.phrases {
        if phrase.id == P::Made.id() {
            phrase.forms[0].segments = Box::new([Segment::Text(copy_of(b"missing number"))]);
        }
    }
    assert_eq!(
        Phrasebook::load(sayings::shapes(), Box::new([broken]), &phrase_limits()).err(),
        Some(Invalid::Holes { id: P::Made.id() })
    );
}
fn exact_limits(tree: &Tree, maximum: Limits) -> Limits {
    let depth = tree
        .nodes()
        .iter()
        .enumerate()
        .map(|(at, _)| {
            1usize
                .checked_add(
                    tree.nodes()
                        .iter()
                        .enumerate()
                        .take(at)
                        .filter(|(index, node)| index.checked_add(usize::try_from(node.size).unwrap()).unwrap() > at)
                        .count(),
                )
                .unwrap()
        })
        .max()
        .unwrap();
    let bindings = tree.nodes().iter().filter(|n| n.binding.is_some() || n.link.is_some() || n.slot.is_some()).count();
    Limits {
        nodes: tree.len(),
        depth: u32::try_from(depth).unwrap(),
        text_per_node: tree.nodes().iter().map(crate::tests::text_bytes).max().unwrap(),
        text_bytes: tree.nodes().iter().map(crate::tests::text_bytes).sum(),
        bindings: u32::try_from(bindings).unwrap(),
        slots: u32::try_from(tree.nodes().iter().filter(|n| n.slot.is_some()).count()).unwrap(),
        ..maximum
    }
}
#[test]
fn historical_ended_view_exact_node_depth_text_binding_and_slot_storage_refuses_each_one_under_whole() {
    let w = words();
    let page = ended(
        OwnerChipEnding::Done(ResultValue::Connector {
            summary: doc(b"whole connector result", true),
            part: Part::Result { connector: u16::MAX, task: u64::MAX },
        }),
        Some(Wall::EPOCH),
    );
    let original = page.clone();
    let f = view::ended_required(&page, &CLIENT, &w).unwrap();
    let tree = ended_tree(&page, &w, None);
    let exact = exact_limits(&tree, limits(f));
    assert!(exact.bindings > 0 && exact.slots > 0);
    assert_eq!(ended_tree(&page, &w, Some(exact)).len(), tree.len());
    for below in [
        Limits { nodes: exact.nodes - 1, ..exact },
        Limits { depth: exact.depth - 1, ..exact },
        Limits { text_per_node: exact.text_per_node - 1, ..exact },
        Limits { text_bytes: exact.text_bytes - 1, ..exact },
        Limits { bindings: exact.bindings - 1, ..exact },
        Limits { slots: exact.slots - 1, ..exact },
    ] {
        std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| ended_tree(&page, &w, Some(below))))
            .expect_err("each one-under whole view storage refuses");
    }
    assert_eq!(page, original);
    let mut empty = page.clone();
    empty.rows = List::with_capacity(CLIENT.rows + 1);
    assert!(view::ended_required(&empty, &CLIENT, &w).is_none());
    let mut text_over = page.clone();
    text_over.after = Some(Cursor::new(copy_of(&[0; 129]), 129).unwrap());
    assert!(view::ended_required(&text_over, &CLIENT, &w).is_none());
    assert!(view::ended_nodes(&crate::Limits { rows: u32::MAX, ..CLIENT }).is_none());
}
#[test]
fn historical_funding_view_exact_geometry_and_reserved_empty_rows_remain_independent_of_scalar_values() {
    let w = words();
    let page = funding(
        Funder::Period { project: u32::MAX, period: u64::MAX },
        Some(Funder::Task(u64::MAX)),
        Some(OwnerPeriodTimes { began: Wall::EPOCH, next: Some(Wall::from_nanos(u64::MAX)) }),
        true,
    );
    let f = view::spend_required(&page, &CLIENT, &w).unwrap();
    let tree = spend_tree(&page, &w, None);
    let exact = exact_limits(&tree, limits(f));
    assert_eq!(exact.bindings, 0);
    assert_eq!(exact.slots, 0);
    assert_eq!(spend_tree(&page, &w, Some(exact)).len(), tree.len());
    for below in [
        Limits { nodes: exact.nodes - 1, ..exact },
        Limits { depth: exact.depth - 1, ..exact },
        Limits { text_per_node: exact.text_per_node - 1, ..exact },
        Limits { text_bytes: exact.text_bytes - 1, ..exact },
    ] {
        std::panic::catch_unwind(|| spend_tree(&page, &w, Some(below)))
            .expect_err("one-under historical scalar tree refuses");
    }
    let mut reserved = page.clone();
    reserved.rows = List::with_capacity(CLIENT.rows + 1);
    assert!(view::spend_required(&reserved, &CLIENT, &w).is_none());
    assert_eq!(reserved.rows.len(), 0);
    reserved.rows = List::with_capacity(CLIENT.rows);
    assert!(view::spend_required(&reserved, &CLIENT, &w).is_some());
    let tree = spend_tree(&reserved, &w, None);
    assert_eq!(tree.len(), 1);
    assert!(!has(&tree, b"Period began:"));
}

#[test]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "ordinary public journey selects the original Task, Tree and Ended requests"
)]
fn historical_public_plan_keeps_owner_tree_early_return_and_companion_ended_read_reset() {
    use super::cards::{document, drive, start};
    use crate::{
        Event, Request,
        action::Act,
        address::{Address, Place, TaskSection},
    };
    let (mut client, frame) = start();
    drive(
        &mut client,
        Event::Watch { watch: frame, event: WatchEvent::Delivered(Delivery::Frame(Box::new(document()))) },
    );
    let out =
        drive(&mut client, Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Plan }) });
    let task = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Task { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let out = drive(
        &mut client,
        Event::Watch {
            watch: task,
            event: WatchEvent::Delivered(Delivery::Task(Box::new(super::cards_objects::task_doc(&[])))),
        },
    );
    let watch = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Tree { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let tree = OwnerTree {
        root: super::cards_objects::task_doc(&[]).chip,
        rows: List::with_capacity(CLIENT.rows),
        folded: List::with_capacity(CLIENT.rows),
        held_below: 0,
        next: None,
    };
    drive(
        &mut client,
        Event::Watch { watch, event: WatchEvent::Delivered(Delivery::OwnerTree(Box::new(tree.clone()))) },
    );
    let out = drive(&mut client, Event::Act { act: Act::OpenEnded { task: 8 } });
    let read = out
        .iter()
        .find_map(|r| match r {
            Request::Read { read, what: Read::Ended { task: 8, after: None } } => Some(*read),
            _ => None,
        })
        .unwrap();
    let page = ended(OwnerChipEnding::Failed(doc(b"companion actual cause", true)), None);
    drive(&mut client, Event::Read { read, result: ReadResult::Value(ReadValue::OwnerEnded(Box::new(page.clone()))) });
    let w = words();
    let cap = crate::limits::of_words(&CLIENT, &w).unwrap();
    let mut b = Builder::new(&cap);
    let mut bindings = JigBindings::new(cap.bindings, CLIENT.text);
    crate::pages::main(&mut b, &client, &w, &mut bindings);
    let rendered = b.finish();
    assert!(has(&rendered, b"companion actual cause"));
    caption(&rendered, &w, P::EndedDelegates, &[]);
    assert_eq!(client.task_page().plan.owner.as_deref(), Some(&tree));
    assert_eq!(client.task_page().owner_ended.as_deref(), Some(&page));
    drive(&mut client, Event::Act { act: Act::OpenEnded { task: 8 } });
    assert!(client.task_page().owner_ended.is_none());
    assert_eq!(client.task_page().plan.owner.as_deref(), Some(&tree));
    assert!(!client.task_page().ended_cut);
    assert!(client.task_page().ended_before.is_none() && client.task_page().ended_after.is_none());
}
