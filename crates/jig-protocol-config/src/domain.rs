//! Deployment, initial identities and connector numbers (configuration.md, 3).
use crate::{Path, Problems, What, check, read};
use alloc::boxed::Box;
use jig_domain_people::{IdentityKey, InitialOwner};
use skein_json::Document;
use skein_lib::List;

#[derive(Debug)]
pub struct Deployment {
    pub directory: Box<[u8]>,
    pub seed: Option<u64>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProviderKind {
    Link,
    Application,
    Deployment,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Provider {
    pub number: u16,
    pub kind: ProviderKind,
}

pub(crate) fn deployment(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Deployment> {
    let mut object = check::object(document, at, path, problems)?;
    let directory = check::text(&mut object, document, b"directory", path, problems);
    let mut seed = None;
    if let Some(at) = object.member(document, b"seed") {
        match read::integer(document, at, u64::MAX) {
            Ok(value) => seed = Some(value),
            Err(what) => problems.push(path.member(b"seed"), what),
        }
    }
    object.finish(document, path, problems);
    Some(Deployment { directory: directory?, seed })
}
pub(crate) fn providers(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Box<[Provider]>> {
    let items = items(document, at, document.len(), path, problems)?;
    let mut providers = List::<Provider>::with_capacity(items.len());
    let mut deployments = 0_u32;
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        let Some(mut object) = check::object(document, *at, &path, problems) else { continue };
        let number = check::short(&mut object, document, b"number", &path, problems);
        let kind = check::text(&mut object, document, b"kind", &path, problems);
        object.finish(document, &path, problems);
        if let Some(number) = number
            && let Some(kind) = kind
        {
            let kind = match kind.as_ref() {
                b"link" => ProviderKind::Link,
                b"application" => ProviderKind::Application,
                b"deployment" => {
                    deployments = deployments.saturating_add(1);
                    if deployments > 1 {
                        problems.push(path.member(b"kind"), What::Unique);
                    }
                    ProviderKind::Deployment
                }
                _ => {
                    problems.push(path.member(b"kind"), What::Form);
                    continue;
                }
            };
            for previous in &providers {
                if previous.number == number {
                    problems.push(path.member(b"number"), What::Unique);
                    break;
                }
            }
            providers.push(Provider { number, kind }).expect("bounded provider input");
        }
    }
    if deployments == 0 {
        problems.push(path.clone(), What::Missing);
    }
    Some(providers.into_boxed())
}
pub(crate) fn owners(
    document: &Document,
    at: u32,
    path: &Path,
    engine: &crate::Engine,
    problems: &mut Problems,
) -> Box<[InitialOwner]> {
    let mut owners = List::<InitialOwner>::with_capacity(engine.limits.core.people.initial_owners);
    let Some(items) = items(document, at, engine.limits.core.people.initial_owners, path, problems) else {
        return owners.into_boxed();
    };
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        let Some(mut object) = check::object(document, *at, &path, problems) else { continue };
        let provider = check::short(&mut object, document, b"provider", &path, problems);
        let subject = check::text(&mut object, document, b"subject", &path, problems);
        object.finish(document, &path, problems);
        if let Some(provider) = provider
            && let Some(subject) = subject
        {
            let mut known = false;
            for configured in &engine.providers {
                known |= configured.number == provider;
            }
            if !known {
                problems.push(path.member(b"provider"), What::Refers);
            }
            if subject.len() > usize::try_from(engine.limits.core.people.identity_bytes).expect("identity bound") {
                problems.push(path.member(b"subject"), What::Bound);
            }
            let identity = IdentityKey { provider, subject };
            for previous in &owners {
                if previous.identity == identity {
                    problems.push(path.clone(), What::Unique);
                    break;
                }
            }
            for project in &engine.projects {
                if owners.push(InitialOwner { project: project.number, identity: identity.clone() }).is_err() {
                    problems.push(path.clone(), What::Bound);
                    break;
                }
            }
        }
    }
    owners.into_boxed()
}
pub(crate) fn connectors(
    document: &Document,
    at: u32,
    path: &Path,
    max: u32,
    problems: &mut Problems,
) -> Option<Box<[u16]>> {
    let items = items(document, at, max, path, problems)?;
    let mut numbers = List::with_capacity(max);
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        match read::integer(document, *at, u64::from(u16::MAX)) {
            Ok(number) => {
                let number = u16::try_from(number).expect("u16 checked");
                if number == jig_domain_core::RECURRING {
                    problems.push(path.clone(), What::Bound);
                }
                if numbers.as_slice().contains(&number) {
                    problems.push(path.clone(), What::Unique);
                }
                numbers.push(number).expect("bounded connector input");
            }
            Err(what) => problems.push(path, what),
        }
    }
    Some(numbers.into_boxed())
}
pub(crate) fn items(document: &Document, at: u32, max: u32, path: &Path, problems: &mut Problems) -> Option<List<u32>> {
    match read::list(document, at, max) {
        Ok(items) => Some(items),
        Err(what) => {
            problems.push(path.clone(), what);
            None
        }
    }
}
pub(crate) fn whole(engine: &crate::Engine, problems: &mut Problems) {
    let limits = &engine.limits.core;
    if limits.people.words > limits.tasks.message_bytes
        || limits.tasks.result_bytes > limits.tasks.message_bytes
        || limits.tasks.message_bytes < 24
    {
        problems.push(Path::root().member(b"limits"), What::Message);
    }
    let mut shares = List::with_capacity(2);
    if let Some(bytes) = jig_domain_core::worst_case(limits) {
        shares.push(crate::Share { name: b"core", bytes }).expect("two memory shares");
    } else {
        problems.push(Path::root().member(b"limits"), What::Bound);
    }
    if let Some(bytes) = jig_domain_host::worst_case(&engine.limits.hub) {
        shares.push(crate::Share { name: b"hub", bytes }).expect("two memory shares");
    } else {
        problems.push(Path::root().member(b"limits"), What::Bound);
    }
    if let Err(what) = crate::fits(engine.memory, shares.as_slice()) {
        problems.push(Path::root().member(b"memory_bytes"), what);
    }
    let Some(mut authority) = jig_domain_authority::Domain::new(engine.rules.clone(), limits.authority) else { return };
    let mut facts = skein_lib::Queue::with_capacity(jig_domain_authority::POLICY_MAX_OUT);
    for (index, project) in engine.projects.iter().enumerate() {
        let path = check::index(&Path::root().member(b"projects"), index);
        jig_domain_authority::step(
            &mut authority,
            jig_domain_authority::Event::Policy { project: project.number, policy: project.policy.clone() },
            &mut facts,
        );
        match facts.pop().expect("one policy admission fact") {
            jig_domain_authority::PolicyFact::Added { .. } | jig_domain_authority::PolicyFact::Changed { .. } => {}
            jig_domain_authority::PolicyFact::Refused { .. } | jig_domain_authority::PolicyFact::Dropped { .. } => {
                problems.push(path.member(b"policy"), What::Bound);
            }
        }
        for (index, role) in project.roles.iter().enumerate() {
            let path = check::index(&path.member(b"roles"), index);
            connector(engine, role.connector, &path.member(b"connector"), problems);
            let mut known = false;
            for configured in &project.policy.roles {
                known |= configured.number == role.role;
            }
            if !known {
                problems.push(path.member(b"role"), What::Refers);
            }
        }
        authority_refs(
            engine,
            &project.policy.ceiling,
            &path.member(b"policy").member(b"ceiling"),
            Some(&project.policy),
            problems,
        );
        for (index, role) in project.policy.roles.iter().enumerate() {
            let path = check::index(&path.member(b"policy").member(b"roles"), index);
            authority_refs(engine, &role.authority, &path.member(b"authority"), Some(&project.policy), problems);
        }
        grant_refs(engine, &project.policy.projections, &path.member(b"policy").member(b"projections"), problems);
        requirement_refs(
            engine,
            &project.policy.requirements,
            &path.member(b"policy").member(b"requirements"),
            problems,
        );
    }
    authority_refs(engine, &engine.rules.ceiling, &Path::root().member(b"rules").member(b"ceiling"), None, problems);
    requirement_refs(
        engine,
        &engine.rules.requirements,
        &Path::root().member(b"rules").member(b"requirements"),
        problems,
    );
    for (index, pair) in engine.rules.implies.entries().iter().enumerate() {
        connector(
            engine,
            pair.connector,
            &check::index(&Path::root().member(b"rules").member(b"implies"), index).member(b"connector"),
            problems,
        );
    }
}
fn connector(engine: &crate::Engine, number: u16, path: &Path, problems: &mut Problems) {
    if !engine.connectors.contains(&number) {
        problems.push(path.clone(), What::Refers);
    }
}
fn grant_refs(engine: &crate::Engine, grants: &[jig_domain_authority::Grant], path: &Path, problems: &mut Problems) {
    for (index, grant) in grants.iter().enumerate() {
        connector(engine, grant.connector, &check::index(path, index).member(b"connector"), problems);
    }
}
fn authority_refs(
    engine: &crate::Engine,
    value: &jig_domain_authority::Authority,
    path: &Path,
    policy: Option<&jig_domain_authority::Policy>,
    problems: &mut Problems,
) {
    grant_refs(engine, &value.grants, &path.member(b"grants"), problems);
    for (index, scope) in value.note_resources.iter().enumerate() {
        connector(
            engine,
            scope.connector,
            &check::index(&path.member(b"note_resources"), index).member(b"connector"),
            problems,
        );
    }
    for (index, executor) in value.delegation.kinds.iter().enumerate() {
        let path = check::index(&path.member(b"delegation").member(b"kinds"), index).member(b"number");
        match executor {
            jig_domain_authority::Executor::Charter(number) => {
                let mut known = false;
                for charter in &engine.charters {
                    known |= charter.number == *number;
                }
                if !known {
                    problems.push(path, What::Refers);
                }
            }
            jig_domain_authority::Executor::Role(number) => {
                if let Some(policy) = policy {
                    let mut known = false;
                    for role in &policy.roles {
                        known |= role.number == *number;
                    }
                    if !known {
                        problems.push(path, What::Refers);
                    }
                }
            }
            jig_domain_authority::Executor::Procedure(_) => {}
        }
    }
}
fn requirement_refs(
    engine: &crate::Engine,
    requirements: &[jig_domain_authority::Requirement],
    path: &Path,
    problems: &mut Problems,
) {
    for (index, requirement) in requirements.iter().enumerate() {
        let path = check::index(path, index);
        connector(engine, requirement.connector, &path.member(b"connector"), problems);
        connector(engine, requirement.judge.connector, &path.member(b"judge").member(b"connector"), problems);
    }
}
