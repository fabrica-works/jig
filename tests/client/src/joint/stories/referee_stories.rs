//! Deliberately broken translation against an actual held core transaction.
use super::fault_stories::{answers, stop_ready};
use super::*;
use crate::joint::transport::Reply;

#[test]
#[should_panic(expected = "nothing shown done before durable")]
fn the_joint_referee_catches_a_translation_answering_stop_before_its_actual_commit() {
    let (mut world, client) = stop_ready();
    let submitted = world.front().session.driver.world.store.applied + 1;
    world.front().session.driver.world.cut = Some(jig_core_world::effects::Cut::Submitted(submitted));
    world.person(client, vec![Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") }]);
    let boundary = world.now().saturating_add(Duration::from_millis(30));
    let outcome = world.run(Until::Time(boundary));
    assert!(matches!(outcome.verdict, skein_world::domain::Verdict::Open { .. }));
    assert_eq!(
        world.front().session.driver.world.reached_cut,
        Some(jig_core_world::effects::Cut::Submitted(submitted))
    );
    assert_eq!(answers(&mut world), 1, "the Stop answer has not reached the actual store");
    let transport = &world.front().session.driver.transport;
    let attempt = transport
        .reply_tokens()
        .into_iter()
        .find_map(|token| {
            if let Some(Reply::Keyed { client: owner, attempt, .. }) = transport.reply(token) {
                (*owner == client).then_some(*attempt)
            } else {
                None
            }
        })
        .expect("the actual pending Stop's original attempt");
    // Deliberate fault: translate the proper owner outcome but release it before its commit.
    let (envelope, answer) = crate::joint::wire::answers::keyed(crate::joint::wire::answers::reply(
        people::Reply::Outcome(people::Outcome::Stopped { task: 1 }),
    ))
    .expect("complete actual translation");
    world.event(
        client,
        jig_client_test_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope,
                    answer: answer.map(jig_client::Answered::Jig),
                    why: None,
                },
            },
        },
    );
}
