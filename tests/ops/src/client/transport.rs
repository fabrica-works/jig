//! Opaque shell token routes and the finite Ops protocol adapter, beside the shared world.
#![allow(clippy::wildcard_imports, reason = "adapter exhaustively translates owner boundary vocabularies")]
use super::{Document, Ops, OpsScript, Subject};
use jig_client::contract::*;
use jig_client_world::{harness::*, observe};
use jig_ops_client_domain::{self as domain, Event, JigOut, Request, infrastructure as infra, observability as obs};
use jig_ops_connectors::{Connector, number};
use jig_scripted_engine::{self as engine, ApplicationScope, Envelope, Up};
use sha2::{Digest, Sha256};
use skein_lib::{RequestEnvelope, Time, Token, Wall};
use skein_world::domain::Ledger;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Channel {
    Jig,
    Observability,
    Infrastructure,
}
#[derive(Clone, Copy)]
struct Route {
    client: ClientId,
    local: Token,
    channel: Channel,
}
/// Scripted jig truth and Ops documents with one shell route per operation lifetime.
pub struct Scripted {
    pub engine: engine::Engine<OpsScript>,
    routes: BTreeMap<Token, Route>,
    parties: BTreeMap<ClientId, u64>,
    hellos: BTreeSet<ClientId>,
    ledger: Ledger<Token, ()>,
    serial: u64,
    committed: usize,
    observed: Vec<Observed>,
    last_attempt: Option<Token>,
    last_watch: Option<Token>,
    last_object: Option<ObjectKey>,
    recovery: Option<Time>,
    sign_ins: Vec<u64>,
    pending_forget: Vec<ObjectKey>,
}
impl Scripted {
    /// Installs owner supplied truth and bounded application documents.
    #[must_use]
    pub fn new(script: engine::Script, seed: u64, ops: OpsScript) -> Self {
        let mut observed = vec![];
        for source in &script.truth.sources {
            source_facts(source, 0, &mut observed);
        }
        Self {
            engine: engine::Engine::with_application(script, seed, ops).expect("Ops fixture admitted"),
            routes: BTreeMap::new(),
            parties: BTreeMap::new(),
            hellos: BTreeSet::new(),
            ledger: Ledger::new("Ops shell operation"),
            serial: 0,
            committed: 0,
            observed,
            last_attempt: None,
            last_watch: None,
            last_object: None,
            recovery: None,
            sign_ins: vec![],
            pending_forget: vec![],
        }
    }
    fn forget(&mut self, now: Time, wall: Wall, key: ObjectKey) {
        self.engine.act(now, wall, engine::Change::Forget { key }).expect("owner object disappearance");
        for mut source in self.engine.truth().sources.clone() {
            if let engine::Rows::Inbox(rows) = &mut source.rows {
                let before = rows.len();
                rows.retain(|row| row.object != key);
                if rows.len() != before {
                    self.engine.act(now, wall, engine::Change::Source(source)).expect("owner listing removal");
                }
            }
        }
    }
    fn open(&mut self, client: ClientId, local: Token, channel: Channel) -> Token {
        self.serial = self.serial.checked_add(1).expect("finite shell route identities");
        let token = Token::new(self.serial);
        self.ledger.open(token, ());
        assert!(self.routes.insert(token, Route { client, local, channel }).is_none());
        token
    }
    fn token(&self, client: ClientId, local: Token, channel: Channel) -> Option<Token> {
        self.routes.iter().find_map(|(token, route)| {
            (route.client == client && route.local == local && route.channel == channel).then_some(*token)
        })
    }
    fn recover(&mut self, now: Time, wall: Wall) {
        if self.recovery.is_some_and(|at| now >= at) {
            self.recovery = None;
            self.observed.push(Observed::Reachable { live: true });
            for key in std::mem::take(&mut self.pending_forget) {
                self.forget(now, wall, key);
            }
            for party in std::mem::take(&mut self.sign_ins) {
                self.engine.act(now, wall, engine::Change::SignIn { sign_in: party, party }).expect("queued sign in");
            }
        }
    }
}
fn envelope(envelope: Envelope) -> RequestEnvelope {
    match envelope {
        Envelope::Final => RequestEnvelope::Final,
        Envelope::Again => RequestEnvelope::Again,
        Envelope::Lost => RequestEnvelope::Lost,
        Envelope::SignedOut => RequestEnvelope::SignedOut,
    }
}
fn scope(subject: Subject) -> ApplicationScope {
    match subject {
        Subject::Observability(obs::Subject { scope: obs::Scope::Project(project), .. })
        | Subject::Infrastructure(infra::Subject { scope: infra::Scope::Project(project), .. }) => {
            ApplicationScope::Project(project)
        }
        Subject::Observability(obs::Subject { scope: obs::Scope::Deployment, .. })
        | Subject::Infrastructure(infra::Subject { scope: infra::Scope::Deployment, .. }) => {
            ApplicationScope::Deployment
        }
    }
}
fn connector(channel: Channel) -> u16 {
    match channel {
        Channel::Observability => number(Connector::Observability),
        Channel::Infrastructure => number(Connector::Infrastructure),
        Channel::Jig => unreachable!("jig is not an application owner"),
    }
}
fn watch(channel: Channel, local: Token, event: WatchEvent) -> Event {
    match channel {
        Channel::Jig => Event::Jig(jig_client::Event::Watch { watch: local, event }),
        Channel::Observability => Event::Observability(obs::Event::Watch {
            watch: local,
            event: match event {
                WatchEvent::Opened => obs::WatchEvent::Opened,
                WatchEvent::Alive => obs::WatchEvent::Alive,
                WatchEvent::Delivered(_) => unreachable!("Ops docs use Up::App"),
            },
        }),
        Channel::Infrastructure => Event::Infrastructure(infra::Event::Watch {
            watch: local,
            event: match event {
                WatchEvent::Opened => infra::WatchEvent::Opened,
                WatchEvent::Alive => infra::WatchEvent::Alive,
                WatchEvent::Delivered(_) => unreachable!("Ops docs use Up::App"),
            },
        }),
    }
}
fn end(channel: Channel, local: Token, why: WatchEnd) -> Event {
    match channel {
        Channel::Jig => Event::Jig(jig_client::Event::Ended { watch: local, end: why }),
        Channel::Observability => Event::Observability(obs::Event::Ended {
            watch: local,
            end: match why {
                WatchEnd::Closed => obs::WatchEnd::Closed,
                WatchEnd::SignedOut => obs::WatchEnd::SignedOut,
                WatchEnd::Dropped
                | WatchEnd::Unknown
                | WatchEnd::NotAllowed
                | WatchEnd::Busy
                | WatchEnd::SubjectEnded
                | WatchEnd::Stopping => obs::WatchEnd::Dropped,
            },
        }),
        Channel::Infrastructure => Event::Infrastructure(infra::Event::Ended {
            watch: local,
            end: match why {
                WatchEnd::Closed => infra::WatchEnd::Closed,
                WatchEnd::SignedOut => infra::WatchEnd::SignedOut,
                WatchEnd::Dropped
                | WatchEnd::Unknown
                | WatchEnd::NotAllowed
                | WatchEnd::Busy
                | WatchEnd::SubjectEnded
                | WatchEnd::Stopping => infra::WatchEnd::Dropped,
            },
        }),
    }
}
impl Front<Ops> for Scripted {
    fn bind(&mut self, client: ClientId, party: u64) {
        self.parties.insert(client, party);
        self.hellos.insert(client);
    }
    #[expect(
        clippy::too_many_lines,
        reason = "the three owner wire vocabularies are exhaustively translated without interpreting payloads"
    )]
    fn wire(&mut self, client: ClientId, now: Time, wall: Wall, request: Request) {
        let party = self.parties[&client];
        match request {
            Request::Send { attempt, key, ask } => {
                let channel = match &ask {
                    domain::Ask::Jig(_) => Channel::Jig,
                    domain::Ask::Infrastructure(_) => Channel::Infrastructure,
                };
                let attempt = self.open(client, attempt, channel);
                self.last_attempt = Some(attempt);
                match ask {
                    domain::Ask::Jig(ask) => {
                        self.engine.send(now, wall, party, key, attempt, *ask).expect("jig send admitted");
                    }
                    domain::Ask::Infrastructure(ask) => {
                        let infra::Ask::MoveEnd { project, environment, revision, until } = &ask;
                        let mut canonical = Sha256::new();
                        canonical.update(b"Ops.MoveEnd.v1");
                        canonical.update(project.to_be_bytes());
                        canonical.update(environment.to_be_bytes());
                        canonical.update(revision.value.to_be_bytes());
                        canonical.update(until.as_nanos().to_be_bytes());
                        self.engine
                            .send_application(
                                now,
                                wall,
                                party,
                                key,
                                attempt,
                                number(Connector::Infrastructure),
                                *project,
                                canonical.finalize().into(),
                                ask,
                            )
                            .expect("environment send admitted");
                    }
                }
            }
            Request::Jig(JigOut::Read { read, what }) => {
                self.last_object = match &what {
                    Read::Task { task } => Some(ObjectKey::Task(*task)),
                    Read::Proposal { proposal } => Some(ObjectKey::Proposal(*proposal)),
                    Read::Escalation { task } => Some(ObjectKey::Escalation(*task)),
                    Read::Question { task, message } => Some(ObjectKey::Question { task: *task, message: *message }),
                    Read::PersonTask { task } => Some(ObjectKey::PersonTask(*task)),
                    Read::Result { task } => Some(ObjectKey::Result(*task)),
                    Read::Frame
                    | Read::Ended { .. }
                    | Read::History { .. }
                    | Read::Activity { .. }
                    | Read::Inbox { .. }
                    | Read::Conversations { .. }
                    | Read::Goals { .. }
                    | Read::Spend { .. }
                    | Read::Project { .. }
                    | Read::Notes { .. }
                    | Read::Deployment => self.last_object,
                };
                let token = self.open(client, read, Channel::Jig);
                self.engine.read(now, party, token, what).expect("jig read admitted");
            }
            Request::Jig(JigOut::Open { watch, subject }) => {
                let token = self.open(client, watch, Channel::Jig);
                self.last_watch = Some(token);
                self.engine.open(now, party, token, subject).expect("jig watch admitted");
            }
            Request::Jig(JigOut::Continue { watch, subject }) => {
                let token = self.token(client, watch, Channel::Jig).expect("live run route");
                let SubjectRun { turn, offset } = run_position(&subject);
                {
                    self.engine.continue_run(token, turn, offset).expect("continue live run");
                }
            }
            Request::Jig(JigOut::Close { watch }) => {
                if let Some(token) = self.token(client, watch, Channel::Jig) {
                    self.engine.close(now, token);
                }
            }
            Request::Observability(out) => match out {
                obs::Out::Read { read, subject } => {
                    let token = self.open(client, read, Channel::Observability);
                    let subject = Subject::Observability(subject);
                    self.engine
                        .read_scoped_application(
                            now,
                            party,
                            token,
                            connector(Channel::Observability),
                            scope(subject),
                            &subject,
                        )
                        .expect("incident read admitted");
                }
                obs::Out::Open { watch, subject } => {
                    let token = self.open(client, watch, Channel::Observability);
                    self.last_watch = Some(token);
                    let subject = Subject::Observability(subject);
                    self.engine
                        .open_scoped_application(
                            now,
                            party,
                            token,
                            connector(Channel::Observability),
                            scope(subject),
                            subject,
                        )
                        .expect("incident watch admitted");
                }
                obs::Out::Close { watch } => {
                    if let Some(token) = self.token(client, watch, Channel::Observability) {
                        self.engine.close(now, token);
                    }
                }
                obs::Out::Fact { .. } => unreachable!("facts classified by shell"),
            },
            Request::Infrastructure(out) => match out {
                infra::Out::Read { read, subject } => {
                    let token = self.open(client, read, Channel::Infrastructure);
                    let subject = Subject::Infrastructure(subject);
                    self.engine
                        .read_scoped_application(
                            now,
                            party,
                            token,
                            connector(Channel::Infrastructure),
                            scope(subject),
                            &subject,
                        )
                        .expect("environment read admitted");
                }
                infra::Out::Open { watch, subject } => {
                    let token = self.open(client, watch, Channel::Infrastructure);
                    self.last_watch = Some(token);
                    let subject = Subject::Infrastructure(subject);
                    self.engine
                        .open_scoped_application(
                            now,
                            party,
                            token,
                            connector(Channel::Infrastructure),
                            scope(subject),
                            subject,
                        )
                        .expect("environment watch admitted");
                }
                infra::Out::Close { watch } => {
                    if let Some(token) = self.token(client, watch, Channel::Infrastructure) {
                        self.engine.close(now, token);
                    }
                }
                infra::Out::Fact { .. } => unreachable!("facts classified by shell"),
            },
            Request::Save { .. }
            | Request::Erase { .. }
            | Request::Show { .. }
            | Request::SignIn { .. }
            | Request::Fact { .. }
            | Request::Jig(JigOut::Answered { .. } | JigOut::Unknown { .. }) => {
                unreachable!("nonwire shell obligation")
            }
        }
    }
    fn poll(&mut self, now: Time, wall: Wall) -> Vec<(ClientId, Event)> {
        self.recover(now, wall);
        let mut delivered: Vec<_> = std::mem::take(&mut self.hellos)
            .into_iter()
            .map(|client| (client, Event::Jig(jig_client::Event::Hello { versions: self.engine.versions() })))
            .collect();
        for up in self.engine.poll(now) {
            let token = match &up {
                Up::Answer { attempt, .. } | Up::ApplicationAnswer { attempt, .. } => *attempt,
                Up::Read { token, .. }
                | Up::Watch { token, .. }
                | Up::ApplicationRead { token, .. }
                | Up::Ended { token, .. } => *token,
                Up::App { watch, .. } => *watch,
            };
            let terminal = matches!(
                &up,
                Up::Answer { .. }
                    | Up::ApplicationAnswer { .. }
                    | Up::Read { .. }
                    | Up::ApplicationRead { .. }
                    | Up::Ended { .. }
            );
            if terminal {
                self.ledger.end(token);
            }
            let Some(route) = self.routes.get(&token).copied() else { continue };
            if terminal {
                self.routes.remove(&token);
            }
            let local = route.local;
            let event = match up {
                Up::Answer { envelope: env, answer, .. } => {
                    Event::Answer { attempt: local, envelope: envelope(env), answer: answer.map(domain::Answered::Jig) }
                }
                Up::ApplicationAnswer { envelope: env, answer, .. } => Event::Answer {
                    attempt: local,
                    envelope: envelope(env),
                    answer: answer.map(|result| {
                        domain::Answered::Infrastructure(match result {
                            Ok(answer) => answer,
                            Err(refusal) => infra::Answer::EntranceRefused(
                                jig_client::contract::EntranceRefusal::new(refusal)
                                    .expect("scripted engine supplies bounded entrance facts"),
                            ),
                        })
                    }),
                },
                Up::Read { terminal, .. } => Event::Jig(jig_client::Event::Read { read: local, result: terminal }),
                Up::Watch { event, .. } => watch(route.channel, local, event),
                Up::App { document, .. } => match document {
                    Document::Observability(doc) => Event::Observability(obs::Event::Watch {
                        watch: local,
                        event: obs::WatchEvent::Delivered(Box::new(doc)),
                    }),
                    Document::Infrastructure(doc) => Event::Infrastructure(infra::Event::Watch {
                        watch: local,
                        event: infra::WatchEvent::Delivered(Box::new(doc)),
                    }),
                },
                Up::ApplicationRead { terminal, .. } => match (route.channel, terminal) {
                    (Channel::Observability, Ok(Document::Observability(doc))) => {
                        Event::Observability(obs::Event::Read {
                            read: local,
                            result: obs::ReadResult::Value(Box::new(doc)),
                        })
                    }
                    (Channel::Infrastructure, Ok(Document::Infrastructure(doc))) => {
                        Event::Infrastructure(infra::Event::Read {
                            read: local,
                            result: infra::ReadResult::Value(Box::new(doc)),
                        })
                    }
                    (Channel::Observability, Err(reason)) => Event::Observability(obs::Event::Read {
                        read: local,
                        result: match reason {
                            ReadNone::SignedOut => obs::ReadResult::SignedOut,
                            ReadNone::Busy | ReadNone::Unreachable => obs::ReadResult::Busy,
                            ReadNone::Unknown | ReadNone::Refused | ReadNone::Gone => obs::ReadResult::Unknown,
                        },
                    }),
                    (Channel::Infrastructure, Err(reason)) => Event::Infrastructure(infra::Event::Read {
                        read: local,
                        result: match reason {
                            ReadNone::SignedOut => infra::ReadResult::SignedOut,
                            ReadNone::Busy | ReadNone::Unreachable => infra::ReadResult::Busy,
                            ReadNone::Unknown | ReadNone::Refused | ReadNone::Gone => infra::ReadResult::Unknown,
                        },
                    }),
                    (Channel::Jig, _)
                    | (Channel::Observability, Ok(Document::Infrastructure(_)))
                    | (Channel::Infrastructure, Ok(Document::Observability(_))) => {
                        panic!("application payload owner mismatch")
                    }
                },
                Up::Ended { why, .. } => end(route.channel, local, why),
            };
            delivered.push((route.client, event));
        }
        delivered
    }
    fn next_due(&self) -> Option<Time> {
        if !self.hellos.is_empty() {
            return Some(Time::ZERO);
        }
        match (self.recovery, self.engine.next_due()) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, None) | (None, a) => a,
        }
    }
    fn observed(&mut self) -> Vec<Observed> {
        let mut out = std::mem::take(&mut self.observed);
        for commit in &self.engine.commits()[self.committed..] {
            out.push(Observed::Durable { commit: commit.number, at: commit.durable.expect("durable commit ledger") });
            if let (Some(party), Some(key)) = (commit.party, commit.key) {
                out.push(Observed::Made { party, key, commit: commit.number });
            }
            let mut facts = vec![];
            for change in &commit.changes {
                change_facts(change, commit.number, &mut facts);
            }
            facts.sort_by_key(|fact| !matches!(fact, Observed::Ended { .. }));
            out.extend(facts);
        }
        self.committed = self.engine.commits().len();
        out
    }
    fn fault(&mut self, now: Time, fault: FrontFault) {
        let fault = match fault {
            FrontFault::Busy => engine::Fault::Busy,
            FrontFault::NotReady => engine::Fault::NotReady,
            FrontFault::Slow { span } => engine::Fault::Slow { span },
            FrontFault::Restart { offline } => {
                self.observed.push(Observed::Reachable { live: false });
                self.recovery = Some(now.saturating_add(offline));
                engine::Fault::Restart { offline }
            }
            FrontFault::DropWatch => {
                let Some(watch) = self.last_watch else { return };
                engine::Fault::DropWatch { watch }
            }
            FrontFault::NoAnswer => {
                let Some(attempt) = self.last_attempt else { return };
                engine::Fault::Lose { attempt }
            }
            FrontFault::SignedOut => {
                engine::Fault::SignOut { sign_in: *self.parties.values().next().expect("bound client") }
            }
            FrontFault::Versions => {
                self.hellos.extend(self.parties.keys().copied());
                engine::Fault::Versions(VersionRange { first: 2, last: 2 })
            }
            FrontFault::ObjectGone => {
                if let Some(key) = self.last_object {
                    if self.recovery.is_some_and(|at| now < at) {
                        self.pending_forget.push(key);
                    } else {
                        self.forget(now, Wall::from_nanos(now.as_nanos()), key);
                    }
                }
                return;
            }
        };
        self.engine.fault(now, fault);
    }
    fn reload(&mut self, client: ClientId, now: Time) {
        self.hellos.insert(client);
        let abandoned: Vec<_> =
            self.routes.iter().filter_map(|(token, route)| (route.client == client).then_some(*token)).collect();
        for token in abandoned {
            self.engine.close(now, token);
            self.routes.remove(&token);
        }
    }
    fn received(&mut self, client: ClientId, event: &Event) {
        let selected = match event {
            Event::Jig(jig_client::Event::Watch { watch, event: WatchEvent::Delivered(_), .. }) => {
                Some((*watch, Channel::Jig))
            }
            Event::Observability(obs::Event::Watch { watch, event: obs::WatchEvent::Delivered(_) }) => {
                Some((*watch, Channel::Observability))
            }
            Event::Infrastructure(infra::Event::Watch { watch, event: infra::WatchEvent::Delivered(_) }) => {
                Some((*watch, Channel::Infrastructure))
            }
            Event::Start { .. }
            | Event::Went { .. }
            | Event::Act { .. }
            | Event::Answer { .. }
            | Event::Jig(_)
            | Event::Observability(_)
            | Event::Infrastructure(_)
            | Event::SignedOut => None,
        };
        if let Some((local, channel)) = selected
            && let Some(token) = self.token(client, local, channel)
        {
            self.engine.taken(token);
        }
    }
    fn sign_in(&mut self, client: ClientId, now: Time, wall: Wall) {
        let party = self.parties[&client];
        if self.recovery.is_some_and(|at| now < at) {
            if !self.sign_ins.contains(&party) {
                self.sign_ins.push(party);
            }
        } else {
            self.engine.act(now, wall, engine::Change::SignIn { sign_in: party, party }).expect("sign in admitted");
        }
    }
}
struct SubjectRun {
    turn: Option<jig_client::contract::ProvisionalTurnName>,
    offset: u32,
}
fn run_position(subject: &jig_client::contract::Subject) -> SubjectRun {
    match subject {
        jig_client::contract::Subject::Run { turn, offset, .. } => SubjectRun { turn: *turn, offset: *offset },
        jig_client::contract::Subject::Frame
        | jig_client::contract::Subject::Task { .. }
        | jig_client::contract::Subject::Tree { .. }
        | jig_client::contract::Subject::Chips { .. }
        | jig_client::contract::Subject::Inbox { .. }
        | jig_client::contract::Subject::Goals { .. }
        | jig_client::contract::Subject::Tasks { .. }
        | jig_client::contract::Subject::Conversations
        | jig_client::contract::Subject::Deployment => unreachable!("only run watches continue"),
    }
}
fn source_facts(source: &engine::Source, commit: u64, out: &mut Vec<Observed>) {
    if let (Read::Activity { task, .. }, engine::Rows::Activity(rows)) = (&source.read, &source.rows) {
        let mut messages = vec![];
        for row in rows {
            if let Activity::Words { message, words, .. } = &row.event {
                messages.push(*message);
                out.push(Observed::Kept { task: *task, message: *message, words: words.bytes().to_vec(), commit });
            }
        }
        out.push(Observed::Retained { task: *task, messages, commit });
    }
}
fn change_facts(change: &engine::Change, commit: u64, out: &mut Vec<Observed>) {
    match change {
        engine::Change::Source(source) => source_facts(source, commit, out),
        engine::Change::Forget { key } => out.push(Observed::Stopped { card: observe::key(*key), commit }),
        engine::Change::Object { key, value, .. } => {
            let terminal = match value {
                ReadValue::Task(task) => task.result.is_some(),
                ReadValue::Result(_) => true,
                ReadValue::Proposal(row) => row.ending.is_some(),
                ReadValue::Escalation(row) => row.ending.is_some(),
                ReadValue::Question(row) => row.ending.is_some(),
                ReadValue::PersonTask(row) => row.ending.is_some(),
                ReadValue::Frame(_)
                | ReadValue::Ended(_)
                | ReadValue::OwnerEnded(_)
                | ReadValue::History(_)
                | ReadValue::Activity(_)
                | ReadValue::Inbox(_)
                | ReadValue::OwnerInbox(_)
                | ReadValue::OwnerProposal(_)
                | ReadValue::OwnerPersonTask(_)
                | ReadValue::OwnerEscalation(_)
                | ReadValue::Conversations(_)
                | ReadValue::Goals(_)
                | ReadValue::Spend(_)
                | ReadValue::OwnerSpend(_)
                | ReadValue::Project(_)
                | ReadValue::OwnerProject(_)
                | ReadValue::Notes(_)
                | ReadValue::Deployment(_) => false,
            };
            if terminal {
                out.push(Observed::Stopped { card: observe::key(*key), commit });
            }
            match value {
                ReadValue::Task(task) if task.result.is_some() => {
                    out.push(Observed::Ended { task: task.chip.task, commit });
                }
                ReadValue::Result(result) => out.push(Observed::Ended { task: result.task.task, commit }),
                ReadValue::Task(_)
                | ReadValue::Frame(_)
                | ReadValue::Proposal(_)
                | ReadValue::Escalation(_)
                | ReadValue::Question(_)
                | ReadValue::PersonTask(_)
                | ReadValue::Ended(_)
                | ReadValue::OwnerEnded(_)
                | ReadValue::History(_)
                | ReadValue::Activity(_)
                | ReadValue::Inbox(_)
                | ReadValue::OwnerInbox(_)
                | ReadValue::OwnerProposal(_)
                | ReadValue::OwnerPersonTask(_)
                | ReadValue::OwnerEscalation(_)
                | ReadValue::Conversations(_)
                | ReadValue::Goals(_)
                | ReadValue::Spend(_)
                | ReadValue::OwnerSpend(_)
                | ReadValue::Project(_)
                | ReadValue::OwnerProject(_)
                | ReadValue::Notes(_)
                | ReadValue::Deployment(_) => {}
            }
        }
        engine::Change::Request(_)
        | engine::Change::Frame(_)
        | engine::Change::SignIn { .. }
        | engine::Change::SignedOut { .. }
        | engine::Change::Project(_)
        | engine::Change::Note(_)
        | engine::Change::DeleteNote { .. }
        | engine::Change::Deployment(_)
        | engine::Change::Tree(_)
        | engine::Change::Seen { .. } => {}
    }
}
