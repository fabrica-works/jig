use crate::Mapping;
use alloc::boxed::Box;
use jig_client_protocol_link::Message;
use skein_lib::Secret;
pub(crate) const LINK_SECRET: [u8; 32] = [3; 32];
const CREDENTIAL: [u8; 40] = [9; 40];
pub(crate) fn mapping() -> Mapping {
    Mapping::new(b"/jig").unwrap()
}
pub(crate) fn credential() -> Secret {
    Secret::new(Box::new(CREDENTIAL))
}
pub(crate) fn messages() -> [Message; 9] {
    [
        Message::Request(Box::new([])),
        Message::Read(Box::new([])),
        Message::Watch(Box::new([])),
        Message::Unwatch(Box::new([])),
        Message::StreamGone(Box::new([])),
        Message::SignInLink(Box::new([])),
        Message::SignInService,
        Message::SignOut,
        Message::Authorize(Box::new([])),
    ]
}

pub(crate) const RECORDS: [&[u8]; 3] = [b"first", b"second", b"last"];
pub(crate) const WHOLE_STREAM: &[u8] = b"data: Zmlyc3Q=\n\ndata: c2Vjb25k\n\ndata: bGFzdA==\n\n";
pub(crate) fn stream_limits() -> crate::Limits {
    crate::Limits { record: 64, chunk: 1024 }
}
pub(crate) fn reader() -> (crate::Stream, skein_lib::Queue<jig_client_protocol_link::Heard>) {
    let mut stream = crate::Stream::new(&stream_limits());
    let mut out = skein_lib::Queue::with_capacity(64);
    stream.opened(200, &mut out);
    (stream, out)
}
