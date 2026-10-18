//! Whole Inbox owner facts and parallel original selection context
//! (client/domain/contract.md, 5–7). Neither row membership nor ancestry is
//! reconstructed here; native reads retain their original query separately.
use super::{owner, reads, results};
use jig_client::DomainLimits as Limits;
use jig_client_domain::contract as client;
use jig_domain_core::documents;
use skein_lib::List;

#[must_use]
pub fn object(value: documents::InboxObject) -> client::OwnerInboxObject {
    match value {
        documents::InboxObject::Question { message } => client::OwnerInboxObject::Question { message },
        documents::InboxObject::Proposal { proposer, number } => client::OwnerInboxObject::Proposal {
            proposer: match proposer {
                jig_domain_people::Proposer::Task(number) => client::Proposer::Task(number),
                jig_domain_people::Proposer::Party(number) => client::Proposer::Party(number),
            },
            number,
        },
        documents::InboxObject::Escalation { revision } => client::OwnerInboxObject::Escalation { revision },
        documents::InboxObject::PersonTask => client::OwnerInboxObject::PersonTask,
        documents::InboxObject::Reply { message } => client::OwnerInboxObject::Reply { message },
        documents::InboxObject::Result { position } => client::OwnerInboxObject::Result { position },
    }
}
#[must_use]
pub fn cursor(value: documents::Cursor) -> client::OwnerInboxCursor {
    match value {
        documents::Cursor::Number { number } => client::OwnerInboxCursor::Number { number },
        documents::Cursor::Inbox { task, object: value, at } => {
            client::OwnerInboxCursor::Inbox { task, object: object(value), at }
        }
    }
}
/// # Errors
/// Refuses a whole linked chip or optional reply above the receiving bounds.
pub fn entry(value: &documents::InboxEntry, limits: &Limits) -> Result<client::OwnerInboxEntry, owner::Invalid> {
    Ok(client::OwnerInboxEntry {
        reply: value.reply.as_ref().map(|value| super::queries::line(value, limits).map(Box::new)).transpose()?,
        task: results::link(&value.task, limits)?,
        project: value.project,
        object: object(value.object),
        activity: value.activity,
        at: value.at,
        unread: value.unread,
        cursor: cursor(value.cursor),
    })
}
/// Source ancestry capacity is independent of ordinary page rows; the complete
/// allocation is charged against document bytes, never clipped to rows.
/// # Errors
/// Refuses outer row allocation or complete reserved roots above the byte ceiling.
pub fn selection(
    value: Option<&List<documents::InboxSelection>>,
    limits: &Limits,
) -> Result<Option<List<client::OwnerInboxSelection>>, owner::Invalid> {
    value
        .map(|value| {
            owner::list(value, limits, |value| {
                let bytes = List::<u64>::worst_case(value.goals.capacity()).ok_or(owner::Invalid::TooLarge)?;
                if bytes > limits.document_bytes {
                    return Err(owner::Invalid::TooLarge);
                }
                let mut goals = List::with_capacity(value.goals.capacity());
                for number in &value.goals {
                    goals.push(*number).map_err(|_| owner::Invalid::TooLarge)?;
                }
                Ok(client::OwnerInboxSelection { cursor: cursor(value.cursor), goals })
            })
        })
        .transpose()
}
/// # Errors
/// Refuses any whole group, linked payload or context allocation above bounds.
pub fn document(value: &documents::Inbox, limits: &Limits) -> Result<client::OwnerInbox, owner::Invalid> {
    Ok(client::OwnerInbox {
        party: value.party,
        needs: owner::list(&value.needs, limits, |value| entry(value, limits))?,
        updates: owner::list(&value.updates, limits, |value| entry(value, limits))?,
        next: value.next.map(cursor),
        cached_needs: value.cached_needs,
        cached_updates: match value.cached_updates {
            documents::Count::Known(value) => client::Count { value, limited: false },
            documents::Count::AtLeast(value) => client::Count { value, limited: true },
        },
        results_seen: value.results_seen,
        updates_time_complete: value.updates_time_complete,
        selection: selection(value.selection.as_ref(), limits)?,
    })
}
/// # Errors
/// Refuses another receiving query; source row membership remains owner-selected.
#[must_use]
pub fn scope(value: &client::Read, text: u32) -> Option<jig_domain_core::reads::Scope> {
    let jig_domain_core::reads::Read::Inbox { group, filter, .. } = reads::read(value, text).ok()? else { return None };
    Some(jig_domain_core::reads::Scope::Inbox { group, filter })
}

