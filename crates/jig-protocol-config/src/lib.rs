//! Strict configuration values and operator problems (protocol/configuration.md, 1 and 10).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod check;
pub mod clients;
mod files;
pub mod hosts;
pub mod llm;
pub use clients::Clients;
pub use files::{File, files};
pub use hosts::{Hosts, WorkerName};
pub use llm::{Account, Endpoint, Header, Llm, SignIn, Source, Transport};
pub mod address;
mod document;
mod fits;
pub mod overrides;
pub mod profile;
pub use fits::{Share, fits};
pub use profile::{Limits, Link, Profile, profile};
pub mod forms;
mod problem;
pub mod read;
pub use address::Address;
pub use document::{DocumentLimits, document};
pub use forms::{HandedIn, digest, handed_in, secret};
pub use problem::{PROBLEMS, Path, Problem, Problems, Segment, What};
/// The engine's protocol configuration (configuration.md, 3).
#[derive(Debug)]
pub struct Engine {
    pub llm: Llm,
    pub hosts: Hosts,
    pub clients: Clients,
    pub limits: Limits,
}
/// Parse jig's engine member before startup constructs any component.
pub fn engine(document: &skein_json::Document, at: u32) -> Result<Engine, Problems> {
    let path = Path::root();
    let mut problems = Problems::empty();
    let Some(mut object) = check::object(document, at, &path, &mut problems) else {
        return Err(problems);
    };
    let mut limits = profile(Profile::Small);
    if let Some(at) = object.required(document, b"limits", &path, &mut problems) {
        match overrides::parse(document, at, &path.member(b"limits")) {
            Ok(value) => limits = value,
            Err(found) => {
                for problem in &found.problems {
                    problems.push(problem.path.clone(), problem.what);
                }
                problems.past = problems.past.saturating_add(found.past);
            }
        }
    }
    let mut llm = None;
    if let Some(at) = object.required(document, b"llm", &path, &mut problems) {
        llm = llm::parse(document, at, &path.member(b"llm"), &limits, &mut problems);
    }
    let mut hosts = None;
    if let Some(at) = object.required(document, b"hosts", &path, &mut problems) {
        hosts = hosts::parse(document, at, &path.member(b"hosts"), &limits, &mut problems);
    }
    let mut clients = None;
    if let Some(at) = object.required(document, b"clients", &path, &mut problems) {
        clients = clients::parse(document, at, &path.member(b"clients"), &mut problems);
    }
    object.finish(document, &path, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(Engine {
        llm: llm.expect("required member checked"),
        hosts: hosts.expect("required member checked"),
        clients: clients.expect("required member checked"),
        limits,
    })
}
#[cfg(test)]
mod tests;
