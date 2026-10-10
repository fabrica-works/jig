//! Token routing between a client root and scripted engine. No root state is read.
#![allow(clippy::wildcard_imports, reason = "world adapters translate the complete owner vocabulary")]
use crate::{harness::*, test::Test};
use jig_client::contract::*;
use jig_scripted_engine::{self as engine, Application, ApplicationValue, Decision, Envelope, Truth, Up};
use jig_test_client_domain::{self as domain, Ask, Event, JigOut, Request, family, first};
use sha2::{Digest, Sha256};
use skein_lib::{RequestEnvelope, Time, Token, Wall};
use skein_world::domain::Ledger;
use std::collections::BTreeMap;
/// Scripted family documents with revision-naming decisions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Family {
    pub documents: BTreeMap<family::Subject, family::Document>,
}
impl Application for Family {
    type Ask = family::Ask;
    type Answer = family::Answer;
    type Subject = family::Subject;
    type Document = family::Document;
    fn decide(&mut self, _: &Truth, _: u64, ask: &Self::Ask, _: Wall) -> Decision<Self::Answer> {
        let answer = match ask {
            family::Ask::Touch { project, item, revision, parameters } => {
                let subject =
                    family::Subject { scope: family::Scope::Project(*project), kind: family::Kind::Item(*item) };
                match self.documents.get_mut(&subject) {
                    Some(family::Document::Item(doc)) if doc.revision == *revision => {
                        doc.revision = Revision { value: doc.revision.value + 1 };
                        if let Some(value) = parameters {
                            doc.value = *value;
                        }
                        family::Answer::Touched { item: *item, revision: doc.revision }
                    }
                    Some(doc) => match doc.revision() {
                        Some(revision) => family::Answer::Moved { item: *item, revision },
                        None => family::Answer::Connector(jig_test_client_domain::connector_values::ConnectorAnswer {
                            value: jig_test_client_domain::connector_values::ApplicationOutcome::Unknown,
                        }),
                    },
                    None => family::Answer::Gone,
                }
            }
            family::Ask::ChangeObject { project, object, revision, value } => {
                let subject =
                    family::Subject { scope: family::Scope::Project(*project), kind: family::Kind::Item(*object) };
                let outcome = match self.documents.get_mut(&subject) {
                    Some(family::Document::Connector(doc)) => match doc.value.as_mut() {
                        jig_test_client_domain::connector_values::ClientDocument::Object { object: number, now }
                            if number == object =>
                        {
                            if now.revision != revision.value {
                                jig_test_client_domain::connector_values::ApplicationOutcome::Moved {
                                    object: *object,
                                    now: *now,
                                }
                            } else if let Some(next) = now.revision.checked_add(1) {
                                now.revision = next;
                                now.value = *value;
                                jig_test_client_domain::connector_values::ApplicationOutcome::Changed {
                                    object: *object,
                                    now: *now,
                                }
                            } else {
                                jig_test_client_domain::connector_values::ApplicationOutcome::Exhausted
                            }
                        }
                        jig_test_client_domain::connector_values::ClientDocument::Object { .. }
                        | jig_test_client_domain::connector_values::ClientDocument::Panel { .. }
                        | jig_test_client_domain::connector_values::ClientDocument::Result { .. }
                        | jig_test_client_domain::connector_values::ClientDocument::Effect { .. }
                        | jig_test_client_domain::connector_values::ClientDocument::Outbox { .. }
                        | jig_test_client_domain::connector_values::ClientDocument::Hold { .. }
                        | jig_test_client_domain::connector_values::ClientDocument::Project { .. }
                        | jig_test_client_domain::connector_values::ClientDocument::Deployment { .. } => {
                            jig_test_client_domain::connector_values::ApplicationOutcome::Unknown
                        }
                    },
                    None | Some(family::Document::Item(_) | family::Document::Part(_)) => {
                        jig_test_client_domain::connector_values::ApplicationOutcome::Unknown
                    }
                };
                family::Answer::Connector(jig_test_client_domain::connector_values::ConnectorAnswer { value: outcome })
            }
        };
        Decision { answer, changes: vec![] }
    }
    fn document(&self, truth: &Truth, party: u64, subject: &Self::Subject) -> Option<Self::Document> {
        let allowed = match subject.scope {
            family::Scope::Project(project) => {
                truth.frames.get(&party).is_some_and(|frame| frame.projects.iter().any(|p| p.project == project))
            }
            family::Scope::Deployment => truth.frames.get(&party).is_some_and(|f| f.deployment_role.is_some()),
        };
        if allowed { self.documents.get(subject).cloned() } else { None }
    }
    fn marks(&self, _: &engine::Change) -> Vec<Self::Subject> {
        self.documents.keys().copied().collect()
    }
    fn fits(&self, value: ApplicationValue<'_, Self>, bounds: &engine::Bounds) -> bool {
        match value {
            ApplicationValue::Document(doc) => family_document_fits(doc, bounds),
            ApplicationValue::State(state) => state.documents.values().all(|doc| family_document_fits(doc, bounds)),
            ApplicationValue::Ask(_) | ApplicationValue::Answer(_) | ApplicationValue::Subject(_) => true,
        }
    }
}
fn family_document_fits(document: &family::Document, bounds: &engine::Bounds) -> bool {
    match document {
        family::Document::Item(_) | family::Document::Part(_) => document.fits(bounds.rows),
        family::Document::Connector(value) => {
            let limits = domain::connector_values::ConnectorLimits {
                rows: bounds.rows,
                paths: bounds.rows,
                segments: bounds.rows,
                segment_bytes: bounds.text,
                bytes: bounds.document_bytes,
            };
            value.matches_subject() && domain::connector_bounds::document_fits(value, &limits)
        }
    }
}
#[derive(Clone, Copy)]
enum Route {
    Jig(ClientId, Token),
    First(ClientId, Token),
}
/// The scripted engine with opaque client token routes.
pub struct Scripted {
    pub engine: engine::Engine<Family>,
    ledger: Ledger<Token, ()>,
    parties: BTreeMap<ClientId, u64>,
    hellos: std::collections::BTreeSet<ClientId>,
    routes: BTreeMap<Token, Route>,
    serial: u64,
    committed: usize,
    last_attempt: Option<Token>,
    last_watch: Option<Token>,
    last_object: Option<ObjectKey>,
    page_watch: Option<Token>,
    link: Vec<Observed>,
    recovery: Option<Time>,
    pending_sign_ins: Vec<u64>,
    pending_faults: Vec<FrontFault>,
}
impl Scripted {
    fn hello_events(&mut self) -> Vec<(ClientId, Event)> {
        std::mem::take(&mut self.hellos)
            .into_iter()
            .map(|client| (client, Event::Jig(jig_client::Event::Hello { versions: self.engine.versions() })))
            .collect()
    }
    /// Installs a checked story and its application documents.
    #[must_use]
    pub fn new(script: engine::Script, seed: u64, family: Family) -> Self {
        let mut initial = Vec::new();
        for source in &script.truth.sources {
            observe_change(&engine::Change::Source(source.clone()), 0, &mut initial);
        }
        Self {
            ledger: Ledger::new("scripted wire operation"),
            engine: engine::Engine::with_application(script, seed, family).expect("script keeps contract"),
            parties: BTreeMap::new(),
            hellos: std::collections::BTreeSet::new(),
            routes: BTreeMap::new(),
            serial: 0,
            committed: 0,
            last_attempt: None,
            last_watch: None,
            last_object: None,
            page_watch: None,
            link: initial,
            recovery: None,
            pending_sign_ins: Vec::new(),
            pending_faults: Vec::new(),
        }
    }
    fn transport(&mut self, client: ClientId, local: Token, first: bool, fresh: bool) -> Token {
        if !fresh
            && let Some((token, _)) = self.routes.iter().rev().find(|(_, r)| match r {
                Route::Jig(c, t) => !first && *c == client && *t == local,
                Route::First(c, t) => first && *c == client && *t == local,
            })
        {
            return *token;
        }
        self.serial += 1;
        let token = Token::new(self.serial);
        self.ledger.open(token, ());
        self.routes.insert(token, if first { Route::First(client, local) } else { Route::Jig(client, local) });
        token
    }
}
fn envelope(value: Envelope) -> RequestEnvelope {
    match value {
        Envelope::Final => RequestEnvelope::Final,
        Envelope::Again => RequestEnvelope::Again,
        Envelope::Lost => RequestEnvelope::Lost,
        Envelope::SignedOut => RequestEnvelope::SignedOut,
    }
}
fn scope(value: family::Scope) -> engine::ApplicationScope {
    match value {
        family::Scope::Project(p) => engine::ApplicationScope::Project(p),
        family::Scope::Deployment => engine::ApplicationScope::Deployment,
    }
}
impl Scripted {
    fn recover(&mut self, now: Time, wall: Wall) {
        if self.recovery.is_some_and(|at| at <= now) {
            self.recovery = None;
            self.link.push(Observed::Reachable { live: true });
            for party in std::mem::take(&mut self.pending_sign_ins) {
                self.engine
                    .act(now, wall, engine::Change::SignIn { sign_in: party, party })
                    .expect("recovered sign in fits");
            }
            for fault in std::mem::take(&mut self.pending_faults) {
                self.fault(now, fault);
            }
        }
    }
    fn routed(&mut self, client: ClientId, request: Request) -> Request {
        match request {
            Request::Send { attempt, key, ask } => {
                let first = matches!(&ask, Ask::First(_));
                Request::Send { attempt: self.transport(client, attempt, first, true), key, ask }
            }
            Request::Jig(JigOut::Read { read, what }) => {
                Request::Jig(JigOut::Read { read: self.transport(client, read, false, true), what })
            }
            Request::Jig(JigOut::Open { watch, subject }) => {
                Request::Jig(JigOut::Open { watch: self.transport(client, watch, false, true), subject })
            }
            Request::Jig(JigOut::Continue { watch, subject }) => {
                Request::Jig(JigOut::Continue { watch: self.transport(client, watch, false, false), subject })
            }
            Request::Jig(JigOut::Close { watch }) => {
                Request::Jig(JigOut::Close { watch: self.transport(client, watch, false, false) })
            }
            Request::First(first::Out::Read { read, subject }) => {
                Request::First(first::Out::Read { read: self.transport(client, read, true, true), subject })
            }
            Request::First(first::Out::Open { watch, subject }) => {
                Request::First(first::Out::Open { watch: self.transport(client, watch, true, true), subject })
            }
            Request::First(first::Out::Close { watch }) => {
                Request::First(first::Out::Close { watch: self.transport(client, watch, true, false) })
            }
            Request::Save { .. }
            | Request::Erase { .. }
            | Request::Show { .. }
            | Request::SignIn { .. }
            | Request::Fact { .. }
            | Request::Jig(JigOut::Answered { .. } | JigOut::Unknown { .. })
            | Request::First(first::Out::Fact { .. }) => request,
        }
    }
}

