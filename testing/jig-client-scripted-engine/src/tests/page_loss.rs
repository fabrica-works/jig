use super::*;

#[test]
fn a_page_loss_flag_is_independent_of_row_text_loss_and_whole_cursor_bytes() {
    let source = ConversationLine {
        cached_replies: 0,
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
        first_words: DocumentText::complete(text(b"words")),
        runtime: Some(Runtime::Idle),
        last_reply: 4,
        seen: 2,
        unread: Some(Count { value: 2, limited: false }),
        changed: Some(Wall::EPOCH),
    };
    let page = ReadValue::Conversations(Box::new(ConversationPage {
        cut: false,
        rows: list(vec![source]),
        before: None,
        after: Some(Cursor::new(Box::from(CURSOR_BYTES[0]), 1024).expect("bounded opaque cursor")),
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

const CURSOR_BYTES: [&[u8]; 1] = [b"\xff/\0"];
