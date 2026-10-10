use crate::phrases::{Hole, HoleKind, Invalid, Language, PhraseLimits, Phrasebook, Segment};
use crate::{Builder, Limits, Role, Tree, owner_outcome as view, owner_outcome_words as phrases, words::Words};
use jig_client::contract::{
    OwnerEscalationChoice, OwnerOutcome, OwnerProposalChoice, ProposalKey, Proposer, Revision, Seen,
};
use skein_lib::{Wall, bytes::copy_of};

fn phrase_limits() -> PhraseLimits {
    PhraseLimits { languages: 2, phrases: 50, holes: 1, segments: 200, text_bytes: 8192, hole_bytes: 0 }
}
fn words() -> Words {
    Words {
        book: Phrasebook::load(phrases::shapes(), Box::new([phrases::english()]), &phrase_limits()).unwrap(),
        language: 0,
        offset_minutes: 0,
    }
}
fn render(value: &OwnerOutcome, words: &Words) -> Tree {
    let required = view::required(words).expect("closed fragment fits");
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
    };
    let mut b = Builder::new(&limits);
    view::render(&mut b, value, words);
    let tree = b.finish();
    assert!(tree.len() <= required.nodes);
    let mut bytes = 0_usize;
    for node in tree.nodes() {
        if let Some(name) = &node.name {
            bytes = bytes.checked_add(name.len()).unwrap();
        }
        for run in &node.text {
            bytes = bytes.checked_add(run.text.len()).unwrap();
        }
    }
    assert!(bytes <= usize::try_from(required.text_bytes).unwrap());
    tree
}
fn has(tree: &Tree, wanted: &[u8]) -> bool {
    for node in tree.nodes() {
        for run in &node.text {
            if run.text.as_ref() == wanted {
                return true;
            }
        }
    }
    false
}
fn expect_lines(tree: &Tree, lines: &[&[u8]]) {
    for line in lines {
        assert!(has(tree, line), "supplied fact is present");
    }
    let mut text_nodes = 0_usize;
    for node in tree.nodes() {
        if node.role == Role::Text {
            text_nodes = text_nodes.checked_add(1).unwrap();
        }
    }
    assert_eq!(text_nodes, lines.len());
}
#[expect(clippy::too_many_lines, reason = "closed fixture lists every owner variant and independent expected facts")]
fn max_cases() -> [(OwnerOutcome, &'static [&'static [u8]]); 26] {
    [
        (
            OwnerOutcome::Closing { chat: u64::MAX, cancelled: u32::MAX, kept: u32::MAX },
            &[
                b"Chat closed",
                b"Chat: 18446744073709551615",
                b"Cancelled delegates: 4294967295",
                b"Kept delegates: 4294967295",
            ],
        ),
        (
            OwnerOutcome::MarkedSeen { seen: Seen::Results { through: u64::MAX } },
            &[b"Read position marked seen", b"Results seen through: 18446744073709551615"],
        ),
        (
            OwnerOutcome::MarkedSeen { seen: Seen::Replies { chat: u64::MAX, through: u64::MAX } },
            &[
                b"Read position marked seen",
                b"Chat: 18446744073709551615",
                b"Replies seen through: 18446744073709551615",
            ],
        ),
        (OwnerOutcome::NoteEdited { name: u64::MAX }, &[b"Note edited", b"Note: 18446744073709551615"]),
        (OwnerOutcome::ServiceMade { person: u64::MAX }, &[b"Service made", b"Person: 18446744073709551615"]),
        (OwnerOutcome::Adopted { project: u32::MAX }, &[b"Project adopted", b"Project: 4294967295"]),
        (OwnerOutcome::GoalStarted { task: u64::MAX }, &[b"Goal started", b"Task: 18446744073709551615"]),
        (
            OwnerOutcome::GoalProposed { proposal: u64::MAX },
            &[b"Goal proposed", b"Proposal number: 18446744073709551615"],
        ),
        (OwnerOutcome::Stopped { task: u64::MAX }, &[b"Task stopped", b"Task: 18446744073709551615"]),
        (OwnerOutcome::Cancelled { task: u64::MAX }, &[b"Task cancelled", b"Task: 18446744073709551615"]),
        (OwnerOutcome::Released { task: u64::MAX }, &[b"Task released", b"Task: 18446744073709551615"]),
        (OwnerOutcome::PersonTaken { task: u64::MAX }, &[b"Person task taken", b"Task: 18446744073709551615"]),
        (
            OwnerOutcome::PersonHandedBack { task: u64::MAX },
            &[b"Person task handed back", b"Task: 18446744073709551615"],
        ),
        (OwnerOutcome::PersonAnswered { task: u64::MAX }, &[b"Person task answered", b"Task: 18446744073709551615"]),
        (OwnerOutcome::Moved { task: u64::MAX }, &[b"Task moved", b"Task: 18446744073709551615"]),
        (
            OwnerOutcome::ProposalDecided {
                proposal: ProposalKey { proposer: Proposer::Task(u64::MAX), number: u64::MAX },
                by: u64::MAX,
                at: Wall::from_nanos(u64::MAX),
                choice: OwnerProposalChoice::Accepted,
            },
            &[
                b"Proposal decided",
                b"Proposing task: 18446744073709551615",
                b"Proposal number: 18446744073709551615",
                b"Decided by person: 18446744073709551615",
                b"Decided at: 18446744073709551615",
                b"Accepted",
            ],
        ),
        (
            OwnerOutcome::Said { task: u64::MAX, message: u64::MAX, released: true },
            &[b"Words admitted", b"Task: 18446744073709551615", b"Message: 18446744073709551615", b"Released: yes"],
        ),
        (
            OwnerOutcome::QuestionAnswered { task: u64::MAX, question: u64::MAX, message: u64::MAX },
            &[
                b"Question answered",
                b"Task: 18446744073709551615",
                b"Question: 18446744073709551615",
                b"Message: 18446744073709551615",
            ],
        ),
        (OwnerOutcome::Prioritised { project: u32::MAX }, &[b"Goal priorities changed", b"Project: 4294967295"]),
        (
            OwnerOutcome::Amended { task: u64::MAX, released: true },
            &[b"Task amended", b"Task: 18446744073709551615", b"Released: yes"],
        ),
        (
            OwnerOutcome::AmendProposed { task: u64::MAX, proposal: u64::MAX, released: true },
            &[
                b"Amendment proposed",
                b"Task: 18446744073709551615",
                b"Proposal number: 18446744073709551615",
                b"Released: yes",
            ],
        ),
        (OwnerOutcome::RolesSet { project: u32::MAX }, &[b"Project roles set", b"Project: 4294967295"]),
        (OwnerOutcome::PolicyChanged { project: u32::MAX }, &[b"Project policy changed", b"Project: 4294967295"]),
        (
            OwnerOutcome::PoolSet { project: u32::MAX, person: u64::MAX },
            &[b"Person pool set", b"Project: 4294967295", b"Person: 18446744073709551615"],
        ),
        (
            OwnerOutcome::EscalationDecided {
                task: u64::MAX,
                revision: Revision { value: u64::MAX },
                by: u64::MAX,
                at: Wall::from_nanos(u64::MAX),
                choice: OwnerEscalationChoice::Released,
            },
            &[
                b"Escalation decided",
                b"Task: 18446744073709551615",
                b"Hold revision: 18446744073709551615",
                b"Decided by person: 18446744073709551615",
                b"Decided at: 18446744073709551615",
                b"Released",
            ],
        ),
        (OwnerOutcome::Started { task: u64::MAX }, &[b"Chat started", b"Task: 18446744073709551615"]),
    ]
}
#[test]
fn all_committed_owner_outcomes_render_supplied_maximum_numbers() {
    let words = words();
    for (value, lines) in max_cases() {
        let tree = render(&value, &words);
        expect_lines(&tree, lines);
    }
}
#[test]
fn proposal_decisions_keep_both_typed_makers_and_every_closed_choice_without_explanation() {
    let words = words();
    for proposer in [Proposer::Task(9), Proposer::Party(9)] {
        for (choice, wanted) in [
            (OwnerProposalChoice::Accepted, b"Accepted".as_slice()),
            (OwnerProposalChoice::Rejected, b"Rejected".as_slice()),
            (OwnerProposalChoice::Passed, b"Passed up".as_slice()),
            (OwnerProposalChoice::Withdrawn, b"Withdrawn".as_slice()),
            (OwnerProposalChoice::Stale, b"No longer applies".as_slice()),
        ] {
            let value = OwnerOutcome::ProposalDecided {
                proposal: ProposalKey { proposer, number: 17 },
                by: 23,
                at: Wall::from_nanos(31),
                choice,
            };
            let tree = render(&value, &words);
            let maker: &[u8] = match proposer {
                Proposer::Task(_) => b"Proposing task: 9",
                Proposer::Party(_) => b"Proposing party: 9",
            };
            expect_lines(
                &tree,
                &[
                    b"Proposal decided",
                    maker,
                    b"Proposal number: 17",
                    b"Decided by person: 23",
                    b"Decided at: 31",
                    wanted,
                ],
            );
            assert_eq!(
                tree.len(),
                view::required(&words).unwrap().nodes,
                "the longest outcome attains the finite node bound"
            );
        }
    }
}
#[test]
fn escalation_decisions_keep_revision_winner_clock_and_every_closed_choice() {
    let words = words();
    for (choice, wanted) in [
        (OwnerEscalationChoice::Released, b"Released".as_slice()),
        (OwnerEscalationChoice::Rejected, b"Rejected".as_slice()),
        (OwnerEscalationChoice::Passed, b"Passed up".as_slice()),
    ] {
        let value = OwnerOutcome::EscalationDecided {
            task: 7,
            revision: Revision { value: 13 },
            by: 19,
            at: Wall::from_nanos(29),
            choice,
        };
        let tree = render(&value, &words);
        expect_lines(
            &tree,
            &[
                b"Escalation decided",
                b"Task: 7",
                b"Hold revision: 13",
                b"Decided by person: 19",
                b"Decided at: 29",
                wanted,
            ],
        );
        assert_eq!(tree.len(), view::required(&words).unwrap().nodes);
    }
}
#[test]
fn released_flags_and_question_message_identities_remain_independent() {
    let words = words();
    for released in [false, true] {
        let wanted: &[u8] = if released { b"Released: yes" } else { b"Released: no" };
        let cases: [(OwnerOutcome, &[&[u8]]); 3] = [
            (
                OwnerOutcome::Said { task: 7, message: 11, released },
                &[b"Words admitted", b"Task: 7", b"Message: 11", wanted],
            ),
            (OwnerOutcome::Amended { task: 13, released }, &[b"Task amended", b"Task: 13", wanted]),
            (
                OwnerOutcome::AmendProposed { task: 17, proposal: 23, released },
                &[b"Amendment proposed", b"Task: 17", b"Proposal number: 23", wanted],
            ),
        ];
        for (value, expected) in cases {
            expect_lines(&render(&value, &words), expected);
        }
    }
    let question = render(&OwnerOutcome::QuestionAnswered { task: 7, question: 11, message: 19 }, &words);
    expect_lines(&question, &[b"Question answered", b"Task: 7", b"Question: 11", b"Message: 19"]);
    let seen = render(&OwnerOutcome::MarkedSeen { seen: Seen::Replies { chat: 13, through: 17 } }, &words);
    expect_lines(&seen, &[b"Read position marked seen", b"Chat: 13", b"Replies seen through: 17"]);
    let proposed = render(&OwnerOutcome::GoalProposed { proposal: 19 }, &words);
    expect_lines(&proposed, &[b"Goal proposed", b"Proposal number: 19"]);
}
#[test]
fn closing_counts_and_pool_person_remain_distinct_from_chat_and_project() {
    let words = words();
    let closing = render(&OwnerOutcome::Closing { chat: 7, cancelled: 11, kept: 19 }, &words);
    expect_lines(&closing, &[b"Chat closed", b"Chat: 7", b"Cancelled delegates: 11", b"Kept delegates: 19"]);
    let pool = render(&OwnerOutcome::PoolSet { project: 13, person: 23 }, &words);
    expect_lines(&pool, &[b"Person pool set", b"Project: 13", b"Person: 23"]);
}
fn selected_language() -> Language {
    let mut language = phrases::english();
    for phrase in &mut language.phrases {
        for form in &mut phrase.forms {
            for segment in &mut form.segments {
                match segment {
                    Segment::Text(bytes) => *bytes = copy_of(b"selected committed outcome language: "),
                    Segment::Hole { .. } => {}
                }
            }
        }
    }
    language
}
#[test]
fn selected_outcome_language_is_checked_and_maximum_fragment_fits() {
    let book =
        Phrasebook::load(phrases::shapes(), Box::new([phrases::english(), selected_language()]), &phrase_limits())
            .unwrap();
    let words = Words { book, language: 1, offset_minutes: 1 };
    let value = OwnerOutcome::ProposalDecided {
        proposal: ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX },
        by: u64::MAX,
        at: Wall::EPOCH,
        choice: OwnerProposalChoice::Rejected,
    };
    let tree = render(&value, &words);
    assert_eq!(tree.len(), view::required(&words).unwrap().nodes);
    assert!(has(&tree, b"selected committed outcome language: 18446744073709551615"));
    assert!(has(&tree, b"selected committed outcome language: 60000000000"));
    assert!(!has(&tree, b"Rejected"));
    assert_eq!(words.book.say(2, phrases::OutcomePhrase::Rejected.id(), &[]).unwrap_err(), Invalid::Language);
    assert_eq!(
        words.book.say(1, phrases::OutcomePhrase::At.id(), &[Hole::Number(7)]).unwrap_err(),
        Invalid::Values { id: phrases::OutcomePhrase::At.id() }
    );
    let mut missing = phrases::english();
    missing.phrases = Box::new([]);
    assert_eq!(
        Phrasebook::load(phrases::shapes(), Box::new([missing]), &phrase_limits()).unwrap_err(),
        Invalid::Missing { id: phrases::OutcomePhrase::Closing.id() }
    );
    let mut wrong = phrases::english();
    for phrase in &mut wrong.phrases {
        if phrase.id == phrases::OutcomePhrase::Revision.id() {
            phrase.forms[0].segments[1] = Segment::Hole { index: 0, kind: HoleKind::Wall };
        }
    }
    assert_eq!(
        Phrasebook::load(phrases::shapes(), Box::new([wrong]), &phrase_limits()).unwrap_err(),
        Invalid::Holes { id: phrases::OutcomePhrase::Revision.id() }
    );
}

