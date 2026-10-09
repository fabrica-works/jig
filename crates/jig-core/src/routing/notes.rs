//! Scoped note events routed through the notes child
//! (`domain/engine.md`, section 4.4).

use super::{Event, Limits, Requests, tag_notes};
use crate::Core;
use skein_lib::{Env, Queue};

use jig_core_notes as notes;

pub(super) fn route_notes(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    event: notes::Event,
) -> Requests {
    let mut child = Queue::with_capacity(notes::max_out(&env.limits.notes));
    notes::step(&mut core.notes, &Env { now: env.now, wall: env.wall, limits: env.limits.notes }, event, &mut child);
    tag_notes(core, env, work, child)
}
