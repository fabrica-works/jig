const SIGN_IN: u64 = 10;
const CREATE: u8 = 31;
const FIRST_MARK: u8 = 32;
const SECOND_MARK: u8 = 33;
const LAST_REPLY: u64 = 2;
use jig_domain_people::{
    Again, Ask, Entry, EntryKind, Holding, IdentityKey, InitialOwner, Outcome, ProposalChoice, ProposalDecision,
    Proposer, Refusal, Reply, Role, Seed, Whom,
};
use jig_people_world::fixtures::member;
use jig_people_world::{Settings, World};
use skein_lib::Duration;
use skein_world::domain::assert_replays;

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
        Ask::SetGoal {
            project: 1,
            spec: b"goal".to_vec().into_boxed_slice(),
            executor: jig_domain_people::GoalExecutor::Charter(1),
            budget: 1,
            priority: 1,
            deadline: None,
        },
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
        Ask::SetGoal {
            project: 1,
            spec: b"goal".to_vec().into_boxed_slice(),
            executor: jig_domain_people::GoalExecutor::Charter(1),
            budget: 1,
            priority: 1,
            deadline: None,
        },
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
        Ask::DecideProposal {
            project: 1,
            proposer: Proposer::Task(7),
            proposal: 8,
            decision: ProposalDecision::Accept,
        },
    );
    world.commit_all();
    let expected = Some(Reply::Outcome(Outcome::ProposalDecided {
        proposer: Proposer::Task(7),
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
            proposer: Proposer::Task(7),
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

fn person_task_world(seed: u64) -> (World, u64, u64) {
    let mut world = World::new(Settings::calm(seed));
    let first = world.signin(10, 1);
    let second = world.signin(11, 2);
    world.commit_all();
    let Reply::SignedIn { person: first, .. } = world.reply(first).expect("first sign-in") else { panic!() };
    let Reply::SignedIn { person: second, .. } = world.reply(second).expect("second sign-in") else { panic!() };
    world.roles(
        1,
        Box::new([
            Holding { person: first, role: Role::Maintainer },
            Holding { person: second, role: Role::Maintainer },
        ]),
    );
    world.commit_all();
    world.waiting(
        99,
        Box::new([Entry {
            task: 99,
            project: 1,
            whom: Whom::Role { project: 1, role: 1 },
            kind: EntryKind::PersonTask,
            at: skein_lib::Wall::EPOCH,
        }]),
    );
    (world, first, second)
}

#[test]
fn an_answer_to_a_person_task_another_holder_took_is_refused_and_told_who() {
    let (mut world, first, _) = person_task_world(62);
    let take = world.request(10, [3; 16], Ask::TakePerson { project: 1, task: 99 });
    let answer = Ask::AnswerPerson {
        project: 1,
        task: 99,
        revision: 0,
        result: jig_domain_people::PersonResult::Report { words: b"answer".as_slice().into() },
    };
    let other = world.request(11, [4; 16], answer.clone());
    assert_eq!(world.reply(take), None);
    assert_eq!(world.reply(other), None);
    world.commit_all();
    let taken = Reply::Outcome(Outcome::Refused(Refusal::Taken { by: first }));
    assert_eq!(world.reply(other), Some(taken));
    world.advance(Duration::from_secs(1));
    let at = skein_lib::Wall::from_nanos(1_000_000_000);
    let winner = world.request(10, [5; 16], answer.clone());
    world.commit_all();
    assert_eq!(world.reply(winner), Some(Reply::Outcome(Outcome::PersonAnswered { task: 99 })));
    world.restart();
    let replay = world.request(11, [4; 16], answer.clone());
    world.commit_all();
    assert_eq!(world.reply(replay), Some(taken), "saved holder fact does not change after an answer");
    let late = world.request(11, [6; 16], answer);
    world.commit_all();
    assert_eq!(
        world.reply(late),
        Some(Reply::Outcome(Outcome::Refused(Refusal::Ended {
            how: jig_domain_people::Ending::Decided { by: first, at }
        })))
    );
    world.assert_settled();
}

#[test]
fn a_person_task_handed_back_is_taken_by_another_holder() {
    let (mut world, first, second) = person_task_world(63);
    world.request(10, [3; 16], Ask::TakePerson { project: 1, task: 99 });
    world.commit_all();
    let other = world.request(11, [4; 16], Ask::HandBackPerson { project: 1, task: 99 });
    world.commit_all();
    assert_eq!(world.reply(other), Some(Reply::Outcome(Outcome::Refused(Refusal::Taken { by: first }))));
    let handback = world.request(10, [5; 16], Ask::HandBackPerson { project: 1, task: 99 });
    world.commit_all();
    assert_eq!(world.reply(handback), Some(Reply::Outcome(Outcome::PersonHandedBack { task: 99 })));
    world.check_role_waiting(99, [first, second], true);
    world.restart();
    let take = world.request(11, [6; 16], Ask::TakePerson { project: 1, task: 99 });
    world.commit_all();
    assert_eq!(world.reply(take), Some(Reply::Outcome(Outcome::PersonTaken { task: 99 })));
    world.check_role_waiting(99, [first, second], false);
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
        limits: jig_domain_people::Limits { batch: 2, facts: 0, ..jig_people_world::LIMITS },
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
    world.held_task(7, 2, jig_domain_people::HoldReason::Budget);
    let ask = Ask::Release { project: 1, task: 7, revision: 1, again: None };
    let call = world.request(10, [7; 16], ask.clone());
    assert!(world.reply(call).is_none());
    world.commit_all();
    let expected = Some(Reply::Outcome(Outcome::Refused(Refusal::Moved {
        now: jig_domain_people::Moved::Hold { revision: 2, reason: jig_domain_people::HoldReason::Budget },
    })));
    assert_eq!(world.reply(call), expected);
    world.held_task(7, 3, jig_domain_people::HoldReason::Deadline);
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
        amendment: jig_domain_people::Amendment {
            spec: Some(jig_domain_people::Spec {
                words: b"new words".as_slice().into(),
                parameters: Box::new([]),
                inputs: Box::new([]),
            }),
            ..jig_people_world::fixtures::person_unchanged(b"new evidence")
        },
    };
    let call = world.request(10, [8; 16], ask.clone());
    world.commit_all();
    let expected =
        Some(Reply::Outcome(Outcome::Refused(Refusal::Moved { now: jig_domain_people::Moved::Spec { revision: 2 } })));
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
    Ask::Application {
        scope: jig_domain_people::ApplicationScope::Project(1),
        connector: 2,
        digest: Sha256::digest(words).into(),
    }
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
    world.held_task(7, 1, jig_domain_people::HoldReason::StoppedBy { party: person });
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
    world.held_task(7, 1, jig_domain_people::HoldReason::Uncertain { entry: 9 });
    let ask = Ask::Release { project: 1, task: 7, revision: 1, again: None };
    let call = world.request(10, [3; 16], ask.clone());
    world.commit_all();
    assert_eq!(
        world.reply(call),
        Some(Reply::Outcome(Outcome::Refused(Refusal::Authority {
            proposable: None,
            lacking: jig_domain_people::Lack::Again { required: true }
        })))
    );
    world.held_task(7, 2, jig_domain_people::HoldReason::Budget);
    world.restart();
    let replay = world.request(10, [3; 16], ask);
    assert_eq!(world.reply(replay), world.reply(call));
}

#[test]
fn an_uncertain_effect_released_without_making_it_again_tells_its_executor() {
    let mut world = World::new(Settings::calm(95));
    member(&mut world);
    world.held_task(7, 1, jig_domain_people::HoldReason::Uncertain { entry: 9 });
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

fn edit() -> jig_domain_people::Amendment {
    jig_domain_people::Amendment {
        spec: Some(jig_domain_people::Spec {
            words: b"changed".as_slice().into(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        }),
        ..jig_people_world::fixtures::person_unchanged(b"evidence")
    }
}

#[test]
fn an_amendment_and_a_release_are_one_decision() {
    let mut world = World::new(Settings::calm(97));
    member(&mut world);
    world.amended_task(7, 1);
    world.held_task(7, 2, jig_domain_people::HoldReason::Stopped);
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
    world.held_task(7, 2, jig_domain_people::HoldReason::Stopped);
    let mut amendment = edit();
    amendment.authority = Some(jig_domain_people::Authority {
        tools: 0,
        grants: Box::new([]),
        delegation: jig_domain_people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
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
    let decision =
        Ask::DecideProposal { project: 1, proposer: Proposer::Task(7), proposal, decision: ProposalDecision::Accept };
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

#[test]
fn reading_the_inbox_moves_no_position_until_the_party_marks_it_seen() {
    let mut world = World::new(Settings::calm(80));
    let person = member(&mut world);
    world.remember_result(person, 7, 40);
    assert_eq!(world.counts(person).unread, 1);
    assert!(world.inbox(person).is_empty());
    assert_eq!(world.results_seen(person), 0);
    let seen = jig_domain_people::Seen::Results { through: 40 };
    let first = world.request(10, [1; 16], Ask::MarkSeen { seen });
    let duplicate = world.request(10, [1; 16], Ask::MarkSeen { seen });
    assert!(world.reply(first).is_none() && world.reply(duplicate).is_none());
    assert_eq!(world.results_seen(person), 40, "live state runs ahead but replies await durability");
    world.commit_all();
    assert_eq!(world.reply(first), Some(Reply::Outcome(Outcome::MarkedSeen { seen })));
    assert_eq!(world.reply(first), world.reply(duplicate));
    assert_eq!(world.counts(person).unread, 0);
    world.restart();
    assert_eq!(world.results_seen(person), 40);
    let retry = world.request(10, [1; 16], Ask::MarkSeen { seen });
    world.commit_all();
    assert_eq!(world.reply(retry), Some(Reply::Outcome(Outcome::MarkedSeen { seen })));
    world.assert_settled();
}

#[test]
fn a_chat_marked_seen_leaves_other_chats_unread() {
    let mut world = World::new(Settings::calm(81));
    let person = member(&mut world);
    let mut chats = Vec::new();
    for key in [1, 2] {
        let call = world.ask(10, [key; 16], b"hello");
        world.commit_all();
        let Some(Reply::Outcome(Outcome::Started { task })) = world.reply(call) else { panic!("chat made") };
        chats.push(task);
        world.waiting(
            task,
            Box::new([Entry {
                task,
                project: 1,
                whom: Whom::Person(person),
                kind: EntryKind::ChatReply { message: 1 },
                at: skein_lib::Wall::EPOCH,
            }]),
        );
    }
    assert_eq!(world.counts(person).unread, 2);
    let seen = jig_domain_people::Seen::Chat { task: chats[0], through: 1 };
    let call = world.request(10, [3; 16], Ask::MarkSeen { seen });
    assert!(world.reply(call).is_none());
    world.commit_all();
    assert_eq!(world.reply(call), Some(Reply::Outcome(Outcome::MarkedSeen { seen })));
    assert_eq!(world.counts(person).unread, 1);
    assert_eq!(world.inbox(person).len(), 1);
    assert_eq!(world.chat_seen(person, chats[1]), 0);
    let other_call = world.signin(20, 2);
    world.commit_all();
    let Some(Reply::SignedIn { person: other, .. }) = world.reply(other_call) else { panic!("other party") };
    world.roles(1, Box::new([Holding { person, role: Role::Member }, Holding { person: other, role: Role::Member }]));
    world.commit_all();
    let refused = world.request(20, [4; 16], Ask::MarkSeen { seen });
    world.commit_all();
    assert_eq!(
        world.reply(refused),
        Some(Reply::Outcome(Outcome::Refused(Refusal::Role { held: Some(Role::Member) })))
    );
    assert_eq!(world.chat_seen(other, chats[0]), 0);
    world.restart();
    assert_eq!(world.chat_seen(person, chats[0]), 1);
    world.forget_chat(chats[0]);
    world.commit_all();
    assert_eq!(world.chat_seen(person, chats[0]), 0);
    assert!(!world.records().contains_key(&jig_domain_people::Key::ReadChat { party: person, task: chats[0] }));
    world.assert_settled();
}

#[test]
fn unread_updates_past_the_limit_are_said_as_more() {
    let mut world = World::new(Settings::calm(82));
    let person = member(&mut world);
    let limit = jig_people_world::LIMITS.inbox_entries;
    for position in 1..=u64::from(limit) + 2 {
        world.remember_result(person, position, position);
    }
    assert_eq!(world.counts(person), jig_domain_people::Counts { needs: 0, unread: limit, more_unread: true });
    let seen = jig_domain_people::Seen::Results { through: 2 };
    world.request(10, [1; 16], Ask::MarkSeen { seen });
    world.commit_all();
    assert_eq!(world.counts(person), jig_domain_people::Counts { needs: 0, unread: limit, more_unread: false });
    world.assert_settled();
}

#[test]
fn a_conversations_read_position_moves_to_the_ended_family_and_reads_back_after_a_restart() {
    let mut world = World::new(Settings::calm(840));
    let person = member(&mut world);
    let call = world.ask(SIGN_IN, [CREATE; 16], b"hello");
    world.commit_all();
    let Some(Reply::Outcome(Outcome::Started { task })) = world.reply(call) else { panic!("chat made") };
    world.waiting(
        task,
        Box::new([Entry {
            task,
            project: 1,
            whom: Whom::Person(person),
            kind: EntryKind::ChatReply { message: LAST_REPLY },
            at: skein_lib::Wall::EPOCH,
        }]),
    );
    let first = world.request(
        SIGN_IN,
        [FIRST_MARK; 16],
        Ask::MarkSeen { seen: jig_domain_people::Seen::Chat { task, through: 1 } },
    );
    world.commit_all();
    assert!(world.reply(first).is_some());
    world.end_conversation(person, task, LAST_REPLY);
    world.commit_all();
    world.restart();
    assert_eq!(world.chat_seen(person, task), 0);
    let seen = jig_domain_people::Seen::Chat { task, through: LAST_REPLY };
    let second = world.request(SIGN_IN, [SECOND_MARK; 16], Ask::MarkSeen { seen });
    world.commit_all();
    assert_eq!(world.reply(second), Some(Reply::Outcome(Outcome::MarkedSeen { seen })));
    assert_eq!(
        world.records().get(&jig_domain_people::Key::EndedRead { task, party: person }),
        Some(&jig_domain_people::Stored::EndedRead { task, party: person, through: LAST_REPLY })
    );
    assert!(!world.records().contains_key(&jig_domain_people::Key::ReadChat { task, party: person }));
}
