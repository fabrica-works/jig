//! Ops own vocabulary translation; jig vocabulary reuses session 45 unchanged.
use jig_ops_client_domain::infrastructure::native;
use jig_ops_domain_infrastructure as source;

#[must_use]
pub fn translate_service(value: source::Service) -> native::Service {
    native::Service { environment: value.environment, name: value.name }
}

#[must_use]
pub fn translate_environment(value: source::Environment) -> native::Environment {
    native::Environment { pool: value.pool, name: value.name }
}

#[must_use]
pub fn translate_pool(value: source::Pool) -> native::Pool {
    native::Pool(value.0)
}

#[must_use]
pub fn translate_resource(value: source::Resource) -> native::Resource {
    match value {
        source::Resource::Service(value) => native::Resource::Service(translate_service(value)),
        source::Resource::Environment(value) => native::Resource::Environment(translate_environment(value)),
        source::Resource::Pool(value) => native::Resource::Pool(translate_pool(value)),
        source::Resource::Quota(value) => native::Resource::Quota(translate_pool(value)),
    }
}

#[must_use]
pub fn translate_hold(value: source::Hold) -> native::Hold {
    match value {
        source::Hold::ExclusiveWait => native::Hold::ExclusiveWait,
        source::Hold::PooledWait => native::Hold::PooledWait,
        source::Hold::None => native::Hold::None,
    }
}

#[must_use]
pub fn translate_resource_role(value: source::ResourceRole) -> native::ResourceRole {
    match value {
        source::ResourceRole::Owned => native::ResourceRole::Owned,
        source::ResourceRole::Participant => native::ResourceRole::Participant,
        source::ResourceRole::Context => native::ResourceRole::Context,
        source::ResourceRole::Unavailable => native::ResourceRole::Unavailable,
    }
}

#[must_use]
pub fn translate_named(value: source::Named) -> native::Named {
    native::Named {
        resource: translate_resource(value.resource),
        role: translate_resource_role(value.role),
        hold: translate_hold(value.hold),
    }
}

#[must_use]
pub fn translate_purpose(value: source::Purpose) -> native::Purpose {
    match value {
        source::Purpose::Call { attempt, completion, position } => {
            native::Purpose::Call { attempt, completion, position }
        }
        source::Purpose::Procedure { purpose } => native::Purpose::Procedure { purpose },
        source::Purpose::Projection { purpose } => native::Purpose::Projection { purpose },
    }
}

#[must_use]
pub fn translate_key(value: source::Key) -> native::Key {
    native::Key {
        deployment: value.deployment,
        task: value.task,
        origin: translate_purpose(value.origin),
        purpose: value.purpose,
    }
}

#[must_use]
pub fn translate_backend(value: source::Backend) -> native::Backend {
    match value {
        source::Backend::OperationIds => native::Backend::OperationIds,
        source::Backend::NoOperationIds => native::Backend::NoOperationIds,
    }
}

#[must_use]
pub fn translate_effect(value: source::Effect) -> native::Effect {
    match value {
        source::Effect::Restart { service, operation } => {
            native::Effect::Restart { service: translate_service(service), operation }
        }
        source::Effect::Scale { service, from, to } => {
            native::Effect::Scale { service: translate_service(service), from, to }
        }
        source::Effect::Rollback { service, from, to } => {
            native::Effect::Rollback { service: translate_service(service), from, to }
        }
        source::Effect::CreateEnvironment { environment, until, price } => {
            native::Effect::CreateEnvironment { environment: translate_environment(environment), until, price }
        }
        source::Effect::TearDown { environment, created_by } => native::Effect::TearDown {
            environment: translate_environment(environment),
            created_by: translate_key(created_by),
        },
    }
}

#[must_use]
pub fn translate_form(value: source::Form) -> native::Form {
    match value {
        source::Form::Creation => native::Form::Creation,
        source::Form::Transition => native::Form::Transition,
        source::Form::Set => native::Form::Set,
    }
}

#[must_use]
pub fn translate_recovery(value: source::Recovery) -> native::Recovery {
    match value {
        source::Recovery::Keyed => native::Recovery::Keyed,
        source::Recovery::Conditional => native::Recovery::Conditional,
        source::Recovery::Unrecoverable => native::Recovery::Unrecoverable,
    }
}

#[must_use]
pub fn translate_description(value: source::Description) -> native::Description {
    native::Description {
        kind: value.kind,
        resources: value.resources.into_vec().into_iter().map(translate_resource).collect::<Vec<_>>().into(),
        condition: value.condition,
        form: translate_form(value.form),
        recovery: translate_recovery(value.recovery),
        price: value.price,
        state: value.state,
    }
}

