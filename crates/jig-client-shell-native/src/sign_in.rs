//! The printed link and the shell's own exchange outcomes (protocol/client.md, 8).
use skein_lib::{
    Secret,
    base64::{self, Alphabet},
};
#[derive(Debug, PartialEq, Eq)]
pub enum Unlinked {
    NotHttp,
    NoSecret,
    Address,
}
pub(crate) fn parse(link: &[u8]) -> Result<(String, Secret), Unlinked> {
    let text = std::str::from_utf8(link).map_err(|_| Unlinked::Address)?;
    let rest = text.strip_prefix("http://").ok_or(Unlinked::NotHttp)?;
    let (host, path) = rest.split_once('/').ok_or(Unlinked::NoSecret)?;
    let _: std::net::SocketAddr = host.parse().map_err(|_| Unlinked::Address)?;
    let secret = path.strip_prefix("jig/v1/sign-in/link?secret=").ok_or(Unlinked::NoSecret)?;
    let secret = base64::decode(Alphabet::Url, secret.as_bytes(), 32).map_err(|_| Unlinked::NoSecret)?;
    if secret.len() != 32 {
        return Err(Unlinked::NoSecret);
    }
    Ok((host.to_owned(), Secret::new(secret)))
}
