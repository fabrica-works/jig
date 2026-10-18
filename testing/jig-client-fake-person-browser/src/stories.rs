//! Stories check the application's own wiring of the client and printed link.
use crate::States;
use skein_fake_person::{Find, Next, Step as PersonStep};
use skein_fake_person_browser::{BrowserFace, Harness, Outcome, Play, Script, Setup, Step, harness::ServerAction};
use skein_lib::{Duration, Time};

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
    harness(setup, false).run(&mut [&mut script])
}

fn harness(setup: &Setup<'_>, stopped: bool) -> Harness {
    let mut expected = setup.expected.clone();
    if stopped {
        expected.extend([
            Box::from(b"net::ERR_CONNECTION_REFUSED".as_slice()),
            Box::from(b"net::ERR_INCOMPLETE_CHUNKED_ENCODING".as_slice()),
        ]);
    }
    Harness::new(Setup {
        server: setup.server.clone(),
        args: setup.args.clone(),
        restart: setup.restart.clone(),
        printed: setup.printed,
        chromium: setup.chromium.clone(),
        expected,
        origins: setup.origins.clone(),
    })
}

fn see(find: Find) -> Step {
    Step::Person(PersonStep::See { find, within: PAGE_WAIT })
}

struct Sequence<'a> {
    parts: [&'a mut dyn Play; 3],
    part: usize,
}
impl Play for Sequence<'_> {
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>> {
        while self.part < self.parts.len() {
            match self.parts[self.part].next(face, now) {
                Next::Done => self.part += 1,
                other => return other,
            }
        }
        Next::Done
    }
    fn at(&self) -> u32 {
        self.parts.iter().map(|part| part.at()).sum()
    }
    fn expected(&self) -> Vec<Box<[u8]>> {
        self.parts.iter().flat_map(|part| part.expected()).collect()
    }
    fn check_deadline(&mut self, face: &mut BrowserFace<'_>, now: Time) {
        if let Some(part) = self.parts.get_mut(self.part) {
            part.check_deadline(face, now);
        }
    }
    fn server_action(&mut self) -> Option<ServerAction> {
        self.parts.get_mut(self.part).and_then(|part| part.server_action())
    }
    fn request(&mut self) -> Option<skein_fake_person_browser::harness::Asked> {
        self.parts.get_mut(self.part).and_then(|part| part.request())
    }
    fn answered(&mut self, status: u16) {
        if let Some(part) = self.parts.get_mut(self.part) {
            part.answered(status);
        }
    }
}

fn pending_reload(setup: &Setup<'_>, states: &States, request: &mut dyn Play, after: Box<[Step]>) -> Outcome {
    let mut before = Script::new(Box::new([see(states.live.clone()), Step::StopServer]));
    let mut after = Script::new(after);
    let mut sequence = Sequence { parts: [&mut before, request, &mut after], part: 0 };
    harness(setup, true).run(&mut [&mut sequence])
}

/// A reload keeps the pending request; the protocol world checks it is made once.
#[must_use]
pub fn a_reload_keeps_a_pending_request(
    setup: &Setup<'_>,
    states: &States,
    request: &mut dyn Play,
    pending: Find,
    made: Find,
) -> Outcome {
    pending_reload(
        setup,
        states,
        request,
        Box::new([see(pending), Step::Person(PersonStep::Reload), Step::Warned(true), Step::StartServer, see(made)]),
    )
}

/// A pending request protects the person from leaving without a closing warning.
#[must_use]
pub fn a_reload_with_a_pending_request_meets_the_closing_warning(
    setup: &Setup<'_>,
    states: &States,
    request: &mut dyn Play,
    pending: Find,
) -> Outcome {
    pending_reload(
        setup,
        states,
        request,
        Box::new([see(pending), Step::Person(PersonStep::Reload), Step::Warned(true)]),
    )
}

/// An answered request leaves no closing warning after the person acts on the page.
#[must_use]
pub fn a_reload_with_nothing_pending_meets_no_warning(
    setup: &Setup<'_>,
    states: &States,
    request: &mut dyn Play,
    made: Find,
) -> Outcome {
    let mut before = Script::new(Box::new([see(states.live.clone())]));
    let mut after = Script::new(Box::new([see(made), Step::Person(PersonStep::Reload), Step::Warned(false)]));
    let mut sequence = Sequence { parts: [&mut before, request, &mut after], part: 0 };
    harness(setup, false).run(&mut [&mut sequence])
}

/// The stopped engine shows offline; starting it again restores live state.
#[must_use]
pub fn a_restart_shows_offline_then_live(setup: &Setup<'_>, states: &States) -> Outcome {
    let mut script = Script::new(Box::new([
        see(states.live.clone()),
        Step::StopServer,
        see(states.offline.clone()),
        Step::StartServer,
        see(states.live.clone()),
    ]));
    harness(setup, true).run(&mut [&mut script])
}
