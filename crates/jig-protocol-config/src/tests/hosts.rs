use super::fixtures;
use crate::What;
fn worker_path(index: u32, name: &[u8]) -> crate::Path {
    fixtures::path(&[b"hosts", b"workers"]).child(crate::Segment::Index(index)).member(name)
}
fn files() -> Box<[crate::File]> {
    crate::files(&fixtures::engine(fixtures::ENGINE_SMALL).unwrap())
}
fn a_workers_listener_on_a_public_address_is_refused() {
    fixtures::refuses(
        br#"{"in_process": "workers"}"#,
        br#"{"socket": "192.0.2.1:7100"}"#,
        fixtures::path(&[b"hosts", b"listener"]).child(crate::Segment::Index(0)),
        What::Plaintext,
    );
}
fn a_worker_named_twice_is_refused() {
    fixtures::refuses(br#""name": "worker-b""#, br#""name": "worker-a""#, worker_path(1, b"name"), What::Unique);
}
fn an_engine_member_under_hosts_is_refused_as_unknown() {
    fixtures::refuses(
        br#""hosts": {"#,
        br#""hosts": {"engine": {},"#,
        fixtures::path(&[b"hosts", b"engine"]),
        What::Unknown,
    );
}
#[test]
fn files_list_each_digest_as_private() {
    let files = files();
    let mut digests = skein_lib::List::with_capacity(2);
    for file in &files {
        if file.path.ends_with(b".digest") {
            digests.push(file).unwrap();
        }
    }
    assert_eq!(digests.len(), 2);
    for file in &digests {
        assert!(file.private);
        assert_eq!(file.max, 65);
    }
}
#[test]
fn hosts_configuration_problems_name_the_invalid_fields() {
    table([
        ("a_workers_listener_on_a_public_address_is_refused", a_workers_listener_on_a_public_address_is_refused),
        ("a_worker_named_twice_is_refused", a_worker_named_twice_is_refused),
        ("an_engine_member_under_hosts_is_refused_as_unknown", an_engine_member_under_hosts_is_refused_as_unknown),
    ]);
}

type Case = (&'static str, fn());

fn table<const N: usize>(cases: [Case; N]) {
    let mut failures = [None; N];
    let mut passed = true;
    for ((name, case), failure) in cases.into_iter().zip(&mut failures) {
        if std::panic::catch_unwind(case).is_err() {
            *failure = Some(name);
            passed = false;
        }
    }
    assert!(passed, "{failures:?}");
}
