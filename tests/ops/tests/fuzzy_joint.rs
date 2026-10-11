//! P reads and acts on the actual engine's public Ops client tree.
use jig_client::address::{Address as JigAddress, TaskSection};
use jig_client_view::Role;
use jig_client_world::Until;
use jig_fake_person::{Find, Step};
use jig_ops_client_domain::{Address, observability as obs};
use jig_ops_world::joint::fixture;
use skein_lib::Duration;
use skein_world::domain::{Verdict, heap::Counting};
#[global_allocator]
static HEAP: Counting = Counting;
const WITHIN: Duration = Duration::from_secs(3);
fn see(words: &[u8]) -> Step<Address> {
    Step::See { find: Find::text(words), within: WITHIN }
}
fn press(role: Role, words: &[u8]) -> Step<Address> {
    Step::Press { find: Find::named(role, words) }
}
#[test]
fn p_accepts_the_restart_from_the_incidents_page() {
    let (mut world, p) = monitor(511);
    fixture::alert(world.front());
    world.person(
        p,
        vec![
            Step::Go { address: Address::Observability(obs::Address::Incidents { project: 1, open: true }) },
            Step::See { find: Find::named(Role::Link, b"Incident 12"), within: WITHIN },
            press(Role::Link, b"Incident 12"),
            see(b"Lead"),
            Step::Go { address: Address::Jig(JigAddress::Task { task: 5, section: TaskSection::Activity }) },
            see(b"read_series"),
            see(b"delegate"),
            Step::Go { address: Address::Jig(JigAddress::Inbox { project: None }) },
            see(b"Restart service"),
            see(b"checkout"),
            see(b"production"),
            Step::Go { address: Address::Jig(JigAddress::Task { task: 5, section: TaskSection::Plan }) },
            see(b"restart checkout"),
            see(b"Restart service"),
            see(b"checkout"),
            see(b"production"),
            Step::See { find: Find::named(Role::Button, b"Accept"), within: WITHIN },
            press(Role::Button, b"Accept"),
            press(Role::Button, b"Confirm"),
            see(b"Accepted"),
            Step::Go { address: Address::Jig(JigAddress::Inbox { project: None }) },
            Step::Gone { find: Find::text(b"Restart service"), within: Duration::from_secs(8) },
            see(b"0 need you"),
            Step::Go { address: Address::Jig(JigAddress::Task { task: 6, section: TaskSection::Panels }) },
            see(b"Restart service"),
            see(b"Restarting"),
            see(b"Waiting for healthy"),
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{}", outcome.trace.join("\n"));
    let outcome = world.run(Until::Time(outcome.now.saturating_add(Duration::from_secs(8))));
    assert_eq!(outcome.verdict, Verdict::Passed, "{}", outcome.trace.join("\n"));
    fixture::refresh(world.front());
    world.person(
        p,
        vec![
            Step::Go { address: Address::Observability(obs::Address::Incident { project: 1, incident: 12 }) },
            see(b"Resolved"),
            see(b"checkout recovered after the accepted restart"),
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{}", outcome.trace.join("\n"));
    assert_durable_restart(&world.front().engine);
}

fn monitor(
    seed: u64,
) -> (jig_client_world::World<fixture::Client, jig_ops_world::joint::front::Joint>, jig_client_world::harness::ClientId)
{
    let mut world = fixture::world(seed);
    let p = world.client_with(
        1,
        Address::Jig(JigAddress::Conversations { project: None }),
        fixture::limits(),
        fixture::client,
    );
    fixture::watch_setup(world.front());
    world.person(
        p,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"First words"), within: WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"First words"), words: b"@watch_setup".as_slice().into() },
            press(Role::Button, b"Start a conversation"),
            see(b"delegate"),
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{}", outcome.trace.join("\n"));
    (world, p)
}

#[test]
fn a_restart_waiting_for_p_is_withdrawn_when_the_service_was_restarted_by_hand() {
    let (mut world, p) = monitor(512);
    fixture::alert(world.front());
    world.person(
        p,
        vec![
            Step::Go { address: Address::Jig(JigAddress::Inbox { project: None }) },
            see(b"Restart service"),
            see(b"checkout"),
            see(b"production"),
            Step::Go { address: Address::Jig(JigAddress::Task { task: 5, section: TaskSection::Plan }) },
            see(b"restart checkout"),
            see(b"Restart service"),
            see(b"checkout"),
            see(b"production"),
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{}", outcome.trace.join("\n"));
    let production = &mut world.front().engine.systems.production;
    assert_eq!(
        production.restart(&jig_ops_fake_production::ServiceName::new("production", "checkout"), 909),
        jig_ops_fake_production::ResultValue::Made
    );
    let outcome = world.run(Until::Time(outcome.now.saturating_add(Duration::from_secs(6))));
    assert_eq!(outcome.verdict, Verdict::Passed, "{}", outcome.trace.join("\n"));
    fixture::refresh(world.front());
    world.person(
        p,
        vec![
            see(b"withdrawn"),
            Step::Go { address: Address::Jig(JigAddress::Inbox { project: None }) },
            Step::Gone { find: Find::text(b"Restart service"), within: Duration::from_secs(8) },
            Step::Go { address: Address::Jig(JigAddress::Task { task: 6, section: TaskSection::Activity }) },
            see(b"healthy"),
            see(b"without restarting"),
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{}", outcome.trace.join("\n"));
    assert_durable_withdrawal(&world.front().engine);
    assert!(
        world
            .front()
            .engine
            .systems
            .production
            .observed()
            .iter()
            .all(|event| !matches!(event, jig_ops_fake_production::ObservedEffect::Restart { operation: 77, .. }))
    );
}

fn assert_durable_withdrawal(engine: &jig_conformance::Harness<jig_ops_world::joint::engine::OpsApplication>) {
    use jig_core_tasks as tasks;
    let history: Vec<_> = engine
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            jig_ops_domain::Record::Core(jig_core::Record::Tasks(tasks::Stored::History(history)))
                if history.task == 6 && history.change == tasks::Change::ProposalWithdrawn =>
            {
                Some(history)
            }
            jig_ops_domain::Record::Core(_)
            | jig_ops_domain::Record::Infrastructure(_)
            | jig_ops_domain::Record::Observability(_) => None,
        })
        .collect();
    assert_eq!(history.len(), 1);
    let proposal = history[0].proposal.as_ref().expect("whole supplied withdrawal history");
    assert_eq!(proposal.proposer, 6);
    assert_eq!(proposal.state, tasks::ProposalState::Withdrawn);
    assert!(engine.peers.conformance.entries.is_empty(), "no restart outbox was committed");
    assert!(engine.peers.conformance.accepted.is_empty(), "P supplied no acceptance");
    let result = engine
        .store
        .rows
        .values()
        .find_map(|row| match row {
            jig_ops_domain::Record::Core(jig_core::Record::Tasks(tasks::Stored::Ended(task))) if task.number == 6 => {
                Some(task)
            }
            jig_ops_domain::Record::Core(_)
            | jig_ops_domain::Record::Infrastructure(_)
            | jig_ops_domain::Record::Observability(_) => None,
        })
        .expect("R genuinely ended");
    assert_eq!(
        result.phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report {
            words: Box::from(b"Service is already healthy; finished without restarting it.".as_slice()),
        }))
    );
}

fn assert_durable_restart(engine: &jig_conformance::Harness<jig_ops_world::joint::engine::OpsApplication>) {
    let restart = jig_ops_fake_production::ObservedEffect::Restart {
        service: jig_ops_fake_production::ServiceName::new("production", "checkout"),
        operation: 77,
        applied: true,
    };
    assert_eq!(engine.systems.production.observed().iter().filter(|event| **event == restart).count(), 1);
    let accepted: Vec<_> =
        engine
            .store
            .rows
            .values()
            .filter_map(|row| match row {
                jig_ops_domain::Record::Core(jig_core::Record::Core(jig_core::CoreRecord::ProposalDecision(
                    decision,
                ))) if decision.proposer == jig_core_people::Proposer::Task(6) => Some(decision),
                jig_ops_domain::Record::Core(_)
                | jig_ops_domain::Record::Observability(_)
                | jig_ops_domain::Record::Infrastructure(_) => None,
            })
            .collect();
    assert_eq!(accepted.len(), 1);
    assert_eq!(accepted[0].by, 1);
    assert_eq!(accepted[0].choice, jig_core_people::ProposalChoice::Accepted);
    assert_eq!(accepted[0].kind, jig_core_tasks::ProposalKind::Effect);
    assert!(accepted[0].created.is_none(), "acceptance funds the original effect, not a new task");
    let native = &engine.peers.conformance.documents;
    assert!(native.iter().all(|(commit, _)| *commit <= engine.store.applied));
    for phase in [
        jig_ops_domain_infrastructure::ProcedurePhase::EffectOutstanding,
        jig_ops_domain_infrastructure::ProcedurePhase::WaitingCondition,
    ] {
        assert!(native.iter().any(|(_, document)| matches!(document,
            jig_ops_domain::ClientDocument::Infrastructure(value)
            if matches!(value.as_ref(), jig_ops_domain_infrastructure::ClientDocument::Panel { task: 6, state, .. } if state.phase == phase))));
    }
    assert!(native.iter().any(|(_, document)| matches!(document,
        jig_ops_domain::ClientDocument::Observability(value)
        if matches!(value.as_ref(), jig_ops_domain_observability::ClientDocument::Incident { incident, .. }
            if incident.number == 12 && incident.lead == Some(5) && incident.state == jig_ops_domain_observability::IncidentState::Resolved))));
}

#[test]
fn q_asks_for_an_environment_in_a_chat_and_accepts_what_it_costs() {
    let mut world = fixture::environment_world(513);
    let q = world.client_with(
        1,
        Address::Jig(JigAddress::Conversations { project: None }),
        fixture::limits(),
        fixture::client,
    );
    world.person(
        q,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"First words"), within: WITHIN },
            Step::Type {
                find: Find::named(Role::TextArea, b"First words"),
                words: b"@environment_chat staging for payments, a week".as_slice().into(),
            },
            press(Role::Button, b"Start a conversation"),
            see(b"propose"),
            Step::Go { address: Address::Jig(JigAddress::Inbox { project: None }) },
            see(b"staging for payments"),
            Step::See { find: Find::named(Role::Button, b"Accept"), within: WITHIN },
            press(Role::Button, b"Accept"),
            see(b"40"),
            see(b"Funded by pool in project 1, person 1, period 1"),
            press(Role::Button, b"Confirm"),
            see(b"Accepted"),
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{}", outcome.trace.join("\n"));
    let outcome = world.run(Until::Time(outcome.now.saturating_add(Duration::from_secs(8))));
    assert_eq!(outcome.verdict, Verdict::Passed, "{}", outcome.trace.join("\n"));
    let front = world.front();
    let native =
        jig_ops_domain_infrastructure::Environment::new(b"staging".as_slice().into(), b"payments".as_slice().into());
    let fact = front
        .engine
        .systems
        .infrastructure(jig_ops_domain_infrastructure::SystemRequest::Environment { environment: native });
    front.source(jig_ops_domain::Event::Infrastructure(jig_ops_domain_infrastructure::Event::System(fact)));
    world.person(
        q,
        vec![
            Step::Go { address: Address::Jig(JigAddress::Task { task: 1, section: TaskSection::Activity }) },
            see(b"staging/payments is ready"),
            Step::Go {
                address: Address::Infrastructure(jig_ops_client_domain::infrastructure::Address::NativeEnvironment {
                    project: 1,
                    environment: jig_ops_client_domain::infrastructure::native::Environment {
                        pool: b"staging".as_slice().into(),
                        name: b"payments".as_slice().into(),
                    },
                }),
            },
            see(b"Ready at (seconds since epoch):"),
            see(b"Revision 0"),
            Step::Type {
                find: Find::named(Role::TextField, b"New end (seconds since epoch)"),
                words: b"1209600".as_slice().into(),
            },
            press(Role::Button, b"Move end"),
            see(b"End moved"),
            see(b"Revision 1"),
            see(b"1209600"),
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{}", outcome.trace.join("\n"));
    assert_durable_environment(world.front());
}

fn assert_durable_environment(front: &jig_ops_world::joint::front::Joint) {
    use jig_core_tasks as tasks;
    use jig_ops_domain as root;
    use jig_ops_domain_infrastructure as infra;
    use jig_ops_fake_production as production;
    let engine = &front.engine;
    let environment = infra::Environment::new(b"staging".as_slice().into(), b"payments".as_slice().into());
    let creations: Vec<_> = engine
        .systems
        .production
        .observed()
        .iter()
        .filter_map(|event| match event {
            production::ObservedEffect::Create { environment, key, price, applied: true } => {
                Some((environment, key, price))
            }
            production::ObservedEffect::Create { .. }
            | production::ObservedEffect::Silence { .. }
            | production::ObservedEffect::Restart { .. }
            | production::ObservedEffect::Scale { .. }
            | production::ObservedEffect::Rollback { .. }
            | production::ObservedEffect::TearDown { .. } => None,
        })
        .collect();
    assert_eq!(creations.len(), 1, "the accepted provision reached the independent system once");
    assert_eq!(*creations[0].0, production::EnvironmentName::new("staging", "payments"));
    assert_eq!(*creations[0].2, 40);
    let decisions: Vec<_> = engine
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(jig_core::Record::Core(jig_core::CoreRecord::ProposalDecision(decision)))
                if decision.proposer == jig_core_people::Proposer::Task(1) =>
            {
                Some(decision)
            }
            jig_ops_domain::Record::Core(_)
            | jig_ops_domain::Record::Infrastructure(_)
            | jig_ops_domain::Record::Observability(_) => None,
        })
        .collect();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].by, 1);
    assert_eq!(decisions[0].kind, tasks::ProposalKind::Batch);
    assert_eq!(decisions[0].choice, jig_core_people::ProposalChoice::Accepted);
    let (first, last) = decisions[0].created.expect("the accepted chat proposal created E");
    assert_eq!(first, last);
    assert_eq!(creations[0].1.task, first);
    let e = engine
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(jig_core::Record::Tasks(tasks::Stored::Ended(task))) if task.number == first => {
                Some(task)
            }
            jig_ops_domain::Record::Core(_)
            | jig_ops_domain::Record::Infrastructure(_)
            | jig_ops_domain::Record::Observability(_) => None,
        })
        .expect("E genuinely finished");
    assert_eq!(e.funder, tasks::Funder::Pool { project: 1, person: 1, period: 1 });
    assert_eq!(e.numbers.spent, 40);
    assert!(matches!(e.phase, tasks::Phase::Ended(tasks::Ending::Done(_))));
    let made = engine
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Infrastructure(infra::Record::Made { environment: found, key, project })
                if *found == environment =>
            {
                Some((key, project))
            }
            jig_ops_domain::Record::Core(_)
            | jig_ops_domain::Record::Infrastructure(_)
            | jig_ops_domain::Record::Observability(_) => None,
        })
        .expect("connector durably owns the genuine creation association");
    assert_eq!(*made.1, Some(1));
    assert_eq!(made.0.task, first);
    assert_eq!(made.0.deployment, creations[0].1.deployment);
    assert_eq!(made.0.purpose, creations[0].1.purpose);
    assert_durable_end(front, &environment);
}

