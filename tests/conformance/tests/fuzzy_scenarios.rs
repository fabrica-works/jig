use jig_conformance::{
    Cut,
    scenarios::{ALL, run},
};
use jig_conformance_world::{Testing, replay};
use std::time::Instant;
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

#[test]
fn every_shared_story_survives_every_commit_and_seeded_random_cuts() {
    let mut random = skein_lib::Rng::new(31);
    for scenario in ALL {
        let started = Instant::now();
        let seed = random.below(1000);
        let cuts = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let reached = std::rc::Rc::clone(&cuts);
        let baseline = replay::every_cut(scenario, seed, move |cut, world| {
            assert!(world.outcome().crashes.contains(&cut), "{scenario:?}, cut {cut}, seed {seed}");
            reached.borrow_mut().push(cut);
        })
        .unwrap_or_else(|error| panic!("{scenario:?}, seed {seed}: {error:?}"));
        assert_eq!(*cuts.borrow(), (1..=baseline.commits).collect::<Vec<_>>(), "every original numbered cut");
        let cut = Cut::Random { seed };
        run::<Testing>(scenario, cut, seed)
            .unwrap_or_else(|error| panic!("{scenario:?}, {cut:?}: {error:?}"))
            .expect("bound kinds");
        eprintln!("shared scenario {scenario:?}: {:?}", started.elapsed());
    }
}
