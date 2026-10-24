//! Settings travel through the real restart, store and commit barrier (engine.md, 17.7).
use jig_core_world::{client_peer::ClientPeer, effects::World, fixtures, peers};
use jig_domain_core::{self as core, documents, settings};
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Token};

#[path = "settings/support.rs"]
mod support;
use support::*;

#[test]
fn the_document_bridge_starts_once_and_preserves_templates_policies_rules_and_models() {
    let mut world = start(64);
    assert_eq!(revision(&world, 1, false).head.by, settings::Decider::FirstStart);
    assert!(revision(&world, 1, false).items.is_empty());
    let signed = sign_in(&mut world);
    let before = frame(&mut world, signed);
    assert_eq!(
        before.projects.get(0).expect("settings story fixture").templates.as_slice(),
        configuration().0.core.defaults.templates.as_ref()
    );
    let policy = configuration().0.core.defaults.policies.get(&1).expect("settings story fixture").policy.clone();
    let reply = ask(&mut world, signed, 1, people::Ask::MakeProject { name: Box::from("Made"), policy: 1 });
    assert!(matches!(reply, people::Reply::Outcome(people::Outcome::ProjectMade { .. })), "{reply:?}");
    let root::Record::Core(core::Record::People(people::Stored::Policy { value, .. })) =
        &world.store.rows[&root::Key::Core(core::Key::People(people::Key::Policy(2)))]
    else {
        panic!("copied project policy")
    };
    assert_eq!(value.ceiling.spend, policy.ceiling.budget.spend);
    assert_eq!(world.domain.settings().rules(), &configuration().0.core.defaults.rules);
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: fleet::Hello {
            bounds: fixtures::bounds(),
            stop_bound: Duration::from_millis(1),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
    let chat = fixtures::chat(&world, 1, Box::from(b"hello".as_slice()));
    let reply = ask(&mut world, signed, 2, chat);
    assert!(matches!(reply, people::Reply::Outcome(people::Outcome::Created { .. })), "{reply:?}");
    let assignment = world.assigned.as_ref().expect("document charter runs");
    assert_eq!(assignment.run.policy.model.name.as_ref(), b"scripted");
    assert_eq!(assignment.run.policy.instructions.as_ref(), b"Answer within the granted authority.");
    restart(&mut world, configuration().0);
    assert_eq!(world.domain.settings().newest(), 1);
    let signed = sign_in(&mut world);
    assert_eq!(
        frame(&mut world, signed).projects.get(0).expect("settings story fixture").templates,
        before.projects.get(0).expect("settings story fixture").templates
    );
}
#[test]
fn a_release_changes_the_template_once_and_archives_its_predecessor() {
    let mut world = start(64);
    let mut config = configuration().0;
    config.core.defaults.templates[0].grace = Some(Duration::from_millis(6000));
    restart(&mut world, config);
    let newest = revision(&world, 2, false);
    assert_eq!(newest.head.by, settings::Decider::Release);
    assert_eq!(newest.head.against, 1);
    assert!(
        matches!(&newest.items[..], [settings::Item { setting: settings::Setting::Template(1), edit: settings::Edit::Set(settings::Value::Template(value)) }] if value.grace == Some(Duration::from_millis(6000)))
    );
    assert_eq!(revision(&world, 1, true).head.by, settings::Decider::FirstStart);
    let signed = sign_in(&mut world);
    assert_eq!(
        frame(&mut world, signed)
            .projects
            .get(0)
            .expect("settings story fixture")
            .templates
            .get(0)
            .expect("settings story fixture")
            .grace,
        Some(Duration::from_millis(6000))
    );
    let mut config = configuration().0;
    config.core.defaults.templates[0].grace = Some(Duration::from_millis(6000));
    restart(&mut world, config);
    assert_eq!(world.domain.settings().newest(), 2);
}
#[test]
fn a_stored_change_wins_until_reset_and_a_dropped_default_is_no_longer_offered() {
    let mut world = start(64);
    let mut template = configuration().0.core.defaults.templates[0].clone();
    template.grace = Some(Duration::from_millis(7000));
    world.seed(Box::new([record(settings::Record::Changed {
        setting: settings::Setting::Template(1),
        changed: settings::Changed { revision: 1, value: Some(settings::Value::Template(Box::new(template))) },
    })]));
    let mut config = configuration().0;
    config.core.defaults.templates[0].grace = Some(Duration::from_millis(6000));
    restart(&mut world, config);
    assert_eq!(world.domain.settings().newest(), 1);
    assert_eq!(
        world.domain.settings().template(1).expect("settings story fixture").grace,
        Some(Duration::from_millis(7000))
    );
    world.erase(key(settings::Key::Changed(settings::Setting::Template(1))));
    let mut config = configuration().0;
    config.core.defaults.templates[0].grace = Some(Duration::from_millis(6000));
    restart(&mut world, config);
    assert_eq!(world.domain.settings().newest(), 1);
    assert_eq!(
        world.domain.settings().template(1).expect("settings story fixture").grace,
        Some(Duration::from_millis(6000))
    );
    let mut config = configuration().0;
    config.core.defaults.templates = Box::new([config.core.defaults.templates[0].clone()]);
    restart(&mut world, config);
    let signed = sign_in(&mut world);
    assert_eq!(frame(&mut world, signed).projects.get(0).expect("settings story fixture").templates.len(), 1);
}
#[test]
fn history_keeps_two_predecessors_after_three_releases() {
    let mut world = start(2);
    for grace in 6..=8 {
        let mut config = configuration().0;
        config.core.defaults.templates[0].grace = Some(Duration::from_secs(grace));
        restart(&mut world, config);
    }
    assert_eq!(world.domain.settings().newest(), 4);
    assert_eq!(world.domain.settings().oldest(), 2);
    assert!(!world.store.rows.contains_key(&key(settings::Key::History(1))));
    for number in 2..=3 {
        assert_eq!(revision(&world, number, true).head.number, number);
    }
    assert_eq!(revision(&world, 4, false).head.number, 4);
}
#[test]
fn a_stored_charter_with_an_unconfigured_model_refuses_the_start_naming_the_change() {
    let mut world = start(64);
    let mut charter = configuration().0.core.defaults.charters[0].clone();
    charter.policy.model.name = Box::from(b"no-longer-configured".as_slice());
    world.seed(Box::new([record(settings::Record::Changed {
        setting: settings::Setting::Charter(1),
        changed: settings::Changed { revision: 1, value: Some(settings::Value::Charter(Box::new(charter))) },
    })]));
    world.restart_with(configuration().0);
    assert_eq!(
        world.domain.settings_failure(),
        Some(core::RestartRequest::Settings {
            failure: settings::Failure { setting: settings::Setting::Charter(1), fault: settings::Fault::Reference },
            changed: true
        })
    );
    assert!(!world.domain.ready());
    assert!(world.stopped);
}

#[test]
fn a_live_agent_task_on_a_dropped_charter_refuses_the_start_before_a_release_commits() {
    let mut world = start(64);
    let signed = sign_in(&mut world);
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    let chat = fixtures::chat(&world, 1, Box::from(b"still live".as_slice()));
    let reply = ask(&mut world, signed, 3, chat);
    assert!(matches!(reply, people::Reply::Outcome(people::Outcome::Created { .. })), "{reply:?}");
    let mut config = configuration().0;
    config.core.defaults.charters = Box::new([]);
    config.core.defaults.templates = Box::new([]);
    config.core.defaults.policies = skein_lib::Map::with_capacity(0);
    config.core.defaults.rules.ceiling.delegation.kinds = Box::new([]);
    world.restart_with(config);
    assert_eq!(
        world.domain.settings_failure(),
        Some(core::RestartRequest::Settings {
            failure: settings::Failure { setting: settings::Setting::Charter(1), fault: settings::Fault::Reference },
            changed: false
        })
    );
    assert!(world.stopped);
    assert_eq!(revision(&world, 1, false).head.number, 1, "refused release was never committed");
    assert!(!world.store.rows.contains_key(&key(settings::Key::Held(2))));
}

#[test]
fn changes_past_their_kinds_bound_refuse_before_the_overflow_value_is_retained() {
    let mut world = start(64);
    let config = configuration().0;
    let bound = configuration().1.core.tasks.charters;
    let extra = bound.checked_add(1).expect("one past the charter count");
    let mut rows = skein_lib::List::with_capacity(extra);
    for number in 1..=extra {
        let mut charter = config.core.defaults.charters[0].clone();
        charter.number = number;
        rows.push(record(settings::Record::Changed {
            setting: settings::Setting::Charter(number),
            changed: settings::Changed { revision: 1, value: Some(settings::Value::Charter(Box::new(charter))) },
        }))
        .expect("bounded fixture rows");
    }
    world.seed(rows.into_boxed());
    world.restart_with(config);
    assert!(!world.domain.ready());
    assert!(!world.domain.settings().changed(settings::Setting::Charter(extra)));
    assert_eq!(
        world.domain.settings_failure(),
        Some(core::RestartRequest::Settings {
            failure: settings::Failure { setting: settings::Setting::Charter(extra), fault: settings::Fault::Count },
            changed: true
        })
    );
    assert_eq!(revision(&world, 1, false).head.number, 1);
}

#[test]
fn an_owner_changes_a_template_once_me_serves_it_and_restart_restores_revision_two() {
    let mut world = start(64);
    let signed = sign_in(&mut world);
    let item = template_edit(&world, 7);
    changed(&change(&mut world, signed, 41, 1, Box::new([item.clone()])), 2);
    let me = frame(&mut world, signed);
    assert_eq!(
        me.projects.get(0).expect("settings fixture").templates.get(0).expect("settings fixture").grace,
        Some(Duration::from_secs(7))
    );
    let head = &revision(&world, 2, false).head;
    assert_eq!(head.by, settings::Decider::Party(OWNER));
    assert_eq!(head.against, 1);
    assert!(!head.moved);
    assert!(head.proposal.is_none());
    let before = world.store.rows.clone();
    changed(&change(&mut world, signed, 41, 1, Box::new([item])), 2);
    assert_eq!(world.store.rows, before, "a saved key never makes a second revision");
    restart(&mut world, configuration().0);
    assert_eq!(world.domain.settings().newest(), 2);
    let signed = sign_in(&mut world);
    assert_eq!(
        frame(&mut world, signed)
            .projects
            .get(0)
            .expect("settings fixture")
            .templates
            .get(0)
            .expect("settings fixture")
            .grace,
        Some(Duration::from_secs(7))
    );
}

#[test]
fn a_changed_future_policy_is_copied_only_by_projects_made_after_it() {
    let mut world = start(64);
    let signed = sign_in(&mut world);
    let old = world.domain.settings().policy(1).expect("settings fixture").clone();
    let mut next = old.clone();
    next.policy.period_spend -= 1;
    for role in &mut next.policy.roles {
        role.period_spend = role.period_spend.min(next.policy.period_spend);
    }
    changed(
        &change(
            &mut world,
            signed,
            41,
            1,
            Box::new([settings::Item {
                setting: settings::Setting::Policy(1),
                edit: settings::Edit::Set(settings::Value::Policy(Box::new(next.clone()))),
            }]),
        ),
        2,
    );
    assert!(matches!(
        ask(&mut world, signed, 42, people::Ask::MakeProject { name: "After".into(), policy: 1 }),
        people::Reply::Outcome(people::Outcome::ProjectMade { project: 2 })
    ));
    for (project, expected) in [(1, old.policy.period_spend), (2, next.policy.period_spend)] {
        let root::Record::Core(core::Record::People(people::Stored::Policy { value, .. })) =
            &world.store.rows[&root::Key::Core(core::Key::People(people::Key::Policy(project)))]
        else {
            panic!("policy record")
        };
        assert_eq!(value.period_spend, expected);
    }
}

#[test]
fn a_narrowed_run_rule_refuses_the_next_oversized_creation_and_keeps_the_old_project_policy() {
    let mut world = start(64);
    let signed = sign_in(&mut world);
    let policy = world.store.rows[&root::Key::Core(core::Key::People(people::Key::Policy(1)))].clone();
    let mut rules = world.domain.settings().rules().clone();
    rules.maximum_run_spend = 10;
    rules.ceiling.budget.spend = 10;
    let mut future = world.domain.settings().policy(1).expect("future policy").clone();
    future.policy.ceiling.budget.spend = 10;
    for role in &mut future.policy.roles {
        role.authority.budget.spend = role.authority.budget.spend.min(10);
    }
    // Run spend is a per-run cap. The authority ceiling
    // separately makes a new task's requested allotment of twenty inadmissible.
    changed(
        &change(
            &mut world,
            signed,
            41,
            1,
            Box::new([
                settings::Item {
                    setting: settings::Setting::Rules,
                    edit: settings::Edit::Set(settings::Value::Rules(Box::new(rules))),
                },
                settings::Item {
                    setting: settings::Setting::Policy(1),
                    edit: settings::Edit::Set(settings::Value::Policy(Box::new(future))),
                },
            ]),
        ),
        2,
    );
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    let mut request = fixtures::chat(&world, 1, Box::from(b"over the rule".as_slice()));
    let people::Ask::Create { batch, .. } = &mut request else { panic!("chat create") };
    batch[0].budget = people::Budget::Named;
    batch[0].authority.spend = 20;
    let answer = ask(&mut world, signed, 42, request);
    assert!(
        matches!(answer, people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Authority { .. }))),
        "{answer:?}"
    );
    assert_eq!(world.store.rows[&root::Key::Core(core::Key::People(people::Key::Policy(1)))], policy);
    restart(&mut world, configuration().0);
    assert_eq!(world.domain.settings().rules().maximum_run_spend, 10);
    assert_eq!(world.store.rows[&root::Key::Core(core::Key::People(people::Key::Policy(1)))], policy);
}

