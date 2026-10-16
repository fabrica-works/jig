//! The test application's native client over simulated or real io.
use crate::{family, front::Side};
use jig_client_protocol_link as link;
use jig_client_shell_native::{Settings, Shell};
use jig_client_test_domain::{self as client, Request};
use jig_client_world::{App, Test};
use jig_domain_client::address::{Address, TaskSection};
use link::Signing;
use skein_fake_person::{Doing, Find, Next, Person, Step, TreeFace};
use skein_io::kernel::{Complete, Fd, Submit};
use skein_lib::{Duration, Env, Queue, RequestRecord, Time, Wall};
use skein_shell::Host;
/// The fake person's finite script, shared by simulated and real worlds.
pub struct Script {
    steps: Box<[Step<client::Address>]>,
}
impl Script {
    /// Start a chat and keep reading until its answer is visible.
    #[must_use]
    pub fn answered_chat(words: &[u8], answer: &[u8]) -> Self {
        let mut steps = Self::start_chat(words).steps.into_vec();
        steps.push(Step::See { find: Find::text(answer), within: Duration::from_secs(5) });
        Self { steps: steps.into_boxed_slice() }
    }
    /// Read this fixture's answer card, excluding its request and instructions.
    #[must_use]
    pub fn answered_report(words: &[u8], answer: &[u8], within: Duration) -> Self {
        let mut steps = Self::start_chat(words).steps.into_vec();
        steps.push(Step::See {
            find: Find::named(skein_ui::Role::Paragraph, answer).within(Box::new([
                skein_fake_person::Target::ExactRole {
                    role: skein_ui::Role::Region,
                    name: b"Historical object".as_slice().into(),
                },
                skein_fake_person::Target::ExactRole {
                    role: skein_ui::Role::Region,
                    name: b"Project: 1".as_slice().into(),
                },
            ])),
            within,
        });
        Self { steps: steps.into_boxed_slice() }
    }
    /// Open the durable conversation and read its answer after a restart.
    #[must_use]
    pub fn found_chat(answer: &[u8]) -> Self {
        Self {
            steps: Box::new([
                Step::Go { address: client::Address::Jig(Address::Task { task: 1, section: TaskSection::Activity }) },
                Step::See { find: Find::text(answer), within: Duration::from_secs(5) },
            ]),
        }
    }
    #[must_use]
    pub fn start_chat(words: &[u8]) -> Self {
        Self {
            steps: vec![
                Step::See {
                    find: Find::named(skein_ui::Role::TextArea, b"First words"),
                    within: Duration::from_secs(1),
                },
                Step::Type { find: Find::named(skein_ui::Role::TextArea, b"First words"), words: words.into() },
                Step::Press { find: Find::named(skein_ui::Role::Button, b"Start a conversation") },
            ]
            .into_boxed_slice(),
        }
    }
}
/// Decoded facts retained outside the client domain for protocol referees.
#[derive(Clone, Debug)]
pub enum Received {
    SignedIn { person: u64 },
    Sent { attempt: skein_lib::Token, key: skein_lib::RequestKey },
    Answer { attempt: skein_lib::Token, answer: jig_domain_client::contract::Answer },
    Delivery(jig_domain_client::contract::Delivery),
    Read(jig_domain_client::contract::ReadResult),
}
/// A native client as a process of a skein world.
pub struct NativeClient {
    shell: Shell,
    saving_signing: Option<Signing>,
    terminal: Option<skein_ui_text::Terminal>,
    #[expect(clippy::disallowed_types, reason = "scenario input hands inherited pipes to the host between iterations")]
    pipes: core::cell::Cell<Option<(Fd, Fd)>>,
    complete: Queue<Complete>,
    side: Side,
    root: client::Domain,
    view: jig_client_test_view::View<'static>,
    person: Person<client::Address>,
    env: Env<client::Limits>,
    out: Queue<Request>,
    patches: Queue<skein_ui::Patch>,
    kept: Vec<client::store::Record>,
    address: client::Address,
    started: bool,
    person_done: bool,
    wake: Option<Time>,
    closing: bool,
    pub received: Vec<Received>,
    pub signed_in: u32,
    pub chats_made: u32,
}
impl NativeClient {
    #[must_use]
    pub fn new(settings: Settings, root: Fd, user: u32, seed: u64, script: Script) -> Self {
        let mut shell = Shell::new(settings);
        shell.adopt(root, user, seed);
        let limits = jig_client_world::test::limits();
        let (root, view) = Test::new(seed);
        let patches = Queue::with_capacity(Test::patch_limit(&view));
        let complete = Queue::with_capacity(shell.operations() + 10);
        Self {
            terminal: None,
            pipes: None.into(),
            complete,
            shell,
            saving_signing: None,
            side: Side::new(),
            root,
            view,
            person: Person::new(script.steps),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            out: Queue::with_capacity(Test::max_out(&limits) * 4),
            patches,
            kept: Vec::new(),
            address: jig_client_world::stories::conversation(),
            started: false,
            person_done: false,
            wake: None,
            closing: false,
            received: Vec::new(),
            signed_in: 0,
            chats_made: 0,
        }
    }
    /// # Errors
    /// Refuses a link outside the configured loopback engine's address.
    pub fn sign_in(&mut self, printed: &[u8]) -> Result<(), jig_client_shell_native::Unlinked> {
        let secret = self.shell.sign_in(printed)?;
        self.side.event(self.env.now, link::Event::SignIn { with: link::With::Link(secret) });
        Ok(())
    }
    fn signing(&mut self, signing: Signing) {
        match signing {
            Signing::SignedIn { party } => {
                self.received.push(Received::SignedIn { person: party });
                self.signed_in += 1;
                self.side.authenticated = true;
            }
            Signing::SignedOut => self.callback(client::Event::SignedOut),
            other @ (Signing::Again | Signing::Lost | Signing::Refused) => panic!("native sign in failed: {other:?}"),
        }
    }
    fn callback(&mut self, event: client::Event) {
        match &event {
            client::Event::Answer { attempt, answer: Some(client::Answered::Jig(answer)), .. } => {
                self.received.push(Received::Answer { attempt: *attempt, answer: answer.clone() });
            }
            client::Event::Jig(jig_domain_client::Event::Watch {
                event: jig_domain_client::contract::WatchEvent::Delivered(delivery),
                ..
            }) => self.received.push(Received::Delivery(delivery.clone())),
            client::Event::Jig(jig_domain_client::Event::Read { result, .. }) => {
                self.received.push(Received::Read(result.clone()));
            }
            client::Event::Answer { .. }
            | client::Event::Start { .. }
            | client::Event::Went { .. }
            | client::Event::Act { .. }
            | client::Event::Jig(_)
            | client::Event::First(_)
            | client::Event::SignedOut => {}
        }
        if matches!(
            &event,
            client::Event::Answer {
                answer: Some(client::Answered::Jig(jig_domain_client::contract::Answer::Done(
                    jig_domain_client::contract::Outcome::Owner(
                        jig_domain_client::contract::OwnerOutcome::Started { .. }
                    )
                ))),
                ..
            }
        ) {
            self.chats_made += 1;
        }
        Test::step(&mut self.root, &self.env, event, &mut self.out);
    }
    fn kept(&mut self, saved: link::Saved) {
        let record = match saved {
            link::Saved::Jig(record) => client::store::Record::Jig(record),
            link::Saved::Family { bytes, .. } => {
                client::store::Record::First(family::restored(&bytes).expect("family place"))
            }
            link::Saved::Table { version, record } => client::store::Record::Table {
                version,
                record: RequestRecord {
                    key: record.key,
                    scope: record.scope,
                    first_sent: record.first_sent,
                    request: match record.request {
                        link::Ask::Jig(ask) => client::Ask::Jig(ask),
                        link::Ask::Family(_) => panic!("chat script restores jig requests"),
                    },
                },
            },
        };
        self.kept.push(record);
    }
    fn save(record: client::store::Record) -> link::Saved {
        match record {
            client::store::Record::Jig(record) => link::Saved::Jig(record),
            client::store::Record::First(record) => {
                link::Saved::Family { family: 1, version: 1, key: Box::new([]), bytes: family::saved(&record) }
            }
            client::store::Record::Table { version, record } => link::Saved::Table {
                version,
                record: RequestRecord {
                    key: record.key,
                    scope: record.scope,
                    first_sent: record.first_sent,
                    request: match record.request {
                        client::Ask::Jig(ask) => link::Ask::Jig(ask),
                        client::Ask::First(ask) => link::Ask::Family(family::request(&ask).expect("family request")),
                    },
                },
            },
        }
    }
    fn outputs(&mut self) {
        while let Some(request) = self.out.pop() {
            match request {
                Request::Save { record } => {
                    self.side.event(self.env.now, link::Event::Save { record: Self::save(record) });
                }
                Request::Erase { key } => self.side.event(
                    self.env.now,
                    link::Event::Erase {
                        key: match key {
                            client::store::Key::Jig(key) => link::Erased::Jig(key),
                            client::store::Key::Table(key) => link::Erased::Table(key),
                            client::store::Key::First(_) => link::Erased::Family { family: 1, key: Box::new([]) },
                        },
                    },
                ),
                Request::Show { address, .. } => {
                    if self.address != address {
                        self.address = address;
                        self.callback(Test::went(address));
                    }
                }
                Request::SignOut => self.side.event(self.env.now, link::Event::SignOut),
                Request::SignIn { .. } => panic!("printed link supplied by caller"),
                request @ (Request::Send { .. } | Request::Jig(_) | Request::First(_) | Request::Fact { .. }) => {
                    if let Request::Send { attempt, key, .. } = &request {
                        self.received.push(Received::Sent { attempt: *attempt, key: *key });
                    }
                    self.side.wire(self.env.now, request);
                }
            }
        }
        while let Some(request) = self.side.out.pop() {
            if let link::Request::Signing { signing } = request {
                if matches!(signing, Signing::SignedIn { .. } | Signing::SignedOut) {
                    self.shell.keep(self.side.link.credential().cloned());
                    self.saving_signing = Some(signing);
                } else {
                    self.signing(signing);
                }
                continue;
            }
            if let link::Request::Kept { record } = request {
                self.kept(record);
            } else {
                match self.side.callback(request) {
                    Ok(event) => self.callback(event),
                    Err(request) => self.shell.down(request),
                }
            }
        }
    }
    fn text_step(&mut self, now: Time, changed: bool) {
        if let Some(mut terminal) = self.terminal.take() {
            terminal.iterate(now, Test::tree(&self.view), changed);
            while let Some(command) = terminal.commands().pop() {
                match command {
                    skein_ui_text::Command::Act { node, first, submit } => {
                        let event = Test::act(&mut self.view, node, first).expect("terminal act");
                        self.callback(event);
                        if submit {
                            let event =
                                Test::act(&mut self.view, node, skein_ui::Act::Submit).expect("terminal submit");
                            self.callback(event);
                        }
                    }
                    skein_ui_text::Command::Print
                    | skein_ui_text::Command::Close
                    | skein_ui_text::Command::Nothing
                    | skein_ui_text::Command::Refused { .. } => unreachable!("terminal queues acts only"),
                }
            }
            while let Some(submit) = terminal.submissions().pop() {
                self.shell.submissions().push(submit);
            }
            self.person_done = terminal.closed();
            self.terminal = Some(terminal);
        }
    }
}
impl Host for NativeClient {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        let mut changed = false;
        if let Some((input, output)) = self.pipes.take() {
            self.terminal.as_mut().expect("terminal supplied").adopt(input, output).expect("pipe room");
        }
        while let Some(complete) = self.complete.pop() {
            if skein_ui_text::Terminal::owns(complete.op) {
                self.terminal.as_mut().expect("terminal operation").completions().push(complete);
            } else {
                self.shell.completions().push(complete);
            }
        }
        self.shell.iterate(now, wall);
        if let Some((credential, kept)) = self.shell.loaded() {
            if let Some(credential) = credential {
                self.side.event(now, link::Event::SignIn { with: link::With::Held(credential) });
                self.side.authenticated = true;
            }
            self.side.event(
                now,
                link::Event::Start {
                    kept: jig_client_world::test::list(kept),
                    versions: jig_client_world::test::list([
                        link::StoreVersion {
                            keeper: link::Keeper::Jig,
                            version: jig_domain_client::store::STORE_VERSION,
                        },
                        link::StoreVersion { keeper: link::Keeper::Table, version: client::store::VERSION },
                        link::StoreVersion { keeper: link::Keeper::Family(1), version: 1 },
                    ]),
                },
            );
        }
        if let Some(ok) = self.shell.kept() {
            let signing = self.saving_signing.take().expect("pending credential");
            self.signing(if ok { signing } else { Signing::Lost });
        }
        while let Some(event) = self.shell.events().pop() {
            self.side.event(now, event);
        }
        self.outputs();
        if self.side.authenticated && !self.started {
            self.started = true;
            let kept = std::mem::take(&mut self.kept);
            self.callback(Test::start(self.address, kept, 0));
        }
        if !self.closing && self.started {
            Test::fire(&mut self.root, &self.env, &mut self.out);
            Test::take(&mut self.root, &self.env, &mut self.out);
            self.side.link.fire(now, &mut self.side.out);
            self.outputs();
            Test::build(&mut self.view, &self.root, &mut self.patches);
            changed = !self.patches.is_empty();
            while self.patches.pop().is_some() {}
            if self.terminal.is_none() {
                self.wake = None;
                let before = self.person.at();
                match self.person.next(&mut TreeFace::new(Test::tree(&self.view)), now) {
                    Next::Do(Doing::Act { node, act }) => {
                        let event = Test::act(&mut self.view, node, act).expect("offered person action");
                        self.callback(event);
                    }
                    Next::Do(Doing::Go { address }) => {
                        self.address = address;
                        self.callback(Test::went(address));
                    }
                    Next::Do(Doing::Reload) => panic!("this native script has no reload"),
                    Next::Wait { until } => self.wake = until,
                    Next::Done => self.person_done = true,
                    Next::Failed { step, why } => panic!("native person step {step}: {why:?}"),
                }
                if self.person.at() != before {
                    self.wake = Some(now);
                }
            }
            self.outputs();
        }
        self.text_step(now, changed);
        if !self.closing && self.person_done && self.root.jig().chats().starting.is_none() {
            self.closing = true;
            self.shell.close();
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.shell.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.shell.work_pending(now)
            || !self.complete.is_empty()
            || self.pipes.get().is_some()
            || self.terminal.as_ref().is_some_and(|terminal| terminal.work_pending(now))
            || !self.out.is_empty()
            || !self.side.out.is_empty()
            || (!self.closing && self.started && self.wake.is_some_and(|at| at <= now))
    }
    fn next_deadline(&self) -> Option<Time> {
        [
            self.shell.next_deadline(),
            self.next_policy_deadline(),
            self.terminal.as_ref().and_then(skein_ui_text::Terminal::next_deadline),
        ]
        .into_iter()
        .flatten()
        .min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        if self.closing {
            None
        } else {
            [Test::next_deadline(&self.root), self.side.link.next_deadline(), self.wake].into_iter().flatten().min()
        }
    }
    fn is_empty(&self) -> bool {
        self.closing && self.shell.is_empty() && self.terminal.as_ref().is_none_or(skein_ui_text::Terminal::closed)
    }
    fn worst_case(&self) -> u64 {
        self.shell.worst_case() + Test::worst_case(&self.env.limits) + 8 * 1024 * 1024
    }
    fn operations(&self) -> u32 {
        self.shell.operations() + if self.terminal.is_some() { 10 } else { 0 }
    }
}

mod world;
pub use world::{Outcome, run, run_terminal};
