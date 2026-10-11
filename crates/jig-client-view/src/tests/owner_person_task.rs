#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary proofs inspect public trees, source data and checked language selection"
)]
use super::cards::{CLIENT, drive, words};
use crate::{
    Builder, JigBindings, Limits, Role, State, Tree, owner_person_task as view,
    owner_person_task_words::{self, PersonPhrase},
    phrases::{Invalid, PhraseLimits, Phrasebook, Segment},
    words::Words,
};
use jig_client::{Domain, Event, action::Act, contract::*, fields::Purpose};
use skein_lib::{Duration, List, Wall, bytes::copy_of};
fn rows<T>(values: impl IntoIterator<Item = T>) -> List<T> {
    let values: Vec<_> = values.into_iter().collect();
    let mut rows = List::with_capacity(u32::try_from(values.len()).unwrap());
    for value in values {
        rows.push(value).ok().unwrap();
    }
    rows
}
fn doc(bytes: &[u8]) -> DocumentText {
    DocumentText { bytes: Text::new(copy_of(bytes), CLIENT.text).unwrap(), cut: true }
}
fn chip(task: u64) -> Chip {
    Chip {
        task,
        title: doc(b"source title"),
        executor: Executor::Role { project: u32::MAX, role: u32::MAX },
        phase: Phase::Active,
        details: Box::new(ChipPhase::Owner {
            phase: OwnerChipPhase::Person { address: PersonAddress::Role(u32::MAX), taken_by: None, since: None },
            details: Box::new(super::owner_chip::details()),
        }),
    }
}
fn result(task: u64) -> Result {
    Result {
        task: chip(task),
        ending: Ending::Done,
        value: Some(ResultValue::OwnerVerdict {
            verdict: u32::MAX,
            words: doc(b"result explanation"),
            fields: rows([ResultField { name: copy_of(b"field/a"), value: copy_of(&[255, 0, 47]) }]),
        }),
        at: None,
        details: Some(Box::new(ResultDetails {
            contract: TaskContractDocument::Report { words: 3 },
            requester: OwnerRequester::Person(u64::MAX),
            executor: OwnerExecutor::Person(PersonAddress::Role(u32::MAX)),
            chosen_by: None,
            delegates: List::with_capacity(3),
            followups: Some(List::with_capacity(3)),
            proposal: None,
            cut: true,
        })),
        acts: List::with_capacity(3),
    }
}
fn value() -> OwnerPersonTask {
    OwnerPersonTask {
        task: chip(8),
        revision: Revision { value: u64::MAX },
        requester: OwnerRequester::Deployment { project: u32::MAX },
        spec: SpecificationDocument {
            specification: DocumentSpecification {
                words: doc(b"question bytes"),
                parameters: rows([ParameterDocument::Bytes { name: u32::MAX, value: doc(&[255, 0, 47]) }]),
                inputs: rows([u64::MAX]),
            },
            cut: true,
        },
        address: PersonAddress::Role(u32::MAX),
        taken_by: None,
        since: None,
        stall: Some(Duration::from_nanos(u64::MAX)),
        deadline: Some(Wall::EPOCH),
        contract: TaskContractDocument::Report { words: 3 },
        results: List::with_capacity(4),
        cut: true,
        acts: rows([
            Offered { act: ActKind::Answer, admission: Admission::Allowed },
            Offered { act: ActKind::Take, admission: Admission::Allowed },
            Offered { act: ActKind::HandBack, admission: Admission::Allowed },
        ]),
    }
}
fn client_limits() -> jig_client::Limits {
    jig_client::Limits { rows: 8, ..CLIENT }
}
fn limits(words: &Words) -> Limits {
    let f = view::worst_case(&client_limits(), words).unwrap();
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
        markdown_depth: 3,
    }
}
fn build(value: &OwnerPersonTask, words: &Words, cap: Limits) -> Tree {
    let d = Domain::new(&client_limits(), 1);
    let mut b = Builder::new(&cap);
    let mut bindings = JigBindings::new(cap.bindings, CLIENT.text);
    view::render(&mut b, &d, value, words, &mut bindings);
    b.finish()
}
fn contains(tree: &Tree, text: &[u8]) -> bool {
    tree.nodes().iter().any(|node| node.text.iter().any(|run| run.text.as_ref() == text))
}
#[test]
fn person_owner_view_retains_ordered_missing_links_full_results_and_independent_loss() {
    let mut value = value();
    value.results = rows([
        OwnerShownResult { task: OwnerTaskLink::Live(chip(101)), result: None },
        OwnerShownResult { task: OwnerTaskLink::Retained { chip: chip(102), at: None }, result: None },
        OwnerShownResult {
            task: OwnerTaskLink::Retained { chip: chip(103), at: Some(Wall::EPOCH) },
            result: Some(Box::new(result(103))),
        },
        OwnerShownResult { task: OwnerTaskLink::Stored { number: 104, status: None }, result: None },
        OwnerShownResult {
            task: OwnerTaskLink::Stored { number: 105, status: Some(OwnerTaskStatus::Cancelled) },
            result: Some(Box::new(result(105))),
        },
    ]);
    let unchanged = value.clone();
    let w = words();
    let tree = build(&value, &w, limits(&w));
    for text in [
        b"Result not supplied".as_slice(),
        b"Ended time unavailable",
        b"Stored task 104",
        b"Stored status unavailable",
        b"Cancelled",
        b"Some shown results were omitted",
        b"Some result metadata was omitted",
        b"result explanation",
        b"field/a",
        &[255, 0, 47],
    ] {
        assert!(contains(&tree, text), "missing {text:?}");
    }
    assert_eq!(
        tree.nodes()
            .iter()
            .flat_map(|node| &node.text)
            .filter(|run| run.text.as_ref() == b"Result not supplied")
            .count(),
        3
    );
    let positions =
        [b"Live task".as_slice(), b"Retained ended task", b"Stored task 104", b"Stored task 105"].map(|text| {
            tree.nodes().iter().position(|node| node.text.iter().any(|run| run.text.as_ref() == text)).unwrap()
        });
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(value, unchanged);
    assert!(view::required(&value, &client_limits(), &w).is_some());
}
#[test]
fn person_owner_card_and_confirmation_show_numeric_rules_and_disable_field_bearing_choices() {
    let mut value = value();
    value.contract = TaskContractDocument::Verdict {
        choices: rows([
            VerdictChoice { code: u32::MAX, words: 3, followups: u32::MAX, fields: List::with_capacity(2) },
            VerdictChoice {
                code: 7,
                words: 4,
                followups: 1,
                fields: rows([FieldRule { name: copy_of(b"field/a"), max: u32::MAX }]),
            },
        ]),
        cut: true,
    };
    let (mut d, id) =
        super::cards_objects::loaded(ObjectKey::PersonTask(8), ReadValue::OwnerPersonTask(Box::new(value)));
    drive(&mut d, Event::Act { act: Act::Intend { object: id.token(), act: ActKind::Answer } });
    let w = words();
    let mut cap = limits(&w);
    cap.nodes = cap.nodes.checked_add(view::form_nodes(&client_limits()).unwrap()).unwrap();
    cap.text_bytes = cap.nodes.checked_mul(cap.text_per_node).unwrap();
    cap.bindings = cap.nodes;
    cap.slots = cap.nodes;
    let mut b = Builder::new(&cap);
    let mut bindings = JigBindings::new(cap.bindings, CLIENT.text);
    crate::cards::card(&mut b, &d, id, &crate::cards::Place::Elsewhere, &w, &mut bindings);
    let tree = b.finish();
    for text in [
        b"Choice code: 4294967295".as_slice(),
        b"Choice code: 7",
        b"field/a",
        b"This choice requires fields the person answer cannot carry",
        b"Some contract choices were omitted",
    ] {
        assert!(contains(&tree, text), "missing {text:?}");
    }
    let disabled =
        tree.nodes().iter().find(|n| n.role == Role::Choice && n.name.as_deref() == Some(b"Choice code: 7")).unwrap();
    assert!(disabled.states.as_slice().contains(&State::Disabled));
    assert_eq!(disabled.binding, None);
    let supported = tree
        .nodes()
        .iter()
        .find(|n| n.role == Role::Choice && n.name.as_deref() == Some(b"Choice code: 4294967295"))
        .unwrap();
    assert!(supported.binding.is_some());
    assert_eq!(d.objects().get(id).unwrap().project, Some(u32::MAX));
    drive(&mut d, Event::Act { act: Act::Choose { option: u32::MAX } });
    drive(
        &mut d,
        Event::Act {
            act: Act::Edit { field: Purpose::Reason, text: Text::new(copy_of(b"abc"), CLIENT.text).unwrap() },
        },
    );
    let mut b = Builder::new(&cap);
    crate::confirmation::confirmation(&mut b, &d, &w, &mut bindings);
    let confirmation = b.finish();
    assert!(contains(&confirmation, b"Choice code: 7"));
    assert!(contains(&confirmation, b"field/a"));
}
#[test]
fn person_owner_approval_renders_opaque_parts_without_inventing_application_scope() {
    let mut value = value();
    value.address = PersonAddress::Party(u64::MAX);
    value.taken_by = Some(u64::MAX);
    value.contract = TaskContractDocument::Approval {
        state: copy_of(&[255, 0, 47]),
        part: OwnerPart {
            connector: u16::MAX,
            kind: OwnerPartKind::Outbox,
            key: OwnerPartKey::Path(rows([copy_of(b"a/b"), copy_of(&[128, 0])])),
        },
    };
    let w = words();
    let cap = limits(&w);
    let client = Domain::new(&client_limits(), 1);
    let mut b = Builder::new(&cap);
    let mut bindings = JigBindings::new(cap.bindings, CLIENT.text);
    view::render(&mut b, &client, &value, &w, &mut bindings);
    let tree = b.finish();
    for text in [
        b"Person destination: 18446744073709551615".as_slice(),
        b"Taken by person: 18446744073709551615",
        b"\\xFF\\x00\\x2F",
        b"a/b",
        b"\\x80\\x00",
        b"Outbox reference",
    ] {
        assert!(contains(&tree, text), "missing {text:?}");
    }
    let slot = tree.nodes().iter().find_map(|node| node.slot).unwrap();
    let meaning = bindings.owner_part(slot.token).unwrap();
    assert_eq!(
        meaning,
        match &value.contract {
            TaskContractDocument::Approval { part, .. } => part,
            _ => unreachable!(),
        }
    );
}
fn full_value(l: &jig_client::Limits) -> OwnerPersonTask {
    let mut value = value();
    value.task.title = doc(&[255; 128]);
    value.spec.specification.words = doc(&[b'q'; 128]);
    value.spec.specification.parameters =
        rows((0..l.rows).map(|name| ParameterDocument::Bytes { name, value: doc(&[255; 128]) }));
    value.spec.specification.inputs = rows(0..u64::from(l.rows));
    let contract = TaskContractDocument::Verdict {
        choices: rows((0..l.rows).map(|code| VerdictChoice {
            code,
            words: u32::MAX,
            followups: u32::MAX,
            fields: rows((0..l.rows).map(|_| FieldRule { name: copy_of(&[255; 128]), max: u32::MAX })),
        })),
        cut: true,
    };
    value.contract = contract.clone();
    value.results = rows((0..l.rows).map(|number| {
        let number = u64::from(number);
        let mut result = result(number);
        result.value = Some(ResultValue::OwnerVerdict {
            verdict: u32::MAX,
            words: doc(&[b'w'; 128]),
            fields: rows((0..l.rows).map(|_| ResultField { name: copy_of(&[255; 128]), value: copy_of(&[128; 128]) })),
        });
        let details = result.details.as_mut().unwrap();
        details.contract = contract.clone();
        details.delegates = rows((0..l.rows).map(|task| OwnerTaskLink::Live(chip(u64::from(task)))));
        details.followups =
            Some(rows((0..l.rows).map(|task| OwnerTaskLink::Retained { chip: chip(u64::from(task)), at: None })));
        result.acts = List::with_capacity(l.rows);
        OwnerShownResult {
            task: OwnerTaskLink::Retained { chip: chip(number), at: None },
            result: Some(Box::new(result)),
        }
    }));
    value.acts = List::with_capacity(l.rows);
    value
}
#[test]
fn person_owner_selected_words_and_all_view_storage_dimensions_refuse_one_under_whole_requirements() {
    let phrase_limits = PhraseLimits {
        languages: 2,
        phrases: crate::owner_words::phrase_count(),
        holes: 4,
        segments: 4096,
        text_bytes: 100_000,
        hole_bytes: 128,
    };
    let normal = crate::phrases::english::language();
    let mut selected = crate::phrases::english::language();
    for phrase in &mut selected.phrases {
        for form in &mut phrase.forms {
            for segment in &mut form.segments {
                if let Segment::Text(bytes) = segment {
                    *bytes = copy_of(b"selected ");
                }
            }
        }
    }
    let w = Words {
        book: Phrasebook::load(crate::phrases::jig::shapes(CLIENT.text), Box::new([normal, selected]), &phrase_limits)
            .unwrap(),
        language: 1,
        offset_minutes: 0,
    };
    let l = client_limits();
    let value = full_value(&l);
    let untouched = value.clone();
    assert!(view::required(&value, &l, &w).is_some());
    let maximum = limits(&w);
    let tree = build(&value, &w, maximum);
    assert!(contains(&tree, b"selected 4294967295"));
    assert!(!contains(&tree, b"Taker not supplied"));
    let nodes = tree.len();
    let depth = u32::try_from(
        tree.nodes()
            .iter()
            .enumerate()
            .map(|(at, _)| {
                1 + tree
                    .nodes()
                    .iter()
                    .enumerate()
                    .take(at)
                    .filter(|(index, node)| index + usize::try_from(node.size).unwrap() > at)
                    .count()
            })
            .max()
            .unwrap(),
    )
    .unwrap();
    let per_node = tree
        .nodes()
        .iter()
        .map(|n| n.name.as_ref().map_or(0, |v| v.len()) + n.text.iter().map(|r| r.text.len()).sum::<usize>())
        .max()
        .unwrap();
    let total = tree
        .nodes()
        .iter()
        .map(|n| n.name.as_ref().map_or(0, |v| v.len()) + n.text.iter().map(|r| r.text.len()).sum::<usize>())
        .sum::<usize>();
    let exact = Limits {
        nodes,
        depth,
        text_per_node: u32::try_from(per_node).unwrap(),
        text_bytes: u32::try_from(total).unwrap(),
        ..maximum
    };
    assert_eq!(build(&value, &w, exact).len(), nodes);
    for below in [
        Limits { nodes: nodes - 1, ..exact },
        Limits { depth: depth - 1, ..exact },
        Limits { text_per_node: exact.text_per_node - 1, ..exact },
        Limits { text_bytes: exact.text_bytes - 1, ..exact },
    ] {
        std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| build(&value, &w, below)))
            .expect_err("one-under storage rejects the whole view");
    }
    assert_eq!(view::max_nodes(&jig_client::Limits { rows: u32::MAX, ..client_limits() }), None);
    let mut over = value.clone();
    over.results = List::with_capacity(9);
    assert!(view::required(&over, &client_limits(), &w).is_none());
    assert_eq!(value, untouched);
    let mut bad = owner_person_task_words::language();
    bad.phrases[1].forms[0].segments = Box::new([Segment::Text(copy_of(b"missing number"))]);
    assert_eq!(
        Phrasebook::load(owner_person_task_words::shapes(), Box::new([bad]), &phrase_limits).err(),
        Some(Invalid::Holes { id: PersonPhrase::Party.id() })
    );
}
