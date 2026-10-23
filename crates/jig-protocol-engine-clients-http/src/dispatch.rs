//! Method and route dispatch (protocol/client.md, 17.1).
use alloc::boxed::Box;
use jig_client_protocol_wire::v18 as w;
use jig_protocol_engine_clients::Message;
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
pub(crate) fn route(prefix: &[u8], call: &skein_http::server::Call) -> Route {
    let Some(path) = call.target.strip_prefix(prefix) else {
        return Route::Missing;
    };
    if call.method == skein_http::Method::Get && path == b"/v1/stream" {
        return Route::Stream;
    }
    if call.method != skein_http::Method::Post {
        return Route::Missing;
    }
    match path {
        b"/v1/sign-in/link" => Route::Link,
        b"/v1/sign-in/deployment" => Route::Service,
        b"/v1/request" => Route::Request,
        b"/v1/sign-out" => Route::SignOut,
        b"/v1/read" => Route::Read,
        b"/v1/watch" => Route::Watch,
        b"/v1/unwatch" => Route::Unwatch,
        b"/v1/stream-gone" => Route::Gone,
        _ => Route::Missing,
    }
}
pub(crate) fn message(route: Route, body: Box<[u8]>) -> Option<Message> {
    Some(match route {
        Route::Link => Message::SignInLink(body),
        Route::Service => Message::SignInService(body),
        Route::SignOut => Message::SignOut(body),
        Route::Request => Message::Request(body),
        Route::Read => Message::Read(body),
        Route::Watch => Message::Watch(body),
        Route::Unwatch => Message::Unwatch(body),
        Route::Gone => Message::Gone(body),
        Route::Stream | Route::Missing => return None,
    })
}

pub(crate) fn reply(
    settings: &crate::Settings,
    call: &crate::call::Call,
    wall: skein_lib::Wall,
    body: Box<[u8]>,
) -> (Box<[u8]>, Option<skein_http::Header>) {
    if !call.browser {
        return (body, None);
    }
    if call.route == Route::SignOut {
        return (body, Some(crate::cleared(settings, call.via)));
    }
    if call.route != Route::Link && call.route != Route::Service {
        return (body, None);
    }
    let mut reader = skein_lib::Reader::new(&body);
    let Ok(w::SignInAnswer::SignedIn(value)) = w::SignInAnswer::decode(&w::CEILINGS, &mut reader) else {
        return (body, None);
    };
    if reader.remaining() != 0 {
        return (body, None);
    }
    let mut parts = value.into_parts();
    let Some(credential) = parts.token.take() else { return (body, None) };
    let life = skein_lib::Duration::from_nanos(parts.expires.saturating_sub(wall.as_nanos()));
    let cookie = crate::cookie(settings, call.via, &credential, life);
    let answer = w::SignInAnswer::SignedIn(w::SignedIn::new(&w::CEILINGS, parts).expect("token removed"));
    let mut bytes = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("short sign-in"));
    answer.encode(&mut bytes).expect("measured answer");
    (bytes.finish(), Some(cookie))
}
