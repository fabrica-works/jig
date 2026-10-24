//! Deployment, initial identities and connector numbers (configuration.md, 3).
use crate::{CodecBound, Directories, Directory, Path, Problem, Problems, What, check, read};
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
    if let Err(problem) = check_effective_limits(limits) {
        problems.push(problem.path, problem.what);
    }
    let settings_path = Path::root().member(b"limits").member(b"core").member(b"settings.history");
    if limits.settings.history == 0 {
        problems.push(settings_path, What::Bound);
    }
    if match jig_domain_core::settings::writes_worst_case(limits) {
        Some(writes) => writes > engine.limits.journal.writes,
        None => true,
    } {
        problems.push(Path::root().member(b"limits").member(b"journal.writes"), What::Bound);
    }
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

/// Check the effective core limits that will be handed to the store (configuration.md, 6).
///
/// This probes the people family and the combined store codec without changing any
/// limits. Scalar refusals name the field that fails at the envelope's next byte.
/// Independent people ceilings retain the generated first failure. Arithmetic and
/// refusals from other store families have distinct named problems.
pub fn check_effective_limits(limits: &jig_domain_core::Limits) -> Result<(), Problem> {
    if codec_admits(limits) {
        return Ok(());
    }
    // Isolate independent ceilings before diagnosing either byte budget. These
    // are probe inputs only; the caller's effective limits are never changed.
    let mut baseline = *limits;
    baseline.policy_bytes = 0;
    baseline.people.amendment_bytes = 0;
    let Some(derived) = jig_protocol_store::codec::people::derived_limits(&baseline) else {
        return Err(Problem { path: Path::root().member(b"limits"), what: What::CodecArithmetic });
    };
    if let Err(problem) = jig_protocol_store::people_v13::check_limits(&derived) {
        let configured = match jig_protocol_store::codec::people::derived_limits(limits) {
            Some(configured) => configured,
            None => derived,
        };
        let at = people_bound(&configured, problem.path);
        return Err(Problem { path: codec_path(at.name), what: What::CodecCeiling { bound: at.bound } });
    }
    if !codec_admits(&baseline) {
        return Err(Problem { path: Path::root().member(b"limits"), what: What::StoreCodec });
    }
    if let Some(maximum) = policy_bytes_max(limits)
        && limits.policy_bytes > maximum
    {
        return Err(policy_envelope(limits, maximum));
    }
    if let Some(maximum) = amendment_bytes_max(limits)
        && limits.people.amendment_bytes > maximum
    {
        return Err(amendment_envelope(limits, maximum));
    }
    // Neither axis admits even zero with the other held fixed. Diagnose both
    // excessive budgets in order, using isolated probe inputs only.
    let maximum = policy_bytes_max(&baseline).expect("zero policy was admitted");
    if limits.policy_bytes > maximum {
        baseline.policy_bytes = limits.policy_bytes;
        return Err(policy_envelope(&baseline, maximum));
    }
    Err(Problem { path: Path::root().member(b"limits"), what: What::StoreCodec })
}

fn policy_envelope(limits: &jig_domain_core::Limits, maximum: u64) -> Problem {
    let mut edge = *limits;
    edge.policy_bytes = maximum.checked_add(1).expect("policy arithmetic refuses u64::MAX");
    Problem {
        path: codec_path(b"core.policy_bytes"),
        what: What::PolicyEnvelope { value: limits.policy_bytes, maximum, bound: binding_bound(&edge, limits) },
    }
}

fn amendment_envelope(limits: &jig_domain_core::Limits, maximum: u32) -> Problem {
    let mut edge = *limits;
    edge.people.amendment_bytes = maximum.checked_add(1).expect("amendment arithmetic refuses u32::MAX");
    Problem {
        path: codec_path(b"people.amendment_bytes"),
        what: What::AmendmentEnvelope {
            value: limits.people.amendment_bytes,
            maximum,
            bound: binding_bound(&edge, limits),
        },
    }
}

