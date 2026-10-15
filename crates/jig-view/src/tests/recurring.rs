#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary tests drive public entries and inspect the semantic tree"
)]
use super::{
    cards::{contains, drive, words},
    pages::{build, named_binding},
    work_pages::{entered, list, project_doc},
};
use jig_domain_client::{
    Event, Request,
    action::Act,
    address::{Address, ProjectSection},
    contract::*,
};

#[test]
fn recurring_keeps_every_template_member_overlap_period_fact_and_last_batch_link() {
    let words = words();
    for overlap in [RecurringOverlap::Skip, RecurringOverlap::Wait] {
        let (mut domain, requests) = entered(Address::Project { project: 4, section: ProjectSection::Recurring });
        let read = requests
            .iter()
            .find_map(|request| match request {
                Request::Read { read, what: Read::Project { .. } } => Some(*read),
                _ => None,
            })
            .unwrap();
        let mut document = project_doc();
        let row = document.recurring.get_mut(0).unwrap();
        row.key = u32::MAX;
        row.overlap = overlap;
        row.current_period = 97;
        row.last_period = 83;
        row.pending_period = Some(91);
        row.last_batch = list([u64::MAX, 10]);
        row.cut = true;
        let final_task = row.batch.get_mut(1).unwrap();
        final_task.number = u64::MAX;
        final_task.executor = OwnerExecutor::Person(PersonAddress::Role(u32::MAX));
        final_task.budget = 37;
        final_task.priority = Some(u32::MAX);
        final_task.dependencies = list([u64::MAX - 1]);
        final_task.contract = TaskContractDocument::Change { connector: u16::MAX, kind: u16::MAX, words: u32::MAX };
        final_task.funder = Funder::Period { project: u32::MAX, period: 71 };
        final_task.cut = true;
        drive(&mut domain, Event::Read { read, result: ReadResult::Value(ReadValue::Project(Box::new(document))) });
        let (tree, mut bindings) = build(&domain, 3, words);
        assert!(contains(&tree, b"Daily work"));
        assert!(contains(&tree, b"Then follow up"));
        assert!(contains(&tree, b"Proposed task 18446744073709551615"));
        assert!(contains(&tree, b"Project role 4294967295"));
        assert!(contains(&tree, b"Budget 37"));
        assert!(contains(&tree, b"Priority 4294967295"));
        assert!(contains(&tree, b"After proposed task 18446744073709551614"));
        assert!(contains(&tree, b"Change contract 65535, kind 65535, word byte limit 4294967295"));
        assert!(contains(&tree, b"Funded by project period 4294967295, period 71"));
        assert!(contains(&tree, b"Some proposed task facts were omitted"));
        assert!(contains(&tree, b"Recurring template 4294967295"));
        assert!(contains(&tree, b"Current period 97"));
        assert!(contains(&tree, b"Last batch period 83"));
        assert!(contains(&tree, b"Pending batch period 91"));
        assert!(contains(&tree, b"Some recurring template facts were omitted"));
        assert_eq!(contains(&tree, b"skip this period"), overlap == RecurringOverlap::Skip);
        assert_eq!(contains(&tree, b"wait for it to end"), overlap == RecurringOverlap::Wait);
        let go =
            crate::decode(&mut bindings, named_binding(&tree, b"Task 18446744073709551615"), crate::ui::Act::Press)
                .unwrap();
        assert_eq!(go, Act::Go { address: Address::task(u64::MAX) });
        let go = crate::decode(&mut bindings, named_binding(&tree, b"Task 10"), crate::ui::Act::Press).unwrap();
        assert_eq!(go, Act::Go { address: Address::task(10) });
    }
}

#[test]
fn recurring_with_no_template_members_or_retained_batch_invents_no_task_or_pending_period() {
    let words = words();
    let (mut domain, requests) = entered(Address::Project { project: 4, section: ProjectSection::Recurring });
    let read = requests
        .iter()
        .find_map(|request| match request {
            Request::Read { read, what: Read::Project { .. } } => Some(*read),
            _ => None,
        })
        .unwrap();
    let mut document = project_doc();
    let row = document.recurring.get_mut(0).unwrap();
    row.batch = list([]);
    row.last_batch = list([]);
    row.pending_period = None;
    row.began = None;
    row.next = None;
    row.cut = false;
    drive(&mut domain, Event::Read { read, result: ReadResult::Value(ReadValue::Project(Box::new(document))) });
    let (tree, _) = build(&domain, 3, words);
    assert!(!contains(&tree, b"Daily work"));
    assert!(!contains(&tree, b"Then follow up"));
    assert!(!contains(&tree, b"Pending batch period"));
    assert!(!contains(&tree, b"Period began at"));
    assert!(!contains(&tree, b"Some recurring template facts were omitted"));
    assert!(!contains(&tree, b"Next runs at"));
    assert!(contains(&tree, b"Runs at the next period"));
}
