//! Core startup values from the checked engine part (configuration.md, 3 and 8).
use crate::{Address, Engine, ProviderKind};
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
        projects.push(project.number).expect("checked project count");
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
    let mut workers_in_process = false;
    for address in &engine.hosts.listener {
        match address {
            Address::Addr(Addr::InProcess(_)) => workers_in_process = true,
            Address::Addr(Addr::Ip(_)) | Address::Host { .. } => {}
        }
    }
    let (chat, close_grace) = crate::templates::chat(&engine.templates).expect("checked chat template");
    domain::Config {
        deployment,
        seed: engine.deployment.seed.unwrap_or(seed),
        workers_in_process,
        hub_turns: engine.limits.hub.turns,
        owners: engine.owners.clone(),
        authority,
        settings: domain::Settings {
            close_grace,
            deployment_provider: deployment_provider.expect("one checked deployment provider"),
            charters: engine.charters.clone(),
            chat,
            resume_bytes: limits.resume_bytes,
            tools: domain::ToolFamilies::JIG,
        },
        projects,
        permission_roles,
        connectors: engine.connectors.clone(),
    }
}
