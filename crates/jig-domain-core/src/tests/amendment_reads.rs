//! Loaded-place admission and its transient reads, scripted by an ordinary Rust parent.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "input tables and output observations are ordinary Rust tests"
)]
use super::*;
use crate::routing::amendment_reads::{Evidence, missing};

const AMENDER: u64 = 1;
const TARGET: u64 = 2;
const LIVE_ANCESTOR: u64 = 4;
const DEPENDENCY: u64 = 12;
const ENDED_PARENT: u64 = 13;
const OTHER_ROOT: u64 = 14;
const PERSON: u64 = 99;

fn place(number: u64, requester: tasks::Party, root: u64, depth: u32) -> tasks::DependencyPlace {
    tasks::DependencyPlace { number, project: 1, requester, root, depth, status: tasks::Status::Done }
}

#[test]
fn amendment_loads_stop_at_the_amenders_tree_boundary() {
    let (core, limits) = make_tree();
    let delegate = place(DEPENDENCY, tasks::Party::Task(AMENDER), AMENDER, 1);
    let descendant = place(DEPENDENCY, tasks::Party::Task(ENDED_PARENT), AMENDER, 2);
    for (label, places, expected) in [
        ("first load", vec![], Some(DEPENDENCY)),
        ("former delegate", vec![delegate], None),
        ("ended requester", vec![descendant], Some(ENDED_PARENT)),
        ("complete chain", vec![descendant, place(ENDED_PARENT, tasks::Party::Task(AMENDER), AMENDER, 1)], None),
        ("first live ancestor", vec![place(DEPENDENCY, tasks::Party::Task(LIVE_ANCESTOR), AMENDER, 2)], None),
        ("different root", vec![tasks::DependencyPlace { root: OTHER_ROOT, ..descendant }], None),
        ("above depth", vec![tasks::DependencyPlace { depth: 0, ..descendant }], None),
        ("requester at amender depth", vec![tasks::DependencyPlace { depth: 1, ..descendant }], None),
    ] {
        let mut evidence = Evidence::new(&limits.tasks);
        for row in places {
            evidence.loaded.push(row.number).unwrap();
            evidence.places.push(row).unwrap();
        }
        assert_eq!(
            missing(&core, &limits.tasks, tasks::Party::Task(AMENDER), TARGET, Some(&[DEPENDENCY]), &evidence),
            expected,
            "{label}"
        );
    }
}

#[test]
fn a_persons_load_uses_its_row_or_its_root() {
    let (core, limits) = make_tree();
    let ended = place(DEPENDENCY, tasks::Party::Task(ENDED_PARENT), OTHER_ROOT, 2);
    for (label, places, expected) in [
        ("own request", vec![tasks::DependencyPlace { requester: tasks::Party::Person(PERSON), ..ended }], None),
        ("live root", vec![tasks::DependencyPlace { root: AMENDER, ..ended }], None),
        ("ended root", vec![ended], Some(OTHER_ROOT)),
        ("root loaded", vec![ended, place(OTHER_ROOT, tasks::Party::Person(PERSON), OTHER_ROOT, 0)], None),
        ("row is root", vec![tasks::DependencyPlace { root: DEPENDENCY, ..ended }], None),
    ] {
        let mut evidence = Evidence::new(&limits.tasks);
        for row in places {
            evidence.loaded.push(row.number).unwrap();
            evidence.places.push(row).unwrap();
        }
        assert_eq!(
            missing(&core, &limits.tasks, tasks::Party::Person(PERSON), TARGET, Some(&[DEPENDENCY]), &evidence),
            expected,
            "{label}"
        );
    }
}

const ATTEMPT: u64 = 7;
const CALL: CallKey = CallKey { task: AMENDER, attempt: ATTEMPT, completion: 1, position: 0 };
const TOKEN: Token = Token::new(77);

fn agent_read(core: &mut Core, limits: &Limits) -> Token {
    core.proofs
        .insert(AMENDER, RunProof::claimed(AMENDER, ATTEMPT, ATTEMPT, None, jig_domain_fleet::HostKind::Worker))
        .unwrap();
    let mut edit = fixtures::unchanged(b"dependency");
    edit.dependencies = Some(Box::new([DEPENDENCY]));
    let out = watch_outputs(
        core,
        limits,
        Event::NamedAction {
            to: ReplyTo::new(TOKEN),
            key: CALL,
            action: NamedAction::Amend { target: TARGET, revision: 0, amendment: edit },
        },
    );
    result_reads::requested(&out, DEPENDENCY)
}

#[test]
fn busy_and_stale_amendment_loads_leave_the_target_unchanged() {
    for (label, busy, pending, proof, expected_answer) in [
        ("store busy", true, true, true, true),
        ("call gone", false, false, true, false),
        ("run gone", false, true, false, false),
    ] {
        let (mut core, limits) = make_tree();
        let before = core.tasks.task(TARGET).unwrap().clone();
        let request = agent_read(&mut core, &limits);
        if !pending {
            core.pending_calls.remove(&CALL);
        }
        if !proof {
            core.proofs.remove(&AMENDER);
        }
        let out = watch_outputs(&mut core, &limits, Event::EndedTaskLoaded { request, row: None, busy });
        let answer = out.iter().find_map(|request| match request {
            Request::Held(held) => match held.as_ref() {
                Held::CallAnswer { part, .. } => Some(part),
                _ => None,
            },
            _ => None,
        });
        assert_eq!(answer.is_some(), expected_answer, "{label}");
        if let Some(answer) = answer {
            assert_eq!(
                answer,
                &CallPart::ControlRefused(tasks::Problem::new(Some(TARGET), tasks::Refusal::Busy)),
                "{label}"
            );
        }
        assert_eq!(core.tasks.task(TARGET), Some(&before), "{label}");
        assert!(core.result_reads.is_empty(), "{label}");
    }
}

#[test]
fn owner_cleanup_drops_all_waiting_amendments_and_late_answers() {
    let (mut core, limits) = make_tree();
    let request = agent_read(&mut core, &limits);
    // Use the same owner cleanup as preparation and run departure.
    routing::result_reads::clear(&mut core, AMENDER);
    assert!(core.result_reads.is_empty());
    assert!(!core.pending_calls.contains_key(&CALL));
    let out = watch_outputs(&mut core, &limits, Event::EndedTaskLoaded { request, row: None, busy: false });
    for request in &out {
        let Request::Decided = request else { panic!("late answer produced {request:?}") };
    }
    assert!(!core.call_parts.contains_key(&CALL));
}

#[test]
fn a_replay_loads_an_ancestor_that_has_ended_meanwhile() {
    let (mut core, limits) = make_tree();
    let mut evidence = Evidence::new(&limits.tasks);
    evidence.loaded.push(DEPENDENCY).unwrap();
    evidence.places.push(place(DEPENDENCY, tasks::Party::Task(LIVE_ANCESTOR), AMENDER, 2)).unwrap();
    assert_eq!(
        missing(&core, &limits.tasks, tasks::Party::Task(AMENDER), TARGET, Some(&[DEPENDENCY]), &evidence),
        None
    );
    let rows = [AMENDER, TARGET].map(|number| core.tasks.task(number).unwrap().clone());
    fixtures::restore(&mut core, &limits, Box::new(rows));
    assert_eq!(
        missing(&core, &limits.tasks, tasks::Party::Task(AMENDER), TARGET, Some(&[DEPENDENCY]), &evidence),
        Some(LIVE_ANCESTOR)
    );
}
