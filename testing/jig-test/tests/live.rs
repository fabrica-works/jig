//! Real accounts and model answers through the shipped service and native client.
#[expect(dead_code, reason = "shared process support; live uses the account sign-in and explicit D2 entries")]
mod support;
use jig_client_protocol_world::native::Script;
use jig_protocol_store::codec;
use live_sign_in::{INSTRUCTIONS, WINDOW, WORDS, capture, completions, store_limits};
use skein_lib::Duration;
use skein_shell::Clock;
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

#[path = "support/live_sign_in.rs"]
mod live_sign_in;

fn configuration(directory: &Path, source: &Path) -> codec::Limits {
    let settings: serde_json::Value =
        serde_json::from_slice(&fs::read(source).expect("live settings")).expect("public live settings JSON");
    let limits = configuration_at(
        directory,
        settings["model"].as_str().expect("live model"),
        settings["address"].as_str().expect("live address"),
    );
    let raw = fs::read(settings["account_file"].as_str().expect("D2 record path")).expect("D2 handed-in fixture");
    support::d2::preseed(directory, &raw);
    limits
}

fn configuration_at(directory: &Path, model: &str, address: &str) -> codec::Limits {
    support::configuration(directory);
    let path = directory.join("config.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("fixture configuration")).expect("fixture JSON");
    let engine = &mut value["engine"];
    engine["llm"]["providers"][0]["models"][0] = model.into();
    let endpoint = &mut engine["llm"]["endpoints"][0];
    endpoint["provider"] = "codex".into();
    endpoint["dialect"] = 1.into();
    endpoint["address"] = serde_json::json!({"socket":address});
    endpoint["transport"] = "tls".into();
    endpoint["server_name"] = "chatgpt.com".into();
    engine["llm"]["models"][0]["name"] = model.into();
    engine["llm"]["models"][0]["output"] = 4096.into();
    engine["llm"]["models"][0]["efforts"] = serde_json::json!(["low"]);
    engine["llm"]["models"][0]["window"] = WINDOW.into();
    engine["charters"][0]["model"] = model.into();
    engine["charters"][0]["turns"] = 5.into();
    engine["charters"][0]["time_ms"] = 240_000.into();
    engine["charters"][0]["waiting_ms"] = 10_000.into();
    let bytes = serde_json::to_vec(&value).expect("live deployment JSON");
    fs::write(&path, &bytes).expect("live deployment configuration");
    let limits = store_limits(&bytes);
    let path = directory.join("agent.json");
    let mut agent: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("agent fixture")).expect("agent JSON");
    // The 64 KiB reasoning item and complete trace exceed standard's memory ceiling.
    agent["profile"]["declared"]["memory"] = (3_u64 << 30).into();
    // The live record names its account: the withdrawn Codex arm, kept for this test only.
    let endpoint = &mut agent["endpoints"][0];
    endpoint["provider"] = "codex".into();
    endpoint["dialect"] = 1.into();
    endpoint["address"] = address.into();
    endpoint["server_name"] = "chatgpt.com".into();
    endpoint.as_object_mut().expect("endpoint").remove("trust_der");
    endpoint["connect"] = 10_000.into();
    endpoint["handshake"] = 10_000.into();
    endpoint["headers"] = serde_json::json!(skein_llm::openai::identity::headers().iter().map(|header|
        serde_json::json!({"name":String::from_utf8_lossy(&header.name),"value":String::from_utf8_lossy(&header.value)})
    ).collect::<Vec<_>>());
    endpoint["models"][0]["name"] = model.into();
    endpoint["models"][0]["output"] = 4096.into();
    endpoint["models"][0]["efforts"] = serde_json::json!(["low"]);
    endpoint["models"][0]["reasoning_item"] = 65_536.into();
    endpoint["models"][0]["window"] = WINDOW.into();
    endpoint["models"][0]["idle"] = 300_000.into();
    agent["trace"] =
        serde_json::json!({"path":directory.join("agent-trace.jsonl"),"capture":"everything","delivery":"complete"});
    fs::write(path, serde_json::to_vec(&agent).expect("live agent JSON")).expect("live agent configuration");
    fs::write(directory.join("charters/chat.txt"), INSTRUCTIONS.as_bytes()).expect("live charter");
    limits
}

