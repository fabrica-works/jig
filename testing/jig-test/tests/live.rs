//! Real accounts and model answers through the shipped service and native client.
#[expect(dead_code, reason = "shared process support; live uses the account sign-in entries")]
mod support;
use skein_shell::Clock;
use std::{fs, os::unix::fs::PermissionsExt};

#[path = "support/live_sign_in.rs"]
mod live_sign_in;

/// The live sign-in configurations, read as jig-test's startup reads them: no network, no person.
#[test]
fn the_live_configurations_are_ones_jig_test_reads() {
    let sign_in = support::scratch();
    live_sign_in::configuration_at(sign_in.path(), "192.0.2.1:443", live_sign_in::Peers::Real);
    assert_eq!(jig_test::check(&sign_in.path().join("config.json")), Ok(()));
    for peers in [live_sign_in::Peers::Fake, live_sign_in::Peers::Refusing] {
        let fake = support::scratch();
        live_sign_in::configuration_at(fake.path(), "127.0.0.1:34443", peers);
        assert_eq!(jig_test::check(&fake.path().join("config.json")), Ok(()));
    }
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
        eprintln!("{}", support::trace::failures(directory.path()));
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
