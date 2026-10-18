//! One paced SSE writer (protocol/client.md, 17.2).
use crate::{Limits, Request, call::Call};
use alloc::boxed::Box;
use jig_protocol_engine_clients::{Ended, FromTransport, Streamed};
use skein_http::{server as http, sse::writer as sse};
use skein_lib::{
    Env, Queue,
    base64::{self, Alphabet},
    bytes,
};
#[derive(Debug)]
pub(crate) struct Stream {
    pub writer: sse::Writer,
    pub ending: bool,
    pub above: Queue<sse::Event>,
    pub below: Queue<skein_lib::stream::Down>,
}
pub(crate) fn env(env: &Env<Limits>) -> Env<sse::Limits> {
    Env { now: env.now, wall: env.wall, limits: sse::Limits { event: env.limits.event, chunk: env.limits.send } }
}
impl Call {
    pub fn stream_ready(&self) -> bool {
        match &self.stream {
            Some(stream) => !stream.above.is_empty() || !stream.below.is_empty(),
            None => false,
        }
    }
    pub fn stream(&mut self, context: &Env<Limits>, message: Streamed) {
        let env = env(context);
        if self.stream.is_none() {
            self.requests.push(http::Request::Discard);
            self.requests.push(http::Request::Respond(http::Response {
                status: 200,
                headers: Box::new([
                    skein_http::Header {
                        name: bytes::copy_of(b"content-type"),
                        value: bytes::copy_of(b"text/event-stream"),
                    },
                    skein_http::Header { name: bytes::copy_of(b"cache-control"), value: bytes::copy_of(b"no-store") },
                    skein_http::Header {
                        name: bytes::copy_of(b"x-content-type-options"),
                        value: bytes::copy_of(b"nosniff"),
                    },
                ]),
                body: http::Body::Chunked,
                close: true,
            }));
            self.stream = Some(Stream {
                writer: sse::Writer::new(&env.limits),
                ending: false,
                above: Queue::with_capacity(2),
                below: Queue::with_capacity(2),
            });
        }
        let request = match message {
            Streamed::Record(record) => {
                let length =
                    base64::encoded_len(Alphabet::Standard, u32::try_from(record.len()).expect("bounded record"))
                        .expect("bounded base64");
                let mut data = skein_lib::Writer::new(usize::try_from(length).expect("u32 fits"));
                base64::encode(Alphabet::Standard, &record, &mut data).expect("measured event");
                sse::Request::Event(sse::Outgoing { name: Box::new([]), data: data.finish(), id: None, retry: None })
            }
            Streamed::Heartbeat => sse::Request::Comment(Box::new([])),
        };
        let stream = self.stream.as_mut().expect("opened writer");
        sse::down(&mut stream.writer, &env, request, &mut stream.above, &mut stream.below);
    }
    pub fn end(&mut self, context: &Env<Limits>, why: Ended) {
        if let Some(stream) = &mut self.stream {
            stream.ending = true;
            stream.finish(context);
        } else {
            self.respond(if why == Ended::SignedOut { 401 } else { 409 }, Box::new([]), &context.limits);
        }
    }
    pub fn stream_up(&mut self, context: &Env<Limits>, out: &mut Queue<Request>) {
        let stream = self.stream.as_mut().expect("ready stream");
        if let Some(down) = stream.below.pop() {
            self.requests.push(http::Request::Reply(down));
        } else if let Some(event) = stream.above.pop() {
            match event {
                sse::Event::Sent => {
                    out.push(Request::Component(FromTransport::StreamTaken { stream: self.name }));
                    stream.finish(context);
                }
                sse::Event::Failed(_) | sse::Event::Refused(_) => {
                    out.push(Request::Component(FromTransport::StreamGone { stream: self.name }));
                    self.requests.push(http::Request::Close);
                }
                sse::Event::Closed => {}
            }
        }
    }
}

impl Stream {
    fn finish(&mut self, context: &Env<Limits>) {
        if self.ending && self.writer.waiting() == sse::Waiting::Above {
            self.ending = false;
            sse::down(&mut self.writer, &env(context), sse::Request::Finish, &mut self.above, &mut self.below);
        }
    }
}
