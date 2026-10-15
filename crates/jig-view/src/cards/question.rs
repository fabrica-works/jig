//! A whole question and its one answer (client/domain/objects.md, section 6).
use crate::ui::Builder;
use crate::{JigBindings, phrases::jig::JigPhrase, words::Words};

use jig_domain_client::{
    Domain,
    contract::OwnerRequester,
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
) -> bool {
    let Some(fact) = client.objects().get(object) else {
        return false;
    };
    let Some(document) = facts(fact) else {
        return false;
    };
    let title = words.say(JigPhrase::Question, &[]);
    if !super::open(b, client, object, place, &title, words, bindings) {
        return false;
    }
    super::chip::chip(b, client, &document.task, words, bindings);
    if let Some(from) = document.from {
        let (phrase, number) = match from {
            OwnerRequester::Task(number) => (JigPhrase::QuestionFromTask, number),
            OwnerRequester::Person(number) => (JigPhrase::QuestionFromPerson, number),
            OwnerRequester::Deployment { project } => (JigPhrase::QuestionFromDeployment, u64::from(project)),
        };
        words.text(b, phrase, &[crate::ui::phrases::Hole::Number(number)]);
    }
    crate::ui::markdown::read(document.words.bytes(), 2, b);
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
    true
}
