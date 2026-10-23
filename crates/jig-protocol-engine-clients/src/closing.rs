//! Signing out, mints, and component draining (protocol/client.md, 8).
use super::{
    Box, Component, Env, Limits, Queue, ReplyTo, Request, SecretOwner, SecretsRequest, Token, Waiting, core, routes, w,
};
use crate::routes::Route;
use skein_lib::{Map, Reader};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    Active,
    Stopped,
    Closing,
    Aborting,
    Closed,
}
impl Component {
    pub(super) fn stopping(&self) -> bool {
        self.phase != Phase::Active
    }
    pub(super) fn closing(&self) -> bool {
        self.phase == Phase::Closing
    }
    pub(super) fn aborting(&self) -> bool {
        self.phase == Phase::Aborting
    }
    pub(super) fn closed(&self) -> bool {
        self.phase == Phase::Closed
    }

    pub(super) fn ended(&mut self, env: &Env<Limits>, sign_in: u64, out: &mut Queue<Request>) {
        out.push(Request::Secrets(SecretsRequest::Erase { owner: SecretOwner::SignIn(sign_in) }));
        for _ in 0..self.pending.exchanges.capacity() {
            let mut found = None;
            for (exchange, waiting) in &self.pending.exchanges {
                match waiting {
                    Waiting::SignOut { sign_in: number, committed: false }
                    | Waiting::Watch { sign_in: number, .. }
                    | Waiting::Keep { sign_in: number, .. }
                    | Waiting::Get { owner: SecretOwner::SignIn(number), .. }
                    | Waiting::LinkGet { sign_in: number, .. }
                    | Waiting::Made { sign_in: number, .. }
                    | Waiting::Checked { sign_in: number, .. }
                    | Waiting::Read { sign_in: number, .. }
                    | Waiting::ApplicationRead { sign_in: number, .. }
                    | Waiting::Ask { sign_in: number, .. }
                    | Waiting::Carry { sign_in: number, .. }
                        if *number == sign_in =>
                    {
                        found = Some(*exchange);
                        break;
                    }
                    Waiting::Core { .. }
                    | Waiting::LinkGet { .. }
                    | Waiting::SignOut { .. }
                    | Waiting::Watch { .. }
                    | Waiting::Keep { .. }
                    | Waiting::Get { .. }
                    | Waiting::Made { .. }
                    | Waiting::Checked { .. }
                    | Waiting::Read { .. }
                    | Waiting::ApplicationRead { .. }
                    | Waiting::Ask { .. }
                    | Waiting::Carry { .. } => {}
                }
            }
            let Some(exchange) = found else {
                break;
            };
            match self.pending.exchanges.remove(&exchange).expect("ended exchange") {
                Waiting::LinkGet { .. } => self.link_sign_in(exchange, None, out),
                Waiting::Keep { .. } | Waiting::Made { .. } => {
                    if !self.mint_refused(exchange, crate::MintRefusal::Bound, out) {
                        self.respond(env, exchange, routes::refused(&env.limits.wire));
                    }
                }
                Waiting::Read { .. }
                | Waiting::ApplicationRead { .. }
                | Waiting::Get { route: Route::Read, .. }
                | Waiting::Checked { route: Route::Read, .. } => {
                    self.respond(env, exchange, routes::read_none(core::reads::Why::SignedOut, &env.limits.wire));
                }
                Waiting::Get { route, .. } | Waiting::Checked { route, .. } => {
                    self.denied(env, exchange, route);
                }
                Waiting::Ask { .. } | Waiting::Carry { .. } => {
                    self.respond(env, exchange, routes::signed_out(&env.limits.wire));
                }
                Waiting::Watch { opening, .. } => {
                    let body = if opening {
                        routes::open_answer(Some(core::WatchRefusal::SignedOut), &env.limits.wire)
                    } else {
                        routes::close_answer(true)
                    };
                    self.respond(env, exchange, body);
                }
                Waiting::SignOut { .. } => {
                    self.pending
                        .exchanges
                        .insert(exchange, Waiting::SignOut { sign_in, committed: true })
                        .expect("same sign-out slot");
                }
                Waiting::Core { .. } => {}
            }
        }
        self.end_streams(Some(sign_in));
    }
    pub(super) fn abort(&mut self, _out: &mut Queue<Request>) {
        if self.aborting() {
            return;
        }
        self.phase = Phase::Aborting;
        self.end_streams(None);
        self.link = None;
        self.pending.exchanges = Map::with_capacity(self.pending.exchanges.capacity());
        self.pending.carried = Map::with_capacity(self.pending.carried.capacity());
    }

