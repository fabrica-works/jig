//! Loaded-place admission and its transient reads, scripted by an ordinary Rust parent.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "input tables and output observations are ordinary Rust tests"
)]
use super::{Harness, amendment, task, to};
use crate::{DependencyPlace, Event, Funder, Party, Problem, Refusal, Request, Status};

const ROOT: u64 = 10;
const AMENDER: u64 = 1;
const TARGET: u64 = 2;
const ANCESTOR: u64 = 3;
const DEPENDENCY: u64 = 9;
const PERSON: u64 = 9;

fn wealthy(number: u64) -> crate::New {
    let mut row = task(number, &[]);
    row.numbers.budget = 1000;
    row.authority.budget.spend = 1000;
    row
}

fn family() -> Harness {
    let mut test = Harness::new();
    test.make(vec![wealthy(ROOT)]);
    let mut parent = wealthy(AMENDER);
    parent.funder = Funder::Task(ROOT);
    test.send(Event::Make { reply_to: to(), creator: Party::Task(ROOT), batch: Box::new([parent]) });
    for (requester, number) in [(AMENDER, TARGET), (AMENDER, ANCESTOR)] {
        let mut child = task(number, &[]);
        child.funder = Funder::Task(requester);
        test.send(Event::Make { reply_to: to(), creator: Party::Task(requester), batch: Box::new([child]) });
    }
    test
}

fn place(number: u64, requester: Party, root: u64, depth: u32, status: Status) -> DependencyPlace {
    DependencyPlace { number, project: 1, requester, root, depth, status }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one data-driven table carries every place and expected verdict")]
fn ended_dependencies_are_judged_only_on_their_places() {
    let done = place(DEPENDENCY, Party::Task(AMENDER), ROOT, 2, Status::Done);
    let refused = Some(Problem::naming(Some(TARGET), Refusal::Dependencies, &[DEPENDENCY]));
    // Each row carries evidence, current introductions and its entire expected verdict.
    for (label, by, places, references, expected) in [
        (
            "introduced",
            Party::Task(AMENDER),
            vec![place(DEPENDENCY, Party::Person(PERSON), 90, 0, Status::Done)],
            vec![DEPENDENCY],
            None,
        ),
        (
            "requester chain",
            Party::Task(AMENDER),
            vec![
                place(DEPENDENCY, Party::Task(8), ROOT, 3, Status::Done),
                place(8, Party::Task(AMENDER), ROOT, 2, Status::Done),
            ],
            vec![],
            None,
        ),
        (
            "live ancestor below",
            Party::Task(AMENDER),
            vec![place(DEPENDENCY, Party::Task(ANCESTOR), ROOT, 3, Status::Done)],
            vec![],
            None,
        ),
        (
            "above amender",
            Party::Task(AMENDER),
            vec![place(DEPENDENCY, Party::Task(AMENDER), ROOT, 0, Status::Done)],
            vec![],
            refused.clone(),
        ),
        (
            "different root",
            Party::Task(AMENDER),
            vec![place(DEPENDENCY, Party::Task(AMENDER), 90, 1, Status::Done)],
            vec![],
            refused.clone(),
        ),
        (
            "person root request",
            Party::Person(PERSON),
            vec![
                place(DEPENDENCY, Party::Task(8), 8, 1, Status::Done),
                place(8, Party::Person(PERSON), 8, 0, Status::Done),
            ],
            vec![],
            None,
        ),
        (
            "maintainer project",
            Party::Person(42),
            vec![place(DEPENDENCY, Party::Person(42), 90, 0, Status::Done)],
            vec![],
            None,
        ),
        ("another project", Party::Person(42), vec![DependencyPlace { project: 2, ..done }], vec![], refused.clone()),
        (
            "unknown names dependency",
            Party::Task(AMENDER),
            vec![],
            vec![],
            Some(Problem::new(Some(DEPENDENCY), Refusal::Unknown)),
        ),
        ("done met", Party::Task(AMENDER), vec![done], vec![], None),
        (
            "failed",
            Party::Task(AMENDER),
            vec![DependencyPlace { status: Status::Failed, ..done }],
            vec![],
            Some(Problem::naming(Some(TARGET), Refusal::DependencyEnded { how: Status::Failed }, &[DEPENDENCY])),
        ),
        (
            "cancelled",
            Party::Task(AMENDER),
            vec![DependencyPlace { status: Status::Cancelled, ..done }],
            vec![],
            Some(Problem::naming(Some(TARGET), Refusal::DependencyEnded { how: Status::Cancelled }, &[DEPENDENCY])),
        ),
    ] {
        let mut test = family();
        // An unrelated name or retained stub is never reach evidence.
        test.domain
            .stubs
            .insert(
                DEPENDENCY,
                crate::Stub { task: DEPENDENCY, phase: Status::Done, result: skein_lib::Token::new(DEPENDENCY) },
            )
            .unwrap();
        let id = *test.domain.names.get(&AMENDER).unwrap();
        test.domain.tasks.get_mut(id).unwrap().record.references = references.into_boxed_slice();
        let before = test.stored.clone();
        let rows = [ROOT, AMENDER, TARGET, ANCESTOR].map(|number| test.domain.task(number).unwrap().clone());
        let stub = *test.domain.stub(DEPENDENCY).unwrap();
        let mut change = amendment();
        change.dependencies = Some(Box::new([DEPENDENCY]));
        let out = test.send(Event::Amend {
            places: places.into_boxed_slice(),
            reply_to: to(),
            by,
            task: TARGET,
            message: 1,
            stop_run: false,
            funder: None,
            amendment: change,
        });
        let problem = out.iter().find_map(|request| match request {
            Request::Refused { problem, .. } => Some(problem.clone()),
            _ => None,
        });
        assert_eq!(problem, expected, "{label}: {out:?}");
        if expected.is_some() {
            assert_eq!(test.stored, before, "{label}");
            assert_eq!(
                [ROOT, AMENDER, TARGET, ANCESTOR].map(|number| test.domain.task(number).unwrap().clone()),
                rows,
                "{label}"
            );
            assert_eq!(test.domain.stub(DEPENDENCY), Some(&stub), "{label}");
        } else {
            assert!(
                out.iter().any(|request| match request {
                    Request::Done { .. } => true,
                    _ => false,
                }),
                "{label}: {out:?}"
            );
            let row = test.domain.task(TARGET).unwrap();
            assert_eq!(row.dependencies.as_ref(), [DEPENDENCY], "{label}");
            assert!(row.waiting_on.is_empty(), "{label}");
            assert_eq!(row.added.as_ref(), [crate::Added { task: DEPENDENCY, at: test.env.wall, by }], "{label}");
        }
    }
}
