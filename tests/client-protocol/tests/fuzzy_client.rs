//! Replayable application events exercise the application-held client and its referee.
use jig_client::{Ask, Asker, Event, Handle, Key, Link, Readable, Watchable, contract};
use jig_client_protocol_world::{client::Story, fixtures};
use skein_lib::{Duration, Rng};
const FIRST_SEED: u64 = 12100;
const SEEDS: u64 = 32;
const EVENTS: u64 = 32;
const HANDLE: Handle = Handle { asker: Asker(1), number: 1 };
const WITHIN: Duration = Duration::from_secs(1);

fn apply(story: &mut Story, choice: u64, number: u64) {
    match choice {
        0 => story.watch(HANDLE, Watchable::Jig(contract::Subject::Task { task: 9 })),
        1 => story.event(Event::Unwatch { handle: HANDLE }),
        2 => story.read(HANDLE, Readable::Jig(contract::Read::Project { project: 1 })),
        3 => {
            story
                .send(Key { asker: Asker(1), number }, Ask::Jig(Box::new(contract::Ask::Stop { project: 1, task: 9 })));
        }
        4 => story.event(Event::SignOut),
        5 => {
            if story.client.link().state == Link::SignedOut {
                story.sign_in();
            }
        }
        6 => {
            if story.client.link().credential.is_some() {
                story.reload();
            }
        }
        _ => panic!("drawn event range"),
    }
}

#[test]
fn random_scripted_applications_keep_the_client_s_rules() {
    let mut reached = [0; 7];
    for seed in FIRST_SEED..FIRST_SEED + SEEDS {
        let mut rng = Rng::new(seed);
        let mut story = Story::live(seed);
        for number in 1..=EVENTS {
            let choice = rng.between(0, 6);
            reached[usize::try_from(choice).expect("drawn event range")] += 1;
            apply(&mut story, choice, number);
            assert!(story.run_until(|_| true, WITHIN), "seed {seed}, event {number}");
        }
        if story.client.link().state == Link::SignedOut {
            story.sign_in();
        }
        assert!(story.run_until(|s| s.client.link().state == Link::Live, WITHIN), "seed {seed}");
        assert_eq!(story.client.link().party, Some(fixtures::FIRST_OWNER), "seed {seed}");
    }
    assert!(reached.iter().all(|count| *count > 0), "every application event drawn: {reached:?}");
}
