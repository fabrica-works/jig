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

/// A host outside the application's allowlist cannot make the request.
#[must_use]
pub fn a_rebound_host_is_answered_421_and_does_nothing(
    setup: &Setup<'_>,
    states: &States,
    target: &Target<'_>,
) -> Outcome {
    let mut path = target.prefix.to_vec();
    path.extend_from_slice(b"/v1/request");
    let mut script = Script::new(Box::new([
        Step::Person(PersonStep::See { find: states.live.clone(), within: PAGE_WAIT }),
        Step::Request {
            method: skein_http::Method::Post,
            path: path.into(),
            host: b"rebound.test".as_slice().into(),
            headers: vec![
                (b"Content-Type".as_slice().into(), b"application/octet-stream".as_slice().into()),
                (b"Jig-Client".as_slice().into(), b"browser".as_slice().into()),
            ],
            body: target.request.into(),
            status: 421,
            within: PAGE_WAIT,
        },
        Step::Person(PersonStep::Gone { find: target.done.clone(), within: ABSENT_WAIT }),
    ]));
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

/// Another origin cannot show the application inside its frame.
#[must_use]
pub fn a_framed_page_shows_nothing(setup: &Setup<'_>, states: &States, target: &Target<'_>) -> Outcome {
    let origin = u32::try_from(setup.origins.len()).expect("origin number fits");
    let mut origins = setup.origins.clone();
    let page = pages::framing(target);
    let path = page.path.clone();
    origins.push(Origin { host: b"localhost".as_slice().into(), pages: vec![page] });
    let mut script = Script::new(Box::new([
        Step::Person(PersonStep::See { find: states.live.clone(), within: PAGE_WAIT }),
        Step::Visit { origin, path },
        Step::Trouble { words: b"frame-ancestors".as_slice().into(), within: PAGE_WAIT },
        Step::Person(PersonStep::Gone { find: states.live.clone(), within: ABSENT_WAIT }),
    ]));
    Harness::new(Setup {
        server: setup.server.clone(),
        args: setup.args.clone(),
        restart: setup.restart.clone(),
        printed: setup.printed,
        chromium: setup.chromium.clone(),
        expected: setup.expected.clone(),
        origins,
    })
    .run(&mut [&mut script])
}
