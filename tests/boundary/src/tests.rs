mod graph;
mod vocabulary;

use crate::manifest::{CrateManifest, parse_manifest, quoted_value, source_files, walk_directory, workspace_manifest};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn kit_root() -> PathBuf {
    // Cargo and nextest run tests from the current package directory. A cached
    // binary may have been compiled in a worktree that has since been removed.
    std::env::current_dir().expect("package directory exists").join("../..").canonicalize().expect("kit root exists")
}

fn quoted_end(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let quote = bytes[start];
    let mut end = start + 1;
    while end < bytes.len() {
        if bytes[end] == quote {
            return end + 1;
        }
        if quote == b'"' && bytes[end] == b'\\' {
            end += 1;
        }
        end += 1;
    }
    bytes.len()
}

fn dependency_source_end(source: &str, key_end: usize) -> Option<usize> {
    let tail = source[key_end..].trim_start_matches([' ', '\t']);
    let value = tail.strip_prefix('=')?.trim_start_matches([' ', '\t']);
    if !value.starts_with(['"', '\'']) {
        return None;
    }
    let start = source.len() - value.len();
    Some(quoted_end(source, start))
}

fn workspace_vocabulary(source: &str) -> String {
    let key = ["g", "it"].concat();
    let mut vocabulary = String::new();
    let mut start = 0;
    while start < source.len() {
        let tail = &source[start..];
        let ch = tail.chars().next().expect("source has remaining text");
        let end = if ch == '#' {
            tail.find('\n').map_or(source.len(), |end| start + end)
        } else if ch == '"' || ch == '\'' {
            quoted_end(source, start)
        } else if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            start + tail.find(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-').unwrap_or(tail.len())
        } else {
            start + ch.len_utf8()
        };
        let token = &source[start..end];
        let is_key = token == key || quoted_value(token) == Some(key.as_str());
        if is_key && let Some(pair_end) = dependency_source_end(source, end) {
            start = pair_end;
        } else {
            vocabulary.push_str(token);
            start = end;
        }
    }
    vocabulary
}

fn manifests() -> Vec<CrateManifest> {
    crate::manifest::manifests(&kit_root())
}

fn forbidden_words() -> Vec<String> {
    [
        vec!["tem", "per"],
        vec!["for", "ge"],
        vec!["for", "gejo"],
        vec!["repo", "sitory"],
        vec!["repo", "sitories"],
        vec!["bra", "nch"],
        vec!["bra", "nches"],
        vec!["pull", " request"],
        vec!["g", "it"],
        vec!["wi", "ki"],
    ]
    .into_iter()
    .map(|parts| parts.concat())
    .collect()
}

#[test]
fn workspace_fixture_checks_dependencies_without_a_package() {
    let forbidden = ["tem", "per", "-sample"].concat();
    let manifest =
        format!("[workspace]\nmembers = []\n[workspace.dependencies]\n{forbidden} = {{ path = \"../example\" }}\n");
    let parsed = parse_manifest(PathBuf::from("fixture/Cargo.toml"), &manifest);
    assert_eq!(parsed.name, None);
    assert_eq!(graph::breaches(&[parsed], graph::all_crates), [format!("fixture/Cargo.toml: {forbidden}")]);
    assert!(!workspace_manifest("[workspace]\n[package]\nname = \"jig-sample\"\n"));
}

#[test]
fn walk_fixture_skips_build_and_hidden_directories() {
    for (name, at_root, expected) in [
        ("target", true, false),
        (&[".", "g", "it"].concat(), true, false),
        (".config", false, false),
        ("tests", true, true),
        ("target", false, true),
        ("golden", true, false),
        ("golden", false, false),
        ("target-x", true, false),
        ("target-x", false, true),
        ("targets", false, true),
    ] {
        assert_eq!(walk_directory(name, at_root), expected, "{name}, at_root {at_root}");
    }
}

#[test]
fn the_store_codecs_world_is_among_the_workspace_worlds() {
    let worlds: Vec<_> =
        manifests().into_iter().filter(|m| m.name.as_deref() == Some("jig-store-codecs-world")).collect();
    assert_eq!(worlds.len(), 1, "one store codecs world");
    let manifest = &worlds[0];
    assert!(manifest.dependencies.iter().all(|name| {
        ["jig-protocol-store", "jig-test-store", "skein-codegen", "skein-lib", "skein-heap"].contains(&name.as_str())
    }));
}

