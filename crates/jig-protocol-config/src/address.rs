//! Operator addresses (configuration.md, 10); only startup resolves host names.
use crate::{Path, Problems, What, read};
use alloc::boxed::Box;
use core::net::SocketAddr;
use skein_io::{Addr, Name};
use skein_json::Document;
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Address {
    Addr(Addr),
    Host { name: Box<[u8]>, port: u16 },
}

#[expect(clippy::disallowed_methods, reason = "configuration is the protocol layer's parsing (configuration.md, 10)")]
fn socket(bytes: &[u8]) -> Result<SocketAddr, What> {
    let text = core::str::from_utf8(bytes).or(Err(What::Form))?;
    text.parse().or(Err(What::Form))
}

pub fn read(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Address> {
    let mut object = match read::Object::new(document, at) {
        Ok(object) => object,
        Err(what) => {
            problems.push(path.clone(), what);
            return None;
        }
    };
    let socket_at = object.member(document, b"socket");
    let host_at = object.member(document, b"host");
    let process_at = object.member(document, b"in_process");
    let forms = u8::from(socket_at.is_some()).checked_add(u8::from(host_at.is_some())).expect("three forms");
    let forms = forms.checked_add(u8::from(process_at.is_some())).expect("three forms");
    let mut result = None;
    if forms != 1 {
        problems.push(path.clone(), What::Form);
    } else if let Some(at) = socket_at {
        let value = match read::text(document, at, 128) {
            Ok(text) => socket(&text),
            Err(what) => Err(what),
        };
        match value {
            Ok(addr) => result = Some(Address::Addr(Addr::Ip(addr))),
            Err(what) => problems.push(path.member(b"socket"), what),
        }
    } else if let Some(at) = process_at {
        let value = read::text(document, at, u32::try_from(Name::MAX).expect("name bound fits u32"));
        match value {
            Ok(text) => match Name::new(&text) {
                Some(name) => result = Some(Address::Addr(Addr::InProcess(name))),
                None => problems.push(path.member(b"in_process"), What::Bound),
            },
            Err(what) => problems.push(path.member(b"in_process"), what),
        }
    } else if let Some(at) = host_at {
        let name = match read::text(document, at, 253) {
            Ok(name) => Some(name),
            Err(what) => {
                problems.push(path.member(b"host"), what);
                None
            }
        };
        let port_at = object.required(document, b"port", path, problems);
        let port = match port_at {
            Some(at) => match read::integer(document, at, u64::from(u16::MAX)) {
                Ok(port) => Some(u16::try_from(port).expect("port bound checked")),
                Err(what) => {
                    problems.push(path.member(b"port"), what);
                    None
                }
            },
            None => None,
        };
        if let Some(name) = name
            && let Some(port) = port
        {
            result = Some(Address::Host { name, port });
        }
    }
    object.finish(document, path, problems);
    result
}
