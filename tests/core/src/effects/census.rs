//! Index rows point-read their ended owners during the restart census.
use super::{Store, World, core, root, tasks};
pub(super) fn range(world: &mut World, step: core::RestartStep) -> Option<core::Census> {
    if step == core::RestartStep::LoadInbox { Some(root::inbox_census(&mut world.domain, &world.limits)) } else { None }
}
pub(super) fn append(
    world: &mut World,
    step: core::RestartStep,
    rows: Box<[root::Record]>,
    restored: &mut Vec<root::Record>,
) {
    if step != core::RestartStep::LoadInbox {
        restored.extend(rows);
        return;
    }
    for row in rows {
        if let Some(owner) = owner(&world.store, &row) {
            world.census_ended = world.census_ended.checked_add(1).expect("world census count");
            restored.push(owner);
        }
    }
}
fn owner(store: &Store, row: &root::Record) -> Option<root::Record> {
    let task = match row {
        root::Record::Core(core::Record::Tasks(tasks::Stored::Result(row))) => row.task,
        root::Record::Core(core::Record::Tasks(tasks::Stored::ConversationOldest(row))) => row.task,
        root::Record::Core(_) | root::Record::Connector { .. } => return None,
    };
    store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))).cloned()
}
