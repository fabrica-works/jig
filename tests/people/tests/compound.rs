use jig_domain_people::{Ask, Operation, Outcome, Refusal, Reply};
use jig_people_world::{Settings, World, fixtures::member};
use skein_lib::List;

fn compound() -> Ask {
    let mut operations = List::with_capacity(2);
    operations.push(Operation::Stop { task: 1 }).expect("first operation");
    operations.push(Operation::Stop { task: 2 }).expect("second operation");
    Ask::Compound { project: 1, operations }
}
#[test]
fn a_compound_answered_before_a_restart_is_answered_once_after_it() {
    let mut world = World::new(Settings::calm(1105));
    member(&mut world);
    world.hold_replies(true);
    let first = world.request(10, [1; 16], compound());
    world.commit_all();
    assert_eq!(world.reply(first), None);
    let routes = world.stats().routes;
    let replies = world.stats().replies;
    world.restart();
    world.hold_replies(false);
    let replay = world.request(10, [1; 16], compound());
    world.commit_all();
    assert_eq!(
        world.reply(replay),
        Some(Reply::Outcome(Outcome::Compound { project: 1, operations: 2, first_message: 0 }))
    );
    assert_eq!(world.stats().routes, routes);
    assert_eq!(world.stats().replies, replies + 1);
    world.assert_settled();
}
#[test]
fn a_compound_refused_keeps_its_refusal_under_its_key_across_a_restart() {
    let mut world = World::new(Settings { authority_refuses: true, ..Settings::calm(1106) });
    member(&mut world);
    let first = world.request(10, [1; 16], compound());
    world.commit_all();
    let answer = Some(Reply::Outcome(Outcome::CompoundRefused { at: 1, why: Refusal::Unknown }));
    assert_eq!(world.reply(first), answer);
    world.restart();
    let routes = world.stats().routes;
    let replay = world.request(10, [1; 16], compound());
    world.commit_all();
    assert_eq!(world.reply(replay), answer);
    assert_eq!(world.stats().routes, routes);
    world.assert_settled();
}
