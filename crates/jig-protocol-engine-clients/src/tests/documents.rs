use super::fixtures;
use crate::translate;
use jig_client_protocol_wire::v8 as w;
fn me_preserves_two_projects_in_its_wire_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::owner::frame(&fixtures::frame_two_projects(), &limits),
        Ok(jig_client_test_records::frame_two_projects_record())
    );
}
const SPARE_FRAME_ROOM: [(u32, u32); 2] = [(32, 2), (1, 32)];
#[test]
fn me_encoding_preserves_rows_despite_spare_list_capacity() {
    let limits = fixtures::wire_limits();
    for (goal_room, project_room) in SPARE_FRAME_ROOM {
        let mut frame = fixtures::frame_two_projects();
        for index in 0..frame.charters.len() {
            let charter = frame.charters.get_mut(index).expect("fixture charter");
            let mut goals = skein_lib::List::with_capacity(goal_room);
            for number in &charter.goals {
                goals.push(*number).expect("fixture goal room");
            }
            charter.goals = goals;
        }
        let mut projects = skein_lib::List::with_capacity(project_room);
        for project in frame.projects.into_boxed() {
            projects.push(project).expect("fixture project room");
        }
        frame.projects = projects;
        assert_eq!(
            translate::owner::frame(&frame, &limits),
            Ok(jig_client_test_records::frame_two_projects_record()),
            "goals room {goal_room}, projects room {project_room}"
        );
    }
}
fn a_task_with_a_hold_becomes_its_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::task::document(&fixtures::task_with_hold(), &limits),
        Ok(jig_client_test_records::task_with_hold_record())
    );
}
fn a_chip_in_each_phase_becomes_its_record() {
    let limits = fixtures::wire_limits();
    let phases = fixtures::phases();
    let records = jig_client_test_records::owner_chip_records();
    let source = fixtures::chip();
    for ((phase, _compact), record) in phases.into_iter().zip(records) {
        let mut chip = source.clone();
        chip.phase = phase;
        assert_eq!(translate::chips::chip(&chip, &limits), Ok(record));
    }
}
#[test]
fn me_overflow_reports_the_wire_bound_it_exceeds() {
    let frame = fixtures::frame_two_projects();
    let limits = fixtures::wire_limits();
    let rows = [
        (w::Limits { me_projects: 1, ..limits }, w::Path::MeProjects),
        (w::Limits { me_charters: 0, ..limits }, w::Path::MeCharters),
        (w::Limits { document_text_bytes: 1, ..limits }, w::Path::DocumentTextBytes),
    ];
    for (limits, path) in rows {
        assert_eq!(
            translate::owner::frame(&frame, &limits),
            Err(w::Problem { path, reason: skein_codec::Reason::Bound })
        );
    }
}

#[test]
fn a_non_utf8_document_text_fails_translation() {
    let limits = fixtures::wire_limits();
    for bytes in [&[0xc3_u8][..], &[255_u8, b'/'][..]] {
        let value = jig_domain_core::documents::Text { bytes: bytes.into(), cut: true };
        assert_eq!(
            translate::owner::text(&value, &limits),
            Err(w::Problem { path: w::Path::DocumentTextBytes, reason: skein_codec::Reason::Utf8 })
        );
    }
}

