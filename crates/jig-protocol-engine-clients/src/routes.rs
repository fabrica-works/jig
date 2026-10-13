//! Method and route dispatch (protocol/client.md, section 4).
use alloc::boxed::Box;
use jig_protocol_client_wire::v5 as w;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Route {
    Link,
    Service,
    Request,
    Missing,
}
pub(crate) fn route(call: &skein_http::server::Call) -> Route {
    if call.method != skein_http::Method::Post {
        return Route::Missing;
    }
    match call.target.as_ref() {
        b"/jig/v1/sign-in/link" => Route::Link,
        b"/jig/v1/sign-in/deployment" => Route::Service,
        b"/jig/v1/request" => Route::Request,
        _ => Route::Missing,
    }
}
pub(crate) fn refused(_limits: &w::Limits) -> Box<[u8]> {
    let answer = w::SignInAnswer::Refused;
    let mut out = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("wire size fits"));
    answer.encode(&mut out).expect("refusal fits");
    out.finish()
}
pub(crate) fn signed_out(_limits: &w::Limits) -> Box<[u8]> {
    let answer = w::Answer::SignedOut;
    let mut out = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("wire size fits"));
    answer.encode(&mut out).expect("signed out fits");
    out.finish()
}
pub(crate) fn signed_in(_limits: &w::Limits, answer: w::SignedIn) -> Box<[u8]> {
    let answer = w::SignInAnswer::SignedIn(answer);
    let mut out = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("wire size fits"));
    answer.encode(&mut out).expect("admitted sign-in");
    out.finish()
}

pub(crate) fn busy() -> Box<[u8]> {
    let answer = w::Answer::Busy;
    let mut out = skein_lib::Writer::new(1);
    answer.encode(&mut out).expect("busy fits");
    out.finish()
}
