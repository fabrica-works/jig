use jig_ops_domain as root;
use jig_ops_domain_infrastructure as infra;
use jig_ops_domain_observability as obs;
use jig_ops_world::conformance;

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

#[test]
fn an_incidents_subject_follows_an_alert_to_its_resolution() {
    let world = conformance::documents(17);
    let incidents: Vec<_> = world
        .peers
        .documents
        .iter()
        .filter_map(|(commit, document)| {
            if let root::ClientDocument::Observability(document) = document
                && let obs::ClientDocument::Incident { incident, fact } = document.as_ref()
            {
                Some((*commit, incident, fact))
            } else {
                None
            }
        })
        .collect();
    assert!(incidents.len() >= 2, "whole incident updates: {:?}", world.peers.documents);
    assert_eq!(incidents[0].1.state, obs::IncidentState::Alerting);
    let (_, resolved, health) = incidents.last().expect("resolved incident");
    assert_eq!(resolved.state, obs::IncidentState::Resolved);
    assert_eq!(resolved.number, 12);
    assert_eq!(resolved.severity, 3);
    assert!(resolved.timeline.len() >= 2);
    assert_eq!(health.as_ref().expect("current service health").errors, 0);
    for (_, incident, _) in &incidents {
        assert_eq!(incident.raised, incidents[0].1.raised, "raised is immutable");
        assert_eq!(incident.project, 1);
        assert_eq!(incident.service, jig_ops_world::service());
    }
    assert!(incidents.windows(2).all(|pair| pair[0].0 <= pair[1].0));
    assert!(!world.outcome().stopped);
}

#[test]
fn a_remediations_panel_says_restarting_then_waiting_for_healthy() {
    let world = conformance::documents(17);
    let panels: Vec<_> = world
        .peers
        .documents
        .iter()
        .filter_map(|(_, document)| {
            if let root::ClientDocument::Infrastructure(document) = document
                && let infra::ClientDocument::Panel { task, state, resources, effects } = document.as_ref()
            {
                Some((*task, state, resources, effects))
            } else {
                None
            }
        })
        .collect();
    let restarting = panels
        .iter()
        .position(|(_, state, _, _)| state.phase == infra::ProcedurePhase::EffectOutstanding)
        .unwrap_or_else(|| panic!("restarting panel: {panels:?}"));
    let waiting = panels
        .iter()
        .position(|(_, state, _, _)| state.phase == infra::ProcedurePhase::WaitingCondition)
        .expect("waiting for healthy panel");
    assert!(restarting < waiting, "panels: {panels:?}");
    assert_eq!(panels[restarting].0, panels[waiting].0);
    for (_, state, resources, _) in &panels {
        assert!(matches!(state.procedure, infra::Procedure::Remediate { operation: 77, .. }));
        assert!(
            resources
                .iter()
                .any(|resource| resource.named.resource == infra::Resource::Service(jig_ops_world::infra_service()))
        );
    }
    assert!(
        world.peers.ended_documents.contains(&(1200 + panels[waiting].0)),
        "the panel ends with live procedure state"
    );
    assert!(!world.outcome().stopped);
}

