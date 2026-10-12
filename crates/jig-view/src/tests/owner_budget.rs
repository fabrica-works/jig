#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect standalone trees and checked language data")]
use crate::{
    Builder, Limits, Tree, owner_budget as view,
    owner_budget_words::{self as data, BudgetPhrase, OwnerBudgetWords},
    phrases::{Invalid, PhraseId, PhraseLimits, Phrasebook, Segment, WallFormat},
    words::Words,
};
use jig_domain_client::contract::{Funder, OwnerBudget, OwnerNumbers, PoolLedger};
use skein_lib::{Wall, bytes::copy_of};

fn numbers() -> OwnerNumbers {
    OwnerNumbers { budget: 100, spent: 4, spent_below: 7, reserved: 11 }
}
fn budget() -> OwnerBudget {
    OwnerBudget {
        numbers: numbers(),
        left: 17,
        funder: Funder::Task(u64::MAX),
        deadline: Some(Wall::from_nanos(120_000_000_000)),
    }
}
fn ledger() -> PoolLedger {
    PoolLedger {
        source: Funder::Pool { project: u32::MAX, person: u64::MAX, period: u64::MAX - 1 },
        parent: Some(Funder::Recurring { project: u32::MAX - 1, task: u64::MAX - 2, period: u64::MAX - 3 }),
        numbers: numbers(),
        left: 17,
        closed: true,
        began: Some(Wall::from_nanos(120_000_000_000)),
        next: Some(Wall::from_nanos(180_000_000_000)),
    }
}
fn phrase_limits() -> PhraseLimits {
    PhraseLimits {
        languages: 2,
        phrases: data::standalone_phrase_count(),
        holes: 4,
        segments: 200,
        text_bytes: 8192,
        hole_bytes: 256,
    }
}
fn words() -> OwnerBudgetWords {
    OwnerBudgetWords::load(Box::new([data::standalone_language()]), &phrase_limits(), 0, 0)
        .expect("whole checked standalone language")
}
fn limits(fragment: view::BudgetFragment) -> Limits {
    Limits {
        nodes: fragment.nodes,
        depth: fragment.depth,
        text_per_node: fragment.text_per_node,
        text_bytes: fragment.text_bytes,
        drawing_bytes: 0,
        styles_per_node: 0,
        patches: 0,
        bindings: 0,
        slots: 0,
        markdown_depth: 0,
    }
}
fn budget_tree(value: &OwnerBudget, words: &Words) -> Tree {
    let required = view::required(value, words).expect("whole checked capacity");
    let mut builder = Builder::new(&limits(required));
    view::render(&mut builder, value, words);
    let tree = builder.finish();
    assert_eq!(tree.len(), required.nodes);
    tree
}
fn ledger_tree(value: &PoolLedger, words: &Words) -> Tree {
    let required = view::ledger_required(value, words).expect("whole checked capacity");
    let mut builder = Builder::new(&limits(required));
    view::ledger(&mut builder, value, words);
    let tree = builder.finish();
    assert_eq!(tree.len(), required.nodes);
    tree
}
fn has(tree: &Tree, bytes: &[u8]) -> bool {
    tree.nodes().iter().any(|node| node.text.iter().any(|run| run.text.as_ref() == bytes))
}
fn contains(tree: &Tree, bytes: &[u8]) -> bool {
    tree.nodes().iter().any(|node| node.text.iter().any(|run| run.text.windows(bytes.len()).any(|part| part == bytes)))
}

