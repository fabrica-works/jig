//! The simulated io world and its private filesystem.
use crate::{fixtures, owner::Owner, referee::Judge};
use skein_fake_machine::{self as disk, How, Item};
use skein_fake_oauth as fake;
use skein_fake_peers::{Transport, oauth as peer};
use skein_io::kernel;
use skein_lib::{Queue, Time, Wall, bytes};
use skein_world::{Host, Machine, Memory, Outcome, World};
use std::net::Ipv4Addr;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Story {
    Granted,
    Refreshed,
    Closed,
    HandedIn { lapses_in: skein_lib::Duration },
    HandedInRejected,
}
pub enum Process {
    Owner(Box<Owner>),
    Issuer(Box<peer::Peer>),
}
impl Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Owner(owner) => owner.as_ref(),
            Self::Issuer(peer) => peer.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Owner(owner) => owner.as_mut(),
            Self::Issuer(peer) => peer.as_mut(),
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
pub struct Files {
    disk: disk::Machine,
    pub writes: usize,
}
impl Files {
    fn new() -> (Self, skein_sim::Handle) {
        let encoded =
            skein_oauth::encode_record(&fixtures::kept(), &fixtures::limits().client.document).expect("kept fixture");
        let mut disk = disk::Machine::for_user(1234);
        let root =
            disk.lay(&[Item::directory(b"secret").mode(0o700), Item::file(b"secret/record", &encoded).mode(0o600)]);
        let startup = disk.open(root, b".", How::Directory).expect("startup root");
        disk.close(root);
        (Self { disk, writes: 0 }, skein_sim::Handle::new(startup.raw()))
    }
}
impl Machine for Files {
    fn step(&mut self, call: skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) {
        self.writes += usize::from(matches!(call.ask, skein_sim::Ask::Write { .. }));
        disk::step(&mut self.disk, call, answers);
    }
}
fn issuer() -> Process {
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
            plans: 2,
        },
        profile.server,
    )
    .expect("issuer");
    peer.queue(fixtures::rotate()).expect("rotation plan");
    Process::Issuer(Box::new(peer))
}
#[must_use]
pub fn run(seed: u64, story: Story) -> Outcome<Process, Files> {
    let (files, root) = Files::new();
    let mut config = skein_sim::Config::calm();
    config.wall = Wall::EPOCH;
    let mut world = World::new(seed, config, Judge::new(story), Memory::Checked).with_machine(files);
    world.spawn(issuer);
    world.spawn_root(root, |fd| Process::Owner(Box::new(Owner::new(fd, story))));
    let outcome = world.run();
    assert_eq!(outcome.machine.disk.open_handles(), 0, "startup root and private keeper settle");
    outcome
}
#[must_use]
pub fn owner(outcome: &Outcome<Process, Files>) -> &Owner {
    outcome
        .procs
        .iter()
        .find_map(|process| match process {
            Process::Owner(owner) => Some(owner.as_ref()),
            Process::Issuer(_) => None,
        })
        .expect("owner process")
}
