//! Parse and check everything before constructing a running service (service.md, 3).
use crate::{Agent, Engine, Files, Limits, Seeds, Service, limits, system::System};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, Problems, Profile, Segment, What, read};
use jig_test_connector as connector;
use jig_test_domain as root;
use skein_json::Document;
use skein_lib::{List, bytes};

type Initial = Box<[(connector::Path, u64)]>;

#[derive(Debug)]
struct Parsed {
    engine: config::Engine,
    limits: Limits,
    first: connector::Config,
    second: connector::Config,
    initial: [Initial; 2],
    credential: Box<[u8]>,
    accounts: bool,
    workers: bool,
}

fn problem(path: Path, what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(path, what);
    problems
}
fn engine_path(path: &Path) -> Path {
    let mut complete = Path::root().member(b"engine");
    for segment in &path.segments {
        complete = complete.child(segment.clone());
    }
    complete
}

fn object(document: &Document, at: u32, path: &Path) -> Result<read::Object, Problems> {
    match read::Object::new(document, at) {
        Ok(value) => Ok(value),
        Err(what) => Err(problem(path.clone(), what)),
    }
}

fn text(document: &Document, at: u32, max: u32, path: &Path) -> Result<Box<[u8]>, Problems> {
    match read::text(document, at, max) {
        Ok(value) => Ok(value),
        Err(what) => Err(problem(path.clone(), what)),
    }
}
fn list(document: &Document, at: u32, max: u32, path: &Path) -> Result<List<u32>, Problems> {
    match read::list(document, at, max) {
        Ok(value) => Ok(value),
        Err(what) => Err(problem(path.clone(), what)),
    }
}
fn integer(document: &Document, at: u32, path: &Path) -> Result<u64, Problems> {
    match read::integer(document, at, u64::MAX) {
        Ok(value) => Ok(value),
        Err(what) => Err(problem(path.clone(), what)),
    }
}

fn components(
    document: &Document,
    whole: &mut read::Object,
    problems: &mut Problems,
) -> Result<(bool, bool), Problems> {
    let mut accounts = false;
    let mut workers = false;
    if let Some(at) = whole.member(document, b"service") {
        let here = Path::root().member(b"service");
        let mut service = object(document, at, &here)?;
        if let Some(at) = service.member(document, b"components") {
            let items = list(document, at, document.len(), &here.member(b"components"))?;
            for (index, at) in items.iter().enumerate() {
                let here = indexed(&here.member(b"components"), index);
                let name = text(document, *at, 32, &here)?;
                let what = match name.as_ref() {
                    b"accounts" if !accounts => {
                        accounts = true;
                        continue;
                    }
                    b"workers" if !workers => {
                        workers = true;
                        continue;
                    }
                    b"accounts" | b"workers" => What::Duplicate,
                    b"clients" => What::Refers,
                    _ => What::Unknown,
                };
                problems.push(here, what);
            }
        }
        service.finish(document, &here, problems);
    }
    Ok((accounts, workers))
}

