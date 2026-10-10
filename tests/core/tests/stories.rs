use jig_core as core;
use jig_core_authority as authority;
use jig_core_notes as notes;
use jig_core_people as people;
use jig_core_tasks as tasks;
use jig_core_world::effects::{World, fixture};
use jig_test_connector as connector;
use jig_test_domain as root;
use skein_lib::{List, Queue, ReplyTo, Token};

#[path = "stories/address.rs"]
mod address;

fn ask(world: &mut World, request: u8, ask: people::Ask) {
    ask_as(world, world.sign_in.expect("signed-in person"), request, ask);
}

fn ask_as(world: &mut World, sign_in: u64, request: u8, ask: people::Ask) {
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(6000 + u64::from(request))),
        sign_in,
        key: [request; 16],
        ask,
    })));
}

fn action(world: &mut World, completion: u32, action: core::NamedAction) {
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(7000 + u64::from(completion))),
        key: world.call_key(completion),
        action,
    }));
}

fn running(world: &World, task: u64) -> (u64, u64) {
    *world.assignments.iter().rev().find(|(number, _)| *number == task).expect("task assigned")
}

fn finish(world: &mut World, run: (u64, u64), words: &[u8]) {
    world.send(root::Event::Answer {
        channel: Token::new(7),
        task: run.0,
        attempt: run.1,
        cumulative: 0,
        end: tasks::End::Finished {
            result: tasks::TaskResult::Report { words: words.into() },
            cancel_delegates: false,
        },
    });
}

fn delegate(world: &mut World, run: (u64, u64), completion: u32, member: core::Delegate) -> u64 {
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(9000 + u64::from(completion))),
        key: core::CallKey { task: run.0, attempt: run.1, completion, position: 0 },
        batch: Box::new([member]),
        stubs: Box::new([]),
    }));
    let core::CallPart::Delegated(numbers) = &world.answers.last().expect("delegate answer").2 else {
        panic!("delegation was not admitted");
    };
    numbers[0]
}

fn member(world: &World, executor: tasks::Executor, words: &[u8]) -> core::Delegate {
    let mut authority = world.assigned.as_ref().expect("task assigned").run.authority.clone();
    authority.budget.spend = 20;
    authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: 1, depth: 1 };
    core::Delegate {
        executor,
        spec: tasks::Spec { words: words.into(), parameters: Box::new([]), inputs: Box::new([]) },
        contract: tasks::Contract::Report { words: 128 },
        authority,
        symbolic_grants: Box::new([]),
        dependencies: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
    }
}

fn plan_world(seed: u64) -> World {
    let (configuration, limits) = jig_core_world::faults::plan_fixture(seed);
    World::configured(seed, false, configuration, limits)
}

#[test]
fn a_plan_of_reports_and_a_procedure_is_revised_midway_and_done() {
    let mut world = plan_world(124);
    let parent = world.assignments[0];
    let mut report = member(&world, tasks::Executor::Agent { charter: 1 }, b"first report");
    report.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let report = delegate(&mut world, parent, 1, report);
    let report_run = running(&world, report);
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(9101)),
        key: core::CallKey { task: parent.0, attempt: parent.1, completion: 2, position: 0 },
        action: core::NamedAction::Amend {
            target: report,
            amendment: tasks::Amendment {
                addressed: None,
                release: None,
                instructions: None,
                spec: Some(tasks::Spec {
                    words: b"revised report".as_slice().into(),
                    parameters: Box::new([]),
                    inputs: Box::new([]),
                }),
                wake: None,
                dependencies: None,
                authority: None,
                reason: b"new evidence".as_slice().into(),
            },
        },
    }));
    assert!(world.inbound.iter().any(
        |word| matches!(word.kind, tasks::MessageKind::Amendment { .. }) && word.words.as_ref() == b"new evidence"
    ));
    finish(&mut world, report_run, b"revised result");
    let mut second = member(&world, tasks::Executor::Agent { charter: 1 }, b"second report");
    second.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let second = delegate(&mut world, parent, 3, second);
    let second_run = running(&world, second);
    finish(&mut world, second_run, b"alternative result");
    let mut choice = member(&world, tasks::Executor::Agent { charter: 1 }, b"choose between the reports");
    choice.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    choice.contract = tasks::Contract::Verdict {
        choices: Box::new([tasks::Verdict { fields: Box::new([]), code: 1, words: 128, followups: 0 }]),
    };
    let choice = delegate(&mut world, parent, 4, choice);
    let choice_run = running(&world, choice);
    world.send(root::Event::Answer {
        channel: Token::new(7),
        task: choice_run.0,
        attempt: choice_run.1,
        cumulative: 0,
        end: tasks::End::Finished {
            result: tasks::TaskResult::Verdict {
                fields: Box::new([]),
                code: 1,
                words: b"choose the revised report".as_slice().into(),
            },
            cancel_delegates: false,
        },
    });
    let mut procedure = member(&world, tasks::Executor::Procedure { connector: 1, code: 1 }, b"carry out the choice");
    procedure.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let procedure = delegate(&mut world, parent, 5, procedure);
    assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(procedure)))));
    finish(&mut world, parent, b"plan done");
    for task in [report, second, choice, procedure, parent.0] {
        assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))));
    }
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::History(history))) if history.task==report && history.change==tasks::Change::Amended)));
}

#[test]
fn a_recurring_task_creates_one_batch_per_period_after_the_reset() {
    let mut world = plan_world(125);
    world.end(tasks::End::Parked, 0);
    let mut authority = member(&world, tasks::Executor::Agent { charter: 1 }, b"template").authority;
    authority.budget.spend = 30;
    let mut member_authority = authority.clone();
    member_authority.budget.spend = 10;
    member_authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    world.send(root::Event::Core(core::Event::StartRecurring {
        project: 1,
        authority,
        template: tasks::RecurringTemplate {
            key: 1,
            overlap: tasks::RecurringOverlap::Skip,
            batch: Box::new([tasks::New {
                number: 1,
                project: 1,
                executor: tasks::Executor::Agent { charter: 1 },
                spec: tasks::Spec {
                    words: b"period work".as_slice().into(),
                    parameters: Box::new([]),
                    inputs: Box::new([]),
                },
                contract: tasks::Contract::Report { words: 128 },
                authority: member_authority,
                numbers: tasks::Numbers { budget: 10, spent: 0, spent_below: 0, reserved: 0 },
                funder: tasks::Funder::Period { project: 1, period: 0 },
                dependencies: Box::new([]),
                holdings: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked: None,
            }]),
        },
    }));
    let first = *world.assignments.last().expect("first period run");
    assert_ne!(first.0, world.assignments[0].0);
    finish(&mut world, first, b"first period done");
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 2, budget: 1000, next: None }));
    let second = *world.assignments.last().expect("second period run");
    assert_ne!(second.0, first.0);
    assert!(
        matches!(world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(second.0)))), Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) if matches!(row.funder, tasks::Funder::Recurring { period: 2, .. }))
    );
    let count = world.assignments.len();
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 2, budget: 1000, next: None }));
    assert_eq!(world.assignments.len(), count);
}

#[test]
fn a_policy_narrowed_while_a_proposal_waits_keeps_the_committed_entry_and_checks_the_live_run() {
    let (configuration, mut limits) = fixture(126, true);
    limits.core.call_records = 8;
    limits.journal.writes = 10_000;
    limits.journal.held = 10_000;
    let mut world = World::configured(126, true, configuration, limits);
    world.call(1, 9200);
    world.observed(13);
    world.hold_writes = true;
    world.call(1, 9201);
    assert_eq!(world.pending_writes.len(), 1);
    let proposal = world.propose(2);
    let run = world.assignments[0];
    ask(
        &mut world,
        43,
        people::Ask::ChangePolicy {
            project: 1,
            change: people::PolicyChange::Requirements {
                requirements: Box::new([people::Requirement {
                    connector: 1,
                    kind: 5,
                    pattern: people::Pattern { segments: Box::new([]), last: people::Last::Open(Box::new([])) },
                    judge: people::Judge { connector: 2, requirement: 9, parameters: 0 },
                    guard: people::Guard::Guarded,
                    must_be_guarded: true,
                }]),
            },
        },
    );
    assert!(
        world
            .people_answers
            .iter()
            .any(|reply| matches!(reply, people::Reply::Outcome(people::Outcome::PolicyChanged { project: 1 })))
    );
    world.release_writes();
    assert_eq!(world.systems[0].observed().iter().filter(|row| row.applied).count(), 1);
    world.decide_proposal(proposal, true, 44);
    assert!(
        world
            .store
            .rows
            .contains_key(&root::Key::Connector { number: 1, key: connector::RecordKey::Proposal(proposal) })
    );
    world.call(3, 9203);
    assert!(matches!(
        world.answers.last(),
        Some((_, _, core::CallPart::EffectDenied { answer: authority::Answer::Refuse, .. }))
    ));
    assert_eq!(world.systems[0].observed().iter().filter(|row| row.applied).count(), 1);
    assert!(world.cancellations.is_empty());
    assert_eq!(*world.assignments.last().expect("run remains live"), run);
    assert_eq!(world.spent(), 3);
}

