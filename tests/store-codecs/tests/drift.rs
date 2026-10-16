//! The committed codecs and golden bytes against their schemas (skein's codec.md, 6).
use skein_codegen::{generate, parse};
use std::fs;
use std::path::{Path, PathBuf};

fn store() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/jig-protocol-store")
}

fn check(family: &str, version: u16) {
    let source =
        fs::read_to_string(store().join(format!("schema/{family}-v{version}.schema"))).expect("schema readable");
    let output = generate(&parse(&source).expect("schema valid"));
    let code = store().join(format!("src/generated/{family}_v{version}.rs"));
    let goldens = store().join(format!("golden/{family}/v{version}"));
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let again = "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-store-codecs-world --test drift";
    if write {
        fs::write(&code, &output.rust).expect("write generated code");
        fs::create_dir_all(&goldens).expect("create golden directory");
    } else {
        assert_eq!(fs::read_to_string(&code).expect("generated code committed"), output.rust, "{again}");
    }
    for golden in output.goldens {
        let path = goldens.join(golden.name);
        if write {
            fs::write(&path, golden.bytes).expect("write golden bytes");
        } else {
            assert_eq!(fs::read(&path).expect("golden bytes committed"), golden.bytes, "{again}");
        }
    }
}

#[test]
fn the_notes_familys_code_and_goldens_have_no_drift() {
    check("notes", 1);
}

#[test]
fn the_component_familys_code_and_goldens_have_no_drift() {
    check("component", 1);
}

#[test]
fn the_tasks_familys_code_and_goldens_have_no_drift() {
    check("tasks", 3);
}

#[test]
fn the_people_familys_code_and_goldens_have_no_drift() {
    check("people", 2);
}

#[test]
fn the_core_familys_code_and_goldens_have_no_drift() {
    check("core", 3);
}

#[test]
fn the_test_connectors_code_and_goldens_have_no_drift() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testing/jig-test-store");
    let source = fs::read_to_string(root.join("schema/test-connector-v1.schema")).expect("schema readable");
    let output = generate(&parse(&source).expect("schema valid"));
    let code = root.join("src/generated/v1.rs");
    let goldens = root.join("golden/v1");
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    if write {
        fs::write(code, &output.rust).expect("write code");
        fs::create_dir_all(&goldens).expect("golden directory");
    } else {
        assert_eq!(fs::read_to_string(code).expect("code committed"), output.rust);
    }
    for golden in output.goldens {
        let path = goldens.join(golden.name);
        if write {
            fs::write(path, golden.bytes).expect("write golden");
        } else {
            assert_eq!(fs::read(path).expect("golden committed"), golden.bytes);
        }
    }
}
