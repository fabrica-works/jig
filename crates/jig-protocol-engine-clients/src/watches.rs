//! Watch exchanges and released stream outputs (protocol/client.md, 4 and 7).
use super::{Box, Component, Env, Limits, Queue, Request, Token, Waiting, core, routes, w};
use crate::{
    ToApplication,
    routes::Route,
    stream::{self, Stream, Taken, Watch},
    translate::watches as translate,
};
use skein_http::{server as http, sse::writer as sse};
use skein_lib::Reader;
impl Component {
    fn stream_owner(&self, name: &[u8; 16], sign_in: u64) -> Option<Token> {
        for (owner, stream) in &self.streams {
            if stream.name == *name && stream.sign_in == sign_in && !stream.closing {
                return Some(*owner);
            }
        }
        None
    }
    fn watcher_owner(&self, watcher: Token) -> Option<Token> {
        for (owner, stream) in &self.streams {
            if stream.watches.contains_key(&watcher) {
                return Some(*owner);
            }
        }
        None
    }
    pub(super) fn watch_route(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        route: Route,
        body: &[u8],
        out: &mut Queue<Request>,
    ) {
        let Some(owner) = self.owner(exchange) else {
            return;
        };
        let mut reader = Reader::new(body);
        match route {
            Route::Stream if body.is_empty() => {
                self.stream_open(env, owner, sign_in);
            }
            Route::Watch => match w::Open::decode(&env.limits.wire, &mut reader) {
                Ok(value) if reader.is_empty() => self.open_watch(env, exchange, sign_in, value, out),
                Ok(_) | Err(_) => {
                    self.connections.get_mut(&owner).expect("exchange owner").respond(400, Box::new([]), &env.limits);
                }
            },
            Route::Unwatch => {
                let Ok(value) = w::WatchClose::decode(&env.limits.wire, &mut reader) else {
                    self.connections.get_mut(&owner).expect("exchange owner").respond(400, Box::new([]), &env.limits);
                    return;
                };
                if !reader.is_empty() {
                    self.connections.get_mut(&owner).expect("exchange owner").respond(400, Box::new([]), &env.limits);
                    return;
                }
                let Some(owner) = self.stream_owner(value.stream(), sign_in) else {
                    self.respond(env, exchange, routes::close_answer(false));
                    return;
                };
                let stream = self.streams.get_mut(&owner).expect("selected stream");
                let mut found = None;
                for (watcher, watch) in &stream.watches {
                    if watch.name == value.watch() {
                        found = Some(*watcher);
                        break;
                    }
                }
                if let Some(watcher) = found {
                    let watch = stream.watches.get_mut(&watcher).expect("named watch");
                    if watch.closing.is_some() {
                        self.connections
                            .get_mut(&self.owner(exchange).expect("exchange owner"))
                            .expect("connection")
                            .respond(409, Box::new([]), &env.limits);
                        return;
                    }
                    watch.closing = Some(exchange);
                    if !watch.unwatch {
                        watch.unwatch = true;
                        out.push(Request::Core(Box::new(core::Event::Unwatch { watcher })));
                    }
                    self.pending
                        .exchanges
                        .insert(exchange, Waiting::Watch { sign_in, watcher, opening: false })
                        .expect("admitted close");
                } else {
                    self.respond(env, exchange, routes::close_answer(false));
                }
            }
            Route::Gone => match w::Gone::decode(&env.limits.wire, &mut reader) {
                Ok(value) if reader.is_empty() => {
                    if let Some(owner) = self.stream_owner(value.stream(), sign_in) {
                        self.streams.get_mut(&owner).expect("selected stream").closing = true;
                    }
                    self.respond(env, exchange, Box::new([]));
                }
                Ok(_) | Err(_) => {
                    self.connections.get_mut(&owner).expect("exchange owner").respond(400, Box::new([]), &env.limits);
                }
            },
            Route::Stream | Route::Link | Route::Service | Route::Read | Route::Request | Route::Missing => {
                self.connections.get_mut(&owner).expect("exchange owner").respond(400, Box::new([]), &env.limits);
            }
        }
    }
    fn stream_open(&mut self, env: &Env<Limits>, owner: Token, sign_in: u64) {
        let mut existing = false;
        for (_, stream) in &self.streams {
            if stream.sign_in == sign_in {
                existing = true;
            }
        }
        if existing || self.streams.len() == self.streams.capacity() {
            self.connections.get_mut(&owner).expect("stream owner").respond(409, Box::new([]), &env.limits);
            return;
        }
        let mut name = [0; 16];
        self.rng.fill(&mut name);
        let hello = translate::hello(name, &self.settings, &env.limits.wire);
        let Ok(hello) = hello else {
            self.connections.get_mut(&owner).expect("stream owner").respond(400, Box::new([]), &env.limits);
            return;
        };
        let mut stream = Stream::new(name, sign_in, env);
        stream.write(env, stream::packet(hello, Taken::None));
        self.streams.insert(owner, stream).expect("admitted stream");
        let connection = self.connections.get_mut(&owner).expect("stream owner");
        connection.stream = Some(owner);
        connection.requests.push(http::Request::Discard);
        connection.requests.push(http::Request::Respond(http::Response {
            status: 200,
            headers: Box::new([skein_http::Header {
                name: skein_lib::bytes::copy_of(b"content-type"),
                value: skein_lib::bytes::copy_of(b"text/event-stream"),
            }]),
            body: http::Body::Chunked,
            close: false,
        }));
    }
    fn open_watch(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        open: w::Open,
        out: &mut Queue<Request>,
    ) {
        let open = open.into_parts();
        let Some(owner) = self.stream_owner(&open.stream, sign_in) else {
            self.respond(env, exchange, routes::open_answer(Some(core::WatchRefusal::Unknown), &env.limits.wire));
            return;
        };
        let stream = self.streams.get(&owner).expect("named stream");
        let mut duplicate = false;
        for (_, watch) in &stream.watches {
            if watch.name == open.watch {
                duplicate = true;
            }
        }
        if duplicate
            || stream.watches.len() == stream.watches.capacity()
            || self.admitted() >= env.limits.exchanges
            || self.active(sign_in) >= env.limits.exchanges_per_client
        {
            self.respond(env, exchange, routes::open_answer(Some(core::WatchRefusal::Busy), &env.limits.wire));
            return;
        }
        let subject = open.subject.into_parts();
        let family = match &subject.owner {
            w::Owner::Jig => None,
            w::Owner::Family(value) => Some(value.number()),
        };
        if !self.serves(family, subject.version) {
            self.connections.get_mut(&self.owner(exchange).expect("exchange owner")).expect("connection").respond(
                400,
                Box::new([]),
                &env.limits,
            );
            return;
        }
        let watcher = self.name();
        let mut project = 0;
        let mut rows = self.settings.rows;
        let request = match subject.owner {
            w::Owner::Jig => {
                let value = translate::decode(&subject.record, &env.limits.wire);
                let event = match value {
                    Some(value) => translate::watch(watcher, sign_in, value, rows),
                    None => None,
                };
                let Some(event) = event else {
                    self.respond(
                        env,
                        exchange,
                        routes::open_answer(Some(core::WatchRefusal::Unknown), &env.limits.wire),
                    );
                    return;
                };
                if let core::Event::Watch { subject, window, .. } = &event {
                    rows = window.rows;
                    if let core::documents::Watched::Goals { project: number } = subject {
                        project = *number;
                    }
                }
                Request::Core(Box::new(event))
            }
            w::Owner::Family(value) => Request::Application(ToApplication::Watch {
                watcher,
                sign_in,
                family: value.number(),
                version: subject.version,
                record: subject.record,
            }),
        };
        self.streams
            .get_mut(&owner)
            .expect("named stream")
            .watches
            .insert(
                watcher,
                Watch {
                    name: open.watch,
                    family,
                    version: subject.version,
                    project,
                    rows,
                    opening: Some(exchange),
                    closing: None,
                    unwatch: false,
                    delivery_pending: false,
                    text_pending: false,
                    end: None,
                    delivery: None,
                    text: None,
                    ended: None,
                },
            )
            .expect("admitted watch");
        self.pending
            .exchanges
            .insert(exchange, Waiting::Watch { sign_in, watcher, opening: true })
            .expect("admitted exchange");
        out.push(request);
    }
    pub(super) fn active(&self, sign_in: u64) -> u32 {
        let mut count = 0_u32;
        for (_, waiting) in &self.pending.exchanges {
            let number = match waiting {
                Waiting::Watch { sign_in, .. }
                | Waiting::Ask { sign_in, .. }
                | Waiting::Read { sign_in, .. }
                | Waiting::ApplicationRead { sign_in, .. } => Some(*sign_in),
                Waiting::Core
                | Waiting::Keep { .. }
                | Waiting::Get { .. }
                | Waiting::Made { .. }
                | Waiting::Checked { .. } => None,
            };
            if number == Some(sign_in) {
                count = count.saturating_add(1);
            }
        }
        count
    }
    pub(super) fn watch_answer(&mut self, env: &Env<Limits>, watcher: Token, why: Option<core::WatchRefusal>) {
        let Some(owner) = self.watcher_owner(watcher) else {
            return;
        };
        let watch =
            self.streams.get_mut(&owner).expect("watch stream").watches.get_mut(&watcher).expect("watch binding");
        let Some(exchange) = watch.opening.take() else {
            return;
        };
        if let Some(Waiting::Watch { watcher: wanted, opening: true, .. }) = self.pending.exchanges.get(&exchange)
            && *wanted == watcher
        {
            self.pending.exchanges.remove(&exchange);
            self.respond(env, exchange, routes::open_answer(why, &env.limits.wire));
        }
        if why.is_some() {
            self.streams.get_mut(&owner).expect("watch stream").watches.remove(&watcher);
        }
    }
    fn queue_packet(
        &mut self,
        env: &Env<Limits>,
        watcher: Token,
        value: Result<w::StreamEvent, w::Problem>,
        taken: Taken,
    ) {
        let Some(owner) = self.watcher_owner(watcher) else {
            return;
        };
        let stream = self.streams.get_mut(&owner).expect("watch stream");
        match value {
            Ok(value) => {
                if !stream.queue(stream::packet(value, taken), env.limits.queued_bytes) {
                    stream.closing = true;
                }
            }
            Err(_) => stream.closing = true,
        }
    }
    pub(super) fn delivery(&mut self, env: &Env<Limits>, watcher: Token, document: &core::documents::Document) {
        let Some(owner) = self.watcher_owner(watcher) else {
            return;
        };
        let watch =
            self.streams.get_mut(&owner).expect("watch stream").watches.get_mut(&watcher).expect("watch binding");
        if watch.family.is_some() || watch.delivery_pending {
            return;
        }
        watch.delivery_pending = true;
        let value = translate::document_packet(
            document,
            watch.project,
            watch.rows,
            watch.name,
            watch.version,
            &env.limits.wire,
        );
        self.queue_packet(env, watcher, value, Taken::Delivery(watcher));
    }
    pub(super) fn application_delivery(&mut self, env: &Env<Limits>, watcher: Token, family: u16, record: Box<[u8]>) {
        let Some(owner) = self.watcher_owner(watcher) else {
            return;
        };
        let watch =
            self.streams.get_mut(&owner).expect("watch stream").watches.get_mut(&watcher).expect("watch binding");
        if watch.family != Some(family) || watch.delivery_pending {
            return;
        }
        watch.delivery_pending = true;
        let value = translate::application_packet(family, watch.version, record, watch.name, &env.limits.wire);
        self.queue_packet(env, watcher, value, Taken::Delivery(watcher));
    }
    pub(super) fn run_text(&mut self, env: &Env<Limits>, watcher: Token, writing: jig_domain_views::Writing) {
        let Some(owner) = self.watcher_owner(watcher) else {
            return;
        };
        let watch =
            self.streams.get_mut(&owner).expect("watch stream").watches.get_mut(&watcher).expect("watch binding");
        if watch.family.is_some() || watch.text_pending {
            return;
        }
        watch.text_pending = true;
        let value = translate::text_packet(&writing, watch.name, &env.limits.wire);
        self.queue_packet(env, watcher, value, Taken::Text(watcher));
    }
    pub(super) fn watch_ended(&mut self, env: &Env<Limits>, watcher: Token, why: jig_domain_views::End) {
        let Some(owner) = self.watcher_owner(watcher) else {
            return;
        };
        let watch =
            self.streams.get_mut(&owner).expect("watch stream").watches.get_mut(&watcher).expect("watch binding");
        if watch.end.is_some() {
            return;
        }
        watch.end = Some(why);
        let name = watch.name;
        let close = watch.closing.take();
        if let Some(exchange) = close
            && let Some(Waiting::Watch { watcher: wanted, opening: false, .. }) = self.pending.exchanges.get(&exchange)
            && *wanted == watcher
        {
            self.pending.exchanges.remove(&exchange);
            self.respond(env, exchange, routes::close_answer(why == jig_domain_views::End::SignedOut));
        }
        self.queue_packet(
            env,
            watcher,
            translate::end_packet(why, name, &env.limits.wire),
            Taken::Ended { watcher, signed_out: why == jig_domain_views::End::SignedOut },
        );
    }
    pub(super) fn end_streams(&mut self, sign_in: Option<u64>) {
        for _ in 0..self.streams.capacity() {
            let mut found = None;
            for (owner, stream) in &self.streams {
                if !stream.closing && (sign_in.is_none() || sign_in == Some(stream.sign_in)) {
                    found = Some(*owner);
                    break;
                }
            }
            let Some(owner) = found else {
                break;
            };
            self.streams.get_mut(&owner).expect("ending stream").closing = true;
        }
    }
    pub(super) fn stream_up(&mut self, env: &Env<Limits>, out: &mut Queue<Request>) -> bool {
        let mut selected = None;
        for (owner, stream) in &self.streams {
            if stream.ready() || self.aborting {
                selected = Some(*owner);
                break;
            }
        }
        let Some(owner) = selected else {
            return false;
        };
        let mut stream = self.streams.remove(&owner).expect("selected stream");
        if stream.closing {
            if !stream.closed_connection {
                stream.closed_connection = true;
                if let Some(connection) = self.connections.get_mut(&owner) {
                    connection.close(out);
                }
            }
            let mut selected = None;
            for (watcher, watch) in &stream.watches {
                if !watch.unwatch
                    || watch.delivery_pending
                    || watch.text_pending
                    || watch.end.is_some()
                    || self.aborting
                {
                    selected = Some(*watcher);
                    break;
                }
            }
            if let Some(watcher) = selected {
                let watch = stream.watches.get_mut(&watcher).expect("closing watch");
                if !watch.unwatch {
                    watch.unwatch = true;
                    out.push(Request::Core(Box::new(core::Event::Unwatch { watcher })));
                } else if watch.delivery_pending {
                    watch.delivery_pending = false;
                    watch.delivery = None;
                    out.push(Request::Core(Box::new(core::Event::Delivered { watcher })));
                } else if watch.text_pending {
                    watch.text_pending = false;
                    watch.text = None;
                    out.push(Request::Core(Box::new(core::Event::StreamDelivered { watcher })));
                } else {
                    let watch = stream.watches.remove(&watcher).expect("ended watch");
                    if let Some(exchange) = watch.opening {
                        self.pending.exchanges.remove(&exchange);
                        self.respond(
                            env,
                            exchange,
                            routes::open_answer(Some(core::WatchRefusal::Unknown), &env.limits.wire),
                        );
                    }
                }
            } else if stream.watches.is_empty() {
                return true;
            }
        } else if let Some(down) = stream.below.pop() {
            if let Some(connection) = self.connections.get_mut(&owner) {
                connection.requests.push(http::Request::Reply(down));
            } else {
                stream.closing = true;
            }
        } else if let Some(event) = stream.above.pop() {
            match event {
                sse::Event::Sent => {
                    stream.waiting = false;
                    stream.bytes = 0;
                    stream.deadline = env.now.saturating_add(self.settings.heartbeat);
                    let taken = ::core::mem::replace(&mut stream.taken, Taken::None);
                    match taken {
                        Taken::Delivery(watcher) => {
                            if let Some(watch) = stream.watches.get_mut(&watcher) {
                                watch.delivery_pending = false;
                            }
                        }
                        Taken::Text(watcher) => {
                            if let Some(watch) = stream.watches.get_mut(&watcher) {
                                watch.text_pending = false;
                            }
                        }
                        Taken::None | Taken::Ended { .. } => {}
                    }
                    if let Some(event) = stream::taken(taken) {
                        out.push(Request::Core(Box::new(event)));
                    }
                    if let Taken::Ended { watcher, signed_out } = taken {
                        stream.watches.remove(&watcher);
                        stream.closing = signed_out;
                    }
                }
                sse::Event::Failed(_) | sse::Event::Refused(_) => stream.closing = true,
                sse::Event::Closed => {}
            }
        } else if let Some(packet) = stream.next() {
            stream.write(env, packet);
        }
        self.streams.insert(owner, stream).expect("same stream slot");
        true
    }
}