/// Largest policy byte budget admitted by the complete codec with all other
/// effective limits unchanged. `None` means even a zero budget is refused.
/// The search makes at most 64 probes after the zero-budget admission check.
#[must_use]
pub fn policy_bytes_max(limits: &jig_domain_core::Limits) -> Option<u64> {
    let mut candidate = *limits;
    candidate.policy_bytes = 0;
    if !codec_admits(&candidate) {
        return None;
    }
    let mut admitted = 0_u64;
    let mut refused = u64::MAX; // people derivation narrows policy bytes to u32
    for _ in 0..u64::BITS {
        let span = refused.checked_sub(admitted)?;
        if span <= 1 {
            break;
        }
        let middle = admitted.checked_add(span.checked_div(2)?)?;
        candidate.policy_bytes = middle;
        if codec_admits(&candidate) {
            admitted = middle;
        } else {
            refused = middle;
        }
    }
    Some(admitted)
}

/// Largest amendment byte budget admitted by the complete codec with all other
/// effective limits unchanged. `None` means even a zero budget is refused.
/// The search makes at most 32 probes after the zero-budget admission check.
#[must_use]
pub fn amendment_bytes_max(limits: &jig_domain_core::Limits) -> Option<u32> {
    let mut candidate = *limits;
    candidate.people.amendment_bytes = 0;
    if !codec_admits(&candidate) {
        return None;
    }
    let mut admitted = 0_u32;
    let mut refused = u32::MAX; // member_dependencies requires checked 3 * amendment
    for _ in 0..u32::BITS {
        let span = refused.checked_sub(admitted)?;
        if span <= 1 {
            break;
        }
        let middle = admitted.checked_add(span.checked_div(2)?)?;
        candidate.people.amendment_bytes = middle;
        if codec_admits(&candidate) {
            admitted = middle;
        } else {
            refused = middle;
        }
    }
    Some(admitted)
}

fn codec_admits(limits: &jig_domain_core::Limits) -> bool {
    jig_protocol_store::codec::people::limits(limits).is_some() && jig_protocol_store::codec::limits(limits).is_some()
}

fn binding_bound(edge: &jig_domain_core::Limits, configured: &jig_domain_core::Limits) -> Option<CodecBound> {
    let edge = jig_protocol_store::codec::people::derived_limits(edge)?;
    let problem = jig_protocol_store::people_v13::check_limits(&edge).err()?;
    let configured = jig_protocol_store::codec::people::derived_limits(configured)?;
    Some(people_bound(&configured, problem.path).bound)
}

pub(crate) fn codec_path(name: &[u8]) -> Path {
    let path = Path::root().member(b"limits");
    match skein_lib::bytes::find(name, b".") {
        Some(dot) => path
            .member(name.get(..dot).expect("dot is within name"))
            .member(name.get(dot.checked_add(1).expect("name length fits usize")..).expect("dot is within name")),
        None => path.member(name),
    }
}

struct PeopleBound {
    name: &'static [u8],
    bound: CodecBound,
}

