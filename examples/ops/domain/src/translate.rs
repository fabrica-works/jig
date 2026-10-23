//! Application vocabulary in small total functions. Copy the core/host pairs;
//! replace resource paths, effect kinds and tool declarations (domain/root.md, 6).
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_ops_domain_infrastructure as infrastructure;
use jig_ops_domain_observability as observability;
use skein_lib::{List, Wall};

pub(crate) fn infrastructure_key(key: core::EffectKey) -> infrastructure::Key {
    let origin = match key.origin {
        core::EffectPurpose::Call { attempt, completion, position } => {
            infrastructure::Purpose::Call { attempt, completion, position }
        }
        core::EffectPurpose::Procedure { purpose } => infrastructure::Purpose::Procedure { purpose },
        core::EffectPurpose::Projection { purpose } => infrastructure::Purpose::Projection { purpose },
    };
    infrastructure::Key { deployment: key.deployment, task: key.task, origin, purpose: key.purpose }
}
pub(crate) fn observability_key(key: core::EffectKey) -> observability::Key {
    let origin = match key.origin {
        core::EffectPurpose::Call { attempt, completion, position } => {
            observability::Purpose::Call { attempt, completion, position }
        }
        core::EffectPurpose::Procedure { purpose } => observability::Purpose::Procedure { purpose },
        core::EffectPurpose::Projection { purpose } => observability::Purpose::Projection { purpose },
    };
    observability::Key { deployment: key.deployment, task: key.task, origin, purpose: key.purpose }
}
pub(crate) fn state(value: u64) -> [u8; 32] {
    let mut state = [0; 32];
    for (slot, byte) in state.get_mut(24..).expect("eight-byte state").iter_mut().zip(value.to_be_bytes()) {
        *slot = byte;
    }
    state
}
pub(crate) fn infrastructure_description(
    number: u16,
    value: infrastructure::Description,
    waits_for_dependencies: bool,
) -> core::connector::EffectDescription {
    let mut names = List::with_capacity(u32::try_from(value.resources.len()).expect("bounded resources"));
    for resource in value.resources {
        names.push(authority::Name { segments: resource.segments() }).expect("one resource name");
    }
    let mut additional = List::with_capacity(names.len().saturating_sub(1));
    for name in names.as_slice().get(1..).unwrap_or_default() {
        additional
            .push(authority::EffectResource { name: name.clone(), access: authority::EffectAccess::Owned })
            .expect("additional names");
    }
    let form = match value.form {
        infrastructure::Form::Creation => core::connector::EffectForm::Creation,
        infrastructure::Form::Transition => core::connector::EffectForm::Transition,
        infrastructure::Form::Set => core::connector::EffectForm::Set,
    };
    let recovery = match value.recovery {
        infrastructure::Recovery::Keyed => core::connector::Recovery::Keyed,
        infrastructure::Recovery::Conditional => core::connector::Recovery::Conditional,

        infrastructure::Recovery::Unrecoverable => core::connector::Recovery::Unrecoverable,
    };
    core::connector::EffectDescription {
        waits_for_dependencies,
        connector: number,
        purpose: value.state,
        form,
        recovery,
        effect: authority::Effect {
            connector: number,
            kind: value.kind,
            name: names.as_slice().first().expect("described resource").clone(),
            state: state(value.state),
            price: value.price,
            access: authority::EffectAccess::Owned,
            additional: additional.into_boxed(),
            guards: Box::new([]),
        },
    }
}
pub(crate) fn observability_description(
    number: u16,
    value: observability::Effect,
    waits_for_dependencies: bool,
) -> core::connector::EffectDescription {
    core::connector::EffectDescription {
        waits_for_dependencies,
        connector: number,
        purpose: value.until,
        form: core::connector::EffectForm::Set,
        recovery: core::connector::Recovery::Keyed,
        effect: authority::Effect {
            connector: number,
            kind: 1,
            name: authority::Name { segments: observability::Resource::AlertRule(value.rule).segments() },
            state: state(value.until),
            price: None,
            access: authority::EffectAccess::Owned,
            additional: Box::new([]),
            guards: Box::new([]),
        },
    }
}
pub(crate) const fn infrastructure_outcome(value: infrastructure::Outcome) -> core::connector::OutboxOutcome {
    match value {
        infrastructure::Outcome::Made => core::connector::OutboxOutcome::Made,
        infrastructure::Outcome::Failed => core::connector::OutboxOutcome::Failed,
        infrastructure::Outcome::Skipped => core::connector::OutboxOutcome::NotMade,
        infrastructure::Outcome::Uncertain => core::connector::OutboxOutcome::Uncertain,
    }
}
pub(crate) const fn observability_outcome(value: observability::Outcome) -> core::connector::OutboxOutcome {
    match value {
        observability::Outcome::Made => core::connector::OutboxOutcome::Made,
        observability::Outcome::Failed => core::connector::OutboxOutcome::Failed,
        observability::Outcome::Uncertain => core::connector::OutboxOutcome::Uncertain,
    }
}
pub(crate) const fn verdict(value: observability::Verdict, wall: Wall) -> (authority::Verdict, Wall) {
    match value {
        observability::Verdict::Met { observed } => {
            (authority::Verdict::Met, Wall::from_nanos(observed.saturating_mul(1_000_000_000)))
        }
        observability::Verdict::Wait => (authority::Verdict::Wait, wall),
        observability::Verdict::Refuse => (authority::Verdict::Refuse, wall),
    }
}
pub(crate) fn service(names: &[authority::Name]) -> Option<observability::Service> {
    let segments = &names.first()?.segments;
    if segments.len() == 4 && segments.first()?.as_ref() == b"env" && segments.get(2)?.as_ref() == b"service" {
        return Some(observability::Service::new(segments.get(1)?.clone(), segments.get(3)?.clone()));
    }
    if segments.len() == 3 && segments.first()?.as_ref() == b"services" {
        Some(observability::Service::new(segments.get(1)?.clone(), segments.get(2)?.clone()))
    } else {
        None
    }
}

