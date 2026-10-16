//! The joint world's real core under the engine clients component.
use jig_client_world::{
    harness::Observed,
    joint::{engine::Driver, truth::Truth},
};
use jig_domain_core as core;
use jig_protocol_engine_clients::{self as clients, Event, Request, SecretOwner, SecretsAnswer, SecretsRequest};
use jig_test_domain as root;
use skein_lib::{Env, Queue, Time, Token, Wall};
use std::collections::{BTreeMap, BTreeSet};
pub(super) struct Engine {
    pub driver: Driver,
    pub component: clients::Component,
    pub env: Env<clients::Limits>,
    pub out: Queue<Request>,
    pub link: Box<[u8]>,
    pub rules: crate::referee::Rules,
    pub limits: root::Limits,
    pub seed: u64,
    secrets: BTreeMap<SecretOwner, clients::Kept>,
    replies: BTreeSet<Token>,
    truth: Truth,
    observed: Vec<Observed>,
    ticked: Option<Time>,
    pub offered: BTreeMap<Token, u64>,
    pub taken: BTreeMap<Token, u64>,
}
impl Engine {
    pub fn bounded(seed: u64, limits: &root::Limits) -> Self {
        let limits = crate::fixtures::journal(*limits);
        let driver = Driver::new(
            jig_core_world::world::config(seed),
            limits,
            crate::fixtures::peers(),
            Time::ZERO,
            crate::fixtures::wall(Time::ZERO),
        );
        let truth = Truth::after(driver.world.store.rows.clone());
        let env =
            Env { now: Time::ZERO, wall: crate::fixtures::wall(Time::ZERO), limits: crate::fixtures::engine_limits() };
        let mut out = Queue::with_capacity(256);
        let component = clients::Component::new(crate::fixtures::settings(limits.core.documents.rows), &env, &mut out)
            .expect("bounded clients component");
        Self {
            driver,
            component,
            env,
            out,
            link: Box::new([]),
            rules: crate::referee::Rules::default(),
            limits,
            seed,
            secrets: BTreeMap::new(),
            replies: BTreeSet::new(),
            truth,
            observed: Vec::new(),
            ticked: None,
            offered: BTreeMap::new(),
            taken: BTreeMap::new(),
        }
    }
    pub fn clock(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        self.driver.world.set_clock(now, wall);
    }
    pub fn event(&mut self, event: Event) {
        self.component.down(&self.env, event, &mut self.out);
    }
    pub fn request(&mut self, request: Request) -> Option<skein_io::Request> {
        match request {
            Request::Io(request) => return Some(request),
            Request::Link { link } => self.link = link,
            Request::Secrets(request) => {
                let answer = match request {
                    SecretsRequest::Keep { owner, value } => {
                        self.rules.secret(value.digest.expose());
                        self.secrets.insert(owner, value);
                        SecretsAnswer::Kept { owner }
                    }
                    SecretsRequest::Erase { owner } => {
                        self.secrets.remove(&owner);
                        SecretsAnswer::Erased { owner }
                    }
                    SecretsRequest::Get { owner } => {
                        SecretsAnswer::Got { owner, value: self.secrets.get(&owner).cloned() }
                    }
                };
                self.event(Event::Secrets(answer));
            }
            Request::Core(event) => self.apply(root::Event::Core(*event)),
            Request::Application(request) => {
                match jig_test_service::family::to_root(
                    request,
                    &crate::fixtures::owner_limits().connector,
                    crate::fixtures::owner_limits().core.documents.rows,
                ) {
                    Ok(event) => self.apply(event),
                    Err(event) => self.event(*event),
                }
            }
            Request::Provider { .. } => panic!("the world uses the printed link provider"),
            Request::Closed => {}
        }
        None
    }
    fn apply(&mut self, mut event: root::Event) {
        if let root::Event::Core(core::Event::Delivered { watcher }) = &event {
            *self.taken.entry(*watcher).or_default() += 1;
        }
        self.rules.domain(&event).expect("no authentication secret in a root input");
        let to = if let root::Event::Core(
            core::Event::SignIn { reply_to, .. }
            | core::Event::People(
                jig_domain_people::Event::Ask { reply_to, .. } | jig_domain_people::Event::SignOut { reply_to, .. },
            ),
        )
        | root::Event::Application { reply_to, .. } = &mut event
        {
            Some(reply_to)
        } else if let root::Event::ReadApplication { reply, .. } | root::Event::Core(core::Event::Read { reply, .. }) =
            &mut event
        {
            Some(reply)
        } else {
            None
        };
        if let Some(to) = to {
            let original = std::mem::replace(to, skein_lib::ReplyTo::new(Token::new(0))).into_token();
            *to = skein_lib::ReplyTo::new(original);
            self.replies.insert(original);
        }
        let watcher = if let root::Event::WatchApplication { watcher, .. } = &event {
            Some(*watcher)
        } else if let root::Event::Core(event) = &event {
            watcher(event)
        } else {
            None
        };
        let peer = self.driver.world.client_peer.as_mut().expect("joint protocol peer");
        peer.clear_replies();
        for token in &self.replies {
            peer.register_reply(*token);
        }
        self.driver.world.send(event);
        let refused = self.deliveries(watcher);
        if let Some(watcher) = watcher
            && !refused
        {
            self.event(Event::Admitted { watcher });
        }
    }
    fn deliveries(&mut self, watcher: Option<Token>) -> bool {
        let peer = self.driver.world.client_peer.as_mut().expect("joint protocol peer");
        let held = std::mem::take(&mut peer.delivered);
        let immediate = std::mem::take(&mut peer.now);
        let mut refused = false;
        for (_, delivery) in held {
            match delivery {
                root::Delivery::Application { to, number, outcome } => {
                    self.event(Event::Application(clients::FromApplication::Answered {
                        to,
                        family: number,
                        record: jig_test_service::family::outcome(&outcome),
                    }));
                }
                root::Delivery::ApplicationDocument { watcher, number, document } => {
                    self.event(Event::Application(clients::FromApplication::Delivery {
                        watcher,
                        family: number,
                        record: jig_test_service::family::document(&document),
                    }));
                }
                root::Delivery::ApplicationRead { reply, number, answer } => {
                    let record = Some(jig_test_service::family::read_answer(&answer));
                    self.event(Event::Application(clients::FromApplication::Read { reply, family: number, record }));
                }
                root::Delivery::Core(held) => {
                    if let core::Held::Delivery { watcher, .. } = &held {
                        *self.offered.entry(*watcher).or_default() += 1;
                    }
                    if let Some(event) = held_event(held) {
                        self.event(event);
                    }
                }
                root::Delivery::Link { .. }
                | root::Delivery::CallBusy { .. }
                | root::Delivery::Projection { .. }
                | root::Delivery::Message { .. }
                | root::Delivery::Restart(_)
                | root::Delivery::CallAnswer { .. }
                | root::Delivery::Assigned { .. }
                | root::Delivery::Fleet(_)
                | root::Delivery::System { .. }
                | root::Delivery::LostRead { .. }
                | root::Delivery::Procedure { .. } => {}
            }
        }
        for now in immediate {
            if let core::Now::WatchRefused { watcher: owner, .. } = &now {
                refused |= Some(*owner) == watcher;
            }
            if let Some(event) = now_event(now) {
                self.event(event);
            }
        }
        refused
    }
    pub fn poll(&mut self) -> bool {
        if self.ticked != Some(self.env.now) {
            self.ticked = Some(self.env.now);
            self.driver.world.fire_due(self.env.now, self.env.wall);
        }
        let peer = self.driver.world.client_peer.as_ref().expect("joint protocol peer");
        let active = !peer.delivered.is_empty() || !peer.now.is_empty();
        self.deliveries(None);
        for snapshot in self.driver.snapshots() {
            self.observed.extend(self.truth.applied(snapshot));
        }
        self.component.fire(&self.env, &mut self.out);
        while self.component.up(&self.env, &mut self.out) {}
        active
    }
    pub fn observed(&mut self) -> Vec<Observed> {
        std::mem::take(&mut self.observed)
    }
    pub fn chats_made(&self) -> usize {
        self.driver
            .world
            .store
            .rows
            .values()
            .filter(|record| {
                matches!(
                    record,
                    root::Record::Core(core::Record::People(jig_domain_people::Stored::Answer {
                        outcome: jig_domain_people::Outcome::Started { .. },
                        ..
                    }))
                )
            })
            .count()
    }
}
fn held_event(held: core::Held) -> Option<Event> {
    Some(match held {
        core::Held::PeopleReply { to, reply } => Event::Reply { to, reply },
        core::Held::Read { reply, answer } => Event::Read { reply, answer },
        core::Held::Delivery { watcher, document } => Event::Delivery { watcher, document },
        core::Held::WatchEnded { watcher, why } => Event::WatchEnded { watcher, why },
        core::Held::Drained
        | core::Held::ApplicationDelivery { .. }
        | core::Held::ApplicationWatchEnded { .. }
        | core::Held::ReadLoad { .. }
        | core::Held::Applied { .. }
        | core::Held::MakeEffect { .. }
        | core::Held::SettledCall { .. }
        | core::Held::Assign { .. }
        | core::Held::Grant { .. }
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
        | core::Held::NotesDeleted { .. } => return None,
    })
}
fn now_event(now: core::Now) -> Option<Event> {
    Some(match now {
        core::Now::SignInRefused { to } => Event::SignInRefused { to },
        core::Now::WatchRefused { watcher, why } => Event::WatchRefused { watcher, why },
        core::Now::RunText { watcher, turn, from, to, text, cut, call } => {
            Event::RunText { watcher, turn, from, to, text, cut, call }
        }
        core::Now::CallBusy { .. }
        | core::Now::Relayed { .. }
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
        | core::Now::PartyInputs { .. }
        | core::Now::CompleteBrief { .. }
        | core::Now::BriefCorePlanned { .. }
        | core::Now::WorkspaceRequest { .. }
        | core::Now::RunPreparationFailed { .. }
        | core::Now::RunPrepared { .. }
        | core::Now::HistoricalEscalation { .. }
        | core::Now::RestoreRefused => return None,
    })
}

