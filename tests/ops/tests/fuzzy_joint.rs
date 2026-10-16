//! P reads and acts on the actual engine's public Ops client tree.
use jig_client_world::{
    Until,
    harness::{ClientId, FrontFault},
};
use jig_conformance::Cut;
use jig_domain_client::address::{Address as JigAddress, TaskSection};
use jig_ops_client_domain::{Address, observability as obs};
use jig_ops_domain as root;
use jig_ops_world::joint::engine::Milestone;
use jig_ops_world::joint::fixture;
use skein_fake_person::{Find, Step};
use skein_lib::Duration;
use skein_ui::Role;
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
    p_story(511, Cut::None, false);
}

type OpsEngine = jig_conformance::Harness<jig_ops_world::joint::engine::OpsApplication>;

type StoryWorld = jig_client_world::World<fixture::Client, jig_ops_world::joint::front::Joint>;

fn incident_lead(engine: &OpsEngine) -> u64 {
    let mut rows = engine.store.rows.values();
    rows.find_map(|row| match row {
        jig_ops_domain::Record::Observability(jig_ops_domain_observability::Record::Incident(incident))
            if incident.project == 1 && incident.number == 12 =>
        {
            incident.lead
        }
        root::Record::Core(_) | root::Record::Observability(_) | root::Record::Infrastructure(_) => None,
    })
    .expect("incident 12 has its retained lead")
}

fn remediation(engine: &OpsEngine, lead: u64) -> u64 {
    let mut rows = engine.store.rows.values();
    rows.find_map(|row| match row {
        jig_ops_domain::Record::Core(jig_domain_core::Record::Tasks(
            jig_domain_tasks::Stored::Live(task) | jig_domain_tasks::Stored::Ended(task),
        )) if task.requester == jig_domain_tasks::Party::Task(lead) => Some(task.number),
        root::Record::Core(_) | root::Record::Observability(_) | root::Record::Infrastructure(_) => None,
    })
    .expect("the lead has its remediation delegate")
}

fn p_story(seed: u64, cut: Cut, noise: bool) -> StoryWorld {
    let (mut world, p) = monitor(seed);
    world.front().engine.cut(cut);
    fixture::alert(world.front());
    let service =
        jig_ops_domain_infrastructure::Service::new(b"production".as_slice().into(), b"checkout".as_slice().into());
    let fact =
        world.front().engine.systems.infrastructure(jig_ops_domain_infrastructure::SystemRequest::Service { service });
    world.front().source(jig_ops_domain::Event::Infrastructure(jig_ops_domain_infrastructure::Event::System(fact)));
    world.person(
        p,
        vec![
            Step::Go { address: Address::Observability(obs::Address::Incidents { project: 1, open: true }) },
            Step::See { find: Find::named(Role::Link, b"Incident 12"), within: WITHIN },
            press(Role::Link, b"Incident 12"),
            see(b"Lead"),
        ],
    );
    passed(&mut world);
    let lead = incident_lead(&world.front().engine);
    world.person(
        p,
        vec![
            Step::Go { address: Address::Jig(JigAddress::Task { task: lead, section: TaskSection::Activity }) },
            see(b"read_series"),
            see(b"delegate"),
            Step::Go { address: Address::Jig(JigAddress::Inbox { project: None }) },
            see(b"Restart service"),
            see(b"checkout"),
            see(b"production"),
        ],
    );
    passed(&mut world);
    let remedy = remediation(&world.front().engine, lead);
    if noise {
        wander(&mut world, p, seed);
    }
    world.person(
        p,
        vec![
            Step::Go { address: Address::Jig(JigAddress::Task { task: lead, section: TaskSection::Plan }) },
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
            Step::Go { address: Address::Jig(JigAddress::Task { task: remedy, section: TaskSection::Panels }) },
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
    assert_durable_restart(&world.front().engine, lead, remedy);
    assert_acceptance_key(&mut world, false);
    world
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
    use jig_domain_tasks as tasks;
    let history: Vec<_> = engine
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            jig_ops_domain::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::History(history)))
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
            jig_ops_domain::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(task)))
                if task.number == 6 =>
            {
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

fn assert_durable_restart(engine: &OpsEngine, lead: u64, remedy: u64) {
    let restart = jig_ops_fake_production::ObservedEffect::Restart {
        service: jig_ops_fake_production::ServiceName::new("production", "checkout"),
        operation: 77,
        applied: true,
    };
    assert_eq!(engine.systems.production.observed().iter().filter(|event| **event == restart).count(), 1);
    let accepted: Vec<_> = engine
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            jig_ops_domain::Record::Core(jig_domain_core::Record::Core(
                jig_domain_core::CoreRecord::ProposalDecision(decision),
            )) if decision.proposer == jig_domain_people::Proposer::Task(remedy) => Some(decision),
            jig_ops_domain::Record::Core(_)
            | jig_ops_domain::Record::Observability(_)
            | jig_ops_domain::Record::Infrastructure(_) => None,
        })
        .collect();
    assert_eq!(accepted.len(), 1);
    assert_eq!(accepted[0].by, 1);
    assert_eq!(accepted[0].choice, jig_domain_people::ProposalChoice::Accepted);
    assert_eq!(accepted[0].kind, jig_domain_tasks::ProposalKind::Effect);
    assert!(accepted[0].created.is_none(), "acceptance funds the original effect, not a new task");
    let native = &engine.peers.conformance.documents;
    assert!(native.iter().all(|(commit, _)| *commit <= engine.store.applied));
    for phase in [
        jig_ops_domain_infrastructure::ProcedurePhase::EffectOutstanding,
        jig_ops_domain_infrastructure::ProcedurePhase::WaitingCondition,
    ] {
        assert!(native.iter().any(|(_, document)| matches!(document,
            jig_ops_domain::ClientDocument::Infrastructure(value)
            if matches!(value.as_ref(), jig_ops_domain_infrastructure::ClientDocument::Panel { task, state, .. } if *task == remedy && state.phase == phase))));
    }
    assert!(native.iter().any(|(_, document)| matches!(document,
        jig_ops_domain::ClientDocument::Observability(value)
        if matches!(value.as_ref(), jig_ops_domain_observability::ClientDocument::Incident { incident, .. }
            if incident.number == 12 && incident.lead == Some(lead) && incident.state == jig_ops_domain_observability::IncidentState::Resolved))));
}

