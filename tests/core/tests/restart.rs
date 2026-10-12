use jig_core_world::effects::World;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;
use jig_test_system::Fault;
use skein_lib::{Duration, Token};

fn hello(world: &mut World, hosting: Box<[fleet::Hosted]>) {
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        channel: Token::new(7),
        hello: fleet::Hello {
            bounds: jig_core_world::fixtures::bounds(),
            stop_bound: Duration::from_millis(100),
            slots: 1,
            workstreams: Box::new([]),
            hosting,
        },
    })));
}

fn account(world: &mut World) {
    world.send(root::Event::Core(core::Event::Account(jig_domain_accounts::Event::Add {
        account: 1,
        generation: 1,
        valid: Some(Duration::from_secs(60)),
    })));
}

fn applied_without_answer(world: &mut World) {
    world.hold_writes = true;
    world.call(1, 400);
    let (number, call) = world.pending_writes.remove(0);
    let _lost = world.systems[usize::from(number - 1)].answer(call, Fault::AfterApply);
    world.hold_writes = false;
}

#[test]
fn the_restart_script_waits_for_each_step_and_each_connector() {
    let limits = jig_core_world::world::limits().core;
    let config = jig_core_world::world::config(111).core;
    let mut core = core::Core::new(config, &limits);
    let steps = [
        core::RestartStep::LoadCore,
        core::RestartStep::LoadInbox,
        core::RestartStep::RestoreConnector { connector: 1 },
        core::RestartStep::RestoreConnector { connector: 2 },
        core::RestartStep::AdoptRuns,
        core::RestartStep::ReadAfresh { connector: 1 },
        core::RestartStep::ReadAfresh { connector: 2 },
        core::RestartStep::SettleOutbox { connector: 1 },
        core::RestartStep::SettleOutbox { connector: 2 },
        core::RestartStep::Open,
    ];
    let mut request = core.restart_begin();
    for step in steps {
        assert_eq!(request, core::RestartRequest::Step(step));
        assert_eq!(core.restart_step(), Some(step));
        assert!(!core.restart_ready());
        assert_eq!(core.restart_begin(), core::RestartRequest::Idle);
        request = core.restart_done(step);
    }
    assert_eq!(request, core::RestartRequest::Idle);
    assert!(core.restart_ready());
    assert_eq!(core.restart_done(core::RestartStep::Open), core::RestartRequest::Idle);
}

#[test]
fn an_out_of_order_restart_completion_refuses_the_start_naming_its_step() {
    let limits = jig_core_world::world::limits().core;
    let mut core = core::Core::new(jig_core_world::world::config(112).core, &limits);
    assert_eq!(core.restart_begin(), core::RestartRequest::Step(core::RestartStep::LoadCore));
    assert_eq!(
        core.restart_done(core::RestartStep::Open),
        core::RestartRequest::Refused { step: core::RestartStep::LoadCore }
    );
    assert_eq!(core.restart_failure(), Some(core::RestartStep::LoadCore));
    assert!(!core.restart_ready());
}

#[test]
fn an_uncertain_effect_across_restarts_is_made_before_its_retry_deadline_and_once() {
    let mut world = World::new(113, false);
    applied_without_answer(&mut world);
    let (key, deadline) = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            root::Record::Connector { number: 1, record: connector::Record::Outbox(row) } => {
                Some((row.key, row.attempt.expect("attempt committed").deadline))
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("sent outbox record");
    world.restart(113, false);
    assert!(world.domain.ready());
    assert!(world.store.rows.values().any(|record| matches!(record, root::Record::Connector { number: 1, record: connector::Record::Made { key: found, .. } } if *found==key)));
    assert!(
        world
            .store
            .rows
            .values()
            .any(|record| matches!(record, root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(_)))))
    );
    let applied = world.systems[0].observed();
    assert_eq!(applied.iter().filter(|row| row.applied).count(), 1);
    assert!(world.trace.iter().any(|line| line.contains("Look")));
    assert!(world.store.rows.values().any(|record| matches!(record, root::Record::Core(core::Record::Core(core::CoreRecord::Call(row))) if matches!(row.part, core::CallPart::Effect { outcome: Some(core::connector::OutboxOutcome::Made), .. }))));
    assert!(deadline > world.wall_time(), "lookup resolved long before the saved retry deadline");
    world.restart(113, false);
    assert_eq!(world.systems[0].observed().iter().filter(|row| row.applied).count(), 1);
}