fn infrastructure_read(
    world: &mut jig_ops_world::InfrastructureWorld,
    scope: infra::ClientScope,
    subject: infra::ClientSubject,
) -> Result<infra::ClientDocument, infra::DocumentMissing> {
    use skein_lib::Token;
    world.event(infra::Event::DocumentPrepare { token: Token::new(801), subject });
    let before = (world.commits, world.production.observed().len());
    let out = world.event(infra::Event::ReadDocument { scope, read: Token::new(801), party: 1 });
    assert_eq!(before, (world.commits, world.production.observed().len()), "reads make no writes or system call");
    let [infra::Request::ReadAnswered { answer, .. }] = out.as_slice() else { panic!("one read answer: {out:?}") };
    answer.clone()
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one owner fixture exercises every part and retained scope through its whole lifecycle"
)]
fn infrastructure_parts_retain_scope_prices_and_attributed_drift_without_making_effects() {
    use jig_ops_world::{InfrastructureWorld, infra_environment};
    use skein_lib::{Token, Wall};
    let mut world = InfrastructureWorld::new(0, infra::Backend::OperationIds);
    let environment = infra_environment();
    let scope = infra::ClientScope::Project(1);
    let named = world.event(infra::Event::Names {
        task: 7,
        project: 1,
        resources: Box::new([
            infra::Resource::Environment(environment.clone()),
            infra::Resource::Pool(infra::Pool(environment.pool.clone())),
        ]),
    });
    world.release(&named);
    world.event(infra::Event::StartProcedure {
        task: 7,
        project: 1,
        procedure: infra::Procedure::Provision {
            environment: environment.clone(),
            until: 100,
            price: 40,
            deadline: 90,
        },
    });
    let fresh = world.event(infra::Event::Procedure { task: 7, signal: infra::ProcedureSignal::Step });
    world.release(&fresh);
    world.event(infra::Event::Procedure { task: 7, signal: infra::ProcedureSignal::Step });
    world
        .release(&[infra::Request::System(infra::SystemRequest::Pool { pool: infra::Pool(environment.pool.clone()) })]);
    let effect = infra::Effect::CreateEnvironment { environment: environment.clone(), until: 100, price: 40 };
    let key = infra::Key::procedure([1; 16], 7, 3);
    world.event(infra::Event::Describe { token: Token::new(1), effect: effect.clone() });
    let kept = world.event(infra::Event::Keep { project: Some(1), token: Token::new(1), entry: 3, key });
    let infra::ClientDocument::Outbox { entry, description } =
        infrastructure_read(&mut world, scope, infra::ClientSubject::Outbox { project: 1, entry: 3 })
            .expect("retained entry")
    else {
        panic!("outbox part")
    };
    assert_eq!(entry.key, key);
    assert_eq!(entry.effect, effect);
    assert_eq!(description.price, Some(40));
    assert_eq!(
        infrastructure_read(
            &mut world,
            infra::ClientScope::Project(2),
            infra::ClientSubject::Outbox { project: 2, entry: 3 }
        ),
        Err(infra::DocumentMissing::Unknown)
    );
    world.release(&kept);
    assert_eq!(
        infrastructure_read(&mut world, scope, infra::ClientSubject::Outbox { project: 1, entry: 3 }),
        Err(infra::DocumentMissing::Unknown),
        "settled entries have no invented archive"
    );
    let read = world.event(infra::Event::Procedure { task: 7, signal: infra::ProcedureSignal::EffectMade });
    world.release(&read);
    world.production.advance(5);
    world.now = 5;
    world.event(infra::Event::Procedure { task: 7, signal: infra::ProcedureSignal::Step });
    let infra::ClientDocument::Panel { state, resources, .. } =
        infrastructure_read(&mut world, scope, infra::ClientSubject::Panel { project: 1, task: 7 })
            .expect("provision panel")
    else {
        panic!("panel")
    };
    assert_eq!(state.phase, infra::ProcedurePhase::Holding);
    assert!(resources.iter().any(|resource| matches!(
        resource.fact,
        infra::ResourceFact::Environment { fact: Some(infra::EnvironmentFact { price: 40, until: 100, .. }), .. }
    )));
    assert!(matches!(
        infrastructure_read(&mut world, scope, infra::ClientSubject::Result { project: 1, task: 7 }),
        Ok(infra::ClientDocument::Result {
            state: infra::ProcedureState { phase: infra::ProcedurePhase::Holding, .. },
            ..
        })
    ));
    world.event(infra::Event::Describe {
        token: Token::new(2),
        effect: infra::Effect::Restart { service: jig_ops_world::infra_service(), operation: 88 },
    });
    world.event(infra::Event::KeepProposal { token: Token::new(2), number: 9, task: 7 });
    let infra::ClientDocument::Effect { description, effect, .. } =
        infrastructure_read(&mut world, scope, infra::ClientSubject::Effect { project: 1, proposal: 9 })
            .expect("proposed restart")
    else {
        panic!("effect part")
    };
    assert!(matches!(effect, infra::Effect::Restart { operation: 88, .. }));
    assert_eq!(description.kind, 1);
    assert_eq!(description.recovery, infra::Recovery::Keyed);
    world.event(infra::Event::Unnamed { task: 7 });
    assert_eq!(
        infrastructure_read(&mut world, scope, infra::ClientSubject::Panel { project: 1, task: 7 }),
        Err(infra::DocumentMissing::Unknown)
    );
    let refresh = world.event(infra::Event::ReadAfresh);
    assert!(refresh.iter().any(|request| matches!(request, infra::Request::System(infra::SystemRequest::Environment { environment: refreshed }) if refreshed == &environment)), "the retained creation refreshes even without live task names");
    world.release(&refresh);
    let infra::ClientDocument::Environment { fact, pool, .. } = infrastructure_read(
        &mut world,
        scope,
        infra::ClientSubject::Environment { project: 1, environment: environment.clone() },
    )
    .expect("creation retains project after task names end") else {
        panic!("environment")
    };
    assert_eq!((fact.price, fact.until), (40, 100));
    assert!(pool.is_some());
    assert_eq!(
        infrastructure_read(
            &mut world,
            infra::ClientScope::Project(2),
            infra::ClientSubject::Environment { project: 2, environment: environment.clone() }
        ),
        Err(infra::DocumentMissing::Unknown)
    );
    world.event(infra::Event::Names {
        task: 7,
        project: 1,
        resources: Box::new([infra::Resource::Environment(infra_environment())]),
    });
    world.production.other_hand(jig_ops_fake_production::Fault::HandDelete {
        environment: jig_ops_fake_production::EnvironmentName::new("staging", "payments"),
    });
    let account = infra::DriftAccount {
        resource: infra::Resource::Environment(environment.clone()),
        kind: infra::DriftKind::Deleted,
        at: Wall::from_nanos(604_000_000_000),
        by: Some(b"mallory".as_slice().into()),
    };
    world.event(infra::Event::System(infra::SystemEvent::Environment {
        environment: environment.clone(),
        fact: None,
        other_hand: true,
    }));
    let attributed = world.event(infra::Event::System(infra::SystemEvent::Drift { account: account.clone() }));
    assert!(
        !attributed.iter().any(|request| matches!(request, infra::Request::Drift { .. })),
        "attribution supplements the existing hold"
    );
    let infra::ClientDocument::Hold { drift, .. } =
        infrastructure_read(&mut world, scope, infra::ClientSubject::Hold { project: 1, task: 7 })
            .expect("hold account")
    else {
        panic!("hold part")
    };
    assert_eq!(drift.as_slice(), std::slice::from_ref(&account));
    world.restart();
    let infra::ClientDocument::Hold { drift, .. } =
        infrastructure_read(&mut world, scope, infra::ClientSubject::Hold { project: 1, task: 7 })
            .expect("restored hold")
    else {
        panic!("hold part")
    };
    assert_eq!(drift.as_slice(), std::slice::from_ref(&account));
    assert!(matches!(
        infrastructure_read(&mut world, scope, infra::ClientSubject::Project { project: 1 }),
        Ok(infra::ClientDocument::Project { .. })
    ));
    assert!(matches!(
        infrastructure_read(&mut world, infra::ClientScope::Deployment, infra::ClientSubject::Deployment),
        Ok(infra::ClientDocument::Deployment { .. })
    ));
}

