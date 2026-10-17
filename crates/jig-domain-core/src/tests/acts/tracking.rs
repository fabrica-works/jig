//! Maintainer admission for tracking and priority amendments.
use super::*;

const PRIORITY_PARTY: u64 = 1;
const PRIORITY_PROJECT: u32 = 1;
const PRIORITY_TASK: u64 = 1;
const POLICY_ROLE: u32 = 4;
const TRACK_MESSAGE: u64 = 8;
const TRACK_REQUEST: u64 = 9;

fn track(core: &mut Core, limits: &Limits) {
    let mut spec = core.tasks.task(PRIORITY_TASK).expect("task fixture").spec.clone();
    spec.words = b"changed specification".as_slice().into();
    send_task(
        core,
        limits,
        tasks::Event::Amend {
            funder: None,
            reply_to: ReplyTo::new(Token::new(TRACK_REQUEST)),
            by: tasks::Party::Person(PRIORITY_PARTY),
            task: PRIORITY_TASK,
            message: TRACK_MESSAGE,
            stop_run: false,
            amendment: tasks::Amendment {
                tracked: Some(tasks::Tracking::Tracked { priority: 1 }),
                spec: Some(spec),
                ..fixtures::unchanged(b"track")
            },
        },
    );
}

fn permit_role(core: &mut Core, limits: &mut Limits, number: u32) {
    limits.authority.roles = 2;
    let mut policy = core.authority.policy(PRIORITY_PROJECT).expect("project policy").clone();
    let mut role = policy.roles[0].clone();
    role.number = number;
    policy.roles = Box::new([policy.roles[0].clone(), role]);
    core.authority = authority::Domain::new(core.authority.rules().clone(), limits.authority).expect("two roles");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut core.authority, authority::Event::Policy { project: PRIORITY_PROJECT, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: PRIORITY_PROJECT }));
}

#[test]
fn a_member_amending_a_priority_is_refused_as_not_a_maintainer() {
    let (mut core, mut limits) = core();
    make_task(&mut core, &limits, PRIORITY_TASK);
    track(&mut core, &limits);
    permit_role(&mut core, &mut limits, people::Role::Member.number());
    let change = tasks::Amendment {
        tracked: Some(tasks::Tracking::Tracked { priority: 2 }),
        ..fixtures::unchanged(b"priority")
    };
    assert_eq!(
        core.amend_admit(
            PRIORITY_PARTY,
            Some(people::Role::Member),
            PRIORITY_PROJECT,
            PRIORITY_TASK,
            &change,
            limits.tasks.depth,
            &limits.tasks
        ),
        Err(people::Refusal::Role { held: Some(people::Role::Member) })
    );
}

#[test]
fn a_policy_role_amending_tracked_is_refused() {
    let (mut core, mut limits) = core();
    make_task(&mut core, &limits, PRIORITY_TASK);
    permit_role(&mut core, &mut limits, POLICY_ROLE);
    let change =
        tasks::Amendment { tracked: Some(tasks::Tracking::Tracked { priority: 2 }), ..fixtures::unchanged(b"tracked") };
    assert_eq!(
        core.amend_admit(
            PRIORITY_PARTY,
            Some(people::Role::Policy { role: POLICY_ROLE }),
            PRIORITY_PROJECT,
            PRIORITY_TASK,
            &change,
            limits.tasks.depth,
            &limits.tasks
        ),
        Err(people::Refusal::Role { held: Some(people::Role::Policy { role: POLICY_ROLE }) })
    );
}

#[test]
fn a_priority_alone_names_no_revision() {
    let (mut core, limits) = core();
    make_task(&mut core, &limits, PRIORITY_TASK);
    track(&mut core, &limits);
    assert_eq!(core.tasks.task(PRIORITY_TASK).expect("tracked task").revisions.spec, 1);
    let change = tasks::Amendment {
        tracked: Some(tasks::Tracking::Tracked { priority: 2 }),
        ..fixtures::unchanged(b"priority")
    };
    assert_eq!(core.party_amend_revision(PRIORITY_TASK, 0, &change), Ok(()));
}
