use jig_client_protocol_wire::v19 as w;
use jig_engine_clients_world::fixtures::{self, Peer};
use jig_protocol_engine_clients::Event;
use skein_lib::{Reader, Writer};
fn record() -> Box<[u8]> {
    let record = w::Authorize::new(
        &w::CEILINGS,
        w::AuthorizeParts { query: Box::from(b"client_id=app&response_type=code".as_slice()) },
    )
    .expect("bounded fixture");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("record size"));
    record.encode(&mut writer).expect("bounded fixture");
    writer.finish()
}
fn answer(peer: &Peer, exchange: skein_lib::Token) -> w::AuthorizeAnswer {
    let mut reader = Reader::new(peer.answered(exchange).expect("authorization answered"));
    let answer = w::AuthorizeAnswer::decode(&w::CEILINGS, &mut reader).expect("bounded fixture");
    assert!(reader.is_empty());
    answer
}
#[test]
fn without_a_provider_authorization_is_refused_before_credential_checking() {
    for signed in [false, true] {
        let mut peer = if signed {
            Peer::signed_in(&fixtures::settings(), fixtures::FIRST_SIGN_IN)
        } else {
            Peer::started(&fixtures::settings())
        };
        let exchange = if signed {
            peer.authenticated_post(b"/jig/v1/authorize", record())
        } else {
            peer.post(b"/jig/v1/authorize", record(), None)
        };
        let w::AuthorizeAnswer::Refused(refused) = answer(&peer, exchange) else { panic!("no provider") };
        assert_eq!(refused.why(), &w::AuthorizeWhy::NoProvider);
        assert!(peer.take_core().is_none(), "no people read or mutation");
    }
}
#[test]
fn authorization_while_stopping_is_not_ready() {
    let mut peer = Peer::started(&fixtures::settings());
    let exchange = peer.accepted_post(b"/jig/v1/authorize", record());
    peer.event(Event::Stop);
    assert_eq!(answer(&peer, exchange), w::AuthorizeAnswer::NotReady);
    assert!(peer.take_core().is_none());
}

#[test]
fn malformed_authorization_records_reach_nothing_and_are_http_400() {
    for body in [Box::from([]), Box::from([255]), Box::from([0, 0, 0, 2, b'x']), Box::from([0, 0, 0, 0, 1])] {
        let mut peer = Peer::started(&fixtures::settings());
        let exchange = peer.post(b"/jig/v1/authorize", body, None);
        assert_eq!(peer.status(exchange), Some(400));
        assert!(peer.take_core().is_none());
    }
}
