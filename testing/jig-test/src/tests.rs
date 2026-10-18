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
        "engine":{{"socket":"127.0.0.1:7200"}},"name":"worker-a","secret":"{}","slots":1,
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

fn fixture_bundle(directory: &std::path::Path) {
    fs::write(directory.join("index.html"), b"<!doctype html><title>t</title>").expect("page");
    let digest = directory.join("0123456789abcdef");
    fs::create_dir_all(digest.join("files")).expect("digest files");
    fs::write(digest.join("boot.js"), b"boot();").expect("script");
    fs::write(digest.join("files/robots.txt"), b"User-agent: *\nDisallow:\n").expect("root file");
}
#[test]
fn a_bundle_directory_is_read_with_its_one_digest_directory() {
    let directory = scratch("bundle-read");
    fixture_bundle(&directory);
    let bundle = super::bundle::read(&directory).expect("bundle read");
    assert_eq!(bundle.page.as_ref(), b"<!doctype html><title>t</title>");
    assert_eq!(bundle.etag.as_ref(), b"\"edaa28e315690734\"");
    assert_eq!(bundle.digest.as_ref(), b"0123456789abcdef");
    assert_eq!(bundle.files.len(), 2);
    let script = bundle.files.get(b"boot.js".as_slice()).expect("script");
    assert_eq!((script.kind, script.bytes.as_ref()), (b"text/javascript".as_slice(), b"boot();".as_slice()));
    let robots = bundle.files.get(b"files/robots.txt".as_slice()).expect("root file");
    assert_eq!(robots.bytes.as_ref(), b"User-agent: *\nDisallow:\n");
    fs::remove_dir_all(directory).expect("scratch cleanup");
}
#[test]
fn a_file_off_the_closed_list_refuses_the_bundle_naming_it() {
    let directory = scratch("bundle-type");
    fixture_bundle(&directory);
    let file = directory.join("0123456789abcdef/private.exe");
    fs::write(&file, b"not served").expect("unknown file");
    let error = super::bundle::read(&directory).expect_err("closed list");
    assert!(error.contains(file.to_str().expect("path")), "{error}");
    assert!(error.contains("closed list"), "{error}");
    fs::remove_dir_all(directory).expect("scratch cleanup");
}
#[test]
fn a_second_digest_directory_refuses_the_bundle() {
    let directory = scratch("bundle-digests");
    fixture_bundle(&directory);
    fs::create_dir(directory.join("fedcba9876543210")).expect("second digest");
    let error = super::bundle::read(&directory).expect_err("one digest");
    assert!(error.contains("digest directory"), "{error}");
    assert!(error.contains(directory.to_str().expect("path")), "{error}");
    fs::remove_dir_all(directory).expect("scratch cleanup");
}

const DEV_MODULE_BYTES: usize = 9 << 20;
const MODULE_FILL: u8 = 0;
#[test]
fn a_module_above_four_mebibytes_is_read() {
    let directory = scratch("dev-module");
    fixture_bundle(&directory);
    fs::write(directory.join("0123456789abcdef/test_bg.wasm"), vec![MODULE_FILL; DEV_MODULE_BYTES])
        .expect("dev-sized module");
    let result = super::bundle::read(&directory)
        .map(|bundle| bundle.files.get(b"test_bg.wasm".as_slice()).expect("module").bytes.len());
    fs::remove_dir_all(directory).expect("scratch cleanup");
    assert_eq!(result, Ok(DEV_MODULE_BYTES));
}

const OVERSIZED_FILE_BYTES: u64 = 4_194_305;
#[test]
fn another_oversized_file_is_refused_naming_it() {
    let directory = scratch("oversized-script");
    fixture_bundle(&directory);
    let path = directory.join("0123456789abcdef/boot.js");
    fs::File::create(&path).expect("script").set_len(OVERSIZED_FILE_BYTES).expect("oversized script");
    let error = super::bundle::read(&directory).expect_err("oversized script refused");
    fs::remove_dir_all(directory).expect("scratch cleanup");
    assert!(error.contains(path.to_str().expect("path")), "{error}");
    assert!(error.contains("TooLarge"), "{error}");
}

const FULL_BUNDLE_MODULE_BYTES: u64 = 16_777_216;
#[test]
fn a_bundle_above_sixteen_mebibytes_is_refused() {
    let directory = scratch("oversized-bundle");
    fixture_bundle(&directory);
    let path = directory.join("0123456789abcdef/test_bg.wasm");
    fs::File::create(&path).expect("module").set_len(FULL_BUNDLE_MODULE_BYTES).expect("full bundle module");
    let error = super::bundle::read(&directory).expect_err("oversized bundle refused");
    fs::remove_dir_all(directory).expect("scratch cleanup");
    assert!(error.contains("bundle exceeds its limits"), "{error}");
}
