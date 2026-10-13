use jig_client_protocol_world::{fixtures, protocol};
use jig_client_world::{Until, stories};
#[test]
fn a_person_signs_in_by_the_printed_link_and_starts_a_chat() {
    let mut w = protocol::world(fixtures::SEED);
    let c = w.client(fixtures::FIRST_OWNER, stories::conversation());
    w.person(c, fixtures::sign_in_then(stories::start_chat_steps()));
    let outcome = w.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert_eq!(w.front().chats_made(), 1);
}
