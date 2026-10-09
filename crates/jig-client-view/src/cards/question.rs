//! A whole question and its one answer (client/domain/objects.md, section 6).
use crate::{Builder, JigBindings, phrases::jig::JigPhrase, words::Words};
use jig_client::{
    Domain,
    fields::Purpose,
    objects::{Object, question::facts},
};
use skein_lib::Id;
/// Build the shared question; its answer field belongs to its open confirmation.
pub fn question(
    b: &mut Builder,
    client: &Domain,
    object: Id<Object>,
    place: &super::Place,
    words: &Words,
    bindings: &mut JigBindings,
) {
    let Some(fact) = client.objects().get(object) else {
        return;
    };
    let Some(document) = facts(fact) else {
        return;
    };
    let title = words.say(JigPhrase::Question, &[]);
    if !super::open(b, client, object, place, &title, words, bindings) {
        return;
    }
    super::chip::chip(b, client, &document.task, words, bindings);
    crate::markdown::read(document.words.bytes(), 2, b);
    words.waiting(b, client.wall(), document.since);
    if super::common::answering(client, object) {
        super::common::field(b, client, Purpose::Answer, JigPhrase::AnswerField, words, bindings);
    }
    if let Some(ending) = &document.ending {
        words.ending(b, ending);
    }
    super::common::answer(b, fact, words);
    super::common::acts(b, &fact.card, &document.acts, object, words, bindings);
    super::close(b);
}
