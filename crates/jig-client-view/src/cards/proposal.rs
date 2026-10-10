//! Proposal action, funding and history in words (client/domain/objects.md, section 4).
use crate::{
    Builder, JigBindings, Role,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::{
    Domain,
    contract::{Amendment, ProposalAction, ProposedTask},
    objects::{Object, proposal::facts},
};
use skein_lib::Id;
/// Build the one proposal card at any place, using the document's acts only.
pub fn proposal(
    b: &mut Builder,
    client: &Domain,
    object: Id<Object>,
    place: &super::Place,
    words: &Words,
    bindings: &mut JigBindings,
) -> bool {
    let Some(object_facts) = client.objects().get(object) else {
        return false;
    };
    let Some(document) = facts(object_facts) else {
        return false;
    };
    let title = words.say(JigPhrase::Proposal, &[]);
    if !super::open(b, client, object, place, &title, words, bindings) {
        return false;
    }
    words.proposer(b, document.proposer);
    words.waiting(b, client.wall(), document.since);
    words.text(b, JigPhrase::ProposalWhy, &[]);
    crate::markdown::read(document.why.bytes(), 2, b);
    action(b, &document.action, words, bindings);
    super::common::authority(b, &document.needs, words, bindings);
    super::common::funding(b, &document.funding, document.needs.budget, words);
    super::common::recipient(b, &document.waiting, words);
    super::common::trail(b, &document.trail, words);
    if let Some(ending) = &document.ending {
        words.ending(b, ending);
    }
    super::common::answer(b, object_facts, words);
    super::common::acts(b, &object_facts.card, &document.acts, object, words, bindings);
    super::close(b);
    true
}
/// Describe every proposal kind without evaluating the authority it asks for.
pub fn action(b: &mut Builder, action: &ProposalAction, words: &Words, bindings: &mut JigBindings) {
    match action {
        ProposalAction::Batch { tasks, own } => {
            words.text(b, JigPhrase::Batch, &[]);
            if *own {
                words.text(b, JigPhrase::OwnTasks, &[]);
            }
            b.open(Role::List);
            for task in tasks {
                proposed_task(b, task, words, bindings);
            }
            b.close();
        }
        ProposalAction::Effect { part } => {
            words.text(b, JigPhrase::Effect, &[]);
            super::common::part(b, part, words, bindings);
        }
        ProposalAction::Widen { before, after } => {
            words.text(b, JigPhrase::Widen, &[]);
            words.text(b, JigPhrase::Before, &[]);
            super::common::authority(b, before, words, bindings);
            words.text(b, JigPhrase::After, &[]);
            super::common::authority(b, after, words, bindings);
        }
        ProposalAction::Amend { task, before, change } => {
            words.text(b, JigPhrase::AmendTask, &[Hole::Number(*task)]);
            words.text(b, JigPhrase::Before, &[]);
            crate::markdown::read(before.bytes(), 2, b);
            words.text(b, JigPhrase::After, &[]);
            amendment(b, change, words, bindings);
        }
    }
}
pub(crate) fn proposed_task(b: &mut Builder, task: &ProposedTask, words: &Words, bindings: &mut JigBindings) {
    b.open(Role::Item);
    words.text(b, JigPhrase::BatchTask, &[Hole::Number(task.number)]);
    crate::specification::document(b, &task.spec, words);
    crate::proposed_task::executor(b, task.executor, words);
    crate::proposed_task::contract(b, &task.contract, words, bindings);
    crate::owner_authority::document_words(b, &task.authority, words);
    words.text(b, JigPhrase::Budget, &[Hole::Amount(task.budget)]);
    crate::proposed_task::funder(b, task.funder, words);
    crate::owner_wake::policy(b, &task.wake, &words.book, words.language);
    if let Some(priority) = task.priority {
        words.text(b, JigPhrase::Priority, &[Hole::Number(u64::from(priority))]);
    }
    for dependency in &task.dependencies {
        words.text(b, JigPhrase::AfterTask, &[Hole::Number(*dependency)]);
    }
    for resource in &task.resources {
        crate::proposed_task::part(b, resource, words, bindings);
    }
    if task.cut {
        crate::proposed_task::text(b, words, crate::proposed_task_words::ProposedPhrase::TaskCut, &[]);
    }
    b.close();
}
/// Changes retain the before/after facts instead of interpreting instructions.
pub fn amendment(b: &mut Builder, change: &Amendment, words: &Words, bindings: &mut JigBindings) {
    if let Some(to) = &change.addressed {
        super::common::recipient(b, to, words);
    }
    if let Some(spec) = &change.spec {
        crate::specification::specification(b, spec, words);
    }
    if let Some(text) = &change.instructions {
        crate::markdown::read(text.bytes(), 2, b);
    }
    if let Some(wake) = &change.wake {
        super::chip::wake(b, wake, words, bindings);
    }
    if let Some(dependencies) = &change.dependencies {
        for task in dependencies {
            words.text(b, JigPhrase::ContextTask, &[Hole::Number(*task)]);
        }
    }
    if let Some(authority) = &change.authority {
        super::common::authority(b, authority, words, bindings);
    }
    b.open(Role::Separator);
    b.close();
}
