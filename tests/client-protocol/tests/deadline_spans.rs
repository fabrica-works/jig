use jig_client::{Ask, Asker, Key, contract as c};
use jig_client_protocol_world::{client::Story, fixtures};
use skein_lib::Duration;
#[test]
fn a_span_crosses_the_client_wire_to_task_admission() {
    let mut story = Story::live_batch(fixtures::SEED, true);
    let mut tasks = fixtures::create_tasks();
    tasks.get_mut(0).expect("first task").deadline = c::NewDeadline::Span(Duration::from_secs(5));
    let before = story.wall().as_nanos();
    let key = Key { asker: Asker::JIG, number: 85 };
    story.send(key, Ask::Jig(Box::new(c::Ask::Create { project: 1, tasks })));
    assert!(story.run_until(|s| !s.answered(key).is_empty(), Duration::from_secs(1)));
    let at = story.task_deadline(1).expect("made with deadline").as_nanos();
    assert!((before + 5_000_000_000..=story.wall().as_nanos() + 5_000_000_000).contains(&at));
}
