#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary fixtures inspect public semantic trees"
)]
use super::{
    cards::{CLIENT, contains, limits, words},
    work_pages::list,
};
use crate::{Builder, JigBindings, Role, Tree, proposed_task};
use jig_client::contract::*;

fn builder() -> Builder {
    let mut limits = limits();
    limits.nodes = 512;
    limits.depth = 16;
    let mut builder = Builder::new(&limits);
    builder.open(Role::Page);
    builder
}
fn finish(mut builder: Builder) -> Tree {
    builder.close();
    builder.finish()
}

#[test]
fn exact_part_slots_keep_separate_literal_segments_all_owner_kinds_and_numeric_keys() {
    let words = words();
    for kind in [
        OwnerPartKind::Resource,
        OwnerPartKind::Pool,
        OwnerPartKind::Result,
        OwnerPartKind::Procedure,
        OwnerPartKind::Effect,
        OwnerPartKind::Outbox,
    ] {
        for key in [
            OwnerPartKey::Path(list([b"a/b".as_slice().into(), b"c\xff".as_slice().into()])),
            OwnerPartKey::Path(list([b"a".as_slice().into(), b"b/c".as_slice().into()])),
            OwnerPartKey::Number(u64::MAX),
        ] {
            let reference = OwnerPart { connector: u16::MAX, kind, key };
            let mut b = builder();
            let mut bindings = JigBindings::new(16, CLIENT.text);
            proposed_task::part(&mut b, &reference, &words, &mut bindings);
            let tree = finish(b);
            let slot = tree.nodes().iter().find_map(|node| node.slot).unwrap();
            assert_eq!(slot.owner, crate::Owner::Side { connector: u16::MAX });
            assert_eq!(bindings.owner_part(slot.token), Some(&reference));
            assert_eq!(bindings.part(slot.token), None);
            assert_eq!(crate::decode(&mut bindings, slot.token, crate::Act::Press), None);
            match &reference.key {
                OwnerPartKey::Path(segments) => {
                    assert_eq!(segments.len(), 2);
                    if segments.get(1).unwrap().as_ref() == b"c\xff" {
                        assert!(contains(&tree, b"a/b"));
                        assert!(contains(&tree, b"\\x63\\xFF"));
                        assert!(!contains(&tree, b"a/b/c"));
                    } else {
                        assert!(contains(&tree, b"b/c"));
                    }
                }
                OwnerPartKey::Number(number) => {
                    assert_eq!(*number, u64::MAX);
                    assert!(contains(&tree, b"Part number 18446744073709551615"));
                }
            }
        }
    }
}

#[test]
fn proposed_executor_and_funder_keep_full_width_codes_and_original_period_identity() {
    let words = words();
    let mut b = builder();
    for executor in [
        OwnerExecutor::Agent { charter: u32::MAX },
        OwnerExecutor::Procedure { connector: u16::MAX, code: u32::MAX },
        OwnerExecutor::Person(PersonAddress::Party(u64::MAX)),
        OwnerExecutor::Person(PersonAddress::Role(u32::MAX)),
    ] {
        proposed_task::executor(&mut b, executor, &words);
    }
    for funder in [
        Funder::Task(u64::MAX),
        Funder::Pool { project: u32::MAX, person: u64::MAX, period: 83 },
        Funder::Recurring { project: u32::MAX, task: u64::MAX, period: 71 },
        Funder::Period { project: u32::MAX, period: 59 },
    ] {
        proposed_task::funder(&mut b, funder, &words);
    }
    let tree = finish(b);
    assert!(contains(&tree, b"Agent charter 4294967295"));
    assert!(contains(&tree, b"Procedure connector 65535, code 4294967295"));
    assert!(contains(&tree, b"Person 18446744073709551615"));
    assert!(contains(&tree, b"Project role 4294967295"));
    assert!(contains(&tree, b"Funded by task 18446744073709551615"));
    assert!(contains(&tree, b"Funded by pool in project 4294967295, person 18446744073709551615, period 83"));
    assert!(contains(&tree, b"Funded by recurring task in project 4294967295, task 18446744073709551615, period 71"));
    assert!(contains(&tree, b"Funded by project period 4294967295, period 59"));
}

#[test]
fn exact_contracts_keep_approval_state_verdict_fields_and_each_result_bound() {
    let words = words();
    let mut b = builder();
    let mut bindings = JigBindings::new(16, CLIENT.text);
    for contract in [
        TaskContractDocument::Approval {
            state: b"opaque\xff".as_slice().into(),
            part: OwnerPart { connector: 9, kind: OwnerPartKind::Resource, key: OwnerPartKey::Number(83) },
        },
        TaskContractDocument::Report { words: u32::MAX },
        TaskContractDocument::Verdict {
            choices: list([
                VerdictChoice {
                    code: u32::MAX,
                    words: 83,
                    followups: 71,
                    fields: list([FieldRule { name: b"field/name".as_slice().into(), max: 59 }]),
                },
                VerdictChoice {
                    code: 31,
                    words: 29,
                    followups: 23,
                    fields: list([FieldRule { name: b"last\xff".as_slice().into(), max: 19 }]),
                },
            ]),
            cut: true,
        },
        TaskContractDocument::Change { connector: u16::MAX, kind: u16::MAX, words: u32::MAX },
    ] {
        proposed_task::contract(&mut b, &contract, &words, &mut bindings);
    }
    let tree = finish(b);
    assert!(contains(&tree, b"Approval of exact state"));
    assert!(contains(&tree, b"\\x6F\\x70\\x61\\x71\\x75\\x65\\xFF"));
    assert!(contains(&tree, b"Report word byte limit 4294967295"));
    assert!(contains(&tree, b"Verdict choice 4294967295, word byte limit 83, follow-up limit 71"));
    assert!(contains(&tree, b"Verdict choice 31, word byte limit 29, follow-up limit 23"));
    assert!(contains(&tree, b"field/name"));
    assert!(contains(&tree, b"Field value byte limit 59"));
    assert!(contains(&tree, b"\\x6C\\x61\\x73\\x74\\xFF"));
    assert!(contains(&tree, b"Field value byte limit 19"));
    assert!(contains(&tree, b"Some contract choices were omitted"));
    assert!(contains(&tree, b"Change contract 65535, kind 65535, word byte limit 4294967295"));
}
