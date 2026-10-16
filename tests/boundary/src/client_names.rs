//! The client crate prefix from README.md, section 15.

use crate::manifest::{CrateManifest, manifests};
use std::{collections::BTreeSet, path::Path};

const KINDS: [&str; 9] = ["domain", "protocol", "engine", "test", "fake", "scripted", "conformance", "shell", "view"];
const SHARED: [&str; 3] = ["-client-protocol-wire", "-client-test-protocol-wire", "-client-test-records"];
const UI_KIT: [&str; 5] =
    ["skein-ui", "skein-ui-text", "skein-ui-web", "skein-fake-person", "skein-fake-person-browser"];

/// Client-side crates outside the project's client prefix, except `allowed`; sorted.
#[must_use]
pub fn breaches(root: &Path, project: &str, allowed: &[&str]) -> Vec<String> {
    of(&manifests(root), project, allowed)
}

pub(crate) fn of(crates: &[CrateManifest], project: &str, allowed: &[&str]) -> Vec<String> {
    let mut side: BTreeSet<&str> = crates
        .iter()
        .filter_map(|item| {
            let name = item.name.as_deref()?;
            (item.dependencies.iter().any(|dependency| seed(dependency)) || shaped(name, project)).then_some(name)
        })
        .collect();
    loop {
        let more: Vec<&str> = crates
            .iter()
            .filter_map(|item| {
                let name = item.name.as_deref()?;
                if side.contains(name) {
                    return None;
                }
                let users: Vec<&str> = crates
                    .iter()
                    .filter(|user| user.dependencies.iter().any(|dependency| dependency == name))
                    .filter_map(|user| user.name.as_deref())
                    .filter(|user| !allowed.contains(user))
                    .collect();
                (!users.is_empty() && users.iter().all(|user| side.contains(user))).then_some(name)
            })
            .collect();
        if more.is_empty() {
            break;
        }
        side.extend(more);
    }
    side.into_iter()
        .filter(|name| !(allowed.contains(name) || name.split('-').next() == Some(project) && under_client(name)))
        .map(str::to_owned)
        .collect()
}

fn under_client(name: &str) -> bool {
    let mut parts = name.split('-');
    let _ = parts.next();
    let Some(kind) = parts.next() else { return false };
    kind == "client" || (!KINDS.contains(&kind) && parts.next() == Some("client"))
}

fn seed(dependency: &str) -> bool {
    UI_KIT.contains(&dependency)
        || (under_client(dependency) && !SHARED.iter().any(|suffix| dependency.ends_with(suffix)))
}

fn shaped(name: &str, project: &str) -> bool {
    let Some(tail) = name.strip_prefix(project).and_then(|tail| tail.strip_prefix('-')) else {
        return false;
    };
    !tail.starts_with("engine-")
        && (tail.starts_with("shell-") || tail.split('-').any(|part| part == "client" || part == "view"))
}
