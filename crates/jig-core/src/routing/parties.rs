//! Party holdings continued through the task hub and connectors
//! (`domain/people.md`, section 5.1).

use super::{Event, Limits, Request, Requests, created_holdings};
use crate::Core;
use skein_lib::{Env, Queue, Token};

use alloc::boxed::Box;
use jig_core_tasks as tasks;

pub(super) fn route_holdings(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    connector: u16,
    holdings: Option<Box<[tasks::Holding]>>,
) -> Requests {
    created_holdings(core, env, work, request, connector, holdings);
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}
