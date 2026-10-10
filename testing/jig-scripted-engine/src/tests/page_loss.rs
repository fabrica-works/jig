use super::*;

#[test]
fn a_page_loss_flag_is_independent_of_row_text_loss_and_whole_cursor_bytes() {
    let source = ConversationLine {
        project: 1,
        chip: {
            let mut chip = task().chip;
            chip.title.cut = true;
            chip
        },
        delegates: 0,
        budget: OwnerBudget {
            numbers: OwnerNumbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
            left: 100,
            funder: Funder::Period { project: 1, period: 3 },
            deadline: None,
        },
        first_words: text(b"words"),
        runtime: Runtime::Idle,
        last_reply: 4,
        seen: 2,
        unread: Count { value: 2, limited: false },
        changed: Wall::EPOCH,
    };
    let page = ReadValue::Conversations(Box::new(ConversationPage {
        cut: false,
        rows: list(vec![source]),
        before: None,
        after: Some(Cursor { value: text(b"\xff/\0") }),
    }));
    let bounds = Bounds { document_bytes: u64::MAX, ..Bounds::default() };
    let before = page.clone();
    assert!(crate::payload::fits(&page, &bounds));
    let mut smaller = bounds;
    smaller.text = 2;
    assert!(!crate::payload::fits(&page, &smaller));
    assert_eq!(page, before);
    let ReadValue::Conversations(page) = page else {
        panic!("conversation page");
    };
    assert!(!page.cut);
    assert!(page.rows.get(0).unwrap().chip.title.cut);
    assert_eq!(page.after.as_ref().unwrap().value.bytes(), b"\xff/\0");
}
