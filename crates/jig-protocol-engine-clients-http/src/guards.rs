//! The example browser guards (protocol/browser.md, 12.2).
use alloc::boxed::Box;
use skein_http::{Method, server::Call};
use skein_lib::List;
pub(crate) fn host(settings: &crate::Settings, hosts: &List<Box<[u8]>>, call: &Call) -> bool {
    let Some(host) = call.header(b"host") else { return false };
    if crate::via(settings, call) == crate::Via::Public {
        return true;
    }
    for accepted in hosts {
        if accepted.as_ref() == host {
            return true;
        }
    }
    false
}
pub(crate) fn content(call: &Call) -> bool {
    call.method != Method::Post || call.header(b"content-type") == Some(b"application/octet-stream")
}
