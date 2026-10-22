//! Whole original Project replies retain their route until native admission.
use super::*;
use crate::harness::ClientId;

#[test]
fn project_read_retains_original_project_client_and_token_until_whole_native_admission() {
    let mut transport = super::tests::transport();
    let native = Token::new(u64::MAX);
    let mut source = wire::projects::tests::source();
    let original = source.clone();
    let to = transport
        .reply_to(Reply::Read {
            client: ClientId(0),
            read: native,
            what: client::Read::Project { project: source.number },
        })
        .into_token();
    let limits = wire::projects::tests::limits();
    source.number = 0;
    let wrong = reads::Answer::Document(Box::new(documents::Document::Project(Box::new(source.clone()))));
    assert_eq!(read(&mut transport, to, &wrong, &limits).err(), Some(Invalid::OwnerRoute));
    assert!(transport.reply(to).is_some());
    source.number = u32::MAX;
    let whole = reads::Answer::Document(Box::new(documents::Document::Project(Box::new(source))));
    assert_eq!(
        read(&mut transport, to, &whole, &Limits { document_bytes: 0, turn_text: limits.turn_text, ..limits }).err(),
        Some(Invalid::Document(wire::documents::Invalid::Payload))
    );
    assert!(transport.reply(to).is_some());
    assert!(read(&mut transport, to, &whole, &Limits { rows: 7, turn_text: limits.turn_text, ..limits }).is_err());
    assert!(transport.reply(to).is_some());
    let Incoming::Callback {
        client: actual_client,
        event:
            Event::Client {
                event:
                    jig_client::Event::Up {
                        up:
                            jig_client::Up::ReadDone {
                                read: actual,
                                result:
                                    jig_client::Readout::Jig(client::ReadResult::Value(client::ReadValue::OwnerProject(
                                        value,
                                    ))),
                            },
                    },
            },
    } = read(&mut transport, to, &whole, &limits).expect("complete original project read")
    else {
        panic!("whole native project callback")
    };
    assert_eq!(actual_client, ClientId(0));
    assert_eq!(actual, native);
    assert_eq!(
        value.as_ref(),
        &wire::projects::document(&original, &limits, &[jig_core_world::world::scripted_model()])
            .expect("complete authentic facts")
    );
    assert_eq!((value.number, value.configured_period), (u32::MAX, 0));
    assert_eq!(value.members.get(4).expect("service").party, u64::MAX);
    assert!(transport.reply(to).is_none());
    assert_eq!(read(&mut transport, to, &whole, &limits).err(), Some(Invalid::MissingRoute));
}

#[test]
fn a_project_document_never_consumes_another_original_query_or_another_project_route() {
    let mut transport = super::tests::transport();
    let source = wire::projects::tests::source();
    let limits = wire::projects::tests::limits();
    let wrong = transport
        .reply_to(Reply::Read { client: ClientId(0), read: Token::new(1), what: client::Read::Me })
        .into_token();
    let other = transport
        .reply_to(Reply::Read { client: ClientId(1), read: Token::new(2), what: client::Read::Project { project: 0 } })
        .into_token();
    let whole = reads::Answer::Document(Box::new(documents::Document::Project(Box::new(source))));
    for to in [wrong, other] {
        assert_eq!(read(&mut transport, to, &whole, &limits).err(), Some(Invalid::OwnerRoute));
        assert!(transport.reply(wrong).is_some());
        assert!(transport.reply(other).is_some());
    }
    let valid = transport
        .reply_to(Reply::Read {
            client: ClientId(0),
            read: Token::new(3),
            what: client::Read::Project { project: u32::MAX },
        })
        .into_token();
    assert!(read(&mut transport, valid, &whole, &limits).is_ok());
    assert!(transport.reply(valid).is_none());
    assert!(transport.reply(wrong).is_some());
    assert!(transport.reply(other).is_some());
}
