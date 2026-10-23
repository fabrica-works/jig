//! Real services, worker-v8 bytes, `ProgramSource`, and the hosted auth fake.
use crate::{
    processes::{self, Proc},
    referee::Rules,
    stories::Story,
    world::{self, Store},
};
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_protocol_worker_wire as wire;
use skein_fake_machine::{How, Item, Opened};
use skein_io::{self as io, kernel};
use skein_lib::{Duration, Queue, Reader, ReplyTo, Time, Token};
use skein_shell::Host;
use skein_world::{HostedProgram, Machine};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub(crate) struct Files {
    pub(crate) files: skein_fake_machine::Machine,
    pub(crate) base: Opened,
}
impl Machine for Files {
    fn open_root(&mut self, path: &[u8]) -> Result<skein_sim::Handle, kernel::Error> {
        self.files
            .open(self.base, path.strip_prefix(b"/").unwrap_or(path), How::Directory)
            .map(|fd| skein_sim::Handle::new(fd.raw()))
            .map_err(skein_fake_machine::kernel_error)
    }
    fn close_root(&mut self, root: skein_sim::Handle) {
        self.files.close(Opened::new(root.raw()));
    }
    fn open_append(
        &mut self,
        root: skein_sim::Handle,
        path: &[u8],
        mode: u32,
    ) -> Result<skein_sim::Handle, kernel::Error> {
        self.files
            .open(Opened::new(root.raw()), path, How::Append { mode })
            .map(|fd| skein_sim::Handle::new(fd.raw()))
            .map_err(skein_fake_machine::kernel_error)
    }
    fn step(&mut self, call: skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) {
        skein_fake_machine::step(&mut self.files, call, answers);
    }
}
#[derive(PartialEq, Eq)]
enum Phase {
    New,
    Sources,
    Asked,
    Login,
    Done,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SignOutAnswer {
    Unconfirmed,
    Answered(fleet::SignOutResult),
}
struct Judge {
    shared: Rc<RefCell<Run>>,
    store: Store,
    rules: Rules,
    phase: Phase,
    sign_out: bool,
    lost: bool,
    login: Option<Rc<Cell<LoginPhase>>>,
    ended: Option<SignOutAnswer>,
    states: Vec<host::LoginState>,
    observed: usize,
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        let hellos = self.observe_wire(procs);
        for proc in procs {
            if self.phase == Phase::Done {
                match proc {
                    Proc::Engine(host) | Proc::Worker(host) if !host.stopping => {
                        host.stopping = true;
                        host.service.shutdown();
                    }
                    Proc::Engine(_)
                    | Proc::Worker(_)
                    | Proc::Auth(_)
                    | Proc::Client(_)
                    | Proc::Agent { .. }
                    | Proc::Peer(_)
                    | Proc::Probe(_) => {}
                }
            }
            match proc {
                Proc::Engine(engine) => {
                    world::answer_store(engine, &mut self.store, &mut self.rules, false, Story::Chat, now);
                    let engine = engine.service.engine().expect("engine");
                    if self.store.applied > 0 && self.phase == Phase::New {
                        engine.parties_in().push(core::Event::Fleet(fleet::Event::Sources {
                            sources: Box::new([fleet::SourceName::new(b"alice").expect("name")]),
                        }));
                        self.phase = Phase::Sources;
                    }
                    while let Some(output) = engine.parties_out().pop() {
                        if let jig_test_service::PartyOutput::Held(jig_test_domain::Delivery::Core(
                            core::Held::SourceSignedOut { to, result, .. },
                        )) = output
                        {
                            assert_eq!(to.into_token(), Token::new(700));
                            assert!(self.ended.is_none(), "sign-out ends once");
                            self.ended = Some(result.map_or(SignOutAnswer::Unconfirmed, SignOutAnswer::Answered));
                        }
                    }
                    if !self.states.is_empty() && self.phase == Phase::Sources {
                        if self.sign_out {
                            engine.parties_in().push(core::Event::Fleet(fleet::Event::SignOut {
                                to: ReplyTo::new(Token::new(700)),
                                channel: Token::new(1),
                                name: fleet::SourceName::new(b"alice").expect("name"),
                            }));
                        }
                        if let Some(login) = &self.login {
                            login.set(LoginPhase::Start);
                            self.phase = Phase::Login;
                        } else {
                            self.phase = Phase::Asked;
                        }
                    }
                    if self.phase == Phase::Login && self.login.as_ref().expect("login story").get() == LoginPhase::Done
                    {
                        engine.parties_in().push(core::Event::Fleet(fleet::Event::Check {
                            channel: Token::new(1),
                            name: fleet::SourceName::new(b"alice").expect("name"),
                        }));
                        self.phase = Phase::Asked;
                    }
                }
                Proc::Worker(_)
                | Proc::Auth(_)
                | Proc::Client(_)
                | Proc::Agent { .. }
                | Proc::Peer(_)
                | Proc::Probe(_) => {}
            }
        }
        self.record(hellos);
    }

