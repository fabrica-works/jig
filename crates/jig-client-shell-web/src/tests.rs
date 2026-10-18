use crate::{
    adapter::{self, Asked, Heard},
    look, sign_in,
};
use skein_ui_web::{fetch, transport};
mod fixtures;
use fixtures::{BOUND, ID};

#[test]
fn a_step_s_writes_leave_before_its_exchanges() {
    let requests = adapter::requests(vec![fixtures::exchange(ID, b"\x01"), fixtures::save(b"k", b"v")]);
    assert_eq!(
        requests.writes,
        [skein_ui_web::store::Record { key: b"k".as_slice().into(), value: b"v".as_slice().into() }]
    );
    assert_eq!(requests.exchanges.len(), 1);
    assert_eq!(
        requests.exchanges[0].headers,
        [
            ("Jig-Client".to_owned(), "browser".to_owned()),
            ("Content-Type".to_owned(), "application/octet-stream".to_owned())
        ]
    );
    assert_eq!(requests.exchanges[0].path.as_ref(), b"/jig/v1/read");
    assert_eq!(requests.exchanges[0].method, "POST");
    assert_eq!(requests.exchanges[0].bound, BOUND);
    let mut platform = fixtures::Platform::default();
    let mut store = skein_ui_web::store::Store::new(b"jig.", None);
    transport::carry(requests, &mut store, &mut platform, &mut Vec::new());
    assert_eq!(platform.calls, ["write", "send"]);
}
#[test]
fn an_answer_past_the_bound_is_an_exchange_failed() {
    let requests = adapter::requests(vec![fixtures::exchange(ID, b"\x01")]);
    let exchange = &requests.exchanges[0];
    let mut fetch = fetch::Fetch::new(1);
    let ticket = fetch.begin(exchange.id, exchange.bound).expect("one flight");
    let too_long = [0; BOUND as usize + 1];
    assert_eq!(
        adapter::fetched(fetch.piece(ticket, &too_long).expect("over bound fails")),
        Heard::Failed { exchange: ID }
    );
    assert_eq!(fetch.finish(ticket, 200), None);
}
#[test]
fn a_refused_write_is_heard_as_not_stored() {
    assert_eq!(
        adapter::carried(transport::Carried::Refused { key: b"k".as_slice().into() }),
        Heard::NotStored { key: b"k".as_slice().into() }
    );
}
#[test]
fn the_link_route_s_secret_is_taken_and_the_address_is_the_home() {
    let mut path = b"/jig/v1/sign-in/link?secret=".to_vec();
    path.extend_from_slice(fixtures::SECRET);
    assert_eq!(
        sign_in::take(&fixtures::route(), &path),
        Some(sign_in::Taken { secret: Box::new([7; 32]), address: b"/".as_slice().into() })
    );
}
const OTHER: &[&[u8]] = &[
    b"/inbox?secret=foo",
    b"/jig/v1/sign-in/link",
    b"/jig/v1/sign-in/link?secret=bad",
    b"/jig/v1/sign-in/link?secret=",
    b"/jig/v1/sign-in/link?other=x",
];
#[test]
fn another_path_takes_no_secret() {
    for path in OTHER {
        assert_eq!(sign_in::take(&fixtures::route(), path), None, "{path:?}");
    }
}
#[test]
fn the_credential_dialog_s_words_are_jig_s() {
    assert_eq!(
        look::phrases(),
        look::Phrases {
            failed: "The application failed.",
            credential: "Service credential",
            keep: "Copy this credential now. It will not be shown again.",
            close: "Close"
        }
    );
}
#[test]
fn the_stream_s_pieces_are_cut_to_the_client_s_piece() {
    const PIECE: u32 = 16;
    const BYTES: usize = 40;
    let requests = adapter::requests(vec![fixtures::stream(PIECE)]);
    assert_eq!(
        requests.stream,
        Some(transport::StreamDuty::Open {
            path: b"/jig/v1/stream".as_slice().into(),
            headers: vec![(b"Jig-Client".as_slice().into(), b"browser".as_slice().into())],
            bound: PIECE
        })
    );
    let mut fetch = fetch::Fetch::new(1);
    let ticket = fetch.open_stream(PIECE);
    assert_eq!(
        adapter::fetched(fetch.stream_piece(ticket, Box::new([0; BYTES])).expect("first piece")),
        Heard::Piece { bytes: Box::new([0; 16]) }
    );
    assert_eq!(adapter::fetched(fetch.read_on().expect("second piece")), Heard::Piece { bytes: Box::new([0; 16]) });
    assert_eq!(adapter::fetched(fetch.read_on().expect("last piece")), Heard::Piece { bytes: Box::new([0; 8]) });
    assert_eq!(fetch.read_on(), None);
}
#[test]
fn the_look_names_the_view_s_styles_and_regions() {
    use jig_client_view::{regions::JigRegion, styles::JigStyle};
    assert_eq!(skein_ui::look::check(&[look::NAMES]), None);
    assert_eq!(look::NAMES.style(JigStyle::Card.token()), Some(b"jig-card".as_slice()));
    for number in 0..48 {
        let region = JigRegion::of(number).expect("region");
        assert_eq!(look::NAMES.region(region.region()), Some(region.name()));
    }
}
#[test]
fn closing_the_stream_is_carried() {
    let requests = adapter::requests(vec![Asked::CloseStream]);
    assert_eq!(requests.stream, Some(transport::StreamDuty::Close));
}
#[test]
fn erasing_a_record_is_carried() {
    let requests = adapter::requests(vec![Asked::Erase { key: b"k".as_slice().into() }]);
    assert_eq!(requests.removes, [Box::from(b"k".as_slice())]);
}
#[test]
fn platform_reports_keep_their_contents() {
    assert_eq!(
        adapter::fetched(fetch::Report::Answered { exchange: ID, status: 200, body: b"ok".as_slice().into() }),
        Heard::Answered { exchange: ID, status: 200, body: b"ok".as_slice().into() }
    );
    assert_eq!(adapter::fetched(fetch::Report::StreamOpened), Heard::StreamOpened);
    assert_eq!(adapter::fetched(fetch::Report::StreamEnded), Heard::StreamEnded);
    assert_eq!(
        adapter::carried(transport::Carried::Written { key: b"k".as_slice().into() }),
        Heard::Stored { key: b"k".as_slice().into() }
    );
    assert_eq!(adapter::carried(transport::Carried::Failed { exchange: ID }), Heard::Failed { exchange: ID });
}
