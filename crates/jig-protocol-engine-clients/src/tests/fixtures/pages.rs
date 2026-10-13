//! Core page literals independent of the shared wire records.
use super::{inbox_document, list};
use jig_domain_core::{documents as d, reads};
use jig_domain_tasks as t;
use skein_lib::{List, Wall};

pub fn page(rows: List<reads::PageRow>) -> reads::Page {
    reads::Page { rows, before: None, next: None, cut: true, inbox_selection: None }
}
pub fn activity_page() -> reads::Page {
    let mut page = page(list([
        reads::PageRow::Activity {
            line: activity_line(13, 31, 37),
            turn: Some(reads::PageTurn::Carried(turn_record(31, 37, 19, Some(0), 23, &[255, 0, 1]))),
        },
        reads::PageRow::Activity {
            line: activity_line(11, 41, 43),
            turn: Some(reads::PageTurn::Carried(turn_record(41, 43, 29, None, 47, &[128, 254]))),
        },
        reads::PageRow::Activity { line: activity_line(7, 53, 59), turn: Some(reads::PageTurn::Unread) },
    ]));
    page.next = Some(reads::Cursor::Store {
        scope: reads::Scope::Activity { task: 7 },
        after: Some(reads::Position::Owner(jig_domain_core::Key::Tasks(t::Key::Activity { task: 7, position: 7 }))),
    });
    page
}
fn activity_line(position: u64, attempt: u64, turn: u32) -> t::ActivityLine {
    t::ActivityLine {
        task: 7,
        position,
        at: Wall::from_nanos(17),
        line: t::Line::Turn { reply: 61, attempt, turn, read: Some(0) },
    }
}
fn turn_record(
    attempt: u64,
    turn: u32,
    spent: u64,
    read: Option<u64>,
    at: u64,
    transcript: &[u8],
) -> jig_domain_core::TurnRecord {
    jig_domain_core::TurnRecord {
        opens_window: false,
        task: 7,
        attempt,
        turn,
        spent,
        read,
        at: Wall::from_nanos(at),
        transcript: Box::from(transcript),
    }
}
pub fn oversized_turn_page() -> reads::Page {
    page(list([reads::PageRow::Activity {
        line: activity_line(13, 31, 37),
        turn: Some(reads::PageTurn::TooLarge { spent: 19, read: Some(0), at: Wall::from_nanos(23), bytes: 8_388_609 }),
    }]))
}
pub fn history_page() -> reads::Page {
    page(list([
        reads::PageRow::History(d::HistoryLine {
            task: 7,
            revision: 19,
            by: t::Party::Person(23),
            change: t::Change::Prioritised { priority: 29 },
            reason: d::Text { bytes: Box::from("é".as_bytes()), cut: true },
            proposal: None,
            amendment: None,
            at: None,
        }),
        reads::PageRow::Milestone(d::MilestoneLine {
            task: 7,
            position: 31,
            phase: d::MilestonePhase::Waiting,
            at: Some(Wall::from_nanos(37)),
        }),
    ]))
}
pub fn inbox_page(updates: bool) -> reads::Page {
    let source = inbox_document();
    let entries = if updates { source.updates } else { source.needs };
    let mut rows = List::with_capacity(entries.len());
    for row in &entries {
        rows.push(reads::PageRow::Inbox(row.clone())).expect("fixture rows");
    }
    let mut page = page(rows);
    page.inbox_selection = source.selection;
    page
}
pub fn notes_page() -> reads::Page {
    page(list([reads::PageRow::Note(d::Note {
        number: 7,
        revision: 11,
        scope: d::NoteLocation::Resources {
            project: 13,
            scope: d::NoteScope {
                connector: 17,
                segments: list([Box::from([255, b'/', 0])]),
                last: d::PatternEnd::Open(Box::from([128, 0])),
            },
        },
        description: d::Text { bytes: Box::from("é".as_bytes()), cut: false },
        body: d::Text { bytes: Box::from(b"body".as_slice()), cut: true },
        writer: jig_domain_notes::Author::Task { task: 19, attempt: 23 },
        tasks: list([29, 31]),
        changed: Some(Wall::from_nanos(37)),
        acts: List::with_capacity(0),
        cut: true,
    })]))
}
