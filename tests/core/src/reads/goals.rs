//! Goal index transport: one ended owner joined, no subtree.
use super::{Store, core, reads, root, tasks};

pub(super) fn candidate(store: &Store, project: u32, value: &core::Record) -> Option<reads::Row> {
    let core::Record::Tasks(tasks::Stored::Goal(goal)) = value else { return None };
    if goal.project != project {
        return None;
    }
    let owner = super::owner(store, goal.task)?;
    Some(reads::Row {
        value: value.clone(),
        owner: Some(Box::new(owner.clone())),
        ancestors: Box::new([]),
        related: None,
        linked: Box::new([]),
        inbox: None,
    })
}
pub(super) fn bounds(project: u32) -> (root::Key, root::Key) {
    let key = |raw| root::Key::Core(core::Key::Tasks(tasks::Key::Goal { project, task: tasks::Newest::from_raw(raw) }));
    (key(0), key(u64::MAX))
}
