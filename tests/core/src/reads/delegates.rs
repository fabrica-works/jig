//! Requester-index transport; history reads only members found through edges.
use super::{Store, core, reads, root, tasks};

pub(super) fn bounds(requester: u64, ended: bool) -> (root::Key, root::Key) {
    let key = |task| {
        root::Key::Core(core::Key::Tasks(if ended {
            tasks::Key::EndedDelegate { requester, task }
        } else {
            tasks::Key::Delegate { requester, task }
        }))
    };
    (key(0), key(u64::MAX))
}

pub(super) fn candidate(store: &Store, requester: u64, value: &core::Record) -> Option<reads::Row> {
    let core::Record::Tasks(tasks::Stored::EndedDelegate(edge)) = value else { return None };
    if edge.requester != requester {
        return None;
    }
    Some(super::row(store, value, super::owner(store, edge.task)?))
}

pub(super) fn history(store: &Store, task: u64, subtree: bool, tree_tasks: u32) -> Option<Vec<reads::Row>> {
    let cap = if subtree { usize::try_from(tree_tasks).ok()? } else { 1 };
    let mut members = vec![task];
    for index in 0..cap {
        let Some(&requester) = members.get(index) else { break };
        if !subtree {
            break;
        }
        let (first, last) = bounds(requester, false);
        for record in store.rows.range(first..=last).map(|(_, record)| record) {
            let root::Record::Core(core::Record::Tasks(tasks::Stored::Delegate(edge))) = record else { continue };
            if !members.contains(&edge.task) {
                if members.len() == cap {
                    return None;
                }
                members.push(edge.task);
            }
        }
    }
    let mut rows = Vec::new();
    for member in members {
        let Some(owner) = super::owner(store, member) else { continue };
        for (first, last) in [
            (
                tasks::Key::History { task: member, revision: 0 },
                tasks::Key::History { task: member, revision: u64::MAX },
            ),
            (
                tasks::Key::Milestone { task: member, position: 0 },
                tasks::Key::Milestone { task: member, position: u64::MAX },
            ),
        ] {
            let first = root::Key::Core(core::Key::Tasks(first));
            let last = root::Key::Core(core::Key::Tasks(last));
            for record in store.rows.range(first..=last).map(|(_, record)| record) {
                let root::Record::Core(value) = record else { continue };
                rows.push(super::row(store, value, owner));
            }
        }
    }
    Some(rows)
}
