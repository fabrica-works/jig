//! Independent literal delivered-fact comparisons do not consult a domain judge.
use super::*;
use jig_core_authority::Implication;
use skein_lib::List;

fn list(values: &[&[u8]]) -> List<Box<[u8]>> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("test rows"));
    for value in values {
        result.push((*value).into()).expect("test room");
    }
    result
}
#[test]
fn supplied_kind_pairs_and_literal_open_patterns_do_not_cross_connectors_or_merge_raw_segments() {
    let mut pairs = List::with_capacity(2);
    pairs.push(Implication { connector: 1, kind: 2, implies: 1 }).expect("pair room");
    pairs.push(Implication { connector: u16::MAX, kind: 2, implies: 1 }).expect("pair room");
    let mut supplied = GrantLine {
        connector: 1,
        kind: 2,
        segments: list(&[b"a", b"bc"]),
        last: PatternEnd::Open(b"x".as_slice().into()),
    };
    let mut draft = people::Grant {
        connector: 1,
        kind: 1,
        pattern: people::Pattern {
            segments: Box::new([b"a".as_slice().into(), b"bc".as_slice().into()]),
            last: people::Last::Exact(b"xyz".as_slice().into()),
        },
    };
    assert!(grant(&supplied, &draft, &pairs), "literal narrower terminal and exact supplied pair");
    draft.connector = 2;
    supplied.connector = 2;
    assert!(!grant(&supplied, &draft, &pairs), "same numeric kinds need their own connector's pair");
    draft.kind = 2;
    assert!(grant(&supplied, &draft, &pairs), "equality remains implicit even without a pair");
    draft.pattern.segments = Box::new([b"ab".as_slice().into(), b"c".as_slice().into()]);
    assert!(!grant(&supplied, &draft, &pairs), "equal concatenation does not equate raw segments");
    draft.pattern.segments = Box::new([b"a".as_slice().into(), b"bc".as_slice().into()]);
    supplied.last = PatternEnd::Exact(b"xyz".as_slice().into());
    draft.pattern.last = people::Last::Open(b"xyz".as_slice().into());
    assert!(!grant(&supplied, &draft, &pairs), "Exact never includes an Open terminal");
    supplied.last = PatternEnd::Open(Box::new([255]));
    draft.pattern.segments = Box::new([b"a".as_slice().into(), b"bc".as_slice().into(), Box::new([255, 0])]);
    draft.pattern.last = people::Last::Exact(Box::new([128]));
    assert!(grant(&supplied, &draft, &pairs), "open terminals include literal opaque descendant segments");
}
