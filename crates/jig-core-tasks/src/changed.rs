//! Retained last-change wall facts (client/domain/pages.md, 6.3).
//! The private published row permits a structural comparison before a save; duplicate
//! publication and cold restore retain the clock. The cache is bounded by one row per task.
use crate::domain::Task;
use alloc::boxed::Box;
use skein_lib::Wall;

pub(crate) fn observe(task: &mut Task, wall: Wall) {
    if task.published.as_deref() == Some(&task.record) {
        return;
    }
    task.record.changed = wall;
    task.published = None;
    task.published = Some(Box::new(task.record.clone()));
}
