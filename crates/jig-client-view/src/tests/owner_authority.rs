#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect standalone trees and language data")]
use crate::owner_authority as view;
use crate::owner_authority_words::{self as words, AuthorityPhrase, OwnerAuthorityWords};
use skein_ui::Limits;
use skein_ui::phrases::{Hole, Invalid, PhraseLimits, Segment, WallFormat};
use skein_ui::{Builder, Tree};

use jig_domain_client::contract::{
    OwnerAuthority, OwnerAuthorityDocument, OwnerAuthorityExecutor, OwnerDelegation, OwnerGrant, OwnerPattern,
    OwnerPatternEnd, OwnerResourceScope,
};
use skein_lib::{List, Wall, bytes::copy_of};
fn fixture() -> OwnerAuthorityDocument {
    let mut grants = List::with_capacity(1);
    let mut segments = List::with_capacity(3);
    for value in [&b"a/b"[..], &[255, 0, 128], &b""[..]] {
        segments.push(copy_of(value)).expect("segments fit");
    }
    grants
        .push(OwnerGrant {
            connector: u16::MAX,
            kind: u16::MAX,
            pattern: OwnerPattern { segments, last: OwnerPatternEnd::Exact(copy_of(b"tail/")) },
        })
        .expect("grant fits");
    let mut kinds = List::with_capacity(3);
    for kind in [
        OwnerAuthorityExecutor::Charter(u32::MAX),
        OwnerAuthorityExecutor::Procedure(u32::MAX),
        OwnerAuthorityExecutor::Role(u32::MAX),
    ] {
        kinds.push(kind).expect("executors fit");
    }
    let mut resources = List::with_capacity(1);
    resources
        .push(OwnerResourceScope {
            connector: 2,
            pattern: OwnerPattern { segments: List::with_capacity(0), last: OwnerPatternEnd::Open(copy_of(b"*raw/")) },
        })
        .expect("resource fits");
    OwnerAuthorityDocument {
        cut: true,
        authority: OwnerAuthority {
            tools: u64::MAX,
            grants,
            delegation: OwnerDelegation { kinds, tasks: u32::MAX, depth: u32::MAX },
            spend: u64::MAX,
            deadline: Some(Wall::from_nanos(120_000_000_000)),
            notes: u8::MAX,
            note_resources: resources,
        },
    }
}
fn phrase_limits() -> PhraseLimits {
    PhraseLimits { languages: 2, phrases: 18, holes: 1, segments: 100, text_bytes: 4096, hole_bytes: 0 }
}
fn english() -> OwnerAuthorityWords {
    OwnerAuthorityWords::load(Box::new([words::english()]), &phrase_limits(), 0, 0).expect("checked English")
}
fn render(value: &OwnerAuthorityDocument, words: &OwnerAuthorityWords) -> Tree {
    let required = view::document_required(value, words).expect("capacity fits");
    let limits = Limits {
        nodes: required.nodes,
        depth: required.depth,
        text_per_node: required.text_per_node,
        text_bytes: required.text_bytes,
        drawing_bytes: 0,
        styles_per_node: 0,
        patches: 0,
        bindings: 0,
        slots: 0,
        markdown_depth: 0,
        scene_marks: 0,
        scene_bytes: 0,
    };
    let mut builder = Builder::new(&limits);
    view::document(&mut builder, value, words);
    let tree = builder.finish();
    assert_eq!(tree.len(), required.nodes);
    assert!(
        tree.nodes().iter().flat_map(|n| &n.text).map(|t| t.text.len()).sum::<usize>()
            <= usize::try_from(required.text_bytes).expect("bytes")
    );
    tree
}
fn has(tree: &Tree, bytes: &[u8]) -> bool {
    tree.nodes().iter().any(|node| node.text.iter().any(|run| run.text.as_ref() == bytes))
}
#[test]
fn exact_authority_fragment_keeps_opaque_boundaries_full_codes_and_explicit_cut() {
    let value = fixture();
    let words = english();
    let tree = render(&value, &words);
    assert_eq!(tree.len(), 26, "one region and every independent fact/segment/end leaf");
    for text in [
        &b"a/b"[..],
        &b"\\xFF\\x00\\x80"[..],
        &b"tail/"[..],
        &b"*raw/"[..],
        &b"Exact ending"[..],
        &b"Open ending"[..],
        &b"Role: 4294967295"[..],
        &b"Charter: 4294967295"[..],
        &b"Procedure: 4294967295"[..],
        &b"Tool bits: 18446744073709551615"[..],
        &b"Note scope bits: 255"[..],
        &b"Authority document is cut"[..],
    ] {
        assert!(has(&tree, text), "exact independent fact retained");
    }
    assert_eq!(value.authority.grants.get(0).unwrap().pattern.segments.get(1).unwrap().as_ref(), &[255, 0, 128]);
    let mut complete = value;
    complete.cut = false;
    let tree = render(&complete, &words);
    assert!(!has(&tree, b"Authority document is cut"));
    assert_eq!(tree.len(), 25);
    let envelope = view::worst_case(3, 5, &words).expect("configured envelope");
    assert!(tree.len() <= envelope.nodes);
    assert_eq!(view::worst_case(u32::MAX, u32::MAX, &words), None);
}
#[test]
fn owner_authority_words_use_selected_language_and_person_offset() {
    let mut translated = words::english();
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
    let words = OwnerAuthorityWords::load(Box::new([words::english(), translated]), &phrase_limits(), 1, -1)
        .expect("all selected words checked");
    let tree = render(&fixture(), &words);
    assert!(has(&tree, b"selected 00:01"));
    assert!(has(&tree, b"selected 4294967295"));
    assert!(!has(&tree, b"Exact ending"));
    assert_eq!(
        words.say(AuthorityPhrase::Deadline, &[Hole::Wall(words.adjusted_wall(Wall::EPOCH))]).as_ref(),
        b"selected 00:00"
    );
}
#[test]
fn authority_phrasebook_rejects_missing_duplicate_wrong_holes_and_unknown_language() {
    let mut language = words::english();
    language.phrases[0].id = AuthorityPhrase::Tasks.id();
    assert_eq!(
        OwnerAuthorityWords::load(Box::new([language]), &phrase_limits(), 0, 0).expect_err("duplicate refused"),
        Invalid::Duplicate { id: AuthorityPhrase::Tasks.id() }
    );
    let mut language = words::english();
    language.phrases = language.phrases.into_vec().into_iter().skip(1).collect();
    assert_eq!(
        OwnerAuthorityWords::load(Box::new([language]), &phrase_limits(), 0, 0).expect_err("missing refused"),
        Invalid::Missing { id: AuthorityPhrase::Tools.id() }
    );
    let mut language = words::english();
    language.phrases[0].forms[0].segments = Box::new([Segment::Text(copy_of(b"no typed hole"))]);
    assert_eq!(
        OwnerAuthorityWords::load(Box::new([language]), &phrase_limits(), 0, 0).expect_err("wrong holes refused"),
        Invalid::Holes { id: AuthorityPhrase::Tools.id() }
    );
    assert_eq!(
        OwnerAuthorityWords::load(Box::new([words::english()]), &phrase_limits(), 1, 0).expect_err("language refused"),
        Invalid::Language
    );
}

#[test]
fn authority_words_reject_unrepresentable_utc_offsets_before_retaining_the_book() {
    for offset in [i32::MIN, i32::MAX] {
        assert_eq!(
            OwnerAuthorityWords::load(Box::new([words::english()]), &phrase_limits(), 0, offset)
                .expect_err("offset arithmetic cannot overflow after loading"),
            Invalid::Overflow
        );
    }
    let positive =
        OwnerAuthorityWords::load(Box::new([words::english()]), &phrase_limits(), 0, 60).expect("person offset fits");
    assert_eq!(positive.adjusted_wall(Wall::from_nanos(u64::MAX)), Wall::from_nanos(u64::MAX));
}
