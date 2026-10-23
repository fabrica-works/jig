//! A holder's moves cross people, the core, the child and the scripted component.
use super::*;

fn change(change: people::AccountChange) -> people::Ask {
    people::Ask::AccountChange { holder: people::AccountHolder::Own, change }
}
fn move_out(world: &mut World, account: u32, source: u32) -> u32 {
    let expected = world.domain.accounts().next_number();
    assert_eq!(
        ask(world, 50, change(people::AccountChange::MoveOut { account })),
        Some(&people::Reply::Outcome(people::Outcome::AccountMovingOut { account, number: expected, source }))
    );
    expected
}
fn watch_own(world: &mut World) {
    world.client_peer = Some(jig_core_world::client_peer::ClientPeer::routed());
    watch(world, world.sign_in.expect("signed in"), OWN_WATCH, core::documents::Watched::Accounts);
    take(world, OWN_WATCH);
}
fn moving(seed: u64) -> (World, u32, u32, u32) {
    let mut world = signing(seed);
    let (account, source) = signed_in(&mut world);
    let number = move_out(&mut world, account, source);
    (world, account, source, number)
}
fn refused(world: &World, why: people::AccountMoveWhy) {
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::AccountMove { why })))
    );
}
fn move_in(world: &mut World, key: u8, number: u64, digest: u8) {
    carry(world, key, change(people::AccountChange::MoveIn { digest: [digest; 32] }), number, digest);
}

#[test]
fn a_signed_in_account_moves_out_and_its_record_is_taken_once() {
    let (mut world, _, source, _) = moving(190);
    assert_eq!(world.take_moved(source).expect("moved bytes").expose(), format!("moved:{source}").as_bytes());
    assert!(world.take_moved(source).is_none());
    let fact = world.domain.accounts().source(source).expect("moving source");
    assert_eq!((fact.state, fact.usable, fact.generation), (accounts::State::Moving, false, None));
}

#[test]
fn the_accounts_document_shows_the_source_moving() {
    let mut world = signing(191);
    let (account, source) = signed_in(&mut world);
    watch_own(&mut world);
    move_out(&mut world, account, source);
    let document = accounts_document(&world, OWN_WATCH);
    let line = document.accounts.get(0).expect("own account");
    assert_eq!(line.number, account);
    let source = line.sources.get(0).expect("moving source");
    assert_eq!((source.state, source.usable), (core::documents::AccountState::Moving, false));
}

#[test]
fn the_deployment_document_shows_its_signed_in_source_moving() {
    let (mut config, mut limits) = signing_configuration(204);
    limits.core.accounts.holder_accounts = 2;
    jig_core_world::peers::journal(&mut limits);
    config.core.settings.llm_providers.providers.first_mut().expect("provider").signed_in_plan = false;
    let mut world = World::configured(204, false, config, limits);
    let sign_in = world.sign_in.expect("signed in");
    let person = party(&world, sign_in);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 0,
        holdings: Box::new([people::Holding { person, role: people::Role::Owner }]),
    })));
    let Some(people::Reply::Outcome(people::Outcome::AccountSignInStarted { source, .. })) = ask(
        &mut world,
        40,
        people::Ask::AccountSignIn { holder: people::AccountHolder::Deployment, provider: PROVIDER },
    )
    .cloned() else {
        panic!("deployment sign-in")
    };
    let account = world.domain.accounts().source(source).expect("signed in").account;
    world.client_peer = Some(jig_core_world::client_peer::ClientPeer::default());
    watch(&mut world, sign_in, DEPLOYMENT_WATCH, core::documents::Watched::Deployment);
    take(&mut world, DEPLOYMENT_WATCH);
    ask(
        &mut world,
        50,
        people::Ask::AccountChange {
            holder: people::AccountHolder::Deployment,
            change: people::AccountChange::MoveOut { account },
        },
    );
    let core::documents::Document::Deployment(document) = delivered(&world, DEPLOYMENT_WATCH) else {
        panic!("deployment document")
    };
    let line = document.accounts.iter().find(|line| line.number == account).expect("moving account");
    assert_eq!((line.state, line.usable), (core::documents::AccountState::Moving, false));
    assert_eq!(line.sources.get(0).expect("moving source").state, core::documents::AccountState::Moving);
}

#[test]
fn written_answers_done_and_removes_the_account_from_its_document() {
    let (mut world, account, source, number) = moving(192);
    watch_own(&mut world);
    assert_eq!(
        ask(&mut world, 51, change(people::AccountChange::Written { number })),
        Some(&people::Reply::Outcome(people::Outcome::AccountChanged))
    );
    assert!(accounts_document(&world, OWN_WATCH).accounts.is_empty());
    assert!(world.domain.accounts().account(account).is_none());
    assert!(world.domain.accounts().source(source).is_none());
    assert!(world.take_moved(source).is_none());
}

#[test]
fn a_late_written_is_refused_as_cancelled() {
    let mut world = signing(193);
    ask(&mut world, 50, change(people::AccountChange::Written { number: 99 }));
    refused(&world, people::AccountMoveWhy::Cancelled);
}

#[test]
fn a_second_move_out_is_refused_while_the_source_is_moving() {
    let (mut world, account, _, _) = moving(194);
    ask(&mut world, 51, change(people::AccountChange::MoveOut { account }));
    refused(&world, people::AccountMoveWhy::Moving);
}

#[test]
fn a_key_only_account_cannot_move_out() {
    let mut world = signing(195);
    carry(&mut world, 40, llm_key(7), 1_u64 << 63, 7);
    let Some(people::Reply::Outcome(people::Outcome::AccountKeyKept { source })) = world.people_answers.last() else {
        panic!("key kept")
    };
    let account = world.domain.accounts().source(*source).expect("key source").account;
    ask(&mut world, 50, change(people::AccountChange::MoveOut { account }));
    refused(&world, people::AccountMoveWhy::NotSignedIn);
}

