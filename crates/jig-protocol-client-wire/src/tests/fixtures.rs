//! Small codec limits and shared client contract records.
use crate::*;
use alloc::boxed::Box;
use skein_lib::{List, Overflow, Writer};

pub fn limits() -> Limits {
    limits_with_activity_bytes(CEILINGS.activity_page_rows)
}

pub fn limits_with_activity_bytes(bound: u32) -> Limits {
    let mut limits = CEILINGS;
    limits.keyed_request = 32;
    limits.owner_spend_page_rows = 2;
    limits.activity_page_rows = bound;
    check_limits(&limits).expect("client test limits below ceilings");
    limits
}

pub fn bytes(n: u32) -> Box<[u8]> {
    skein_lib::bytes::zeroed(usize::try_from(n).expect("small bound"))
}

pub fn ceiling_text(n: u32) -> Box<[u8]> {
    let mut words = bytes(n);
    words.fill(b'x');
    words
}

pub fn start_chat() -> StartChat {
    let limits = limits();
    StartChat::new(&limits, StartChatParts { project: 17, words: ceiling_text(limits.start_chat_words) })
        .expect("bounded chat")
}

pub fn refused_moved() -> Refused {
    let limits = limits();
    let how = EndingTaskFields::new(&limits, EndingTaskFieldsParts { value: 2 }).expect("ending index");
    let reason = HoldReasonDependencyFields::new(&limits, HoldReasonDependencyFieldsParts { task: 31, how: 1 })
        .expect("dependency index");
    let words = DocumentText::new(&limits, DocumentTextParts { bytes: Box::from(b"failed".as_slice()), cut: true })
        .expect("bounded failure words");
    let failed = OwnerChipEndingFailedFields::new(&limits, OwnerChipEndingFailedFieldsParts { value: words })
        .expect("complete failure");
    let mut rows = List::with_capacity(3);
    rows.push(Fact::HoldReason(
        HoldReasonRow::new(&limits, HoldReasonRowParts { parent: None, value: HoldReasonFields::Dependency(reason) })
            .expect("root fact"),
    ))
    .expect("three rows");
    rows.push(Fact::Ending(
        EndingRow::new(&limits, EndingRowParts { parent: Some(0), value: EndingFields::Task(how) })
            .expect("ending fact"),
    ))
    .expect("three rows");
    rows.push(Fact::OwnerChipEnding(
        OwnerChipEndingRow::new(
            &limits,
            OwnerChipEndingRowParts { parent: Some(1), value: OwnerChipEndingFields::Failed(failed) },
        )
        .expect("failure fact"),
    ))
    .expect("three rows");
    let rows = piece_items(&rows, Fact::measure, Fact::encode);
    let reason = HoldReason::new(&limits, HoldReasonParts { rows }).expect("flattened reason");
    let reason = CurrentHoldDocument::new(&limits, CurrentHoldDocumentParts { value: reason }).expect("current hold");
    let what =
        MovedHold::new(&limits, MovedHoldParts { reason: Some(CurrentHold::Document(reason)) }).expect("moved hold");
    let now = Revision::new(&limits, RevisionParts { value: u64::MAX }).expect("whole revision");
    let moved = RefusalMoved::new(&limits, RefusalMovedParts { now, what: Moved::Hold(what) }).expect("refusal facts");
    Refused::new(&limits, RefusedParts { refusal: Refusal::Moved(moved) }).expect("complete refusal")
}

pub fn encoded<T>(value: &T, measure: fn(&T) -> u32, encode: fn(&T, &mut Writer) -> Result<(), Overflow>) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(measure(value)).expect("measured record"));
    encode(value, &mut writer).expect("measured room");
    writer.finish()
}

pub fn start_chat_bytes() -> Box<[u8]> {
    encoded(&start_chat(), StartChat::measure, StartChat::encode)
}

pub fn owned() -> Owned {
    let limits = limits();
    let family = Family::new(&limits, FamilyParts { number: u16::MAX }).expect("whole family number");
    Owned::new(&limits, OwnedParts { owner: Owner::Family(family), version: u32::MAX, record: start_chat_bytes() })
        .expect("whole owner's version")
}

pub fn read_none(none: ReadNone) -> ReadAnswerNone {
    ReadAnswerNone::new(&limits(), ReadAnswerNoneParts { value: none }).expect("wire read outcome")
}