#[test]
fn q_asks_for_an_environment_in_a_chat_and_accepts_what_it_costs() {
    q_story(513, Cut::None, false, false);
}

fn q_story(seed: u64, cut: Cut, noise: bool, reload: bool) -> StoryWorld {
    let mut world = fixture::environment_world(seed);
    let q = world.client_with(
        1,
        Address::Jig(JigAddress::Conversations { project: None }),
        fixture::limits(),
        fixture::client,
    );
    world.person(q, vec![Step::See { find: Find::named(Role::TextArea, b"First words"), within: WITHIN }]);
    passed(&mut world);
    world.front().engine.cut(cut);
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
    if noise {
        wander(&mut world, q, seed);
    }
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
        ],
    );
    passed(&mut world);
    move_end(&mut world, q, reload);
    world
}

fn move_end(world: &mut StoryWorld, q: ClientId, reload: bool) {
    if reload {
        world.fault(FrontFault::NoAnswer);
    }
    world.person(q, vec![press(Role::Button, b"Move end")]);
    let outcome = world.run(Until::Steps(1));
    assert!(matches!(outcome.verdict, Verdict::Passed | Verdict::Open { .. }), "{}", outcome.trace.join("\n"));
    if reload {
        let crashes_before = world.front().engine.outcome().crashes.len();
        let mut retired = false;
        for _ in 0..128 {
            let until = world.now().saturating_add(Duration::from_millis(10));
            world.run(Until::Time(until));
            let front = world.front();
            if front.lost_final_answers == 1 || front.engine.outcome().crashes.len() > crashes_before {
                retired = true;
                break;
            }
        }
        let front = world.front();
        assert!(
            retired,
            "actual original reply loss: asks {:?} lost {} crashes {:?}",
            front.engine.peers.application_asks,
            front.lost_final_answers,
            front.engine.outcome().crashes
        );
        world.reload(q);
        // A restored table request belongs to Kept, rather than the former
        // volatile environment card. Read the owner's actual resulting revision.
        world.person(q, vec![see(b"Revision 1"), see(b"1209600")]);
    } else {
        world.person(q, vec![see(b"End moved"), see(b"Revision 1"), see(b"1209600")]);
    }
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{}", outcome.trace.join("\n"));
    assert_durable_environment(world.front());
    if reload {
        assert_native_retries(world);
    }
}

