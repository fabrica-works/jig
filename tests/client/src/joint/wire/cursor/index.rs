//! Update index cursor fields follow the domain key (store.md 4.1).
use super::{Reader, Writer, tasks};
pub(super) fn put(
    w: &mut Writer,
    party: u64,
    at: tasks::Newest,
    task: tasks::Newest,
    object: tasks::UpdateObject,
) -> Option<()> {
    w.byte(12)?;
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

pub(super) fn conversation(w: &mut Writer, tag: u8, party: u64, task: u64) -> Option<()> {
    w.byte(tag)?;
    w.u64(party)?;
    w.u64(task)
}

#[cfg(test)]
mod tests {
    use jig_domain_core::{Key, reads};
    use jig_domain_tasks::{self as tasks, Newest};

    #[test]
    fn conversation_index_keys_round_trip_in_their_original_family() {
        for key in [
            tasks::Key::ConversationNewest { party: u64::MAX, task: Newest::of(0) },
            tasks::Key::ConversationOldest { party: 0, task: u64::MAX },
        ] {
            let source = reads::Cursor::Store {
                scope: reads::Scope::Conversations,
                after: Some(reads::Position::Owner(Key::Tasks(key))),
            };
            let encoded = super::super::encode(&source, 200).expect("conversation key cursor");
            assert_eq!(super::super::decode(&encoded, 200), Some(source));
        }
    }
}
