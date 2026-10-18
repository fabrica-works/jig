//! Shared page wire literals; neither side derives its value from a translator.
use super::{inbox_both_groups_record, limits, w, wire_text};
use skein_lib::{List, Writer};
fn rows<T, const N: usize>(values: [T; N]) -> List<T> {
    let mut rows = List::with_capacity(u32::try_from(N).expect("literal rows"));
    for row in values {
        rows.push(row).ok().expect("literal row");
    }
    rows
}
fn encoded(measure: u32, encode: impl FnOnce(&mut Writer) -> Result<(), skein_lib::Overflow>) -> Box<[u8]> {
    let mut out = Writer::new(usize::try_from(measure).expect("literal size"));
    encode(&mut out).expect("literal record");
    out.finish()
}
fn pieces<const N: usize>(values: [Box<[u8]>; N]) -> Box<[u8]> {
    let size = values.iter().map(|v| v.len().checked_add(4).expect("literal piece size")).sum();
    let mut out = Writer::new(size);
    for value in values {
        skein_codec::piece::write(&mut out, &value).expect("literal piece");
    }
    out.finish()
}
fn activity_row(position: u64, attempt: u64, number: u32, decoded: Option<w::ArchivedTurn>) -> w::ActivityLine {
    let l = limits();
    w::ActivityLine::new(
        &l,
        w::ActivityLineParts {
            position,
            at: 17,
            event: w::Activity::Owner(
                w::ActivityOwner::new(
                    &l,
                    w::ActivityOwnerParts {
                        source: w::OwnerActivityLine::new(
                            &l,
                            w::OwnerActivityLineParts {
                                task: 7,
                                position,
                                at: 17,
                                line: w::OwnerActivity::Turn(
                                    w::OwnerActivityTurn::new(
                                        &l,
                                        w::OwnerActivityTurnParts { reply: 61, attempt, turn: number, read: Some(0) },
                                    )
                                    .expect("literal"),
                                ),
                            },
                        )
                        .expect("literal"),
                        decoded,
                    },
                )
                .expect("literal"),
            ),
        },
    )
    .expect("literal")
}
fn archive(
    attempt: u64,
    number: u32,
    cumulative: u64,
    read: Option<u64>,
    at: u64,
    turn: w::TurnBytes,
) -> w::ArchivedTurn {
    let l = limits();
    w::ArchivedTurn::new(
        &l,
        w::ArchivedTurnParts {
            task: 7,
            name: w::TurnName::new(&l, w::TurnNameParts { attempt, number }).expect("literal"),
            cumulative,
            read,
            at,
            turn,
        },
    )
    .expect("literal")
}
#[must_use]
pub fn carried_activity_page() -> w::ActivityPage {
    let l = limits();
    let first = activity_row(
        13,
        31,
        37,
        Some(archive(
            31,
            37,
            19,
            Some(0),
            23,
            w::TurnBytes::Carried(
                w::TurnCarried::new(&l, w::TurnCarriedParts { bytes: Box::from([255, 0, 1]) }).expect("literal"),
            ),
        )),
    );
    let second = activity_row(
        11,
        41,
        43,
        Some(archive(
            41,
            43,
            29,
            None,
            47,
            w::TurnBytes::Carried(
                w::TurnCarried::new(&l, w::TurnCarriedParts { bytes: Box::from([128, 254]) }).expect("literal"),
            ),
        )),
    );
    let third = activity_row(7, 53, 59, None);
    w::ActivityPage::new(
        &l,
        w::ActivityPageParts {
            cut: true,
            rows: pieces([
                encoded(first.measure(), |w| first.encode(w)),
                encoded(second.measure(), |w| second.encode(w)),
                encoded(third.measure(), |w| third.encode(w)),
            ]),
            before: Some(
                w::Cursor::new(
                    &l,
                    w::CursorParts {
                        value: Box::from([
                            1, 1, 10, 7, 0, 0, 0, 0, 0, 0, 0, 1, 0, 19, 0, 0, 0, 1, 1, 11, 0, 0, 0, 0, 0, 0, 0, 7, 0,
                            0, 0, 0, 0, 0, 0, 7,
                        ]),
                    },
                )
                .expect("literal"),
            ),
            after: None,
        },
    )
    .expect("literal")
}
#[must_use]
pub fn oversized_activity_page() -> w::ActivityPage {
    let l = limits();
    let row = activity_row(
        13,
        31,
        37,
        Some(archive(
            31,
            37,
            19,
            Some(0),
            23,
            w::TurnBytes::TooLarge(
                w::TurnTooLarge::new(&l, w::TurnTooLargeParts { bytes: 8_388_609 }).expect("literal"),
            ),
        )),
    );
    w::ActivityPage::new(
        &l,
        w::ActivityPageParts {
            cut: true,
            rows: pieces([encoded(row.measure(), |w| row.encode(w))]),
            before: None,
            after: None,
        },
    )
    .expect("literal")
}
#[must_use]
pub fn mixed_history_page() -> w::HistoryPage {
    let l = limits();
    let mut reason = wire_text("é".as_bytes()).into_parts();
    reason.cut = true;
    let change = w::HistoryRow::Change(Box::new(
        w::HistoryRowChange::new(
            &l,
            w::HistoryRowChangeParts {
                value: w::OwnerHistoryLine::new(
                    &l,
                    w::OwnerHistoryLineParts {
                        task: 7,
                        revision: 19,
                        by: w::OwnerRequester::Person(
                            w::OwnerRequesterPerson::new(&l, w::OwnerRequesterPersonParts { value: 23 })
                                .expect("literal"),
                        ),
                        change: w::OwnerHistoryChange::Amended,
                        reason: w::DocumentText::new(&l, reason).expect("literal"),
                        proposal: None,
                        amendment: None,
                        at: None,
                    },
                )
                .expect("literal"),
            },
        )
        .expect("literal"),
    ));
    let milestone = w::HistoryRow::Milestone(
        w::HistoryRowMilestone::new(
            &l,
            w::HistoryRowMilestoneParts {
                value: w::OwnerMilestoneLine::new(
                    &l,
                    w::OwnerMilestoneLineParts {
                        task: 7,
                        position: 31,
                        phase: w::OwnerMilestonePhase::Waiting,
                        at: Some(37),
                    },
                )
                .expect("literal"),
            },
        )
        .expect("literal"),
    );
    w::HistoryPage::new(
        &l,
        w::HistoryPageParts {
            cut: true,
            rows: pieces([
                encoded(change.measure(), |w| change.encode(w)),
                encoded(milestone.measure(), |w| milestone.encode(w)),
            ]),
            before: None,
            after: None,
        },
    )
    .expect("literal")
}
#[must_use]
pub fn selected_inbox_pages() -> [w::OwnerInboxPage; 2] {
    let source = inbox_both_groups_record().into_parts();
    [source.needs, source.updates].map(|rows| {
        w::OwnerInboxPage::new(
            &limits(),
            w::OwnerInboxPageParts { cut: true, rows, before: None, after: None, selection: source.selection.clone() },
        )
        .expect("literal")
    })
}
#[must_use]
pub fn scoped_notes_page() -> w::NotePage {
    let l = limits();
    let pattern = w::OwnerPattern::new(
        &l,
        w::OwnerPatternParts {
            segments: rows([Box::<[u8]>::from([255, b'/', 0])]),
            last: w::OwnerPatternEnd::Open(
                w::OwnerPatternEndOpen::new(&l, w::OwnerPatternEndOpenParts { value: Box::from([128, 0]) })
                    .expect("literal"),
            ),
        },
    )
    .expect("literal");
    let mut body = wire_text(b"body").into_parts();
    body.cut = true;
    let row = w::Note::new(
        &l,
        w::NoteParts {
            number: 7,
            revision: w::Revision::new(&l, w::RevisionParts { value: 11 }).expect("literal"),
            scope: w::NoteScope::Resource(
                w::NoteScopeResource::new(&l, w::NoteScopeResourceParts { project: 13, connector: 17, pattern })
                    .expect("literal"),
            ),
            description: wire_text("é".as_bytes()),
            body: w::DocumentText::new(&l, body).expect("literal"),
            writer: w::NoteAuthor::Task(
                w::NoteAuthorTask::new(&l, w::NoteAuthorTaskParts { task: 19, attempt: 23 }).expect("literal"),
            ),
            tasks: rows([29, 31]),
            changed: Some(37),
            acts: Box::new([]),
            cut: true,
        },
    )
    .expect("literal");
    w::NotePage::new(
        &l,
        w::NotePageParts {
            cut: true,
            rows: pieces([encoded(row.measure(), |w| row.encode(w))]),
            before: None,
            after: None,
        },
    )
    .expect("literal")
}
