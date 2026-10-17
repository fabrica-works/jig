//! Escalations keep the whole hold story and document-offered acts (objects.md, 5).
use crate::JigBindings;
use crate::phrases::jig::JigPhrase;
use crate::words::Words;
use skein_ui::Builder;
use skein_ui::phrases::Hole;

use crate::{
    Domain,
    objects::{Body, Object},
};
use jig_client::contract::{Hold, Phase, SpendLine};
use skein_lib::{Id, List};
/// Build the held task's card in any place, preserving prior result and holders.
pub fn escalation(
    b: &mut Builder,
    client: &Domain,
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
        Some(Body::Escalation(document)) => document,
        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::Task(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::OwnerEscalation(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => return false,
    };
    if !super::open(b, client, object, place, &words.say(JigPhrase::Escalation, &[]), words, bindings) {
        return false;
    }
    super::chip::chip(b, client, &document.task, words, bindings);
    held(b, client, &document.hold, words, bindings);
    super::common::recipient(b, &document.waiting, words);
    super::common::trail(b, &document.trail, words, text_limit);
    if let Some(ending) = &document.ending {
        words.ending(b, ending);
    }
    super::common::answer(b, facts, words, text_limit);
    super::common::acts(b, &facts.card, &document.acts, object, words, bindings);
    super::close(b);
    true
}
/// Facts retained when the task entered its hold, including spend attribution.
pub fn held(b: &mut Builder, client: &Domain, hold: &Hold, words: &Words, bindings: &mut JigBindings) {
    super::hold::reason(b, &hold.reason, words, bindings);
    words.waiting(b, client.wall(), hold.since);
    words.text(b, JigPhrase::BeforeHold, &[]);
    phase(b, hold.before, words);
    if let Some(value) = &hold.result {
        super::result::value_nodes(b, client, value, words, bindings);
    }
    spend(b, &hold.spend, words);
}
/// Compact prior phase is a fact, distinct from the live chip's detailed phase.
pub fn phase(b: &mut Builder, phase: Phase, words: &Words) {
    let label = match phase {
        Phase::Waiting => JigPhrase::Waiting,
        Phase::Idle => JigPhrase::Idle,
        Phase::Due => JigPhrase::DuePhase,
        Phase::Preparing => JigPhrase::PreparingPhase,
        Phase::Claimed => JigPhrase::ClaimedPhase,
        Phase::Running => JigPhrase::Running,
        Phase::Active => JigPhrase::ActivePhase,
        Phase::BackingOff => JigPhrase::Retrying,
        Phase::Closing => JigPhrase::ClosingPhase,
        Phase::Held => JigPhrase::Held,
        Phase::Ended => JigPhrase::Ended,
        Phase::Done => JigPhrase::Done,
        Phase::Failed => JigPhrase::FailedPhase,
        Phase::Cancelled => JigPhrase::CancelledPhase,
    };
    words.text(b, label, &[]);
}
/// Run and delegate attribution remains exactly the committed snapshot.
pub fn spend(b: &mut Builder, rows: &List<SpendLine>, words: &Words) {
    for row in rows {
        match row.attempt {
            Some(attempt) => words.text(
                b,
                JigPhrase::SpentRun,
                &[Hole::Number(row.task), Hole::Number(attempt), Hole::Amount(row.spent)],
            ),
            None => words.text(b, JigPhrase::SpentDelegate, &[Hole::Number(row.task), Hole::Amount(row.spent)]),
        }
    }
}
