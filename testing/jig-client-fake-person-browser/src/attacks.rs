//! Browser-origin attacks leave no result on the application's loaded page.
use crate::{States, Target, pages};
use skein_fake_person::Step as PersonStep;
use skein_fake_person_browser::{Harness, Outcome, Script, Setup, Step, origins::Origin};
use skein_lib::Duration;

const PAGE_WAIT: Duration = Duration::from_secs(5);
const ABSENT_WAIT: Duration = Duration::from_secs(1);

fn cross_site(setup: &Setup<'_>, states: &States, target: &Target<'_>, host: &[u8]) -> Outcome {
    let origin = u32::try_from(setup.origins.len()).expect("origin number fits");
    let mut origins = setup.origins.clone();
    let page = pages::forging(target);
    let path = page.path.clone();
    origins.push(Origin { host: host.into(), pages: vec![page] });
    let mut expected = setup.expected.clone();
    expected.push(b"Failed to load resource".as_slice().into());
    let mut script = Script::new(Box::new([
        Step::Person(PersonStep::See { find: states.live.clone(), within: PAGE_WAIT }),
        Step::Visit { origin, path },
        Step::Trouble { words: b"blocked by CORS policy".as_slice().into(), within: PAGE_WAIT },
        Step::Person(PersonStep::Go { address: target.page.into() }),
        Step::Person(PersonStep::See { find: states.live.clone(), within: PAGE_WAIT }),
        Step::Person(PersonStep::Gone { find: target.done.clone(), within: ABSENT_WAIT }),
    ]));
    Harness::new(Setup {
        server: setup.server.clone(),
        args: setup.args.clone(),
        restart: setup.restart.clone(),
        printed: setup.printed,
        chromium: setup.chromium.clone(),
        expected,
        origins,
    })
    .run(&mut [&mut script])
}

/// Another port of the same host cannot make the request as the person.
#[must_use]
pub fn a_cross_site_request_from_another_port_does_nothing(
    setup: &Setup<'_>,
    states: &States,
    target: &Target<'_>,
) -> Outcome {
    cross_site(setup, states, target, b"127.0.0.1")
}

/// Another host name, and hence another site, cannot make the request.
#[must_use]
pub fn a_cross_site_request_from_another_host_does_nothing(
    setup: &Setup<'_>,
    states: &States,
    target: &Target<'_>,
) -> Outcome {
    cross_site(setup, states, target, b"localhost")
}