fn a_run_writing_with_a_call_becomes_its_run_state() {
    let limits = fixtures::wire_limits();
    let source = fixtures::run_writing();
    assert_eq!(translate::live::run(&source, &limits), Ok(jig_client_test_records::run_writing_record()));
}
fn a_run_text_keeps_its_turn_retry_and_offsets() {
    let limits = fixtures::wire_limits();
    assert_eq!(translate::live::writing(&fixtures::writing(), &limits), Ok(jig_client_test_records::writing_record()));
}
fn an_inbox_with_both_groups_becomes_its_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::inbox::document(&fixtures::inbox_document(), &limits),
        Ok(jig_client_test_records::inbox_both_groups_record())
    );
}
fn a_deployment_document_becomes_its_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::deployment::document(&fixtures::deployment_document(), &limits),
        Ok(jig_client_test_records::deployment_rows_record())
    );
}
fn conversations_keep_their_rows_more_and_opaque_cursor() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::live::conversations(
            &fixtures::conversations(),
            jig_client_test_records::live_cursor_record(),
            &limits
        ),
        Ok(jig_client_test_records::conversations_rows_record())
    );
}
fn goals_keep_all_ten_phase_counts_and_budget() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::live::goals(7, &fixtures::goals(), &limits),
        Ok(jig_client_test_records::goals_rows_record())
    );
}
fn tasks_keep_the_filter_requester_goal_and_changed_wall() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::live::tasks(&fixtures::tasks_document(), &limits),
        Ok(jig_client_test_records::tasks_rows_record())
    );
}
fn a_tree_keeps_its_rows_folds_dependencies_and_opaque_cursor() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::live::tree(&fixtures::tree_document(), Some(jig_client_test_records::live_cursor_record()), &limits),
        Ok(jig_client_test_records::tree_rows_record())
    );
}
#[test]
fn live_nested_pieces_past_the_wire_bounds_are_refused_naming_them() {
    let source = fixtures::deployment_document();
    let limits = fixtures::wire_limits();
    let rows = [
        ("host runs", w::Limits { owner_host_runs: 0, ..limits }, w::Path::OwnerHostRuns),
        ("deployment hosts", w::Limits { owner_deployment_hosts: 0, ..limits }, w::Path::OwnerDeploymentHosts),
        ("deployment attempts", w::Limits { owner_deployment_attempts: 0, ..limits }, w::Path::OwnerDeploymentAttempts),
        ("account runs", w::Limits { owner_account_runs: 0, ..limits }, w::Path::OwnerAccountRuns),
        ("account waiting", w::Limits { owner_account_waiting: 0, ..limits }, w::Path::OwnerAccountWaiting),
        ("deployment accounts", w::Limits { owner_deployment_accounts: 0, ..limits }, w::Path::OwnerDeploymentAccounts),
    ];
    for (label, limits, path) in rows {
        assert_eq!(
            translate::deployment::document(&source, &limits),
            Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
            "{label}"
        );
    }
}

#[test]
fn documents_become_their_literal_records() {
    table([
        ("me_preserves_two_projects_in_its_wire_record", me_preserves_two_projects_in_its_wire_record),
        ("a_task_with_a_hold_becomes_its_record", a_task_with_a_hold_becomes_its_record),
        ("a_chip_in_each_phase_becomes_its_record", a_chip_in_each_phase_becomes_its_record),
        ("a_run_writing_with_a_call_becomes_its_run_state", a_run_writing_with_a_call_becomes_its_run_state),
        ("a_run_text_keeps_its_turn_retry_and_offsets", a_run_text_keeps_its_turn_retry_and_offsets),
        ("an_inbox_with_both_groups_becomes_its_record", an_inbox_with_both_groups_becomes_its_record),
        ("a_deployment_document_becomes_its_record", a_deployment_document_becomes_its_record),
        ("conversations_keep_their_rows_more_and_opaque_cursor", conversations_keep_their_rows_more_and_opaque_cursor),
        ("goals_keep_all_ten_phase_counts_and_budget", goals_keep_all_ten_phase_counts_and_budget),
        (
            "tasks_keep_the_filter_requester_goal_and_changed_wall",
            tasks_keep_the_filter_requester_goal_and_changed_wall,
        ),
        (
            "a_tree_keeps_its_rows_folds_dependencies_and_opaque_cursor",
            a_tree_keeps_its_rows_folds_dependencies_and_opaque_cursor,
        ),
    ]);
}

