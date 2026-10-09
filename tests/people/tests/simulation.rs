use jig_core_people::{
    Again, Ask, Entry, EntryKind, Holding, IdentityKey, InitialOwner, Outcome, ProposalChoice, ProposalDecision,
    Refusal, Reply, Role, Seed, Whom,
};
use jig_people_world::{Settings, World};
use skein_lib::Duration;
use skein_world::domain::assert_replays;

fn member(world: &mut World) -> u64 {
    let call = world.signin(10, 1);
    assert!(world.reply(call).is_none());
    world.commit_all();
    let Reply::SignedIn { person, .. } = world.reply(call).expect("test values are admitted and fit") else {
        panic!("signed in")
    };
    world.roles(1, Box::new([Holding { person, role: Role::Member }]));
    world.commit_all();
    person
}

#[test]
fn the_people_stories_and_both_restart_cuts() {
    for seed in 0..3 {
        let mut world = World::new(Settings::random(seed));
        world.run();
    }
}

#[test]
fn state_runs_ahead_while_duplicate_replies_wait_for_the_same_commit() {
    let mut world = World::new(Settings::calm(4));
    member(&mut world);
    let first = world.ask(10, [1; 16], b"hello");
    let duplicate = world.ask(10, [1; 16], b"hello");
    assert!(world.reply(first).is_none() && world.reply(duplicate).is_none());
    assert_eq!(world.stats().routes, 1);
    assert_eq!(world.tasks(), 0);
    world.commit_all();
    assert_eq!(world.reply(first), world.reply(duplicate));
    assert_eq!(world.tasks(), 1);
    world.assert_settled();
}

#[test]
fn a_commit_failed_or_lost_releases_no_reply_and_the_retry_makes_one_task() {
    let mut world = World::new(Settings::calm(5));
    member(&mut world);
    let first = world.ask(10, [1; 16], b"hello");
    assert!(world.reply(first).is_none());
    world.restart();
    assert_eq!(world.tasks(), 0);
    let retry = world.ask(10, [1; 16], b"hello");
    world.commit_all();
    assert!(matches!(world.reply(retry), Some(Reply::Outcome(Outcome::Started { .. }))));
    assert_eq!(world.tasks(), 1);
    world.assert_settled();
}

#[test]
fn a_request_sent_twice_is_made_once_across_a_restart() {
    let mut world = World::new(Settings::calm(6));
    member(&mut world);
    world.hold_replies(true);
    let first = world.ask(10, [1; 16], b"hello");
    world.commit_all();
    assert!(world.reply(first).is_none());
    assert_eq!(world.tasks(), 1);
    let routes = world.stats().routes;
    world.restart();
    world.hold_replies(false);
    let retry = world.ask(10, [1; 16], b"hello");
    world.commit_all();
    assert!(world.reply(retry).is_some());
    assert_eq!(world.stats().routes, routes);
    assert_eq!(world.tasks(), 1);
    world.assert_settled();
}

#[test]
fn an_observer_is_refused_a_request_and_told_why() {
    let mut world = World::new(Settings::calm(13));
    let call = world.signin(10, 1);
    world.commit_all();
    let Reply::SignedIn { person, .. } = world.reply(call).expect("person signed in") else { panic!("signed in") };
    world.roles(1, Box::new([Holding { person, role: Role::Observer }]));
    world.commit_all();
    let before = world.stats().routes;
    let call = world.request(
        10,
        [3; 16],
        Ask::SetGoal { project: 1, spec: b"goal".to_vec().into_boxed_slice(), charter: 1, budget: 1, priority: 1 },
    );
    assert!(world.reply(call).is_none(), "refusal waits for its answer commit");
    world.commit_all();
    assert_eq!(world.reply(call), Some(Reply::Outcome(Outcome::Refused(Refusal::Role { held: Some(Role::Observer) }))));
    assert_eq!(world.stats().routes, before);
    world.assert_settled();
}

