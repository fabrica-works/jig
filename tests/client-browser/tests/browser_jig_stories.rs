//! jig's reusable browser stories against its reference application.
use jig_client_browser_world::Tier;
use jig_client_fake_person_browser::stories;
use skein_fake_person::Find;
use skein_fake_person_browser::Outcome;
use skein_ui::Role;

const FIRST_TASK: u64 = 1;
const HEADING_LEVEL: u8 = 1;

fn made() -> Find {
    Find::exactly_named(Role::Heading { level: HEADING_LEVEL }, format!("Task {FIRST_TASK}").as_bytes())
}
fn pending() -> Find {
    Find::text(b"Being decided: waiting for the engine")
}

#[test]
fn jig_s_spent_link_story_holds_for_jig_test() {
    let tier = Tier::new("spent-link");
    assert_eq!(stories::a_spent_link_signs_the_person_out(&tier.setup(), &tier.states()), Outcome::Done);
}

#[test]
fn jig_s_pending_request_story_holds_for_jig_test() {
    let tier = Tier::reserved("pending-reload");
    assert_eq!(
        stories::a_reload_keeps_a_pending_request(
            &tier.setup(),
            &tier.states(),
            &mut tier.request_script(),
            pending(),
            made()
        ),
        Outcome::Done
    );
}
#[test]
fn a_pending_request_meets_the_closing_warning() {
    let tier = Tier::reserved("pending-warning");
    assert_eq!(
        stories::a_reload_with_a_pending_request_meets_the_closing_warning(
            &tier.setup(),
            &tier.states(),
            &mut tier.request_script(),
            pending()
        ),
        Outcome::Done
    );
}
#[test]
fn jig_s_no_warning_story_holds_for_jig_test() {
    let tier = Tier::new("no-warning");
    assert_eq!(
        stories::a_reload_with_nothing_pending_meets_no_warning(
            &tier.setup(),
            &tier.states(),
            &mut tier.request_script(),
            made()
        ),
        Outcome::Done
    );
}
#[test]
fn jig_s_restart_story_holds_for_jig_test() {
    let tier = Tier::reserved("restart");
    assert_eq!(stories::a_restart_shows_offline_then_live(&tier.setup(), &tier.states()), Outcome::Done);
}