fn parse(input: &[u8]) -> Result<Parsed, Problems> {
    let length = u32::try_from(input.len()).or(Err(problem(Path::root(), What::Bound)))?;
    let bounds = config::DocumentLimits { bytes: length, tokens: length, depth: 32 };
    let document = match config::document(input, &bounds) {
        Ok(value) => value,
        Err(found) => return Err(problem(found.path, found.what)),
    };
    let mut problems = Problems::empty();
    let path = Path::root();
    let mut whole = object(&document, 0, &path)?;
    let at = whole.required(&document, b"engine", &path, &mut problems);
    let application = whole.required(&document, b"application", &path, &mut problems);
    if let Some(at) = whole.member(&document, b"workers") {
        let items = list(&document, at, document.len(), &path.member(b"workers"))?;
        if !items.is_empty() {
            problems.push(path.member(b"workers"), What::Refers);
        }
    }
    let (accounts, workers) = components(&document, &mut whole, &mut problems)?;
    whole.finish(&document, &path, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let at = at.expect("required engine checked");
    let engine = match config::engine(&document, at) {
        Ok(value) => value,
        Err(found) => {
            for found in &found.problems {
                problems.push(engine_path(&found.path), found.what);
            }
            problems.past = problems.past.saturating_add(found.past);
            return Err(problems);
        }
    };
    let mut engine_object = object(&document, at, &path.member(b"engine"))?;
    let at = engine_object.member(&document, b"limits").expect("engine parser checked limits");
    let mut profile = object(&document, at, &path)?;
    let at = profile.member(&document, b"profile").expect("engine parser checked profile");
    let profile = text(&document, at, 16, &path)?;
    let profile = match profile.as_ref() {
        b"small" => Profile::Small,
        b"standard" => Profile::Standard,
        _ => return Err(problem(path, What::Form)),
    };
    let limits = limits::for_profile(profile, &engine.limits);
    let path = Path::root().member(b"application");
    let mut application = object(&document, application.expect("required application checked"), &path)?;
    let at = application.required(&document, b"credential", &path, &mut problems);
    let credential = match at {
        Some(at) => text(&document, at, length, &path.member(b"credential"))?,
        None => Box::default(),
    };
    let at = application.required(&document, b"connectors", &path, &mut problems);
    let items = match at {
        Some(at) => list(&document, at, 2, &path.member(b"connectors"))?,
        None => List::with_capacity(0),
    };
    if items.len() != 2 {
        problems.push(path.member(b"connectors"), What::Bound);
    }
    application.finish(&document, &path, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let first = connector_config(&document, *items.get(0).expect("two connectors"), 0, &limits.root.connector)?;
    let second = connector_config(&document, *items.get(1).expect("two connectors"), 1, &limits.root.connector)?;
    Ok(Parsed {
        engine,
        limits,
        first: first.0,
        second: second.0,
        initial: [first.1, second.1],
        credential,
        accounts,
        workers,
    })
}

fn indexed(path: &Path, index: usize) -> Path {
    path.child(Segment::Index(u32::try_from(index).expect("bounded list")))
}
fn resource(
    document: &Document,
    at: u32,
    path: &Path,
    limits: &connector::Limits,
) -> Result<connector::Path, Problems> {
    let items = list(document, at, limits.path_segments, path)?;
    let mut parts = List::with_capacity(limits.path_segments);
    for at in &items {
        parts.push(text(document, *at, limits.segment_bytes, path)?).expect("bounded path");
    }
    connector::Path::new(parts, limits.segment_bytes).ok_or(problem(path.clone(), What::Bound))
}

fn connector_config(
    document: &Document,
    at: u32,
    index: usize,
    limits: &connector::Limits,
) -> Result<(connector::Config, Initial), Problems> {
    let path = indexed(&Path::root().member(b"application").member(b"connectors"), index);
    let mut problems = Problems::empty();
    let mut fields = object(document, at, &path)?;
    let prefix = fields.required(document, b"prefix", &path, &mut problems);
    let resources = fields.required(document, b"resources", &path, &mut problems);
    fields.finish(document, &path, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let prefix = resource(document, prefix.expect("prefix checked"), &path.member(b"prefix"), limits)?;
    let items = list(document, resources.expect("resources checked"), limits.resources, &path.member(b"resources"))?;
    let mut resources = List::<connector::ResourceSpec>::with_capacity(limits.resources);
    let mut initial = List::with_capacity(limits.resources);
    for (index, at) in items.iter().enumerate() {
        let here = indexed(&path.member(b"resources"), index);
        let mut row = object(document, *at, &here)?;
        let name = row.required(document, b"path", &here, &mut problems);
        let state = row.required(document, b"state", &here, &mut problems);
        row.finish(document, &here, &mut problems);
        if !problems.is_empty() {
            return Err(problems);
        }
        let name = resource(document, name.expect("path checked"), &here.member(b"path"), limits)?;
        let state = integer(document, state.expect("state checked"), &here.member(b"state"))?;
        if !name.under(&prefix) {
            return Err(problem(here.member(b"path"), What::Refers));
        }
        for previous in &resources {
            if previous.path == name {
                return Err(problem(here.member(b"path"), What::Duplicate));
            }
        }
        initial.push((name.clone(), state)).expect("resource count checked");
        resources
            .push(connector::ResourceSpec { path: name, credential: None, hold: connector::Hold::None, writable: true })
            .expect("resource count checked");
    }
    let config = connector::Config {
        deployment: [0; 16],
        prefix,
        resources: resources.into_boxed(),
        pools: Box::default(),
        topics: Box::default(),
        kinds: Box::new([connector::KindSpec {
            kind: 1,
            form: connector::Form::Set,
            recovery: connector::Recovery::Keyed,
            price: None,
        }]),
        requirements: Box::default(),
        procedures: Box::default(),
    };
    Ok((config, initial.into_boxed()))
}

pub fn named(document: &[u8]) -> Result<List<Box<[u8]>>, Problems> {
    let parsed = parse(document)?;
    let files = config::files(&parsed.engine);
    let mut names =
        List::with_capacity(u32::try_from(files.len()).expect("file count").checked_add(1).expect("credential file"));
    for file in files {
        if !names.as_slice().contains(&file.path) {
            names.push(file.path).expect("file count");
        }
    }
    if !names.as_slice().contains(&parsed.credential) {
        names.push(parsed.credential).expect("credential file");
    }
    Ok(names)
}

fn check_files(parsed: &Parsed, files: &Files) -> Result<(), Problems> {
    let mut problems = Problems::empty();
    for file in config::files(&parsed.engine) {
        let mut found = None;
        for (name, contents) in &files.entries {
            if *name == file.path {
                found = Some(contents.expose());
            }
        }
        match found {
            Some(contents) if contents.len() <= usize::try_from(file.max).expect("file bound") => {}
            Some(_) => problems.push(engine_path(&file.at), What::Bound),
            None => problems.push(engine_path(&file.at), What::Unreadable),
        }
    }
    let mut credential = false;
    for (name, _) in &files.entries {
        credential |= *name == parsed.credential;
    }
    if !credential {
        problems.push(Path::root().member(b"application").member(b"credential"), What::Unreadable);
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(())
}

pub fn startup(
    document: &[u8],
    files: &Files,
    _agent: Option<&Agent>,
    seeds: &Seeds,
    runtime: Option<crate::AccountsRuntime>,
) -> Result<Service, Problems> {
    let mut parsed = parse(document)?;
    check_files(&parsed, files)?;
    for charter in &mut parsed.engine.charters {
        for (name, contents) in &files.entries {
            if *name == charter.policy.instructions {
                charter.policy.instructions = bytes::copy_of(contents.expose());
                break;
            }
        }
    }
    let trust = match &runtime {
        Some(runtime) if runtime.trust_roots > 0 => {
            skein_tls::roots_worst_case(runtime.trust_roots, runtime.trust_root_bytes)
        }
        Some(_) | None => Some(0),
    }
    .ok_or(problem(Path::root().member(b"engine").member(b"memory_bytes"), What::Bound))?;
    let configured = crate::workers::configured_bytes(&parsed.engine)
        .ok_or(problem(Path::root().member(b"engine").member(b"memory_bytes"), What::Bound))?;
    let bytes = limits::worst_case(&parsed.limits)
        .ok_or(problem(Path::root().member(b"engine").member(b"memory_bytes"), What::Bound))?
        .checked_add(trust)
        .ok_or(problem(Path::root().member(b"engine").member(b"memory_bytes"), What::Bound))?;
    let bytes = bytes
        .checked_add(configured)
        .ok_or(problem(Path::root().member(b"engine").member(b"memory_bytes"), What::Bound))?;
    if let Err(what) = config::fits(parsed.engine.memory, &[config::Share { name: b"service", bytes }]) {
        return Err(problem(Path::root().member(b"engine").member(b"memory_bytes"), what));
    }
    parsed.first.deployment = seeds.deployment;
    parsed.second.deployment = seeds.deployment;
    let max = root::max_out(&parsed.limits.root);
    let accounts = if parsed.accounts {
        Some(crate::accounts::Accounts::new(
            &parsed.engine,
            files,
            seeds,
            runtime,
            max.checked_mul(6).expect("account routing room"),
        )?)
    } else {
        None
    };
    let workers = if parsed.workers {
        Some(crate::workers::Workers::new(
            &parsed.engine,
            files,
            seeds.deployment,
            &parsed.limits.workers,
            max.checked_mul(2)
                .expect("engine routes")
                .checked_mul(jig_protocol_engine_workers::MAX_OUT_DOWN)
                .expect("worker routing room"),
        )?)
    } else {
        None
    };
    let mut application = skein_lib::Map::with_capacity(parsed.limits.root.core.accounts.credentials);
    let credential = files.contents(&parsed.credential).expect("credential file checked");
    if credential.len() > usize::try_from(parsed.limits.root.connector.value_bytes).expect("credential bound") {
        return Err(problem(Path::root().member(b"application").member(b"credential"), What::Bound));
    }
    application.insert(1, config::secret(credential.into())).expect("one application credential");
    let values = crate::values::Values {
        application,
        accounts: jig_protocol_accounts::table::Table::new(
            parsed.limits.root.core.accounts.accounts.checked_add(1).expect("numbered accounts"),
            parsed.limits.accounts.client.document.token_bytes,
        ),
    };
    let core = config::core(&parsed.engine, seeds.deployment, seeds.seed);
    let root = root::Config { core, first: parsed.first, second: parsed.second };
    let connector = parsed.limits.root.connector;
    let mut systems =
        [System::new(connector.resources, connector.made), System::new(connector.resources, connector.made)];
    for (index, initial) in parsed.initial.into_iter().enumerate() {
        let system = systems.get_mut(index).expect("two systems");
        for (resource, state) in initial {
            system.seed(resource, state).expect("configuration checked resource count");
        }
    }
    let engine = Engine::new(root, &parsed.limits.root, systems, accounts, workers, values);
    Ok(Service::new(engine, parsed.limits.io, bytes))
}
