//! One live link, fresh bearer tokens, and durable digest admission (client.md, 8).
use crate::{Kept, SecretOwner, Settings, routes::Route};
use alloc::boxed::Box;
use jig_client_protocol_wire::v8 as w;
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
    Provider { provider: u16 },
    SignOut { sign_in: u64, committed: bool },
    Watch { sign_in: u64, watcher: Token, opening: bool },
    Keep { sign_in: u64, owner: SecretOwner, answer: Answer },
    Get { owner: SecretOwner, digest: Secret, body: Box<[u8]>, route: Route },
    Made { sign_in: u64, party: u64 },
    Read { sign_in: u64, what: Box<jig_domain_core::reads::Read> },
    ApplicationRead { sign_in: u64, family: u16, version: u32 },
    Ask { sign_in: u64, family: Option<u16>, version: u32, amendment: Option<([u8; 16], Box<jig_domain_people::Ask>)> },
    Checked { sign_in: u64, body: Box<[u8]>, route: Route },
}
#[derive(Debug)]
pub(crate) enum Answer {
    SignedIn(w::SignedIn),
    Service(w::Answer),
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

/// v8 amendment confirmations use explicit engine proposals until jig 14.5.
pub(crate) fn propose_amend(
    kept: &mut Option<([u8; 16], Box<jig_domain_people::Ask>)>,
    reply: &jig_domain_people::Reply,
    sign_in: u64,
    exchange: Token,
) -> Option<jig_domain_people::Event> {
    use jig_domain_people as p;
    let p::Reply::Outcome(p::Outcome::Refused(p::Refusal::Authority {
        proposable: Some(p::Accepter::Role(_)), ..
    })) = reply
    else {
        return None;
    };
    let (key, ask) = kept.as_mut()?;
    let p::Ask::Amend { project, task, revision, amendment, .. } = ask.as_ref() else { return None };
    let next = p::Ask::Propose {
        project: *project,
        action: p::Action::Amend { task: *task, revision: *revision, amendment: Box::new(amendment.clone()) },
        reason: amendment.reason.clone(),
    };
    *key.last_mut().expect("fixed request key") ^= 0x80;
    **ask = next.clone();
    Some(p::Event::Ask { reply_to: skein_lib::ReplyTo::new(exchange), sign_in, key: *key, ask: next })
}
