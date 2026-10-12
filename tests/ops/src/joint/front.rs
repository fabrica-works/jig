//! The real Ops engine and client meet at an ordinary protocol face (testing.md, 5).
use super::engine::OpsApplication;
use crate::conformance::{Config, Story};
use jig_client_world::{
    harness::{ClientId, Front, FrontFault, Observed},
    joint::{
        archive::{Archives, Started},
        packets::{self, Incoming},
        read_packets,
        transport::{KeyedOwner, Reply, Watched},
        wire,
    },
};
use jig_conformance::Harness;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_ops_domain as root;
use jig_ops_domain_client::{self as client, infrastructure as infra, observability as obs};
use skein_lib::{Duration, ReplyTo, Time, Token, Wall};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Subject {
    Infrastructure(infra::Subject),
    Observability(obs::Subject),
}
struct Watch {
    client: ClientId,
    native: Token,
    subject: Subject,
    flight: bool,
}
struct Read {
    client: ClientId,
    native: Token,
    subject: Subject,
}
pub struct Joint {
    pub engine: Harness<OpsApplication>,
    /// Whole Environment facts actually returned to a client, independent of
    /// the conformance peers' incident/procedure-only document observations.
    pub environment_documents: Vec<jig_ops_domain_infrastructure::ClientDocument>,
    pub lost_final_answers: u64,
    pub disconnects: u64,
    pub reconnects: u64,
    pub limits: client::Limits,
    codec: jig_protocol_transcript::Limits,
    story: Story,
    archives: Archives,
    watches: BTreeMap<Token, Watch>,
    reads: BTreeMap<Token, Read>,
    next: u64,
    application_rows: u32,
    callbacks: Vec<(ClientId, client::Event)>,
    parked: BTreeMap<ClientId, Vec<client::Request>>,
    now: Time,
    due: Option<Time>,
    work: bool,
    observed: Vec<Observed>,
    pending_observed: Vec<Observed>,
    offline_until: Option<Time>,
    delay_until: Option<Time>,
    lose_answers: u32,
    crashes_seen: usize,
}
impl Joint {
    #[must_use]
    pub fn new(seed: u64, limits: client::Limits, codec: jig_protocol_transcript::Limits) -> Self {
        Self::for_story(seed, limits, codec, Story::Alert)
    }
    #[must_use]
    pub fn for_story(seed: u64, limits: client::Limits, codec: jig_protocol_transcript::Limits, story: Story) -> Self {
        let configuration = Config::new(seed, story).client_backed().client_recovery();
        let application_rows = configuration.limits().core.documents.rows;
        let mut engine = Harness::with_trace(configuration, seed, false);
        engine.drain().expect("actual Ops cold start");
        Self {
            engine,
            environment_documents: Vec::new(),
            lost_final_answers: 0,
            disconnects: 0,
            reconnects: 0,
            limits,
            codec,
            story,
            archives: Archives::default(),
            watches: BTreeMap::new(),
            reads: BTreeMap::new(),
            next: 1 << 63,
            application_rows,
            callbacks: Vec::new(),
            parked: BTreeMap::new(),
            now: Time::ZERO,
            due: None,
            work: true,
            observed: Vec::new(),
            pending_observed: Vec::new(),
            offline_until: None,
            delay_until: None,
            lose_answers: 0,
            crashes_seen: 0,
        }
    }
    fn token(&mut self) -> Token {
        let value = self.next;
        self.next = self.next.checked_add(1).expect("finite Ops route namespace");
        Token::new(value)
    }
    fn send_event(&mut self, event: root::Event) {
        self.work = true;
        self.engine.send(event);
    }
    pub fn source(&mut self, event: root::Event) {
        if let root::Event::Observability(jig_ops_domain_observability::Event::System(
            jig_ops_domain_observability::SystemEvent::Alert { own, alert },
        )) = &event
        {
            self.engine.peers.alert_news.push((*own, alert.clone()));
        }
        self.send_event(event);
    }
    fn advance(&mut self, now: Time) {
        if now.as_nanos() > self.engine.clock().now {
            self.engine.advance(now.as_nanos() - self.engine.clock().now);
            self.work = true;
        }
        self.now = now;
    }
    fn incoming(&mut self, packet: Incoming) {
        match packet {
            Incoming::Callback { client, event } => {
                let event = match event {
                    jig_test_domain_client::Event::Jig(value) => client::Event::Jig(value),
                    jig_test_domain_client::Event::Answer { attempt, envelope, answer } => client::Event::Answer {
                        attempt,
                        envelope,
                        answer: answer.map(|answer| match answer {
                            jig_test_domain_client::Answered::Jig(value) => client::Answered::Jig(value),
                            jig_test_domain_client::Answered::First(_) => {
                                panic!("Ops never registers first-owner replies")
                            }
                        }),
                    },
                    jig_test_domain_client::Event::SignedOut => client::Event::SignedOut,
                    jig_test_domain_client::Event::First(_)
                    | jig_test_domain_client::Event::Start { .. }
                    | jig_test_domain_client::Event::Went { .. }
                    | jig_test_domain_client::Event::Act { .. } => panic!("45 protocol emits only its jig owner"),
                };
                self.callbacks.push((client, event));
            }
            Incoming::Authenticated { client, .. } => {
                for value in self.parked.remove(&client).unwrap_or_default() {
                    self.request(client, value);
                }
            }
            Incoming::SignInAgain { .. } | Incoming::SignInFailed { .. } => {
                panic!("unexpected genuine provider refusal")
            }
        }
    }
    fn jig_packet(&mut self, watcher: Token, packet: Incoming) {
        if self.engine.peers.transport.is_closing(watcher) {
            for event in self.engine.peers.transport.closed_flights(watcher) {
                self.send_event(root::Event::Core(event));
            }
        } else {
            self.incoming(packet);
        }
    }
    fn request(&mut self, client: ClientId, value: client::Request) {
        if self.offline_until.is_some() {
            self.parked.entry(client).or_default().push(value);
            return;
        }
        let Some(sign_in) = self.engine.peers.transport.sign_in(client) else {
            self.parked.entry(client).or_default().push(value);
            return;
        };
        match value {
            client::Request::Send { attempt, key, ask } => match ask {
                client::Ask::Jig(value) => {
                    let ask = wire::asks::ask(&value).expect("whole jig ask uses 45 wire");
                    let reply_to = self.engine.peers.transport.reply_to(Reply::Keyed {
                        client,
                        attempt,
                        key,
                        owner: KeyedOwner::Jig,
                    });
                    self.send_event(root::Event::Core(core::Event::People(people::Event::Ask {
                        reply_to,
                        sign_in,
                        key,
                        ask,
                    })));
                }
                client::Ask::Infrastructure(ask) => {
                    let (project, value, digest) = super::infrastructure::ask(ask).expect("whole native Ops ask");
                    let reply_to = self.engine.peers.transport.reply_to(Reply::Keyed {
                        client,
                        attempt,
                        key,
                        owner: KeyedOwner::Application { connector: 2 },
                    });
                    self.send_event(root::Event::Application { reply_to, sign_in, key, project, digest, value });
                }
            },
            client::Request::Jig(value) => match value {
                client::JigOut::Read { read, what } => {
                    let owner = wire::reads::read(&what, self.limits.jig.text).expect("whole jig read uses 45 wire");
                    let reply = self.engine.peers.transport.reply_to(Reply::Read { client, read, what });
                    self.send_event(root::Event::Core(core::Event::Read { reply, sign_in, read: owner }));
                }
                client::JigOut::Open { watch, subject } => {
                    let watcher = self.engine.peers.transport.open(client, watch, Watched::Jig(subject.clone()));
                    let event = wire::watches::watch(
                        watcher,
                        sign_in,
                        self.engine.peers.transport.party(client),
                        &subject,
                        self.limits.jig.rows,
                    )
                    .expect("whole jig watch uses 45 wire");
                    self.send_event(root::Event::Core(event));
                }
                client::JigOut::Continue { watch, subject } => {
                    if let Some(event) =
                        self.engine.peers.transport.continued(client, watch, subject).expect("original continuation")
                    {
                        self.send_event(root::Event::Core(event));
                    }
                }
                client::JigOut::Close { watch } => {
                    if let Some(watcher) = self.engine.peers.transport.watch_token(client, watch) {
                        let terminals = self.engine.peers.transport.close_watch(watcher);
                        self.send_event(root::Event::Core(core::Event::Unwatch { watcher }));
                        for event in terminals {
                            self.send_event(root::Event::Core(event));
                        }
                    }
                }
                client::JigOut::Answered { .. } | client::JigOut::Unknown { .. } => {
                    panic!("shell duty never goes on wire")
                }
            },
            client::Request::Infrastructure(value) => match value {
                infra::Out::Read { read, subject } => {
                    self.application_read(client, read, Subject::Infrastructure(subject), sign_in);
                }
                infra::Out::Open { watch, subject } => {
                    self.application_open(client, watch, Subject::Infrastructure(subject), sign_in);
                }
                infra::Out::Close { watch } => self.application_close(client, watch, 2),
                infra::Out::Fact { .. } => panic!("shell duty never goes on wire"),
            },
            client::Request::Observability(value) => match value {
                obs::Out::Read { read, subject } => {
                    self.application_read(client, read, Subject::Observability(subject), sign_in);
                }
                obs::Out::Open { watch, subject } => {
                    self.application_open(client, watch, Subject::Observability(subject), sign_in);
                }
                obs::Out::Close { watch } => self.application_close(client, watch, 1),
                obs::Out::Fact { .. } => panic!("shell duty never goes on wire"),
            },
            client::Request::Save { .. }
            | client::Request::Erase { .. }
            | client::Request::Show { .. }
            | client::Request::SignIn { .. }
            | client::Request::Fact { .. } => panic!("shell duty never goes on wire"),
        }
    }
    fn application_read(&mut self, client: ClientId, native: Token, subject: Subject, sign_in: u64) {
        let (scope, owner) = subject_route(subject.clone()).expect("actual owner subject");
        let token = self.token();
        self.reads.insert(token, Read { client, native, subject });
        self.send_event(root::Event::ReadApplication { reply: ReplyTo::new(token), sign_in, scope, subject: owner });
    }
    fn application_open(&mut self, client: ClientId, native: Token, subject: Subject, sign_in: u64) {
        let (scope, owner) = subject_route(subject.clone()).expect("actual owner subject");
        let watcher = self.token();
        self.watches.insert(watcher, Watch { client, native, subject, flight: false });
        self.send_event(root::Event::WatchApplication {
            watcher,
            sign_in,
            scope,
            subject: owner,
            window: core::documents::Window { rows: self.application_rows, after: None },
        });
    }
    fn application_close(&mut self, client: ClientId, native: Token, connector: u16) {
        let watcher = self.watches.iter().find_map(|(watcher, route)| {
            if route.client == client && route.native == native && route.subject.connector() == connector {
                Some(*watcher)
            } else {
                None
            }
        });
        if let Some(watcher) = watcher {
            self.send_event(root::Event::Core(core::Event::Unwatch { watcher }));
        }
    }
}
impl Subject {
    fn connector(&self) -> u16 {
        match self {
            Self::Infrastructure(_) => 2,
            Self::Observability(_) => 1,
        }
    }
}
fn subject_route(subject: Subject) -> Result<(core::documents::ApplicationScope, root::ClientSubject), &'static str> {
    match subject {
        Subject::Infrastructure(subject) => {
            use jig_ops_domain_infrastructure::ClientSubject as S;
            let project = match subject.scope {
                infra::Scope::Project(project) => Some(project),
                infra::Scope::Deployment => None,
            };
            let owner = match subject.kind {
                infra::Kind::NativeEnvironment(environment) => S::Environment {
                    project: project.ok_or("environment scope")?,
                    environment: jig_ops_domain_infrastructure::Environment {
                        pool: environment.pool,
                        name: environment.name,
                    },
                },
                infra::Kind::NativeEnvironments { .. } => S::Project { project: project.ok_or("environments scope")? },
                infra::Kind::Panel(task) => S::Panel { project: project.ok_or("panel scope")?, task },
                infra::Kind::Result(task) => S::Result { project: project.ok_or("result scope")?, task },
                infra::Kind::OwnerEffect(proposal) => S::Effect { project: project.ok_or("effect scope")?, proposal },
                infra::Kind::Outbox(entry) => S::Outbox { project: project.ok_or("outbox scope")?, entry },
                infra::Kind::Hold(task) => S::Hold { project: project.ok_or("hold scope")?, task },
                infra::Kind::Project(named) => {
                    if project != Some(named) {
                        return Err("project identity");
                    }
                    S::Project { project: named }
                }
                infra::Kind::Deployment => {
                    if project.is_some() {
                        return Err("deployment scope");
                    }
                    S::Deployment
                }
                infra::Kind::Effect(_) => return Err("maker effect has no native raw identity"),
                infra::Kind::Environments { .. } | infra::Kind::Environment(_) => {
                    return Err("legacy numeric identity is absent from the native owner");
                }
            };
            let scope = match subject.scope {
                infra::Scope::Project(project) => core::documents::ApplicationScope::Project(project),
                infra::Scope::Deployment => core::documents::ApplicationScope::Deployment,
            };
            Ok((scope, root::ClientSubject::Infrastructure(owner)))
        }
        Subject::Observability(subject) => {
            use jig_ops_domain_observability::ClientSubject as S;
            let project = match subject.scope {
                obs::Scope::Project(project) => Some(project),
                obs::Scope::Deployment => None,
            };
            let owner = match subject.kind {
                obs::Kind::Incidents { .. } => S::Project { project: project.ok_or("incidents scope")? },
                obs::Kind::Incident(incident) => S::Incident { project: project.ok_or("incident scope")?, incident },
                obs::Kind::Outbox(entry) => S::Outbox { project: project.ok_or("outbox scope")?, entry },
                obs::Kind::Project(named) => {
                    if project != Some(named) {
                        return Err("project identity");
                    }
                    S::Project { project: named }
                }
                obs::Kind::Deployment => {
                    if project.is_some() {
                        return Err("deployment scope");
                    }
                    S::Deployment
                }
                obs::Kind::Panel(_) | obs::Kind::Result(_) | obs::Kind::Effect(_) | obs::Kind::Hold(_) => {
                    return Err("subject absent from actual Obs source");
                }
            };
            let scope = match subject.scope {
                obs::Scope::Project(project) => core::documents::ApplicationScope::Project(project),
                obs::Scope::Deployment => core::documents::ApplicationScope::Deployment,
            };
            Ok((scope, root::ClientSubject::Observability(owner)))
        }
    }
}
impl Joint {
    fn release_callbacks(&mut self) -> Vec<(ClientId, client::Event)> {
        let mut callbacks = Vec::new();
        for (client, event) in std::mem::take(&mut self.callbacks) {
            if matches!(event, client::Event::Answer { envelope: skein_lib::RequestEnvelope::Final, .. })
                && self.lose_answers > 0
            {
                self.lose_answers -= 1;
                self.lost_final_answers += 1;
            } else {
                callbacks.push((client, event));
            }
        }
        callbacks
    }
    fn drop_watches(&mut self) {
        for watcher in self.engine.peers.transport.watches() {
            let packet = packets::dropped(&mut self.engine.peers.transport, watcher).expect("original dropped route");
            self.incoming(packet);
            self.send_event(root::Event::Core(core::Event::Unwatch { watcher }));
        }
        for (watcher, watch) in std::mem::take(&mut self.watches) {
            let event = match watch.subject {
                Subject::Infrastructure(_) => client::Event::Infrastructure(infra::Event::Ended {
                    watch: watch.native,
                    end: infra::WatchEnd::Dropped,
                }),
                Subject::Observability(_) => {
                    client::Event::Observability(obs::Event::Ended { watch: watch.native, end: obs::WatchEnd::Dropped })
                }
            };
            self.callbacks.push((watch.client, event));
            self.send_event(root::Event::Core(core::Event::Unwatch { watcher }));
        }
    }
    fn detach(&mut self, client: ClientId) {
        self.archives.detach(client);
        self.parked.remove(&client);
        self.callbacks.retain(|(owner, _)| *owner != client);
        self.reads.retain(|_, read| read.client != client);
        for watcher in self.engine.peers.transport.detach(client) {
            self.send_event(root::Event::Core(core::Event::Unwatch { watcher }));
        }
        let watches: Vec<_> =
            self.watches.iter().filter_map(|(token, watch)| (watch.client == client).then_some(*token)).collect();
        for watcher in watches {
            self.watches.remove(&watcher).expect("original application watch");
            self.send_event(root::Event::Core(core::Event::Unwatch { watcher }));
        }
    }
    fn disconnected(&mut self, now: Time, offline: Duration) {
        self.disconnects += 1;
        for (client, event) in std::mem::take(&mut self.callbacks) {
            let event = match event {
                client::Event::Infrastructure(infra::Event::Read { read, .. }) => {
                    Some(client::Event::Infrastructure(infra::Event::Read { read, result: infra::ReadResult::Busy }))
                }
                client::Event::Observability(obs::Event::Read { read, .. }) => {
                    Some(client::Event::Observability(obs::Event::Read { read, result: obs::ReadResult::Busy }))
                }
                client::Event::Start { .. }
                | client::Event::Went { .. }
                | client::Event::Act { .. }
                | client::Event::Answer { .. }
                | client::Event::Jig(_)
                | client::Event::Infrastructure(_)
                | client::Event::Observability(_)
                | client::Event::SignedOut => None,
            };
            if let Some(event) = event {
                self.callbacks.push((client, event));
            }
        }
        for (_, read) in std::mem::take(&mut self.reads) {
            let event = match read.subject {
                Subject::Infrastructure(_) => client::Event::Infrastructure(infra::Event::Read {
                    read: read.native,
                    result: infra::ReadResult::Busy,
                }),
                Subject::Observability(_) => {
                    client::Event::Observability(obs::Event::Read { read: read.native, result: obs::ReadResult::Busy })
                }
            };
            self.callbacks.push((read.client, event));
        }
        self.delay_until = None;
        self.due = None;
        self.lose_answers = 0;
        self.drop_watches();
        for client in self.engine.peers.transport.clients() {
            self.archives.detach(client);
            self.engine.peers.transport.detach(client);
        }
        // Cold restoration can advance the engine clock beyond this front's
        // poll. Publish its durable facts only once the client clock catches up.
        self.pending_observed.append(&mut self.engine.peers.observations);
        self.observed.push(Observed::Reachable { live: false });
        self.offline_until = Some(now.saturating_add(offline));
        self.work = true;
    }
    fn observe_environment(&mut self, document: &root::ClientDocument) {
        if let root::ClientDocument::Infrastructure(document) = document
            && matches!(document.as_ref(), jig_ops_domain_infrastructure::ClientDocument::Environment { .. })
        {
            self.environment_documents.push(document.as_ref().clone());
        }
    }
    fn read_packet(&mut self, reply: Token, answer: &core::reads::Answer) {
        let Some(route) = self.engine.peers.transport.reply(reply) else {
            return;
        };
        if matches!(route, Reply::Archive { .. }) {
            if let Some(packet) = self
                .archives
                .answer(&mut self.engine.peers.transport, reply, answer, &self.codec, &self.limits.jig)
                .expect("original whole turn decoding")
            {
                self.incoming(packet);
            }
        } else if matches!(route, Reply::Read { what: jig_domain_client::contract::Read::Activity { .. }, .. })
            && let core::reads::Answer::Page(page) = answer
        {
            match self
                .archives
                .start(&mut self.engine.peers.transport, reply, page, &self.limits.jig)
                .expect("original whole activity page")
            {
                Started::Complete(packet) => self.incoming(packet),
                Started::Fetch(events) => {
                    for event in events {
                        self.send_event(root::Event::Core(event));
                    }
                }
            }
        } else {
            let packet = read_packets::read(&mut self.engine.peers.transport, reply, answer, &self.limits.jig)
                .expect("whole jig read uses 45 wire");
            self.incoming(packet);
        }
    }
    #[expect(clippy::too_many_lines, reason = "held Ops packets keep their original exhaustive client routes")]
    fn packet(&mut self, packet: root::Output) {
        match packet {
            root::Output::Core(core::Held::PeopleReply { to, sign_in, reply }) => {
                let to = to.into_token();
                if self.engine.peers.transport.reply(to).is_some() {
                    if let Reply::Keyed { client, attempt, owner: KeyedOwner::Application { connector: 2 }, .. } =
                        self.engine.peers.transport.reply(to).expect("original reply").clone()
                    {
                        let (envelope, body) =
                            wire::answers::keyed(wire::answers::reply(reply)).expect("original keyed disposition");
                        let answer = body.map(|answer| {
                            jig_scripted_engine::admit_answer(
                                &answer,
                                &jig_scripted_engine::Bounds {
                                    rows: self.limits.jig.rows,
                                    text: self.limits.jig.text,
                                    document_bytes: self.limits.jig.document_bytes,
                                    ..jig_scripted_engine::Bounds::default()
                                },
                            )
                            .expect("whole entrance refusal admission");
                            let jig_domain_client::contract::Answer::Refused(why) = answer else {
                                panic!("application entrance terminal")
                            };
                            client::Answered::Infrastructure(infra::Answer::EntranceRefused(
                                jig_domain_client::contract::EntranceRefusal::new(why).expect("whole entrance refusal"),
                            ))
                        });
                        if envelope == skein_lib::RequestEnvelope::SignedOut {
                            self.engine.peers.transport.signed_out(client);
                        }
                        self.engine.peers.transport.take_reply(to).expect("consume original reply");
                        self.callbacks.push((client, client::Event::Answer { attempt, envelope, answer }));
                    } else {
                        let packet = packets::people_reply(
                            &mut self.engine.peers.transport,
                            to,
                            sign_in,
                            reply,
                            &self.limits.jig,
                        )
                        .expect("whole people reply uses 45 wire");
                        self.incoming(packet);
                    }
                }
            }
            root::Output::Core(core::Held::Delivery { watcher, document }) => {
                if self.engine.peers.transport.watch(watcher).is_some() {
                    let packet =
                        packets::document(&mut self.engine.peers.transport, watcher, &document, &self.limits.jig)
                            .unwrap_or_else(|error| {
                                panic!("whole jig document uses 45 wire: {error:?}; original {document:?}")
                            });
                    self.jig_packet(watcher, packet);
                }
            }
            root::Output::Core(core::Held::Read { reply, answer }) => self.read_packet(reply.into_token(), &answer),
            root::Output::Core(core::Held::WatchEnded { watcher, why }) => {
                if self.engine.peers.transport.watch(watcher).is_some() {
                    let packet = packets::ended(&mut self.engine.peers.transport, watcher, why)
                        .expect("original watch terminal");
                    self.incoming(packet);
                }
            }
            root::Output::Core(core::Held::ApplicationWatchEnded { watcher, why, connector, .. }) => {
                self.application_end(watcher, connector, why);
            }
            root::Output::ApplicationDocument { watcher, document } => {
                self.observe_environment(&document);
                if let Some(route) = self.watches.get_mut(&watcher) {
                    assert!(!route.flight, "one actual application document in flight");
                    let callback = application_document(route.subject.clone(), document, &self.limits);
                    route.flight = true;
                    let event = match callback {
                        ApplicationDocument::Infrastructure(document) => {
                            client::Event::Infrastructure(infra::Event::Watch {
                                watch: route.native,
                                event: infra::WatchEvent::Delivered(document),
                            })
                        }
                        ApplicationDocument::Observability(document) => {
                            client::Event::Observability(obs::Event::Watch {
                                watch: route.native,
                                event: obs::WatchEvent::Delivered(document),
                            })
                        }
                    };
                    self.callbacks.push((route.client, event));
                }
            }
            root::Output::ApplicationRead { reply, answer } => {
                if let Ok(document) = &answer {
                    self.observe_environment(document);
                }
                if let Some(route) = self.reads.remove(&reply.into_token()) {
                    let event = match route.subject {
                        Subject::Infrastructure(_) => {
                            let result = match answer {
                                Ok(document) => {
                                    let ApplicationDocument::Infrastructure(document) =
                                        application_document(route.subject.clone(), document, &self.limits)
                                    else {
                                        unreachable!("original infrastructure read owner")
                                    };
                                    infra::ReadResult::Value(document)
                                }
                                Err(root::ClientMissing::NotAllowed) => infra::ReadResult::NotAllowed,
                                Err(root::ClientMissing::Unknown) => infra::ReadResult::Unknown,
                            };
                            client::Event::Infrastructure(infra::Event::Read { read: route.native, result })
                        }
                        Subject::Observability(_) => {
                            let result = match answer {
                                Ok(document) => {
                                    let ApplicationDocument::Observability(document) =
                                        application_document(route.subject.clone(), document, &self.limits)
                                    else {
                                        unreachable!("original observability read owner")
                                    };
                                    obs::ReadResult::Value(document)
                                }
                                Err(root::ClientMissing::NotAllowed) => obs::ReadResult::NotAllowed,
                                Err(root::ClientMissing::Unknown) => obs::ReadResult::Unknown,
                            };
                            client::Event::Observability(obs::Event::Read { read: route.native, result })
                        }
                    };
                    self.callbacks.push((route.client, event));
                }
            }
            root::Output::Now(core::Now::RunText { watcher, turn, from, to, text, cut, call }) => {
                if self.engine.peers.transport.watch(watcher).is_some() {
                    let writing = jig_domain_views::Writing { turn, from, to, text, cut, call };
                    let packet = packets::stream(&mut self.engine.peers.transport, watcher, &writing, &self.limits.jig)
                        .expect("whole run stream uses 45 wire");
                    self.jig_packet(watcher, packet);
                }
            }
            root::Output::Now(core::Now::WatchRefused { watcher, why }) => {
                if self.engine.peers.transport.watch(watcher).is_some() {
                    let packet = packets::refused(&mut self.engine.peers.transport, watcher, why)
                        .expect("original refused watch");
                    self.incoming(packet);
                } else if self.watches.contains_key(&watcher) {
                    self.application_refused(watcher, why);
                }
            }
            root::Output::Now(core::Now::SignInRefused { .. }) => panic!("actual provider capacity exhausted"),
            root::Output::Application { to, outcome } => {
                let to = to.into_token();
                let Some(route) = self.engine.peers.transport.take_reply(to) else {
                    // A real reload retired this reply's original client epoch.
                    return;
                };
                let Reply::Keyed { client, attempt, owner: KeyedOwner::Application { connector: 2 }, .. } = route
                else {
                    panic!("wrong native application owner");
                };
                let answer = super::infrastructure::answer(outcome);
                self.callbacks.push((
                    client,
                    client::Event::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        answer: Some(client::Answered::Infrastructure(answer)),
                    },
                ));
            }
            root::Output::Core(
                core::Held::ApplicationDelivery { .. }
                | core::Held::ReadLoad { .. }
                | core::Held::Applied { .. }
                | core::Held::MakeEffect { .. }
                | core::Held::SettledCall { .. }
                | core::Held::Assign { .. }
                | core::Held::Relayed { .. }
                | core::Held::Inbound { .. }
                | core::Held::Relay { .. }
                | core::Held::CallAnswer { .. }
                | core::Held::Result { .. }
                | core::Held::Acknowledge { .. }
                | core::Held::TaskTerminalAcknowledged { .. }
                | core::Held::TaskTurnKept { .. }
                | core::Held::AcknowledgeTurn { .. }
                | core::Held::Cancel { .. }
                | core::Held::Refuse { .. }
                | core::Held::TurnBusy { .. }
                | core::Held::StopRun { .. }
                | core::Held::NotesLoad { .. }
                | core::Held::NotesWritten { .. }
                | core::Held::NotesDeleted { .. },
            )
            | root::Output::Now(
                core::Now::Relayed { .. }
                | core::Now::EffectAnswer { .. }
                | core::Now::SettledCallRefused { .. }
                | core::Now::Call { .. }
                | core::Now::DropCall { .. }
                | core::Now::Undelivered { .. }
                | core::Now::Account(_)
                | core::Now::NotesIndexed { .. }
                | core::Now::NotesRecalled { .. }
                | core::Now::NotesRefused { .. }
                | core::Now::NoteBusy { .. }
                | core::Now::DropPayload { .. }
                | core::Now::DropAssignment { .. }
                | core::Now::TurnPayload { .. }
                | core::Now::AnswerPayload { .. }
                | core::Now::AcceptedTurn { .. }
                | core::Now::RefusedPayload { .. }
                | core::Now::Activate { .. }
                | core::Now::PrepareAgent { .. }
                | core::Now::StartPreparation { .. }
                | core::Now::SeenChat { .. }
                | core::Now::HistoricalProposal { .. }
                | core::Now::HistoricalPersonTask { .. }
                | core::Now::CompleteBrief { .. }
                | core::Now::BriefCorePlanned { .. }
                | core::Now::WorkspaceRequest { .. }
                | core::Now::RunPreparationFailed { .. }
                | core::Now::RunPrepared { .. }
                | core::Now::HistoricalEscalation { .. }
                | core::Now::ProcedureDelegateOutcome { .. }
                | core::Now::RestoreRefused,
            )
            | root::Output::IncidentAssociationRefused { .. }
            | root::Output::Commit { .. }
            | root::Output::Load { .. }
            | root::Output::CoreLoad(_)
            | root::Output::Observability(_)
            | root::Output::Infrastructure(_)
            | root::Output::Read { .. }
            | root::Output::Agent(_)
            | root::Output::ToChild(_)
            | root::Output::Busy
            | root::Output::Stop => panic!("non-client output escaped Ops peers"),
        }
    }
    fn application_end(&mut self, watcher: Token, connector: u16, why: jig_domain_views::End) {
        let Some(route) = self.watches.remove(&watcher) else {
            return;
        };
        assert_eq!(route.subject.connector(), connector, "actual owner terminal");
        let event = match route.subject {
            Subject::Infrastructure(_) => client::Event::Infrastructure(infra::Event::Ended {
                watch: route.native,
                end: match why {
                    jig_domain_views::End::Unwatched => infra::WatchEnd::Closed,
                    jig_domain_views::End::Gone => infra::WatchEnd::SubjectEnded,
                    jig_domain_views::End::NotAllowed => infra::WatchEnd::NotAllowed,
                    jig_domain_views::End::SignedOut => infra::WatchEnd::SignedOut,
                    jig_domain_views::End::Stopping => infra::WatchEnd::Stopping,
                },
            }),
            Subject::Observability(_) => client::Event::Observability(obs::Event::Ended {
                watch: route.native,
                end: match why {
                    jig_domain_views::End::Unwatched => obs::WatchEnd::Closed,
                    jig_domain_views::End::Gone => obs::WatchEnd::SubjectEnded,
                    jig_domain_views::End::NotAllowed => obs::WatchEnd::NotAllowed,
                    jig_domain_views::End::SignedOut => obs::WatchEnd::SignedOut,
                    jig_domain_views::End::Stopping => obs::WatchEnd::Stopping,
                },
            }),
        };
        self.callbacks.push((route.client, event));
    }
}
enum ApplicationDocument {
    Infrastructure(Box<infra::Document>),
    Observability(Box<obs::Document>),
}
fn application_document(
    subject: Subject,
    document: root::ClientDocument,
    limits: &client::Limits,
) -> ApplicationDocument {
    match subject {
        Subject::Infrastructure(subject) => {
            let root::ClientDocument::Infrastructure(document) = document else { panic!("wrong connector document") };
            let document = infra::Document::Native(Box::new(infra::native::Document {
                subject,
                value: super::infrastructure::translate_value(*document),
            }));
            assert!(document.fits(&limits.infrastructure), "whole infrastructure admission");
            ApplicationDocument::Infrastructure(Box::new(document))
        }
        Subject::Observability(subject) => {
            let root::ClientDocument::Observability(document) = document else { panic!("wrong connector document") };
            let document = obs::Document::Native(Box::new(obs::native::Document {
                subject,
                value: super::observability::document(*document),
            }));
            assert!(document.fits(&limits.observability), "whole observability admission");
            ApplicationDocument::Observability(Box::new(document))
        }
    }
}
impl Front<super::fixture::Client> for Joint {
    fn bind(&mut self, client: ClientId, party: u64) {
        assert_eq!(party, 1, "the independently configured story account");
        let (subject, login): (u64, &[u8]) = match self.story {
            Story::Alert | Story::Night => (7, b"oncall"),
            Story::Environment => (8, b"developer"),
        };
        self.engine.peers.transport.bind(
            client,
            party,
            people::Identity {
                key: people::IdentityKey { provider: 0, subject: subject.to_be_bytes().into() },
                login: login.into(),
                name: Box::new([]),
            },
        );
        self.callbacks.push((client, client::Event::Jig(self.engine.peers.transport.hello())));
        let event = self.engine.peers.transport.start_sign_in(client);
        self.send_event(root::Event::Core(event));
    }
    fn wire(&mut self, client: ClientId, now: Time, _wall: Wall, request: client::Request) {
        self.advance(now);
        self.request(client, request);
    }
    fn poll(&mut self, now: Time, _wall: Wall) -> Vec<(ClientId, client::Event)> {
        self.advance(now);
        if let Some(until) = self.offline_until {
            if now < until {
                return std::mem::take(&mut self.callbacks);
            }
            self.offline_until = None;
            self.reconnects += 1;
            self.observed.append(&mut self.pending_observed);
            self.observed.push(Observed::Reachable { live: true });
            for client in self.engine.peers.transport.clients() {
                self.callbacks.push((client, client::Event::Jig(self.engine.peers.transport.hello())));
            }
            for (client, requests) in std::mem::take(&mut self.parked) {
                for request in requests {
                    self.request(client, request);
                }
            }
        }
        if self.due.is_some_and(|due| now < due) {
            return Vec::new();
        }
        if self.due.take().is_some() {
            self.delay_until = None;
            self.observed.append(&mut self.pending_observed);
            return self.release_callbacks();
        }
        if self.work {
            self.work = false;
            for _ in 0..128 {
                self.engine.drain().expect("actual Ops decisions satisfy conformance referee");
                self.engine.peers.protocol.check().expect("every consumed normal packet reaches its original ACK");
                let crashes = self.engine.outcome().crashes.len();
                if crashes != self.crashes_seen {
                    self.crashes_seen = crashes;
                    self.disconnected(Time::from_nanos(self.engine.clock().now), Duration::from_millis(150));
                    return std::mem::take(&mut self.callbacks);
                }
                let packets = std::mem::take(&mut self.engine.peers.packets);
                if packets.is_empty() {
                    self.work = false;
                    let due = Time::from_nanos(self.engine.clock().now);
                    self.due = Some(self.delay_until.map_or(due, |until| until.max(due)));
                    self.pending_observed.append(&mut self.engine.peers.observations);
                    return Vec::new();
                }
                for packet in packets {
                    self.packet(packet);
                }
            }
            panic!("finite original Ops protocol work");
        }
        Vec::new()
    }
    fn next_due(&self) -> Option<Time> {
        if let Some(until) = self.offline_until {
            return Some(until);
        }
        self.due.or_else(|| {
            if self.work {
                Some(self.now)
            } else {
                // Advancing the conformance clock supplies the actual owners'
                // timer events, including fleet adoption and subsecond task retries.
                Some(Time::from_nanos(self.engine.clock().now.checked_add(100_000_000).expect("finite timer clock")))
            }
        })
    }
    fn observed(&mut self) -> Vec<Observed> {
        std::mem::take(&mut self.observed)
    }
    fn fault(&mut self, now: Time, fault: FrontFault) {
        self.advance(now);
        match fault {
            FrontFault::Restart { offline } => {
                self.engine.crash().expect("actual conformance cold restart");
                self.crashes_seen = self.engine.outcome().crashes.len();
                self.disconnected(now, offline);
            }
            FrontFault::Slow { span } => {
                let until = now.saturating_add(span);
                self.delay_until = Some(until);
                self.due = self.due.map(|due| due.max(until));
            }
            FrontFault::NoAnswer => self.lose_answers = self.lose_answers.checked_add(1).expect("finite lost answers"),
            FrontFault::DropWatch => self.drop_watches(),
            FrontFault::Busy
            | FrontFault::NotReady
            | FrontFault::SignedOut
            | FrontFault::Versions
            | FrontFault::ObjectGone => {
                panic!("not an Ops joint story fault: {fault:?}");
            }
        }
    }
    fn reload(&mut self, client: ClientId, now: Time) {
        self.advance(now);
        self.detach(client);
        if self.offline_until.is_none() {
            self.callbacks.push((client, client::Event::Jig(self.engine.peers.transport.hello())));
        }
    }
    fn received(&mut self, client: ClientId, event: &client::Event) {
        match event {
            client::Event::Jig(jig_domain_client::Event::Watch {
                watch,
                event: jig_domain_client::contract::WatchEvent::Delivered(_),
            }) => {
                if let Some(core::Event::Delivered { watcher }) = self.engine.peers.transport.taken(client, *watch) {
                    self.engine.peers.protocol.consumed(watcher);
                    self.send_event(root::Event::Core(core::Event::Delivered { watcher }));
                }
            }
            client::Event::Infrastructure(infra::Event::Watch { watch, event: infra::WatchEvent::Delivered(_) }) => {
                self.application_taken(client, *watch, 2);
            }
            client::Event::Observability(obs::Event::Watch { watch, event: obs::WatchEvent::Delivered(_) }) => {
                self.application_taken(client, *watch, 1);
            }
            client::Event::Start { .. }
            | client::Event::Went { .. }
            | client::Event::Act { .. }
            | client::Event::Answer { .. }
            | client::Event::Jig(_)
            | client::Event::Infrastructure(_)
            | client::Event::Observability(_)
            | client::Event::SignedOut => {}
        }
    }
    fn sign_in(&mut self, client: ClientId, now: Time, _wall: Wall) {
        self.advance(now);
        let event = self.engine.peers.transport.start_sign_in(client);
        self.send_event(root::Event::Core(event));
    }
}
impl Joint {
    fn application_taken(&mut self, client: ClientId, native: Token, connector: u16) {
        let watcher = self.watches.iter().find_map(|(watcher, route)| {
            if route.client == client
                && route.native == native
                && route.subject.connector() == connector
                && route.flight
            {
                Some(*watcher)
            } else {
                None
            }
        });
        if let Some(watcher) = watcher {
            self.watches.get_mut(&watcher).expect("original application route").flight = false;
            self.engine.peers.protocol.consumed(watcher);
            self.send_event(root::Event::Core(core::Event::Delivered { watcher }));
        }
    }
}
impl Joint {
    fn application_refused(&mut self, watcher: Token, why: core::WatchRefusal) {
        let Some(route) = self.watches.remove(&watcher) else {
            return;
        };
        let event = match route.subject {
            Subject::Infrastructure(_) => client::Event::Infrastructure(infra::Event::Ended {
                watch: route.native,
                end: match why {
                    core::WatchRefusal::SignedOut => infra::WatchEnd::SignedOut,
                    core::WatchRefusal::NotAllowed => infra::WatchEnd::NotAllowed,
                    core::WatchRefusal::Unknown => infra::WatchEnd::Unknown,
                    core::WatchRefusal::Busy => infra::WatchEnd::Busy,
                },
            }),
            Subject::Observability(_) => client::Event::Observability(obs::Event::Ended {
                watch: route.native,
                end: match why {
                    core::WatchRefusal::SignedOut => obs::WatchEnd::SignedOut,
                    core::WatchRefusal::NotAllowed => obs::WatchEnd::NotAllowed,
                    core::WatchRefusal::Unknown => obs::WatchEnd::Unknown,
                    core::WatchRefusal::Busy => obs::WatchEnd::Busy,
                },
            }),
        };
        self.callbacks.push((route.client, event));
    }
}
