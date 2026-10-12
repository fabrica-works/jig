use super::fixtures;
use crate::What;
fn worker_path(index: u32, name: &[u8]) -> crate::Path {
    fixtures::path(&[b"hosts", b"workers"]).child(crate::Segment::Index(index)).member(name)
}
fn files() -> Box<[crate::File]> {
    crate::files(&fixtures::engine(fixtures::ENGINE_SMALL).unwrap())
}
#[test]
fn a_workers_listener_on_a_public_address_is_refused() {
    fixtures::refuses(
        br#"{"in_process": "workers"}"#,
        br#"{"socket": "192.0.2.1:7100"}"#,
        fixtures::path(&[b"hosts", b"listener"]).child(crate::Segment::Index(0)),
        What::Plaintext,
    );
}
#[test]
fn a_worker_named_twice_is_refused() {
    fixtures::refuses(br#""name": "worker-b""#, br#""name": "worker-a""#, worker_path(1, b"name"), What::Unique);
}
#[test]
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
fn handed_in(bytes: &[u8]) -> Result<crate::HandedIn, crate::Problems> {
    crate::handed_in(bytes, &fixtures::LIMITS)
}
#[test]
fn a_handed_in_file_reads_its_token_and_lapse() {
    let record = handed_in(br#"{"access_token":"secret","expires_at":2,"account_id":"person"}"#).unwrap();
    assert_eq!(record.token.expose(), b"secret");
    assert_eq!(record.lapses, skein_lib::Wall::from_nanos(2_000_000_000));
    assert_eq!(record.account_id.as_deref(), Some(&b"person"[..]));
}
#[test]
fn a_clients_listener_on_a_public_address_is_refused() {
    fixtures::refuses(
        b"127.0.0.1:7100",
        b"192.0.2.1:7100",
        fixtures::path(&[b"clients", b"listener"]),
        What::Plaintext,
    );
}
#[test]
fn a_handed_in_file_refuses_an_unknown_member() {
    let problems = handed_in(br#"{"access_token":"secret","expires_at":2,"absent":1}"#).unwrap_err();
    assert_eq!(problems.problems.get(0).unwrap().what, What::Unknown);
}