impl Front<Test> for Scripted {
    fn bind(&mut self, client: ClientId, party: u64) {
        self.parties.insert(client, party);
        self.hellos.insert(client);
    }
    fn wire(&mut self, client: ClientId, now: Time, wall: Wall, request: Request) {
        let party = self.parties[&client];
        let request = self.routed(client, request);
        match request {
            Request::Send { attempt, key, ask } => {
                self.last_attempt = Some(attempt);
                match ask {
                    Ask::Jig(ask) => {
                        self.engine.send(now, wall, party, key, attempt, *ask).expect("jig send fits");
                    }
                    Ask::First(ask) => {
                        let project = match &ask {
                            family::Ask::Touch { project, .. } | family::Ask::ChangeObject { project, .. } => *project,
                        };
                        let digest: [u8; 32] = Sha256::digest(format!("{ask:?}").as_bytes()).into();
                        self.engine
                            .send_application(now, wall, party, key, attempt, 1, project, digest, ask)
                            .expect("family send fits");
                    }
                }
            }
            Request::Jig(out) => match out {
                JigOut::Read { read, what } => {
                    self.last_object = match &what {
                        Read::Task { task } => Some(ObjectKey::Task(*task)),
                        Read::Proposal { proposal } => Some(ObjectKey::Proposal(*proposal)),
                        Read::Escalation { task } => Some(ObjectKey::Escalation(*task)),
                        Read::Question { task, message } => {
                            Some(ObjectKey::Question { task: *task, message: *message })
                        }
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
                    self.engine.read(now, party, read, what).expect("read fits");
                }
                JigOut::Open { watch, subject } => {
                    self.last_watch = Some(watch);
                    if let Subject::Task { task } = &subject {
                        self.last_object = Some(ObjectKey::Task(*task));
                        self.page_watch = Some(watch);
                    }
                    self.engine.open(now, party, watch, subject).expect("watch fits");
                }
                JigOut::Continue { watch, subject } => match subject {
                    Subject::Run { turn, offset, .. } => {
                        self.engine.continue_run(watch, turn, offset).expect("run watch exists");
                    }
                    Subject::Frame
                    | Subject::Task { .. }
                    | Subject::Tree { .. }
                    | Subject::Chips { .. }
                    | Subject::Inbox { .. }
                    | Subject::Goals { .. }
                    | Subject::Tasks { .. }
                    | Subject::Conversations
                    | Subject::Deployment => unreachable!("only runs continue"),
                },
                JigOut::Close { watch } => self.engine.close(now, watch),
                JigOut::Answered { .. } | JigOut::Unknown { .. } => {}
            },
            Request::First(out) => match out {
                first::Out::Read { read, subject } => {
                    self.engine
                        .read_scoped_application(now, party, read, 1, scope(subject.scope), &subject)
                        .expect("family read fits");
                }
                first::Out::Open { watch, subject } => {
                    self.last_watch = Some(watch);
                    self.engine
                        .open_scoped_application(now, party, watch, 1, scope(subject.scope), subject)
                        .expect("family watch fits");
                }
                first::Out::Close { watch } => self.engine.close(now, watch),
                first::Out::Fact { .. } => {}
            },
            Request::Save { .. }
            | Request::Erase { .. }
            | Request::Show { .. }
            | Request::SignIn { .. }
            | Request::Fact { .. } => unreachable!("shell classified a wire request"),
        }
    }
    fn poll(&mut self, now: Time, wall: Wall) -> Vec<(ClientId, Event)> {
        self.recover(now, wall);
        let mut events = self.hello_events();
        events.extend(
            self.engine
                .poll(now)
                .into_iter()
                .filter_map(|up| {
                    let token = match &up {
                        Up::Answer { attempt, .. } | Up::ApplicationAnswer { attempt, .. } => *attempt,
                        Up::Read { token, .. }
                        | Up::ApplicationRead { token, .. }
                        | Up::Watch { token, .. }
                        | Up::Ended { token, .. } => *token,
                        Up::App { watch, .. } => *watch,
                    };
                    let terminal = matches!(
                        &up,
                        Up::Answer { .. }
                            | Up::Read { .. }
                            | Up::ApplicationAnswer { .. }
                            | Up::ApplicationRead { .. }
                            | Up::Ended { .. }
                    );
                    if terminal {
                        self.ledger.end(token);
                    }
                    let route = self.routes.get(&token).copied()?;
                    if terminal {
                        self.routes.remove(&token);
                    }
                    let (client, local) = match route {
                        Route::Jig(c, t) | Route::First(c, t) => (c, t),
                    };
                    let event = match up {
                        Up::Answer { attempt: _, envelope: e, answer } => Event::Answer {
                            attempt: local,
                            envelope: envelope(e),
                            answer: answer.map(domain::Answered::Jig),
                        },
                        Up::Read { token: _, terminal } => {
                            Event::Jig(jig_client::Event::Read { read: local, result: terminal })
                        }
                        Up::Watch { token: _, event } => match route {
                            Route::Jig(_, _) => Event::Jig(jig_client::Event::Watch { watch: local, event }),
                            Route::First(_, _) => Event::First(first::Event::Watch {
                                watch: local,
                                event: match event {
                                    WatchEvent::Opened => family::WatchEvent::Opened,
                                    WatchEvent::Alive => family::WatchEvent::Alive,
                                    WatchEvent::Delivered(_) => {
                                        panic!("application watches carry application documents")
                                    }
                                },
                            }),
                        },
                        Up::ApplicationAnswer { attempt: _, envelope: e, answer } => Event::Answer {
                            attempt: local,
                            envelope: envelope(e),
                            answer: answer.map(|a| {
                                domain::Answered::First(a.expect("test family fixture requires a typed outcome"))
                            }),
                        },
                        Up::ApplicationRead { token: _, terminal } => Event::First(first::Event::Read {
                            read: local,
                            result: match terminal {
                                Ok(d) => family::ReadResult::Value(d),
                                Err(ReadNone::Busy | ReadNone::Unreachable) => family::ReadResult::Busy,
                                Err(ReadNone::SignedOut) => family::ReadResult::SignedOut,
                                Err(ReadNone::Unknown | ReadNone::Refused | ReadNone::Gone) => {
                                    family::ReadResult::Unknown
                                }
                            },
                        }),
                        Up::App { watch: _, document } => Event::First(first::Event::Watch {
                            watch: local,
                            event: family::WatchEvent::Delivered(document),
                        }),
                        Up::Ended { token: _, why } => match route {
                            Route::Jig(_, _) => Event::Jig(jig_client::Event::Ended { watch: local, end: why }),
                            Route::First(_, _) => Event::First(first::Event::Ended {
                                watch: local,
                                end: match why {
                                    WatchEnd::Closed => family::WatchEnd::Closed,
                                    WatchEnd::SignedOut => family::WatchEnd::SignedOut,
                                    WatchEnd::Dropped => family::WatchEnd::Dropped,
                                    WatchEnd::Unknown => family::WatchEnd::Unknown,
                                    WatchEnd::NotAllowed => family::WatchEnd::NotAllowed,
                                    WatchEnd::Busy => family::WatchEnd::Busy,
                                    WatchEnd::SubjectEnded => family::WatchEnd::SubjectEnded,
                                    WatchEnd::Stopping => family::WatchEnd::Stopping,
                                },
                            }),
                        },
                    };
                    Some((client, event))
                })
                .collect::<Vec<_>>(),
        );
        events
    }
    fn next_due(&self) -> Option<Time> {
        if self.hellos.is_empty() { self.recovery.or_else(|| self.engine.next_due()) } else { Some(Time::ZERO) }
    }
    fn observed(&mut self) -> Vec<Observed> {
        let mut seen = std::mem::take(&mut self.link);
        for commit in &self.engine.commits()[self.committed..] {
            seen.push(Observed::Durable { commit: commit.number, at: commit.durable.expect("durable ledger") });
            if let (Some(party), Some(key)) = (commit.party, commit.key) {
                seen.push(Observed::Made { party, key, commit: commit.number });
            }
            let mut facts = Vec::new();
            for change in &commit.changes {
                observe_change(change, commit.number, &mut facts);
            }
            // All changes in this durable commit are atomic. Ending discharges retention first.
            facts.sort_by_key(|fact| !matches!(fact, Observed::Ended { .. }));
            seen.extend(facts);
        }
        self.committed = self.engine.commits().len();
        seen
    }
    fn fault(&mut self, now: Time, fault: FrontFault) {
        if matches!(fault, FrontFault::ObjectGone) {
            if self.recovery.is_some_and(|at| now < at) {
                self.pending_faults.push(fault);
                return;
            }
            if let Some(key) = self.last_object {
                let wall = Wall::from_nanos(1_000_000_000_000 + now.as_nanos());
                self.engine.act(now, wall, engine::Change::Forget { key }).expect("forget fits");
                // Its authoritative listing stops naming the vanished card too.
                let sources = self.engine.truth().sources.clone();
                for mut source in sources {
                    if let engine::Rows::Inbox(rows) = &mut source.rows {
                        let before = rows.len();
                        rows.retain(|row| row.object != key);
                        if rows.len() != before {
                            self.engine.act(now, wall, engine::Change::Source(source)).expect("listing fits");
                        }
                    }
                }
            }
            return;
        }
        if let FrontFault::Restart { offline } = &fault {
            self.link.push(Observed::Reachable { live: false });
            self.recovery = Some(now.saturating_add(*offline));
        }
        let fault = match fault {
            FrontFault::Busy => engine::Fault::Busy,
            FrontFault::NotReady => engine::Fault::NotReady,
            FrontFault::Restart { offline } => engine::Fault::Restart { offline },
            FrontFault::Slow { span } => engine::Fault::Slow { span },
            FrontFault::DropWatch => {
                let Some(watch) = self.page_watch.or(self.last_watch) else {
                    return;
                };
                engine::Fault::DropWatch { watch }
            }
            FrontFault::NoAnswer => {
                let Some(attempt) = self.last_attempt else {
                    return;
                };
                engine::Fault::Lose { attempt }
            }
            FrontFault::SignedOut => {
                engine::Fault::SignOut { sign_in: *self.parties.values().next().expect("client bound") }
            }
            FrontFault::Versions => {
                self.hellos.extend(self.parties.keys().copied());
                engine::Fault::Versions(VersionRange { first: 2, last: 2 })
            }
            FrontFault::ObjectGone => unreachable!("object disappearance handled above"),
        };
        self.engine.fault(now, fault);
    }
    fn reload(&mut self, client: ClientId, now: Time) {
        self.hellos.insert(client);
        let tokens: Vec<_> = self
            .routes
            .iter()
            .filter_map(|(t, r)| match r {
                Route::Jig(c, _) | Route::First(c, _) => (*c == client).then_some(*t),
            })
            .collect();
        for token in tokens {
            self.engine.close(now, token);
            self.routes.remove(&token);
        }
    }
    fn received(&mut self, client: ClientId, event: &Event) {
        let selected = match event {
            Event::Jig(jig_client::Event::Watch { watch, event: WatchEvent::Delivered(_) }) => Some((*watch, false)),
            Event::First(first::Event::Watch { watch, event: family::WatchEvent::Delivered(_) }) => {
                Some((*watch, true))
            }
            Event::Start { .. }
            | Event::Went { .. }
            | Event::Act { .. }
            | Event::Answer { .. }
            | Event::Jig(_)
            | Event::First(_)
            | Event::SignedOut => None,
        };
        if let Some((local, first)) = selected {
            let token = self.transport(client, local, first, false);
            self.engine.taken(token);
        }
    }
    fn sign_in(&mut self, client: ClientId, now: Time, wall: Wall) {
        let party = self.parties[&client];
        if self.recovery.is_some_and(|at| now < at) {
            if !self.pending_sign_ins.contains(&party) {
                self.pending_sign_ins.push(party);
            }
            return;
        }
        self.engine.act(now, wall, engine::Change::SignIn { sign_in: party, party }).expect("sign in fits");
    }
}

fn observe_change(change: &engine::Change, commit: u64, seen: &mut Vec<Observed>) {
    match change {
        engine::Change::Source(source) => {
            if let (Read::Activity { task, .. }, engine::Rows::Activity(rows)) = (&source.read, &source.rows) {
                for row in rows {
                    if let Activity::Words { message, words, .. } = &row.event {
                        seen.push(Observed::Kept {
                            task: *task,
                            message: *message,
                            words: words.bytes().to_vec(),
                            commit,
                        });
                    }
                }
                seen.push(Observed::Retained {
                    task: *task,
                    messages: rows
                        .iter()
                        .filter_map(|row| match &row.event {
                            Activity::Words { message, .. } => Some(*message),
                            Activity::RunBegan { .. }
                            | Activity::RunEnded { .. }
                            | Activity::Turn { .. }
                            | Activity::Result { .. }
                            | Activity::News { .. }
                            | Activity::Decision { .. } => None,
                        })
                        .collect(),
                    commit,
                });
            }
        }
        engine::Change::Forget { key } => seen.push(Observed::Stopped { card: crate::observe::key(*key), commit }),
        engine::Change::Object { key, value, .. } => {
            let terminal = match value {
                ReadValue::Proposal(p) => p.ending.is_some(),
                ReadValue::Escalation(e) => e.ending.is_some(),
                ReadValue::Question(q) => q.ending.is_some(),
                ReadValue::PersonTask(p) => p.ending.is_some(),
                ReadValue::Task(t) => t.result.is_some(),
                ReadValue::Result(_) => true,
                ReadValue::Frame(_)
                | ReadValue::Ended(_)
                | ReadValue::History(_)
                | ReadValue::Activity(_)
                | ReadValue::Inbox(_)
                | ReadValue::OwnerInbox(_)
                | ReadValue::Conversations(_)
                | ReadValue::Goals(_)
                | ReadValue::Spend(_)
                | ReadValue::Project(_)
                | ReadValue::Notes(_)
                | ReadValue::Deployment(_) => false,
            };
            if terminal {
                seen.push(Observed::Stopped { card: crate::observe::key(*key), commit });
            }
            match value {
                ReadValue::Task(t) if t.result.is_some() => seen.push(Observed::Ended { task: t.chip.task, commit }),
                ReadValue::Result(r) => seen.push(Observed::Ended { task: r.task.task, commit }),
                ReadValue::Task(_)
                | ReadValue::Frame(_)
                | ReadValue::Proposal(_)
                | ReadValue::Escalation(_)
                | ReadValue::Question(_)
                | ReadValue::PersonTask(_)
                | ReadValue::Ended(_)
                | ReadValue::History(_)
                | ReadValue::Activity(_)
                | ReadValue::Inbox(_)
                | ReadValue::OwnerInbox(_)
                | ReadValue::Conversations(_)
                | ReadValue::Goals(_)
                | ReadValue::Spend(_)
                | ReadValue::Project(_)
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