#[test]
fn drift_holds_a_task_and_the_client_reads_why_it_needs_a_decision() {
    let mut world = World::new(127, false);
    let task = world.call_key(1).task;
    let resource = jig_test_connector_world::path(1, 1);
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::Adopt { project: 1, resource: resource.clone(), role: connector::ResourceRole::Owned },
    });
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::Names { task, project: 1, resources: Box::new([resource.clone()]) },
    });
    for state in [10, 11] {
        world.systems[0].other_hand(&resource, state);
        world.send(root::Event::Connector {
            number: 1,
            event: connector::Event::System(connector::SystemEvent::Fact {
                resource: resource.clone(),
                fact: connector::Fact { state: Some(state), observed: world.wall_time(), pending: false },
                origin: connector::Origin::Other,
            }),
        });
    }
    assert_eq!(world.domain.escalation(1, task).expect("client held-task context").why, tasks::Hold::Drift);
    assert!(world.domain.escalation(99, task).is_none(), "another party cannot read the held reason");
    assert!(
        matches!(world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))), Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) if matches!(row.phase, tasks::Phase::Held { why: tasks::Hold::Drift, .. }))
    );
    assert_eq!(world.cancellations, [world.assignments[0]]);
}

#[test]
fn a_goal_cancelled_with_live_runs_and_an_effect_in_flight_closes_deepest_first() {
    let mut world = plan_world(128);
    let parent = world.assignments[0];
    let mut child = member(&world, tasks::Executor::Agent { charter: 1 }, b"child");
    child.authority.budget.spend = 40;
    let child = delegate(&mut world, parent, 1, child);
    let child_run = running(&world, child);
    let mut grandchild = member(&world, tasks::Executor::Agent { charter: 1 }, b"grandchild");
    grandchild.authority.budget.spend = 15;
    grandchild.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let grandchild = delegate(&mut world, child_run, 1, grandchild);
    let grandchild_run = running(&world, grandchild);
    world.hold_writes = true;
    world.call(1, 9301);
    assert_eq!(world.pending_writes.len(), 1, "the effect is committed and in flight");
    assert_eq!(world.call_key(1).task, grandchild);
    ask(&mut world, 45, people::Ask::Cancel { project: 1, task: child, reason: b"goal withdrawn".as_slice().into() });
    assert_eq!(world.cancellations, [grandchild_run, child_run]);
    for run in [grandchild_run, child_run] {
        world.send(root::Event::Answer {
            channel: Token::new(7),
            task: run.0,
            attempt: run.1,
            cumulative: 0,
            end: tasks::End::Parked,
        });
    }
    assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(grandchild)))));
    assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(child)))));
    world.release_writes();
    let ended: Vec<_> = world
        .commits
        .iter()
        .flatten()
        .filter_map(|write| match write {
            root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) => Some(row.number),
            root::Write::Save(_) | root::Write::Erase(_) => None,
        })
        .collect();
    assert_eq!(ended, [grandchild, child]);
    assert_eq!(world.systems[0].observed().iter().filter(|row| row.applied).count(), 1);
}

#[test]
fn a_note_written_corrected_by_a_person_and_recalled_keeps_the_corrected_body() {
    let (mut configuration, limits) = fixture(121, false);
    let mut rules = configuration.core.authority.rules().clone();
    rules.ceiling.notes = authority::Scopes::PROJECT;
    let mut policy = configuration.core.authority.policy(1).expect("project policy").clone();
    policy.ceiling.notes = authority::Scopes::PROJECT;
    policy.roles[0].authority.notes = authority::Scopes::PROJECT;
    let mut checked = authority::Domain::new(rules, limits.core.authority).expect("bounded note policy");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut checked, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    configuration.core.authority = checked;
    configuration.core.settings.chat_authority.notes = authority::Scopes::PROJECT;
    let mut world = World::configured(121, false, configuration, limits);
    let key = world.call_key(1);
    action(
        &mut world,
        1,
        core::NamedAction::Note {
            entry: notes::New {
                name: 0,
                scope: notes::Scope::Project { project: 1 },
                description: b"original".as_slice().into(),
                body: b"first body".as_slice().into(),
                references: List::with_capacity(1),
                author: notes::Author::Task { task: key.task, attempt: key.attempt },
            },
            recalled: None,
        },
    );
    let core::CallPart::NoteWritten { name, revision: 1 } = world.answers.last().expect("written answer").2 else {
        panic!("note write was not admitted");
    };
    ask(
        &mut world,
        41,
        people::Ask::EditNote {
            project: 1,
            name,
            scope: Box::new(people::NoteScope::Project),
            change: Box::new(people::NoteChange::Correct {
                description: b"corrected".as_slice().into(),
                body: b"second body".as_slice().into(),
                references: Box::new([]),
                recalled: 1,
            }),
        },
    );
    assert!(world.people_answers.iter().any(
        |reply| matches!(reply, people::Reply::Outcome(people::Outcome::NoteEdited { name: found }) if *found==name)
    ));
    action(&mut world, 2, core::NamedAction::Recall { by: notes::Recall::Name { name }, page: 0 });
    assert!(
        matches!(world.answers.last(), Some((_, _, core::CallPart::NoteRecalled { entries, more: false })) if entries.len()==1 && entries[0].revision==2 && entries[0].body.as_ref()==b"second body")
    );
    assert!(
        matches!(world.store.rows.get(&root::Key::Core(core::Key::Notes(notes::Key::Entry { name }))), Some(root::Record::Core(core::Record::Notes(notes::Record::Entry(entry)))) if entry.revision==2 && entry.author==notes::Author::Party { party: 1 })
    );
}

#[test]
fn a_persons_whole_words_reach_a_running_run_and_retire_only_with_its_committed_read() {
    let mut world = World::new(122, false);
    let message = world.say(42);
    let inbound = world.inbound.last().expect("running run received words");
    assert_eq!(inbound.number, message);
    assert_eq!(inbound.words.as_ref(), [42]);
    assert!(matches!(inbound.kind, tasks::MessageKind::Words));
    let task = world.call_key(1).task;
    assert!(
        matches!(world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))), Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) if row.inbox.iter().any(|word| word.number==message))
    );
    world.turn(1, 1, Some(message), b"read the person's complete words");
    assert!(
        matches!(world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))), Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) if row.inbox.is_empty())
    );
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) if turn.read==Some(message))));
}

#[test]
fn a_judges_observed_facts_may_change_after_a_verdict_but_new_effects_use_the_new_facts() {
    let mut world = World::new(123, true);
    world.call(1, 8000);
    world.observed(13);
    world.hold_writes = true;
    world.call(1, 8001);
    assert_eq!(world.pending_writes.len(), 1);
    let committed = world.store.applied;
    world.systems[1].other_hand(&jig_test_connector_world::path(1, 1), 14);
    world.send(root::Event::Connector {
        number: 2,
        event: connector::Event::System(connector::SystemEvent::Fact {
            resource: jig_test_connector_world::path(1, 1),
            fact: connector::Fact { state: Some(14), observed: world.wall_time(), pending: false },
            origin: connector::Origin::Other,
        }),
    });
    assert_eq!(world.store.applied, committed, "a new observation does not rewrite the committed entry");
    world.release_writes();
    assert_eq!(world.systems[0].observed().iter().filter(|entry| entry.applied).count(), 1);
    world.call(2, 8002);
    assert!(matches!(
        world.answers.last(),
        Some((_, _, core::CallPart::EffectDenied { answer: authority::Answer::Refuse, .. }))
    ));
    assert_eq!(world.systems[0].observed().iter().filter(|entry| entry.applied).count(), 1);
    assert_eq!(world.spent(), 3);
}

