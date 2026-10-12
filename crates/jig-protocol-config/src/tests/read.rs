use super::fixtures::{self, document, path};
use crate::{Address, DocumentLimits, Path, Problem, Problems, What, address, read};
use skein_io::Addr;
use skein_lib::Duration;

fn object_problems(bytes: &[u8], required: &[&[u8]]) -> Problems {
    let doc = document(bytes);
    let mut object = read::Object::new(&doc, 0).expect("fixture object");
    let mut problems = Problems::empty();
    let path = path(&[b"hosts"]);
    for name in required {
        let _at = object.required(&doc, name, &path, &mut problems);
    }
    object.finish(&doc, &path, &mut problems);
    problems
}
fn address(bytes: &[u8]) -> Result<Address, Problems> {
    let doc = document(bytes);
    let mut problems = Problems::empty();
    let value = address::read(&doc, 0, &Path::root(), &mut problems);
    if problems.is_empty() { Ok(value.expect("address produced without problems")) } else { Err(problems) }
}
fn one_problem(problems: Problems) -> Problem {
    assert_eq!(problems.problems.len(), 1);
    assert_eq!(problems.past, 0);
    problems.problems.get(0).expect("one problem").clone()
}
#[test]
fn an_unknown_member_is_refused_naming_its_path() {
    let problems = object_problems(br#"{"mystery": {"nested": [1, 2]}}"#, &[]);
    assert_eq!(one_problem(problems), Problem { path: path(&[b"hosts", b"mystery"]), what: What::Unknown });
}
#[test]
fn a_member_named_twice_is_refused() {
    let problems = object_problems(br#"{"name":"a", "name":"b"}"#, &[b"name"]);
    assert_eq!(one_problem(problems), Problem { path: path(&[b"hosts", b"name"]), what: What::Duplicate });
}
#[test]
fn a_missing_member_is_refused() {
    assert_eq!(
        one_problem(object_problems(b"{}", &[b"name"])),
        Problem { path: path(&[b"hosts", b"name"]), what: What::Missing }
    );
}
#[test]
fn an_integer_past_its_bound_is_refused() {
    assert_eq!(read::integer(&document(b"65536"), 0, 65535), Err(What::Bound));
}
#[test]
fn a_duration_reads_as_milliseconds() {
    assert_eq!(read::duration(&document(b"1250"), 0), Ok(Duration::from_millis(1250)));
}
#[test]
fn an_address_with_two_forms_is_refused() {
    let problems = address(br#"{"socket":"127.0.0.1:7100", "in_process":"workers"}"#).expect_err("two forms");
    assert_eq!(one_problem(problems), Problem { path: Path::root(), what: What::Form });
}
#[test]
fn an_in_process_name_past_its_bound_is_refused() {
    let problems = address(br#"{"in_process":"abcdefghijklmnopqrstuvwxyz0123456"}"#).expect_err("33-byte name");
    assert_eq!(one_problem(problems), Problem { path: path(&[b"in_process"]), what: What::Bound });
}
#[test]
fn an_ipv6_socket_address_is_read() {
    let expected = core::net::SocketAddr::from((core::net::Ipv6Addr::LOCALHOST, 7100));
    assert_eq!(address(br#"{"socket":"[::1]:7100"}"#), Ok(Address::Addr(Addr::Ip(expected))));
}
#[test]
fn problems_past_the_bound_are_counted() {
    let mut problems = Problems::empty();
    for _ in 0_u32..35 {
        problems.push(Path::root(), What::Bound);
    }
    assert_eq!(problems.problems.len(), 32);
    assert_eq!(problems.past, 3);
}
#[test]
fn a_list_past_its_count_is_refused() {
    assert_eq!(read::list(&document(b"[1, {\"child\": true}, 3]"), 0, 2), Err(What::Bound));
}
#[test]
fn a_boolean_is_read() {
    assert_eq!(read::boolean(&document(b"false"), 0), Ok(false));
}
#[test]
fn a_text_past_its_byte_bound_is_refused() {
    assert_eq!(read::text(&document(br#""abcd""#), 0, 3), Err(What::Bound));
}
#[test]
fn a_document_past_its_token_bound_is_refused() {
    let bounds = DocumentLimits { tokens: 2, ..fixtures::LIMITS };
    assert_eq!(crate::document(b"[1]", &bounds), Err(Problem { path: Path::root(), what: What::Json }));
}
#[test]
fn trailing_document_bytes_are_refused() {
    assert_eq!(crate::document(b"{} true", &fixtures::LIMITS), Err(Problem { path: Path::root(), what: What::Json }));
}
#[test]
fn a_host_address_is_read_with_its_port() {
    assert_eq!(
        address(br#"{"host":"issuer.example","port":443}"#),
        Ok(Address::Host { name: skein_lib::bytes::copy_of(b"issuer.example"), port: 443 })
    );
}

#[test]
fn a_host_with_two_invalid_members_reports_both() {
    let problems = address(br#"{"host":false,"port":65536}"#).expect_err("invalid host and port");
    assert_eq!(
        problems.problems.as_slice(),
        &[Problem { path: path(&[b"host"]), what: What::Kind }, Problem { path: path(&[b"port"]), what: What::Bound }]
    );
}
