//! An escalation shows the task owner’s facts and independent offered decisions
//! (client/domain/objects.md, 5; contract.md, 5.1–5.3).
use crate::JigBindings;
use crate::phrases::jig::JigPhrase;
use crate::words::Words;
use skein_ui::Builder;
use skein_ui::phrases::Hole;

use crate::{
    Domain,
    objects::{Body, Object},
};
use skein_lib::Id;

/// Build a held task’s escalation without reconstructing another owner’s context.
pub fn card(
    b: &mut Builder,
    client: &Domain,
    object: Id<Object>,
    place: &super::Place,
    words: &Words,
    bindings: &mut JigBindings,
) -> bool {
    let Some(facts) = client.objects().get(object) else {
        return false;
    };
    let document = match &facts.body {
        Some(Body::OwnerEscalation(document)) => document,
        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::Task(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Escalation(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => return false,
    };
    if document.acts.capacity() > jig_client::contract::MAX_OFFERED_ACTS {
        return false;
    }
    if !super::open(b, client, object, place, &words.say(JigPhrase::Escalation, &[]), words, bindings) {
        return false;
    }
    super::chip::chip(b, client, &document.task.chip, words, bindings);
    if let Some(taker) = &document.task.taken_by {
        words.text(b, JigPhrase::TakenByNumber, &[Hole::Number(*taker)]);
    }
    crate::sections::task::overview(b, client, &document.task, words, bindings);
    crate::sections::task::budget(b, &document.task, words, bindings);
    super::common::answer(b, facts, words);
    super::common::acts(b, &facts.card, &document.acts, object, words, bindings);
    super::close(b);
    true
}
