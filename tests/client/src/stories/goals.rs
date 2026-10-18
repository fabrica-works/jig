//! Goals board fixtures use tasks and the same task query as the real owner.
use super::*;
pub const FIRST: u64 = 19;
pub const SECOND: u64 = 20;
pub const PROJECT: u32 = 1;
pub const PARTY: u64 = 7;

#[must_use]
pub fn address(section: jig_client_view::address::ProjectSection) -> RootAddress {
    RootAddress::Jig(Address::Project { project: PROJECT, section })
}
#[must_use]
pub fn board(seed: u64) -> World<Test, Scripted> {
    let mut first = super::task();
    first.chip.task = FIRST;
    first.chip.title = DocumentText::complete(text(b"First goal"));
    first.priority = Some(0);
    first.label = 2;
    let mut second = first.clone();
    second.chip.task = SECOND;
    second.chip.title = DocumentText::complete(text(b"Second goal"));
    second.priority = Some(1);
    let mut world = conversations::list_world(seed, &[first, second]);
    let mut project = super::project();
    project.acts = list([Offered { act: ActKind::Prioritise, admission: Admission::Allowed, effect: None }]);
    world
        .front()
        .engine
        .act(skein_lib::Time::ZERO, Wall::EPOCH, jig_client_scripted_engine::Change::Project(Box::new(project)))
        .expect("fixture admitted");
    world
}

#[must_use]
pub fn amendment() -> Amendment {
    Amendment {
        addressed: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        instructions: Some(text(b"changed instructions")),
        tracked: None,
    }
}
#[must_use]
pub fn amend_request() -> Ask {
    Ask::Amend {
        project: PROJECT,
        task: FIRST,
        revision: Some(Revision { value: 1 }),
        change: Box::new(amendment()),
        reason: text(b"because"),
    }
}
#[must_use]
pub fn refused(seed: u64, proposable: Option<bool>) -> World<Test, Scripted> {
    let mut task = super::task();
    task.chip.task = FIRST;
    task.acts = list([Offered { act: ActKind::Amend, admission: Admission::Allowed, effect: None }]);
    let mut world = conversations::list_world(seed, &[task]);
    world
        .front()
        .engine
        .script(jig_client_scripted_engine::Rule {
            party: Some(PARTY),
            ask: amend_request(),
            answer: Answer::Refused {
                refusal: Refusal::Authority { lacking: Lacking::Budget { needed: 2, left: 1 }, proposable },
                at: None,
            },
            changes: vec![],
        })
        .expect("fixture admitted");
    let proposal = super::proposal();
    world
        .front()
        .engine
        .script(jig_client_scripted_engine::Rule {
            party: Some(PARTY),
            ask: Ask::Propose {
                project: PROJECT,
                action: ProposeAction::Amendment {
                    task: FIRST,
                    revision: Revision { value: 1 },
                    change: Box::new(amendment()),
                },
                reason: text(b"because"),
            },
            answer: Answer::Done(Outcome::Proposed { proposal: proposal.key }),
            changes: vec![jig_client_scripted_engine::Change::Object {
                project: PROJECT,
                key: ObjectKey::Proposal(proposal.key),
                value: ReadValue::Proposal(Box::new(proposal)),
            }],
        })
        .expect("fixture admitted");
    world
}
pub fn offer_proposal(world: &mut World<Test, Scripted>) {
    let mut task = super::task();
    task.chip.task = FIRST;
    task.chip.title = DocumentText::complete(text(b"Amendment now proposable"));
    task.acts = list([Offered {
        act: ActKind::Amend,
        effect: None,
        admission: Admission::Bounded {
            within: jig_client::contract::Bounds::Amend {
                budget_left: 1,
                grants: list([]),
                implications: list([]),
                authority: Box::new(task.authority.clone()),
                ceiling: Box::new(task.authority.clone()),
            },
            beyond: Beyond::Proposed { to: Recipient::Role { project: PROJECT, role: 1 } },
        },
    }]);
    let now = world.now();
    world
        .front()
        .engine
        .act(
            now,
            Wall::EPOCH,
            jig_client_scripted_engine::Change::Object {
                project: PROJECT,
                key: ObjectKey::Task(FIRST),
                value: ReadValue::Task(Box::new(task)),
            },
        )
        .expect("fixture admitted");
}

pub(super) fn creating() -> Creating {
    Creating {
        within: CreateBounds {
            authority: Box::new(owner_authority()),
            pool_left: 1,
            charters: list([1]),
            procedures: list([]),
            latest: None,
            budget: 1,
            maximum_budget: 100,
            maximum_deadline: None,
        },
        beyond: Beyond::Proposed { to: Recipient::Role { project: 1, role: 1 } },
    }
}
pub(super) fn ask(me: &Me, executor: OwnerExecutor) -> Ask {
    match jig_client::shapes::set_goal(
        me,
        1,
        jig_client::shapes::GoalDraft {
            words: text(b"Resolve incident"),
            executor,
            budget: 2,
            deadline: None,
            priority: Some(0),
        },
        text(b""),
        Wall::EPOCH,
    ) {
        jig_client::shapes::Judged::Propose { ask, .. } => ask,
        value @ (jig_client::shapes::Judged::Create(_)
        | jig_client::shapes::Judged::Refused(_)
        | jig_client::shapes::Judged::NotOffered(_)) => panic!("fixture goal must be proposed: {value:?}"),
    }
}
