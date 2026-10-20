//! Startup for the reference binary (protocol/service.md, 7; configuration.md, 7–8).
use jig_protocol_config::{self as config, read};
use jig_test_service::{self as service, Files, Seeds};
use skein_io::kernel::{Exit, Fd};
use skein_lib::List;
use skein_shell::{Clock, Config, Kernel};
use std::{
    ffi::OsString,
    net::ToSocketAddrs,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

mod bundle;
mod host;
pub use host::Host;
const CONFIG_BYTES: u32 = 1 << 20;
const USAGE: &str = "usage: jig-test serve CONFIG.json | jig-test agent AGENT.json";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Serve { configuration: PathBuf },
    Agent { configuration: PathBuf },
}

pub fn configure(mut args: impl Iterator<Item = OsString>) -> Result<Command, String> {
    let command = args.next().ok_or(USAGE)?;
    let configuration = args.next().ok_or(USAGE)?.into();
    if args.next().is_some() {
        return Err(USAGE.into());
    }
    if command == "serve" {
        Ok(Command::Serve { configuration })
    } else if command == "agent" {
        Ok(Command::Agent { configuration })
    } else {
        Err(USAGE.into())
    }
}

pub fn run(command: Command) -> Result<u8, String> {
    let exit = match command {
        Command::Serve { configuration } => serve(&configuration)?,
        Command::Agent { configuration } => smith_agent_shell::run(&configuration, Box::new(std::io::stderr()))?,
    };
    Ok(match exit {
        Exit::Code(code) => code,
        Exit::Signal(_) => 1,
    })
}
fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}
fn member(document: &skein_json::Document, at: u32, key: &[u8]) -> Result<u32, String> {
    read::Object::new(document, at)
        .map_err(|why| format!("configuration: {why:?}"))?
        .member(document, key)
        .ok_or_else(|| format!("configuration lacks {}", String::from_utf8_lossy(key)))
}
fn read_configuration(path: &Path) -> Result<Box<[u8]>, String> {
    skein_shell::read_file(path, CONFIG_BYTES).map_err(|why| format!("configuration {}: {why:?}", path.display()))
}
fn read_files(document: &[u8], bounds: &[config::File], user: u32) -> Result<Files, String> {
    let names = service::named(document).map_err(|why| format!("configuration: {why:?}"))?;
    let mut entries = List::with_capacity(names.len());
    for name in names.into_boxed() {
        let max = bounds.iter().filter(|file| file.path == name).map(|file| file.max).min().unwrap_or(CONFIG_BYTES);
        let value = skein_shell::read_private_file(path(&name), max, user)
            .map_err(|why| format!("startup file {}: {why:?}", path(&name).display()))?;
        entries.push((name, value)).expect("named files capacity");
    }
    Ok(Files { entries })
}
fn agent(worker: &config::Worker) -> Result<service::Agent, String> {
    let configuration = smith_agent_shell::config::read(path(&worker.agent.configuration))?;
    let limits = &configuration.service.limits;
    let host_limits = smith_local_service::profile::host_limits(&configuration.profile, limits)
        .map_err(|why| format!("agent host: {why:?}"))?;
    let channel = limits.channel;
    let mut protocol = smith_host_protocol::Limits {
        bodies: channel.bodies,
        charter: channel.charter,
        transcript: channel.transcript,
        endpoints: channel.endpoints,
        channel: channel.channel,
        start_bytes: channel.start_bytes,
        calls: host_limits.calls,
    };
    protocol.channel.output_bytes = smith_host_protocol::output_bytes(&protocol).ok_or("agent output bound")?;
    let mut models = Vec::new();
    for endpoint in &configuration.declarations.endpoints {
        for model in &endpoint.models {
            models.push(jig_domain_host::ModelBound {
                name: model.name.clone(),
                window: model.window,
                output: model.output,
            });
        }
    }
    let policy = configuration.profile.policy;
    Ok(service::Agent {
        endpoints: configuration.service.channel_endpoints,
        limits: protocol,
        host_limits,
        bounds: jig_domain_host::Bounds {
            models: models.into_boxed_slice(),
            turns: policy.max_turns,
            spend: policy.max_spend,
            time: policy.max_time,
            waiting: policy.max_waiting,
            checks: policy.check_timeout,
            message_bytes: limits.domain.run.message_bytes,
            inbox: policy.inbox,
            unacknowledged: policy.unacknowledged,
        },
    })
}
fn account_transport(
    configured: &config::Transport,
    trust_roots: &mut u32,
    trust_root_bytes: &mut u32,
) -> Result<skein_oauth_accounts::Transport, String> {
    Ok(match configured {
        config::Transport::Plaintext => skein_oauth_accounts::Transport::Plaintext,
        config::Transport::Tls { server_name, trust } => {
            let source = match trust {
                Some(file) => skein_shell::Trust::Der(path(file).into()),
                None => {
                    skein_shell::Trust::Machine { ssl_cert_file: std::env::var_os("SSL_CERT_FILE").map(PathBuf::from) }
                }
            };
            let roots = skein_shell::read_trust_roots(
                &source,
                &skein_shell::RootBounds { bundle_bytes: CONFIG_BYTES, certificates: 1024, certificate_bytes: 65_536 },
            )
            .map_err(|why| format!("account trust: {why:?}"))?;
            *trust_roots = trust_roots
                .checked_add(u32::try_from(roots.certificates.len()).expect("roots count"))
                .ok_or("trust count")?;
            for root in &roots.certificates {
                *trust_root_bytes = (*trust_root_bytes).max(u32::try_from(root.len()).expect("root size"));
            }
            let (trust, parsed) = skein_tls::Config::from_der(&roots.certificates, &[])
                .map_err(|why| format!("account trust: {why:?}"))?;
            eprintln!(
                "jig-test: trust roots taken {}, skipped {}, malformed {}, other {}",
                roots.certificates.len(),
                parsed.skipped,
                roots.malformed,
                roots.other_blocks
            );
            let name = std::str::from_utf8(server_name).map_err(|why| why.to_string())?;
            let server_name = skein_tls::Name::new(name).ok_or("account server name")?;
            skein_oauth_accounts::Transport::Tls { server_name, trust }
        }
    })
}
fn account_origin(uri: &[u8]) -> Result<Box<[u8]>, String> {
    let uri = std::str::from_utf8(uri).map_err(|why| why.to_string())?;
    let (scheme, rest) = uri.split_once("://").ok_or("account sign-in origin requires a URL")?;
    let authority = rest.split('/').next().ok_or("account sign-in origin requires an authority")?;
    Ok(format!("{scheme}://{authority}").into_bytes().into_boxed_slice())
}
fn account_endpoint(
    uri: &[u8],
    configured: &config::Transport,
    trust_roots: &mut u32,
    trust_root_bytes: &mut u32,
) -> Result<skein_oauth_accounts::Endpoint, String> {
    let uri = std::str::from_utf8(uri).map_err(|why| why.to_string())?;
    let (scheme, rest) = uri.split_once("://").ok_or("account sign-in endpoint requires an HTTP URL")?;
    let authority = rest.split('/').next().ok_or("account sign-in endpoint requires an authority")?;
    let port = match scheme {
        "http" => 80,
        "https" => 443,
        _ => return Err("account sign-in endpoint requires HTTP or HTTPS".into()),
    };
    let address = if let Ok(address) = authority.parse::<std::net::SocketAddr>() {
        address
    } else {
        let (host, port) = match authority.rsplit_once(':') {
            Some((host, port)) => (host, port.parse::<u16>().map_err(|why| why.to_string())?),
            None => (authority, port),
        };
        (host, port)
            .to_socket_addrs()
            .map_err(|why| why.to_string())?
            .next()
            .ok_or("account sign-in endpoint has no address")?
    };
    if matches!(configured, config::Transport::Plaintext) && (scheme != "http" || !address.ip().is_loopback()) {
        return Err("plaintext account sign-in endpoint must be HTTP on loopback".into());
    }
    if matches!(configured, config::Transport::Tls { .. }) && scheme != "https" {
        return Err("TLS account sign-in endpoint must use HTTPS".into());
    }
    Ok(skein_oauth_accounts::Endpoint {
        address,
        transport: account_transport(configured, trust_roots, trust_root_bytes)?,
    })
}
fn accounts(engine: &config::Engine) -> Result<service::AccountsRuntime, String> {
    let mut issuers = List::with_capacity(u32::try_from(engine.llm.accounts.len()).expect("account count"));
    let mut trust_roots = 0_u32;
    let mut trust_root_bytes = 0_u32;
    for account in &engine.llm.accounts {
        if let config::Source::SignIn(source) = &account.source {
            let address = match &source.endpoint {
                config::Address::Addr(address) => {
                    address.socket().ok_or("an account issuer requires a socket address")?
                }
                config::Address::Host { name, port } => {
                    let name = std::str::from_utf8(name).map_err(|why| why.to_string())?;
                    (name, *port)
                        .to_socket_addrs()
                        .map_err(|why| why.to_string())?
                        .next()
                        .ok_or("account endpoint has no address")?
                }
            };
            let transport = account_transport(&source.transport, &mut trust_roots, &mut trust_root_bytes)?;
            issuers
                .push((account.number, skein_oauth_accounts::Endpoint { address, transport }))
                .expect("configured account count");
        }
    }
    let mut providers = List::with_capacity(u32::try_from(engine.llm.providers.len()).expect("provider count"));
    for provider in &engine.llm.providers {
        if let Some(registration) = &provider.sign_in {
            let endpoint = account_endpoint(
                &registration.token,
                &registration.endpoint_transport,
                &mut trust_roots,
                &mut trust_root_bytes,
            )?;
            let mut origins = List::with_capacity(3);
            for uri in [Some(&registration.authorization), Some(&registration.keys), registration.revocation.as_ref()]
                .into_iter()
                .flatten()
            {
                let endpoint =
                    account_endpoint(uri, &registration.endpoint_transport, &mut trust_roots, &mut trust_root_bytes)?;
                origins
                    .push(skein_oauth_accounts::Origin { uri: account_origin(uri)?, endpoint })
                    .expect("three account sign-in origins");
            }
            providers.push((provider.number, endpoint, origins)).expect("configured provider count");
        }
    }
    Ok(service::AccountsRuntime { issuers, providers, trust_roots, trust_root_bytes })
}
fn prepare(bytes: &[u8], owned: &mut Vec<Fd>) -> Result<service::Service, String> {
    let document =
        config::document(bytes, &config::DocumentLimits { bytes: CONFIG_BYTES, tokens: CONFIG_BYTES, depth: 32 })
            .map_err(|why| format!("configuration: {why:?}"))?;
    let mut whole = read::Object::new(&document, 0).map_err(|why| format!("configuration: {why:?}"))?;
    let engine = whole
        .member(&document, b"engine")
        .map(|at| config::engine(&document, at))
        .transpose()
        .map_err(|why| format!("engine: {why:?}"))?;
    let workers = match whole.member(&document, b"workers") {
        Some(at) => read::list(&document, at, document.len()).map_err(|why| format!("workers: {why:?}"))?,
        None => List::with_capacity(0),
    };
    let mut specs = Vec::new();
    let mut bounds = engine.as_ref().map(config::files).unwrap_or_default().into_vec();
    for at in &workers {
        let worker = config::worker(&document, *at).map_err(|why| format!("worker: {why:?}"))?;
        bounds.extend(config::worker_files(&worker).into_vec());
        specs.push(worker);
    }
    let user = skein_shell::effective_user();
    let files = read_files(bytes, &bounds, user)?;
    let agent = specs.first().map(agent).transpose()?;
    if specs
        .iter()
        .any(|worker| worker.agent.configuration != specs.first().expect("nonempty workers").agent.configuration)
    {
        return Err("one-process workers must share their agent configuration".into());
    }
    let mut roots = List::with_capacity(workers.len());
    if !specs.is_empty() {
        let application = member(&document, 0, b"application")?;
        let at = member(&document, application, b"workers")?;
        let paths = read::list(&document, at, workers.len()).map_err(|why| format!("worker roots: {why:?}"))?;
        for at in &paths {
            let at = member(&document, *at, b"root")?;
            let name = read::text(&document, at, 4096).map_err(|why| format!("worker root: {why:?}"))?;
            let root = skein_shell::open_root(path(&name)).map_err(|errno| format!("worker root: errno {errno}"))?;
            owned.push(root);
            roots.push(root).expect("worker count");
        }
    }
    let mut accounts_enabled = false;
    if let Some(at) = whole.member(&document, b"service") {
        let at = member(&document, at, b"components")?;
        for at in &read::list(&document, at, document.len()).map_err(|why| format!("components: {why:?}"))? {
            accounts_enabled |=
                read::text(&document, *at, 32).map_err(|why| format!("component: {why:?}"))?.as_ref() == b"accounts";
        }
    }
    let runtime = if accounts_enabled { engine.as_ref().map(accounts).transpose()? } else { None };
    let seed = skein_shell::seed().map_err(|errno| format!("random seed: errno {errno}"))?;
    let crypto = skein_shell::crypto_seed().map_err(|errno| format!("cryptographic seed: errno {errno}"))?;
    let mut deployment = [0_u8; 16];
    let drawn = skein_shell::crypto_seed().map_err(|errno| format!("deployment id: errno {errno}"))?;
    deployment.copy_from_slice(drawn.expose().get(..16).expect("32-byte seed"));
    // Drawn at every start; the service keeps it at the first and restores
    // the kept one at every later start (configuration.md 8).
    let mut installation = [0_u8; 16];
    let drawn = skein_shell::crypto_seed().map_err(|errno| format!("installation id: errno {errno}"))?;
    installation.copy_from_slice(drawn.expose().get(..16).expect("32-byte seed"));
    let seeds = Seeds { deployment, installation, seed, crypto };
    let mut service =
        service::startup(bytes, &files, agent.as_ref(), &seeds, runtime, Some(service::WorkerRuntime { roots }))
            .map_err(|why| format!("startup: {why:?}"))?;
    if let Some(directory) = service.bundle_directory() {
        let bundle = bundle::read(path(directory))?;
        service.serve(bundle).map_err(|why| format!("serve bundle: {why:?}"))?;
    }
    if let Some(engine) = engine {
        let root = skein_shell::make_root(path(&engine.deployment.directory), 0o700)
            .map_err(|errno| format!("deployment directory: errno {errno}"))?;
        owned.push(root);
        service.adopt_store(root, user).map_err(|_| "store refused its deployment root")?;
    }
    Ok(service)
}
fn serve(path: &Path) -> Result<Exit, String> {
    let bytes = read_configuration(path)?;
    let mut owned = Vec::new();
    let prepared = prepare(&bytes, &mut owned);
    let mut service = match prepared {
        Ok(service) => service,
        Err(why) => {
            for fd in owned {
                skein_shell::close_keeper_fd(fd);
            }
            return Err(why);
        }
    };
    let opened = skein_shell::open_termination_signals()
        .map_err(|errno| format!("signals: errno {errno}"))
        .and_then(|signals| {
            owned.push(signals);
            service.adopt_signals(signals).map_err(|_| "signals refused".into())
        })
        .and_then(|()| Kernel::open(Config { operations: service.operations() }).map_err(|why| why.to_string()));
    let mut kernel = match opened {
        Ok(kernel) => kernel,
        Err(why) => {
            for fd in owned {
                skein_shell::close_keeper_fd(fd);
            }
            return Err(why);
        }
    };
    Ok(skein_shell::drive(&mut kernel, &Clock::new(), &mut Host::new(service)))
}

#[cfg(test)]
mod tests;
