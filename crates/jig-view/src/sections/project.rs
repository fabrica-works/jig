//! Project sections compose shared cards and connector parts (pages.md, 6).
use crate::{
    Builder, JigBindings, Role, Run, cards,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_domain_client::{
    Domain,
    action::Act,
    address::{Address, ProjectSection},
    contract::{ObjectKey, Project},
};
pub mod goals;
mod notes;
pub mod owner;
mod owner_goals;
mod recurring;
mod tasks;
/// The project's shared identity while any of its sections is shown.
pub fn header(b: &mut Builder, client: &Domain, words: &Words, _bindings: &mut JigBindings) -> bool {
    let Some(number) = client.shown_project() else {
        return false;
    };
    b.open(Role::Region);
    b.region(crate::regions::JigRegion::ProjectHeader.region());
    b.name(&words.say(JigPhrase::ProjectPage, &[]));
    words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(number))]);
    if let Some(doc) = &client.project_page().document {
        b.text(Run::Plain, doc.name.bytes());
    } else if client.project_page().owner.is_none() {
        words.status(b, JigPhrase::LoadingObject, &[]);
    }
    b.close();
    true
}
/// Every section linked to its address, with the address's own marked current.
pub fn tabs(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) -> bool {
    let Some(number) = client.shown_project() else {
        return false;
    };
    if client.project_page().document.is_none() && client.project_page().owner.is_none() {
        return false;
    }
    b.open(Role::Tabs);
    b.region(crate::regions::JigRegion::ProjectTabs.region());
    match &client.project_page().document {
        Some(doc) => b.name(doc.name.bytes()),
        None => b.name(&words.say(JigPhrase::ContextProject, &[Hole::Number(u64::from(number))])),
    }
    for section in [
        ProjectSection::Overview,
        ProjectSection::Goals,
        ProjectSection::Tasks,
        ProjectSection::Notes,
        ProjectSection::Settings,
        ProjectSection::Resources,
        ProjectSection::Policy,
        ProjectSection::Roles,
        ProjectSection::Spend,
        ProjectSection::Recurring,
    ] {
        let address = Address::Project { project: number, section };
        let current = match client.page() {
            jig_domain_client::pages::Page::Work { address: own, .. } => own == &address,
            jig_domain_client::pages::Page::Starting
            | jig_domain_client::pages::Page::SigningIn
            | jig_domain_client::pages::Page::Elsewhere
            | jig_domain_client::pages::Page::Missing
            | jig_domain_client::pages::Page::OutOfDate
            | jig_domain_client::pages::Page::SomeoneElse => false,
        };
        crate::pages::work::tab(b, words, bindings, phrase(section), Act::Go { address }, current);
    }
    b.close();
    true
}
pub(crate) fn all_shown(client: &Domain) -> bool {
    let Some(project) = client.shown_project() else {
        return false;
    };
    for section in [ProjectSection::Overview, ProjectSection::Goals, ProjectSection::Tasks, ProjectSection::Notes] {
        if !client.shows(jig_domain_client::pages::Section::Project { project, section }) {
            return false;
        }
    }
    if client.shows(jig_domain_client::pages::Section::Project { project, section: ProjectSection::Settings }) {
        return true;
    }
    for section in [
        ProjectSection::Resources,
        ProjectSection::Policy,
        ProjectSection::Roles,
        ProjectSection::Spend,
        ProjectSection::Recurring,
    ] {
        if !client.shows(jig_domain_client::pages::Section::Project { project, section }) {
            return false;
        }
    }
    true
}
pub(crate) fn section(
    b: &mut Builder,
    client: &Domain,
    words: &Words,
    bindings: &mut JigBindings,
    project: u32,
    selected: ProjectSection,
) -> bool {
    if !client.shows(jig_domain_client::pages::Section::Project { project, section: selected }) {
        return false;
    }
    crate::pages::work::section(b, words, phrase(selected), region(selected));
    crate::pages::mark_current(b, client, Address::Project { project, section: selected });
    if let Some(doc) = &client.project_page().owner {
        owner::section(b, doc, selected, client, words, bindings);
        b.close();
        return true;
    }
    let Some(doc) = &client.project_page().document else {
        b.close();
        return true;
    };
    let number = project;
    match selected {
        ProjectSection::Overview => {
            inbox_cards(b, client, words, bindings);
            for goal in &client.project_page().goals {
                goals::line(b, client, goal, words, bindings);
            }
            crate::pages::work::budget(b, &doc.spend, words);
            slots(b, doc, selected, words, bindings);
        }
        ProjectSection::Goals => goals::goals(b, client, doc, words, bindings),
        ProjectSection::Tasks => tasks::tasks(b, client, doc, words, bindings),
        ProjectSection::Notes => notes::notes(b, client, number, words, bindings),
        ProjectSection::Settings => {
            for section in [
                ProjectSection::Resources,
                ProjectSection::Policy,
                ProjectSection::Roles,
                ProjectSection::Spend,
                ProjectSection::Recurring,
            ] {
                crate::pages::work::section(b, words, phrase(section), region(section));
                crate::pages::mark_current(b, client, Address::Project { project, section });
                settings(b, client, doc, section, words, bindings);
                b.close();
            }
            slots(b, doc, selected, words, bindings);
        }
        ProjectSection::Resources
        | ProjectSection::Policy
        | ProjectSection::Roles
        | ProjectSection::Spend
        | ProjectSection::Recurring => {
            settings(b, client, doc, selected, words, bindings);
            slots(b, doc, selected, words, bindings);
        }
    }
    b.close();
    true
}
const fn phrase(section: ProjectSection) -> JigPhrase {
    match section {
        ProjectSection::Overview => JigPhrase::Overview,
        ProjectSection::Goals => JigPhrase::Goals,
        ProjectSection::Tasks => JigPhrase::Tasks,
        ProjectSection::Notes => JigPhrase::Notes,
        ProjectSection::Settings => JigPhrase::Settings,
        ProjectSection::Resources => JigPhrase::Resources,
        ProjectSection::Policy => JigPhrase::Policy,
        ProjectSection::Roles => JigPhrase::Roles,
        ProjectSection::Spend => JigPhrase::Spend,
        ProjectSection::Recurring => JigPhrase::Recurring,
    }
}
fn slots(b: &mut Builder, doc: &Project, section: ProjectSection, words: &Words, bindings: &mut JigBindings) {
    let count = doc
        .parts
        .len()
        .checked_add(doc.resources.len())
        .expect("bounded parts")
        .checked_add(doc.policy.requirements.len())
        .expect("bounded parts");
    for part in &jig_domain_client::pages::project::parts(doc, section, count) {
        cards::common::part(b, part, words, bindings);
    }
}
fn settings(
    b: &mut Builder,
    client: &Domain,
    doc: &Project,
    section: ProjectSection,
    words: &Words,
    bindings: &mut JigBindings,
) {
    match section {
        ProjectSection::Policy => {
            cards::common::authority(b, &doc.policy.ceiling, words, bindings);
            words.text(b, JigPhrase::Period, &[Hole::Number(doc.policy.period)]);
            words.text(b, JigPhrase::PolicySpend, &[Hole::Amount(doc.policy.spend)]);
            for role in &doc.policy.roles {
                words.text(b, JigPhrase::RoleNumber, &[Hole::Number(u64::from(role.number))]);
                words.text(b, JigPhrase::Allotment, &[Hole::Amount(role.allotment)]);
                cards::common::authority(b, &role.authority, words, bindings);
                for kind in &role.decisions {
                    words.text(b, JigPhrase::DecidesKind, &[Hole::Number(u64::from(*kind))]);
                }
            }
            for decider in &doc.policy.deciders {
                words.text(b, JigPhrase::DecidesKind, &[Hole::Number(u64::from(decider.kind))]);
                words.text(b, JigPhrase::RoleNumber, &[Hole::Number(u64::from(decider.role))]);
            }
        }
        ProjectSection::Roles => {
            for holding in &doc.roles {
                words.text(b, JigPhrase::PartyNumber, &[Hole::Number(holding.party)]);
                words.held_role(b, Some(holding.role));
            }
            for service in &doc.services {
                b.text(Run::Plain, service.name.bytes());
                words.text(b, JigPhrase::PartyNumber, &[Hole::Number(service.number)]);
            }
        }
        ProjectSection::Spend => {
            period(b, doc.period, &doc.spend, &doc.pools, words);
            task_spend(b, client, &doc.spend_lines, words, bindings);
            if let Some(page) = &client.project_page().owner_spend {
                crate::historical_pages::spend_render(b, page, words);
            }
            for past in &client.project_page().spend {
                period(b, past.period, &past.budget, &past.pools, words);
                task_spend(b, client, &past.tasks, words, bindings);
            }
            crate::pages::cut(b, words, client.project_page().spend_cut);
            crate::pages::work::cursors(
                b,
                words,
                bindings,
                client.project_page().spend_before.as_ref(),
                client.project_page().spend_after.as_ref(),
                jig_domain_client::pages::project::SPEND_WINDOW,
            );
        }
        ProjectSection::Recurring => {
            for row in &doc.recurring {
                recurring::recurring(b, client, row, words, bindings);
            }
        }
        ProjectSection::Resources
        | ProjectSection::Overview
        | ProjectSection::Goals
        | ProjectSection::Tasks
        | ProjectSection::Notes
        | ProjectSection::Settings => {}
    }
}
fn period(
    b: &mut Builder,
    number: u64,
    budget: &jig_domain_client::contract::Budget,
    pools: &skein_lib::List<jig_domain_client::contract::Pool>,
    words: &Words,
) {
    words.text(b, JigPhrase::Period, &[Hole::Number(number)]);
    crate::pages::work::budget(b, budget, words);
    for pool in pools {
        words.text(b, JigPhrase::PartyNumber, &[Hole::Number(pool.party)]);
        crate::pages::work::budget(b, &pool.budget, words);
    }
}
fn task_spend(
    b: &mut Builder,
    client: &Domain,
    tasks: &skein_lib::List<jig_domain_client::contract::TaskSpend>,
    words: &Words,
    bindings: &mut JigBindings,
) {
    for task in tasks {
        cards::chip::chip(b, client, &task.chip, words, bindings);
        words.amount(b, task.spent);
        if let Some(id) = client.objects().find(ObjectKey::Task(task.chip.task)) {
            cards::card(b, client, id, &cards::Place::Elsewhere, words, bindings);
        }
    }
}