fn observability_read(
    world: &mut jig_ops_world::World,
    scope: obs::ClientScope,
    subject: obs::ClientSubject,
) -> Result<obs::ClientDocument, obs::DocumentMissing> {
    use skein_lib::Token;
    world.event(obs::Event::DocumentPrepare { token: Token::new(802), subject });
    let before = (world.commits, world.production.observed().len());
    let out = world.event(obs::Event::ReadDocument { scope, read: Token::new(802), party: 1 });
    assert_eq!(before, (world.commits, world.production.observed().len()));
    let [obs::Request::ReadAnswered { answer, .. }] = out.as_slice() else { panic!("one read answer: {out:?}") };
    answer.clone()
}

#[test]
fn incident_parts_keep_raised_scope_and_cut_timeline_across_a_cold_start() {
    let mut world = jig_ops_world::World::with_templates(
        0,
        Box::new([obs::TriageTemplate { number: 1, delegate: Box::from(&b"triage"[..]) }]),
        Box::new([obs::LeadTemplate { number: 1, delegate: Box::from(&b"lead"[..]) }]),
    );
    let service = jig_ops_world::service();
    world.event(obs::Event::StartWatch { task: 7, project: 1, services: Box::new([service.clone()]), template: 1 });
    world.event(obs::Event::System(obs::SystemEvent::Alert {
        own: false,
        alert: obs::Alert { number: 12, service: service.clone(), severity: 3 },
    }));
    let kept = world.event(obs::Event::System(obs::SystemEvent::Alert {
        own: false,
        alert: obs::Alert { number: 13, service: service.clone(), severity: 0 },
    }));
    assert!(kept.iter().any(|request| matches!(request, obs::Request::News { subscribers, .. } if subscribers.iter().all(|subscriber| subscriber.class == obs::Class::Keep))));
    assert!(
        matches!(
            observability_read(
                &mut world,
                obs::ClientScope::Project(1),
                obs::ClientSubject::Alert { project: 1, alert: 13 }
            ),
            Ok(obs::ClientDocument::Alert { alert: obs::Alert { severity: 0, .. }, .. })
        ),
        "a kept alert has an owned account without waking the watch"
    );
    open_incident_from_triage(&mut world, &service);
    let subject = obs::ClientSubject::Incident { project: 1, incident: 12 };
    let scope = obs::ClientScope::Project(1);
    let obs::ClientDocument::Incident { incident: first, .. } =
        observability_read(&mut world, scope, subject.clone()).expect("incident")
    else {
        panic!("incident")
    };
    for observed in 1..=20 {
        world.now = observed;
        world.event(obs::Event::System(obs::SystemEvent::Fact {
            service: service.clone(),
            fact: obs::Fact {
                observed,
                healthy_replicas: 1,
                errors: 20,
                error_rate_percent: 20,
                load_percent: 80,
                load: Box::new([]),
            },
        }));
    }
    let obs::ClientDocument::Incident { incident, .. } =
        observability_read(&mut world, scope, subject.clone()).expect("current incident")
    else {
        panic!("incident")
    };
    assert!(incident.cut);
    assert_eq!(incident.timeline.len(), jig_ops_world::limits().samples_per_service);
    assert_eq!(incident.raised, first.raised);
    assert_eq!(incident.timeline.last().expect("bounded newest health").health.expect("observed health").observed, 20);
    assert_eq!(
        observability_read(
            &mut world,
            obs::ClientScope::Project(2),
            obs::ClientSubject::Incident { project: 2, incident: 12 }
        ),
        Err(obs::DocumentMissing::Unknown)
    );
    assert!(matches!(
        observability_read(&mut world, scope, obs::ClientSubject::Alert { project: 1, alert: 12 }),
        Ok(obs::ClientDocument::Alert { state: obs::IncidentState::Alerting, .. })
    ));
    assert!(matches!(
        observability_read(&mut world, scope, obs::ClientSubject::Service { project: 1, service: service.clone() }),
        Ok(obs::ClientDocument::Service { .. })
    ));
    assert!(matches!(
        observability_read(&mut world, scope, obs::ClientSubject::Project { project: 1 }),
        Ok(obs::ClientDocument::Project { .. })
    ));
    assert!(matches!(
        observability_read(&mut world, obs::ClientScope::Deployment, obs::ClientSubject::Deployment),
        Ok(obs::ClientDocument::Deployment { .. })
    ));
    world.event(obs::Event::StopWatch { task: 7 });
    let refresh = world.event(obs::Event::ReadAfresh);
    assert!(refresh.iter().any(|request| matches!(request, obs::Request::System(obs::SystemRequest::Facts { service: refreshed }) if refreshed == &service)), "retained incidents refresh health even after their watch ends");
    world.release(&refresh);
    world.restart();
    let obs::ClientDocument::Incident { incident, .. } =
        observability_read(&mut world, scope, subject).expect("retained incident")
    else {
        panic!("incident")
    };
    assert_eq!(incident.raised, first.raised);
    assert!(incident.cut);
    assert_eq!(incident.state, obs::IncidentState::Resolved);
}

