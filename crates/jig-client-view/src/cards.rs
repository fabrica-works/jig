//! One shared card frame in every place (client/domain/objects.md, section 2).
//! The place changes context only. Card state and terminal facts are owned by
//! the domain; this module neither changes them nor judges offered acts.
use crate::{
    Builder, JigBindings, Role,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::{
    Domain,
    action::Act,
    contract::Chip,
    objects::{Card, Left, Object},
};
use skein_lib::{Id, List};

/// The context line chosen by the page that shows this same object.
#[derive(Debug)]
pub enum Place {
    /// The inbox adds project and lineage chips.
    Inbox { lineage: List<Chip> },
    /// The conversation already supplies its context.
    Conversation,
    /// The task page already supplies its context.
    TaskPage,
    /// An application page shows the object's project.
    Elsewhere,
}
/// Open the named region, heading, context and state; caller adds content then calls `close`.
pub fn open(
    b: &mut Builder,
    client: &Domain,
    object: Id<Object>,
    place: &Place,
    title: &[u8],
    words: &Words,
    bindings: &mut JigBindings,
) -> bool {
    let Some(facts) = client.objects().get(object) else {
        return false;
    };
    b.open(Role::Item);
    let key = crate::pages::object_key(facts.key);
    b.key(key);
    b.open(Role::Region);
    b.name(title);
    b.style(crate::JigStyle::Card.token());
    match facts.card {
        Card::Deciding { .. } => b.state(crate::State::Busy),
        Card::Open | Card::Refused { .. } | Card::Unknown { .. } | Card::Leaving { .. } => {}
    }
    b.open(Role::Heading { level: 2 });
    b.text(crate::Run::Plain, title);
    b.close();
    match place {
        Place::Inbox { lineage } => {
            if let Some(project) = facts.project {
                words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(project))]);
            }
            for task in lineage {
                chip::chip(b, client, task, words, bindings);
            }
        }
        Place::Elsewhere => {
            if let Some(project) = facts.project {
                words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(project))]);
            }
        }
        Place::Conversation | Place::TaskPage => {}
    }
    state(b, &facts.card, object, words, bindings);
    true
}
/// Build the card's authoritative state, including full refusal and departure facts.
pub fn state(b: &mut Builder, card: &Card, object: Id<Object>, words: &Words, bindings: &mut JigBindings) {
    match card {
        Card::Open => words.status(b, JigPhrase::Open, &[]),
        Card::Deciding { progress, .. } => words.progress(b, *progress),
        Card::Refused { refusal } => {
            b.open(Role::Status);
            b.name(&words.say(JigPhrase::Refused, &[]));
            words.refusal(b, refusal);
            b.close();
        }
        Card::Unknown { .. } => {
            words.status(b, JigPhrase::Unknown, &[]);
            crate::frame::control(
                b,
                words,
                bindings,
                Role::Button,
                JigPhrase::LookAgain,
                &[],
                Act::LookAgain { object: object.token() },
            );
        }
        Card::Leaving { how, .. } => {
            b.open(Role::Status);
            b.name(&words.say(JigPhrase::Leaving, &[]));
            match how {
                Left::Looking => words.text(b, JigPhrase::Looking, &[]),
                Left::Answered { outcome } => words.outcome(b, outcome),
                Left::Ended { how } => words.ending(b, how),
                Left::OwnerEnded { how } => crate::owner_chip::ending_summary(b, how, words),
                Left::Unknown => words.text(b, JigPhrase::GoneUnknown, &[]),
            }
            b.close();
        }
    }
}

/// Close the card region and its keyed item after building its content.
pub fn close(b: &mut Builder) {
    b.close();
    b.close();
}

pub mod chip;
pub mod common;
pub mod escalation;
pub mod hold;
pub mod person;
pub mod proposal;
pub mod question;
pub mod result;

/// Build one object's card wherever it is named; the place changes its context only.
pub fn card(
    b: &mut Builder,
    client: &Domain,
    object: Id<Object>,
    place: &Place,
    words: &Words,
    bindings: &mut JigBindings,
) -> bool {
    let Some(facts) = client.objects().get(object) else {
        return false;
    };
    if facts.reply.is_some() {
        return result::reply(b, client, object, place, words, bindings);
    }
    match &facts.body {
        Some(jig_client::objects::Body::Proposal(_)) => {
            proposal::proposal(b, client, object, place, words, bindings);
        }
        Some(jig_client::objects::Body::Question(_)) => {
            question::question(b, client, object, place, words, bindings);
        }
        Some(jig_client::objects::Body::PersonTask(_)) => {
            person::person_task(b, client, object, place, words, bindings);
        }
        Some(jig_client::objects::Body::Result(_)) => {
            result::result(b, client, object, place, words, bindings);
        }
        Some(jig_client::objects::Body::Chip(chip)) => {
            let title =
                words.say(JigPhrase::ChipNumber, &[Hole::Number(chip.task), crate::words::name(chip.title.bytes())]);
            if open(b, client, object, place, &title, words, bindings) {
                chip::chip(b, client, chip, words, bindings);
                close(b);
            }
        }
        Some(jig_client::objects::Body::Task(task)) => {
            let title = words.say(
                JigPhrase::ChipNumber,
                &[Hole::Number(task.chip.task), crate::words::name(task.chip.title.bytes())],
            );
            if open(b, client, object, place, &title, words, bindings) {
                chip::chip(b, client, &task.chip, words, bindings);
                if let Some(result) = &task.result {
                    crate::task_result::render(b, client, result, words, bindings);
                }
                common::acts(b, &facts.card, &task.acts, object, words, bindings);
                close(b);
            }
        }
        Some(jig_client::objects::Body::Escalation(_)) => {
            escalation::escalation(b, client, object, place, words, bindings);
        }
        Some(jig_client::objects::Body::Note { document, .. }) => {
            if open(b, client, object, place, document.description.bytes(), words, bindings) {
                crate::document_text::loss(b, &document.description, words);
                crate::document_text::markdown(b, &document.body, words);
                if document.cut {
                    words.text(b, JigPhrase::NoteReferencesCut, &[]);
                }
                match document.writer {
                    jig_client::contract::NoteAuthor::Party { party } => {
                        words.text(b, JigPhrase::NoteWriterParty, &[Hole::Number(party)]);
                    }
                    jig_client::contract::NoteAuthor::Task { task, attempt } => {
                        words.text(b, JigPhrase::NoteWriterTask, &[Hole::Number(task), Hole::Number(attempt)]);
                    }
                }
                if let Some(changed) = document.changed {
                    words.wall(b, changed);
                } else {
                    words.text(b, JigPhrase::NoteChangedNotSupplied, &[]);
                }
                for task in &document.tasks {
                    crate::frame::control(
                        b,
                        words,
                        bindings,
                        Role::Link,
                        JigPhrase::TaskPage,
                        &[Hole::Number(*task)],
                        Act::Go { address: jig_client::address::Address::task(*task) },
                    );
                }
                common::acts(b, &facts.card, &document.acts, object, words, bindings);
                close(b);
            }
        }
        None => {
            let title = words.say(JigPhrase::Object, &[]);
            if open(b, client, object, place, &title, words, bindings) {
                words.status(b, JigPhrase::LoadingObject, &[]);
                close(b);
            }
        }
    }
    true
}
/// A refusal without a current object remains a named card of its whole facts.
pub fn refusal(b: &mut Builder, refusal: &jig_client::contract::Refusal, words: &Words) {
    b.open(Role::Region);
    b.name(&words.say(JigPhrase::Refused, &[]));
    words.refusal(b, refusal);
    b.close();
}