/// Ops names its two requirements in its connector vocabulary.
pub(crate) const fn requirement(number: u16) -> Option<observability::Requirement> {
    match number {
        1 => Some(observability::Requirement::HealthyReplicaElsewhere),
        2 => Some(observability::Requirement::LoadBelow { percent: 20, for_seconds: 1800 }),
        _ => None,
    }
}

/// Ops's procedure parameters are typed spec carriers, never root policy.
pub(crate) fn spec_service(spec: &tasks::Spec) -> Option<infrastructure::Service> {
    let mut environment = None;
    let mut name = None;
    for parameter in &spec.parameters {
        match parameter {
            tasks::Parameter::Bytes { name: 1, value } => environment = Some(value.clone()),
            tasks::Parameter::Bytes { name: 2, value } => name = Some(value.clone()),
            tasks::Parameter::Bytes { .. } | tasks::Parameter::Number { .. } | tasks::Parameter::Resource { .. } => {}
        }
    }
    Some(infrastructure::Service::new(environment?, name?))
}
pub(crate) fn spec_number(spec: &tasks::Spec, number: u32) -> Option<u64> {
    for parameter in &spec.parameters {
        match parameter {
            tasks::Parameter::Number { name, value } => {
                if *name == number {
                    return Some(*value);
                }
            }
            tasks::Parameter::Bytes { .. } | tasks::Parameter::Resource { .. } => {}
        }
    }
    None
}
fn spec_environment(spec: &tasks::Spec) -> Option<infrastructure::Environment> {
    let mut pool = None;
    let mut name = None;
    for parameter in &spec.parameters {
        match parameter {
            tasks::Parameter::Bytes { name: 1, value } => pool = Some(value.clone()),
            tasks::Parameter::Bytes { name: 2, value } => name = Some(value.clone()),
            tasks::Parameter::Bytes { .. } | tasks::Parameter::Number { .. } | tasks::Parameter::Resource { .. } => {}
        }
    }
    Some(infrastructure::Environment::new(pool?, name?))
}
pub(crate) fn procedure(code: u32, spec: &tasks::Spec, deployment: [u8; 16]) -> Option<infrastructure::Procedure> {
    if code == infrastructure::PROVISION_CODE {
        return Some(infrastructure::Procedure::Provision {
            environment: spec_environment(spec)?,
            until: spec_number(spec, 3)?,
            price: spec_number(spec, 4)?,
            deadline: spec_number(spec, 5)?,
        });
    }
    if code == infrastructure::TEAR_DOWN_CODE {
        return Some(infrastructure::Procedure::TearDown {
            environment: spec_environment(spec)?,
            created_by: infrastructure::Key::procedure(deployment, spec_number(spec, 3)?, spec_number(spec, 4)?),
            deadline: spec_number(spec, 5)?,
        });
    }
    let service = spec_service(spec)?;
    match code {
        1 => Some(infrastructure::Procedure::Scale {
            service,
            replicas: u32::try_from(spec_number(spec, 3)?).ok()?,
            deadline: spec_number(spec, 4)?,
        }),
        2 => Some(infrastructure::Procedure::Remediate {
            service,
            operation: spec_number(spec, 3)?,
            deadline: spec_number(spec, 4)?,
        }),
        _ => None,
    }
}
pub(crate) fn holdings(number: u16, executor: tasks::Executor, spec: &tasks::Spec) -> Option<Box<[tasks::Holding]>> {
    match jig_ops_domain_connectors::connector(number) {
        Some(jig_ops_domain_connectors::Connector::Infrastructure) => {}
        Some(jig_ops_domain_connectors::Connector::Observability) | None => return Some(Box::new([])),
    }
    match executor {
        tasks::Executor::Procedure { connector, code } => match jig_ops_domain_connectors::connector(connector) {
            Some(jig_ops_domain_connectors::Connector::Infrastructure) => {
                if code == infrastructure::TEAR_DOWN_CODE {
                    return Some(Box::new([tasks::Holding::Write {
                        resource: tasks::Name {
                            connector: number,
                            path: infrastructure::Resource::Environment(spec_environment(spec)?).segments(),
                        },
                        kind: 2,
                    }]));
                }
                if code == infrastructure::PROVISION_CODE {
                    let environment = spec_environment(spec)?;
                    return Some(Box::from([
                        tasks::Holding::Write {
                            resource: tasks::Name {
                                connector: number,
                                path: infrastructure::Resource::Environment(environment.clone()).segments(),
                            },
                            kind: 2,
                        },
                        tasks::Holding::Slot {
                            pool: tasks::Name {
                                connector: number,
                                path: infrastructure::Resource::Pool(infrastructure::Pool(environment.pool)).segments(),
                            },
                            kind: 3,
                        },
                    ]));
                }
            }
            Some(jig_ops_domain_connectors::Connector::Observability) | None => return Some(Box::new([])),
        },
        tasks::Executor::Agent { .. } | tasks::Executor::Person(_) => return Some(Box::new([])),
    }
    match spec_service(spec) {
        Some(service) => Some(Box::new([tasks::Holding::Write {
            resource: tasks::Name { connector: number, path: infrastructure::Resource::Service(service).segments() },
            kind: 1,
        }])),
        None => {
            if spec.parameters.is_empty() {
                Some(Box::new([]))
            } else {
                None
            }
        }
    }
}
pub(crate) fn decimal(bytes: &[u8]) -> Option<u64> {
    let mut value = 0_u64;
    if bytes.is_empty() {
        return None;
    }
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}

