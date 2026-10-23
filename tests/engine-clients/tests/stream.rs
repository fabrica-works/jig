use jig_client_protocol_wire::v19 as w;
use jig_domain_core as core;
use jig_engine_clients_world::fixtures::{self, Peer};
use skein_lib::{Duration, List, Reader, Writer};
fn streaming() -> (Peer, [u8; 16]) {
    let mut peer = Peer::signed_in(&fixtures::settings(), fixtures::FIRST_SIGN_IN);
    let stream = peer.open_stream();
    (peer, stream)
}
fn watched() -> (Peer, [u8; 16], skein_lib::Token) {
    let (mut peer, stream) = streaming();
    let watcher = peer.open_watch(stream, &fixtures::me_subject());
    (peer, stream, watcher)
}
#[test]
fn two_streams_of_one_sign_in_each_open_with_their_own_names() {
    let (mut peer, first) = streaming();
    let second = peer.open_stream();
    assert_ne!(first, second);
    assert_eq!(peer.events(first), vec![hello(first)]);
    assert_eq!(peer.events(second), vec![hello(second)]);
}
#[test]
fn a_stream_opens_with_its_hello() {
    let (peer, stream) = streaming();
    assert_ne!(stream, [0; 16]);
    assert_eq!(peer.events(stream), vec![hello(stream)]);
}
#[test]
fn a_me_watch_opens_on_the_small_profile() {
    let (mut core, env, sign_in) = fixtures::signed_small_core();
    let settings = jig_protocol_engine_clients::Settings { rows: env.limits.documents.rows, ..fixtures::settings() };
    let mut peer = Peer::signed_in(&settings, sign_in);
    let stream = peer.open_stream();
    let (exchange, event) = peer.request_watch_event(stream, &fixtures::me_subject());
    let core::Event::Watch { watcher, window, .. } = &event else { panic!("frame watch") };
    assert_eq!(window.rows, 4, "Small");
    let watcher = *watcher;
    let core::Requests::Out(mut out) = core::step(&mut core, &env, event);
    let mut delivered = false;
    while let Some(request) = out.pop() {
        if let core::Request::Held(value) = request
            && let core::Held::Delivery { document, .. } = *value
        {
            peer.event(jig_protocol_engine_clients::Event::Admitted { watcher });
            peer.event(jig_protocol_engine_clients::Event::Delivery { watcher, document });
            delivered = true;
        }
    }
    assert!(delivered, "Small core admits and constructs the frame");
    assert_eq!(
        w::OpenAnswer::decode(&fixtures::limits().wire, &mut Reader::new(peer.answered(exchange).expect("answer"))),
        Ok(w::OpenAnswer::Admitted),
        "Small"
    );
}
#[test]
fn a_slow_reader_gets_the_next_delivery_only_after_the_last() {
    let (mut peer, stream, watcher) = watched();
    peer.stop_reading(stream);
    peer.release_delivery(watcher, fixtures::me_document(7));
    peer.release_delivery(watcher, fixtures::me_document(8));
    peer.release_delivery(watcher, fixtures::me_document(9));
    assert_eq!(peer.taken(watcher), 0);
    assert_eq!(peer.events(stream).len(), 1);
    peer.read_all(stream);
    assert_eq!(peer.taken(watcher), 2);
    assert_eq!(peer.events(stream), vec![hello(stream), frame(1, 7), frame(1, 9)]);
}
#[test]
fn a_delivery_is_marked_taken_only_once_written() {
    let (mut peer, stream, watcher) = watched();
    peer.stop_reading(stream);
    peer.release_delivery(watcher, fixtures::me_document(7));
    assert_eq!(peer.taken(watcher), 0);
    peer.read_all(stream);
    assert_eq!(peer.taken(watcher), 1);
    peer.read_all(stream);
    assert_eq!(peer.taken(watcher), 1);
}
#[test]
fn a_stream_cut_mid_event_ends_its_watches() {
    let (mut peer, stream, watcher) = watched();
    let before = peer.stream_bytes(stream).len();
    peer.read_one_piece(stream);
    peer.release_delivery(watcher, fixtures::me_document(7));
    assert_eq!(peer.events(stream).len(), 1);
    assert_eq!(peer.taken(watcher), 0);
    let prefix = peer.stream_bytes(stream).len();
    assert!(prefix > before);
    peer.cut_stream(stream);
    assert!(peer.unwatched(watcher));
    assert_eq!(peer.stream_bytes(stream).len(), prefix);
}
#[test]
fn a_silent_stream_carries_a_heartbeat_comment() {
    let (mut peer, stream) = streaming();
    let before = peer.stream_bytes(stream).len();
    peer.advance_time(Duration::from_secs(4));
    assert_eq!(peer.stream_bytes(stream).len(), before);
    peer.advance_time(Duration::from_secs(1));
    assert_eq!(&peer.stream_bytes(stream)[before..], b":\n\n");
    assert_eq!(peer.events(stream).len(), 1);
}
#[test]
fn run_text_leaves_without_waiting_for_a_commit() {
    let (mut peer, stream) = streaming();
    let subject = w::Subject::Run(
        w::SubjectRun::new(&fixtures::limits().wire, w::SubjectRunParts { task: 7, turn: None, offset: 0 })
            .expect("run subject"),
    );
    let watcher = peer.open_watch(stream, &subject);
    peer.release_text(watcher);
    assert_eq!(peer.text_taken(watcher), 1);
    assert_eq!(peer.events(stream), vec![hello(stream), text(1)]);
}
#[test]
fn a_watch_refused_in_its_step_is_answered_refused() {
    let (mut peer, stream) = streaming();
    let (exchange, watcher) = peer.request_watch(stream, &fixtures::me_subject());
    peer.refuse_watch(watcher, core::WatchRefusal::NotAllowed);
    let expected = w::OpenAnswer::Refused(
        w::OpenAnswerRefused::new(
            &fixtures::limits().wire,
            w::OpenAnswerRefusedParts { why: w::WatchRefusal::NotAllowed },
        )
        .expect("refusal"),
    );
    assert_eq!(
        w::OpenAnswer::decode(&fixtures::limits().wire, &mut Reader::new(peer.answered(exchange).expect("answer"))),
        Ok(expected)
    );
    assert_eq!(peer.events(stream).len(), 1);
}

