#![expect(clippy::manual_map, reason = "step code matches optional authenticated identities without closures")]
//! Filters and live rows are the domain's facts (pages.md, 6.3).
use crate::phrases::jig::JigPhrase;
use crate::words::Words;
use crate::{JigBindings, cards};
use jig_client::contract::Phase;
use jig_client::words::Word;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role};

use crate::{Borrowed, action::Act};
use jig_client::contract::{ExecutorKind, ObjectKey, PhaseClass, Project, TaskQuery};
pub(super) fn tasks(b: &mut Builder, client: &Borrowed<'_>, _doc: &Project, words: &Words, bindings: &mut JigBindings) {
    let project_page = client.project_page();
    let filter = project_page.task_filter;
    crate::pages::work::control(
        b,
        words,
        bindings,
        JigPhrase::AllTasks,
        Act::FilterTasks { filter: TaskQuery { phase: None, executor: None, root: None, requester: None, ..filter } },
        filter.phase.is_none() && filter.executor.is_none() && filter.root.is_none() && filter.requester.is_none(),
    );
    crate::pages::work::control(
        b,
        words,
        bindings,
        JigPhrase::AskedByMe,
        Act::FilterTasks {
            filter: TaskQuery {
                requester: if filter.requester.is_some() {
                    None
                } else {
                    match client.frame().party() {
                        Some(party) => Some(jig_client::contract::OwnerRequester::Person(party)),
                        None => None,
                    }
                },
                ..filter
            },
        },
        filter.requester.is_some(),
    );
    for phase in [PhaseClass::Working, PhaseClass::Waiting, PhaseClass::Held, PhaseClass::Closing, PhaseClass::Ended] {
        crate::pages::work::control(
            b,
            words,
            bindings,
            phrase(phase),
            Act::FilterTasks { filter: TaskQuery { phase: Some(phase), ..filter } },
            filter.phase == Some(phase),
        );
    }
    for executor in [ExecutorKind::Agent, ExecutorKind::Procedure, ExecutorKind::Person] {
        let phrase = match executor {
            ExecutorKind::Agent => JigPhrase::Agent,
            ExecutorKind::Procedure => JigPhrase::Procedure,
            ExecutorKind::Person => JigPhrase::Person,
        };
        crate::pages::work::control(
            b,
            words,
            bindings,
            phrase,
            Act::FilterTasks { filter: TaskQuery { executor: Some(executor), ..filter } },
            filter.executor == Some(executor),
        );
    }
    b.open(Role::List);
    for row in &project_page.tasks {
        b.open(Role::Item);
        b.key(crate::pages::object_key(ObjectKey::Task(row.chip.task)));
        cards::chip::chip(b, client, &row.chip, words, bindings);
        if let Some(id) = client.objects().find(ObjectKey::Task(row.chip.task)) {
            cards::card(b, client, id, &cards::Place::Elsewhere, words, bindings);
        }
        crate::owner_result::requester(b, row.requester, words);
        words.wall(b, row.changed);
        let goal = task_root(client, row.chip.task);
        if let Some(goal) = goal {
            crate::frame::control(
                b,
                words,
                bindings,
                Role::Button,
                JigPhrase::OnlyGoal,
                &[Hole::Number(goal)],
                Act::FilterTasks { filter: TaskQuery { root: Some(goal), ..filter } },
            );
        }
        b.close();
    }
    b.close();
    if project_page.tasks_more {
        words.status(b, JigPhrase::MoreRows, &[]);
    }
}
const fn phrase(phase: PhaseClass) -> JigPhrase {
    match phase {
        PhaseClass::Working => JigPhrase::Word(Word::Phase(Phase::Active)),
        PhaseClass::Waiting => JigPhrase::Word(Word::Phase(Phase::Waiting)),
        PhaseClass::Held => JigPhrase::Word(Word::Phase(Phase::Held)),
        PhaseClass::Closing => JigPhrase::Word(Word::Phase(Phase::Closing)),
        PhaseClass::Ended => JigPhrase::Word(Word::Phase(Phase::Ended)),
    }
}

fn task_root(client: &Borrowed<'_>, task: u64) -> Option<u64> {
    let id = client.objects().find(ObjectKey::Task(task))?;
    let object = client.objects().get(id)?;
    if let Some(crate::objects::Body::Task(task)) = object.body { task.goal } else { None }
}
