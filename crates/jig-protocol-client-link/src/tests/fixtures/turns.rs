use super::*;
use skein_lib::{List, Wall};
pub const FULL_TURN: &[u8] = &[
    0, 3, 0, 1, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 255, 255, 255, 255, 0,
    0, 0, 3, 97, 98, 99, 0, 0, 0, 3, 97, 98, 99, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255,
    255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255,
    255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 1, 0, 0, 0, 0, 1,
    0, 0, 0, 0, 3, 97, 98, 99, 1, 0, 0, 0, 3, 97, 98, 99, 0, 0, 0, 3, 0, 127, 255,
];
pub fn turn_line(position: u64, at: u64, number: u32, bytes: &[u8]) -> w::ActivityLine {
    let lim = w::CEILINGS;
    let turn = w::ActivityTurn::new(
        &lim,
        w::ActivityTurnParts {
            name: w::TurnName::new(&lim, w::TurnNameParts { attempt: 44, number }).unwrap(),
            turn: Box::from(bytes),
        },
    )
    .unwrap();
    w::ActivityLine::new(&lim, w::ActivityLineParts { position, at, event: w::Activity::Turn(turn) }).unwrap()
}
pub fn activity_rows(rows: &[w::ActivityLine]) -> w::ActivityPage {
    let mut length = 0_usize;
    for row in rows {
        length = length.checked_add(usize::try_from(row.measure()).unwrap()).unwrap().checked_add(4).unwrap();
    }
    let mut piece = Writer::new(length);
    for row in rows {
        let mut bytes = Writer::new(usize::try_from(row.measure()).unwrap());
        row.encode(&mut bytes).unwrap();
        skein_codec::piece::write(&mut piece, &bytes.finish()).unwrap();
    }
    w::ActivityPage::new(
        &w::CEILINGS,
        w::ActivityPageParts { rows: piece.finish(), cut: true, before: None, after: Some(wire_cursor(UTF8_TEXTS[0])) },
    )
    .unwrap()
}
pub fn decoded_turn(words: &[u8], cut: bool) -> c::TurnRead {
    let mut blocks = List::with_capacity(1);
    blocks.push(c::Block::Text(text(words))).unwrap();
    let mut messages = List::with_capacity(1);
    messages.push(c::Message { speaker: c::Speaker::Person, blocks }).unwrap();
    c::TurnRead::Decoded(c::Turn {
        messages,
        used: c::Usage { input: u64::MAX, output: u64::MAX, cached: u64::MAX, cache_written: u64::MAX },
        spent: u64::MAX,
        read_through: None,
        cut,
    })
}
pub fn expected_activity_rows(rows: List<c::ActivityLine>) -> c::ActivityPage {
    c::ActivityPage { rows, cut: true, before: None, after: Some(raw_cursor(UTF8_TEXTS[0])) }
}
pub fn expected_turn_line(position: u64, at: u64, number: u32, turn: c::TurnRead) -> c::ActivityLine {
    c::ActivityLine {
        position,
        at: Wall::from_nanos(at),
        event: c::Activity::Turn { name: c::TurnName { attempt: 44, number }, turn },
    }
}
pub fn decoded_activity_record() -> w::ActivityPage {
    activity_rows(&[turn_line(10, 20, 1, FULL_TURN), turn_line(11, 21, 2, FULL_TURN)])
}
pub fn decoded_activity_page() -> c::ActivityPage {
    expected_two_turns(decoded_turn(b"abc", false))
}
pub fn expected_two_turns(first: c::TurnRead) -> c::ActivityPage {
    let mut rows = List::with_capacity(2);
    rows.push(expected_turn_line(10, 20, 1, first)).unwrap();
    rows.push(expected_turn_line(11, 21, 2, decoded_turn(b"abc", false))).unwrap();
    expected_activity_rows(rows)
}
pub fn unread_activity_record(bytes: &[u8]) -> w::ActivityPage {
    activity_rows(&[turn_line(10, 20, 1, bytes), turn_line(11, 21, 2, FULL_TURN)])
}
pub fn unread_activity_page(why: c::TurnProblem) -> c::ActivityPage {
    expected_two_turns(c::TurnRead::Undecodable { why })
}
pub fn unread_pages() -> [(w::ActivityPage, c::ActivityPage); 3] {
    [
        (unread_activity_record(&[0, 4]), unread_activity_page(c::TurnProblem::Version { found: 4 })),
        (unread_activity_record(&[0, 3]), unread_activity_page(c::TurnProblem::Malformed)),
        (unread_activity_record(&[0, 3, 255]), unread_activity_page(c::TurnProblem::Malformed)),
    ]
}
pub fn cut_activity_record() -> w::ActivityPage {
    activity_rows(&[turn_line(10, 20, 1, FULL_TURN)])
}
pub fn cut_activity_page() -> c::ActivityPage {
    expected_one_activity_line(expected_turn_line(10, 20, 1, decoded_turn(b"ab", true)))
}
pub fn expected_one_activity_line(row: c::ActivityLine) -> c::ActivityPage {
    let mut rows = List::with_capacity(1);
    rows.push(row).unwrap();
    expected_activity_rows(rows)
}
pub fn archived_activity_record() -> w::ActivityPage {
    let lim = w::CEILINGS;
    let source = w::OwnerActivityLine::new(
        &lim,
        w::OwnerActivityLineParts {
            task: 7,
            position: 88,
            at: 66,
            line: w::OwnerActivity::Turn(
                w::OwnerActivityTurn::new(
                    &lim,
                    w::OwnerActivityTurnParts { reply: 999, attempt: 44, turn: 3, read: Some(77) },
                )
                .unwrap(),
            ),
        },
    )
    .unwrap();
    let decoded = w::ArchivedTurn::new(
        &lim,
        w::ArchivedTurnParts {
            task: 7,
            name: w::TurnName::new(&lim, w::TurnNameParts { attempt: 44, number: 3 }).unwrap(),
            cumulative: 123,
            read: Some(78),
            at: 99,
            turn: Box::from([0, 4]),
        },
    )
    .unwrap();
    let event = w::Activity::Owner(
        w::ActivityOwner::new(&lim, w::ActivityOwnerParts { source, decoded: Some(decoded) }).unwrap(),
    );
    let row = w::ActivityLine::new(&lim, w::ActivityLineParts { position: 50, at: 51, event }).unwrap();
    activity_rows(&[row])
}
pub fn archived_activity_page() -> c::ActivityPage {
    let source = c::OwnerActivityLine {
        task: 7,
        position: 88,
        at: Wall::from_nanos(66),
        line: c::OwnerActivity::Turn { reply: 999, attempt: 44, turn: 3, read: Some(77) },
    };
    let decoded = c::ArchivedTurn {
        task: 7,
        name: c::TurnName { attempt: 44, number: 3 },
        cumulative: 123,
        read: Some(78),
        at: Wall::from_nanos(99),
        turn: c::TurnRead::Undecodable { why: c::TurnProblem::Version { found: 4 } },
    };
    expected_one_activity_line(c::ActivityLine {
        position: 50,
        at: Wall::from_nanos(51),
        event: c::Activity::Owner { source: Box::new(source), decoded: Some(decoded) },
    })
}

pub fn turn_limits(text: u32, document_bytes: u64) -> Limits {
    let original = limits();
    let mut client = original.client;
    client.text = text;
    client.document_bytes = document_bytes;
    Limits::new(client, 4, original.deadline).unwrap()
}

const UTF8_TEXTS: [&[u8]; 1] = [b"~\0\x01"];
