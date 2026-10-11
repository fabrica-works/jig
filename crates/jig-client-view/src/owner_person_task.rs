//! Pending person work, supplied contracts and shown results
//! (client/domain/objects.md, 7; view.md, 7 and 10).
use crate::{
    Builder, JigBindings, JigMeaning, Role, Run, State,
    owner_person_task_words::PersonPhrase as P,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use alloc::boxed::Box;
use jig_client::{
    Domain, Limits,
    action::Act,
    contract::{
        Chip, ChipPhase, OwnerPersonTask, OwnerTaskLink, ParameterDocument, PersonAddress, TaskContractDocument,
    },
    fields::Purpose,
    objects::Object,
};
use skein_lib::Id;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PersonFragment {
    pub nodes: u32,
    pub text_bytes: u32,
    pub text_per_node: u32,
    pub depth: u32,
    pub bindings: u32,
    pub slots: u32,
}
/// Every shown link and optional whole result, specification and contract.
#[must_use]
pub fn max_nodes(l: &Limits) -> Option<u32> {
    // The reused card/link chip renderer shows the brief phase, not the
    // standalone full metadata/result renderer. All metadata remains stored.
    let chip = crate::chip_nodes(l)?;
    crate::sizes::result_facts(l)?
        .checked_add(chip)?
        .checked_add(5)?
        .checked_mul(l.rows)?
        .checked_add(chip)?
        .checked_add(crate::specification::nodes(l)?)?
        .checked_add(crate::proposed_task::contract_nodes(l)?)?
        .checked_add(20)
}
/// Choices, entered fields and Cannot alongside the full contract.
#[must_use]
pub fn form_nodes(l: &Limits) -> Option<u32> {
    crate::proposed_task::contract_nodes(l)?.checked_add(controls_nodes(l)?)
}
/// Form controls beside a contract already rendered in the card.
pub(crate) fn controls_nodes(l: &Limits) -> Option<u32> {
    l.rows.checked_mul(3)?.checked_add(5)
}
#[must_use]
pub fn worst_case(l: &Limits, words: &Words) -> Option<PersonFragment> {
    let nodes = max_nodes(l)?;
    let mut phrase = crate::owner_words::phrase_bytes(&words.book)?;
    for phrase_id in crate::phrases::jig::ALL {
        phrase = phrase.max(words.book.capacity(phrase_id.id())?);
    }
    let text_per_node = phrase.max(crate::literal_segment::max_bytes(l.text)?).checked_mul(2)?;
    Some(PersonFragment {
        nodes,
        text_bytes: nodes.checked_mul(text_per_node)?,
        text_per_node,
        depth: l.rows.checked_add(32)?,
        bindings: nodes,
        slots: nodes,
    })
}
fn chip_fits(chip: &Chip, l: &Limits, words: &Words) -> Option<()> {
    if !chip.title.bytes.fits(l.text) {
        return None;
    }
    match chip.details.as_ref() {
        ChipPhase::Owner { details, .. } => {
            crate::owner_chip::required(details, l, words)?;
        }
        ChipPhase::Active(_)
        | ChipPhase::Waiting { .. }
        | ChipPhase::Idle { .. }
        | ChipPhase::Starting
        | ChipPhase::Running { .. }
        | ChipPhase::WaitingForWords { .. }
        | ChipPhase::BackingOff { .. }
        | ChipPhase::Procedure { .. }
        | ChipPhase::Person { .. }
        | ChipPhase::Closing(_)
        | ChipPhase::Held { .. }
        | ChipPhase::Ended { .. } => {}
    }
    Some(())
}
fn link_fits(link: &OwnerTaskLink, l: &Limits, words: &Words) -> Option<()> {
    match link {
        OwnerTaskLink::Live(chip) | OwnerTaskLink::Retained { chip, .. } => chip_fits(chip, l, words),
        OwnerTaskLink::Stored { .. } => Some(()),
    }
}
fn contract_fits(contract: &TaskContractDocument, l: &Limits) -> Option<()> {
    match contract {
        TaskContractDocument::Approval { state, part } => {
            if u32::try_from(state.len()).ok()? > l.text {
                return None;
            }
            crate::owner_chip::part_fits(part, l)?;
        }
        TaskContractDocument::Verdict { choices, .. } => {
            if choices.capacity() > l.rows {
                return None;
            }
            for choice in choices {
                if choice.fields.capacity() > l.rows {
                    return None;
                }
                for field in &choice.fields {
                    if u32::try_from(field.name.len()).ok()? > l.text {
                        return None;
                    }
                }
            }
        }
        TaskContractDocument::Report { .. } | TaskContractDocument::Change { .. } => {}
    }
    Some(())
}
/// Whole outer and nested source capacities; admitted older result fragments
/// keep their existing envelopes rather than expanding through task pages.
#[must_use]
pub fn required(value: &OwnerPersonTask, l: &Limits, words: &Words) -> Option<PersonFragment> {
    if value.results.capacity() > l.rows || value.acts.capacity() > jig_client::contract::MAX_OFFERED_ACTS {
        return None;
    }
    chip_fits(&value.task, l, words)?;
    contract_fits(&value.contract, l)?;
    let spec = &value.spec.specification;
    if !spec.words.bytes.fits(l.text) || spec.parameters.capacity() > l.rows || spec.inputs.capacity() > l.rows {
        return None;
    }
    for parameter in &spec.parameters {
        match parameter {
            ParameterDocument::Bytes { value, .. } => {
                if !value.bytes.fits(l.text) {
                    return None;
                }
            }
            ParameterDocument::Number { .. } | ParameterDocument::Resource { .. } => {}
        }
    }
    for shown in &value.results {
        link_fits(&shown.task, l, words)?;
        if let Some(result) = &shown.result {
            chip_fits(&result.task, l, words)?;
            if result.acts.capacity() > jig_client::contract::MAX_OFFERED_ACTS {
                return None;
            }
            if let Some(value) = &result.value {
                crate::owner_chip::result_nodes(value, l)?;
            }
            if let Some(details) = &result.details {
                contract_fits(&details.contract, l)?;
                if details.delegates.capacity() > l.rows {
                    return None;
                }
                for link in &details.delegates {
                    link_fits(link, l, words)?;
                }
                if let Some(followups) = &details.followups {
                    if followups.capacity() > l.rows {
                        return None;
                    }
                    for link in followups {
                        link_fits(link, l, words)?;
                    }
                }
            }
        }
    }
    worst_case(l, words)
}
fn saying(words: &Words, phrase: P, holes: &[Hole]) -> Box<[u8]> {
    words.book.say(words.language, phrase.id(), holes).expect("checked person words and typed holes").text
}
fn text(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    b.text(Run::Plain, &saying(words, phrase, holes));
}
/// Render only the supplied facts. Missing results stay beside their whole links.
pub fn render(b: &mut Builder, client: &Domain, value: &OwnerPersonTask, words: &Words, bindings: &mut JigBindings) {
    b.open(Role::Region);
    b.name(&saying(words, P::PersonTask, &[]));
    crate::cards::chip::chip(b, client, &value.task, words, bindings);
    text(b, words, P::Revision, &[Hole::Number(value.revision.value)]);
    crate::owner_result::requester(b, value.requester, words);
    crate::specification::document(b, &value.spec, words);
    match value.address {
        PersonAddress::Party(number) => text(b, words, P::Party, &[Hole::Number(number)]),
        PersonAddress::Role(number) => text(b, words, P::Role, &[Hole::Number(u64::from(number))]),
    }
    match value.taken_by {
        Some(number) => text(b, words, P::Taker, &[Hole::Number(number)]),
        None => text(b, words, P::TakerAbsent, &[]),
    }
    match value.since {
        Some(at) => text(b, words, P::Since, &[Hole::Wall(words.adjusted_wall(at))]),
        None => text(b, words, P::SinceAbsent, &[]),
    }
    match value.stall {
        Some(span) => text(b, words, P::Stall, &[Hole::Span(span)]),
        None => text(b, words, P::StallAbsent, &[]),
    }
    match value.deadline {
        Some(at) => text(b, words, P::Deadline, &[Hole::Wall(words.adjusted_wall(at))]),
        None => text(b, words, P::DeadlineAbsent, &[]),
    }
    b.open(Role::List);
    b.name(&saying(words, P::Results, &[]));
    for shown in &value.results {
        b.open(Role::Item);
        crate::owner_result::link(b, client, &shown.task, words, bindings);
        match &shown.result {
            Some(result) => crate::cards::result::facts(b, client, result, words, bindings),
            None => text(b, words, P::MissingResult, &[]),
        }
        b.close();
    }
    b.close();
    if value.cut {
        text(b, words, P::Cut, &[]);
    }
    crate::proposed_task::contract(b, &value.contract, words, bindings);
    b.close();
}
fn controls(
    b: &mut Builder,
    client: &Domain,
    object: Id<Object>,
    contract: &TaskContractDocument,
    words: &Words,
    bindings: &mut JigBindings,
) {
    let answering = crate::cards::common::answering(client, object);
    match contract {
        TaskContractDocument::Verdict { choices, .. } => {
            b.open(Role::ChoiceGroup);
            b.name(&words.say(JigPhrase::Options, &[]));
            for choice in choices {
                b.open(Role::Choice);
                let caption = saying(words, P::Choice, &[Hole::Number(u64::from(choice.code))]);
                b.name(&caption);
                b.text(Run::Plain, &caption);
                if answering && choice.fields.is_empty() {
                    if let Some(open) = client.confirmation()
                        && open.choices.option == Some(choice.code)
                    {
                        b.state(State::Selected);
                    }
                    b.bind(bindings.insert(JigMeaning::Action(Act::Choose { option: choice.code })));
                } else {
                    b.state(State::Disabled);
                }
                b.close();
                if !choice.fields.is_empty() {
                    text(b, words, P::FieldsUnavailable, &[]);
                }
            }
            b.close();
        }
        TaskContractDocument::Report { .. } => {
            if answering {
                crate::cards::common::field(b, client, Purpose::Answer, JigPhrase::AnswerField, words, bindings);
            }
        }
        TaskContractDocument::Change { .. } => text(b, words, P::ChangeUnavailable, &[]),
        TaskContractDocument::Approval { .. } => {}
    }
    if answering {
        crate::frame::control(b, words, bindings, Role::Button, JigPhrase::CannotDo, &[], Act::CannotDo);
        crate::cards::common::field(b, client, Purpose::Reason, JigPhrase::ReasonField, words, bindings);
    }
}
/// The confirmation reuses the same full contract and supported form controls.
pub fn form(
    b: &mut Builder,
    client: &Domain,
    object: Id<Object>,
    contract: &TaskContractDocument,
    words: &Words,
    bindings: &mut JigBindings,
) {
    crate::proposed_task::contract(b, contract, words, bindings);
    controls(b, client, object, contract, words, bindings);
}
/// One shared native card and original keyed answer, without a pending ending.
pub fn card(
    b: &mut Builder,
    client: &Domain,
    object: Id<Object>,
    place: &crate::cards::Place,
    words: &Words,
    bindings: &mut JigBindings,
) -> bool {
    let Some(fact) = client.objects().get(object) else {
        return false;
    };
    let Some(value) = jig_client::objects::person_task::owner_facts(fact) else {
        return false;
    };
    if !crate::cards::open(b, client, object, place, &saying(words, P::PersonTask, &[]), words, bindings) {
        return false;
    }
    render(b, client, value, words, bindings);
    controls(b, client, object, &value.contract, words, bindings);
    crate::cards::common::answer(b, fact, words);
    crate::cards::common::acts(b, &fact.card, &value.acts, object, words, bindings);
    crate::cards::close(b);
    true
}