#[test]
fn changed_charter_instructions_and_effort_prepare_the_next_new_task() {
    let (mut config, limits) = configuration();
    config.core.settings.models[0].efforts = Box::new([Box::from(b"low".as_slice()), Box::from(b"high".as_slice())]);
    config.core.settings.models[0].effort = Some(Box::from(b"low".as_slice()));
    config.core.defaults.charters[0].policy.model = config.core.settings.models[0].clone();
    let mut world = World::first_start_with_accounts(config, &limits);
    world.client_peer = Some(ClientPeer::default());
    let signed = sign_in(&mut world);
    let mut charter = world.domain.settings().charter(1).expect("settings fixture").clone();
    charter.policy.instructions = Box::from(b"Changed instructions.".as_slice());
    charter.policy.model.effort = Some(Box::from(b"high".as_slice()));
    // A selected effort is a charter default; the model's declared efforts stay configured.
    changed(
        &change(
            &mut world,
            signed,
            41,
            1,
            Box::new([settings::Item {
                setting: settings::Setting::Charter(1),
                edit: settings::Edit::Set(settings::Value::Charter(Box::new(charter))),
            }]),
        ),
        2,
    );
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: fleet::Hello {
            bounds: fixtures::bounds(),
            stop_bound: Duration::from_millis(1),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
    let chat = fixtures::chat(&world, 1, Box::from(b"new task".as_slice()));
    assert!(matches!(ask(&mut world, signed, 42, chat), people::Reply::Outcome(people::Outcome::Created { .. })));
    let assigned = world.assigned.as_ref().expect("next run");
    assert_eq!(assigned.run.policy.instructions.as_ref(), b"Changed instructions.");
    assert_eq!(assigned.run.policy.model.effort.as_deref(), Some(b"high".as_slice()));
}

#[test]
fn refused_changes_preserve_all_settings_rows_and_the_newest() {
    let mut world = start(64);
    let signed = sign_in(&mut world);
    let mut invalid = world.domain.settings().template(1).expect("settings fixture").clone();
    invalid.executor = people::TemplateExecutor::Charter(999);
    let cases: [(u32, Box<[settings::Item]>, people::Refusal); 3] = [
        (0, Box::new([template_edit(&world, 7)]), people::Refusal::Stale { newest: 1 }),
        (
            1,
            Box::<[settings::Item]>::default(),
            people::Refusal::NotWellFormed { at: 0, why: people::Malformed::SettingsUnchanged },
        ),
        (
            1,
            Box::new([settings::Item {
                setting: settings::Setting::Template(1),
                edit: settings::Edit::Set(settings::Value::Template(Box::new(invalid))),
            }]),
            people::Refusal::NotWellFormed {
                at: 0,
                why: people::Malformed::Settings {
                    setting: settings::Setting::Template(1),
                    fault: settings::Fault::Reference,
                },
            },
        ),
    ];
    for (at, (against, items, refusal)) in cases.into_iter().enumerate() {
        let before = settings_rows(&world);
        assert_eq!(
            change(&mut world, signed, u8::try_from(at + 41).expect("settings fixture"), against, items),
            people::Reply::Outcome(people::Outcome::Refused(refusal))
        );
        assert_eq!(world.domain.settings().newest(), 1);
        let after = settings_rows(&world);
        assert_eq!(before, after);
    }
}

#[test]
fn a_live_tasks_charter_removal_is_full_and_commits_no_settings() {
    let mut world = start(64);
    let signed = sign_in(&mut world);
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    let chat = fixtures::chat(&world, 1, Box::from(b"live".as_slice()));
    let people::Reply::Outcome(people::Outcome::Created { first, .. }) = ask(&mut world, signed, 40, chat) else {
        panic!("live agent")
    };
    let mut rules = world.domain.settings().rules().clone();
    rules.ceiling.delegation.kinds = Box::new([]);
    let before_settings = settings_rows(&world);
    let items = Box::new([
        settings::Item { setting: settings::Setting::Charter(1), edit: settings::Edit::Removed },
        settings::Item { setting: settings::Setting::Template(1), edit: settings::Edit::Removed },
        settings::Item { setting: settings::Setting::Policy(1), edit: settings::Edit::Removed },
        settings::Item {
            setting: settings::Setting::Rules,
            edit: settings::Edit::Set(settings::Value::Rules(Box::new(rules))),
        },
    ]);
    assert_eq!(
        change(&mut world, signed, 41, 1, items),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Full {
            revision: 1,
            tasks: Box::new([first]),
            more: 0
        }))
    );
    assert_eq!(world.domain.settings().newest(), 1);
    assert!(!world.store.rows.contains_key(&key(settings::Key::Held(2))));
    assert!(world.domain.settings().charter(1).is_some());
    assert_eq!(settings_rows(&world), before_settings);
}

