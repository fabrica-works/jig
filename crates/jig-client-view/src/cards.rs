//! One shared card frame in every place (client/domain/objects.md, section 2).
//! The place changes context only. Card state and terminal facts are owned by
//! the domain; this module neither changes them nor judges offered acts.
use crate::JigBindings;
use crate::phrases::jig::JigPhrase;
use crate::words::Words;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role};

use crate::{
    Borrowed,
    action::Act,
    objects::{Card, Left, Object},
};
use jig_client::contract::Chip;
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
    client: &Borrowed<'_>,
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
        Card::Deciding { .. } => b.state(skein_ui::State::Busy),
        Card::Open | Card::NotSent { .. } | Card::Refused { .. } | Card::Unknown { .. } | Card::Leaving { .. } => {}
    }
    b.open(Role::Heading { level: 2 });
    b.text(skein_ui::Run::Plain, title);
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
    if let Card::Refused { refusal } = &facts.card {
        if let jig_client::contract::Refusal::Circle { tasks, .. } = refusal {
            for task in tasks {
                chip::named(b, client, *task, words, bindings);
            }
        }
        if let jig_client::contract::Refusal::Kept { pending } = refusal {
            for task in pending {
                chip::named(b, client, *task, words, bindings);
            }
        }
        if let jig_client::contract::Refusal::Dependency { dependency, .. } = refusal {
            chip::named(b, client, *dependency, words, bindings);
        }
    }
    if client.ui.refused_amendments.get(&facts.key).is_some()
        && let Card::Refused { refusal: jig_client::contract::Refusal::Authority { proposable: Some(true), .. } } =
            facts.card
    {
        crate::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::ProposeInstead,
            &[],
            Act::ProposeInstead { object: object.token() },
        );
    }
    true
}
/// Build the card's authoritative state, including full refusal and departure facts.
pub fn state(b: &mut Builder, card: &Card, object: Id<Object>, words: &Words, bindings: &mut JigBindings) {
    let text_limit = bindings.text_limit();
    match card {
        Card::NotSent { why } => words.status(b, why.phrase(), &[]),
        Card::Open => words.status(b, JigPhrase::Open, &[]),
        Card::Deciding { progress, .. } => words.progress(b, *progress),
        Card::Refused { refusal } => {
            b.open(Role::Status);
            b.name(&words.say(JigPhrase::Refused, &[]));
            words.refusal(b, refusal, text_limit);
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
                Left::Historical { .. } => words.text(b, JigPhrase::Leaving, &[]),
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
    client: &Borrowed<'_>,
    object: Id<Object>,
    place: &Place,
    words: &Words,
    bindings: &mut JigBindings,
) -> bool {
    let text_limit = bindings.text_limit();
    let Some(facts) = client.objects().get(object) else {
        return false;
    };
    if let Some(rendered) = received_card(b, client, object, place, words, bindings, facts.reply.is_some()) {
        return rendered;
    }
    match &facts.body {
        Some(crate::objects::Body::Historical(_)) => return false,
        Some(crate::objects::Body::OwnerProposal(_)) => {
            proposal::owner_proposal(b, client, object, place, words, bindings);
        }
        Some(crate::objects::Body::Proposal(_)) => {
            proposal::proposal(b, client, object, place, words, bindings);
        }
        Some(crate::objects::Body::Question(_)) => {
            question::question(b, client, object, place, words, bindings);
        }
        Some(crate::objects::Body::OwnerPersonTask(_)) => {
            crate::owner_person_task::card(b, client, object, place, words, bindings);
        }
        Some(crate::objects::Body::PersonTask(_)) => {
            person::person_task(b, client, object, place, words, bindings);
        }
        Some(crate::objects::Body::Result(_)) => {
            result::result(b, client, object, place, words, bindings);
        }
        Some(crate::objects::Body::Chip(chip)) => {
            let title =
                words.say(JigPhrase::ChipNumber, &[Hole::Number(chip.task), crate::words::name(chip.title.bytes())]);
            if open(b, client, object, place, &title, words, bindings) {
                chip::chip(b, client, chip, words, bindings);
                close(b);
            }
        }
        Some(crate::objects::Body::Task(task)) => {
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
                task_footer(b, client, object, task, words, bindings);
            }
        }
        Some(crate::objects::Body::OwnerEscalation(_)) => {
            return owner_escalation::card(b, client, object, place, words, bindings);
        }
        Some(crate::objects::Body::Escalation(_)) => {
            escalation::escalation(b, client, object, place, words, bindings);
        }
        Some(crate::objects::Body::Note { document, .. }) => {
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
                        Act::Go { address: crate::address::Address::task(*task) },
                    );
                }
                common::answer(b, &facts, words, text_limit);
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
fn received_card(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: Id<Object>,
    place: &Place,
    words: &Words,
    bindings: &mut JigBindings,
    reply: bool,
) -> Option<bool> {
    if crate::owner_historical::card(b, client, object, place, words, bindings) {
        return Some(true);
    }
    if reply {
        return Some(result::reply(b, client, object, place, words, bindings));
    }
    None
}
/// A refusal without a current object remains a named card of its whole facts.
pub fn refusal(b: &mut Builder, refusal: &jig_client::contract::Refusal, words: &Words, text_limit: u32) {
    b.open(Role::Region);
    b.name(&words.say(JigPhrase::Refused, &[]));
    words.refusal(b, refusal, text_limit);
    b.close();
}

pub mod owner_escalation;

fn task_footer(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: Id<Object>,
    task: &jig_client::contract::Task,
    words: &Words,
    bindings: &mut JigBindings,
) {
    close(b);
    if crate::confirm::can_close(client, task) {
        crate::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::ActClose,
            &[],
            Act::CloseChat { object: object.token() },
        );
    }
}
