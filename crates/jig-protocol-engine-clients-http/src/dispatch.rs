//! Method and route dispatch (protocol/client.md, 17.1).
use alloc::boxed::Box;
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
