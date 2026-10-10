//! Family pages and part fragments, with jig objects left to jig (root.md, 6–9).
use crate::{
    bindings::{Bindings, Meaning},
    drawing,
    words::{Word, Words},
};
use jig_client::contract::{ObjectKey, Part, Proposer};
use jig_client_view::{Builder, Key, Owner, Role, SlotRef, State, phrases::Hole};
use jig_test_client_domain::{
    family::{Document, ItemState, Kind, Scope, Subject},
    first::{self, Card, Value},
};
use skein_lib::{Decimal, Id, Token};
pub const NAV: SlotRef =
    SlotRef { owner: Owner::Side { connector: jig_test_client_domain::FIRST }, token: Token::new(0) };
/// A fragment includes a document heading, drawing fallback, card status and form.
pub const PART_NODES: u32 = 37;
pub const NAV_NODES: u32 = 2;
pub fn navigation(b: &mut Builder, address: first::Address, words: &Words, bindings: &mut Bindings) {
    b.open(Role::Link);
    let text = words.say(Word::Navigation, &[]);
    b.name(&text);
    words.text(b, Word::Navigation, &[]);
    let binding = bindings.insert(Meaning::Action(first::Act::Go { address }));
    b.link(binding);
    b.bind(binding);
    b.close();
}
pub fn page(
    b: &mut Builder,
    side: &first::Side,
    words: &Words,
    jig: &jig_client_view::words::Words,
    bindings: &mut Bindings,
) {
    b.open(Role::Heading { level: 1 });
    words.text(b, Word::Family, &[]);
    b.close();
    let Some(id) = side.page() else {
        words.status(b, Word::Unknown, &[]);
        return;
    };
    b.open(Role::List);
    object(b, side, id, words, jig, bindings);
    let object = side.object(id).expect("page object is named");
    match &object.value {
        Value::Known(Document::Item(item)) => {
            jig_slot(b, ObjectKey::Task(item.task), words, bindings);
            for proposal in &item.proposals {
                jig_slot(b, ObjectKey::Proposal(*proposal), words, bindings);
            }
        }
        Value::Known(Document::Part(_) | Document::Connector(_)) | Value::Loading | Value::Unknown => {}
    }
    b.close();
}
fn jig_slot(b: &mut Builder, key: ObjectKey, words: &Words, bindings: &mut Bindings) {
    let binding = bindings.insert(Meaning::Object(key));
    b.slot(SlotRef { owner: Owner::Jig, token: binding.token }, &words.say(Word::Loading, &[]));
}
pub fn part(
    b: &mut Builder,
    side: &first::Side,
    part: &Part,
    generation: u32,
    words: &Words,
    jig: &jig_client_view::words::Words,
    bindings: &mut Bindings,
) {
    if !side.contains(part, generation) {
        words.status(b, Word::Unshowable, &[]);
        return;
    }
    match part {
        Part::Connector { connector } => words.text(b, Word::Connector, &[Hole::Number(u64::from(*connector))]),
        Part::Resource { path, .. } => {
            words.text(b, Word::Resource, &[Hole::Name(skein_lib::bytes::copy_of(path.bytes()))]);
        }
        Part::Kind { kind, .. } => words.text(b, Word::Kind, &[Hole::Number(u64::from(*kind))]),
        Part::Panel { .. }
        | Part::Result { .. }
        | Part::Effect { .. }
        | Part::Outbox { .. }
        | Part::Hold { .. }
        | Part::Project { .. }
        | Part::Deployment { .. } => {
            let Some(facts) = side.part(part) else {
                words.status(b, Word::Unshowable, &[]);
                return;
            };
            let id = side.find(facts.subject).expect("held part is registered");
            b.open(Role::List);
            object(b, side, id, words, jig, bindings);
            b.close();
        }
    }
}
fn object(
    b: &mut Builder,
    side: &first::Side,
    id: Id<first::Object>,
    words: &Words,
    jig: &jig_client_view::words::Words,
    bindings: &mut Bindings,
) {
    let Some(object) = side.object(id) else {
        words.status(b, Word::Unknown, &[]);
        return;
    };
    b.open(Role::Item);
    b.key(key(&object.subject));
    if let Kind::Outbox(entry) = object.subject.kind {
        words.text(b, Word::OutboxEntry, &[Hole::Number(entry)]);
    }
    b.open(Role::Region);
    let title = match &object.value {
        Value::Known(Document::Item(item)) => words.say(Word::Item, &[Hole::Number(item.key)]),
        Value::Known(Document::Part(part)) => words.say(Word::Part, &[Hole::Number(part.revision.value)]),
        Value::Known(Document::Connector(_)) | Value::Loading | Value::Unknown => words.say(Word::Family, &[]),
    };
    b.name(&title);
    match &object.value {
        Value::Loading => {
            b.open(Role::Status);
            b.name(&words.say(Word::Loading, &[]));
            b.state(State::Busy);
            words.text(b, Word::Loading, &[]);
            b.close();
        }
        Value::Unknown => words.status(b, Word::Unknown, &[]),
        Value::Known(document) => match document {
            Document::Item(item) => {
                b.open(Role::Heading { level: 2 });
                words.text(b, Word::Item, &[Hole::Number(item.key)]);
                b.close();
                let word = match item.state {
                    ItemState::Ready => Word::Ready,
                    ItemState::Held => Word::Held,
                    ItemState::Ended => Word::Ended,
                };
                words.status(b, word, &[]);
                drawing::value(b, words, item.value);
                if item.touchable {
                    button(
                        b,
                        Word::Touch,
                        first::Act::Touch { object: id.token(), revision: item.revision },
                        &object.card,
                        words,
                        bindings,
                    );
                }
            }
            Document::Connector(value) => {
                crate::connector_values::document(b, value, jig);
                if let jig_test_client_domain::connector_values::ClientDocument::Object { now, .. } =
                    value.value.as_ref()
                {
                    let revision = jig_client::contract::Revision { value: now.revision };
                    b.open(Role::TextField);
                    b.name(&words.say(Word::Parameters, &[]));
                    b.value(Decimal::of(object.draft.unwrap_or(now.value)).as_bytes(), now.revision);
                    if disabled(&object.card) {
                        b.state(State::Disabled);
                    } else {
                        b.bind(bindings.insert(Meaning::Parameters { object: id.token(), revision }));
                    }
                    b.close();
                    button(
                        b,
                        Word::Apply,
                        first::Act::Touch { object: id.token(), revision },
                        &object.card,
                        words,
                        bindings,
                    );
                }
            }
            Document::Part(part) => {
                b.open(Role::Heading { level: 3 });
                words.text(b, Word::Part, &[Hole::Number(part.revision.value)]);
                b.close();
                drawing::value(b, words, part.value);
                if let Some(parameters) = part.parameters
                    && parameters.amendable
                {
                    b.open(Role::TextField);
                    b.name(&words.say(Word::Parameters, &[]));
                    let value = object.draft.unwrap_or(parameters.value);
                    b.value(Decimal::of(value).as_bytes(), part.revision.value);
                    if disabled(&object.card) {
                        b.state(State::Disabled);
                    } else {
                        b.bind(
                            bindings.insert(Meaning::Parameters { object: id.token(), revision: parameters.revision }),
                        );
                    }
                    b.close();
                    button(
                        b,
                        Word::Apply,
                        first::Act::Touch { object: id.token(), revision: parameters.revision },
                        &object.card,
                        words,
                        bindings,
                    );
                }
            }
        },
    }
    card(b, &object.card, words, jig);
    b.close();
    b.close();
}
fn disabled(card: &Card) -> bool {
    match card {
        Card::Deciding { .. } | Card::Unknown => true,
        Card::Open | Card::Refused { .. } | Card::Done { .. } => false,
    }
}
fn button(b: &mut Builder, word: Word, action: first::Act, card: &Card, words: &Words, bindings: &mut Bindings) {
    b.open(Role::Button);
    b.name(&words.say(word, &[]));
    words.text(b, word, &[]);
    if disabled(card) {
        b.state(State::Disabled);
    } else {
        b.bind(bindings.insert(Meaning::Action(action)));
    }
    b.close();
}
fn card(b: &mut Builder, card: &Card, words: &Words, jig: &jig_client_view::words::Words) {
    match card {
        Card::Open => {}
        Card::Unknown => words.status(b, Word::Unknown, &[]),
        Card::Deciding { progress, .. } => {
            let word = match progress {
                skein_lib::RequestProgress::InFlight => Word::Deciding,
                skein_lib::RequestProgress::Retrying => Word::Retrying,
                skein_lib::RequestProgress::Parked => Word::Parked,
                skein_lib::RequestProgress::Unknown => Word::Unknown,
            };
            words.status(b, word, &[]);
        }
        Card::Refused { answer } | Card::Done { answer } => match answer {
            first::Answer::Touched { revision, .. } => words.status(b, Word::Touched, &[Hole::Number(revision.value)]),
            first::Answer::Moved { revision, .. } => words.status(b, Word::Moved, &[Hole::Number(revision.value)]),
            first::Answer::Gone => words.status(b, Word::Gone, &[]),
            first::Answer::Refused(refusal) => jig.refusal(b, refusal.refusal()),
            first::Answer::Connector(value) => {
                let saying = jig
                    .book
                    .say(jig.language, crate::connector_values_words::Word::Answer.id(), &[])
                    .expect("checked application terminal name");
                b.open(Role::Status);
                b.name(&saying.text);
                crate::connector_values::answer(b, value, jig);
                b.close();
            }
        },
    }
}

