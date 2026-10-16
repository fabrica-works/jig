use super::{CrateManifest, kit_root, manifests, parse_manifest};
use std::{fs, path::PathBuf};

pub(super) fn breaches(manifests: &[CrateManifest], role: fn(&CrateManifest) -> bool) -> Vec<String> {
    manifests
        .iter()
        .filter(|manifest| role(manifest))
        .flat_map(|manifest| {
            manifest
                .dependencies
                .iter()
                .filter(|dependency| dependency.starts_with(&["tem", "per"].concat()))
                .map(|dependency| format!("{}: {dependency}", manifest.path.display()))
        })
        .collect()
}

pub(super) fn all_crates(_: &CrateManifest) -> bool {
    true
}

#[test]
fn no_kit_crate_depends_on_an_application_crate_even_when_empty() {
    let failures = breaches(&manifests(), all_crates);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

const CORE_FAMILY: [&str; 9] = [
    "jig-domain-core",
    "jig-domain-accounts",
    "jig-domain-authority",
    "jig-domain-brief",
    "jig-domain-fleet",
    "jig-domain-notes",
    "jig-domain-people",
    "jig-domain-tasks",
    "jig-domain-views",
];

fn core_breaches(manifests: &[CrateManifest]) -> Vec<String> {
    let failures: Vec<_> = manifests
        .iter()
        .filter(|manifest| manifest.name.as_deref().is_some_and(|name| CORE_FAMILY.contains(&name)))
        .flat_map(|manifest| {
            manifest
                .dependencies
                .iter()
                .filter(|name| {
                    if manifest.name.as_deref() == Some("jig-domain-core") {
                        **name != "skein-lib" && !CORE_FAMILY.contains(&name.as_str())
                    } else {
                        **name != "skein-lib"
                    }
                })
                .map(|name| format!("{}: {name}", manifest.path.display()))
                .collect::<Vec<_>>()
        })
        .collect();
    failures
}

#[test]
fn core_and_children_use_only_the_kit_and_foundation_even_when_empty() {
    let failures = core_breaches(&manifests());
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn hosts_and_translation_use_no_core_crates_even_when_empty() {
    let failures: Vec<_> = manifests()
        .into_iter()
        .filter(|manifest| matches!(manifest.name.as_deref(), Some("jig-domain-host" | "jig-protocol-charter")))
        .flat_map(|manifest| {
            manifest
                .dependencies
                .iter()
                .filter(|name| CORE_FAMILY.contains(&name.as_str()))
                .map(|name| format!("{}: {name}", manifest.path.display()))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn host_hub_links_only_skein() {
    let failures: Vec<_> = manifests()
        .into_iter()
        .filter(|manifest| manifest.name.as_deref() == Some("jig-domain-host"))
        .flat_map(|manifest| {
            manifest
                .dependencies
                .iter()
                .filter(|name| name.as_str() != "skein-lib")
                .map(|name| format!("{}: {name}", manifest.path.display()))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn client_view_links_only_its_domain_and_foundation() {
    let views: Vec<_> =
        manifests().into_iter().filter(|manifest| manifest.name.as_deref() == Some("jig-client-view")).collect();
    assert_eq!(views.len(), 1, "the client view has one manifest");
    let failures: Vec<_> = views
        .into_iter()
        .flat_map(|manifest| {
            manifest
                .dependencies
                .iter()
                .filter(|name| {
                    !["skein-lib", "skein-ui", "jig-domain-client", "jig-client-domain"].contains(&name.as_str())
                })
                .map(|name| format!("{}: {name}", manifest.path.display()))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn client_domain_depends_on_foundation_alone() {
    let failures: Vec<_> = manifests()
        .into_iter()
        .filter(|manifest| manifest.name.as_deref() == Some("jig-domain-client"))
        .flat_map(|manifest| {
            manifest
                .dependencies
                .iter()
                .filter(|name| !["skein-lib", "jig-client-domain"].contains(&name.as_str()))
                .map(|name| format!("{}: {name}", manifest.path.display()))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn client_step(name: &str) -> bool {
    matches!(
        name,
        "jig-domain-client"
            | "jig-client-domain"
            | "jig-client-view"
            | "jig-protocol-transcript"
            | "jig-client-protocol-wire"
            | "jig-client-protocol-http"
            | "jig-client-protocol-link"
            | "jig-client-test-domain"
            | "jig-client-test-view"
            | "jig-ops-domain-connectors"
            | "jig-ops-client-domain"
            | "jig-ops-client-view"
    )
}

#[test]
fn client_and_engine_step_crates_share_no_dependencies() {
    let failures: Vec<_> = manifests()
        .into_iter()
        .flat_map(|manifest| {
            let name = manifest.name.as_deref().unwrap_or("");
            manifest
                .dependencies
                .iter()
                .filter(|dependency| {
                    (client_step(name) && CORE_FAMILY.contains(&dependency.as_str()))
                        || (CORE_FAMILY.contains(&name) && client_step(dependency))
                })
                .map(|dependency| format!("{}: {dependency}", manifest.path.display()))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn client_roles_cover_step_crates_and_exclude_the_world() {
    assert!(client_step("jig-domain-client"));
    assert!(client_step("jig-client-view"));
    assert!(client_step("jig-protocol-transcript"));
    assert!(client_step("jig-client-test-domain"));
    assert!(client_step("jig-client-test-view"));
    assert!(client_step("jig-ops-domain-connectors"));
    assert!(!client_step("jig-client-world"));
    assert!(!client_step("jig-client-scripted-engine"));
}

/// Only the below-domain transcript translator links Smith (client/domain/README.md, 2).
#[test]
fn client_transcript_is_the_client_only_smith_link() {
    let clients: Vec<_> =
        manifests().into_iter().filter(|manifest| manifest.name.as_deref().is_some_and(client_step)).collect();
    let mut decoders = 0_u32;
    let mut failures = Vec::new();
    for manifest in clients {
        let decoder = manifest.name.as_deref() == Some("jig-protocol-transcript");
        if decoder {
            decoders += 1;
            let mut dependencies = manifest.dependencies.clone();
            dependencies.sort();
            assert_eq!(
                dependencies,
                ["jig-client-domain", "skein-lib", "smith-transcript"],
                "transcript links only its three roles"
            );
        }
        for dependency in &manifest.dependencies {
            if dependency.starts_with("smith-") && !(decoder && dependency == "smith-transcript") {
                failures.push(format!("{}: {dependency}", manifest.path.display()));
            }
        }
    }
    assert_eq!(decoders, 1, "one transcript translator owns the Smith boundary");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn scripted_engine_links_only_the_client_and_foundation() {
    let engines: Vec<_> = manifests()
        .into_iter()
        .filter(|manifest| manifest.name.as_deref() == Some("jig-client-scripted-engine"))
        .collect();
    assert_eq!(engines.len(), 1);
    for manifest in engines {
        assert!(
            manifest
                .dependencies
                .iter()
                .all(|name| ["skein-lib", "jig-domain-client", "jig-client-domain"].contains(&name.as_str())),
            "{}",
            manifest.path.display()
        );
    }
}

#[test]
fn test_client_domain_links_only_its_child_owner_vocabulary_and_foundation() {
    let roots: Vec<_> =
        manifests().into_iter().filter(|manifest| manifest.name.as_deref() == Some("jig-client-test-domain")).collect();
    assert_eq!(roots.len(), 1, "one test application client domain");
    let mut dependencies = roots[0].dependencies.clone();
    dependencies.sort();
    assert_eq!(
        dependencies,
        ["jig-client-domain", "jig-domain-client", "jig-test-connector", "skein-lib"],
        "the client root depends only on its client child, pure owner vocabulary and foundation"
    );
    let source = fs::read_to_string(kit_root().join("testing/jig-client-test-domain/src/lib.rs"))
        .expect("root source is readable");
    assert!(source.contains("cfg_attr(not(test), no_std)"), "the client root is step code");
    assert!(source.contains("forbid(unsafe_code)"), "the client root forbids unsafe code");
}

/// Sharing the test application's exact vocabulary adds no engine/core/Smith edge.
#[test]
fn test_connector_vocabulary_and_client_value_kit_keep_their_exact_roles() {
    let all = manifests();
    for (name, expected) in [
        ("jig-test-connector", vec!["skein-lib"]),
        ("jig-client-test-kit", vec!["jig-client-test-domain", "jig-test-connector", "skein-lib"]),
    ] {
        let selected: Vec<_> = all.iter().filter(|manifest| manifest.name.as_deref() == Some(name)).collect();
        assert_eq!(selected.len(), 1, "one exact {name} role");
        let mut dependencies = selected[0].dependencies.clone();
        dependencies.sort();
        assert_eq!(dependencies, expected, "{name} links only its owned vocabulary");
    }
}

#[test]
fn test_client_view_links_only_client_children_and_foundation() {
    let roots: Vec<_> =
        manifests().into_iter().filter(|manifest| manifest.name.as_deref() == Some("jig-client-test-view")).collect();
    assert_eq!(roots.len(), 1, "one test application client view");
    let mut dependencies = roots[0].dependencies.clone();
    dependencies.sort();
    assert_eq!(
        dependencies,
        [
            "jig-client-domain",
            "jig-client-test-domain",
            "jig-client-view",
            "jig-domain-client",
            "skein-lib",
            "skein-ui"
        ]
    );
    let source = fs::read_to_string(kit_root().join("testing/jig-client-test-view/src/lib.rs"))
        .expect("view source is readable");
    assert!(source.contains("cfg_attr(not(test), no_std)"), "the view root is step code");
    assert!(source.contains("forbid(unsafe_code)"), "the view root forbids unsafe code");
}

#[test]
fn client_world_joins_only_the_client_and_test_roles() {
    let worlds: Vec<_> = manifests().into_iter().filter(|m| m.name.as_deref() == Some("jig-client-world")).collect();
    assert_eq!(worlds.len(), 1, "one shared client world");
    for manifest in worlds {
        assert!(
            manifest.dependencies.iter().all(|d| [
                "skein-lib",
                "skein-world",
                "jig-client-domain",
                "jig-client-protocol-link",
                "jig-domain-client",
                // Whole committed turns use the existing below-domain codec.
                "jig-protocol-transcript",
                "smith-transcript",
                "jig-client-view",
                "skein-ui",
                "skein-ui-text",
                "jig-client-test-domain",
                "jig-client-test-view",
                "skein-fake-person",
                "jig-client-scripted-engine",
                // The real joint world joins the core and pure test connector
                // (client/domain/testing.md, 5); client step crates stay separate.
                "jig-domain-core",
                "jig-domain-authority",
                "jig-domain-people",
                "jig-domain-fleet",
                "jig-domain-accounts",
                "jig-domain-tasks",
                "jig-domain-views",
                "jig-domain-notes",
                // Reuse the actual root/store/workers/systems and referee, not a harness copy.
                "jig-core-world",
                // Scripted parties remain beside the real joint protocol peer.
                "jig-fake-parties",
                "jig-fake-workers",
                "jig-test-domain",
                "jig-test-connector",
                "jig-client-test-kit",
                "sha2"
            ]
            .contains(&d.as_str())),
            "{}",
            manifest.path.display()
        );
    }
}

#[test]
fn ops_connector_identities_link_only_the_foundation() {
    let roots: Vec<_> =
        manifests().into_iter().filter(|m| m.name.as_deref() == Some("jig-ops-domain-connectors")).collect();
    assert_eq!(roots.len(), 1, "one Ops connector vocabulary");
    assert_eq!(roots[0].dependencies, ["skein-lib"]);
    let source =
        fs::read_to_string(kit_root().join("examples/ops/domain-connectors/src/lib.rs")).expect("connector source");
    assert!(source.contains("cfg_attr(not(test), no_std)"), "connector identities are step code");
    assert!(source.contains("forbid(unsafe_code)"), "connector identities forbid unsafe");
}

#[test]
fn ops_client_domain_is_step_code_with_only_client_and_connector_vocabulary() {
    let roots: Vec<_> =
        manifests().into_iter().filter(|m| m.name.as_deref() == Some("jig-ops-client-domain")).collect();
    assert_eq!(roots.len(), 1);
    let mut dependencies = roots[0].dependencies.clone();
    dependencies.sort();
    assert_eq!(dependencies, ["jig-client-domain", "jig-domain-client", "jig-ops-domain-connectors", "skein-lib"]);
    let source = fs::read_to_string(kit_root().join("examples/ops/client-domain/src/lib.rs")).expect("domain source");
    assert!(source.contains("cfg_attr(not(test), no_std)"));
    assert!(source.contains("forbid(unsafe_code)"));
}

#[test]
fn ops_client_view_is_step_code_with_only_client_and_connector_vocabulary() {
    let roots: Vec<_> = manifests().into_iter().filter(|m| m.name.as_deref() == Some("jig-ops-client-view")).collect();
    assert_eq!(roots.len(), 1);
    let mut dependencies = roots[0].dependencies.clone();
    dependencies.sort();
    assert_eq!(
        dependencies,
        [
            "jig-client-domain",
            "jig-client-view",
            "jig-domain-client",
            "jig-ops-client-domain",
            "jig-ops-domain-connectors",
            "skein-lib",
            "skein-ui"
        ]
    );
    let source = fs::read_to_string(kit_root().join("examples/ops/client-view/src/lib.rs")).expect("view source");
    assert!(source.contains("cfg_attr(not(test), no_std)"));
    assert!(source.contains("forbid(unsafe_code)"));
}

fn application_breaches(manifests: &[CrateManifest]) -> Vec<String> {
    let mut failures = Vec::new();
    let first_application = ["tem", "per"].concat();
    for manifest in manifests {
        if !manifest.name.as_deref().is_some_and(|name| {
            name.starts_with("jig-protocol-") || (name.starts_with("jig-client-protocol-") && !name.ends_with("-world"))
        }) {
            continue;
        }
        for dependency in &manifest.dependencies {
            if dependency.starts_with("jig-ops-")
                || dependency.starts_with("jig-test-")
                || dependency.starts_with("jig-client-test-")
                || dependency.starts_with(&first_application)
            {
                failures.push(format!("{}: {dependency}", manifest.path.display()));
            }
        }
    }
    failures
}

fn smith_breaches(manifests: &[CrateManifest]) -> Vec<String> {
    let mut failures = Vec::new();
    for manifest in manifests {
        let name = manifest.name.as_deref().unwrap_or("");
        if !(name.starts_with("jig-domain-")
            || name.starts_with("jig-protocol-")
            || name == "jig-client-view"
            || name.starts_with("jig-client-shell-")
            || (name.starts_with("jig-client-protocol-") && !name.ends_with("-world")))
        {
            continue;
        }
        for dependency in &manifest.dependencies {
            let allowed = match name {
                "jig-protocol-charter" => {
                    ["smith-domain-run", "smith-charter", "smith-channel"].contains(&dependency.as_str())
                }
                "jig-protocol-transcript" => dependency == "smith-transcript",
                _ => false,
            };
            if dependency.starts_with("smith-") && !allowed {
                failures.push(format!("{}: {dependency}", manifest.path.display()));
            }
        }
    }
    failures
}

fn test_worker_smith_breaches(manifests: &[CrateManifest]) -> Vec<String> {
    let mut failures = Vec::new();
    for manifest in manifests {
        if !matches!(manifest.name.as_deref(), Some("jig-test-worker" | "jig-test-service")) {
            continue;
        }
        for dependency in &manifest.dependencies {
            if dependency.starts_with("smith-")
                && !(manifest.name.as_deref() == Some("jig-test-service") && dependency == "smith-domain-run")
                && !["smith-host-domain", "smith-host-protocol", "smith-protocol-channel", "smith-channel"]
                    .contains(&dependency.as_str())
            {
                failures.push(format!("{}: {dependency}", manifest.path.display()));
            }
        }
    }
    failures
}

fn client_engine_breaches(manifests: &[CrateManifest]) -> Vec<String> {
    let mut failures = Vec::new();
    for manifest in manifests {
        if !matches!(
            manifest.name.as_deref(),
            Some(
                "jig-domain-client"
                    | "jig-client-view"
                    | "jig-client-protocol-wire"
                    | "jig-client-protocol-link"
                    | "jig-client-shell-native"
            )
        ) {
            continue;
        }
        for dependency in &manifest.dependencies {
            let engine_protocol = dependency.starts_with("jig-protocol-")
                && !["jig-client-protocol-wire", "jig-client-protocol-link", "jig-protocol-transcript"]
                    .contains(&dependency.as_str());
            if CORE_FAMILY.contains(&dependency.as_str()) || dependency == "jig-domain-host" || engine_protocol {
                failures.push(format!("{}: {dependency}", manifest.path.display()));
            }
        }
    }
    failures
}

#[test]
fn protocol_crates_link_no_application_crate_even_when_empty() {
    let failures = application_breaches(&manifests());
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn worker_wire_links_only_skeins_codec_and_channel_vocabulary() {
    let roots: Vec<_> = manifests()
        .into_iter()
        .filter(|manifest| manifest.name.as_deref() == Some("jig-protocol-worker-wire"))
        .collect();
    assert_eq!(roots.len(), 1, "one worker wire codec crate");
    let mut dependencies = roots[0].dependencies.clone();
    dependencies.sort();
    assert_eq!(dependencies, ["skein-channel", "skein-codec", "skein-lib"]);
}

#[test]
fn a_protocol_crate_naming_an_ops_crate_is_reported() {
    let manifest = parse_manifest(
        PathBuf::from("fixture/tools/Cargo.toml"),
        "[package]\nname = \"jig-protocol-tools\"\n[dependencies]\njig-ops-domain.workspace = true\n",
    );
    assert_eq!(application_breaches(&[manifest]), ["fixture/tools/Cargo.toml: jig-ops-domain"]);
}

#[test]
fn only_the_charter_and_the_transcript_link_smith_even_when_empty() {
    let failures = smith_breaches(&manifests());
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_protocol_crate_linking_smith_is_reported_unless_charter_or_transcript() {
    let tools = parse_manifest(
        PathBuf::from("fixture/tools/Cargo.toml"),
        "[package]\nname = \"jig-protocol-tools\"\n[dependencies]\nsmith-charter.workspace = true\n",
    );
    let charter = parse_manifest(
        PathBuf::from("fixture/charter/Cargo.toml"),
        "[package]\nname = \"jig-protocol-charter\"\n[dependencies]\nsmith-charter.workspace = true\n",
    );
    assert_eq!(smith_breaches(&[tools, charter]), ["fixture/tools/Cargo.toml: smith-charter"]);
}

#[test]
fn test_worker_and_service_link_only_the_four_smith_crates_even_when_empty() {
    let failures = test_worker_smith_breaches(&manifests());
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_test_service_linking_smith_charter_is_reported() {
    let manifest = parse_manifest(
        PathBuf::from("fixture/service/Cargo.toml"),
        "[package]\nname = \"jig-test-service\"\n[dependencies]\nsmith-charter.workspace = true\n",
    );
    assert_eq!(test_worker_smith_breaches(&[manifest]), ["fixture/service/Cargo.toml: smith-charter"]);
}

#[test]
fn client_crates_link_no_engine_side_crate_even_when_empty() {
    let failures = client_engine_breaches(&manifests());
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_native_shell_naming_the_store_is_reported() {
    let manifest = parse_manifest(
        PathBuf::from("fixture/shell/Cargo.toml"),
        "[package]\nname = \"jig-client-shell-native\"\n[dependencies]\njig-protocol-store.workspace = true\n",
    );
    assert_eq!(client_engine_breaches(&[manifest]), ["fixture/shell/Cargo.toml: jig-protocol-store"]);
}

#[test]
fn configuration_links_only_its_vocabulary_and_foundation() {
    let manifests = manifests();
    let config = manifests
        .iter()
        .find(|manifest| manifest.name.as_deref() == Some("jig-protocol-config"))
        .expect("configuration manifest");
    for dependency in &config.dependencies {
        assert!(
            ["skein-lib", "skein-json", "skein-io", "skein-oauth", "skein-oauth-accounts", "skein-tls"]
                .contains(&dependency.as_str())
                || CORE_FAMILY.contains(&dependency.as_str())
                || ["jig-domain-host", "jig-protocol-engine-clients", "jig-client-protocol-wire", "skein-http"]
                    .contains(&dependency.as_str())
                || ["jig-protocol-store", "skein-kv"].contains(&dependency.as_str()),
            "configuration dependency: {dependency}"
        );
    }
}

#[test]
fn client_wire_links_only_its_codec_and_foundation() {
    let wires: Vec<_> = manifests()
        .into_iter()
        .filter(|manifest| manifest.name.as_deref() == Some("jig-client-protocol-wire"))
        .collect();
    assert_eq!(wires.len(), 1, "one client wire family");
    let mut dependencies = wires[0].dependencies.clone();
    dependencies.sort();
    assert_eq!(dependencies, ["skein-codec", "skein-lib"]);
}

#[test]
fn engine_worker_channels_link_only_their_boundary_and_foundation() {
    let roots: Vec<_> = manifests()
        .into_iter()
        .filter(|manifest| manifest.name.as_deref() == Some("jig-protocol-engine-workers"))
        .collect();
    assert_eq!(roots.len(), 1, "one engine worker channel component");
    let mut dependencies = roots[0].dependencies.clone();
    dependencies.sort();
    assert_eq!(
        dependencies,
        ["jig-domain-host", "jig-protocol-worker-wire", "skein-channel", "skein-codec", "skein-io", "skein-lib"]
    );
}

#[test]
fn worker_link_links_only_its_wire_host_and_channel_vocabulary() {
    let roots: Vec<_> = manifests()
        .into_iter()
        .filter(|manifest| manifest.name.as_deref() == Some("jig-protocol-worker-link"))
        .collect();
    assert_eq!(roots.len(), 1, "one worker link component");
    let mut dependencies = roots[0].dependencies.clone();
    dependencies.sort();
    assert_eq!(
        dependencies,
        ["jig-domain-host", "jig-protocol-worker-wire", "skein-channel", "skein-codec", "skein-lib"]
    );
}

#[test]
fn the_identity_dependency_is_allowed_for_the_test_service_alone() {
    for (label, name, dependency, expected) in [
        ("service identity", "jig-test-service", "smith-domain-run", Vec::<String>::new()),
        ("worker identity", "jig-test-worker", "smith-domain-run", vec!["fixture/Cargo.toml: smith-domain-run".into()]),
        ("service domain", "jig-test-service", "smith-domain", vec!["fixture/Cargo.toml: smith-domain".into()]),
    ] {
        let manifest = parse_manifest(
            PathBuf::from("fixture/Cargo.toml"),
            &format!("[package]\nname = \"{name}\"\n[dependencies]\n{dependency}.workspace = true\n"),
        );
        assert_eq!(test_worker_smith_breaches(&[manifest]), expected, "{label}");
    }
}

/// Client crates outside the prefix until the client's restructuring removes them.
const PENDING: [&str; 1] = ["jig-domain-client"];

#[test]
fn client_crates_carry_the_client_prefix() {
    let allowed = [
        "jig-protocol-engine-clients",
        "jig-protocol-engine-clients-http",
        "jig-engine-clients-world",
        "jig-fake-parties",
        "jig-protocol-transcript",
        "jig-test",
        "jig-test-service",
        "jig-one-process-world",
        "jig-ops-world",
    ];
    assert_eq!(crate::client_names::breaches(&kit_root(), "jig", &allowed), PENDING);
}

#[test]
fn a_crate_on_the_client_without_the_prefix_is_reported() {
    let page = parse_manifest(
        PathBuf::from("fixture/page/Cargo.toml"),
        "[package]\nname = \"jig-page\"\n[dependencies]\njig-client.workspace = true\n",
    );
    let client = parse_manifest(PathBuf::from("fixture/client/Cargo.toml"), "[package]\nname = \"jig-client\"\n");
    assert_eq!(crate::client_names::of(&[page, client], "jig", &[]), ["jig-page"]);
}

#[test]
fn a_crate_named_as_a_view_without_the_prefix_is_reported() {
    let page = parse_manifest(
        PathBuf::from("fixture/page/Cargo.toml"),
        "[package]\nname = \"jig-pages-view\"\n[dependencies]\nskein-lib.workspace = true\n",
    );
    assert_eq!(crate::client_names::of(&[page], "jig", &[]), ["jig-pages-view"]);
}

#[test]
fn client_domain_crate_links_only_the_foundation() {
    let domains: Vec<_> = manifests().into_iter().filter(|m| m.name.as_deref() == Some("jig-client-domain")).collect();
    assert_eq!(domains.len(), 1, "one client domain");
    assert_eq!(domains[0].dependencies, ["skein-lib"]);
}
