use super::*;

fn escalation() -> OwnerEscalation {
    let mut task = task_doc(&[ActKind::Stop]);
    task.context = None;
    task.person_revision = Some(Revision { value: 23 });
    task.taken_by = Some(u64::MAX);
    task.result = Some(TaskResult::Ending(Box::new(OwnerChipEnding::Failed(DocumentText::complete(text(
        b"supplied prior task failure",
    ))))));
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

fn maximum_card_envelope() {
    let mut row = escalation();
    let whole = vec![b'x'; usize::try_from(CLIENT.text).unwrap()];
    row.task.spec = specification(text(&whole));
    row.task.instructions = Some(DocumentText::complete(text(&whole)));
    row.task.dependencies = list(
        (0..CLIENT.rows)
            .map(|number| OwnerTaskLink::Stored { number: u64::from(number), status: Some(OwnerTaskStatus::Done) }),
    );
    row.task.references = row.task.dependencies.clone();
    row.task.inputs = row.task.dependencies.clone();
    row.task.resources = list((0..CLIENT.rows).map(|number| {
        TaskResource::Part(Part::Resource {
            connector: u16::try_from(number).unwrap(),
            path: list((0..CLIENT.rows).map(|_| Box::<[u8]>::from(whole.clone()))),
        })
    }));
    let original = row.clone();
    let (domain, id) = loaded(ObjectKey::Escalation(8), ReadValue::OwnerEscalation(Box::new(row)));
    let bound = crate::sizes::owner_escalation_nodes(&CLIENT).unwrap().checked_add(1).unwrap();
    let allocation =
        crate::Limits { nodes: bound, depth: 32, text_bytes: bound.checked_mul(1024).unwrap(), ..limits() };
    let mut builder = Builder::new(&allocation);
    let mut bindings = JigBindings::new(bound, CLIENT.text);
    builder.open(Role::Page);
    assert!(cards::card(&mut builder, &domain, id, &cards::Place::Elsewhere, &words(), &mut bindings));
    builder.close();
    let tree = builder.finish();
    super::super::cards_access::check(&tree);
    assert!(tree.len() <= bound);
    assert!(contains(&tree, &whole));
    assert_eq!(
        domain.objects().get(id).unwrap().body,
        Some(jig_client::objects::Body::OwnerEscalation(Box::new(original)))
    );
}

#[test]
fn owner_escalation_named_card_keeps_current_holder_prior_stage_and_whole_task_without_other_context() {
    maximum_card_envelope();
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
    assert!(contains(&tree, b"supplied prior task failure"));
    assert!(tree.nodes().iter().any(|node| node.role == Role::Region && node.name.as_deref() == Some(b"Project: 4")));
    assert_eq!(domain.objects().get(id).unwrap().body.as_ref().unwrap().ending(), None);
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

fn build_closed_offers(row: &OwnerEscalation, allocation: crate::Limits, w: &crate::words::Words) -> Tree {
    let (domain, id) = loaded(ObjectKey::Escalation(8), ReadValue::OwnerEscalation(Box::new(row.clone())));
    let mut builder = Builder::new(&allocation);
    let mut bindings = JigBindings::new(allocation.bindings, CLIENT.text);
    builder.open(Role::Page);
    assert!(cards::card(&mut builder, &domain, id, &cards::Place::Elsewhere, w, &mut bindings));
    builder.close();
    builder.finish()
}

fn closed_offers(refused: bool) -> OwnerEscalation {
    let kinds = [ActKind::Release, ActKind::LeaveHeld, ActKind::PassUp, ActKind::AmendRelease, ActKind::Cancel];
    let mut row = escalation();
    row.acts = List::with_capacity(MAX_OFFERED_ACTS);
    let reason = HoldReason::Tries {
        failures: list([
            AttemptFailure {
                attempt: 0,
                class: FailureClass::HostLost,
                reason: text(b"whole offered failure"),
                at: Wall::EPOCH,
            },
            AttemptFailure {
                attempt: u64::MAX,
                class: FailureClass::Permanent,
                reason: text(b"whole offered failure"),
                at: Wall::from_nanos(u64::MAX),
            },
        ]),
    };
    for _ in 0_u32..5 {
        for act in kinds {
            let admission = if refused {
                Admission::Refused(Refusal::Moved {
                    now: Revision { value: u64::MAX },
                    what: Moved::Hold { reason: Some(CurrentHold::Document(Box::new(reason.clone()))) },
                })
            } else {
                Admission::Allowed
            };
            row.acts.push(Offered { act, admission }).unwrap();
        }
    }
    row
}

fn check_exact_storage(row: &OwnerEscalation, allocation: crate::Limits, tree: &Tree, w: &crate::words::Words) {
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
    let per_node = tree.nodes().iter().map(crate::tree::Node::text_bytes).max().unwrap();
    let total = tree.nodes().iter().map(crate::tree::Node::text_bytes).sum::<u32>();
    let exact = crate::Limits { nodes: tree.len(), depth, text_per_node: per_node, text_bytes: total, ..allocation };
    assert_eq!(build_closed_offers(row, exact, w).len(), tree.len());
    for below in [
        crate::Limits { nodes: exact.nodes - 1, ..exact },
        crate::Limits { depth: exact.depth - 1, ..exact },
        crate::Limits { text_per_node: exact.text_per_node - 1, ..exact },
        crate::Limits { text_bytes: exact.text_bytes - 1, ..exact },
    ] {
        std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| build_closed_offers(row, below, w)))
            .expect_err("one-under storage refuses the whole escalation tree");
    }
}

