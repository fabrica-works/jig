//! Family pages and part fragments, with jig objects left to jig (root.md, 6–9).
use crate::{
    bindings::{Bindings, Meaning},
    drawing,
    words::{Word, Words},
};
use jig_client_domain::contract::{ObjectKey, Part, Proposer};
use jig_client_test_domain::{
    family::{Document, ItemState, Kind, Scope, Subject},
    first::{self, Card, Value},
};
use jig_client_view::Owner;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Key, Role, SlotRef, State};

use skein_lib::{Decimal, Id, Token};
pub const NAV: SlotRef =
    SlotRef { owner: (Owner::Application { number: jig_client_test_domain::FIRST }).opaque(), token: Token::new(0) };
/// A fragment includes a document heading, drawing fallback, card status and form.
pub const PART_NODES: u32 = 79;
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
    root: &jig_client_test_domain::Domain,
    words: &Words,
    jig: &jig_client_view::words::Words,
    bindings: &mut Bindings,
    hide_effect: bool,
) {
    let side = root.first();
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
        Value::Known(document @ Document::Item(item)) => {
            jig_slot(b, ObjectKey::Task(item.task), words, bindings);
            for (index, proposal) in item.proposals.iter().enumerate() {
                if index == 0 {
                    crate::proposal::section(b, root, *proposal, jig, words, bindings, hide_effect);
                }
                jig_slot(b, ObjectKey::Proposal(*proposal), words, bindings);
            }
            for section in document.sections().into_boxed() {
                let binding = bindings.insert(Meaning::Section(section));
                b.slot(SlotRef { owner: Owner::Jig.opaque(), token: binding.token }, &words.say(Word::Loading, &[]));
            }
        }
        Value::Known(Document::Part(_) | Document::Connector(_))
        | Value::Loading
        | Value::Unknown
        | Value::Unsupported => {}
    }
    b.close();
}
fn jig_slot(b: &mut Builder, key: ObjectKey, words: &Words, bindings: &mut Bindings) {
    let binding = bindings.insert(Meaning::Object(key));
    b.slot(SlotRef { owner: Owner::Jig.opaque(), token: binding.token }, &words.say(Word::Loading, &[]));
}
pub fn part(
    b: &mut Builder,
    side: &first::Side,
    part: &Part,
    words: &Words,
    jig: &jig_client_view::words::Words,
    bindings: &mut Bindings,
) {
    if !side.contains(part) {
        words.status(b, Word::Unshowable, &[]);
        return;
    }
    match part {
        Part::Connector { connector } => words.text(b, Word::Connector, &[Hole::Number(u64::from(*connector))]),
        Part::Resource { path, .. } => {
            jig.text(b, jig_client_view::phrases::jig::JigPhrase::ResourceSegments, &[]);
            for segment in path {
                jig_client_view::literal_segment::render(b, segment);
            }
        }
        Part::Kind { kind, .. } => words.text(b, Word::Kind, &[Hole::Number(u64::from(*kind))]),
        Part::Panel { .. }
        | Part::Result { .. }
        | Part::Effect { .. }
        | Part::OwnerEffect { .. }
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
    let text_limit = bindings.text_limit();
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
        Value::Known(Document::Connector(_)) | Value::Loading | Value::Unknown | Value::Unsupported => {
            words.say(Word::Family, &[])
        }
    };
    b.name(&title);
    departure(b, object.terminal, words);
    match &object.value {
        Value::Loading => {
            if !first::unavailable(object) {
                b.open(Role::Status);
                b.name(&words.say(Word::Loading, &[]));
                b.state(State::Busy);
                words.text(b, Word::Loading, &[]);
                b.close();
            }
        }
        Value::Unknown => words.status(b, Word::Unknown, &[]),
        Value::Unsupported => words.status(b, Word::Unshowable, &[]),
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
                if item.touchable && !first::unavailable(object) {
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
                connector_form(b, object, id, value.value.as_ref(), words, bindings);
            }
            Document::Part(part) => {
                b.open(Role::Heading { level: 3 });
                words.text(b, Word::Part, &[Hole::Number(part.revision.value)]);
                b.close();
                drawing::value(b, words, part.value);
                if let Some(parameters) = part.parameters
                    && parameters.amendable
                    && !first::unavailable(object)
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
    card(b, &object.card, words, jig, text_limit);
    b.close();
    b.close();
}
fn departure(b: &mut Builder, terminal: Option<first::FamilyTerminal>, words: &Words) {
    let word = match terminal {
        Some(terminal) => match terminal {
            first::FamilyTerminal::Unknown => Some(Word::Unknown),
            first::FamilyTerminal::Unsupported => Some(Word::Unshowable),
            first::FamilyTerminal::NotAllowed => Some(Word::NotAllowed),
            first::FamilyTerminal::SubjectEnded => Some(Word::SubjectEnded),
            first::FamilyTerminal::Busy => Some(Word::WatchBusy),
            first::FamilyTerminal::Stopping => Some(Word::WatchStopping),
            first::FamilyTerminal::Dropped => Some(Word::Retrying),
            first::FamilyTerminal::SignedOut => Some(Word::Parked),
            first::FamilyTerminal::Closed => None,
        },
        None => None,
    };
    if let Some(word) = word {
        words.status(b, word, &[]);
    }
}
fn connector_form(
    b: &mut Builder,
    object: &first::Object,
    id: Id<first::Object>,
    value: &jig_client_test_domain::connector_values::ClientDocument,
    words: &Words,
    bindings: &mut Bindings,
) {
    match value {
        jig_client_test_domain::connector_values::ClientDocument::Object { now, .. } => {
            if !first::unavailable(object) {
                owner_object_form(b, object, id, now, words, bindings);
            }
        }
        jig_client_test_domain::connector_values::ClientDocument::Panel { .. }
        | jig_client_test_domain::connector_values::ClientDocument::Result { .. }
        | jig_client_test_domain::connector_values::ClientDocument::Effect { .. }
        | jig_client_test_domain::connector_values::ClientDocument::Outbox { .. }
        | jig_client_test_domain::connector_values::ClientDocument::Hold { .. }
        | jig_client_test_domain::connector_values::ClientDocument::Project { .. }
        | jig_client_test_domain::connector_values::ClientDocument::Deployment { .. } => {}
    }
}
fn owner_object_form(
    b: &mut Builder,
    object: &first::Object,
    id: Id<first::Object>,
    now: &jig_client_test_domain::connector_values::ApplicationObject,
    words: &Words,
    bindings: &mut Bindings,
) {
    let revision = jig_client_domain::contract::Revision { value: now.revision };
    b.open(Role::TextField);
    b.name(&words.say(Word::Parameters, &[]));
    b.value(Decimal::of(object.draft.unwrap_or(now.value)).as_bytes(), now.revision);
    if disabled(&object.card) {
        b.state(State::Disabled);
    } else {
        b.bind(bindings.insert(Meaning::Parameters { object: id.token(), revision }));
    }
    b.close();
    button(b, Word::Apply, first::Act::Touch { object: id.token(), revision }, &object.card, words, bindings);
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
    if disabled(card) {
        b.state(State::Disabled);
    } else {
        b.bind(bindings.insert(Meaning::Action(action)));
    }
    b.close();
}
fn card(b: &mut Builder, card: &Card, words: &Words, jig: &jig_client_view::words::Words, text_limit: u32) {
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
            first::Answer::Refused(refusal) => jig.refusal(b, refusal.refusal(), text_limit),
            first::Answer::Connector(value) => {
                let saying = jig
                    .book
                    .say(jig.language, crate::connector_values_words::Word::Answer.id(), &[])
                    .expect("checked application terminal name");
                b.open(Role::Status);
                b.name(&saying.text);
                crate::connector_values::answer(b, value, jig, text_limit);
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
