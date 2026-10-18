use super::fixtures;
use jig_protocol_config::{Path, Segment, What};

#[test]
fn an_in_process_engine_address_naming_no_listener_is_refused() {
    let orphan =
        fixtures::with(fixtures::WORKER_DOCUMENT, br#""socket":"127.0.0.1:7200""#, br#""in_process":"workers""#);
    let combined = fixtures::with(
        &fixtures::combined_document(),
        br#""tasks.result_bytes":32"#,
        br#""authority.projects":1,"tasks.result_bytes":32"#,
    );
    let other = fixtures::with(&combined, br#""in_process":"workers""#, br#""in_process":"absent""#);
    for (label, document) in [("no engine", orphan), ("another listener", other)] {
        let found = crate::named(&document).expect_err(label);
        assert_eq!(found.problems.len(), 1, "{label}");
        let problem = found.problems.get(0).expect(label);
        assert_eq!(problem.path, Path::root().member(b"workers").child(Segment::Index(0)).member(b"engine"), "{label}");
        assert_eq!(problem.what, What::Refers, "{label}");
    }
}

#[test]
fn a_template_grace_past_the_tasks_limit_refuses_the_service_start() {
    let document = fixtures::with(&fixtures::engine_document(), br#""grace_ms": 5000"#, br#""grace_ms": 600001"#);
    let found = crate::named(&document).expect_err("template grace");
    assert_eq!(found.problems.get(0).expect("problem").what, What::Grace { template: 1 });
}

#[test]
fn a_service_whose_operations_cannot_close_a_conversation_is_refused() {
    let document = fixtures::with(
        &fixtures::engine_document(),
        br#""overrides": {"#,
        br#""overrides": {"people.operations":4,"tasks.delegates":4,"#,
    );
    let found = crate::named(&document).expect_err("operations per request");
    let problem = found.problems.get(0).expect("problem");
    assert_eq!(problem.path, Path::root().member(b"engine").member(b"limits"));
    assert_eq!(problem.what, What::Operations { operations: 4, delegates: 4, goals: 8 });
}

#[test]
fn a_link_on_a_non_loopback_listener_refuses_the_start() {
    let document = fixtures::with(&fixtures::engine_document(), b"127.0.0.1:7100", b"192.0.2.1:8080");
    let found = crate::startup::parse(&document).expect_err("off-loopback link");
    let path = Path::root().member(b"engine").member(b"clients");
    assert_eq!(found.problems.len(), 2);
    for (index, member) in [b"listener".as_slice(), b"link"].into_iter().enumerate() {
        let problem = found.problems.get(u32::try_from(index).expect("two inputs")).expect("input");
        assert_eq!(problem.path, path.member(member));
        assert_eq!(problem.what, What::Plaintext);
    }
}
