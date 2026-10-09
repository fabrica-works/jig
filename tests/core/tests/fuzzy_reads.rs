use jig_core::{self as core, reads};
use jig_core_people as people;
use jig_core_world::{
    effects::Cut,
    reads::{ask, latest},
    stories::reads::world,
};
use jig_test_domain as root;
use skein_lib::{ReplyTo, Rng, Token};

#[test]
fn seeded_reads_wait_for_prior_commits_and_keep_receipts_across_restart_cuts() {
    for seed in 801..=812 {
        let replay = std::panic::catch_unwind(|| {
            let mut rng = Rng::new(seed);
            let mut world = world(seed);
            let task = world.assigned.as_ref().expect("chat").task;
            world.store_delay = u32::try_from(rng.below(6)).expect("bounded delay");
            let number = world.store.applied + 1;
            world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
            world.cut = Some(Cut::Submitted(number));
            world.send(root::Event::Core(core::Event::People(people::Event::Ask {
                reply_to: ReplyTo::new(Token::new(30_001)),
                sign_in: world.sign_in.expect("signed in"),
                key: [1; 16],
                ask: people::Ask::Say { project: 1, task, words: b"before the cut".as_slice().into(), release: false },
            })));
            ask(&mut world, 1, reads::Read::Task { task });
            assert!(world.reads.as_ref().expect("party").receipts.is_empty());
            if rng.below(2) == 0 {
                let number = world.store.release_held().expect("the deciding commit completion");
                world.send(root::Event::Committed { number });
                assert_eq!(world.reads.as_ref().expect("party").receipts.len(), 1);
            }
            let observed = world.reads.as_ref().expect("party").receipts.len();
            world.restart(seed, false);
            assert_eq!(world.reads.as_ref().expect("party").receipts.len(), observed);
            ask(&mut world, 2, reads::Read::Task { task });
            assert!(matches!(latest(&world), reads::Answer::Document(_)));
            ask(&mut world, 3, reads::Read::Activity { task, before: None });
            let reads::Answer::Page(page) = latest(&world) else { panic!("durable owner page") };
            assert!(page.rows.len() <= 2);
            assert_eq!(world.reads.as_ref().expect("party").receipts.len(), observed + 2);
        });
        assert!(replay.is_ok(), "read crash replay seed {seed}");
    }
}