#[test]
fn nothing_is_decided_before_every_connector_has_settled_its_outbox() {
    let mut world = World::new(115, false);
    applied_without_answer(&mut world);
    world.end(tasks::End::Parked, 0);
    for record in world.store.rows.values_mut() {
        if let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = record {
            row.phase = tasks::Phase::Active(tasks::Active::Due);
        }
    }
    world.assigned = None;
    world.hold_lookups = true;
    world.restart(115, false);
    assert!(!world.domain.ready());
    assert_eq!(world.pending_lookups.len(), 1);
    hello(&mut world, Box::new([]));
    account(&mut world);
    assert!(world.assigned.is_none(), "a due run cannot be assigned while recovery waits");
    world.finish_lookups();
    assert!(world.domain.ready());
    assert!(world.assigned.is_some(), "the due run starts after every connector settled");
}

#[test]
fn a_restart_after_a_turn_committed_resumes_from_it() {
    let mut world = World::new(116, false);
    world.turn(1, 3, None, b"committed whole turn");
    let assignment = world.assigned.as_ref().expect("live worker");
    let hosted = fleet::Hosted {
        run: Token::new(assignment.task),
        attempt: Token::new(assignment.attempt),
        phase: fleet::Phase::Active,
    };
    world.restart(116, false);
    hello(&mut world, Box::new([hosted]));
    world.end(tasks::End::Parked, 3);
    account(&mut world);
    world.say(90);
    assert_eq!(
        world.assigned.as_ref().expect("resumed").transcript.as_ref(),
        [Box::<[u8]>::from(&b"committed whole turn"[..])]
    );
    assert_eq!(world.spent(), 3);
}

#[test]
fn an_engine_claim_is_settled_at_once_instead_of_waiting_for_a_worker() {
    let mut world = World::new(117, false);
    world.turn(1, 2, None, b"engine turn");
    for record in world.store.rows.values_mut() {
        if let root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof))) = record {
            proof.host = fleet::HostKind::Engine;
        }
    }
    world.restart(117, false);
    assert!(world.store.rows.values().any(|record| matches!(record, root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) if row.tries.lost==1)));
    assert!(world.domain.ready());
}

#[test]
fn an_unfound_uncertain_entry_keeps_its_saved_retry_deadline_across_restarts() {
    let mut world = World::new(118, false);
    world.hold_writes = true;
    world.call(1, 401);
    let attempt = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            root::Record::Connector { record: connector::Record::Outbox(row), .. } => row.attempt,
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("durable attempt");
    world.restart(118, false);
    assert!(world.domain.ready());
    assert!(world.systems[0].observed().is_empty(), "not found does not retry before the saved deadline");
    assert!(world.store.rows.values().any(|record| matches!(record, root::Record::Connector { record: connector::Record::Outbox(row), .. } if row.attempt==Some(attempt))));
    world.restart(118, false);
    assert!(world.systems[0].observed().is_empty());
    assert!(world.store.rows.values().any(|record| matches!(record, root::Record::Connector { record: connector::Record::Outbox(row), .. } if row.attempt==Some(attempt))));
}

#[test]
fn an_uncertain_effect_released_without_making_it_again_tells_its_executor() {
    use jig_domain_people as people;
    use skein_lib::ReplyTo;
    let mut world = World::new(119, false);
    world.recovery = connector::Recovery::Unrecoverable;
    world.restart(119, false);
    let task = world.call_key(1).task;
    applied_without_answer(&mut world);
    world.end(tasks::End::Parked, 0);
    world.restart(119, false);
    assert!(world.domain.ready());
    assert!(world.store.rows.values().any(|record| matches!(record, root::Record::Connector { record: connector::Record::Outbox(row), .. } if row.phase==connector::EffectPhase::Held)));
    assert!(world.store.rows.values().any(|record| matches!(record, root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) if matches!(row.phase, tasks::Phase::Held { why: tasks::Hold::Uncertain { .. }, .. }))));
    assert_eq!(world.systems[0].observed().iter().filter(|row| row.applied).count(), 1);
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))]
    else {
        panic!("live held task")
    };
    let revision = row.revisions.hold;
    let sign_in = world.sign_in.expect("restored sign-in");
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(7001)),
        sign_in,
        key: [80; 16],
        ask: people::Ask::Release { project: 1, task, revision, again: None },
    })));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Authority {
            proposable: None,
            lacking: people::Lack::Again { required: true }
        })))
    );
    let made = world.systems[0].observed().iter().filter(|row| row.applied).count();
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    let answers = world.people_answers.len();
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(7002)),
        sign_in,
        key: [81; 16],
        ask: people::Ask::Release { project: 1, task, revision, again: Some(false) },
    })));
    assert_eq!(world.people_answers.len(), answers, "release answer waits for the outbox decision");
    world.release_commits();
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::Released { task })));
    assert_eq!(world.systems[0].observed().iter().filter(|row| row.applied).count(), made);
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))]
    else {
        panic!("live released task")
    };
    assert!(
        row.inbox.iter().any(|word| matches!(word.kind, tasks::MessageKind::EffectNotMade { .. })),
        "the resumed executor reads the not-made outcome"
    );
    assert!(!world.store.rows.values().any(
        |row| matches!(row, root::Record::Connector { record: connector::Record::Outbox(row), .. } if row.task == task)
    ));
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Call(row))) if row.key.task == task && matches!(row.part, core::CallPart::Effect { outcome: Some(core::connector::OutboxOutcome::NotMade), .. }))));
}

