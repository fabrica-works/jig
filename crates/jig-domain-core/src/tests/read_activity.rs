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

fn failure(owner: Box<tasks::TaskRecord>, why: reads::TurnFailure) -> reads::Loaded {
    reads::Loaded::TurnProblem { task: 1, attempt: 1, turn: 2, owner, why }
}
fn turn_load(core: &mut Core, limits: &Limits) -> Token {
    let Held::ReadLoad { load, .. } = read_start(core, limits, 10, reads::Read::Turn { task: 1, attempt: 1, turn: 2 })
    else {
        panic!("turn load")
    };
    load
}
#[test]
fn an_invisible_turns_failure_is_unknown() {
    for why in [reads::TurnFailure::Malformed, reads::TurnFailure::TooLarge { bytes: 8192 }] {
        let (mut core, limits) = watch_core();
        let mut owner = Box::new(core.tasks.task(1).expect("owner").clone());
        owner.project = 2;
        let load = turn_load(&mut core, &limits);
        assert_eq!(
            loaded_answer(&mut core, &limits, load, failure(owner, why)),
            Some(reads::Answer::None(reads::Why::Unknown)),
            "{why:?}"
        );
    }
}
#[test]
fn a_turn_failure_rechecks_its_sign_in() {
    let (mut core, limits) = watch_core();
    let owner = Box::new(core.tasks.task(1).expect("owner").clone());
    let load = turn_load(&mut core, &limits);
    watch_outputs(
        &mut core,
        &limits,
        Event::People(people::Event::SignOut { sign_in: 10, reply_to: ReplyTo::new(Token::new(91)) }),
    );
    assert_eq!(
        loaded_answer(&mut core, &limits, load, failure(owner, reads::TurnFailure::Malformed)),
        Some(reads::Answer::None(reads::Why::SignedOut))
    );
}
#[test]
fn a_mismatched_turn_failure_is_unknown() {
    for (label, task, attempt, turn) in [("task", 2, 1, 2), ("attempt", 1, 2, 2), ("turn", 1, 1, 3)] {
        let (mut core, limits) = watch_core();
        let owner = Box::new(core.tasks.task(1).expect("owner").clone());
        let load = turn_load(&mut core, &limits);
        let result = reads::Loaded::TurnProblem { task, attempt, turn, owner, why: reads::TurnFailure::Malformed };
        assert_eq!(
            loaded_answer(&mut core, &limits, load, result),
            Some(reads::Answer::None(reads::Why::Unknown)),
            "{label}"
        );
    }
}
#[test]
fn a_visible_turn_failure_keeps_its_reason_within_its_bound() {
    for (why, expected, tiny) in [
        (reads::TurnFailure::Malformed, reads::Why::Malformed, false),
        (reads::TurnFailure::TooLarge { bytes: 8192 }, reads::Why::TooLarge { bytes: 8192 }, false),
        (reads::TurnFailure::Malformed, reads::Why::Busy, true),
    ] {
        let (mut core, mut limits) = watch_core();
        if tiny {
            limits.load_bytes = 1;
        }
        let owner = Box::new(core.tasks.task(1).expect("owner").clone());
        let load = turn_load(&mut core, &limits);
        assert_eq!(
            loaded_answer(&mut core, &limits, load, failure(owner, why)),
            Some(reads::Answer::None(expected)),
            "{why:?} tiny={tiny}"
        );
        assert_eq!(loaded_answer(&mut core, &limits, load, reads::Loaded::Busy), None);
    }
}
