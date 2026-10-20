use super::fixtures;
use crate::{Path, Problem, Segment, What};
use jig_domain_accounts as accounts;

const LLM: &[u8] = b"\"llm\": {\n";
const PROVIDER: &[u8] = br#"{"name": "Fake", "number": 1, "models": ["small"], "sources": {"sign_in": {"issuer": "https://issuer.example", "client": "client-a", "scopes": ["chat", "offline_access"]}, "keys": true, "programs": ["claude_code"]}}"#;

/// The small engine with `llm.providers` set to `list`.
fn with_providers(list: &[u8]) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(LLM.len().checked_add(list.len()).unwrap().checked_add(15).unwrap());
    writer.put(LLM).unwrap();
    writer.put(b"\"providers\": ").unwrap();
    writer.put(list).unwrap();
    writer.put(b",\n").unwrap();
    fixtures::with(fixtures::ENGINE_SMALL, LLM, &writer.finish())
}
/// One provider: `PROVIDER` with `from` replaced by `to`.
fn one(from: &[u8], to: &[u8]) -> Box<[u8]> {
    let provider = fixtures::with(PROVIDER, from, to);
    let mut writer = skein_lib::Writer::new(provider.len().checked_add(2).unwrap());
    writer.put(b"[").unwrap();
    writer.put(&provider).unwrap();
    writer.put(b"]").unwrap();
    with_providers(&writer.finish())
}
fn the_provider() -> Box<[u8]> {
    one(br#""number": 1"#, br#""number": 1"#)
}
fn provider_path(index: u32) -> Path {
    fixtures::path(&[b"llm", b"providers"]).child(Segment::Index(index))
}
fn sources_path() -> Path {
    provider_path(0).member(b"sources")
}
fn first_problem(bytes: &[u8], path: Path, what: What) {
    let problems = fixtures::engine(bytes).unwrap_err();
    assert_eq!(problems.problems.get(0), Some(&Problem { path, what }));
}
fn settings(bytes: &[u8]) -> accounts::Providers {
    let engine = fixtures::engine(bytes).unwrap();
    crate::core(&engine, [0; 16], 7).settings.llm_providers
}

#[test]
fn providers_reach_the_accounts_settings() {
    let engine = fixtures::engine(&the_provider()).unwrap();
    let provider = engine.llm.providers.first().unwrap();
    let registration = provider.sign_in.as_ref().unwrap();
    assert_eq!(
        (provider.name.as_ref(), registration.issuer.as_ref(), registration.client.as_ref()),
        (&b"Fake"[..], &b"https://issuer.example"[..], &b"client-a"[..])
    );
    assert_eq!(registration.scopes.as_ref(), [Box::from(&b"chat"[..]), Box::from(&b"offline_access"[..])]);
    assert_eq!(
        settings(&the_provider()),
        accounts::Providers {
            providers: Box::new([accounts::Provider {
                number: 1,
                models: Box::new([Box::from(&b"small"[..])]),
                offers: accounts::Offers {
                    sign_in: true,
                    key: true,
                    programs: Box::new([Box::from(&b"claude_code"[..])])
                },
            }]),
        }
    );
}
#[test]
fn no_providers_section_and_an_empty_one_name_none() {
    assert_eq!(settings(fixtures::ENGINE_SMALL), accounts::Providers::none());
    assert_eq!(settings(&with_providers(b"[]")), accounts::Providers::none());
}
#[test]
fn a_provider_may_offer_no_source() {
    let bytes = one(br#"{"sign_in": {"issuer": "https://issuer.example", "client": "client-a", "scopes": ["chat", "offline_access"]}, "keys": true, "programs": ["claude_code"]}"#, b"{}");
    let providers = settings(&bytes);
    assert_eq!(
        providers.providers.first().unwrap().offers,
        accounts::Offers { sign_in: false, key: false, programs: Box::new([]) }
    );
}

fn a_client_secret_is_refused_naming_the_provider() {
    first_problem(
        &one(br#""client": "client-a""#, br#""client": "client-a", "secret": "secrets/client""#),
        sources_path().member(b"sign_in").member(b"secret"),
        What::Confidential,
    );
}
fn a_provider_naming_an_unknown_model_is_refused() {
    first_problem(
        &one(br#""models": ["small"]"#, br#""models": ["large"]"#),
        provider_path(0).member(b"models").child(Segment::Index(0)),
        What::Refers,
    );
}
fn a_model_in_no_provider_is_refused() {
    first_problem(
        &one(br#""models": ["small"]"#, br#""models": []"#),
        fixtures::path(&[b"llm", b"models"]).child(Segment::Index(0)).member(b"name"),
        What::NoProvider,
    );
}
fn a_model_in_two_providers_is_refused() {
    let second = fixtures::with(PROVIDER, br#""number": 1"#, br#""number": 2"#);
    let second = fixtures::with(&second, br#""name": "Fake""#, br#""name": "Other""#);
    let mut writer = skein_lib::Writer::new(PROVIDER.len().checked_add(second.len()).unwrap().checked_add(3).unwrap());
    writer.put(b"[").unwrap();
    writer.put(PROVIDER).unwrap();
    writer.put(b",").unwrap();
    writer.put(&second).unwrap();
    writer.put(b"]").unwrap();
    first_problem(
        &with_providers(&writer.finish()),
        provider_path(1).member(b"models").child(Segment::Index(0)),
        What::Unique,
    );
}
fn a_program_name_outside_its_form_is_refused() {
    first_problem(
        &one(br#""programs": ["claude_code"]"#, br#""programs": ["Claude Code"]"#),
        sources_path().member(b"programs").child(Segment::Index(0)),
        What::Form,
    );
}
fn a_program_name_past_32_bytes_is_refused() {
    first_problem(
        &one(br#""programs": ["claude_code"]"#, br#""programs": ["abcdefghijklmnopqrstuvwxyz0123456"]"#),
        sources_path().member(b"programs").child(Segment::Index(0)),
        What::Form,
    );
}
fn a_scope_with_a_space_is_refused() {
    first_problem(
        &one(br#""scopes": ["chat", "#, br#""scopes": ["chat read", "#),
        sources_path().member(b"sign_in").member(b"scopes").child(Segment::Index(0)),
        What::Form,
    );
}
fn an_unknown_source_is_refused() {
    first_problem(
        &one(br#""keys": true"#, br#""keys": true, "handed_in": true"#),
        sources_path().member(b"handed_in"),
        What::Unknown,
    );
}

#[test]
fn provider_problems_name_the_provider_and_field() {
    table([
        ("a_client_secret_is_refused_naming_the_provider", a_client_secret_is_refused_naming_the_provider),
        ("a_provider_naming_an_unknown_model_is_refused", a_provider_naming_an_unknown_model_is_refused),
        ("a_model_in_no_provider_is_refused", a_model_in_no_provider_is_refused),
        ("a_model_in_two_providers_is_refused", a_model_in_two_providers_is_refused),
        ("a_program_name_outside_its_form_is_refused", a_program_name_outside_its_form_is_refused),
        ("a_program_name_past_32_bytes_is_refused", a_program_name_past_32_bytes_is_refused),
        ("a_scope_with_a_space_is_refused", a_scope_with_a_space_is_refused),
        ("an_unknown_source_is_refused", an_unknown_source_is_refused),
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
