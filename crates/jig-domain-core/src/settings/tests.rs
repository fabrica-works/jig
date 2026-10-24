use super::*;
use crate::tests::{config, limits};
use jig_domain_tasks as tasks;
use skein_lib::Duration;

fn template() -> Template {
    Template {
        number: 1,
        name: Box::from(b"chat".as_slice()),
        executor: people::TemplateExecutor::Charter(1),
        contract: tasks::Contract::Report { words: 1 },
        shallower: 0,
        budget: people::TemplateBudget::Given,
        deadline: people::TemplateDeadline::None,
        tracked: None,
        flags: tasks::Flags::PLAIN,
        label: 0,
        grace: None,
        llm: None,
    }
}
fn charter(current: &mut Current) -> &mut Charter {
    match &mut current.defaults.get_mut(&Setting::Charter(1)).unwrap().value {
        Value::Charter(value) => value,
        Value::Template(_) | Value::Policy(_) | Value::Rules(_) => unreachable!(),
    }
}
#[test]
fn lookups_apply_sets_removals_additions_and_resets_without_a_third_set() {
    let mut current = Current::new(config(1).defaults, &limits());
    let original = current.charter(1).unwrap().clone();
    let mut changed = original.clone();
    changed.spend += 1;
    current
        .changes
        .insert(Setting::Charter(1), Changed { value: Some(Value::Charter(Box::new(changed.clone()))), revision: 2 })
        .unwrap();
    assert_eq!(current.charter(1), Some(&changed));
    current.changes.insert(Setting::Charter(1), Changed { value: None, revision: 3 }).unwrap();
    assert!(current.charter(1).is_none());
    let mut added = original.clone();
    added.number = 2;
    current
        .changes
        .insert(Setting::Charter(2), Changed { value: Some(Value::Charter(Box::new(added.clone()))), revision: 4 })
        .unwrap();
    assert_eq!(current.charter(2), Some(&added));
    current.changes.remove(&Setting::Charter(1));
    assert_eq!(current.charter(1), Some(&original));
    assert_eq!(current.offered().as_slice(), &[Setting::Charter(1), Setting::Charter(2), Setting::Rules]);
}
fn fault_case(fault: Fault) {
    let mut config = config(1);
    let limits = limits();
    let mut policy = config.seed_projects[0].policy.clone();
    config.defaults.templates = Box::new([template()]);
    let mut current = Current::new(config.defaults, &limits);
    let setting = match fault {
        Fault::Shape => {
            let count = limits.run_bytes.checked_add(1).expect("one past the run bound");
            let mut bytes = List::with_capacity(count);
            for _ in 0..count {
                bytes.push(b'x').expect("instruction byte count");
            }
            charter(&mut current).policy.instructions = bytes.into_boxed();
            Setting::Charter(1)
        }
        Fault::Count => {
            let original = current.charter(1).unwrap().clone();
            let extra = limits.tasks.charters.checked_add(1).expect("one past the charter bound");
            for number in 2..=extra {
                let mut value = original.clone();
                value.number = number;
                current
                    .defaults
                    .insert(Setting::Charter(number), Default { value: Value::Charter(Box::new(value)), revision: 1 })
                    .unwrap();
            }
            Setting::Charter(extra)
        }
        Fault::Reference => {
            let mut value = template();
            value.executor = people::TemplateExecutor::Charter(999);
            current
                .changes
                .insert(Setting::Template(1), Changed { value: Some(Value::Template(Box::new(value))), revision: 2 })
                .unwrap();
            Setting::Template(1)
        }
        Fault::Rules => {
            policy.policy.period_spend =
                current.rules().period_spend.checked_add(1).expect("one past the policy period bound");
            current
                .changes
                .insert(Setting::Policy(1), Changed { value: Some(Value::Policy(Box::new(policy))), revision: 2 })
                .unwrap();
            Setting::Policy(1)
        }
        Fault::Budget => {
            let value = charter(&mut current);
            value.policy.turns = value.policy.reserve_turns;
            Setting::Charter(1)
        }
        Fault::Grace => {
            let mut value = template();
            value.grace = Some(Duration::from_secs(86_400));
            current
                .changes
                .insert(Setting::Template(1), Changed { value: Some(Value::Template(Box::new(value))), revision: 2 })
                .unwrap();
            Setting::Template(1)
        }
        Fault::Owner => {
            policy.policy.roles[0].requests = authority::Requests(authority::Requests::ALL.0 & !1);
            current
                .changes
                .insert(Setting::Policy(1), Changed { value: Some(Value::Policy(Box::new(policy))), revision: 2 })
                .unwrap();
            Setting::Policy(1)
        }
    };
    assert_eq!(
        check(&current, &config.settings, &config.connectors, &limits),
        Err(Failure { setting, fault }),
        "{fault:?}"
    );
}

