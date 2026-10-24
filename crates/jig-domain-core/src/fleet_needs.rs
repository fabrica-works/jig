//! The core's configured charter requirements for fleet admission (engine.md, 8).
use crate::{Config, Limits};
use jig_domain_fleet::{CharterNeeds, ModelNeed, Needs};
use skein_lib::List;

/// Build the owned requirements used by the core for fleet admission.
///
/// `config` and `limits` must describe the same validated deployment as for
/// [`crate::Core::new`]. This only projects its requirements; it does not validate
/// configuration. The result owns its charter list, models and model names and
/// can outlive `config`.
///
/// # Panics
/// The charter count must fit `limits.tasks.charters`, and each charter's
/// alternative count plus its main model must fit a `u32`.
#[must_use]
pub fn configured(config: &Config, limits: &Limits) -> Needs {
    let mut charters = List::with_capacity(limits.tasks.charters);
    for charter in &config.defaults.charters {
        let policy = &charter.policy;
        let count = u32::try_from(policy.alternatives.len())
            .expect("configured alternatives")
            .checked_add(1)
            .expect("main model");
        let mut models = List::with_capacity(count);
        models.push(model(&policy.model)).expect("main model reserved");
        for alternative in &policy.alternatives {
            models.push(model(alternative)).expect("configured alternative count");
        }
        charters
            .push(CharterNeeds {
                charter: charter.number,
                kinds: charter.hosts,
                models: models.into_boxed(),
                turns: policy.turns,
                time: policy.time,
                waiting: policy.waiting,
                change_reserve: if policy.modify { Some(policy.reserve_time) } else { None },
            })
            .expect("configured charter count");
    }
    Needs {
        charters: charters.into_boxed(),
        spend: config.defaults.rules.maximum_run_spend,
        message_bytes: limits.tasks.message_bytes,
        start_messages: limits.tasks.run_inbox,
        unacknowledged: config.hub_turns,
    }
}
pub(crate) fn current(core: &crate::Core, limits: &Limits) -> Needs {
    let mut charters = List::with_capacity(limits.tasks.charters);
    for setting in &core.current.offered() {
        let number = match setting {
            crate::settings::Setting::Charter(number) => *number,
            crate::settings::Setting::Template(_)
            | crate::settings::Setting::Policy(_)
            | crate::settings::Setting::Rules => continue,
        };
        let charter = core.charter(number).expect("offered charter");
        let policy = &charter.policy;
        let count = u32::try_from(policy.alternatives.len())
            .expect("configured alternatives")
            .checked_add(1)
            .expect("main model");
        let mut models = List::with_capacity(count);
        models.push(model(&policy.model)).expect("main model reserved");
        for alternative in &policy.alternatives {
            models.push(model(alternative)).expect("configured alternative count");
        }
        charters
            .push(CharterNeeds {
                charter: charter.number,
                kinds: charter.hosts,
                models: models.into_boxed(),
                turns: policy.turns,
                time: policy.time,
                waiting: policy.waiting,
                change_reserve: if policy.modify { Some(policy.reserve_time) } else { None },
            })
            .expect("configured charter count");
    }
    Needs {
        charters: charters.into_boxed(),
        spend: core.current.rules().maximum_run_spend,
        message_bytes: limits.tasks.message_bytes,
        start_messages: limits.tasks.run_inbox,
        unacknowledged: core.hub_turns,
    }
}
pub(crate) fn model(value: &crate::Model) -> ModelNeed {
    ModelNeed { name: value.name.clone(), window: value.window, output: value.output }
}
