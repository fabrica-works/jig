use jig_client_protocol_wire::v8 as w;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_engine_clients_world::fixtures::{self, Peer};
use skein_lib::{
    Writer,
    base64::{self, Alphabet},
};
fn signed_in(peer: &mut Peer) -> Box<[u8]> {
    let link = peer.printed_link();
    let exchange = peer.post_link(&link);
    peer.answer_signed_in(fixtures::FIRST_SIGN_IN, fixtures::FIRST_PERSON);
    peer.answer_kept(fixtures::FIRST_SIGN_IN);
    let token = peer.sign_in_answer(exchange);
    let mut out = Writer::new(61);
    out.put(b"Bearer ").expect("prefix");
    base64::encode(Alphabet::Url, token.token().expose(), &mut out).expect("bearer");
    out.finish()
}
fn request() -> Box<[u8]> {
    let limits = fixtures::limits().wire;
    let ask = w::Ask::StartChat(
        w::StartChat::new(&limits, w::StartChatParts { project: 3, words: Box::from(b"hello".as_slice()) })
            .expect("ask"),
    );
    let mut out = Writer::new(usize::try_from(ask.measure()).expect("ask size"));
    ask.encode(&mut out).expect("ask bytes");
    let record =
        w::Keyed::new(&limits, w::KeyedParts { key: [5; 16], owner: w::Owner::Jig, version: 8, request: out.finish() })
            .expect("keyed request");
    let mut out = Writer::new(usize::try_from(record.measure()).expect("record size"));
    record.encode(&mut out).expect("request bytes");
    out.finish()
}
#[test]
fn a_native_keyed_request_reaches_the_core_after_its_bearer_is_checked() {
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = signed_in(&mut peer);
    peer.post(b"/jig/v1/request", request(), Some(&bearer));
    let Some(core::Event::People(people::Event::Ask { sign_in, key, ask, .. })) = peer.take_core() else {
        panic!("authenticated ask");
    };
    assert_eq!(sign_in, fixtures::FIRST_SIGN_IN);
    assert_eq!(key, [5; 16]);
    assert_eq!(ask, people::Ask::StartChat { project: 3, words: Box::from(b"hello".as_slice()) });
}

#[test]
fn a_native_request_waits_for_the_released_reply_and_takes_its_task() {
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = signed_in(&mut peer);
    let socket = peer.post(b"/jig/v1/request", request(), Some(&bearer));
    let Some(core::Event::People(people::Event::Ask { reply_to, .. })) = peer.take_core() else {
        panic!("ask");
    };
    assert_eq!(peer.answered(socket), None);
    peer.answer_request(reply_to, people::Reply::Outcome(people::Outcome::Started { task: 8 }));
    let limits = fixtures::limits().wire;
    let record = w::Answer::decode(&limits, &mut skein_lib::Reader::new(peer.answered(socket).expect("answer")))
        .expect("wire answer");
    let w::Answer::FinalAnswer(answer) = record else {
        panic!("final answer");
    };
    assert_eq!(answer.owner(), &w::Owner::Jig);
    assert_eq!(answer.version(), 8);
    let outcome = w::Outcome::Owner(
        w::OutcomeOwner::new(
            &limits,
            w::OutcomeOwnerParts {
                value: w::OwnerOutcome::Started(
                    w::OwnerOutcomeStarted::new(&limits, w::OwnerOutcomeStartedParts { task: 8 }).expect("started"),
                ),
            },
        )
        .expect("owner"),
    );
    assert_eq!(
        w::Final::decode(&limits, &mut skein_lib::Reader::new(answer.record())),
        Ok(w::Final::Done(w::Done::new(&limits, w::DoneParts { outcome }).expect("done")))
    );
}
