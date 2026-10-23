use super::*;
pub use jig_client_test_records::FULL_TURN;
use skein_lib::{List, Wall};
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
    jig_client_test_records::decoded_activity_record()
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
    jig_client_test_records::unread_activity_record(bytes)
}
pub fn unread_activity_page(why: c::TurnProblem) -> c::ActivityPage {
    expected_two_turns(c::TurnRead::Undecodable { why })
}
pub fn unread_pages() -> [(w::ActivityPage, c::ActivityPage); 3] {
    [
        (unread_activity_record(&[0, 4]), unread_activity_page(c::TurnProblem::Version { found: 4 })),
        (unread_activity_record(&[0, 6]), unread_activity_page(c::TurnProblem::Malformed)),
        (unread_activity_record(&[0, 6, 255]), unread_activity_page(c::TurnProblem::Malformed)),
    ]
}
pub fn cut_activity_record() -> w::ActivityPage {
    jig_client_test_records::cut_activity_record()
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
    jig_client_test_records::archived_activity_record()
}
pub fn owner_turn_activity_record(turn: Option<w::TurnBytes>) -> w::ActivityPage {
    jig_client_test_records::owner_turn_activity_record(turn)
}
pub fn archived_activity_page() -> c::ActivityPage {
    archived_activity_page_with_problem(c::TurnProblem::Version { found: 4 })
}
fn archived_activity_page_with_problem(why: c::TurnProblem) -> c::ActivityPage {
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
        turn: c::TurnRead::Undecodable { why },
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
    client.turn_text = text;
    client.document_bytes = document_bytes;
    Limits::new(client, 4).unwrap()
}

const UTF8_TEXTS: [&[u8]; 1] = [b"~\0\x01"];

// One byte above the wire's carried-turn ceiling; the marker reports the original size.
pub const MARKER_BYTES: u64 = 8 * 1024 * 1024 + 1;

pub fn owner_turn_pages() -> [(&'static str, w::ActivityPage, c::ActivityPage); 2] {
    let large = w::TurnTooLarge::new(&w::CEILINGS, w::TurnTooLargeParts { bytes: MARKER_BYTES }).unwrap();
    [
        (
            "too large",
            owner_turn_activity_record(Some(w::TurnBytes::TooLarge(large))),
            archived_activity_page_with_problem(c::TurnProblem::TooLarge { bytes: MARKER_BYTES }),
        ),
        (
            "malformed unread",
            owner_turn_activity_record(None),
            expected_one_activity_line(c::ActivityLine {
                position: 50,
                at: Wall::from_nanos(51),
                event: c::Activity::Turn {
                    name: c::TurnName { attempt: 44, number: 3 },
                    turn: c::TurnRead::Undecodable { why: c::TurnProblem::Malformed },
                },
            }),
        ),
    ]
}

pub fn turn_read() -> c::Read {
    c::Read::Turn { task: 7, attempt: 44, turn: 3 }
}
pub fn single_turn_record(body: &[u8]) -> w::ArchivedTurn {
    w::ArchivedTurn::new(
        &w::CEILINGS,
        w::ArchivedTurnParts {
            task: 7,
            name: w::TurnName::new(&w::CEILINGS, w::TurnNameParts { attempt: 44, number: 3 }).unwrap(),
            cumulative: 123,
            read: Some(78),
            at: 99,
            turn: w::TurnBytes::Carried(
                w::TurnCarried::new(&w::CEILINGS, w::TurnCarriedParts { bytes: Box::from(body) }).unwrap(),
            ),
        },
    )
    .unwrap()
}
pub fn single_turn_value() -> w::ReadValue {
    w::ReadValue::Turn(single_turn_record(FULL_TURN))
}
pub fn expected_single_turn() -> c::ArchivedTurn {
    c::ArchivedTurn {
        task: 7,
        name: c::TurnName { attempt: 44, number: 3 },
        cumulative: 123,
        read: Some(78),
        at: Wall::from_nanos(99),
        turn: decoded_turn(b"abc", false),
    }
}
pub fn reading_turn(limits: &Limits) -> (Harness, Token) {
    let mut harness = Harness::new(limits);
    let exchange = harness.read(Readable::Jig(turn_read()));
    (harness, exchange)
}
// The existing v7 golden occupies 146 bytes; v9's Some(read) archive and tag add 51.
pub const TURN_PAYLOAD_BYTES: u64 = 146;
pub const TURN_DOCUMENT_BYTES: u64 = TURN_PAYLOAD_BYTES + 51;
pub fn answered_turn(limits: &Limits, answer: &[u8]) -> Harness {
    let (mut harness, exchange) = reading_turn(limits);
    harness.answer(exchange, answer);
    harness
}

pub fn malformed_turn_answers() -> [(&'static str, Box<[u8]>); 4] {
    // Value tag + Jig owner + version precede the owned-record length.
    const RECORD_LENGTH: usize = 1 + 1 + 4;
    // Envelope scalars + read-value tag + archive scalars, including Some(read).
    const PAYLOAD_LENGTH: usize = RECORD_LENGTH + 4 + 1 + 8 + 8 + 4 + 8 + 1 + 8 + 8 + 1;
    let original = jig_read_value_answer(&single_turn_value());
    let mut trailing = Writer::new(original.len().checked_add(1).unwrap());
    trailing.put(&original).unwrap();
    trailing.put(&[0]).unwrap();
    [
        ("truncated", Box::from(&original[..original.len().checked_sub(1).unwrap()])),
        ("trailing", trailing.finish()),
        ("record length", false_turn_length(&original, RECORD_LENGTH)),
        ("payload length", false_turn_length(&original, PAYLOAD_LENGTH)),
    ]
}
fn false_turn_length(original: &[u8], offset: usize) -> Box<[u8]> {
    let mut writer = Writer::new(original.len());
    writer.put(&original[..offset]).unwrap();
    writer.put(&1_u32.to_be_bytes()).unwrap();
    writer.put(&original[offset.checked_add(4).unwrap()..]).unwrap();
    writer.finish()
}
