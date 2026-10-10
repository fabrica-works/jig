#![expect(
    clippy::disallowed_methods,
    reason = "ordinary tests inspect standalone trees and mutate checked language data"
)]
use crate::{
    Builder, JigBindings, Limits, Tree, owner_result as view,
    owner_result_words::{self as result_words, ResultPhrase},
    phrases::{Invalid, PhraseLimits, Phrasebook, Segment, WallFormat},
    words::Words,
};
use jig_client::{
    Domain,
    contract::{
        Chip, ChipPhase, Executor, OwnerExecutor, OwnerPart, OwnerPartKey, OwnerPartKind, OwnerRequester,
        OwnerTaskLink, OwnerTaskStatus, PersonAddress, Phase, ProposalKey, Proposer, ResultDetails,
        TaskContractDocument, Text,
    },
};
use skein_lib::{List, Wall, bytes::copy_of};

fn client_limits() -> jig_client::Limits {
    jig_client::Limits { rows: 8, ..super::cards::CLIENT }
}
fn phrase_limits() -> PhraseLimits {
    PhraseLimits {
        languages: 2,
        phrases: crate::owner_words::phrase_count(),
        holes: 3,
        segments: 4096,
        text_bytes: 100_000,
        hole_bytes: 128,
    }
}
fn words() -> Words {
    Words { book: crate::phrases::english::english(128).unwrap(), language: 0, offset_minutes: 0 }
}
fn chip() -> Chip {
    Chip {
        task: u64::MAX,
        title: Text::new(copy_of(b"supplied title"), 128).unwrap(),
        executor: Executor::Agent { charter: u32::MAX },
        phase: Phase::Due,
        details: Box::new(ChipPhase::Starting),
    }
}
fn details() -> ResultDetails {
    ResultDetails {
        contract: TaskContractDocument::Report { words: u32::MAX },
        requester: OwnerRequester::Deployment { project: u32::MAX },
        executor: OwnerExecutor::Person(PersonAddress::Role(u32::MAX)),
        chosen_by: None,
        delegates: List::with_capacity(0),
        followups: None,
        proposal: None,
        cut: false,
    }
}
fn build(value: &ResultDetails, words: &Words) -> Tree {
    let client = Domain::new(&client_limits(), 1);
    let limits = view::limits(&client_limits(), words).unwrap();
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(limits.bindings, 128);
    view::render(&mut b, &client, value, words, &mut bindings);
    b.finish()
}
fn has(tree: &Tree, text: &[u8]) -> bool {
    tree.nodes().iter().any(|node| node.text.iter().any(|run| run.text.as_ref() == text))
}
#[test]
fn result_metadata_keeps_requester_variants_full_width_chooser_and_typed_proposer() {
    let mut value = details();
    let words = words();
    for (requester, expected) in [
        (OwnerRequester::Task(u64::MAX), b"Requested by task 18446744073709551615".as_slice()),
        (OwnerRequester::Person(u64::MAX), b"Requested by person 18446744073709551615".as_slice()),
        (OwnerRequester::Deployment { project: u32::MAX }, b"Requested by deployment in project 4294967295".as_slice()),
    ] {
        value.requester = requester;
        assert!(has(&build(&value, &words), expected));
    }
    assert!(has(&build(&value, &words), b"Choice maker not supplied"));
    value.chosen_by = Some(u64::MAX);
    value.proposal = Some(ProposalKey { proposer: Proposer::Task(u64::MAX), number: u64::MAX });
    let task = build(&value, &words);
    assert!(has(&task, b"Choice made by person 18446744073709551615"));
    assert!(has(&task, b"Proposal from task 18446744073709551615, number 18446744073709551615"));
    value.proposal = Some(ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX });
    let party = build(&value, &words);
    assert!(has(&party, b"Proposal from party 18446744073709551615, number 18446744073709551615"));
    assert!(!has(&party, b"Proposal from task 18446744073709551615, number 18446744073709551615"));
    assert!(has(&party, b"Project role 4294967295"));
}
#[test]
fn result_task_links_keep_live_retained_stored_optional_times_and_all_statuses() {
    let mut value = details();
    value.delegates = List::with_capacity(7);
    for link in [
        OwnerTaskLink::Live(chip()),
        OwnerTaskLink::Retained { chip: chip(), at: None },
        OwnerTaskLink::Retained { chip: chip(), at: Some(Wall::from_nanos(120_000_000_000)) },
        OwnerTaskLink::Stored { number: u64::MAX, status: None },
        OwnerTaskLink::Stored { number: u64::MAX, status: Some(OwnerTaskStatus::Done) },
        OwnerTaskLink::Stored { number: u64::MAX, status: Some(OwnerTaskStatus::Failed) },
        OwnerTaskLink::Stored { number: u64::MAX, status: Some(OwnerTaskStatus::Cancelled) },
    ] {
        value.delegates.push(link).unwrap();
    }
    let tree = build(&value, &words());
    for expected in [
        b"Live task".as_slice(),
        b"Retained ended task",
        b"Ended time unavailable",
        b"Ended at 00:02",
        b"Stored task 18446744073709551615",
        b"Stored status unavailable",
        b"Done",
        b"Failed",
        b"Cancelled",
    ] {
        assert!(has(&tree, expected));
    }
    assert_eq!(
        tree.nodes().iter().filter(|n| n.binding.is_some()).count(),
        3,
        "stored references acquire no fabricated chip link"
    );
    assert_eq!(value.delegates.get(1), Some(&OwnerTaskLink::Retained { chip: chip(), at: None }));
}
#[test]
fn result_followups_distinguish_absent_empty_nonempty_and_preserve_aggregate_cut() {
    let mut value = details();
    let words = words();
    assert!(has(&build(&value, &words), b"Follow-up facts unavailable"));
    value.followups = Some(List::with_capacity(2));
    let empty = build(&value, &words);
    assert!(has(&empty, b"Follow-up task references: 0"));
    assert!(!has(&empty, b"Follow-up facts unavailable"));
    value.followups.as_mut().unwrap().push(OwnerTaskLink::Stored { number: 42, status: None }).unwrap();
    value.cut = true;
    let cut = build(&value, &words);
    assert!(has(&cut, b"Follow-up task references: 1"));
    assert!(has(&cut, b"Stored task 42"));
    assert!(has(&cut, b"Some result metadata was omitted"));
    assert!(!has(&empty, b"Some result metadata was omitted"));
}
#[test]
fn result_metadata_reuses_whole_contract_opaque_parts_and_owner_executor() {
    let mut value = details();
    let mut segments = List::with_capacity(2);
    segments.push(copy_of(b"a/b")).unwrap();
    segments.push(copy_of(&[255, 0])).unwrap();
    value.contract = TaskContractDocument::Approval {
        state: copy_of(&[128, 0]),
        part: OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Outbox, key: OwnerPartKey::Path(segments) },
    };
    value.executor = OwnerExecutor::Procedure { connector: u16::MAX, code: u32::MAX };
    let tree = build(&value, &words());
    for expected in [
        b"\\x80\\x00".as_slice(),
        b"a/b",
        b"\\xFF\\x00",
        b"Outbox reference",
        b"Procedure connector 65535, code 4294967295",
    ] {
        assert!(has(&tree, expected));
    }
    assert_eq!(tree.nodes().iter().filter(|n| n.slot.is_some()).count(), 1);
    value.contract = TaskContractDocument::Verdict { choices: List::with_capacity(0), cut: true };
    let cut = build(&value, &words());
    assert!(has(&cut, b"Some contract choices were omitted"));
    assert!(!has(&cut, b"Some result metadata was omitted"), "contract loss never invents aggregate loss");
}
#[test]
fn result_words_check_shapes_select_language_and_refuse_insufficient_tree_capacity() {
    let limits = phrase_limits();
    let normal = crate::phrases::english::language();
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
    let book = Phrasebook::load(crate::phrases::jig::shapes(128), Box::new([normal, translated]), &limits).unwrap();
    let words = Words { book, language: 1, offset_minutes: 0 };
    let tree = build(&details(), &words);
    assert!(has(&tree, b"selected 4294967295"));
    assert!(!has(&tree, b"Follow-up facts unavailable"));
    let mut language = result_words::language();
    language.phrases[0].id = ResultPhrase::Contract.id();
    assert_eq!(
        Phrasebook::load(result_words::shapes(), Box::new([language]), &limits).err(),
        Some(Invalid::Duplicate { id: ResultPhrase::Contract.id() })
    );
    let mut language = result_words::language();
    language.phrases[2].forms[0].segments = Box::new([Segment::Text(copy_of(b"missing hole"))]);
    assert_eq!(
        Phrasebook::load(result_words::shapes(), Box::new([language]), &limits).err(),
        Some(Invalid::Holes { id: ResultPhrase::RequesterTask.id() })
    );
    let value = details();
    let client = Domain::new(&client_limits(), 1);
    let mut capacity = view::limits(&client_limits(), &words).unwrap();
    assert_eq!(tree.len(), 9, "one region and eight independent metadata leaves");
    capacity.nodes = 9;
    capacity.depth = 2;
    capacity.text_per_node = 19;
    capacity.text_bytes = 112;
    let mut b = Builder::new(&capacity);
    let mut bindings = JigBindings::new(capacity.bindings, 128);
    view::render(&mut b, &client, &value, &words, &mut bindings);
    assert_eq!(b.finish().len(), 9, "minimal nodes, depth and selected saying bytes fit");
    capacity.nodes = capacity.nodes.checked_sub(1).unwrap();
    assert!(
        std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
            let mut b = Builder::new(&capacity);
            let mut bindings = JigBindings::new(capacity.bindings, 128);
            view::render(&mut b, &client, &value, &words, &mut bindings);
        }))
        .is_err(),
        "one fewer node refuses before a partial tree is retained"
    );
    capacity.nodes = 9;
    for below in [
        Limits { depth: 1, ..capacity },
        Limits { text_per_node: 18, ..capacity },
        Limits { text_bytes: 111, ..capacity },
    ] {
        assert!(
            std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
                let mut b = Builder::new(&below);
                let mut bindings = JigBindings::new(below.bindings, 128);
                view::render(&mut b, &client, &value, &words, &mut bindings);
            }))
            .is_err(),
            "one below a checked storage dimension refuses"
        );
    }
    assert_eq!(view::nodes(&jig_client::Limits { rows: u32::MAX, ..client_limits() }), None);
}
