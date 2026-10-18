//! Listener, exchanges, and durable sign-in sequencing (protocol/client.md, 8).
use crate::{
    Event, Kept, Limits, Request, SecretOwner, SecretsAnswer, SecretsRequest, Settings,
    connection::Connection,
    routes::{self, Route},
    sign_in::{self, Link, Pending, Waiting},
};
use alloc::boxed::Box;
use jig_client_protocol_wire::v8 as w;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_http::server as http;
use skein_lib::{
    CryptoRng, Env, Map, Queue, ReplyTo, Time, Token,
    stream::{Read, Up},
};
#[path = "closing.rs"]
mod closing;
#[path = "mint.rs"]
mod mint;
#[path = "reads.rs"]
mod reads;
#[path = "services.rs"]
mod services;
#[path = "watches.rs"]
mod watches;
const LISTENER: Token = Token::new(0);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unusable {
    Limits,
    Seed,
    Listener,
}
#[derive(Debug)]
pub struct Component {
    settings: Settings,
    rng: CryptoRng,
    link: Option<Link>,
    listener: Option<Token>,
    connections: Map<Token, Connection>,
    pending: Pending,
    streams: Map<Token, crate::stream::Stream>,
    next: u64,
    phase: closing::Phase,
    listening: bool,
    listener_closing: bool,
}
impl Component {
    pub fn new(settings: Settings, env: &Env<Limits>, out: &mut Queue<Request>) -> Result<Self, Unusable> {
        crate::worst_case(&env.limits).ok_or(Unusable::Limits)?;
        if settings.served.capacity() > env.limits.families
            || settings.providers.capacity() > env.limits.providers
            || settings.heartbeat == skein_lib::Duration::ZERO
            || settings.rows == 0
        {
            return Err(Unusable::Limits);
        }
        let Ok(hello) = crate::translate::watches::hello([0; 16], &settings, &env.limits.wire) else {
            return Err(Unusable::Limits);
        };
        let length = skein_lib::base64::encoded_len(skein_lib::base64::Alphabet::Standard, hello.measure())
            .ok_or(Unusable::Limits)?;
        if length > env.limits.queued_bytes {
            return Err(Unusable::Limits);
        }
        if let Some(identity) = &settings.link {
            let bytes = identity.key.subject.len().checked_add(identity.login.len()).ok_or(Unusable::Limits)?;
            let bytes = bytes.checked_add(identity.name.len()).ok_or(Unusable::Limits)?;
            if bytes > usize::try_from(env.limits.identity_bytes).expect("u32 fits") {
                return Err(Unusable::Limits);
            }
        }
        if settings.link.is_some() {
            let addr = settings.listener.socket().ok_or(Unusable::Listener)?;
            if !addr.ip().is_loopback() {
                return Err(Unusable::Listener);
            }
        }
        let rng = CryptoRng::new(&settings.seed, settings.stream).ok_or(Unusable::Seed)?;
        let mut value = Self {
            settings,
            rng,
            link: None,
            listener: None,
            connections: Map::with_capacity(env.limits.connections),
            pending: Pending {
                mints: Map::with_capacity(env.limits.mints),
                exchanges: Map::with_capacity(
                    env.limits.exchanges.checked_add(env.limits.connections).expect("checked pending capacity"),
                ),
            },
            streams: Map::with_capacity(env.limits.streams),
            next: 1,
            phase: closing::Phase::Active,
            listening: true,
            listener_closing: false,
        };
        out.push(Request::Io(skein_io::Request::Listen { owner: LISTENER, addr: value.settings.listener }));
        value.fresh_link(env, out);
        Ok(value)
    }
    fn name(&mut self) -> Token {
        let token = Token::new(self.next);
        self.next = self.next.checked_add(1).expect("component names do not exhaust");
        token
    }
    fn fresh_link(&mut self, env: &Env<Limits>, out: &mut Queue<Request>) {
        if self.settings.link.is_none() || self.link.is_some() {
            return;
        }
        let secret = sign_in::draw(&mut self.rng);
        self.link = Some(Link { secret: secret.clone(), until: env.now.saturating_add(env.limits.link_span) });
        out.push(Request::Link { secret });
    }
    pub fn down(&mut self, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
        assert!(!self.closed(), "the owner sends nothing after Closed");
        match event {
            Event::Io(event) => self.io(env, event, out),
            Event::Reply { to, reply } => self.reply(env, to.into_token(), reply, out),
            Event::SignInRefused { to } => {
                let exchange = to.into_token();
                let why = if self.stopping() { crate::MintRefusal::NotReady } else { crate::MintRefusal::Bound };
                if !self.mint_refused(exchange, why, out) {
                    self.refused(env, exchange);
                }
            }
            Event::Secrets(answer) => self.secrets(env, answer, out),
            Event::Application(crate::FromApplication::Undecodable { to }) => {
                let exchange = to.into_token();
                self.pending.exchanges.remove(&exchange);
                if let Some(owner) = self.owner(exchange) {
                    self.connections.get_mut(&owner).expect("exchange owner").respond(400, Box::new([]), &env.limits);
                }
            }
            Event::SignInEnded { sign_in } => self.ended(env, sign_in, out),
            Event::Application(crate::FromApplication::Answered { to, family, record }) => {
                self.application(env, to.into_token(), family, record, out);
            }
            Event::Stop => self.stop(out),
            Event::Close => self.close(out),
            Event::Abort => self.abort(out),
            Event::Read { reply, answer } => self.read_answer(env, reply.into_token(), &answer, out),
            Event::Application(crate::FromApplication::Read { reply, family, record }) => {
                self.application_read(env, reply.into_token(), family, record, out);
            }
            Event::Admitted { watcher } => self.watch_answer(env, watcher, None),
            Event::WatchRefused { watcher, why } => self.watch_answer(env, watcher, Some(why)),
            Event::Delivery { watcher, document } => self.delivery(env, watcher, &document),
            Event::WatchEnded { watcher, why } => self.watch_ended(env, watcher, why),
            Event::RunText { watcher, turn, from, to, text, cut, call } => {
                self.run_text(env, watcher, jig_domain_views::Writing { turn, from, to, text, cut, call });
            }
            Event::Application(crate::FromApplication::Delivery { watcher, family, record }) => {
                self.application_delivery(env, watcher, family, record);
            }
            Event::Mint { exchange, identity } => self.mint(env, exchange, identity, out),
        }
    }
    fn io(&mut self, env: &Env<Limits>, event: skein_io::Event, out: &mut Queue<Request>) {
        match event {
            skein_io::Event::Listening { owner, listener, addr } => {
                if owner == LISTENER {
                    self.listening = false;
                    self.listener = Some(listener);
                    self.settings.listener = addr;
                    self.fresh_link(env, out);
                    self.close_listener(out);
                }
            }
            skein_io::Event::Accepted { owner, socket, .. } => {
                if owner != LISTENER || self.stopping() || self.connections.len() == self.connections.capacity() {
                    out.push(Request::Io(skein_io::Request::Reject { socket }));
                } else {
                    let owner = self.name();
                    self.connections
                        .insert(owner, Connection::new(socket, &env.limits, env.now))
                        .expect("admitted socket");
                    out.push(Request::Io(skein_io::Request::Bind { socket, owner }));
                }
            }
            skein_io::Event::Stream { owner, up } => {
                if let Some(connection) = self.connections.get_mut(&owner)
                    && !connection.closing
                {
                    connection.feed(env, up);
                }
            }
            skein_io::Event::Closed { owner } => {
                if let Some(stream) = self.streams.get_mut(&owner) {
                    stream.phase = crate::stream::Phase::Closing;
                }
                if owner == LISTENER {
                    self.listener = None;
                }
                if let Some(connection) = self.connections.remove(&owner)
                    && let Some(exchange) = connection.exchange
                {
                    self.pending.exchanges.remove(&exchange);
                }
            }
            skein_io::Event::Failed { owner, .. } => {
                if owner == LISTENER {
                    self.listening = false;
                }
                if let Some(connection) = self.connections.get_mut(&owner) {
                    connection.close(out);
                } else if let Some(listener) = self.listener {
                    out.push(Request::Io(skein_io::Request::Abort { entity: listener }));
                }
            }
            skein_io::Event::Connecting { .. }
            | skein_io::Event::Connected { .. }
            | skein_io::Event::Output { .. }
            | skein_io::Event::Spawned { .. }
            | skein_io::Event::Exited { .. }
            | skein_io::Event::Usage { .. }
            | skein_io::Event::Resized
            | skein_io::Event::Window { .. }
            | skein_io::Event::Shutdown { .. } => {}
        }
    }
    /// Advances one queued HTTP transition, bounding each invocation's output.
    pub fn up(&mut self, env: &Env<Limits>, out: &mut Queue<Request>) -> bool {
        if self.aborting() {
            if let Some((name, _)) = self.pending.mints.first() {
                let name = *name;
                self.mint_refused(name, crate::MintRefusal::NotReady, out);
                return true;
            }
            if self.stream_up(env, out) {
                return true;
            }
            let mut closing = None;
            for (owner, connection) in &self.connections {
                if !connection.closing || connection.draining {
                    closing = Some(*owner);
                    break;
                }
            }
            if let Some(owner) = closing {
                self.connections.get_mut(&owner).expect("closing connection").close(out);
                return true;
            }
            if self.settled() {
                self.phase = closing::Phase::Closed;
                out.push(Request::Closed);
                return true;
            }
            return false;
        }
        let mut ready = None;
        for (exchange, waiting) in &self.pending.exchanges {
            match waiting {
                Waiting::Checked { .. } => {
                    ready = Some(*exchange);
                    break;
                }
                Waiting::Core
                | Waiting::SignOut { .. }
                | Waiting::Watch { .. }
                | Waiting::Get { .. }
                | Waiting::Keep { .. }
                | Waiting::Made { .. }
                | Waiting::Read { .. }
                | Waiting::ApplicationRead { .. }
                | Waiting::Ask { .. } => {}
            }
        }
        if let Some(exchange) = ready {
            match self.pending.exchanges.remove(&exchange).expect("ready exchange") {
                Waiting::Checked { sign_in, body, route } => match route {
                    Route::Read => self.read(env, exchange, sign_in, &body, out),
                    Route::Watch | Route::Unwatch | Route::Gone | Route::Stream => {
                        self.watch_route(env, exchange, sign_in, route, &body, out);
                    }
                    Route::Request => self.request(env, exchange, sign_in, &body, out),
                    Route::SignOut => self.sign_out(env, exchange, sign_in, &body, out),
                    Route::Link | Route::Service | Route::Missing => {}
                },
                Waiting::Core
                | Waiting::SignOut { .. }
                | Waiting::Watch { .. }
                | Waiting::Get { .. }
                | Waiting::Keep { .. }
                | Waiting::Made { .. }
                | Waiting::Read { .. }
                | Waiting::ApplicationRead { .. }
                | Waiting::Ask { .. } => {}
            }
            return true;
        }
        let mut selected = None;
        for (owner, connection) in &self.connections {
            if !connection.closing
                && (!connection.below.is_empty() || !connection.requests.is_empty() || !connection.events.is_empty())
            {
                selected = Some(*owner);
                break;
            }
        }
        let Some(owner) = selected else {
            if self.stream_up(env, out) {
                return true;
            }
            return self.close_up(env, out);
        };
        let mut connection = self.connections.remove(&owner).expect("selected connection");
        if !connection.progress(env, out)
            && let Some(event) = connection.events.pop()
        {
            self.http(env, &mut connection, event, out);
        }
        self.connections.insert(owner, connection).expect("same slot");
        true
    }
    fn http(&mut self, env: &Env<Limits>, connection: &mut Connection, event: http::Event, out: &mut Queue<Request>) {
        match event {
            http::Event::Call(call) => {
                connection.route = routes::route(&call);
                connection.exchange = Some(self.name());
                connection.call = Some(call);
                match connection.route {
                    Route::Missing => connection.respond(404, Box::new([]), &env.limits),
                    Route::Link
                    | Route::Service
                    | Route::SignOut
                    | Route::Request
                    | Route::Read
                    | Route::Watch
                    | Route::Unwatch
                    | Route::Gone
                    | Route::Stream => connection.read(&env.limits),
                }
            }
            http::Event::Body(Up::Bytes(bytes)) => {
                if connection.body.append(&bytes).is_err() {
                    connection.respond(413, Box::new([]), &env.limits);
                } else {
                    connection.read(&env.limits);
                }
            }
            http::Event::Body(Up::End) => self.body(env, connection, out),
            http::Event::Reply(up) if connection.stream.is_some() => {
                if let Some(stream) = self.streams.get_mut(&connection.stream.expect("stream connection")) {
                    skein_http::sse::writer::up(
                        &mut stream.writer,
                        &crate::stream::writer_env(env),
                        up,
                        &mut stream.above,
                        &mut stream.below,
                    );
                }
            }
            http::Event::Reply(Up::Room) => connection.send(&env.limits),
            http::Event::Done(reuse) => {
                if let Some(exchange) = connection.exchange {
                    self.pending.exchanges.remove(&exchange);
                }
                connection.exchange = None;
                connection.call = None;
                connection.body.clear();
                connection.response = Box::new([]);
                if self.closing() {
                    if let Some(owner) = connection.stream
                        && let Some(stream) = self.streams.get_mut(&owner)
                    {
                        stream.phase = crate::stream::Phase::Closing;
                        stream.closed_connection = true;
                    }
                    connection.finish(out);
                } else {
                    match reuse {
                        http::Reuse::Keep => connection.requests.push(http::Request::Next),
                        http::Reuse::Close => connection.close(out),
                    }
                }
            }
            http::Event::Ended
            | http::Event::Failed(_)
            | http::Event::Refused(_)
            | http::Event::Body(Up::Failed(_) | Up::Room)
            | http::Event::Reply(Up::Failed(_) | Up::End | Up::Bytes(_)) => connection.close(out),
            http::Event::Closed => {}
        }
    }
    fn body(&mut self, env: &Env<Limits>, connection: &mut Connection, out: &mut Queue<Request>) {
        let length = connection.body.len();
        let body = match connection.body.meet(Read::Fill(length)) {
            Some(body) => body,
            None => Box::new([]),
        };
        let exchange = connection.exchange.expect("call owns exchange");
        if self.stopping()
            && let Some(answer) = routes::stopped(connection.route)
        {
            connection.respond(200, answer, &env.limits);
            return;
        }
        match connection.route {
            Route::Link => {
                let Ok(record) = w::LinkSignIn::decode(&env.limits.wire, &mut skein_lib::Reader::new(&body)) else {
                    connection.respond(400, Box::new([]), &env.limits);
                    return;
                };
                let live = match &self.link {
                    Some(link) => {
                        env.now < link.until && sign_in::constant_time_eq(link.secret.expose(), record.secret())
                    }
                    None => false,
                };
                if !live {
                    if expired(self.link.as_ref(), env.now) {
                        self.link = None;
                    }
                    self.fresh_link(env, out);
                    connection.respond(200, routes::refused(&env.limits.wire), &env.limits);
                    return;
                }
                if self.admitted() >= env.limits.exchanges {
                    connection.respond(200, routes::refused(&env.limits.wire), &env.limits);
                    return;
                }
                self.link = None;
                self.pending.exchanges.insert(exchange, Waiting::Core).expect("admitted exchange");
                out.push(Request::Core(Box::new(core::Event::SignIn {
                    reply_to: ReplyTo::new(exchange),
                    identity: self.settings.link.as_ref().expect("link enabled").clone(),
                })));
            }
            Route::Service
            | Route::SignOut
            | Route::Request
            | Route::Read
            | Route::Watch
            | Route::Unwatch
            | Route::Gone
            | Route::Stream => {
                let call = connection.call.as_ref().expect("call retained");
                let Some((sign_in, digest)) = sign_in::bearer(call.header(b"authorization")) else {
                    let answer = routes::unauthenticated(connection.route, &env.limits.wire);
                    connection.respond(if connection.route == Route::Stream { 401 } else { 200 }, answer, &env.limits);
                    return;
                };
                let owner = if connection.route == Route::Service {
                    SecretOwner::Service(sign_in)
                } else {
                    SecretOwner::SignIn(sign_in)
                };
                self.pending
                    .exchanges
                    .insert(exchange, Waiting::Get { owner, digest, body, route: connection.route })
                    .expect("admitted exchange");
                out.push(Request::Secrets(SecretsRequest::Get { owner }));
            }
            Route::Missing => {}
        }
    }
    fn reply(&mut self, env: &Env<Limits>, exchange: Token, reply: people::Reply, out: &mut Queue<Request>) {
        let Some(waiting) = self.pending.exchanges.remove(&exchange) else {
            return;
        };
        match waiting {
            Waiting::SignOut { sign_in, committed } => match reply {
                people::Reply::SignedOut => {
                    self.pending
                        .exchanges
                        .insert(exchange, Waiting::SignOut { sign_in, committed })
                        .expect("same sign-out slot");
                    if !committed {
                        self.ended(env, sign_in, out);
                    }
                }
                people::Reply::Again(people::Again::NotReady) => {
                    self.respond(env, exchange, routes::sign_out_not_ready());
                }
                people::Reply::SignedIn { .. }
                | people::Reply::Outcome(_)
                | people::Reply::Again(_)
                | people::Reply::Refused(_) => {
                    if let Some(owner) = self.owner(exchange) {
                        self.connections.get_mut(&owner).expect("exchange owner").close(out);
                    }
                }
            },
            Waiting::Core => self.sign_in_reply(env, exchange, reply, out),
            Waiting::Ask { .. } => self.ask_reply(env, exchange, reply, out, waiting),
            Waiting::Watch { .. }
            | Waiting::Read { .. }
            | Waiting::ApplicationRead { .. }
            | Waiting::Keep { .. }
            | Waiting::Get { .. }
            | Waiting::Made { .. }
            | Waiting::Checked { .. } => {
                self.pending.exchanges.insert(exchange, waiting).expect("same slot");
            }
        }
    }
    fn ask_reply(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        reply: people::Reply,
        out: &mut Queue<Request>,
        waiting: Waiting,
    ) {
        let Waiting::Ask { sign_in, .. } = waiting else { unreachable!("ask reply route") };
        match reply {
            people::Reply::Outcome(people::Outcome::ServiceMade { person }) => {
                self.pending.exchanges.insert(exchange, Waiting::Made { sign_in, party: person }).expect("same slot");
                out.push(Request::Secrets(SecretsRequest::Get { owner: SecretOwner::Service(person) }));
            }
            people::Reply::Outcome(people::Outcome::Applied { .. }) => {
                self.pending.exchanges.insert(exchange, waiting).expect("same slot");
            }
            people::Reply::SignedIn { .. }
            | people::Reply::SignedOut
            | people::Reply::Outcome(_)
            | people::Reply::Again(_)
            | people::Reply::Refused(_) => {
                let answer = crate::translate::answers::answer(&reply, &env.limits.wire);
                self.answer(env, exchange, answer, out);
            }
        }
    }
    fn answer(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        answer: Result<w::KeyedAnswer, w::Problem>,
        out: &mut Queue<Request>,
    ) {
        let Some(owner) = self.owner(exchange) else {
            return;
        };
        let connection = self.connections.get_mut(&owner).expect("exchange owner");
        match answer {
            Ok(answer) => {
                let mut body = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("wire size fits"));
                answer.encode(&mut body).expect("measured answer");
                connection.respond(200, body.finish(), &env.limits);
            }
            Err(_) => connection.close(out),
        }
    }
    fn application(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        family: u16,
        record: Box<[u8]>,
        out: &mut Queue<Request>,
    ) {
        let Some(waiting) = self.pending.exchanges.remove(&exchange) else {
            return;
        };
        match waiting {
            Waiting::Ask { family: Some(number), version, .. } if number == family => {
                let answer = match w::Family::new(&env.limits.wire, w::FamilyParts { number }) {
                    Ok(value) => {
                        crate::translate::answers::owned(w::Owner::Family(value), version, record, &env.limits.wire)
                    }
                    Err(problem) => Err(problem),
                };
                self.answer(env, exchange, answer, out);
            }
            Waiting::Core
            | Waiting::SignOut { .. }
            | Waiting::Watch { .. }
            | Waiting::Keep { .. }
            | Waiting::Get { .. }
            | Waiting::Made { .. }
            | Waiting::Checked { .. }
            | Waiting::Read { .. }
            | Waiting::ApplicationRead { .. }
            | Waiting::Ask { .. } => {
                self.pending.exchanges.insert(exchange, waiting).expect("same slot");
            }
        }
    }
    fn refused(&mut self, env: &Env<Limits>, exchange: Token) {
        if self.pending.exchanges.remove(&exchange).is_some() {
            self.respond(env, exchange, routes::refused(&env.limits.wire));
        }
    }
    fn owner(&self, exchange: Token) -> Option<Token> {
        for (owner, connection) in &self.connections {
            if connection.exchange == Some(exchange) && !connection.closing {
                return Some(*owner);
            }
        }
        None
    }
    fn respond(&mut self, env: &Env<Limits>, exchange: Token, body: Box<[u8]>) {
        if let Some(owner) = self.owner(exchange) {
            self.connections.get_mut(&owner).expect("exchange owner").respond(200, body, &env.limits);
        }
    }
    fn denied(&mut self, env: &Env<Limits>, exchange: Token, route: Route) {
        if let Some(owner) = self.owner(exchange) {
            let status = if route == Route::Stream { 401 } else { 200 };
            self.connections.get_mut(&owner).expect("exchange owner").respond(
                status,
                routes::unauthenticated(route, &env.limits.wire),
                &env.limits,
            );
        }
    }
    fn admitted(&self) -> u32 {
        let mut count = 0_u32;
        for (_, waiting) in &self.pending.exchanges {
            match waiting {
                Waiting::Core
                | Waiting::SignOut { .. }
                | Waiting::Watch { .. }
                | Waiting::Keep { .. }
                | Waiting::Made { .. }
                | Waiting::Read { .. }
                | Waiting::ApplicationRead { .. }
                | Waiting::Ask { .. } => {
                    count = count.saturating_add(1);
                }
                Waiting::Get { .. } | Waiting::Checked { .. } => {}
            }
        }
        count
    }
    #[must_use]
    #[expect(clippy::manual_map, reason = "step code matches optional deadlines without closures")]
    pub fn next_deadline(&self) -> Option<Time> {
        let mut next = match &self.link {
            Some(link) => Some(link.until),
            None => None,
        };
        for (_, connection) in &self.connections {
            if !connection.closing && connection.stream.is_none() {
                next = Some(match next {
                    Some(next) => next.min(connection.deadline),
                    None => connection.deadline,
                });
            }
        }
        for (_, stream) in &self.streams {
            if !stream.closing() {
                next = Some(match next {
                    Some(next) => next.min(stream.deadline),
                    None => stream.deadline,
                });
            }
        }
        next
    }
    pub fn fire(&mut self, env: &Env<Limits>, out: &mut Queue<Request>) {
        if expired(self.link.as_ref(), env.now) {
            self.link = None;
            return;
        }
        let mut due = None;
        for (owner, stream) in &self.streams {
            if !stream.closing() && stream.deadline <= env.now {
                due = Some(*owner);
                break;
            }
        }
        if let Some(owner) = due {
            let stream = self.streams.get_mut(&owner).expect("due stream");
            if stream.waiting || stream.finishing() {
                stream.phase = crate::stream::Phase::Closing;
            } else {
                stream.heartbeat(env);
            }
            return;
        }
        let mut due = None;
        for (owner, connection) in &self.connections {
            if !connection.closing && connection.stream.is_none() && connection.deadline <= env.now {
                due = Some(*owner);
                break;
            }
        }
        if let Some(owner) = due {
            self.connections.get_mut(&owner).expect("due connection").close(out);
        }
    }

    pub fn reclaim(&mut self) {}
}
fn expired(link: Option<&Link>, now: Time) -> bool {
    match link {
        Some(link) => link.until <= now,
        None => false,
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[path = "request.rs"]
mod request;
