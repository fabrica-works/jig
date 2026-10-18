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
    let document = fixtures::with_server_listener(br#"{"socket":"192.0.2.1:7100"}"#);
    let found = crate::startup::parse(&document).expect_err("off-loopback link");
    assert_eq!(
        *fixtures::paths(&found),
        [fixtures::path(&[b"application", b"server", b"listener"]), fixtures::path(&[b"engine", b"clients", b"link"]),]
    );
    for problem in &found.problems {
        assert_eq!(problem.what, What::Plaintext);
    }
}

#[test]
fn a_clients_listener_on_a_public_address_is_refused() {
    let document = fixtures::with_server_listener(br#"{"socket":"192.0.2.1:7100"}"#);
    let document = fixtures::with(&document, b"\"link\": true", b"\"link\": false");
    let found = crate::startup::parse(&document).expect_err("public listener");
    assert_eq!(*fixtures::paths(&found), [fixtures::path(&[b"application", b"server", b"listener"])]);
    assert_eq!(found.problems.get(0).expect("listener").what, What::Plaintext);
}

#[test]
fn an_engine_clients_member_naming_a_listener_is_refused() {
    let document = fixtures::with(
        &fixtures::engine_document(),
        br#""clients": {"#,
        br#""clients": {"listener":{"socket":"127.0.0.1:7100"},"#,
    );
    let found = crate::startup::parse(&document).expect_err("unknown listener");
    assert_eq!(*fixtures::paths(&found), [fixtures::path(&[b"engine", b"clients", b"listener"])]);
    assert_eq!(found.problems.get(0).expect("listener").what, What::Unknown);
}

#[test]
fn the_server_listens_where_the_application_member_says() {
    const PORT: u16 = 7199;
    let document = fixtures::with_server_listener(br#"{"socket":"127.0.0.1:7199"}"#);
    let document = fixtures::with(&document, br#""components":[]"#, br#""components":["clients"]"#);
    let mut service =
        crate::startup(&document, &fixtures::files(&document), None, &fixtures::seeds(), None, None).expect("startup");
    service.clock(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH);
    let engine = fixtures::engine(&mut service);
    let clients = engine.clients.as_mut().expect("clients configured");
    clients.iterate(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        &mut engine.events,
        &engine.env.limits.connector,
        engine.env.limits.core.documents.rows,
    );
    let Some(skein_io::Request::Listen { addr, .. }) = clients.io.pop() else { panic!("listener requested") };
    assert_eq!(addr, skein_io::Addr::Ip(core::net::SocketAddr::new(core::net::IpAddr::from([127, 0, 0, 1]), PORT)));
}