#[test]
fn store_schema_module_names_pass_the_source_guard() {
    let source = "use crate::component_v1 as wire; use crate::notes_v1 as wire;";
    let failures = super::names::breaches(Path::new("fixture/codec.rs"), source);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn client_link_depends_only_on_client_vocabulary_and_foundation() {
    let link =
        manifests().into_iter().find(|m| m.name.as_deref() == Some("jig-client-protocol-link")).expect("link manifest");
    assert!(link.dependencies.iter().all(|name| {
        ["skein-lib", "skein-codec", "jig-client-domain", "jig-client-protocol-wire"].contains(&name.as_str())
            || name == "jig-protocol-transcript"
    }));
}

#[test]
fn the_test_store_links_only_its_codec_boundary_and_foundation() {
    let store = manifests().into_iter().find(|m| m.name.as_deref() == Some("jig-test-store")).expect("store manifest");
    assert!(store.dependencies.iter().all(|name| {
        [
            "jig-test-connector",
            "jig-test-domain",
            "jig-domain-core",
            "jig-domain-notes",
            "jig-domain-people",
            "jig-protocol-store",
            "skein-kv",
            "skein-lib",
            "skein-codec",
        ]
        .contains(&name.as_str())
    }));
    let source = fs::read_to_string(kit_root().join("testing/jig-test-store/src/lib.rs")).expect("store source");
    assert!(source.contains("cfg_attr(not(test), no_std)"));
}

#[test]
fn engine_clients_link_only_their_boundary_wire_and_foundation() {
    let selected: Vec<_> = manifests()
        .into_iter()
        .filter(|manifest| manifest.name.as_deref() == Some("jig-protocol-engine-clients"))
        .collect();
    assert_eq!(selected.len(), 1, "one engine client component");
    let mut dependencies = selected[0].dependencies.clone();
    dependencies.sort();
    assert_eq!(
        dependencies,
        [
            "jig-client-protocol-wire",
            "jig-domain-accounts",
            "jig-domain-authority",
            "jig-domain-core",
            "jig-domain-fleet",
            "jig-domain-notes",
            "jig-domain-people",
            "jig-domain-tasks",
            "jig-domain-views",
            "jig-protocol-store",
            "skein-codec",
            "skein-http",
            "skein-io",
            "skein-lib"
        ]
    );
}

#[test]
fn unqualified_ip_variant_names_stay_refused() {
    let cases: [(&str, &[&str]); 6] = [
        ("core::net::IpAddr::V4", &[]),
        ("core::net::IpAddr::V6", &[]),
        ("V4", &["fixture/address.rs:1: identifier V4"]),
        ("V6", &["fixture/address.rs:1: identifier V6"]),
        ("jig::V4", &["fixture/address.rs:1: identifier V4"]),
        ("customcore::net::IpAddr::V4", &["fixture/address.rs:1: identifier V4"]),
    ];
    for (source, expected) in cases {
        assert_eq!(super::names::breaches(Path::new("fixture/address.rs"), source), expected, "{source}");
    }
}

#[test]
fn the_store_files_kit_uses_only_the_store_and_foundation() {
    let kit = manifests()
        .into_iter()
        .find(|m| m.name.as_deref() == Some("jig-test-store-files"))
        .expect("store files manifest");
    assert!(kit.dependencies.iter().all(|name| {
        [
            "jig-conformance",
            "jig-test-store",
            "jig-test-domain",
            "jig-test-connector",
            "jig-protocol-store",
            "jig-domain-core",
            "skein-lib",
            "skein-io",
            "skein-kv",
            "skein-fake-machine",
        ]
        .contains(&name.as_str())
    }));
    let source = fs::read_to_string(kit.path).expect("store files manifest is readable");
    assert!(source.contains("[lints.clippy]"));
    assert!(kit_root().join("testing/jig-test-store-files/clippy.toml").is_file());
}

#[test]
fn the_native_shell_links_only_the_clients_components_and_foundation() {
    let shell = manifests()
        .into_iter()
        .find(|manifest| manifest.name.as_deref() == Some("jig-client-shell-native"))
        .expect("native shell manifest");
    let mut dependencies = shell.dependencies;
    dependencies.sort();
    assert_eq!(
        dependencies,
        ["jig-client", "jig-client-protocol-http", "jig-client-protocol-wire", "skein-http", "skein-io", "skein-lib"]
    );
}
