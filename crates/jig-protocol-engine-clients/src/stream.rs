//! One paced SSE writer, with one outstanding item of each kind per watch (client.md, 7).
use crate::{Limits, translate::watches as translate};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_protocol_client_wire::v7 as w;
use skein_http::sse::writer as sse;
use skein_lib::{Env, Map, Queue, Time, Token, stream::Down};
#[derive(Debug)]
pub(crate) struct Watch {
    pub name: u64,
    pub family: Option<u16>,
    pub version: u32,
    pub project: u32,
    pub rows: u32,
    pub opening: Option<Token>,
    pub closing: Option<Token>,
    pub unwatch: bool,
    pub delivery_pending: bool,
    pub text_pending: bool,
    pub end: Option<jig_domain_views::End>,
    pub delivery: Option<Packet>,
    pub text: Option<Packet>,
    pub ended: Option<Packet>,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Taken {
    None,
    Delivery(Token),
    Text(Token),
    Ended { watcher: Token, signed_out: bool },
}
#[derive(Debug)]
pub(crate) struct Packet {
    pub event: sse::Outgoing,
    pub taken: Taken,
}
#[derive(Debug)]
pub(crate) struct Stream {
    pub name: [u8; 16],
    pub sign_in: u64,
    pub writer: sse::Writer,
    pub watches: Map<Token, Watch>,
    pub waiting: bool,
    pub closing: bool,
    pub closed_connection: bool,
    pub deadline: Time,
    pub above: Queue<sse::Event>,
    pub below: Queue<Down>,
    pub taken: Taken,
    pub bytes: u32,
}
impl Stream {
    pub fn new(name: [u8; 16], sign_in: u64, env: &Env<Limits>) -> Self {
        Self {
            name,
            sign_in,
            writer: sse::Writer::new(&limits(&env.limits)),
            watches: Map::with_capacity(env.limits.watches_per_stream),
            waiting: false,
            closing: false,
            closed_connection: false,
            deadline: env.now,
            above: Queue::with_capacity(2),
            below: Queue::with_capacity(2),
            taken: Taken::None,
            bytes: 0,
        }
    }
    pub fn write(&mut self, env: &Env<Limits>, packet: Packet) {
        assert!(!self.waiting, "one SSE event at a time");
        self.bytes = u32::try_from(packet.event.data.len()).expect("bounded packet");
        self.taken = packet.taken;
        self.waiting = true;
        self.deadline = env.now.saturating_add(env.limits.stall);
        sse::down(
            &mut self.writer,
            &writer_env(env),
            sse::Request::Event(packet.event),
            &mut self.above,
            &mut self.below,
        );
    }
    pub fn heartbeat(&mut self, env: &Env<Limits>) {
        self.waiting = true;
        self.taken = Taken::None;
        self.deadline = env.now.saturating_add(env.limits.stall);
        sse::down(
            &mut self.writer,
            &writer_env(env),
            sse::Request::Comment(Box::new([])),
            &mut self.above,
            &mut self.below,
        );
    }
    pub fn queue(&mut self, packet: Packet, bound: u32) -> bool {
        let length = u32::try_from(packet.event.data.len()).expect("bounded packet");
        let mut held = self.bytes;
        for (_, watch) in &self.watches {
            for packet in [&watch.delivery, &watch.text, &watch.ended].into_iter().flatten() {
                held = held.saturating_add(u32::try_from(packet.event.data.len()).expect("bounded packet"));
            }
        }
        if length > bound.saturating_sub(held) {
            return false;
        }
        let (watcher, kind) = match packet.taken {
            Taken::Delivery(watcher) => (watcher, 0_u8),
            Taken::Text(watcher) => (watcher, 1_u8),
            Taken::Ended { watcher, .. } => (watcher, 2_u8),
            Taken::None => return false,
        };
        let Some(watch) = self.watches.get_mut(&watcher) else {
            return true;
        };
        let slot = if kind == 0 {
            &mut watch.delivery
        } else if kind == 1 {
            &mut watch.text
        } else {
            &mut watch.ended
        };
        if slot.is_none() {
            *slot = Some(packet);
        }
        true
    }
    pub fn next(&mut self) -> Option<Packet> {
        if self.waiting || self.closing {
            return None;
        }
        let mut selected = None;
        for (watcher, watch) in &self.watches {
            if watch.opening.is_none() && (watch.delivery.is_some() || watch.text.is_some() || watch.ended.is_some()) {
                selected = Some(*watcher);
                break;
            }
        }
        let watch = self.watches.get_mut(&selected?)?;
        if let Some(packet) = watch.delivery.take() {
            return Some(packet);
        }
        if let Some(packet) = watch.text.take() {
            return Some(packet);
        }
        watch.ended.take()
    }
    pub fn ready(&self) -> bool {
        if self.closing {
            if !self.closed_connection {
                return true;
            }
            if self.watches.is_empty() {
                return true;
            }
            for (_, watch) in &self.watches {
                if !watch.unwatch || watch.delivery_pending || watch.text_pending || watch.end.is_some() {
                    return true;
                }
            }
            return false;
        }
        if !self.above.is_empty() || !self.below.is_empty() {
            return true;
        }
        if !self.waiting {
            for (_, watch) in &self.watches {
                if watch.opening.is_none()
                    && (watch.delivery.is_some() || watch.text.is_some() || watch.ended.is_some())
                {
                    return true;
                }
            }
        }
        false
    }
}
pub(crate) fn packet(value: w::StreamEvent, taken: Taken) -> Packet {
    Packet { event: translate::packet(&value), taken }
}
pub(crate) fn taken(taken: Taken) -> Option<core::Event> {
    match taken {
        Taken::Delivery(watcher) => Some(core::Event::Delivered { watcher }),
        Taken::Text(watcher) => Some(core::Event::StreamDelivered { watcher }),
        Taken::None | Taken::Ended { .. } => None,
    }
}
pub(crate) fn limits(limits: &Limits) -> sse::Limits {
    sse::Limits { event: limits.queued_bytes.checked_add(32).expect("checked framed bound"), chunk: limits.http.send }
}
pub(crate) fn writer_env(env: &Env<Limits>) -> Env<sse::Limits> {
    Env { now: env.now, wall: env.wall, limits: limits(&env.limits) }
}