#[test]
fn four_owner_changes_keep_two_history_records_beside_revision_five() {
    let mut world = start(2);
    let signed = sign_in(&mut world);
    for newest in 1..=4 {
        let item = template_edit(&world, u64::from(newest + 5));
        changed(
            &change(&mut world, signed, u8::try_from(newest + 40).expect("settings fixture"), newest, Box::new([item])),
            newest + 1,
        );
    }
    assert_eq!(world.domain.settings().newest(), 5);
    assert_eq!(world.domain.settings().oldest(), 3);
    for number in 1..=2 {
        assert!(!world.store.rows.contains_key(&key(settings::Key::History(number))));
    }
    for number in 3..=4 {
        assert_eq!(revision(&world, number, true).head.number, number);
    }
    assert_eq!(revision(&world, 5, false).head.number, 5);
}

#[test]
fn a_member_cannot_change_settings_and_the_key_keeps_that_refusal() {
    let mut world = start(64);
    let signed = sign_in(&mut world);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: people::DEPLOYMENT,
        holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Member }]),
    })));
    let item = template_edit(&world, 7);
    let answer = change(&mut world, signed, 41, 1, Box::new([item.clone()]));
    assert!(matches!(
        answer,
        people::Reply::Refused(people::Refusal::Role { held: Some(people::Role::Member) })
            | people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role {
                held: Some(people::Role::Member)
            }))
    ));
    assert_eq!(world.domain.settings().newest(), 1);
    assert!(!world.store.rows.contains_key(&key(settings::Key::Changed(settings::Setting::Template(1)))));
    assert_eq!(change(&mut world, signed, 41, 1, Box::new([item])), answer);
}

