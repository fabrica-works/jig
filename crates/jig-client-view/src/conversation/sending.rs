//! Conversation words use the original admitted intent, never a later composer edit.
//! The draft owner clears that edit on its durable receipt; this module only keeps
//! the submitted words and their read position. See client/domain/client.md, 10.
use crate::{Ui, intents::Intent};
use jig_client::contract::{Answer, Ask, FrameParty, Limit, Text};
use skein_lib::{Id, Token};

pub(crate) fn words(ask: &Ask, task: u64) -> Option<&Text> {
    match ask {
        Ask::Message { task: written, words, .. } => {
            if *written == task {
                Some(words)
            } else {
                None
            }
        }
        Ask::Compound { operations, .. } => {
            for operation in operations {
                if let jig_client::contract::Operation::Message { task: written, words, .. } = operation
                    && *written == task
                {
                    return Some(words);
                }
            }
            None
        }
        Ask::Create { .. }
        | Ask::StartChat { .. }
        | Ask::Answer { .. }
        | Ask::Take { .. }
        | Ask::HandBack { .. }
        | Ask::Decide { .. }
        | Ask::Release { .. }
        | Ask::Amend { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Close { .. }
        | Ask::Move { .. }
        | Ask::SetGoal { .. }
        | Ask::Prioritise { .. }
        | Ask::Seen { .. }
        | Ask::EditNote { .. }
        | Ask::DeleteNote { .. }
        | Ask::ChangePolicy { .. }
        | Ask::SetRoles { .. }
        | Ask::MakeService { .. }
        | Ask::SetPool { .. } => None,
    }
}
/// Checks room before a new write is emitted; joined requests are already admitted.
pub(crate) fn admission(domain: &mut Ui, scope: u64, ask: &Ask) -> Option<Limit> {
    domain.conversation.words.bind(domain.confirmed);
    if domain.confirmed != Some(scope) || words(ask, domain.conversation.task).is_none() {
        return None;
    }
    if domain.conversation.words.admits() {
        None
    } else {
        Some(Limit::ConversationWords { limit: domain.conversation.words.rows.capacity() })
    }
}
/// Keeps the actual joined intent's snapshot, including across page navigation.
pub(crate) fn pending(domain: &mut Ui, id: Id<Intent>, lent: &jig_client::Client) {
    domain.conversation.words.bind(domain.confirmed);
    let intent = domain.intents.get(id).expect("admitted intent exists");
    if domain.confirmed != Some(intent.scope) {
        return;
    }
    let Some(text) = words(&intent.ask, domain.conversation.task) else {
        return;
    };
    let by = author(domain, lent);
    let kept = domain.conversation.words.begin_frame(id.token(), by, text.clone());
    assert!(kept, "new writes check room; restored pending intents fit their paired capacity");
    domain.changed();
}
/// Restores pending writes first, before durable activity uses spare word room.
pub(crate) fn restore(domain: &mut Ui, lent: &jig_client::Client) {
    domain.conversation.words.bind(domain.confirmed);
    let by = author(domain, lent);
    for (id, ()) in domain.intents.ids() {
        let intent = domain.intents.get(*id).expect("registered intent exists");
        if domain.confirmed == Some(intent.scope)
            && !domain.conversation.words.pending(id.token())
            && let Some(text) = words(&intent.ask, domain.conversation.task)
        {
            let kept = domain.conversation.words.begin_frame(id.token(), by.clone(), text.clone());
            assert!(kept, "pending intents restore before activity within their paired capacity");
        }
    }
}
/// A duplicate terminal never gets here; only a durable words result supplies a number.
pub(crate) fn answered(domain: &mut Ui, request: Token, answer: &Answer) {
    if !domain.conversation.words.pending(request) {
        return;
    }
    let message = match answer {
        Answer::Done(jig_client::contract::Outcome::Owner(jig_client::contract::OwnerOutcome::Compound {
            first_message,
            ..
        })) => Some(*first_message),
        Answer::Done(outcome) => match crate::outcome::route(outcome) {
            crate::outcome::Route::Words { task, message } => {
                if task == domain.conversation.task {
                    Some(message)
                } else {
                    None
                }
            }
            crate::outcome::Route::Chat { .. }
            | crate::outcome::Route::Goal { .. }
            | crate::outcome::Route::Proposed
            | crate::outcome::Route::Other => None,
        },
        Answer::Refused { refusal: _, .. } => None,
    };
    match message {
        Some(message) => {
            let applied = domain.conversation.words.sent(request, message);
            assert!(applied, "pending words exist");
        }
        None => domain.conversation.words.refused(request),
    }
    domain.changed();
}

fn author(domain: &Ui, lent: &jig_client::Client) -> Option<FrameParty> {
    let frame = domain.frame(lent).document?;
    Some(frame.party.clone())
}
