//! A cheap random-person smoke test; all independent scenarios remain focused.
use jig_client_world::random::{self, Start};
use skein_world::domain::{Verdict, heap::Counting};
#[global_allocator]
static HEAP: Counting = Counting;
#[test]
fn random_people_smoke_the_story_and_random_world_boundaries() {
    for (seed, start) in [(71, Start::NewChat), (72, Start::Held)] {
        let outcome = random::story(seed, start);
        assert_eq!(outcome.verdict, Verdict::Passed);
        assert!(outcome.callbacks > 10);
        assert!(outcome.heap_peak > 0);
        assert!(outcome.heap_peak <= outcome.heap_bound);
    }
    let outcome = random::world(73);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.callbacks > 10);
}

#[test]
fn a_restart_with_old_due_output_advances_to_recovery_instead_of_spinning() {
    let outcome = random::story(110, Start::NewChat);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.callbacks > 10);
    assert!(outcome.faults > 0);
}

#[test]
fn signed_out_tabs_resume_the_link_obligation_only_after_confirmation() {
    let outcome = random::story(107, Start::NewChat);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.faults > 0);
}

#[test]
fn selecting_the_current_page_is_still_navigation_for_the_referee() {
    assert_eq!(random::story(111, Start::Chat).verdict, Verdict::Passed);
}

#[test]
fn a_persons_inbox_filter_changes_visibility_without_deciding_the_proposal() {
    assert_eq!(random::story(111, Start::Proposal).verdict, Verdict::Passed);
}

#[test]
fn clearing_the_filter_shows_the_same_still_waiting_proposal() {
    use jig_client_view::Role;
    use jig_client_world::{Until, stories};
    use jig_fake_person::{Find, Step};
    use skein_lib::Duration;
    let mut world = stories::proposal_world(111);
    let client = world.client(7, stories::inbox());
    world.person(
        client,
        vec![
            Step::See { find: Find::text(b"Resolve the incident"), within: Duration::from_secs(1) },
            Step::Press { find: Find::named(Role::Button, b"Reply") },
            Step::Gone { find: Find::text(b"Resolve the incident"), within: Duration::from_secs(1) },
            Step::Press { find: Find::named(Role::Button, b"All kinds") },
            Step::See { find: Find::text(b"Resolve the incident"), within: Duration::from_secs(1) },
            Step::See { find: Find::named(Role::Button, b"Accept"), within: Duration::from_secs(1) },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert!(world.front().engine.commits().iter().all(|commit| commit.key.is_none()));
}
