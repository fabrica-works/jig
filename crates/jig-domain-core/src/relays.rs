//! Committed offers bound the run's unread inbox (tasks.md, 8.2; engine.md, 7.4).
use crate::{Core, Held, PendingRelay};

pub(crate) fn room(core: &Core, task: u64, most: u32) -> bool {
    let Some(proof) = core.proofs.get(&task) else { return false };
    if proof.terminal.is_some() {
        return false;
    }
    let Some(offered) = proof.offered else { return true };
    let mut after = match proof.turn {
        Some(turn) => turn.read,
        None => None,
    };
    for _ in 0..most {
        let Some(word) = core.tasks.next_waiting(task, after) else { return true };
        if word.number > offered {
            return true;
        }
        after = Some(word.number);
    }
    false
}

pub(crate) fn next(core: &mut Core, task: u64, most: u32) -> Option<Held> {
    if core.stopping || core.relaying.contains_key(&task) || !room(core, task, most) {
        return None;
    }
    let proof = core.proofs.get(&task)?;
    let attempt = proof.attempt;
    let previous = proof.offered;
    let word = core.tasks.next_waiting(task, previous)?;
    assert!(
        core.relaying
            .insert(task, PendingRelay { previous, message: word.number })
            .expect("relay per live task")
            .is_none(),
        "one relay per run"
    );
    Some(Held::Relay { task, attempt, previous, word })
}
