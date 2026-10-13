//! Host adapters expose only service ports to the scenario (service.md, 4).
use crate::fixtures;
use jig_test_service::Service;
use skein_io::kernel;
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use skein_world::{Inherited, PipeHost};
use smith_agent_process_world as agent_fixture;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) enum Proc {
    Engine(Box<ServiceHost>),
    Worker(Box<ServiceHost>),
    Agent { host: Box<PipeHost<smith_agent_shell::Agent>>, errors: agent_fixture::AgentErrors },
    Peer(Box<skein_fake_peers::llm::Peer>),
}

pub(crate) struct ServiceHost {
    pub(crate) service: Service,
    pub(crate) wire: Wire,
    pub(crate) stopping: bool,
    worker: bool,
}
impl ServiceHost {
    fn new(service: Service, worker: bool) -> Self {
        Self { service, wire: Wire::default(), stopping: false, worker }
    }
}
impl Host for ServiceHost {
    fn iterate(&mut self, now: Time, wall: Wall) {
        for complete in self.service.completions().iter() {
            self.wire.complete(complete, self.worker);
        }
        jig_test_service::iterate(&mut self.service, now, wall);
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        self.service.completions()
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
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
        self.service.worst_case().checked_add(2_097_152).expect("port observations")
    }
    fn operations(&self) -> u32 {
        self.service.operations()
    }
}
impl Proc {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Engine(host) | Self::Worker(host) => host.as_ref(),
            Self::Agent { host, .. } => host.as_ref(),
            Self::Peer(peer) => peer.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Engine(host) | Self::Worker(host) => host.as_mut(),
            Self::Agent { host, .. } => host.as_mut(),
            Self::Peer(peer) => peer.as_mut(),
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
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
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
    fn exit(&self) -> Option<kernel::Exit> {
        self.host().exit()
    }
    fn worst_case(&self) -> u64 {
        self.host().worst_case().checked_add(4096).expect("process wrapper")
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
}

pub(crate) fn engine(inherited: &Inherited) -> Proc {
    let mut service = jig_test_service::startup(
        &fixtures::engine_document(),
        &fixtures::files(false),
        None,
        &fixtures::seeds(),
        None,
        None,
    )
    .expect("engine configuration");
    service.adopt_signals(inherited.signal).expect("engine signal source");
    Proc::Engine(Box::new(ServiceHost::new(service, false)))
}
pub(crate) fn worker(root: kernel::Fd) -> Proc {
    let mut roots = skein_lib::List::with_capacity(1);
    roots.push(root).expect("one worker root");
    let service = jig_test_service::startup(
        fixtures::WORKER_DOCUMENT,
        &fixtures::files(true),
        Some(&fixtures::agent()),
        &fixtures::seeds(),
        None,
        Some(jig_test_service::WorkerRuntime { roots }),
    )
    .expect("worker configuration");
    Proc::Worker(Box::new(ServiceHost::new(service, true)))
}
pub(crate) fn agent(_spawn: &kernel::Spawn, inherited: &Inherited) -> Proc {
    let fd = inherited.pipes.iter().find(|(child, _)| *child == 2).expect("stderr").1;
    let (errors, capture) = agent_fixture::AgentErrors::pipe(fd);
    let agent = agent_fixture::configured(fixtures::agent_configuration(), fixtures::SEED, inherited, errors.clone());
    Proc::Agent { host: Box::new(capture.host(agent)), errors }
}

/// Decode exactly the bytes the kernel accepted or received, once per completed operation.
/// A short send or receive extends its descriptor's retained frame prefix.
#[derive(Default)]
pub(crate) struct Wire {
    completed: BTreeSet<skein_lib::Token>,
    streams: BTreeMap<i32, Vec<u8>>,
    pub(crate) acknowledgements: Vec<(u64, u64)>,
    pub(crate) calls_answered: u32,
}
impl Wire {
    fn complete(&mut self, complete: &kernel::Complete, worker: bool) {
        let Ok(kernel::Done::Count(count)) = complete.result else { return };
        let (fd, bytes) = match (&complete.kind, worker) {
            (kernel::Op::Send { fd, bytes, from }, false) => {
                let start = usize::try_from(*from).expect("send offset");
                (*fd, &bytes[start..start + usize::try_from(count).expect("send length")])
            }
            (kernel::Op::Recv { fd, buf }, true) => (*fd, &buf[..usize::try_from(count).expect("receive length")]),
            _ => return,
        };
        if !self.completed.insert(complete.op) {
            return;
        }
        let stream = self.streams.entry(fd.raw()).or_default();
        stream.extend_from_slice(bytes);
        while stream.len() >= 8 {
            let header = skein_channel::parse_header(&stream[..8]).expect("channel frame header");
            let end = 8 + usize::try_from(header.body_len).expect("bounded frame body");
            if stream.len() < end {
                break;
            }
            if header.kind == 0x0185 {
                let ack = jig_protocol_worker_wire::Acknowledge::decode(
                    &jig_protocol_worker_wire::CEILINGS,
                    &mut skein_lib::Reader::new(&stream[8..end]),
                )
                .expect("channel acknowledgement");
                self.acknowledgements.push((ack.task(), ack.attempt()));
            } else if header.kind == 0x0182 {
                self.calls_answered += 1;
            }
            stream.drain(..end);
        }
    }
}

pub(crate) struct Machine(pub(crate) skein_fake_machine::Machine);
impl skein_world::Machine for Machine {
    fn step(&mut self, call: skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) {
        skein_fake_machine::step(&mut self.0, call, answers);
    }
}