#[test]
fn normal_words_register_every_outcome_once_and_bound_the_selected_language() {
    use crate::phrases::{english, jig};
    use skein_lib::List;

    let normal_shapes = jig::shapes(super::cards::CLIENT.text);
    assert_eq!(normal_shapes.len(), usize::try_from(crate::owner_words::phrase_count()).unwrap());
    for outcome in phrases::ALL {
        let mut copies = 0_u32;
        for shape in &normal_shapes {
            if shape.id == outcome.id() {
                copies = copies.checked_add(1).unwrap();
            }
        }
        assert_eq!(copies, 1, "each owner outcome shape is registered once");
    }
    let limits = PhraseLimits {
        languages: 2,
        phrases: crate::owner_words::phrase_count(),
        holes: 3,
        segments: 5000,
        text_bytes: 200_000,
        hole_bytes: super::cards::CLIENT.text,
    };
    let mut selected = english::language();
    for phrase in &mut selected.phrases {
        if phrase.id == phrases::OutcomePhrase::Rejected.id() {
            phrase.forms[0].segments[0] = Segment::Text(Box::new([b'R'; 2048]));
        }
    }
    let book = Phrasebook::load(normal_shapes, Box::new([english::language(), selected]), &limits).unwrap();
    for outcome in phrases::ALL {
        assert!(book.capacity(outcome.id()).is_some());
    }
    let words = Words { book, language: 1, offset_minutes: 1 };
    let fragment = view::required(&words).unwrap();
    assert!(fragment.text_per_node >= 2048);
    assert!(Limits::of_words(&super::cards::CLIENT, &words).unwrap().text_per_node >= 2048);
    let value = OwnerOutcome::ProposalDecided {
        proposal: ProposalKey { proposer: Proposer::Party(u64::MAX), number: 17 },
        by: 23,
        at: Wall::EPOCH,
        choice: OwnerProposalChoice::Rejected,
    };
    let tree = render(&value, &words);
    assert_eq!(tree.len(), fragment.nodes);
    assert!(has(&tree, &[b'R'; 2048]));
    assert!(has(&tree, b"Proposing party: 18446744073709551615"));
    assert!(has(&tree, b"Proposal number: 17"));
    assert!(has(&tree, b"Decided by person: 23"));
    assert!(!has(&tree, b"Rejected"));

    let mut incomplete = english::language();
    let mut kept = List::with_capacity(crate::owner_words::phrase_count());
    for phrase in incomplete.phrases {
        if phrase.id != phrases::OutcomePhrase::ReleasedYes.id() {
            kept.push(phrase).unwrap();
        }
    }
    incomplete.phrases = kept.into_boxed();
    assert_eq!(
        Phrasebook::load(jig::shapes(super::cards::CLIENT.text), Box::new([incomplete]), &limits).unwrap_err(),
        Invalid::Missing { id: phrases::OutcomePhrase::ReleasedYes.id() }
    );
}
