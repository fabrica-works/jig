use super::*;
use crate::payload::Payload;
use core::mem::size_of;
fn semantic() -> HistoryRow {
    let task = task();
    let before = OwnerAmendmentState {
        addressed: None,
        taken_by: None,
        instructions: None,
        spec: task.spec,
        authority: task.authority,
        wake: task.wake,
        dependencies: List::with_capacity(1),
        cut: false,
    };
    let mut after = before.clone();
    after.instructions = Some(DocumentText::complete(text(b"")));
    HistoryRow::Change(Box::new(OwnerHistoryLine {
        task: 8,
        revision: u64::MAX,
        by: OwnerRequester::Deployment { project: u32::MAX },
        change: OwnerHistoryChange::Amended,
        reason: DocumentText { bytes: text(b"source reason"), cut: true },
        proposal: Some(OwnerProposalHistory {
            number: u64::MAX,
            proposer: u64::MAX,
            action: OwnerRetainedAction::Release { task: 8 },
            as_holder: true,
            state: OwnerProposalHistoryState::Rejected {
                by: OwnerRequester::Person(u64::MAX),
                reason: DocumentText::complete(text(b"independent proposal reason")),
            },
        }),
        amendment: Some((Box::new(before), Box::new(after))),
        at: None,
    }))
}
fn milestone() -> HistoryRow {
    HistoryRow::Milestone(Box::new(OwnerMilestoneLine {
        task: 8,
        position: u64::MAX,
        phase: OwnerMilestonePhase::Closing {
            stage: ClosingStage::Run { attempt: u64::MAX },
            ending: OwnerChipEnding::Done(ResultValue::OwnerVerdict {
                verdict: u32::MAX,
                words: DocumentText { bytes: text(b""), cut: true },
                fields: List::with_capacity(1),
            }),
        },
        at: Some(Wall::EPOCH),
    }))
}
fn legacy() -> HistoryRow {
    HistoryRow::Legacy(Box::new(HistoryLine {
        position: u64::MAX,
        task: 8,
        by: Actor::Party(7),
        at: Wall::from_nanos(u64::MAX),
        change: HistoryChange::Made { batch: u64::MAX },
    }))
}
#[test]
fn whole_history_page_prices_all_typed_boxes_stored_presence_and_reserved_rows_without_trimming() {
    let mut bounds = live_script().bounds;
    bounds.rows = 3;
    bounds.document_bytes = u64::MAX;
    let values = [semantic(), milestone(), legacy()];
    let page = HistoryPage {
        cut: true,
        rows: list(values.to_vec()),
        before: Some(Cursor { value: text(b"opaque before") }),
        after: Some(Cursor { value: text(b"opaque after") }),
    };
    let mut expected = u64::try_from(size_of::<HistoryPage>()).unwrap()
        + List::<HistoryRow>::worst_case(3).unwrap()
        + u64::try_from(b"opaque before".len() + b"opaque after".len()).unwrap();
    for value in &values {
        match value {
            HistoryRow::Change(line) => expected += crate::owner_history::bytes(line, &bounds).unwrap(),
            HistoryRow::Milestone(line) => expected += crate::owner_milestone::bytes(line, &bounds).unwrap(),
            HistoryRow::Legacy(line) => {
                expected += u64::try_from(size_of::<HistoryLine>()).unwrap() + line.heap(&bounds).unwrap();
            }
        }
    }
    assert_eq!(page.heap(&bounds).unwrap() + u64::try_from(size_of::<HistoryPage>()).unwrap(), expected);
    assert!(crate::payload::fits(&page, &Bounds { document_bytes: expected, ..bounds }));
    assert!(!crate::payload::fits(&page, &Bounds { document_bytes: expected - 1, ..bounds }));
    assert_eq!(page.rows.as_slice(), &values);
    assert!(page.cut);
    let row = semantic();
    let row_bytes = row.heap(&bounds).unwrap() + u64::try_from(size_of::<HistoryRow>()).unwrap();
    let source = Source {
        party: 1,
        read: Read::History { task: 8, subtree: true, after: None },
        rows: Rows::History(vec![row.clone()]),
    };
    assert!(source.validate(&Bounds { document_bytes: row_bytes, ..bounds }).is_ok());
    assert_eq!(source.validate(&Bounds { document_bytes: row_bytes - 1, ..bounds }), Err(Invalid::Payload));
    assert_eq!(source.rows, Rows::History(vec![row]));
}
#[test]
fn scripted_history_pages_keep_all_owner_and_legacy_rows_under_opaque_source_continuations() {
    let values = vec![semantic(), milestone(), legacy()];
    let mut setup = live_script();
    setup.bounds.rows = 1;
    setup.truth.sources.push(Source {
        party: 1,
        read: Read::History { task: 8, subtree: false, after: None },
        rows: Rows::History(values.clone()),
    });
    let mut engine = Engine::new(setup, 209).unwrap();
    let mut cursor = None;
    for (index, expected) in values.iter().enumerate() {
        let now = Time::from_nanos(10_000_000);
        engine
            .read(
                now,
                11,
                Token::new(u64::try_from(index).unwrap() + 20),
                Read::History { task: 8, subtree: false, after: cursor },
            )
            .unwrap();
        let out = engine.poll(now);
        let Up::Read { terminal: ReadResult::Value(ReadValue::History(page)), .. } = &out[0] else {
            panic!("whole supplied history page");
        };
        assert_eq!(page.rows.as_slice(), core::slice::from_ref(expected));
        assert_eq!(page.before.is_some(), index != 0);
        assert_eq!(page.after.is_some(), index != 2);
        assert!(!page.cut);
        cursor = page.after.clone();
    }
}
