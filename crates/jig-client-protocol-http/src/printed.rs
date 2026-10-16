//! The printed sign-in URL is part of the mapping (protocol/client.md, 8).
use crate::Mapping;
use alloc::boxed::Box;
use skein_lib::{
    Secret,
    base64::{self, Alphabet},
    bytes,
};
#[derive(Debug, PartialEq, Eq)]
pub struct Printed {
    pub host: Box<[u8]>,
    pub secret: Secret,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Unlinked {
    NotHttp,
    NoSecret,
    Address,
}
#[expect(clippy::disallowed_methods, reason = "the HTTP mapping parses its printed URL")]
pub fn printed(mapping: &Mapping, url: &[u8]) -> Result<Printed, Unlinked> {
    let text = core::str::from_utf8(url).or(Err(Unlinked::Address))?;
    let rest = text.strip_prefix("http://").ok_or(Unlinked::NotHttp)?;
    let (host, path) = rest.split_once('/').ok_or(Unlinked::NoSecret)?;
    let _: core::net::SocketAddr = host.parse().or(Err(Unlinked::Address))?;
    let prefix = mapping.prefix.get(1..).expect("validated prefix");
    let path = path.as_bytes().strip_prefix(prefix).ok_or(Unlinked::NoSecret)?;
    let secret = path.strip_prefix(b"/v1/sign-in/link?secret=").ok_or(Unlinked::NoSecret)?;
    let secret = base64::decode(Alphabet::Url, secret, 32).or(Err(Unlinked::NoSecret))?;
    if secret.len() != 32 {
        return Err(Unlinked::NoSecret);
    }
    Ok(Printed { host: bytes::copy_of(host.as_bytes()), secret: Secret::new(secret) })
}
