//! The opaque bearer credential (protocol/client.md, 17.3).
use skein_lib::{
    Secret,
    base64::{self, Alphabet},
};
pub(crate) fn read(call: &skein_http::server::Call) -> Option<Secret> {
    let bytes = base64::decode(Alphabet::Url, call.header(b"authorization")?.strip_prefix(b"Bearer ")?, 40).ok()?;
    if bytes.len() != 40 {
        return None;
    }
    Some(Secret::new(bytes))
}
