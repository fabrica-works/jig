//! The test application's native client over simulated or real io.
use crate::{family, front::Side};
use jig_client_world::{App, Test};
use jig_fake_person::{Doing, Find, Next, Person, Step, TreeFace};
use jig_protocol_client_link as link;
use jig_shell_native::{Settings, Shell, Signing};
use jig_test_domain_client::{self as client, Request};
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
    #[must_use]
    pub fn start_chat(words: &[u8]) -> Self {
        Self {
            steps: vec![
                Step::See {
                    find: Find::named(jig_view::Role::TextArea, b"First words"),
                    within: Duration::from_secs(1),
                },
                Step::Type { find: Find::named(jig_view::Role::TextArea, b"First words"), words: words.into() },
                Step::Press { find: Find::named(jig_view::Role::Button, b"Start a conversation") },
            ]
            .into_boxed_slice(),
        }
    }
}
/// A native client as a process of a skein world.
pub struct NativeClient {
    shell: Shell,
    side: Side,
    root: client::Domain,
    view: jig_test_view::View<'static>,
    person: Person<client::Address>,
    env: Env<client::Limits>,
    out: Queue<Request>,
    patches: Queue<jig_view::Patch>,
    kept: Vec<client::store::Record>,
    address: client::Address,
    started: bool,
    person_done: bool,
    wake: Option<Time>,
    closing: bool,
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
        Self {
            shell,
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
            signed_in: 0,
            chats_made: 0,
        }
    }
    /// # Errors
    /// Refuses a link outside the configured loopback engine's address.
    pub fn sign_in(&mut self, printed: &[u8]) -> Result<(), jig_shell_native::Unlinked> {
        self.shell.sign_in(printed)
    }
    fn callback(&mut self, event: client::Event) {
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
                Request::SignOut => self.shell.sign_out(),
                Request::SignIn { .. } => panic!("printed link supplied by caller"),
                request @ (Request::Send { .. } | Request::Jig(_) | Request::First(_) | Request::Fact { .. }) => {
                    self.side.wire(self.env.now, request);
                }
            }
        }
        while let Some(request) = self.side.out.pop() {
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
}
impl Host for NativeClient {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        self.shell.iterate(now, wall);
        if let Some(kept) = self.shell.loaded() {
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
        while let Some(signing) = self.shell.signing() {
            match signing {
                Signing::SignedIn { .. } => {
                    self.signed_in += 1;
                    self.side.authenticated = true;
                }
                Signing::SignedOut { .. } => self.callback(client::Event::SignedOut),
                other @ (Signing::Again { .. } | Signing::Lost { .. } | Signing::Refused { .. }) => {
                    panic!("native sign in failed: {other:?}")
                }
            }
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
            while self.patches.pop().is_some() {}
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
            self.outputs();
            if self.person_done && self.chats_made > 0 {
                self.closing = true;
                self.shell.close();
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.shell.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.shell.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.shell.work_pending(now)
            || !self.out.is_empty()
            || !self.side.out.is_empty()
            || (!self.closing && self.started && self.wake.is_some_and(|at| at <= now))
    }
    fn next_deadline(&self) -> Option<Time> {
        [self.shell.next_deadline(), self.next_policy_deadline()].into_iter().flatten().min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        if self.closing {
            None
        } else {
            [Test::next_deadline(&self.root), self.side.link.next_deadline(), self.wake].into_iter().flatten().min()
        }
    }
    fn is_empty(&self) -> bool {
        self.closing && self.shell.is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.shell.worst_case() + Test::worst_case(&self.env.limits) + 8 * 1024 * 1024
    }
    fn operations(&self) -> u32 {
        self.shell.operations()
    }
}

mod world;
pub use world::{Outcome, run};
