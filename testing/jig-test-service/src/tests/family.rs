use crate::tests::fixtures::family as f;
use jig_client_test_protocol_wire::codec;
use jig_protocol_engine_clients::{Event, FromApplication, ToApplication};
use jig_test_connector as d;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

fn request(record: Box<[u8]>) -> ToApplication {
    ToApplication::Request {
        reply_to: ReplyTo::new(Token::new(27)),
        sign_in: 19,
        key: [23; 16],
        family: 1,
        version: 1,
        record,
        digest: [29; 32],
    }
}
#[test]
fn an_application_request_round_trips_through_its_record() {
    for (label, release) in [
        ("no release", None),
        ("unspecified", Some(f::RELEASE)),
        ("repeat", Some(d::ApplicationRelease { again: Some(true), ..f::RELEASE })),
        ("skip", Some(d::ApplicationRelease { again: Some(false), ..f::RELEASE })),
    ] {
        let value = d::ApplicationRequest { release, ..f::REQUEST };
        let record = codec::request_record(f::PROJECT, &value);
        assert_eq!(
            codec::request(&record, &f::limits()),
            Ok(codec::DecodedRequest { project: f::PROJECT, value }),
            "{label}"
        );
        let root::Event::Application { reply_to, sign_in, key, scope, number, digest, value: request_value } =
            crate::family::to_root(request(record), &f::limits(), 2).expect("root input")
        else {
            panic!("{label}: application input")
        };
        assert_eq!(
            (reply_to.into_token(), sign_in, key, scope, number, digest, request_value),
            (
                Token::new(27),
                19,
                [23; 16],
                jig_domain_people::ApplicationScope::Project(f::PROJECT),
                1,
                [29; 32],
                value
            ),
            "{label}"
        );
    }
}
#[test]
fn a_client_document_round_trips_through_its_record() {
    for (label, value) in f::documents() {
        assert_eq!(codec::document_value(&codec::document(&value), &f::limits()), Ok(value), "{label}");
    }
}
#[test]
fn a_request_that_does_not_decode_reaches_no_root() {
    let good = codec::request_record(f::PROJECT, &f::REQUEST);
    let mut writer = skein_lib::Writer::new(good.len() + 1);
    writer.put(&good).expect("whole record");
    writer.put(&[0]).expect("one trailing byte");
    let trailing = writer.finish();
    let mut other_version = good.clone();
    *other_version.first_mut().expect("version") = 9;
    let cases: [(&str, Box<[u8]>); 4] = [
        ("empty", Box::new([])),
        ("truncated", good[..good.len() - 1].into()),
        ("wrong version", other_version),
        ("trailing", trailing),
    ];
    for (label, record) in cases {
        let answer = crate::family::to_root(request(record), &f::limits(), 2).expect_err("no root input");
        let Event::Application(FromApplication::Undecodable { to }) = *answer else {
            panic!("{label}: undecodable terminal")
        };
        assert_eq!(to.into_token(), Token::new(27), "{label}");
    }
}
#[test]
fn every_scoped_subject_and_read_round_trips_without_inference() {
    for (label, value) in f::subjects() {
        assert_eq!(codec::subject_value(&codec::subject(&value), &f::limits()), Ok(value), "{label}: subject");
        assert_eq!(codec::read_value(&codec::read(&value), &f::limits()), Ok(value), "{label}: read");
    }
}
#[test]
fn a_family_read_retains_its_exact_absence_reason() {
    for (label, value) in [
        ("value", Ok(d::ClientDocument::Object { object: 1, now: f::OBJECT })),
        ("unknown", Err(d::DocumentMissing::Unknown)),
        ("not allowed", Err(d::DocumentMissing::NotAllowed)),
    ] {
        assert_eq!(codec::read_answer_value(&codec::read_answer(&value), &f::limits()), Ok(value), "{label}");
    }
}
#[test]
fn an_application_outcome_retains_every_terminal_and_its_facts() {
    for (label, value) in f::outcomes().into_boxed() {
        assert_eq!(codec::outcome_value(&codec::outcome(&value), &f::limits()), Ok(value), "{label}");
    }
}
#[test]
fn a_document_above_each_receiving_bound_is_refused() {
    let value = d::ClientDocument::Project { project: f::PROJECT, resources: f::list([f::resource()]) };
    let record = codec::document(&value);
    for (label, limits) in [
        ("resources", d::Limits { resources: 0, ..f::limits() }),
        ("segments", d::Limits { path_segments: 1, ..f::limits() }),
        ("segment bytes", d::Limits { segment_bytes: 2, ..f::limits() }),
    ] {
        assert_eq!(codec::document_value(&record, &limits), Err(codec::Undecodable), "{label}");
    }
}
