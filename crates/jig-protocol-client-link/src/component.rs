//! One bounded exchange per request-table attempt (protocol/client.md, 5).
use crate::{Answered, Ask, Event, Fact, Family, Limits, Readable, Readout, Request, Unusable, Watchable, translate};
use alloc::boxed::Box;
use jig_protocol_client_wire::{self as wire, v4 as w};
use skein_lib::{Map, Queue, Reader, RequestEnvelope, RequestKey, Time, Token, Writer};
#[derive(Debug)]
pub(crate) struct Attempt {
    token: Token,
    owner: w::Owner,
    version: u32,
    until: Time,
    kind: ExchangeKind,
}
#[derive(Clone, Copy, Debug)]
enum ExchangeKind {
    Attempt,
    Read,
    WatchOpen { name: u64 },
    WatchClose { name: u64 },
}
#[expect(
    missing_debug_implementations,
    reason = "the owned SSE reader and base64 decoder are opaque foundation machines"
)]
pub struct Link {
    limits: Limits,
    attempts: Map<Token, Attempt>,
    next: u64,
    stream: Option<crate::stream::Stream>,
    watches: crate::watches::Watches,
}
impl Link {
    pub fn new(limits: &Limits) -> Result<Self, Unusable> {
        crate::worst_case(limits).ok_or(Unusable::Limits)?;
        Ok(Self {
            limits: *limits,
            attempts: Map::with_capacity(limits.exchanges),
            next: 1,
            stream: None,
            watches: crate::watches::Watches::new(limits),
        })
    }
    #[must_use]
    pub fn max_out(&self) -> u32 {
        crate::limits::max_out(&self.limits).expect("validated output bound")
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let mut first = None;
        for (_, attempt) in &self.attempts {
            first = Some(match first {
                Some(value) => core::cmp::min(value, attempt.until),
                None => attempt.until,
            });
        }
        if let Some(stream) = &self.stream {
            first = Some(match first {
                Some(value) => core::cmp::min(value, stream.deadline()),
                None => stream.deadline(),
            });
        }
        first
    }
    pub fn reclaim(&mut self) {}
    #[must_use]
    pub fn worst_case(limits: &Limits) -> Option<u64> {
        crate::worst_case(limits)
    }
    pub fn step(&mut self, now: Time, event: Event, out: &mut Queue<Request>) {
        match event {
            Event::Open { watch, subject } => self.open(now, watch, subject, out),
            Event::Continue { watch, subject } => {
                self.forget(watch, out);
                self.open(now, watch, Watchable::Jig(subject), out);
            }
            Event::Close { watch } => self.close(now, watch, out),
            Event::StreamOpened { status } => {
                if status != 200 {
                    self.drop_stream(now, out);
                }
            }
            Event::StreamBytes { bytes } => self.stream_bytes(now, &bytes, out),
            Event::StreamEnded => self.drop_stream(now, out),
            Event::Send { attempt, key, ask } => self.send(now, attempt, key, ask, out),
            Event::Read { read, what } => self.read(now, read, what, out),
            Event::Answered { exchange, status, body } => self.answered(exchange, status, &body, out),
            Event::Failed { exchange } => {
                if let Some(attempt) = self.attempts.remove(&exchange) {
                    self.failure(attempt, out);
                }
            }
        }
    }
    pub fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        let mut expired = skein_lib::List::with_capacity(self.limits.exchanges);
        for (exchange, attempt) in &self.attempts {
            if now >= attempt.until {
                expired.push(*exchange).expect("bounded attempts");
            }
        }
        for exchange in &expired {
            let attempt = self.attempts.remove(exchange).expect("collected live exchange");
            out.push(Request::Abandon { exchange: *exchange });
            self.failure(attempt, out);
        }
        if let Some(stream) = &self.stream
            && now >= stream.deadline()
        {
            self.drop_stream(now, out);
        }
    }
    fn send(&mut self, now: Time, attempt: Token, key: RequestKey, ask: Ask, out: &mut Queue<Request>) {
        if self.attempts.len() == self.attempts.capacity() {
            lost(attempt, out);
            return;
        }
        let Some(next) = self.next.checked_add(1) else {
            lost(attempt, out);
            return;
        };
        let Ok(record) = keyed(key, ask, &self.limits.wire) else {
            lost(attempt, out);
            return;
        };
        let exchange = Token::new(self.next);
        self.next = next;
        let state = Attempt {
            token: attempt,
            owner: record.owner().clone(),
            version: record.version(),
            until: now.saturating_add(self.limits.deadline),
            kind: ExchangeKind::Attempt,
        };
        self.attempts.insert(exchange, state).expect("checked exchange capacity");
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("u32 fits usize"));
        record.encode(&mut writer).expect("measured envelope");
        out.push(Request::Exchange { exchange, path: b"/jig/v1/request", body: writer.finish() });
    }
    fn read(&mut self, now: Time, read: Token, what: Readable, out: &mut Queue<Request>) {
        let owner = match &what {
            Readable::Jig(_) => w::Owner::Jig,
            Readable::Family(value) => w::Owner::Family(
                w::Family::new(&self.limits.wire, w::FamilyParts { number: value.family })
                    .expect("unbounded family number"),
            ),
        };
        if self.attempts.len() == self.attempts.capacity() {
            read_none(read, &owner, jig_domain_client::contract::ReadNone::Unreachable, out);
            return;
        }
        let Some(next) = self.next.checked_add(1) else {
            read_none(read, &owner, jig_domain_client::contract::ReadNone::Unreachable, out);
            return;
        };
        let Ok(record) = translate::reads::owned(what, &self.limits.wire) else {
            read_none(read, &owner, jig_domain_client::contract::ReadNone::Unreachable, out);
            return;
        };
        let exchange = Token::new(self.next);
        self.next = next;
        let state = Attempt {
            token: read,
            owner,
            version: record.version(),
            until: now.saturating_add(self.limits.deadline),
            kind: ExchangeKind::Read,
        };
        self.attempts.insert(exchange, state).expect("checked exchange capacity");
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("u32 fits usize"));
        record.encode(&mut writer).expect("measured read envelope");
        out.push(Request::Exchange { exchange, path: b"/jig/v1/read", body: writer.finish() });
    }
    fn answered(&mut self, exchange: Token, status: u16, body: &[u8], out: &mut Queue<Request>) {
        let Some(attempt) = self.attempts.remove(&exchange) else { return };
        match attempt.kind {
            ExchangeKind::Attempt => {}
            ExchangeKind::WatchOpen { name } | ExchangeKind::WatchClose { name } => {
                self.watch_answer(attempt, name, status, body, out);
                return;
            }
            ExchangeKind::Read => {
                self.read_answer(attempt, status, body, out);
                return;
            }
        }
        if status != 200 {
            lost(attempt.token, out);
            return;
        }
        let Ok(record) = w::Answer::decode(&self.limits.wire, &mut Reader::new(body)) else {
            lost(attempt.token, out);
            return;
        };
        let envelope = match record {
            w::Answer::Busy | w::Answer::NotReady => RequestEnvelope::Again,
            w::Answer::SignedOut => RequestEnvelope::SignedOut,
            w::Answer::FinalAnswer(record) => {
                let answer = if record.owner() != &attempt.owner || record.version() != attempt.version {
                    Err(translate::Undecodable::Version)
                } else {
                    match record.owner() {
                        w::Owner::Jig => match translate::answers::answer(w::Answer::FinalAnswer(record), &self.limits)
                        {
                            Ok(answer) => Ok(Answered::Jig(answer)),
                            Err(problem) => Err(problem),
                        },
                        w::Owner::Family(owner) => Ok(Answered::Family(Family {
                            family: owner.number(),
                            version: record.version(),
                            bytes: skein_lib::bytes::copy_of(record.record()),
                        })),
                    }
                };
                match answer {
                    Ok(answer) => {
                        out.push(Request::Answer {
                            attempt: attempt.token,
                            envelope: RequestEnvelope::Final,
                            answer: Some(answer),
                        });
                        return;
                    }
                    Err(_) => {
                        out.push(Request::Fact { fact: Fact::UndecodableAnswer });
                        RequestEnvelope::Lost
                    }
                }
            }
        };
        out.push(Request::Answer { attempt: attempt.token, envelope, answer: None });
    }
    fn read_answer(&self, attempt: Attempt, status: u16, body: &[u8], out: &mut Queue<Request>) {
        if status != 200 {
            failed(attempt, out);
            return;
        }
        let result = match w::ReadAnswer::decode(&self.limits.wire, &mut Reader::new(body)) {
            Ok(w::ReadAnswer::None(record)) => {
                read_none(attempt.token, &attempt.owner, translate::reads::none(record.value()), out);
                return;
            }
            Ok(w::ReadAnswer::Value(record)) => read_value(&attempt, record, &self.limits),
            Err(_) => Err(translate::Undecodable::Malformed),
        };
        match result {
            Ok(result) => out.push(Request::Read { read: attempt.token, result }),
            Err(_) => {
                out.push(Request::Fact { fact: Fact::UndecodableDocument });
                failed(attempt, out);
            }
        }
    }
}
fn read_value(attempt: &Attempt, record: w::Owned, limits: &Limits) -> Result<Readout, translate::Undecodable> {
    if record.owner() != &attempt.owner || record.version() != attempt.version {
        return Err(translate::Undecodable::Version);
    }
    match record.owner() {
        w::Owner::Jig => {
            let value = translate::reads::value(record.record(), limits)?;
            Ok(Readout::Jig(jig_domain_client::contract::ReadResult::Value(value)))
        }
        w::Owner::Family(owner) => {
            let family = owner.number();
            Ok(Readout::Family { family, record: Some(record.into_parts().record) })
        }
    }
}
fn read_none(read: Token, owner: &w::Owner, why: jig_domain_client::contract::ReadNone, out: &mut Queue<Request>) {
    let result = match owner {
        w::Owner::Jig => Readout::Jig(jig_domain_client::contract::ReadResult::None(why)),
        w::Owner::Family(owner) => Readout::Family { family: owner.number(), record: None },
    };
    out.push(Request::Read { read, result });
}
fn failed(attempt: Attempt, out: &mut Queue<Request>) {
    match attempt.kind {
        ExchangeKind::Attempt => lost(attempt.token, out),
        ExchangeKind::WatchOpen { .. } | ExchangeKind::WatchClose { .. } => {
            unreachable!("watch failure belongs to its map")
        }
        ExchangeKind::Read => {
            read_none(attempt.token, &attempt.owner, jig_domain_client::contract::ReadNone::Unreachable, out);
        }
    }
}
fn lost(attempt: Token, out: &mut Queue<Request>) {
    out.push(Request::Answer { attempt, envelope: RequestEnvelope::Lost, answer: None });
}
fn keyed(key: RequestKey, ask: Ask, limits: &w::Limits) -> Result<w::Keyed, w::Problem> {
    let (owner, version, request) = match ask {
        Ask::Jig(ask) => {
            let record = translate::asks::ask(&ask, limits)?;
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("u32 fits usize"));
            record.encode(&mut writer).expect("measured ask");
            (w::Owner::Jig, wire::VERSION, writer.finish())
        }
        Ask::Family(record) => {
            let owner = w::Family::new(limits, w::FamilyParts { number: record.family })?;
            (w::Owner::Family(owner), record.version, record.bytes)
        }
    };
    w::Keyed::new(limits, w::KeyedParts { key, owner, version, request })
}

