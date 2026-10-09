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
        let family::Ask::Touch { project, item, revision, parameters } = ask;
        let subject = family::Subject { scope: family::Scope::Project(*project), kind: family::Kind::Item(*item) };
        let answer = match self.documents.get_mut(&subject) {
            Some(family::Document::Item(doc)) if doc.revision == *revision => {
                doc.revision = Revision { value: doc.revision.value + 1 };
                if let Some(value) = parameters {
                    doc.value = *value;
                }
                family::Answer::Touched { item: *item, revision: doc.revision }
            }
            Some(doc) => family::Answer::Moved { item: *item, revision: doc.revision() },
            None => family::Answer::Gone,
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
            ApplicationValue::Document(doc) => doc.fits(bounds.rows),
            ApplicationValue::State(state) => state.documents.values().all(|doc| doc.fits(bounds.rows)),
            ApplicationValue::Ask(_) | ApplicationValue::Answer(_) | ApplicationValue::Subject(_) => true,
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
    routes: BTreeMap<Token, Route>,
    serial: u64,
    committed: usize,
    last_attempt: Option<Token>,
    last_watch: Option<Token>,
}
impl Scripted {
    /// Installs a checked story and its application documents.
    #[must_use]
    pub fn new(script: engine::Script, seed: u64, family: Family) -> Self {
        Self {
            ledger: Ledger::new("scripted wire operation"),
            engine: engine::Engine::with_application(script, seed, family).expect("script keeps contract"),
            parties: BTreeMap::new(),
            routes: BTreeMap::new(),
            serial: 0,
            committed: 0,
            last_attempt: None,
            last_watch: None,
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
impl Front<Test> for Scripted {
    fn bind(&mut self, client: ClientId, party: u64) {
        self.parties.insert(client, party);
    }
    fn wire(&mut self, client: ClientId, now: Time, wall: Wall, request: Request) {
        let party = self.parties[&client];
        let request = match request {
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
        };
        match request {
            Request::Send { attempt, key, ask } => {
                self.last_attempt = Some(attempt);
                match ask {
                    Ask::Jig(ask) => {
                        self.engine.send(now, wall, party, key, attempt, *ask).expect("jig send fits");
                    }
                    Ask::First(ask) => {
                        let family::Ask::Touch { project, .. } = &ask;
                        let digest: [u8; 32] = Sha256::digest(format!("{ask:?}").as_bytes()).into();
                        self.engine
                            .send_application(now, wall, party, key, attempt, 1, *project, digest, ask)
                            .expect("family send fits");
                    }
                }
            }
            Request::Jig(out) => match out {
                JigOut::Read { read, what } => {
                    self.engine.read(now, party, read, what).expect("read fits");
                }
                JigOut::Open { watch, subject } => {
                    self.last_watch = Some(watch);
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
    fn poll(&mut self, now: Time, _: Wall) -> Vec<(ClientId, Event)> {
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
                                WatchEvent::Delivered(_) => panic!("application watches carry application documents"),
                            },
                        }),
                    },
                    Up::ApplicationAnswer { attempt: _, envelope: e, answer } => Event::Answer {
                        attempt: local,
                        envelope: envelope(e),
                        answer: answer.map(|a| domain::Answered::First(a.expect("family refusal is typed by owner"))),
                    },
                    Up::ApplicationRead { token: _, terminal } => Event::First(first::Event::Read {
                        read: local,
                        result: match terminal {
                            Ok(d) => family::ReadResult::Value(d),
                            Err(ReadNone::Busy | ReadNone::Unreachable) => family::ReadResult::Busy,
                            Err(ReadNone::SignedOut) => family::ReadResult::SignedOut,
                            Err(ReadNone::Unknown | ReadNone::Refused | ReadNone::Gone) => family::ReadResult::Unknown,
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
                                WatchEnd::Dropped
                                | WatchEnd::Unknown
                                | WatchEnd::NotAllowed
                                | WatchEnd::Busy
                                | WatchEnd::SubjectEnded
                                | WatchEnd::Stopping => family::WatchEnd::Dropped,
                            },
                        }),
                    },
                };
                Some((client, event))
            })
            .collect()
    }
    fn next_due(&self) -> Option<Time> {
        self.engine.next_due()
    }
    fn observed(&mut self) -> Vec<Observed> {
        let mut seen = Vec::new();
        for commit in &self.engine.commits()[self.committed..] {
            seen.push(Observed::Durable { commit: commit.number, at: commit.durable.expect("durable ledger") });
            if let (Some(party), Some(key)) = (commit.party, commit.key) {
                seen.push(Observed::Made { party, key, commit: commit.number });
            }
        }
        self.committed = self.engine.commits().len();
        seen
    }
    fn fault(&mut self, now: Time, fault: FrontFault) {
        let fault = match fault {
            FrontFault::Busy => engine::Fault::Busy,
            FrontFault::NotReady => engine::Fault::NotReady,
            FrontFault::Restart { offline } => engine::Fault::Restart { offline },
            FrontFault::Slow { span } => engine::Fault::Slow { span },
            FrontFault::DropWatch => engine::Fault::DropWatch { watch: self.last_watch.expect("watch opened") },
            FrontFault::NoAnswer => engine::Fault::Lose { attempt: self.last_attempt.expect("attempt sent") },
            FrontFault::SignedOut => {
                engine::Fault::SignOut { sign_in: *self.parties.values().next().expect("client bound") }
            }
            FrontFault::Versions => engine::Fault::Versions(VersionRange { first: 2, last: 2 }),
        };
        self.engine.fault(now, fault);
    }
    fn reload(&mut self, client: ClientId, now: Time) {
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
        self.engine.act(now, wall, engine::Change::SignIn { sign_in: party, party }).expect("sign in fits");
    }
}