fn assert_durable_environment(front: &jig_ops_world::joint::front::Joint) {
    use jig_domain_tasks as tasks;
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
            root::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::ProposalDecision(
                decision,
            ))) if decision.proposer == jig_domain_people::Proposer::Task(1) => Some(decision),
            jig_ops_domain::Record::Core(_)
            | jig_ops_domain::Record::Infrastructure(_)
            | jig_ops_domain::Record::Observability(_) => None,
        })
        .collect();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].by, 1);
    assert_eq!(decisions[0].kind, tasks::ProposalKind::Batch);
    assert_eq!(decisions[0].choice, jig_domain_people::ProposalChoice::Accepted);
    let (first, last) = decisions[0].created.expect("the accepted chat proposal created E");
    assert_eq!(first, last);
    assert_eq!(creations[0].1.task, first);
    let e = engine
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(task))) if task.number == first => {
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
            root::Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::Answer {
                key,
                ask,
                outcome: jig_domain_people::Outcome::Applied { connector: 2, token },
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
            revision: jig_client_domain::contract::Revision { value: 0 },
            until: 1_209_600,
        })
        .expect("the protocol hash includes all supplied owner facts, independent of the local view handle");
    assert_eq!(request.revision, 0, "Q named the owner's seen end revision");
    assert_eq!(markers[0].1.as_ref(), &jig_domain_people::Ask::Application { project: 1, connector: 2, digest });
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

fn passed(world: &mut StoryWorld) {
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{}", outcome.trace.join("\n"));
}

/// Seeded choices among links from the actual displayed tree. Every press goes
/// through the fake person's ordinary binding, between the story's steps.
fn wander(world: &mut StoryWorld, person: ClientId, seed: u64) {
    world.person(
        person,
        vec![Step::Go { address: Address::Observability(obs::Address::Incidents { project: 1, open: true }) }],
    );
    passed(world);
    world.random_person_without_button(person, b"Sign out");
    passed(world);
    let mut random = skein_lib::Rng::new(seed);
    for _ in 0..3 {
        let links: Vec<_> = world
            .tree(person)
            .nodes()
            .iter()
            .filter(|node| node.role == Role::Link && node.link.is_some())
            .filter_map(|node| node.name.clone())
            .collect();
        assert!(!links.is_empty(), "actual story page has navigation links");
        let selected = usize::try_from(random.below(links.len() as u64)).expect("bounded link index");
        world.person(person, vec![press(Role::Link, &links[selected])]);
        passed(world);
        match random.below(3) {
            0 => world.fault(FrontFault::DropWatch),
            1 => world.fault(FrontFault::Slow { span: Duration::from_millis(20) }),
            2 => {}
            value => panic!("bounded random choice {value}"),
        }
        passed(world);
    }
}

fn position(world: &mut StoryWorld, wanted: Milestone) -> u64 {
    world
        .front()
        .engine
        .peers
        .milestones
        .iter()
        .find_map(|(milestone, commit)| (*milestone == wanted).then_some(*commit))
        .expect("genuine durable milestone reached")
}

fn assert_recovered(world: &mut StoryWorld, commit: u64) {
    let until = world.now().saturating_add(Duration::from_millis(100));
    let outcome = world.run(Until::Time(until));
    assert_eq!(outcome.verdict, Verdict::Passed, "{}", outcome.trace.join("\n"));
    let front = world.front();
    assert_eq!(front.engine.outcome().crashes, vec![commit]);
    assert_eq!(front.disconnects, 1);
    assert_eq!(front.reconnects, 1);
    assert!(!front.engine.outcome().stopped);
    front.engine.peers.protocol.check().expect("all consumed packets retain their actual ACK");
}

fn assert_acceptance_key(world: &mut StoryWorld, replayed: bool) {
    use jig_domain_people as people;
    let front = world.front();
    let remedy = remediation(&front.engine, incident_lead(&front.engine));
    let requests: Vec<_> = front
        .engine
        .peers
        .keyed_asks
        .iter()
        .filter(|(_, _, ask)| {
            matches!(
                ask,
                people::Ask::DecideProposal {
                    proposer: people::Proposer::Task(task),
                    decision: people::ProposalDecision::Accept,
                    ..
                } if *task == remedy
            )
        })
        .collect();
    assert!(requests.len() >= if replayed { 2 } else { 1 }, "actual authenticated acceptance attempts: {requests:?}");
    assert!(
        requests.iter().all(|request| request == &requests[0]),
        "P's whole original authenticated key and ask survive"
    );
    let (_, key, ask) = requests[0];
    let answers: Vec<_> = front
        .engine
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            jig_ops_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer {
                key: found,
                ask: made_ask,
                outcome,
                ..
            })) if found.key == *key => Some((found, made_ask, outcome)),
            jig_ops_domain::Record::Core(_)
            | jig_ops_domain::Record::Infrastructure(_)
            | jig_ops_domain::Record::Observability(_) => None,
        })
        .collect();
    assert_eq!(answers.len(), 1, "one actual keyed acceptance answer");
    assert_eq!(answers[0].0.person, 1);
    assert_eq!(answers[0].1.as_ref(), ask);
    assert!(matches!(
        answers[0].2,
        people::Outcome::ProposalDecided {
            proposer: people::Proposer::Task(task),
            by: 1,
            choice: people::ProposalChoice::Accepted,
            ..
        } if *task == remedy
    ));
}

