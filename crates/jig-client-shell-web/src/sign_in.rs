//! Take the printed link out of history before starting the host (protocol/browser.md, 11).
use skein_lib::base64::{self, Alphabet};
#[derive(Debug)]
pub struct Route<'a> {
    pub path: &'a [u8],
    pub home: &'a [u8],
}
#[derive(Debug, PartialEq, Eq)]
pub struct Taken {
    pub secret: Box<[u8]>,
    pub address: Box<[u8]>,
}
/// Read the mapping's base64url secret only at the configured link route.
#[must_use]
pub fn take(route: &Route<'_>, path_and_query: &[u8]) -> Option<Taken> {
    let text = std::str::from_utf8(path_and_query).ok()?;
    let (path, query) = text.split_once('?')?;
    if path.as_bytes() != route.path {
        return None;
    }
    let value = query.split('&').find_map(|part| part.strip_prefix("secret="))?;
    let secret = base64::decode(Alphabet::Url, value.as_bytes(), 32).ok()?;
    if secret.len() != 32 {
        return None;
    }
    Some(Taken { secret, address: route.home.into() })
}
#[cfg(target_arch = "wasm32")]
pub mod web {
    pub use crate::web::{show_once, take_from_page};
}
