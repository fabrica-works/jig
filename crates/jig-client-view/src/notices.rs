//! Notice outcomes without a visible card (client/domain/client.md, 13).
use crate::{
    Builder, JigBindings, Role,
    cards::common,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::{
    Domain,
    action::Act,
    contract::{Answer, ObjectKey, Seen},
    notices::Kind,
    pages::IntentPurpose,
};
/// Build each visible notice as a named status, with one oldest-notice dismissal.
pub fn notices(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) {
    for (id, ()) in client.notices().ids() {
        if let Some(notice) = client.notices().get(*id) {
            facts(b, &notice.kind, words);
        }
    }
    if !client.notices().ids().is_empty()
        && client.confirmation().is_none()
        && client.project_page().form.confirmation.is_none()
    {
        crate::frame::control(b, words, bindings, Role::Button, JigPhrase::Dismiss, &[], Act::Dismiss);
    }
}
/// Whole notice facts, shared by the status renderer and vocabulary tests.
pub fn facts(b: &mut Builder, kind: &Kind, words: &Words) {
    b.open(Role::Status);
    b.name(&words.say(JigPhrase::Notice, &[]));
    match kind {
        Kind::Link(link) => words.link(b, *link),
        Kind::Answer { object, answer: outcome } => {
            object_words(b, *object, words);
            answer(b, outcome, words);
        }
        Kind::PageAnswer { answer: outcome } => answer(b, outcome, words),
        Kind::Unknown { object, act } => {
            object_words(b, *object, words);
            words.text(b, common::act_phrase(*act), &[]);
            words.text(b, JigPhrase::UnknownAction, &[]);
        }
        Kind::PageUnknown { purpose } => {
            match purpose {
                IntentPurpose::StartChat => words.text(b, JigPhrase::Conversations, &[]),
                IntentPurpose::SetGoal => words.text(b, JigPhrase::ActSetGoal, &[]),
                IntentPurpose::Seen(seen) => match seen {
                    Seen::Results { through } => words.text(b, JigPhrase::ResultsThrough, &[Hole::Number(*through)]),
                    Seen::Replies { chat, through } => {
                        words.text(b, JigPhrase::RepliesThrough, &[Hole::Number(*chat), Hole::Number(*through)]);
                    }
                },
            }
            words.text(b, JigPhrase::UnknownAction, &[]);
        }
        Kind::Refused(which) => words.refusal(b, &jig_client::contract::Refusal::Limit { which: *which }),
    }
    b.close();
}
fn answer(b: &mut Builder, answer: &Answer, words: &Words) {
    match answer {
        Answer::Done(outcome) => words.outcome(b, outcome),
        Answer::Refused(why) => words.refusal(b, why),
    }
}
fn object_words(b: &mut Builder, object: ObjectKey, words: &Words) {
    match object {
        ObjectKey::Task(task) | ObjectKey::Escalation(task) | ObjectKey::PersonTask(task) | ObjectKey::Result(task) => {
            words.text(b, JigPhrase::TaskPage, &[Hole::Number(task)]);
        }
        ObjectKey::Question { task, message } => {
            words.text(b, JigPhrase::RepliesThrough, &[Hole::Number(task), Hole::Number(message)]);
        }
        ObjectKey::Proposal(key) => {
            words.proposal(b, key);
        }
        ObjectKey::Note { project, note } => {
            words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(project))]);
            words.text(b, JigPhrase::MadeNote, &[Hole::Number(note)]);
        }
    }
}