#[test]
fn an_exhausted_delegate_escalates_to_a_person_who_releases_it_to_finish() {
    let mut world = plan_world(128);
    let parent = world.assignments[0];
    let mut child = member(&world, tasks::Executor::Agent { charter: 1 }, b"retry this delegate");
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let child = delegate(&mut world, parent, 1, child);
    for index in 0..2 {
        let run = running(&world, child);
        world.send(root::Event::Answer {
            channel: Token::new(7),
            task: run.0,
            attempt: run.1,
            cumulative: 0,
            end: tasks::End::Failed(tasks::Class::Run),
        });
        if index == 0 {
            world.advance(skein_lib::Duration::from_secs(2));
            assert!(running(&world, child).1 > run.1);
        }
    }
    assert!(matches!(world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(child)))),
        Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))))
            if matches!(row.phase, tasks::Phase::Held { why: tasks::Hold::Failures(tasks::Class::Run), .. })));
    // The parent receives the escalation first, then passes its current revision
    // to the person. The person decides through their authenticated client face.
    let revision = match world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(child)))) {
        Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) => match row.escalation {
            tasks::Escalation::Waiting { revision, .. } => revision,
            tasks::Escalation::Unheld { .. }
            | tasks::Escalation::Routing { .. }
            | tasks::Escalation::Rejected { .. } => panic!("escalation not routed"),
        },
        row => panic!("held child missing: {row:?}"),
    };
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(9300)),
        key: core::CallKey { task: parent.0, attempt: parent.1, completion: 2, position: 0 },
        action: core::NamedAction::DecideEscalation { task: child, revision, choice: core::EscalationChoice::Pass },
    }));
    let revision = match world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(child)))) {
        Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) => match row.escalation {
            tasks::Escalation::Waiting { revision, holder: tasks::EscalationHolder::Person(_), .. } => revision,
            ref escalation @ (tasks::Escalation::Unheld { .. }
            | tasks::Escalation::Routing { .. }
            | tasks::Escalation::Waiting { .. }
            | tasks::Escalation::Rejected { .. }) => panic!("not passed to a person: {escalation:?}"),
        },
        row => panic!("held child missing: {row:?}"),
    };
    ask(
        &mut world,
        44,
        people::Ask::DecideEscalation {
            project: 1,
            task: child,
            revision,
            decision: people::EscalationDecision::Release,
        },
    );
    assert!(
        world
            .people_answers
            .iter()
            .any(|reply| matches!(reply, people::Reply::Outcome(people::Outcome::EscalationDecided { .. })))
    );
    let released = running(&world, child);
    finish(&mut world, released, b"delegate recovered");
    finish(&mut world, parent, b"parent done");
    for task in [child, parent.0] {
        assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))));
    }
}

fn resource_member(world: &World, id: u64, words: &[u8]) -> core::Delegate {
    let mut child = member(world, tasks::Executor::Agent { charter: 1 }, words);
    child.spec.parameters = Box::new([tasks::Parameter::Resource { name: 1, connector: 1, resource: id }]);
    child.authority.budget.spend = 10;
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    child
}

fn configure_resources(world: &mut World) {
    for id in [1, 2] {
        world.send(root::Event::Connector {
            number: 1,
            event: connector::Event::Adopt {
                project: 1,
                resource: jig_test_connector_world::path(1, id),
                role: connector::ResourceRole::Owned,
            },
        });
    }
    world.configure_holds(
        1,
        Box::new([
            tasks::Kind { connector: 1, kind: 1, hold: tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits } },
            tasks::Kind { connector: 1, kind: 2, hold: tasks::HoldKind::Pooled { taken: tasks::Taken::Waits } },
        ]),
    );
}

fn pool_size(world: &mut World, slots: u32) {
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::System(connector::SystemEvent::Pool {
            path: jig_test_connector_world::path(1, 2),
            slots,
            lost: Box::new([]),
        }),
    });
}

#[test]
fn two_tasks_waiting_for_the_last_pool_slot_take_it_one_at_a_time() {
    let mut world = plan_world(129);
    configure_resources(&mut world);
    pool_size(&mut world, 1);
    let parent = world.assignments[0];
    let mut children = Vec::new();
    for completion in 1..=3 {
        let spec = resource_member(&world, 2, b"pool work");
        children.push(delegate(&mut world, parent, completion, spec));
    }
    assert_eq!(world.assignments.len(), 2, "one pool holder and the parent");
    for (index, child) in children.iter().enumerate() {
        let run = running(&world, *child);
        assert_eq!(world.assignments.len(), index + 2);
        finish(&mut world, run, b"pool slot released");
    }
    finish(&mut world, parent, b"all pool work done");
}

#[test]
fn a_shrinking_pool_keeps_its_holders_and_the_waiter_waits_until_it_drains() {
    let mut world = plan_world(130);
    configure_resources(&mut world);
    pool_size(&mut world, 2);
    let parent = world.assignments[0];
    let mut children = Vec::new();
    for completion in 1..=3 {
        let spec = resource_member(&world, 2, b"pool work");
        children.push(delegate(&mut world, parent, completion, spec));
    }
    assert_eq!(world.assignments.len(), 3);
    pool_size(&mut world, 1);
    let first = running(&world, children[0]);
    finish(&mut world, first, b"first done");
    assert_eq!(world.assignments.len(), 3, "the second holder still occupies the sole remaining slot");
    let second = running(&world, children[1]);
    finish(&mut world, second, b"second done");
    assert_eq!(world.assignments.len(), 4);
    let third = running(&world, children[2]);
    finish(&mut world, third, b"waiter done");
    finish(&mut world, parent, b"pool drained");
}

#[test]
fn a_run_hands_a_resource_to_a_procedure_while_retaining_its_writer_slot() {
    let mut world = plan_world(131);
    configure_resources(&mut world);
    let parent = world.assignments[0];
    let mut holder = resource_member(&world, 1, b"hold the write resource");
    holder.authority.budget.spend = 20;
    holder.authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Procedure(1)]), tasks: 1, depth: 1 };
    let holder = delegate(&mut world, parent, 1, holder);
    let holder_run = running(&world, holder);
    let mut procedure = resource_member(&world, 1, b"carry out the write");
    procedure.executor = tasks::Executor::Procedure { connector: 1, code: 1 };
    procedure.authority.budget.spend = 5;
    let procedure = delegate(&mut world, holder_run, 1, procedure);
    let row = |task| match world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))) {
        Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) => row,
        row => panic!("live task missing: {row:?}"),
    };
    assert!(row(holder).holdings.is_empty(), "the task hold was handed to the procedure");
    assert!(row(procedure).holds_taken);
    assert!(world.store.rows.values().any(|row| matches!(row,
        root::Record::Core(core::Record::Tasks(tasks::Stored::Writer(slot)))
            if slot.writer == tasks::Writer::Run { task: holder, attempt: holder_run.1 })));
    assert!(world.systems[0].observed().is_empty(), "the procedure's effect waits for the writer");
    world.send(root::Event::Answer {
        channel: Token::new(7),
        task: holder,
        attempt: holder_run.1,
        cumulative: 0,
        end: tasks::End::Parked,
    });
    world.advance(skein_lib::Duration::from_secs(2));
    assert_eq!(world.systems[0].observed().iter().filter(|row| row.applied).count(), 1);
    assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(procedure)))));
    let holder_run = running(&world, holder);
    finish(&mut world, holder_run, b"handoff done");
    finish(&mut world, parent, b"parent done");
}

#[test]
fn a_lost_writer_is_read_afresh_only_after_its_loss_is_committed() {
    let mut world = plan_world(132);
    configure_resources(&mut world);
    let parent = world.assignments[0];
    let spec = resource_member(&world, 1, b"writer that disappears");
    let child = delegate(&mut world, parent, 1, spec);
    let run = running(&world, child);
    let reads = world.systems[0].reads().len();
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    world.send(root::Event::Core(core::Event::Fleet(jig_core_fleet::Event::Lost { channel: Token::new(7) })));
    world.advance(skein_lib::Duration::from_secs(6));
    assert_eq!(world.systems[0].reads().len(), reads, "fresh reads wait for the loss commit's completion");
    world.release_commits();
    world.advance(skein_lib::Duration::from_secs(1));
    assert_eq!(world.systems[0].reads().len(), reads + 1);
    assert!(!world.store.rows.values().any(|row| matches!(row,
        root::Record::Core(core::Record::Tasks(tasks::Stored::Writer(slot)))
            if slot.writer == tasks::Writer::Run { task: child, attempt: run.1 })));
}

#[test]
fn a_procedures_effect_writer_survives_a_cold_restart_and_frees_after_settlement() {
    let mut world = plan_world(133);
    configure_resources(&mut world);
    let parent = world.assignments[0];
    let mut procedure = resource_member(&world, 1, b"durable write flight");
    procedure.executor = tasks::Executor::Procedure { connector: 1, code: 1 };
    procedure.authority.budget.spend = 5;
    world.hold_writes = true;
    let procedure = delegate(&mut world, parent, 1, procedure);
    assert_eq!(world.pending_writes.len(), 1);
    assert!(world.store.rows.values().any(|row| matches!(row,
        root::Record::Core(core::Record::Tasks(tasks::Stored::Writer(slot)))
            if matches!(slot.writer, tasks::Writer::Effect { .. }))));
    world.restart_with(jig_core_world::faults::plan_fixture(133).0);
    world.release_writes();
    assert!(world.domain.ready(), "restart trace: {:#?}", world.trace.iter().rev().take(16).collect::<Vec<_>>());
    world.advance(skein_lib::Duration::from_secs(2));
    world.send(root::Event::ConnectorTimer { number: 1 });
    assert_eq!(world.systems[0].observed().iter().filter(|row| row.applied).count(), 1);
    assert!(!world.store.rows.values().any(|row| matches!(row,
        root::Record::Core(core::Record::Tasks(tasks::Stored::Writer(slot)))
            if matches!(slot.writer, tasks::Writer::Effect { .. }))));
    assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(procedure)))));
}