/// Stable card identity from its public document subject (testing.md, 6).
/// Slab identities remain the side's private routing vocabulary.
#[must_use]
pub fn key(subject: &Subject) -> Key {
    let project = match subject.scope {
        Scope::Project(project) => u64::from(project),
        Scope::Deployment => 0,
    };
    match subject.kind {
        Kind::Item(item) => Key { owner: NAV.owner, kind: 10, a: project, b: item },
        Kind::Panel(task) => Key { owner: NAV.owner, kind: 11, a: project, b: task },
        Kind::Result(task) => Key { owner: NAV.owner, kind: 12, a: project, b: task },
        Kind::Effect(proposal) => match proposal.proposer {
            Proposer::Task(task) => Key { owner: NAV.owner, kind: 13, a: task, b: proposal.number },
            Proposer::Party(party) => Key { owner: NAV.owner, kind: 19, a: party, b: proposal.number },
        },
        Kind::OwnerEffect(reference) => {
            let supplied_scope = match subject.scope {
                Scope::Project(project) => u64::from(project),
                Scope::Deployment => u64::MAX,
            };
            Key { owner: NAV.owner, kind: 20, a: supplied_scope, b: reference }
        }
        Kind::Outbox(entry) => match subject.scope {
            Scope::Project(project) => Key { owner: NAV.owner, kind: 17, a: u64::from(project), b: entry },
            Scope::Deployment => Key { owner: NAV.owner, kind: 18, a: 0, b: entry },
        },
        Kind::Hold(task) => Key { owner: NAV.owner, kind: 14, a: project, b: task },
        Kind::Project(project) => Key { owner: NAV.owner, kind: 15, a: u64::from(project), b: 0 },
        Kind::Deployment => Key { owner: NAV.owner, kind: 16, a: 0, b: 0 },
    }
}