#[test]
fn a_settings_answer_and_the_new_me_watch_wait_together_for_durability() {
    let mut world = start(64);
    let signed = sign_in(&mut world);
    let watcher = Token::new(45);
    world.send(root::Event::Core(core::Event::Watch {
        watcher,
        sign_in: signed,
        subject: documents::Watched::Frame,
        window: documents::Window { rows: 8, after: None },
    }));
    let peer = world.client_peer.as_mut().expect("client");
    assert!(matches!(peer.delivered.pop(), Some((_, root::Delivery::Core(core::Held::Delivery { .. })))));
    world.send(root::Event::Core(core::Event::Delivered { watcher }));
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    let item = template_edit(&world, 7);
    world.send(root::Event::Core(core::Event::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        key: [41; 16],
        digest: [41; 32],
        change: Box::new(settings::Change { against: 1, items: Box::new([item]), running: settings::Running::Moves }),
    }));
    assert!(world.client_peer.as_ref().expect("client").delivered.is_empty());
    world.release_commits();
    let peer = world.client_peer.as_ref().expect("client");
    assert_eq!(peer.delivered.len(), 2);
    for (commit, delivery) in &peer.delivered {
        assert!(*commit <= world.store.applied);
        if let root::Delivery::Core(core::Held::PeopleReply { reply, .. }) = delivery {
            changed(reply, 2);
        } else if let root::Delivery::Core(core::Held::Delivery { document, .. }) = delivery {
            let documents::Document::Frame(frame) = document.as_ref() else { panic!("me watch") };
            assert_eq!(
                frame.projects.get(0).expect("project").templates.get(0).expect("template").grace,
                Some(Duration::from_secs(7))
            );
        } else {
            panic!("settings publication {delivery:?}");
        }
    }
    assert!(revision(&world, 2, false).head.moved);
}