#[test]
fn owner_escalation_closed_outer_controls_and_whole_refusals_fit_finite_view_and_refuse_one_under_storage() {
    let w = words();
    for refused in [false, true] {
        let row = closed_offers(refused);
        let original = row.clone();
        assert_eq!(row.acts.capacity(), MAX_OFFERED_ACTS);
        assert_eq!(row.acts.len(), MAX_OFFERED_ACTS);
        assert_eq!(row.task.context, None);
        let (domain, id) = loaded(ObjectKey::Escalation(8), ReadValue::OwnerEscalation(Box::new(row.clone())));
        let mut allocation = crate::Limits::of_words(&CLIENT, &w).unwrap();
        allocation.nodes = crate::sizes::owner_escalation_nodes(&CLIENT).unwrap() + 1;
        allocation.text_bytes = allocation.nodes * allocation.text_per_node;
        let mut builder = Builder::new(&allocation);
        let mut bindings = JigBindings::new(allocation.bindings, CLIENT.text);
        builder.open(Role::Page);
        assert!(cards::card(&mut builder, &domain, id, &cards::Place::Elsewhere, &w, &mut bindings));
        builder.close();
        let tree = builder.finish();
        super::super::cards_access::check(&tree);
        assert!(tree.len() <= allocation.nodes);
        let decoded: Vec<_> = tree
            .nodes()
            .iter()
            .filter_map(|node| {
                if node.role != Role::Button {
                    return None;
                }
                let binding = node.binding?;
                match decode(&mut bindings, binding.token, Act::Press) {
                    Some(Action::Intend { object, act }) => {
                        assert_eq!(object, id.token());
                        Some(act)
                    }
                    _ => None,
                }
            })
            .collect();
        if refused {
            assert!(decoded.is_empty());
            assert!(contains(&tree, b"whole offered failure"));
            assert!(
                tree.nodes()
                    .iter()
                    .filter(|node| node.text.iter().any(|run| skein_lib::bytes::find(
                        &run.text,
                        b"whole offered failure"
                    )
                    .is_some()))
                    .count()
                    >= usize::try_from(MAX_OFFERED_ACTS).unwrap()
            );
        } else {
            assert_eq!(decoded, row.acts.iter().map(|offer| offer.act).collect::<Vec<_>>());
        }
        assert!(contains(&tree, b"whole task specification"));
        assert!(contains(&tree, b"supplied prior task failure"));
        assert_eq!(domain.objects().get(id).unwrap().body.as_ref().unwrap().ending(), None);
        check_exact_storage(&row, allocation, &tree, &w);
        assert_eq!(row, original);
        assert_eq!(
            domain.objects().get(id).unwrap().body,
            Some(jig_client::objects::Body::OwnerEscalation(Box::new(original)))
        );
    }
    let mut oversized = escalation();
    oversized.acts = List::with_capacity(MAX_OFFERED_ACTS + 1);
    let original = oversized.clone();
    let (domain, id) = loaded(ObjectKey::Escalation(8), ReadValue::OwnerEscalation(Box::new(oversized)));
    let allocation = crate::Limits::of_words(&CLIENT, &w).unwrap();
    let mut builder = Builder::new(&allocation);
    let mut bindings = JigBindings::new(allocation.bindings, CLIENT.text);
    builder.open(Role::Page);
    assert!(!cards::card(&mut builder, &domain, id, &cards::Place::Elsewhere, &w, &mut bindings));
    builder.close();
    assert_eq!(builder.finish().len(), 1);
    assert_eq!(
        domain.objects().get(id).unwrap().body,
        Some(jig_client::objects::Body::OwnerEscalation(Box::new(original)))
    );
    assert_eq!(crate::sizes::owner_escalation_nodes(&jig_client::Limits { rows: u32::MAX, ..CLIENT }), None);
}
