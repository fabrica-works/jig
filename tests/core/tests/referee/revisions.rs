use super::*;

fn ownership_catches_an_amendment_made_past_the_revision_it_named() {
    let mut fake = Fake::ready(Recovery::Keyed);
    fake.send(Observed::Named { task: 1, revision: Revision::Spec(0) }).expect("named amendment");
    let mut rows = snapshot();
    rows.tasks.get_mut(&1).expect("task").spec_revision = 1;
    fake.commit(2, rows.clone()).expect("revision zero was named");
    rows.tasks.get_mut(&1).expect("task").spec_revision = 2;
    fake.caught(Observed::Durable { number: 3, snapshot: rows }, Promise::Ownership);
}

fn ownership_catches_a_release_of_a_hold_nobody_named() {
    let mut fake = Fake::ready(Recovery::Keyed);
    let mut rows = snapshot();
    rows.tasks.get_mut(&1).expect("task").phase = Phase::Held;
    rows.tasks.get_mut(&1).expect("task").hold_revision = 1;
    fake.commit(2, rows).expect("held");
    fake.caught(Observed::Durable { number: 3, snapshot: snapshot() }, Promise::Ownership);
}

fn two_amendments_each_naming_the_revision_they_moved_are_valid() {
    let mut fake = Fake::ready(Recovery::Keyed);
    for revision in [0, 1] {
        fake.send(Observed::Named { task: 1, revision: Revision::Spec(revision) }).expect("named amendment");
    }
    fake.send(Observed::Restart).expect("evidence survives restart");
    let mut rows = snapshot();
    rows.tasks.get_mut(&1).expect("task").spec_revision = 2;
    fake.commit(2, rows).expect("both revisions named in one commit");
}

fn a_release_naming_the_hold_after_a_pass_up_is_valid() {
    let mut fake = Fake::ready(Recovery::Keyed);
    let mut rows = snapshot();
    let task = rows.tasks.get_mut(&1).expect("task");
    task.phase = Phase::Held;
    task.hold_revision = 2;
    fake.commit(2, rows.clone()).expect("held");
    fake.send(Observed::Named { task: 1, revision: Revision::Hold(2) }).expect("pass names hold");
    fake.commit(3, rows.clone()).expect("passing leaves the hold revision alone");
    rows.tasks.get_mut(&1).expect("task").phase = Phase::Waiting;
    fake.commit(4, rows).expect("named hold releases");
}

fn a_cancel_of_a_held_task_needs_no_named_hold() {
    let mut fake = Fake::ready(Recovery::Keyed);
    let mut rows = snapshot();
    rows.tasks.get_mut(&1).expect("task").phase = Phase::Held;
    fake.commit(2, rows.clone()).expect("held");
    rows.tasks.get_mut(&1).expect("task").phase = Phase::Closing;
    fake.commit(3, rows).expect("cancellation requires no named hold");
}

pub(super) fn revisions_referee_cases() {
    ownership_catches_an_amendment_made_past_the_revision_it_named();
    ownership_catches_a_release_of_a_hold_nobody_named();
    two_amendments_each_naming_the_revision_they_moved_are_valid();
    a_release_naming_the_hold_after_a_pass_up_is_valid();
    a_cancel_of_a_held_task_needs_no_named_hold();
}
