//! Person tasks present their contract and inputs (client/domain/objects.md, section 7).
use crate::phrases::jig::JigPhrase;
use crate::words::{Words, name};
use crate::{JigBindings, JigMeaning};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run, State};

use jig_client_domain::contract::{OptionLine, PersonContract};
use jig_domain_client::{
    Domain,
    action::Act,
    fields::Purpose,
    objects::{Object, person_task::facts},
};
use skein_lib::{Id, List};
/// Build the person-task card, addressed and taken from committed facts.
pub fn person_task(
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
    let title = words.say(JigPhrase::PersonTask, &[]);
    if !super::open(b, client, object, place, &title, words, bindings) {
        return false;
    }
    super::chip::chip(b, client, &document.task, words, bindings);
    words.actor(b, document.requester);
    crate::specification::document(b, &document.spec, words);
    super::common::recipient(b, &document.addressed, words);
    if let Some(by) = &document.taken {
        super::common::taken_by(b, by, words);
    }
    if let Some(since) = document.since {
        words.text(b, JigPhrase::Since, &[Hole::Wall(words.adjusted_wall(since))]);
    }
    if let Some(stall) = document.stall {
        words.text(b, JigPhrase::StallSpan, &[Hole::Span(stall)]);
    }
    if let Some(deadline) = document.deadline {
        words.deadline(b, deadline);
    }
    b.open(Role::List);
    for result in &document.inputs {
        b.open(Role::Item);
        super::result::facts(b, client, result, words, bindings);
        b.close();
    }
    b.close();
    contract(b, client, &document.contract, words, bindings);
    if let Some(ending) = &document.ending {
        words.ending(b, ending);
    }
    super::common::answer(b, fact, words);
    super::common::acts(b, &fact.card, &document.acts, object, words, bindings);
    super::close(b);
    true
}
/// Show the contract's named state or options without a second answer form.
pub fn contract(
    b: &mut Builder,
    client: &Domain,
    contract: &PersonContract,
    words: &Words,
    bindings: &mut JigBindings,
) {
    render_contract(b, client, contract, false, words, bindings);
}
/// Build the sole editable form in the object's Answer confirmation.
pub fn form(
    b: &mut Builder,
    client: &Domain,
    object: Id<Object>,
    contract: &PersonContract,
    words: &Words,
    bindings: &mut JigBindings,
) {
    let answering = super::common::answering(client, object);
    render_contract(b, client, contract, answering, words, bindings);
}
fn render_contract(
    b: &mut Builder,
    client: &Domain,
    contract: &PersonContract,
    answering: bool,
    words: &Words,
    bindings: &mut JigBindings,
) {
    match contract {
        PersonContract::Choice { options: rows } => options(b, client, rows, answering, words, bindings),
        PersonContract::Approval { state, part } => {
            words.text(b, JigPhrase::ApprovalState, &[name(state.bytes())]);
            super::common::part(b, part, words, bindings);
        }
        PersonContract::Words => {
            if answering {
                super::common::field(b, client, Purpose::Answer, JigPhrase::AnswerField, words, bindings);
            }
        }
    }
    if answering {
        crate::frame::control(b, words, bindings, Role::Button, JigPhrase::CannotDo, &[], Act::CannotDo);
        super::common::field(b, client, Purpose::Reason, JigPhrase::ReasonField, words, bindings);
    }
}
fn options(
    b: &mut Builder,
    client: &Domain,
    options: &List<OptionLine>,
    answering: bool,
    words: &Words,
    bindings: &mut JigBindings,
) {
    b.open(if answering { Role::ChoiceGroup } else { Role::List });
    b.name(&words.say(JigPhrase::Options, &[]));
    for option in options {
        b.open(if answering { Role::Choice } else { Role::Item });
        b.name(option.words.bytes());
        b.text(Run::Plain, option.words.bytes());
        if answering && option.fields.is_empty() {
            if let Some(open) = client.confirmation()
                && open.choices.option == Some(option.number)
            {
                b.state(State::Selected);
            }
            b.bind(bindings.insert(JigMeaning::Action(Act::Choose { option: option.number })));
        } else if answering {
            b.state(State::Disabled);
        }
        b.close();
    }
    b.close();
}
