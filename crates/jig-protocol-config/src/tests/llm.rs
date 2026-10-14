use super::fixtures;
use crate::{What, files};

fn model(bytes: &[u8]) -> jig_domain_core::Model {
    fixtures::engine(bytes).unwrap().llm.models.first().unwrap().clone()
}
fn engine() -> crate::Engine {
    fixtures::engine(fixtures::ENGINE_SMALL).unwrap()
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
fn an_endpoint_naming_an_unknown_account_is_refused() {
    fixtures::refuses(br#""account": 1"#, br#""account": 2"#, endpoint_path(b"account"), What::Refers);
}
fn two_endpoints_with_one_name_are_refused() {
    let endpoint = br#"{"name":"fake","number":2,"dialect":0,"account":1,"provider":"fake","address":{"socket":"127.0.0.1:7102"},"transport":"plaintext","headers":[],"connect_ms":10000,"handshake_ms":10000}"#;
    let mut writer = skein_lib::Writer::new(endpoint.len().checked_add(4).expect("small endpoint fixture"));
    writer.put(b"},").unwrap();
    writer.put(endpoint).unwrap();
    writer.put(b"],").unwrap();
    let added = writer.finish();
    fixtures::refuses(
        b"}],",
        &added,
        fixtures::path(&[b"llm", b"endpoints"]).child(crate::Segment::Index(1)).member(b"name"),
        What::Unique,
    );
}
fn plaintext_to_a_public_address_is_refused() {
    fixtures::refuses(b"127.0.0.1:7101", b"192.0.2.1:7101", endpoint_path(b"address"), What::Plaintext);
}
#[test]
fn a_handed_in_account_names_its_private_file() {
    let files = files(&engine());
    let file = named_file(&files, b"accounts/chat.json");
    assert!(file.private);
    assert_eq!(
        file.at,
        fixtures::path(&[b"llm", b"accounts"])
            .child(crate::Segment::Index(0))
            .member(b"source")
            .member(b"handed_in")
            .member(b"file")
    );
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
#[test]
fn a_model_uses_its_endpoints_number_dialect_and_account() {
    let model = model(fixtures::ENGINE_SMALL);
    assert_eq!((model.endpoint, model.dialect, model.account), (1, 0, 1));
}

const SIGN_IN: &[u8] = br#"{"sign_in":{"client":"client-a","secret":"secrets/client","authorization":"https://issuer.example/authorize","token":"https://issuer.example/token","endpoint":{"socket":"127.0.0.1:7102"},"transport":"plaintext","redirect":"http://127.0.0.1:7103/callback","scope":"chat"}}"#;
fn signed_engine(source: &[u8]) -> crate::Engine {
    let bytes = fixtures::with(fixtures::ENGINE_SMALL, br#"{"handed_in": {"file": "accounts/chat.json"}}"#, source);
    fixtures::engine(&bytes).unwrap()
}
#[test]
fn a_sign_in_account_names_its_private_client_secret() {
    let files = files(&signed_engine(SIGN_IN));
    assert!(named_file(&files, b"secrets/client").private);
}
fn a_sign_in_plaintext_refusal_names_its_endpoint() {
    let source = fixtures::with(SIGN_IN, b"127.0.0.1:7102", b"192.0.2.1:7102");
    fixtures::refuses(
        br#"{"handed_in": {"file": "accounts/chat.json"}}"#,
        &source,
        fixtures::path(&[b"llm", b"accounts"])
            .child(crate::Segment::Index(0))
            .member(b"source")
            .member(b"sign_in")
            .member(b"endpoint"),
        What::Plaintext,
    );
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
        ("an_endpoint_naming_an_unknown_account_is_refused", an_endpoint_naming_an_unknown_account_is_refused),
        ("two_endpoints_with_one_name_are_refused", two_endpoints_with_one_name_are_refused),
        ("plaintext_to_a_public_address_is_refused", plaintext_to_a_public_address_is_refused),
        ("a_sign_in_plaintext_refusal_names_its_endpoint", a_sign_in_plaintext_refusal_names_its_endpoint),
        ("an_in_process_llm_endpoint_is_refused", an_in_process_llm_endpoint_is_refused),
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