// This table translates generated limit paths, never derives bounds. Values and
// ceilings come from the codec's records; their arithmetic has one owner.
#[expect(
    clippy::too_many_lines,
    reason = "one projection lists every generated people limit path without deriving bounds"
)]
fn people_bound(
    limits: &jig_protocol_store::people_v13::Limits,
    path: jig_protocol_store::people_v13::Path,
) -> PeopleBound {
    use jig_protocol_store::people_v13 as wire;
    let ceilings = &wire::CEILINGS;
    if path == wire::Path::SegmentBytes {
        return PeopleBound {
            name: b"authority.segment_bytes",
            bound: CodecBound {
                field: b"Segment.bytes",
                needed: limits.segment_bytes,
                ceiling: ceilings.segment_bytes,
            },
        };
    }
    if path == wire::Path::IdentityKeySubject {
        return PeopleBound {
            name: b"people.identity_bytes",
            bound: CodecBound {
                field: b"IdentityKey.subject",
                needed: limits.identity_key_subject,
                ceiling: ceilings.identity_key_subject,
            },
        };
    }
    if path == wire::Path::IdentityLogin {
        return PeopleBound {
            name: b"people.identity_bytes",
            bound: CodecBound {
                field: b"Identity.login",
                needed: limits.identity_login,
                ceiling: ceilings.identity_login,
            },
        };
    }
    if path == wire::Path::IdentityName {
        return PeopleBound {
            name: b"people.identity_bytes",
            bound: CodecBound {
                field: b"Identity.name",
                needed: limits.identity_name,
                ceiling: ceilings.identity_name,
            },
        };
    }
    if path == wire::Path::LastExactValue {
        return PeopleBound {
            name: b"authority.segment_bytes",
            bound: CodecBound {
                field: b"LastExact.value",
                needed: limits.last_exact_value,
                ceiling: ceilings.last_exact_value,
            },
        };
    }
    if path == wire::Path::LastOpenValue {
        return PeopleBound {
            name: b"authority.segment_bytes",
            bound: CodecBound {
                field: b"LastOpen.value",
                needed: limits.last_open_value,
                ceiling: ceilings.last_open_value,
            },
        };
    }
    if path == wire::Path::PatternSegments {
        return PeopleBound {
            name: b"core.policy_bytes",
            bound: CodecBound {
                field: b"Pattern.segments",
                needed: limits.pattern_segments,
                ceiling: ceilings.pattern_segments,
            },
        };
    }
    if path == wire::Path::DelegationKinds {
        return PeopleBound {
            name: b"core.policy_bytes",
            bound: CodecBound {
                field: b"Delegation.kinds",
                needed: limits.delegation_kinds,
                ceiling: ceilings.delegation_kinds,
            },
        };
    }
    if path == wire::Path::AuthorityGrants {
        return PeopleBound {
            name: b"core.policy_bytes",
            bound: CodecBound {
                field: b"Authority.grants",
                needed: limits.authority_grants,
                ceiling: ceilings.authority_grants,
            },
        };
    }
    if path == wire::Path::AuthorityNoteResources {
        return PeopleBound {
            name: b"core.policy_bytes",
            bound: CodecBound {
                field: b"Authority.note_resources",
                needed: limits.authority_note_resources,
                ceiling: ceilings.authority_note_resources,
            },
        };
    }
    if path == wire::Path::PolicyValueProjections {
        return PeopleBound {
            name: b"authority.grants",
            bound: CodecBound {
                field: b"PolicyValue.projections",
                needed: limits.policy_value_projections,
                ceiling: ceilings.policy_value_projections,
            },
        };
    }
    if path == wire::Path::PolicyValueRoles {
        return PeopleBound {
            name: b"authority.roles",
            bound: CodecBound {
                field: b"PolicyValue.roles",
                needed: limits.policy_value_roles,
                ceiling: ceilings.policy_value_roles,
            },
        };
    }
    if path == wire::Path::PolicyValueRequirements {
        return PeopleBound {
            name: b"authority.requirements",
            bound: CodecBound {
                field: b"PolicyValue.requirements",
                needed: limits.policy_value_requirements,
                ceiling: ceilings.policy_value_requirements,
            },
        };
    }
    if path == wire::Path::PolicyValuePermissions {
        return PeopleBound {
            name: b"core.policy_bytes",
            bound: CodecBound {
                field: b"PolicyValue.permissions",
                needed: limits.policy_value_permissions,
                ceiling: ceilings.policy_value_permissions,
            },
        };
    }
    if path == wire::Path::PolicyValueTemplates {
        return PeopleBound {
            name: b"core.templates",
            bound: CodecBound {
                field: b"PolicyValue.templates",
                needed: limits.policy_value_templates,
                ceiling: ceilings.policy_value_templates,
            },
        };
    }
    if path == wire::Path::PolicyChangeRequirementsRequirements {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"PolicyChangeRequirements.requirements",
                needed: limits.policy_change_requirements_requirements,
                ceiling: ceilings.policy_change_requirements_requirements,
            },
        };
    }
    if path == wire::Path::PolicyChangeProjectionsGrants {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"PolicyChangeProjections.grants",
                needed: limits.policy_change_projections_grants,
                ceiling: ceilings.policy_change_projections_grants,
            },
        };
    }
    if path == wire::Path::PolicyChangePermissionsMappings {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"PolicyChangePermissions.mappings",
                needed: limits.policy_change_permissions_mappings,
                ceiling: ceilings.policy_change_permissions_mappings,
            },
        };
    }
    if path == wire::Path::PolicyChangeTemplatesOverrides {
        return PeopleBound {
            name: b"core.templates",
            bound: CodecBound {
                field: b"PolicyChangeTemplates.overrides",
                needed: limits.policy_change_templates_overrides,
                ceiling: ceilings.policy_change_templates_overrides,
            },
        };
    }
    if path == wire::Path::NoteChangeCorrectDescription {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"NoteChangeCorrect.description",
                needed: limits.note_change_correct_description,
                ceiling: ceilings.note_change_correct_description,
            },
        };
    }
    if path == wire::Path::NoteChangeCorrectBody {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"NoteChangeCorrect.body",
                needed: limits.note_change_correct_body,
                ceiling: ceilings.note_change_correct_body,
            },
        };
    }
    if path == wire::Path::NoteChangeCorrectReferences {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"NoteChangeCorrect.references",
                needed: limits.note_change_correct_references,
                ceiling: ceilings.note_change_correct_references,
            },
        };
    }
    if path == wire::Path::ResourceNamePath {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"ResourceName.path",
                needed: limits.resource_name_path,
                ceiling: ceilings.resource_name_path,
            },
        };
    }
    if path == wire::Path::AdoptionOptions {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"Adoption.options",
                needed: limits.adoption_options,
                ceiling: ceilings.adoption_options,
            },
        };
    }
    if path == wire::Path::PersonResultApprovedState {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"PersonResultApproved.state",
                needed: limits.person_result_approved_state,
                ceiling: ceilings.person_result_approved_state,
            },
        };
    }
    if path == wire::Path::PersonResultApprovedWords {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"PersonResultApproved.words",
                needed: limits.person_result_approved_words_some,
                ceiling: ceilings.person_result_approved_words_some,
            },
        };
    }
    if path == wire::Path::PersonResultReportWords {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"PersonResultReport.words",
                needed: limits.person_result_report_words,
                ceiling: ceilings.person_result_report_words,
            },
        };
    }
    if path == wire::Path::PersonResultVerdictWords {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"PersonResultVerdict.words",
                needed: limits.person_result_verdict_words,
                ceiling: ceilings.person_result_verdict_words,
            },
        };
    }
    if path == wire::Path::PersonResultFailureReason {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"PersonResultFailure.reason",
                needed: limits.person_result_failure_reason,
                ceiling: ceilings.person_result_failure_reason,
            },
        };
    }
    if path == wire::Path::ProposalDecisionRejectReason {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"ProposalDecisionReject.reason",
                needed: limits.proposal_decision_reject_reason,
                ceiling: ceilings.proposal_decision_reject_reason,
            },
        };
    }
    if path == wire::Path::EscalationDecisionRejectReason {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"EscalationDecisionReject.reason",
                needed: limits.escalation_decision_reject_reason,
                ceiling: ceilings.escalation_decision_reject_reason,
            },
        };
    }
    if path == wire::Path::ParameterBytesValue {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"ParameterBytes.value",
                needed: limits.parameter_bytes_value,
                ceiling: ceilings.parameter_bytes_value,
            },
        };
    }
    if path == wire::Path::SpecWords {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound { field: b"Spec.words", needed: limits.spec_words, ceiling: ceilings.spec_words },
        };
    }
    if path == wire::Path::SpecParameters {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"Spec.parameters",
                needed: limits.spec_parameters,
                ceiling: ceilings.spec_parameters,
            },
        };
    }
    if path == wire::Path::SpecInputs {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound { field: b"Spec.inputs", needed: limits.spec_inputs, ceiling: ceilings.spec_inputs },
        };
    }
    if path == wire::Path::LlmChoiceSubagents {
        return PeopleBound {
            name: b"people-v13.LlmChoice.subagents",
            bound: CodecBound {
                field: b"LlmChoice.subagents",
                needed: limits.llm_choice_subagents,
                ceiling: ceilings.llm_choice_subagents,
            },
        };
    }
    if path == wire::Path::AmendmentInstructions {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"Amendment.instructions",
                needed: limits.amendment_instructions_some,
                ceiling: ceilings.amendment_instructions_some,
            },
        };
    }
    if path == wire::Path::AmendmentDependencies {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"Amendment.dependencies",
                needed: limits.amendment_dependencies_some,
                ceiling: ceilings.amendment_dependencies_some,
            },
        };
    }
    if path == wire::Path::AmendmentReason {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"Amendment.reason",
                needed: limits.amendment_reason,
                ceiling: ceilings.amendment_reason,
            },
        };
    }
    if path == wire::Path::FieldRuleName {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"FieldRule.name",
                needed: limits.field_rule_name,
                ceiling: ceilings.field_rule_name,
            },
        };
    }
    if path == wire::Path::VerdictFields {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"Verdict.fields",
                needed: limits.verdict_fields,
                ceiling: ceilings.verdict_fields,
            },
        };
    }
    if path == wire::Path::ContractVerdictChoices {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"ContractVerdict.choices",
                needed: limits.contract_verdict_choices,
                ceiling: ceilings.contract_verdict_choices,
            },
        };
    }
    if path == wire::Path::MemberWords {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound { field: b"Member.words", needed: limits.member_words, ceiling: ceilings.member_words },
        };
    }
    if path == wire::Path::MemberParameters {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"Member.parameters",
                needed: limits.member_parameters,
                ceiling: ceilings.member_parameters,
            },
        };
    }
    if path == wire::Path::MemberInputs {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"Member.inputs",
                needed: limits.member_inputs,
                ceiling: ceilings.member_inputs,
            },
        };
    }
    if path == wire::Path::MemberDependencies {
        return PeopleBound {
            name: b"people.amendment_bytes",
            bound: CodecBound {
                field: b"Member.dependencies",
                needed: limits.member_dependencies,
                ceiling: ceilings.member_dependencies,
            },
        };
    }
    if path == wire::Path::MemberNamed {
        return PeopleBound {
            name: b"core.policy_bytes",
            bound: CodecBound { field: b"Member.named", needed: limits.member_named, ceiling: ceilings.member_named },
        };
    }
    if path == wire::Path::OperationCancelReason {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"OperationCancel.reason",
                needed: limits.operation_cancel_reason,
                ceiling: ceilings.operation_cancel_reason,
            },
        };
    }
    if path == wire::Path::OperationMoveReason {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"OperationMove.reason",
                needed: limits.operation_move_reason,
                ceiling: ceilings.operation_move_reason,
            },
        };
    }
    if path == wire::Path::OperationMessageWords {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"OperationMessage.words",
                needed: limits.operation_message_words,
                ceiling: ceilings.operation_message_words,
            },
        };
    }
    if path == wire::Path::AskCompoundOperations {
        return PeopleBound {
            name: b"people.operations",
            bound: CodecBound {
                field: b"AskCompound.operations",
                needed: limits.ask_compound_operations,
                ceiling: ceilings.ask_compound_operations,
            },
        };
    }
    if path == wire::Path::AskCreateBatch {
        return PeopleBound {
            name: b"people.batch",
            bound: CodecBound {
                field: b"AskCreate.batch",
                needed: limits.ask_create_batch,
                ceiling: ceilings.ask_create_batch,
            },
        };
    }
    if path == wire::Path::AskMakeServiceName {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"AskMakeService.name",
                needed: limits.ask_make_service_name,
                ceiling: ceilings.ask_make_service_name,
            },
        };
    }
    if path == wire::Path::ActionBatchMembers {
        return PeopleBound {
            name: b"people.batch",
            bound: CodecBound {
                field: b"ActionBatch.members",
                needed: limits.action_batch_members,
                ceiling: ceilings.action_batch_members,
            },
        };
    }
    if path == wire::Path::AskProposeReason {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"AskPropose.reason",
                needed: limits.ask_propose_reason,
                ceiling: ceilings.ask_propose_reason,
            },
        };
    }
    if path == wire::Path::AskCancelReason {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"AskCancel.reason",
                needed: limits.ask_cancel_reason,
                ceiling: ceilings.ask_cancel_reason,
            },
        };
    }
    if path == wire::Path::AskMoveReason {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"AskMove.reason",
                needed: limits.ask_move_reason,
                ceiling: ceilings.ask_move_reason,
            },
        };
    }
    if path == wire::Path::AskSayWords {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound { field: b"AskSay.words", needed: limits.ask_say_words, ceiling: ceilings.ask_say_words },
        };
    }
    if path == wire::Path::AskAnswerQuestionWords {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"AskAnswerQuestion.words",
                needed: limits.ask_answer_question_words,
                ceiling: ceilings.ask_answer_question_words,
            },
        };
    }
    if path == wire::Path::AskSetRolesHoldings {
        return PeopleBound {
            name: b"people.holdings",
            bound: CodecBound {
                field: b"AskSetRoles.holdings",
                needed: limits.ask_set_roles_holdings,
                ceiling: ceilings.ask_set_roles_holdings,
            },
        };
    }
    if path == wire::Path::AskRenameName {
        return PeopleBound {
            name: b"people-v13.AskRename.name",
            bound: CodecBound {
                field: b"AskRename.name",
                needed: limits.ask_rename_name,
                ceiling: ceilings.ask_rename_name,
            },
        };
    }
    if path == wire::Path::RefusalFullTasks {
        return PeopleBound {
            name: b"tasks.circle",
            bound: CodecBound {
                field: b"RefusalFull.tasks",
                needed: limits.refusal_full_tasks,
                ceiling: ceilings.refusal_full_tasks,
            },
        };
    }
    if path == wire::Path::AskMakeProjectName {
        return PeopleBound {
            name: b"people-v13.AskMakeProject.name",
            bound: CodecBound {
                field: b"AskMakeProject.name",
                needed: limits.ask_make_project_name,
                ceiling: ceilings.ask_make_project_name,
            },
        };
    }
    if path == wire::Path::AccountChangeDeclareProgramName {
        return PeopleBound {
            name: b"people-v13.AccountChangeDeclareProgram.name",
            bound: CodecBound {
                field: b"AccountChangeDeclareProgram.name",
                needed: limits.account_change_declare_program_name,
                ceiling: ceilings.account_change_declare_program_name,
            },
        };
    }
    if path == wire::Path::AccountChangeAddKeyDigest {
        return PeopleBound {
            name: b"people-v13.AccountChangeAddKey.digest",
            bound: CodecBound {
                field: b"AccountChangeAddKey.digest",
                needed: limits.account_change_add_key_digest,
                ceiling: ceilings.account_change_add_key_digest,
            },
        };
    }
    if path == wire::Path::AccountChangeReplaceKeyDigest {
        return PeopleBound {
            name: b"people-v13.AccountChangeReplaceKey.digest",
            bound: CodecBound {
                field: b"AccountChangeReplaceKey.digest",
                needed: limits.account_change_replace_key_digest,
                ceiling: ceilings.account_change_replace_key_digest,
            },
        };
    }
    if path == wire::Path::AccountChangeSetUsesUses {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"AccountChangeSetUses.uses",
                needed: limits.account_change_set_uses_uses_some,
                ceiling: ceilings.account_change_set_uses_uses_some,
            },
        };
    }
    if path == wire::Path::AccountChangeMoveInDigest {
        return PeopleBound {
            name: b"people-v13.AccountChangeMoveIn.digest",
            bound: CodecBound {
                field: b"AccountChangeMoveIn.digest",
                needed: limits.account_change_move_in_digest,
                ceiling: ceilings.account_change_move_in_digest,
            },
        };
    }
    if path == wire::Path::StoredRolesHoldings {
        return PeopleBound {
            name: b"people.holdings",
            bound: CodecBound {
                field: b"StoredRoles.holdings",
                needed: limits.stored_roles_holdings,
                ceiling: ceilings.stored_roles_holdings,
            },
        };
    }
    if path == wire::Path::ModelNameName {
        return PeopleBound {
            name: b"people-v13.ModelName.name",
            bound: CodecBound {
                field: b"ModelName.name",
                needed: limits.model_name_name,
                ceiling: ceilings.model_name_name,
            },
        };
    }
    if path == wire::Path::StoredPolicyName {
        return PeopleBound {
            name: b"people-v13.StoredPolicy.name",
            bound: CodecBound {
                field: b"StoredPolicy.name",
                needed: limits.stored_policy_name,
                ceiling: ceilings.stored_policy_name,
            },
        };
    }
    if path == wire::Path::StoredPolicyModels {
        return PeopleBound {
            name: b"people-v13.StoredPolicy.models",
            bound: CodecBound {
                field: b"StoredPolicy.models",
                needed: limits.stored_policy_models,
                ceiling: ceilings.stored_policy_models,
            },
        };
    }
    if path == wire::Path::RefusalCircleTasks {
        return PeopleBound {
            name: b"tasks.circle",
            bound: CodecBound {
                field: b"RefusalCircle.tasks",
                needed: limits.refusal_circle_tasks,
                ceiling: ceilings.refusal_circle_tasks,
            },
        };
    }
    if path == wire::Path::RefusalKeptPending {
        return PeopleBound {
            name: b"tasks.dependencies",
            bound: CodecBound {
                field: b"RefusalKept.pending",
                needed: limits.refusal_kept_pending,
                ceiling: ceilings.refusal_kept_pending,
            },
        };
    }
    if path == wire::Path::OutcomeAccountSignInStartedAddress {
        return PeopleBound {
            name: b"people-v13.OutcomeAccountSignInStarted.address",
            bound: CodecBound {
                field: b"OutcomeAccountSignInStarted.address",
                needed: limits.outcome_account_sign_in_started_address,
                ceiling: ceilings.outcome_account_sign_in_started_address,
            },
        };
    }
    if path == wire::Path::AccountHostFailureName {
        return PeopleBound {
            name: b"people.program_host_name_bytes",
            bound: CodecBound {
                field: b"AccountHostFailure.name",
                needed: limits.account_host_failure_name,
                ceiling: ceilings.account_host_failure_name,
            },
        };
    }
    if path == wire::Path::OutcomeAccountProgramDeclaredName {
        return PeopleBound {
            name: b"people-v13.OutcomeAccountProgramDeclared.name",
            bound: CodecBound {
                field: b"OutcomeAccountProgramDeclared.name",
                needed: limits.outcome_account_program_declared_name,
                ceiling: ceilings.outcome_account_program_declared_name,
            },
        };
    }
    if path == wire::Path::OutcomeAccountSignedOutFailedHosts {
        return PeopleBound {
            name: b"people.program_hosts",
            bound: CodecBound {
                field: b"OutcomeAccountSignedOut.failed_hosts",
                needed: limits.outcome_account_signed_out_failed_hosts,
                ceiling: ceilings.outcome_account_signed_out_failed_hosts,
            },
        };
    }
    if path == wire::Path::OutcomeAccountRemovedFailedHosts {
        return PeopleBound {
            name: b"people.program_hosts",
            bound: CodecBound {
                field: b"OutcomeAccountRemoved.failed_hosts",
                needed: limits.outcome_account_removed_failed_hosts,
                ceiling: ceilings.outcome_account_removed_failed_hosts,
            },
        };
    }
    if path == wire::Path::ActCloseReason {
        return PeopleBound {
            name: b"people.words",
            bound: CodecBound {
                field: b"ActClose.reason",
                needed: limits.act_close_reason,
                ceiling: ceilings.act_close_reason,
            },
        };
    }
    if path == wire::Path::ActMakeProjectName {
        return PeopleBound {
            name: b"people-v13.ActMakeProject.name",
            bound: CodecBound {
                field: b"ActMakeProject.name",
                needed: limits.act_make_project_name,
                ceiling: ceilings.act_make_project_name,
            },
        };
    }
    if path == wire::Path::StoredAnswerModels {
        return PeopleBound {
            name: b"people-v13.StoredAnswer.models",
            bound: CodecBound {
                field: b"StoredAnswer.models",
                needed: limits.stored_answer_models,
                ceiling: ceilings.stored_answer_models,
            },
        };
    }
    unreachable!("generated check_limits returns a limit path listed above")
}
