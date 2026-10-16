use jig_core_world::effects::World;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Time, Token, Wall};

#[test]
fn a_chats_flags_and_last_activity_survive_a_cold_restart() {
    let mut world = World::new(171, false);
    world.set_clock(Time::from_nanos(9_000_000_000), Wall::from_nanos(9_000_000_000));
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(8001)),
        sign_in: world.sign_in.expect("signed in"),
        key: [1; 16],
        ask: people::Ask::Say { project: 1, task: 1, words: b"heard at nine".as_slice().into(), release: false },
    })));
    world.restart(171, false);
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Live(1)))]
    else {
        panic!("the chat is live after the restart")
    };
    assert_eq!(row.label, 0);
    assert_eq!(row.flags, tasks::Flags { conversation: true, listed: false, retention: tasks::Retention::Kept });
    assert_eq!(row.last_activity, Wall::from_nanos(9_000_000_000));
}