#[test]
fn the_whole_check_names_shape_and_its_setting() {
    fault_case(Fault::Shape);
}
#[test]
fn the_whole_check_names_count_and_its_setting() {
    fault_case(Fault::Count);
}
#[test]
fn the_whole_check_names_reference_and_its_setting() {
    fault_case(Fault::Reference);
}
#[test]
fn the_whole_check_names_rules_and_its_setting() {
    fault_case(Fault::Rules);
}
#[test]
fn the_whole_check_names_budget_and_its_setting() {
    fault_case(Fault::Budget);
}
#[test]
fn the_whole_check_names_grace_and_its_setting() {
    fault_case(Fault::Grace);
}
#[test]
fn the_whole_check_names_owner_and_its_setting() {
    fault_case(Fault::Owner);
}

#[test]
fn pruning_a_lowered_history_limit_uses_only_the_commits_remaining_room() {
    let (writes, oldest) = revision_writes(None, List::with_capacity(0), 1, 20, 2, 4);
    assert_eq!(oldest, 4);
    assert_eq!(writes.len(), 4);
    let crate::Write::Save(crate::Record::Core(crate::CoreRecord::Settings(Record::Extent { oldest: stored }))) =
        &writes.as_slice()[3]
    else {
        panic!("the extent closes the pruning commit")
    };
    assert_eq!(*stored, 4);
    let (writes, oldest) = revision_writes(None, List::with_capacity(0), oldest, 20, 2, 32);
    assert_eq!(oldest, 18);
    assert_eq!(writes.len(), 15);
    let (writes, oldest) = revision_writes(None, List::with_capacity(0), oldest, 20, 2, 32);
    assert!(writes.is_empty());
    assert_eq!(oldest, 18);
}

#[test]
fn the_whole_check_refuses_unconfigured_model_bits() {
    let config = config(1);
    let mut current = Current::new(config.defaults, &limits());
    let mut rules = current.rules().clone();
    rules.ceiling.models = authority::Models::Named(2);
    current
        .changes
        .insert(Setting::Rules, Changed { value: Some(Value::Rules(Box::new(rules))), revision: 2 })
        .unwrap();
    assert_eq!(
        check(&current, &config.settings, &config.connectors, &limits()),
        Err(Failure { setting: Setting::Rules, fault: Fault::Reference })
    );
}
#[test]
fn the_whole_check_refuses_unconfigured_contract_connectors() {
    let config = config(1);
    let mut current = Current::new(config.defaults, &limits());
    let mut value = template();
    value.contract = tasks::Contract::Change { connector: u16::MAX, kind: 1, words: 1 };
    current
        .changes
        .insert(Setting::Template(1), Changed { value: Some(Value::Template(Box::new(value))), revision: 2 })
        .unwrap();
    assert_eq!(
        check(&current, &config.settings, &config.connectors, &limits()),
        Err(Failure { setting: Setting::Template(1), fault: Fault::Reference })
    );
}
