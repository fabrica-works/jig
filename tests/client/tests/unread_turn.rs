use jig_client_world::{Until, stories};
use jig_fake_person::{Find, Step};
use skein_lib::Duration;

#[test]
fn surrounding_turns_show_when_one_does_not_decode() {
    let mut world = stories::unread_turn::world(1);
    let client = world.client_with(7, stories::chat(), stories::unread_turn::limits(), stories::unread_turn::client);
    world.person(
        client,
        vec![
            Step::See { find: Find::text(b"Before the malformed turn"), within: Duration::from_secs(1) },
            Step::See {
                find: Find::text(b"Turn could not be read: malformed transcript"),
                within: Duration::from_secs(1),
            },
            Step::See { find: Find::text(b"After the malformed turn"), within: Duration::from_secs(1) },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
}