pub(crate) const fn news_class(value: observability::Class) -> tasks::NewsClass {
    match value {
        observability::Class::Wake => tasks::NewsClass::Wakes,
        observability::Class::Keep => tasks::NewsClass::Kept,
    }
}
pub(crate) fn alert_words(alert: &observability::Alert) -> Box<[u8]> {
    let bytes = b"alert "
        .len()
        .checked_add(skein_lib::Decimal::of(alert.number).as_bytes().len())
        .expect("alert number")
        .checked_add(b" severity ".len())
        .expect("severity heading")
        .checked_add(skein_lib::Decimal::of(u64::from(alert.severity)).as_bytes().len())
        .expect("severity");
    let mut writer = skein_lib::Writer::new(bytes);
    writer.put(b"alert ").expect("alert heading");
    writer.put(skein_lib::Decimal::of(alert.number).as_bytes()).expect("alert identity");
    writer.put(b" severity ").expect("severity heading");
    writer.put(skein_lib::Decimal::of(u64::from(alert.severity)).as_bytes()).expect("severity");
    writer.finish()
}
pub(crate) fn health_words(_service: &observability::Service, healthy: bool) -> Box<[u8]> {
    if healthy { Box::from(&b"healthy"[..]) } else { Box::from(&b"unhealthy"[..]) }
}
pub(crate) fn triage(delegate: &[u8], alerts: &[u64]) -> Option<Box<[core::Delegate]>> {
    let mut batch = jig_protocol_tools::batch(delegate, &template_limits()).ok()?;
    if batch.len() != 1 {
        return None;
    }
    for member in &mut batch {
        let mut room = member.spec.words.len().checked_add(b"\nAlerts:".len())?;
        for alert in alerts {
            room = room.checked_add(1)?.checked_add(skein_lib::Decimal::of(*alert).as_bytes().len())?;
        }
        let mut words = skein_lib::Writer::new(room);
        words.put(&member.spec.words).ok()?;
        words.put(b"\nAlerts:").ok()?;
        for alert in alerts {
            words.put(b" ").ok()?;
            words.put(skein_lib::Decimal::of(*alert).as_bytes()).ok()?;
        }
        member.spec.words = words.finish();
    }
    Some(batch)
}