type Case = (&'static str, fn());

fn table<const N: usize>(cases: [Case; N]) {
    let mut failures = [None; N];
    let mut passed = true;
    for ((name, case), failure) in cases.into_iter().zip(&mut failures) {
        if std::panic::catch_unwind(case).is_err() {
            *failure = Some(name);
            passed = false;
        }
    }
    assert!(passed, "{failures:?}");
}

const TEMPLATE: u32 = 2;
const TEMPLATE_CHARTER: u32 = 1;
const TEMPLATE_HOLDER_ROLE: u32 = 0;
#[test]
fn a_me_document_carries_each_project_s_templates_and_bounds() {
    use jig_domain_core::acts;
    use jig_domain_people as p;
    use jig_domain_tasks as t;
    let mut frame = fixtures::frame_two_projects();
    for index in 0..frame.projects.len() {
        let project = frame.projects.get_mut(index).unwrap();
        project.templates = fixtures::list([jig_domain_core::Template {
            number: TEMPLATE,
            name: b"task".as_slice().into(),
            executor: p::TemplateExecutor::Charter(TEMPLATE_CHARTER),
            contract: t::Contract::Report { words: 32 },
            shallower: 1,
            budget: p::TemplateBudget::Amount(42),
            deadline: p::TemplateDeadline::Given,
            tracked: Some(1),
            flags: t::Flags::PLAIN,
            label: TEMPLATE,
            grace: None,
        }]);
        project.creating = Some(acts::Creating {
            within: acts::Bounds::Goal {
                authority: Box::new(fixtures::authority()),
                pool_left: 50,
                charters: fixtures::list([TEMPLATE_CHARTER]),
                procedures: skein_lib::List::with_capacity(0),
                latest: None,
                budget: 50,
                maximum_budget: 100,
                maximum_deadline: None,
            },
            beyond: acts::Beyond::Proposed {
                to: acts::Recipient::Role { project: project.project, role: TEMPLATE_HOLDER_ROLE },
            },
        });
    }
    let record = translate::owner::frame(&frame, &w::CEILINGS).unwrap();
    let mut reader = skein_lib::Reader::new(record.projects());
    for source in &frame.projects {
        let bytes = skein_codec::piece::split(&mut reader).unwrap().unwrap();
        let project = w::MeProject::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&bytes)).unwrap();
        assert_eq!(project.project(), source.project);
        let mut templates = skein_lib::Reader::new(project.templates());
        let bytes = skein_codec::piece::split(&mut templates).unwrap().unwrap();
        let template = w::Template::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&bytes)).unwrap();
        assert_eq!(template.number(), TEMPLATE);
        assert_eq!(
            template.budget(),
            &w::TemplateBudget::Amount(
                w::TemplateAmount::new(&w::CEILINGS, w::TemplateAmountParts { value: 42 }).unwrap()
            )
        );
        assert_eq!(project.creating().as_ref().unwrap().within().budget(), 50);
        assert_eq!(
            project.creating().as_ref().unwrap().beyond(),
            &w::Beyond::Proposed(
                w::BeyondProposed::new(
                    &w::CEILINGS,
                    w::BeyondProposedParts {
                        to: w::Recipient::Role(
                            w::RecipientRole::new(
                                &w::CEILINGS,
                                w::RecipientRoleParts { project: source.project, role: TEMPLATE_HOLDER_ROLE }
                            )
                            .unwrap()
                        )
                    }
                )
                .unwrap()
            )
        );
    }
    assert_eq!(skein_codec::piece::split(&mut reader), Ok(None));
}
#[test]
fn a_project_document_carries_its_policy_s_template_overrides() {
    let mut project = fixtures::project_document();
    project.policy.templates = fixtures::list([jig_domain_people::TemplateOverride {
        template: TEMPLATE,
        executor: None,
        budget: Some(jig_domain_people::TemplateBudget::Amount(42)),
        deadline: None,
        priority: None,
    }]);
    let record = translate::objects::project(&project, &w::CEILINGS).unwrap();
    let entry = record.policy().templates().get(0).unwrap();
    assert_eq!(entry.template(), TEMPLATE);
    assert_eq!(
        entry.budget(),
        &Some(w::TemplateBudget::Amount(
            w::TemplateAmount::new(&w::CEILINGS, w::TemplateAmountParts { value: 42 }).unwrap()
        ))
    );
}
