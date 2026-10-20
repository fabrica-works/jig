use jig_core_world::Fault;
use jig_core_world::{
    client_peer::ClientPeer,
    effects::World,
    observations::Observer,
    peers::{Peers, fixture},
    referee::{Observed, Receipt},
};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_fake_parties::{self as parties, Act, Party, Role};
use jig_fake_workers::{Script, Worker};
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

const SCRIPTED: u64 = u64::MAX - 20;
const CLIENT: Token = Token::new(u64::MAX);
const OTHER_CLIENT: Token = Token::new(1);
const CHAT_BUDGET: u64 = 40;
const SUBJECT: &[u8] = b"same identity";

fn world(peer: ClientPeer) -> World {
    let (config, limits) = fixture(1212, 0);
    let mut world = World::scripted(
        config,
        limits,
        Peers::new(
            vec![Worker::new(7, 1, vec![Box::new([Script::Wait])])],
            vec![Party::new(1, Role::Owner, SCRIPTED, Box::new([]))],
        ),
    );
    world.client_peer = Some(peer);
    world
}

fn sign_in(token: Token) -> root::Event {
    root::Event::Core(core::Event::SignIn {
        reply_to: ReplyTo::new(token),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: SUBJECT.into() },
            login: b"person".as_slice().into(),
            name: b"Person".as_slice().into(),
        },
    })
}

fn scripted_sign_in(world: &mut World) {
    world.peers.as_mut().expect("peers").parties[0]
        .extend(Box::new([Act::SignIn { provider: 0, subject: SUBJECT.into() }]));
}

fn replies(world: &mut World) -> Vec<(u64, Token, people::Reply)> {
    std::mem::take(&mut world.client_peer.as_mut().expect("client peer").delivered)
        .into_iter()
        .map(|(commit, delivery)| {
            let root::Delivery::Core(core::Held::PeopleReply { to, reply }) = delivery else {
                panic!("whole people reply expected: {delivery:?}");
            };
            (commit, to.into_token(), reply)
        })
        .collect()
}

fn signed_in(world: &mut World, token: Token) -> (u64, u64) {
    world.client_peer.as_mut().expect("client peer").register_reply(token);
    world.send(sign_in(token));
    let receipts = replies(world);
    assert_eq!(receipts.len(), 1);
    let (commit, destination, reply) = receipts[0].clone();
    assert_eq!(destination, token);
    assert!(commit > 0 && commit <= world.store.applied);
    let people::Reply::SignedIn { person, sign_in, .. } = reply else { panic!("successful sign-in") };
    assert!(world.store.rows.values().any(|row| matches!(row,
        root::Record::Core(core::Record::People(people::Stored::SignIn { number, person: owner, .. }))
            if *number == sign_in && *owner == person)));
    (person, sign_in)
}

fn mark_seen(token: Token, sign_in: u64, key: [u8; 16]) -> root::Event {
    root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(token),
        sign_in,
        key,
        ask: people::Ask::MarkSeen { seen: people::Seen::Results { through: 0 } },
    }))
}

#[test]
fn an_explicit_client_destination_receives_its_whole_durable_sign_in() {
    let mut world = world(ClientPeer::routed());
    let before = world.store.applied;
    let (person, sign_in) = signed_in(&mut world, CLIENT);
    assert_eq!(world.store.applied, before + 1);
    assert!(world.store.rows.values().any(|row| matches!(row,
        root::Record::Core(core::Record::People(people::Stored::Person { number, identity }))
            if *number == person && identity.key.provider == 0 && identity.key.subject.as_ref() == SUBJECT)));
    assert!(world.commits.iter().flatten().any(|write| matches!(write,
        root::Write::Save(root::Record::Core(core::Record::People(people::Stored::SignIn { number, .. })))
            if *number == sign_in)));
    assert!(world.people_answers.is_empty());
    assert!(world.peers.as_ref().expect("peers").parties[0].replies.is_empty());
}

