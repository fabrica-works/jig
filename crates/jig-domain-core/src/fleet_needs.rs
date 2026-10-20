//! The core's configured charter requirements for fleet admission (engine.md, 8).
use crate::{Config, Limits};
use jig_domain_fleet::{CharterNeeds, ModelNeed, Needs};
use skein_lib::List;

pub(crate) fn configured(config: &Config, limits: &Limits) -> Needs {
    let mut charters = List::with_capacity(limits.tasks.charters);
    for charter in &config.settings.charters {
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
        spend: config.authority.rules().maximum_run_spend,
        message_bytes: limits.tasks.message_bytes,
        start_messages: limits.tasks.run_inbox,
        unacknowledged: config.hub_turns,
    }
}
pub(crate) fn model(value: &crate::Model) -> ModelNeed {
    ModelNeed { name: value.name.clone(), window: value.window, output: value.output }
}
