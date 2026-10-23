//! One live link, fresh bearer tokens, and durable digest admission (client.md, 8).
use crate::{Kept, SecretOwner, routes::Route};
use alloc::boxed::Box;
use jig_client_protocol_wire::v13 as w;
use skein_lib::{CryptoRng, Map, Secret, Time, Token, Writer};
#[derive(Debug)]
pub(crate) struct Link {
    pub secret: Secret,
    pub until: Time,
}
#[derive(Debug)]
pub(crate) enum Waiting {
    Core,
    SignOut { sign_in: u64, committed: bool },
    Watch { sign_in: u64, watcher: Token, opening: bool },
    Keep { sign_in: u64, owner: SecretOwner, answer: Answer },
    Get { owner: SecretOwner, digest: Secret, body: Box<[u8]>, route: Route },
    Made { sign_in: u64, party: u64 },
    Read { sign_in: u64, what: Box<jig_domain_core::reads::Read> },
    ApplicationRead { sign_in: u64, family: u16, version: u32 },
    Ask { sign_in: u64, family: Option<u16>, version: u32 },
    // A request whose carried value the store is keeping: the core sees
    // `request` only once the value is durable (store.md 9).
    Carry { sign_in: u64, family: Option<u16>, version: u32, request: Box<CarriedRequest> },
    Checked { sign_in: u64, body: Box<[u8]>, route: Route },
}
/// An admitted value's request, without the value or a speculative carried number.
#[derive(Debug)]
pub(crate) enum CarriedRequest {
    Application(crate::ToApplication),
    AccountKey { key: [u8; 16], ask: Box<jig_domain_people::Ask>, digest: [u8; 32] },
}
#[derive(Debug)]
pub(crate) enum Answer {
    SignedIn(w::SignedIn),
    Minted { credential: Secret, expires: skein_lib::Wall },
    Service(w::KeyedAnswer),
}
#[derive(Debug)]
pub(crate) struct Pending {
    pub exchanges: Map<Token, Waiting>,
    pub mints: Map<Token, Token>,
    /// The values this component's decodes kept, by exchange: none while
    /// the store keeps it, then its number until the core answers or
    /// leaves it unused (store.md 9).
    pub carried: Map<Token, Option<u64>>,
}
pub(crate) fn draw(rng: &mut CryptoRng) -> Secret {
    let mut bytes = [0; 32];
    rng.fill(&mut bytes);
    Secret::new(Box::new(bytes))
}
pub(crate) fn token(rng: &mut CryptoRng, number: u64) -> Secret {
    let mut out = Writer::new(40);
    out.put(&number.to_be_bytes()).expect("eight bytes");
    out.put(draw(rng).expose()).expect("32 bytes");
    Secret::new(out.finish())
}
pub(crate) fn digest(bytes: &[u8]) -> Secret {
    Secret::new(Box::new(skein_lib::sha256::digest(bytes)))
}
/// A carrying request's digest: its record and its value, each after its
/// length, so the same key sent with another value is a conflict.
pub(crate) fn carrying(record: &[u8], value: &Secret) -> [u8; 32] {
    let mut hash = skein_lib::sha256::Sha256::new();
    for part in [record, value.expose()] {
        hash.update(&u64::try_from(part.len()).expect("usize fits u64").to_be_bytes());
        hash.update(part);
    }
    hash.finish()
}
pub(crate) fn credential(value: Option<&Secret>) -> Option<(u64, Secret)> {
    let bytes = value?.expose();
    if bytes.len() != 40 {
        return None;
    }
    let number = u64::from_be_bytes(bytes.get(..8)?.try_into().ok()?);
    Some((number, digest(bytes)))
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
