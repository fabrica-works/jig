//! The actual root/store/peer driver beneath the joint Front (testing.md, 5).
//! It accepts supplied clocks and owner inputs and exposes only external peer
//! deliveries. Request durability and every business decision stay in the root.
use super::transport::Transport;
use jig_client::contract::VersionRange;
use jig_core as core;
use jig_core_world::{client_peer::ClientPeer, effects::World, peers::Peers};
use jig_test_domain as root;
use skein_lib::{Time, Wall};

/// A value received by the independent protocol peer, after the existing
/// store and engine referee observed its release, or through the immediate door.
#[derive(Debug)]
pub enum Packet {
    Held { commit: u64, delivery: Box<root::Delivery> },
    Immediate(Box<core::Now>),
}

pub struct Driver {
    pub world: World,
    pub transport: Transport,
    pub referee: super::referee::Referee,
}

impl Driver {
    #[must_use]
    pub fn new(configuration: root::Config, limits: root::Limits, peers: Peers, now: Time, wall: Wall) -> Self {
        let configured = limits.core.documents.versions;
        let mut world = World::scripted_clock(configuration, limits, peers, now, wall);
        world.client_peer = Some(ClientPeer::routed());
        world.store_peer = Some(jig_core_world::store_peer::StorePeer::after(world.store.applied));
        Self {
            world,
            transport: Transport::new(VersionRange { first: configured.minimum, last: configured.maximum }),
            referee: super::referee::Referee::default(),
        }
    }

    pub fn send(&mut self, now: Time, wall: Wall, event: root::Event) {
        self.register_replies();
        if let root::Event::Core(core::Event::Delivered { watcher }) = &event {
            self.referee.acknowledged(*watcher);
        }
        self.world.set_clock(now, wall);
        self.world.send(event);
    }

    /// Install an independently supplied ordinary person script and pump its
    /// real root requests. Released client packets remain queued for polling.
    pub fn scripted_party(&mut self, party: jig_fake_parties::Party, now: Time, wall: Wall) {
        self.register_replies();
        self.world.peers.as_mut().expect("ordinary scripted peers").parties.push(party);
        self.world.fire_due(now, wall);
    }

    pub fn poll(&mut self, now: Time, wall: Wall) -> Vec<Packet> {
        self.referee.check().expect("peer dropped a normal delivery acknowledgement");
        self.register_replies();
        if self.world.reached_cut.is_none() {
            self.world.fire_due(now, wall);
        }
        let peer = self.world.client_peer.as_mut().expect("actual protocol peer");
        let mut packets = Vec::new();
        for (commit, delivery) in std::mem::take(&mut peer.delivered) {
            packets.push(Packet::Held { commit, delivery: Box::new(delivery) });
        }
        for immediate in std::mem::take(&mut peer.now) {
            packets.push(Packet::Immediate(Box::new(immediate)));
        }
        packets
    }

    /// Cold-start the real root with externally supplied configuration. Released
    /// packets of the old connection cannot cross into its replacement.
    pub fn restart(&mut self, configuration: root::Config, now: Time, wall: Wall) {
        self.register_replies();
        self.world.set_clock(now, wall);
        let peer = self.world.client_peer.as_mut().expect("actual protocol peer");
        peer.delivered.clear();
        peer.now.clear();
        self.world.restart_with(configuration);
    }

    /// Pause only the actual cold-root load pages. Existing authenticated
    /// protocol routes remain independent; the owner supplies transient replies.
    pub fn pause_restore(&mut self, configuration: root::Config, now: Time, wall: Wall) {
        self.world.hold_restore_pages = true;
        self.restart(configuration, now, wall);
    }

    /// Deliver every retained original page at the supplied clock. This does
    /// not invent a reply, reconnect a client or acknowledge a document.
    pub fn finish_restore(&mut self, now: Time, wall: Wall) {
        self.register_replies();
        self.world.set_clock(now, wall);
        self.world.finish_restore_pages();
    }

