//! The reference application's pages exercise skein's one-person shell stories.
use jig_client_browser_world::Tier;
use skein_fake_person::Find;
use skein_fake_person_browser::{Outcome, stories};

#[test]
fn a_panic_at_start_reloads_then_says_jig_failed() {
    let tier = Tier::new("panic-start");
    assert_eq!(stories::panic(&tier.setup(), b"/test/panic", Find::text(b"The application failed.")), Outcome::Done);
}
