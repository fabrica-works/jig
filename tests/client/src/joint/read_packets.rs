//! Retained original read routes at the protocol boundary (contract.md, 6).
//! Whole page admission and typed identity precede consuming a terminal route.
use super::{
    packets::Incoming,
    transport::{Reply, Transport},
    wire,
};
use jig_domain_client::{Limits, contract as client};
use jig_domain_core::{documents, reads};
use jig_test_domain_client::Event;
use skein_lib::Token;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    MissingRoute,
    OwnerRoute,
    Document(wire::documents::Invalid),
    Page(wire::pages::Invalid),
    PendingValue,
    Historical(wire::historical::Invalid),
}

/// Translate a held answer for its exact original native query.
/// # Errors
/// Leaves the original route intact for mismatched queries, unsupported whole
/// carriers and values above the receiving bounds.
pub fn read(
    transport: &mut Transport,
    to: Token,
    answer: &reads::Answer,
    limits: &Limits,
) -> Result<Incoming, Invalid> {
    let route = transport.reply(to).ok_or(Invalid::MissingRoute)?;
    let Reply::Read { client, read, what } = route else { return Err(Invalid::OwnerRoute) };
    let client = *client;
    let read = *read;
    let result = match answer {
        reads::Answer::None(why) => client::ReadResult::None(match why {
            reads::Why::Unknown => client::ReadNone::Unknown,
            reads::Why::Busy => client::ReadNone::Busy,
            reads::Why::Gone => client::ReadNone::Gone,
            reads::Why::SignedOut => client::ReadNone::SignedOut,
        }),
        reads::Answer::Page(value) => {
            let value = match what {
                client::Read::Activity { task, .. } => wire::pages::activity_page(*task, value, limits),
                client::Read::Notes { scope, description, .. } => {
                    wire::pages::notes_page(scope, description.as_ref(), value, limits)
                }
                client::Read::Conversations { .. } => wire::pages::conversations_page(value, limits),
                client::Read::Goals { project, .. } => wire::pages::goals_page(*project, value, limits),
                client::Read::Inbox { .. } => wire::pages::inbox_page(what, value, limits),
                client::Read::History { task, subtree, .. } => {
                    wire::pages::history_page(*task, *subtree, value, limits)
                }
                client::Read::Ended { task, .. } => wire::ledger_pages::ended(*task, value, limits),
                client::Read::Spend { project, .. } => wire::ledger_pages::spend(*project, value, limits),
                client::Read::Frame
                | client::Read::Task { .. }
                | client::Read::Proposal { .. }
                | client::Read::Escalation { .. }
                | client::Read::Question { .. }
                | client::Read::PersonTask { .. }
                | client::Read::Result { .. }
                | client::Read::Project { .. }
                | client::Read::Deployment => return Err(Invalid::PendingValue),
            }
            .map_err(Invalid::Page)?;
            client::ReadResult::Value(value)
        }
        reads::Answer::Document(value) => match value.as_ref() {
            documents::Document::Frame(value) => {
                if *what != client::Read::Frame || value.party != transport.party(client) {
                    return Err(Invalid::OwnerRoute);
                }
                client::ReadResult::Value(wire::documents::frame_read(value, limits).map_err(Invalid::Document)?)
            }
            documents::Document::Task(value) => {
                let client::Read::Task { task } = what else { return Err(Invalid::OwnerRoute) };
                if *task != value.chip.number {
                    return Err(Invalid::OwnerRoute);
                }
                client::ReadResult::Value(wire::documents::task_read(value, limits).map_err(Invalid::Document)?)
            }
            documents::Document::Proposal(value) => {
                let client::Read::Proposal { proposal } = what else { return Err(Invalid::OwnerRoute) };
                if *proposal != wire::proposals::key(value.proposer, value.number) {
                    return Err(Invalid::OwnerRoute);
                }
                client::ReadResult::Value(wire::documents::proposal_read(value, limits).map_err(Invalid::Document)?)
            }
            documents::Document::Escalation(value) => escalation(what, value, limits)?,
            documents::Document::Project(value) => project(what, value, limits)?,
            documents::Document::Deployment(value) => deployment(what, value, limits)?,
            documents::Document::PersonTask(value) => {
                let client::Read::PersonTask { task } = what else { return Err(Invalid::OwnerRoute) };
                if *task != value.task.number {
                    return Err(Invalid::OwnerRoute);
                }
                client::ReadResult::Value(wire::documents::person_task_read(value, limits).map_err(Invalid::Document)?)
            }
            documents::Document::Result(value) => {
                let client::Read::Result { task } = what else { return Err(Invalid::OwnerRoute) };
                if *task != value.task.number {
                    return Err(Invalid::OwnerRoute);
                }
                client::ReadResult::Value(wire::documents::result_read(value, limits).map_err(Invalid::Document)?)
            }
            documents::Document::Question(value) => {
                let client::Read::Question { task, message } = what else { return Err(Invalid::OwnerRoute) };
                if *task != value.task.number || *message != value.message {
                    return Err(Invalid::OwnerRoute);
                }
                client::ReadResult::Value(wire::documents::question_read(value, limits).map_err(Invalid::Document)?)
            }
            documents::Document::Chips(_)
            | documents::Document::Run(_)
            | documents::Document::Tree(_)
            | documents::Document::Goals(_)
            | documents::Document::Tasks(_)
            | documents::Document::Note(_)
            | documents::Document::Inbox(_)
            | documents::Document::Conversations(_) => return Err(Invalid::PendingValue),
        },
        reads::Answer::Historical(value) => {
            client::ReadResult::Value(wire::historical::read(what, value, limits).map_err(Invalid::Historical)?)
        }
    };
    transport.take_reply(to).expect("validated original read route");
    Ok(Incoming::Callback { client, event: Event::Jig(jig_domain_client::Event::Read { read, result }) })
}

