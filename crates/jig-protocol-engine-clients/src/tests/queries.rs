use super::fixtures;
use crate::translate::watches;
use jig_client_protocol_wire::v19 as w;
use jig_domain_core::{Event, documents as d};
use jig_domain_tasks as t;
use skein_lib::Token;
const QUERY_PROJECT: u32 = 3;
const QUERY_ROOT: u64 = 7;
const QUERY_PARENT: u64 = 8;
const QUERY_REQUESTER: u64 = 9;
const QUERY_WATCHER: Token = Token::new(11);
const QUERY_SIGN_IN: u64 = 41;
#[test]
fn a_task_query_subject_reaches_the_core_with_its_keys() {
    let limits = fixtures::wire();
    let query = w::TaskQuery::new(
        &limits,
        w::TaskQueryParts {
            project: Some(QUERY_PROJECT),
            requester: Some(w::OwnerRequester::Person(
                w::OwnerRequesterPerson::new(&limits, w::OwnerRequesterPersonParts { value: QUERY_REQUESTER }).unwrap(),
            )),
            root: Some(QUERY_ROOT),
            parent: Some(QUERY_PARENT),
            label: Some(5),
            phase: Some(w::PhaseClass::Held),
            executor: Some(w::ExecutorKind::Person),
            tracked: Some(true),
            order: w::QueryOrder::Priority,
        },
    )
    .unwrap();
    let subject = w::Subject::Tasks(w::SubjectTasks::new(&limits, w::SubjectTasksParts { query, window: 2 }).unwrap());
    let Some(Event::Watch { watcher, sign_in, subject, window }) =
        watches::watch(QUERY_WATCHER, QUERY_SIGN_IN, subject, 32)
    else {
        panic!("watch")
    };
    assert_eq!((watcher, sign_in, window), (QUERY_WATCHER, QUERY_SIGN_IN, d::Window { rows: 2, after: None }));
    let d::Watched::Query { query } = subject else { panic!("task query") };
    assert_eq!(query.project, Some(QUERY_PROJECT));
    assert_eq!(query.requester, Some(t::Party::Person(QUERY_REQUESTER)));
    assert_eq!((query.root, query.parent, query.label), (Some(QUERY_ROOT), Some(QUERY_PARENT), Some(5)));
    assert_eq!(
        (query.phase, query.executor, query.tracked, query.order),
        (Some(d::PhaseClass::Held), Some(d::ExecutorKind::Person), Some(true), d::QueryOrder::Priority)
    );
}
const WAITING_ROLE: u32 = 2;
#[test]
fn a_task_document_carries_its_label_flags_activity_and_oldest_waiting_entry() {
    let limits = fixtures::wire_limits();
    let mut source = fixtures::task_with_hold();
    source.label = 5;
    source.flags = t::Flags {
        ended_by: t::EndedBy::Result,
        conversation: true,
        listed: true,
        retention: t::Retention::ForgottenOnceRead,
    };
    source.last_activity = skein_lib::Wall::from_nanos(13);
    source.waits_on = d::WaitsOn::Party {
        whom: jig_domain_people::Whom::Role { project: QUERY_PROJECT, role: WAITING_ROLE },
        entries: 3,
        task: QUERY_ROOT,
        kind: jig_domain_people::EntryKind::Question { message: fixtures::NUMBER },
    };
    let value = crate::translate::task::document(&source, &[], &limits).unwrap();
    assert_eq!(
        (
            value.label(),
            value.flags().conversation(),
            value.flags().listed(),
            value.flags().retention(),
            value.last_activity()
        ),
        (5, true, true, &w::Retention::ForgottenOnceRead, 13)
    );
    let w::WaitsOn::Party(wait) = value.waits_on() else { panic!("party") };
    let w::Whom::Role(whom) = wait.whom() else { panic!("role") };
    assert_eq!((whom.project(), whom.role()), (QUERY_PROJECT, WAITING_ROLE));
    assert_eq!((wait.entries(), wait.task(), wait.kind()), (3, QUERY_ROOT, &w::WaitedFor::Question));
}