#[test]
fn an_exclusive_resource_that_refuses_waiting_refuses_the_second_batch() {
    let seed = 151;
    let (mut configuration, limits) = jig_core_world::faults::plan_fixture(seed);
    configuration.first.resources[0].hold = connector::Hold::Exclusive { mode: connector::HoldMode::Refuse };
    let mut world = World::configured(seed, false, configuration, limits);
    // The kind default waits: the individual connector report must override it.
    configure_resources(&mut world);
    let parent = world.assignments[0];
    let child = resource_member(&world, 1, b"first holder");
    let first = delegate(&mut world, parent, 1, child);
    let before = world.assignments.len();
    let child = resource_member(&world, 1, b"second holder");
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(9152)),
        key: core::CallKey { task: parent.0, attempt: parent.1, completion: 2, position: 0 },
        batch: Box::new([child]),
        stubs: Box::new([]),
    }));
    assert_eq!(world.assignments.len(), before);
    assert!(matches!(
        world.answers.last().expect("second batch answer").2,
        core::CallPart::DelegationRefused(ref problem) if problem.why == tasks::Refusal::HoldTaken && problem.blocked_by.as_deref() == Some(&[first])
    ));
    let run = running(&world, first);
    finish(&mut world, run, b"first done");
    finish(&mut world, parent, b"exclusive work done");
}

#[test]
fn a_batch_waits_without_creating_a_task_or_reserving_funding_until_its_resource_is_reported() {
    let mut world = plan_world(152);
    let parent = world.assignments[0];
    let resource = jig_test_connector_world::path(1, 1);
    let before = match world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(parent.0)))) {
        Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) => row.numbers,
        row => panic!("parent row: {row:?}"),
    };
    let member = resource_member(&world, 1, b"wait for its report");
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Make {
        reply_to: ReplyTo::new(Token::new(u64::MAX - 3)),
        creator: tasks::Party::Task(parent.0),
        batch: Box::new([tasks::New {
            number: 100,
            project: 1,
            executor: member.executor,
            spec: member.spec,
            contract: member.contract,
            numbers: tasks::Numbers { budget: 10, spent: 0, spent_below: 0, reserved: 0 },
            authority: member.authority,
            funder: tasks::Funder::Task(parent.0),
            dependencies: Box::new([]),
            holdings: Box::new([tasks::Holding::Write {
                resource: tasks::Name { connector: 1, path: resource.segments().into() },
                kind: 1,
            }]),
            wake: tasks::WakePolicy::DEFAULT,
            recurring: None,
            tracked: None,
        }]),
    })));
    let key = root::Key::Core(core::Key::Tasks(tasks::Key::Live(100)));
    assert!(!world.store.rows.contains_key(&key));
    assert_eq!(world.assignments.len(), 1);
    let row = world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(parent.0)))).expect("parent row");
    assert!(matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) if row.numbers == before));
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::Adopt { project: 1, resource: resource.clone(), role: connector::ResourceRole::Owned },
    });
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::Names { task: 100, project: 1, resources: Box::new([resource]) },
    });
    assert!(world.store.rows.contains_key(&key), "a connector report resumes the whole batch");
    assert_eq!(world.assignments.len(), 2);
    let run = running(&world, 100);
    finish(&mut world, run, b"reported work done");
    finish(&mut world, parent, b"done");
}

#[test]
fn re_adopting_a_resource_as_context_refuses_the_existing_tasks_next_effect() {
    let mut world = World::new(153, false);
    let resource = jig_test_connector_world::path(1, 1);
    let task = world.call_key(1).task;
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::Adopt { project: 1, resource: resource.clone(), role: connector::ResourceRole::Owned },
    });
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::Names { task, project: 1, resources: Box::new([resource.clone()]) },
    });
    world.send(root::Event::Core(core::Event::EffectConnector(core::connector::Event::Resource {
        project: 2,
        name: tasks::Name { connector: 1, path: resource.segments().into() },
        role: core::connector::ResourceRole::Context,
        hold: core::connector::HoldKind::Exclusive { wait: true },
    })));
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::Adopt {
            project: 1,
            resource: resource.clone(),
            role: connector::ResourceRole::Participating,
        },
    });
    world.call(1, 8153);
    assert_eq!(world.systems[0].observed().iter().filter(|entry| entry.applied).count(), 1);
    let spent = world.spent();
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::Adopt { project: 1, resource, role: connector::ResourceRole::Context },
    });
    let mut context_effect = World::effect();
    context_effect.purpose = 21;
    world.send(root::Event::EffectCall {
        to: ReplyTo::new(Token::new(8155)),
        key: world.call_key(2),
        number: 1,
        effect: context_effect,
        deadline: skein_lib::Wall::from_nanos(1_000_000_000),
        proposal: None,
    });
    assert!(matches!(world.answers.last(), Some((_, _, core::CallPart::EffectDenied {
        answer: authority::Answer::Refuse, findings,
    })) if findings.contains(&authority::Finding::ResourceAccess)));
    assert_eq!(world.systems[0].observed().iter().filter(|entry| entry.applied).count(), 1);
    assert_eq!(world.spent(), spent);
}

#[test]
fn a_resource_becoming_unavailable_holds_its_waiter_and_refuses_a_new_hold() {
    let mut world = plan_world(154);
    configure_resources(&mut world);
    let parent = world.assignments[0];
    let spec = resource_member(&world, 1, b"hold the resource");
    let first = delegate(&mut world, parent, 1, spec);
    let spec = resource_member(&world, 1, b"wait for the resource");
    let waiting = delegate(&mut world, parent, 2, spec);
    assert_eq!(world.assignments.len(), 2);
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::System(connector::SystemEvent::Unavailable {
            project: 1,
            resource: jig_test_connector_world::path(1, 1),
        }),
    });
    for task in [first, waiting] {
        assert!(matches!(world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))),
            Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))))
                if matches!(row.phase, tasks::Phase::Held { why: tasks::Hold::ResourceUnavailable, .. })));
    }
    assert_eq!(world.assignments.len(), 2, "an unavailable waiter is not assigned");
    let spec = resource_member(&world, 1, b"new hold on unavailable resource");
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(9155)),
        key: core::CallKey { task: parent.0, attempt: parent.1, completion: 3, position: 0 },
        batch: Box::new([spec]),
        stubs: Box::new([]),
    }));
    assert!(matches!(world.answers.last().expect("unavailable batch answer").2,
        core::CallPart::DelegationRefused(ref problem) if problem.why == tasks::Refusal::ResourceUnavailable));
}

#[test]
fn re_adopting_a_running_workspace_as_context_stops_its_writer() {
    let mut world = plan_world(155);
    configure_resources(&mut world);
    let parent = world.assignments[0];
    let spec = resource_member(&world, 1, b"write this resource");
    let child = delegate(&mut world, parent, 1, spec);
    assert_eq!(world.assignments.len(), 2);
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::Adopt {
            project: 1,
            resource: jig_test_connector_world::path(1, 1),
            role: connector::ResourceRole::Context,
        },
    });
    assert!(matches!(world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(child)))),
        Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))))
            if matches!(row.phase, tasks::Phase::Held { why: tasks::Hold::ResourceAccess, .. })));
    assert_eq!(world.assignments.len(), 2);
}

#[test]
fn a_context_adoption_still_refuses_effects_after_a_cold_restart() {
    let mut world = World::new(156, false);
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::Adopt {
            project: 1,
            resource: jig_test_connector_world::path(1, 1),
            role: connector::ResourceRole::Context,
        },
    });
    world.restart_with(fixture(156, false).0);
    world.call(1, 8156);
    assert!(matches!(world.answers.last(), Some((_, _, core::CallPart::EffectDenied {
        answer: authority::Answer::Refuse, findings,
    })) if findings.contains(&authority::Finding::ResourceAccess)));
    assert!(world.systems[0].observed().is_empty());
}

fn live_row(world: &World, task: u64) -> &tasks::TaskRecord {
    match world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))) {
        Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) => row,
        row => panic!("live task row missing: {row:?}"),
    }
}

fn amendment(words: &[u8]) -> people::Amendment {
    people::Amendment {
        addressed: None,
        instructions: None,
        spec: Some(people::Spec { words: words.into(), parameters: Box::new([]), inputs: Box::new([]) }),
        wake: None,
        dependencies: None,
        authority: None,
        reason: b"new evidence".as_slice().into(),
    }
}