#[test]
fn an_independent_scripted_party_completes_its_original_request_ledger() {
    let mut world = world(ClientPeer::routed());
    let (client_person, _) = signed_in(&mut world, CLIENT);
    scripted_sign_in(&mut world);
    world.peers.as_mut().expect("peers").parties[0]
        .extend(Box::new([Act::Request { key: [12; 16], ask: parties::Ask::SeeResults { through: 0 } }]));
    world.drain();
    let peers = world.peers.as_ref().expect("peers");
    let party = &peers.parties[0];
    assert_eq!(party.person, Some(client_person));
    assert!(party.quiescent());
    assert_eq!(peers.requests.len(), 2);
    assert_eq!(party.replies.len(), 2);
    assert_eq!(
        party.replies[0],
        (
            SCRIPTED,
            parties::Reply::SignedIn { person: client_person, sign_in: party.sign_in.expect("scripted sign-in") }
        )
    );
    assert_eq!(party.replies[1], (SCRIPTED + 1, parties::Reply::Done));
    assert!(matches!(peers.requests[1], parties::Up::Request { to, key, .. } if to == SCRIPTED + 1 && key == [12; 16]));
    assert!(world.client_peer.as_ref().expect("client peer").delivered.is_empty());
    assert_eq!(world.people_answers.len(), 2);
}

#[test]
fn interleaved_destinations_route_by_registration_even_for_the_same_person() {
    let mut world = world(ClientPeer::routed());
    scripted_sign_in(&mut world);
    let (person, first_session) = signed_in(&mut world, CLIENT);
    let (same_person, second_session) = signed_in(&mut world, OTHER_CLIENT);
    assert_eq!(person, same_person);
    assert_ne!(first_session, second_session);
    assert_eq!(world.peers.as_ref().expect("peers").parties[0].person, Some(person));
    world.peers.as_mut().expect("peers").parties[0]
        .extend(Box::new([Act::Request { key: [13; 16], ask: parties::Ask::SeeResults { through: 0 } }]));
    world
        .send_all(vec![mark_seen(CLIENT, second_session, [14; 16]), mark_seen(OTHER_CLIENT, second_session, [15; 16])]);
    let receipts = replies(&mut world);
    assert_eq!(receipts.len(), 2);
    assert_eq!(receipts[0].1, CLIENT);
    assert_eq!(receipts[1].1, OTHER_CLIENT);
    for (commit, _, reply) in receipts {
        assert!(commit > 0 && commit <= world.store.applied);
        assert_eq!(
            reply,
            people::Reply::Outcome(people::Outcome::MarkedSeen { seen: people::Seen::Results { through: 0 } })
        );
    }
    let party = &world.peers.as_ref().expect("peers").parties[0];
    assert!(party.quiescent());
    assert_eq!(party.replies.len(), 2);
    let scripted_session = party.sign_in.expect("independent session");
    assert_ne!(scripted_session, first_session);
    assert_ne!(scripted_session, second_session);
    assert_eq!(party.replies[0], (SCRIPTED, parties::Reply::SignedIn { person, sign_in: scripted_session }));
    assert_eq!(party.replies[1], (SCRIPTED + 1, parties::Reply::Done));
    assert_eq!(world.people_answers.len(), 2);
}

#[test]
fn registered_and_scripted_sign_ins_wait_for_release_and_are_observed_once() {
    let mut world = world(ClientPeer::routed());
    let referee = world.observer.referee.clone();
    world.observer = Observer::recording(&fixture(1212, 0).0);
    world.observer.referee = referee;
    scripted_sign_in(&mut world);
    world.client_peer.as_mut().expect("client peer").register_reply(CLIENT);
    let before = world.store.applied;
    world.store.fault(Fault::Hold { commits: 1 });
    world.send(sign_in(CLIENT));
    assert!(world.store.applied > before);
    assert!(world.client_peer.as_ref().expect("client peer").delivered.is_empty());
    assert!(world.peers.as_ref().expect("peers").parties[0].replies.is_empty());
    let before_release = world.observer.take();
    assert!(!before_release.iter().any(|value| matches!(value,
        Observed::Released { requires } if requires.iter().any(|receipt| matches!(receipt, Receipt::SignIn(_))))));
    for observed in before_release {
        world.observer.referee.observe(0, observed).expect("original durability referee before release");
    }
    let number = world.store.release_held().expect("real held completion");
    world.send(root::Event::Committed { number, fill: 0 });
    let receipts = replies(&mut world);
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].0, world.store.applied);
    assert!(receipts[0].0 >= number);
    assert_eq!(receipts[0].1, CLIENT);
    assert!(world.peers.as_ref().expect("peers").parties[0].replies.is_empty(), "the second completion is still held");
    let scripted_commit = world.store.release_held().expect("scripted sign-in completion remains held");
    assert!(scripted_commit > number);
    world.send(root::Event::Committed { number: scripted_commit, fill: 0 });
    assert!(world.client_peer.as_ref().expect("client peer").delivered.is_empty());
    let party = &world.peers.as_ref().expect("peers").parties[0];
    assert_eq!(party.replies.len(), 1);
    assert_eq!(party.replies[0].0, SCRIPTED);
    let people::Reply::SignedIn { sign_in, .. } = receipts[0].2 else { panic!("client session") };
    let sessions = [sign_in, party.sign_in.expect("scripted session")];
    let after_release = world.observer.take();
    for session in sessions {
        assert_eq!(
            after_release
                .iter()
                .filter(|value| matches!(value,
            Observed::Released { requires } if requires.contains(&Receipt::SignIn(session))))
                .count(),
            1
        );
    }
    for observed in after_release {
        world.observer.referee.observe(0, observed).expect("original durability referee after release");
    }
}