/// Read kinds and resources belong to the application's connector vocabulary.
pub(crate) fn read_description(number: u16, read: &observability::Read) -> authority::Effect {
    let (kind, resource) = match read {
        observability::Read::Logs { service, .. } => (2, observability::Resource::Logs(service.clone())),
        observability::Read::Series { service, .. } => (3, observability::Resource::Metrics(service.clone())),
        observability::Read::Fired { service, .. } => (4, observability::Resource::Service(service.clone())),
    };
    authority::Effect {
        connector: number,
        kind,
        name: authority::Name { segments: resource.segments() },
        state: [0; 32],
        price: None,
        access: authority::EffectAccess::Context,
        additional: Box::new([]),
        guards: Box::new([]),
    }
}

/// The connector owns adoption roles; the root carries them into core admission.
pub(crate) const fn resource_role(role: infrastructure::ResourceRole) -> core::connector::ResourceRole {
    match role {
        infrastructure::ResourceRole::Owned => core::connector::ResourceRole::Owned,
        infrastructure::ResourceRole::Participant => core::connector::ResourceRole::Participant,
        infrastructure::ResourceRole::Context => core::connector::ResourceRole::Context,
        infrastructure::ResourceRole::Unavailable => core::connector::ResourceRole::Unavailable,
    }
}

/// Connector-local service/environment write and pool-slot kinds, also used at cold restore.
pub(crate) fn hold_kinds(number: u16) -> Box<[tasks::Kind]> {
    Box::new([
        tasks::Kind { connector: number, kind: 1, hold: tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits } },
        tasks::Kind { connector: number, kind: 2, hold: tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits } },
        tasks::Kind { connector: number, kind: 3, hold: tasks::HoldKind::Pooled { taken: tasks::Taken::Waits } },
    ])
}

