//! Compatibility from the protocol stream hello; contract.md, 8 and protocol/client.md, 9.
//! Frame supplies identity and business facts, never a version range.
use crate::{
    Domain, Request,
    contract::{CLIENT_VERSION, VersionRange},
    limits::Limits,
    link::Link,
    pages::Page,
    watches::Following,
};
use skein_lib::{Env, Queue};

pub(crate) fn hello(domain: &mut Domain, env: &Env<Limits>, versions: VersionRange, out: &mut Queue<Request>) {
    if domain.blocked {
        return;
    }
    let first = domain.versions.is_none();
    domain.versions = Some(versions);
    if !versions.contains(CLIENT_VERSION) {
        domain.blocked = true;
        domain.confirmed = None;
        domain.page = Page::OutOfDate;
        domain.close_all(out);
        domain.set_link(Link::Offline, out);
        out.push(Request::Confirmed { party: None });
        domain.changed();
        return;
    }
    let live = match domain.frame_watch {
        Some(id) => match domain.watches.get(id) {
            Some(watch) => match watch.state {
                Following::Live => true,
                Following::Opening { .. } | Following::Reopening { .. } | Following::Closing | Following::Closed => {
                    false
                }
            },
            None => false,
        },
        None => false,
    };
    if first
        && live
        && let Some(frame) = domain.frame.document.take()
    {
        crate::domain::frame_delivery(domain, env, frame, out);
    }
}
