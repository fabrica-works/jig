#![expect(clippy::disallowed_macros, reason = "tests build boundary strings with ordinary Rust")]
use crate::{NameRule, name_rule, same_name};

#[test]
fn project_names_take_only_their_shape_in_rule_order() {
    // core.md, 6.1: 64 UTF-8 bytes; é straddles the byte boundary.
    let at = format!("{}é", "a".repeat(62));
    let past = format!("{}é", "a".repeat(63));
    for (name, rule) in [
        (at.as_str(), None),
        (past.as_str(), Some(NameRule::Long)),
        ("", Some(NameRule::Empty)),
        ("Ta\u{7}lly", Some(NameRule::Control)),
        (" Tally", Some(NameRule::Space)),
        ("Tally\u{a0}", Some(NameRule::Space)),
        ("Ta lly", None),
        ("12", Some(NameRule::Digits)),
        ("1a", None),
        ("١٢", None),
        (" \u{7}", Some(NameRule::Control)),
    ] {
        assert_eq!(name_rule(name), rule, "{name:?}");
    }
}

#[test]
fn project_names_compare_by_simple_case_folding() {
    for (a, b, same) in [
        ("Tally", "tally", true),
        ("ß", "ẞ", true),
        ("Σ", "σ", true),
        ("Σ", "ς", true),
        ("σ", "ς", true),
        ("\u{212a}", "k", true),
        ("ſ", "s", true),
        ("Tally", "Tallies", false),
        ("a", "á", false),
        ("ß", "ss", false),
    ] {
        assert_eq!(same_name(a, b), same, "{a:?} / {b:?}");
        assert_eq!(same_name(b, a), same, "{b:?} / {a:?}");
    }
}
