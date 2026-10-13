//! Activity pages retain the one turn supplied beside each line.
use super::*;
fn line() -> tasks::ActivityLine {
    tasks::ActivityLine {
        task: 1,
        position: 3,
        at: Wall::from_nanos(19),
        line: tasks::Line::Turn { reply: 2, attempt: 1, turn: 2, read: Some(12) },
    }
}
fn turn() -> TurnRecord {
    TurnRecord {
        task: 1,
        attempt: 1,
        turn: 2,
        opens_window: false,
        spent: 17,
        read: Some(12),
        at: Wall::from_nanos(19),
        transcript: Box::new(*b"opaque"),
    }
}
fn size(task: u64, attempt: u64, turn: u32) -> reads::UnjoinedTurn {
    reads::UnjoinedTurn::TooLarge {
        task,
        attempt,
        turn,
        spent: 17,
        read: Some(12),
        at: Wall::from_nanos(19),
        bytes: 8192,
    }
}
fn answer(
    core: &mut Core,
    limits: &Limits,
    related: Option<Record>,
    unjoined: Option<reads::UnjoinedTurn>,
) -> reads::Answer {
    let owner = core.tasks.task(1).expect("visible task").clone();
    let mut row = stored_row(Record::Tasks(tasks::Stored::Activity(line())), Some(owner));
    row.related = related;
    row.unjoined = unjoined;
    let Held::ReadLoad { load, .. } = read_start(core, limits, 10, reads::Read::Activity { task: 1, before: None })
    else {
        panic!("activity load")
    };
    let answer = loaded_answer(core, limits, load, reads::Loaded::Page { rows: Box::new([row]), next: None })
        .expect("one held terminal");
    core.reclaim();
    answer
}
#[test]
fn an_activity_row_keeps_its_turn() {
    let (mut core, limits) = watch_core();
    let cases = [
        ("carried", Some(Record::Core(CoreRecord::Turn(turn()))), None, reads::PageTurn::Carried(turn())),
        (
            "too large",
            None,
            Some(size(1, 1, 2)),
            reads::PageTurn::TooLarge { spent: 17, read: Some(12), at: Wall::from_nanos(19), bytes: 8192 },
        ),
        ("unread", None, Some(reads::UnjoinedTurn::Unread { task: 1, attempt: 1, turn: 2 }), reads::PageTurn::Unread),
    ];
    for (label, related, unjoined, expected) in cases {
        let reads::Answer::Page(page) = answer(&mut core, &limits, related, unjoined) else { panic!("{label}") };
        assert_eq!(page.rows.get(0), Some(&reads::PageRow::Activity { line: line(), turn: Some(expected) }), "{label}");
    }
}
#[test]
fn a_turn_of_another_line_is_unknown() {
    for (label, task, attempt, number) in [("task", 2, 1, 2), ("attempt", 1, 2, 2), ("turn", 1, 1, 3)] {
        for (carrier, related, unjoined) in [
            (
                "carried",
                Some(Record::Core(CoreRecord::Turn(TurnRecord { task, attempt, turn: number, ..turn() }))),
                None,
            ),
            ("too large", None, Some(size(task, attempt, number))),
            ("unread", None, Some(reads::UnjoinedTurn::Unread { task, attempt, turn: number })),
        ] {
            let mut row = stored_row(Record::Tasks(tasks::Stored::Activity(line())), None);
            row.related = related;
            row.unjoined = unjoined;
            assert_eq!(reads::activity::row(&line(), &row), Err(Missing::Unknown), "{carrier} {label}");
        }
    }
}
