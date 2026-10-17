use super::*;
use crate::harness::ClientId;

#[test]
fn historical_six_reads_keep_original_client_query_and_token_until_whole_value_admission() {
    for (query, source) in wire::historical::tests::sources() {
        let mut transport = tests::transport();
        let token = Token::new(u64::MAX);
        let to = transport.reply_to(Reply::Read { client: ClientId(0), read: token, what: query.clone() }).into_token();
        let answer = reads::Answer::Historical(Box::new(source));
        let limits = wire::historical::tests::limits();
        assert!(matches!(
            read(&mut transport, to, &answer, &Limits { document_bytes: 0, ..limits }).err(),
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
                Event::Jig(jig_client_view::Event::Read {
                    read: original,
                    result: client::ReadResult::Value(client::ReadValue::Historical(_)),
                }),
        } = read(&mut transport, to, &answer, &limits).expect("whole actual historical source")
        else {
            panic!("exact original callback")
        };
        assert_eq!((actual, original), (ClientId(0), token));
        assert!(transport.reply(to).is_none());
        assert_eq!(read(&mut transport, to, &answer, &limits).err(), Some(Invalid::MissingRoute));
    }
}
