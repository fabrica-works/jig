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
