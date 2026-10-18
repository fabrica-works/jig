//! Whole independent values for the family codec tests.
use jig_test_connector as d;
use skein_lib::{List, Wall};
pub const PROJECT: u32 = 7;
pub const REQUEST: d::ApplicationRequest =
    d::ApplicationRequest { object: u64::MAX, revision: 17, value: 0, release: None };
pub const RELEASE: d::ApplicationRelease = d::ApplicationRelease { task: u64::MAX, revision: 31, again: None };
pub const OBJECT: d::ApplicationObject = d::ApplicationObject { revision: 18, value: u64::MAX };
pub fn limits() -> d::Limits {
    d::Limits {
        resources: 2,
        resources_per_task: 2,
        resources_per_effect: 1,
        entries: 2,
        path_segments: 2,
        segment_bytes: 16,
        ..crate::limits::SMALL
    }
}
pub fn path() -> d::Path {
    d::Path::new(list([b"one".as_slice().into(), [0, 255].as_slice().into()]), limits().segment_bytes)
        .expect("literal opaque path")
}
pub fn list<T, const N: usize>(rows: [T; N]) -> List<T> {
    let mut result = List::with_capacity(u32::try_from(N).expect("test list"));
    for row in rows {
        assert!(result.push(row).is_ok(), "same count");
    }
    result
}
pub fn effect() -> d::Effect {
    d::Effect { kind: 9, resources: Box::new([path()]), purpose: u64::MAX, condition: Some(3), target: 4, state: 5 }
}
pub fn description() -> d::Description {
    d::Description {
        kind: 9,
        resources: Box::new([path()]),
        purpose: u64::MAX,
        condition: true,
        form: d::Form::Transition,
        recovery: d::Recovery::Unrecoverable,
        price: Some(0),
        state: 5,
    }
}
pub fn entry() -> d::OutboxEntry {
    d::OutboxEntry {
        number: 10,
        task: 11,
        key: d::Key { attempt: 12, completion: 13, position: 14, deployment: [15; 16], task: 16, purpose: 17 },
        effect: effect(),
        attempt: Some(d::Attempt { number: 18, sent: Wall::from_nanos(19), deadline: Wall::from_nanos(20) }),
        phase: d::EffectPhase::Uncertain,
    }
}
pub fn resource() -> d::ClientResource {
    d::ClientResource {
        path: path(),
        role: d::ResourceRole::Participating,
        hold: d::Hold::Pooled { mode: d::HoldMode::Wait },
        fact: Some(d::Fact { state: Some(21), observed: Wall::from_nanos(22), pending: true }),
    }
}
pub fn documents() -> [(&'static str, d::ClientDocument); 8] {
    [
        ("object", d::ClientDocument::Object { object: 1, now: OBJECT }),
        (
            "panel",
            d::ClientDocument::Panel {
                task: 2,
                procedure: Some(d::ProcedureState {
                    project: PROJECT,
                    task: 2,
                    number: 3,
                    resource: path(),
                    index: 4,
                    steps: 5,
                    awaiting: true,
                    delegate: None,
                    revision: 0,
                    deadline: Some(Wall::from_nanos(6)),
                }),
                result: Some(7),
                resources: list([resource()]),
                effects: list([entry()]),
            },
        ),
        ("result", d::ClientDocument::Result { task: 8, code: u16::MAX }),
        ("effect", d::ClientDocument::Effect { proposal: 9, task: 10, effect: effect(), description: description() }),
        ("outbox", d::ClientDocument::Outbox { entry: entry(), description: description() }),
        ("hold", d::ClientDocument::Hold { task: 11, resources: list([resource()]), effects: list([entry()]) }),
        ("project", d::ClientDocument::Project { project: PROJECT, resources: list([resource()]) }),
        (
            "deployment",
            d::ClientDocument::Deployment {
                outbox: list([entry()]),
                subject_limit: 12,
                staged_limit: 13,
                request_budget: d::DiagnosticAvailability::NotKept,
                system_last_response: d::DiagnosticAvailability::NotKept,
            },
        ),
    ]
}
pub fn subjects() -> [(&'static str, d::ClientSubject); 8] {
    [
        ("object", d::ClientSubject::Object { project: PROJECT, object: u64::MAX }),
        ("panel", d::ClientSubject::Panel { project: PROJECT, task: 2 }),
        ("result", d::ClientSubject::Result { project: PROJECT, task: 3 }),
        ("effect", d::ClientSubject::Effect { project: PROJECT, proposal: 4 }),
        ("outbox", d::ClientSubject::Outbox { project: PROJECT, entry: 5 }),
        ("hold", d::ClientSubject::Hold { project: PROJECT, task: 6 }),
        ("project", d::ClientSubject::Project { project: PROJECT }),
        ("deployment", d::ClientSubject::Deployment),
    ]
}

pub fn outcomes() -> List<(&'static str, d::ApplicationOutcome)> {
    use d::application_core::{Ending, HoldReason, Lack, LimitKind, Moved, Party, Refusal};
    let mut cases = List::with_capacity(16);
    for case in [
        ("changed", d::ApplicationOutcome::Changed { object: 1, now: OBJECT }),
        ("moved", d::ApplicationOutcome::Moved { object: 2, now: OBJECT }),
        ("role", d::ApplicationOutcome::Role { held: 3 }),
        ("unknown", d::ApplicationOutcome::Unknown),
        ("exhausted", d::ApplicationOutcome::Exhausted),
    ] {
        cases.push(case).expect("16 terminals");
    }
    for (label, why) in [
        ("role absent", Refusal::Role { held: None }),
        ("authority", Refusal::Authority { lacking: Lack::Budget { needs: 5, left: 6 }, proposable: Some(false) }),
        ("limit", Refusal::Limit { which: LimitKind::Authority }),
        ("moved hold", Refusal::Moved { now: Moved::Hold { revision: 7, reason: HoldReason::PoolLost { pool: 8 } } }),
        (
            "ended",
            Refusal::Ended {
                how: Ending::Cancelled { by: Party::Deployment { project: PROJECT }, at: Wall::from_nanos(9) },
            },
        ),
        ("taken", Refusal::Taken { by: 10 }),
        ("not waiting", Refusal::NotWaiting),
        ("no further", Refusal::NoFurther),
        ("unknown core", Refusal::Unknown),
        ("not offered", Refusal::NotOffered),
        ("key conflict", Refusal::KeyConflict),
    ] {
        cases.push((label, d::ApplicationOutcome::CoreRefused { why })).expect("16 terminals");
    }
    cases
}