#[test]
fn a_service_sets_a_goal_its_role_allows_and_is_refused_a_chat() {
    let owners = Box::new([InitialOwner {
        project: 1,
        identity: IdentityKey { provider: 0, subject: 1_u64.to_be_bytes().into() },
    }]);
    let mut world = World::with_owners(Settings::calm(14), owners);
    world.roles(1, Box::new([]));
    world.commit_all();
    let owner = world.signin(10, 1);
    world.commit_all();
    assert!(matches!(world.reply(owner), Some(Reply::SignedIn { .. })));
    let made = world.request(
        10,
        [1; 16],
        Ask::MakeService { project: 1, name: b"build".to_vec().into_boxed_slice(), role: Role::Member },
    );
    world.commit_all();
    let Some(Reply::Outcome(Outcome::ServiceMade { person })) = world.reply(made) else { panic!("service made") };
    let signed_in = world.signin_service(11, person);
    world.commit_all();
    assert!(matches!(world.reply(signed_in), Some(Reply::SignedIn { person: found, .. }) if found == person));
    let goal = world.request(
        11,
        [2; 16],
        Ask::SetGoal { project: 1, spec: b"goal".to_vec().into_boxed_slice(), charter: 1, budget: 1, priority: 1 },
    );
    world.commit_all();
    assert!(matches!(world.reply(goal), Some(Reply::Outcome(Outcome::GoalStarted { .. }))));
    let routes = world.stats().routes;
    let chat = world.ask(11, [3; 16], b"hello");
    world.commit_all();
    assert_eq!(world.reply(chat), Some(Reply::Outcome(Outcome::Refused(Refusal::Role { held: Some(Role::Member) }))));
    assert_eq!(world.stats().routes, routes);
    assert_eq!(world.tasks(), 1);
    world.assert_settled();
}

#[test]
fn two_maintainers_decide_one_proposal_and_the_second_is_told_who_how_and_when() {
    let mut world = World::new(Settings::calm(15));
    let first = world.signin(10, 1);
    let second = world.signin(11, 2);
    world.commit_all();
    let Some(Reply::SignedIn { person: first_person, .. }) = world.reply(first) else { panic!("first signed in") };
    let Some(Reply::SignedIn { person: second_person, .. }) = world.reply(second) else { panic!("second signed in") };
    world.roles(
        1,
        Box::new([
            Holding { person: first_person, role: Role::Maintainer },
            Holding { person: second_person, role: Role::Maintainer },
        ]),
    );
    world.commit_all();
    let first = world.request(
        10,
        [1; 16],
        Ask::DecideProposal { project: 1, proposer: 7, proposal: 8, decision: ProposalDecision::Accept },
    );
    world.commit_all();
    let expected = Some(Reply::Outcome(Outcome::ProposalDecided {
        proposer: 7,
        proposal: 8,
        by: first_person,
        at: skein_lib::Wall::EPOCH,
        choice: ProposalChoice::Accepted,
    }));
    assert_eq!(world.reply(first), expected);
    world.advance(Duration::from_secs(1));
    let second = world.request(
        11,
        [2; 16],
        Ask::DecideProposal {
            project: 1,
            proposer: 7,
            proposal: 8,
            decision: ProposalDecision::Reject { reason: b"no".to_vec().into_boxed_slice() },
        },
    );
    world.commit_all();
    assert_eq!(world.reply(second), expected);
    world.assert_settled();
}

#[test]
fn roles_are_seeded_from_an_adopted_resources_parties() {
    let mut world = World::new(Settings::calm(16));
    world.roles(1, Box::new([]));
    world.commit_all();
    let identity = IdentityKey { provider: 0, subject: 77_u64.to_be_bytes().into() };
    world.seed(1, Box::new([Seed { identity: identity.clone(), candidate: 42, role: Role::Maintainer }]));
    world.commit_all();
    assert_eq!(world.role_for_identity(identity.clone(), 1), Some((42, Role::Maintainer)));
    let call = world.signin(10, 77);
    world.commit_all();
    assert!(matches!(world.reply(call), Some(Reply::SignedIn { person: 42, .. })));
    assert_eq!(world.role_for_identity(identity, 1), Some((42, Role::Maintainer)));
    world.assert_settled();
}

#[test]
fn a_role_task_leaves_every_inbox_after_one_holder_takes_it() {
    let mut world = World::new(Settings::calm(17));
    let first = world.signin(10, 1);
    let second = world.signin(11, 2);
    world.commit_all();
    let Some(Reply::SignedIn { person: first_person, .. }) = world.reply(first) else { panic!("first signed in") };
    let Some(Reply::SignedIn { person: second_person, .. }) = world.reply(second) else { panic!("second signed in") };
    world.roles(
        1,
        Box::new([
            Holding { person: first_person, role: Role::Maintainer },
            Holding { person: second_person, role: Role::Maintainer },
        ]),
    );
    world.commit_all();
    let task = 99;
    world.waiting(
        task,
        Box::new([Entry {
            task,
            project: 1,
            whom: Whom::Role { project: 1, role: 1 },
            kind: EntryKind::PersonTask,
            at: skein_lib::Wall::EPOCH,
        }]),
    );
    world.check_role_waiting(task, [first_person, second_person], true);
    let call = world.request(10, [3; 16], Ask::TakePerson { project: 1, task });
    world.commit_all();
    assert_eq!(world.reply(call), Some(Reply::Outcome(Outcome::PersonTaken { task })));
    world.check_role_waiting(task, [first_person, second_person], false);
    world.assert_settled();
}

