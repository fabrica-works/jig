#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary tests inspect public trees and select supplied watch requests"
)]
use super::cards::{CLIENT, contains, shared_words as words, text};
use crate::{Builder, JigBindings, Tree};
use jig_client::{Domain, Request, address::Address, contract::*};
use skein_lib::List;
fn list<T>(items: impl IntoIterator<Item = T>) -> List<T> {
    let items: Vec<_> = items.into_iter().collect();
    let mut value = List::with_capacity(u32::try_from(items.len()).unwrap());
    for item in items {
        assert!(value.push(item).is_ok());
    }
    value
}
fn chip(phase: OwnerChipPhase, details: OwnerChipDetails) -> Chip {
    Chip {
        task: 8,
        title: DocumentText { bytes: text(b"partial title"), cut: true },
        executor: Executor::Agent { charter: 1 },
        phase: phase.compact(),
        details: Box::new(ChipPhase::Owner { phase, details: Box::new(details) }),
    }
}
fn brief(value: &Chip) -> Tree {
    let client = Domain::new(&CLIENT, 1);
    let limits = crate::Limits::of_words(&CLIENT, words()).unwrap();
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(limits.bindings, CLIENT.text);
    crate::cards::chip::chip(&mut b, &client, value, words(), &mut bindings);
    let tree = b.finish();
    assert!(tree.len() <= crate::chip_nodes(&CLIENT).unwrap());
    tree
}
#[test]
fn brief_exact_chips_show_title_loss_and_distinct_phase_without_inventing_ending_clock() {
    let cases = [
        (
            OwnerChipPhase::Waiting { dependencies: list([u64::MAX]), more: true, holds: true },
            b"More dependencies are not shown".as_slice(),
        ),
        (
            OwnerChipPhase::Person { address: PersonAddress::Role(u32::MAX), taken_by: Some(u64::MAX), since: None },
            b"4294967295",
        ),
        (OwnerChipPhase::Closing(ClosingStage::Run { attempt: u64::MAX }), b"18446744073709551615"),
        (
            OwnerChipPhase::Held { reason: HoldCategory::Uncertain { entry: u64::MAX }, revision: u64::MAX },
            b"18446744073709551615",
        ),
        (OwnerChipPhase::Active(RunStage::Running { attempt: u64::MAX }), b"18446744073709551615"),
        (OwnerChipPhase::Ended, b"Ended"),
    ];
    for (phase, label) in cases {
        let mut details = super::owner_chip::details();
        let executor_label = match &phase {
            OwnerChipPhase::Person { address, .. } => {
                details.executor = OwnerExecutor::Person(*address);
                b"Project role 4294967295".as_slice()
            }
            OwnerChipPhase::Active(_) => {
                details.executor = OwnerExecutor::Agent { charter: u32::MAX };
                b"Agent charter 4294967295".as_slice()
            }
            _ => {
                details.executor = OwnerExecutor::Procedure { connector: u16::MAX, code: u32::MAX };
                b"Procedure connector 65535, code 4294967295".as_slice()
            }
        };
        let tree = brief(&chip(phase, details));
        assert!(contains(&tree, label));
        assert!(contains(&tree, b"Title is incomplete"));
        assert!(contains(&tree, executor_label));
        assert!(!contains(&tree, b"Ended at:"));
        assert!(!tree.nodes().iter().any(|node| node.text.iter().any(|run| run.text.as_ref() == b"Done")));
    }
}
#[test]
fn task_overview_expands_whole_metadata_but_nested_verdict_chips_stay_brief() {
    let mut inner_details = super::owner_chip::details();
    inner_details.ending =
        Some(OwnerChipEnding::Failed(DocumentText::complete(text(b"nested ending remains carried"))));
    let inner = chip(OwnerChipPhase::Ended, inner_details);
    let mut details = super::owner_chip::details();
    details.resources_cut = true;
    details.resources_observed = false;
    details.ended_at = None;
    details.ending = Some(OwnerChipEnding::Done(ResultValue::Verdict {
        verdict: u32::MAX,
        words: DocumentText { bytes: text(b"whole outer result"), cut: true },
        fields: list([]),
        tasks: list([inner.clone()]),
        proposal: None,
    }));
    let metadata_nodes = crate::owner_chip::required(&details, &CLIENT, words()).unwrap().nodes;
    assert!(metadata_nodes <= crate::owner_chip::max_nodes(&CLIENT).unwrap());
    let mut doc = super::cards_objects::task_doc(&[]);
    doc.chip = chip(OwnerChipPhase::Ended, details);
    let (mut client, out) = super::page_capacity::entered(Address::task(8));
    let watch = out
        .iter()
        .find_map(|request| match request {
            Request::Open { watch, subject: Subject::Task { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    super::pages::deliver(&mut client, watch, Delivery::Task(Box::new(doc)));
    let tree = super::page_capacity::render(&client, words());
    assert!(contains(&tree, b"whole outer result"));
    assert!(contains(&tree, b"Project: 4294967295"));
    assert!(contains(&tree, b"Resource holder and writer facts unavailable"));
    assert!(!contains(&tree, b"nested ending remains carried"));
    assert!(!contains(&tree, b"Ended at:"));
    assert_eq!(inner.phase, Phase::Ended);
    assert_eq!(inner.title.whole(), None);
    let bound = crate::Limits::of_words(&CLIENT, words()).unwrap();
    assert!(tree.nodes().iter().all(|node| node.text_bytes() <= bound.text_per_node));
}