#[test]
fn a_live_llm_choice_outside_new_charter_models_is_full_and_names_only_the_bounded_first_tasks() {
    let (mut config, mut limits) = configuration();
    let mut other = config.core.settings.models[0].clone();
    other.name = Box::from(b"other".as_slice());
    config.core.settings.models = Box::new([config.core.settings.models[0].clone(), other.clone()]);
    config.core.defaults.charters[0].policy.alternatives = Box::new([other.clone()]);
    limits.core.tasks.circle = 1;
    peers::journal(&mut limits);
    let mut world = World::first_start_with_accounts(config, &limits);
    world.client_peer = Some(ClientPeer::default());
    let signed = sign_in(&mut world);
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    let mut made = Vec::new();
    for key in [39, 40] {
        let mut chat = fixtures::chat(&world, 1, Box::from(b"model held".as_slice()));
        let people::Ask::Create { batch, .. } = &mut chat else { panic!("agent batch") };
        batch[0].budget = people::Budget::Named;
        batch[0].authority.spend = 20;
        let people::Reply::Outcome(people::Outcome::Created { first, .. }) = ask(&mut world, signed, key, chat) else {
            panic!("agent task")
        };
        made.push(first);
    }
    let mut charter = world.domain.settings().charter(1).expect("charter").clone();
    charter.policy.model = other;
    charter.policy.alternatives = Box::new([]);
    let before_settings = settings_rows(&world);
    let before = revision(&world, 1, false).clone();
    assert_eq!(
        change(
            &mut world,
            signed,
            41,
            1,
            Box::new([settings::Item {
                setting: settings::Setting::Charter(1),
                edit: settings::Edit::Set(settings::Value::Charter(Box::new(charter)))
            }])
        ),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Full {
            revision: 1,
            tasks: Box::new([made[0]]),
            more: 1
        }))
    );
    assert_eq!(world.domain.settings().newest(), 1);
    assert_eq!(revision(&world, 1, false), &before);
    assert_eq!(settings_rows(&world), before_settings);
    assert!(!world.store.rows.contains_key(&key(settings::Key::Changed(settings::Setting::Charter(1)))));
}

#[test]
fn a_changed_run_cap_applies_to_the_next_run_without_capping_its_tasks_total_allotment() {
    let mut world = start(64);
    let signed = sign_in(&mut world);
    let mut rules = world.domain.settings().rules().clone();
    rules.maximum_run_spend = 20;
    changed(
        &change(
            &mut world,
            signed,
            41,
            1,
            Box::new([settings::Item {
                setting: settings::Setting::Rules,
                edit: settings::Edit::Set(settings::Value::Rules(Box::new(rules))),
            }]),
        ),
        2,
    );
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: fleet::Hello {
            bounds: fixtures::bounds(),
            stop_bound: Duration::from_millis(1),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
    let mut chat = fixtures::chat(&world, 1, Box::from(b"forty across runs".as_slice()));
    let people::Ask::Create { batch, .. } = &mut chat else { panic!("chat") };
    batch[0].budget = people::Budget::Named;
    batch[0].authority.spend = 40;
    assert!(matches!(ask(&mut world, signed, 42, chat), people::Reply::Outcome(people::Outcome::Created { .. })));
    assert_eq!(world.assigned.as_ref().expect("new run").run.budget, 20);
}
