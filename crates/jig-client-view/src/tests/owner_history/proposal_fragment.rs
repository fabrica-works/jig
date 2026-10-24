//! Bare retained proposal fragments remain passive and preserve their old history position.
use super::*;
use crate::owner_history::HistoryFragment;
use skein_ui::Role;

fn limits(fragment: HistoryFragment) -> Limits {
    Limits {
        nodes: fragment.nodes,
        depth: fragment.depth,
        text_bytes: fragment.text_bytes,
        text_per_node: fragment.text_per_node,
        drawing_bytes: 0,
        styles_per_node: 2,
        patches: 0,
        bindings: fragment.bindings,
        slots: fragment.slots,
        markdown_depth: 2,
        scene_marks: 0,
        scene_bytes: 0,
    }
}
fn render(value: &OwnerProposalHistory, words: &Words, limits: &Limits) -> (Tree, JigBindings) {
    let mut bindings = JigBindings::new(limits.bindings, 8);
    let mut builder = Builder::new(limits);
    view::proposal(&mut builder, value, words, &mut bindings, client_limits().rows);
    (builder.finish(), bindings)
}
fn bare(value: &OwnerProposalHistory, words: &Words) -> (Tree, JigBindings, HistoryFragment) {
    let source = value.clone();
    let fragment = view::proposal_required(value, &client_limits(), words).unwrap();
    assert_eq!(Some(fragment.nodes), view::proposal_max_nodes(&client_limits()));
    assert_eq!(Some(fragment), view::proposal_worst_case(&client_limits(), words));
    assert_eq!(Some(fragment), view::worst_case(&client_limits(), words));
    let (tree, bindings) = render(value, words, &limits(fragment));
    assert_eq!(source, *value);
    assert!(tree.len() <= fragment.nodes);
    let mut bytes = 0_u32;
    let mut slots = 0_u32;
    for node in tree.nodes() {
        assert!(crate::tests::text_bytes(node) <= fragment.text_per_node);
        bytes = bytes.checked_add(crate::tests::text_bytes(node)).unwrap();
        if let Some(name) = &node.name {
            assert!(name.len() <= usize::try_from(fragment.text_per_node).unwrap());
            bytes = bytes.checked_add(u32::try_from(name.len()).unwrap()).unwrap();
        }
        assert!(node.binding.is_none(), "retained facts supply no current controls");
        assert_ne!(node.role, Role::Button);
        assert_ne!(node.role, Role::TextField);
        assert_ne!(node.role, Role::TextArea);
        assert_ne!(node.role, Role::Choice);
        if let Some(slot) = node.slot {
            slots = slots.checked_add(1).unwrap();
            assert_eq!(bindings.owner_part(slot.token), Some(&part()));
        }
    }
    assert!(bytes <= fragment.text_bytes);
    assert!(slots <= fragment.slots);
    assert!(slots <= fragment.bindings);
    (tree, bindings, fragment)
}
fn states(action: OwnerRetainedAction) {
    let words = normal_words();
    for state in [
        OwnerProposalHistoryState::Pending {
            holder: OwnerHistoryHolder::Policy { project: u32::MAX, kind: OwnerMessageProposalKind::Amend },
            since: Wall::EPOCH,
        },
        OwnerProposalHistoryState::Accepted { by: OwnerRequester::Task(u64::MAX) },
        OwnerProposalHistoryState::Rejected {
            by: OwnerRequester::Deployment { project: u32::MAX },
            reason: doc(UTF8_TEXTS[0], true),
        },
        OwnerProposalHistoryState::Withdrawn,
    ] {
        let value = OwnerProposalHistory { state, as_holder: false, ..proposal(action.clone()) };
        let (tree, _, _) = bare(&value, words);
        saying(&tree, words, P::Proposal, &[Hole::Number(u64::MAX)]);
        saying(&tree, words, P::Proposer, &[Hole::Number(u64::MAX)]);
        saying(&tree, words, P::NotAsHolder, &[]);
        match &value.state {
            OwnerProposalHistoryState::Pending { .. } => {
                saying(&tree, words, P::Pending, &[]);
                saying(&tree, words, P::HolderPolicy, &[Hole::Number(u64::from(u32::MAX))]);
                saying(&tree, words, P::Since, &[Hole::Wall(Wall::EPOCH)]);
            }
            OwnerProposalHistoryState::Accepted { .. } => saying(&tree, words, P::Accepted, &[]),
            OwnerProposalHistoryState::Rejected { .. } => {
                saying(&tree, words, P::Rejected, &[]);
                assert!(has(&tree, b"Only part of this text was supplied"));
            }
            OwnerProposalHistoryState::Withdrawn => saying(&tree, words, P::Withdrawn, &[]),
        }
        let history_name = words.book().say(words.language(), P::History.id(), &[]).unwrap();
        assert!(!tree.nodes().iter().any(|node| node.name.as_deref() == Some(history_name.text.as_ref())));
        let absent_clock = words.book().say(words.language(), P::AtAbsent.id(), &[]).unwrap();
        assert!(!has(&tree, &absent_clock.text));
    }
}
#[test]
fn retained_proposal_batch_fragment_keeps_full_nested_values_in_every_state_without_current_controls() {
    states(actions().into_iter().next().unwrap());
}
#[test]
fn retained_proposal_effect_fragment_keeps_full_width_raw_coordinates_in_every_state_without_scope() {
    states(actions().into_iter().nth(1).unwrap());
    let action = OwnerRetainedAction::ProcedureEffect {
        part: part(),
        authority: state().authority,
        step: u64::MAX,
        purpose: u64::MAX,
    };
    states(action.clone());
    let words = normal_words();
    let (tree, _, _) = bare(&proposal(action), words);
    saying(&tree, words, P::ProcedureStep, &[Hole::Number(u64::MAX)]);
    saying(&tree, words, P::Purpose, &[Hole::Number(u64::MAX)]);
    assert!(!has(&tree, b"Effect attempt:"));
    assert!(!has(&tree, b"Effect completion:"));
    assert!(!has(&tree, b"Effect position:"));
}
#[test]
fn retained_proposal_amend_fragment_keeps_boxed_optional_replacements_in_every_state() {
    states(actions().into_iter().nth(2).unwrap());
}
#[test]
fn retained_proposal_widen_fragment_keeps_whole_authority_in_every_state() {
    states(actions().into_iter().nth(3).unwrap());
}
#[test]
fn retained_proposal_release_fragment_keeps_named_task_in_every_state() {
    states(actions().into_iter().nth(4).unwrap());
}
#[test]
fn retained_proposal_fragment_delegation_preserves_old_history_nodes_metadata_geometry_and_slots() {
    let words = normal_words();
    let old_context =
        OwnerHistoryLine { amendment: Some((Box::new(state()), Box::new(state()))), at: Some(Wall::EPOCH), ..line() };
    let (prefix, _) = build(&old_context, words);
    let original_bound = view::required(&old_context, &client_limits(), words).unwrap();
    for action in actions() {
        let value = proposal(action);
        let (fragment_tree, _, fragment) = bare(&value, words);
        let history = OwnerHistoryLine { proposal: Some(value), ..old_context.clone() };
        assert_eq!(view::required(&history, &client_limits(), words), Some(original_bound));
        assert_eq!(fragment, original_bound);
        let (whole, _) = build(&history, words);
        let mut heading = prefix.nodes()[0].clone();
        heading.size = whole.nodes()[0].size;
        assert_eq!(heading, whole.nodes()[0]);
        assert_eq!(&whole.nodes()[1..prefix.nodes().len()], &prefix.nodes()[1..]);
        assert_eq!(&whole.nodes()[prefix.nodes().len()..], fragment_tree.nodes());
        assert_eq!(whole.len(), prefix.len().checked_add(fragment_tree.len()).unwrap());
    }
}
#[test]
fn retained_proposal_bare_fragment_uses_the_selected_checked_language_for_every_retained_action() {
    let words = words(true);
    for action in actions() {
        let value = proposal(action);
        let (tree, _, _) = bare(&value, &words);
        saying(&tree, &words, P::Proposal, &[Hole::Number(u64::MAX)]);
        saying(&tree, &words, P::Proposer, &[Hole::Number(u64::MAX)]);
        saying(&tree, &words, P::AsHolder, &[]);
        saying(&tree, &words, P::Pending, &[]);
        saying(&tree, &words, P::Since, &[Hole::Wall(Wall::from_nanos(7))]);
    }
}
#[test]
fn retained_proposal_fragment_selected_language_and_exact_occupied_nodes_refuse_one_under() {
    let words = words(true);
    let value = proposal(OwnerRetainedAction::Amend { task: 0, change: Box::new(delta()) });
    let (tree, _, fragment) = bare(&value, &words);
    assert!(tree.nodes().iter().flat_map(|node| &node.text).any(|run| run.text.starts_with(b"selected history: ")));
    let exact = Limits { nodes: tree.len(), ..limits(fragment) };
    let (again, _) = render(&value, &words, &exact);
    assert_eq!(again.nodes(), tree.nodes());
    let one_under = Limits { nodes: exact.nodes.checked_sub(1).unwrap(), ..exact };
    match std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| render(&value, &words, &one_under))) {
        Err(_) => {}
        Ok(_) => panic!("one node below the occupied fragment must refuse"),
    }
}
#[test]
fn retained_proposal_fragment_validation_refuses_whole_allocated_rows_and_text_without_projection() {
    let words = normal_words();
    let limits = client_limits();
    let value = proposal(OwnerRetainedAction::Batch(List::with_capacity(limits.rows.checked_add(1).unwrap())));
    assert!(view::proposal_required(&value, &limits, words).is_none());
    let value = OwnerProposalHistory {
        state: OwnerProposalHistoryState::Rejected { by: OwnerRequester::Person(0), reason: doc(UTF8_TEXTS[1], false) },
        ..proposal(OwnerRetainedAction::Release { task: 0 })
    };
    assert!(view::proposal_required(&value, &limits, words).is_some());
    assert!(view::proposal_required(&value, &crate::Limits { text: 7, ..limits }, words).is_none());
    for (action, fits) in actions().into_iter().skip(1).take(3).zip([true, false, true]) {
        let value = proposal(action);
        assert!(view::proposal_required(&value, &limits, words).is_some());
        assert_eq!(view::proposal_required(&value, &crate::Limits { rows: 2, ..limits }, words).is_some(), fits);
    }
    assert!(view::proposal_max_nodes(&crate::Limits { rows: u32::MAX, ..limits }).is_none());
}
#[test]
fn retained_proposal_fragment_preserves_absent_empty_and_independent_retained_losses() {
    let words = normal_words();
    let absent = OwnerRetainedAmendment {
        addressed: None,
        instructions: None,
        release: None,
        spec: None,
        authority: None,
        wake: None,
        dependencies: None,
        reason: doc(b"", false),
        cut: false,
    };
    let empty = OwnerRetainedAmendment {
        instructions: Some(doc(b"", true)),
        dependencies: Some(List::with_capacity(3)),
        cut: true,
        ..absent.clone()
    };
    let complete_empty = OwnerRetainedAmendment { instructions: Some(doc(b"", false)), cut: false, ..empty.clone() };
    let local_loss = OwnerRetainedAmendment { cut: false, ..empty.clone() };
    let dependency_loss = OwnerRetainedAmendment { cut: true, ..complete_empty.clone() };
    for (change, supplied, text_cut, dependency_cut) in [
        (absent, false, false, false),
        (complete_empty, true, false, false),
        (local_loss, true, true, false),
        (dependency_loss, true, false, true),
        (empty, true, true, true),
    ] {
        let value = proposal(OwnerRetainedAction::Amend { task: 0, change: Box::new(change) });
        let (tree, _, _) = bare(&value, words);
        match &value.action {
            OwnerRetainedAction::Amend { change, .. } => {
                assert_eq!(change.instructions.is_some(), supplied);
                assert_eq!(change.dependencies.is_some(), supplied);
                if let Some(instructions) = &change.instructions {
                    assert!(instructions.bytes().is_empty());
                    assert_eq!(instructions.cut, text_cut);
                }
                assert_eq!(change.cut, dependency_cut);
                if let Some(values) = &change.dependencies {
                    assert!(values.is_empty());
                    assert_eq!(values.capacity(), 3);
                }
            }
            OwnerRetainedAction::Settings(_)
            | OwnerRetainedAction::Batch(_)
            | OwnerRetainedAction::Effect { .. }
            | OwnerRetainedAction::ProcedureEffect { .. }
            | OwnerRetainedAction::Widen { .. }
            | OwnerRetainedAction::Release { .. } => unreachable!(),
        }
        assert_eq!(has(&tree, b"Instruction replacement not supplied"), !supplied);
        assert_eq!(has(&tree, b"Instructions are empty"), supplied && !text_cut);
        assert_eq!(has(&tree, b"Dependency replacement not supplied"), !supplied);
        assert_eq!(has(&tree, b"Retained dependency replacement"), supplied);
        assert_eq!(has(&tree, b"Only part of this text was supplied"), text_cut);
        assert_eq!(has(&tree, b"Retained dependency list incomplete"), dependency_cut);
    }
}

const UTF8_TEXTS: [&[u8]; 2] = [b"~\0", &[b'~'; 8]];
