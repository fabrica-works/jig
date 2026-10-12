//! Ordered intervals for domain loads (protocol/store.md, 4.3, 7.1 and 7.2).
use crate::keys;
use alloc::boxed::Box;
use jig_domain_core::{self as core, RestoreRange};
use jig_domain_notes as notes;
use skein_kv::{KeyWriter, Range};

/// The restart hands each live family up in the core's declared order.
#[must_use]
pub fn restore(range: &RestoreRange) -> Box<[Range]> {
    Box::new([match range {
        RestoreRange::Deployment => Range::prefix(&[keys::JIG, 0, 0]),
        RestoreRange::People => Range::prefix(&[keys::JIG, 2]),
        RestoreRange::Tasks => Range { start: Box::new([keys::JIG, 1, 0]), end: Some(Box::new([keys::JIG, 1, 6])) },
        RestoreRange::Core => Range { start: Box::new([keys::JIG, 0, 1]), end: Some(Box::new([keys::JIG, 0, 4])) },
    }])
}

/// One globally named entry or the ordered lines of one exact scope.
#[must_use]
pub fn notes(range: &notes::Range) -> Box<[Range]> {
    Box::new([match range {
        notes::Range::Entry { name } => point(&core::Key::Notes(notes::Key::Entry { name: *name })),
        notes::Range::Lines { scope, after: cursor } => {
            let first = encoded(&core::Key::Notes(notes::Key::Line { scope: scope.clone(), name: 0 }));
            let length = first.len().checked_sub(8).expect("the line's last field is its u64 name");
            let prefix = first.get(..length).expect("remove only the final name field");
            let mut range = Range::prefix(prefix);
            if let Some(name) = cursor {
                let key = encoded(&core::Key::Notes(notes::Key::Line { scope: scope.clone(), name: *name }));
                range.start = after(&key);
            }
            range
        }
    }])
}
fn encoded(key: &core::Key) -> Box<[u8]> {
    keys::encode(key, u32::MAX).expect("the domain's bounded key fits a u32 length")
}
fn point(key: &core::Key) -> Range {
    let start = encoded(key);
    let end = after(&start);
    Range { start, end: Some(end) }
}
fn after(key: &[u8]) -> Box<[u8]> {
    let mut writer = KeyWriter::new(u32::MAX);
    for &byte in key {
        writer.tag(byte);
    }
    writer.tag(0);
    writer.finish().expect("the domain's bounded key and one cursor byte fit a u32 length")
}

/// Scopes served from fixed keys; other scopes need their owner's record or an index.
#[must_use]
pub fn scope(party: u64, scope: &core::reads::Scope) -> Option<Box<[Range]>> {
    use core::reads::Scope;
    Some(match scope {
        Scope::Task { task } | Scope::PersonTask { task } | Scope::Result { task } => Box::new([
            point(&core::Key::Tasks(jig_domain_tasks::Key::Live(*task))),
            point(&core::Key::Tasks(jig_domain_tasks::Key::Ended(*task))),
        ]),
        Scope::Reply { task, activity } => {
            Box::new([point(&core::Key::Tasks(jig_domain_tasks::Key::Activity { task: *task, position: *activity }))])
        }
        Scope::History { task, subtree: false } => {
            Box::new([numbered_prefix(&[keys::JIG, 1, 9], *task), numbered_prefix(&[keys::JIG, 1, 10], *task)])
        }
        Scope::Proposal { proposer: jig_domain_people::Proposer::Party(owner), proposal } => {
            if *owner == party {
                Box::new([
                    point(&core::Key::Tasks(jig_domain_tasks::Key::PersonProposal(*proposal))),
                    point(&core::Key::Tasks(jig_domain_tasks::Key::DecidedProposal(*proposal))),
                ])
            } else {
                Box::new([])
            }
        }
        Scope::Spend { project } => Box::new([
            numbered_prefix(&[keys::JIG, 1, 2, 3], u64::from(*project)),
            numbered_prefix(&[keys::JIG, 1, 7, 3], u64::from(*project)),
        ]),
        Scope::Notes { project } => {
            let project_scope = numbered_prefix(&[keys::JIG, 3, 1, 1], u64::from(*project));
            let goals = numbered_prefix(&[keys::JIG, 3, 1, 2], u64::from(*project));
            let resources = numbered_prefix(&[keys::JIG, 3, 1, 3], u64::from(*project));
            if *project == 0 {
                Box::new([Range::prefix(&[keys::JIG, 3, 1, 0]), project_scope, goals, resources])
            } else {
                Box::new([project_scope, goals, resources])
            }
        }
        Scope::ScopedNotes { scope, .. } => notes(&notes::Range::Lines { scope: scope.clone(), after: None }),
        Scope::Proposal { proposer: jig_domain_people::Proposer::Task(_), .. }
        | Scope::Escalation { .. }
        | Scope::Question { .. }
        | Scope::Turn { .. }
        | Scope::EndedDelegates { .. }
        | Scope::History { subtree: true, .. }
        | Scope::Activity { .. }
        | Scope::Inbox { .. }
        | Scope::Conversations
        | Scope::Goals { .. } => return None,
    })
}
fn numbered_prefix(prefix: &[u8], number: u64) -> Range {
    let mut writer = KeyWriter::new(u32::MAX);
    for &byte in prefix {
        writer.tag(byte);
    }
    writer.u64(number);
    let bytes = writer.finish().expect("fixed-width family prefix fits a u32 length");
    Range::prefix(&bytes)
}
