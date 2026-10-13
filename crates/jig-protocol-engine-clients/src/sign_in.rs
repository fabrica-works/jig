//! One live link, fresh bearer tokens, and durable digest admission (client.md, 8).
use crate::{Kept, Settings};
use alloc::boxed::Box;
use jig_protocol_client_wire::v5 as w;
use skein_lib::{
    CryptoRng, Map, Secret, Time, Token, Writer,
    base64::{self, Alphabet},
};
#[derive(Debug)]
pub(crate) struct Link {
    pub secret: Secret,
    pub until: Time,
}
#[derive(Debug)]
pub(crate) enum Waiting {
    Core,
    Keep { sign_in: u64, answer: w::SignedIn },
    Get { sign_in: u64, digest: Secret, body: Box<[u8]> },
    Ask { sign_in: u64 },
    Checked { sign_in: u64, body: Box<[u8]> },
}
#[derive(Debug)]
pub(crate) struct Pending {
    pub exchanges: Map<Token, Waiting>,
}
pub(crate) fn draw(rng: &mut CryptoRng) -> Secret {
    let mut bytes = [0; 32];
    rng.fill(&mut bytes);
    Secret::new(Box::new(bytes))
}
pub(crate) fn printed(settings: &Settings, secret: &Secret) -> Option<Box<[u8]>> {
    let address = settings.listener.socket()?;
    let host = address_text(address)?;
    let encoded = url(secret.expose());
    let parts: [&[u8]; 4] = [b"http://", &host, b"/jig/v1/sign-in/link?secret=", &encoded];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len())?;
    }
    let mut out = Writer::new(length);
    for part in parts {
        out.put(part).ok()?;
    }
    Some(out.finish())
}
pub(crate) fn url(bytes: &[u8]) -> Box<[u8]> {
    let length = base64::encoded_len(Alphabet::Url, u32::try_from(bytes.len()).expect("short secret"))
        .expect("short encoded secret");
    let mut out = Writer::new(usize::try_from(length).expect("u32 fits"));
    base64::encode(Alphabet::Url, bytes, &mut out).expect("measured base64");
    out.finish()
}
pub(crate) fn token(rng: &mut CryptoRng, number: u64) -> Secret {
    let mut out = Writer::new(40);
    out.put(&number.to_be_bytes()).expect("eight bytes");
    out.put(draw(rng).expose()).expect("32 bytes");
    Secret::new(out.finish())
}
pub(crate) fn digest(bytes: &[u8]) -> Secret {
    let mut out = Writer::new(32);
    for word in skein_io::digest::digest(bytes).0 {
        out.put(&word.to_be_bytes()).expect("four words");
    }
    Secret::new(out.finish())
}
pub(crate) fn bearer(header: Option<&[u8]>) -> Option<(u64, Secret)> {
    let bytes = base64::decode(Alphabet::Url, header?.strip_prefix(b"Bearer ")?, 40).ok()?;
    if bytes.len() != 40 {
        return None;
    }
    let number = u64::from_be_bytes(bytes.get(..8)?.try_into().ok()?);
    Some((number, digest(&bytes)))
}
pub(crate) fn accepted(value: Option<&Kept>, digest: &Secret, wall: skein_lib::Wall) -> bool {
    match value {
        Some(value) => value.expires > wall && constant_time_eq(value.digest.expose(), digest.expose()),
        None => false,
    }
}
pub(crate) fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut different = 0_u8;
    for (a, b) in left.iter().zip(right) {
        different |= a ^ b;
    }
    different == 0
}

fn address_text(address: core::net::SocketAddr) -> Option<Box<[u8]>> {
    let mut text = skein_lib::Intake::with_capacity(64);
    match address.ip() {
        core::net::IpAddr::V4(ip) => {
            for (at, octet) in ip.octets().iter().enumerate() {
                if at != 0 {
                    text.append(b".").ok()?;
                }
                text.append(skein_lib::Decimal::of(u64::from(*octet)).as_bytes()).ok()?;
            }
        }
        core::net::IpAddr::V6(ip) => {
            if !ip.is_loopback() {
                return None;
            }
            text.append(b"[::1]").ok()?;
        }
    }
    text.append(b":").ok()?;
    text.append(skein_lib::Decimal::of(u64::from(address.port())).as_bytes()).ok()?;
    text.meet(skein_lib::stream::Read::Fill(text.len()))
}
