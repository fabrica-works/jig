//! M3's client wire is served while task B/C engine ingress is explicitly pending.
use jig_client_protocol_wire::v19 as w;
use jig_engine_clients_world::fixtures::{self, Peer};
use skein_lib::{Reader, Writer};
macro_rules! encoded {
    ($value:expr) => {{
        let value = $value;
        let mut out = Writer::new(usize::try_from(value.measure()).expect("wire bytes"));
        value.encode(&mut out).expect("measured fixture");
        out.finish()
    }};
}
#[test]
fn settings_requests_reads_and_watch_are_not_served() {
    let limits = fixtures::limits().wire;
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = fixtures::signed_in(&mut peer);
    let ask = w::Ask::ChangeSettings(
        w::ChangeSettings::new(
            &limits,
            w::ChangeSettingsParts { against: 4, items: Box::new([]), running: w::Running::Stays },
        )
        .expect("change"),
    );
    let keyed = w::Keyed::new(
        &limits,
        w::KeyedParts { key: [7; 16], owner: w::Owner::Jig, version: 19, request: encoded!(&ask), value: None },
    )
    .expect("keyed");
    let socket = peer.post(b"/jig/v1/request", encoded!(&keyed), Some(&bearer));
    assert!(matches!(
        w::KeyedAnswer::decode(&limits, &mut Reader::new(peer.answered(socket).expect("answer"))).expect("wire answer"),
        w::KeyedAnswer::NotServed(_)
    ));
    for read in [
        w::Read::Settings,
        w::Read::Setting(
            w::ReadSetting::new(&limits, w::ReadSettingParts { setting: w::Setting::Rules, revision: 4 })
                .expect("setting"),
        ),
        w::Read::Revision(w::ReadRevision::new(&limits, w::ReadRevisionParts { number: 4 }).expect("revision")),
        w::Read::Revisions(w::ReadRevisions::new(&limits, w::ReadRevisionsParts { before: Some(5) }).expect("page")),
    ] {
        let owned =
            w::Owned::new(&limits, w::OwnedParts { owner: w::Owner::Jig, version: 19, record: encoded!(&read) })
                .expect("owned");
        let socket = peer.post(b"/jig/v1/read", encoded!(&owned), Some(&bearer));
        assert!(matches!(
            w::ReadAnswer::decode(&limits, &mut Reader::new(peer.answered(socket).expect("read answer")))
                .expect("wire answer"),
            w::ReadAnswer::NotServed(_)
        ));
    }
    // A fresh seeded peer installs the same first test credential for its stream.
    let mut peer = Peer::signed_in(&fixtures::settings(), fixtures::FIRST_SIGN_IN);
    let stream = peer.open_stream();
    let subject = w::Owned::new(
        &limits,
        w::OwnedParts { owner: w::Owner::Jig, version: 19, record: encoded!(&w::Subject::Settings) },
    )
    .expect("subject");
    let open = w::Open::new(&limits, w::OpenParts { stream, watch: 999, subject }).expect("open");
    let socket = peer.post(b"/jig/v1/watch", encoded!(&open), Some(&bearer));
    assert!(matches!(
        w::OpenAnswer::decode(&limits, &mut Reader::new(peer.answered(socket).expect("watch answer")))
            .expect("wire answer"),
        w::OpenAnswer::NotServed(_)
    ));
    assert!(peer.take_core().is_none());
}
