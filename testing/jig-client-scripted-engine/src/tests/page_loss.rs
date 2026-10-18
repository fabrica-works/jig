use super::*;

const CURSOR_BYTES: [&[u8]; 1] = [b"\xff/\0"];

#[test]
fn a_page_loss_flag_is_independent_of_row_text_loss_and_whole_cursor_bytes() {
    let source = TaskLine {
        chip: {
            let mut chip = {
                let mut chip = task().chip;
                chip.title.cut = true;
                chip
            };
            chip.title = DocumentText { bytes: text(b"words"), cut: true };
            chip
        },
        project: 1,
        requester: jig_client_domain::contract::OwnerRequester::Person(1),
        tracked: None,
        label: 1,
        flags: jig_client_domain::contract::TaskFlags {
            conversation: true,
            listed: false,
            retention: jig_client_domain::contract::Retention::ForgottenOnceRead,
        },
        changed: Wall::EPOCH,
        last_activity: Wall::EPOCH,
        ended_at: None,
        budget: OwnerBudget {
            numbers: OwnerNumbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
            left: 100,
            funder: Funder::Period { project: 1, period: 3 },
            deadline: None,
        },
        waits_on: jig_client_domain::contract::WaitsOn::Nothing,
        phases: jig_client_domain::contract::OwnerPhaseCounts {
            waiting: 0,
            idle: 0,
            starting: 0,
            running: 0,
            backing_off: 0,
            closing: 0,
            held: 0,
            done: 0,
            failed: 0,
            cancelled: 0,
        },
        held_below: 0,
        conversation: Some(jig_client_domain::contract::ConversationRead {
            seen: 2,
            unread: Count { value: 2, limited: false },
        }),
    };
    let page = ReadValue::Tasks(Box::new(TaskPage {
        live: list(vec![source]),
        live_after: Some(Cursor::new(Box::from(CURSOR_BYTES[0]), 1024).expect("bounded opaque cursor")),
        ended: EndedGroup::Rows { rows: list(vec![]), after: None, cut: false },
    }));
    let bounds = Bounds { document_bytes: u64::MAX, ..Bounds::default() };
    let before = page.clone();
    assert!(crate::payload::fits(&page, &bounds));
    let mut smaller = bounds;
    smaller.text = 2;
    assert!(!crate::payload::fits(&page, &smaller));
    assert_eq!(page, before);
    let ReadValue::Tasks(page) = page else {
        panic!("conversation page");
    };
    assert!(matches!(page.ended, EndedGroup::Rows { cut: false, .. }));
    assert!(page.live.get(0).unwrap().chip.title.cut);
    assert_eq!(page.live_after.as_ref().unwrap().value.bytes(), b"\xff/\0");
}
