use super::*;
use crate::address::{Address, TaskSection};

pub(super) fn page(row: Task) -> (Tree, JigBindings) {
    let (mut domain, out) = super::super::work_pages::entered(Address::task(8));
    let watch =
        out.iter()
            .find_map(|request| match request {
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
    super::super::pages::deliver(&mut domain, watch, Delivery::Task(Box::new(row)));
    let built = super::super::pages::build(&domain, 3, words());
    super::super::cards_access::check(&built.0);
    assert!(built.0.len() <= crate::task_nodes(&CLIENT, 0).unwrap());
    built
}

#[test]
fn task_contract_overview_preserves_opaque_approval_and_named_verdict_caps_without_option_labels() {
    let reference = OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Result, key: OwnerPartKey::Number(u64::MAX) };
    let mut choices = List::with_capacity(2);
    let mut fields = List::with_capacity(2);
    fields.push(FieldRule { name: Box::from(*b"field\xff/"), max: u32::MAX }).unwrap();
    choices.push(VerdictChoice { code: u32::MAX, words: u32::MAX, followups: u32::MAX, fields }).unwrap();
    for contract in [
        TaskContractDocument::Report { words: u32::MAX },
        TaskContractDocument::Approval { state: Box::from(*b"\xff\0/"), part: reference.clone() },
        TaskContractDocument::Verdict { choices: choices.clone(), cut: false },
        TaskContractDocument::Verdict { choices, cut: true },
        TaskContractDocument::Change { connector: u16::MAX, kind: u16::MAX, words: u32::MAX },
    ] {
        let mut row = task_doc(&[]);
        row.spec.specification.words = DocumentText::complete(text(b""));
        row.result_contract = contract.clone();
        row.cut = false;
        let (tree, bindings) = page(row);
        assert!(contains(&tree, b"Result expected"));
        assert!(!contains(&tree, b"Some task lists are incomplete"));
        match contract {
            TaskContractDocument::Report { .. } => assert!(contains(&tree, b"Report word byte limit 4294967295")),
            TaskContractDocument::Approval { state, .. } => {
                assert_eq!(&*state, b"\xff\0/");
                assert!(contains(&tree, b"\\xFF\\x00\\x2F"));
                let slots: Vec<_> = tree.nodes().iter().filter_map(|node| node.slot).collect();
                assert_eq!(slots.len(), 1);
                assert_eq!(bindings.owner_part(slots[0].token), Some(&reference));
            }
            TaskContractDocument::Verdict { cut, .. } => {
                assert!(contains(
                    &tree,
                    b"Verdict choice 4294967295, word byte limit 4294967295, follow-up limit 4294967295"
                ));
                assert!(contains(&tree, b"\\x66\\x69\\x65\\x6C\\x64\\xFF\\x2F"));
                assert!(contains(&tree, b"Field value byte limit 4294967295"));
                assert_eq!(contains(&tree, b"Some contract choices were omitted"), cut);
            }
            TaskContractDocument::Change { .. } => {
                assert!(contains(&tree, b"Change contract 65535, kind 65535, word byte limit 4294967295"));
            }
        }
    }
}

#[test]
fn task_header_links_the_supplied_served_goal_without_turning_ancestor_identity_into_self_goal() {
    for goal in [None, Some(8), Some(0), Some(u64::MAX)] {
        let mut row = task_doc(&[]);
        row.goal = goal;
        let (tree, mut bindings) = page(row);
        let mut goals = Vec::new();
        for node in tree.nodes() {
            if let Some(binding) = node.binding
                && let Some(Action::Go { address: Address::Task { task, section: TaskSection::Overview } }) =
                    decode(&mut bindings, binding.token, Act::Press)
                && (task == 0 || task == u64::MAX)
            {
                goals.push(task);
            }
        }
        assert_eq!(goals, goal.filter(|number| *number != 8).into_iter().collect::<Vec<_>>());
        assert_eq!(contains(&tree, b"Goal"), goal == Some(8));
        assert_eq!(contains(&tree, b"Serves goal"), goal.is_some_and(|number| number != 8));
        assert!(contains(&tree, b"Priority 1"));
    }
}

#[test]
fn maximum_nested_task_contracts_fit_the_page_bound_and_preserve_every_raw_slot_and_field() {
    let raw = vec![0xff; usize::try_from(CLIENT.text).unwrap()].into_boxed_slice();
    let mut segments = List::with_capacity(CLIENT.rows);
    for _ in 0..CLIENT.rows {
        segments.push(raw.clone()).unwrap();
    }
    let reference = OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Resource, key: OwnerPartKey::Path(segments) };
    let mut fields = List::with_capacity(CLIENT.rows);
    for _ in 0..CLIENT.rows {
        fields.push(FieldRule { name: raw.clone(), max: u32::MAX }).unwrap();
    }
    let mut choices = List::with_capacity(CLIENT.rows);
    for _ in 0..CLIENT.rows {
        choices
            .push(VerdictChoice { code: u32::MAX, words: u32::MAX, followups: u32::MAX, fields: fields.clone() })
            .unwrap();
    }
    for contract in [
        TaskContractDocument::Approval { state: raw.clone(), part: reference.clone() },
        TaskContractDocument::Verdict { choices, cut: true },
    ] {
        let mut row = task_doc(&[]);
        row.goal = Some(u64::MAX);
        row.spec.specification.words = DocumentText::complete(text(b""));
        row.result_contract = contract.clone();
        let (tree, bindings) = page(row);
        let expected = "\\xFF".repeat(usize::try_from(CLIENT.text).unwrap());
        assert!(contains(&tree, expected.as_bytes()));
        match contract {
            TaskContractDocument::Approval { state, .. } => {
                assert_eq!(state, raw);
                let slots: Vec<_> = tree.nodes().iter().filter_map(|node| node.slot).collect();
                assert_eq!(slots.len(), 1);
                assert_eq!(bindings.owner_part(slots[0].token), Some(&reference));
            }
            TaskContractDocument::Verdict { choices, .. } => {
                assert_eq!(choices.len(), CLIENT.rows);
                let fields = tree
                    .nodes()
                    .iter()
                    .filter(|node| {
                        node.text
                            .iter()
                            .flat_map(|run| run.text.iter().copied())
                            .eq(b"Field value byte limit 4294967295".iter().copied())
                    })
                    .count();
                assert_eq!(fields, usize::try_from(CLIENT.rows * CLIENT.rows).unwrap());
                assert!(contains(&tree, b"Some contract choices were omitted"));
            }
            TaskContractDocument::Report { .. } | TaskContractDocument::Change { .. } => unreachable!(),
        }
    }
}

