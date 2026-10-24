//! Native settings ingress uses the same held-answer boundary as other reads.
use jig_client_protocol_wire::v19 as w;
use jig_domain_core::{self as core, reads, settings};
use jig_engine_clients_world::fixtures::{self, Peer};
use skein_lib::{List, Reader, Writer};

fn encoded(read: &w::Read) -> Box<[u8]> {
    let limits = fixtures::limits().wire;
    let mut bytes = Writer::new(usize::try_from(read.measure()).expect("read measure"));
    read.encode(&mut bytes).expect("read bytes");
    let value = w::Owned::new(&limits, w::OwnedParts { owner: w::Owner::Jig, version: 19, record: bytes.finish() })
        .expect("read owner");
    let mut bytes = Writer::new(usize::try_from(value.measure()).expect("owned measure"));
    value.encode(&mut bytes).expect("owned bytes");
    bytes.finish()
}
#[test]
fn every_settings_read_waits_for_the_core_answer() {
    let limits = fixtures::limits().wire;
    for (request, expected) in [
        (w::Read::Settings, reads::Read::Settings),
        (
            w::Read::Setting(
                w::ReadSetting::new(&limits, w::ReadSettingParts { setting: w::Setting::Rules, revision: 6 })
                    .expect("setting"),
            ),
            reads::Read::Setting { setting: settings::Setting::Rules, revision: 6 },
        ),
        (
            w::Read::Revision(w::ReadRevision::new(&limits, w::ReadRevisionParts { number: 3 }).expect("revision")),
            reads::Read::Revision { number: 3 },
        ),
        (
            w::Read::Revisions(
                w::ReadRevisions::new(&limits, w::ReadRevisionsParts { before: Some(5) }).expect("revisions"),
            ),
            reads::Read::Revisions { before: Some(5) },
        ),
    ] {
        let mut peer = Peer::started(&fixtures::settings());
        let bearer = fixtures::signed_in(&mut peer);
        let socket = peer.post(b"/jig/v1/read", encoded(&request), Some(&bearer));
        let Some(core::Event::Read { reply, read, .. }) = peer.take_core() else {
            panic!("authenticated settings read")
        };
        assert_eq!(read, expected);
        assert!(peer.answered(socket).is_none());
        peer.answer_read(reply, reads::Answer::None(reads::Why::Unknown));
        let answer = w::ReadAnswer::decode(&limits, &mut Reader::new(peer.answered(socket).expect("held answer")))
            .expect("answer");
        assert_eq!(
            answer,
            w::ReadAnswer::None(
                w::ReadAnswerNone::new(&limits, w::ReadAnswerNoneParts { value: w::ReadNone::Unknown })
                    .expect("unknown")
            )
        );
    }
}
#[test]
fn settings_watch_delivers_its_inventory_on_the_stream() {
    let mut peer = Peer::signed_in(&fixtures::settings(), fixtures::FIRST_SIGN_IN);
    let stream = peer.open_stream();
    let watcher = peer.open_watch(stream, &w::Subject::Settings);
    assert_eq!(peer.requested_subject(), Some(&core::documents::Watched::Settings));
    peer.release_delivery(
        watcher,
        core::documents::Document::Settings(Box::new(core::documents::Settings {
            newest: 6,
            settings: List::with_capacity(0),
            held: List::with_capacity(0),
            pending: List::with_capacity(0),
        })),
    );
    let events = peer.events(stream);
    let w::StreamEvent::Delivery(delivery) = events.last().expect("inventory delivery") else {
        panic!("settings delivery")
    };
    let record = delivery.document().record();
    let value = w::DeliveryValue::decode(&fixtures::limits().wire, &mut Reader::new(record)).expect("inventory record");
    let w::DeliveryValue::Settings(value) = value else { panic!("settings inventory") };
    assert_eq!(value.newest(), 6);
}
