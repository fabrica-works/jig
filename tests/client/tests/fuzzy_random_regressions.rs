//! Retained named random regressions, with their complete seeds and heap checks.
use jig_client_world::random::{self, Start};
use skein_world::domain::{Verdict, heap::Counting};
#[global_allocator]
static HEAP: Counting = Counting;
#[test]
fn random_people_smoke_newchat_story_boundaries() {
    let outcome = random::story(71, Start::NewChat);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.callbacks > 10);
    assert!(outcome.heap_peak > 0);
    assert!(outcome.heap_peak <= outcome.heap_bound);
}

#[test]
fn random_people_smoke_held_story_boundaries() {
    let outcome = random::story(72, Start::Held);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.callbacks > 10);
    assert!(outcome.heap_peak > 0);
    assert!(outcome.heap_peak <= outcome.heap_bound);
}

#[test]
fn random_people_smoke_random_world_boundaries() {
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
