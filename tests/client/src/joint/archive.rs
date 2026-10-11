//! Fetch actual immutable turn records behind supplied activity references
//! (client.md, 10; client/domain/contract.md, 7). No live root state is inspected.
use super::{
    packets::Incoming,
    transport::{Reply, Transport},
    wire,
};
use crate::harness::ClientId;
use jig_client::{Limits, contract as client};
use jig_core::{Event, reads};
use jig_scripted_engine as admission;
use jig_test_client_domain::Event as ClientEvent;
use skein_lib::Token;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    Route,
    Session,
    Page(wire::pages::Invalid),
    Archive(wire::activity::Invalid),
    Identity,
    Payload,
}
#[derive(Debug)]
pub enum Started {
    Complete(Incoming),
    Fetch(Vec<Event>),
}
struct Page {
    client: ClientId,
    read: Token,
    value: Box<client::ActivityPage>,
    waiting: BTreeSet<Token>,
}
#[derive(Default)]
pub struct Archives {
    pages: BTreeMap<Token, Page>,
}
fn callback(client: ClientId, read: Token, page: Box<client::ActivityPage>) -> Incoming {
    Incoming::Callback {
        client,
        event: ClientEvent::Jig(jig_client::Event::Read {
            read,
            result: client::ReadResult::Value(client::ReadValue::Activity(page)),
        }),
    }
}
impl Archives {
    /// Hold one admitted activity page while its exact unique references load.
    /// # Errors
    /// Rejects a missing original query/session or an inadmissible whole page.
    pub fn start(
        &mut self,
        transport: &mut Transport,
        to: Token,
        source: &reads::Page,
        limits: &Limits,
    ) -> Result<Started, Invalid> {
        let Some(Reply::Read { client, read, what: client::Read::Activity { task, .. } }) = transport.reply(to) else {
            return Err(Invalid::Route);
        };
        let (client, read, task) = (*client, *read, *task);
        let sign_in = transport.sign_in(client).ok_or(Invalid::Session)?;
        let client::ReadValue::Activity(page) =
            wire::pages::activity_page(task, source, limits).map_err(Invalid::Page)?
        else {
            unreachable!("exact activity page translation")
        };
        let mut names = BTreeSet::new();
        for line in &page.rows {
            if let client::Activity::Owner { source, .. } = &line.event
                && let client::OwnerActivity::Turn { attempt, turn, .. } = source.line
            {
                names.insert((source.task, attempt, turn));
            }
        }
        let mut waiting = BTreeSet::new();
        let mut events = Vec::new();
        for (task, attempt, turn) in names {
            let (token, reply) = transport.routed_reply(Reply::Archive { client, request: to, task, attempt, turn });
            waiting.insert(token);
            events.push(Event::Read { reply, sign_in, read: reads::Read::Turn { task, attempt, turn } });
        }
        transport.take_reply(to).expect("validated original query");
        if waiting.is_empty() {
            Ok(Started::Complete(callback(client, read, page)))
        } else {
            assert!(self.pages.insert(to, Page { client, read, value: page, waiting }).is_none(), "one held query");
            Ok(Started::Fetch(events))
        }
    }

