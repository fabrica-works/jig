//! Forged requests originate in Chromium on pages outside the application.
use jig_client_browser_world::Tier;
use jig_client_fake_person_browser::attacks;
use skein_fake_person_browser::Outcome;

#[test]
fn a_page_on_another_port_cannot_make_a_request_as_the_person() {
    let tier = Tier::new("cross-port");
    let target = tier.target();
    assert_eq!(
        attacks::a_cross_site_request_from_another_port_does_nothing(&tier.setup(), &tier.states(), &target),
        Outcome::Done
    );
}

#[test]
fn a_page_of_another_site_cannot_make_a_request_as_the_person() {
    let tier = Tier::new("cross-host");
    let target = tier.target();
    assert_eq!(
        attacks::a_cross_site_request_from_another_host_does_nothing(&tier.setup(), &tier.states(), &target),
        Outcome::Done
    );
}
