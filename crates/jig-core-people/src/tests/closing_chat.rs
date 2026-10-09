//! The keyed close retains its exact cohort choices and terminal counts.
use super::*;

#[test]
fn closing_retains_the_whole_disposal_payload_and_replays_its_counts() {
    let mut test = Test::new(LIMITS);
    test.member();
    let mut disposals = List::with_capacity(2);
    disposals.push(Disposal { task: 8, keep: Keep::MakeMine }).unwrap();
    disposals.push(Disposal { task: 9, keep: Keep::Cancel }).unwrap();
    let ask = Ask::Close { project: 1, chat: 7, disposals };
    let request = route(&test.request(10, 1, ask.clone()));
    let outcome = Outcome::Closing { chat: 7, cancelled: 1, kept: 1 };
    let out = test.send(Event::Decided { request, outcome });
    assert_eq!(reply(&out), Reply::Outcome(outcome));
    assert_eq!(reply(&test.request(10, 1, ask.clone())), Reply::Outcome(outcome));
    let Ask::Close { mut disposals, .. } = ask else { panic!("close fixture") };
    disposals.clear();
    disposals.push(Disposal { task: 8, keep: Keep::Cancel }).unwrap();
    let changed = Ask::Close { project: 1, chat: 7, disposals };
    assert_eq!(reply(&test.request(10, 1, changed)), Reply::Refused(Refusal::KeyConflict));
}
