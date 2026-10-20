//! Deployment rules and seeded project policies (configuration.md, 3; authority.md, 6).
use crate::{Path, Problems, What, check, domain, read};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core::Model;
use jig_domain_people::PermissionRole;
use skein_json::Document;
use skein_lib::{List, Wall};

#[derive(Debug)]
pub struct Project {
    pub number: u32,
    pub roles: Box<[PermissionRole]>,
    pub templates: Box<[jig_domain_people::TemplateOverride]>,
    pub policy: authority::Policy,
}

pub(crate) fn rules(
    document: &Document,
    at: u32,
    path: &Path,
    limits: &authority::Limits,
    configured: &[Model],
    problems: &mut Problems,
) -> Option<authority::Rules> {
    let mut object = check::object(document, at, path, problems)?;
    let at = object.required(document, b"ceiling", path, problems);
    let ceiling = value(document, at, &path.member(b"ceiling"), limits, Some(configured), problems);
    let period_spend = check::integer(&mut object, document, b"period_spend", path, problems);
    let minimum_run_spend = check::integer(&mut object, document, b"minimum_run_spend", path, problems);
    let maximum_run_spend = check::integer(&mut object, document, b"maximum_run_spend", path, problems);
    let implies = implications(&mut object, document, path, limits, problems);
    let requirements = requirements(&mut object, document, path, limits, problems);
    object.finish(document, path, problems);
    let rules = authority::Rules {
        ceiling: ceiling?,
        period_spend: period_spend?,
        minimum_run_spend: minimum_run_spend?,
        maximum_run_spend: maximum_run_spend?,
        implies: implies?,
        requirements: requirements?,
    };
    if authority::Domain::new(rules.clone(), *limits).is_none() {
        problems.push(path.clone(), What::Bound);
    }
    Some(rules)
}
pub(crate) fn projects(
    document: &Document,
    at: u32,
    path: &Path,
    limits: &crate::Limits,
    configured: &[Model],
    problems: &mut Problems,
) -> Option<Box<[Project]>> {
    let items = domain::items(document, at, limits.core.authority.projects, path, problems)?;
    let mut projects = List::<Project>::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        let Some(mut object) = check::object(document, *at, &path, problems) else { continue };
        let number = check::count(&mut object, document, b"number", &path, problems);
        let at = object.required(document, b"policy", &path, problems);
        let policy = policy(document, at, &path.member(b"policy"), limits, configured, problems);
        let roles = permissions(&mut object, document, &path, limits, problems);
        object.finish(document, &path, problems);
        if let Some(number) = number
            && let Some((policy, templates)) = policy
            && let Some(roles) = roles
        {
            for previous in &projects {
                if previous.number == number {
                    problems.push(path.member(b"number"), What::Unique);
                    break;
                }
            }
            projects.push(Project { number, roles, templates, policy }).expect("bounded project input");
        }
    }
    Some(projects.into_boxed())
}
fn permissions(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    limits: &crate::Limits,
    problems: &mut Problems,
) -> Option<Box<[PermissionRole]>> {
    let items = check::list(object, document, b"roles", document.len(), path, problems)?;
    let mut roles = List::<PermissionRole>::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path.member(b"roles"), index);
        let Some(mut object) = check::object(document, *at, &path, problems) else { continue };
        let connector = check::short(&mut object, document, b"connector", &path, problems);
        let permission = check::short(&mut object, document, b"permission", &path, problems);
        let role = check::count(&mut object, document, b"role", &path, problems);
        object.finish(document, &path, problems);
        if let Some(connector) = connector
            && let Some(permission) = permission
            && let Some(role) = role
        {
            for previous in &roles {
                if previous.connector == connector && previous.permission == permission {
                    problems.push(path.clone(), What::Unique);
                    break;
                }
            }
            roles.push(PermissionRole { connector, permission, role }).expect("bounded permissions");
        }
    }
    if u64::from(items.len()).checked_mul(u64::try_from(size_of::<PermissionRole>()).expect("role size"))
        > Some(limits.core.policy_bytes)
    {
        problems.push(path.member(b"roles"), What::Bound);
    }
    Some(roles.into_boxed())
}
fn policy(
    document: &Document,
    at: Option<u32>,
    path: &Path,
    limits: &crate::Limits,
    configured: &[Model],
    problems: &mut Problems,
) -> Option<(authority::Policy, Box<[jig_domain_people::TemplateOverride]>)> {
    let mut object = check::object(document, at?, path, problems)?;
    let at = object.required(document, b"templates", path, problems);
    let templates =
        crate::templates::overrides(document, at?, &path.member(b"templates"), limits.core.templates, problems);
    let limits = &limits.core.authority;
    let escalation = check::optional(&mut object, document, b"escalation_role", path, problems);
    let escalation_role = optional_count(escalation, &path.member(b"escalation_role"), problems);
    let at = object.required(document, b"ceiling", path, problems);
    let ceiling = value(document, at, &path.member(b"ceiling"), limits, Some(configured), problems);
    let projections = grants(&mut object, document, b"projections", path, limits, problems);
    let period_spend = check::integer(&mut object, document, b"period_spend", path, problems);
    let roles = roles(&mut object, document, path, limits, problems);
    let requirements = requirements(&mut object, document, path, limits, problems);
    object.finish(document, path, problems);
    Some((
        authority::Policy {
            escalation_role: escalation_role.ok()?,
            ceiling: ceiling?,
            projections: projections?,
            period_spend: period_spend?,
            roles: roles?,
            requirements: requirements?,
        },
        templates?,
    ))
}
fn roles(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    limits: &authority::Limits,
    problems: &mut Problems,
) -> Option<Box<[authority::Role]>> {
    let items = check::list(object, document, b"roles", limits.roles, path, problems)?;
    let mut roles = List::<authority::Role>::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path.member(b"roles"), index);
        let Some(mut object) = check::object(document, *at, &path, problems) else { continue };
        let number = check::count(&mut object, document, b"number", &path, problems);
        let at = object.required(document, b"authority", &path, problems);
        let authority = value(document, at, &path.member(b"authority"), limits, None, problems);
        let period_spend = check::integer(&mut object, document, b"period_spend", &path, problems);
        let requests = check::short(&mut object, document, b"requests", &path, problems);
        let decides = check::short(&mut object, document, b"decides", &path, problems);
        object.finish(document, &path, problems);
        if let Some(number) = number
            && let Some(authority) = authority
            && let Some(period_spend) = period_spend
            && let Some(requests) = requests
            && let Some(decides) = decides
        {
            match u8::try_from(decides) {
                Ok(decides) => {
                    for previous in &roles {
                        if previous.number == number {
                            problems.push(path.member(b"number"), What::Unique);
                            break;
                        }
                    }
                    roles
                        .push(authority::Role {
                            number,
                            authority,
                            period_spend,
                            requests: authority::Requests(requests),
                            decides: authority::Proposals(decides),
                        })
                        .expect("bounded roles");
                }
                Err(_) => problems.push(path.member(b"decides"), What::Bound),
            }
        }
    }
    Some(roles.into_boxed())
}
/// An authority's form. A ceiling, given the configured models, may name fewer of them; a role's
/// names no models yet, and so narrows none.
fn value(
    document: &Document,
    at: Option<u32>,
    path: &Path,
    limits: &authority::Limits,
    configured: Option<&[Model]>,
    problems: &mut Problems,
) -> Option<authority::Authority> {
    let mut object = check::object(document, at?, path, problems)?;
    let tools = check::integer(&mut object, document, b"tools", path, problems);
    let models = match configured {
        Some(configured) => models(&mut object, document, path, configured, problems),
        None => Some(authority::Models::ALL),
    };
    let grants = grants(&mut object, document, b"grants", path, limits, problems);
    let at = object.required(document, b"delegation", path, problems);
    let delegation = delegation(document, at, &path.member(b"delegation"), limits, problems);
    let at = object.required(document, b"budget", path, problems);
    let budget = budget(document, at, &path.member(b"budget"), problems);
    let notes = check::short(&mut object, document, b"notes", path, problems);
    let note_resources = scopes(&mut object, document, path, limits, problems);
    object.finish(document, path, problems);
    let notes = notes?;
    if notes > 7 {
        problems.push(path.member(b"notes"), What::Bound);
    }
    Some(authority::Authority {
        tools: authority::Tools(tools?),
        models: models?,
        grants: grants?,
        delegation: delegation?,
        budget: budget?,
        notes: authority::Scopes(u8::try_from(notes).ok()?),
        note_resources: note_resources?,
    })
}
/// The configured models a ceiling's `models` names, one bit each in configuration's order;
/// absent, every configured model.
fn models(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    configured: &[Model],
    problems: &mut Problems,
) -> Option<authority::Models> {
    let Some(at) = object.member(document, b"models") else {
        let count = u32::try_from(configured.len()).unwrap_or(u32::MAX);
        return Some(authority::Models(u64::MAX.checked_shr(u64::BITS.saturating_sub(count)).unwrap_or(0)));
    };
    let path = path.member(b"models");
    let items = match read::list(document, at, crate::llm::MODELS) {
        Ok(items) => items,
        Err(what) => {
            problems.push(path, what);
            return None;
        }
    };
    let mut models = authority::Models::NONE;
    let mut named = true;
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path, index);
        let name = match read::text(document, *at, 4_096) {
            Ok(name) => name,
            Err(what) => {
                problems.push(path, what);
                named = false;
                continue;
            }
        };
        let mut bit = None;
        for (number, model) in configured.iter().enumerate() {
            if model.name == name {
                bit = 1_u64.checked_shl(u32::try_from(number).unwrap_or(u32::MAX));
                break;
            }
        }
        match bit {
            Some(bit) if models.0 & bit == 0 => models.0 |= bit,
            Some(_) => {
                problems.push(path, What::Unique);
                named = false;
            }
            None => {
                problems.push(path, What::Refers);
                named = false;
            }
        }
    }
    if named { Some(models) } else { None }
}
fn budget(document: &Document, at: Option<u32>, path: &Path, problems: &mut Problems) -> Option<authority::Budget> {
    let mut object = check::object(document, at?, path, problems)?;
    let spend = check::integer(&mut object, document, b"spend", path, problems);
    let deadline = check::optional(&mut object, document, b"deadline_unix_seconds", path, problems);
    object.finish(document, path, problems);
    let deadline = match deadline.ok()? {
        None => None,
        Some(seconds) => match seconds.checked_mul(1_000_000_000) {
            Some(nanos) => Some(Wall::from_nanos(nanos)),
            None => {
                problems.push(path.member(b"deadline_unix_seconds"), What::Bound);
                return None;
            }
        },
    };
    Some(authority::Budget { spend: spend?, deadline })
}
fn delegation(
    document: &Document,
    at: Option<u32>,
    path: &Path,
    limits: &authority::Limits,
    problems: &mut Problems,
) -> Option<authority::Delegation> {
    let mut object = check::object(document, at?, path, problems)?;
    let tasks = check::count(&mut object, document, b"tasks", path, problems);
    let depth = check::count(&mut object, document, b"depth", path, problems);
    let items = check::list(&mut object, document, b"kinds", limits.executors, path, problems);
    let mut kinds = List::with_capacity(limits.executors);
    if let Some(items) = items {
        for (index, at) in items.iter().enumerate() {
            let path = check::index(&path.member(b"kinds"), index);
            let Some(mut object) = check::object(document, *at, &path, problems) else { continue };
            let kind = check::text(&mut object, document, b"kind", &path, problems);
            let number = check::count(&mut object, document, b"number", &path, problems);
            object.finish(document, &path, problems);
            if let Some(kind) = kind
                && let Some(number) = number
            {
                let kind = match kind.as_ref() {
                    b"charter" => authority::Executor::Charter(number),
                    b"procedure" => authority::Executor::Procedure(number),
                    b"role" => authority::Executor::Role(number),
                    _ => {
                        problems.push(path.member(b"kind"), What::Form);
                        continue;
                    }
                };
                kinds.push(kind).expect("bounded executor count");
            }
        }
    }
    object.finish(document, path, problems);
    Some(authority::Delegation { tasks: tasks?, depth: depth?, kinds: kinds.into_boxed() })
}
fn optional_count(value: Result<Option<u64>, What>, path: &Path, problems: &mut Problems) -> Result<Option<u32>, What> {
    match value? {
        None => Ok(None),
        Some(value) => match u32::try_from(value) {
            Ok(value) => Ok(Some(value)),
            Err(_) => {
                problems.push(path.clone(), What::Bound);
                Err(What::Bound)
            }
        },
    }
}
fn grants(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    limits: &authority::Limits,
    problems: &mut Problems,
) -> Option<Box<[authority::Grant]>> {
    let items = check::list(object, document, name, limits.grants, path, problems)?;
    let mut grants = List::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path.member(name), index);
        let Some(mut object) = check::object(document, *at, &path, problems) else { continue };
        let connector = check::short(&mut object, document, b"connector", &path, problems);
        let kind = check::short(&mut object, document, b"kind", &path, problems);
        let at = object.required(document, b"pattern", &path, problems);
        let pattern = pattern(document, at, &path.member(b"pattern"), limits, problems);
        object.finish(document, &path, problems);
        if let Some(connector) = connector
            && let Some(kind) = kind
            && let Some(pattern) = pattern
        {
            grants.push(authority::Grant { connector, kind, pattern }).expect("bounded grants");
        }
    }
    Some(grants.into_boxed())
}
fn scopes(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    limits: &authority::Limits,
    problems: &mut Problems,
) -> Option<Box<[authority::ResourceScope]>> {
    let items = check::list(object, document, b"note_resources", limits.grants, path, problems)?;
    let mut scopes = List::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path.member(b"note_resources"), index);
        let Some(mut object) = check::object(document, *at, &path, problems) else { continue };
        let connector = check::short(&mut object, document, b"connector", &path, problems);
        let at = object.required(document, b"pattern", &path, problems);
        let pattern = pattern(document, at, &path.member(b"pattern"), limits, problems);
        object.finish(document, &path, problems);
        if let Some(connector) = connector
            && let Some(pattern) = pattern
        {
            scopes.push(authority::ResourceScope { connector, pattern }).expect("bounded resource scopes");
        }
    }
    Some(scopes.into_boxed())
}
fn pattern(
    document: &Document,
    at: Option<u32>,
    path: &Path,
    limits: &authority::Limits,
    problems: &mut Problems,
) -> Option<authority::Pattern> {
    let mut object = check::object(document, at?, path, problems)?;
    let items = check::list(&mut object, document, b"segments", limits.segments, path, problems);
    let mut segments = List::with_capacity(limits.segments);
    if let Some(items) = items {
        for (index, at) in items.iter().enumerate() {
            let path = check::index(&path.member(b"segments"), index);
            match read::text(document, *at, limits.segment_bytes) {
                Ok(segment) => segments.push(segment).expect("bounded segments"),
                Err(what) => problems.push(path, what),
            }
        }
    }
    let kind = check::text(&mut object, document, b"last", path, problems);
    let bytes = check::text(&mut object, document, b"value", path, problems);
    object.finish(document, path, problems);
    let bytes = bytes?;
    if bytes.len() > usize::try_from(limits.segment_bytes).expect("segment bound") {
        problems.push(path.member(b"value"), What::Bound);
    }
    let last = match kind?.as_ref() {
        b"exact" => authority::Last::Exact(bytes),
        b"open" => authority::Last::Open(bytes),
        _ => {
            problems.push(path.member(b"last"), What::Form);
            return None;
        }
    };
    Some(authority::Pattern { segments: segments.into_boxed(), last })
}
fn implications(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    limits: &authority::Limits,
    problems: &mut Problems,
) -> Option<authority::Implies> {
    let items = check::list(object, document, b"implies", limits.implications, path, problems)?;
    let mut pairs = List::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path.member(b"implies"), index);
        let Some(mut object) = check::object(document, *at, &path, problems) else { continue };
        let connector = check::short(&mut object, document, b"connector", &path, problems);
        let kind = check::short(&mut object, document, b"kind", &path, problems);
        let implies = check::short(&mut object, document, b"implies", &path, problems);
        object.finish(document, &path, problems);
        if let Some(connector) = connector
            && let Some(kind) = kind
            && let Some(implies) = implies
        {
            pairs.push(authority::Implication { connector, kind, implies }).expect("bounded implications");
        }
    }
    let implies = authority::Implies::new(pairs.into_boxed(), limits.implications);
    if implies.is_none() {
        problems.push(path.member(b"implies"), What::Bound);
    }
    implies
}
fn requirements(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    limits: &authority::Limits,
    problems: &mut Problems,
) -> Option<Box<[authority::Requirement]>> {
    let items = check::list(object, document, b"requirements", limits.requirements, path, problems)?;
    let mut requirements = List::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path.member(b"requirements"), index);
        if let Some(requirement) = requirement(document, *at, &path, limits, problems) {
            requirements.push(requirement).expect("bounded requirements");
        }
    }
    Some(requirements.into_boxed())
}
fn requirement(
    document: &Document,
    at: u32,
    path: &Path,
    limits: &authority::Limits,
    problems: &mut Problems,
) -> Option<authority::Requirement> {
    let mut object = check::object(document, at, path, problems)?;
    let connector = check::short(&mut object, document, b"connector", path, problems);
    let kind = check::short(&mut object, document, b"kind", path, problems);
    let at = object.required(document, b"pattern", path, problems);
    let pattern = pattern(document, at, &path.member(b"pattern"), limits, problems);
    let at = object.required(document, b"judge", path, problems);
    let judge = judge(document, at, &path.member(b"judge"), problems);
    let guard = check::text(&mut object, document, b"guard", path, problems);
    let mut freshness = None;
    if let Some(at) = object.member(document, b"freshness_ms") {
        match read::duration(document, at) {
            Ok(value) => freshness = Some(value),
            Err(what) => problems.push(path.member(b"freshness_ms"), what),
        }
    }
    let must_be_guarded = check::boolean(&mut object, document, b"must_be_guarded", path, problems);
    object.finish(document, path, problems);
    let guard = match guard?.as_ref() {
        b"guarded" => {
            if freshness.is_some() {
                problems.push(path.member(b"freshness_ms"), What::Unknown);
            }
            authority::Guard::Guarded
        }
        b"observed" => match freshness {
            Some(freshness) => authority::Guard::Observed { freshness },
            None => {
                problems.push(path.member(b"freshness_ms"), What::Missing);
                return None;
            }
        },
        _ => {
            problems.push(path.member(b"guard"), What::Form);
            return None;
        }
    };
    Some(authority::Requirement {
        connector: connector?,
        kind: kind?,
        pattern: pattern?,
        judge: judge?,
        guard,
        must_be_guarded: must_be_guarded?,
    })
}
fn judge(document: &Document, at: Option<u32>, path: &Path, problems: &mut Problems) -> Option<authority::Judge> {
    let mut object = check::object(document, at?, path, problems)?;
    let connector = check::short(&mut object, document, b"connector", path, problems);
    let requirement = check::short(&mut object, document, b"requirement", path, problems);
    let parameters = check::count(&mut object, document, b"parameters", path, problems);
    object.finish(document, path, problems);
    Some(authority::Judge { connector: connector?, requirement: requirement?, parameters: parameters? })
}