/// Both live tests' configurations, read as jig-test's startup reads them: no network, no person.
#[test]
fn the_live_configurations_are_ones_jig_test_reads() {
    let sign_in = support::scratch();
    live_sign_in::configuration_at(sign_in.path(), "192.0.2.1:443", live_sign_in::Peers::Real);
    assert_eq!(jig_test::check(&sign_in.path().join("config.json")), Ok(()));
    let fake = support::scratch();
    live_sign_in::configuration_at(fake.path(), "127.0.0.1:34443", live_sign_in::Peers::Fake);
    assert_eq!(jig_test::check(&fake.path().join("config.json")), Ok(()));
    let d2 = support::scratch();
    configuration_at(d2.path(), "gpt-6-luna", "192.0.2.1:443");
    assert_eq!(jig_test::check(&d2.path().join("config.json")), Ok(()));
}

#[test]
fn a_native_client_s_chat_is_answered_by_a_real_model() {
    let source = std::env::var_os("JIG_LIVE_CONFIG").expect("the live proof script supplies configuration");
    let directory = support::scratch();
    let limits = configuration(directory.path(), Path::new(&source));
    let clock = Clock::new();
    let started = clock.now().now;
    let result = std::panic::catch_unwind(|| {
        support::run_d2(
            directory.path(),
            Script::answered_report(WORDS.as_bytes(), support::ANSWER, Duration::from_secs(240)),
            None,
        )
    });
    let directory = directory.keep();
    let count = completions(&directory, &limits);
    let wall_ms = clock.now().now.saturating_since(started).as_nanos() / 1_000_000;
    eprintln!("live completions={count}; wall={wall_ms}ms; state={}", directory.display());
    let captured = capture(&directory, &Path::new(&source).with_file_name("live-request.json"));
    let outcome = result.unwrap_or_else(|failed| {
        eprintln!("live failed state retained at {}", directory.display());
        std::panic::resume_unwind(failed)
    });
    assert!(captured, "the provider request includes charter and task words");
    assert!((1..=5).contains(&count), "at most five real completions, and at least one answer");
    assert_eq!(outcome, (1, 1));
}

#[test]
fn a_person_signs_in_to_chatgpt_and_a_real_model_answers() {
    let directory = support::scratch();
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).expect("private live scratch");
    let clock = Clock::new();
    let started = clock.now().now;
    let result = std::panic::catch_unwind(|| {
        let limits = live_sign_in::configuration(directory.path());
        let outcome = live_sign_in::run(directory.path(), started, live_sign_in::Peers::Real);
        live_sign_in::check(directory.path(), &limits, outcome, live_sign_in::Peers::Real);
    });
    let wall_ms = clock.now().now.saturating_since(started).as_nanos() / 1_000_000;
    eprintln!("live ChatGPT wall={wall_ms}ms");
    if let Err(failed) = result {
        // The secrets' store holds the sign-in's record (store.md, section 2), and a
        // trace that failed its check may hold a credential.
        for secret in ["state/secrets", "secrets", "agent-trace.jsonl"] {
            let path = directory.path().join(secret);
            if path.is_dir() {
                fs::remove_dir_all(path).expect("erase failed live test's secrets before retaining diagnostics");
            } else if path.exists() {
                fs::remove_file(path).expect("erase failed live test's trace before retaining diagnostics");
            }
        }
        let path = directory.keep();
        eprintln!("live failed state retained without secrets at {}", path.display());
        eprintln!(
            "A sign-in that did not sign out may still be listed at the issuer. Disconnect it in ChatGPT's settings, under Apps."
        );
        std::panic::resume_unwind(failed);
    }
}

// Reuse the account sign-in world's browser source; it remains a private test helper.
use jig_protocol_accounts_world::fixtures;
#[expect(dead_code, reason = "only the accounts process judge uses browser status diagnostics")]
#[path = "../../../tests/protocol-accounts/src/visitor.rs"]
mod account_sign_in_visitor;
