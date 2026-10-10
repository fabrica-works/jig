//! Maximal whole fact envelopes attain the declared allocation, including empty reserves.
use super::*;
use crate::documents::{AuthorityLine, GrantLine, NoteScope, PatternEnd, Versions};

fn bytes(value: u8, count: u32) -> Box<[u8]> {
    let mut result = List::with_capacity(count);
    for _ in 0..count {
        result.push(value).unwrap();
    }
    result.into_boxed()
}

fn bounded(limits: &DocumentLimits, order_rows: u32) -> Admission {
    let mut segments = List::with_capacity(limits.lines);
    for _ in 0..limits.lines {
        segments.push(bytes(255, limits.text_bytes)).unwrap();
    }
    let last = PatternEnd::Open(bytes(254, limits.text_bytes));
    let mut grants = List::with_capacity(limits.lines);
    let mut note_resources = List::with_capacity(limits.lines);
    let mut executors = List::with_capacity(limits.lines);
    let mut roles = List::with_capacity(limits.lines);
    for _ in 0..limits.lines {
        grants
            .push(GrantLine { connector: u16::MAX, kind: u16::MAX, segments: segments.clone(), last: last.clone() })
            .unwrap();
        note_resources.push(NoteScope { connector: u16::MAX, segments: segments.clone(), last: last.clone() }).unwrap();
        executors.push(tasks::AuthorityExecutor::Role(u32::MAX)).unwrap();
        roles.push(u32::MAX).unwrap();
    }
    let authority = AuthorityLine {
        tools: tasks::Tools(u64::MAX),
        grants: grants.clone(),
        executors,
        tasks: u32::MAX,
        depth: u32::MAX,
        ceiling: tasks::Budget { spend: u64::MAX, deadline: None },
        notes: tasks::Scopes(7),
        note_resources,
        cut: false,
    };
    let mut implications = List::with_capacity(order_rows);
    for _ in 0..order_rows {
        implications
            .push(jig_core_authority::Implication { connector: u16::MAX, kind: u16::MAX, implies: u16::MAX - 1 })
            .unwrap();
    }
    Admission::Bounded {
        within: Bounds::Amend {
            budget_left: u64::MAX,
            grants,
            implications,
            authority: Box::new(authority.clone()),
            ceiling: Box::new(authority),
        },
        beyond: Beyond::Proposed { to: Recipient::Roles { project: u32::MAX, roles } },
    }
}
fn limits() -> DocumentLimits {
    DocumentLimits { rows: 2, text_bytes: 2, chips: 2, lines: 2, versions: Versions { minimum: 1, maximum: 1 } }
}

#[test]
fn two_maximal_amendment_offers_attain_the_whole_declared_heap_including_the_kind_relation() {
    let limits = limits();
    let admission = bounded(&limits, limits.lines);
    assert_eq!(receiving(admission.clone(), &limits), Ok(admission.clone()));
    let mut offers = List::with_capacity(MAX_ACTS);
    offers.push(Offered { act: Act::Amend, admission: admission.clone() }).unwrap();
    offers.push(Offered { act: Act::AmendRelease, admission }).unwrap();
    let whole = heap_bytes(&offers).unwrap();
    assert_eq!(whole, heap_worst_case(&limits).unwrap());
    let pair_heap = List::<jig_core_authority::Implication>::worst_case(limits.lines).unwrap();
    for index in 0..offers.len() {
        let offer = offers.get_mut(index).unwrap();
        let Admission::Bounded { within: Bounds::Amend { implications, .. }, .. } = &mut offer.admission else {
            panic!("amendment facts")
        };
        implications.clear();
    }
    assert_eq!(heap_bytes(&offers), Some(whole), "reserved relation rows stay priced when empty");
    for index in 0..offers.len() {
        let offer = offers.get_mut(index).unwrap();
        let Admission::Bounded { within: Bounds::Amend { implications, .. }, .. } = &mut offer.admission else {
            panic!("amendment facts")
        };
        *implications = List::with_capacity(0);
    }
    assert_eq!(heap_bytes(&offers), Some(whole - 2 * pair_heap));
}

#[test]
fn a_short_relation_with_excess_reserved_rows_refuses_the_whole_offer() {
    let limits = limits();
    let mut admission = bounded(&limits, limits.lines + 1);
    let Admission::Bounded { within: Bounds::Amend { implications, .. }, .. } = &mut admission else {
        panic!("amendment facts")
    };
    implications.clear();
    assert_eq!(receiving(admission, &limits), Err(Missing::TooLarge));
}