#[cfg(test)]
pub(crate) mod tests {
    const UTF8_WORDS: &[u8] = b"~\0";

    use super::*;
    use jig_domain_people as people;
    use skein_lib::Wall;
    pub(crate) fn limits() -> Limits {
        Limits { rows: 8, text: 200, document_bytes: 100_000, ..crate::test::limits().client.domain }
    }
    pub(crate) fn source() -> documents::Inbox {
        let objects = [
            documents::InboxObject::Question { message: 0 },
            documents::InboxObject::Proposal { proposer: people::Proposer::Task(u64::MAX), number: 0 },
            documents::InboxObject::Proposal { proposer: people::Proposer::Party(u64::MAX), number: 0 },
            documents::InboxObject::Escalation { revision: u64::MAX },
            documents::InboxObject::PersonTask,
            documents::InboxObject::Reply { message: u64::MAX },
            documents::InboxObject::Result { position: 0 },
        ];
        let mut needs = List::with_capacity(8);
        let mut updates = List::with_capacity(8);
        let mut selection = List::with_capacity(8);
        for (index, object) in objects.into_iter().enumerate() {
            let cursor = documents::Cursor::Inbox { task: u64::MAX, object, at: None };
            let mut goals = List::with_capacity(12);
            goals.push(u64::MAX).expect("self");
            goals.push(0).expect("original root");
            selection.push(documents::InboxSelection { cursor, goals }).expect("context");
            let value = documents::InboxEntry {
                reply: if index == 5 { Some(Box::new(super::super::rows::tests::task_source())) } else { None },
                task: documents::TaskLink::Stored { number: u64::MAX, status: None },
                project: None,
                object,
                activity: if index == 5 { Some(0) } else { None },
                at: None,
                unread: if index == 5 { Some(false) } else { None },
                cursor,
            };
            if index >= 5 {
                updates.push(value).expect("update");
            } else {
                needs.push(value).expect("need");
            }
        }
        documents::Inbox {
            party: 17,
            needs,
            updates,
            next: Some(documents::Cursor::Number { number: 0 }),
            cached_needs: u32::MAX,
            cached_updates: documents::Count::AtLeast(0),
            results_seen: u64::MAX,
            updates_time_complete: false,
            selection: Some(selection),
        }
    }
    #[test]
    fn whole_inbox_keeps_all_raw_objects_optional_facts_source_groups_totals_and_reserved_ancestry() {
        let source = source();
        let original = source.clone();
        let copied = document(&source, &limits()).expect("whole source");
        assert_eq!(source, original);
        assert_eq!((copied.party, copied.needs.capacity(), copied.updates.capacity()), (17, 8, 8));
        assert_eq!((copied.needs.len(), copied.updates.len()), (5, 2));
        assert_eq!(copied.cached_needs, u32::MAX);
        assert_eq!(copied.cached_updates, client::Count { value: 0, limited: true });
        assert_eq!(copied.results_seen, u64::MAX);
        assert!(!copied.updates_time_complete);
        assert_eq!(copied.next, Some(client::OwnerInboxCursor::Number { number: 0 }));
        for (actual, original) in
            copied.needs.iter().chain(copied.updates.iter()).zip(source.needs.iter().chain(source.updates.iter()))
        {
            assert_eq!(actual.object, object(original.object));
            assert_eq!(actual.cursor, cursor(original.cursor));
            assert_eq!(actual.task, client::OwnerTaskLink::Stored { number: u64::MAX, status: None });
            assert_eq!(
                (actual.project, actual.at, actual.activity, actual.unread),
                (original.project, original.at, original.activity, original.unread)
            );
        }
        let roots = copied.selection.as_ref().expect("complete original context");
        assert_eq!(roots.capacity(), 8);
        for value in roots {
            assert_eq!(value.goals.capacity(), 12);
            assert_eq!(value.goals.iter().copied().collect::<Vec<_>>(), vec![u64::MAX, 0]);
        }
        let reply = copied.updates.get(0).expect("reply row").reply.as_ref().expect("whole reply");
        assert!(reply.chip.title.cut);
        assert_eq!(reply.chip.title.bytes.bytes(), UTF8_WORDS);
        assert_eq!(reply.conversation, None);
        assert_eq!(
            reply.budget,
            owner::budget(
                &source.updates.get(0).expect("original reply").reply.as_ref().expect("original header").budget
            )
        );
    }
    #[test]
    fn inbox_context_absence_empty_and_whole_reserved_roots_refuse_without_clipping() {
        let mut source = source();
        for selection in [None, Some(List::with_capacity(8))] {
            source.selection = selection;
            let copied = document(&source, &limits()).expect("presence retained");
            assert_eq!(copied.selection.as_ref().map(List::capacity), source.selection.as_ref().map(List::capacity));
        }
        source = self::source();
        let original = source.clone();
        assert_eq!(document(&source, &Limits { rows: 7, ..limits() }), Err(owner::Invalid::TooLarge));
        assert_eq!(document(&source, &Limits { document_bytes: 95, ..limits() }), Err(owner::Invalid::TooLarge));
        assert_eq!(source, original);
        let client::Delivery::OwnerInbox(value) =
            super::super::documents::inbox_delivery(&source, &limits()).expect("whole admitted source")
        else {
            panic!("owner inbox")
        };
        assert_eq!(value.selection.as_ref().expect("context").get(0).expect("row").goals.capacity(), 12);
        assert_eq!(
            super::super::documents::inbox_delivery(&source, &Limits { document_bytes: 100, ..limits() }),
            Err(super::super::documents::Invalid::Payload)
        );
        source.cached_updates = documents::Count::Known(0);
        source.selection = None;
        assert_eq!(
            document(&source, &limits()).expect("known zero").cached_updates,
            client::Count { value: 0, limited: false }
        );
    }
    #[test]
    fn inbox_links_and_cursor_absence_epoch_and_zero_remain_independent() {
        let chip = super::super::chips::tests::source_chip();
        let mut value = source().needs.get(0).expect("source question").clone();
        for link in [
            documents::TaskLink::Live(chip.clone()),
            documents::TaskLink::Retained { chip: chip.clone(), at: None },
            documents::TaskLink::Retained { chip, at: Some(Wall::EPOCH) },
            documents::TaskLink::Stored { number: 0, status: Some(jig_domain_tasks::Status::Cancelled) },
        ] {
            value.task = link;
            value.project = Some(0);
            value.at = Some(Wall::EPOCH);
            value.unread = Some(false);
            value.cursor = documents::Cursor::Number { number: 0 };
            let copied = entry(&value, &limits()).expect("whole source link");
            assert_eq!(copied.task, results::link(&value.task, &limits()).expect("whole exact link"));
            assert_eq!((copied.project, copied.at, copied.unread), (Some(0), Some(Wall::EPOCH), Some(false)));
            assert_eq!(copied.cursor, client::OwnerInboxCursor::Number { number: 0 });
        }
        assert_ne!(
            cursor(documents::Cursor::Inbox { task: 0, object: value.object, at: None }),
            cursor(documents::Cursor::Inbox { task: 0, object: value.object, at: Some(Wall::EPOCH) })
        );
    }
}
