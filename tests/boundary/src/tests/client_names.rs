use crate::{client_names::of, manifest::CrateManifest};
use std::path::PathBuf;

fn manifest(name: &str, dependencies: &[&str]) -> CrateManifest {
    CrateManifest {
        path: PathBuf::from("fixture/Cargo.toml"),
        name: Some(name.to_owned()),
        dependencies: dependencies.iter().map(|name| (*name).to_owned()).collect(),
    }
}

#[test]
fn capability_clients_do_not_seed_ui_crates() {
    for dependency in ["foundation-resource-client", "foundation-echo-client", "foundation-resource-client-domain"] {
        let engine = manifest("sample-engine-domain", &[dependency]);
        assert_eq!(of(&[engine], "sample", &[]), Vec::<String>::new(), "{dependency}");
    }
}

#[test]
fn ui_dependencies_still_report_crates_outside_the_prefix() {
    for dependency in [
        "skein-ui",
        "skein-ui-text",
        "skein-ui-web",
        "skein-fake-person",
        "skein-fake-person-browser",
        "jig-client",
        "jig-client-domain",
        "jig-ops-client",
        "jig-ops-client-view",
        "sample-client",
        "sample-client-view",
        "sample-ops-client",
    ] {
        let page = manifest("sample-page", &[dependency]);
        assert_eq!(of(&[page], "sample", &[]), ["sample-page"], "{dependency}");
    }
}

#[test]
fn compound_project_clients_keep_their_accepted_prefix() {
    for name in ["sample-ops-client", "sample-ops-client-domain", "sample-ops-client-shell-web"] {
        let client = manifest(name, &["jig-client"]);
        assert_eq!(of(&[client], "sample-ops", &[]), Vec::<String>::new(), "{name}");
    }
}

#[test]
fn compound_project_dependencies_still_seed_ui_crates() {
    let page = manifest("sample-ops-page", &["sample-ops-client"]);
    assert_eq!(of(&[page], "sample-ops", &[]), ["sample-ops-page"]);
}

#[test]
fn shared_wire_records_do_not_seed_engine_users() {
    for dependency in [
        "jig-client-protocol-wire",
        "jig-client-test-protocol-wire",
        "jig-client-test-records",
        "jig-ops-client-protocol-wire",
        "sample-ops-client-protocol-wire",
    ] {
        let engine = manifest("sample-ops-engine-domain", &[dependency]);
        assert_eq!(of(&[engine], "sample-ops", &[]), Vec::<String>::new(), "{dependency}");
    }
}

#[test]
fn exclusive_helpers_of_ui_crates_are_still_reported() {
    let client = manifest("sample-client", &["sample-helper"]);
    let helper = manifest("sample-helper", &[]);
    assert_eq!(of(&[client, helper], "sample", &[]), ["sample-helper"]);
}

#[test]
fn helpers_shared_with_an_engine_keep_their_names() {
    let client = manifest("sample-client", &["sample-helper"]);
    let engine = manifest("sample-engine-domain", &["sample-helper", "foundation-resource-client"]);
    let helper = manifest("sample-helper", &[]);
    assert_eq!(of(&[client, engine, helper], "sample", &[]), Vec::<String>::new());
}

#[test]
fn layer_names_do_not_become_application_client_seeds() {
    for dependency in ["jig-domain-client", "jig-protocol-client", "sample-engine-client"] {
        let engine = manifest("sample-engine-domain", &[dependency]);
        assert_eq!(of(&[engine], "sample", &[]), Vec::<String>::new(), "{dependency}");
    }
}