#[test]
fn transient_root_wait_keeps_one_route_and_each_waiter_receives_one_reply() {
    let mut world = World::new(Settings::calm(7));
    member(&mut world);
    world.defer_routes(true);
    let calls = (0..3).map(|_| world.ask(10, [1; 16], b"hello")).collect::<Vec<_>>();
    let extra = world.ask(10, [1; 16], b"hello");
    world.commit_all();
    assert_eq!(world.reply(extra), Some(Reply::Again(Again::Busy)));
    assert_eq!(world.stats().routes, 1);
    world.decide_deferred();
    world.commit_all();
    for call in calls {
        assert!(matches!(world.reply(call), Some(Reply::Outcome(Outcome::Started { .. }))));
    }
    assert_eq!(world.tasks(), 1);
    world.assert_settled();
}

#[test]
fn pending_authorisation_can_be_lost_and_retried_without_repeating_durable_work() {
    let mut world = World::new(Settings::calm(8));
    member(&mut world);
    world.defer_routes(true);
    let first = world.ask(10, [1; 16], b"hello");
    assert!(world.reply(first).is_none());
    world.restart();
    world.defer_routes(false);
    let retry = world.ask(10, [1; 16], b"hello");
    world.commit_all();
    assert!(world.reply(retry).is_some());
    assert_eq!(world.tasks(), 1);
    world.assert_settled();
}

#[test]
fn an_expired_sign_in_is_refused() {
    let owners = Box::new([InitialOwner {
        project: 1,
        identity: IdentityKey { provider: 0, subject: 1_u64.to_be_bytes().into() },
    }]);
    let mut world = World::with_owners(Settings::calm(9), owners);
    world.roles(1, Box::new([]));
    world.commit_all();
    let call = world.signin(10, 1);
    world.commit_all();
    assert!(world.reply(call).is_some());
    world.advance(Duration::from_secs(30));
    world.restart();
    let call = world.ask(10, [1; 16], b"owner");
    world.commit_all();
    assert!(matches!(world.reply(call), Some(Reply::Outcome(Outcome::Started { .. }))));
    world.advance(Duration::from_secs(31));
    world.restart();
    let call = world.ask(10, [2; 16], b"expired");
    world.commit_all();
    assert_eq!(world.reply(call), Some(Reply::SignedOut));
    world.assert_settled();
}

#[test]
fn facts_change_nothing_and_a_seed_replays() {
    let run = |seed| {
        let mut world = World::new(Settings::random(seed));
        world.run();
        (world.trace().to_vec(), world.stats().clone())
    };
    assert_replays(3, 4, run);
    let mut none = World::new(Settings {
        limits: jig_core_people::Limits { facts: 0, ..jig_people_world::LIMITS },
        ..Settings::calm(3)
    });
    none.run();
    let mut many = World::new(Settings::calm(3));
    many.run();
    assert_eq!(none.trace(), many.trace());
}

#[test]
fn a_release_naming_an_old_hold_is_refused_as_moved_saying_what_holds_it_now() {
    let mut world = World::new(Settings::calm(90));
    member(&mut world);
    world.held_task(7, 2, jig_core_people::HoldReason::Budget);
    let ask = Ask::Release { project: 1, task: 7, revision: 1, again: None };
    let call = world.request(10, [7; 16], ask.clone());
    assert!(world.reply(call).is_none());
    world.commit_all();
    let expected = Some(Reply::Outcome(Outcome::Refused(Refusal::Moved {
        now: jig_core_people::Moved::Hold { revision: 2, reason: jig_core_people::HoldReason::Budget },
    })));
    assert_eq!(world.reply(call), expected);
    world.held_task(7, 3, jig_core_people::HoldReason::Deadline);
    world.restart();
    let routes = world.stats().routes;
    let replay = world.request(10, [7; 16], ask);
    assert_eq!(world.reply(replay), expected);
    assert_eq!(world.stats().routes, routes);
    world.assert_settled();
}

