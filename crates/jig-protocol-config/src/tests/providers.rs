use super::fixtures;
use crate::{Path, Problem, Segment, What};
use jig_domain_accounts as accounts;

const LLM: &[u8] = b"\"llm\": {\n";
const PROVIDER: &[u8] = br#"{"name": "Fake", "signed_in_plan": true, "manage_usage": "https://usage.example/", "number": 1, "models": ["small"], "sources": {"sign_in": {"issuer": "https://issuer.example", "client": "client-a", "scopes": ["chat", "offline_access"], "name_hint": "jig", "authorization": "https://issuer.example/authorize", "token": "https://issuer.example/token", "revocation": "https://issuer.example/revoke", "keys": "https://issuer.example/keys", "transport": "tls", "server_name": "issuer.example", "must_grant": ["chat"], "resource": "https://resource.example", "redirect_path": "callback"}, "keys": true, "programs": ["claude_code"]}}"#;

/// The small engine with `llm.providers` set to `list`.
fn with_providers(list: &[u8]) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(LLM.len().checked_add(list.len()).unwrap().checked_add(15).unwrap());
    writer.put(LLM).unwrap();
    writer.put(b"\"providers\": ").unwrap();
    writer.put(list).unwrap();
    writer.put(b",\n").unwrap();
    let base = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""providers": [{"name":"Fake","number":1,"models":["small"],"sources":{}}],"#,
        b"",
    );
    fixtures::with(&base, LLM, &writer.finish())
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
                name: b"Fake".as_slice().into(),
                signed_in_plan: true,
                manage_usage: Some(b"https://usage.example/".as_slice().into()),
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
fn a_model_without_a_provider_is_refused_when_providers_is_omitted() {
    let bytes = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""providers": [{"name":"Fake","number":1,"models":["small"],"sources":{}}],"#,
        b"",
    );
    first_problem(
        &bytes,
        fixtures::path(&[b"llm", b"models"]).child(Segment::Index(0)).member(b"name"),
        What::NoProvider,
    );
}
#[test]
fn a_model_without_a_provider_is_refused_when_providers_is_empty() {
    first_problem(
        &with_providers(b"[]"),
        fixtures::path(&[b"llm", b"models"]).child(Segment::Index(0)).member(b"name"),
        What::NoProvider,
    );
}
#[test]
fn no_models_need_no_providers() {
    // Parse just the LLM value: the deployment's charter still requires its model.
    for value in [br#"{"endpoints":[],"models":[]}"#.as_slice(), br#"{"providers":[],"endpoints":[],"models":[]}"#] {
        let doc = fixtures::document(value);
        let mut problems = crate::Problems::empty();
        let llm = crate::llm::parse(&doc, 0, &Path::root(), &crate::Directories::working(), &mut problems);
        assert!(llm.unwrap().providers.is_empty());
        assert!(problems.problems.is_empty());
    }
}
#[test]
fn a_provider_may_offer_no_source() {
    let bytes = one(br#"{"sign_in": {"issuer": "https://issuer.example", "client": "client-a", "scopes": ["chat", "offline_access"], "name_hint": "jig", "authorization": "https://issuer.example/authorize", "token": "https://issuer.example/token", "revocation": "https://issuer.example/revoke", "keys": "https://issuer.example/keys", "transport": "tls", "server_name": "issuer.example", "must_grant": ["chat"], "resource": "https://resource.example", "redirect_path": "callback"}, "keys": true, "programs": ["claude_code"]}"#, b"{}");
    let bytes = fixtures::with(&bytes, br#""signed_in_plan": true"#, br#""signed_in_plan": false"#);
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

#[test]
fn an_account_sign_in_registration_names_its_endpoints_and_its_must_grant_scopes() {
    let engine = fixtures::engine(&the_provider()).unwrap();
    let r = engine.llm.providers[0].sign_in.as_ref().unwrap();
    assert_eq!(&*r.authorization, b"https://issuer.example/authorize");
    assert_eq!(&*r.token, b"https://issuer.example/token");
    assert_eq!(r.revocation.as_deref(), Some(b"https://issuer.example/revoke".as_slice()));
    assert_eq!(&*r.keys, b"https://issuer.example/keys");
    assert_eq!(r.must_grant.as_ref(), [Box::<[u8]>::from(b"chat".as_slice())]);
    assert_eq!(r.name_hint.as_deref(), Some(b"jig".as_slice()));
    assert_eq!(r.resource.as_deref(), Some(b"https://resource.example".as_slice()));
    assert_eq!((&*r.redirect_path, r.redirect_port), (b"callback".as_slice(), None));
    match r.endpoint_transport {
        crate::Transport::Tls { .. } => (),
        crate::Transport::Plaintext => panic!("configured TLS account sign-in transport"),
    }
}
#[test]
fn an_account_sign_in_must_grant_scope_outside_the_scopes_is_refused_at_its_name() {
    first_problem(
        &one(br#""must_grant": ["chat"]"#, br#""must_grant": ["other"]"#),
        sources_path().member(b"sign_in").member(b"must_grant").child(Segment::Index(0)),
        What::Refers,
    );
}
#[test]
fn an_account_sign_in_registration_with_a_client_secret_is_still_refused() {
    a_client_secret_is_refused_naming_the_provider();
}
#[test]
fn a_profile_whose_driver_could_make_a_longer_account_visit_than_the_address_bound_is_refused() {
    let mut engine = fixtures::engine(fixtures::ENGINE_SMALL).unwrap();
    engine.limits.driver.client.url_bytes = accounts::ADDRESS + 1;
    let mut problems = crate::Problems::empty();
    crate::domain::whole(&engine, &mut problems);
    assert_eq!(
        problems.problems.get(0).unwrap(),
        &Problem { path: fixtures::path(&[b"limits", b"driver.url_bytes"]), what: What::Bound }
    );
}

#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary bounded configuration fixtures (programming-model.md, 10.2)")]
fn provider_terms_text_holds_at_each_byte_bound() {
    // Two UTF-8 bytes per character, filling the 256-byte name bound exactly.
    let name = b"\xc3\xa9".repeat(usize::try_from(accounts::PROVIDER_NAME).unwrap() / 2);
    let mut address = b"https://usage.example/".to_vec();
    address.resize(usize::try_from(accounts::MANAGE_USAGE).unwrap(), b'a');
    let bytes = one(b"Fake", &name);
    let bytes = fixtures::with(&bytes, b"https://usage.example/", &address);
    let configured = settings(&bytes);
    let provider = configured.providers.first().unwrap();
    assert_eq!(provider.name.as_ref(), name);
    assert_eq!(provider.manage_usage.as_deref(), Some(address.as_slice()));
}
#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary bounded configuration fixtures (programming-model.md, 10.2)")]
fn provider_terms_one_past_each_byte_bound_names_the_field() {
    for (field, old, bound, prefix) in [
        (b"name".as_slice(), b"Fake".as_slice(), accounts::PROVIDER_NAME, b"".as_slice()),
        (
            b"manage_usage".as_slice(),
            b"https://usage.example/".as_slice(),
            accounts::MANAGE_USAGE,
            b"https://usage.example/".as_slice(),
        ),
    ] {
        let mut value = prefix.to_vec();
        value.resize(usize::try_from(bound).unwrap() + 1, b'a');
        first_problem(&one(old, &value), provider_path(0).member(field), What::Bound);
    }
}
#[test]
fn provider_names_refuse_controls_at_their_field() {
    for value in [b"".as_slice(), b"\\u0001", b"\\u0085"] {
        first_problem(&one(b"Fake", value), provider_path(0).member(b"name"), What::Form);
    }
}
#[test]
fn provider_names_refuse_display_formatting() {
    for control in [
        b"\\u061c".as_slice(),
        b"\\u200e",
        b"\\u200f",
        b"\\u202a",
        b"\\u202b",
        b"\\u202c",
        b"\\u202d",
        b"\\u202e",
        b"\\u2066",
        b"\\u2067",
        b"\\u2068",
        b"\\u2069",
        b"\\u200b",
        b"\\ufeff",
    ] {
        first_problem(&one(b"Fake", control), provider_path(0).member(b"name"), What::Form);
    }
}
#[test]
fn manage_addresses_refuse_unsafe_forms_at_their_field() {
    for value in [
        b"http://usage.example/".as_slice(),
        b"https:///path",
        b"https://user:password@usage.example/",
        b"https://usage.example/?token=secret",
        b"https://usage.example/#token",
        b"HTTPS://usage.example/",
        b"https://usage.example/\xc3\xa9",
        b"https://usage.example/a b",
    ] {
        first_problem(&one(b"https://usage.example/", value), provider_path(0).member(b"manage_usage"), What::Form);
    }
}
#[test]
fn manage_addresses_accept_bracketed_ipv6() {
    let configured = settings(&one(b"https://usage.example/", b"https://[::1]:443/usage"));
    assert_eq!(configured.providers[0].manage_usage.as_deref(), Some(b"https://[::1]:443/usage".as_slice()));
}
#[test]
fn configured_signed_in_terms_can_be_false_without_affecting_program_offers() {
    let configured = settings(&one(br#""signed_in_plan": true"#, br#""signed_in_plan": false"#));
    let provider = configured.providers.first().unwrap();
    assert!(!provider.signed_in_plan);
    assert!(provider.plan(accounts::Kind::Program));
    assert_eq!(provider.manage_usage.as_deref(), Some(b"https://usage.example/".as_slice()));
}

#[test]
fn an_offered_sign_in_requires_explicit_plan_terms() {
    first_problem(&one(br#""signed_in_plan": true, "#, b""), provider_path(0).member(b"signed_in_plan"), What::Missing);
}
#[test]
fn plan_sources_require_a_manage_address_at_its_exact_path() {
    first_problem(
        &one(br#""manage_usage": "https://usage.example/", "#, b""),
        provider_path(0).member(b"manage_usage"),
        What::Missing,
    );
}
#[test]
fn a_provider_without_plan_offers_needs_no_plan_terms() {
    let bytes = with_providers(br#"[{"name":"Local", "number":1, "models":["small"], "sources":{}}]"#);
    let configured = settings(&bytes);
    let provider = configured.providers.first().unwrap();
    assert!(!provider.signed_in_plan);
    assert_eq!(provider.manage_usage, None);
}
#[test]
fn public_manage_addresses_refuse_malformed_authorities() {
    for address in [
        b"https://:".as_slice(),
        b"https://.example/",
        b"https://-bad.example/",
        b"https://usage.example:0/",
        b"https://usage.example:65536/",
        b"https://[]/",
    ] {
        first_problem(&one(b"https://usage.example/", address), provider_path(0).member(b"manage_usage"), What::Form);
    }
}

#[test]
fn non_plan_sign_ins_may_omit_the_manage_address() {
    let bytes = one(br#""signed_in_plan": true"#, br#""signed_in_plan": false"#);
    let bytes = fixtures::with(&bytes, br#""programs": ["claude_code"]"#, br#""programs": []"#);
    let bytes = fixtures::with(&bytes, br#""manage_usage": "https://usage.example/", "#, b"");
    let configured = settings(&bytes);
    let provider = configured.providers.first().unwrap();
    assert!(!provider.signed_in_plan);
    assert_eq!(provider.manage_usage, None);
}

#[test]
fn a_provider_without_sign_in_cannot_claim_a_signed_in_plan() {
    let bytes = with_providers(br#"[{"name":"Keys", "number":1, "models":["small"], "sources":{"keys":true}, "signed_in_plan":true, "manage_usage":"https://usage.example/"}]"#);
    first_problem(&bytes, provider_path(0).member(b"signed_in_plan"), What::Form);
}

#[test]
fn manage_addresses_accept_private_hosts() {
    for address in [b"https://localhost/".as_slice(), b"https://127.0.0.1/", b"https://10.0.0.1/"] {
        assert_eq!(
            settings(&one(b"https://usage.example/", address)).providers[0].manage_usage.as_deref(),
            Some(address)
        );
    }
}
