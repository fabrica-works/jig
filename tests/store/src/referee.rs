//! Whole-prefix, header, page and transcript promises (protocol/store.md, 14).
use crate::{Restored, model::Model, root};
use jig_domain_core as core;

#[derive(Debug)]
pub struct Violation(pub &'static str);

pub struct Referee;

impl Referee {
    /// Check recovered data against independently retained commit prefixes.
    ///
    /// # Errors
    /// Returns the first broken store promise.
    pub fn check(model: &Model, restored: &Restored) -> Result<(), Violation> {
        if restored.last < model.acknowledged || restored.last > model.acknowledged + 1 {
            return Err(Violation("last acknowledged or one whole uncertain commit"));
        }
        let expected = model.prefixes.get(&restored.last).ok_or(Violation("recovery has a staged prefix"))?;
        if restored.rows != expected.rows || restored.application != expected.application {
            return Err(Violation("all restored and loaded pages equal one whole model prefix"));
        }
        for (task, turns) in &expected.turns {
            if restored.turns(*task) != turns {
                return Err(Violation("every committed turn is readable whole"));
            }
        }
        for (task, turns) in &restored.transcripts {
            if turns.as_slice() != expected.turns.get(task).map_or(&[][..], Vec::as_slice) {
                return Err(Violation("no readable turn past its commit"));
            }
        }
        if restored.last > 0 {
            let key = core::Key::Core(core::CoreKey::Deployment);
            if restored.rows.get(&key) != Some(&root::header(restored.last)) {
                return Err(Violation("deployment header agrees exactly with store number"));
            }
        }
        Ok(())
    }
}