    /// Mirror only the transport's explicitly live routes before the root can
    /// emit a reply. Taking or detaching a route retires its registration at
    /// the next boundary; capturing an envelope never retires it implicitly.
    fn register_replies(&mut self) {
        let peer = self.world.client_peer.as_mut().expect("actual protocol peer");
        peer.clear_replies();
        for token in self.transport.reply_tokens() {
            peer.register_reply(token);
        }
    }

    /// Apply through the external store while withholding its completion. Both
    /// independent observers see the actual applied rows at the supplied clock
    /// before that completion can release any engine output.
    /// # Errors
    /// Returns the existing store's failed transaction number.
    pub fn withhold_completion(&mut self) -> Result<Option<u64>, u64> {
        let result = self.world.store.tick(true);
        let now = self.world.clocks().0;
        self.world.observer.durable(now.as_nanos(), &self.world.store);
        self.world.store_peer.as_mut().expect("actual durable store peer").capture(now, &self.world.store);
        result
    }

    /// Resume an explicitly paused durability cut through the actual store's
    /// existing release path. Polling alone must never cross the selected cut.
    pub fn release_commits(&mut self) {
        self.world.reached_cut = None;
        self.world.release_commits();
    }

    /// Transfer the external store peer's complete applied transactions once.
    /// Their clock and application index come from the store boundary, before
    /// any protocol projection or acknowledgement.
    pub fn snapshots(&mut self) -> Vec<jig_core_world::store_peer::DurableSnapshot> {
        self.world.store_peer.as_mut().expect("actual durable store peer").take()
    }

