//! The shipped service and agent, killed and restarted over one durable store.
mod support;
use jig_client_protocol_world::native::Script;

/// The provider, answering one chat to a run lent the handed-in file's
/// token as a bearer.
fn peer() -> skein_fake_peers::llm::Peer {
    let mut script = jig_channel_world::fixtures::one_chat(b"Hello");
    script.result = br#"{"form":"report","text":"Process answer","fields":{}}"#.as_slice().into();
    jig_channel_world::fixtures::provider(
        Box::new([skein_fake_llm_domain::api::Script {
            cue: Box::default(),
            turns: jig_channel_world::fixtures::provider_turns(&script),
        }]),
        skein_llm::Credential::bearer(b"token".as_slice().into()),
        skein_lib::Duration::ZERO,
        skein_fake_peers::Transport::Tls,
        skein_fake_llm_protocol::documents::Provider::OpenAi,
    )
}

#[test]
fn a_killed_jig_test_restarts_and_the_work_is_found() {
    let directory = support::scratch();
    support::configuration(directory.path());
    assert_eq!(support::run(directory.path(), Script::answered_chat(b"Hello", support::ANSWER), Some(peer())), (1, 1));
    assert_eq!(support::run(directory.path(), Script::found_chat(support::ANSWER), None), (1, 0));
}

/// llm.md 5.1 and store.md 9: the first start keeps the handed-in record in
/// the secrets' store. The next names the account a sign-in source whose
/// issuer nobody serves, and the handed-in file is gone: only the record
/// the store kept can lend the token the provider expects.
#[test]
fn sign_in_state_survives_a_restart_through_the_store() {
    let directory = support::scratch();
    support::configuration(directory.path());
    assert_eq!(support::run(directory.path(), Script::answered_chat(b"Hello", support::ANSWER), Some(peer())), (1, 1));
    let path = directory.path().join("config.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("deployment JSON")).expect("deployment JSON");
    value["engine"]["llm"]["accounts"][0]["source"] = serde_json::json!({"sign_in":{
        "client":"client","authorization":"http://127.0.0.1:7101/authorize","token":"http://127.0.0.1:7101/token",
        "endpoint":{"socket":"127.0.0.1:7101"},"transport":"plaintext","redirect":"http://localhost:7123/callback",
        "scope":"read"}});
    std::fs::write(&path, serde_json::to_vec(&value).expect("deployment JSON")).expect("deployment JSON");
    std::fs::remove_file(directory.path().join("accounts/chat.json")).expect("handed-in file");
    assert_eq!(support::run(directory.path(), Script::answered_chat(b"Hello", support::ANSWER), Some(peer())), (1, 1));
}

#[path = "support/browser.rs"]
mod browser;
#[test]
fn jig_test_serves_its_bundle_s_page_under_the_reference_policy() {
    let directory = support::scratch();
    browser::configuration(directory.path(), &browser::fixture_bundle(directory.path()));
    let mut binary = browser::start(directory.path());
    let link = browser::printed_link_line(&mut binary);
    let (status, headers, body) = browser::get(&link, "/tasks/7");
    assert_eq!(status, 200);
    assert_eq!(browser::field(&headers, "content-security-policy"), Some(browser::POLICY));
    assert_eq!(body, browser::PAGE);
}

#[test]
fn a_party_signs_in_through_a_client_and_opens_the_address_at_the_issuer() {
    let directory = support::scratch();
    accounts::configuration(directory.path());
    accounts::run(directory.path());
}

// Reuse the account sign-in world's browser source; it remains a private test helper.
use jig_protocol_accounts_world::fixtures;
#[path = "../../../tests/protocol-accounts/src/visitor.rs"]
mod account_sign_in_visitor;

#[path = "support/accounts.rs"]
mod accounts;

#[path = "support/account_keys.rs"]
mod account_keys;
#[test]
fn a_party_adds_replaces_and_removes_a_key() {
    let directory = support::scratch();
    account_keys::configuration(directory.path());
    account_keys::run(directory.path());
}
