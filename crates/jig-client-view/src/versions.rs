//! Compatibility from the protocol stream hello; contract.md, 8 and protocol/client.md, 9.
//! Frame supplies identity and business facts, never a version range.
use crate::{Domain, Request, limits::ui::Limits, link::Link, page::Page, watches::Following};
use skein_lib::{Env, Queue};

pub(crate) fn hello(domain: &mut Domain, env: &Env<Limits>, served: bool, out: &mut Queue<Request>) {
    if domain.blocked {
        return;
    }
    let first = domain.versions.is_none();
    domain.versions = Some(served);
    if !served {
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
                Following::Opening | Following::Closing | Following::Closed => false,
            },
            None => false,
        },
        None => false,
    };
    if first
        && live
        && let Some(frame) = domain.frame.document.take()
    {
        crate::ui::frame_delivery(domain, env, frame, out);
    }
}
