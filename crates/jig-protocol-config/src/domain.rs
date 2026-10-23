//! Deployment, initial identities and connector numbers (configuration.md, 3).
use crate::{Directories, Directory, Path, Problems, What, check, read};
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

pub(crate) fn deployment(
    document: &Document,
    at: u32,
    path: &Path,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Deployment> {
    let mut object = check::object(document, at, path, problems)?;
    let directory = check::path(&mut object, document, b"directory", path, Directory::Data, directories, problems);
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
    engine: &crate::Engine,
    problems: &mut Problems,
) -> Box<[InitialOwner]> {
    let path = &Path::root().member(b"owners");
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
            // With no projects, retain one bootstrap entry per owner for the deployment.
            // initial_owners still bounds retained entries, not distinct identities.
            if engine.projects.is_empty()
                && owners
                    .push(InitialOwner { project: jig_domain_people::DEPLOYMENT, identity: identity.clone() })
                    .is_err()
            {
                problems.push(path.clone(), What::Bound);
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
fn operations(limits: &jig_domain_core::Limits, problems: &mut Problems) {
    if u64::from(limits.people.operations)
        < (u64::from(limits.tasks.delegates) + 1).max(u64::from(limits.tasks.project_goals))
    {
        problems.push(
            Path::root().member(b"limits"),
            What::Operations {
                operations: limits.people.operations,
                delegates: limits.tasks.delegates,
                goals: limits.tasks.project_goals,
            },
        );
    }
}
pub(crate) fn whole(engine: &crate::Engine, problems: &mut Problems) {
    if engine.limits.driver.client.url_bytes > jig_domain_accounts::ADDRESS {
        problems.push(Path::root().member(b"limits").member(b"driver.url_bytes"), What::Bound);
    }
    let limits = &engine.limits.core;
    if engine.projects.len() >= usize::try_from(limits.people.projects).expect("project capacity fits usize") {
        problems.push(Path::root().member(b"projects"), What::Bound);
    }
    operations(limits, problems);
    if limits.people.words > limits.tasks.message_bytes
        || limits.tasks.result_bytes > limits.tasks.message_bytes
        || limits.tasks.message_bytes < 24
    {
        problems.push(Path::root().member(b"limits"), What::Message);
    }
    let mut shares = List::with_capacity(3);
    if let Some(bytes) = jig_domain_core::worst_case(limits) {
        shares.push(crate::Share { name: b"core", bytes }).expect("three memory shares");
    } else {
        problems.push(Path::root().member(b"limits"), What::Bound);
    }
    if let Some(bytes) = jig_domain_host::worst_case(&engine.limits.hub) {
        shares.push(crate::Share { name: b"hub", bytes }).expect("three memory shares");
    } else {
        problems.push(Path::root().member(b"limits"), What::Bound);
    }
    match provider_terms_bytes(&engine.llm.providers) {
        Some(bytes) => shares.push(crate::Share { name: b"provider terms", bytes }).expect("three memory shares"),
        None => problems.push(Path::root().member(b"llm").member(b"providers"), What::Bound),
    }
    if let Err(what) = crate::fits(engine.memory, shares.as_slice()) {
        problems.push(Path::root().member(b"memory_bytes"), what);
    }
    for (index, project) in engine.projects.iter().enumerate() {
        let path = check::index(&Path::root().member(b"projects"), index);
        if !check_policy(engine, project.number, &project.policy, &project.roles, &project.templates, &path, problems) {
            problems.push(path.member(b"policy"), What::Bound);
        }
    }
    for (index, policy) in engine.policies.iter().enumerate() {
        let path = check::index(&Path::root().member(b"policies"), index);
        if !check_policy(engine, policy.number, &policy.policy, &policy.roles, &policy.templates, &path, problems) {
            problems.push(path, What::Bound);
        }
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
fn check_policy(
    engine: &crate::Engine,
    number: u32,
    policy: &jig_domain_authority::Policy,
    roles: &[jig_domain_people::PermissionRole],
    templates: &[jig_domain_people::TemplateOverride],
    path: &Path,
    problems: &mut Problems,
) -> bool {
    let Some(mut authority) = jig_domain_authority::Domain::new(engine.rules.clone(), engine.limits.core.authority)
    else {
        return false;
    };
    let mut facts = skein_lib::Queue::with_capacity(jig_domain_authority::POLICY_MAX_OUT);
    jig_domain_authority::step(
        &mut authority,
        jig_domain_authority::Event::Policy { project: number, policy: policy.clone() },
        &mut facts,
    );
    let admitted = match facts.pop().expect("one policy admission fact") {
        jig_domain_authority::PolicyFact::Added { .. } | jig_domain_authority::PolicyFact::Changed { .. } => true,
        jig_domain_authority::PolicyFact::Refused { .. } | jig_domain_authority::PolicyFact::Dropped { .. } => false,
    };
    crate::templates::check_overrides(engine, templates, &path.member(b"policy").member(b"templates"), problems);
    for (index, role) in roles.iter().enumerate() {
        let path = check::index(&path.member(b"roles"), index);
        connector(engine, role.connector, &path.member(b"connector"), problems);
        let mut known = false;
        for configured in &policy.roles {
            known |= configured.number == role.role;
        }
        if !known {
            problems.push(path.member(b"role"), What::Refers);
        }
    }
    authority_refs(engine, &policy.ceiling, &path.member(b"policy").member(b"ceiling"), Some(policy), problems);
    for (index, role) in policy.roles.iter().enumerate() {
        let path = check::index(&path.member(b"policy").member(b"roles"), index);
        authority_refs(engine, &role.authority, &path.member(b"authority"), Some(policy), problems);
    }
    grant_refs(engine, &policy.projections, &path.member(b"policy").member(b"projections"), problems);
    requirement_refs(engine, &policy.requirements, &path.member(b"policy").member(b"requirements"), problems);
    let bytes = match jig_domain_core::policy_snapshot(policy, roles, templates) {
        Some(value) => jig_domain_people::policy_bytes(&value),
        None => None,
    };
    let bounded = match bytes {
        Some(bytes) => bytes <= engine.limits.core.policy_bytes,
        None => false,
    };
    admitted && bounded
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

/// The parser's added terms and the two runtime copies (settings and accounts).
/// Charge a full provider's inline size per copy, including padding; the original
/// configured name was already present, while runtime names and all URLs are new.
fn provider_terms_bytes(providers: &[crate::llm::Provider]) -> Option<u64> {
    let mut total = 0_u64;
    for provider in providers {
        let address = match &provider.manage_usage {
            Some(value) => u64::try_from(value.len()).ok()?,
            None => 0,
        };
        let inline = u64::try_from(size_of::<jig_domain_accounts::Provider>()).ok()?.checked_mul(3)?;
        let names = u64::try_from(provider.name.len()).ok()?.checked_mul(2)?;
        total = total.checked_add(inline)?.checked_add(names)?.checked_add(address.checked_mul(3)?)?;
    }
    Some(total)
}
