//! Original Deployment routes are consumed only after whole native admission.
use super::super::transport::Watched;
use super::*;
use crate::harness::ClientId;

#[test]
fn deployment_read_keeps_original_query_client_and_token_until_whole_owner_admission() {
    let mut transport = super::tests::transport();
    let native = Token::new(u64::MAX);
    let source = wire::deployment::tests::source();
    let limits = wire::deployment::tests::limits();
    let answer = reads::Answer::Document(Box::new(documents::Document::Deployment(Box::new(source.clone()))));
    let wrong =
        transport.reply_to(Reply::Read { client: ClientId(0), read: native, what: client::Read::Frame }).into_token();
    let to = transport
        .reply_to(Reply::Read { client: ClientId(0), read: native, what: client::Read::Deployment })
        .into_token();
    assert_eq!(read(&mut transport, wrong, &answer, &limits).err(), Some(Invalid::OwnerRoute));
    assert_eq!(
        read(&mut transport, to, &answer, &Limits { document_bytes: 0, ..limits }).err(),
        Some(Invalid::Document(wire::documents::Invalid::Payload))
    );
    assert!(read(&mut transport, to, &answer, &Limits { rows: 7, ..limits }).is_err());
    assert!(transport.reply(to).is_some() && transport.reply(wrong).is_some());
    let Incoming::Callback {
        client: actual_client,
        event:
            Event::Jig(jig_client::Event::Read {
                read: actual,
                result: client::ReadResult::Value(client::ReadValue::OwnerDeployment(value)),
            }),
    } = read(&mut transport, to, &answer, &limits).expect("whole original read")
    else {
        panic!("whole owner deployment")
    };
    assert_eq!((actual_client, actual), (ClientId(0), native));
    assert_eq!(*value, wire::deployment::document(&source, &limits).expect("whole owner facts"));
    assert!(transport.reply(to).is_none() && transport.reply(wrong).is_some());
    assert_eq!(read(&mut transport, to, &answer, &limits).err(), Some(Invalid::MissingRoute));
}

#[test]
fn deployment_watch_keeps_original_subject_and_ack_until_whole_delivery_admission() {
    let mut transport = super::tests::transport();
    let native = Token::new(u64::MAX);
    let source = wire::deployment::tests::source();
    let limits = wire::deployment::tests::limits();
    let document = documents::Document::Deployment(Box::new(source.clone()));
    let wrong = transport.open(ClientId(0), Token::new(1), Watched::Jig(client::Subject::Frame));
    let watcher = transport.open(ClientId(0), native, Watched::Jig(client::Subject::Deployment));
    assert_eq!(
        super::super::packets::document(&mut transport, wrong, &document, &limits).err(),
        Some(super::super::packets::Invalid::OwnerRoute)
    );
    assert_eq!(
        super::super::packets::document(&mut transport, watcher, &document, &Limits { document_bytes: 0, ..limits })
            .err(),
        Some(super::super::packets::Invalid::Document(wire::documents::Invalid::Payload))
    );
    assert!(
        super::super::packets::document(&mut transport, watcher, &document, &Limits { rows: 7, ..limits }).is_err()
    );
    assert!(transport.taken(ClientId(0), native).is_none());
    let Incoming::Callback {
        client: actual_client,
        event:
            Event::Jig(jig_client::Event::Watch {
                watch,
                event: client::WatchEvent::Delivered(client::Delivery::OwnerDeployment(value)),
            }),
    } = super::super::packets::document(&mut transport, watcher, &document, &limits).expect("whole original delivery")
    else {
        panic!("whole owner deployment delivery")
    };
    assert_eq!((actual_client, watch), (ClientId(0), native));
    assert_eq!(*value, wire::deployment::document(&source, &limits).expect("whole owner facts"));
    assert!(transport.watch(wrong).is_some() && transport.watch(watcher).is_some());
    assert!(
        matches!(transport.taken(ClientId(0), native), Some(jig_domain_core::Event::Delivered { watcher: actual }) if actual == watcher)
    );
    assert!(transport.taken(ClientId(0), native).is_none());
    assert!(transport.taken(ClientId(0), Token::new(1)).is_none());
}
