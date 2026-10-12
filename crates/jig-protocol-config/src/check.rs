//! Collected field and reference checks (configuration.md, 1 and 6).
use crate::{Address, Path, Problems, What, address, read};
use alloc::boxed::Box;
use skein_io::Addr;
use skein_json::Document;
use skein_lib::{Duration, List};
pub(crate) fn object(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<read::Object> {
    match read::Object::new(document, at) {
        Ok(object) => Some(object),
        Err(what) => {
            problems.push(path.clone(), what);
            None
        }
    }
}
pub(crate) fn text(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[u8]>> {
    let at = object.required(document, name, path, problems)?;
    match read::text(document, at, 4_096) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.member(name), what);
            None
        }
    }
}
pub(crate) fn integer(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<u64> {
    let at = object.required(document, name, path, problems)?;
    match read::integer(document, at, u64::MAX) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.member(name), what);
            None
        }
    }
}
pub(crate) fn count(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<u32> {
    let value = integer(object, document, name, path, problems)?;
    match u32::try_from(value) {
        Ok(value) => Some(value),
        Err(_) => {
            problems.push(path.member(name), What::Bound);
            None
        }
    }
}
pub(crate) fn duration(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<Duration> {
    let at = object.required(document, name, path, problems)?;
    match read::duration(document, at) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.member(name), what);
            None
        }
    }
}
pub(crate) fn address(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<Address> {
    let at = object.required(document, name, path, problems)?;
    address::read(document, at, &path.member(name), problems)
}
pub(crate) fn list(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    max: u32,
    path: &Path,
    problems: &mut Problems,
) -> Option<List<u32>> {
    let at = object.required(document, name, path, problems)?;
    match read::list(document, at, max) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.member(name), what);
            None
        }
    }
}
pub(crate) fn local(address: &Address, path: &Path, problems: &mut Problems) {
    let local = match address {
        Address::Addr(Addr::Ip(socket)) => socket.ip().is_loopback(),
        Address::Addr(Addr::InProcess(_)) => true,
        Address::Host { .. } => false,
    };
    if !local {
        problems.push(path.clone(), What::Plaintext);
    }
}