#[test]
fn me_after_restart_belongs_to_the_signed_in_party() {
    let mut peer = Peer::signed_in(&fixtures::settings(), fixtures::FIRST_SIGN_IN);
    peer.restart(&fixtures::settings());
    let stream = peer.open_stream();
    let watcher = peer.open_watch(stream, &fixtures::me_subject());
    assert_eq!(peer.requested_subject(), Some(&core::documents::Watched::Frame));
    peer.release_current_frame(watcher);
    assert_eq!(peer.events(stream), vec![hello(stream), frame(1, 7)]);
    assert_eq!(peer.taken(watcher), 1);
}
fn hello(stream: [u8; 16]) -> w::StreamEvent {
    let lim = jig_client_test_records::limits();
    let mut served = List::with_capacity(1);
    served
        .push(
            w::Served::new(&lim, w::ServedParts { owner: w::Owner::Jig, first: 19, last: 19 }).expect("literal range"),
        )
        .expect("one range");
    w::StreamEvent::Hello(
        w::Hello::new(&lim, w::HelloParts { stream, served, heartbeat: Duration::from_secs(5) })
            .expect("literal hello"),
    )
}
fn frame(watch: u64, needs: u32) -> w::StreamEvent {
    let lim = jig_client_test_records::limits();
    let mut frame = jig_client_test_records::frame_record().into_parts();
    frame.needs_total = needs;
    let value = w::DeliveryValue::Me(w::Me::new(&lim, frame).expect("literal frame"));
    let mut record = Writer::new(usize::try_from(value.measure()).expect("literal size"));
    value.encode(&mut record).expect("literal frame bytes");
    let document = w::Owned::new(&lim, w::OwnedParts { owner: w::Owner::Jig, version: 19, record: record.finish() })
        .expect("literal owner");
    w::StreamEvent::Delivery(w::Delivery::new(&lim, w::DeliveryParts { watch, document }).expect("literal delivery"))
}
fn text(watch: u64) -> w::StreamEvent {
    w::StreamEvent::Text(
        w::StreamText::new(
            &jig_client_test_records::limits(),
            w::StreamTextParts { watch, value: jig_client_test_records::run_text_record() },
        )
        .expect("literal text"),
    )
}
