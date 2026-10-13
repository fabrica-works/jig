//! Small codec limits and shared client contract records.
use crate::*;
use alloc::boxed::Box;
use skein_lib::{List, Overflow, Writer};

pub fn limits() -> Limits {
    let mut limits = CEILINGS;
    limits.keyed_request = 32;
    limits.owner_spend_page_rows = 2;
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
