use super::*;
use crate::harness::ClientId;
use skein_lib::List;

#[test]
fn historical_six_reads_keep_original_client_query_and_token_until_whole_value_admission() {
    for (query, source) in wire::historical::tests::sources() {
        let mut transport = tests::transport();
        let token = Token::new(u64::MAX);
        let to = transport.reply_to(Reply::Read { client: ClientId(0), read: token, what: query.clone() }).into_token();
        let answer = reads::Answer::Historical(Box::new(source));
        let limits = wire::historical::tests::limits();
        assert!(matches!(
            read(&mut transport, to, &answer, &Limits { document_bytes: 0, turn_text: limits.turn_text, ..limits })
                .err(),
            Some(Invalid::Historical(
                wire::historical::Invalid::Payload
                    | wire::historical::Invalid::Document(wire::documents::Invalid::Payload)
            ))
        ));
        let Some(Reply::Read { client, read: original, what }) = transport.reply(to) else {
            panic!("original route survives whole refusal")
        };
        assert_eq!((*client, *original), (ClientId(0), token));
        assert_eq!(what, &query);
        let Incoming::Callback {
            client: actual,
            event:
                Event::Client {
                    event:
                        jig_client::Event::Up {
                            up:
                                jig_client::Up::ReadDone {
                                    read: original,
                                    result:
                                        jig_client::Readout::Jig(client::ReadResult::Value(client::ReadValue::Historical(
                                            _,
                                        ))),
                                },
                        },
                },
        } = read(&mut transport, to, &answer, &limits).expect("whole actual historical source")
        else {
            panic!("exact original callback")
        };
        assert_eq!((actual, original), (ClientId(0), token));
        assert!(transport.reply(to).is_none());
        assert_eq!(read(&mut transport, to, &answer, &limits).err(), Some(Invalid::MissingRoute));
    }
}

#[test]
fn mixed_history_read_admits_both_owner_row_kinds_before_consuming_the_exact_original_terminal_route() {
    let mut transport = tests::transport();
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
    let limits =
        Limits { rows: 4, text: 200, turn_text: 200, document_bytes: 100_000, ..crate::test::limits().client.domain };
    assert_eq!(
        read(&mut transport, to, &source, &Limits { document_bytes: 0, turn_text: limits.turn_text, ..limits }).err(),
        Some(Invalid::Page(wire::pages::Invalid::Payload))
    );
    assert!(transport.reply(to).is_some());
    let Incoming::Callback {
        client: ClientId(0),
        event:
            Event::Client {
                event:
                    jig_client::Event::Up {
                        up:
                            jig_client::Up::ReadDone {
                                read: actual,
                                result:
                                    jig_client::Readout::Jig(client::ReadResult::Value(client::ReadValue::History(page))),
                            },
                    },
            },
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
