//! Watch delivery events routed through the views child
//! (`domain/engine.md`, section 11).

use super::{Limits, Requests, tag_view};
use crate::Core;
use skein_lib::{Env, Queue};

use jig_core_views as views;

pub(super) fn route_view(core: &mut Core, env: &Env<Limits>, event: views::Event) -> Requests {
    let mut out = Queue::with_capacity(views::max_out(&env.limits.views));
    views::step(&mut core.views, &Env { now: env.now, wall: env.wall, limits: env.limits.views }, event, &mut out);
    tag_view(out, views::max_out(&env.limits.views))
}
