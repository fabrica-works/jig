//! Deployment authority is explicit and names configured connectors.
#![expect(clippy::disallowed_methods, reason = "tests are ordinary Rust (programming-model.md, 10.2)")]
use super::fixtures::{ENGINE_SMALL, engine, with};
use crate::{Path, Segment, What};
use jig_domain_people::{ActGrant, ActKind};

#[test]
fn absent_acts_grant_nothing() {
    assert!(engine(ENGINE_SMALL).expect("valid engine").acts.is_empty());
}
#[test]
fn a_valid_grant_keeps_every_kind_including_make_project() {
    let document = with(ENGINE_SMALL, br#""connectors": [1],"#, br#""connectors": [1], "acts": [{"connector":1,"may":["close","add_key","remove_account","adopt","make_project"]}],"#);
    let engine = engine(&document).expect("configured acts");
    let expected = Box::new([ActGrant {
        connector: 1,
        may: Box::new([ActKind::Close, ActKind::AddKey, ActKind::RemoveAccount, ActKind::Adopt, ActKind::MakeProject]),
    }]);
    assert_eq!(&*engine.acts, &*expected);
    assert_eq!(&*crate::core(&engine, [1; 16], 1).grants, &*expected);
}
#[test]
fn invalid_grants_refuse_the_start_at_their_paths() {
    let base = Path::root().member(b"acts");
    let index = |at| base.child(Segment::Index(at));
    for (grants, path, what) in [
        (br#"[{"connector":2,"may":["close"]}]"#.as_slice(), index(0).member(b"connector"), What::Refers),
        (
            br#"[{"connector":1,"may":[]},{"connector":1,"may":[]}]"#.as_slice(),
            index(1).member(b"connector"),
            What::Unique,
        ),
        (
            br#"[{"connector":1,"may":["unknown"]}]"#.as_slice(),
            index(0).member(b"may").child(Segment::Index(0)),
            What::Form,
        ),
        (
            br#"[{"connector":1,"may":["close","close"]}]"#.as_slice(),
            index(0).member(b"may").child(Segment::Index(1)),
            What::Unique,
        ),
    ] {
        let member = [br#""connectors": [1], "acts": "#.as_slice(), grants, b","].concat();
        let document = with(ENGINE_SMALL, br#""connectors": [1],"#, &member);
        let problems = engine(&document).expect_err("grant refused");
        assert_eq!(problems.problems.as_slice(), &[crate::Problem { path, what }]);
    }
}
