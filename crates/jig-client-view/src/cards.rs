//! One shared card frame in every place (client/domain/objects.md, section 2).
//! The place changes context only. Card state and terminal facts are owned by
//! the domain; this module neither changes them nor judges offered acts.
use crate::{
    Builder, JigBindings, Key, Role,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::{
    Domain,
    action::Act,
    contract::ObjectKey,
    objects::{Card, Left, Object},
};
use skein_lib::{Id, List};

/// The context line chosen by the page that shows this same object.
#[derive(Debug)]
pub enum Place {
    /// The inbox adds project and lineage numbers.
    Inbox { lineage: List<u64> },
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
    let key = match facts.key {
        ObjectKey::Task(task) => Key { owner: crate::Owner::Jig, kind: 0, a: task, b: 0 },
        ObjectKey::Proposal(key) => Key { owner: crate::Owner::Jig, kind: 1, a: key.proposer, b: key.number },
        ObjectKey::Escalation(task) => Key { owner: crate::Owner::Jig, kind: 2, a: task, b: 0 },
        ObjectKey::Question { task, message } => Key { owner: crate::Owner::Jig, kind: 3, a: task, b: message },
        ObjectKey::PersonTask(task) => Key { owner: crate::Owner::Jig, kind: 4, a: task, b: 0 },
        ObjectKey::Result(task) => Key { owner: crate::Owner::Jig, kind: 5, a: task, b: 0 },
        ObjectKey::Note { project, note } => Key { owner: crate::Owner::Jig, kind: 6, a: u64::from(project), b: note },
    };
    b.key(key);
    b.open(Role::Region);
    b.name(title);
    b.open(Role::Heading { level: 2 });
    b.text(crate::Run::Plain, title);
    b.close();
    match place {
        Place::Inbox { lineage } => {
            if let Some(project) = facts.project {
                words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(project))]);
            }
            for task in lineage {
                words.text(b, JigPhrase::ContextTask, &[Hole::Number(*task)]);
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
