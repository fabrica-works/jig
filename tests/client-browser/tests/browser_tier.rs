//! A printed link reaches the reference inbox under the reference policy.
use jig_client_browser_world::Tier;
use skein_fake_person::{Find, Step};
use skein_fake_person_browser::{Harness, Outcome, Script, harness};
use skein_lib::Duration;
use skein_ui::Role;
const HEADING_LEVEL: u8 = 1;
const INBOX_WAIT: Duration = Duration::from_secs(5);
#[test]
fn a_person_opens_the_printed_link_and_sees_their_inbox() {
    let tier = Tier::new("printed-link");
    let mut person = Script::new(Box::new([harness::Step::Person(Step::See {
        find: Find::exactly_named(Role::Heading { level: HEADING_LEVEL }, &tier.words(b"inbox")),
        within: INBOX_WAIT,
    })]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut person]), Outcome::Done);
}
