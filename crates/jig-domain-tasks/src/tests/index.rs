//! Index keys read newest first forwards (store.md 7.4).
use crate::{Key, Newest, UpdateObject};
use skein_lib::Wall;

#[test]
fn a_later_update_sorts_before_an_earlier_one_for_one_party() {
    let earlier = Key::Update {
        party: 7,
        at: Newest::of(Wall::from_nanos(1_000).as_nanos()),
        task: Newest::of(30),
        object: UpdateObject::Result,
    };
    let later = Key::Update {
        party: 7,
        at: Newest::of(Wall::from_nanos(2_000).as_nanos()),
        task: Newest::of(20),
        object: UpdateObject::Result,
    };
    assert!(later < earlier);
    assert_eq!(Newest::of(30).value(), 30);
}

#[test]
fn a_result_sorts_before_a_reply_of_the_same_task_and_time() {
    let result = Key::Update { party: 7, at: Newest::of(1_000), task: Newest::of(30), object: UpdateObject::Result };
    let reply = Key::Update {
        party: 7,
        at: Newest::of(1_000),
        task: Newest::of(30),
        object: UpdateObject::Reply { message: Newest::of(2), activity: Newest::of(3) },
    };
    assert!(result < reply);
}

#[test]
fn one_partys_updates_sort_before_the_next_partys() {
    let first = Key::Update { party: 7, at: Newest::of(1_000), task: Newest::of(30), object: UpdateObject::Result };
    let second = Key::Update { party: 8, at: Newest::of(2_000), task: Newest::of(20), object: UpdateObject::Result };
    assert!(first < second);
}

#[test]
fn a_newer_conversation_sorts_first_in_the_newest_family() {
    let older = Key::ConversationNewest { party: 7, task: Newest::of(20) };
    let newer = Key::ConversationNewest { party: 7, task: Newest::of(30) };
    assert!(newer < older);
}

#[test]
fn a_newer_goal_sorts_first_within_its_project() {
    let older = Key::Goal { project: 1, task: Newest::of(20) };
    let newer = Key::Goal { project: 1, task: Newest::of(30) };
    assert!(newer < older);
}

#[test]
fn lifetime_ending_counts_saturate() {
    let mut counts = crate::Endings { done: u32::MAX, failed: u32::MAX - 1, cancelled: u32::MAX - 1 };
    counts.add(crate::Endings { done: 1, failed: 2, cancelled: 2 });
    assert_eq!(counts, crate::Endings { done: u32::MAX, failed: u32::MAX, cancelled: u32::MAX });
    assert_eq!(counts.ended(crate::Status::Done), counts);
}

const REQUESTER: u64 = 7;
const INDEXED_TASK: u64 = 30;

#[test]
fn a_newer_ended_task_sorts_first_within_party_and_label() {
    let row = crate::EndedByParty { party: REQUESTER, label: 3, ended_at: Wall::from_nanos(1_000), task: INDEXED_TASK };
    let later = crate::EndedByParty { ended_at: Wall::from_nanos(2_000), ..row };
    assert!(later.key() < row.key());
    assert!(!row.key().live());
}

#[test]
fn one_label_sorts_before_the_next_for_one_party() {
    let row = crate::EndedByParty { party: REQUESTER, label: 3, ended_at: Wall::from_nanos(1_000), task: INDEXED_TASK };
    let next = crate::EndedByParty { label: 4, ended_at: Wall::from_nanos(2_000), ..row };
    assert!(row.key() < next.key());
}
