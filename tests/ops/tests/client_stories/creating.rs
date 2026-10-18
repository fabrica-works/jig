use super::*;
use jig_client_domain::contract::{Answer, Made, Outcome};
use skein_lib::{Time, Token, Wall};
const SEED: u64 = 503;
const PARTY: u64 = 7;
const PROJECT: u32 = 1;
const CHAT: u64 = 9;
const ATTEMPT: Token = Token::new(1);
const KEY: [u8; 16] = [1; 16];
#[test]
fn the_environment_rule_accepts_the_templates_chat_request() {
    let mut world = stories::environment_world(SEED);
    let engine = &mut world.front().engine;
    let ask = jig_client::shapes::chat(
        &engine.truth().frames[&PARTY],
        PROJECT,
        jig_client_world::stories::text(b"staging for one week"),
        Wall::EPOCH,
    )
    .expect("fixture offers the chat template");
    engine.send(Time::ZERO, Wall::EPOCH, PARTY, KEY, ATTEMPT, ask).expect("request fits the fixture");
    assert_eq!(
        engine.pending_commits().last().expect("one pending decision").answer,
        Some(Answer::Done(Outcome::Made { what: Made::Tasks { tasks: jig_client_world::test::list([CHAT]) } }))
    );
}