#[test]
fn a_release_naming_an_old_hold_is_refused_as_moved_saying_what_holds_it_now() {
    let mut world = World::new(150, false);
    let task = world.call_key(1).task;
    ask(&mut world, 50, people::Ask::Stop { project: 1, task });
    let revision = live_row(&world, task).revisions.hold;
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Hold { task, why: tasks::Hold::Budget })));
    ask(&mut world, 51, people::Ask::Release { project: 1, task, revision, again: None });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Moved {
            now: people::Moved::Hold { revision: revision + 1, reason: people::HoldReason::Budget }
        })))
    );
    assert!(matches!(live_row(&world, task).phase, tasks::Phase::Held { why: tasks::Hold::Budget, .. }));
    let refusal = world.people_answers.last().copied();
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Hold { task, why: tasks::Hold::Deadline })));
    ask(&mut world, 51, people::Ask::Release { project: 1, task, revision, again: None });
    assert_eq!(world.people_answers.last().copied(), refusal, "replay retains the refused decision's facts");
}

#[test]
fn a_release_after_a_pass_up_keeps_the_hold_revision_and_uses_the_partys_tree() {
    let mut world = World::new(151, false);
    let task = world.call_key(1).task;
    ask(&mut world, 52, people::Ask::Stop { project: 1, task });
    world.end(tasks::End::Parked, 0);
    let revision = live_row(&world, task).revisions.hold;
    ask(
        &mut world,
        53,
        people::Ask::DecideEscalation { project: 1, task, revision, decision: people::EscalationDecision::Pass },
    );
    assert!(matches!(
        world.people_answers.last(),
        Some(people::Reply::Outcome(people::Outcome::EscalationDecided {
            choice: people::EscalationChoice::Passed,
            ..
        }))
    ));
    assert_eq!(live_row(&world, task).revisions.hold, revision);
    ask(&mut world, 54, people::Ask::Release { project: 1, task, revision, again: None });
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::Released { task })));
    assert!(!matches!(live_row(&world, task).phase, tasks::Phase::Held { .. }));
}

#[test]
fn an_amendment_and_a_release_are_one_decision() {
    let (configuration, limits) = amendment_world_fixture(152);
    let mut world = World::configured(152, false, configuration, limits);
    let task = world.call_key(1).task;
    ask(&mut world, 54, people::Ask::Stop { project: 1, task });
    world.end(tasks::End::Parked, 0);
    let hold = live_row(&world, task).revisions.hold;
    let revision = live_row(&world, task).revisions.spec;
    let answers = world.people_answers.len();
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    ask(
        &mut world,
        55,
        people::Ask::Amend { project: 1, task, revision, release: Some(hold), amendment: amendment(b"first change") },
    );
    assert_eq!(world.people_answers.len(), answers, "the combined outcome waits for durability");
    world.release_commits();
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Amended { task, released: true }))
    );
    assert!(!matches!(live_row(&world, task).phase, tasks::Phase::Held { .. }));
    ask(
        &mut world,
        56,
        people::Ask::Amend { project: 1, task, revision, release: None, amendment: amendment(b"stale change") },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Moved {
            now: people::Moved::Spec { revision: revision + 1 }
        })))
    );
    assert_eq!(live_row(&world, task).spec.words.as_ref(), b"first change");
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Hold { task, why: tasks::Hold::Budget })));
    let now = live_row(&world, task).revisions.hold;
    ask(
        &mut world,
        79,
        people::Ask::Amend {
            project: 1,
            task,
            revision: revision + 1,
            release: Some(hold),
            amendment: amendment(b"must not land"),
        },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Moved {
            now: people::Moved::Hold { revision: now, reason: people::HoldReason::Budget }
        })))
    );
    assert_eq!(live_row(&world, task).spec.words.as_ref(), b"first change");
    world.end(tasks::End::Parked, 0);
    a_widening_amendment_with_release_is_proposed_and_released_on_acceptance(&mut world, 152);
}

#[test]
fn a_party_short_of_budget_hears_what_the_request_needs_and_what_is_left() {
    let mut world = World::new(153, false);
    ask(&mut world, 57, people::Ask::StartChat { project: 1, words: b"next".as_slice().into() });
    let expected = people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Authority {
        proposable: None,
        lacking: people::Lack::Budget { needs: 300, left: 200 },
    }));
    assert_eq!(world.people_answers.last(), Some(&expected));
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        1,
    );
    ask(&mut world, 57, people::Ask::StartChat { project: 1, words: b"next".as_slice().into() });
    assert_eq!(world.people_answers.last(), Some(&expected), "the freed budget does not change a saved refusal");
}

#[test]
fn a_party_deciding_a_cancelled_task_hears_who_cancelled_it_and_when() {
    let mut world = World::new(154, false);
    let task = world.call_key(1).task;
    let person = match live_row(&world, task).requester {
        tasks::Party::Person(person) => person,
        tasks::Party::Task(_) | tasks::Party::Deployment { .. } => panic!("person's chat"),
    };
    ask(&mut world, 58, people::Ask::Cancel { project: 1, task, reason: b"finished".as_slice().into() });
    let tasks::Phase::Closing(tasks::Closing { ending: tasks::Ending::Cancelled { at, .. }, .. }) =
        &live_row(&world, task).phase
    else {
        panic!("durable cancellation is closing")
    };
    let at = *at;
    assert!(at > skein_lib::Wall::EPOCH, "cancellation uses the advanced world clock");
    ask(&mut world, 59, people::Ask::Release { project: 1, task, revision: 0, again: None });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Ended {
            how: people::Ending::Cancelled { by: people::Party::Person(person), at },
        })))
    );
}

fn person_world_fixture(seed: u64) -> (root::Config, root::Limits) {
    let (mut configuration, mut limits) = jig_core_world::faults::plan_fixture(seed);
    limits.core.tasks.person_stall = skein_lib::Duration::from_secs(1);
    let kinds: Box<[authority::Executor]> = Box::new([authority::Executor::Charter(1), authority::Executor::Role(0)]);
    configuration.core.settings.chat_authority.delegation.kinds.clone_from(&kinds);
    let mut rules = configuration.core.authority.rules().clone();
    rules.ceiling.delegation.kinds.clone_from(&kinds);
    let mut policy = configuration.core.authority.policy(1).expect("plan policy").clone();
    policy.ceiling.delegation.kinds.clone_from(&kinds);
    policy.roles[0].authority.delegation.kinds = kinds;
    let mut domain = authority::Domain::new(rules, limits.core.authority).expect("bounded person-task policy");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    configuration.core.authority = domain;
    (configuration, limits)
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one person-task story checks revision, handoff and durable first-answer facts"
)]
fn a_person_answering_a_person_task_whose_inputs_changed_is_refused_as_moved() {
    let (configuration, limits) = person_world_fixture(160);
    let mut world = World::configured(160, false, configuration, limits);
    let parent = world.assignments[0];
    let mut input = member(&world, tasks::Executor::Agent { charter: 1 }, b"input report");
    input.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let input = delegate(&mut world, parent, 1, input);
    let input_run = running(&world, input);
    let mut choice = member(&world, tasks::Executor::Person(tasks::PersonAddress::Role(0)), b"answer with the input");
    choice.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    choice.dependencies = Box::new([core::Dependency::Existing(input)]);
    let choice = delegate(&mut world, parent, 2, choice);
    let revision = live_row(&world, choice).revisions.person;
    assert_eq!(live_row(&world, choice).person_wait, None);
    finish(&mut world, input_run, b"input arrived");
    let waiting = live_row(&world, choice).person_wait.expect("dependency completion starts the person wait");
    assert_eq!(waiting.stall, limits.core.tasks.person_stall);
    world.advance(skein_lib::Duration::from_secs(2));
    let notice = live_row(&world, choice).person_wait.expect("retained wait").notice.expect("root numbers stall news");
    let news =
        live_row(&world, parent.0).inbox.iter().find(|word| word.number == notice.message).expect("committed news");
    assert_eq!(news.kind, tasks::MessageKind::PersonStalled { task: choice });
    assert_eq!(news.from, tasks::Party::Task(choice));
    assert_eq!(news.at, notice.at);
    let now = live_row(&world, choice).revisions.person;
    assert_eq!(now, revision + 1);
    ask(
        &mut world,
        62,
        people::Ask::AnswerPerson {
            project: 1,
            task: choice,
            revision,
            result: people::PersonResult::Report { words: b"old observation".as_slice().into() },
        },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Moved {
            now: people::Moved::Person { revision: now },
        })))
    );
    assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(choice)))));
    let first_sign_in = world.sign_in.expect("first party signed in");
    let first_person = match live_row(&world, parent.0).requester {
        tasks::Party::Person(person) => person,
        tasks::Party::Task(_) | tasks::Party::Deployment { .. } => panic!("person's chat"),
    };
    world.send(root::Event::Core(core::Event::SignIn {
        reply_to: ReplyTo::new(Token::new(6100)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: 8_u64.to_be_bytes().into() },
            login: b"second".as_slice().into(),
            name: b"second holder".as_slice().into(),
        },
    }));
    let second_sign_in = world.sign_in.expect("second party signed in");
    let Some(people::Reply::SignedIn { person: second_person, .. }) = world.people_answers.last().copied() else {
        panic!("second sign-in")
    };
    world.sign_in = Some(first_sign_in);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 1,
        holdings: Box::new([
            people::Holding { person: first_person, role: people::Role::Owner },
            people::Holding { person: second_person, role: people::Role::Owner },
        ]),
    })));
    ask(&mut world, 63, people::Ask::TakePerson { project: 1, task: choice });
    ask_as(&mut world, second_sign_in, 65, people::Ask::TakePerson { project: 1, task: choice });
    let taken = people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Taken { by: first_person }));
    assert_eq!(world.people_answers.last(), Some(&taken));
    let answer = people::Ask::AnswerPerson {
        project: 1,
        task: choice,
        revision: now,
        result: people::PersonResult::Report { words: b"current observation".as_slice().into() },
    };
    ask_as(&mut world, second_sign_in, 66, answer.clone());
    assert_eq!(world.people_answers.last(), Some(&taken));
    assert_eq!(live_row(&world, choice).taken_by, Some(first_person));
    ask(&mut world, 67, people::Ask::HandBackPerson { project: 1, task: choice });
    assert_eq!(live_row(&world, choice).taken_by, None);
    let waiting = live_row(&world, choice).person_wait;
    world.restart_with(person_world_fixture(160).0);
    assert_eq!(live_row(&world, choice).person_wait, waiting, "handoff and restart preserve the same wait and notice");
    ask_as(&mut world, second_sign_in, 68, people::Ask::TakePerson { project: 1, task: choice });
    ask(&mut world, 69, answer.clone());
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Taken { by: second_person })))
    );
    ask_as(&mut world, second_sign_in, 70, answer.clone());
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::PersonAnswered { task: choice }))
    );
    assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(choice)))));
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(choice)))]
    else {
        panic!("ended person task")
    };
    let winner = row.person_answer.expect("retained first answer");
    assert_eq!(winner.by, second_person);
    let ended = people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Ended {
        how: people::Ending::Decided { by: winner.by, at: winner.at },
    }));
    ask(&mut world, 71, answer.clone());
    assert_eq!(world.people_answers.last(), Some(&ended), "the ended owner row identifies who answered");
    world.restart_with(person_world_fixture(160).0);
    ask(&mut world, 69, answer.clone());
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Taken { by: second_person })))
    );
    ask(&mut world, 72, answer);
    assert_eq!(world.people_answers.last(), Some(&ended), "first answer survives cold restoration");
}

