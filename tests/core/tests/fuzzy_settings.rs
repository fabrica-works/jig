//! Random settings decisions agree with their durable rows through cold restarts.
#[path = "settings/support.rs"]
mod support;
use jig_domain_core::{self as core, settings};
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{Duration, Rng};
use support::*;

fn agree(world: &jig_core_world::effects::World) {
    let current = world.domain.settings();
    let mut expected = std::collections::BTreeMap::new();
    let mut changes = std::collections::BTreeMap::new();
    for row in world.store.rows.values() {
        if let root::Record::Core(core::Record::Core(core::CoreRecord::Settings(row))) = row {
            match row {
                settings::Record::Default { setting, default } => {
                    expected.insert(*setting, default.value.clone());
                }
                settings::Record::Changed { setting, changed } => {
                    changes.insert(*setting, changed.value.clone());
                }
                settings::Record::Revision { .. }
                | settings::Record::Extent { .. }
                | settings::Record::Charter { .. }
                | settings::Record::Proposal { .. } => {}
            }
        }
    }
    for (setting, value) in changes {
        match value {
            Some(value) => {
                expected.insert(setting, value);
            }
            None => {
                expected.remove(&setting);
            }
        }
    }
    assert_eq!(current.offered().as_slice(), expected.keys().copied().collect::<Vec<_>>());
    for (setting, value) in expected {
        assert_eq!(current.value(setting), Some(&value));
    }
    assert_eq!(revision(world, current.newest(), false).head, *current.newest_head());
}
#[test]
fn random_small_settings_changes_and_stale_requests_survive_restarts() {
    let count =
        std::env::var("JIG_CORE_SETTINGS_SEEDS").ok().map_or(24, |value| value.parse::<u64>().expect("seed count"));
    let mut coverage = [false; 5];
    for seed in 713..713 + count {
        let result = std::panic::catch_unwind(|| {
            let mut reached = [false; 5];
            let mut rng = Rng::new(seed);
            let mut world = start(2);
            let signed = sign_in(&mut world);
            assert_eq!(revision(&world, 1, false).head.by, settings::Decider::FirstStart);
            assert!(world.store.rows.contains_key(&key(settings::Key::Default(settings::Setting::Rules))));
            assert!(matches!(
                ask(&mut world, signed, 1, people::Ask::MakeProject { name: "Random project".into(), policy: 1 }),
                people::Reply::Outcome(people::Outcome::ProjectMade { .. })
            ));
            assert!(world.store.rows.values().any(|value| *value == record(settings::Record::Extent { oldest: 0 })));
            for step in 0..16_u8 {
                let previous = world.domain.settings().newest();
                let stale = rng.below(4) == 0;
                reached[0] |= stale;
                let against = previous - u32::from(stale);
                let operation = rng.below(3);
                reached[usize::try_from(operation + 1).expect("three edits")] = true;
                let item = match operation {
                    0 => settings::Item { setting: settings::Setting::Template(1), edit: settings::Edit::Reset },
                    1 => template_edit(&world, 6 + rng.below(8)),
                    _ => {
                        let mut charter = world.domain.settings().charter(1).expect("settings fixture").clone();
                        charter.policy.instructions =
                            format!("random settings {seed} {step}").into_bytes().into_boxed_slice();
                        settings::Item {
                            setting: settings::Setting::Charter(1),
                            edit: settings::Edit::Set(settings::Value::Charter(Box::new(charter))),
                        }
                    }
                };
                let before_rows = settings_rows(&world);
                let before = world.domain.settings().value(item.setting).cloned();
                let answer = change(&mut world, signed, step + 10, against, Box::new([item.clone()]));
                if stale {
                    assert_eq!(settings_rows(&world), before_rows);
                    assert_eq!(
                        answer,
                        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Stale { newest: previous }))
                    );
                } else if world.domain.settings().value(item.setting) == before.as_ref() {
                    assert_eq!(
                        answer,
                        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
                            at: 0,
                            why: people::Malformed::SettingsUnchanged
                        }))
                    );
                } else {
                    changed(&answer, previous + 1);
                }
                agree(&world);
                if rng.below(4) == 0 {
                    reached[4] = true;
                    restart(&mut world, configuration().0);
                    agree(&world);
                }
            }
            let me = frame(&mut world, signed);
            assert_eq!(
                me.projects.get(0).expect("settings fixture").templates.get(0).expect("settings fixture").grace,
                world.domain.settings().template(1).expect("settings fixture").grace
            );
            assert!(
                world.domain.settings().template(1).expect("settings fixture").grace.expect("settings fixture")
                    >= Duration::from_secs(5)
            );
            restart(&mut world, configuration().0);
            agree(&world);
            reached
        });
        let Ok(reached) = result else { panic!("settings replay seed {seed}") };
        for at in 0..coverage.len() {
            coverage[at] |= reached[at];
        }
    }
    assert_eq!(coverage, [true; 5], "stale, reset, template set, charter set and restart all reached");
}
