//! One HTTP server and its bounded request and response (protocol/client.md, 4).
use crate::{Limits, Request, dispatch::Route};
use alloc::boxed::Box;
use skein_http::server as http;
use skein_lib::{
    Env, Intake, Queue, Token, bytes,
    stream::{Down, Read},
};
#[derive(Debug)]
pub(crate) struct Call {
    pub name: Token,
    pub stream: Option<crate::stream::Stream>,
    pub events: Queue<http::Event>,
    pub requests: Queue<http::Request>,
    pub body_kind: http::Body,
    pub credential: Option<skein_lib::Secret>,
    pub browser: bool,
    pub body: Intake,
    pub route: Route,
    pub response: Box<[u8]>,
    pub offset: usize,
    pub closed: bool,
    pub streaming: bool,
}
impl Call {
    pub fn new(name: Token, limits: &Limits) -> Self {
        Self {
            name,
            stream: None,
            events: Queue::with_capacity(8),
            requests: Queue::with_capacity(8),
            body_kind: http::Body::None,
            credential: None,
            browser: false,
            body: Intake::with_capacity(limits.body),
            route: Route::Missing,
            response: Box::new([]),
            offset: 0,
            closed: false,
            streaming: false,
        }
    }
    pub fn read(&mut self, limits: &Limits) {
        let read = match self.body_kind {
            http::Body::Length(length) => Read::Fill(
                u32::try_from(length)
                    .expect("checked HTTP body")
                    .saturating_sub(self.body.len())
                    .min(limits.read)
                    .max(1),
            ),
            http::Body::None | http::Body::Chunked => Read::Arrived { max: limits.read },
        };
        self.requests.push(http::Request::Body(Down::Demand { read, room: 0 }));
    }

    pub fn respond(&mut self, status: u16, body: Box<[u8]>, limits: &Limits) {
        self.respond_with_cookie(status, body, None, limits);
    }
    pub fn respond_with_cookie(
        &mut self,
        status: u16,
        body: Box<[u8]>,
        cookie: Option<skein_http::Header>,
        limits: &Limits,
    ) {
        self.response = body;
        self.offset = 0;
        self.requests.push(http::Request::Discard);
        let content = skein_http::Header {
            name: bytes::copy_of(b"content-type"),
            value: bytes::copy_of(b"application/octet-stream"),
        };
        let cache = skein_http::Header { name: bytes::copy_of(b"cache-control"), value: bytes::copy_of(b"no-store") };
        let nosniff =
            skein_http::Header { name: bytes::copy_of(b"x-content-type-options"), value: bytes::copy_of(b"nosniff") };
        let headers = match cookie {
            Some(cookie) => Box::from([content, cache, nosniff, cookie]),
            None => Box::from([content, cache, nosniff]),
        };
        self.requests.push(http::Request::Respond(http::Response {
            status,
            headers,
            body: http::Body::Length(u64::try_from(self.response.len()).expect("bounded response")),
            close: false,
        }));
        if self.response.is_empty() {
            self.requests.push(http::Request::Reply(Down::Finish));
        } else {
            self.room(limits);
        }
    }
    fn room(&mut self, limits: &Limits) {
        let left = self.response.len().checked_sub(self.offset).expect("response offset");
        self.requests.push(http::Request::Reply(Down::Demand {
            read: Read::Nothing,
            room: limits.send.min(u32::try_from(left).expect("bounded response")),
        }));
    }
    pub fn send(&mut self, limits: &Limits) {
        let end = self.offset.saturating_add(usize::try_from(limits.send).expect("u32 fits")).min(self.response.len());
        self.requests.push(http::Request::Reply(Down::Send(bytes::copy_of(
            self.response.get(self.offset..end).expect("response slice"),
        ))));
        self.offset = end;
        if end == self.response.len() {
            self.requests.push(http::Request::Reply(Down::Finish));
        } else {
            self.room(limits);
        }
    }
    fn finish(&mut self, out: &mut Queue<Request>) {
        self.closed = true;
        if self.streaming {
            out.push(Request::Component(jig_protocol_engine_clients::FromTransport::StreamGone { stream: self.name }));
        }
        out.push(Request::Http { call: self.name, request: http::Request::Next });
    }
    pub fn http(&mut self, env: &Env<Limits>, event: http::Event, out: &mut Queue<Request>) {
        use skein_lib::stream::Up;
        match event {
            http::Event::Body(Up::Bytes(bytes)) => {
                if self.body.append(&bytes).is_err() {
                    self.route = Route::Missing;
                    self.respond(413, Box::new([]), &env.limits);
                } else {
                    self.read(&env.limits);
                }
            }
            http::Event::Body(Up::End) => {
                if self.route != Route::Missing {
                    let body = self.body.meet(Read::Fill(self.body.len())).unwrap_or_default();
                    let credential = self.credential.take();
                    let message = match crate::dispatch::message(self.route, body) {
                        Some(message) => jig_protocol_engine_clients::FromTransport::Exchange {
                            exchange: self.name,
                            credential,
                            message,
                        },
                        None => {
                            self.streaming = true;
                            jig_protocol_engine_clients::FromTransport::StreamOpen { stream: self.name, credential }
                        }
                    };
                    out.push(Request::Component(message));
                }
            }
            http::Event::Reply(up) if self.stream.is_some() => {
                let stream = self.stream.as_mut().expect("stream writer");
                skein_http::sse::writer::up(
                    &mut stream.writer,
                    &crate::stream::env(env),
                    up,
                    &mut stream.above,
                    &mut stream.below,
                );
            }
            http::Event::Reply(Up::Room) => self.send(&env.limits),
            http::Event::Done(_)
            | http::Event::Ended
            | http::Event::Failed(_)
            | http::Event::Refused(_)
            | http::Event::Closed
            | http::Event::Body(Up::Failed(_) | Up::Room)
            | http::Event::Reply(Up::Failed(_) | Up::End | Up::Bytes(_)) => self.finish(out),
            http::Event::Call(_) => unreachable!("server supplies one head"),
        }
    }
}
