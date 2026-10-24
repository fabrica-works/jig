//! Settings requests reach the core; task C supplies the settings reads and watch.
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
fn settings_requests_reach_the_core_and_keep_its_stale_fact() {
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
    let Some(jig_domain_core::Event::ChangeSettings { reply_to, sign_in, key, digest, change }) = peer.take_core()
    else {
        panic!("settings ingress")
    };
    assert_eq!(sign_in, fixtures::FIRST_SIGN_IN);
    assert_eq!(key, [7; 16]);
    assert_eq!(digest, skein_lib::sha256::digest(&encoded!(&ask)));
    assert_eq!(change.against, 4);
    assert!(change.items.is_empty());
    assert_eq!(peer.answered(socket), None);
    peer.answer_request(
        reply_to,
        jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::Refused(jig_domain_people::Refusal::Stale {
            newest: 5,
        })),
    );
    let answer =
        w::KeyedAnswer::decode(&limits, &mut Reader::new(peer.answered(socket).expect("answer"))).expect("wire answer");
    let w::KeyedAnswer::FinalAnswer(owned) = answer else { panic!("final answer") };
    let final_value = w::Final::decode(&limits, &mut Reader::new(owned.record())).expect("final");
    let w::Final::Refused(refused) = final_value else { panic!("stale refusal") };
    let w::Refusal::Stale(value) = refused.refusal() else { panic!("stale fact") };
    assert_eq!(value.newest(), 5);
}
#[test]
fn settings_reads_and_watch_are_not_served() {
    let limits = fixtures::limits().wire;
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = fixtures::signed_in(&mut peer);
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
