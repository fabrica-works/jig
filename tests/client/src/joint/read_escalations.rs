//! Original opaque Escalation replies are consumed only after whole admission.
use super::*;
use crate::harness::ClientId;

#[test]
fn escalation_read_retains_original_reply_until_whole_identity_and_reserved_envelope_are_admitted() {
    let mut transport = super::tests::transport();
    let native = Token::new(u64::MAX);
    let source = wire::escalations::tests::source();
    let original = source.clone();
    let what = client::Read::Escalation { task: source.task.chip.number };
    let to = transport.reply_to(Reply::Read { client: ClientId(0), read: native, what }).into_token();
    let limits = wire::rows::tests::limits();
    let mut wrong = source.clone();
    wrong.task.chip.number = 0;
    let wrong = reads::Answer::Document(Box::new(documents::Document::Escalation(wrong)));
    assert_eq!(read(&mut transport, to, &wrong, &limits).err(), Some(Invalid::OwnerRoute));
    assert!(transport.reply(to).is_some());
    let whole = reads::Answer::Document(Box::new(documents::Document::Escalation(source)));
    assert_eq!(
        read(&mut transport, to, &whole, &Limits { document_bytes: 0, ..limits }).err(),
        Some(Invalid::Document(wire::documents::Invalid::Payload))
    );
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
                                    jig_client::Readout::Jig(client::ReadResult::Value(client::ReadValue::OwnerEscalation(
                                        value,
                                    ))),
                            },
                    },
            },
    } = read(&mut transport, to, &whole, &limits).expect("original whole escalation")
    else {
        panic!("whole native escalation callback")
    };
    assert_eq!(actual_client, ClientId(0));
    assert_eq!(actual, native);
    assert_eq!(value.task.chip.task, u64::MAX);
    assert_eq!(value.task.project, u32::MAX);
    assert_eq!(value.task.hold.as_ref().expect("source hold").revision().value, 17);
    assert!(value.task.context.is_none());
    assert!(value.task.result.is_some());
    assert_eq!(value.acts.capacity(), client::MAX_OFFERED_ACTS);
    assert_eq!(value.task.acts.capacity(), client::MAX_OFFERED_ACTS);
    assert_eq!(
        value.acts.iter().map(|offer| offer.act).collect::<Vec<_>>(),
        [client::ActKind::Release, client::ActKind::PassUp, client::ActKind::Release]
    );
    assert_eq!(value.as_ref(), &wire::escalations::document(&original, &limits, &[]).expect("complete owner facts"));
    assert!(transport.reply(to).is_none());
    assert_eq!(read(&mut transport, to, &whole, &limits).err(), Some(Invalid::MissingRoute));
}

#[test]
fn an_escalation_document_never_consumes_a_different_original_read_or_another_client_route() {
    let mut transport = super::tests::transport();
    let source = wire::escalations::tests::source();
    let limits = wire::rows::tests::limits();
    let wrong = transport
        .reply_to(Reply::Read {
            client: ClientId(0),
            read: Token::new(1),
            what: client::Read::Task { task: source.task.chip.number },
        })
        .into_token();
    let other = transport
        .reply_to(Reply::Read { client: ClientId(1), read: Token::new(2), what: client::Read::Escalation { task: 0 } })
        .into_token();
    let whole = reads::Answer::Document(Box::new(documents::Document::Escalation(source)));
    for to in [wrong, other] {
        assert_eq!(read(&mut transport, to, &whole, &limits).err(), Some(Invalid::OwnerRoute));
        assert!(transport.reply(wrong).is_some());
        assert!(transport.reply(other).is_some());
    }
    let valid = transport
        .reply_to(Reply::Read {
            client: ClientId(0),
            read: Token::new(3),
            what: client::Read::Escalation { task: u64::MAX },
        })
        .into_token();
    assert!(read(&mut transport, valid, &whole, &limits).is_ok());
    assert!(transport.reply(wrong).is_some());
    assert!(transport.reply(other).is_some());
    assert!(transport.reply(valid).is_none());
}
