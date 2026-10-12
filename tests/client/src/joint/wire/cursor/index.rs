//! Update index cursor fields follow the domain key (store.md 4.1).
use super::{Reader, Writer, tasks};
pub(super) fn put(
    w: &mut Writer,
    party: u64,
    at: tasks::Newest,
    task: tasks::Newest,
    object: tasks::UpdateObject,
) -> Option<()> {
    w.byte(10)?;
    w.u64(party)?;
    w.u64(at.raw())?;
    w.u64(task.raw())?;
    match object {
        tasks::UpdateObject::Result => w.byte(0)?,
        tasks::UpdateObject::Reply { message, activity } => {
            w.byte(1)?;
            w.u64(message.raw())?;
            w.u64(activity.raw())?;
        }
    }
    Some(())
}
pub(super) fn take(r: &mut Reader<'_>) -> Option<tasks::Key> {
    Some(tasks::Key::Update {
        party: r.u64()?,
        at: tasks::Newest::from_raw(r.u64()?),
        task: tasks::Newest::from_raw(r.u64()?),
        object: match r.byte()? {
            0 => tasks::UpdateObject::Result,
            1 => tasks::UpdateObject::Reply {
                message: tasks::Newest::from_raw(r.u64()?),
                activity: tasks::Newest::from_raw(r.u64()?),
            },
            _ => return None,
        },
    })
}
