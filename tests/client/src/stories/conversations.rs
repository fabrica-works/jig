//! A chat's list fixtures use the same task facts as the real owner's query.
use super::{frame, list, task, text};
use jig_client_domain::contract::*;
use skein_lib::{List, Wall};

#[must_use]
pub fn templates() -> List<Template> {
    let chat = Template {
        llm: None,
        number: jig_client::shapes::CHAT,
        name: DocumentText::complete(text(b"Chat")),
        executor: TemplateExecutor::Charter(1),
        contract: TaskContractDocument::Report { words: 32 },
        shallower: 1,
        budget: TemplateBudget::AsAllowed { cap: None },
        deadline: TemplateDeadline::None,
        tracked: None,
        flags: TaskFlags { conversation: true, listed: false, retention: Retention::Kept },
        label: 31,
        grace: None,
    };
    let mut goal = chat.clone();
    goal.number = jig_client::shapes::GOAL;
    goal.executor = TemplateExecutor::Chosen;
    goal.budget = TemplateBudget::Given;
    goal.deadline = TemplateDeadline::Given;
    goal.label = 2;
    goal.tracked = Some(0);
    goal.flags.conversation = false;
    goal.flags.listed = true;
    list([chat, goal])
}
#[must_use]
pub fn chat(number: u64, title: &[u8], activity: Wall) -> Task {
    let mut task = task();
    task.chip.task = number;
    task.chip.title = DocumentText::complete(text(title));
    task.chip.phase = Phase::Idle;
    task.label = templates().get(0).expect("chat template").label;
    task.last_activity = activity;
    task.flags.conversation = true;
    task.priority = None;
    task.chip.details = Box::new(ChipPhase::Owner {
        phase: OwnerChipPhase::Active(RunStage::Idle),
        details: Box::new(OwnerChipDetails {
            dependencies: skein_lib::List::with_capacity(0),
            project: task.project,
            executor: OwnerExecutor::Agent { charter: 1 },
            phase_since: Wall::EPOCH,
            run_since: None,
            attempt: None,
            retry: None,
            wake: task.wake,
            person_wait: None,
            hold: None,
            budget: task.budget.clone(),
            activity: task.activity,
            resources: list([]),
            resources_cut: false,
            resources_observed: false,
            procedure: None,
            live_delegates: 0,
            closing_connectors: None,
            ending: None,
            ended_at: None,
            observation: None,
        }),
    });
    task
}
#[must_use]
pub fn list_world(seed: u64, tasks: &[Task]) -> super::World<super::Test, super::Scripted> {
    let mut world = super::world(seed);
    for task in tasks {
        world
            .front()
            .engine
            .act(
                skein_lib::Time::ZERO,
                Wall::EPOCH,
                jig_client_scripted_engine::Change::Object {
                    project: task.project,
                    key: ObjectKey::Task(task.chip.task),
                    value: ReadValue::Task(Box::new(task.clone())),
                },
            )
            .expect("fixture task admitted");
    }
    world
}
#[must_use]
pub fn without_chats(seed: u64) -> super::World<super::Test, super::Scripted> {
    let mut world = super::world(seed);
    let mut me = frame(7);
    for index in 0..me.projects.len() {
        me.projects.get_mut(index).expect("project index").templates.clear();
    }
    world
        .front()
        .engine
        .act(skein_lib::Time::ZERO, Wall::EPOCH, jig_client_scripted_engine::Change::Me(me))
        .expect("fixture me admitted");
    world
}