fn inbox_cards(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) {
    for entry in &client.inbox().rows {
        if let Some(id) = client.objects().find(entry.card_key()) {
            cards::card(
                b,
                client,
                id,
                &match entry {
                    jig_domain_client::contract::InboxRow::Entry(value) => {
                        cards::Place::Inbox { lineage: value.lineage.clone() }
                    }
                    jig_domain_client::contract::InboxRow::Owner(_) => cards::Place::Elsewhere,
                },
                words,
                bindings,
            );
        }
    }
}

/// Retained funding and earlier period rows preserve the original spend window.
fn historical_spend(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) {
    let page = client.project_page();
    if let Some(spend) = &page.owner_spend {
        crate::historical_pages::spend_render(b, spend, words);
    }
    for past in &page.spend {
        period(b, past.period, &past.budget, &past.pools, words);
        task_spend(b, client, &past.tasks, words, bindings);
    }
    crate::pages::cut(b, words, page.spend_cut);
    crate::pages::work::cursors(
        b,
        words,
        bindings,
        page.spend_before.as_ref(),
        page.spend_after.as_ref(),
        jig_domain_client::pages::project::SPEND_WINDOW,
    );
}

pub(crate) const fn region(section: ProjectSection) -> crate::regions::JigRegion {
    use crate::regions::JigRegion;
    match section {
        ProjectSection::Overview => JigRegion::ProjectOverview,
        ProjectSection::Goals => JigRegion::ProjectGoals,
        ProjectSection::Tasks => JigRegion::ProjectTasks,
        ProjectSection::Notes => JigRegion::ProjectNotes,
        ProjectSection::Settings => JigRegion::ProjectSettings,
        ProjectSection::Resources => JigRegion::ProjectResources,
        ProjectSection::Policy => JigRegion::ProjectPolicy,
        ProjectSection::Roles => JigRegion::ProjectRoles,
        ProjectSection::Spend => JigRegion::ProjectSpend,
        ProjectSection::Recurring => JigRegion::ProjectRecurring,
    }
}
