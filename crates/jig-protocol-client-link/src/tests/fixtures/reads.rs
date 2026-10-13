use super::*;
use skein_lib::List;
pub fn read_ending_record() -> w::Ending {
    let lim = w::CEILINGS;
    let row = w::Fact::Ending(
        w::EndingRow::new(&lim, w::EndingRowParts { parent: None, value: w::EndingFields::Done }).unwrap(),
    );
    w::Ending::new(&lim, w::EndingParts { rows: fact_bytes(&row) }).unwrap()
}
pub fn read_hold_reason_record() -> w::HoldReason {
    let lim = w::CEILINGS;
    let row = w::Fact::HoldReason(
        w::HoldReasonRow::new(
            &lim,
            w::HoldReasonRowParts {
                parent: None,
                value: w::HoldReasonFields::Tries(
                    w::HoldReasonTriesFields::new(
                        &lim,
                        w::HoldReasonTriesFieldsParts { failures: List::with_capacity(0) },
                    )
                    .unwrap(),
                ),
            },
        )
        .unwrap(),
    );
    w::HoldReason::new(&lim, w::HoldReasonParts { rows: fact_bytes(&row) }).unwrap()
}
#[path = "reads_part0.rs"]
mod part0;
pub use part0::*;
#[path = "reads_part1.rs"]
mod part1;
pub use part1::*;
#[path = "reads_part2.rs"]
mod part2;
pub use part2::*;
#[path = "reads_part3.rs"]
mod part3;
pub use part3::*;
#[path = "reads_part4.rs"]
mod part4;
pub use part4::*;
#[path = "reads_part5.rs"]
mod part5;
pub use part5::*;
#[path = "reads_part6.rs"]
mod part6;
pub use part6::*;
#[path = "reads_part7.rs"]
mod part7;
pub use part7::*;

pub fn raw_cursor(value: &[u8]) -> c::Cursor {
    c::Cursor { value: c::Text::new(Box::from(value), 256).unwrap() }
}
pub fn wire_cursor(value: &[u8]) -> w::Cursor {
    w::Cursor::new(&w::CEILINGS, w::CursorParts { value: Box::from(value) }).unwrap()
}
pub fn note_page_with_cursors() -> w::NotePage {
    let mut parts = read_wire_note_page().into_parts();
    parts.before = Some(wire_cursor(&[255; 48]));
    parts.after = Some(wire_cursor(&[0; 48]));
    parts.cut = true;
    w::NotePage::new(&w::CEILINGS, parts).unwrap()
}
pub fn expected_note_page_with_cursors() -> c::NotePage {
    c::NotePage {
        rows: List::with_capacity(0),
        before: Some(raw_cursor(&[255; 48])),
        after: Some(raw_cursor(&[0; 48])),
        cut: true,
    }
}
pub fn object_page_piece(piece: Box<[u8]>) -> w::ObjectPage {
    let mut parts = read_wire_object_page().into_parts();
    parts.rows = piece;
    w::ObjectPage::new(&w::CEILINGS, parts).unwrap()
}
pub fn result_rows(count: u32) -> Box<[u8]> {
    let record = read_wire_result();
    let mut encoded = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut encoded).unwrap();
    let encoded = encoded.finish();
    let size = encoded.len().checked_add(4).unwrap().checked_mul(usize::try_from(count).unwrap()).unwrap();
    let mut writer = Writer::new(size);
    for _ in 0..count {
        skein_codec::piece::write(&mut writer, &encoded).unwrap();
    }
    writer.finish()
}
pub fn expected_object_page(count: u32) -> c::ObjectPage {
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        rows.push(read_value_result()).unwrap();
    }
    c::ObjectPage { rows, cut: false, before: None, after: None }
}

pub fn historical_task_record() -> w::OwnerHistorical {
    let lim = w::CEILINGS;
    let ending = w::Fact::Ending(
        w::EndingRow::new(
            &lim,
            w::EndingRowParts {
                parent: None,
                value: w::EndingFields::Failed(
                    w::EndingFailedFields::new(
                        &lim,
                        w::EndingFailedFieldsParts { reason: Some(Box::from(b"failed".as_slice())) },
                    )
                    .unwrap(),
                ),
            },
        )
        .unwrap(),
    );
    let mut result = read_wire_result().into_parts();
    result.ending = w::Ending::new(&lim, w::EndingParts { rows: fact_bytes(&ending) }).unwrap();
    let result = w::ResultDocument::new(&lim, result).unwrap();
    let mut historical = read_wire_owner_historical().into_parts();
    historical.at = Some(u64::MAX);
    historical.value = w::OwnerHistoricalValue::Task(
        w::OwnerHistoricalValueTask::new(&lim, w::OwnerHistoricalValueTaskParts { document: task_record(), result })
            .unwrap(),
    );
    w::OwnerHistorical::new(&lim, historical).unwrap()
}
pub fn expected_historical_task() -> c::OwnerHistorical {
    let mut result = read_value_result();
    result.ending = c::Ending::Failed { reason: Some(text(b"failed")) };
    c::OwnerHistorical {
        key: c::ObjectKey::Task(7),
        value: c::OwnerHistoricalValue::Task { document: Box::new(task()), result: Box::new(result) },
        at: Some(skein_lib::Wall::from_nanos(u64::MAX)),
        by: None,
    }
}

pub fn trailing_result_row() -> Box<[u8]> {
    let record = read_wire_result();
    let length = usize::try_from(record.measure()).unwrap().checked_add(1).unwrap();
    let mut row = Writer::new(length);
    record.encode(&mut row).unwrap();
    row.put(&[255]).unwrap();
    let mut piece = Writer::new(length.checked_add(4).unwrap());
    skein_codec::piece::write(&mut piece, &row.finish()).unwrap();
    piece.finish()
}
pub fn proposal_with_words(words: &[u8]) -> w::Proposal {
    let mut parts = read_wire_proposal().into_parts();
    parts.why = Box::from(words);
    w::Proposal::new(&w::CEILINGS, parts).unwrap()
}
