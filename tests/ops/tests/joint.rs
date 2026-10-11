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
    let mut world = fixture::world(511);
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
    let inbox =
        world.client_with(1, Address::Jig(JigAddress::Inbox { project: None }), fixture::limits(), fixture::client);
    fixture::alert(world.front());
    world.person(
        inbox,
        vec![
            see(b"Restart service"),
            Step::Gone { find: Find::text(b"Restart service"), within: Duration::from_secs(8) },
            see(b"0 need you"),
        ],
    );
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