fn assert_durable_end(
    front: &jig_ops_world::joint::front::Joint,
    environment: &jig_ops_domain_infrastructure::Environment,
) {
    use jig_ops_domain as root;
    use jig_ops_domain_infrastructure as infra;
    let engine = &front.engine;
    let ends: Vec<_> = engine.systems.production.end_changes.iter().filter(|change| change.applied).collect();
    assert_eq!(ends.len(), 1);
    assert_eq!(ends[0].environment, jig_ops_fake_production::EnvironmentName::new("staging", "payments"));
    assert_eq!(ends[0].revision, 1);
    assert_eq!(ends[0].until, 1_209_600);
    let answers: Vec<_> = engine
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Infrastructure(infra::Record::ApplicationAnswer { token, outcome, .. }) => {
                Some((token, outcome))
            }
            jig_ops_domain::Record::Core(_)
            | jig_ops_domain::Record::Infrastructure(_)
            | jig_ops_domain::Record::Observability(_) => None,
        })
        .collect();
    assert_eq!(answers.len(), 1);
    assert_eq!(
        *answers[0].1,
        infra::ApplicationOutcome::MovedEnd {
            environment: environment.clone(),
            now: infra::EnvironmentEnd { revision: 1, until: 1_209_600 },
        }
    );
    let markers: Vec<_> = engine
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(jig_core::Record::People(jig_core_people::Stored::Answer {
                key,
                ask,
                outcome: jig_core_people::Outcome::Applied { connector: 2, token },
                ..
            })) => Some((key, ask, token)),
            jig_ops_domain::Record::Core(_)
            | jig_ops_domain::Record::Infrastructure(_)
            | jig_ops_domain::Record::Observability(_) => None,
        })
        .collect();
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].0.person, 1);
    assert_eq!(markers[0].2, answers[0].0);
    let (_, request, digest) =
        jig_ops_world::joint::infrastructure::ask(jig_ops_client_domain::infrastructure::Ask::NativeMoveEnd {
            object: skein_lib::Token::new(0),
            project: 1,
            environment: jig_ops_client_domain::infrastructure::native::Environment {
                pool: environment.pool.clone(),
                name: environment.name.clone(),
            },
            revision: jig_client::contract::Revision { value: 0 },
            until: 1_209_600,
        })
        .expect("the protocol hash includes all supplied owner facts, independent of the local view handle");
    assert_eq!(request.revision, 0, "Q named the owner's seen end revision");
    assert_eq!(markers[0].1.as_ref(), &jig_core_people::Ask::Application { project: 1, connector: 2, digest });
    assert!(front.environment_documents.iter().any(|document| matches!(document,
        infra::ClientDocument::Environment {
            environment: found, fact, end_revision: 0, end: None, ..
        } if found == environment && fact.until == 604_800 && fact.price == 40)));
    let end = engine
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Infrastructure(infra::Record::EnvironmentEnd { environment: found, now, .. })
                if found == environment =>
            {
                Some(*now)
            }
            jig_ops_domain::Record::Core(_)
            | jig_ops_domain::Record::Infrastructure(_)
            | jig_ops_domain::Record::Observability(_) => None,
        })
        .expect("the owner saved its desired end separately from the observed lease");
    assert_eq!(end, infra::EnvironmentEnd { revision: 1, until: 1_209_600 });
}
