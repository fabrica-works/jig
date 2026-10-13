//! A single Service owns both layers; the world owns its external parties and machine.
use crate::{Script, fixtures};
use jig_channel_world::{
    fixtures as channel,
    world::store::{Capture, Store},
};
use jig_client_protocol_world::native::NativeClient;
use jig_domain_core as core;
use jig_test_domain as root;
use skein_io::kernel::{Complete, Submit};
use skein_lib::{List, Queue, Time, Wall};
use skein_shell::Host;
use skein_world::{Inherited, Memory, Referee};

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub seed: u64,
    pub config: skein_sim::Config,
}
#[derive(Debug)]
pub struct Outcome {
    pub verdict: skein_world::domain::Verdict,
    pub chats_made: u32,
    pub runs_answered: u32,
    pub drained: bool,
    pub claims_settled_at_restart: u32,
}
struct ServiceHost {
    service: jig_test_service::Service,
    capture: Capture,
    store: Store,
    codec: jig_protocol_store::codec::Limits,
    connector: jig_test_connector::Limits,
}
impl Host for ServiceHost {
    fn iterate(&mut self, now: Time, wall: Wall) {
        for complete in self.service.completions().iter() {
            self.capture.complete(complete);
        }
        self.store.observe(&mut self.capture.events, &self.codec, &self.connector);
        jig_test_service::iterate(&mut self.service, now, wall);
        for submit in self.service.submissions().iter() {
            self.capture.pending(&submit.kind);
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.service.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.service.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.service.work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.service.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.service.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.service.is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.service.worst_case() + 8 * 1024 * 1024
    }
    fn operations(&self) -> u32 {
        self.service.operations()
    }
}
enum Proc {
    Service(Box<ServiceHost>),
    Client(Box<NativeClient>),
    Agent { host: Box<skein_world::PipeHost<smith_agent_shell::Agent>> },
    Peer(Box<skein_fake_peers::llm::Peer>),
}
impl Proc {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Service(host) => host.as_ref(),
            Self::Client(host) => host.as_ref(),
            Self::Agent { host, .. } => host.as_ref(),
            Self::Peer(host) => host.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Service(host) => host.as_mut(),
            Self::Client(host) => host.as_mut(),
            Self::Agent { host, .. } => host.as_mut(),
            Self::Peer(host) => host.as_mut(),
        }
    }
}
impl Host for Proc {
    fn drain(&mut self) {
        self.host_mut().drain();
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.host().next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.host().worst_case()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
}
fn service(inherited: &Inherited) -> Proc {
    let root = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"state").expect("store root").1;
    let worker = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"worker").expect("worker root").1;
    let mut roots = List::with_capacity(1);
    roots.push(worker).expect("worker root");
    let mut service = jig_test_service::startup(
        &fixtures::document(),
        &fixtures::files(),
        Some(&channel::agent()),
        &channel::seeds(),
        None,
        Some(jig_test_service::WorkerRuntime { roots }),
    )
    .expect("one-process startup");
    service.adopt_store(root, 1000).expect("actual deployment store");
    service.adopt_signals(inherited.signal).expect("service signals");
    let mut capture = Capture::default();
    capture.root(root);
    let limits = channel::store_limits();
    let codec = jig_protocol_store::codec::limits(&limits.root.core).expect("admitted fixture codecs");
    Proc::Service(Box::new(ServiceHost {
        service,
        capture,
        store: Store::new(),
        codec,
        connector: limits.root.connector,
    }))
}
fn agent(_spawn: &skein_io::kernel::Spawn, inherited: &Inherited) -> Proc {
    let stderr = inherited.pipes.iter().find(|(child, _)| *child == 2).expect("agent stderr").1;
    let (errors, capture) = smith_agent_process_world::AgentErrors::pipe(stderr);
    let host = smith_agent_process_world::configured(channel::agent_configuration(), fixtures::SEED, inherited, errors);
    Proc::Agent { host: Box::new(capture.host(host)) }
}
struct Judge {
    handed: bool,
    done: bool,
}
fn terminals(store: &Store) -> u32 {
    u32::try_from(
        store
            .rows
            .values()
            .filter(|row| {
                matches!(row,
                    root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(terminal)))
                    if matches!(terminal.end, jig_domain_tasks::End::Finished { .. })
                )
            })
            .count(),
    )
    .expect("fixture terminal count")
}
impl Referee<Proc> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Proc]) {
        let clients_done = procs
            .iter()
            .filter_map(|proc| match proc {
                Proc::Client(client) => Some(client.is_empty()),
                Proc::Service(_) | Proc::Agent { .. } | Proc::Peer(_) => None,
            })
            .all(|done| done);
        let (services, other) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut services[0] else { panic!("one service process") };
        if !self.handed
            && let Some(link) = service.service.links().pop()
        {
            for proc in other.iter_mut() {
                if let Proc::Client(client) = proc {
                    client.sign_in(&link).expect("printed link");
                }
            }
            self.handed = true;
        }
        if !self.done && clients_done && terminals(&service.store) > 0 {
            service.service.shutdown();
            self.done = true;
        }
        if self.done {
            for proc in other {
                if let Proc::Peer(peer) = proc {
                    peer.shutdown();
                }
            }
        }
    }
    fn observe(&mut self, _now: Time, _procs: &[Proc]) {}
    fn next_deadline(&self) -> Option<Time> {
        if self.done { None } else { Some(Time::from_nanos(10_000_000_000)) }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        if !self.done && now >= Time::from_nanos(10_000_000_000) {
            Some("one-process answer not seen".into())
        } else {
            None
        }
    }
    fn passed(&self) -> bool {
        self.done
    }
}
struct Machine(skein_fake_machine::Machine);
impl skein_world::Machine for Machine {
    fn open_root(&mut self, path: &[u8]) -> Result<skein_sim::Handle, skein_io::kernel::Error> {
        let index = match path {
            b"state" => 0,
            b"agent-root" => 1,
            _ => return Err(skein_io::kernel::Error::NotFound),
        };
        Ok(skein_sim::Handle::new(self.0.reopen_root(index).raw()))
    }
    fn close_root(&mut self, root: skein_sim::Handle) {
        self.0.close(skein_fake_machine::Opened::new(root.raw()));
    }
    fn step(&mut self, call: skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) {
        skein_fake_machine::step(&mut self.0, call, answers);
    }
}
#[must_use]
pub fn run(settings: Settings, people: Box<[Script]>) -> Outcome {
    assert_eq!(people.len(), 1, "the first one-process story has one person");
    let mut machine = skein_fake_machine::Machine::for_user(1000);
    let _deployment = machine.lay(&[]);
    let _worker = machine.lay(&[]);
    let client_root = machine.lay(&[]);
    let judge = Judge { handed: false, done: false };
    let mut world = skein_world::World::new(settings.seed, settings.config, judge, Memory::Unchecked)
        .with_machine(Machine(machine));
    world.host(skein_world::HostedProgram {
        program: b"agent-program".as_slice().into(),
        make: agent,
        instances: 1,
        operations: smith_agent_process_world::limits().routes.checked_add(2).expect("agent operations"),
    });
    world.spawn_restartable(
        vec![
            skein_world::StartupRoot { name: b"state".as_slice().into(), path: b"state".as_slice().into() },
            skein_world::StartupRoot { name: b"worker".as_slice().into(), path: b"agent-root".as_slice().into() },
        ],
        service,
    );
    for script in people {
        world.spawn_root(skein_sim::Handle::new(client_root.raw()), |root| {
            Proc::Client(Box::new(NativeClient::new(fixtures::client_settings(), root, 1000, settings.seed, script)))
        });
    }
    world.spawn(|| Proc::Peer(Box::new(fixtures::provider())));
    let outcome = world.run();
    let chats_made = outcome
        .procs
        .iter()
        .filter_map(|proc| match proc {
            Proc::Client(client) => Some(client.chats_made),
            Proc::Service(_) | Proc::Agent { .. } | Proc::Peer(_) => None,
        })
        .sum();
    Outcome {
        verdict: skein_world::domain::Verdict::Passed,
        chats_made,
        runs_answered: outcome
            .procs
            .iter()
            .find_map(|proc| match proc {
                Proc::Service(service) => Some(terminals(&service.store)),
                Proc::Client(_) | Proc::Agent { .. } | Proc::Peer(_) => None,
            })
            .expect("one service"),
        drained: outcome.procs.iter().all(Host::is_empty),
        claims_settled_at_restart: 0,
    }
}
