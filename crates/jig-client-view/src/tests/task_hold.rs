use super::*;

#[test]
fn owner_hold_confirmation_keeps_exact_prior_counts_loss_clock_presence_and_repeat_bindings() {
    let reference = OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Hold, key: OwnerPartKey::Number(u64::MAX) };
    for uncertain in [false, true] {
        for wait in [None, Some(Wall::from_nanos(u64::MAX))] {
            for prior in [
                OwnerPriorPhase::Waiting,
                OwnerPriorPhase::Active(RunStage::Claimed { attempt: u64::MAX }),
                OwnerPriorPhase::Closing(ClosingStage::Effects),
            ] {
                let held = OwnerChipHold {
                    source: Some(reference.clone()),
                    reason: if uncertain {
                        HoldCategory::Uncertain { entry: u64::MAX }
                    } else {
                        HoldCategory::Procedure
                    },
                    revision: u64::MAX,
                    before: prior,
                    escalation: Some(OwnerHoldEscalation::Rejected {
                        by: u64::MAX,
                        reason: DocumentText { bytes: text(b"exact held rejection"), cut: true },
                    }),
                    tries: OwnerFailureTries {
                        transient: 1,
                        permanent: 2,
                        run: 3,
                        agent: 4,
                        lost: 5,
                        invalid: u32::MAX,
                    },
                    hold_wait_since: wait,
                };
                let mut row = task_doc(&[ActKind::Release]);
                row.hold = Some(TaskHold::Owner(Box::new(held.clone())));
                let (mut domain, id, _) = loaded_with_watch(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
                drive(&mut domain, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::Release } });
                let envelope = limits();
                let mut bindings = JigBindings::new(envelope.bindings, CLIENT.text);
                let tree = dialog(&domain, &words(), &mut bindings);
                assert!(contains(&tree, b"Release continues from the shown lifecycle; failure counts reset"));
                assert!(contains(&tree, b"Hold revision: 18446744073709551615"));
                assert!(contains(&tree, b"Left held by: 18446744073709551615"));
                assert!(contains(&tree, b"exact held rejection"));
                assert!(contains(&tree, b"Supplied words incomplete"));
                for caption in [
                    b"Transient tries: 1".as_slice(),
                    b"Permanent tries: 2",
                    b"Run tries: 3",
                    b"Agent tries: 4",
                    b"Lost tries: 5",
                    b"Invalid tries: 4294967295",
                ] {
                    assert!(contains(&tree, caption));
                }
                assert_eq!(contains(&tree, b"Hold wait began:"), wait.is_some());
                let expected_prior = match prior {
                    OwnerPriorPhase::Waiting => b"Prior stage waiting".as_slice(),
                    OwnerPriorPhase::Active(_) => b"Prior stage claimed attempt: 18446744073709551615",
                    OwnerPriorPhase::Closing(_) => b"Prior closing effects",
                };
                assert!(contains(&tree, expected_prior));
                let mut choices = Vec::new();
                for node in tree.nodes() {
                    if let Some(binding) = node.binding
                        && let Some(Action::Again { again }) = decode(&mut bindings, binding.token, Act::Press)
                    {
                        choices.push(again);
                    }
                    if let Some(slot) = node.slot {
                        assert_eq!(bindings.owner_part(slot.token), Some(&reference));
                        assert_eq!(bindings.part(slot.token), None);
                    }
                }
                assert_eq!(choices, if uncertain { vec![true, false] } else { vec![] });
                assert!(!contains(&tree, b"whole report"));
                let Some(jig_client::objects::Body::Task(retained)) = &domain.objects().get(id).unwrap().body else {
                    panic!("whole task")
                };
                assert_eq!(retained.hold, Some(TaskHold::Owner(Box::new(held))));
            }
        }
    }
}
