//! Opaque bearer and browser credentials (protocol/client.md, 17.3).
use crate::Settings;
use alloc::boxed::Box;
use skein_lib::{
    Decimal, Duration, Secret, Writer,
    base64::{self, Alphabet},
};
#[derive(Debug)]
pub(crate) enum Read {
    Bearer(Secret),
    Cookie(Secret),
    None,
}
pub(crate) fn read(call: &skein_http::server::Call) -> Read {
    if let Some(header) = call.header(b"authorization")
        && let Some(value) = header.strip_prefix(b"Bearer ")
        && let Some(secret) = decode(value)
    {
        return Read::Bearer(secret);
    }
    if let Some(header) = call.header(b"cookie") {
        let mut start = 0;
        for end in 0..=header.len() {
            if end == header.len() || header.get(end) == Some(&b';') {
                let pair = header.get(start..end).expect("cookie pair").trim_ascii();
                if let Some(value) = pair.strip_prefix(b"jig-sign-in=")
                    && let Some(secret) = decode(value)
                {
                    return Read::Cookie(secret);
                }
                start = end.saturating_add(1);
            }
        }
    }
    Read::None
}
fn decode(value: &[u8]) -> Option<Secret> {
    let bytes = base64::decode(Alphabet::Url, value, 40).ok()?;
    if bytes.len() != 40 {
        return None;
    }
    Some(Secret::new(bytes))
}
/// Set the mapping's cookie after a sign-in or `Minted` (protocol/client.md, 17.3).
#[must_use]
pub fn cookie(settings: &Settings, credential: &Secret, life: Duration) -> skein_http::Header {
    header(settings, credential.expose(), life.as_nanos().checked_div(1_000_000_000).expect("seconds"))
}
/// Clear the mapping's cookie when signing out.
#[must_use]
pub fn cleared(settings: &Settings) -> skein_http::Header {
    header(settings, &[], 0)
}
fn header(settings: &Settings, credential: &[u8], seconds: u64) -> skein_http::Header {
    let encoded = base64::encoded_len(Alphabet::Url, u32::try_from(credential.len()).expect("bounded credential"))
        .expect("credential base64");
    let age = Decimal::of(seconds);
    let attributes = b"; HttpOnly; SameSite=Strict; Max-Age=";
    let secure = if settings.loopback { b"".as_slice() } else { b"; Secure".as_slice() };
    let size = b"jig-sign-in="
        .len()
        .checked_add(usize::try_from(encoded).expect("base64 fits"))
        .expect("cookie")
        .checked_add(b"; Path=".len())
        .expect("cookie")
        .checked_add(settings.prefix.len())
        .expect("cookie")
        .checked_add(attributes.len())
        .expect("cookie")
        .checked_add(age.as_bytes().len())
        .expect("cookie")
        .checked_add(secure.len())
        .expect("cookie");
    let mut value = Writer::new(size);
    value.put(b"jig-sign-in=").expect("cookie name");
    base64::encode(Alphabet::Url, credential, &mut value).expect("credential");
    value.put(b"; Path=").expect("path attribute");
    value.put(&settings.prefix).expect("path");
    value.put(attributes).expect("attributes");
    value.put(age.as_bytes()).expect("age");
    value.put(secure).expect("secure attribute");
    skein_http::Header { name: Box::from(b"set-cookie".as_slice()), value: value.finish() }
}
