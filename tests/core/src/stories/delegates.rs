//! A live conversation with two direct agent delegates.
use jig_domain_tasks as tasks;
pub fn delegate_pair(world: &mut crate::effects::World) -> (u64, u64) {
    use crate::fixtures::{delegate, member};
    let parent = world.assignments[0];
    let first = member(world, tasks::Executor::Agent { charter: 1 }, b"first");
    let first = delegate(world, parent, 1, first);
    let second = member(world, tasks::Executor::Agent { charter: 1 }, b"second");
    let second = delegate(world, parent, 2, second);
    (first, second)
}