/// The owner templates always request one delegate, matching the outcome's one child.
pub(crate) fn lead(delegate: &[u8]) -> Option<Box<[core::Delegate]>> {
    let batch = jig_protocol_tools::batch(delegate, &template_limits()).ok()?;
    if batch.len() != 1 {
        return None;
    }
    Some(batch)
}

/// The closed result shape is handed to its owner, without its explanation.
pub(crate) fn watch_result(
    child: u64,
    result: &tasks::ResultKind,
    limits: &observability::Limits,
) -> observability::WatchMessage {
    let verdict = match result {
        tasks::ResultKind::Verdict { code: 1, fields } if fields.is_empty() => {
            Some(observability::TriageVerdict::Noise)
        }
        tasks::ResultKind::Verdict { code: 2, fields } => triage_fields(fields, limits),
        tasks::ResultKind::Verdict { .. } => None,
        tasks::ResultKind::Approved { .. }
        | tasks::ResultKind::Report
        | tasks::ResultKind::Change { .. }
        | tasks::ResultKind::Failed
        | tasks::ResultKind::Cancelled => return observability::WatchMessage::Ended { child },
    };
    match verdict {
        Some(verdict) => observability::WatchMessage::Triage { child, verdict },
        None => observability::WatchMessage::InvalidTriage { child },
    }
}

// Ops's declared machine-field vocabulary. These are explicit result values,
// never task numbers derived from report text or from the alerts they cover.
fn triage_fields(
    fields: &[tasks::ResultField],
    limits: &observability::Limits,
) -> Option<observability::TriageVerdict> {
    if fields.len() != 4 {
        return None;
    }
    let mut alerts = None;
    let mut environment = None;
    let mut service = None;
    let mut severity = None;
    for field in fields {
        match field.name.as_ref() {
            b"alerts" if alerts.is_none() => {
                let mut numbers = List::with_capacity(limits.alerts_per_batch);
                let mut start = 0;
                for (end, byte) in field.value.iter().enumerate() {
                    if *byte == b' ' {
                        numbers.push(decimal(field.value.get(start..end)?)?).ok()?;
                        start = end.checked_add(1)?;
                    }
                }
                numbers.push(decimal(field.value.get(start..)?)?).ok()?;
                alerts = Some(numbers.into_boxed());
            }
            b"environment" if environment.is_none() => environment = Some(field.value.clone()),
            b"service" if service.is_none() => service = Some(field.value.clone()),
            b"severity" if severity.is_none() => severity = Some(u8::try_from(decimal(&field.value)?).ok()?),
            _ => return None,
        }
    }
    let environment = environment?;
    let service = service?;
    if environment.is_empty()
        || service.is_empty()
        || u32::try_from(environment.len()).ok()? > limits.name_bytes
        || u32::try_from(service.len()).ok()? > limits.name_bytes
    {
        return None;
    }
    Some(observability::TriageVerdict::Incident {
        alerts: alerts?,
        service: observability::Service::new(environment, service),
        severity: severity?,
    })
}

const fn template_limits() -> jig_protocol_tools::Limits {
    jig_protocol_tools::Limits { input_bytes: 65536, words: 65536, batch: 2, depth: 16 }
}

pub(crate) fn infrastructure_added(values: Box<[tasks::Added]>) -> Box<[infrastructure::Added]> {
    let mut added = List::with_capacity(u32::try_from(values.len()).expect("bounded dependencies"));
    for value in values {
        let by = match value.by {
            tasks::Party::Person(person) => infrastructure::Party::Person(person),
            tasks::Party::Task(task) => infrastructure::Party::Task(task),
            tasks::Party::Deployment { project } => infrastructure::Party::Deployment { project },
        };
        added.push(infrastructure::Added { task: value.task, at: value.at, by }).expect("one row per dependency");
    }
    added.into_boxed()
}
