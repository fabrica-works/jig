use super::*;
use jig_client::contract::{ObjectKey, OwnerRequester, Phase, ReadValue};
use stories::closing::{CANCELLED, CHAT, MOVED, PARTY};
const SEED: u64 = 893;
const TASKS: [(u64, Phase, OwnerRequester); 3] = [
    (CHAT, Phase::Closing, OwnerRequester::Person(PARTY)),
    (CANCELLED, Phase::Ended, OwnerRequester::Task(CHAT)),
    (MOVED, Phase::Running, OwnerRequester::Person(PARTY)),
];
#[test]
fn closing_a_chat_cancels_one_delegate_and_moves_the_other_to_the_person() {
    let mut world = stories::closing::world(SEED);
    let client = world.client(PARTY, stories::chat());
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Close"), within: WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Close") },
            Step::Press { find: Find::named(Role::Button, b"Cancel delegate 10 and its subtree") },
            Step::Press { find: Find::named(Role::Button, b"Take over delegate 11, reserving 1") },
            Step::Type { find: Find::named(Role::TextArea, b"Reason"), words: b"finished".as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Confirm") },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
    assert_eq!(world.front().engine.commits().iter().filter(|commit| commit.ask.is_some()).count(), 1);
    for (task, phase, requester) in TASKS {
        let ReadValue::Task(value) = &world.front().engine.truth().objects[&ObjectKey::Task(task)] else {
            panic!("task document")
        };
        assert_eq!((value.chip.phase, value.requester), (phase, requester), "task {task}");
    }
}
