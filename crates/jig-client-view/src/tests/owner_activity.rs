#![expect(clippy::disallowed_methods, reason = "ordinary proofs inspect public checked languages and exact trees")]
use crate::{
    Builder, Limits, Tree, owner_activity as view,
    owner_activity_words::{self as data, ActivityPhrase as P},
    phrases::{Invalid, PhraseLimits, Phrasebook, Segment, SpanUnit, WallFormat},
    words::Words,
};
use jig_client::contract::*;
use skein_lib::{List, Wall, bytes::copy_of};
fn text(bytes: &[u8]) -> Text {
    Text::new(copy_of(bytes), u32::try_from(bytes.len()).unwrap()).unwrap()
}
fn fields() -> List<ResultField> {
    let mut fields = List::with_capacity(3);
    fields.push(ResultField { name: copy_of(&[255, 0]), value: copy_of(b"opaque") }).unwrap();
    fields.push(ResultField { name: copy_of(b"a/b"), value: copy_of(&[254, 1]) }).unwrap();
    fields
}
fn fixture(kind: OwnerMessageKind) -> OwnerWord {
    OwnerWord {
        number: u64::MAX,
        from: OwnerRequester::Person(u64::MAX),
        kind,
        words: text(&[255, 0]),
        at: Wall::from_nanos(u64::MAX),
        hits: u32::MAX,
        eligible: true,
    }
}
fn activity(line: OwnerActivity) -> OwnerActivityLine {
    OwnerActivityLine { task: u64::MAX, position: u64::MAX, at: Wall::from_nanos(u64::MAX), line }
}
fn client_limits() -> jig_client::Limits {
    jig_client::Limits { rows: 3, text: 8, ..super::cards::CLIENT }
}
fn phrase_limits() -> PhraseLimits {
    PhraseLimits {
        languages: 2,
        phrases: crate::owner_words::phrase_count(),
        holes: 4,
        segments: 8000,
        text_bytes: 150_000,
        hole_bytes: 8,
    }
}
fn normal_words() -> &'static Words {
    static WORDS: std::sync::OnceLock<Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| words(false))
}
fn words(selected: bool) -> Words {
    let mut english = crate::phrases::english::language();
    english.wall = WallFormat::Nanos;
    english.span_unit = SpanUnit::Nanos;
    let languages: Box<[crate::phrases::Language]> = if selected {
        let mut translated = english.clone();
        for phrase in &mut translated.phrases {
            for form in &mut phrase.forms {
                for segment in &mut form.segments {
                    if let Segment::Text(bytes) = segment {
                        *bytes = copy_of(b"selected activity: ");
                    }
                }
            }
        }
        Box::new([english, translated])
    } else {
        Box::new([english])
    };
    Words {
        book: Phrasebook::load(crate::phrases::jig::shapes(8), languages, &phrase_limits()).unwrap(),
        language: if selected { 1 } else { 0 },
        offset_minutes: 0,
    }
}
fn tree_limits(f: view::ActivityFragment) -> Limits {
    Limits {
        nodes: f.nodes,
        depth: f.depth,
        text_bytes: f.text_bytes,
        text_per_node: f.text_per_node,
        drawing_bytes: 0,
        styles_per_node: 2,
        patches: 0,
        bindings: f.bindings,
        slots: f.slots,
        markdown_depth: 2,
    }
}
fn build(value: &OwnerActivityLine, words: &Words) -> Tree {
    let f = view::required(value, &client_limits(), words).unwrap();
    let limits = tree_limits(f);
    assert!(crate::worst_case(&limits).is_some());
    let mut b = Builder::new(&limits);
    view::render(&mut b, value, words);
    let tree = b.finish();
    envelope(&tree, f);
    tree
}
fn build_word(value: &OwnerWord, words: &Words) -> Tree {
    let f = view::word_required(value, &client_limits(), words).unwrap();
    let mut b = Builder::new(&tree_limits(f));
    view::word(&mut b, value, words);
    let tree = b.finish();
    envelope(&tree, f);
    tree
}
fn envelope(tree: &Tree, f: view::ActivityFragment) {
    assert!(tree.len() <= f.nodes);
    let mut total = 0_u64;
    for node in tree.nodes() {
        assert!(node.text_bytes() <= f.text_per_node);
        total += u64::from(node.text_bytes());
        if let Some(name) = &node.name {
            assert!(name.len() <= usize::try_from(f.text_per_node).unwrap());
            total += u64::try_from(name.len()).unwrap();
        }
        assert!(node.binding.is_none() && node.slot.is_none());
        if node.role == crate::Role::Region {
            assert!(node.name.is_some());
        }
    }
    assert!(total <= u64::from(f.text_bytes));
    assert_eq!((f.bindings, f.slots), (0, 0));
}
fn has(tree: &Tree, bytes: &[u8]) -> bool {
    tree.nodes().iter().flat_map(|n| &n.text).any(|run| run.text.as_ref() == bytes)
}
fn kinds() -> Box<[(OwnerMessageKind, &'static [u8])]> {
    Box::new([
        (OwnerMessageKind::Escalation { task: u64::MAX, revision: u64::MAX }, b"Kind: escalation"),
        (
            OwnerMessageKind::Proposal {
                proposer: u64::MAX,
                proposal: u64::MAX,
                kind: OwnerMessageProposalKind::Batch,
            },
            b"Kind: proposal",
        ),
        (OwnerMessageKind::PersonStalled { task: u64::MAX }, b"Kind: person stalled"),
        (OwnerMessageKind::ProposalDecision { proposal: u64::MAX, accepted: true }, b"Kind: proposal decision"),
        (OwnerMessageKind::Words, b"Kind: words"),
        (OwnerMessageKind::EffectNotMade { entry: u64::MAX }, b"Kind: effect not made"),
        (OwnerMessageKind::Amendment { revision: u64::MAX }, b"Kind: amendment"),
        (OwnerMessageKind::Question, b"Kind: question"),
        (OwnerMessageKind::Answer { question: u64::MAX }, b"Kind: answer"),
        (
            OwnerMessageKind::Notice { subscription: u64::MAX, target: u64::MAX, state: OwnerNoticeState::Held },
            b"Kind: notice",
        ),
        (OwnerMessageKind::Timer { subscription: u64::MAX }, b"Kind: timer"),
        (OwnerMessageKind::News { subscription: u64::MAX, class: OwnerNewsClass::Wakes }, b"Kind: news"),
        (OwnerMessageKind::Result(OwnerWordResult::Report), b"Kind: result"),
    ])
}
#[test]
fn owner_activity_all_eight_lines_keep_task_commit_clock_and_exact_variant_facts() {
    let lines: Box<[(OwnerActivity, &[u8])]> = Box::new([
        (
            OwnerActivity::Words {
                message: u64::MAX,
                by: OwnerRequester::Task(u64::MAX),
                kind: OwnerMessageKind::Words,
                words: text(&[255, 0]),
            },
            b"Words recorded",
        ),
        (OwnerActivity::Heard(fixture(OwnerMessageKind::Question)), b"Message heard"),
        (
            OwnerActivity::Notified {
                recipient: OwnerRequester::Deployment { project: u32::MAX },
                word: fixture(OwnerMessageKind::Words),
            },
            b"Message notified",
        ),
        (OwnerActivity::RunBegan { attempt: u64::MAX, why: OwnerRunWhy::Made, resumed: false }, b"Run began"),
        (OwnerActivity::RunEnded { attempt: u64::MAX, end: OwnerRunEnd::Refused, spent: u64::MAX }, b"Run ended"),
        (
            OwnerActivity::Turn { reply: u64::MAX, attempt: u64::MAX, turn: u32::MAX, read: Some(u64::MAX) },
            b"Committed turn reference",
        ),
        (OwnerActivity::Decided { revision: u64::MAX }, b"Decision recorded"),
        (OwnerActivity::Milestone { position: u64::MAX }, b"Milestone"),
    ]);
    assert_eq!(lines.len(), 8);
    for (line, label) in lines {
        let value = activity(line);
        let original = value.clone();
        let tree = build(&value, normal_words());
        for bytes in [
            label,
            b"Activity task: 18446744073709551615",
            b"Commit position: 18446744073709551615",
            b"Recorded at: 18446744073709551615",
        ] {
            assert!(has(&tree, bytes));
        }
        assert_line_fields(&tree, &value.line);
        assert_eq!(value, original);
        for absent in [b"Fresh tail".as_slice(), b"Failure reason", b"Decoded turn", b"Unknown sender"] {
            assert!(!has(&tree, absent));
        }
    }
}
fn assert_line_fields(tree: &Tree, line: &OwnerActivity) {
    match line {
        OwnerActivity::Words { .. } => {
            assert!(has(tree, b"Message: 18446744073709551615"));
            assert!(has(tree, b"Task: 18446744073709551615"));
            assert!(has(tree, b"\\xFF\\x00"));
        }
        OwnerActivity::Heard(_) => {
            assert!(has(tree, b"Message: 18446744073709551615"));
            assert!(has(tree, b"Merged hints: 4294967295"));
        }
        OwnerActivity::Notified { .. } => {
            assert!(has(tree, b"Deployment project: 4294967295"));
            assert!(has(tree, b"Person: 18446744073709551615"));
        }
        OwnerActivity::RunBegan { .. } => {
            assert!(has(tree, b"Attempt: 18446744073709551615"));
            assert!(has(tree, b"Cause: made"));
            assert!(has(tree, b"Not resumed"));
        }
        OwnerActivity::RunEnded { .. } => {
            assert!(has(tree, b"Attempt: 18446744073709551615"));
            assert!(has(tree, b"Spent: 18446744073709551615"));
            assert!(has(tree, b"End: refused"));
        }
        OwnerActivity::Turn { .. } => {
            assert!(has(tree, b"Archive reply: 18446744073709551615"));
            assert!(has(tree, b"Turn: 4294967295"));
            assert!(has(tree, b"Last read message: 18446744073709551615"));
        }
        OwnerActivity::Decided { .. } => assert!(has(tree, b"Revision: 18446744073709551615")),
        OwnerActivity::Milestone { .. } => assert_eq!(
            tree.nodes()
                .iter()
                .flat_map(|n| &n.text)
                .filter(|r| r.text.as_ref() == b"Commit position: 18446744073709551615")
                .count(),
            2
        ),
    }
}
#[test]
fn owner_activity_all_thirteen_message_kinds_keep_whole_word_hits_clock_and_flags() {
    assert_eq!(kinds().len(), 13);
    for (kind, label) in kinds() {
        let word = fixture(kind);
        let original = word.clone();
        let tree = build_word(&word, normal_words());
        for bytes in [
            label,
            b"Message: 18446744073709551615",
            b"Person: 18446744073709551615",
            b"Recorded at: 18446744073709551615",
            b"Merged hints: 4294967295",
            b"Eligible",
            b"\\xFF\\x00",
        ] {
            assert!(has(&tree, bytes));
        }
        assert_kind_numbers(&tree, &word.kind);
        assert_eq!(word, original);
        let mut ineligible = word;
        ineligible.eligible = false;
        ineligible.hits = 0;
        let tree = build_word(&ineligible, normal_words());
        assert!(has(&tree, b"Not eligible"));
        assert!(has(&tree, b"Merged hints: 0"));
    }
}
fn assert_kind_numbers(tree: &Tree, kind: &OwnerMessageKind) {
    match kind {
        OwnerMessageKind::Escalation { .. } => {
            assert!(has(tree, b"Task: 18446744073709551615"));
            assert!(has(tree, b"Revision: 18446744073709551615"));
        }
        OwnerMessageKind::Proposal { .. } => {
            assert!(has(tree, b"Proposer number: 18446744073709551615"));
            assert!(has(tree, b"Proposal: 18446744073709551615"));
        }
        OwnerMessageKind::PersonStalled { .. } => assert!(has(tree, b"Task: 18446744073709551615")),
        OwnerMessageKind::ProposalDecision { .. } => assert!(has(tree, b"Proposal: 18446744073709551615")),
        OwnerMessageKind::EffectNotMade { .. } => assert!(has(tree, b"Entry: 18446744073709551615")),
        OwnerMessageKind::Amendment { .. } => assert!(has(tree, b"Revision: 18446744073709551615")),
        OwnerMessageKind::Answer { .. } => assert!(has(tree, b"Question: 18446744073709551615")),
        OwnerMessageKind::Notice { .. } => {
            assert!(has(tree, b"Subscription: 18446744073709551615"));
            assert!(has(tree, b"Target task: 18446744073709551615"));
        }
        OwnerMessageKind::Timer { .. } | OwnerMessageKind::News { .. } => {
            assert!(has(tree, b"Subscription: 18446744073709551615"))
        }
        OwnerMessageKind::Words | OwnerMessageKind::Question | OwnerMessageKind::Result(_) => {}
    }
}
#[test]
fn owner_activity_equal_sender_numbers_keep_task_person_and_deployment_distinct() {
    let senders: Box<[(OwnerRequester, &[u8])]> = Box::new([
        (OwnerRequester::Task(7), b"Task: 7"),
        (OwnerRequester::Person(7), b"Person: 7"),
        (OwnerRequester::Deployment { project: 7 }, b"Deployment project: 7"),
    ]);
    let mut retained = List::with_capacity(3);
    for (from, label) in senders {
        let word = OwnerWord { from, ..fixture(OwnerMessageKind::Words) };
        let tree = build_word(&word, normal_words());
        assert!(has(&tree, label));
        for previous in &retained {
            assert_ne!(&word, previous);
        }
        let notified = activity(OwnerActivity::Notified { recipient: from, word: word.clone() });
        assert!(has(&build(&notified, normal_words()), label));
        retained.push(word).unwrap();
    }
    assert_eq!(retained.len(), 3);
    let tree = build_word(
        &OwnerWord { from: OwnerRequester::Deployment { project: u32::MAX }, ..fixture(OwnerMessageKind::Words) },
        normal_words(),
    );
    assert!(has(&tree, b"Deployment project: 4294967295"));
}
#[test]
fn owner_activity_closed_run_causes_and_ends_keep_all_six_failure_classes_and_counters() {
    let tries = OwnerFailureTries { transient: 1, permanent: 2, run: 3, agent: 4, lost: 5, invalid: u32::MAX };
    let causes: Box<[(OwnerRunWhy, &[u8])]> = Box::new([
        (OwnerRunWhy::Made, b"Cause: made"),
        (OwnerRunWhy::Ready, b"Cause: ready"),
        (OwnerRunWhy::Message { message: u64::MAX }, b"Cause: message"),
        (OwnerRunWhy::Released, b"Cause: released"),
        (OwnerRunWhy::Retry { tries }, b"Cause: retry"),
        (OwnerRunWhy::Narrowed, b"Cause: narrowed"),
        (OwnerRunWhy::Changed, b"Cause: changed"),
    ]);
    for (why, label) in causes {
        for resumed in [false, true] {
            let value = activity(OwnerActivity::RunBegan { attempt: u64::MAX, why, resumed });
            let tree = build(&value, normal_words());
            assert!(has(&tree, label));
            assert!(has(&tree, b"Attempt: 18446744073709551615"));
            assert!(has(&tree, if resumed { b"Resumed" } else { b"Not resumed" }));
        }
    }
    let tree = build(
        &activity(OwnerActivity::RunBegan { attempt: 9, why: OwnerRunWhy::Retry { tries }, resumed: true }),
        normal_words(),
    );
    for label in [
        b"Transient failures: 1".as_slice(),
        b"Permanent failures: 2",
        b"Run failures: 3",
        b"Agent failures: 4",
        b"Lost failures: 5",
        b"Invalid failures: 4294967295",
    ] {
        assert!(has(&tree, label));
    }
    let ends: Box<[(OwnerRunEnd, &[u8])]> = Box::new([
        (OwnerRunEnd::Parked, b"End: parked"),
        (OwnerRunEnd::Finished, b"End: finished"),
        (OwnerRunEnd::Cancelled, b"End: cancelled"),
        (OwnerRunEnd::Refused, b"End: refused"),
    ]);
    for (end, label) in ends {
        let tree =
            build(&activity(OwnerActivity::RunEnded { attempt: u64::MAX, end, spent: u64::MAX }), normal_words());
        assert!(has(&tree, label));
        assert!(has(&tree, b"Spent: 18446744073709551615"));
        assert!(has(&tree, b"Attempt: 18446744073709551615"));
    }
    assert_max_tries();
    let classes: Box<[(HoldFailure, &[u8])]> = Box::new([
        (HoldFailure::Transient, b"Class: transient"),
        (HoldFailure::Permanent, b"Class: permanent"),
        (HoldFailure::Run, b"Class: run"),
        (HoldFailure::Agent, b"Class: agent"),
        (HoldFailure::Lost, b"Class: lost"),
        (HoldFailure::Invalid, b"Class: invalid"),
    ]);
    for (class, label) in classes {
        let tree = build(
            &activity(OwnerActivity::RunEnded { attempt: 7, end: OwnerRunEnd::Failed { class }, spent: 0 }),
            normal_words(),
        );
        assert!(has(&tree, b"End: failed"));
        assert!(has(&tree, label));
        assert!(!has(&tree, b"Failure reason"));
    }
}
fn assert_max_tries() {
    let tries = OwnerFailureTries {
        transient: u32::MAX,
        permanent: u32::MAX,
        run: u32::MAX,
        agent: u32::MAX,
        lost: u32::MAX,
        invalid: u32::MAX,
    };
    let tree = build(
        &activity(OwnerActivity::RunBegan { attempt: 0, why: OwnerRunWhy::Retry { tries }, resumed: false }),
        normal_words(),
    );
    for label in [
        b"Transient failures: 4294967295".as_slice(),
        b"Permanent failures: 4294967295",
        b"Run failures: 4294967295",
        b"Agent failures: 4294967295",
        b"Lost failures: 4294967295",
        b"Invalid failures: 4294967295",
    ] {
        assert!(has(&tree, label));
    }
}
#[test]
fn owner_activity_archive_read_none_zero_and_max_are_distinct_without_decoding_or_clock_ordering() {
    let mut retained = List::with_capacity(3);
    for read in [None, Some(0), Some(u64::MAX)] {
        let value = activity(OwnerActivity::Turn { reply: u64::MAX, attempt: u64::MAX, turn: u32::MAX, read });
        let tree = build(&value, normal_words());
        assert!(has(&tree, b"Archive reply: 18446744073709551615"));
        assert!(has(&tree, b"Turn: 4294967295"));
        match read {
            None => assert!(has(&tree, b"Last read message not supplied")),
            Some(0) => assert!(has(&tree, b"Last read message: 0")),
            Some(_) => assert!(has(&tree, b"Last read message: 18446744073709551615")),
        }
        for previous in &retained {
            assert_ne!(&value, previous);
        }
        retained.push(value).unwrap();
    }
    let first = OwnerActivityLine {
        position: 9,
        at: Wall::from_nanos(100),
        ..activity(OwnerActivity::Decided { revision: 3 })
    };
    let later = OwnerActivityLine { position: 10, at: Wall::from_nanos(0), ..first.clone() };
    assert!(has(&build(&first, normal_words()), b"Commit position: 9"));
    assert!(has(&build(&later, normal_words()), b"Recorded at: 0"));
    assert_eq!(first.position, 9);
    assert_eq!(later.position, 10);
    assert_time_presentation();
}
fn assert_time_presentation() {
    let value = OwnerWord { at: Wall::from_nanos(123), ..fixture(OwnerMessageKind::Words) };
    let presentations: Box<[(i32, &[u8])]> =
        Box::new([(1, b"Recorded at: 60000000123"), (0, b"Recorded at: 123"), (-1, b"Recorded at: 0")]);
    for (offset, label) in presentations {
        let mut words = normal_words().clone();
        words.offset_minutes = offset;
        assert!(has(&build_word(&value, &words), label));
        let line = OwnerActivityLine { at: value.at, ..activity(OwnerActivity::Heard(value.clone())) };
        assert!(has(&build(&line, &words), label));
        assert_eq!(value.at.as_nanos(), 123);
    }
}
#[test]
fn owner_activity_all_six_result_metadata_keep_approved_presence_opaque_fields_and_change_identity() {
    let variants: Box<[(OwnerWordResult, &[u8])]> = Box::new([
        (OwnerWordResult::Approved { state: text(&[254, 0]), words_present: true }, b"Result: approved"),
        (OwnerWordResult::Report, b"Result: report"),
        (OwnerWordResult::Verdict { code: u32::MAX, fields: fields() }, b"Result: verdict"),
        (OwnerWordResult::Change { connector: u16::MAX, kind: u16::MAX, resource: u64::MAX }, b"Result: change"),
        (OwnerWordResult::Failed, b"Result: failed"),
        (OwnerWordResult::Cancelled, b"Result: cancelled"),
    ]);
    for (result, label) in variants {
        let tree = build_word(&fixture(OwnerMessageKind::Result(result)), normal_words());
        assert!(has(&tree, label));
    }
    let tree = build_word(
        &fixture(OwnerMessageKind::Result(OwnerWordResult::Verdict { code: u32::MAX, fields: fields() })),
        normal_words(),
    );
    for label in [b"Verdict code: 4294967295".as_slice(), b"\\xFF\\x00", b"opaque", b"a/b", b"\\xFE\\x01"] {
        assert!(has(&tree, label));
    }
    let codes: Box<[&[u8]]> = tree
        .nodes()
        .iter()
        .flat_map(|n| &n.text)
        .filter(|r| r.run == crate::Run::Code)
        .map(|r| r.text.as_ref())
        .collect();
    assert_eq!(codes.as_ref(), [b"\\xFF\\x00".as_slice(), b"opaque", b"a/b", b"\\xFE\\x01", b"\\xFF\\x00"].as_slice());
    let tree = build_word(
        &fixture(OwnerMessageKind::Result(OwnerWordResult::Change {
            connector: u16::MAX,
            kind: u16::MAX,
            resource: u64::MAX,
        })),
        normal_words(),
    );
    for label in [b"Connector: 65535".as_slice(), b"Change kind: 65535", b"Resource: 18446744073709551615"] {
        assert!(has(&tree, label));
    }
    for present in [false, true] {
        let word = OwnerWord {
            kind: OwnerMessageKind::Result(OwnerWordResult::Approved { state: text(b""), words_present: present }),
            words: text(b""),
            ..fixture(OwnerMessageKind::Words)
        };
        assert!(has(
            &build_word(&word, normal_words()),
            if present { b"Explanation words present" } else { b"Explanation words absent" }
        ));
    }
    let invalid =
        fixture(OwnerMessageKind::Result(OwnerWordResult::Approved { state: text(b""), words_present: false }));
    assert!(view::word_required(&invalid, &client_limits(), normal_words()).is_none());
    let invalid_line =
        activity(OwnerActivity::Words { message: 1, by: invalid.from, kind: invalid.kind, words: invalid.words });
    assert!(view::required(&invalid_line, &client_limits(), normal_words()).is_none());
}
#[test]
fn owner_activity_nested_message_proposal_notice_news_and_decision_values_remain_closed() {
    let proposals: Box<[(OwnerMessageProposalKind, &[u8])]> = Box::new([
        (OwnerMessageProposalKind::Batch, b"Proposal kind: batch"),
        (OwnerMessageProposalKind::Effect, b"Proposal kind: effect"),
        (OwnerMessageProposalKind::Amend, b"Proposal kind: amend"),
        (OwnerMessageProposalKind::Widen, b"Proposal kind: widen"),
        (OwnerMessageProposalKind::Release, b"Proposal kind: release"),
    ]);
    for (kind, label) in proposals {
        let tree =
            build_word(&fixture(OwnerMessageKind::Proposal { proposer: u64::MAX, proposal: 7, kind }), normal_words());
        assert!(has(&tree, label));
        assert!(has(&tree, b"Proposer number: 18446744073709551615"));
        assert!(has(&tree, b"Proposal: 7"));
    }
    let states: Box<[(OwnerNoticeState, &[u8])]> = Box::new([
        (OwnerNoticeState::Held, b"Notice: held"),
        (OwnerNoticeState::Done, b"Notice: done"),
        (OwnerNoticeState::Failed, b"Notice: failed"),
        (OwnerNoticeState::Cancelled, b"Notice: cancelled"),
    ]);
    for (state, label) in states {
        let tree = build_word(
            &fixture(OwnerMessageKind::Notice { subscription: u64::MAX, target: u64::MAX, state }),
            normal_words(),
        );
        assert!(has(&tree, label));
        assert!(has(&tree, b"Subscription: 18446744073709551615"));
        assert!(has(&tree, b"Target task: 18446744073709551615"));
    }
    let classes: Box<[(OwnerNewsClass, &[u8])]> = Box::new([
        (OwnerNewsClass::Wakes, b"News: wakes"),
        (OwnerNewsClass::Kept, b"News: kept"),
        (OwnerNewsClass::Dropped, b"News: dropped"),
    ]);
    for (class, label) in classes {
        let tree = build_word(&fixture(OwnerMessageKind::News { subscription: 8, class }), normal_words());
        assert!(has(&tree, label));
        assert!(has(&tree, b"Subscription: 8"));
    }
    for accepted in [false, true] {
        let tree =
            build_word(&fixture(OwnerMessageKind::ProposalDecision { proposal: u64::MAX, accepted }), normal_words());
        assert!(has(&tree, if accepted { b"Accepted" } else { b"Rejected" }));
    }
}
#[test]
fn owner_activity_selected_normal_language_and_full_opaque_tree_envelope_refuse_whole_excess() {
    let selected = words(true);
    let mut word = fixture(OwnerMessageKind::Words);
    word.kind = OwnerMessageKind::Result(OwnerWordResult::Verdict { code: u32::MAX, fields: List::with_capacity(3) });
    if let OwnerMessageKind::Result(OwnerWordResult::Verdict { fields, .. }) = &mut word.kind {
        for _ in 0_u32..3 {
            fields.push(ResultField { name: copy_of(&[254; 8]), value: copy_of(&[253; 8]) }).unwrap();
        }
    }
    word.words = text(&[255; 8]);
    let value = activity(OwnerActivity::Notified {
        recipient: OwnerRequester::Deployment { project: u32::MAX },
        word: word.clone(),
    });
    let tree = build(&value, &selected);
    assert!(has(&tree, b"selected activity: 4294967295"));
    for bytes in [
        b"\\xFF\\xFF\\xFF\\xFF\\xFF\\xFF\\xFF\\xFF".as_slice(),
        b"\\xFE\\xFE\\xFE\\xFE\\xFE\\xFE\\xFE\\xFE",
        b"\\xFD\\xFD\\xFD\\xFD\\xFD\\xFD\\xFD\\xFD",
    ] {
        assert!(has(&tree, bytes));
    }
    assert!(tree.nodes().iter().any(|n| n.name.as_ref().is_some_and(|name| name.as_ref() == b"selected activity: ")));
    assert!(!has(&tree, b"Result: verdict"));
    let cloned = selected.clone();
    assert_eq!(build(&value, &cloned).nodes(), tree.nodes());
    for phrase in data::ALL {
        assert!(selected.book.capacity(phrase.id()).is_some());
    }
    assert!(view::required(&value, &jig_client::Limits { text: 7, ..client_limits() }, &selected).is_none());
    assert!(view::required(&value, &jig_client::Limits { rows: 2, ..client_limits() }, &selected).is_none());
    assert!(view::worst_case(&jig_client::Limits { rows: u32::MAX, ..client_limits() }, &selected).is_none());
    assert!(view::worst_case(&jig_client::Limits { text: u32::MAX, ..client_limits() }, &selected).is_none());
    let reserved =
        fixture(OwnerMessageKind::Result(OwnerWordResult::Verdict { code: 1, fields: List::with_capacity(4) }));
    assert!(view::word_required(&reserved, &client_limits(), &selected).is_none());
    assert_each_text_bound(&selected);
    let saved = word.clone();
    if let OwnerMessageKind::Result(OwnerWordResult::Verdict { fields, .. }) = &mut word.kind {
        fields.get_mut(0).unwrap().name = copy_of(b"123456789");
    }
    assert!(view::word_required(&word, &client_limits(), &selected).is_none());
    assert_eq!(saved.words.bytes(), &[255; 8]);
}
fn assert_each_text_bound(words: &Words) {
    let mutations: [fn(&mut OwnerWord); 4] = [
        |s| s.words = text(b"123456789"),
        |s| {
            s.kind =
                OwnerMessageKind::Result(OwnerWordResult::Approved { state: text(b"123456789"), words_present: true })
        },
        |s| {
            if let OwnerMessageKind::Result(OwnerWordResult::Verdict { fields, .. }) = &mut s.kind {
                fields.get_mut(0).unwrap().name = copy_of(b"123456789");
            }
        },
        |s| {
            if let OwnerMessageKind::Result(OwnerWordResult::Verdict { fields, .. }) = &mut s.kind {
                fields.get_mut(0).unwrap().value = copy_of(b"123456789");
            }
        },
    ];
    for mutate in mutations {
        let mut value = fixture(OwnerMessageKind::Result(OwnerWordResult::Verdict { code: 7, fields: fields() }));
        mutate(&mut value);
        let saved = value.clone();
        assert!(view::word_required(&value, &client_limits(), words).is_none());
        assert!(view::required(&activity(OwnerActivity::Heard(value.clone())), &client_limits(), words).is_none());
        assert!(view::word_required(&value, &jig_client::Limits { text: 9, ..client_limits() }, words).is_some());
        assert_eq!(value, saved);
    }
    for capacity in [0_u32, 1, 3] {
        let value = fixture(OwnerMessageKind::Result(OwnerWordResult::Verdict {
            code: 0,
            fields: List::with_capacity(capacity),
        }));
        assert!(view::word_required(&value, &client_limits(), words).is_some());
    }
}
#[test]
fn owner_activity_checked_normal_book_rejects_missing_duplicate_wrong_number_and_wall_holes() {
    let mut language = crate::phrases::english::language();
    language.phrases = language.phrases.into_vec().into_iter().filter(|p| p.id != P::Activity.id()).collect();
    assert_eq!(
        Phrasebook::load(crate::phrases::jig::shapes(8), Box::new([language]), &phrase_limits()).err(),
        Some(Invalid::Missing { id: P::Activity.id() })
    );
    let mut language = crate::phrases::english::language();
    language.phrases.iter_mut().find(|p| p.id == P::Activity.id()).unwrap().id = P::Word.id();
    assert_eq!(
        Phrasebook::load(crate::phrases::jig::shapes(8), Box::new([language]), &phrase_limits()).err(),
        Some(Invalid::Duplicate { id: P::Word.id() })
    );
    for phrase in [P::Task, P::At] {
        let mut language = crate::phrases::english::language();
        language.phrases.iter_mut().find(|p| p.id == phrase.id()).unwrap().forms[0].segments =
            Box::new([Segment::Text(copy_of(b"missing typed hole"))]);
        assert_eq!(
            Phrasebook::load(crate::phrases::jig::shapes(8), Box::new([language]), &phrase_limits()).err(),
            Some(Invalid::Holes { id: phrase.id() })
        );
    }
}
