use jig_conformance::Cut;
use jig_ops_fake_production as production;
use jig_ops_world::conformance::night;

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

#[test]
fn the_night_scales_staging_under_an_observed_requirement_without_an_agent() {
    let world = night(17, Cut::None);
    assert!(world.peers.assignments.is_empty());
    assert_eq!(
        world
            .systems
            .production
            .observed()
            .iter()
            .filter(|effect| matches!(effect, production::ObservedEffect::Scale { applied: true, from: 3, to: 1, .. }))
            .count(),
        1,
        "{}",
        world.trace.join("\n")
    );
}

#[test]
fn the_nights_scale_entry_survives_the_commits_around_its_admission() {
    let baseline = night(17, Cut::None);
    let entry = baseline
        .peers
        .entries
        .iter()
        .filter_map(|(commit, kind)| (*kind == 2).then_some(*commit))
        .min()
        .expect("scale entry");
    for commit in [entry - 1, entry, entry + 1] {
        let world = night(17, Cut::AfterCommit(commit));
        assert_eq!(world.outcome().crashes, vec![commit]);
        assert!(world.peers.assignments.is_empty());
        assert_eq!(
            world
                .systems
                .production
                .observed()
                .iter()
                .filter(|effect| matches!(
                    effect,
                    production::ObservedEffect::Scale { applied: true, from: 3, to: 1, .. }
                ))
                .count(),
            1,
            "cut {commit}: {}",
            world.trace.join("\n")
        );
    }
}

#[test]
fn the_alert_storm_reaches_triage_and_the_person_accepts_the_production_restart() {
    let world = jig_ops_world::conformance::alert(17, Cut::None);
    assert_alert(&world);
}

#[test]
fn the_alert_storm_survives_the_commit_before_lead_receipt() {
    alert_at(0);
}

#[test]
fn the_alert_storm_survives_the_commit_at_lead_receipt() {
    alert_at(1);
}

#[test]
fn the_alert_storm_survives_the_commit_after_lead_receipt() {
    alert_at(2);
}

#[test]
fn the_alert_storm_survives_the_commit_at_person_acceptance() {
    alert_at(3);
}

#[test]
fn the_alert_storm_survives_the_commit_at_restart_entry() {
    alert_at(4);
}

#[test]
fn the_alert_storm_survives_the_commit_after_restart_entry() {
    alert_at(5);
}

fn alert_at(selected: usize) {
    let world = jig_ops_world::conformance::alert(17, Cut::None);
    assert_alert(&world);
    let acceptance = world.peers.accepted[0];
    let entry = world
        .peers
        .entries
        .iter()
        .filter_map(|(commit, kind)| (*kind == 1).then_some(*commit))
        .min()
        .expect("restart entry");
    let lead_commit = world.peers.lead_commits.first().expect("same-decision incident lead receipt").0;
    let commit = [lead_commit - 1, lead_commit, lead_commit + 1, acceptance, entry, entry + 1][selected];
    let recovered = jig_ops_world::conformance::alert(17, Cut::AfterCommit(commit));
    assert_eq!(recovered.outcome().crashes, vec![commit]);
    assert_alert(&recovered);
}

fn assert_alert(world: &jig_conformance::Harness<jig_ops_world::conformance::Ops>) {
    assert!(!world.outcome().stopped, "crashes {:?}: {}", world.outcome().crashes, world.trace.join("\n"));
    assert!(world.peers.results.contains(&5), "on-call read the incident report");
    let ended = |number| {
        world.store.rows.get(&jig_ops_domain::Key::Core(jig_core::Key::Tasks(jig_core_tasks::Key::Ended(number))))
    };
    for task in [3, 4, 5, 6] {
        assert!(ended(task).is_some(), "task {task} missing: {}", world.trace.join("\n"));
    }
    for (number, requester) in [(3, 2), (4, 2), (5, 2), (6, 5)] {
        let Some(jig_ops_domain::Record::Core(jig_core::Record::Tasks(jig_core_tasks::Stored::Ended(row)))) =
            ended(number)
        else {
            panic!("ended task {number}")
        };
        assert_eq!(row.requester, jig_core_tasks::Party::Task(requester), "W -> T1,T2,I -> R");
        if number == 3 || number == 4 {
            assert_eq!(row.authority.delegation.tasks, 0, "triage has no delegate authority");
            assert_eq!(row.authority.delegation.depth, 0);
            assert!(row.delegates.is_empty());
            let jig_core_tasks::Contract::Verdict { choices } = &row.contract else {
                panic!("triage's closed verdict")
            };
            assert_eq!(choices.len(), 2);
            for choice in choices {
                assert_eq!(choice.followups, 0);
            }
        }
    }
    let (_, incident, association) = world.peers.lead_commits.first().expect("durable lead association");
    assert_eq!(incident.lead, Some(5));
    assert_eq!(incident.alerts.as_ref(), &[12]);
    assert_eq!(association.task, 2);
    assert_eq!(association.child, Some(5));
    assert_eq!(association.kind, jig_ops_domain_observability::DelegationKind::Lead { incident: 12 });
    assert_eq!(world.peers.assignments.iter().filter(|(task, _)| *task == 3).count(), 1, "one triage for the storm");
    assert_eq!(
        world
            .systems
            .production
            .observed()
            .iter()
            .filter(|effect| matches!(effect, production::ObservedEffect::Restart { applied: true, operation: 77, .. }))
            .count(),
        1,
        "{}",
        world.trace.join("\n")
    );
}