    /// Attach an independently fetched whole record and release the page once.
    /// A genuine absent record keeps the supplied archive reference undecoded.
    /// # Errors
    /// Rejects an unrelated record, codec failure or complete receiving overflow
    /// without consuming that reply or changing the held page.
    pub fn answer(
        &mut self,
        transport: &mut Transport,
        to: Token,
        answer: &reads::Answer,
        codec: &jig_client_transcript::Limits,
        limits: &Limits,
    ) -> Result<Option<Incoming>, Invalid> {
        let Some(Reply::Archive { request, task, attempt, turn, .. }) = transport.reply(to) else {
            return Err(Invalid::Route);
        };
        let (request, task, attempt, turn) = (*request, *task, *attempt, *turn);
        let held = self.pages.get(&request).ok_or(Invalid::Route)?;
        if !held.waiting.contains(&to) {
            return Err(Invalid::Route);
        }
        let record = match answer {
            reads::Answer::Historical(value) => {
                let reads::Past::Turn(value) = &value.value else { return Err(Invalid::Identity) };
                if (value.task, value.attempt, value.turn) != (task, attempt, turn) {
                    return Err(Invalid::Identity);
                }
                Some(value)
            }
            reads::Answer::None(_) => None,
            reads::Answer::Document(_) | reads::Answer::Page(_) => return Err(Invalid::Identity),
        };
        let mut page = held.value.clone();
        if let Some(record) = record {
            for index in 0..page.rows.len() {
                let line = page.rows.get_mut(index).expect("allocated page row");
                if matches!(&line.event, client::Activity::Owner { source, .. }
                    if source.task == task && matches!(source.line, client::OwnerActivity::Turn {
                        attempt: actual, turn: number, .. } if actual == attempt && number == turn))
                {
                    *line = wire::activity::with_archive(line, record, codec, limits).map_err(Invalid::Archive)?;
                }
            }
        }
        let value = client::ReadValue::Activity(page);
        admission::admit_read_value(
            &value,
            &admission::Bounds {
                text: limits.text,
                rows: limits.rows,
                document_bytes: limits.document_bytes,
                ..admission::Bounds::default()
            },
        )
        .map_err(|_| Invalid::Payload)?;
        let client::ReadValue::Activity(page) = value else { unreachable!("whole activity envelope") };
        let held = self.pages.get_mut(&request).expect("validated held query");
        held.value = page;
        held.waiting.remove(&to);
        transport.take_reply(to).expect("validated archive reply");
        if held.waiting.is_empty() {
            let page = self.pages.remove(&request).expect("completed held page");
            Ok(Some(callback(page.client, page.read, page.value)))
        } else {
            Ok(None)
        }
    }

