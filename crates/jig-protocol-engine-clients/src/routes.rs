//! Method and route dispatch (protocol/client.md, section 4).
use alloc::boxed::Box;
use jig_client_protocol_wire::v8 as w;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Route {
    Link,
    Service,
    SignOut,
    Request,
    Read,
    Watch,
    Unwatch,
    Gone,
    Stream,
    Missing,
}
pub(crate) fn route(call: &skein_http::server::Call) -> Route {
    if call.method == skein_http::Method::Get && call.target.as_ref() == b"/jig/v1/stream" {
        return Route::Stream;
    }
    if call.method != skein_http::Method::Post {
        return Route::Missing;
    }
    match call.target.as_ref() {
        b"/jig/v1/sign-in/link" => Route::Link,
        b"/jig/v1/sign-in/deployment" => Route::Service,
        b"/jig/v1/request" => Route::Request,
        b"/jig/v1/sign-out" => Route::SignOut,
        b"/jig/v1/read" => Route::Read,
        b"/jig/v1/watch" => Route::Watch,
        b"/jig/v1/unwatch" => Route::Unwatch,
        b"/jig/v1/stream-gone" => Route::Gone,
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
    let answer = w::KeyedAnswer::SignedOut;
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
    let answer = w::KeyedAnswer::Busy;
    let mut out = skein_lib::Writer::new(1);
    answer.encode(&mut out).expect("busy fits");
    out.finish()
}

pub(crate) fn read_none(why: jig_domain_core::reads::Why, limits: &w::Limits) -> Box<[u8]> {
    let answer = crate::translate::reads::none(why, limits);
    let mut out = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("wire size fits"));
    answer.encode(&mut out).expect("measured read refusal");
    out.finish()
}

pub(crate) fn open_answer(why: Option<jig_domain_core::WatchRefusal>, limits: &w::Limits) -> Box<[u8]> {
    let answer = match why {
        Some(why) => w::OpenAnswer::Refused(
            w::OpenAnswerRefused::new(
                limits,
                w::OpenAnswerRefusedParts { why: crate::translate::watches::refused(why) },
            )
            .expect("refusal fits"),
        ),
        None => w::OpenAnswer::Admitted,
    };
    let mut out = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("wire size fits"));
    answer.encode(&mut out).expect("measured watch answer");
    out.finish()
}
pub(crate) fn close_answer(signed_out: bool) -> Box<[u8]> {
    let answer = if signed_out { w::CloseAnswer::SignedOut } else { w::CloseAnswer::Closed };
    let mut out = skein_lib::Writer::new(1);
    answer.encode(&mut out).expect("close answer fits");
    out.finish()
}
pub(crate) fn unauthenticated(route: Route, limits: &w::Limits) -> Box<[u8]> {
    match route {
        Route::Read => read_none(jig_domain_core::reads::Why::SignedOut, limits),
        Route::Watch => open_answer(Some(jig_domain_core::WatchRefusal::SignedOut), limits),
        Route::Unwatch => close_answer(true),
        Route::Gone | Route::Stream | Route::Missing => Box::new([]),
        Route::Link | Route::Service => refused(limits),
        Route::SignOut => sign_out_answer(),
        Route::Request => signed_out(limits),
    }
}

pub(crate) fn sign_out_answer() -> Box<[u8]> {
    let mut out = skein_lib::Writer::new(1);
    w::SignOutAnswer::SignedOut.encode(&mut out).expect("signed out fits");
    out.finish()
}

pub(crate) fn not_ready() -> Box<[u8]> {
    let mut out = skein_lib::Writer::new(1);
    w::KeyedAnswer::NotReady.encode(&mut out).expect("not ready fits");
    out.finish()
}

pub(crate) fn sign_in_not_ready() -> Box<[u8]> {
    let mut out = skein_lib::Writer::new(1);
    w::SignInAnswer::NotReady.encode(&mut out).expect("not ready fits");
    out.finish()
}
pub(crate) fn sign_out_not_ready() -> Box<[u8]> {
    let mut out = skein_lib::Writer::new(1);
    w::SignOutAnswer::NotReady.encode(&mut out).expect("not ready fits");
    out.finish()
}
pub(crate) fn stopped(route: Route) -> Option<Box<[u8]>> {
    match route {
        Route::Link | Route::Service => Some(sign_in_not_ready()),
        Route::SignOut => Some(sign_out_not_ready()),
        Route::Request => Some(not_ready()),
        Route::Read | Route::Watch | Route::Unwatch | Route::Gone | Route::Stream | Route::Missing => None,
    }
}

pub(crate) fn encoded(answer: &w::KeyedAnswer) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("wire size fits"));
    answer.encode(&mut writer).expect("measured answer");
    writer.finish()
}

pub(crate) fn encoded_read(answer: &w::ReadAnswer) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("wire size fits"));
    answer.encode(&mut writer).expect("measured answer");
    writer.finish()
}

pub(crate) fn encoded_open(answer: &w::OpenAnswer) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("wire size fits"));
    answer.encode(&mut writer).expect("measured answer");
    writer.finish()
}