#[test]
fn keyed_client_work_replays_after_explicit_registration_reset_and_cold_restart() {
    let mut world = world(ClientPeer::routed());
    let (person, session) = signed_in(&mut world, CLIENT);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 1,
        holdings: Box::new([people::Holding { person, role: people::Role::Owner }]),
    })));
    let creation = jig_core_world::fixtures::chat(&world, 1, b"one chat".as_slice().into());
    let chat = || {
        root::Event::Core(core::Event::People(people::Event::Ask {
            reply_to: ReplyTo::new(CLIENT),
            sign_in: session,
            key: [16; 16],
            ask: creation.clone(),
        }))
    };
    world.send(chat());
    let first = replies(&mut world);
    assert_eq!(first.len(), 1);
    let people::Reply::Outcome(people::Outcome::Created { first: task, .. }) = first[0].2 else {
        panic!("chat started")
    };
    world.send(chat());
    let duplicate = replies(&mut world);
    assert_eq!(duplicate.len(), 1);
    assert_eq!((duplicate[0].1, &duplicate[0].2), (CLIENT, &first[0].2));
    assert_eq!(world.peers.as_ref().expect("peers").workers[0].seen.len(), 1);
    world.client_peer.as_mut().expect("client peer").clear_replies();
    assert!(!world.client_peer.as_mut().expect("client peer").unregister_reply(CLIENT));
    world.restart_with(fixture(1212, 0).0);
    world.client_peer.as_mut().expect("client peer").register_reply(CLIENT);
    world.send(chat());
    let replay = replies(&mut world);
    assert_eq!(replay.len(), 1);
    assert_eq!((replay[0].1, &replay[0].2), (CLIENT, &first[0].2));
    assert!(replay[0].0 <= world.store.applied);
    assert_eq!(
        world
            .store
            .rows
            .values()
            .filter(|row| matches!(
                row,
                root::Record::Core(core::Record::Tasks(tasks::Stored::Live(_) | tasks::Stored::Ended(_)))
            ))
            .count(),
        1,
        "replay creates no second task"
    );
    assert_eq!(
        world
            .store
            .rows
            .values()
            .filter(|row| matches!(row,
        root::Record::Core(core::Record::Tasks(tasks::Stored::Live(value) | tasks::Stored::Ended(value)))
            if value.number == task))
            .count(),
        1
    );
    assert_eq!(
        world
            .store
            .rows
            .values()
            .filter(|row| matches!(row,
        root::Record::Core(core::Record::People(people::Stored::Answer { key, outcome, .. }))
            if key.person == person && key.key == [16; 16] && *outcome == people::Outcome::Created { first: task, last: task, given: Some(people::Given { task, amount: CHAT_BUDGET }) }))
            .count(),
        1
    );
    assert!(world.client_peer.as_mut().expect("client peer").unregister_reply(CLIENT));
    assert!(!world.client_peer.as_mut().expect("client peer").unregister_reply(CLIENT));
}

#[test]
fn default_capture_and_independent_worker_delivery_keep_their_original_paths() {
    let mut world = world(ClientPeer::default());
    // No registration: the original default takes every client-facing reply.
    world.send(sign_in(CLIENT));
    let received = replies(&mut world);
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].1, CLIENT);
    let people::Reply::SignedIn { person, sign_in: session, .. } = received[0].2 else { panic!("sign-in succeeded") };
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 1,
        holdings: Box::new([people::Holding { person, role: people::Role::Owner }]),
    })));
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(OTHER_CLIENT),
        sign_in: session,
        key: [17; 16],
        ask: jig_core_world::fixtures::chat(&world, 1, b"worker still receives work".as_slice().into()),
    })));
    let received = replies(&mut world);
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].1, OTHER_CLIENT);
    let people::Reply::Outcome(people::Outcome::Created { first: task, .. }) = received[0].2 else {
        panic!("chat started")
    };
    assert_eq!(world.peers.as_ref().expect("peers").workers[0].seen[0].task, task);
    assert_eq!(world.assignments.len(), 1);
}
