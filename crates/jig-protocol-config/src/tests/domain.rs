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
fn operations_below_delegates_plus_one_refuse_the_start() {
    refuses(
        br#""profile": "small""#,
        br#""profile": "small", "overrides": {"people.operations": 4, "tasks.project_goals": 4}"#,
        What::Operations { operations: 4, delegates: 4, goals: 4 },
    );
}

#[test]
fn operations_below_goals_per_project_refuse_the_start() {
    refuses(
        br#""profile": "small""#,
        br#""profile": "small", "overrides": {"people.operations": 7}"#,
        What::Operations { operations: 7, delegates: 4, goals: 8 },
    );
}

#[test]
fn the_shipped_profiles_hold_operations_per_request() {
    for (profile, expected) in [(crate::Profile::Small, (8, 4, 8)), (crate::Profile::Standard, (16, 8, 16))] {
        let limits = crate::profile(profile).core;
        assert_eq!((limits.people.operations, limits.tasks.delegates, limits.tasks.project_goals), expected);
        assert!(limits.people.operations > limits.tasks.delegates, "{profile:?}");
        assert!(limits.people.operations >= limits.tasks.project_goals, "{profile:?}");
    }
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
fn the_cores_settings_take_the_deployment_provider() {
    let core = core_config(fixtures::ENGINE_SMALL);
    assert_eq!(core.settings.deployment_provider, 2);
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
fn an_owner_naming_an_unknown_provider_is_refused() {
    refuses(br#""provider": 1"#, br#""provider": 3"#, What::Refers);
}
fn a_message_bound_below_a_partys_words_is_refused() {
    refuses(br#""profile": "small""#, br#""profile": "small", "overrides": {"people.words": 4097}"#, What::Message);
}
fn a_retention_member_is_refused_as_unknown() {
    refuses(br#""directory": "state""#, br#""directory": "state", "retention": {}"#, What::Unknown);
}
fn a_projects_resources_member_is_refused_as_unknown() {
    refuses(br#""projects": [{"number": 1"#, br#""projects": [{"number": 1, "resources": []"#, What::Unknown);
}
fn a_projects_homes_member_is_refused_as_unknown() {
    refuses(br#""projects": [{"number": 1"#, br#""projects": [{"number": 1, "homes": []"#, What::Unknown);
}
fn two_deployment_providers_are_refused() {
    refuses(br#""kind": "link""#, br#""kind": "deployment""#, What::Unique);
}
fn a_deployment_provider_is_required() {
    refuses(br#""kind": "deployment""#, br#""kind": "application""#, What::Missing);
}
fn a_connector_numbered_as_the_cores_own_is_refused() {
    refuses(br#""connectors": [1]"#, br#""connectors": [65535]"#, What::Bound);
}
#[test]
fn a_ceiling_without_models_takes_every_configured_model() {
    let engine = engine();
    assert_eq!(engine.rules.ceiling.models, jig_domain_authority::Models::ALL);
    assert_eq!(engine.projects[0].policy.ceiling.models, jig_domain_authority::Models::ALL);
}
#[test]
fn a_ceilings_models_name_the_configured_models_it_keeps() {
    let ceiling = br#""ceiling": {"tools": 1023,"#;
    let named = fixtures::engine(&fixtures::with(
        fixtures::ENGINE_SMALL,
        ceiling,
        br#""ceiling": {"tools": 1023, "models": ["small"],"#,
    ));
    assert_eq!(named.unwrap().rules.ceiling.models, jig_domain_authority::Models::Named(0b1));
    let none = fixtures::engine(&fixtures::with(
        fixtures::ENGINE_SMALL,
        ceiling,
        br#""ceiling": {"tools": 1023, "models": [],"#,
    ));
    assert_eq!(none.unwrap().rules.ceiling.models, jig_domain_authority::Models::NONE);
}
fn a_ceiling_naming_an_unknown_model_is_refused() {
    fixtures::refuses(
        br#""ceiling": {"tools": 1023,"#,
        br#""ceiling": {"tools": 1023, "models": ["large"],"#,
        fixtures::path(&[b"rules", b"ceiling", b"models"]).child(crate::Segment::Index(0)),
        What::Refers,
    );
}
fn a_ceiling_naming_a_model_twice_is_refused() {
    fixtures::refuses(
        br#""ceiling": {"tools": 1023,"#,
        br#""ceiling": {"tools": 1023, "models": ["small", "small"],"#,
        fixtures::path(&[b"rules", b"ceiling", b"models"]).child(crate::Segment::Index(1)),
        What::Unique,
    );
}
const ROLE: &[u8] = br#""authority": {"tools": 1023,"#;
const NAMED: &[u8] = br#""authority": {"tools": 1023, "models": "#;
fn role_models(models: &[u8]) -> Result<crate::Engine, crate::Problems> {
    let mut writer = skein_lib::Writer::new(NAMED.len().checked_add(models.len()).unwrap().checked_add(1).unwrap());
    writer.put(NAMED).unwrap();
    writer.put(models).unwrap();
    writer.put(b",").unwrap();
    fixtures::engine(&fixtures::with(fixtures::ENGINE_SMALL, ROLE, &writer.finish()))
}
fn role(engine: &crate::Engine) -> jig_domain_authority::Models {
    engine.projects[0].policy.roles[0].authority.models
}
fn role_models_path(index: usize) -> crate::Path {
    fixtures::path(&[b"projects"])
        .child(crate::Segment::Index(0))
        .member(b"policy")
        .member(b"roles")
        .child(crate::Segment::Index(0))
        .member(b"authority")
        .member(b"models")
        .child(crate::Segment::Index(u32::try_from(index).unwrap()))
}
#[test]
fn a_roles_models_name_the_configured_models_it_keeps() {
    assert_eq!(role(&engine()), jig_domain_authority::Models::ALL, "absent names every configured model");
    assert_eq!(role(&role_models(br#"["small"]"#).unwrap()), jig_domain_authority::Models::Named(0b1));
    assert_eq!(role(&role_models(b"[]").unwrap()), jig_domain_authority::Models::NONE);
}
fn a_role_naming_an_unknown_model_is_refused() {
    let problems = role_models(br#"["large"]"#).unwrap_err();
    assert_eq!(problems.problems.get(0), Some(&crate::Problem { path: role_models_path(0), what: What::Refers }));
}
fn a_role_naming_a_model_twice_is_refused() {
    let problems = role_models(br#"["small", "small"]"#).unwrap_err();
    assert_eq!(problems.problems.get(0), Some(&crate::Problem { path: role_models_path(1), what: What::Unique }));
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
fn a_permission_naming_an_unknown_role_is_refused() {
    refuses(br#""permission": 1, "role": 1"#, br#""permission": 1, "role": 9"#, What::Refers);
}
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
fn a_message_bound_below_a_delegate_result_is_refused() {
    refuses(
        br#""profile": "small""#,
        br#""profile": "small", "overrides": {"tasks.result_bytes": 4097}"#,
        What::Message,
    );
}
#[test]
fn memory_below_the_components_sum_is_refused() {
    let document = fixtures::with(fixtures::ENGINE_SMALL, br#""memory_bytes": 34359738368"#, br#""memory_bytes": 1"#);
    let problems = fixtures::engine(&document).unwrap_err();
    // Worker program sources add 6,723,936 bytes: the hub's and the fleet's source records
    // and pending replacements, their channel and routed output room, and the journal
    // writes and KV operations an extra Sources output at hello needs.
    // Named authority selectors add 14,819,720 bytes over main: 12,632,064 for the
    // 32 answered and 16 pending payload name-context envelopes (48 * 263,168),
    // 526,336 for one routed Event and its people::Request::Route (2 * 263,168),
    // and 1,661,320 for native selector tags, name-list carriers and their derived
    // retained-storage and queue envelopes. People's pending clone is priced there.
    // x1c adds 0 bytes here: Account's alignment absorbs its now-required provider's
    // removed Option tag. Provider registrations and the D2 input are priced by the service.
    // JP10c-b adds 248 bytes over f448c8d4: 240 inline bytes for the parser and
    // two runtime provider copies, plus 8 bytes for their two "Fake" names.
    // This fixture has no manage address; supplied addresses cost three copies.
    // Over 3ca78043, x1d adds 345,296 bytes (people LlmChoice 24 -> 32; Member 360 -> 368;
    // Amendment 320 -> 328; Ask 360 -> 368; Task 1,464 -> 1,504). The pin was already priced.
    // People: 1 * 8 amendment scratch + 48 * (8 * 8) compound-ask payload copies
    // (32 answered + 2 * 8 pending), + 77 * 8 answered B-tree slots (7 nodes * 11),
    // + 8 * 16 pending inline asks = 3,824.
    // Documents: 20 * (56 * 40) = 44,800: one construction + 4 watches + 4 loads
    // + (4 rows + 3) read-answer copies + 4 act scratch; each Project envelope
    // prices (4 lines squared + 8 * 4 + 8) task carriers, each gaining the 40-byte wait.
    // Routed/load payloads: (18,538 writes + 4 loads) * (2 members * 8) = 296,672.
    // 3,824 + 44,800 + 296,672 = 345,296; hub unchanged.
    // JP10c-c keeps the document envelope unchanged (20 * 0); readiness scratch
    // now coexists with deployment's views: 2 * 32 Pending + 2 * 12 ProjectRole
    // + 4 * 8 Coverage + 4 * 4 candidate numbers, plus the largest temporary:
    // max(4 * 32 ranked candidates, (4 * 3) * 64 source Facts,
    //     (2 workers + 8 history + 0 engine) * 40 HostView + 1 * 24 ModelNeed
    //     + 1 * 4096 model-name bytes) = 4520.
    // Delta = 64 + 24 + 32 + 16 + 4520 = 4656; 18_677_344_295 + 4656
    // = 18_677_348_951. Other document scratch paths remain below deployment.
    // y1b adds 114,476 bytes over main's 18_677_348_951:
    // Issued: 3 nodes * 600 + 12 names * 32 = 2,184.
    // Pending operations: 3 nodes * 512 + 12 sources * 9 hosts * 8,
    // plus 9 terminal rows * 8 = 2,472.
    // Removal unions: 4 accounts * 352 + 11 B-tree slots * 32 inline growth = 1,760.
    // Fleet: 2 channel slots * 72 + 2 workers * 3 pending nodes * 176 growth,
    // + 2 * 3 check nodes * 480 + 2 * 3 result nodes * 744
    // + 2 * 12 result details * 4,096 = 106,848.
    // One extra scratch list + cleanup names: 2 * 12 * 33 = 792;
    // target snapshot: (8 configured workers + 1 engine) * 4 = 36;
    // configured names: 8 * (16 carrier + 32 bytes) = 384.
    // 2,184 + 2,472 + 1,760 + 106,848 + 792 + 36 + 384 = 114,476.
    // Resuming one worker's deferred source commands preserves the small profile's
    // existing output/journal/KV envelopes; the engine has no program backend.
    // Facts add 416: 2 lost-history nodes * 11 slots * 8 bytes,
    // plus 3 host-view copies * (2 connected + 8 history rows) * 8 bytes.
    // Readiness's max(128 ranking, 768 facts, 4520 hosts/model) moves to
    // max(128, 768, 4600): 10 HostViews * (48 - 40) = 80 of those bytes.
    // Program documents have their own explicit bound but remain below the
    // pre-existing largest document envelope; routed and journal caps are unchanged.
    // Total y1b delta: 114,476 + 416 = 114,892 over main.
    assert_eq!(problems.problems.get(0).unwrap().what, What::Memory { over: 18_677_463_843 });
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
fn an_implication_table_not_transitively_closed_is_refused() {
    refuses(
        br#""implies": []"#,
        br#""implies": [{"connector":1,"kind":1,"implies":2},{"connector":1,"kind":2,"implies":3}]"#,
        What::Bound,
    );
}

#[test]
fn domain_configuration_problems_name_the_invalid_fields() {
    table([
        ("an_owner_naming_an_unknown_provider_is_refused", an_owner_naming_an_unknown_provider_is_refused),
        ("a_message_bound_below_a_partys_words_is_refused", a_message_bound_below_a_partys_words_is_refused),
        ("a_retention_member_is_refused_as_unknown", a_retention_member_is_refused_as_unknown),
        ("a_projects_resources_member_is_refused_as_unknown", a_projects_resources_member_is_refused_as_unknown),
        ("a_projects_homes_member_is_refused_as_unknown", a_projects_homes_member_is_refused_as_unknown),
        ("two_deployment_providers_are_refused", two_deployment_providers_are_refused),
        ("a_deployment_provider_is_required", a_deployment_provider_is_required),
        ("a_connector_numbered_as_the_cores_own_is_refused", a_connector_numbered_as_the_cores_own_is_refused),
        ("a_permission_naming_an_unknown_role_is_refused", a_permission_naming_an_unknown_role_is_refused),
        (
            "a_project_policy_past_the_deployment_ceiling_is_refused",
            a_project_policy_past_the_deployment_ceiling_is_refused,
        ),
        ("a_message_bound_below_a_delegate_result_is_refused", a_message_bound_below_a_delegate_result_is_refused),
        (
            "an_implication_table_not_transitively_closed_is_refused",
            an_implication_table_not_transitively_closed_is_refused,
        ),
        ("a_ceiling_naming_an_unknown_model_is_refused", a_ceiling_naming_an_unknown_model_is_refused),
        ("a_ceiling_naming_a_model_twice_is_refused", a_ceiling_naming_a_model_twice_is_refused),
        ("a_role_naming_an_unknown_model_is_refused", a_role_naming_an_unknown_model_is_refused),
        ("a_role_naming_a_model_twice_is_refused", a_role_naming_a_model_twice_is_refused),
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

#[test]
fn the_catalogue_key_bound_follows_the_component() {
    let mut engine = engine();
    engine.limits.driver.client.document.token_bytes = 123;
    let core = crate::core(&engine, [0; 16], 1);
    assert_eq!(core.settings.key_bytes, engine.limits.driver.client.document.token_bytes);
}
#[test]
fn the_catalogue_key_bound_respects_the_carried_value_limit() {
    let engine = engine();
    let core = crate::core(&engine, [0; 16], 1);
    assert_eq!(core.settings.key_bytes, engine.limits.clients.wire.keyed_value_some);
}
