use super::{limits, w};
use skein_lib::Duration;
use skein_lib::List;
#[must_use]
pub fn run_state_record() -> w::RunState {
    let lim = limits();
    w::RunState::new(&lim, w::RunStateParts { task: 7, attempt: None, committed_turn: 7, writing: None, call: None })
        .expect("literal fixture fits")
}
#[must_use]
pub fn run_text_record() -> w::RunText {
    let lim = limits();
    w::RunText::new(
        &lim,
        w::RunTextParts {
            attempt: 7,
            began: None,
            turn: w::ProvisionalTurnName::new(&lim, w::ProvisionalTurnNameParts { attempt: 7, number: 7, retry: 7 })
                .expect("literal fixture fits"),
            from: 7,
            through: 7,
            text: Box::from([]),
            cut: false,
            call: None,
        },
    )
    .expect("literal fixture fits")
}
#[must_use]
pub fn current_call_record() -> w::CurrentCall {
    let lim = limits();
    w::CurrentCall::new(
        &lim,
        w::CurrentCallParts {
            identity: w::CallIdentity::new(
                &lim,
                w::CallIdentityParts { owner: 7, sequence: 7, retry: 7, id: Box::from([]) },
            )
            .expect("literal fixture fits"),
            tool: Box::from([]),
            input: Box::from([]),
            cut: false,
            began: None,
        },
    )
    .expect("literal fixture fits")
}
#[must_use]
pub fn tasks_record() -> w::Tasks {
    let lim = limits();
    w::Tasks::new(&lim, w::TasksParts { rows: Box::from([]), more: false }).expect("literal fixture fits")
}
#[must_use]
pub fn owner_deployment_record() -> w::OwnerDeployment {
    let lim = limits();
    w::OwnerDeployment::new(
        &lim,
        w::OwnerDeploymentParts {
            store: w::StoreFill::new(&lim, w::StoreFillParts { bytes: 7, mark: 7 }).expect("literal fixture fits"),
            hosts: Box::from([]),
            attempts: Box::from([]),
            accounts: Box::from([]),
            tasks: w::OwnerCapacity::new(&lim, w::OwnerCapacityParts { used: 7, limit: 7 })
                .expect("literal fixture fits"),
            workers: w::OwnerCapacity::new(&lim, w::OwnerCapacityParts { used: 7, limit: 7 })
                .expect("literal fixture fits"),
            attempts_capacity: w::OwnerCapacity::new(&lim, w::OwnerCapacityParts { used: 7, limit: 7 })
                .expect("literal fixture fits"),
            waiting_attempts: 7,
            proposals: w::OwnerCapacity::new(&lim, w::OwnerCapacityParts { used: 7, limit: 7 })
                .expect("literal fixture fits"),
            projects: List::with_capacity(0),
            grace_bound: Duration::from_nanos(19),
            hello_refusals: List::with_capacity(0),
            hosts_forgotten: 7,
            hello_refusals_forgotten: 7,
            person_room_refusals: List::with_capacity(0),
            person_room_refusals_forgotten: 7,
            person_answer_retention: Duration::from_nanos(19),
            task_room_refusals: List::with_capacity(0),
            task_room_refusals_forgotten: 7,
            cut: false,
            connectors: List::with_capacity(0),
            connectors_cut: false,
        },
    )
    .expect("literal fixture fits")
}
#[must_use]
pub fn owner_tree_record() -> w::OwnerTree {
    w::OwnerTree::new(
        &limits(),
        w::OwnerTreeParts {
            root: super::chip_record(),
            rows: Box::from([]),
            folded: List::with_capacity(0),
            held_below: 19,
            next: None,
        },
    )
    .expect("literal fixture fits")
}
#[must_use]
pub fn owner_inbox_record() -> w::OwnerInbox {
    w::OwnerInbox::new(
        &limits(),
        w::OwnerInboxParts {
            party: 19,
            needs: Box::from([]),
            updates: Box::from([]),
            next: None,
            cached_needs: 7,
            cached_updates: w::Count::new(&limits(), w::CountParts { value: 9, limited: true })
                .expect("literal fixture fits"),
            results_seen: 13,
            updates_time_complete: false,
            selection: Some(Box::from([])),
        },
    )
    .expect("literal fixture fits")
}