    /// Reload forgets only this client's held pages; Transport detaches its replies.
    pub fn detach(&mut self, client: ClientId) {
        self.pages.retain(|_, page| page.client != client);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_core_people as people;
    use jig_core_tasks as tasks;
    use skein_lib::{List, Wall};
    fn transport() -> Transport {
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
        value.signed_in(ClientId(0), 17, 0);
        value
    }
    fn query(transport: &mut Transport) -> Token {
        transport
            .reply_to(Reply::Read {
                client: ClientId(0),
                read: Token::new(u64::MAX),
                what: client::Read::Activity { task: 31, before: None },
            })
            .into_token()
    }
    fn page() -> reads::Page {
        let mut rows = List::with_capacity(3);
        for position in [0, u64::MAX] {
            rows.push(reads::PageRow::Activity(tasks::ActivityLine {
                task: 31,
                position,
                at: Wall::EPOCH,
                line: tasks::Line::Turn { reply: 9, attempt: 17, turn: 23, read: Some(0) },
            }))
            .expect("supplied row");
        }
        reads::Page { before: None, inbox_selection: None, rows, next: None, cut: false }
    }
    fn codec() -> jig_client_transcript::Limits {
        jig_client_transcript::Limits {
            smith: smith_transcript::v2::CEILINGS,
            bytes: 92,
            turn: client::TurnLimits { messages: 1, blocks: 1, text: 3 },
        }
    }
    fn record(task: u64) -> jig_core::TurnRecord {
        jig_core::TurnRecord {
            task,
            attempt: 17,
            turn: 23,
            spent: u64::MAX,
            read: Some(0),
            at: Wall::EPOCH,
            transcript: include_bytes!("../../../../crates/jig-client-transcript/tests/data/record_turn_full.bin")
                .as_slice()
                .into(),
        }
    }
    #[test]
    fn real_archive_reads_deduplicate_exact_references_and_release_only_the_whole_original_page() {
        let mut transport = transport();
        let mut archives = Archives::default();
        let to = query(&mut transport);
        let limits = Limits { rows: 4, text: 200, document_bytes: 100_000, ..crate::test::limits().jig };
        let Started::Fetch(mut events) = archives.start(&mut transport, to, &page(), &limits).expect("whole page")
        else {
            panic!("actual internal reads");
        };
        assert_eq!(events.len(), 1);
        assert!(transport.reply(to).is_none());
        let Event::Read { reply, sign_in, read: reads::Read::Turn { task, attempt, turn } } = events.remove(0) else {
            panic!("exact immutable reference");
        };
        assert_eq!((sign_in, task, attempt, turn), (0, 31, 17, 23));
        let reply = reply.into_token();
        let wrong = reads::Answer::Historical(Box::new(reads::Historical {
            value: reads::Past::Turn(record(32)),
            at: None,
            by: None,
        }));
        assert!(matches!(archives.answer(&mut transport, reply, &wrong, &codec(), &limits), Err(Invalid::Identity)));
        assert!(transport.reply(reply).is_some());
        let actual = reads::Answer::Historical(Box::new(reads::Historical {
            value: reads::Past::Turn(record(31)),
            at: None,
            by: None,
        }));
        let Some(Incoming::Callback {
            client: ClientId(0),
            event:
                ClientEvent::Jig(jig_client::Event::Read {
                    read,
                    result: client::ReadResult::Value(client::ReadValue::Activity(page)),
                }),
        }) = archives.answer(&mut transport, reply, &actual, &codec(), &limits).expect("whole actual record")
        else {
            panic!("one original whole callback");
        };
        assert_eq!(read, Token::new(u64::MAX));
        assert_eq!(page.rows.capacity(), 3);
        assert_eq!(page.rows.len(), 2);
        for line in &page.rows {
            let client::Activity::Owner { decoded: Some(decoded), .. } = &line.event else {
                panic!("supplied record decoded");
            };
            assert_eq!((decoded.task, decoded.name.attempt, decoded.name.number), (31, 17, 23));
            assert_eq!(decoded.cumulative, u64::MAX);
            assert_eq!(decoded.read, Some(0));
            assert_eq!(decoded.at, Wall::EPOCH);
            assert_eq!(decoded.turn.used.input, u64::MAX);
        }
        assert!(transport.reply(reply).is_none());
        assert!(archives.pages.is_empty());
        assert!(matches!(archives.answer(&mut transport, reply, &actual, &codec(), &limits), Err(Invalid::Route)));
    }
    #[test]
    fn genuine_missing_archive_keeps_the_reference_and_detached_late_reads_have_no_route() {
        let mut transport = transport();
        let mut archives = Archives::default();
        let limits = Limits { rows: 4, text: 200, document_bytes: 100_000, ..crate::test::limits().jig };
        let to = query(&mut transport);
        let Started::Fetch(mut events) = archives.start(&mut transport, to, &page(), &limits).expect("page") else {
            panic!("read");
        };
        let Event::Read { reply, .. } = events.remove(0) else {
            panic!("read");
        };
        let Some(Incoming::Callback {
            event:
                ClientEvent::Jig(jig_client::Event::Read {
                    result: client::ReadResult::Value(client::ReadValue::Activity(page)),
                    ..
                }),
            ..
        }) = archives
            .answer(&mut transport, reply.into_token(), &reads::Answer::None(reads::Why::Gone), &codec(), &limits)
            .expect("genuine absence")
        else {
            panic!("unchanged whole source references");
        };
        for line in &page.rows {
            assert!(line.event.decoded_turn().is_none());
            assert_eq!(line.event.read_position(), Some(0));
        }
        let to = query(&mut transport);
        let Started::Fetch(mut events) = archives.start(&mut transport, to, &self::page(), &limits).expect("new page")
        else {
            panic!("read");
        };
        let Event::Read { reply, .. } = events.remove(0) else {
            panic!("read");
        };
        archives.detach(ClientId(0));
        assert!(transport.detach(ClientId(0)).is_empty());
        assert!(matches!(
            archives.answer(
                &mut transport,
                reply.into_token(),
                &reads::Answer::None(reads::Why::Gone),
                &codec(),
                &limits
            ),
            Err(Invalid::Route)
        ));
        assert!(archives.pages.is_empty());
    }
}
