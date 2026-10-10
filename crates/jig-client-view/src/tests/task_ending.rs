use super::*;

#[test]
fn task_overview_shows_only_supplied_ending_facts_with_opaque_state_loss_and_raw_slots() {
    let mut segments = List::with_capacity(CLIENT.rows);
    segments.push(Box::from(*b"a/b")).unwrap();
    segments.push(Box::from([255_u8, 0])).unwrap();
    let part = OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Result, key: OwnerPartKey::Path(segments) };
    for result in [
        None,
        Some(ResultValue::Approved { state: Box::from([255_u8; 32]), words: None }),
        Some(ResultValue::Approved {
            state: Box::from([255_u8; 32]),
            words: Some(DocumentText { bytes: text(b""), cut: true }),
        }),
        Some(ResultValue::Change {
            kind: u16::MAX,
            part: part.clone(),
            words: DocumentText::complete(text(b"retained")),
        }),
    ] {
        let mut row = task_doc(&[]);
        row.result = Some(TaskResult::Ending(Box::new(OwnerChipEnding::Cancelled {
            reason: DocumentText { bytes: text(b"source reason"), cut: true },
            result: result.clone(),
            by: OwnerRequester::Deployment { project: u32::MAX },
            at: Wall::from_nanos(u64::MAX),
        })));
        let (tree, bindings) = task_contract::page(row.clone());
        assert!(contains(&tree, b"Cancelled with reason"));
        assert!(contains(&tree, b"source reason"));
        assert!(contains(&tree, b"Only part of this text was supplied"));
        assert!(contains(&tree, b"Cancellation requester deployment project: 4294967295"));
        assert!(contains(&tree, b"Cancelled at:"));
        assert!(!contains(&tree, b"Result time was not supplied"));
        assert!(tree.len() <= crate::task_nodes(&CLIENT, 0).unwrap());
        match result {
            Some(ResultValue::Approved { .. }) => {
                let escaped = "\\xFF".repeat(32);
                assert!(contains(&tree, escaped.as_bytes()));
            }
            Some(ResultValue::Change { .. }) => {
                let mut found = 0_u32;
                for node in tree.nodes() {
                    if let Some(slot) = node.slot {
                        assert_eq!(bindings.owner_part(slot.token), Some(&part));
                        assert_eq!(bindings.part(slot.token), None);
                        found += 1;
                    }
                }
                assert_eq!(found, 1);
                assert!(contains(&tree, b"a/b"));
                assert!(contains(&tree, b"\\xFF\\x00"));
            }
            None => assert!(!contains(&tree, b"Approval state")),
            Some(
                ResultValue::Report(_)
                | ResultValue::Words(_)
                | ResultValue::Failure(_)
                | ResultValue::Verdict { .. }
                | ResultValue::OwnerVerdict { .. }
                | ResultValue::Connector { .. }
                | ResultValue::Choice { .. },
            ) => unreachable!(),
        }
        let (domain, _) = start();
        let envelope = limits();
        let mut builder = Builder::new(&envelope);
        builder.open(Role::Page);
        let mut bindings = JigBindings::new(envelope.bindings, CLIENT.text);
        crate::task_result::render(&mut builder, &domain, row.result.as_ref().unwrap(), &words(), &mut bindings);
        builder.close();
        let tree = builder.finish();
        assert!(tree.len() <= crate::task_result::nodes(&CLIENT).unwrap());
        assert!(contains(&tree, b"source reason"));
    }
}

#[test]
fn task_result_document_keeps_its_own_time_value_and_departure_state_without_owner_conversion() {
    let result = result();
    let mut row = task_doc(&[]);
    row.result = Some(TaskResult::Document(Box::new(result.clone())));
    let (tree, _) = task_contract::page(row);
    assert!(contains(&tree, b"whole failure"));
    assert!(contains(&tree, b"whole report"));
    assert!(!contains(&tree, b"Cancellation requester deployment project:"));
    assert_eq!(result.at, Some(Wall::EPOCH));
}
