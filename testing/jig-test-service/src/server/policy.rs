//! The reference origin's policy (protocol/browser.md, 12.1).
use alloc::boxed::Box;
use skein_http::{Header, server::Response};
use skein_lib::{List, bytes};

pub const FIELDS: [(&[u8], &[u8]); 5] = [
    (b"Content-Security-Policy", b"default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'"),
    (b"X-Content-Type-Options", b"nosniff"),
    (b"Cross-Origin-Opener-Policy", b"same-origin"),
    (b"Cross-Origin-Resource-Policy", b"same-origin"),
    (b"Referrer-Policy", b"no-referrer"),
];
pub(super) fn add(response: &mut Response) {
    let capacity = u32::try_from(response.headers.len()).expect("bounded response headers");
    let mut headers = List::with_capacity(capacity.checked_add(5).expect("policy fields"));
    for header in core::mem::replace(&mut response.headers, Box::new([])) {
        headers.push(header).expect("existing fields");
    }
    for (name, value) in FIELDS {
        let mut present = false;
        for header in &headers {
            present |= header.name.eq_ignore_ascii_case(name);
        }
        if !present {
            headers.push(Header { name: bytes::copy_of(name), value: bytes::copy_of(value) }).expect("policy room");
        }
    }
    response.headers = headers.into_boxed();
}
