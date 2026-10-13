//! Total translations of the connector client values.
use super::facts::{decode_refusal, encode_refusal};
use super::{Undecodable, d, w};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
pub(super) fn encode_key(value: &d::Key) -> w::Key {
    let d::Key { attempt, completion, position, deployment, task, purpose } = value;
    w::Key::new(
        &w::CEILINGS,
        w::KeyParts {
            attempt: *attempt,
            completion: *completion,
            position: *position,
            deployment: *deployment,
            task: *task,
            purpose: *purpose,
        },
    )
    .expect("admitted fields fit schema")
}
pub(super) fn decode_key(value: w::Key, _limits: &d::Limits) -> d::Key {
    let w::KeyParts { attempt, completion, position, deployment, task, purpose } = value.into_parts();
    d::Key { attempt, completion, position, deployment, task, purpose }
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn encode_effect(value: &d::Effect) -> w::Effect {
    let d::Effect { kind, resources, purpose, condition, target, state } = value;
    w::Effect::new(
        &w::CEILINGS,
        w::EffectParts {
            kind: *kind,
            resources: encode_effect_resources(resources),
            purpose: *purpose,
            condition: match condition {
                Some(value) => Some(*value),
                None => None,
            },
            target: *target,
            state: *state,
        },
    )
    .expect("admitted fields fit schema")
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn decode_effect(value: w::Effect, limits: &d::Limits) -> Result<d::Effect, Undecodable> {
    let w::EffectParts { kind, resources, purpose, condition, target, state } = value.into_parts();
    Ok(d::Effect {
        kind,
        resources: decode_effect_resources(resources, limits)?,
        purpose,
        condition: match condition {
            Some(value) => Some(value),
            None => None,
        },
        target,
        state,
    })
}
pub(super) fn encode_attempt(value: &d::Attempt) -> w::Attempt {
    let d::Attempt { number, sent, deadline } = value;
    w::Attempt::new(
        &w::CEILINGS,
        w::AttemptParts { number: *number, sent: sent.as_nanos(), deadline: deadline.as_nanos() },
    )
    .expect("admitted fields fit schema")
}
pub(super) fn decode_attempt(value: w::Attempt, _limits: &d::Limits) -> d::Attempt {
    let w::AttemptParts { number, sent, deadline } = value.into_parts();
    d::Attempt { number, sent: Wall::from_nanos(sent), deadline: Wall::from_nanos(deadline) }
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn encode_procedure_state(value: &d::ProcedureState) -> w::ProcedureState {
    let d::ProcedureState { project, task, number, resource, index, steps, awaiting, deadline } = value;
    w::ProcedureState::new(
        &w::CEILINGS,
        w::ProcedureStateParts {
            project: *project,
            task: *task,
            number: *number,
            resource: encode_resource_path(resource),
            index: *index,
            steps: *steps,
            awaiting: *awaiting,
            deadline: match deadline {
                Some(value) => Some(value.as_nanos()),
                None => None,
            },
        },
    )
    .expect("admitted fields fit schema")
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn decode_procedure_state(
    value: w::ProcedureState,
    limits: &d::Limits,
) -> Result<d::ProcedureState, Undecodable> {
    let w::ProcedureStateParts { project, task, number, resource, index, steps, awaiting, deadline } =
        value.into_parts();
    Ok(d::ProcedureState {
        project,
        task,
        number,
        resource: decode_resource_path(resource, limits)?,
        index,
        steps,
        awaiting,
        deadline: match deadline {
            Some(value) => Some(Wall::from_nanos(value)),
            None => None,
        },
    })
}
pub(super) fn encode_application_object(value: &d::ApplicationObject) -> w::ApplicationObject {
    let d::ApplicationObject { revision, value } = value;
    w::ApplicationObject::new(&w::CEILINGS, w::ApplicationObjectParts { revision: *revision, value: *value })
        .expect("admitted fields fit schema")
}
pub(super) fn decode_application_object(value: w::ApplicationObject, _limits: &d::Limits) -> d::ApplicationObject {
    let w::ApplicationObjectParts { revision, value } = value.into_parts();
    d::ApplicationObject { revision, value }
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn encode_outbox_entry(value: &d::OutboxEntry) -> w::OutboxEntry {
    let d::OutboxEntry { number, task, key, effect, attempt, phase } = value;
    w::OutboxEntry::new(
        &w::CEILINGS,
        w::OutboxEntryParts {
            number: *number,
            task: *task,
            key: encode_key(key),
            effect: encode_effect(effect),
            attempt: match attempt {
                Some(value) => Some(encode_attempt(value)),
                None => None,
            },
            phase: encode_effect_phase(*phase),
        },
    )
    .expect("admitted fields fit schema")
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn decode_outbox_entry(value: w::OutboxEntry, limits: &d::Limits) -> Result<d::OutboxEntry, Undecodable> {
    let w::OutboxEntryParts { number, task, key, effect, attempt, phase } = value.into_parts();
    Ok(d::OutboxEntry {
        number,
        task,
        key: decode_key(key, limits),
        effect: decode_effect(effect, limits)?,
        attempt: match attempt {
            Some(value) => Some(decode_attempt(value, limits)),
            None => None,
        },
        phase: decode_effect_phase(phase, limits),
    })
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn encode_application_release(value: &d::ApplicationRelease) -> w::ApplicationRelease {
    let d::ApplicationRelease { task, revision, again } = value;
    w::ApplicationRelease::new(
        &w::CEILINGS,
        w::ApplicationReleaseParts {
            task: *task,
            revision: *revision,
            again: match again {
                Some(value) => Some(*value),
                None => None,
            },
        },
    )
    .expect("admitted fields fit schema")
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn decode_application_release(value: w::ApplicationRelease, _limits: &d::Limits) -> d::ApplicationRelease {
    let w::ApplicationReleaseParts { task, revision, again } = value.into_parts();
    d::ApplicationRelease {
        task,
        revision,
        again: match again {
            Some(value) => Some(value),
            None => None,
        },
    }
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn encode_application_request(value: &d::ApplicationRequest) -> w::ApplicationRequest {
    let d::ApplicationRequest { object, revision, value, release } = value;
    w::ApplicationRequest::new(
        &w::CEILINGS,
        w::ApplicationRequestParts {
            object: *object,
            revision: *revision,
            value: *value,
            release: match release {
                Some(value) => Some(encode_application_release(value)),
                None => None,
            },
        },
    )
    .expect("admitted fields fit schema")
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn decode_application_request(value: w::ApplicationRequest, limits: &d::Limits) -> d::ApplicationRequest {
    let w::ApplicationRequestParts { object, revision, value, release } = value.into_parts();
    d::ApplicationRequest {
        object,
        revision,
        value,
        release: match release {
            Some(value) => Some(decode_application_release(value, limits)),
            None => None,
        },
    }
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn encode_fact(value: &d::Fact) -> w::Fact {
    let d::Fact { state, observed, pending } = value;
    w::Fact::new(
        &w::CEILINGS,
        w::FactParts {
            state: match state {
                Some(value) => Some(*value),
                None => None,
            },
            observed: observed.as_nanos(),
            pending: *pending,
        },
    )
    .expect("admitted fields fit schema")
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn decode_fact(value: w::Fact, _limits: &d::Limits) -> d::Fact {
    let w::FactParts { state, observed, pending } = value.into_parts();
    d::Fact {
        state: match state {
            Some(value) => Some(value),
            None => None,
        },
        observed: Wall::from_nanos(observed),
        pending,
    }
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn encode_description(value: &d::Description) -> w::Description {
    let d::Description { kind, resources, purpose, condition, form, recovery, price, state } = value;
    w::Description::new(
        &w::CEILINGS,
        w::DescriptionParts {
            kind: *kind,
            resources: encode_description_resources(resources),
            purpose: *purpose,
            condition: *condition,
            form: encode_form(*form),
            recovery: encode_recovery(*recovery),
            price: match price {
                Some(value) => Some(*value),
                None => None,
            },
            state: *state,
        },
    )
    .expect("admitted fields fit schema")
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn decode_description(value: w::Description, limits: &d::Limits) -> Result<d::Description, Undecodable> {
    let w::DescriptionParts { kind, resources, purpose, condition, form, recovery, price, state } = value.into_parts();
    Ok(d::Description {
        kind,
        resources: decode_description_resources(resources, limits)?,
        purpose,
        condition,
        form: decode_form(form, limits),
        recovery: decode_recovery(recovery, limits),
        price: match price {
            Some(value) => Some(value),
            None => None,
        },
        state,
    })
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn encode_client_resource(value: &d::ClientResource) -> w::ClientResource {
    let d::ClientResource { path, role, hold, fact } = value;
    w::ClientResource::new(
        &w::CEILINGS,
        w::ClientResourceParts {
            path: encode_resource_path(path),
            role: encode_resource_role(*role),
            hold: encode_hold(*hold),
            fact: match fact {
                Some(value) => Some(encode_fact(value)),
                None => None,
            },
        },
    )
    .expect("admitted fields fit schema")
}
#[expect(clippy::manual_map, reason = "step code translates optional fields without closures")]
pub(super) fn decode_client_resource(
    value: w::ClientResource,
    limits: &d::Limits,
) -> Result<d::ClientResource, Undecodable> {
    let w::ClientResourceParts { path, role, hold, fact } = value.into_parts();
    Ok(d::ClientResource {
        path: decode_resource_path(path, limits)?,
        role: decode_resource_role(role, limits),
        hold: decode_hold(hold, limits),
        fact: match fact {
            Some(value) => Some(decode_fact(value, limits)),
            None => None,
        },
    })
}
pub(super) fn encode_application_outcome(value: &d::ApplicationOutcome) -> w::ApplicationOutcome {
    match value {
        d::ApplicationOutcome::Changed { object, now } => w::ApplicationOutcome::Changed(
            w::ApplicationOutcomeChanged::new(
                &w::CEILINGS,
                w::ApplicationOutcomeChangedParts { object: *object, now: encode_application_object(now) },
            )
            .expect("admitted fields fit schema"),
        ),
        d::ApplicationOutcome::Moved { object, now } => w::ApplicationOutcome::Moved(
            w::ApplicationOutcomeMoved::new(
                &w::CEILINGS,
                w::ApplicationOutcomeMovedParts { object: *object, now: encode_application_object(now) },
            )
            .expect("admitted fields fit schema"),
        ),
        d::ApplicationOutcome::Role { held } => w::ApplicationOutcome::Role(
            w::ApplicationOutcomeRole::new(&w::CEILINGS, w::ApplicationOutcomeRoleParts { held: *held })
                .expect("admitted fields fit schema"),
        ),
        d::ApplicationOutcome::Unknown => w::ApplicationOutcome::Unknown,
        d::ApplicationOutcome::Exhausted => w::ApplicationOutcome::Exhausted,
        d::ApplicationOutcome::CoreRefused { why } => w::ApplicationOutcome::CoreRefused(
            w::ApplicationOutcomeCoreRefused::new(
                &w::CEILINGS,
                w::ApplicationOutcomeCoreRefusedParts { why: encode_refusal(why) },
            )
            .expect("admitted fields fit schema"),
        ),
    }
}
pub(super) fn decode_application_outcome(value: w::ApplicationOutcome, limits: &d::Limits) -> d::ApplicationOutcome {
    match value {
        w::ApplicationOutcome::Changed(value) => {
            let w::ApplicationOutcomeChangedParts { object, now } = value.into_parts();
            d::ApplicationOutcome::Changed { object, now: decode_application_object(now, limits) }
        }
        w::ApplicationOutcome::Moved(value) => {
            let w::ApplicationOutcomeMovedParts { object, now } = value.into_parts();
            d::ApplicationOutcome::Moved { object, now: decode_application_object(now, limits) }
        }
        w::ApplicationOutcome::Role(value) => {
            let w::ApplicationOutcomeRoleParts { held } = value.into_parts();
            d::ApplicationOutcome::Role { held }
        }
        w::ApplicationOutcome::Unknown => d::ApplicationOutcome::Unknown,
        w::ApplicationOutcome::Exhausted => d::ApplicationOutcome::Exhausted,
        w::ApplicationOutcome::CoreRefused(value) => {
            let w::ApplicationOutcomeCoreRefusedParts { why } = value.into_parts();
            d::ApplicationOutcome::CoreRefused { why: decode_refusal(why, limits) }
        }
    }
}
pub(super) fn encode_client_subject(value: &d::ClientSubject) -> w::ClientSubject {
    match value {
        d::ClientSubject::Object { project, object } => w::ClientSubject::Object(
            w::ClientSubjectObject::new(
                &w::CEILINGS,
                w::ClientSubjectObjectParts { project: *project, object: *object },
            )
            .expect("admitted fields fit schema"),
        ),
        d::ClientSubject::Panel { project, task } => w::ClientSubject::Panel(
            w::ClientSubjectPanel::new(&w::CEILINGS, w::ClientSubjectPanelParts { project: *project, task: *task })
                .expect("admitted fields fit schema"),
        ),
        d::ClientSubject::Result { project, task } => w::ClientSubject::Result(
            w::ClientSubjectResult::new(&w::CEILINGS, w::ClientSubjectResultParts { project: *project, task: *task })
                .expect("admitted fields fit schema"),
        ),
        d::ClientSubject::Effect { project, proposal } => w::ClientSubject::Effect(
            w::ClientSubjectEffect::new(
                &w::CEILINGS,
                w::ClientSubjectEffectParts { project: *project, proposal: *proposal },
            )
            .expect("admitted fields fit schema"),
        ),
        d::ClientSubject::Outbox { project, entry } => w::ClientSubject::Outbox(
            w::ClientSubjectOutbox::new(&w::CEILINGS, w::ClientSubjectOutboxParts { project: *project, entry: *entry })
                .expect("admitted fields fit schema"),
        ),
        d::ClientSubject::Hold { project, task } => w::ClientSubject::Hold(
            w::ClientSubjectHold::new(&w::CEILINGS, w::ClientSubjectHoldParts { project: *project, task: *task })
                .expect("admitted fields fit schema"),
        ),
        d::ClientSubject::Project { project } => w::ClientSubject::Project(
            w::ClientSubjectProject::new(&w::CEILINGS, w::ClientSubjectProjectParts { project: *project })
                .expect("admitted fields fit schema"),
        ),
        d::ClientSubject::Deployment => w::ClientSubject::Deployment,
    }
}
pub(super) fn decode_client_subject(value: w::ClientSubject, _limits: &d::Limits) -> d::ClientSubject {
    match value {
        w::ClientSubject::Object(value) => {
            let w::ClientSubjectObjectParts { project, object } = value.into_parts();
            d::ClientSubject::Object { project, object }
        }
        w::ClientSubject::Panel(value) => {
            let w::ClientSubjectPanelParts { project, task } = value.into_parts();
            d::ClientSubject::Panel { project, task }
        }
        w::ClientSubject::Result(value) => {
            let w::ClientSubjectResultParts { project, task } = value.into_parts();
            d::ClientSubject::Result { project, task }
        }
        w::ClientSubject::Effect(value) => {
            let w::ClientSubjectEffectParts { project, proposal } = value.into_parts();
            d::ClientSubject::Effect { project, proposal }
        }
        w::ClientSubject::Outbox(value) => {
            let w::ClientSubjectOutboxParts { project, entry } = value.into_parts();
            d::ClientSubject::Outbox { project, entry }
        }
        w::ClientSubject::Hold(value) => {
            let w::ClientSubjectHoldParts { project, task } = value.into_parts();
            d::ClientSubject::Hold { project, task }
        }
        w::ClientSubject::Project(value) => {
            let w::ClientSubjectProjectParts { project } = value.into_parts();
            d::ClientSubject::Project { project }
        }
        w::ClientSubject::Deployment => d::ClientSubject::Deployment,
    }
}
pub(super) fn encode_form(value: d::Form) -> w::Form {
    match &value {
        d::Form::Creation => w::Form::Creation,
        d::Form::Transition => w::Form::Transition,
        d::Form::Set => w::Form::Set,
    }
}
pub(super) fn decode_form(value: w::Form, _limits: &d::Limits) -> d::Form {
    match value {
        w::Form::Creation => d::Form::Creation,
        w::Form::Transition => d::Form::Transition,
        w::Form::Set => d::Form::Set,
    }
}
pub(super) fn encode_recovery(value: d::Recovery) -> w::Recovery {
    match &value {
        d::Recovery::Keyed => w::Recovery::Keyed,
        d::Recovery::Conditional => w::Recovery::Conditional,
        d::Recovery::Idempotent => w::Recovery::Idempotent,
        d::Recovery::Unrecoverable => w::Recovery::Unrecoverable,
    }
}
pub(super) fn decode_recovery(value: w::Recovery, _limits: &d::Limits) -> d::Recovery {
    match value {
        w::Recovery::Keyed => d::Recovery::Keyed,
        w::Recovery::Conditional => d::Recovery::Conditional,
        w::Recovery::Idempotent => d::Recovery::Idempotent,
        w::Recovery::Unrecoverable => d::Recovery::Unrecoverable,
    }
}
pub(super) fn encode_hold_mode(value: d::HoldMode) -> w::HoldMode {
    match &value {
        d::HoldMode::Refuse => w::HoldMode::Refuse,
        d::HoldMode::Wait => w::HoldMode::Wait,
    }
}
pub(super) fn decode_hold_mode(value: w::HoldMode, _limits: &d::Limits) -> d::HoldMode {
    match value {
        w::HoldMode::Refuse => d::HoldMode::Refuse,
        w::HoldMode::Wait => d::HoldMode::Wait,
    }
}
pub(super) fn encode_hold(value: d::Hold) -> w::Hold {
    match &value {
        d::Hold::Exclusive { mode } => w::Hold::Exclusive(
            w::HoldExclusive::new(&w::CEILINGS, w::HoldExclusiveParts { mode: encode_hold_mode(*mode) })
                .expect("admitted fields fit schema"),
        ),
        d::Hold::Pooled { mode } => w::Hold::Pooled(
            w::HoldPooled::new(&w::CEILINGS, w::HoldPooledParts { mode: encode_hold_mode(*mode) })
                .expect("admitted fields fit schema"),
        ),
        d::Hold::None => w::Hold::None,
    }
}
pub(super) fn decode_hold(value: w::Hold, limits: &d::Limits) -> d::Hold {
    match value {
        w::Hold::Exclusive(value) => {
            let w::HoldExclusiveParts { mode } = value.into_parts();
            d::Hold::Exclusive { mode: decode_hold_mode(mode, limits) }
        }
        w::Hold::Pooled(value) => {
            let w::HoldPooledParts { mode } = value.into_parts();
            d::Hold::Pooled { mode: decode_hold_mode(mode, limits) }
        }
        w::Hold::None => d::Hold::None,
    }
}
pub(super) fn encode_diagnostic_availability(value: d::DiagnosticAvailability) -> w::DiagnosticAvailability {
    match &value {
        d::DiagnosticAvailability::NotKept => w::DiagnosticAvailability::NotKept,
    }
}
pub(super) fn decode_diagnostic_availability(
    value: w::DiagnosticAvailability,
    _limits: &d::Limits,
) -> d::DiagnosticAvailability {
    match value {
        w::DiagnosticAvailability::NotKept => d::DiagnosticAvailability::NotKept,
    }
}
pub(super) fn encode_effect_phase(value: d::EffectPhase) -> w::EffectPhase {
    match &value {
        d::EffectPhase::Kept => w::EffectPhase::Kept,
        d::EffectPhase::Sent => w::EffectPhase::Sent,
        d::EffectPhase::Uncertain => w::EffectPhase::Uncertain,
        d::EffectPhase::Retry => w::EffectPhase::Retry,
        d::EffectPhase::Repeat => w::EffectPhase::Repeat,
        d::EffectPhase::Held => w::EffectPhase::Held,
    }
}
pub(super) fn decode_effect_phase(value: w::EffectPhase, _limits: &d::Limits) -> d::EffectPhase {
    match value {
        w::EffectPhase::Kept => d::EffectPhase::Kept,
        w::EffectPhase::Sent => d::EffectPhase::Sent,
        w::EffectPhase::Uncertain => d::EffectPhase::Uncertain,
        w::EffectPhase::Retry => d::EffectPhase::Retry,
        w::EffectPhase::Repeat => d::EffectPhase::Repeat,
        w::EffectPhase::Held => d::EffectPhase::Held,
    }
}
pub(super) fn encode_resource_role(value: d::ResourceRole) -> w::ResourceRole {
    match &value {
        d::ResourceRole::Owned => w::ResourceRole::Owned,
        d::ResourceRole::Participating => w::ResourceRole::Participating,
        d::ResourceRole::Context => w::ResourceRole::Context,
        d::ResourceRole::Unavailable => w::ResourceRole::Unavailable,
    }
}
pub(super) fn decode_resource_role(value: w::ResourceRole, _limits: &d::Limits) -> d::ResourceRole {
    match value {
        w::ResourceRole::Owned => d::ResourceRole::Owned,
        w::ResourceRole::Participating => d::ResourceRole::Participating,
        w::ResourceRole::Context => d::ResourceRole::Context,
        w::ResourceRole::Unavailable => d::ResourceRole::Unavailable,
    }
}
pub(super) fn encode_effect_resources(values: &[d::Path]) -> List<w::ResourcePath> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("bounded list"));
    for value in values {
        result.push(encode_resource_path(value)).expect("same count");
    }
    result
}
pub(super) fn decode_effect_resources(
    values: List<w::ResourcePath>,
    limits: &d::Limits,
) -> Result<Box<[d::Path]>, Undecodable> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_resource_path(value, limits)?).expect("same count");
    }
    Ok(result.into_boxed())
}
pub(super) fn encode_description_resources(values: &[d::Path]) -> List<w::ResourcePath> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("bounded list"));
    for value in values {
        result.push(encode_resource_path(value)).expect("same count");
    }
    result
}
pub(super) fn decode_description_resources(
    values: List<w::ResourcePath>,
    limits: &d::Limits,
) -> Result<Box<[d::Path]>, Undecodable> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_resource_path(value, limits)?).expect("same count");
    }
    Ok(result.into_boxed())
}
pub(super) fn encode_clientdocumentpanel_resources(values: &List<d::ClientResource>) -> List<w::ClientResource> {
    let mut result = List::with_capacity(values.len());
    for value in values {
        result.push(encode_client_resource(value)).expect("same count");
    }
    result
}
pub(super) fn encode_clientdocumentpanel_effects(values: &List<d::OutboxEntry>) -> List<w::OutboxEntry> {
    let mut result = List::with_capacity(values.len());
    for value in values {
        result.push(encode_outbox_entry(value)).expect("same count");
    }
    result
}
pub(super) fn decode_clientdocumentpanel_resources(
    values: List<w::ClientResource>,
    limits: &d::Limits,
) -> Result<List<d::ClientResource>, Undecodable> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_client_resource(value, limits)?).expect("same count");
    }
    Ok(result)
}
pub(super) fn decode_clientdocumentpanel_effects(
    values: List<w::OutboxEntry>,
    limits: &d::Limits,
) -> Result<List<d::OutboxEntry>, Undecodable> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_outbox_entry(value, limits)?).expect("same count");
    }
    Ok(result)
}
pub(super) fn encode_clientdocumenthold_resources(values: &List<d::ClientResource>) -> List<w::ClientResource> {
    let mut result = List::with_capacity(values.len());
    for value in values {
        result.push(encode_client_resource(value)).expect("same count");
    }
    result
}
pub(super) fn encode_clientdocumenthold_effects(values: &List<d::OutboxEntry>) -> List<w::OutboxEntry> {
    let mut result = List::with_capacity(values.len());
    for value in values {
        result.push(encode_outbox_entry(value)).expect("same count");
    }
    result
}
pub(super) fn decode_clientdocumenthold_resources(
    values: List<w::ClientResource>,
    limits: &d::Limits,
) -> Result<List<d::ClientResource>, Undecodable> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_client_resource(value, limits)?).expect("same count");
    }
    Ok(result)
}
pub(super) fn decode_clientdocumenthold_effects(
    values: List<w::OutboxEntry>,
    limits: &d::Limits,
) -> Result<List<d::OutboxEntry>, Undecodable> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_outbox_entry(value, limits)?).expect("same count");
    }
    Ok(result)
}
pub(super) fn encode_clientdocumentproject_resources(values: &List<d::ClientResource>) -> List<w::ClientResource> {
    let mut result = List::with_capacity(values.len());
    for value in values {
        result.push(encode_client_resource(value)).expect("same count");
    }
    result
}
pub(super) fn decode_clientdocumentproject_resources(
    values: List<w::ClientResource>,
    limits: &d::Limits,
) -> Result<List<d::ClientResource>, Undecodable> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_client_resource(value, limits)?).expect("same count");
    }
    Ok(result)
}
pub(super) fn encode_clientdocumentdeployment_outbox(values: &List<d::OutboxEntry>) -> List<w::OutboxEntry> {
    let mut result = List::with_capacity(values.len());
    for value in values {
        result.push(encode_outbox_entry(value)).expect("same count");
    }
    result
}
pub(super) fn decode_clientdocumentdeployment_outbox(
    values: List<w::OutboxEntry>,
    limits: &d::Limits,
) -> Result<List<d::OutboxEntry>, Undecodable> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_outbox_entry(value, limits)?).expect("same count");
    }
    Ok(result)
}
pub(super) fn encode_resource_path(path: &d::Path) -> w::ResourcePath {
    let mut segments = List::with_capacity(u32::try_from(path.segments().len()).expect("bounded path"));
    for bytes in path.segments() {
        segments
            .push(w::Segment::new(&w::CEILINGS, w::SegmentParts { bytes: bytes.clone() }).expect("bounded segment"))
            .expect("same count");
    }
    w::ResourcePath::new(&w::CEILINGS, w::ResourcePathParts { segments }).expect("bounded path")
}
pub(super) fn decode_resource_path(path: w::ResourcePath, limits: &d::Limits) -> Result<d::Path, Undecodable> {
    let mut segments = List::with_capacity(path.segments().len());
    for segment in path.into_parts().segments.into_boxed() {
        segments.push(segment.into_parts().bytes).expect("same count");
    }
    d::Path::new(segments, limits.segment_bytes).ok_or(Undecodable)
}
