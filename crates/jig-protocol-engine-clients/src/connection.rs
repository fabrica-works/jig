//! One HTTP server and its bounded request and response (protocol/client.md, 4).
use crate::{Limits, Request, routes::Route};
use alloc::boxed::Box;
use skein_http::server as http;
use skein_lib::{
    Env, Intake, Queue, Time, Token, bytes,
    stream::{Down, Read, Up},
};
#[derive(Debug)]
pub(crate) struct Connection {
    pub socket: Token,
    pub stream: Option<Token>,
    pub http: http::Server,
    pub events: Queue<http::Event>,
    pub requests: Queue<http::Request>,
    pub below: Queue<Down>,
    pub call: Option<http::Call>,
    pub body: Intake,
    pub route: Route,
    pub exchange: Option<Token>,
    pub response: Box<[u8]>,
    pub offset: usize,
    pub closing: bool,
    pub draining: bool,
    pub deadline: Time,
}
impl Connection {
    pub fn new(socket: Token, limits: &Limits, now: Time) -> Self {
        let mut requests = Queue::with_capacity(8);
        requests.push(http::Request::Next);
        Self {
            socket,
            stream: None,
            http: http::Server::new(&limits.http),
            events: Queue::with_capacity(8),
            requests,
            below: Queue::with_capacity(8),
            call: None,
            body: Intake::with_capacity(u32::try_from(limits.http.body).expect("checked body cap")),
            route: Route::Missing,
            exchange: None,
            response: Box::new([]),
            offset: 0,
            closing: false,
            draining: false,
            deadline: now.saturating_add(limits.stall),
        }
    }
    pub fn feed(&mut self, env: &Env<Limits>, up: Up) {
        self.deadline = env.now.saturating_add(env.limits.stall);
        http::up(&mut self.http, &http_env(env), up, &mut self.events, &mut self.below);
    }
    pub fn progress(&mut self, env: &Env<Limits>, out: &mut Queue<Request>) -> bool {
        if let Some(down) = self.below.pop() {
            out.push(Request::Io(skein_io::Request::Stream { stream: self.socket, down }));
            return true;
        }
        if let Some(request) = self.requests.pop() {
            http::down(&mut self.http, &http_env(env), request, &mut self.events, &mut self.below);
            return true;
        }
        false
    }
    pub fn read(&mut self, limits: &Limits) {
        let read = match self.call.as_ref().expect("reading a call").body {
            http::Body::Length(length) => Read::Fill(
                u32::try_from(length)
                    .expect("checked HTTP body")
                    .saturating_sub(self.body.len())
                    .min(limits.http.read)
                    .max(1),
            ),
            http::Body::None | http::Body::Chunked => Read::Arrived { max: limits.http.read },
        };
        self.requests.push(http::Request::Body(Down::Demand { read, room: 0 }));
    }

    pub fn respond(&mut self, status: u16, body: Box<[u8]>, limits: &Limits) {
        self.response = body;
        self.offset = 0;
        self.requests.push(http::Request::Discard);
        self.requests.push(http::Request::Respond(http::Response {
            status,
            headers: Box::new([skein_http::Header {
                name: bytes::copy_of(b"content-type"),
                value: bytes::copy_of(b"application/octet-stream"),
            }]),
            body: http::Body::Length(u64::try_from(self.response.len()).expect("bounded response")),
            close: false,
        }));
        if !self.response.is_empty() {
            self.room(limits);
        }
    }
    fn room(&mut self, limits: &Limits) {
        let left = self.response.len().checked_sub(self.offset).expect("response offset");
        self.requests.push(http::Request::Reply(Down::Demand {
            read: Read::Nothing,
            room: limits.http.send.min(u32::try_from(left).expect("bounded response")),
        }));
    }
    pub fn send(&mut self, limits: &Limits) {
        let end =
            self.offset.saturating_add(usize::try_from(limits.http.send).expect("u32 fits")).min(self.response.len());
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
    pub fn finish(&mut self, out: &mut Queue<Request>) {
        if !self.closing {
            self.closing = true;
            self.draining = true;
            out.push(Request::Io(skein_io::Request::Close { entity: self.socket }));
        }
    }
    pub fn close(&mut self, out: &mut Queue<Request>) {
        if !self.closing || self.draining {
            self.closing = true;
            self.draining = false;
            out.push(Request::Io(skein_io::Request::Abort { entity: self.socket }));
        }
    }
}
fn http_env(env: &Env<Limits>) -> Env<http::Limits> {
    Env { now: env.now, wall: env.wall, limits: env.limits.http }
}
