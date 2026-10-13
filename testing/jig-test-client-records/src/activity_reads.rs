//! Shared activity wire records; original client values remain independent.
use super::w;
use skein_lib::Writer;
pub const FULL_TURN: &[u8] = &[
    0, 3, 0, 1, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 255, 255, 255, 255, 0,
    0, 0, 3, 97, 98, 99, 0, 0, 0, 3, 97, 98, 99, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255,
    255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255, 255,
    255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 1, 0, 0, 0, 0, 1,
    0, 0, 0, 0, 3, 97, 98, 99, 1, 0, 0, 0, 3, 97, 98, 99, 0, 0, 0, 3, 0, 127, 255,
];
const UTF8_TEXTS: [&[u8]; 1] = [b"~\0\x01"];
fn wire_cursor(value: &[u8]) -> w::Cursor {
    w::Cursor::new(&w::CEILINGS, w::CursorParts { value: Box::from(value) }).expect("literal fixture fits")
}
#[must_use]
pub fn turn_line(position: u64, at: u64, number: u32, bytes: &[u8]) -> w::ActivityLine {
    let lim = w::CEILINGS;
    let turn = w::ActivityTurn::new(
        &lim,
        w::ActivityTurnParts {
            name: w::TurnName::new(&lim, w::TurnNameParts { attempt: 44, number }).expect("literal fixture fits"),
            turn: Box::from(bytes),
        },
    )
    .expect("literal fixture fits");
    w::ActivityLine::new(&lim, w::ActivityLineParts { position, at, event: w::Activity::Turn(turn) })
        .expect("literal fixture fits")
}

#[must_use]
pub fn activity_rows(rows: &[w::ActivityLine]) -> w::ActivityPage {
    let mut length = 0_usize;
    for row in rows {
        length = length
            .checked_add(usize::try_from(row.measure()).expect("literal fixture fits"))
            .expect("literal fixture fits")
            .checked_add(4)
            .expect("literal fixture fits");
    }
    let mut piece = Writer::new(length);
    for row in rows {
        let mut bytes = Writer::new(usize::try_from(row.measure()).expect("literal fixture fits"));
        row.encode(&mut bytes).expect("literal fixture fits");
        skein_codec::piece::write(&mut piece, &bytes.finish()).expect("literal fixture fits");
    }
    w::ActivityPage::new(
        &w::CEILINGS,
        w::ActivityPageParts { rows: piece.finish(), cut: true, before: None, after: Some(wire_cursor(UTF8_TEXTS[0])) },
    )
    .expect("literal fixture fits")
}

#[must_use]
pub fn decoded_activity_record() -> w::ActivityPage {
    activity_rows(&[turn_line(10, 20, 1, FULL_TURN), turn_line(11, 21, 2, FULL_TURN)])
}

#[must_use]
pub fn unread_activity_record(bytes: &[u8]) -> w::ActivityPage {
    activity_rows(&[turn_line(10, 20, 1, bytes), turn_line(11, 21, 2, FULL_TURN)])
}

#[must_use]
pub fn cut_activity_record() -> w::ActivityPage {
    activity_rows(&[turn_line(10, 20, 1, FULL_TURN)])
}

fn owner_turn_source() -> w::OwnerActivityLine {
    let lim = w::CEILINGS;
    w::OwnerActivityLine::new(
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
                .expect("literal fixture fits"),
            ),
        },
    )
    .expect("literal fixture fits")
}

#[must_use]
pub fn archived_activity_record() -> w::ActivityPage {
    let lim = w::CEILINGS;
    let turn =
        w::TurnCarried::new(&lim, w::TurnCarriedParts { bytes: Box::from([0, 4]) }).expect("literal fixture fits");
    owner_turn_activity_record(Some(w::TurnBytes::Carried(turn)))
}

#[must_use]
pub fn owner_turn_activity_record(turn: Option<w::TurnBytes>) -> w::ActivityPage {
    let lim = w::CEILINGS;
    let source = owner_turn_source();
    let mut decoded = None;
    if let Some(turn) = turn {
        decoded = Some(
            w::ArchivedTurn::new(
                &lim,
                w::ArchivedTurnParts {
                    task: 7,
                    name: w::TurnName::new(&lim, w::TurnNameParts { attempt: 44, number: 3 })
                        .expect("literal fixture fits"),
                    cumulative: 123,
                    read: Some(78),
                    at: 99,
                    turn,
                },
            )
            .expect("literal fixture fits"),
        );
    }
    let event = w::Activity::Owner(
        w::ActivityOwner::new(&lim, w::ActivityOwnerParts { source, decoded }).expect("literal fixture fits"),
    );
    let row =
        w::ActivityLine::new(&lim, w::ActivityLineParts { position: 50, at: 51, event }).expect("literal fixture fits");
    activity_rows(&[row])
}
