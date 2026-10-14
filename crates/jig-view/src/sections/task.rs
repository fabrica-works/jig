//! The universal task page and executor sections (client/domain/pages.md, 5).
use crate::{
    Builder, JigBindings, Role, cards,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_domain_client::{
    Domain,
    action::Act,
    address::{Address, TaskSection},
    contract::{Executor, ObjectKey, Task},
    objects::Body,
};
mod history;
mod plan;
/// The task's shared card and context, while any of its sections shows.
pub fn header(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) -> bool {
    let Some(number) = shown(client) else {
        return false;
    };
    b.open(Role::Region);
    b.region(crate::regions::JigRegion::TaskHeader.region());
    b.name(&words.say(JigPhrase::TaskPage, &[Hole::Number(number)]));
    let Some(id) = client.task_page().header else {
        words.status(b, JigPhrase::LoadingObject, &[]);
        b.close();
        return true;
    };
    cards::card(b, client, id, &cards::Place::TaskPage, words, bindings);
    let Some(doc) = document(client, number) else {
        b.close();
        return true;
    };
    words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(doc.project))]);
    if let Some(goal) = doc.goal {
        if goal == doc.chip.task {
            words.text(b, JigPhrase::Goal, &[]);
        } else {
            crate::frame::control(
                b,
                words,
                bindings,
                Role::Link,
                JigPhrase::ServedGoal,
                &[Hole::Number(goal)],
                Act::Go { address: Address::task(goal) },
            );
        }
    }
    if let Some(priority) = doc.priority {
        words.text(b, JigPhrase::Priority, &[Hole::Number(u64::from(priority))]);
    }
    for chip in &doc.lineage_chips {
        cards::chip::chip(b, client, chip, words, bindings);
    }
    b.close();
    true
}
fn shown(client: &Domain) -> Option<u64> {
    for section in client.sections() {
        if client.shows(*section) {
            match section {
                jig_domain_client::pages::Section::Task { task, .. } => return Some(*task),
                jig_domain_client::pages::Section::Inbox { .. }
                | jig_domain_client::pages::Section::Conversations { .. }
                | jig_domain_client::pages::Section::Project { .. }
                | jig_domain_client::pages::Section::Deployment => {}
            }
        }
    }
    None
}
fn document(client: &Domain, number: u64) -> Option<&Task> {
    let id = client.objects().find(ObjectKey::Task(number))?;
    match &client.objects().get(id)?.body {
        Some(Body::Task(doc)) => Some(doc),
        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Escalation(_)
            | Body::OwnerEscalation(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}
/// Every offered section, linked to its address with the current address marked.
pub fn tabs(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) -> bool {
    let Some(number) = shown(client) else {
        return false;
    };
    let Some(doc) = document(client, number) else {
        return false;
    };
    let sections = jig_domain_client::pages::task::offers(doc);
    b.open(Role::Tabs);
    b.region(crate::regions::JigRegion::TaskTabs.region());
    b.name(&words.say(JigPhrase::TaskPage, &[Hole::Number(number)]));
    for offered in &sections.offered {
        let address = Address::Task { task: number, section: *offered };
        crate::pages::work::tab(
            b,
            words,
            bindings,
            section_phrase(*offered),
            Act::Go { address },
            match client.page() {
                jig_domain_client::pages::Page::Work { address: own, .. } => own == &address,
                jig_domain_client::pages::Page::Starting
                | jig_domain_client::pages::Page::SigningIn
                | jig_domain_client::pages::Page::Elsewhere
                | jig_domain_client::pages::Page::Missing
                | jig_domain_client::pages::Page::OutOfDate
                | jig_domain_client::pages::Page::SomeoneElse => false,
            },
        );
    }
    b.close();
    true
}
pub(crate) fn section(
    b: &mut Builder,
    client: &Domain,
    words: &Words,
    bindings: &mut JigBindings,
    task: u64,
    section: TaskSection,
) -> bool {
    if !client.shows(jig_domain_client::pages::Section::Task { task, section }) {
        return false;
    }
    crate::pages::work::section(b, words, section_phrase(section), region(section));
    if let Some(doc) = document(client, task) {
        match section {
            TaskSection::Overview => overview(b, client, doc, words, bindings),
            TaskSection::Activity => activity(b, client, doc, words, bindings),
            TaskSection::Plan => plan::plan(b, client, words, bindings),
            TaskSection::History => history::history(b, client, words, bindings),
            TaskSection::Budget => budget(b, doc, words, bindings),
            TaskSection::Panels => slots(b, doc, section, words, bindings),
        }
    }
    b.close();
    true
}
const fn region(section: TaskSection) -> crate::regions::JigRegion {
    use crate::regions::JigRegion;
    match section {
        TaskSection::Overview => JigRegion::TaskOverview,
        TaskSection::Activity => JigRegion::TaskActivity,
        TaskSection::Plan => JigRegion::TaskPlan,
        TaskSection::History => JigRegion::TaskHistory,
        TaskSection::Budget => JigRegion::TaskBudget,
        TaskSection::Panels => JigRegion::TaskPanels,
    }
}
pub(super) const fn section_phrase(section: TaskSection) -> JigPhrase {
    match section {
        TaskSection::Overview => JigPhrase::Overview,
        TaskSection::Activity => JigPhrase::Activity,
        TaskSection::Plan => JigPhrase::Plan,
        TaskSection::History => JigPhrase::History,
        TaskSection::Budget => JigPhrase::BudgetAuthority,
        TaskSection::Panels => JigPhrase::Panels,
    }
}
fn slots(b: &mut Builder, doc: &Task, section: TaskSection, words: &Words, bindings: &mut JigBindings) {
    let context_parts = match &doc.context {
        Some(context) => context.parts.len(),
        None => 0,
    };
    let count = doc
        .resources
        .len()
        .checked_add(context_parts)
        .expect("bounded task parts")
        .checked_add(doc.authority.authority.grants.len().checked_mul(2).expect("bounded grants"))
        .expect("bounded task parts")
        .checked_add(1)
        .expect("result slot fits");
    for part in &jig_domain_client::pages::task::parts(doc, section, count) {
        cards::common::part(b, part, words, bindings);
    }
    match section {
        TaskSection::Overview | TaskSection::Panels => {
            for resource in &doc.resources {
                match resource {
                    jig_domain_client::contract::TaskResource::Owner(value) => {
                        crate::owner_chip::resource(b, value, words, bindings);
                    }
                    jig_domain_client::contract::TaskResource::Part(_) => {}
                }
            }
        }
        TaskSection::Activity | TaskSection::Plan | TaskSection::History | TaskSection::Budget => {}
    }
}
pub(crate) fn overview(b: &mut Builder, client: &Domain, doc: &Task, words: &Words, bindings: &mut JigBindings) {
    if let jig_domain_client::contract::ChipPhase::Owner { details, .. } = doc.chip.details.as_ref() {
        crate::owner_chip::render(b, details, client, words, bindings);
    }
    crate::specification::document(b, &doc.spec, words);
    crate::instructions::document(b, doc.instructions.as_ref(), words);
    words.text(b, JigPhrase::ResultExpected, &[]);
    crate::proposed_task::contract(b, &doc.result_contract, words, bindings);
    crate::owner_result::requester(b, doc.requester, words);
    if doc.cut {
        words.text(b, JigPhrase::TaskListsCut, &[]);
    }
    crate::owner_wake::policy(b, &doc.wake, &words.book, words.language);
    crate::owner_wake::subscriptions(b, &doc.subscriptions, &words.book, words.language);
    for link in &doc.dependencies {
        task_link(b, client, link, words, bindings);
    }
    for link in &doc.references {
        task_link(b, client, link, words, bindings);
    }
    for link in &doc.inputs {
        task_link(b, client, link, words, bindings);
        if let Some(id) = client.objects().find(ObjectKey::Result(link.number())) {
            cards::card(b, client, id, &cards::Place::TaskPage, words, bindings);
        }
    }
    if let Some(result) = &doc.result {
        crate::task_result::render(b, client, result, words, bindings);
    }
    if let Some(context) = &doc.context {
        for slot in &context.writer_slots {
            cards::common::part(b, &slot.resource, words, bindings);
            crate::frame::control(
                b,
                words,
                bindings,
                Role::Link,
                JigPhrase::ContextTask,
                &[Hole::Number(slot.task)],
                Act::Go { address: Address::task(slot.task) },
            );
            if let Some(attempt) = slot.attempt {
                words.text(b, JigPhrase::Attempt, &[Hole::Number(attempt)]);
            }
        }
    }
    slots(b, doc, TaskSection::Overview, words, bindings);
    match doc.chip.executor {
        Executor::Agent { charter } => words.text(b, JigPhrase::Charter, &[Hole::Number(u64::from(charter))]),
        Executor::Party { .. } | Executor::Role { .. } => {
            if let Some(id) = client.objects().find(ObjectKey::PersonTask(doc.chip.task)) {
                cards::card(b, client, id, &cards::Place::TaskPage, words, bindings);
            }
        }
        Executor::Procedure { .. } => {}
    }
}
fn task_link(
    b: &mut Builder,
    client: &Domain,
    link: &jig_domain_client::contract::OwnerTaskLink,
    words: &Words,
    bindings: &mut JigBindings,
) {
    crate::owner_result::link(b, client, link, words, bindings);
    match link {
        jig_domain_client::contract::OwnerTaskLink::Live(_)
        | jig_domain_client::contract::OwnerTaskLink::Retained { .. } => {}
        jig_domain_client::contract::OwnerTaskLink::Stored { number, .. } => crate::frame::control(
            b,
            words,
            bindings,
            Role::Link,
            JigPhrase::ContextTask,
            &[Hole::Number(*number)],
            Act::Go { address: Address::task(*number) },
        ),
    }
}
fn activity(b: &mut Builder, client: &Domain, doc: &Task, words: &Words, bindings: &mut JigBindings) {
    use jig_domain_client::contract::ChipPhase;
    match doc.chip.executor {
        Executor::Agent { .. } => {
            match doc.chip.details.as_ref() {
                ChipPhase::Owner { details, .. } => {
                    if let Some(observation) = &details.observation {
                        crate::owner_chip::observation(b, observation, words);
                    }
                }
                ChipPhase::Active(_)
                | ChipPhase::Waiting { .. }
                | ChipPhase::Idle { .. }
                | ChipPhase::Starting
                | ChipPhase::Running { .. }
                | ChipPhase::WaitingForWords { .. }
                | ChipPhase::BackingOff { .. }
                | ChipPhase::Procedure { .. }
                | ChipPhase::Person { .. }
                | ChipPhase::Held { .. }
                | ChipPhase::Closing(_)
                | ChipPhase::Ended { .. } => {}
            }
            if let Some(context) = &doc.context {
                crate::pages::conversations::runtime(b, client, &context.runtime, words, bindings);
            }
            crate::turns::conversation(b, client, words, bindings);
            for proposal in &client.task_page().proposals {
                let key = ObjectKey::Proposal(*proposal);
                if let Some(id) = client.objects().find(key)
                    && !crate::turns::shows_object(client.conversation(), key)
                {
                    let object = client.objects().get(id).expect("shared proposal exists");
                    // Source-less cards use the common after-page departure
                    // region, including its final read and supplied ending.
                    if !object.sources.is_empty() {
                        cards::card(b, client, id, &cards::Place::TaskPage, words, bindings);
                    }
                }
            }
        }
        Executor::Procedure { .. } => slots(b, doc, TaskSection::Activity, words, bindings),
        Executor::Party { .. } | Executor::Role { .. } => {
            if let Some(id) = client.objects().find(ObjectKey::PersonTask(doc.chip.task)) {
                cards::card(b, client, id, &cards::Place::TaskPage, words, bindings);
            }
            if let Some(result) = &doc.result {
                crate::task_result::render(b, client, result, words, bindings);
            }
        }
    }
}
pub(crate) fn budget(b: &mut Builder, doc: &Task, words: &Words, _bindings: &mut JigBindings) {
    crate::owner_budget::render(b, &doc.budget, words);
    if let Some(context) = &doc.context {
        for spend in &context.spend {
            match spend.attempt {
                Some(attempt) => words.text(
                    b,
                    JigPhrase::SpentRun,
                    &[Hole::Number(spend.task), Hole::Number(attempt), Hole::Amount(spend.spent)],
                ),
                None => words.text(b, JigPhrase::SpentDelegate, &[Hole::Number(spend.task), Hole::Amount(spend.spent)]),
            }
        }
    }
    match doc.chip.executor {
        Executor::Agent { .. } | Executor::Procedure { .. } => {
            crate::owner_authority::document_words(b, &doc.authority, words);
        }
        Executor::Party { .. } | Executor::Role { .. } => {}
    }
}

pub(crate) fn scratch(rows: u32) -> Option<u64> {
    plan::scratch(rows)
}
