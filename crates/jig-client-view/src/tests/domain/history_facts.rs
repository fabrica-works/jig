use super::spine::{LIMITS, delivered, drive, frame, start};
use crate::tests::fixtures::{Fixture, ReadPage};
use crate::{
    Event, Request,
    action::Act,
    address::{Address, Place, TaskSection},
    windows::HistoryWindow,
};
use jig_client::contract::*;
use jig_client::thread::window::{Edge, Edges, End};
use skein_lib::{List, Time, Token, Wall};

fn text(bytes: &[u8]) -> Text {
    Text::new(Box::from(bytes), LIMITS.text).unwrap()
}
fn change(task: u64, revision: u64) -> HistoryRow {
    HistoryRow::Change(Box::new(OwnerHistoryLine {
        task,
        revision,
        by: OwnerRequester::Deployment { project: u32::MAX },
        change: OwnerHistoryChange::Amended,
        reason: DocumentText { bytes: text(b""), cut: true },
        proposal: None,
        amendment: None,
        at: None,
    }))
}
fn milestone(task: u64, position: u64) -> HistoryRow {
    HistoryRow::Milestone(Box::new(OwnerMilestoneLine {
        task,
        position,
        phase: OwnerMilestonePhase::Active(RunStage::Claimed { attempt: u64::MAX }),
        at: Some(Wall::EPOCH),
    }))
}
fn line(task: u64, position: u64) -> HistoryRow {
    HistoryRow::Line(Box::new(HistoryLine {
        task,
        position,
        by: Actor::Party(7),
        at: Wall::from_nanos(u64::MAX),
        change: HistoryChange::Made { batch: u64::MAX },
    }))
}
fn rows(values: [HistoryRow; 2]) -> List<HistoryRow> {
    let mut rows = List::with_capacity(2);
    for value in values {
        rows.push(value).unwrap();
    }
    rows
}
fn edges() -> Edges {
    Edges { before: Edge::End, after: Edge::End }
}
#[test]
fn mixed_history_window_retains_revision_position_and_line_identity_in_source_order() {
    let semantic = change(u64::MAX, 9);
    let lifecycle = milestone(u64::MAX, 9);
    let supplied_line = line(u64::MAX, 9);
    let mut window = HistoryWindow::new(2);
    assert_eq!(window.slide(rows([semantic.clone(), lifecycle.clone()]), edges(), End::After), 0);
    assert_eq!(window.rows.as_slice(), &[semantic.clone(), lifecycle.clone()]);
    let opaque = Cursor::new(Box::from(b"opaque reverse".as_slice()), 1024).expect("bounded opaque cursor");
    let mut next = List::with_capacity(2);
    next.push(supplied_line.clone()).unwrap();
    assert_eq!(window.slide(next, Edges { before: Edge::More(opaque.clone()), after: Edge::End }, End::After), 1);
    assert_eq!(window.rows.as_slice(), &[lifecycle, supplied_line]);
    assert_eq!(window.edges.before, Edge::More(opaque));
    assert!(!window.live(rows([change(0, 0), milestone(0, 0)]), edges(), End::Before));
    assert_eq!(window.slide(rows([semantic.clone(), line(0, 0)]), edges(), End::Before), 2);
    assert_eq!(window.rows.as_slice(), &[semantic, line(0, 0)]);
    assert!(window.edges.reaches(End::Before));
}
fn history_read(requests: &[Request]) -> jig_client::Handle {
    for request in requests {
        if let Request::Client {
            asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::History { .. }) },
        } = request
        {
            return *read;
        }
    }
    panic!("original history request");
}
fn page(cut: bool, after: Option<Cursor>) -> ReadValue {
    ReadValue::History(Box::new(HistoryPage {
        cut,
        rows: rows([change(u64::MAX, 0), milestone(u64::MAX, 0)]),
        before: None,
        after,
    }))
}
#[test]
fn owner_history_original_scope_cursors_and_page_loss_survive_continuation_and_reset() {
    let mut domain = Fixture::new(&LIMITS);
    let watch = start(&mut domain, List::with_capacity(0));
    let initial = delivered(&mut domain, Time::ZERO, watch, frame(7));
    super::pages::finish_initial_inbox_reads(&mut domain, &initial);
    let original = history_read(&drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::History }) },
    ));
    let subtree =
        history_read(&drive(&mut domain, Time::ZERO, Event::Act { action: Act::HistoryScope { subtree: true } }));
    drive(&mut domain, Time::ZERO, ReadPage { read: original, result: ReadResult::Value(page(true, None)) });
    assert!(domain.history_rows().rows.is_empty());
    assert!(!domain.history_rows().cut);
    let cursor = Cursor::new(Box::from(b"whole supplied cursor".as_slice()), 1024).expect("bounded opaque cursor");
    drive(
        &mut domain,
        Time::ZERO,
        ReadPage { read: subtree, result: ReadResult::Value(page(true, Some(cursor.clone()))) },
    );
    assert_eq!(domain.history_rows().rows.as_slice(), &[change(u64::MAX, 0), milestone(u64::MAX, 0)]);
    assert!(domain.history_rows().cut);
    let more = drive(
        &mut domain,
        Time::ZERO,
        Event::Act { action: Act::More { window: Token::new(crate::page::task::HISTORY_WINDOW), end: End::After } },
    );
    let continued = history_read(&more);
    let mut matching_reads = 0_u32;
    for request in &more {
        if let Request::Client { asked: jig_client::Event::Read { read: jig_client::Readable::Jig(what), .. } } =
            request
            && *what == (Read::History { task: 8, subtree: true, after: Some(cursor.clone()) })
        {
            matching_reads += 1;
        }
    }
    assert_eq!(matching_reads, 1);
    drive(&mut domain, Time::ZERO, ReadPage { read: continued, result: ReadResult::Value(page(false, None)) });
    assert!(domain.history_rows().cut);
    assert_eq!(domain.history_rows().rows.len(), 2);
    drive(&mut domain, Time::ZERO, Event::Act { action: Act::HistoryScope { subtree: false } });
    assert!(domain.history_rows().rows.is_empty());
    assert!(!domain.history_rows().cut);
}
