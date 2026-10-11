//! Project sections compose shared cards and connector parts (pages.md, 6).
use crate::{
    Builder, JigBindings, Role, Run, cards,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::{
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
/// Show the project document and its selected bounded section.
pub fn project(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) {
    let (number, selected) = match client.page() {
        jig_client::pages::Page::Work { address: Address::Project { project, section }, .. } => (*project, *section),
        jig_client::pages::Page::Work { .. }
        | jig_client::pages::Page::Starting
        | jig_client::pages::Page::Elsewhere
        | jig_client::pages::Page::SigningIn
        | jig_client::pages::Page::Missing
        | jig_client::pages::Page::OutOfDate
        | jig_client::pages::Page::SomeoneElse => return,
    };
    super::heading(b, words, JigPhrase::ProjectPage, &[]);
    words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(number))]);
    if let Some(doc) = &client.project_page().owner {
        b.open(Role::Tabs);
        b.name(&words.say(JigPhrase::ContextProject, &[Hole::Number(u64::from(number))]));
        for section in [
            ProjectSection::Overview,
            ProjectSection::Goals,
            ProjectSection::Tasks,
            ProjectSection::Notes,
            ProjectSection::Settings,
        ] {
            let highlighted = match selected {
                ProjectSection::Resources
                | ProjectSection::Policy
                | ProjectSection::Roles
                | ProjectSection::Spend
                | ProjectSection::Recurring => section == ProjectSection::Settings,
                ProjectSection::Overview
                | ProjectSection::Goals
                | ProjectSection::Tasks
                | ProjectSection::Notes
                | ProjectSection::Settings => selected == section,
            };
            super::work::control(
                b,
                words,
                bindings,
                phrase(section),
                Act::Go { address: Address::Project { project: number, section } },
                highlighted,
            );
        }
        b.close();
        super::work::section(b, words, phrase(selected));
        owner::section(b, doc, selected, client, words, bindings);
        b.close();
        super::excess(b, client, words);
        return;
    }
    let Some(doc) = &client.project_page().document else {
        words.status(b, JigPhrase::LoadingObject, &[]);
        return;
    };
    b.text(Run::Plain, doc.name.bytes());
    b.open(Role::Tabs);
    b.name(doc.name.bytes());
    for section in [
        ProjectSection::Overview,
        ProjectSection::Goals,
        ProjectSection::Tasks,
        ProjectSection::Notes,
        ProjectSection::Settings,
    ] {
        let highlighted = match selected {
            ProjectSection::Resources
            | ProjectSection::Policy
            | ProjectSection::Roles
            | ProjectSection::Spend
            | ProjectSection::Recurring => section == ProjectSection::Settings,
            ProjectSection::Overview
            | ProjectSection::Goals
            | ProjectSection::Tasks
            | ProjectSection::Notes
            | ProjectSection::Settings => selected == section,
        };
        super::work::control(
            b,
            words,
            bindings,
            phrase(section),
            Act::Go { address: Address::Project { project: number, section } },
            highlighted,
        );
    }
    b.close();
    super::work::section(b, words, phrase(selected));
    match selected {
        ProjectSection::Overview => {
            inbox_cards(b, client, words, bindings);
            for goal in &client.project_page().goals {
                goals::line(b, client, goal, words, bindings);
            }
            super::work::budget(b, &doc.spend, words);
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
                super::work::section(b, words, phrase(section));
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
    super::excess(b, client, words);
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
    for part in &jig_client::pages::project::parts(doc, section, count) {
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
            super::cut(b, words, client.project_page().spend_cut);
            super::work::cursors(
                b,
                words,
                bindings,
                client.project_page().spend_before.as_ref(),
                client.project_page().spend_after.as_ref(),
                jig_client::pages::project::SPEND_WINDOW,
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
    budget: &jig_client::contract::Budget,
    pools: &skein_lib::List<jig_client::contract::Pool>,
    words: &Words,
) {
    words.text(b, JigPhrase::Period, &[Hole::Number(number)]);
    super::work::budget(b, budget, words);
    for pool in pools {
        words.text(b, JigPhrase::PartyNumber, &[Hole::Number(pool.party)]);
        super::work::budget(b, &pool.budget, words);
    }
}
fn task_spend(
    b: &mut Builder,
    client: &Domain,
    tasks: &skein_lib::List<jig_client::contract::TaskSpend>,
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
                    jig_client::contract::InboxRow::Entry(value) => {
                        cards::Place::Inbox { lineage: value.lineage.clone() }
                    }
                    jig_client::contract::InboxRow::Owner(_) => cards::Place::Elsewhere,
                },
                words,
                bindings,
            );
        }
    }
}
