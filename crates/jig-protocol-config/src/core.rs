//! Core startup values from the checked engine part (configuration.md, 3 and 8).
use crate::{Address, Engine, ProviderKind};
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_core as domain;
use skein_io::Addr;
use skein_lib::{List, Map, Queue};

/// Build the core's values after startup reads `files` and replaces each charter's
/// `policy.instructions` file name with its bounded contents. No file is read here.
#[must_use]
pub fn core(engine: &Engine, deployment: [u8; 16], seed: u64) -> domain::Config {
    let limits = &engine.limits.core;
    let mut authority = authority::Domain::new(engine.rules.clone(), limits.authority).expect("checked rules");
    let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
    let mut projects = List::with_capacity(limits.authority.projects);
    let mut names = Map::with_capacity(limits.authority.projects);
    let mut template_overrides = Map::with_capacity(limits.authority.projects);
    let mut permission_roles = Map::with_capacity(limits.authority.projects);
    for project in &engine.projects {
        authority::step(
            &mut authority,
            authority::Event::Policy { project: project.number, policy: project.policy.clone() },
            &mut facts,
        );
        match facts.pop().expect("one checked policy admission") {
            authority::PolicyFact::Added { .. } | authority::PolicyFact::Changed { .. } => {}
            authority::PolicyFact::Dropped { .. } | authority::PolicyFact::Refused { .. } => {
                unreachable!("checked project policies")
            }
        }
        template_overrides.insert(project.number, project.templates.clone()).expect("checked projects");
        projects.push(project.number).expect("checked project count");
        names.insert(project.number, project.name.clone()).expect("checked project count");
        let previous = permission_roles.insert(project.number, project.roles.clone()).expect("checked project count");
        assert!(previous.is_none(), "checked unique project");
    }
    let mut deployment_provider = None;
    for provider in &engine.providers {
        match provider.kind {
            ProviderKind::Deployment => deployment_provider = Some(provider.number),
            ProviderKind::Link | ProviderKind::Application => {}
        }
    }
    let mut llm_providers =
        List::with_capacity(u32::try_from(engine.llm.providers.len()).expect("checked providers fit u32"));
    for provider in &engine.llm.providers {
        let offers = accounts::Offers {
            sign_in: provider.sign_in.is_some(),
            key: provider.keys,
            programs: provider.programs.clone(),
        };
        let value = accounts::Provider {
            number: provider.number,
            name: provider.name.clone(),
            signed_in_plan: provider.signed_in_plan,
            manage_usage: provider.manage_usage.clone(),
            models: provider.models.clone(),
            offers,
        };
        llm_providers.push(value).expect("one per configured provider");
    }
    let mut workers_in_process = false;
    for address in &engine.hosts.listener {
        match address {
            Address::Addr(Addr::InProcess(_)) => workers_in_process = true,
            Address::Addr(Addr::Ip(_)) | Address::Host { .. } => {}
        }
    }
    domain::Config {
        worker_names: {
            let mut names = List::with_capacity(engine.limits.link.names);
            for worker in &engine.hosts.workers {
                names.push(worker.name.clone()).expect("checked worker count");
            }
            names.into_boxed()
        },
        deployment,
        seed: engine.deployment.seed.unwrap_or(seed),
        workers_in_process,
        hub_turns: engine.limits.hub.turns,
        owners: engine.owners.clone(),
        grants: engine.acts.clone(),
        authority,
        settings: domain::Settings {
            templates: engine.templates.clone(),
            deployment_provider: deployment_provider.expect("one checked deployment provider"),
            models: engine.llm.models.clone(),
            charters: engine.charters.clone(),
            resume_bytes: limits.resume_bytes,
            tools: domain::ToolFamilies::JIG,
            key_bytes: engine
                .limits
                .driver
                .client
                .document
                .token_bytes
                .min(engine.limits.clients.wire.keyed_value_some),
            llm_providers: accounts::Providers { providers: llm_providers.into_boxed() },
        },
        projects,
        names,
        permission_roles,
        template_overrides,
        connectors: engine.connectors.clone(),
    }
}
