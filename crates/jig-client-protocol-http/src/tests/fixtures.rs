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
pub(crate) fn messages() -> [Message; 7] {
    [
        Message::Request(Box::new([])),
        Message::Read(Box::new([])),
        Message::Watch(Box::new([])),
        Message::Unwatch(Box::new([])),
        Message::StreamGone(Box::new([])),
        Message::SignInLink(Box::new([])),
        Message::SignOut,
    ]
}