fn assert_native_retries(world: &mut StoryWorld) {
    let front = world.front();
    let requests = &front.engine.peers.application_asks;
    assert!(requests.len() >= 2, "actual pending whole request was replayed after reload: {requests:?}");
    assert!(requests.iter().all(|request| request == &requests[0]), "same authenticated key, whole request and digest");
    let key = requests[0].1;
    assert!(
        front.engine.store.rows.values().any(|row| matches!(row,
        jig_ops_domain::Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::Answer {
            key: found, outcome: jig_domain_people::Outcome::Applied { connector: 2, .. }, ..
        })) if found.key == key)),
        "the actual keyed answer survived the reload"
    );
}

#[test]
fn p_acceptance_survives_the_commit_before_its_durable_decision() {
    let mut baseline = p_story(531, Cut::None, false);
    let commit = position(&mut baseline, Milestone::Acceptance(6)) - 1;
    let mut recovered = p_story(531, Cut::AfterCommit(commit), false);
    assert_recovered(&mut recovered, commit);
    assert_acceptance_key(&mut recovered, false);
}

#[test]
fn p_acceptance_survives_its_durable_decision_commit() {
    let mut baseline = p_story(532, Cut::None, false);
    let commit = position(&mut baseline, Milestone::Acceptance(6));
    let mut recovered = p_story(532, Cut::AfterCommit(commit), false);
    assert_recovered(&mut recovered, commit);
    assert_acceptance_key(&mut recovered, true);
}

#[test]
fn p_acceptance_survives_the_commit_after_its_durable_decision() {
    let mut baseline = p_story(533, Cut::None, false);
    let commit = position(&mut baseline, Milestone::Acceptance(6)) + 1;
    let mut recovered = p_story(533, Cut::AfterCommit(commit), false);
    assert_recovered(&mut recovered, commit);
    assert_acceptance_key(&mut recovered, true);
}

#[test]
fn q_provision_survives_its_acceptance_commit_and_move_end_survives_reload() {
    let mut baseline = q_story(534, Cut::None, false, false);
    let commit = position(&mut baseline, Milestone::Acceptance(1));
    let mut recovered = q_story(534, Cut::AfterCommit(commit), false, true);
    assert_recovered(&mut recovered, commit);
}

#[test]
fn q_provision_survives_its_made_commit_and_move_end_survives_reload() {
    let mut baseline = q_story(535, Cut::None, false, false);
    let commit = position(&mut baseline, Milestone::Made);
    let mut recovered = q_story(535, Cut::AfterCommit(commit), false, true);
    assert_recovered(&mut recovered, commit);
}

#[test]
fn q_move_end_survives_its_durable_answer_commit_and_reload() {
    let mut baseline = q_story(536, Cut::None, false, false);
    let commit = position(&mut baseline, Milestone::MoveEnd);
    let mut recovered = q_story(536, Cut::AfterCommit(commit), false, true);
    assert_recovered(&mut recovered, commit);
}

#[test]
fn p_story_survives_seeded_commit_crashes_and_random_presses() {
    for seed in [537, 548] {
        let mut world = p_story(seed, Cut::Random { seed: seed ^ 0x33 }, true);
        assert_eq!(world.front().engine.outcome().crashes.len(), 1);
        assert_eq!(world.front().disconnects, 1);
        assert_eq!(world.front().reconnects, 1);
    }
}

#[test]
fn q_story_survives_seeded_commit_crashes_reload_and_random_presses() {
    for seed in [539, 540] {
        let mut world = q_story(seed, Cut::Random { seed: seed ^ 0x33 }, true, true);
        assert_eq!(world.front().engine.outcome().crashes.len(), 1);
        assert_eq!(world.front().disconnects, 1);
        assert_eq!(world.front().reconnects, 1);
    }
}
