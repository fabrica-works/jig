use super::{list, task, text};
use crate::{Bounds, payload::fits};
use jig_client::contract::*;
use skein_lib::Wall;

fn template(local: u16, words: &[u8]) -> ProposedTask {
    ProposedTask {
        local,
        spec: text(words),
        executor: Executor::Agent { charter: 1 },
        budget: 20,
        deadline: None,
        dependencies: list(vec![]),
        grants: list(vec![]),
    }
}

fn recurring() -> Recurring {
    let mut chip = task().chip;
    chip.title = text(b"job");
    chip.details = Box::new(ChipPhase::Starting);
    Recurring {
        chip,
        key: u32::MAX,
        batch: list(vec![template(1, b"first"), template(2, b"second")]),
        budget: u64::MAX,
        overlap: RecurringOverlap::Wait,
        current_period: u64::MAX,
        began: Some(Wall::from_nanos(u64::MAX)),
        next: None,
        last_period: u64::MAX,
        pending_period: Some(u64::MAX),
        last_batch: list(vec![u64::MAX, 9]),
        cut: true,
    }
}

#[test]
fn recurring_prices_the_whole_template_batch_and_final_members_owned_bytes() {
    let row = recurring();
    let bytes = u64::try_from(core::mem::size_of::<Recurring>()).unwrap()
        + u64::try_from(core::mem::size_of::<ChipPhase>()).unwrap()
        + skein_lib::List::<ProposedTask>::worst_case(2).unwrap()
        + skein_lib::List::<u64>::worst_case(2).unwrap()
        + 3
        + 5
        + 6;
    let mut bounds = Bounds { rows: 2, document_bytes: bytes, ..Bounds::default() };
    assert!(fits(&row, &bounds));
    bounds.document_bytes = bytes - 1;
    assert!(!fits(&row, &bounds));
    bounds.document_bytes = bytes;
    let mut longer = row;
    longer.batch.get_mut(1).unwrap().spec = text(b"seconds");
    assert!(!fits(&longer, &bounds), "the final member is priced, even when the first still fits");
}

#[test]
fn recurring_checks_allocated_batch_last_batch_and_nested_member_capacities() {
    let bounds = Bounds { rows: 2, document_bytes: u64::MAX, ..Bounds::default() };
    assert!(fits(&recurring(), &bounds));
    let mut row = recurring();
    row.batch = skein_lib::List::with_capacity(3);
    assert!(!fits(&row, &bounds), "allocated template capacity is checked even when empty");
    let mut row = recurring();
    row.last_batch = skein_lib::List::with_capacity(3);
    assert!(!fits(&row, &bounds));
    let mut row = recurring();
    row.batch.get_mut(1).unwrap().dependencies = skein_lib::List::with_capacity(3);
    assert!(!fits(&row, &bounds));
    let mut row = recurring();
    row.chip.details = Box::new(ChipPhase::Waiting { dependencies: skein_lib::List::with_capacity(3), hold: None });
    assert!(!fits(&row, &bounds));
    let mut row = recurring();
    row.batch.get_mut(1).unwrap().spec = Text::new(vec![0; 4097].into_boxed_slice(), 5000).unwrap();
    assert!(!fits(&row, &bounds), "the final template member's text is checked");
}
