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
