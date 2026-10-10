//! Keep: family pages, owner keys and jig object slots (root.md, 6–9).
//! Replace: incidents, health and observability'service supplied facts.
use crate::{
    bindings::{Bindings, Meaning},
    drawing,
    words::{Word, Words},
};
use jig_client::contract::{ObjectKey, Part};
use jig_client_view::{Builder, Key, Owner, Role, Run, SlotRef, phrases::Hole};
use jig_ops_client_domain::{
    Action,
    observability::{
        self, Value,
        documents::{Document, Incident, Kind, Scope, State, Subject},
    },
};
use skein_lib::Token;
pub const NAV: SlotRef = SlotRef {
    owner: Owner::Side { connector: jig_ops_connectors::number(jig_ops_connectors::Connector::Observability) },
    token: Token::new(0),
};
pub fn navigation(builder: &mut Builder, project: u32, words: &Words, bindings: &mut Bindings) {
    link(builder, Word::Incidents, &[], observability::Address::Incidents { project, open: true }, words, bindings);
}
fn link(
    builder: &mut Builder,
    word: Word,
    holes: &[Hole],
    address: observability::Address,
    words: &Words,
    bindings: &mut Bindings,
) {
    builder.open(Role::Link);
    builder.name(&words.say(word, holes));
    words.text(builder, word, holes);
    let binding = bindings.insert(Meaning::Action(Action::Observability(observability::Act::Go { address })));
    builder.bind(binding);
    builder.link(binding);
    builder.close();
}
pub fn page(builder: &mut Builder, side: &observability::Side, words: &Words, bindings: &mut Bindings) {
    builder.open(Role::Heading { level: 1 });
    words.text(builder, Word::Incidents, &[]);
    builder.close();
    let Some(id) = side.page() else {
        words.status(builder, Word::Unknown, &[]);
        return;
    };
    let object = side.object(id).expect("page is named");
    builder.open(Role::Item);
    builder.key(key(&object.subject));
    match &object.value {
        Value::Loading => words.status(builder, Word::Loading, &[]),
        Value::Unknown => words.status(builder, Word::Unknown, &[]),
        Value::Known(document) => document_view(builder, document, words, bindings),
    }
    builder.close();
}
fn jig_slot(builder: &mut Builder, key: ObjectKey, words: &Words, bindings: &mut Bindings) {
    let binding = bindings.insert(Meaning::Object(key));
    builder.slot(SlotRef { owner: Owner::Jig, token: binding.token }, &words.say(Word::Loading, &[]));
}
pub fn show_incident(builder: &mut Builder, incident: &Incident, words: &Words, bindings: &mut Bindings, slots: bool) {
    builder.open(Role::Region);
    builder.name(&words.say(Word::Incident, &[Hole::Number(incident.number)]));
    builder.open(Role::Heading { level: 2 });
    words.text(builder, Word::Incident, &[Hole::Number(incident.number)]);
    builder.close();
    show_service(builder, &incident.service, words);
    words.status(
        builder,
        match incident.state {
            State::Alerting => Word::Open,
            State::Resolved => Word::Resolved,
        },
        &[],
    );
    words.text(builder, Word::Severity, &[Hole::Number(u64::from(incident.severity))]);
    words.text(builder, Word::Raised, &[Hole::Wall(incident.raised)]);
    drawing::rate(builder, words, incident);
    match incident.lead {
        Some(lead) => {
            words.text(builder, Word::Lead, &[Hole::Number(lead)]);
            if slots {
                jig_slot(builder, ObjectKey::Task(lead), words, bindings);
            }
        }
        None => words.status(builder, Word::NoLead, &[]),
    }
    if slots {
        for proposal in &incident.proposals {
            jig_slot(builder, ObjectKey::Proposal(*proposal), words, bindings);
        }
    }
    builder.open(Role::List);
    builder.name(&words.say(Word::Timeline, &[]));
    for point in &incident.timeline {
        builder.open(Role::Item);
        words.text(builder, Word::At, &[Hole::Wall(point.at)]);
        if let Some(health) = point.health {
            health_view(builder, health, words);
        }
        builder.close();
    }
    if incident.cut {
        words.status(builder, Word::Cut, &[]);
    }
    builder.close();
    builder.close();
}
fn show_service(builder: &mut Builder, service: &observability::documents::Service, words: &Words) {
    words.text(builder, Word::Service, &[Hole::Name(skein_lib::bytes::copy_of(&service.name))]);
    words.text(builder, Word::InEnvironment, &[Hole::Name(skein_lib::bytes::copy_of(&service.environment))]);
}
fn health_view(builder: &mut Builder, health: observability::documents::Health, words: &Words) {
    words.text(builder, Word::Healthy, &[Hole::Number(u64::from(health.healthy_replicas))]);
    words.text(builder, Word::Errors, &[Hole::Number(u64::from(health.errors))]);
    words.text(builder, Word::Rate, &[Hole::Number(u64::from(health.error_rate_percent))]);
    builder.text(Run::Plain, b" %");
    words.text(builder, Word::Observed, &[Hole::Wall(health.observed)]);
}
fn document_view(builder: &mut Builder, document: &Document, words: &Words, bindings: &mut Bindings) {
    match document {
        Document::Incident(incident) => show_incident(builder, incident, words, bindings, true),
        Document::Incidents { incidents, .. } => {
            builder.open(Role::List);
            for incident in incidents {
                builder.open(Role::Item);
                builder.key(key(&Subject {
                    scope: Scope::Project(incident.project),
                    kind: Kind::Incident(incident.number),
                }));
                link(
                    builder,
                    Word::Incident,
                    &[Hole::Number(incident.number)],
                    observability::Address::Incident { project: incident.project, incident: incident.number },
                    words,
                    bindings,
                );
                show_incident(builder, incident, words, bindings, false);
                builder.close();
            }
            builder.close();
        }
        Document::Part(part) => {
            if let Some(service) = &part.service {
                show_service(builder, service, words);
            }
            if let Some(health) = part.health {
                health_view(builder, health, words);
            }
            if let Some(incident) = &part.incident {
                show_incident(builder, incident, words, bindings, false);
            }
            for incident in &part.incidents {
                show_incident(builder, incident, words, bindings, false);
            }
            for service in &part.services {
                show_service(builder, &service.service, words);
                if let Some(health) = service.health {
                    health_view(builder, health, words);
                }
            }
            for entry in &part.effects {
                words.text(builder, Word::Entry, &[Hole::Number(entry.entry)]);
                words.text(builder, Word::Task, &[Hole::Number(entry.task)]);
                words.text(builder, Word::Silence, &[Hole::Number(entry.alert)]);
                words.text(builder, Word::End, &[Hole::Wall(entry.until)]);
                words.text(builder, Word::Attempt, &[Hole::Number(u64::from(entry.attempt))]);
                words.text(builder, Word::Deadline, &[Hole::Wall(entry.deadline)]);
                words.status(
                    builder,
                    match entry.state {
                        observability::documents::EffectState::Kept => Word::Kept,
                        observability::documents::EffectState::Sent => Word::Sent,
                        observability::documents::EffectState::Uncertain => Word::Uncertain,
                        observability::documents::EffectState::Held => Word::Held,
                    },
                    &[],
                );
            }
            if let Some(view) = part.watches {
                words.text(builder, Word::Watches, &[Hole::Number(u64::from(view))]);
            }
            if let Some(view) = part.request_limit {
                words.text(builder, Word::RequestLimit, &[Hole::Number(u64::from(view))]);
            }
            if let Some(view) = part.incident_limit {
                words.text(builder, Word::IncidentLimit, &[Hole::Number(u64::from(view))]);
            }
        }
    }
}
pub fn part(
    builder: &mut Builder,
    side: &observability::Side,
    part: &Part,
    generation: u32,
    words: &Words,
    bindings: &mut Bindings,
) {
    if !side.contains(part, generation) {
        words.status(builder, Word::Unshowable, &[]);
        return;
    }
    match part {
        Part::Connector { .. } => words.text(
            builder,
            Word::Connector,
            &[Hole::Number(u64::from(jig_ops_connectors::number(jig_ops_connectors::Connector::Observability)))],
        ),
        Part::Resource { path, .. } => {
            words.text(builder, Word::Resource, &[Hole::Name(skein_lib::bytes::copy_of(path.bytes()))]);
        }
        Part::Kind { kind, .. } => words.text(builder, Word::Kind, &[Hole::Number(u64::from(*kind))]),
        Part::Panel { .. }
        | Part::Result { .. }
        | Part::Effect { .. }
        | Part::Outbox { .. }
        | Part::Hold { .. }
        | Part::Project { .. }
        | Part::Deployment { .. } => match side.part(part) {
            Some(object) => match &object.value {
                Value::Loading => words.status(builder, Word::Loading, &[]),
                Value::Unknown => words.status(builder, Word::Unknown, &[]),
                Value::Known(document) => document_view(builder, document, words, bindings),
            },
            None => words.status(builder, Word::Unshowable, &[]),
        },
    }
}
/// Public family identity; routing tokens are never used as display identity.
#[must_use]
pub fn key(service: &Subject) -> Key {
    let value = match service.scope {
        Scope::Project(part) => u64::from(part),
        Scope::Deployment => 0,
    };
    let (kind, builder) = match service.kind {
        Kind::Incidents { open } => (1, u64::from(open)),
        Kind::Incident(incident) => (2, incident),
        Kind::Panel(t) => (3, t),
        Kind::Result(t) => (4, t),
        Kind::Effect(part) => return Key { owner: NAV.owner, kind: 5, a: part.proposer, b: part.number },
        Kind::Outbox(entry) => (6, entry),
        Kind::Hold(t) => (7, t),
        Kind::Project(part) => (8, u64::from(part)),
        Kind::Deployment => (9, 0),
    };
    Key { owner: NAV.owner, kind, a: value, b: builder }
}
