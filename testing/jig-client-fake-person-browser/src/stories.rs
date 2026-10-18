//! Stories check the application's own wiring of the client and printed link.
use crate::States;
use skein_fake_person::Step as PersonStep;
use skein_fake_person_browser::{Harness, Outcome, Script, Setup, Step};
use skein_lib::Duration;

const PAGE_WAIT: Duration = Duration::from_secs(5);

pub(crate) fn spent_link_steps(states: &States) -> Box<[Step]> {
    Box::new([
        Step::Person(PersonStep::See { find: states.live.clone(), within: PAGE_WAIT }),
        Step::Reopen,
        Step::Person(PersonStep::See { find: states.signed_out.clone(), within: PAGE_WAIT }),
    ])
}

/// A spent printed link is refused, and the person sees the sign-in page.
#[must_use]
pub fn a_spent_link_signs_the_person_out(setup: &Setup<'_>, states: &States) -> Outcome {
    let mut script = Script::new(spent_link_steps(states));
    Harness::new(Setup {
        server: setup.server.clone(),
        args: setup.args.clone(),
        restart: setup.restart.clone(),
        printed: setup.printed,
        chromium: setup.chromium.clone(),
        expected: setup.expected.clone(),
        origins: setup.origins.clone(),
    })
    .run(&mut [&mut script])
}
