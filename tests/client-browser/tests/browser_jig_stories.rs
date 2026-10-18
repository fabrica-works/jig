//! jig's reusable browser stories against its reference application.
use jig_client_browser_world::Tier;
use jig_client_fake_person_browser::stories;
use skein_fake_person_browser::Outcome;

#[test]
fn jig_s_spent_link_story_holds_for_jig_test() {
    let tier = Tier::new("spent-link");
    assert_eq!(stories::a_spent_link_signs_the_person_out(&tier.setup(), &tier.states()), Outcome::Done);
}
