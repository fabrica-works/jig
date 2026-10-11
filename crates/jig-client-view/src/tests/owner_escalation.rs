use super::*;

fn escalation() -> OwnerEscalation {
    let mut task = task_doc(&[ActKind::Stop]);
    task.context = None;
    task.person_revision = Some(Revision { value: 23 });
    task.taken_by = Some(u64::MAX);
    task.chip.executor = Executor::Agent { charter: 1 };
    let held = OwnerChipHold {
        source: None,
        reason: HoldCategory::Budget,
        revision: u64::MAX,
        before: OwnerPriorPhase::Closing(ClosingStage::Effects),
        escalation: Some(OwnerHoldEscalation::Waiting {
            holder: OwnerEscalationHolder::Person(u64::MAX),
            entry: 17,
            since: Wall::EPOCH,
        }),
        tries: OwnerFailureTries { transient: 1, permanent: 2, run: 3, agent: 4, lost: 5, invalid: 6 },
        hold_wait_since: None,
    };
    let mut details = super::super::owner_chip::details();
    details.project = task.project;
    details.hold = Some(held.clone());
    details.executor = OwnerExecutor::Agent { charter: 1 };
    task.chip.details = Box::new(ChipPhase::Owner {
        phase: OwnerChipPhase::Held { reason: HoldCategory::Budget, revision: u64::MAX },
        details: Box::new(details),
    });
    task.hold = Some(TaskHold::Owner(Box::new(held)));
    OwnerEscalation {
        task: Box::new(task),
        acts: list([
            Offered { act: ActKind::Release, admission: Admission::Allowed },
            Offered { act: ActKind::AmendRelease, admission: Admission::Allowed },
        ]),
    }
}

#[test]
fn owner_escalation_named_card_keeps_current_holder_prior_stage_and_whole_task_without_other_context() {
    let row = escalation();
    let (domain, id) = loaded(ObjectKey::Escalation(8), ReadValue::OwnerEscalation(Box::new(row.clone())));
    let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
    let tree = render(&domain, id, &words(), &mut bindings);
    assert!(contains(&tree, b"whole task specification"));
    assert!(contains(&tree, b"whole task instructions"));
    assert!(contains(&tree, b"Prior closing effects"));
    assert!(contains(&tree, b"Hold revision: 18446744073709551615"));
    assert!(contains(&tree, b"18446744073709551615"));
    assert!(contains(&tree, b"Budget: 20"));
    let names: Vec<_> = tree.nodes().iter().filter_map(|node| node.name.as_deref()).collect();
    assert!(names.contains(&b"Escalation".as_slice()));
    assert!(tree.nodes().iter().any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Release")));
    assert!(!tree.nodes().iter().any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Stop")));
    assert!(tree.len() <= crate::sizes::owner_escalation_nodes(&CLIENT).unwrap() + 1);
    assert_eq!(domain.objects().get(id).unwrap().body, Some(jig_client::objects::Body::OwnerEscalation(Box::new(row))));
}

#[test]
fn owner_escalation_amendment_inputs_keep_whole_source_baselines_and_selected_deltas_and_loss_guards() {
    for lost in [false, true] {
        let mut row = escalation();
        if lost {
            row.task.cut = true;
            row.task.spec.cut = true;
            row.task.authority.cut = true;
        }
        let original = row.clone();
        let (mut domain, id) = loaded(ObjectKey::Escalation(8), ReadValue::OwnerEscalation(Box::new(row)));
        drive(&mut domain, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::AmendRelease } });
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
        let tree = dialog(&domain, &words(), &mut bindings);
        assert!(contains(&tree, b"whole task specification"));
        assert!(contains(&tree, b"whole task instructions"));
        assert!(contains(&tree, b"Hold revision: 18446744073709551615"));
        let inputs: Vec<_> = tree
            .nodes()
            .iter()
            .filter_map(|node| node.binding.map(|binding| (node.name.as_deref(), binding.token)))
            .collect();
        let budget = inputs.iter().find(|(name, _)| *name == Some(b"New budget")).map(|(_, token)| *token);
        assert_eq!(budget.is_some(), !lost);
        if let Some(token) = budget {
            let act = decode(&mut bindings, token, Act::Input { value: Box::from(*b"9") }).unwrap();
            let Action::Change { change } = act else { panic!("selected amendment") };
            let mut expected = original.task.authority.whole().unwrap();
            expected.spend = 9;
            assert_eq!(change.authority, Some(expected));
            assert_eq!(change.spec, None);
            assert_eq!(change.wake, None);
            assert_eq!(change.dependencies, None);
            drive(&mut domain, Event::Act { act: Action::Change { change } });
            assert_eq!(domain.confirmation().unwrap().revision, Some(original.task.spec_revision));
            assert_eq!(domain.confirmation().unwrap().release_revision, Some(Revision { value: u64::MAX }));
        }
        assert_eq!(
            domain.objects().get(id).unwrap().body,
            Some(jig_client::objects::Body::OwnerEscalation(Box::new(original)))
        );
    }
}