#[must_use]
pub fn translate_phase(value: source::Phase) -> native::Phase {
    match value {
        source::Phase::Kept => native::Phase::Kept,
        source::Phase::Sent => native::Phase::Sent,
        source::Phase::Uncertain => native::Phase::Uncertain,
        source::Phase::Held => native::Phase::Held,
    }
}

#[must_use]
pub fn translate_entry(value: source::Entry) -> native::Entry {
    native::Entry {
        project: value.project,
        number: value.number,
        key: translate_key(value.key),
        effect: translate_effect(value.effect),
        attempt: value.attempt,
        deadline: value.deadline,
        phase: translate_phase(value.phase),
    }
}

#[must_use]
pub fn translate_service_fact(value: source::ServiceFact) -> native::ServiceFact {
    native::ServiceFact {
        version: value.version,
        replicas: value.replicas,
        healthy: value.healthy,
        revision: value.revision,
        observed: value.observed,
    }
}

#[must_use]
pub fn translate_environment_fact(value: source::EnvironmentFact) -> native::EnvironmentFact {
    native::EnvironmentFact {
        created_by: translate_key(value.created_by),
        ready_at: value.ready_at,
        until: value.until,
        price: value.price,
    }
}

#[must_use]
pub fn translate_procedure(value: source::Procedure) -> native::Procedure {
    match value {
        source::Procedure::Remediate { service, operation, deadline } => {
            native::Procedure::Remediate { service: translate_service(service), operation, deadline }
        }
        source::Procedure::Scale { service, replicas, deadline } => {
            native::Procedure::Scale { service: translate_service(service), replicas, deadline }
        }
        source::Procedure::Provision { environment, until, price, deadline } => {
            native::Procedure::Provision { environment: translate_environment(environment), until, price, deadline }
        }
        source::Procedure::TearDown { environment, created_by, deadline } => native::Procedure::TearDown {
            environment: translate_environment(environment),
            created_by: translate_key(created_by),
            deadline,
        },
    }
}

#[must_use]
pub fn translate_procedure_phase(value: source::ProcedurePhase) -> native::ProcedurePhase {
    match value {
        source::ProcedurePhase::Active => native::ProcedurePhase::Active,
        source::ProcedurePhase::ReadingFacts => native::ProcedurePhase::ReadingFacts,
        source::ProcedurePhase::FactsReady => native::ProcedurePhase::FactsReady,
        source::ProcedurePhase::EffectOutstanding => native::ProcedurePhase::EffectOutstanding,
        source::ProcedurePhase::WaitingCondition => native::ProcedurePhase::WaitingCondition,
        source::ProcedurePhase::Holding => native::ProcedurePhase::Holding,
        source::ProcedurePhase::Done => native::ProcedurePhase::Done,
        source::ProcedurePhase::Held => native::ProcedurePhase::Held,
    }
}

#[must_use]
pub fn translate_procedure_state(value: source::ProcedureState) -> native::ProcedureState {
    native::ProcedureState {
        task: value.task,
        procedure: translate_procedure(value.procedure),
        phase: translate_procedure_phase(value.phase),
    }
}

#[must_use]
pub fn translate_drift_kind(value: source::DriftKind) -> native::DriftKind {
    match value {
        source::DriftKind::Changed => native::DriftKind::Changed,
        source::DriftKind::Deleted => native::DriftKind::Deleted,
    }
}

#[must_use]
pub fn translate_drift_account(value: source::DriftAccount) -> native::DriftAccount {
    native::DriftAccount {
        resource: translate_resource(value.resource),
        kind: translate_drift_kind(value.kind),
        at: value.at,
        by: value.by,
    }
}

#[must_use]
pub fn translate_environment_end(value: source::EnvironmentEnd) -> native::EnvironmentEnd {
    native::EnvironmentEnd { revision: value.revision, until: value.until }
}

#[must_use]
pub fn translate_client_pool(value: source::ClientPool) -> native::ClientPool {
    native::ClientPool { quota: value.quota, used: value.used }
}

#[must_use]
pub fn translate_resource_fact(value: source::ResourceFact) -> native::ResourceFact {
    match value {
        source::ResourceFact::Service(value) => native::ResourceFact::Service(value.map(translate_service_fact)),
        source::ResourceFact::Environment { fact, end } => native::ResourceFact::Environment {
            fact: fact.map(translate_environment_fact),
            end: end.map(translate_environment_end),
        },
        source::ResourceFact::Pool(value) => native::ResourceFact::Pool(value.map(translate_client_pool)),
        source::ResourceFact::Quota(value) => native::ResourceFact::Quota(value.map(translate_client_pool)),
    }
}

#[must_use]
pub fn translate_client_resource(value: source::ClientResource) -> native::ClientResource {
    native::ClientResource { named: translate_named(value.named), fact: translate_resource_fact(value.fact) }
}

