//! The real Ops engine and client meet at an ordinary protocol face (testing.md, 5).
use super::engine::OpsApplication;
use crate::conformance::{Config, Story};
use jig_client_world::harness::{ClientId, Front, FrontFault, Observed};
use jig_conformance::Harness;
use jig_ops_client_domain::{self as client, infrastructure as infra, observability as obs};
use jig_ops_domain as root;
use skein_lib::{Duration, Time, Wall};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Subject {
    Infrastructure(infra::Subject),
    Observability(obs::Subject),
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
    pub(super) story: Story,
    pub(super) seed: u64,
    bytes: Option<super::bytes::Bytes>,
    pub(super) callbacks: Vec<(ClientId, client::Event)>,
    now: Time,
    pub(super) due: Option<Time>,
    pub(super) work: bool,
    pub(super) observed: Vec<Observed>,
    pub(super) pending_observed: Vec<Observed>,
    pub(super) offline_until: Option<Time>,
    pub(super) lose_answers: u32,
    pub(super) crashes_seen: usize,
}
impl Joint {
    #[must_use]
    pub fn new(seed: u64, limits: client::Limits) -> Self {
        Self::for_story(seed, limits, Story::Alert)
    }
    #[must_use]
    pub fn for_story(seed: u64, limits: client::Limits, story: Story) -> Self {
        let configuration = Config::new(seed, story).client_backed().client_recovery();
        let mut engine = Harness::with_trace(configuration, seed, false);
        engine.drain().expect("actual Ops cold start");
        let mut joint = Self {
            engine,
            environment_documents: Vec::new(),
            lost_final_answers: 0,
            disconnects: 0,
            reconnects: 0,
            limits,
            story,
            seed,
            bytes: Some(super::bytes::Bytes::new(seed, story, limits.jig.rows)),
            callbacks: Vec::new(),
            now: Time::ZERO,
            due: None,
            work: true,
            observed: Vec::new(),
            pending_observed: Vec::new(),
            offline_until: None,
            lose_answers: 0,
            crashes_seen: 0,
        };
        let mut bytes = joint.bytes.take().expect("new byte face");
        bytes.pump(&mut joint, Time::ZERO, Wall::EPOCH);
        joint.bytes = Some(bytes);
        joint
    }
    pub(super) fn send_event(&mut self, event: root::Event) {
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
}
pub(super) fn subject_route(
    subject: Subject,
) -> Result<(jig_domain_people::ApplicationScope, root::ClientSubject), &'static str> {
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
                infra::Scope::Project(project) => jig_domain_people::ApplicationScope::Project(project),
                infra::Scope::Deployment => jig_domain_people::ApplicationScope::Deployment,
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
                obs::Scope::Project(project) => jig_domain_people::ApplicationScope::Project(project),
                obs::Scope::Deployment => jig_domain_people::ApplicationScope::Deployment,
            };
            Ok((scope, root::ClientSubject::Observability(owner)))
        }
    }
}
impl Joint {
    pub(super) fn disconnected(&mut self, now: Time, offline: Duration) {
        self.disconnects += 1;
        self.callbacks.clear();
        self.due = None;
        self.lose_answers = 0;
        // Cold restoration can advance the engine clock beyond this front's
        // poll. Publish its durable facts only once the client clock catches up.
        self.pending_observed.append(&mut self.engine.peers.observations);
        self.observed.push(Observed::Reachable { live: false });
        self.offline_until = Some(now.saturating_add(offline));
        self.work = true;
    }
    pub(super) fn observe_environment(&mut self, document: &root::ClientDocument) {
        if let root::ClientDocument::Infrastructure(document) = document
            && matches!(document.as_ref(), jig_ops_domain_infrastructure::ClientDocument::Environment { .. })
        {
            self.environment_documents.push(document.as_ref().clone());
        }
    }
}
pub(super) enum ApplicationDocument {
    Infrastructure(Box<infra::Document>),
    Observability(Box<obs::Document>),
}
pub(super) fn application_document(
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
        self.bytes.as_mut().expect("byte face").bind(client);
    }
    fn starting(&mut self, client: ClientId) -> Vec<client::Event> {
        let events = self.bytes.as_mut().expect("byte face").starting(client);
        // A held credential initializes the client before Start restores its
        // records, as in the reusable protocol world's reload surface.
        if matches!(
            events.first(),
            Some(client::Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Held(_) } })
        ) {
            return events;
        }
        self.callbacks.extend(events.into_iter().map(|event| (client, event)));
        Vec::new()
    }
    fn stored(&mut self, client: ClientId, records: &[client::store::Record]) {
        if self.bytes.as_mut().expect("byte face").stored(client, records) {
            self.callbacks.push((client, client::Event::Client { event: jig_client::Event::Stored }));
        }
    }
    fn receipts(&mut self, client: ClientId, event: &client::Event) -> Vec<jig_client_world::harness::Receipt> {
        self.bytes.as_mut().expect("byte face").receipts(client, event)
    }
    fn delivers(&mut self, client: ClientId, event: &client::Event) -> bool {
        self.bytes.as_mut().expect("byte face").delivers(client, event)
    }
    fn wire(&mut self, client: ClientId, now: Time, _wall: Wall, request: client::Request) {
        self.advance(now);
        if let client::Request::Client { request } = request {
            self.bytes.as_mut().expect("byte face").send(client, now, request);
        } else {
            panic!("all wire work uses the linked byte client");
        }
    }
    fn poll(&mut self, now: Time, wall: Wall) -> Vec<(ClientId, client::Event)> {
        self.advance(now);
        let mut bytes = self.bytes.take().expect("byte face");
        let callbacks = bytes.pump(self, now, wall);
        self.bytes = Some(bytes);
        callbacks
    }
    fn next_due(&self) -> Option<Time> {
        if let Some(until) = self.offline_until {
            return Some(until);
        }
        self.bytes.as_ref().and_then(super::bytes::Bytes::next_due).into_iter().chain(self.due).min().or_else(|| {
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
                let mut bytes = self.bytes.take().expect("byte face");
                bytes.restart(self, now.saturating_add(offline));
                self.bytes = Some(bytes);
            }
            FrontFault::Slow { span } => {
                self.bytes.as_mut().expect("byte face").slow(now, span);
            }
            FrontFault::NoAnswer => self.lose_answers = self.lose_answers.checked_add(1).expect("finite lost answers"),
            FrontFault::DropWatch => self.bytes.as_mut().expect("byte face").drop_stream(),
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
        self.bytes.as_mut().expect("byte face").reload(client);
    }
    fn received(&mut self, _client: ClientId, _event: &client::Event) {}
    fn sign_out(&mut self, client: ClientId, now: Time, _wall: Wall) {
        self.callbacks.push((client, client::Event::Client { event: jig_client::Event::SignOut }));
        self.advance(now);
    }
    fn sign_in(&mut self, client: ClientId, _now: Time, _wall: Wall) {
        let events = self.bytes.as_mut().expect("byte face").starting(client);
        self.callbacks.extend(events.into_iter().map(|event| (client, event)));
    }
}