#[test]
fn a_carried_move_in_makes_an_account_and_refreshes_its_source() {
    let mut world = signing(196);
    watch_own(&mut world);
    move_in(&mut world, 50, 1_u64 << 63, 7);
    let Some(people::Reply::Outcome(people::Outcome::AccountMovedIn { account, renewed: false })) =
        world.people_answers.last()
    else {
        panic!("account moved in: {:?}", world.people_answers.last())
    };
    let account = *account;
    take(&mut world, OWN_WATCH);
    let document = accounts_document(&world, OWN_WATCH);
    let line = document.accounts.get(0).expect("moved account");
    assert_eq!(line.number, account);
    let source = line.sources.get(0).expect("signed-in source").number;
    let fact = world.domain.accounts().source(source).expect("refreshed source");
    assert_eq!((fact.state, fact.usable, fact.generation), (accounts::State::Granted, true, Some(1)));
    assert!(world.carried_unused.is_empty());
}

#[test]
fn a_move_in_of_another_applications_record_is_refused_as_not_ours() {
    let mut world = signing(197);
    world.fail_move_open = true;
    let before = world.people_answers.len();
    move_in(&mut world, 50, 1_u64 << 63, 7);
    refused(&world, people::AccountMoveWhy::NotOurs);
    unused_before_answer(&world, before, 1_u64 << 63);
}

#[test]
fn a_move_in_of_a_subject_another_holder_holds_is_refused_as_held() {
    let mut world = signing(198);
    signed_in(&mut world);
    jig_core_world::fixtures::second_party(&mut world, b"move-holder", people::Role::Member);
    let before = world.people_answers.len();
    move_in(&mut world, 50, 1_u64 << 63, 7);
    refused(&world, people::AccountMoveWhy::Held);
    unused_before_answer(&world, before, 1_u64 << 63);
}

#[test]
fn a_restart_cancels_the_move_and_refreshes_the_source_before_it_is_usable() {
    let mut world = signing(199);
    let (account, source) = signed_in(&mut world);
    world.announce_grant(source, 7, accounts::Form::Credential(accounts::Scheme::Bearer));
    let number = move_out(&mut world, account, source);
    world.restart_with(signing_configuration(199).0);
    let fact = world.domain.accounts().source(source).expect("returned source");
    assert_eq!((fact.account, fact.state, fact.usable), (account, accounts::State::Granted, true));
    assert_eq!(fact.generation, Some(8));
    assert!(world.take_moved(source).is_none());
    ask(&mut world, 51, change(people::AccountChange::Written { number }));
    refused(&world, people::AccountMoveWhy::Cancelled);
}

#[test]
fn a_moves_deadline_returns_and_refreshes_the_source() {
    let (mut world, _, source, _) = moving(205);
    let deadline = signing_configuration(205).1.core.accounts.move_deadline;
    let (now, wall) = world.clocks();
    world.fire_due(now.saturating_add(deadline), Wall::from_nanos(wall.as_nanos() + deadline.as_nanos()));
    let fact = world.domain.accounts().source(source).expect("returned source");
    assert_eq!((fact.state, fact.usable), (accounts::State::Granted, true));
    assert!(world.take_moved(source).is_none());
}

#[test]
fn a_move_out_replay_keeps_its_number_and_has_no_second_record() {
    let (mut world, account, source, number) = moving(200);
    world.take_moved(source).expect("first record");
    assert_eq!(
        ask(&mut world, 50, change(people::AccountChange::MoveOut { account })),
        Some(&people::Reply::Outcome(people::Outcome::AccountMovingOut { account, number, source }))
    );
    assert!(world.take_moved(source).is_none());
}

#[test]
fn a_move_in_replay_erases_the_unused_carried_value() {
    let mut world = signing(201);
    move_in(&mut world, 50, 1_u64 << 63, 7);
    assert!(matches!(
        world.people_answers.last(),
        Some(people::Reply::Outcome(people::Outcome::AccountMovedIn { renewed: false, .. }))
    ));
    let answer = world.people_answers.last().cloned();
    let before = world.people_answers.len();
    move_in(&mut world, 50, (1_u64 << 63) + 1, 7);
    assert_eq!(world.people_answers.last().cloned(), answer);
    unused_before_answer(&world, before, (1_u64 << 63) + 1);
}

#[test]
fn a_move_in_rejects_a_different_carried_digest_before_admission() {
    let mut world = signing(202);
    let before = world.people_answers.len();
    carry(&mut world, 50, change(people::AccountChange::MoveIn { digest: [7; 32] }), 1_u64 << 63, 8);
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Refused(people::Refusal::NotOffered)));
    unused_before_answer(&world, before, 1_u64 << 63);
}

#[test]
fn another_holder_cannot_move_out_or_finish_the_holders_move() {
    let (mut world, account, source, number) = moving(203);
    jig_core_world::fixtures::second_party(&mut world, b"another-holder", people::Role::Member);
    for (key, value) in [
        (51, (people::AccountChange::MoveOut { account }, people::Refusal::Unknown)),
        (
            52,
            (
                people::AccountChange::Written { number },
                people::Refusal::AccountMove { why: people::AccountMoveWhy::Cancelled },
            ),
        ),
    ] {
        assert_eq!(
            ask(&mut world, key, change(value.0)),
            Some(&people::Reply::Outcome(people::Outcome::Refused(value.1)))
        );
        assert_eq!(world.domain.accounts().source(source).expect("still moving").state, accounts::State::Moving);
    }
}