    #[must_use]
    pub fn next_due(&self) -> Option<Time> {
        let peer = self.world.client_peer.as_ref().expect("actual protocol peer");
        if peer.delivered.is_empty() && peer.now.is_empty() {
            self.world.next_due()
        } else {
            Some(self.world.clocks().0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{harness::ClientId, joint::transport::Reply};
    use jig_core_people as people;

    #[test]
    fn actual_root_driver_releases_the_supplied_identity_and_independent_session_only_after_its_store_commit() {
        let now = Time::from_nanos(17);
        let wall = Wall::from_nanos(1_000_000_000_000);
        let mut limits = jig_core_world::world::limits();
        limits.core.documents.versions = core::documents::Versions { minimum: 1, maximum: u32::MAX };
        let mut driver =
            Driver::new(jig_core_world::world::config(8455), limits, Peers::new(Vec::new(), Vec::new()), now, wall);
        let supplied = people::Identity {
            key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
            login: b"login".as_slice().into(),
            name: b"Person".as_slice().into(),
        };
        driver.transport.bind(ClientId(0), 1, supplied.clone());
        assert_eq!(
            driver.transport.hello(),
            jig_client::Event::Hello { versions: VersionRange { first: 1, last: u32::MAX } }
        );
        driver.world.cut = Some(jig_core_world::effects::Cut::Submitted(driver.world.store.applied + 1));
        let event = driver.transport.start_sign_in(ClientId(0));
        let core::Event::SignIn { identity, .. } = &event else { panic!("real sign-in") };
        assert_eq!(identity, &supplied);
        driver.send(now, wall, root::Event::Core(event));
        assert!(driver.world.reached_cut.is_some(), "the existing world paused at actual submission");
        assert!(driver.withhold_completion().expect("the sign-in becomes durable").is_none());
        assert!(driver.poll(now, wall).is_empty(), "no protocol answer crosses a withheld durable completion");
        driver.release_commits();
        let mut signed_in = None;
        for packet in driver.poll(now, wall) {
            let Packet::Held { commit, delivery } = packet else { panic!("sign-in must be durable") };
            assert!(commit > 0);
            let root::Delivery::Core(core::Held::PeopleReply { to, sign_in, reply }) = *delivery else {
                panic!("owner reply")
            };
            assert_eq!(driver.transport.take_reply(to.into_token()), Some(Reply::SignIn { client: ClientId(0) }));
            let people::Reply::SignedIn { person, expires } = reply else { panic!("actual signed-in answer") };
            assert_eq!(person, 1);
            assert_eq!(expires.as_nanos(), wall.as_nanos() + limits.core.people.sign_in_lifetime.as_nanos());
            let session = sign_in.expect("actual core session");
            driver.transport.signed_in(ClientId(0), person, session);
            signed_in = Some(session);
        }
        let session = signed_in.expect("one durable answer");
        assert_eq!(driver.transport.sign_in(ClientId(0)), Some(session));
        assert!(driver.world.store.rows.values().any(|row| matches!(row, root::Record::Core(core::Record::People(people::Stored::SignIn { number, person: 1, .. })) if *number == session)));
        assert!(driver.poll(now, wall).is_empty(), "the answer is moved to its original peer once");
        assert_eq!(driver.world.clocks(), (now, wall));
    }
    #[test]
    fn real_driver_transfers_whole_durable_snapshots_before_the_held_answer_without_repeating_or_using_wall_time() {
        let now = Time::from_nanos(17);
        let wall = Wall::from_nanos(1_000_000_000_000);
        let mut driver = Driver::new(
            jig_core_world::world::config(8455),
            jig_core_world::world::limits(),
            Peers::new(Vec::new(), Vec::new()),
            now,
            wall,
        );
        assert!(driver.snapshots().is_empty(), "bootstrap is already applied before peer installation");
        driver.transport.bind(
            ClientId(0),
            1,
            people::Identity {
                key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
                login: Box::new([]),
                name: Box::new([]),
            },
        );
        let expected = driver.world.store.applied + 1;
        driver.world.cut = Some(jig_core_world::effects::Cut::Durable(expected));
        let event = driver.transport.start_sign_in(ClientId(0));
        driver.send(now, wall, root::Event::Core(event));
        assert!(driver.poll(now, wall).is_empty(), "the durable cut still withholds the owner answer");
        let snapshots = driver.snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].commit, expected);
        assert_eq!(snapshots[0].at, now);
        assert_ne!(snapshots[0].at.as_nanos(), wall.as_nanos());
        assert_eq!(
            snapshots[0].rows, driver.world.store.rows,
            "whole applied rows, not submitted changes or a business projection"
        );
        assert!(snapshots[0].rows.values().any(|row| matches!(
            row,
            root::Record::Core(core::Record::People(people::Stored::SignIn { person: 1, .. }))
        )));
        assert!(driver.snapshots().is_empty(), "one external observation per applied transaction");
        driver.release_commits();
        assert_eq!(driver.poll(now, wall).len(), 1, "the original answer follows actual release");
        assert!(driver.snapshots().is_empty(), "releasing the same acknowledgement invents no transaction");
        assert_eq!(snapshots[0].commit, expected, "transferred snapshots remain immutable");
    }

    #[test]
    fn real_driver_retires_only_protocol_routes_and_keeps_same_person_scripted_requests_independent() {
        use jig_fake_parties::{self as parties, Act, Party, Role};
        use skein_lib::Token;

        let now = Time::from_nanos(17);
        let wall = Wall::from_nanos(1_000_000_000_000);
        let (configuration, limits) = jig_core_world::peers::fixture(1212, 0);
        let mut driver = Driver::new(configuration, limits, Peers::new(Vec::new(), Vec::new()), now, wall);
        let subject: Box<[u8]> = b"same identity".as_slice().into();
        driver.transport.bind(
            ClientId(0),
            1,
            people::Identity {
                key: people::IdentityKey { provider: 0, subject: subject.clone() },
                login: Box::new([]),
                name: Box::new([]),
            },
        );
        let event = driver.transport.start_sign_in(ClientId(0));
        assert_eq!(driver.transport.reply_tokens(), [Token::new(1)]);
        driver.send(now, wall, root::Event::Core(event));
        let mut packets = driver.poll(now, wall);
        assert_eq!(packets.len(), 1);
        let Packet::Held { commit, delivery } = packets.remove(0) else { panic!("durable sign-in") };
        let root::Delivery::Core(core::Held::PeopleReply { to, sign_in, reply }) = *delivery else {
            panic!("whole sign-in envelope")
        };
        assert!(commit <= driver.world.store.applied);
        let destination = to.into_token();
        assert_eq!(destination, Token::new(1));
        assert_eq!(driver.transport.take_reply(destination), Some(Reply::SignIn { client: ClientId(0) }));
        let people::Reply::SignedIn { person, .. } = reply else { panic!("actual person") };
        let session = sign_in.expect("actual sign-in session");
        driver.transport.signed_in(ClientId(0), person, session);
        assert!(driver.transport.reply_tokens().is_empty());

        // A caller can reuse a retired opaque destination. Ownership follows
        // the live route table, never that destination's old person or range.
        driver.world.peers.as_mut().expect("ordinary peers").parties.push(Party::new(
            1,
            Role::Owner,
            1,
            Box::new([
                Act::SignIn { provider: 0, subject },
                Act::Request { key: [91; 16], ask: parties::Ask::SeeResults { through: 0 } },
            ]),
        ));
        assert!(driver.poll(now, wall).is_empty(), "both foreign replies resume the scripted ledger");
        let peers = driver.world.peers.as_ref().expect("ordinary peers");
        let party = &peers.parties[0];
        assert!(party.quiescent());
        assert_eq!(party.person, Some(person));
        assert_ne!(party.sign_in, Some(session));
        assert_eq!(party.replies.len(), 2);
        assert_eq!(
            party.replies[0],
            (1, parties::Reply::SignedIn { person, sign_in: party.sign_in.expect("separate session") })
        );
        assert_eq!(party.replies[1], (2, parties::Reply::Done));
        assert_eq!(peers.requests.len(), 2);
        assert!(matches!(peers.requests[1], parties::Up::Request { to: 2, key, .. } if key == [91; 16]));
        assert_eq!(driver.world.people_answers.len(), 2);
        assert!(driver.world.store.rows.values().any(|row| matches!(row,
            root::Record::Core(core::Record::People(people::Stored::Answer { key, .. }))
                if *key == people::RequestKey { person, key: [91; 16] })));

        let token = driver.transport.reply_to(Reply::Keyed {
            client: ClientId(0),
            attempt: Token::new(99),
            key: [92; 16],
            owner: crate::joint::transport::KeyedOwner::Jig,
        });
        assert_eq!(driver.transport.reply_tokens(), [Token::new(2)]);
        driver.send(
            now,
            wall,
            root::Event::Core(core::Event::People(people::Event::Ask {
                reply_to: token,
                sign_in: session,
                key: [92; 16],
                ask: people::Ask::MarkSeen { seen: people::Seen::Results { through: 0 } },
            })),
        );
        let packets = driver.poll(now, wall);
        assert_eq!(packets.len(), 1, "a newly issued route is registered before the real root step");
        assert_eq!(driver.world.people_answers.len(), 2, "owned reply never enters the scripted path");
        let Packet::Held { delivery, .. } = &packets[0] else { panic!("durable keyed answer") };
        let root::Delivery::Core(core::Held::PeopleReply { to: _, reply, .. }) = delivery.as_ref() else {
            panic!("whole keyed reply")
        };
        assert_eq!(
            *reply,
            people::Reply::Outcome(people::Outcome::MarkedSeen { seen: people::Seen::Results { through: 0 } })
        );
        driver.transport.detach(ClientId(0));
        assert!(driver.transport.reply_tokens().is_empty());
        assert!(driver.poll(now, wall).is_empty(), "detachment retires registrations without inventing input");
    }
}