#[test]
fn a_refused_claim_keeps_the_conversation_for_the_next_resume() {
    let mut world = World::new(120, false);
    world.turn(1, 1, None, b"durable conversation");
    world.end(tasks::End::Parked, 1);
    let assignment = world.assigned.as_ref().expect("first assignment");
    let task = assignment.task;
    let key = root::Key::Core(core::Key::Core(core::CoreKey::RunProof(task)));
    let root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(saved))) = &world.store.rows[&key] else {
        panic!("durable proof")
    };
    let limits = jig_core_world::world::limits().core;
    let mut core = core::Core::new(jig_core_world::world::config(120).core, &limits);
    core.replaced.insert(task, saved.clone()).expect("one durable replacement");
    let mut pending = saved.clone();
    pending.attempt += 1;
    pending.conversation = pending.attempt;
    core.proofs.insert(task, pending.clone()).expect("one pending proof");
    core.claiming.insert(task, pending.attempt).expect("one pending claim");
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let _decision = core::step(&mut core, &env, core::Event::ClaimRefused { task });
    assert_eq!(core.proofs.get(&task), Some(saved));
    assert!(core.replaced.is_empty());
    core.begin_transcript(task, saved.attempt);
    assert_eq!(core.transcripts.get(&task).expect("next resume").from, saved.conversation);
}

#[test]
fn a_restart_restores_people_before_live_tasks_and_core_rows() {
    let mut world = World::new(120, false);
    world.restart(120, false);
    assert_eq!(
        world.restore_ranges,
        [
            core::RestoreRange::Deployment,
            core::RestoreRange::People,
            core::RestoreRange::Tasks,
            core::RestoreRange::Core,
        ]
    );
    assert!(world.domain.ready());
}

fn restored_task(world: &World, task: u64) -> &tasks::TaskRecord {
    let row = &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))];
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task))) = row else { panic!("live task") };
    task
}

#[test]
fn a_restart_with_workers_in_process_settles_their_claims_at_once() {
    let mut world = World::new(121, false);
    world.turn(1, 2, None, b"worker turn");
    let task = world.task();
    let before = world.wall_time().as_nanos();
    let mut config = jig_core_world::effects::fixture(121, false).0;
    config.core.workers_in_process = true;
    world.restart_with(config);
    assert!(world.domain.ready());
    assert_eq!(restored_task(&world, task).tries.lost, 1);
    assert!(world.wall_time().as_nanos() - before < Duration::from_secs(5).as_nanos());
}

#[test]
fn a_restart_with_workers_in_process_assigns_without_a_grace() {
    let mut world = World::new(122, false);
    let (task, attempt) = (world.task(), world.attempt());
    let before = world.wall_time().as_nanos();
    let mut config = jig_core_world::effects::fixture(122, false).0;
    config.core.workers_in_process = true;
    world.assigned = None;
    world.restart_with(config);
    assert_eq!(restored_task(&world, task).refusals, 1, "an unworked claim was not started");
    assert_eq!(restored_task(&world, task).tries.lost, 0);
    hello(&mut world, Box::new([]));
    account(&mut world);
    world.advance(Duration::from_millis(100));
    let assigned = world.assigned.as_ref().expect("next attempt passes only its retry backoff");
    assert_eq!(assigned.task, task);
    assert!(assigned.attempt > attempt);
    assert!(world.wall_time().as_nanos() - before < Duration::from_secs(5).as_nanos());
}

#[test]
fn a_restart_with_workers_elsewhere_still_waits_for_them() {
    let mut world = World::new(123, false);
    world.turn(1, 2, None, b"remote turn");
    let (task, attempt) = (world.task(), world.attempt());
    world.assigned = None;
    world.restart(123, false);
    account(&mut world);
    world.advance(Duration::from_millis(100));
    assert!(world.assigned.is_none());
    assert_eq!(restored_task(&world, task).tries.lost, 0);
    hello(
        &mut world,
        Box::new([fleet::Hosted { run: Token::new(task), attempt: Token::new(attempt), phase: fleet::Phase::Active }]),
    );
    assert_eq!(restored_task(&world, task).attempt, attempt);
    assert_eq!(restored_task(&world, task).phase, tasks::Phase::Active(tasks::Active::Running { attempt }));
    assert_eq!(restored_task(&world, task).tries.lost, 0);
}