#[test]
fn refused_chat_header_shows_the_supplied_cause_and_receipt_time() {
    for (cause, expected) in [
        (TaskWaitRefused::Program(OpenProgram::Model), b"This model runs through a program".as_slice()),
        (
            TaskWaitRefused::Program(OpenProgram::Source { account: 17, source: 23 }),
            b"Account 17 would use its program source 23".as_slice(),
        ),
        (
            TaskWaitRefused::Hello {
                kind: OwnerHostKind::Worker,
                at: Wall::EPOCH,
                why: OwnerHelloRefusal::Position { worker: 7 },
            },
            b"Unconfigured or occupied worker position 7".as_slice(),
        ),
    ] {
        let mut row = task_doc(&[]);
        row.flags.conversation = true;
        row.waits_on = WaitsOn::Nothing;
        row.run_wait = Some(TaskRunWait::Refused(cause));
        row.chip.phase = Phase::Due;
        row.chip.details = Box::new(ChipPhase::Owner {
            phase: OwnerChipPhase::Active(RunStage::Due),
            details: Box::new(super::super::owner_chip::details()),
        });
        let (tree, _) = page(row);
        assert!(contains(&tree, b"Cannot start yet"));
        assert!(contains(&tree, expected));
        match cause {
            TaskWaitRefused::Hello { .. } => {
                assert!(contains(&tree, b"A worker was refused at 01:00"));
            }
            TaskWaitRefused::Program(_) => {
                assert!(contains(&tree, b"A chat can't stay open through a program yet"));
                assert!(contains(&tree, b"To chat now, choose a model that doesn't run through a program"));
            }
        }
    }
}