#[test]
fn observability_outbox_parts_use_retained_scope_and_end_when_settled() {
    use skein_lib::Token;
    let mut world = jig_ops_world::World::new(0);
    let key = obs::Key::procedure([1; 16], 7, 3);
    let subject = obs::ClientSubject::Outbox { project: 1, entry: 3 };
    let effect = obs::Effect { rule: b"checkout-errors".as_slice().into(), until: 100 };
    world.event(obs::Event::DocumentPrepare { token: Token::new(803), subject: subject.clone() });
    world.event(obs::Event::Describe { token: Token::new(1), effect: effect.clone() });
    let kept = world.event(obs::Event::Keep { token: Token::new(1), entry: 3, key, project: Some(1) });
    assert!(kept.iter().any(
        |request| matches!(request,obs::Request::Marked { subjects } if subjects.as_slice().contains(&Token::new(803)))
    ));
    let obs::ClientDocument::Outbox { entry } =
        observability_read(&mut world, obs::ClientScope::Project(1), subject.clone()).expect("retained silence")
    else {
        panic!("outbox")
    };
    assert_eq!(entry.effect, effect);
    assert_eq!(entry.key, key);
    assert_eq!(entry.project, Some(1));
    assert_eq!(
        observability_read(
            &mut world,
            obs::ClientScope::Project(2),
            obs::ClientSubject::Outbox { project: 2, entry: 3 }
        ),
        Err(obs::DocumentMissing::Unknown)
    );
    world.restart();
    assert!(
        observability_read(&mut world, obs::ClientScope::Project(1), subject).is_err(),
        "restart settles the committed entry; no archive is invented"
    );
    let effect = obs::Effect { rule: b"checkout-errors".as_slice().into(), until: 200 };
    world.event(obs::Event::Describe { token: Token::new(2), effect });
    world.event(obs::Event::Keep {
        token: Token::new(2),
        entry: 4,
        key: obs::Key::procedure([1; 16], 7, 4),
        project: None,
    });
    assert_eq!(
        observability_read(
            &mut world,
            obs::ClientScope::Project(1),
            obs::ClientSubject::Outbox { project: 1, entry: 4 }
        ),
        Err(obs::DocumentMissing::Unknown),
        "unknown owner scope is never inferred from an entry number"
    );
}

