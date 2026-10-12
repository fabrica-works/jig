#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary tests inspect complete typed cards and loaded language data"
)]
use super::{
    cards::{CLIENT, contains, text, words},
    cards_objects::{loaded, question, render},
};
use crate::{
    Builder, JigBindings, Limits, Tree,
    phrases::{
        HoleKind, PhraseLimits, Phrasebook, Segment, WallFormat, english,
        jig::{JigPhrase, shapes},
    },
    words::Words,
};
use jig_domain_client::{
    Domain,
    contract::*,
    objects::{Object, question::facts},
};
use skein_lib::{Id, List, Wall, bytes::copy_of};
fn fixture(from: Option<OwnerRequester>) -> (Domain, Id<Object>, Question) {
    let mut value = question();
    value.from = from;
    value.words = text(&[b'w', 255, 0, 128, b'?']);
    value.since = Wall::from_nanos(23);
    let (d, id) = loaded(
        ObjectKey::Question { task: value.task.task, message: value.message },
        ReadValue::Question(Box::new(value.clone())),
    );
    assert_eq!(facts(d.objects().get(id).unwrap()), Some(&value));
    (d, id, value)
}
fn tree(d: &Domain, id: Id<Object>, w: &Words) -> Tree {
    render(d, id, w, &mut JigBindings::new(64, CLIENT.text))
}
fn selected() -> Words {
    let mut first = english::language();
    first.wall = WallFormat::Nanos;
    let mut second = first.clone();
    for phrase in &mut second.phrases {
        let suffix = if phrase.id == JigPhrase::QuestionFromTask.id() {
            Some(b" task/source".as_slice())
        } else if phrase.id == JigPhrase::QuestionFromPerson.id() {
            Some(b" person/source".as_slice())
        } else if phrase.id == JigPhrase::QuestionFromDeployment.id() {
            Some(b" deployment/source".as_slice())
        } else {
            None
        };
        if let Some(suffix) = suffix {
            for form in &mut phrase.forms {
                form.segments =
                    Box::new([Segment::Hole { index: 0, kind: HoleKind::Number }, Segment::Text(copy_of(suffix))]);
            }
        }
    }
    Words {
        book: Phrasebook::load(
            shapes(CLIENT.text),
            Box::new([first, second]),
            &PhraseLimits {
                languages: 2,
                phrases: crate::owner_words::phrase_count(),
                holes: 4,
                segments: 6000,
                text_bytes: 100_000,
                hole_bytes: CLIENT.text,
            },
        )
        .unwrap(),
        language: 1,
        offset_minutes: 60,
    }
}
#[test]
fn question_sender_cards_render_exact_closed_kinds_full_widths_and_no_claim_for_absence() {
    for (from, expected) in [
        (None, b"".as_slice()),
        (Some(OwnerRequester::Task(0)), b"Question from task 0"),
        (Some(OwnerRequester::Task(u64::MAX)), b"Question from task 18446744073709551615"),
        (Some(OwnerRequester::Person(0)), b"Question from person 0"),
        (Some(OwnerRequester::Person(u64::MAX)), b"Question from person 18446744073709551615"),
        (Some(OwnerRequester::Deployment { project: 0 }), b"Question from deployment project 0"),
        (Some(OwnerRequester::Deployment { project: u32::MAX }), b"Question from deployment project 4294967295"),
    ] {
        let (d, id, value) = fixture(from);
        let shown = tree(&d, id, words());
        if from.is_some() {
            assert!(contains(&shown, expected));
        }
        let (task, person, deployment) = match from {
            None => (false, false, false),
            Some(from) => match from {
                OwnerRequester::Task(_) => (true, false, false),
                OwnerRequester::Person(_) => (false, true, false),
                OwnerRequester::Deployment { .. } => (false, false, true),
            },
        };
        assert_eq!(contains(&shown, b"Question from task "), task);
        assert_eq!(contains(&shown, b"Question from person "), person);
        assert_eq!(contains(&shown, b"Question from deployment project "), deployment);
        assert!(contains(&shown, value.words.bytes()), "whole opaque words survive markdown");
        assert!(contains(&shown, b"Waiting "));
        assert!(
            shown
                .nodes()
                .iter()
                .any(|node| node.role == crate::Role::Button && node.name.as_deref() == Some(b"Answer"))
        );
        assert_eq!(facts(d.objects().get(id).unwrap()), Some(&value));
    }
}
#[test]
fn question_sender_selected_language_keeps_reordered_numeric_holes_whole_words_and_waiting_clock() {
    let w = selected();
    for (from, expected) in [
        (OwnerRequester::Task(u64::MAX), b"18446744073709551615 task/source".as_slice()),
        (OwnerRequester::Person(u64::MAX), b"18446744073709551615 person/source"),
        (OwnerRequester::Deployment { project: u32::MAX }, b"4294967295 deployment/source"),
    ] {
        let (d, id, value) = fixture(Some(from));
        let shown = tree(&d, id, &w);
        assert!(contains(&shown, expected));
        assert!(!contains(&shown, b"Question from "));
        assert!(contains(&shown, value.words.bytes()));
        let mut b = Builder::new(&Limits::of_words(&CLIENT, &w).unwrap());
        w.waiting(&mut b, d.wall(), value.since);
        for node in b.finish().nodes() {
            for run in &node.text {
                assert!(contains(&shown, &run.text));
            }
        }
        assert_eq!(facts(d.objects().get(id).unwrap()), Some(&value));
    }
    let (d, id, _) = fixture(None);
    assert!(!contains(&tree(&d, id, &w), b"/source"));
}
#[test]
fn question_sender_maximum_card_has_finite_bounds_and_retains_independent_older_endings() {
    let w = selected();
    for ending in [
        None,
        Some(Ending::Done),
        Some(Ending::Failed { reason: Some(text(b"old failure")) }),
        Some(Ending::Cancelled {
            by: Actor::Party(u64::MAX),
            reason: Some(text(b"old cancellation")),
            at: Some(Wall::from_nanos(u64::MAX)),
        }),
    ] {
        let mut value = question();
        value.from = Some(OwnerRequester::Deployment { project: u32::MAX });
        value.words = text(&[b'x'; 128]);
        value.task.title = DocumentText::complete(text(&[b't'; 128]));
        let mut dependencies = List::with_capacity(CLIENT.rows);
        for number in [0, u64::MAX] {
            dependencies.push(number).unwrap();
        }
        value.task.phase = Phase::Waiting;
        *value.task.details = ChipPhase::Waiting { dependencies, hold: None };
        value.ending = ending;
        let (d, id) = loaded(
            ObjectKey::Question { task: value.task.task, message: value.message },
            ReadValue::Question(Box::new(value.clone())),
        );
        let mut limits = Limits::of_words(&CLIENT, &w).unwrap();
        limits.nodes = crate::question_nodes(&CLIENT).unwrap();
        let mut b = Builder::new(&limits);
        let mut bindings = JigBindings::new(limits.bindings, CLIENT.text);
        assert!(crate::cards::question::question(&mut b, &d, id, &crate::cards::Place::Elsewhere, &w, &mut bindings));
        let shown = b.finish();
        assert!(shown.len() <= limits.nodes);
        let mut total = 0_u64;
        for node in shown.nodes() {
            assert!(node.text_bytes() <= limits.text_per_node);
            total = total.checked_add(u64::from(node.text_bytes())).unwrap();
        }
        assert!(total <= u64::from(limits.text_bytes));
        assert!(contains(&shown, b"4294967295 deployment/source"));
        assert!(contains(&shown, value.words.bytes()));
        assert!(contains(&shown, value.task.title.bytes.bytes()));
        assert_eq!(facts(d.objects().get(id).unwrap()), Some(&value));
        let mut older = value.clone();
        older.from = None;
        let (baseline, object) = loaded(
            ObjectKey::Question { task: older.task.task, message: older.message },
            ReadValue::Question(Box::new(older)),
        );
        let old = tree(&baseline, object, &w);
        let buttons = |t: &Tree| {
            t.nodes()
                .iter()
                .filter(|node| node.role == crate::Role::Button)
                .map(|node| node.name.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(buttons(&shown), buttons(&old), "sender never changes independently supplied ending/act controls");
        if let Some(ending) = &value.ending {
            let mut ending_builder = Builder::new(&Limits::of_words(&CLIENT, &w).unwrap());
            w.ending(&mut ending_builder, ending);
            for node in ending_builder.finish().nodes() {
                for run in &node.text {
                    assert!(contains(&shown, &run.text));
                }
            }
        }
    }
    for phrase in [JigPhrase::QuestionFromTask, JigPhrase::QuestionFromPerson, JigPhrase::QuestionFromDeployment] {
        assert_eq!(
            phrase.shape(CLIENT.text).holes.as_ref(),
            &[crate::phrases::HoleShape { kind: HoleKind::Number, bytes: 0 }]
        );
        assert!(
            w.say(phrase, &[crate::phrases::Hole::Number(u64::MAX)]).len()
                <= usize::try_from(w.book.capacity(phrase.id()).unwrap()).unwrap()
        );
    }
    assert!(crate::question_nodes(&jig_domain_client::Limits { rows: u32::MAX, ..CLIENT }).is_none());
}
