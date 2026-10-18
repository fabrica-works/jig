use crate::{States, stories};
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::Step;
use skein_lib::Duration;

const PAGE_WAIT: Duration = Duration::from_secs(5);

#[test]
fn the_spent_link_story_reopens_after_seeing_live() {
    let states =
        States { live: Find::text(b"Ready"), offline: Find::text(b"Unreachable"), signed_out: Find::text(b"Enter") };
    let steps = stories::spent_link_steps(&states);
    match steps.as_ref() {
        [
            Step::Person(PersonStep::See { find: live, within: first }),
            Step::Reopen,
            Step::Person(PersonStep::See { find: signed_out, within: last }),
        ] => {
            assert_eq!(live, &Find::text(b"Ready"));
            assert_eq!(signed_out, &Find::text(b"Enter"));
            assert_eq!((*first, *last), (PAGE_WAIT, PAGE_WAIT));
        }
        _ => panic!("see live, reopen the printed URL, then see signed out"),
    }
}