#[test]
fn budget_says_all_owner_numbers_exact_task_funder_and_only_the_supplied_deadline() {
    let value = budget();
    let words = words();
    let tree = budget_tree(&value, words.words());
    assert_eq!(tree.len(), 9);
    for text in [
        &b"Budget: 100"[..],
        &b"Spent directly: 4"[..],
        &b"Settled spend below: 7"[..],
        &b"Reserved for funded work: 11"[..],
        &b"Left: 17"[..],
        &b"Funded by"[..],
        &b"Deadline: 120000000000"[..],
    ] {
        assert!(has(&tree, text), "owner accounting is never merged or recomputed");
    }
    assert!(!has(&tree, b"Left: 78"), "availability is an owner fact");
    assert!(contains(&tree, b"18446744073709551615"));
    let mut without_deadline = value;
    without_deadline.deadline = None;
    let tree = budget_tree(&without_deadline, words.words());
    assert_eq!(tree.len(), 8);
    assert!(!contains(&tree, b"Deadline:"));
}
#[test]
fn ledger_keeps_distinct_parent_periods_full_source_widths_and_explicit_optional_facts() {
    let value = ledger();
    let words = words();
    let tree = ledger_tree(&value, words.words());
    assert_eq!(tree.len(), 13);
    for text in [
        &b"Ledger source"[..],
        &b"Parent funder"[..],
        &b"Ledger is closed"[..],
        &b"Left: 17"[..],
        &b"Period began: 120000000000"[..],
        &b"Next period begins: 180000000000"[..],
    ] {
        assert!(has(&tree, text));
    }
    for number in [
        &b"4294967295"[..],
        &b"4294967294"[..],
        &b"18446744073709551615"[..],
        &b"18446744073709551614"[..],
        &b"18446744073709551613"[..],
        &b"18446744073709551612"[..],
    ] {
        assert!(contains(&tree, number), "source identity never becomes an Actor or current period");
    }
    let mut absent = value;
    absent.parent = None;
    absent.began = None;
    absent.next = None;
    absent.closed = false;
    let tree = ledger_tree(&absent, words.words());
    assert_eq!(tree.len(), 9);
    assert!(has(&tree, b"Ledger is open"));
    for label in [&b"Parent funder"[..], &b"Period began:"[..], &b"Next period begins:"[..], &b"Ledger is closed"[..]] {
        assert!(!contains(&tree, label), "absence never supplies a parent or calendar fact");
    }
}
#[test]
fn budget_and_shared_funder_sayings_borrow_selected_normal_words_and_person_offset() {
    let mut translated = crate::phrases::english::language();
    translated.wall = WallFormat::Clock;
    for phrase in &mut translated.phrases {
        for form in &mut phrase.forms {
            for segment in &mut form.segments {
                match segment {
                    Segment::Text(bytes) => *bytes = copy_of(b"selected "),
                    Segment::Hole { .. } => {}
                }
            }
        }
    }
    let languages: Box<[_]> = Box::new([crate::phrases::english::language(), translated]);
    let mut segments = 0_u32;
    let mut text_bytes = 0_u32;
    for language in &languages {
        for phrase in &language.phrases {
            for form in &phrase.forms {
                segments = segments
                    .checked_add(u32::try_from(form.segments.len()).expect("form segment count fits"))
                    .expect("both complete languages' segment count fits");
                for segment in &form.segments {
                    match segment {
                        Segment::Text(bytes) => {
                            text_bytes = text_bytes
                                .checked_add(u32::try_from(bytes.len()).expect("literal byte count fits"))
                                .expect("both complete languages' literal byte count fits");
                        }
                        Segment::Hole { .. } => {}
                    }
                }
            }
        }
    }
    let limits = PhraseLimits {
        languages: 2,
        phrases: crate::owner_words::phrase_count(),
        holes: 4,
        segments,
        text_bytes,
        hole_bytes: 256,
    };
    let book = Phrasebook::load(crate::phrases::jig::shapes(256), languages, &limits)
        .expect("normal composed vocabulary checks both supplied languages whole");
    let words = Words { book, language: 1, offset_minutes: -1 };
    let tree = budget_tree(&budget(), &words);
    assert!(has(&tree, b"selected 17"));
    assert!(has(&tree, b"selected 00:01"));
    assert!(contains(&tree, b"selected 18446744073709551615"));
    assert!(!contains(&tree, b"Budget:"));
    assert!(!contains(&tree, b"Funded by"));
    let tree = ledger_tree(&ledger(), &words);
    assert!(has(&tree, b"selected 17"));
    assert!(has(&tree, b"selected 00:01"));
    assert!(has(&tree, b"selected 00:02"));
    assert!(!contains(&tree, b"Ledger source"));
    assert!(
        crate::owner_words::phrase_bytes(&words.book).expect("whole normal saying envelope")
            >= view::ledger_worst_case(&words).expect("whole ledger envelope").text_per_node
    );
    let mut value = budget();
    value.deadline = Some(Wall::EPOCH);
    assert!(has(&budget_tree(&value, &words), b"selected 00:00"));
}
#[test]
fn budget_words_check_the_whole_closed_language_holes_selection_and_offset_before_retaining_it() {
    let mut language = data::standalone_language();
    language.phrases[0].id = BudgetPhrase::Spent.id();
    assert_eq!(
        OwnerBudgetWords::load(Box::new([language]), &phrase_limits(), 0, 0).expect_err("duplicate refused"),
        Invalid::Duplicate { id: BudgetPhrase::Spent.id() }
    );
    let mut language = data::standalone_language();
    language.phrases = language.phrases.into_vec().into_iter().skip(1).collect();
    assert_eq!(
        OwnerBudgetWords::load(Box::new([language]), &phrase_limits(), 0, 0).expect_err("missing refused"),
        Invalid::Missing { id: BudgetPhrase::Budget.id() }
    );
    let mut language = data::standalone_language();
    language.phrases[0].forms[0].segments = Box::new([Segment::Text(copy_of(b"no amount hole"))]);
    assert_eq!(
        OwnerBudgetWords::load(Box::new([language]), &phrase_limits(), 0, 0).expect_err("wrong holes refused"),
        Invalid::Holes { id: BudgetPhrase::Budget.id() }
    );
    assert_eq!(
        OwnerBudgetWords::load(Box::new([data::language()]), &phrase_limits(), 0, 0)
            .expect_err("shared sayings must also be supplied"),
        Invalid::Missing { id: PhraseId { owner: crate::Owner::Jig, number: 5000 } }
    );
    assert_eq!(
        OwnerBudgetWords::load(Box::new([data::standalone_language()]), &phrase_limits(), 1, 0)
            .expect_err("unknown language refused"),
        Invalid::Language
    );
    for offset in [i32::MIN, i32::MAX] {
        assert_eq!(
            OwnerBudgetWords::load(Box::new([data::standalone_language()]), &phrase_limits(), 0, offset)
                .expect_err("unrepresentable offset refused before retaining words"),
            Invalid::Overflow
        );
    }
}
#[test]
fn normal_budget_sizing_refuses_a_book_without_shared_funder_sayings() {
    let book = Phrasebook::load(data::shapes(), Box::new([data::language()]), &phrase_limits())
        .expect("budget-only language is checked for its own shapes");
    let words = Words { book, language: 0, offset_minutes: 0 };
    assert_eq!(view::required(&budget(), &words), None);
    assert_eq!(view::ledger_required(&ledger(), &words), None);
    assert_eq!(view::worst_case(&words), None);
    assert_eq!(view::ledger_worst_case(&words), None);
}