fn open_incident_from_triage(world: &mut jig_ops_world::World, service: &obs::Service) {
    world.event(obs::Event::WatchStep { task: 7, step: 1, batch: 1, messages: Box::new([obs::WatchMessage::News]) });
    world.event(obs::Event::ProcedureDelegateOutcome { task: 7, step: 1, child: Some(100) });
    world.event(obs::Event::WatchStep {
        task: 7,
        step: 2,
        batch: 2,
        messages: Box::new([obs::WatchMessage::Triage {
            child: 100,
            verdict: obs::TriageVerdict::Incident { alerts: Box::new([12]), service: service.clone(), severity: 3 },
        }]),
    });
}

#[test]
fn deployment_documents_supply_actual_proposal_and_read_capacities_without_writes_or_system_calls() {
    let mut infrastructure = jig_ops_world::InfrastructureWorld::new(0, infra::Backend::OperationIds);
    let mut observability = jig_ops_world::World::new(0);
    for cold in [false, true] {
        if cold {
            infrastructure.restart();
            observability.restart();
        }
        let infra::ClientDocument::Deployment { effects, proposal_limit, staged_limit, .. } =
            infrastructure_read(&mut infrastructure, infra::ClientScope::Deployment, infra::ClientSubject::Deployment)
                .expect("bounded Deployment fixture")
        else {
            panic!("whole infrastructure Deployment")
        };
        assert_eq!(proposal_limit, jig_ops_world::infrastructure_limits().proposals);
        assert_eq!(staged_limit, jig_ops_world::infrastructure_limits().staged);
        assert!(effects.is_empty());
        assert_eq!(effects.capacity(), jig_ops_world::infrastructure_limits().effects);
        let obs::ClientDocument::Deployment { effects, read_limit, incident_limit, watches } =
            observability_read(&mut observability, obs::ClientScope::Deployment, obs::ClientSubject::Deployment)
                .expect("bounded Deployment fixture")
        else {
            panic!("whole observability Deployment")
        };
        assert_eq!(read_limit, jig_ops_world::limits().reads);
        assert_eq!(
            incident_limit,
            jig_ops_world::limits()
                .watches
                .checked_mul(jig_ops_world::limits().alerts_per_batch)
                .expect("bounded Deployment fixture")
        );
        assert_eq!(watches, 0);
        assert!(effects.is_empty());
        assert_eq!(effects.capacity(), jig_ops_world::limits().effects);
    }
}
