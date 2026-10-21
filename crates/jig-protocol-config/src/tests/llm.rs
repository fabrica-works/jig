use super::fixtures;
use crate::{What, files};

fn model(bytes: &[u8]) -> jig_domain_core::Model {
    fixtures::engine(bytes).unwrap().llm.models.first().unwrap().clone()
}
fn model_path(name: &[u8]) -> crate::Path {
    fixtures::path(&[b"llm", b"models"]).child(crate::Segment::Index(0)).member(name)
}
fn endpoint_path(name: &[u8]) -> crate::Path {
    fixtures::path(&[b"llm", b"endpoints"]).child(crate::Segment::Index(0)).member(name)
}
fn named_file<'a>(files: &'a [crate::File], name: &[u8]) -> &'a crate::File {
    for file in files {
        if file.path.as_ref() == name {
            return file;
        }
    }
    panic!("expected configured file {name:?}")
}
fn a_model_naming_an_unknown_endpoint_is_refused() {
    fixtures::refuses(br#""endpoint": "fake""#, br#""endpoint": "nowhere""#, model_path(b"endpoint"), What::Refers);
}
#[test]
fn an_endpoint_account_is_unknown_at_its_exact_path() {
    fixtures::refuses(br#""dialect": 0,"#, br#""dialect": 0, "account": 1,"#, endpoint_path(b"account"), What::Unknown);
}
fn two_endpoints_with_one_name_are_refused() {
    let endpoint = br#"{"name":"fake","number":2,"dialect":0,"provider":"fake","address":{"socket":"127.0.0.1:7102"},"transport":"plaintext","headers":[],"connect_ms":10000,"handshake_ms":10000}"#;
    let mut writer = skein_lib::Writer::new(
        endpoint
            .len()
            .checked_add(b"\"handshake_ms\": 10000\n},".len())
            .expect("small endpoint fixture")
            .checked_add(2)
            .expect("small endpoint fixture"),
    );
    writer.put(b"\"handshake_ms\": 10000\n},").unwrap();
    writer.put(endpoint).unwrap();
    writer.put(b"],").unwrap();
    let added = writer.finish();
    fixtures::refuses(
        b"\"handshake_ms\": 10000\n}],",
        &added,
        fixtures::path(&[b"llm", b"endpoints"]).child(crate::Segment::Index(1)).member(b"name"),
        What::Unique,
    );
}
fn plaintext_to_a_public_address_is_refused() {
    fixtures::refuses(b"127.0.0.1:7101", b"192.0.2.1:7101", endpoint_path(b"address"), What::Plaintext);
}
#[test]
fn configured_accounts_are_unknown_at_their_exact_path() {
    fixtures::refuses(
        br#""llm": {"#,
        br#""llm": {"accounts":[{"number":1,"source":{"handed_in":{"file":"old.json"}}}],"#,
        fixtures::path(&[b"llm", b"accounts"]),
        What::Unknown,
    );
}
#[test]
fn a_zero_price_or_unit_is_refused_at_its_path() {
    let cases: &[(&[u8], &[u8], &[u8])] = &[
        (br#""input": 1"#, br#""input": 0"#, b"input"),
        (br#""cached": 1"#, br#""cached": 0"#, b"cached"),
        (br#""output": 1,"#, br#""output": 0,"#, b"output"),
        (br#""unit": 1000"#, br#""unit": 0"#, b"unit"),
        (br#""unit": 1000"#, br#""unit": 1000, "cache_write": 0"#, b"cache_write"),
    ];
    for (from, to, member) in cases {
        fixtures::refuses(from, to, model_path(b"prices").member(member), What::Bound);
    }
}
#[test]
fn a_cache_write_price_may_be_left_out() {
    assert_eq!(model(fixtures::ENGINE_SMALL).cache_write_price, None);
}
#[test]
fn a_cache_write_price_is_kept_when_named() {
    let bytes = fixtures::with(fixtures::ENGINE_SMALL, br#""unit": 1000"#, br#""unit": 1000, "cache_write": 3"#);
    let bytes = fixtures::with(&bytes, br#""spend": 20"#, br#""spend": 60"#);
    assert_eq!(model(&bytes).cache_write_price, Some(3));
}
fn with_description(description: &str) -> Box<[u8]> {
    let member = format!(r#""output": 1024, "description": {description},"#);
    fixtures::with(fixtures::ENGINE_SMALL, br#""output": 1024,"#, member.as_bytes())
}
#[test]
fn a_model_description_holds_512_bytes_of_text() {
    // 256 two-byte characters reach MODEL_DESCRIPTION_BYTES without counting characters.
    let description = "é".repeat(256);
    let bytes = with_description(&format!("\"{description}\""));
    let engine = fixtures::engine(&bytes).unwrap();
    assert_eq!(engine.llm.descriptions.as_ref(), [Some(description.into_bytes().into_boxed_slice())]);
}
#[test]
fn a_model_without_a_description_keeps_none() {
    let engine = fixtures::engine(fixtures::ENGINE_SMALL).unwrap();
    assert_eq!(engine.llm.descriptions.as_ref(), [None]);
}
#[test]
fn invalid_model_descriptions_are_refused_at_their_path() {
    // One byte beyond MODEL_DESCRIPTION_BYTES.
    let long = format!("\"{}\"", "d".repeat(513));
    for (description, what) in [(long.as_str(), What::Bound), (r#""""#, What::Form), ("7", What::Kind)] {
        let problems = fixtures::engine(&with_description(description)).unwrap_err();
        assert_eq!(problems.problems.get(0), Some(&crate::Problem { path: model_path(b"description"), what }));
    }
}
#[test]
fn a_later_model_keeps_its_description_in_order() {
    let bytes = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""unit": 1000}
}]"#,
        br#""unit": 1000}
}, {"name":"later","description":"For later work","endpoint":"fake","window":8192,"output":1024,
"prices":{"input":1,"cached":1,"output":1,"unit":1000}}]"#,
    );
    let bytes = fixtures::with(&bytes, br#""models":["small"]"#, br#""models":["small","later"]"#);
    let engine = fixtures::engine(&bytes).unwrap();
    let names: Vec<_> = engine.llm.models.iter().map(|model| model.name.as_ref()).collect();
    assert_eq!(names, [b"small".as_slice(), b"later".as_slice()]);
    assert_eq!(engine.llm.descriptions.as_ref(), [None, Some(Box::from(b"For later work".as_slice()))]);
}
#[test]
fn a_model_takes_the_efforts_it_names_in_order() {
    let bytes = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""output": 1024,"#,
        br#""output": 1024, "efforts": ["low", "high"],"#,
    );
    let low: Box<[u8]> = Box::from(&b"low"[..]);
    let high: Box<[u8]> = Box::from(&b"high"[..]);
    assert_eq!(model(&bytes).efforts.as_ref(), [low, high]);
}
#[test]
fn a_model_naming_no_efforts_takes_none() {
    assert!(model(fixtures::ENGINE_SMALL).efforts.is_empty());
}
fn refuses_efforts(efforts: &[u8], path: crate::Path, what: What) {
    let head = br#""output": 1024, "efforts": "#;
    let mut writer = skein_lib::Writer::new(head.len().checked_add(efforts.len()).unwrap().checked_add(1).unwrap());
    writer.put(head).unwrap();
    writer.put(efforts).unwrap();
    writer.put(b",").unwrap();
    fixtures::refuses(br#""output": 1024,"#, &writer.finish(), path, what);
}
fn effort_path(index: u32) -> crate::Path {
    model_path(b"efforts").child(crate::Segment::Index(index))
}
fn more_efforts_than_the_bound_are_refused() {
    let efforts = br#"["a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q"]"#;
    refuses_efforts(efforts, model_path(b"efforts"), What::Bound);
}
fn an_effort_longer_than_smiths_bound_is_refused() {
    refuses_efforts(br#"["low", "an-effort-named-past-thirty-two-bytes"]"#, effort_path(1), What::Bound);
}
fn an_empty_effort_is_refused() {
    refuses_efforts(br#"["low", ""]"#, effort_path(1), What::Form);
}
fn a_repeated_effort_is_refused() {
    refuses_efforts(br#"["low", "high", "low"]"#, effort_path(2), What::Unique);
}
fn an_effort_that_is_not_a_name_is_refused() {
    refuses_efforts(br#"["low", 2]"#, effort_path(1), What::Kind);
}
fn efforts_that_are_not_a_list_are_refused() {
    refuses_efforts(br#""low""#, model_path(b"efforts"), What::Kind);
}
#[test]
fn a_model_uses_its_endpoint() {
    let model = model(fixtures::ENGINE_SMALL);
    assert_eq!((model.endpoint, model.dialect), (1, 0));
}

#[test]
fn a_tls_endpoint_names_its_trust_file() {
    let bytes = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""transport": "plaintext""#,
        br#""transport":"tls","server_name":"issuer.example","trust":"certs/trust.der""#,
    );
    let files = files(&fixtures::engine(&bytes).unwrap());
    assert!(!named_file(&files, b"certs/trust.der").private);
}
fn an_in_process_llm_endpoint_is_refused() {
    fixtures::refuses(
        br#"{"socket": "127.0.0.1:7101"}"#,
        br#"{"in_process": "llm"}"#,
        endpoint_path(b"address"),
        What::Form,
    );
}
#[test]
fn endpoint_headers_are_retained() {
    let bytes = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""headers": []"#,
        br#""headers": [{"name":"x-client","value":"test"}]"#,
    );
    let engine = fixtures::engine(&bytes).unwrap();
    let header = engine.llm.endpoints.first().unwrap().headers.first().unwrap();
    assert_eq!((header.name.as_ref(), header.value.as_ref()), (&b"x-client"[..], &b"test"[..]));
}

#[test]
fn llm_configuration_problems_name_the_invalid_fields() {
    table([
        ("a_model_naming_an_unknown_endpoint_is_refused", a_model_naming_an_unknown_endpoint_is_refused),
        ("two_endpoints_with_one_name_are_refused", two_endpoints_with_one_name_are_refused),
        ("plaintext_to_a_public_address_is_refused", plaintext_to_a_public_address_is_refused),
        ("an_in_process_llm_endpoint_is_refused", an_in_process_llm_endpoint_is_refused),
        ("more_efforts_than_the_bound_are_refused", more_efforts_than_the_bound_are_refused),
        ("an_effort_longer_than_smiths_bound_is_refused", an_effort_longer_than_smiths_bound_is_refused),
        ("an_empty_effort_is_refused", an_empty_effort_is_refused),
        ("a_repeated_effort_is_refused", a_repeated_effort_is_refused),
        ("an_effort_that_is_not_a_name_is_refused", an_effort_that_is_not_a_name_is_refused),
        ("efforts_that_are_not_a_list_are_refused", efforts_that_are_not_a_list_are_refused),
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