fn watcher(event: &core::Event) -> Option<Token> {
    match event {
        core::Event::Watch { watcher, .. } | core::Event::WatchRun { watcher, .. } => Some(*watcher),
        core::Event::Fill { .. }
        | core::Event::Stopping
        | core::Event::Marked { .. }
        | core::Event::ReadLoaded { .. }
        | core::Event::Read { .. }
        | core::Event::Application { .. }
        | core::Event::ApplicationPart { .. }
        | core::Event::ApplicationFinished { .. }
        | core::Event::ProjectionSettled { .. }
        | core::Event::SettledCall { .. }
        | core::Event::EffectStart { .. }
        | core::Event::EffectConnector(_)
        | core::Event::EffectDeadline
        | core::Event::SignIn { .. }
        | core::Event::Unwatch { .. }
        | core::Event::Delivered { .. }
        | core::Event::StreamDelivered { .. }
        | core::Event::RunCall { .. }
        | core::Event::RunText { .. }
        | core::Event::Tasks(_)
        | core::Event::StartRecurring { .. }
        | core::Event::StartBrief { .. }
        | core::Event::BriefNotes { .. }
        | core::Event::BriefAssembled { .. }
        | core::Event::WorkspacePrepared { .. }
        | core::Event::ClaimPrepared { .. }
        | core::Event::ClaimRefused { .. }
        | core::Event::ConnectorRepair { .. }
        | core::Event::Period { .. }
        | core::Event::PreparationFailed { .. }
        | core::Event::TranscriptUnread { .. }
        | core::Event::PersonProposal(_)
        | core::Event::PersonEscalation(_)
        | core::Event::TaskEscalation(_)
        | core::Event::People(_)
        | core::Event::Fleet(_)
        | core::Event::TurnPayload { .. }
        | core::Event::AnswerPayload { .. }
        | core::Event::AcceptedTurn { .. }
        | core::Event::RefusedPayload { .. }
        | core::Event::Activate { .. }
        | core::Event::PreparedAgent { .. }
        | core::Event::Forget { .. }
        | core::Event::SeenChatLoaded { .. }
        | core::Event::HistoricalProposal { .. }
        | core::Event::HistoricalPersonTask { .. }
        | core::Event::PartyInputsLoaded { .. }
        | core::Event::HistoricalEscalation { .. }
        | core::Event::NamedAnswer { .. }
        | core::Event::SubscribeTopic { .. }
        | core::Event::NamedAction { .. }
        | core::Event::DelegateValidated { .. }
        | core::Event::DelegateInputFailed { .. }
        | core::Event::DelegateHoldings { .. }
        | core::Event::ProcedureHoldings { .. }
        | core::Event::ProcedureStep { .. }
        | core::Event::Brief(_)
        | core::Event::Holdings { .. }
        | core::Event::Account(_)
        | core::Event::View(_)
        | core::Event::Notes(_) => None,
    }
}