pub fn funding_rows(count: u32) -> List<OwnerFundingRecord> {
    let limits = limits();
    let period = OwnerPeriodTimes::new(&limits, OwnerPeriodTimesParts { began: u64::MAX, next: None })
        .expect("optional next period");
    let numbers =
        OwnerNumbers::new(&limits, OwnerNumbersParts { budget: u64::MAX, spent: 11, spent_below: 17, reserved: 23 })
            .expect("owner funding numbers");
    let source =
        FunderPeriod::new(&limits, FunderPeriodParts { project: 17, period: u64::MAX }).expect("whole period identity");
    let row = OwnerFundingRecord::new(
        &limits,
        OwnerFundingRecordParts {
            period: Some(period),
            funder: Funder::Period(source),
            parent: None,
            numbers,
            made: u32::MAX,
            closed: true,
        },
    )
    .expect("whole funding row");
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        rows.push(row.clone()).expect("reserved funding rows");
    }
    rows
}

pub fn piece_items<T>(
    rows: &List<T>,
    measure: fn(&T) -> u32,
    encode: fn(&T, &mut Writer) -> Result<(), Overflow>,
) -> Box<[u8]> {
    let mut length = 0_u32;
    for row in rows {
        length = length.checked_add(measure(row)).expect("small fixture row");
        length = length.checked_add(4).expect("small fixture piece");
    }
    let mut out = Writer::new(usize::try_from(length).expect("fixture length fits"));
    for row in rows {
        let bytes = encoded(row, measure, encode);
        skein_codec::piece::write(&mut out, &bytes).expect("measured piece room");
    }
    out.finish()
}

pub fn activity_line(turn: Box<[u8]>) -> ActivityLine {
    let limits = limits();
    let name = TurnName::new(&limits, TurnNameParts { attempt: u64::MAX, number: u32::MAX })
        .expect("whole committed turn name");
    let event = ActivityTurn::new(&limits, ActivityTurnParts { name, turn }).expect("opaque turn bytes");
    ActivityLine::new(&limits, ActivityLineParts { position: u64::MAX, at: u64::MAX, event: Activity::Turn(event) })
        .expect("whole activity line")
}

pub fn activity_rows(count: u32, width: u32) -> List<ActivityLine> {
    let row = activity_line(bytes(width));
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        rows.push(row.clone()).expect("reserved activity rows");
    }
    rows
}

pub fn activity_page(limits: &Limits, rows: Box<[u8]>) -> ActivityPage {
    ActivityPage::new(limits, ActivityPageParts { cut: true, rows, before: None, after: None })
        .expect("page within its piece bound")
}

pub fn encoded_activity_page(limits: &Limits, rows: &List<ActivityLine>) -> Box<[u8]> {
    let rows = piece_items(rows, ActivityLine::measure, ActivityLine::encode);
    encoded(&activity_page(limits, rows), ActivityPage::measure, ActivityPage::encode)
}

fn next_activity_row(limits: &Limits, reader: &mut skein_lib::Reader<'_>) -> Result<Option<ActivityLine>, Problem> {
    let bytes = match skein_codec::piece::split(reader) {
        Ok(bytes) => bytes,
        Err(reason) => return Err(Problem { path: Path::ActivityPageRows, reason }),
    };
    let Some(bytes) = bytes else { return Ok(None) };
    let row = ActivityLine::decode(limits, &mut skein_lib::Reader::new(&bytes))?;
    Ok(Some(row))
}

pub fn decode_activity_page(limits: &Limits, bytes: &[u8], row_limit: u32) -> Result<List<ActivityLine>, Problem> {
    let page = ActivityPage::decode(limits, &mut skein_lib::Reader::new(bytes))?;
    let mut reader = skein_lib::Reader::new(page.rows());
    let mut rows = List::with_capacity(row_limit);
    while let Some(row) = next_activity_row(limits, &mut reader)? {
        if rows.push(row).is_err() {
            return Err(Problem { path: Path::ActivityPageRows, reason: skein_codec::Reason::Bound });
        }
    }
    Ok(rows)
}

pub fn malformed_activity_pieces() -> [Box<[u8]>; 3] {
    let rows = activity_rows(1, 8);
    let valid = piece_items(&rows, ActivityLine::measure, ActivityLine::encode);
    let mut out = Writer::new(valid.len().checked_add(1).expect("small trailing fragment"));
    out.put(&valid).expect("room for valid row");
    out.put(&[255]).expect("room for trailing fragment");
    [Box::from([0, 0, 0]), Box::from([0, 0, 0, 5, 1, 2]), out.finish()]
}
