use jig_core::{
    self as core,
    documents::{Document, Subject},
};
use jig_core_tasks as tasks;
use jig_core_world::watches::{latest, open, world};
use jig_test_domain as root;
use skein_lib::{Rng, Token};

#[test]
fn seeded_parties_take_frame_chat_tree_and_inbox_deliveries_at_their_own_pace() {
    for seed in 711..=734 {
        let replay = std::panic::catch_unwind(|| {
            let mut rng = Rng::new(seed);
            let mut world = world(seed, true);
            let assignment = world.assigned.as_ref().expect("assignment");
            let (task, attempt) = (assignment.task, assignment.attempt);
            open(&mut world, 5, Subject::Run { task });
            world.store_delay = u32::try_from(rng.below(6)).expect("small delay");
            let mut burst = Vec::new();
            for turn in 1..=3 {
                burst.push(root::Event::Text {
                    task,
                    attempt,
                    turn,
                    offset: 0,
                    text: b"provisional".as_slice().into(),
                });
                burst.push(root::Event::Core(core::Event::StreamDelivered { watcher: Token::new(5) }));
                burst.push(root::Event::Turn {
                    channel: Token::new(7),
                    task,
                    attempt,
                    turn,
                    cumulative: u64::from(turn),
                    read: None,
                    transcript: b"committed turn".as_slice().into(),
                });
            }
            world.send_all(burst);
            world.end(tasks::End::Parked, 3);
            let Document::Run(run) = latest(&world, 5) else { panic!("run page") };
            assert!(run.attempt.is_none());
            assert!(run.writing.is_none());
            world.watches.as_ref().expect("watches").finish();
        });
        assert!(replay.is_ok(), "watch pace replay seed {seed}");
    }
}