#[path = "stories/watches.rs"]
mod watches;

fn amendment_world_fixture(seed: u64) -> (root::Config, root::Limits) {
    let (mut configuration, limits) = fixture(seed, false);
    let rules = configuration.core.authority.rules().clone();
    let mut policy = configuration.core.authority.policy(1).expect("amendment policy").clone();
    policy.roles[0].period_spend = 1000;
    configuration.core.settings.person_budget = 1000;
    let mut authority_domain = authority::Domain::new(rules, limits.core.authority).expect("bounded amendment policy");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut authority_domain, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    configuration.core.authority = authority_domain;
    (configuration, limits)
}

fn a_widening_amendment_with_release_is_proposed_and_released_on_acceptance(world: &mut World, seed: u64) {
    let task = world.call_key(1).task;
    let hold = live_row(world, task).revisions.hold;
    let revision = live_row(world, task).revisions.spec;

    let mut change = amendment(b"widened task");
    let mut widened_authority = people::Authority {
        tools: 1023,
        grants: Box::new([people::Grant {
            connector: 1,
            kind: 5,
            pattern: people::Pattern { segments: Box::new([]), last: people::Last::Open(Box::new([])) },
        }]),
        delegation: people::Delegation {
            kinds: Box::new([people::Executor::Charter(1), people::Executor::Procedure(1)]),
            tasks: 2,
            depth: 2,
        },
        spend: 600,
        deadline: None,
        notes: 0,
        note_resources: Box::new([]),
    };
    change.authority = Some(widened_authority.clone());

    ask(world, 81, people::Ask::Amend { project: 1, task, revision, amendment: change, release: Some(hold) });
    let Some(people::Reply::Outcome(people::Outcome::AmendProposed { proposal, released: false, .. })) =
        world.people_answers.last().copied()
    else {
        panic!("widening proposed: {:?}", world.people_answers)
    };
    assert_eq!(live_row(world, task).revisions.spec, revision);
    assert!(matches!(live_row(world, task).phase, tasks::Phase::Held { .. }));
    let (restored_configuration, _) = amendment_world_fixture(seed);
    world.restart_with(restored_configuration);
    assert!(
        matches!(&live_row(world, task).proposal.as_ref().expect("retained amendment proposal").action, tasks::ProposalAction::Amend { amendment, .. } if amendment.release == Some(hold))
    );

    widened_authority.spend = 1000;
    ask(
        world,
        83,
        people::Ask::ChangePolicy {
            project: 1,
            change: people::PolicyChange::Role(people::PolicyRole {
                number: 0,
                authority: widened_authority,
                period_spend: 1000,
                requests: authority::Requests::ALL.0,
                decides: authority::Proposals::ALL.0,
            }),
        },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::PolicyChanged { project: 1 }))
    );
    let answers = world.people_answers.len();
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    ask(
        world,
        82,
        people::Ask::DecideProposal {
            project: 1,
            proposer: task,
            proposal,
            decision: people::ProposalDecision::Accept,
        },
    );
    assert_eq!(world.people_answers.len(), answers);
    world.release_commits();
    assert!(
        matches!(
            world.people_answers.last(),
            Some(people::Reply::Outcome(people::Outcome::ProposalDecided {
                choice: people::ProposalChoice::Accepted,
                ..
            }))
        ),
        "acceptance: {:?}",
        world.people_answers
    );
    assert_eq!(live_row(world, task).spec.words.as_ref(), b"widened task");
    assert_eq!(live_row(world, task).numbers.budget, 600);
    assert!(!matches!(live_row(world, task).phase, tasks::Phase::Held { .. }));
}

#[test]
fn a_goal_with_a_deadline_is_held_past_it() {
    let (configuration, limits) = amendment_world_fixture(161);
    let mut world = World::configured(161, false, configuration, limits);
    let deadline = skein_lib::Wall::from_nanos(1_000_000_000);
    ask(
        &mut world,
        90,
        people::Ask::SetGoal {
            project: 1,
            spec: b"finish before the deadline".as_slice().into(),
            executor: jig_core_people::GoalExecutor::Charter(1),
            budget: 600,
            priority: 0,
            deadline: Some(deadline),
        },
    );
    let Some(people::Reply::Outcome(people::Outcome::GoalProposed { proposal })) = world.people_answers.last().copied()
    else {
        panic!("goal proposed: {:?}", world.people_answers)
    };
    let proposal_key = root::Key::Core(core::Key::Tasks(tasks::Key::PersonProposal(proposal)));
    let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(row)))) =
        world.store.rows.get(&proposal_key)
    else {
        panic!("durable goal proposal")
    };
    let goal = row.goal.number;
    let proposer = row.proposer;
    assert_eq!(row.goal.authority.budget.deadline, Some(deadline));
    world.restart_with(amendment_world_fixture(161).0);
    let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(row)))) =
        world.store.rows.get(&proposal_key)
    else {
        panic!("restored goal proposal")
    };
    assert_eq!(row.goal.authority.budget.deadline, Some(deadline));
    ask(
        &mut world,
        91,
        people::Ask::ChangePolicy {
            project: 1,
            change: people::PolicyChange::Role(people::PolicyRole {
                number: 0,
                authority: people::Authority {
                    tools: 1023,
                    grants: Box::new([people::Grant {
                        connector: 1,
                        kind: 5,
                        pattern: people::Pattern { segments: Box::new([]), last: people::Last::Open(Box::new([])) },
                    }]),
                    delegation: people::Delegation {
                        kinds: Box::new([people::Executor::Charter(1), people::Executor::Procedure(1)]),
                        tasks: 2,
                        depth: 2,
                    },
                    spend: 1000,
                    deadline: None,
                    notes: 0,
                    note_resources: Box::new([]),
                },
                period_spend: 1000,
                requests: authority::Requests::ALL.0,
                decides: authority::Proposals::ALL.0,
            }),
        },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::PolicyChanged { project: 1 }))
    );
    ask(
        &mut world,
        92,
        people::Ask::DecideProposal { project: 1, proposer, proposal, decision: people::ProposalDecision::Accept },
    );
    assert!(
        matches!(
            world.people_answers.last(),
            Some(people::Reply::Outcome(people::Outcome::ProposalDecided {
                choice: people::ProposalChoice::Accepted,
                ..
            }))
        ),
        "acceptance: {:?}",
        world.people_answers
    );
    assert_eq!(live_row(&world, goal).authority.budget.deadline, Some(deadline));
    world.advance(skein_lib::Duration::from_secs(2));
    world.send(root::Event::Core(core::Event::Account(jig_core_accounts::Event::Add {
        account: 1,
        generation: 1,
        valid: Some(skein_lib::Duration::from_secs(60)),
    })));
    let chat = world.assignments[0];
    finish(&mut world, chat, b"chat finished");
    assert!(
        matches!(live_row(&world, goal).phase, tasks::Phase::Held { why: tasks::Hold::Deadline, .. }),
        "deadline hold: {:?}",
        live_row(&world, goal).phase
    );
}

