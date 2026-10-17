//! Proposal action, funding and history in words (client/domain/objects.md, section 4).
use crate::JigBindings;
use crate::phrases::jig::JigPhrase;
use crate::words::Words;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role};

use crate::{
    Borrowed,
    objects::{Object, proposal::facts},
};
use jig_client::contract::{Amendment, ProposalAction, ProposedTask};
use skein_lib::Id;
/// Build the one proposal card at any place, using the document's acts only.
pub fn proposal(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: Id<Object>,
    place: &super::Place,
    words: &Words,
    bindings: &mut JigBindings,
) -> bool {
    let text_limit = bindings.text_limit();
    let Some(object_facts) = client.objects().get(object) else {
        return false;
    };
    let Some(document) = facts(&object_facts) else {
        return false;
    };
    let title = words.say(JigPhrase::Proposal, &[]);
    if !super::open(b, client, object, place, &title, words, bindings) {
        return false;
    }
    words.proposer(b, document.proposer);
    words.waiting(b, client.wall(), document.since);
    words.text(b, JigPhrase::ProposalWhy, &[]);
    skein_ui::markdown::read(document.why.bytes(), 2, b);
    action(b, &document.action, words, bindings);
    super::common::authority(b, &document.needs, words, bindings);
    super::common::funding(b, &document.funding, document.needs.budget, words);
    super::common::recipient(b, &document.waiting, words);
    super::common::trail(b, &document.trail, words, text_limit);
    if let Some(ending) = &document.ending {
        words.ending(b, ending);
    }
    super::common::answer(b, &object_facts, words, text_limit);
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
            skein_ui::markdown::read(before.bytes(), 2, b);
            words.text(b, JigPhrase::After, &[]);
            amendment(b, change, words, bindings);
        }
    }
}
pub(crate) fn proposed_task(b: &mut Builder, task: &ProposedTask, words: &Words, bindings: &mut JigBindings) {
    let text_limit = bindings.text_limit();
    b.open(Role::Item);
    words.text(b, JigPhrase::BatchTask, &[Hole::Number(task.number)]);
    crate::specification::document(b, &task.spec, words);
    crate::proposed_task::executor(b, task.executor, words);
    crate::proposed_task::contract(b, &task.contract, words, bindings);
    crate::owner_authority::document_words(b, &task.authority, words, text_limit);
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
    let text_limit = bindings.text_limit();
    if let Some(to) = &change.addressed {
        address_change(b, to, words);
    }
    if let Some(spec) = &change.spec {
        crate::specification::specification(b, spec, words);
    }
    if let Some(text) = &change.instructions {
        skein_ui::markdown::read(text.bytes(), 2, b);
    }
    if let Some(wake) = &change.wake {
        crate::owner_wake::policy(b, wake, &words.book, words.language);
    }
    if let Some(dependencies) = &change.dependencies {
        for task in dependencies {
            words.text(b, JigPhrase::ContextTask, &[Hole::Number(*task)]);
        }
    }
    if let Some(authority) = &change.authority {
        crate::owner_authority::render_words(b, authority, words, text_limit);
    }
    b.open(Role::Separator);
    b.close();
}

/// A named person revision and exact closed destination; the ask owns its project scope.
pub(crate) fn address_change(b: &mut Builder, change: &jig_client::contract::AddressChange, words: &Words) {
    words.text(b, JigPhrase::PersonAddressRevision, &[Hole::Number(change.revision.value)]);
    crate::proposed_task::executor(b, jig_client::contract::OwnerExecutor::Person(change.address), words);
}

/// The same keyed card for the independently supplied pending owner document.
pub fn owner_proposal(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: Id<Object>,
    place: &super::Place,
    words: &Words,
    bindings: &mut JigBindings,
) -> bool {
    let text_limit = bindings.text_limit();
    let Some(facts) = client.objects().get(object) else {
        return false;
    };
    let document = match &facts.body {
        Some(crate::objects::Body::OwnerProposal(document)) => document,
        Some(
            crate::objects::Body::Historical(_)
            | crate::objects::Body::Chip(_)
            | crate::objects::Body::Task(_)
            | crate::objects::Body::Proposal(_)
            | crate::objects::Body::Escalation(_)
            | crate::objects::Body::OwnerEscalation(_)
            | crate::objects::Body::Question(_)
            | crate::objects::Body::PersonTask(_)
            | crate::objects::Body::OwnerPersonTask(_)
            | crate::objects::Body::Result(_)
            | crate::objects::Body::Note { .. },
        )
        | None => return false,
    };
    let title = words.say(JigPhrase::Proposal, &[]);
    if !super::open(b, client, object, place, &title, words, bindings) {
        return false;
    }
    crate::owner_proposal::render(b, client, document, words, bindings);
    super::common::answer(b, &facts, words, text_limit);
    super::common::acts(b, &facts.card, &document.acts, object, words, bindings);
    super::close(b);
    true
}
