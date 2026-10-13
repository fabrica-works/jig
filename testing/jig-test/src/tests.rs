use super::{CONFIG_BYTES, configure, prepare, read_configuration};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

fn scratch(case: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("jig-test-{}-{case}", std::process::id()));
    fs::create_dir(&path).expect("scratch directory");
    path
}

#[test]
fn serve_without_a_configuration_path_is_a_usage_error() {
    assert_eq!(configure(["serve".into()].into_iter()), Err(super::USAGE.into()));
}

#[test]
fn a_configuration_file_past_its_bound_is_refused() {
    let directory = scratch("configuration-bound");
    let path = directory.join("config.json");
    let file = fs::File::create(&path).expect("configuration file");
    file.set_len(u64::from(CONFIG_BYTES) + 1).expect("one byte past bound");
    assert!(read_configuration(&path).expect_err("oversized configuration").contains("TooLarge"));
    fs::remove_dir_all(directory).expect("scratch cleanup");
}

#[test]
fn a_secret_file_readable_by_others_refuses_the_start() {
    let directory = scratch("private-secret");
    let secret = directory.join("worker.secret");
    fs::write(&secret, b"private-worker-value").expect("secret file");
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o644)).expect("public permissions");
    let bytes = format!(
        r#"{{"workers":[{{
        "engine":{{"in_process":"workers"}},"name":"worker-a","secret":"{}","slots":1,
        "graces":{{"grace_ms":10000,"agent_stop_ms":5000,"delivery_ms":1000,"save_ms":1000}},
        "agent":{{"program":"jig-test","configuration":"agent.json","arguments":["agent"]}},
        "limits":{{"profile":"small"}},"memory_bytes":1152921504606846976
    }}],"application":{{"workers":[{{"root":"agent-root"}}]}}}}"#,
        secret.display()
    );
    let mut owned = Vec::new();
    let error = prepare(bytes.as_bytes(), &mut owned).err().expect("start refused");
    assert!(error.contains("Unsafe"), "private read refusal: {error}");
    assert!(!error.contains("private-worker-value"), "diagnostic contains no secret");
    assert!(owned.is_empty(), "refusal happens before opening runtime descriptors");
    fs::remove_dir_all(directory).expect("scratch cleanup");
}
