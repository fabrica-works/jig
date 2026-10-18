//! Bundle routing and bounded reply chunks (skein's ui-web.md, 4.4).
use super::{Answer, Connection, policy};
use alloc::boxed::Box;
use skein_http::{Header, Method, server as http};
use skein_lib::{
    List, Map, bytes,
    stream::{Down, Read},
};

/// One digest-directory file, typed by the closed extension list.
#[derive(Debug)]
pub struct File {
    pub kind: &'static [u8],
    pub bytes: Box<[u8]>,
}
/// Bounded browser data read by startup (protocol/service.md, 7).
#[derive(Debug)]
pub struct Bundle {
    pub page: Box<[u8]>,
    pub etag: Box<[u8]>,
    pub digest: Box<[u8]>,
    pub files: Map<Box<[u8]>, File>,
}
#[derive(Debug)]
pub(super) struct Reply {
    answer: Answer,
    path: Box<[u8]>,
    offset: usize,
}
#[must_use]
pub fn kind(path: &[u8]) -> Option<&'static [u8]> {
    let mut dot = None;
    for (at, byte) in path.iter().enumerate() {
        if *byte == b'.' {
            dot = Some(at);
        }
        if *byte == b'/' {
            dot = None;
        }
    }
    let dot = dot?;
    match path.get(dot.checked_add(1)?..)? {
        b"html" => Some(b"text/html; charset=utf-8"),
        b"js" => Some(b"text/javascript"),
        b"wasm" => Some(b"application/wasm"),
        b"css" => Some(b"text/css"),
        b"svg" => Some(b"image/svg+xml"),
        b"png" => Some(b"image/png"),
        b"webp" => Some(b"image/webp"),
        b"ico" => Some(b"image/x-icon"),
        b"woff2" => Some(b"font/woff2"),
        b"txt" => Some(b"text/plain; charset=utf-8"),
        b"webmanifest" => Some(b"application/manifest+json"),
        _ => None,
    }
}
fn target(call: &http::Call) -> &[u8] {
    match bytes::find(&call.target, b"?") {
        Some(at) => call.target.get(..at).expect("query position"),
        None => &call.target,
    }
}
impl Bundle {
    fn file(&self, path: &[u8]) -> Option<&File> {
        let name = if path == b"/robots.txt" {
            b"files/robots.txt".as_slice()
        } else {
            let rest = path.strip_prefix(b"/bundle/")?;
            let slash = bytes::find(rest, b"/")?;
            if rest.get(..slash)? != self.digest.as_ref() {
                return None;
            }
            rest.get(slash.checked_add(1)?..)?
        };
        for (path, file) in &self.files {
            if path.as_ref() == name {
                return Some(file);
            }
        }
        None
    }
    fn content(&self, reply: &Reply) -> &[u8] {
        match reply.answer {
            Answer::File | Answer::Root => &self.file(&reply.path).expect("routed file").bytes,
            Answer::Page => &self.page,
            Answer::Mount | Answer::NotModified | Answer::NotFound | Answer::NotAllowed => b"",
        }
    }
}
pub(super) fn route(bundle: Option<&Bundle>, call: &http::Call) -> Answer {
    if jig_protocol_engine_clients_http::claims(&crate::clients::http_settings(), call) {
        return Answer::Mount;
    }
    if call.method != Method::Get {
        return Answer::NotAllowed;
    }
    let Some(bundle) = bundle else { return Answer::NotFound };
    let path = target(call);
    if path.starts_with(b"/bundle/") {
        return if bundle.file(path).is_some() { Answer::File } else { Answer::NotFound };
    }
    if path == b"/robots.txt" && bundle.file(path).is_none() {
        return Answer::NotFound;
    }
    for header in &call.headers {
        if header.name.eq_ignore_ascii_case(b"if-none-match") && header.value == bundle.etag {
            return Answer::NotModified;
        }
    }
    if path == b"/robots.txt" { Answer::Root } else { Answer::Page }
}
fn field(headers: &mut List<Header>, name: &[u8], value: &[u8]) {
    headers.push(Header { name: bytes::copy_of(name), value: bytes::copy_of(value) }).expect("answer fields");
}
pub(super) fn start(bundle: Option<&Bundle>, head: &http::Call, answer: Answer, conn: &mut Connection, send: u32) {
    let reply = Reply { answer, path: bytes::copy_of(target(head)), offset: 0 };
    let mut headers = List::with_capacity(3);
    let mut length = 0;
    let status = match answer {
        Answer::Page | Answer::File | Answer::Root => {
            let bundle = bundle.expect("routed bundle");
            let kind = match answer {
                Answer::File | Answer::Root => bundle.file(&reply.path).expect("routed file").kind,
                Answer::Page => b"text/html; charset=utf-8",
                Answer::Mount | Answer::NotModified | Answer::NotFound | Answer::NotAllowed => {
                    unreachable!("content answer")
                }
            };
            field(&mut headers, b"Content-Type", kind);
            length = bundle.content(&reply).len();
            200
        }
        Answer::NotModified => 304,
        Answer::NotFound => 404,
        Answer::NotAllowed => 405,
        Answer::Mount => unreachable!("mount answers itself"),
    };
    match answer {
        Answer::File => field(&mut headers, b"Cache-Control", b"public, max-age=31536000, immutable"),
        Answer::Page | Answer::Root | Answer::NotModified => {
            field(&mut headers, b"Cache-Control", b"no-cache");
            field(&mut headers, b"ETag", &bundle.expect("revalidated bundle").etag);
        }
        Answer::Mount | Answer::NotFound | Answer::NotAllowed => {}
    }
    let mut response = http::Response {
        status,
        headers: headers.into_boxed(),
        body: if status == 304 {
            http::Body::None
        } else {
            http::Body::Length(u64::try_from(length).expect("bounded file"))
        },
        close: false,
    };
    policy::add(&mut response);
    conn.requests.push(http::Request::Discard);
    conn.requests.push(http::Request::Respond(response));
    if length != 0 {
        demand(conn, length, send);
    }
    conn.reply = Some(reply);
}
fn demand(conn: &mut Connection, left: usize, send: u32) {
    conn.requests.push(http::Request::Reply(Down::Demand {
        read: Read::Nothing,
        room: send.min(u32::try_from(left).expect("bounded file")),
    }));
}
pub(super) fn send(bundle: &Bundle, conn: &mut Connection, send: u32) {
    let reply = conn.reply.as_mut().expect("own reply");
    let body = bundle.content(reply);
    let end = reply.offset.saturating_add(usize::try_from(send).expect("send fits")).min(body.len());
    let bytes = bytes::copy_of(body.get(reply.offset..end).expect("reply slice"));
    reply.offset = end;
    let left = body.len().checked_sub(end).expect("reply cursor");
    conn.requests.push(http::Request::Reply(Down::Send(bytes)));
    if left == 0 {
        conn.requests.push(http::Request::Reply(Down::Finish));
    } else {
        demand(conn, left, send);
    }
}