#[path = "stories/offers.rs"]
mod offers;

fn approval_world(seed: u64) -> (World, u64, u64, u64) {
    let (configuration, limits) = person_world_fixture(seed);
    let mut world = World::configured(seed, false, configuration, limits);
    let parent = world.assignments[0];
    let mut approval = member(&world, tasks::Executor::Person(tasks::PersonAddress::Role(0)), b"approve shown state");
    approval.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    approval.contract = tasks::Contract::Approval {
        state: Box::from(&b"state\x00\xff"[..]),
        part: tasks::Part { connector: 1, kind: tasks::PartKind::Resource, key: tasks::PartKey::Number(45) },
    };
    let task = delegate(&mut world, parent, 1, approval);
    ask(&mut world, 81, people::Ask::TakePerson { project: 1, task });
    let revision = live_row(&world, task).revisions.person;
    (world, parent.0, task, revision)
}

#[test]
fn a_person_approves_the_exact_state_and_its_delegate_result_waits_for_commit() {
    let (mut world, parent, task, revision) = approval_world(451);
    world.people_answers.clear();
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    ask(
        &mut world,
        82,
        people::Ask::AnswerPerson {
            project: 1,
            task,
            revision,
            result: people::PersonResult::Approved {
                state: Box::from(&b"state\x00\xff"[..]),
                words: Some(Box::from(&b"looks correct"[..])),
            },
        },
    );
    assert!(world.people_answers.is_empty(), "the reply waits for the approval transaction");
    assert!(
        !world
            .inbound
            .iter()
            .any(|message| matches!(message.kind, tasks::MessageKind::Result(tasks::ResultKind::Approved { .. })))
    );
    let number = world.store.release_held().expect("withheld approval completion");
    world.send(root::Event::Committed { number });
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::PersonAnswered { task })));
    let message = live_row(&world, parent)
        .inbox
        .iter()
        .find(|message| message.from == tasks::Party::Task(task))
        .expect("delegate result delivered");
    assert_eq!(
        message.kind,
        tasks::MessageKind::Result(tasks::ResultKind::Approved {
            state: Box::from(&b"state\x00\xff"[..]),
            words_present: true
        })
    );
    assert_eq!(message.words.as_ref(), b"looks correct");
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))]
    else {
        panic!("ended approval owner");
    };
    assert_eq!(
        row.phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Approved {
            state: Box::from(&b"state\x00\xff"[..]),
            words: Some(Box::from(&b"looks correct"[..]))
        }))
    );
    world.restart_with(person_world_fixture(451).0);
    let message = live_row(&world, parent)
        .inbox
        .iter()
        .find(|message| message.from == tasks::Party::Task(task))
        .expect("result survives cold restore");
    assert_eq!(
        message.kind,
        tasks::MessageKind::Result(tasks::ResultKind::Approved {
            state: Box::from(&b"state\x00\xff"[..]),
            words_present: true
        })
    );
    assert_eq!(message.words.as_ref(), b"looks correct");
}

#[test]
fn a_person_approving_a_different_state_or_wrong_revision_is_refused_as_moved() {
    let (mut world, _, task, revision) = approval_world(452);
    for (request, named, state) in
        [(82, revision, b"other state".as_slice()), (83, revision + 1, b"state\x00\xff".as_slice())]
    {
        ask(
            &mut world,
            request,
            people::Ask::AnswerPerson {
                project: 1,
                task,
                revision: named,
                result: people::PersonResult::Approved { state: state.into(), words: None },
            },
        );
        assert_eq!(
            world.people_answers.last(),
            Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Moved {
                now: people::Moved::Person { revision }
            })))
        );
        assert_eq!(live_row(&world, task).person_answer, None);
        assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))));
    }
    ask(
        &mut world,
        84,
        people::Ask::AnswerPerson {
            project: 1,
            task,
            revision,
            result: people::PersonResult::Approved { state: Box::from(&b"state\x00\xff"[..]), words: None },
        },
    );
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::PersonAnswered { task })));
}

#[test]
fn a_person_declines_an_approval_with_the_existing_cannot_do_result() {
    let (mut world, parent, task, revision) = approval_world(453);
    ask(
        &mut world,
        82,
        people::Ask::AnswerPerson {
            project: 1,
            task,
            revision,
            result: people::PersonResult::Failure { reason: Box::from(&b"the shown state is unsuitable"[..]) },
        },
    );
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::PersonAnswered { task })));
    let message = live_row(&world, parent)
        .inbox
        .iter()
        .find(|message| message.from == tasks::Party::Task(task))
        .expect("declined result delivered");
    assert_eq!(message.kind, tasks::MessageKind::Result(tasks::ResultKind::Failed));
    assert_eq!(message.words.as_ref(), b"the shown state is unsuitable");
}

#[test]
fn approval_delegate_results_preserve_absent_and_explicitly_empty_explanations() {
    for (seed, words, present) in [(454, None, false), (455, Some(Box::<[u8]>::from([])), true)] {
        let (mut world, parent, task, revision) = approval_world(seed);
        ask(
            &mut world,
            82,
            people::Ask::AnswerPerson {
                project: 1,
                task,
                revision,
                result: people::PersonResult::Approved { state: Box::from(&b"state\x00\xff"[..]), words },
            },
        );
        let message = live_row(&world, parent)
            .inbox
            .iter()
            .find(|message| message.from == tasks::Party::Task(task))
            .expect("approval result delivered");
        assert_eq!(
            message.kind,
            tasks::MessageKind::Result(tasks::ResultKind::Approved {
                state: Box::from(&b"state\x00\xff"[..]),
                words_present: present
            })
        );
        assert!(message.words.is_empty());
        assert!(
            world.inbound.iter().any(|delivered| delivered == message),
            "typed result reaches the independent host"
        );
    }
}

#[test]
fn an_approval_reaches_its_asking_procedure_connector_as_the_whole_durable_input() {
    let (mut world, parent) = procedure_world(456, 1);
    let (step, connector, code, context) = world.procedure_inputs.last().expect("first procedure input");
    assert_eq!(context.task, parent);
    let mut child = member(&world, tasks::Executor::Person(tasks::PersonAddress::Role(0)), b"approve this state");
    child.authority.budget.spend = 5;
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    child.contract = tasks::Contract::Approval {
        state: Box::from(&b"procedure\x00\xff"[..]),
        part: tasks::Part { connector: 1, kind: tasks::PartKind::Resource, key: tasks::PartKey::Number(45) },
    };
    world.send(root::Event::Core(core::Event::ProcedureStep {
        task: parent,
        step: *step,
        connector: *connector,
        code: u32::from(*code),
        action: core::ProcedureAction::Delegate(Box::new([child])),
    }));
    let task = *live_row(&world, parent).delegates.first().expect("person delegate");
    ask(&mut world, 81, people::Ask::TakePerson { project: 1, task });
    let revision = live_row(&world, task).revisions.person;
    let before = world.procedure_inputs.len();
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    ask(
        &mut world,
        82,
        people::Ask::AnswerPerson {
            project: 1,
            task,
            revision,
            result: people::PersonResult::Approved {
                state: Box::from(&b"procedure\x00\xff"[..]),
                words: Some(Box::new([])),
            },
        },
    );
    assert_eq!(world.procedure_inputs.len(), before, "connector cannot receive an unacknowledged commit");
    let number = world.store.release_held().expect("approval transaction held");
    world.send(root::Event::Committed { number });
    let (_, connector, _, context) = world.procedure_inputs.last().expect("procedure receives approval");
    assert_eq!(*connector, 1);
    assert_eq!(context.task, parent);
    let result =
        context.inbox.iter().find(|message| message.from == tasks::Party::Task(task)).expect("whole result input");
    assert_eq!(
        result.kind,
        tasks::MessageKind::Result(tasks::ResultKind::Approved {
            state: Box::from(&b"procedure\x00\xff"[..]),
            words_present: true
        })
    );
    assert!(result.words.is_empty());
    assert_eq!(world.procedure_inputs.len(), before + 1);
}

