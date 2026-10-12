//! Frame deployment membership comes from the same source as deployment admission,
//! independently of bounded project rows and their loss.
#![expect(
    clippy::disallowed_macros,
    reason = "producer proofs are ordinary Rust under programming-model.md section 10.2"
)]
use super::*;

fn source(deployment_role: Option<people::Role>) -> (Core, Limits) {
    let (mut core, mut limits) = core();
    limits.people.projects = 3;
    let identity = core.people.identity(1).unwrap().clone();
    core.people = people::Domain::new(&limits.people, Box::new([]), core.settings.deployment_provider);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    let mut rows = vec![
        people::Stored::Person { number: 1, identity },
        people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Policy { role: 4 } }]),
        },
        people::Stored::Roles {
            project: u32::MAX,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
    ];
    if let Some(role) = deployment_role {
        rows.push(people::Stored::Roles { project: 0, holdings: Box::new([people::Holding { person: 1, role }]) });
    }
    for record in rows {
        people::step(&mut core.people, &env, people::Event::Restore { record }, &mut out);
        assert_eq!(out.pop(), None, "retained membership source accepted");
    }
    people::step(&mut core.people, &env, people::Event::Restored, &mut out);
    assert_eq!(out.pop(), None, "membership source restored");
    (core, limits)
}

#[expect(clippy::wildcard_enum_match_arm, reason = "ordinary proof helper receives a Frame document")]
fn frame(core: &Core, limits: &Limits) -> Frame {
    match make(core, limits, 1, &Subject::Frame { party: 1 }, &window()).expect("frame document") {
        Document::Frame(value) => value,
        _ => panic!("frame document"),
    }
}

#[test]
fn frame_deployment_role_preserves_every_builtin_and_admitted_policy_and_full_width_number() {
    for role in [
        people::Role::Owner,
        people::Role::Maintainer,
        people::Role::Member,
        people::Role::Observer,
        people::Role::Policy { role: 4 },
        people::Role::Policy { role: u32::MAX },
    ] {
        let (core, limits) = source(Some(role));
        let value = frame(&core, &limits);
        assert_eq!(value.deployment_role, Some(role));
        assert_eq!(value.party, 1);
        assert_eq!(value.login.bytes.as_ref(), b"owner");
        assert_eq!(value.name.bytes.as_ref(), b"A Person");
        assert_eq!(value.retention, limits.people.request_retention);
        assert!(!value.cut);
        assert_eq!(value.projects.get(0).unwrap().role, role);
        assert_eq!(value.projects.get(1).unwrap().role, people::Role::Policy { role: 4 });
        assert_eq!(value.projects.get(2).unwrap().project, u32::MAX);
        let mut held = u64::try_from(size_of::<Document>()).unwrap()
            + List::<FrameProjectRole>::worst_case(value.projects.capacity()).unwrap()
            + List::<OfferedCharter>::worst_case(value.charters.capacity()).unwrap()
            + u64::try_from(value.login.bytes.len() + value.name.bytes.len()).unwrap();
        for charters in &value.charters {
            held += List::<u32>::worst_case(charters.goals.capacity()).unwrap();
        }
        assert!(document_worst_case(&limits.documents).unwrap() >= held);
        let Ok(Document::Deployment(_)) = make(&core, &limits, 1, &Subject::Deployment, &window()) else {
            panic!("deployment document admitted");
        };
    }
}

#[test]
fn frame_deployment_role_is_independent_of_clipped_project_rows_and_never_inferred_from_another_project() {
    for supplied in [None, Some(people::Role::Member), Some(people::Role::Policy { role: u32::MAX })] {
        let (core, mut limits) = source(supplied);
        limits.documents.lines = 1;
        let value = frame(&core, &limits);
        assert_eq!(value.deployment_role, supplied);
        assert!(value.cut);
        assert_eq!(value.projects.len(), 1);
        assert_eq!(value.projects.capacity(), 1);
        assert_eq!(value.charters.len(), 1);
        match supplied {
            Some(role) => {
                assert_eq!(value.projects.get(0).unwrap().project, 0);
                assert_eq!(value.projects.get(0).unwrap().role, role);
                let Ok(Document::Deployment(_)) = make(&core, &limits, 1, &Subject::Deployment, &window()) else {
                    panic!("clipped deployment document admitted");
                };
            }
            None => {
                assert_eq!(value.projects.get(0).unwrap().project, 1);
                assert_eq!(value.projects.get(0).unwrap().role, people::Role::Policy { role: 4 });
                assert_eq!(make(&core, &limits, 1, &Subject::Deployment, &window()), Err(Missing::NotAllowed));
            }
        }
    }
}

#[test]
fn frame_deployment_role_changes_only_when_its_own_retained_membership_changes() {
    let (mut core, limits) = source(None);
    assert_eq!(frame(&core, &limits).deployment_role, None);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    people::step(
        &mut core.people,
        &env,
        people::Event::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Policy { role: u32::MAX } }]),
        },
        &mut out,
    );
    let Some(people::Request::Save { record: people::Stored::Roles { project: 1, .. } }) = out.pop() else {
        panic!("project roles saved");
    };
    assert_eq!(out.pop(), None);
    assert_eq!(frame(&core, &limits).deployment_role, None);
    people::step(
        &mut core.people,
        &env,
        people::Event::Roles {
            project: 0,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Observer }]),
        },
        &mut out,
    );
    let Some(people::Request::Save { record: people::Stored::Roles { project: 0, .. } }) = out.pop() else {
        panic!("deployment roles saved");
    };
    assert_eq!(out.pop(), None);
    assert_eq!(frame(&core, &limits).deployment_role, Some(people::Role::Observer));
    people::step(&mut core.people, &env, people::Event::Roles { project: 0, holdings: Box::new([]) }, &mut out);
    let Some(people::Request::Save { record: people::Stored::Roles { project: 0, .. } }) = out.pop() else {
        panic!("deployment roles released");
    };
    assert_eq!(out.pop(), None);
    assert_eq!(frame(&core, &limits).deployment_role, None);
    assert_eq!(make(&core, &limits, 1, &Subject::Deployment, &window()), Err(Missing::NotAllowed));
}

#[test]
fn frame_deployment_role_does_not_invent_a_reserved_policy_number_refused_by_its_source() {
    let (mut core, limits) = source(None);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    people::step(
        &mut core.people,
        &env,
        people::Event::Roles {
            project: 0,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Policy { role: 0 } }]),
        },
        &mut out,
    );
    let Some(people::Request::RolesRefused { project: 0, .. }) = out.pop() else {
        panic!("reserved policy refused");
    };
    assert_eq!(out.pop(), None);
    assert_eq!(core.people.role(1, 0), None);
    assert_eq!(frame(&core, &limits).deployment_role, None);
    assert_eq!(make(&core, &limits, 1, &Subject::Deployment, &window()), Err(Missing::NotAllowed));
}
