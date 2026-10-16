use crate::{What, tests::fixtures};

fn refuses(from: &[u8], to: &[u8], what: What) {
    let document = fixtures::with(fixtures::ENGINE_SMALL, from, to);
    let problems = fixtures::engine(&document).expect_err("refused charter");
    assert_eq!(problems.problems.get(0).expect("one problem").what, what);
}

fn a_charter_whose_budget_cannot_hold_its_reserve_is_refused() {
    // 8192 input plus 1024 output at 1 per 1000 rounds to 10; one reserve turn needs 20.
    refuses(br#""spend": 20"#, br#""spend": 19"#, What::Budget { charter: 1 });
}

#[test]
fn a_charter_whose_budget_holds_its_reserve_is_read() {
    let engine = fixtures::engine(fixtures::ENGINE_SMALL).expect("valid charter");
    assert_eq!(engine.charters.len(), 1);
    assert_eq!(engine.charters[0].spend, 20);
}

fn a_charter_turns_not_above_its_reserve_are_refused() {
    refuses(br#""turns": 4"#, br#""turns": 1"#, What::Reserve { charter: 1 });
}

fn a_charter_time_not_above_its_reserve_is_refused() {
    refuses(br#""time_ms": 30000"#, br#""time_ms": 5000"#, What::Reserve { charter: 1 });
}

fn a_charter_naming_an_unknown_model_is_refused() {
    refuses(br#""model": "small""#, br#""model": "absent""#, What::Refers);
}

#[test]
fn a_charter_whose_models_name_two_accounts_is_refused() {
    let alternative = br#""models": [{"name":"other", "endpoint":"second", "window":8192,
        "output":1024, "prices":{"input":1,"cached":1,"output":1,"unit":1000}},"#;
    let endpoint = br#""endpoints": [{"name":"second", "number":2, "dialect":0,"account":2,
        "provider":"fake-provider", "address":{"socket":"127.0.0.1:7102"},"transport":"plaintext",
        "headers":[],"connect_ms":10000,"handshake_ms":10000},"#;
    let document = fixtures::with(fixtures::ENGINE_SMALL, br#""models": ["#, alternative);
    let document = fixtures::with(&document, br#""endpoints": ["#, endpoint);
    let document = fixtures::with(
        &document,
        br#""accounts": ["#,
        br#""accounts": [{"number":2,"source":{"handed_in":{"file":"accounts/other.json"}}},"#,
    );
    let document = fixtures::with(&document, br#""alternatives": []"#, br#""alternatives": ["other"]"#);
    let problems = fixtures::engine(&document).expect_err("two accounts");
    assert_eq!(problems.problems.get(0).expect("one problem").what, What::Accounts { charter: 1 });
}

#[test]
fn charters_configuration_problems_name_the_invalid_fields() {
    table([
        (
            "a_charter_whose_budget_cannot_hold_its_reserve_is_refused",
            a_charter_whose_budget_cannot_hold_its_reserve_is_refused,
        ),
        ("a_charter_turns_not_above_its_reserve_are_refused", a_charter_turns_not_above_its_reserve_are_refused),
        ("a_charter_time_not_above_its_reserve_is_refused", a_charter_time_not_above_its_reserve_is_refused),
        ("a_charter_naming_an_unknown_model_is_refused", a_charter_naming_an_unknown_model_is_refused),
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
