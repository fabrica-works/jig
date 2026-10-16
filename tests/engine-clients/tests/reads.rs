use jig_client_protocol_wire::v8 as w;
use jig_domain_core as core;
use jig_engine_clients_world::fixtures::{self, Peer};
use skein_lib::{
    Reader, Writer,
    base64::{self, Alphabet},
};
fn signed_in(peer: &mut Peer) -> Box<[u8]> {
    let link = peer.printed_link();
    let exchange = peer.post_link(&link);
    peer.answer_signed_in(fixtures::FIRST_SIGN_IN, fixtures::FIRST_PERSON);
    peer.answer_kept(fixtures::FIRST_SIGN_IN);
    let answer = peer.sign_in_answer(exchange);
    let mut out = Writer::new(61);
    out.put(b"Bearer ").expect("prefix");
    base64::encode(Alphabet::Url, answer.token().expose(), &mut out).expect("bearer");
    out.finish()
}
fn read() -> Box<[u8]> {
    let limits = fixtures::limits().wire;
    let value = w::Owned::new(&limits, w::OwnedParts { owner: w::Owner::Jig, version: 8, record: Box::new([0]) })
        .expect("frame read");
    let mut bytes = Writer::new(usize::try_from(value.measure()).expect("wire size"));
    value.encode(&mut bytes).expect("read envelope");
    bytes.finish()
}
#[test]
fn a_native_read_waits_for_its_released_answer() {
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = signed_in(&mut peer);
    let socket = peer.post(b"/jig/v1/read", read(), Some(&bearer));
    let Some(core::Event::Read { reply, sign_in, read }) = peer.take_core() else { panic!("authenticated read") };
    assert_eq!(sign_in, fixtures::FIRST_SIGN_IN);
    assert_eq!(read, core::reads::Read::Frame);
    assert_eq!(peer.answered(socket), None);
    peer.answer_read(reply, core::reads::Answer::None(core::reads::Why::Gone));
    let limits = fixtures::limits().wire;
    let expected = w::ReadAnswer::None(
        w::ReadAnswerNone::new(&limits, w::ReadAnswerNoneParts { value: w::ReadNone::Gone }).expect("gone"),
    );
    assert_eq!(
        w::ReadAnswer::decode(&limits, &mut Reader::new(peer.answered(socket).expect("read answer"))),
        Ok(expected)
    );
}
