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

fn assert_alert(world: &jig_conformance::Harness<jig_ops_world::conformance::Ops>) {
    assert!(!world.outcome().stopped, "crashes {:?}: {}", world.outcome().crashes, world.trace.join("\n"));
    assert!(world.peers.results.contains(&5), "on-call read the incident report");
    let ended = |number| {
        world
            .store
            .rows
            .get(&jig_ops_domain::Key::Core(jig_domain_core::Key::Tasks(jig_domain_tasks::Key::Ended(number))))
    };
    for task in [3, 4, 5, 6] {
        assert!(ended(task).is_some(), "task {task} missing: {}", world.trace.join("\n"));
    }
    for (number, requester) in [(3, 2), (4, 2), (5, 2), (6, 5)] {
        let Some(jig_ops_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(row)))) =
            ended(number)
        else {
            panic!("ended task {number}")
        };
        assert_eq!(row.requester, jig_domain_tasks::Party::Task(requester), "W -> T1,T2,I -> R");
        if number == 3 || number == 4 {
            assert_eq!(row.authority.delegation.tasks, 0, "triage has no delegate authority");
            assert_eq!(row.authority.delegation.depth, 0);
            assert!(row.delegates.is_empty());
            let jig_domain_tasks::Contract::Verdict { choices } = &row.contract else {
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

fn result_world() -> jig_conformance::Harness<jig_ops_world::conformance::Ops> {
    use jig_domain_core as core;
    use jig_domain_people as people;
    use jig_ops_domain as root;
    use jig_ops_world::conformance::{Config, Ops, Story};
    use skein_lib::{ReplyTo, Token};
    let mut world = jig_conformance::Harness::<Ops>::new(Config::new(176, Story::Alert), 176);
    world.drain().expect("cold root");
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(177)),
        sign_in: world.peers.sign_in.expect("signed in"),
        key: [177; 16],
        ask: jig_ops_world::conformance::chat(b"a result reading story"),
    })));
    for _ in 0..128 {
        world.iterate().expect("scripted chat starts");
        if world.peers.has_active_agent() {
            break;
        }
    }
    assert!(world.peers.has_active_agent(), "assigned scripted chat");
    world
}

#[test]
fn a_run_whose_result_does_not_decode_fails_its_attempt_as_invalid() {
    use jig_domain_core as core;
    use jig_domain_tasks as tasks;
    use jig_ops_domain as root;
    let mut world = result_world();
    let (task, attempt) = world.peers.assignments[0];
    let event = world.peers.finish_result(b"not a smith result");
    world.send(event);
    world.drain().expect("invalid result is answered");
    assert_eq!(world.peers.read_results.len(), 1, "the protocol routing read the fenced result");
    let Some(root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(terminal)))) =
        world.store.rows.get(&root::Key::Core(core::Key::Core(core::CoreKey::Terminal { task, attempt })))
    else {
        panic!("durable failed attempt");
    };
    assert_eq!(terminal.end, tasks::End::Failed(tasks::Class::Invalid));
}

#[test]
fn a_stale_attempts_answer_is_fenced_before_its_result_is_read() {
    use jig_domain_core as core;
    use jig_domain_fleet as fleet;
    use jig_ops_domain as root;
    use skein_lib::Token;
    let mut world = jig_ops_world::conformance::alert(175, Cut::None);
    let (task, attempt) = world.peers.assignments[0];
    let read = world.peers.read_results.len();
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Answer {
        channel: Token::new(0),
        run: Token::new(task),
        attempt: Token::new(attempt),
        answer: fleet::Answer::Ended,
        payload: Token::new(99_999),
    })));
    world.drain().expect("stale answer discarded at the fleet fence");
    assert_eq!(world.peers.read_results.len(), read);
}

#[test]
fn an_ops_call_that_does_not_decode_is_answered_naming_the_field() {
    let mut world = result_world();
    let answers = world.peers.call_answers.len();
    let commits = world.outcome().commits;
    let event = world.peers.tool_call(b"effect_restart", br#"{"environment":"production","operation":77}"#, true);
    world.send(event);
    for _ in 0..128 {
        world.iterate().expect("undecodable effect answered");
        if world.peers.call_answers.len() > answers {
            break;
        }
    }
    assert_eq!(world.outcome().commits, commits, "an undecodable call decides nothing");
    assert_eq!(
        world.peers.call_answers[answers],
        (true, b"effect_restart: the field service is missing".as_slice().into())
    );
    world.drain().expect("scripted run settles after the answer");
    assert!(world.peers.entries.is_empty(), "an undecodable effect admits no outbox entry");
}

#[test]
fn an_ops_read_is_decoded_outside_the_root_and_answered() {
    let mut world = result_world();
    let answers = world.peers.call_answers.len();
    let event = world.peers.tool_call(
        b"read_logs",
        br#"{"environment":"production","service":"checkout","from":0,"through":0,"max_bytes":256,"filter":""}"#,
        false,
    );
    world.send(event);
    world.drain().expect("decoded read answered");
    let (error, text) = &world.peers.call_answers[answers];
    assert!(!error, "the authenticated task admits its connector read");
    assert_eq!(text.as_ref(), b"baseline", "the production log read is answered unchanged");
}
