//! A printed link reaches the reference inbox under the reference policy.
use jig_client_browser_world::Tier;
use skein_fake_person::{Find, Step};
use skein_fake_person_browser::{Harness, Outcome, Script, harness};
use skein_lib::Duration;
use skein_ui::Role;
const HEADING_LEVEL: u8 = 1;
const INBOX_WAIT: Duration = Duration::from_secs(10);
const FIRST_TASK: u64 = 1;
const OK_STATUS: u16 = 200;
#[test]
fn a_person_opens_the_printed_link_and_sees_their_inbox() {
    let tier = Tier::new("printed-link");
    let mut person = Script::new(Box::new([harness::Step::Person(Step::See {
        find: Find::exactly_named(Role::Heading { level: HEADING_LEVEL }, &tier.words(b"inbox")),
        within: INBOX_WAIT,
    })]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut person]), Outcome::Done);
}

#[test]
fn the_root_file_is_served_as_the_bundle_s_robots_txt() {
    let tier = Tier::new("root-file");
    let mut person = Script::new(Box::new([harness::Step::Request {
        method: skein_http::Method::Get,
        path: b"/robots.txt".as_slice().into(),
        host: b"localhost".as_slice().into(),
        headers: vec![],
        body: Box::new([]),
        status: OK_STATUS,
        within: INBOX_WAIT,
    }]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut person]), Outcome::Done);
}

#[test]
fn under_a_prefix_the_inbox_s_links_carry_the_prefix() {
    let tier = Tier::new("prefixed-inbox");
    let task = format!("Task {FIRST_TASK}");
    let steps = [
        Step::See {
            find: Find::exactly_named(Role::Heading { level: HEADING_LEVEL }, &tier.words(b"inbox")),
            within: INBOX_WAIT,
        },
        Step::Go { address: b"/conversations".as_slice().into() },
        Step::See { find: Find::named(Role::TextArea, b"First words"), within: INBOX_WAIT },
        Step::Type { find: Find::named(Role::TextArea, b"First words"), words: b"Prefix story".as_slice().into() },
        Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
        Step::See { find: Find::exactly_named(Role::Button, b"Cancel"), within: INBOX_WAIT },
        Step::Press { find: Find::exactly_named(Role::Button, b"Cancel") },
        Step::Type { find: Find::named(Role::TextArea, b"Your reason"), words: b"Ready for inbox".as_slice().into() },
        Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
        Step::See { find: Find::text(b"Cancelled"), within: INBOX_WAIT },
        Step::Go { address: b"/test/prefixed/inbox".as_slice().into() },
        Step::See {
            find: Find::exactly_named(Role::Heading { level: HEADING_LEVEL }, &tier.words(b"inbox")),
            within: INBOX_WAIT,
        },
        Step::See { find: Find::named(Role::Link, task.as_bytes()), within: INBOX_WAIT },
        Step::Press { find: Find::named(Role::Link, task.as_bytes()) },
        Step::See {
            find: Find::exactly_named(Role::Heading { level: HEADING_LEVEL }, task.as_bytes()),
            within: INBOX_WAIT,
        },
    ];
    let mut person = Script::new(steps.into_iter().map(harness::Step::Person).collect());
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut person]), Outcome::Done);
}