    pub(super) fn sign_out(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        body: &[u8],
        out: &mut Queue<Request>,
    ) {
        if self.stopping() {
            self.respond(env, exchange, routes::sign_out_not_ready());
            return;
        }
        let mut reader = Reader::new(body);
        if w::SignOut::decode(&env.limits.wire, &mut reader).is_err() || !reader.is_empty() {
            if let Some(owner) = self.owner(exchange) {
                self.connections
                    .get_mut(&owner)
                    .expect("exchange owner")
                    .respond(crate::Reply::NotARecord, &env.limits);
            }
            return;
        }
        self.pending
            .exchanges
            .insert(exchange, Waiting::SignOut { sign_in, committed: false })
            .expect("same checked slot");
        out.push(Request::Core(Box::new(core::Event::People(jig_domain_people::Event::SignOut {
            reply_to: ReplyTo::new(exchange),
            sign_in,
        }))));
    }
    pub(super) fn erased(&mut self, env: &Env<Limits>, owner: SecretOwner) {
        for _ in 0..self.pending.exchanges.capacity() {
            let mut found = None;
            for (exchange, waiting) in &self.pending.exchanges {
                match waiting {
                    Waiting::SignOut { sign_in, committed: true } if owner == SecretOwner::SignIn(*sign_in) => {
                        found = Some(*exchange);
                        break;
                    }
                    Waiting::Core { .. }
                    | Waiting::LinkGet { .. }
                    | Waiting::SignOut { .. }
                    | Waiting::Watch { .. }
                    | Waiting::Keep { .. }
                    | Waiting::Get { .. }
                    | Waiting::Made { .. }
                    | Waiting::Read { .. }
                    | Waiting::ApplicationRead { .. }
                    | Waiting::Ask { .. }
                    | Waiting::Carry { .. }
                    | Waiting::Checked { .. } => {}
                }
            }
            let Some(exchange) = found else { break };
            self.pending.exchanges.remove(&exchange);
            self.respond(env, exchange, routes::sign_out_answer());
        }
    }
    pub(super) fn stop(&mut self, _out: &mut Queue<Request>) {
        if self.phase == Phase::Active {
            self.phase = Phase::Stopped;
        }
    }
    pub(super) fn close(&mut self, out: &mut Queue<Request>) {
        if !self.aborting() {
            self.phase = Phase::Closing;
            self.link = None;
            self.stop(out);
        }
    }
    pub(super) fn settled(&self) -> bool {
        let mut carrying = false;
        for (_, kept) in &self.pending.carried {
            carrying |= kept.is_none();
        }
        !self.closed()
            && !carrying
            && self.connections.is_empty()
            && self.streams.is_empty()
            && self.pending.mints.is_empty()
    }
    pub(super) fn close_up(&mut self, _env: &Env<Limits>, out: &mut Queue<Request>) -> bool {
        if !self.closing() {
            return false;
        }

        let mut unwatch = None;
        let mut finish = None;
        for (owner, stream) in &self.streams {
            if stream.closing() {
                continue;
            }
            for (watcher, watch) in &stream.watches {
                if !watch.unwatch {
                    unwatch = Some((*owner, *watcher));
                    break;
                }
            }
            if unwatch.is_some() {
                break;
            }
            if stream.watches.is_empty() && !stream.waiting {
                finish = Some(*owner);
                break;
            }
        }
        if let Some((owner, watcher)) = unwatch {
            self.streams
                .get_mut(&owner)
                .expect("closing stream")
                .watches
                .get_mut(&watcher)
                .expect("closing watch")
                .unwatch = true;
            out.push(Request::Core(Box::new(core::Event::Unwatch { watcher })));
            return true;
        }
        if let Some(owner) = finish {
            let stream = self.streams.get_mut(&owner).expect("finishing stream");
            stream.phase = crate::stream::Phase::Closing;
            stream.ending = crate::Ended::Stopping;
            return true;
        }
        if self.settled() {
            self.phase = Phase::Closed;
            out.push(Request::Closed);
            return true;
        }
        false
    }
}