#[must_use]
pub fn translate_client_environment(value: source::ClientEnvironment) -> native::ClientEnvironment {
    native::ClientEnvironment {
        environment: translate_environment(value.environment),
        fact: translate_environment_fact(value.fact),
        end: value.end.map(translate_environment_end),
        pool: value.pool.map(translate_client_pool),
    }
}

fn translate_resources(source: skein_lib::List<source::ClientResource>) -> skein_lib::List<native::ClientResource> {
    let mut values = skein_lib::List::with_capacity(source.capacity());
    for value in source.into_boxed() {
        values.push(translate_client_resource(value)).expect("source list capacity");
    }
    values
}
#[must_use]
pub fn translate_value(value: source::ClientDocument) -> native::Value {
    match value {
        source::ClientDocument::Environment { environment, fact, end, pool, end_revision } => {
            native::Value::Environment {
                end_revision,
                environment: translate_environment(environment),
                fact: translate_environment_fact(fact),
                end: end.map(translate_environment_end),
                pool: pool.map(translate_client_pool),
            }
        }
        source::ClientDocument::Panel { task, state, resources, effects } => native::Value::Panel {
            task,
            state: translate_procedure_state(state),
            resources: translate_resources(resources),
            effects: {
                let source = effects;
                let mut values = skein_lib::List::with_capacity(source.capacity());
                for value in &source {
                    values.push(translate_entry(value.clone())).expect("source list capacity");
                }
                values
            },
        },
        source::ClientDocument::Result { task, state, resources } => native::Value::Result {
            task,
            state: translate_procedure_state(state),
            resources: translate_resources(resources),
        },
        source::ClientDocument::Effect { proposal, task, effect, description } => native::Value::Effect {
            proposal,
            task,
            effect: translate_effect(effect),
            description: translate_description(description),
        },
        source::ClientDocument::Outbox { entry, description } => {
            native::Value::Outbox { entry: translate_entry(entry), description: translate_description(description) }
        }
        source::ClientDocument::Hold { task, resources, drift, effects } => native::Value::Hold {
            task,
            resources: translate_resources(resources),
            drift: {
                let source = drift;
                let mut values = skein_lib::List::with_capacity(source.capacity());
                for value in &source {
                    values.push(translate_drift_account(value.clone())).expect("source list capacity");
                }
                values
            },
            effects: {
                let source = effects;
                let mut values = skein_lib::List::with_capacity(source.capacity());
                for value in &source {
                    values.push(translate_entry(value.clone())).expect("source list capacity");
                }
                values
            },
        },
        source::ClientDocument::Project { project, resources, environments } => native::Value::Project {
            project,
            resources: translate_resources(resources),
            environments: {
                let source = environments;
                let mut values = skein_lib::List::with_capacity(source.capacity());
                for value in &source {
                    values.push(translate_client_environment(value.clone())).expect("source list capacity");
                }
                values
            },
        },
        source::ClientDocument::Deployment { backend, effects, made, proposal_limit, staged_limit } => {
            native::Value::Deployment {
                backend: translate_backend(backend),
                effects: {
                    let source = effects;
                    let mut values = skein_lib::List::with_capacity(source.capacity());
                    for value in &source {
                        values.push(translate_entry(value.clone())).expect("source list capacity");
                    }
                    values
                },
                made,
                proposal_limit,
                staged_limit,
            }
        }
    }
}