#[test]
fn an_amendment_on_a_spec_amended_meanwhile_is_refused_with_its_revision() {
    let mut world = World::new(Settings::calm(91));
    member(&mut world);
    world.amended_task(7, 2);
    let ask = Ask::Amend {
        release: None,
        project: 1,
        task: 7,
        revision: 1,
        amendment: jig_core_people::Amendment {
            spec: Some(jig_core_people::Spec {
                words: b"new words".as_slice().into(),
                parameters: Box::new([]),
                inputs: Box::new([]),
            }),
            wake: None,
            dependencies: None,
            authority: None,
            reason: b"new evidence".as_slice().into(),
        },
    };
    let call = world.request(10, [8; 16], ask.clone());
    world.commit_all();
    let expected =
        Some(Reply::Outcome(Outcome::Refused(Refusal::Moved { now: jig_core_people::Moved::Spec { revision: 2 } })));
    assert_eq!(world.reply(call), expected);
    world.amended_task(7, 3);
    world.restart();
    let routes = world.stats().routes;
    let replay = world.request(10, [8; 16], ask);
    assert_eq!(world.reply(replay), expected);
    assert_eq!(world.stats().routes, routes);
    world.assert_settled();
}

fn application(words: &[u8]) -> Ask {
    use sha2::{Digest, Sha256};
    Ask::Application { project: 1, connector: 2, digest: Sha256::digest(words).into() }
}

#[test]
fn an_application_request_sent_twice_is_routed_once() {
    let mut world = World::new(Settings::calm(92));
    member(&mut world);
    let ask = application(b"extend object 7 at revision 3");
    let first = world.request(10, [9; 16], ask.clone());
    let duplicate = world.request(10, [9; 16], ask.clone());
    assert!(world.reply(first).is_none() && world.reply(duplicate).is_none());
    assert_eq!(world.stats().routes, 1);
    world.commit_all();
    assert_eq!(world.reply(first), world.reply(duplicate));
    assert!(matches!(world.reply(first), Some(Reply::Outcome(Outcome::Applied { connector: 2, .. }))));
    world.restart();
    let replay = world.request(10, [9; 16], ask);
    assert_eq!(world.reply(first), world.reply(replay));
    assert_eq!(world.stats().routes, 1);
    world.assert_settled();
}

#[test]
fn an_application_request_under_a_used_key_with_another_digest_is_a_conflict() {
    let mut world = World::new(Settings::calm(93));
    member(&mut world);
    world.request(10, [9; 16], application(b"extend object 7"));
    let conflict = world.request(10, [9; 16], application(b"extend object 8"));
    world.commit_all();
    assert_eq!(world.reply(conflict), Some(Reply::Refused(Refusal::KeyConflict)));
    let jig_conflict = world.ask(10, [9; 16], b"hello");
    assert_eq!(world.reply(jig_conflict), Some(Reply::Refused(Refusal::KeyConflict)));
    assert_eq!(world.stats().routes, 1);
    world.assert_settled();
}

#[test]
fn words_to_a_stopped_chat_release_it_in_one_request() {
    let mut world = World::new(Settings::calm(93));
    let person = member(&mut world);
    world.held_task(7, 1, jig_core_people::HoldReason::StoppedBy { party: person });
    let ask = Ask::Say { project: 1, task: 7, words: b"continue".as_slice().into(), release: true };
    let call = world.request(10, [2; 16], ask.clone());
    assert!(world.reply(call).is_none());
    assert!(world.sayings.is_empty(), "no parent decision visible ahead of commit");
    world.commit_all();
    let Some(Reply::Outcome(Outcome::Said { message, released: true, .. })) = world.reply(call) else {
        panic!("words must release the person's stop")
    };
    assert_eq!(world.sayings[&message], (7, b"continue".as_slice().into(), true));
    world.restart();
    let again = world.request(10, [2; 16], ask);
    assert_eq!(world.reply(again), world.reply(call));
    assert_eq!(world.sayings.len(), 1, "saved replay never adds another word");
}

#[test]
fn an_uncertain_release_keeps_the_missing_choice_facts_on_replay() {
    let mut world = World::new(Settings::calm(94));
    member(&mut world);
    world.held_task(7, 1, jig_core_people::HoldReason::Uncertain { entry: 9 });
    let ask = Ask::Release { project: 1, task: 7, revision: 1, again: None };
    let call = world.request(10, [3; 16], ask.clone());
    world.commit_all();
    assert_eq!(
        world.reply(call),
        Some(Reply::Outcome(Outcome::Refused(Refusal::Authority {
            lacking: jig_core_people::Lack::Again { required: true }
        })))
    );
    world.held_task(7, 2, jig_core_people::HoldReason::Budget);
    world.restart();
    let replay = world.request(10, [3; 16], ask);
    assert_eq!(world.reply(replay), world.reply(call));
}

