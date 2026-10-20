use jig_client_protocol_wire::v10 as w;
use jig_domain_core as core;
use jig_engine_clients_world::fixtures::{self, Peer};
use skein_lib::{Reader, Writer};
fn read() -> Box<[u8]> {
    let limits = fixtures::limits().wire;
    let read = w::Read::Me;
    let mut record = Writer::new(usize::try_from(read.measure()).expect("read size"));
    read.encode(&mut record).expect("me read");
    let value = w::Owned::new(&limits, w::OwnedParts { owner: w::Owner::Jig, version: 10, record: record.finish() })
        .expect("frame read");
    let mut bytes = Writer::new(usize::try_from(value.measure()).expect("wire size"));
    value.encode(&mut bytes).expect("read envelope");
    bytes.finish()
}
#[test]
fn a_native_read_waits_for_its_released_answer() {
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = fixtures::signed_in(&mut peer);
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

#[test]
fn a_family_read_hands_its_scope_and_window_to_the_application() {
    use jig_protocol_engine_clients::{Served, ToApplication};
    let mut settings = fixtures::settings();
    settings.served = skein_lib::List::with_capacity(2);
    settings.served.push(Served { family: None, first: 10, last: 10 }).expect("jig");
    settings.served.push(Served { family: Some(1), first: 1, last: 1 }).expect("family");
    let mut peer = Peer::started(&settings);
    let bearer = fixtures::signed_in(&mut peer);
    let record = fixtures::family_read(w::Scope::Own, fixtures::window(3));
    peer.post(b"/jig/v1/read", record, Some(&bearer));
    let Some(ToApplication::Read { scope, window, family, .. }) = peer.take_application() else {
        panic!("family read")
    };
    assert_eq!(family, 1);
    assert_eq!(scope, jig_domain_people::ApplicationScope::Own);
    assert_eq!(window, core::documents::Window { rows: 3, after: None });
}