/// The Ops protocol encodes its complete scoped request before hashing it.
/// # Errors
/// Numeric fixture identities cannot represent the native owner identity.
pub fn ask(
    value: jig_ops_client_domain::infrastructure::Ask,
) -> Result<(u32, source::ApplicationRequest, [u8; 32]), &'static str> {
    use jig_ops_client_domain::infrastructure::Ask;
    use sha2::{Digest, Sha256};
    let (project, value) = match value {
        Ask::NativeMoveEnd { project, environment, revision, until, object: _ } => (
            project,
            source::ApplicationRequest {
                environment: source::Environment { pool: environment.pool, name: environment.name },
                revision: revision.value,
                until,
            },
        ),
        Ask::MoveEnd { .. } => return Err("legacy numeric environment identity is absent from the native owner"),
    };
    let mut digest = Sha256::new();
    digest.update(b"ops/infrastructure/move-end/1");
    digest.update(2_u16.to_be_bytes());
    digest.update(project.to_be_bytes());
    digest.update(u64::try_from(value.environment.pool.len()).expect("bounded pool name").to_be_bytes());
    digest.update(&value.environment.pool);
    digest.update(u64::try_from(value.environment.name.len()).expect("bounded environment name").to_be_bytes());
    digest.update(&value.environment.name);
    digest.update(value.revision.to_be_bytes());
    digest.update(value.until.to_be_bytes());
    Ok((project, value, digest.finalize().into()))
}
#[must_use]
pub fn answer(value: source::ApplicationOutcome) -> jig_ops_client_domain::infrastructure::Answer {
    use jig_client::contract::Revision;
    use jig_ops_client_domain::infrastructure::{Answer, documents::Refusal};
    match value {
        source::ApplicationOutcome::MovedEnd { environment, now } => Answer::NativeMovedEnd {
            environment: translate_environment(environment),
            revision: Revision { value: now.revision },
            until: now.until,
        },
        source::ApplicationOutcome::Moved { environment, now } => Answer::NativeMoved {
            environment: translate_environment(environment),
            revision: Revision { value: now.revision },
            until: now.until,
        },
        source::ApplicationOutcome::Role { held } => Answer::Refused { why: Refusal::Role { held } },
        source::ApplicationOutcome::Unknown => Answer::Gone,
        source::ApplicationOutcome::Past => Answer::Refused { why: Refusal::Past },
        source::ApplicationOutcome::Exhausted => Answer::Refused { why: Refusal::Exhausted },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_client::contract::Revision;
    use jig_ops_client_domain::infrastructure::{Answer, Ask, documents::Refusal};
    use skein_lib::Token;
    #[test]
    fn native_move_end_wire_keeps_scope_raw_names_whole_width_values_and_distinct_terminals() {
        let environment = native::Environment { pool: b"Q.pool".as_slice().into(), name: b"preview".as_slice().into() };
        let request = Ask::NativeMoveEnd {
            object: Token::new(17),
            project: u32::MAX,
            environment: environment.clone(),
            revision: Revision { value: u64::MAX },
            until: u64::MAX,
        };
        let (project, value, digest) = ask(request.clone()).expect("native ask");
        assert_eq!(project, u32::MAX);
        assert_eq!(
            value,
            source::ApplicationRequest {
                environment: source::Environment { pool: environment.pool.clone(), name: environment.name.clone() },
                revision: u64::MAX,
                until: u64::MAX
            }
        );
        let (_, _, replay_digest) = ask(Ask::NativeMoveEnd {
            object: Token::new(29),
            project,
            environment: environment.clone(),
            revision: Revision { value: u64::MAX },
            until: u64::MAX,
        })
        .expect("replayed semantic request");
        assert_eq!(digest, replay_digest, "view handles are not owner request facts");
        for changed in [
            Ask::NativeMoveEnd {
                object: Token::new(17),
                project: project - 1,
                environment: environment.clone(),
                revision: Revision { value: u64::MAX },
                until: u64::MAX,
            },
            Ask::NativeMoveEnd {
                object: Token::new(17),
                project,
                environment: native::Environment { pool: b"P.pool".as_slice().into(), name: environment.name.clone() },
                revision: Revision { value: u64::MAX },
                until: u64::MAX,
            },
            Ask::NativeMoveEnd {
                object: Token::new(17),
                project,
                environment: environment.clone(),
                revision: Revision { value: 0 },
                until: u64::MAX,
            },
            Ask::NativeMoveEnd {
                object: Token::new(17),
                project,
                environment: environment.clone(),
                revision: Revision { value: u64::MAX },
                until: 0,
            },
        ] {
            assert_ne!(digest, ask(changed).expect("changed native ask").2);
        }
        let now = source::EnvironmentEnd { revision: u64::MAX, until: u64::MAX };
        assert_eq!(
            answer(source::ApplicationOutcome::MovedEnd { environment: value.environment.clone(), now }),
            Answer::NativeMovedEnd {
                environment: environment.clone(),
                revision: Revision { value: u64::MAX },
                until: u64::MAX
            }
        );
        assert_eq!(
            answer(source::ApplicationOutcome::Moved { environment: value.environment, now }),
            Answer::NativeMoved { environment, revision: Revision { value: u64::MAX }, until: u64::MAX }
        );
        assert_eq!(
            answer(source::ApplicationOutcome::Role { held: u32::MAX }),
            Answer::Refused { why: Refusal::Role { held: u32::MAX } }
        );
        assert_eq!(answer(source::ApplicationOutcome::Unknown), Answer::Gone);
        assert_eq!(answer(source::ApplicationOutcome::Past), Answer::Refused { why: Refusal::Past });
        assert_eq!(answer(source::ApplicationOutcome::Exhausted), Answer::Refused { why: Refusal::Exhausted });
        assert!(
            ask(Ask::MoveEnd {
                project,
                environment: 0,
                revision: Revision { value: 0 },
                until: skein_lib::Wall::EPOCH
            })
            .is_err(),
            "a numeric fixture identity never becomes an opaque owner identity"
        );
    }
}