#[test]
fn an_uncertain_effect_released_without_making_it_again_tells_its_executor() {
    let mut world = World::new(Settings::calm(95));
    member(&mut world);
    world.held_task(7, 1, jig_core_people::HoldReason::Uncertain { entry: 9 });
    let ask = Ask::Release { project: 1, task: 7, revision: 1, again: Some(false) };
    let call = world.request(10, [4; 16], ask.clone());
    assert!(world.reply(call).is_none());
    assert!(world.executor_outcomes.is_empty());
    world.commit_all();
    assert_eq!(world.reply(call), Some(Reply::Outcome(Outcome::Released { task: 7 })));
    assert_eq!(world.executor_outcomes, [(9, false)]);
    world.restart();
    let replay = world.request(10, [4; 16], ask);
    assert_eq!(world.reply(replay), world.reply(call));
    assert_eq!(world.executor_outcomes, [(9, false)]);
}

fn edit() -> jig_core_people::Amendment {
    jig_core_people::Amendment {
        spec: Some(jig_core_people::Spec {
            words: b"changed".as_slice().into(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        }),
        wake: None,
        dependencies: None,
        authority: None,
        reason: b"evidence".as_slice().into(),
    }
}

#[test]
fn an_amendment_and_a_release_are_one_decision() {
    let mut world = World::new(Settings::calm(97));
    member(&mut world);
    world.amended_task(7, 1);
    world.held_task(7, 2, jig_core_people::HoldReason::Stopped);
    let ask = Ask::Amend { project: 1, task: 7, revision: 1, amendment: edit(), release: Some(2) };
    let call = world.request(10, [1; 16], ask.clone());
    assert!(world.reply(call).is_none());
    assert_eq!(world.spec_revision(7), Some(1));
    assert!(world.held(7));
    world.commit_all();
    let expected = Some(Reply::Outcome(Outcome::Amended { task: 7, released: true }));
    assert_eq!(world.reply(call), expected);
    assert_eq!(world.spec_revision(7), Some(2));
    assert!(!world.held(7));
    world.restart();
    let routes = world.stats().routes;
    let replay = world.request(10, [1; 16], ask);
    assert_eq!(world.reply(replay), expected);
    assert_eq!(world.stats().routes, routes);
    assert_eq!(world.spec_revision(7), Some(2));
    world.assert_settled();
}

#[test]
fn a_widening_amendment_with_release_is_proposed_and_released_on_acceptance() {
    let mut world = World::new(Settings { authority_refuses: true, ..Settings::calm(98) });
    member(&mut world);
    world.amended_task(7, 1);
    world.held_task(7, 2, jig_core_people::HoldReason::Stopped);
    let mut amendment = edit();
    amendment.authority = Some(jig_core_people::Authority {
        tools: 0,
        grants: Box::new([]),
        delegation: jig_core_people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
        spend: 1,
        deadline: None,
        notes: 0,
        note_resources: Box::new([]),
    });
    let call = world.request(10, [1; 16], Ask::Amend { project: 1, task: 7, revision: 1, amendment, release: Some(2) });
    world.commit_all();
    let Some(Reply::Outcome(Outcome::AmendProposed { task: 7, proposal, released: false })) = world.reply(call) else {
        panic!("widening proposed")
    };
    assert_eq!(world.spec_revision(7), Some(1));
    assert!(world.held(7));
    world.restart();
    let decision = Ask::DecideProposal { project: 1, proposer: 7, proposal, decision: ProposalDecision::Accept };
    let call = world.request(10, [2; 16], decision.clone());
    assert!(world.reply(call).is_none());
    assert_eq!(world.spec_revision(7), Some(1));
    assert!(world.held(7));
    world.commit_all();
    assert!(matches!(
        world.reply(call),
        Some(Reply::Outcome(Outcome::ProposalDecided { choice: ProposalChoice::Accepted, .. }))
    ));
    assert_eq!(world.spec_revision(7), Some(2));
    assert!(!world.held(7));
    world.restart();
    let replay = world.request(10, [2; 16], decision);
    assert_eq!(world.reply(replay), world.reply(call));
    assert_eq!(world.spec_revision(7), Some(2));
    world.assert_settled();
}