impl Link {
    fn token(&mut self) -> Option<Token> {
        let next = self.next.checked_add(1)?;
        let token = Token::new(self.next);
        self.next = next;
        Some(token)
    }
    fn open(&mut self, now: Time, watch: Token, subject: Watchable, out: &mut Queue<Request>) {
        if self.watches.0.contains_key(&watch) {
            return;
        }
        if self.watches.0.len() == self.watches.0.capacity() {
            out.push(Request::Ended { watch, end: jig_domain_client::contract::WatchEnd::Busy });
            return;
        }
        let Ok(subject) = translate::watches::owned(subject, &self.limits.wire) else {
            out.push(Request::Ended { watch, end: jig_domain_client::contract::WatchEnd::Dropped });
            return;
        };
        let Some(name) = self.token() else {
            out.push(Request::Ended { watch, end: jig_domain_client::contract::WatchEnd::Dropped });
            return;
        };
        let opening = crate::watches::Opening {
            name: name.raw(),
            owner: subject.owner().clone(),
            version: subject.version(),
            subject: Some(subject),
            live: false,
            closing: false,
        };
        self.watches.0.insert(watch, opening).expect("checked watch capacity");
        match &self.stream {
            None => {
                self.stream = Some(crate::stream::Stream::new(&self.limits, now));
                out.push(Request::OpenStream);
            }
            Some(stream) => {
                if stream.name.is_some() {
                    self.send_open(now, watch, out);
                }
            }
        }
    }
    fn send_open(&mut self, now: Time, watch: Token, out: &mut Queue<Request>) {
        let Some(stream) = &self.stream else { return };
        let Some(stream) = stream.name else { return };
        if self.attempts.len() == self.attempts.capacity() {
            self.watches.end(watch, jig_domain_client::contract::WatchEnd::Busy, out);
            return;
        }
        let Some(exchange) = self.token() else {
            self.watches.end(watch, jig_domain_client::contract::WatchEnd::Dropped, out);
            return;
        };
        let opening = self.watches.0.get_mut(&watch).expect("pending opening");
        let Some(subject) = opening.subject.take() else { return };
        let record = w::Open::new(&self.limits.wire, w::OpenParts { stream, watch: opening.name, subject })
            .expect("checked owned subject");
        let attempt = Attempt {
            token: watch,
            owner: opening.owner.clone(),
            version: opening.version,
            until: now.saturating_add(self.limits.deadline),
            kind: ExchangeKind::WatchOpen { name: opening.name },
        };
        self.attempts.insert(exchange, attempt).expect("checked opening exchange capacity");
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("u32 fits usize"));
        record.encode(&mut writer).expect("measured opening");
        out.push(Request::Exchange { exchange, path: b"/jig/v1/watch", body: writer.finish() });
    }
    fn close(&mut self, now: Time, watch: Token, out: &mut Queue<Request>) {
        let Some(opening) = self.watches.0.get(&watch) else { return };
        if opening.closing {
            return;
        }
        if opening.subject.is_some() {
            self.watches.end(watch, jig_domain_client::contract::WatchEnd::Closed, out);
            return;
        }
        let name = opening.name;
        let Some(stream) = &self.stream else { return };
        let stream = stream.name.expect("sent opening has hello");
        self.cancel_open(watch, name, out);
        if self.attempts.len() == self.attempts.capacity() {
            self.watches.end(watch, jig_domain_client::contract::WatchEnd::Dropped, out);
            return;
        }
        let Some(exchange) = self.token() else { return self.drop_stream(now, out) };
        let opening = self.watches.0.get_mut(&watch).expect("closing watch");
        opening.closing = true;
        let attempt = Attempt {
            token: watch,
            owner: opening.owner.clone(),
            version: opening.version,
            until: now.saturating_add(self.limits.deadline),
            kind: ExchangeKind::WatchClose { name },
        };
        self.attempts.insert(exchange, attempt).expect("checked closing exchange capacity");
        self.unwatch(exchange, stream, name, out);
    }
    fn cancel_open(&mut self, watch: Token, name: u64, out: &mut Queue<Request>) {
        let mut tokens = skein_lib::List::with_capacity(self.limits.exchanges);
        for (exchange, attempt) in &self.attempts {
            match attempt.kind {
                ExchangeKind::WatchOpen { name: old } | ExchangeKind::WatchClose { name: old } => {
                    if attempt.token == watch && old == name {
                        tokens.push(*exchange).expect("bounded exchanges");
                    }
                }
                ExchangeKind::Attempt | ExchangeKind::Read => {}
            }
        }
        for exchange in &tokens {
            assert!(self.attempts.remove(exchange).is_some(), "collected live watch exchange");
            out.push(Request::Abandon { exchange: *exchange });
        }
    }
    fn forget(&mut self, watch: Token, out: &mut Queue<Request>) {
        if let Some(opening) = self.watches.0.remove(&watch) {
            self.cancel_open(watch, opening.name, out);
            if opening.subject.is_none()
                && let Some(stream) = &self.stream
            {
                let stream = stream.name.expect("sent opening has hello");
                if let Some(exchange) = self.token() {
                    self.unwatch(exchange, stream, opening.name, out);
                }
            }
        }
    }
    fn unwatch(&self, exchange: Token, stream: [u8; 16], watch: u64, out: &mut Queue<Request>) {
        let record =
            w::WatchClose::new(&self.limits.wire, w::WatchCloseParts { stream, watch }).expect("fixed closing names");
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("u32 fits usize"));
        record.encode(&mut writer).expect("measured closing");
        out.push(Request::Exchange { exchange, path: b"/jig/v1/unwatch", body: writer.finish() });
    }
    fn watch_answer(&mut self, attempt: Attempt, name: u64, status: u16, body: &[u8], out: &mut Queue<Request>) {
        if self.watches.named(name) != Some(attempt.token) {
            return;
        }
        if let ExchangeKind::WatchOpen { .. } = attempt.kind
            && self.watches.0.get(&attempt.token).expect("named opening").live
        {
            return;
        }
        if status != 200 {
            self.failure(attempt, out);
            return;
        }
        match attempt.kind {
            ExchangeKind::WatchOpen { .. } => match w::OpenAnswer::decode(&self.limits.wire, &mut Reader::new(body)) {
                Ok(w::OpenAnswer::Admitted) => self.watches.opened(attempt.token, out),
                Ok(w::OpenAnswer::Refused(why)) => {
                    self.watches.end(attempt.token, translate::watches::refused(why.why()), out);
                }
                Err(_) => self.failure(attempt, out),
            },
            ExchangeKind::WatchClose { .. } => {
                match w::CloseAnswer::decode(&self.limits.wire, &mut Reader::new(body)) {
                    Ok(w::CloseAnswer::Closed) => {
                        self.watches.end(attempt.token, jig_domain_client::contract::WatchEnd::Closed, out);
                    }
                    Ok(w::CloseAnswer::SignedOut) => {
                        self.watches.end(attempt.token, jig_domain_client::contract::WatchEnd::SignedOut, out);
                    }
                    Err(_) => self.failure(attempt, out),
                }
            }
            ExchangeKind::Attempt | ExchangeKind::Read => unreachable!("watch exchange kind"),
        }
    }
    fn failure(&mut self, attempt: Attempt, out: &mut Queue<Request>) {
        match attempt.kind {
            ExchangeKind::WatchOpen { name } => {
                if let Some(opening) = self.watches.0.get(&attempt.token)
                    && opening.name == name
                    && !opening.live
                {
                    self.watches.end(attempt.token, jig_domain_client::contract::WatchEnd::Dropped, out);
                }
            }
            ExchangeKind::WatchClose { name } => {
                if self.watches.named(name) == Some(attempt.token) {
                    self.watches.end(attempt.token, jig_domain_client::contract::WatchEnd::Dropped, out);
                }
            }
            ExchangeKind::Attempt | ExchangeKind::Read => failed(attempt, out),
        }
    }
    fn stream_bytes(&mut self, now: Time, bytes: &[u8], out: &mut Queue<Request>) {
        let Some(stream) = &mut self.stream else { return };
        if bytes.len() > usize::try_from(self.limits.stream_chunk()).expect("u32 fits usize") {
            out.push(Request::Fact { fact: Fact::UndecodableEvent });
            self.drop_stream(now, out);
            return;
        }
        if bytes.is_empty() {
            return;
        }
        stream.progress(now);
        let mut at = 0_usize;
        for _ in 0..bytes.len() {
            if at == bytes.len() {
                break;
            }
            let remaining = bytes.get(at..).expect("remaining stream bytes");
            let piece = line_piece(remaining, self.limits.stream_chunk());
            at = at.checked_add(piece.len()).expect("piece within input");
            let Some(stream) = &mut self.stream else { break };
            let result = stream.piece(piece, &self.limits);
            let result = match result {
                Ok((record, heartbeat)) => {
                    if heartbeat {
                        self.watches.alive(out);
                    }
                    match record {
                        Some(record) => self.stream_event(now, record, out),
                        None => Ok(()),
                    }
                }
                Err(problem) => Err(problem),
            };
            if result.is_err() {
                out.push(Request::Fact { fact: Fact::UndecodableEvent });
                self.drop_stream(now, out);
                break;
            }
        }
    }
    fn stream_event(
        &mut self,
        now: Time,
        record: w::StreamEvent,
        out: &mut Queue<Request>,
    ) -> Result<(), crate::Undecodable> {
        let stream = self.stream.as_mut().expect("active stream");
        match record {
            w::StreamEvent::Hello(hello) => {
                stream.hello(&hello)?;
                for served in hello.served() {
                    if served.first() > served.last() {
                        return Err(crate::Undecodable::Malformed);
                    }
                    let family = match served.owner() {
                        w::Owner::Jig => None,
                        w::Owner::Family(owner) => Some(owner.number()),
                    };
                    out.push(Request::Hello {
                        family,
                        versions: jig_domain_client::contract::VersionRange {
                            first: served.first(),
                            last: served.last(),
                        },
                    });
                }
                let mut pending = skein_lib::List::with_capacity(self.limits.client.watches);
                for (watch, opening) in &self.watches.0 {
                    if opening.subject.is_some() {
                        pending.push(*watch).expect("bounded watches");
                    }
                }
                for watch in &pending {
                    self.send_open(now, *watch, out);
                }
            }
            w::StreamEvent::Delivery(value) => {
                if stream.name.is_none() {
                    return Err(crate::Undecodable::Malformed);
                }
                self.watches.delivery(value, &self.limits, out)?;
            }
            w::StreamEvent::Ended(value) => {
                if stream.name.is_none() {
                    return Err(crate::Undecodable::Malformed);
                }
                if let Some(watch) = self.watches.named(value.watch()) {
                    self.cancel_open(watch, value.watch(), out);
                    self.watches.end(watch, translate::watches::ended(value.why()), out);
                }
            }
            w::StreamEvent::Text(value) => {
                if stream.name.is_none() {
                    return Err(crate::Undecodable::Malformed);
                }
                if let Some(watch) = self.watches.named(value.watch()) {
                    let opening = self.watches.0.get(&watch).expect("named run watch");
                    if !opening.closing {
                        if opening.owner != w::Owner::Jig || opening.version != wire::VERSION {
                            return Err(crate::Undecodable::Version);
                        }
                        let value = translate::live::run_text(value.into_parts().value, &self.limits)?;
                        self.watches.opened(watch, out);
                        out.push(Request::Watch {
                            watch,
                            event: jig_domain_client::contract::WatchEvent::Delivered(
                                jig_domain_client::contract::Delivery::Run(Box::new(value)),
                            ),
                        });
                    }
                }
            }
        }
        self.watches.alive(out);
        Ok(())
    }
    fn drop_stream(&mut self, _now: Time, out: &mut Queue<Request>) {
        let Some(stream) = self.stream.take() else { return };
        let mut watches = skein_lib::List::with_capacity(self.limits.client.watches);
        for (watch, _) in &self.watches.0 {
            watches.push(*watch).expect("bounded watches");
        }
        for watch in &watches {
            let name = self.watches.0.get(watch).expect("collected watch").name;
            self.cancel_open(*watch, name, out);
            self.watches.end(*watch, jig_domain_client::contract::WatchEnd::Dropped, out);
        }
        if let Some(stream) = stream.name
            && let Some(exchange) = self.token()
        {
            let record = w::Gone::new(&self.limits.wire, w::GoneParts { stream }).expect("fixed stream name");
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("u32 fits usize"));
            record.encode(&mut writer).expect("measured stream gone");
            out.push(Request::Exchange { exchange, path: b"/jig/v1/stream-gone", body: writer.finish() });
        }
        out.push(Request::CloseStream);
        out.push(Request::Fact { fact: Fact::StreamDropped });
    }
}

fn line_piece(bytes: &[u8], chunk: u32) -> &[u8] {
    let maximum = core::cmp::min(bytes.len(), usize::try_from(chunk).expect("u32 fits usize"));
    let piece = bytes.get(..maximum).expect("bounded scan");
    for (at, byte) in piece.iter().enumerate() {
        if *byte == b'\r' || *byte == b'\n' {
            return piece.get(..at.saturating_add(1)).expect("scan stops at ending");
        }
    }
    piece
}
