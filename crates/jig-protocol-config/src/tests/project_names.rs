//! Seed names are checked together (configuration.md, 3; core.md, 6.1).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "tests are ordinary Rust (programming-model.md, 10.2)"
)]
use super::{domain::two_projects, fixtures};
use crate::{Path, Problem, Segment, What};

fn named(document: &[u8], number: u32, json: &str) -> Box<[u8]> {
    let from = format!("\"number\": {number}");
    let to = format!("{from}, \"name\": {json}");
    let start = skein_lib::bytes::find(document, b"\"projects\": [").expect("project fixture");
    let mut result = document[..start].to_vec();
    result.extend_from_slice(&fixtures::with(&document[start..], from.as_bytes(), to.as_bytes()));
    result.into_boxed_slice()
}

fn project_path(index: u32) -> Path {
    Path::root().member(b"projects").child(Segment::Index(index))
}

#[test]
fn a_configured_project_keeps_its_name() {
    let document = named(fixtures::ENGINE_SMALL, 1, "\"Tally\"");
    let engine = fixtures::engine(&document).expect("named project");
    assert_eq!(engine.projects[0].name.as_ref(), "Tally");
    let core = super::domain::core_config(&document);
    assert_eq!(core.names.get(&1).map(Box::as_ref), Some("Tally"));
}

#[test]
fn an_unnamed_project_takes_its_number_in_decimal() {
    let document =
        fixtures::with(fixtures::ENGINE_SMALL, br#""projects": [{"number": 1"#, br#""projects": [{"number": 3"#);
    let engine = fixtures::engine(&document).expect("unnamed project 3");
    assert_eq!(engine.projects[0].name.as_ref(), "Project 3");
}

#[test]
fn a_configured_name_out_of_shape_is_refused_at_its_member() {
    let long = format!("\"{}é\"", "a".repeat(63)); // 65 bytes, past core.md 6.1's 64.
    for json in ["\"\"", long.as_str(), "\"Ta\\u0007lly\"", "\" Tally\"", "\"12\""] {
        let document = named(fixtures::ENGINE_SMALL, 1, json);
        let problems = fixtures::engine(&document).expect_err("name out of shape");
        assert_eq!(
            problems.problems.as_slice(),
            &[Problem { path: project_path(0).member(b"name"), what: What::Form }],
            "{json}"
        );
    }
}

#[test]
fn a_later_configured_name_colliding_under_simple_folding_is_refused() {
    let document = named(&named(&two_projects(b'2'), 1, "\"Tally\""), 2, "\"tally\"");
    let problems = fixtures::engine(&document).expect_err("case-folded name collision");
    assert_eq!(problems.problems.as_slice(), &[Problem { path: project_path(1).member(b"name"), what: What::Unique }]);
}

#[test]
fn given_and_configured_names_are_checked_together() {
    for (document, path) in [
        (named(&two_projects(b'2'), 1, "\"Project 2\""), project_path(1)),
        (named(&two_projects(b'2'), 2, "\"project 1\""), project_path(1).member(b"name")),
    ] {
        let problems = fixtures::engine(&document).expect_err("given name collision");
        assert_eq!(problems.problems.as_slice(), &[Problem { path, what: What::Unique }]);
    }
}