#[test]
fn approval_state_and_explanation_share_the_person_answer_byte_bound() {
    let (mut world, parent, task, revision) = approval_world(457);
    let bound = usize::try_from(person_world_fixture(457).1.core.people.words).expect("small bound");
    let state = b"state\x00\xff";
    ask(
        &mut world,
        82,
        people::Ask::AnswerPerson {
            project: 1,
            task,
            revision,
            result: people::PersonResult::Approved {
                state: state.as_slice().into(),
                words: Some(vec![7; bound - state.len() + 1].into_boxed_slice()),
            },
        },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Refused(people::Refusal::Limit { which: people::LimitKind::Words }))
    );
    assert_eq!(live_row(&world, task).person_answer, None);
    assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))));
    ask(
        &mut world,
        83,
        people::Ask::AnswerPerson {
            project: 1,
            task,
            revision,
            result: people::PersonResult::Approved {
                state: state.as_slice().into(),
                words: Some(vec![7; bound - state.len()].into_boxed_slice()),
            },
        },
    );
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::PersonAnswered { task })));
    let message = live_row(&world, parent)
        .inbox
        .iter()
        .find(|message| message.from == tasks::Party::Task(task))
        .expect("bounded result");
    assert_eq!(message.payload_bytes(), Some(bound));
}

#[test]
fn a_verdicts_machine_fields_reach_its_requester_whole_after_commit_and_restart() {
    let mut world = plan_world(461);
    let parent = world.assignments[0];
    let mut triage = member(&world, tasks::Executor::Agent { charter: 1 }, b"return named machine fields");
    triage.contract = tasks::Contract::Verdict {
        choices: Box::new([tasks::Verdict {
            code: 2,
            words: 4,
            followups: 0,
            fields: Box::new([tasks::FieldRule { name: b"state".as_slice().into(), max: 8 }]),
        }]),
    };
    let child = delegate(&mut world, parent, 1, triage);
    let run = running(&world, child);
    let fields: Box<[tasks::ResultField]> =
        Box::new([tasks::ResultField { name: b"state".as_slice().into(), value: b"\x00\xff".as_slice().into() }]);
    let before = world.inbound.len();
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    world.send(root::Event::Answer {
        channel: Token::new(7),
        task: run.0,
        attempt: run.1,
        cumulative: 0,
        end: tasks::End::Finished {
            result: tasks::TaskResult::Verdict { code: 2, words: b"why".as_slice().into(), fields: fields.clone() },
            cancel_delegates: false,
        },
    });
    assert_eq!(world.inbound.len(), before, "result waits for its commit completion");
    let number = world.store.release_held().expect("held verdict transaction");
    world.send(root::Event::Committed { number });
    let expected = tasks::MessageKind::Result(tasks::ResultKind::Verdict { code: 2, fields: fields.clone() });
    assert!(world.inbound.iter().any(|message| message.from == tasks::Party::Task(child)
        && message.kind == expected
        && message.words.as_ref() == b"why"));
    let message = live_row(&world, parent.0)
        .inbox
        .iter()
        .find(|message| message.from == tasks::Party::Task(child))
        .expect("durable delegate message");
    assert_eq!(message.kind, expected);
    assert_eq!(message.payload_bytes(), Some(10));
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(child)))]
    else {
        panic!("durable ended owner");
    };
    assert_eq!(
        row.phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Verdict {
            code: 2,
            words: b"why".as_slice().into(),
            fields
        }))
    );
    world.restart_with(jig_core_world::faults::plan_fixture(461).0);
    let message = live_row(&world, parent.0)
        .inbox
        .iter()
        .find(|message| message.from == tasks::Party::Task(child))
        .expect("restored delegate message");
    assert_eq!(message.kind, expected);
    assert_eq!(message.words.as_ref(), b"why");
}

fn procedure_world(seed: u64, number: u16) -> (World, u64) {
    let (configuration, limits) = procedure_configuration(seed);
    let mut world = World::configured(seed, false, configuration, limits);
    world.hold_procedures = true;
    let root_run = world.assignments[0];
    let mut procedure = member(&world, tasks::Executor::Procedure { connector: number, code: 1 }, b"ask for approval");
    procedure.authority.delegation.kinds = Box::new([tasks::AuthorityExecutor::Role(0)]);
    let parent = delegate(&mut world, root_run, 1, procedure);
    (world, parent)
}

#[test]
fn a_procedure_delegate_identity_is_committed_with_its_child_by_the_actual_connector() {
    let (mut world, parent) = procedure_world(458, 2);
    let (step, number, code, _) = world.procedure_inputs.last().expect("procedure input");
    let (step, number, code) = (*step, *number, *code);
    let mut child = member(&world, tasks::Executor::Person(tasks::PersonAddress::Role(0)), b"child");
    child.authority.budget.spend = 5;
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    world.send(root::Event::Core(core::Event::ProcedureStep {
        task: parent,
        step,
        connector: number,
        code: u32::from(code),
        action: core::ProcedureAction::Delegate(Box::new([child])),
    }));
    let child = *live_row(&world, parent).delegates.first().expect("committed child");
    let record = connector::Record::ProcedureDelegateOutcome { task: parent, step, child: Some(child) };
    assert_eq!(
        world.store.rows.get(&root::Key::Connector { number: 2, key: record.key() }),
        Some(&root::Record::Connector { number: 2, record: record.clone() })
    );
    assert!(!world.store.rows.contains_key(&root::Key::Connector { number: 1, key: record.key() }));
    assert!(world.commits.iter().any(|writes| {
        writes.iter().any(|write| matches!(write, root::Write::Save(root::Record::Connector { number: 2, record: saved }) if saved == &record))
            && writes.iter().any(|write| matches!(write, root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(saved)))) if saved.number == child))
    }), "child and connector identity share one durable decision");
    world.restart_with(procedure_configuration(458).0);
    assert_eq!(
        world.store.rows.get(&root::Key::Connector { number: 2, key: record.key() }),
        Some(&root::Record::Connector { number: 2, record })
    );
}

#[test]
fn a_refused_procedure_delegation_retains_the_exact_none_outcome() {
    let (mut world, parent) = procedure_world(459, 2);
    let (step, number, code, _) = world.procedure_inputs.last().expect("procedure input");
    let (step, number, code) = (*step, *number, *code);
    let mut child = member(&world, tasks::Executor::Person(tasks::PersonAddress::Role(0)), b"over budget");
    child.authority.budget.spend = u64::MAX;
    world.send(root::Event::Core(core::Event::ProcedureStep {
        task: parent,
        step,
        connector: number,
        code: u32::from(code),
        action: core::ProcedureAction::Delegate(Box::new([child])),
    }));
    assert!(live_row(&world, parent).delegates.is_empty());
    let record = connector::Record::ProcedureDelegateOutcome { task: parent, step, child: None };
    assert_eq!(
        world.store.rows.get(&root::Key::Connector { number: 2, key: record.key() }),
        Some(&root::Record::Connector { number: 2, record })
    );
}

fn procedure_configuration(seed: u64) -> (root::Config, root::Limits) {
    let (mut configuration, mut limits) = person_world_fixture(seed);
    limits.core.authority.executors = 3;
    limits.core.tasks.executor_kinds = 3;
    let kinds: Box<[authority::Executor]> =
        Box::new([authority::Executor::Charter(1), authority::Executor::Role(0), authority::Executor::Procedure(1)]);
    configuration.core.settings.chat_authority.delegation.kinds.clone_from(&kinds);
    let mut rules = configuration.core.authority.rules().clone();
    rules.ceiling.delegation.kinds.clone_from(&kinds);
    let mut policy = configuration.core.authority.policy(1).expect("project policy").clone();
    policy.ceiling.delegation.kinds.clone_from(&kinds);
    policy.roles[0].authority.delegation.kinds = kinds;
    let mut domain = authority::Domain::new(rules, limits.core.authority).expect("procedure policy");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    configuration.core.authority = domain;
    (configuration, limits)
}

#[test]
fn procedure_delegate_outcomes_without_a_live_procedure_owner_are_not_routed() {
    let (configuration, limits) = person_world_fixture(460);
    let mut world = World::configured(460, false, configuration, limits);
    let agent = world.assignments[0].0;
    for task in [agent, u64::MAX] {
        world.send(root::Event::Core(core::Event::ProcedureStep {
            task,
            step: u64::MAX,
            connector: 2,
            code: 1,
            action: core::ProcedureAction::Delegate(Box::new([])),
        }));
    }
    assert!(!world.store.rows.values().any(|row| matches!(
        row,
        root::Record::Connector { record: connector::Record::ProcedureDelegateOutcome { .. }, .. }
    )));
    assert!(!world.commits.iter().flatten().any(|write| matches!(
        write,
        root::Write::Save(root::Record::Connector { record: connector::Record::ProcedureDelegateOutcome { .. }, .. })
    )));
}

#[path = "stories/procedure_goals.rs"]
mod procedure_goals;