fn escalation(
    what: &client::Read,
    value: &documents::Escalation,
    limits: &Limits,
) -> Result<client::ReadResult, Invalid> {
    let client::Read::Escalation { task } = what else { return Err(Invalid::OwnerRoute) };
    if *task != value.task.chip.number {
        return Err(Invalid::OwnerRoute);
    }
    Ok(client::ReadResult::Value(wire::documents::escalation_read(value, limits).map_err(Invalid::Document)?))
}

fn project(what: &client::Read, value: &documents::Project, limits: &Limits) -> Result<client::ReadResult, Invalid> {
    let client::Read::Project { project } = what else { return Err(Invalid::OwnerRoute) };
    if *project != value.number {
        return Err(Invalid::OwnerRoute);
    }
    Ok(client::ReadResult::Value(wire::documents::project_read(value, limits).map_err(Invalid::Document)?))
}

#[cfg(test)]
#[path = "read_projects.rs"]
mod project_tests;

#[cfg(test)]
#[path = "read_escalations.rs"]
mod escalation_tests;

#[cfg(test)]
#[path = "read_ledger_pages.rs"]
mod ledger_page_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::ClientId;
    use jig_domain_people as people;
    use skein_lib::List;
    pub(super) fn transport() -> Transport {
        let mut value = Transport::new(client::VersionRange { first: 1, last: u32::MAX });
        value.bind(
            ClientId(0),
            17,
            people::Identity {
                key: people::IdentityKey { provider: 0, subject: Box::new([]) },
                login: Box::new([]),
                name: Box::new([]),
            },
        );
        value
    }
    #[test]
    fn proposal_reads_keep_the_original_maker_kind_and_whole_pending_value_before_route_consumption() {
        let mut transport = transport();
        let native = Token::new(u64::MAX);
        let mut source = wire::proposals::tests::source();
        let proposal = wire::proposals::key(source.proposer, source.number);
        let to = transport
            .reply_to(Reply::Read { client: ClientId(0), read: native, what: client::Read::Proposal { proposal } })
            .into_token();
        source.proposer = documents::Proposer::Party(u64::MAX);
        let wrong = reads::Answer::Document(Box::new(documents::Document::Proposal(Box::new(source.clone()))));
        let limits = wire::rows::tests::limits();
        assert_eq!(read(&mut transport, to, &wrong, &limits).err(), Some(Invalid::OwnerRoute));
        assert!(transport.reply(to).is_some());
        source.proposer = documents::Proposer::Task(u64::MAX);
        let whole = reads::Answer::Document(Box::new(documents::Document::Proposal(Box::new(source))));
        assert_eq!(
            read(&mut transport, to, &whole, &Limits { document_bytes: 0, ..limits }).err(),
            Some(Invalid::Document(wire::documents::Invalid::Payload))
        );
        assert!(transport.reply(to).is_some());
        let Incoming::Callback {
            event:
                Event::Jig(jig_domain_client::Event::Read {
                    read: actual,
                    result: client::ReadResult::Value(client::ReadValue::OwnerProposal(value)),
                }),
            ..
        } = read(&mut transport, to, &whole, &limits).expect("exact original pending proposal")
        else {
            panic!("whole pending proposal callback")
        };
        assert_eq!(actual, native);
        assert_eq!(value.key, proposal);
        assert_eq!(value.funding.capacity(), 4);
        assert!(value.reason.cut);
        assert!(transport.reply(to).is_none());
        assert_eq!(read(&mut transport, to, &whole, &limits).err(), Some(Invalid::MissingRoute));
    }
    #[test]
    fn person_task_reads_keep_original_task_and_whole_optional_results_before_consuming_the_terminal_route() {
        let mut transport = transport();
        let native = Token::new(u64::MAX);
        let mut source = wire::person_tasks::tests::source();
        let to = transport
            .reply_to(Reply::Read {
                client: ClientId(0),
                read: native,
                what: client::Read::PersonTask { task: source.task.number },
            })
            .into_token();
        let limits = Limits { rows: 8, ..wire::rows::tests::limits() };
        source.task.number = 0;
        let wrong = reads::Answer::Document(Box::new(documents::Document::PersonTask(Box::new(source.clone()))));
        assert_eq!(read(&mut transport, to, &wrong, &limits).err(), Some(Invalid::OwnerRoute));
        assert!(transport.reply(to).is_some());
        source.task.number = u64::MAX;
        let whole = reads::Answer::Document(Box::new(documents::Document::PersonTask(Box::new(source))));
        assert_eq!(
            read(&mut transport, to, &whole, &Limits { document_bytes: 0, ..limits }).err(),
            Some(Invalid::Document(wire::documents::Invalid::Payload))
        );
        assert!(transport.reply(to).is_some());
        let Incoming::Callback {
            event:
                Event::Jig(jig_domain_client::Event::Read {
                    read: actual,
                    result: client::ReadResult::Value(client::ReadValue::OwnerPersonTask(value)),
                }),
            ..
        } = read(&mut transport, to, &whole, &limits).expect("exact original person work")
        else {
            panic!("whole pending person read callback")
        };
        assert_eq!(actual, native);
        assert_eq!(value.task.task, u64::MAX);
        assert_eq!((value.results.len(), value.results.capacity()), (3, 4));
        assert!(value.results.get(0).expect("missing whole result").result.is_none());
        assert!(value.results.get(1).expect("complete result").result.is_some());
        assert!(transport.reply(to).is_none());
        assert_eq!(read(&mut transport, to, &whole, &limits).err(), Some(Invalid::MissingRoute));
    }
    #[test]
    fn task_reads_keep_original_identity_and_admit_whole_owner_state_before_consuming_the_route() {
        let mut transport = transport();
        let native = Token::new(u64::MAX);
        let mut source = wire::task::tests::source();
        let original = client::Read::Task { task: source.chip.number };
        let to = transport.reply_to(Reply::Read { client: ClientId(0), read: native, what: original }).into_token();
        let limits = wire::rows::tests::limits();
        source.chip.number = 0;
        let wrong = reads::Answer::Document(Box::new(documents::Document::Task(Box::new(source.clone()))));
        assert_eq!(read(&mut transport, to, &wrong, &limits).err(), Some(Invalid::OwnerRoute));
        assert!(transport.reply(to).is_some());
        source.chip.number = u64::MAX;
        let whole = reads::Answer::Document(Box::new(documents::Document::Task(Box::new(source))));
        assert_eq!(
            read(&mut transport, to, &whole, &Limits { document_bytes: 0, ..limits }).err(),
            Some(Invalid::Document(wire::documents::Invalid::Payload))
        );
        assert!(transport.reply(to).is_some());
        let Incoming::Callback {
            event:
                Event::Jig(jig_domain_client::Event::Read {
                    read: actual,
                    result: client::ReadResult::Value(client::ReadValue::Task(value)),
                }),
            ..
        } = read(&mut transport, to, &whole, &limits).expect("whole original task")
        else {
            panic!("task callback")
        };
        assert_eq!(actual, native);
        assert!(value.context.is_none());
        assert_eq!(value.hold.as_ref().expect("owner hold").revision().value, 17);
        assert!(transport.reply(to).is_none());
        assert_eq!(read(&mut transport, to, &whole, &limits).err(), Some(Invalid::MissingRoute));
    }

    #[test]
    fn question_reads_keep_original_task_message_and_sender_before_consuming_exact_answer_route() {
        let mut transport = transport();
        let native = Token::new(u64::MAX);
        let mut source = wire::documents::tests::question_source();
        let original = client::Read::Question { task: source.task.number, message: source.message };
        let to = transport.reply_to(Reply::Read { client: ClientId(0), read: native, what: original }).into_token();
        let limits = wire::rows::tests::limits();
        source.message = 0;
        let wrong = reads::Answer::Document(Box::new(documents::Document::Question(source.clone())));
        assert_eq!(read(&mut transport, to, &wrong, &limits).err(), Some(Invalid::OwnerRoute));
        assert!(transport.reply(to).is_some());
        source.message = u64::MAX;
        source.from = jig_domain_tasks::Party::Deployment { project: 0 };
        let source = reads::Answer::Document(Box::new(documents::Document::Question(source)));
        assert_eq!(
            read(&mut transport, to, &source, &Limits { document_bytes: 0, ..limits }).err(),
            Some(Invalid::Document(wire::documents::Invalid::Payload))
        );
        assert!(transport.reply(to).is_some());
        let Incoming::Callback {
            client: ClientId(0),
            event:
                Event::Jig(jig_domain_client::Event::Read {
                    read: actual,
                    result: client::ReadResult::Value(client::ReadValue::Question(value)),
                }),
        } = read(&mut transport, to, &source, &limits).expect("original whole question")
        else {
            panic!("question callback")
        };
        assert_eq!(actual, native);
        assert_eq!(value.message, u64::MAX);
        assert_eq!(value.from, Some(client::OwnerRequester::Deployment { project: 0 }));
        assert_eq!(value.words.bytes(), &[255, 0]);
        assert!(transport.reply(to).is_none());
        assert_eq!(read(&mut transport, to, &source, &limits).err(), Some(Invalid::MissingRoute));
    }

    #[test]
    fn inbox_read_admits_original_group_query_context_and_whole_rows_before_consuming_terminal() {
        let mut transport = transport();
        let native = Token::new(u64::MAX);
        let original = client::Read::Inbox {
            group: client::InboxReadGroup::Needs,
            project: None,
            kind: Some(client::InboxGroup::Questions),
            goal: Some(17),
            after: None,
        };
        let to =
            transport.reply_to(Reply::Read { client: ClientId(0), read: native, what: original.clone() }).into_token();
        let source = wire::inbox::tests::source();
        let mut rows = List::with_capacity(8);
        rows.push(reads::PageRow::Inbox(source.needs.get(0).expect("stored source").clone())).expect("source row");
        let mut page = reads::Page {
            rows,
            before: None,
            next: Some(reads::Cursor::Store {
                scope: reads::Scope::Inbox {
                    group: reads::Group::Updates,
                    filter: reads::Filter { project: None, goal: Some(17) },
                },
                after: None,
            }),
            cut: true,
            inbox_selection: source.selection,
        };
        let limits = wire::inbox::tests::limits();
        let wrong = reads::Answer::Page(Box::new(page.clone()));
        assert_eq!(
            read(&mut transport, to, &wrong, &limits).err(),
            Some(Invalid::Page(wire::pages::Invalid::Identity))
        );
        assert!(transport.reply(to).is_some());
        page.next = Some(reads::Cursor::Store {
            scope: wire::inbox::scope(&original, 200).expect("actual original query"),
            after: None,
        });
        let source = reads::Answer::Page(Box::new(page));
        assert_eq!(
            read(&mut transport, to, &source, &Limits { document_bytes: 100, ..limits }).err(),
            Some(Invalid::Page(wire::pages::Invalid::Payload))
        );
        assert!(transport.reply(to).is_some());
        let Incoming::Callback {
            client: ClientId(0),
            event:
                Event::Jig(jig_domain_client::Event::Read {
                    read: actual,
                    result: client::ReadResult::Value(client::ReadValue::OwnerInbox(value)),
                }),
        } = read(&mut transport, to, &source, &limits).expect("complete original owner answer")
        else {
            panic!("original owner read")
        };
        assert_eq!(actual, native);
        assert!(value.cut);
        assert_eq!((value.rows.capacity(), value.rows.len()), (8, 1));
        assert_eq!(
            value.rows.get(0).expect("stored source").task,
            client::OwnerTaskLink::Stored { number: u64::MAX, status: None }
        );
        assert_eq!(
            value
                .selection
                .as_ref()
                .expect("original source ancestry")
                .get(0)
                .expect("roots")
                .goals
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![u64::MAX, 0]
        );
        assert!(transport.reply(to).is_none());
        assert_eq!(read(&mut transport, to, &source, &limits).err(), Some(Invalid::MissingRoute));
    }

    #[test]
    fn result_reads_admit_the_complete_source_before_consuming_the_original_task_and_reply() {
        let mut transport = transport();
        let native = Token::new(u64::MAX);
        let to = transport
            .reply_to(Reply::Read { client: ClientId(0), read: native, what: client::Read::Result { task: u64::MAX } })
            .into_token();
        let mut source = wire::results::tests::document();
        let limits = Limits { rows: 8, text: 200, document_bytes: 100_000, ..crate::test::limits().jig };
        source.task.number = 0;
        let wrong = reads::Answer::Document(Box::new(documents::Document::Result(Box::new(source.clone()))));
        assert_eq!(read(&mut transport, to, &wrong, &limits).err(), Some(Invalid::OwnerRoute));
        assert!(transport.reply(to).is_some());
        source.task.number = u64::MAX;
        let source = reads::Answer::Document(Box::new(documents::Document::Result(Box::new(source))));
        assert_eq!(
            read(&mut transport, to, &source, &Limits { document_bytes: 0, ..limits }).err(),
            Some(Invalid::Document(wire::documents::Invalid::Payload))
        );
        assert!(transport.reply(to).is_some());
        let Incoming::Callback {
            client: ClientId(0),
            event:
                Event::Jig(jig_domain_client::Event::Read {
                    read: actual,
                    result: client::ReadResult::Value(client::ReadValue::Result(value)),
                }),
        } = read(&mut transport, to, &source, &limits).expect("whole actual Result")
        else {
            panic!("original Result callback")
        };
        assert_eq!(actual, native);
        assert_eq!(value.task.task, u64::MAX);
        assert_eq!(value.at, None);
        assert!(matches!(value.ending, client::Ending::Task(_)));
        assert_eq!(value.value, None);
        assert!(value.details.is_some());
        assert!(transport.reply(to).is_none());
        assert_eq!(read(&mut transport, to, &source, &limits).err(), Some(Invalid::MissingRoute));
    }

    #[test]
    fn actual_read_absence_keeps_its_owner_reason_and_original_native_token() {
        let mut transport = transport();
        let native = Token::new(u64::MAX);
        for (why, expected) in [
            (reads::Why::Unknown, client::ReadNone::Unknown),
            (reads::Why::Busy, client::ReadNone::Busy),
            (reads::Why::Gone, client::ReadNone::Gone),
        ] {
            let to = transport
                .reply_to(Reply::Read { client: ClientId(0), read: native, what: client::Read::Result { task: 31 } })
                .into_token();
            let Incoming::Callback {
                client: ClientId(0),
                event: Event::Jig(jig_domain_client::Event::Read { read: actual, result }),
            } = read(&mut transport, to, &reads::Answer::None(why), &crate::test::limits().jig)
                .expect("actual terminal")
            else {
                panic!("original native read");
            };
            assert_eq!(actual, native);
            assert_eq!(result, client::ReadResult::None(expected));
            assert!(transport.reply(to).is_none());
            assert!(matches!(
                read(&mut transport, to, &reads::Answer::None(why), &crate::test::limits().jig),
                Err(Invalid::MissingRoute)
            ));
        }
    }
    #[test]
    fn whole_activity_read_rejects_reserved_overflow_before_consuming_the_original_query() {
        let mut transport = transport();
        let native = Token::new(0);
        let to = transport
            .reply_to(Reply::Read {
                client: ClientId(0),
                read: native,
                what: client::Read::Activity { task: 31, before: None },
            })
            .into_token();
        let answer = reads::Answer::Page(Box::new(reads::Page {
            before: None,
            inbox_selection: None,
            rows: List::with_capacity(3),
            next: None,
            cut: false,
        }));
        let limits = Limits { rows: 2, ..crate::test::limits().jig };
        assert!(matches!(
            read(&mut transport, to, &answer, &limits),
            Err(Invalid::Page(wire::pages::Invalid::Owner(_)))
        ));
        assert!(transport.reply(to).is_some());
        let Incoming::Callback {
            event:
                Event::Jig(jig_domain_client::Event::Read {
                    read: actual,
                    result: client::ReadResult::Value(client::ReadValue::Activity(page)),
                }),
            ..
        } = read(&mut transport, to, &answer, &Limits { rows: 3, ..limits }).expect("whole reserved collection")
        else {
            panic!("actual whole page");
        };
        assert_eq!(actual, native);
        assert_eq!(page.rows.capacity(), 3);
        assert!(page.rows.is_empty());
        assert_eq!(page.before, None);
        assert_eq!(page.after, None);
        assert!(transport.reply(to).is_none());
    }
    #[test]
    fn a_scoped_note_page_consumes_only_its_original_read_after_exact_query_and_whole_admission() {
        let mut transport = transport();
        let native = Token::new(u64::MAX);
        let limits = Limits { rows: 2, text: 200, document_bytes: 100_000, ..crate::test::limits().jig };
        let query = client::Read::Notes {
            scope: client::NoteScope::Project(u32::MAX),
            description: Some(client::Text::new(Box::new([]), limits.text).expect("explicit empty search")),
            after: None,
        };
        let to = transport.reply_to(Reply::Read { client: ClientId(0), read: native, what: query }).into_token();
        let mut page = reads::Page {
            before: None,
            inbox_selection: None,
            rows: List::with_capacity(2),
            next: Some(reads::Cursor::Store {
                scope: reads::Scope::ScopedNotes {
                    scope: jig_domain_notes::Scope::Project { project: 0 },
                    description: Some(Box::new([])),
                },
                after: None,
            }),
            cut: true,
        };
        assert!(matches!(
            read(&mut transport, to, &reads::Answer::Page(Box::new(page.clone())), &limits),
            Err(Invalid::Page(wire::pages::Invalid::Identity))
        ));
        assert!(transport.reply(to).is_some());
        page.next = Some(reads::Cursor::Store {
            scope: reads::Scope::ScopedNotes {
                scope: jig_domain_notes::Scope::Project { project: u32::MAX },
                description: Some(Box::new([])),
            },
            after: None,
        });
        assert!(matches!(
            read(
                &mut transport,
                to,
                &reads::Answer::Page(Box::new(page.clone())),
                &Limits { document_bytes: 0, ..limits }
            ),
            Err(Invalid::Page(wire::pages::Invalid::Payload))
        ));
        assert!(transport.reply(to).is_some());
        let Incoming::Callback {
            client: ClientId(0),
            event:
                Event::Jig(jig_domain_client::Event::Read {
                    read: original,
                    result: client::ReadResult::Value(client::ReadValue::Notes(value)),
                }),
        } = read(&mut transport, to, &reads::Answer::Page(Box::new(page.clone())), &limits)
            .expect("whole original Notes query")
        else {
            panic!("original native read callback")
        };
        assert_eq!(original, native);
        assert!(value.cut);
        assert_eq!(value.rows.capacity(), 2);
        assert!(value.rows.is_empty());
        assert_eq!(value.before, None);
        assert_eq!(wire::cursor::decode(value.after.as_ref().expect("actual after"), limits.text), page.next);
        assert!(transport.reply(to).is_none());
        assert!(matches!(
            read(&mut transport, to, &reads::Answer::Page(Box::new(page)), &limits),
            Err(Invalid::MissingRoute)
        ));
    }

    #[test]
    fn mixed_history_read_admits_both_owner_row_kinds_before_consuming_the_exact_original_terminal_route() {
        let mut transport = transport();
        let native = Token::new(u64::MAX);
        let task = u64::MAX;
        let to = transport
            .reply_to(Reply::Read {
                client: ClientId(0),
                read: native,
                what: client::Read::History { task, subtree: false, after: None },
            })
            .into_token();
        let mut rows = List::with_capacity(4);
        rows.push(reads::PageRow::History(documents::HistoryLine {
            task,
            revision: 0,
            by: jig_domain_tasks::Party::Task(0),
            change: jig_domain_tasks::Change::Amended,
            reason: documents::Text { bytes: Box::new([]), cut: true },
            proposal: None,
            amendment: None,
            at: None,
        }))
        .expect("semantic row");
        rows.push(reads::PageRow::Milestone(documents::MilestoneLine {
            task,
            position: u64::MAX,
            phase: documents::MilestonePhase::Waiting,
            at: Some(skein_lib::Wall::EPOCH),
        }))
        .expect("lifecycle row");
        let source = reads::Answer::Page(Box::new(reads::Page {
            before: None,
            inbox_selection: None,
            rows,
            next: Some(reads::Cursor::Store { scope: reads::Scope::History { task, subtree: false }, after: None }),
            cut: true,
        }));
        let limits = Limits { rows: 4, text: 200, document_bytes: 100_000, ..crate::test::limits().jig };
        assert_eq!(
            read(&mut transport, to, &source, &Limits { document_bytes: 0, ..limits }).err(),
            Some(Invalid::Page(wire::pages::Invalid::Payload))
        );
        assert!(transport.reply(to).is_some());
        let Incoming::Callback {
            client: ClientId(0),
            event:
                Event::Jig(jig_domain_client::Event::Read {
                    read: actual,
                    result: client::ReadResult::Value(client::ReadValue::History(page)),
                }),
        } = read(&mut transport, to, &source, &limits).expect("whole original mixed terminal")
        else {
            panic!("original history route")
        };
        assert_eq!(actual, native);
        assert!(page.cut);
        assert_eq!((page.rows.capacity(), page.rows.len()), (4, 2));
        assert!(
            matches!(page.rows.get(0), Some(client::HistoryRow::Change(value)) if value.revision == 0 && value.at.is_none())
        );
        assert!(
            matches!(page.rows.get(1), Some(client::HistoryRow::Milestone(value)) if value.position == u64::MAX && value.at == Some(skein_lib::Wall::EPOCH))
        );
        assert!(transport.reply(to).is_none());
        assert_eq!(read(&mut transport, to, &source, &limits).err(), Some(Invalid::MissingRoute));
    }
}

fn deployment(
    what: &client::Read,
    value: &documents::Deployment,
    limits: &Limits,
) -> Result<client::ReadResult, Invalid> {
    if *what != client::Read::Deployment {
        return Err(Invalid::OwnerRoute);
    }
    Ok(client::ReadResult::Value(wire::documents::deployment_read(value, limits).map_err(Invalid::Document)?))
}

#[cfg(test)]
#[path = "deployment_packets.rs"]
mod deployment_tests;

#[cfg(test)]
#[path = "historical_packets.rs"]
mod historical_tests;
