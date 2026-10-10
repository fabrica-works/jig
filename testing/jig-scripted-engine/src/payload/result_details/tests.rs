use super::*;
use crate::payload::fits;
use core::mem::size_of;
use jig_client::contract::{
    Chip, ChipPhase, Executor, FieldRule, OwnerExecutor, OwnerPart, OwnerPartKey, OwnerPartKind, PersonAddress, Phase,
    ProposalKey, Proposer, TaskContractDocument, Text, VerdictChoice,
};
use skein_lib::{List, Wall, bytes::copy_of};

fn chip() -> Chip {
    let mut dependencies = List::with_capacity(3);
    dependencies.push(u64::MAX).unwrap();
    Chip {
        task: u64::MAX,
        title: jig_client::contract::DocumentText::complete(Text::new(copy_of(b"raw"), 3).unwrap()),
        executor: Executor::Agent { charter: u32::MAX },
        phase: Phase::Waiting,
        details: Box::new(ChipPhase::Waiting { dependencies, hold: None }),
    }
}
fn metadata(contract: TaskContractDocument) -> ResultDetails {
    let mut delegates = List::with_capacity(4);
    delegates.push(OwnerTaskLink::Live(chip())).unwrap();
    let mut followups = List::with_capacity(2);
    followups.push(OwnerTaskLink::Stored { number: u64::MAX, status: Some(OwnerTaskStatus::Cancelled) }).unwrap();
    ResultDetails {
        contract,
        requester: OwnerRequester::Deployment { project: u32::MAX },
        executor: OwnerExecutor::Person(PersonAddress::Role(u32::MAX)),
        chosen_by: Some(u64::MAX),
        delegates,
        followups: Some(followups),
        proposal: Some(ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX }),
        cut: true,
    }
}
fn at_bound(value: &ResultDetails, bounds: &mut Bounds, heap: u64) {
    let size = u64::try_from(size_of::<ResultDetails>()).unwrap();
    assert_eq!(value.heap(bounds), Some(heap));
    bounds.document_bytes = size.checked_add(heap).unwrap();
    assert!(fits(value, bounds), "whole value fits exactly");
    bounds.document_bytes = bounds.document_bytes.checked_sub(1).unwrap();
    assert!(!fits(value, bounds), "one byte below rejects the whole value");
}

#[test]
fn result_metadata_prices_allocated_lists_nested_chips_and_verdict_field_bytes() {
    let mut fields = List::with_capacity(3);
    fields.push(FieldRule { name: copy_of(&[255, 0, 128]), max: u32::MAX }).unwrap();
    let mut choices = List::with_capacity(2);
    choices.push(VerdictChoice { code: u32::MAX, words: u32::MAX, followups: u32::MAX, fields }).unwrap();
    let mut value = metadata(TaskContractDocument::Verdict { choices, cut: true });
    let contract = List::<VerdictChoice>::worst_case(2)
        .unwrap()
        .checked_add(List::<FieldRule>::worst_case(3).unwrap())
        .unwrap()
        .checked_add(3)
        .unwrap();
    let chip = 3_u64
        .checked_add(u64::try_from(size_of::<ChipPhase>()).unwrap())
        .unwrap()
        .checked_add(List::<u64>::worst_case(3).unwrap())
        .unwrap();
    let delegates = List::<OwnerTaskLink>::worst_case(4).unwrap().checked_add(chip).unwrap();
    let heap =
        contract.checked_add(delegates).unwrap().checked_add(List::<OwnerTaskLink>::worst_case(2).unwrap()).unwrap();
    let mut bounds = Bounds { text: 3, rows: 4, ..Bounds::default() };
    at_bound(&value, &mut bounds, heap);
    let followup_allocation = List::<OwnerTaskLink>::worst_case(2).unwrap();
    value.followups = Some(List::with_capacity(2));
    at_bound(&value, &mut bounds, heap);
    value.followups = None;
    at_bound(&value, &mut bounds, heap.checked_sub(followup_allocation).unwrap());
    bounds.document_bytes = u64::MAX;
    bounds.rows = 3;
    assert!(!fits(&value, &bounds), "unused delegate capacity is admitted, not only its length");
    bounds.rows = 4;
    bounds.text = 2;
    assert!(!fits(&value, &bounds), "nested raw field name and chip title keep their byte bound");
}

#[test]
fn result_metadata_prices_opaque_approval_paths_and_every_task_link_form() {
    let mut path = List::with_capacity(3);
    path.push(copy_of(b"a/b")).unwrap();
    path.push(copy_of(&[255, 0])).unwrap();
    let mut value = metadata(TaskContractDocument::Approval {
        state: copy_of(&[128, 0, 255]),
        part: OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Outbox, key: OwnerPartKey::Path(path) },
    });
    let mut bounds = Bounds { text: 3, rows: 4, ..Bounds::default() };
    let base = 8_u64
        .checked_add(List::<Box<[u8]>>::worst_case(3).unwrap())
        .unwrap()
        .checked_add(List::<OwnerTaskLink>::worst_case(4).unwrap())
        .unwrap()
        .checked_add(List::<OwnerTaskLink>::worst_case(2).unwrap())
        .unwrap();
    let chip_heap = 3_u64
        .checked_add(u64::try_from(size_of::<ChipPhase>()).unwrap())
        .unwrap()
        .checked_add(List::<u64>::worst_case(3).unwrap())
        .unwrap();
    for link in [
        OwnerTaskLink::Live(chip()),
        OwnerTaskLink::Retained { chip: chip(), at: Some(Wall::from_nanos(u64::MAX)) },
        OwnerTaskLink::Retained { chip: chip(), at: None },
    ] {
        *value.delegates.get_mut(0).unwrap() = link;
        at_bound(&value, &mut bounds, base.checked_add(chip_heap).unwrap());
    }
    for status in [None, Some(OwnerTaskStatus::Done), Some(OwnerTaskStatus::Failed), Some(OwnerTaskStatus::Cancelled)] {
        *value.delegates.get_mut(0).unwrap() = OwnerTaskLink::Stored { number: u64::MAX, status };
        at_bound(&value, &mut bounds, base);
    }
    bounds.document_bytes = u64::MAX;
    bounds.text = 2;
    assert!(!fits(&value, &bounds), "opaque state and slash-bearing path segments keep separate raw byte bounds");
}