    fn observe(&mut self, _now: Time, _procs: &[Proc]) {}
    fn next_deadline(&self) -> Option<Time> {
        None
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now > Time::ZERO.saturating_add(Duration::from_secs(30))).then(|| "source exchange settles".into())
    }
    fn passed(&self) -> bool {
        self.phase == Phase::Done
    }
    fn last_word(&self) -> skein_world::LastWord {
        skein_world::LastWord::LastExpectation
    }
}
impl Judge {
    fn observe_wire(&mut self, procs: &[Proc]) -> usize {
        let mut snapshots = Vec::new();
        let mut hellos = 0;
        for proc in procs {
            if let Proc::Worker(worker) = proc {
                hellos = worker.wire.frames.iter().filter(|(_, kind, _)| *kind == 0x0100).count();
                for (_, kind, bytes) in worker.wire.frames.iter().skip(self.observed) {
                    if *kind == wire::Kind::Logins.number() {
                        let record =
                            wire::Logins::decode(&wire::CEILINGS, &mut Reader::new(bytes)).expect("whole login bytes");
                        if record.logins().len() == 1 {
                            snapshots.push(record);
                        }
                    }
                }
                self.observed = worker.wire.frames.len();
            }
        }
        for snapshot in snapshots {
            let row = snapshot.into_parts().logins.into_boxed().into_iter().next().expect("one source").into_parts();
            assert_eq!(row.name.as_ref(), b"alice");
            self.states.push(match row.state {
                wire::LoginState::Usable => host::LoginState::Usable,
                wire::LoginState::NotSignedIn => host::LoginState::NotSignedIn,
                wire::LoginState::SignedInOtherwise(method) => {
                    host::LoginState::SignedInOtherwise(host::LoginMethod::new(method.method()).expect("method"))
                }
                wire::LoginState::Failed(failure) => {
                    let failure = failure.into_parts();
                    assert!(failure.detail.len() <= 4096);
                    host::LoginState::Failed {
                        reason: match failure.reason {
                            wire::SourceFailure::Missing => host::SourceFailure::Missing,
                            wire::SourceFailure::Deadline => host::SourceFailure::Deadline,
                            wire::SourceFailure::Unreadable => host::SourceFailure::Unreadable,
                            wire::SourceFailure::Exited => host::SourceFailure::Exited,
                            wire::SourceFailure::Unconfigured => host::SourceFailure::Unconfigured,
                            wire::SourceFailure::Unknown => host::SourceFailure::Unknown,
                            wire::SourceFailure::Closed => host::SourceFailure::Closed,
                        },
                        detail: failure.detail,
                    }
                }
            });
        }
        hellos
    }
    fn record(&mut self, hellos: usize) {
        self.shared.borrow_mut().states.clone_from(&self.states);
        self.shared.borrow_mut().ended.clone_from(&self.ended);
        if self.phase == Phase::Asked
            && (!self.sign_out || self.ended.is_some())
            && (!self.lost || hellos >= 2 && self.states.len() >= 2)
            && (self.login.is_none() || self.states.last() == Some(&host::LoginState::Usable))
        {
            self.phase = Phase::Done;
        }
    }
}
#[derive(Default)]
pub struct Run {
    pub states: Vec<host::LoginState>,
    pub ended: Option<SignOutAnswer>,
    pub observations: Vec<smith_fake_claude::Observation>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum FollowUp {
    Observe,
    LoginThenCheck,
}

#[must_use]
fn run_inner(
    login: Option<&[u8]>,
    story: &smith_fake_claude::Story,
    sign_out: bool,
    lost: bool,
    configured: bool,
    follow_up: FollowUp,
) -> Run {
    let login_then_check = follow_up == FollowUp::LoginThenCheck;
    let mut items = vec![
        Item::directory(b"state"),
        Item::directory(b"private"),
        Item::directory(b"private/home"),
        Item::directory(b"private/programs"),
        Item::directory(b"private/programs/alice"),
    ];
    for item in &mut items {
        item.mode = 0o700;
    }
    items.push(Item::file(b"private/programs/alice/fake-claude.story", &story.encode()));
    if let Some(login) = login {
        items.push(Item::file(b"private/programs/alice/fake-claude.login", login));
    }
    let mut files = skein_fake_machine::Machine::new();
    let base = files.lay(&items);
    let root = files.open(base, b".", How::Directory).expect("private worker root");
    let shared = Rc::new(RefCell::new(Run::default()));
    let login_driver = login_then_check.then(|| Rc::new(Cell::new(LoginPhase::Idle)));
    let login_root = login_then_check.then(|| files.open(base, b".", How::Directory).expect("login launcher root"));
    let judge = Judge {
        shared: shared.clone(),
        store: Store::default(),
        rules: Rules::default(),
        phase: Phase::New,
        sign_out,
        lost,
        login: login_driver.clone(),
        ended: None,
        states: Vec::new(),
        observed: 0,
    };
    let config = skein_sim::Config::calm();
    let mut world =
        skein_world::World::new(901, config, judge, skein_world::Memory::Unchecked).with_machine(Files { files, base });
    let factory = smith_fake_claude::hosted(b"claude");
    world.host_startup(
        HostedProgram {
            program: factory.program,
            make: processes::auth,
            instances: factory.instances,
            operations: factory.operations,
        },
        |spawn| smith_fake_claude::config_root(spawn).into_iter().collect(),
        |spawn| smith_fake_claude::observation_append(spawn).into_iter().collect(),
    );
    world.spawn_restartable(
        vec![skein_world::StartupRoot { name: b"state".as_slice().into(), path: b"state".as_slice().into() }],
        processes::engine,
    );
    world.spawn_root(
        skein_sim::Handle::new(root.raw()),
        if configured { processes::worker_programs } else { processes::worker },
    );
    if let (Some(state), Some(root)) = (login_driver, login_root) {
        world.spawn_root(skein_sim::Handle::new(root.raw()), |root| Proc::Auth(Box::new(Login::new(root, state))));
    }
    let mut reset = false;
    let mut outcome = world.run_with_faults(|_, _, submissions| {
        let cut = lost && !reset && submissions.iter().any(|submit| {
            if sign_out {
                matches!(&submit.kind, kernel::Op::Send { bytes, from: 0, .. } if bytes.len() >= 8 && skein_channel::parse_header(&bytes[..8]).is_ok_and(|header| header.kind == wire::Kind::SignedOut.number()))
            } else {
                !shared.borrow().states.is_empty() && matches!(&submit.kind, kernel::Op::Send { .. })
            }
        });
        reset |= cut;
        Some(skein_sim::Faults { reset: if cut { 1000 } else { 0 }, ..config.faults })
    });
    assert!(!shared.borrow().states.is_empty(), "{:?}", outcome.trace);
    let bytes = match outcome.machine.files.open(base, b"private/programs/alice/fake-claude.observations", How::Read) {
        Ok(fd) => {
            let bytes = outcome.machine.files.read(fd, 0, 65536).expect("observations");
            outcome.machine.files.close(fd);
            bytes
        }
        Err(skein_fake_machine::Refusal::NotFound) => Vec::new(),
        Err(error) => panic!("observation file: {error:?}"),
    };
    outcome.machine.files.close(base);
    let mut result = std::mem::take(&mut *shared.borrow_mut());
    result.observations = smith_fake_claude::observations(&bytes);
    result
}

#[must_use]
pub fn run(login: Option<&[u8]>, story: &smith_fake_claude::Story, sign_out: bool, lost: bool) -> Run {
    run_inner(login, story, sign_out, lost, true, FollowUp::Observe)
}
#[must_use]
pub fn unconfigured() -> Run {
    run_inner(None, &smith_fake_claude::Story::default(), false, false, false, FollowUp::Observe)
}

#[must_use]
pub fn login_then_check() -> Run {
    run_inner(None, &smith_fake_claude::Story::default(), false, false, true, FollowUp::LoginThenCheck)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LoginPhase {
    Idle,
    Start,
    Running,
    Closing,
    Done,
}

const LOGIN_ROOT_CLOSE: Token = Token::new(u64::MAX);

// An independent user runs the hosted fake's real auth login. The worker only
// learns of that change when the engine asks it to check the source again.
struct Login {
    root: kernel::Fd,
    state: Rc<Cell<LoginPhase>>,
    command: Option<smith_protocol_program::Command>,
    io: io::Io,
    env: skein_lib::Env<io::Limits>,
    completions: Queue<kernel::Complete>,
    submissions: Queue<kernel::Submit>,
    above: Queue<io::Event>,
    below: Queue<io::Request>,
}
impl Login {
    fn new(root: kernel::Fd, state: Rc<Cell<LoginPhase>>) -> Self {
        let limits = io::Limits { sockets: 3, ..smith_agent_process_world::limits().io };
        Self {
            root,
            state,
            command: None,
            io: io::Io::new(&limits),
            env: skein_lib::Env { now: Time::ZERO, wall: skein_lib::Wall::EPOCH, limits },
            completions: Queue::with_capacity(128),
            submissions: Queue::with_capacity(128),
            above: Queue::with_capacity(128),
            below: Queue::with_capacity(128),
        }
    }
}
impl Host for Login {
    fn iterate(&mut self, now: Time, wall: skein_lib::Wall) {
        self.env.now = now;
        self.env.wall = wall;
        if self.state.get() == LoginPhase::Start {
            self.state.set(LoginPhase::Running);
            self.command = Some(smith_protocol_program::Command::new(
                Token::new(1),
                smith_protocol_program::Launch {
                    program: b"claude".as_slice().into(),
                    arguments: smith_protocol_program::SIGN_IN.iter().map(|arg| Box::from(*arg)).collect(),
                    environment: Box::new([Box::from(b"CLAUDE_CONFIG_DIR=/private/programs/alice".as_slice())]),
                    root: self.root,
                    directory: b".".as_slice().into(),
                },
                now.saturating_add(Duration::from_secs(5)),
                64,
                &mut self.below,
            ));
        }
        while let Some(complete) = self.completions.pop() {
            if complete.op == LOGIN_ROOT_CLOSE {
                assert_eq!(complete.result, Ok(kernel::Done::Nothing));
                self.state.set(LoginPhase::Done);
            } else {
                io::up(&mut self.io, &self.env, complete, &mut self.above, &mut self.submissions);
            }
        }
        while self.io.is_ready() {
            io::resume(&mut self.io, &self.env, &mut self.above, &mut self.submissions);
        }
        while self.io.is_due(now) {
            io::fire(&mut self.io, &self.env, &mut self.above, &mut self.submissions);
        }
        if let Some(command) = &mut self.command {
            while let Some(event) = self.above.pop() {
                command.from_io(event, &mut self.below);
            }
            command.fire(now, &mut self.below);
            if let Some(ended) = command.ended()
                && self.state.get() == LoginPhase::Running
            {
                assert!(
                    matches!(ended, smith_protocol_program::Ended::Exited { exit: kernel::Exit::Code(0), .. }),
                    "{ended:?}"
                );
                self.submissions
                    .push(kernel::Submit { op: LOGIN_ROOT_CLOSE, kind: kernel::Op::Close { fd: self.root } });
                self.state.set(LoginPhase::Closing);
            }
        }
        while let Some(request) = self.below.pop() {
            io::down(&mut self.io, &self.env, request, &mut self.submissions);
        }
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.state.get() == LoginPhase::Start
            || !self.completions.is_empty()
            || !self.above.is_empty()
            || !self.below.is_empty()
            || self.io.is_ready()
            || self.io.is_due(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        [self.io.next_deadline(), self.command.as_ref().and_then(smith_protocol_program::Command::next_deadline)]
            .into_iter()
            .flatten()
            .min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.state.get() == LoginPhase::Done && self.io.is_empty()
    }
    fn worst_case(&self) -> u64 {
        4_194_304
    }
    fn operations(&self) -> u32 {
        io::operations(&self.env.limits).expect("login launcher io").checked_add(1).expect("root close operation")
    }
}
