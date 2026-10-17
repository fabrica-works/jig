//! Result contracts and unread replies (client/domain/objects.md, section 8).
use crate::JigBindings;
use crate::phrases::jig::JigPhrase;
use crate::words::{Words, name};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use crate::{
    Borrowed,
    action::Act,
    objects::{Object, result::facts as document},
};
use jig_client::contract::{Result as ResultDocument, ResultValue};
use skein_lib::Id;
/// Build a result's shared card and all its contract payloads.
pub fn result(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: Id<Object>,
    place: &super::Place,
    words: &Words,
    bindings: &mut JigBindings,
) -> bool {
    let text_limit = bindings.text_limit();
    let Some(fact) = client.objects().get(object) else {
        return false;
    };
    let Some(document) = document(&fact) else {
        return false;
    };
    let title = words.say(JigPhrase::Result, &[]);
    if !super::open(b, client, object, place, &title, words, bindings) {
        return false;
    }
    facts(b, client, document, words, bindings);
    super::common::answer(b, &fact, words, text_limit);
    super::common::acts(b, &fact.card, &document.acts, object, words, bindings);
    super::close(b);
    true
}
/// Render passive input results side by side without duplicating their domain objects.
pub fn facts(
    b: &mut Builder,
    client: &Borrowed<'_>,
    facts: &ResultDocument,
    words: &Words,
    bindings: &mut JigBindings,
) {
    b.open(Role::Region);
    let title = words.say(JigPhrase::ChipNumber, &[Hole::Number(facts.task.task), name(facts.task.title.bytes())]);
    b.name(&title);
    super::chip::chip(b, client, &facts.task, words, bindings);
    words.ending(b, &facts.ending);
    match facts.at {
        Some(at) => words.wall(b, at),
        None => words.text(b, JigPhrase::ResultTimeUnknown, &[]),
    }
    if let Some(details) = &facts.details {
        crate::owner_result::render(b, client, details, words, bindings);
    }
    if let Some(value) = &facts.value {
        value_nodes(b, client, value, words, bindings);
    }
    b.close();
}
/// The whole result vocabulary, including connector slots and follow-up chips.
pub fn value_nodes(
    b: &mut Builder,
    client: &Borrowed<'_>,
    value: &ResultValue,
    words: &Words,
    bindings: &mut JigBindings,
) {
    match value {
        ResultValue::Report(text) | ResultValue::Words(text) => crate::document_text::markdown(b, text, words),
        ResultValue::OwnerVerdict { verdict, words: explanation, fields } => {
            verdict_payload(b, *verdict, explanation, fields, words);
        }
        ResultValue::Verdict { verdict, words: explanation, fields, tasks, proposal } => {
            verdict_payload(b, *verdict, explanation, fields, words);
            for task in tasks {
                super::chip::chip(b, client, task, words, bindings);
            }
            if let Some(key) = proposal
                && let Some(id) = client.objects().find(jig_client::contract::ObjectKey::Proposal(*key))
            {
                super::proposal::proposal(b, client, id, &super::Place::Conversation, words, bindings);
            }
        }
        ResultValue::Connector { summary, part } => {
            crate::document_text::markdown(b, summary, words);
            super::common::part(b, part, words, bindings);
        }
        ResultValue::Change { kind, part, words: explanation } => {
            words.text(b, JigPhrase::ChangeResultKind, &[Hole::Number(u64::from(*kind))]);
            crate::document_text::markdown(b, explanation, words);
            crate::proposed_task::part(b, part, words, bindings);
        }
        ResultValue::Choice { option, by, reason } => {
            words.text(b, JigPhrase::ChoiceResult, &[Hole::Number(u64::from(*option)), name(by.name.bytes())]);
            crate::document_text::markdown(b, reason, words);
        }
        ResultValue::Failure(reason) => {
            words.text(b, JigPhrase::Word(jig_client::words::Word::Phase(jig_client::contract::Phase::Failed)), &[]);
            crate::document_text::markdown(b, reason, words);
        }
        ResultValue::Approved { state, words: explanation } => {
            words.text(b, JigPhrase::ApprovalExpected, &[]);
            b.text(Run::Plain, state);
            if let Some(explanation) = explanation {
                crate::document_text::markdown(b, explanation, words);
            }
        }
    }
}
fn verdict_payload(
    b: &mut Builder,
    verdict: u32,
    explanation: &jig_client::contract::DocumentText,
    fields: &skein_lib::List<jig_client::contract::ResultField>,
    words: &Words,
) {
    words.text(b, JigPhrase::Verdict, &[Hole::Number(u64::from(verdict))]);
    crate::document_text::markdown(b, explanation, words);
    for field in fields {
        b.text(Run::Plain, &field.name);
        skein_ui::markdown::read(&field.value, 2, b);
    }
}
/// A reply opens the chat at its domain-kept first unread reply position.
pub fn reply(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: Id<Object>,
    place: &super::Place,
    words: &Words,
    bindings: &mut JigBindings,
) -> bool {
    let Some(fact) = client.objects().get(object) else {
        return false;
    };
    let Some(reply) = &fact.reply else {
        return false;
    };
    let title = words.say(JigPhrase::Reply, &[]);
    if !super::open(b, client, object, place, &title, words, bindings) {
        return false;
    }
    match &fact.body {
        Some(crate::objects::Body::Chip(chip)) => super::chip::chip(b, client, chip, words, bindings),
        Some(crate::objects::Body::Task(task)) => super::chip::chip(b, client, &task.chip, words, bindings),
        Some(
            crate::objects::Body::Historical(_)
            | crate::objects::Body::Proposal(_)
            | crate::objects::Body::OwnerProposal(_)
            | crate::objects::Body::Escalation(_)
            | crate::objects::Body::OwnerEscalation(_)
            | crate::objects::Body::Question(_)
            | crate::objects::Body::PersonTask(_)
            | crate::objects::Body::OwnerPersonTask(_)
            | crate::objects::Body::Result(_)
            | crate::objects::Body::Note { .. },
        )
        | None => words.text(b, JigPhrase::ContextTask, &[Hole::Number(reply.chat)]),
    }
    crate::document_text::plain(b, &reply.first_words, words);
    crate::frame::control(
        b,
        words,
        bindings,
        Role::Link,
        JigPhrase::FirstUnread,
        &[Hole::Number(reply.first_unread)],
        Act::OpenChat { task: reply.chat },
    );
    super::close(b);
    true
}
