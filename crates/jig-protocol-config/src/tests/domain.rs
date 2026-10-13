use crate::{What, tests::fixtures};

fn engine() -> crate::Engine {
    fixtures::engine(fixtures::ENGINE_SMALL).expect("whole engine fixture")
}
fn core_config(document: &[u8]) -> jig_domain_core::Config {
    let mut engine = fixtures::engine(document).expect("checked engine");
    engine.charters[0].policy.instructions = Box::from(&b"Answer the question."[..]);
    crate::core(&engine, [3; 16], 11)
}
fn refuses(from: &[u8], to: &[u8], what: What) {
    let document = fixtures::with(fixtures::ENGINE_SMALL, from, to);
    let problems = fixtures::engine(&document).expect_err("refused domain configuration");
    assert_eq!(problems.problems.get(0).expect("one problem").what, what);
}
fn two_projects(number: u8) -> Box<[u8]> {
    let prefix = br#""projects": ["#;
    let start = skein_lib::bytes::find(fixtures::ENGINE_SMALL, prefix).unwrap().checked_add(prefix.len()).unwrap();
    let end = fixtures::ENGINE_SMALL.len().checked_sub(3).unwrap();
    let project = fixtures::ENGINE_SMALL.get(start..end).unwrap();
    let second = fixtures::with(
        project,
        br#""number": 1"#,
        &[b'"', b'n', b'u', b'm', b'b', b'e', b'r', b'"', b':', b' ', number],
    );
    let mut writer = skein_lib::Writer::new(project.len().checked_add(second.len()).unwrap().checked_add(1).unwrap());
    writer.put(project).unwrap();
    writer.put(b",").unwrap();
    writer.put(&second).unwrap();
    fixtures::with(fixtures::ENGINE_SMALL, project, &writer.finish())
}
fn with_grants(connector: u8) -> Box<[u8]> {
    let grant = br#""grants": [{"connector": 1, "kind": 2, "pattern": {"segments": ["service"], "last": "open", "value": ""}}]"#;
    let grant = fixtures::with(
        grant,
        br#""connector": 1"#,
        &[b'"', b'c', b'o', b'n', b'n', b'e', b'c', b't', b'o', b'r', b'"', b':', b' ', connector],
    );
    let mut document = fixtures::with(fixtures::ENGINE_SMALL, br#""grants": []"#, &grant);
    for _ in 0_u32..2 {
        document = fixtures::with(&document, br#""grants": []"#, &grant);
    }
    document
}
#[test]
fn the_small_engine_document_parses_whole() {
    let engine = engine();
    assert_eq!(engine.deployment.directory.as_ref(), b"state");
    assert_eq!(engine.deployment.seed, Some(7));
    assert_eq!(engine.providers.len(), 2);
}
#[test]
fn the_cores_config_takes_the_charters_and_projects() {
    let core = core_config(fixtures::ENGINE_SMALL);
    assert_eq!(core.settings.charters.len(), 1);
    assert_eq!(core.projects.as_slice(), &[1]);
    assert_eq!(core.settings.charters[0].policy.instructions.as_ref(), b"Answer the question.");
}
#[test]
fn the_cores_policy_and_permissions_are_seeded_per_project() {
    let core = core_config(&two_projects(b'2'));
    assert_eq!(core.authority.policy(2).unwrap().period_spend, 100);
    assert_eq!(
        core.permission_roles.get(&2).unwrap().as_ref(),
        &[jig_domain_people::PermissionRole { connector: 1, permission: 1, role: 1 }]
    );
}
#[test]
fn the_cores_chat_settings_take_the_provider_charter_and_grace() {
    let core = core_config(fixtures::ENGINE_SMALL);
    assert_eq!(core.settings.deployment_provider, 2);
    assert_eq!(core.settings.chat, 1);
    assert_eq!(core.settings.close_grace, skein_lib::Duration::from_secs(5));
}
#[test]
fn the_cores_limits_and_tools_take_the_profile_and_jigs_numbering() {
    let core = core_config(fixtures::ENGINE_SMALL);
    assert_eq!(core.settings.resume_bytes, 65536);
    assert_eq!(core.hub_turns, 8);
    assert_eq!(core.settings.tools.message, jig_domain_authority::Tools(2));
}
#[test]
fn an_in_process_listener_tells_the_core_workers_share_its_process() {
    assert!(core_config(fixtures::ENGINE_SMALL).workers_in_process);
}
#[test]
fn a_socket_listener_tells_the_core_workers_have_their_own_process() {
    let document =
        fixtures::with(fixtures::ENGINE_SMALL, br#"{"in_process": "workers"}"#, br#"{"socket":"127.0.0.1:7102"}"#);
    assert!(!core_config(&document).workers_in_process);
}
#[test]
fn a_configured_seed_overrides_the_shells_draw() {
    assert_eq!(core_config(fixtures::ENGINE_SMALL).seed, 7);
}
#[test]
fn a_shell_seed_is_used_when_configuration_has_no_seed() {
    let document = fixtures::with(fixtures::ENGINE_SMALL, br#", "seed": 7"#, b"");
    assert_eq!(core_config(&document).seed, 11);
}
#[test]
fn a_project_numbered_twice_is_refused() {
    let problems = fixtures::engine(&two_projects(b'1')).expect_err("duplicate project");
    assert_eq!(problems.problems.get(0).unwrap().what, What::Unique);
}
#[test]
fn an_owner_naming_an_unknown_provider_is_refused() {
    refuses(br#""provider": 1"#, br#""provider": 3"#, What::Refers);
}
#[test]
fn a_message_bound_below_a_partys_words_is_refused() {
    refuses(br#""profile": "small""#, br#""profile": "small", "overrides": {"people.words": 4097}"#, What::Message);
}
#[test]
fn a_retention_member_is_refused_as_unknown() {
    refuses(br#""directory": "state""#, br#""directory": "state", "retention": {}"#, What::Unknown);
}
#[test]
fn a_projects_resources_member_is_refused_as_unknown() {
    refuses(br#""projects": [{"number": 1"#, br#""projects": [{"number": 1, "resources": []"#, What::Unknown);
}
#[test]
fn a_projects_homes_member_is_refused_as_unknown() {
    refuses(br#""projects": [{"number": 1"#, br#""projects": [{"number": 1, "homes": []"#, What::Unknown);
}
#[test]
fn two_deployment_providers_are_refused() {
    refuses(br#""kind": "link""#, br#""kind": "deployment""#, What::Unique);
}
#[test]
fn a_deployment_provider_is_required() {
    refuses(br#""kind": "deployment""#, br#""kind": "application""#, What::Missing);
}
#[test]
fn a_connector_numbered_as_the_cores_own_is_refused() {
    refuses(br#""connectors": [1]"#, br#""connectors": [65535]"#, What::Bound);
}
#[test]
fn each_owner_owns_every_seeded_project() {
    let engine = fixtures::engine(&two_projects(b'2')).unwrap();
    assert_eq!(engine.owners.len(), 2);
    assert_eq!(engine.owners[0].project, 1);
    assert_eq!(engine.owners[1].project, 2);
    assert_eq!(engine.owners[0].identity.subject.as_ref(), b"owner-a");
    assert_eq!(engine.owners[1].identity.subject.as_ref(), b"owner-a");
}
#[test]
fn a_permission_naming_an_unknown_role_is_refused() {
    refuses(br#""permission": 1, "role": 1"#, br#""permission": 1, "role": 9"#, What::Refers);
}
#[test]
fn a_project_policy_past_the_deployment_ceiling_is_refused() {
    refuses(
        br#""escalation_role": 1, "period_spend": 100"#,
        br#""escalation_role": 1, "period_spend": 101"#,
        What::Bound,
    );
}
#[test]
fn charter_instruction_files_are_named_with_the_available_policy_bytes() {
    let engine = engine();
    let files = crate::files(&engine);
    let file = instruction_file(&files);
    assert!(!file.private);
    assert_eq!(file.at, fixtures::path(&[b"charters"]).child(crate::Segment::Index(0)).member(b"instructions"));
    assert!(file.max < engine.limits.core.run_bytes);
}
#[test]
fn a_message_bound_below_a_delegate_result_is_refused() {
    refuses(
        br#""profile": "small""#,
        br#""profile": "small", "overrides": {"tasks.result_bytes": 4097}"#,
        What::Message,
    );
}
#[test]
fn memory_below_the_components_sum_is_refused() {
    let document = fixtures::with(fixtures::ENGINE_SMALL, br#""memory_bytes": 17179869184"#, br#""memory_bytes": 1"#);
    let problems = fixtures::engine(&document).unwrap_err();
    assert_eq!(problems.problems.get(0).unwrap().what, What::Memory { over: 11_900_377_351 });
}
fn instruction_file(files: &[crate::File]) -> &crate::File {
    for file in files {
        if file.path.as_ref() == b"charters/chat.txt" {
            return file;
        }
    }
    panic!("instruction file configured")
}
#[test]
fn resource_grants_read_their_literal_pattern() {
    let engine = fixtures::engine(&with_grants(b'1')).unwrap();
    assert_eq!(engine.rules.ceiling.grants[0].pattern.segments.as_ref(), &[Box::<[u8]>::from(&b"service"[..])]);
    assert_eq!(engine.rules.ceiling.grants[0].pattern.last, jig_domain_authority::Last::Open(Box::default()));
}
#[test]
fn a_grant_naming_an_unknown_connector_is_refused() {
    let problems = fixtures::engine(&with_grants(b'2')).unwrap_err();
    assert_eq!(problems.problems.get(0).unwrap().what, What::Refers);
}
#[test]
fn a_requirement_reads_its_judge_and_freshness() {
    let requirement = br#""requirements": [{"connector": 1,"kind":2,"pattern":{"segments":[],"last":"open","value":""},"judge":{"connector":1,"requirement":3,"parameters":4},"guard":"observed","freshness_ms":5000,"must_be_guarded":false}]"#;
    let document = fixtures::with(fixtures::ENGINE_SMALL, br#""requirements": []"#, requirement);
    let engine = fixtures::engine(&document).unwrap();
    assert_eq!(engine.rules.requirements[0].judge.parameters, 4);
    assert_eq!(
        engine.rules.requirements[0].guard,
        jig_domain_authority::Guard::Observed { freshness: skein_lib::Duration::from_secs(5) }
    );
}
#[test]
fn an_implication_table_not_transitively_closed_is_refused() {
    refuses(
        br#""implies": []"#,
        br#""implies": [{"connector":1,"kind":1,"implies":2},{"connector":1,"kind":2,"implies":3}]"#,
        What::Bound,
    );
}
