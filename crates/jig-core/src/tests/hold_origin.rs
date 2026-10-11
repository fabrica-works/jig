//! Connector notifications and documents retain only their opaque owner references.

use super::*;

#[test]
fn held_connector_parts_keep_exact_origin_and_refuse_shortened_historical_keys() {
    let (mut core, mut limits) = make_tree();
    let env = Env { now: Time::ZERO, wall: Wall::from_nanos(9), limits };
    let source = tasks::Name { connector: 2, path: Box::new([Box::from(&b"whole"[..])]) };
    let _out =
        step(&mut core, &env, Event::EffectConnector(connector::Event::Drift { task: 2, resource: source.clone() }));
    let row = core.tasks.task(2).unwrap();
    let tasks::Phase::Held { source: Some(origin), .. } = &row.phase else { panic!("retained source") };
    assert_eq!(**origin, tasks::HoldSource::Resource(source));
    let chip = chip_from_row(&limits, row).unwrap();
    let part = chip.details.hold.unwrap().source.unwrap();
    assert_eq!(part.connector, 2);
    assert_eq!(part.kind, PartKind::Resource);
    let PartKey::Path(path) = part.key else { panic!("whole path") };
    assert_eq!(path.get(0).unwrap().as_ref(), b"whole");
    let milestone = tasks::Milestone { task: 2, position: row.milestone, phase: row.phase.clone(), at: env.wall };
    let line = milestone_from_row(&limits, &milestone).unwrap();
    let MilestonePhase::Held { source: Some(_), .. } = line.phase else { panic!("historical source") };
    limits.documents.text_bytes = 4;
    assert_eq!(chip_from_row(&limits, row), Err(Missing::TooLarge));
    assert_eq!(milestone_from_row(&limits, &milestone), Err(Missing::TooLarge));
}

#[test]
fn failed_and_uncertain_outbox_parts_keep_the_actual_connector_and_entry() {
    for outcome in [connector::OutboxOutcome::Failed, connector::OutboxOutcome::Held { entry: 8 }] {
        let (mut core, limits) = make_tree();
        let _out = step(
            &mut core,
            &Env { now: Time::ZERO, wall: Wall::from_nanos(9), limits },
            Event::EffectConnector(connector::Event::Outbox { connector: 2, task: 2, entry: 8, outcome }),
        );
        let chip = chip_from_row(&limits, core.tasks.task(2).unwrap()).unwrap();
        assert_eq!(
            chip.details.hold.unwrap().source,
            Some(Part { connector: 2, kind: PartKind::Outbox, key: PartKey::Number(8) })
        );
    }
}

#[test]
fn drift_task_document_keeps_the_hold_account_separate_from_the_whole_resource_origin() {
    let (mut core, limits) = make_tree();
    let resource = tasks::Name { connector: 2, path: Box::new([Box::from(&b"whole"[..]), Box::from(&[255, 0][..])]) };
    let _out = step(
        &mut core,
        &Env { now: Time::ZERO, wall: Wall::from_nanos(9), limits },
        Event::EffectConnector(connector::Event::Drift { task: 2, resource: resource.clone() }),
    );
    let row = core.tasks.task(2).unwrap();
    let tasks::Phase::Held { why, source: Some(origin), .. } = &row.phase else { panic!("held drift") };
    assert_eq!(*why, tasks::Hold::Drift);
    assert_eq!(**origin, tasks::HoldSource::Resource(resource.clone()));
    let Document::Task(task) = make(&core, &limits, 1, &Subject::Task { task: 2 }, &window()).unwrap() else {
        panic!("held task document")
    };
    let hold = task.hold.unwrap();
    assert_eq!(hold.account, Some(Part { connector: 2, kind: PartKind::Hold, key: PartKey::Number(2) }));
    let source = hold.source.unwrap();
    assert_eq!(source.connector, resource.connector);
    assert_eq!(source.kind, PartKind::Resource);
    let PartKey::Path(segments) = source.key else { panic!("resource origin") };
    assert_eq!(segments.as_slice(), resource.path.as_ref());
    assert_eq!(hold.reason, tasks::Hold::Drift);
    assert_eq!(hold.revision, row.revisions.hold);
    let Document::Escalation(escalation) =
        make(&core, &limits, 1, &Subject::Escalation { task: 2 }, &window()).unwrap()
    else {
        panic!("escalation")
    };
    assert_eq!(escalation.task.hold.unwrap().account, hold.account);
}

#[test]
fn failed_effect_task_document_names_the_held_task_account_and_preserves_the_actual_outbox_origin() {
    let (mut core, limits) = make_tree();
    let _out = step(
        &mut core,
        &Env { now: Time::ZERO, wall: Wall::from_nanos(9), limits },
        Event::EffectConnector(connector::Event::Outbox {
            connector: 2,
            task: 2,
            entry: 8,
            outcome: connector::OutboxOutcome::Failed,
        }),
    );
    let row = core.tasks.task(2).unwrap();
    let tasks::Phase::Held { why, source: Some(origin), .. } = &row.phase else { panic!("held failed effect") };
    assert_eq!(*why, tasks::Hold::EffectFailed);
    assert_eq!(**origin, tasks::HoldSource::Effect { connector: 2, entry: 8 });
    let Document::Task(task) = make(&core, &limits, 1, &Subject::Task { task: 2 }, &window()).unwrap() else {
        panic!("held task document")
    };
    let hold = task.hold.unwrap();
    assert_eq!(hold.account, Some(Part { connector: 2, kind: PartKind::Hold, key: PartKey::Number(2) }));
    assert_eq!(hold.source, Some(Part { connector: 2, kind: PartKind::Outbox, key: PartKey::Number(8) }));
    assert_eq!(hold.reason, tasks::Hold::EffectFailed);
    assert_eq!(hold.revision, row.revisions.hold);
    let Document::Escalation(escalation) =
        make(&core, &limits, 1, &Subject::Escalation { task: 2 }, &window()).unwrap()
    else {
        panic!("escalation")
    };
    assert_eq!(escalation.task.hold.unwrap().account, hold.account);
}
