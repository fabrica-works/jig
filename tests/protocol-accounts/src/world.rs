//! The simulated io world; the owner's records live in its secrets' store.
use crate::{fixtures, owner::Owner, referee::Judge, visitor::Visitor};
use skein_fake_oauth as fake;
use skein_fake_peers::{Transport, oauth as peer};
use skein_io::kernel;
use skein_lib::{Queue, Time, Wall, bytes};
use skein_world::{Host, Memory, Outcome, World};
use std::net::Ipv4Addr;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Story {
    Granted,
    Refreshed,
    Closed,
    HandedIn {
        lapses_in: skein_lib::Duration,
    },
    HandedInRejected,
    RateLimitedRefresh,
    RefusedRefresh,
    RejectedClient,
    DamagedRecord,
    ClosedExchange,
    /// A sign-in asked of the driver, its visit made by the visitor up to
    /// the issuer's redirect, then cancelled.
    SignIn,
}
/// The installation's identifier at a start (configuration.md 8): the one
/// this start drew, and the one an earlier start kept in the store, if any.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Installation {
    pub drawn: [u8; 16],
    pub kept: Option<[u8; 16]>,
}
/// A first start's: nothing kept yet.
pub const FIRST_START: Installation = Installation { drawn: [5; 16], kept: None };
pub enum Process {
    Owner(Box<Owner>),
    Issuer(Box<peer::Peer>),
    Visitor(Box<Visitor>),
}
impl Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Owner(owner) => owner.as_ref(),
            Self::Issuer(peer) => peer.as_ref(),
            Self::Visitor(visitor) => visitor.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Owner(owner) => owner.as_mut(),
            Self::Issuer(peer) => peer.as_mut(),
            Self::Visitor(visitor) => visitor.as_mut(),
        }
    }
}
impl Host for Process {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.host().next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.host().worst_case()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
}
fn issuer(story: Story) -> Process {
    let profile = fixtures::limits();
    let mut peer = peer::Peer::new(
        (Ipv4Addr::LOCALHOST, 31000).into(),
        Transport::Plaintext,
        skein_fake_peers::Limits {
            io: profile.io,
            connections: 2,
            queue: fixtures::ROOM,
            plaintext: 32_768,
            ciphertext: 32_768,
            observations: 16,
            observation_bytes: 4096,
        },
        fake::Config {
            terms: None,
            authorization_url: bytes::copy_of(fixtures::AUTHORIZE),
            token_endpoint: bytes::copy_of(fixtures::TOKEN),
            client_id: bytes::copy_of(b"client"),
            client_secret: None,
            redirect_uri: bytes::copy_of(fixtures::REDIRECT),
            refresh_token: bytes::copy_of(b"seed"),
        },
        fake::Limits {
            document: profile.client.document,
            uri_bytes: 256,
            request_bytes: 1024,
            codes: 1,
            rotations: 2,
            // The registration's one parameter, `ext_agent_host_id`.
            carried: 1,
            plans: 2,
        },
        skein_http::server::Limits {
            // skein-fake-peers' issuer response bounds the redirect URL,
            // six escaped strings, and 267 bytes of framing.
            response: profile.client.uri_bytes
                + profile.client.document.string_bytes * fixtures::ISSUER_ESCAPED_STRINGS
                + fixtures::ISSUER_REDIRECT_FRAMING_BYTES,
            ..profile.server
        },
    )
    .expect("issuer");
    for plan in fixtures::issuer_plans(story) {
        peer.queue(plan).expect("issuer plan");
    }
    Process::Issuer(Box::new(peer))
}
/// A story from the record its store holds at the start.
#[must_use]
pub fn run(seed: u64, story: Story) -> Outcome<Process> {
    run_from(seed, story, fixtures::held(story).as_deref())
}
/// A story from `held`, the record an earlier run left in the store: a
/// start after a restart.
#[must_use]
pub fn run_from(seed: u64, story: Story, held: Option<&[u8]>) -> Outcome<Process> {
    start(seed, story, held, FIRST_START)
}
/// A story from `held` and the installation's identifier as `installation`
/// finds it.
#[must_use]
pub fn start(seed: u64, story: Story, held: Option<&[u8]>, installation: Installation) -> Outcome<Process> {
    let mut config = skein_sim::Config::calm();
    config.wall = Wall::EPOCH;
    let mut world = World::new(seed, config, Judge::new(story), Memory::Checked);
    world.spawn(|| issuer(story));
    world.spawn(|| Process::Owner(Box::new(Owner::new(story, held, installation))));
    if story == Story::SignIn {
        world.spawn(|| Process::Visitor(Box::default()));
    }
    world.run()
}
#[must_use]
pub fn owner(outcome: &Outcome<Process>) -> &Owner {
    outcome
        .procs
        .iter()
        .find_map(|process| match process {
            Process::Owner(owner) => Some(owner.as_ref()),
            Process::Issuer(_) | Process::Visitor(_) => None,
        })
        .expect("owner process")
}
/// What the issuer saw, in order.
#[must_use]
pub fn issuer_saw(outcome: &Outcome<Process>) -> &[peer::Observation] {
    outcome
        .procs
        .iter()
        .find_map(|process| match process {
            Process::Issuer(peer) => Some(peer.observations()),
            Process::Owner(_) | Process::Visitor(_) => None,
        })
        .expect("issuer process")
}
#[must_use]
pub fn visitor(outcome: &Outcome<Process>) -> &Visitor {
    outcome
        .procs
        .iter()
        .find_map(|process| match process {
            Process::Visitor(visitor) => Some(visitor.as_ref()),
            Process::Owner(_) | Process::Issuer(_) => None,
        })
        .expect("visitor process")
}

pub(crate) fn exchange_seen(processes: &[Process]) -> bool {
    processes.iter().any(|process| matches!(process, Process::Issuer(peer) if peer.observations().iter().any(|event| matches!(event, peer::Observation::Post { .. }))))
}
pub(crate) fn exchange_finished(processes: &[Process]) -> bool {
    processes.iter().any(|process| matches!(process, Process::Issuer(peer) if peer.observations().iter().any(|event| matches!(event, peer::Observation::Answered { status: 200, .. }))))
}
