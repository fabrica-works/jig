//! Channel-wide safety and liveness (protocol/channel.md, 12).
use crate::{
    processes::Proc,
    world::{Store, answer_parties, answer_store, diagnostics, party_event, terminal, turns, worker_wire},
};
use jig_domain_core as core;
use jig_fake_parties as parties;
use skein_lib::Time;
use std::{cell::RefCell, rc::Rc};
#[expect(clippy::struct_excessive_bools, reason = "independent party, injection and settlement controls")]
pub struct Referee {
    /// Owned outside every engine process, so restart cannot drop durable rows.
    pub(crate) store: Rc<RefCell<Store>>,
    pub(crate) party: parties::Party,
    pub(crate) period: bool,
    pub(crate) passed: bool,
    pub(crate) stopped: bool,
    pub(crate) rules: Rules,
    pub(crate) observed: usize,
    pub(crate) expected_turns: u32,
    pub(crate) expected_answers: usize,
    pub(crate) story: crate::stories::Story,
    pub(crate) injected: bool,
    pub(crate) resumed_attempt: u64,
    pub(crate) engine_observed: usize,
    pub(crate) roots_observed: std::collections::BTreeMap<u64, u64>,
    pub(crate) formatted: String,
    pub(crate) grants_observed: usize,
    pub(crate) engine_instance: Option<u64>,
    pub(crate) views: Rc<RefCell<crate::world::Views>>,
}
impl skein_world::Referee<Proc> for Referee {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        self.inject(now, procs);
        for proc in procs {
            match proc {
                Proc::Engine(host) => {
                    let engine = host.service.engine().expect("engine layer");
                    let mut store = self.store.borrow_mut();
                    if self.engine_instance.is_some_and(|instance| instance != host.revision.0) {
                        store.crash();
                    }
                    self.engine_instance = Some(host.revision.0);
                    answer_store(
                        engine,
                        &mut store,
                        &mut self.rules,
                        matches!(self.story, crate::stories::Story::HeldCommit | crate::stories::Story::Drain)
                            && now < Time::from_nanos(1_000_000_000),
                        self.story,
                        now,
                    );
                    self.views.borrow_mut().messages_ended = self.rules.message_ends.len();
                    self.views.borrow_mut().refusals_kept = !self.rules.refused_messages.is_empty();
                    answer_parties(engine, &mut self.party, &mut self.views.borrow_mut());
                    if !self.period && store.applied > 0 {
                        engine.parties_in().push(core::Event::Period {
                            project: 1,
                            period: 1,
                            budget: 100,
                            next: None,
                        });
                        self.period = true;
                    }
                    if self.period
                        && !self.passed
                        && let Some(up) = self.party.tick(now)
                    {
                        engine.parties_in().push(party_event(up));
                    }
                    if self.passed && !host.stopping {
                        host.stopping = true;
                        host.service.shutdown();
                    }
                }
                Proc::Worker(host) if self.passed && !host.stopping => {
                    host.stopping = true;
                    host.service.shutdown();
                }
                Proc::Peer(peer) if self.passed => peer.shutdown(),
                Proc::Probe(peer) if self.passed => peer.shutdown(),
                Proc::Worker(_) | Proc::Peer(_) | Proc::Agent { .. } | Proc::Probe(_) => {}
            }
        }
        self.stopped = self.passed;
    }
    fn observe(&mut self, now: Time, procs: &[Proc]) {
        if self.observe_probe(procs) {
            return;
        }
        let worker = worker_wire(procs);
        self.observe_worker(worker);
        self.observe_roots(worker, procs);
        let engine = procs
            .iter()
            .find_map(|proc| if let Proc::Engine(host) = proc { Some(host) } else { None })
            .expect("engine");
        self.observe_engine(now, engine);
        let store = self.store.borrow();
        let turns = turns(&store);
        assert!(turns.len() <= self.expected_turns as usize, "bounded committed turns for the script");
        let engine = procs
            .iter()
            .find_map(|proc| match proc {
                Proc::Engine(engine) => Some(engine),
                Proc::Worker(_) | Proc::Agent { .. } | Proc::Peer(_) | Proc::Probe(_) => None,
            })
            .expect("engine process");
        assert!(worker.acknowledgements.len() <= self.expected_answers, "one received answer acknowledgement");
        assert!(engine.wire.acknowledgements.len() <= self.expected_answers, "one sent answer acknowledgement");
        for &(task, attempt) in &worker.acknowledgements {
            assert!(
                terminal(&store, task, attempt, self.story),
                "acknowledgement follows a durable successful terminal: {task}/{attempt} at {now:?}; replies {:?}; turns {:?}; diagnostics {}",
                self.party.replies,
                crate::world::turns(&store).iter().map(|turn| (turn.attempt, turn.turn, turn.read)).collect::<Vec<_>>(),
                diagnostics(procs)
            );
        }
        let replied = matches!(
            self.story,
            crate::stories::Story::Refusals(
                crate::stories::MessageRefusal::Full | crate::stories::MessageRefusal::NameInUse
            )
        ) || procs.iter().any(|proc| match proc {
            Proc::Peer(peer) => {
                matches!(
                    self.story,
                    crate::stories::Story::InvalidCharter
                        | crate::stories::Story::Shutdown
                        | crate::stories::Story::Drain
                        | crate::stories::Story::BeyondGrace
                ) || smith_agent_process_world::fake::replied(peer)
            }
            Proc::Engine(_) | Proc::Worker(_) | Proc::Agent { .. } | Proc::Probe(_) => false,
        });
        let ended = if matches!(self.story, crate::stories::Story::InvalidCharter) {
            self.rules.refused_ends.len() == 1 && worker.acknowledgements.is_empty()
        } else if matches!(self.story, crate::stories::Story::InvalidTranscript) {
            self.rules.refused_ends.len() == 1 && worker.acknowledgements.len() == self.expected_answers
        } else {
            worker.acknowledgements.len() == self.expected_answers
        };
        if self.party.quiescent()
            && turns.len() == self.expected_turns as usize
            && ended
            && replied
            && self.rules.complete()
        {
            self.passed = true;
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.passed {
            return (!self.stopped).then_some(Time::ZERO);
        }
        if matches!(self.story, crate::stories::Story::HeldCommit) && !self.injected
            || matches!(self.story, crate::stories::Story::Drain) && !self.store.borrow().pending.is_empty()
        {
            Some(Time::from_nanos(1_000_000_000))
        } else {
            Some(Time::from_nanos(300_000_000_000))
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.passed && now >= Time::from_nanos(300_000_000_000)).then(|| {
            format!(
                "story {:?} completes; replies {:?}, durable turns {}, rules {:?}",
                self.story,
                self.party.replies,
                turns(&self.store.borrow()).len(),
                self.rules
            )
        })
    }
    fn passed(&self) -> bool {
        self.passed && self.stopped
    }
}

/// Event ledger used by the live referee and its planted-breach tests.
/// Replayed wire transmissions are not new durable decisions.
#[derive(Debug, Default)]
pub struct Rules {
    turns: std::collections::BTreeSet<(u64, u64, u32)>,
    turn_acks: std::collections::BTreeSet<(u64, u64, u32)>,
    answers: std::collections::BTreeSet<(u64, u64)>,
    refused_starts: std::collections::BTreeSet<(u64, u64)>,
    refused_ends: std::collections::BTreeSet<(u64, u64)>,
    answer_acks: std::collections::BTreeSet<(u64, u64)>,
    lost_ends: std::collections::BTreeSet<(u64, u64)>,
    calls: std::collections::BTreeSet<(u64, u64, Box<[u8]>)>,
    call_answers: std::collections::BTreeSet<(u64, u64, Box<[u8]>)>,
    messages: std::collections::BTreeSet<(u64, u64, u64)>,
    refused_messages: std::collections::BTreeSet<(u64, u64, u64)>,
    message_ends: std::collections::BTreeSet<(u64, u64, u64)>,
    running: std::collections::BTreeMap<u64, (u64, Time)>,
    failed: Option<String>,
}
impl Rules {
    pub fn turned(&mut self, task: u64, attempt: u64, number: u32) {
        if !self.turns.insert((task, attempt, number)) {
            self.fail("a turn reaches the durable engine once");
        }
    }
    pub fn turn_acknowledged(&mut self, task: u64, attempt: u64, number: u32) {
        let key = (task, attempt, number);
        if !self.turns.contains(&key) || !self.turn_acks.insert(key) {
            self.fail("a turn is acknowledged once, after its durable commit");
        }
    }
    pub fn answered(&mut self, task: u64, attempt: u64) {
        let unread: Vec<_> = self
            .messages
            .range((task, attempt, 0)..=(task, attempt, u64::MAX))
            .filter(|key| !self.message_ends.contains(key))
            .copied()
            .collect();
        for (_, _, name) in unread {
            self.message_ended(task, attempt, name);
        }
        // The worker's terminal tail reports that the agent has already gone.
        self.running.remove(&task);
        if self.refused_starts.contains(&(task, attempt)) {
            if !self.refused_ends.insert((task, attempt)) {
                self.fail("a refused start ends once");
            }
        } else if !self.answers.insert((task, attempt)) {
            self.fail("an answer reaches the durable engine once");
        }
    }
    pub fn lost_ended(&mut self, task: u64, attempt: u64, now: Time) {
        if self.running.get(&task).is_some_and(|(_, until)| now < *until) {
            self.fail("a lost attempt ends only after its worker's stop bound");
        }
        if !self.lost_ends.insert((task, attempt)) {
            self.fail("a lost attempt ends once");
        }
        self.running.remove(&task);
        let unread: Vec<_> = self
            .messages
            .range((task, attempt, 0)..=(task, attempt, u64::MAX))
            .filter(|key| !self.message_ends.contains(key))
            .copied()
            .collect();
        for (_, _, name) in unread {
            self.message_ended(task, attempt, name);
        }
        if self.refused_starts.contains(&(task, attempt)) {
            self.refused_ends.insert((task, attempt));
        }
    }
    pub fn refused(&mut self, task: u64, attempt: u64) {
        self.running.remove(&task);
        if !self.refused_starts.insert((task, attempt)) {
            self.fail("a refused start reaches the engine at most once");
        }
    }
    pub fn acknowledged(&mut self, task: u64, attempt: u64) {
        let key = (task, attempt);
        if self.refused_starts.contains(&key) {
            self.fail("a refused start is never acknowledged");
        }
        if !self.answers.contains(&key) || !self.answer_acks.insert(key) {
            self.fail("an answer is acknowledged once, after its durable commit");
        }
        self.running.remove(&task);
    }
    pub fn called(&mut self, task: u64, attempt: u64, name: &[u8]) {
        self.calls.insert((task, attempt, name.into()));
    }
    pub fn call_answered(&mut self, task: u64, attempt: u64, name: &[u8]) {
        let key = (task, attempt, name.into());
        if !self.calls.contains(&key) || !self.call_answers.insert(key) {
            self.fail("each relayed call is answered once");
        }
    }
    pub fn sent(&mut self, task: u64, attempt: u64, name: u64) {
        self.messages.insert((task, attempt, name));
    }
    pub fn message_ended(&mut self, task: u64, attempt: u64, name: u64) {
        let key = (task, attempt, name);
        if !self.messages.contains(&key) || !self.message_ends.insert(key) {
            self.fail("each sent message ends once");
        }
    }
    pub fn message_refused(&mut self, task: u64, attempt: u64, name: u64) {
        self.refused_messages.insert((task, attempt, name));
        if !self.message_ends.contains(&(task, attempt, name)) {
            self.message_ended(task, attempt, name);
        }
    }
    pub fn read(&mut self, task: u64, attempt: u64, through: Option<u64>) {
        let Some(through) = through else { return };
        let read: Vec<_> = self
            .messages
            .range((task, attempt, 0)..=(task, attempt, through))
            .filter(|key| !self.message_ends.contains(key))
            .copied()
            .collect();
        for (_, _, name) in read {
            self.message_ended(task, attempt, name);
        }
    }
    fn complete(&self) -> bool {
        self.turns == self.turn_acks
            && self.answers == self.answer_acks
            && self.calls == self.call_answers
            && self.messages == self.message_ends
            && self.refused_starts == self.refused_ends
            && self.running.is_empty()
    }
    pub fn placed(&mut self, task: u64, attempt: u64, now: Time) {
        if self.running.get(&task).is_some_and(|(last, until)| *last != attempt && now < *until) {
            self.fail("no new attempt while a worker may run the last");
        }
        self.running.insert(task, (attempt, Time::from_nanos(u64::MAX)));
    }
    pub fn lost(&mut self, task: u64, attempt: u64, may_run_until: Time) {
        if let Some((current, until)) = self.running.get_mut(&task)
            && *current == attempt
        {
            *until = may_run_until;
        }
    }
    pub fn roots(&mut self, state: &str, grants: &[&[u8]]) {
        for value in grants {
            let bytes = format!("{value:?}");
            if !value.is_empty() && state.contains(&bytes) {
                self.fail("a credential grant value appears in a root's state");
            }
        }
    }
    fn fail(&mut self, why: &str) {
        if self.failed.is_none() {
            self.failed = Some(why.into());
        }
    }
    #[must_use]
    pub fn verdict(&self) -> skein_world::domain::Verdict {
        use skein_world::domain::{Failure, Verdict};
        if let Some(why) = &self.failed {
            Verdict::Failed(Failure { at: Time::ZERO, why: why.clone(), pending: Vec::new() })
        } else {
            Verdict::Passed
        }
    }
}

struct Roots<'a>(&'a jig_test_service::Service);
impl std::fmt::Debug for Roots<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt_roots(formatter)
    }
}

impl Referee {
    fn inject(&mut self, now: Time, procs: &mut [Proc]) {
        if matches!(self.story, crate::stories::Story::HeldCommit) && now >= Time::from_nanos(1_000_000_000) {
            self.injected = true;
        }
        self.drain_engine(procs);
        self.resume_runs(procs);
        self.inject_refusal(procs);
        if matches!(self.story, crate::stories::Story::Inbox) && !self.injected {
            self.injected = true;
            for proc in procs.iter_mut() {
                if let Proc::Worker(host) = proc {
                    host.relay.until = Some(Time::from_nanos(1_000_000_000));
                }
            }
        }
        if matches!(self.story, crate::stories::Story::Facts) && !self.injected
            && procs.iter().any(|proc| matches!(proc, Proc::Peer(peer) if smith_agent_process_world::fake::queries(peer).next().is_some())) {
            self.injected = true;
            for proc in procs.iter_mut() {
                if let Proc::Engine(host) = proc {
                    let input = host.service.engine().expect("engine").parties_in();
                    input.push(core::Event::Watch { watcher: skein_lib::Token::new(70), sign_in: 1,
                        subject: core::documents::Watched::Task { task: 1 },
                        window: core::documents::Window { rows: 1, after: None } });
                    input.push(core::Event::WatchRun { watcher: skein_lib::Token::new(71), sign_in: 1,
                        task: 1, turn: None, offset: 0 });
                }
            }
        }
        if matches!(self.story, crate::stories::Story::Shutdown) && !self.injected
            && procs.iter().any(|proc| matches!(proc, Proc::Peer(peer) if smith_agent_process_world::fake::queries(peer).next().is_some())) {
            self.injected = true;
            for proc in procs.iter_mut() {
                if let Proc::Worker(host) = proc { host.service.shutdown(); host.stopping = true; }
            }
        }
        let (engine, worker) = match self.story {
            crate::stories::Story::Partition { engine, worker } => (engine, worker),
            crate::stories::Story::BeyondGrace => {
                (skein_lib::Duration::from_secs(40), skein_lib::Duration::from_secs(40))
            }
            crate::stories::Story::Chat
            | crate::stories::Story::Stalled
            | crate::stories::Story::Quiet
            | crate::stories::Story::Replaced
            | crate::stories::Story::Reconnect
            | crate::stories::Story::Shutdown
            | crate::stories::Story::Drain
            | crate::stories::Story::Refusals(_)
            | crate::stories::Story::Opening(_)
            | crate::stories::Story::Progress
            | crate::stories::Story::Facts
            | crate::stories::Story::Inbox
            | crate::stories::Story::Call
            | crate::stories::Story::Wait
            | crate::stories::Story::InvalidTranscript
            | crate::stories::Story::MidFrame
            | crate::stories::Story::Window
            | crate::stories::Story::InvalidCharter
            | crate::stories::Story::Restart { .. }
            | crate::stories::Story::HeldCommit => return,
        };
        if self.injected
            || !procs.iter().any(|proc| match proc {
                Proc::Worker(host) => host.wire.frames.iter().any(|(_, kind, _)| *kind == 0x0180),
                Proc::Engine(_) | Proc::Agent { .. } | Proc::Peer(_) | Proc::Probe(_) => false,
            })
        {
            return;
        }
        self.injected = true;
        for proc in procs {
            match proc {
                Proc::Engine(host) => {
                    host.relay.until = Some(now.saturating_add(engine));
                    host.relay.channel = host.wire.channel;
                }
                Proc::Worker(host) => {
                    host.relay.until = Some(now.saturating_add(worker));
                    host.relay.channel = host.wire.channel;
                }
                Proc::Agent { .. } | Proc::Peer(_) | Proc::Probe(_) => {}
            }
        }
    }
}

impl Referee {
    fn observe_probe(&mut self, procs: &[Proc]) -> bool {
        if let crate::stories::Story::Opening(case) = self.story {
            let probe = procs
                .iter()
                .find_map(|proc| if let Proc::Probe(peer) = proc { Some(peer) } else { None })
                .expect("probe");
            let expected = match case {
                crate::stories::Opening::Secret => Some(256),
                crate::stories::Opening::Application => Some(257),
                crate::stories::Opening::Version => Some(1),
                crate::stories::Opening::Terms => Some(2),
                crate::stories::Opening::Family | crate::stories::Opening::Garbage => Some(3),
            };
            if let Some(reason) = probe.refused {
                assert_eq!(Some(reason), expected, "opening {case:?}");
                self.passed = true;
            }
            if matches!(case, crate::stories::Opening::Family | crate::stories::Opening::Garbage) && probe.ended {
                self.passed = true;
            }
            return true;
        }
        if matches!(self.story, crate::stories::Story::Replaced) {
            self.passed = procs.iter().filter(|proc| matches!(proc, Proc::Probe(peer) if peer.ready)).count() == 2
                && procs.iter().any(|proc| matches!(proc, Proc::Probe(peer) if peer.ended));
            return true;
        }
        if matches!(self.story, crate::stories::Story::Stalled | crate::stories::Story::Quiet) {
            self.passed = procs.iter().any(|proc| matches!(proc, Proc::Engine(host) if host.wire.closed))
                && procs.iter().any(|proc| matches!(proc, Proc::Probe(peer) if peer.ready));
            return true;
        }
        false
    }
}

impl Referee {
    fn observe_worker(&mut self, worker: &crate::processes::Wire) {
        for (_, kind, bytes) in &worker.frames[self.observed..] {
            let mut reader = skein_lib::Reader::new(bytes);
            match *kind {
                0x0183 => {
                    let kept =
                        jig_protocol_worker_wire::TurnKept::decode(&jig_protocol_worker_wire::CEILINGS, &mut reader)
                            .expect("kept turn");
                    self.rules.turn_acknowledged(kept.task(), kept.attempt(), kept.number());
                }
                0x0182 => {
                    let reply =
                        jig_protocol_worker_wire::Relayed::decode(&jig_protocol_worker_wire::CEILINGS, &mut reader)
                            .expect("relayed call");
                    if matches!(reply.answer(), jig_protocol_worker_wire::RelayAnswer::Text(_)) {
                        self.rules.call_answered(reply.task(), reply.attempt(), reply.name());
                    }
                }
                0x0185 => {
                    let ack =
                        jig_protocol_worker_wire::Acknowledge::decode(&jig_protocol_worker_wire::CEILINGS, &mut reader)
                            .expect("answer ack");
                    self.rules.acknowledged(ack.task(), ack.attempt());
                }
                _ => {}
            }
        }
        self.observed = worker.frames.len();
    }
}

impl Referee {
    fn observe_roots(&mut self, worker: &crate::processes::Wire, procs: &[Proc]) {
        let grants: Vec<_> = worker.grants.iter().map(AsRef::as_ref).collect();
        if self.grants_observed != grants.len() {
            self.roots_observed.clear();
            self.grants_observed = grants.len();
        }
        for proc in procs {
            if let Proc::Engine(host) | Proc::Worker(host) = proc
                && !grants.is_empty()
                && self.roots_observed.get(&host.revision.0) != Some(&host.revision.1)
            {
                use std::fmt::Write;
                self.formatted.clear();
                write!(&mut self.formatted, "{:?}", Roots(&host.service)).expect("root state");
                self.rules.roots(&self.formatted, &grants);
                self.roots_observed.insert(host.revision.0, host.revision.1);
            }
        }
        assert_eq!(self.rules.verdict(), skein_world::domain::Verdict::Passed);
    }
}

impl Referee {
    fn observe_engine(&mut self, now: Time, engine: &crate::processes::ServiceHost) {
        for (&(task, attempt), &fd) in &engine.wire.placements {
            if let Some(until) = engine.wire.lost.get(&fd) {
                self.rules.lost(task, attempt, *until);
            }
        }
        if self.engine_observed > engine.wire.frames.len() {
            self.engine_observed = 0;
        }
        for (at, kind, bytes) in &engine.wire.frames[self.engine_observed..] {
            if *kind == 0x0181 {
                let message = jig_protocol_worker_wire::Inbound::decode(
                    &jig_protocol_worker_wire::CEILINGS,
                    &mut skein_lib::Reader::new(bytes),
                )
                .expect("inbound message");
                self.rules.sent(message.task(), message.attempt(), message.name());
            }
            if *kind == 0x0103 {
                let answer = jig_protocol_worker_wire::Answer::decode(
                    &jig_protocol_worker_wire::CEILINGS,
                    &mut skein_lib::Reader::new(bytes),
                )
                .expect("worker answer");
                if matches!(answer.ending(), jig_protocol_worker_wire::Ending::Refused(_)) {
                    assert!(
                        !matches!(self.story, crate::stories::Story::Window),
                        "the compaction fixture must admit its start: {:?}",
                        answer.ending()
                    );
                    self.rules.refused(answer.task(), answer.attempt());
                }
            }
            if *kind == 0x0102 {
                let call = jig_protocol_worker_wire::Call::decode(
                    &jig_protocol_worker_wire::CEILINGS,
                    &mut skein_lib::Reader::new(bytes),
                )
                .expect("call");
                self.rules.called(call.task(), call.attempt(), call.name());
            }
            if *kind == 0x0180 {
                let assignment = jig_protocol_worker_wire::Assignment::decode(
                    &jig_protocol_worker_wire::CEILINGS,
                    &mut skein_lib::Reader::new(bytes),
                )
                .expect("assignment");
                self.rules.placed(assignment.task(), assignment.attempt(), *at);
                for message in assignment.messages() {
                    self.rules.sent(assignment.task(), assignment.attempt(), message.name());
                }
            }
        }
        self.engine_observed = engine.wire.frames.len();
        assert_eq!(self.rules.verdict(), skein_world::domain::Verdict::Passed, "at {now:?}");
    }
}

impl Referee {
    fn resume_runs(&mut self, procs: &[Proc]) {
        let parked = self
            .store
            .borrow()
            .rows
            .values()
            .filter_map(|row| match row {
                jig_test_domain::Record::Core(core::Record::Core(core::CoreRecord::Terminal(answer)))
                    if answer.end == jig_domain_tasks::End::Parked =>
                {
                    Some(answer.attempt)
                }
                jig_test_domain::Record::Core(_) | jig_test_domain::Record::Connector { .. } => None,
            })
            .max();
        if matches!(self.story, crate::stories::Story::Window | crate::stories::Story::InvalidTranscript)
            && parked.is_some_and(|at| at > self.resumed_attempt)
            && self.party.quiescent()
        {
            self.injected = true;
            self.resumed_attempt = parked.expect("parked attempt");
            self.party.extend(Box::new([parties::Act::Request {
                key: [u8::try_from(self.resumed_attempt + 2).expect("few attempts"); 16],
                ask: parties::Ask::Say {
                    task: parties::Task::Last,
                    words: b"Continue now".as_slice().into(),
                    release: false,
                },
            }]));
        }
        if matches!(self.story, crate::stories::Story::Wait)
            && !self.injected
            && procs.iter().any(|proc| matches!(proc, Proc::Worker(host) if host.wire.waited))
        {
            self.injected = true;
            self.party.extend(Box::new([parties::Act::Request {
                key: [2; 16],
                ask: parties::Ask::Say {
                    task: parties::Task::Last,
                    words: b"Continue now".as_slice().into(),
                    release: false,
                },
            }]));
        }
    }
    fn inject_refusal(&mut self, procs: &mut [Proc]) {
        if let crate::stories::Story::Refusals(reason) = self.story {
            let ending = matches!(reason, crate::stories::MessageRefusal::Ending);
            if ending && !self.injected {
                for proc in procs.iter_mut() {
                    if let Proc::Worker(host) = proc {
                        host.relay.until = Some(Time::from_nanos(1_000_000_000));
                        host.relay.wait_only = true;
                    }
                }
            }
            let ready = procs.iter().any(|proc| matches!(proc, Proc::Engine(host) if host.wire.frames.iter().any(|(_, kind, _)| *kind == if ending { 0x0101 } else { 0x0180 })));
            if !self.injected && ready {
                self.injected = true;
                let count = 1;
                self.party.extend(
                    (0..count)
                        .map(|at| parties::Act::Request {
                            key: [at + 2; 16],
                            ask: parties::Ask::Say {
                                task: parties::Task::Last,
                                words: if matches!(
                                    reason,
                                    crate::stories::MessageRefusal::TooLarge | crate::stories::MessageRefusal::Replay
                                ) {
                                    vec![b'x'; 256].into()
                                } else {
                                    b"From the person".as_slice().into()
                                },
                                release: false,
                            },
                        })
                        .collect(),
                );
            }
        }
    }
}

impl Referee {
    fn drain_engine(&mut self, procs: &mut [Proc]) {
        if !matches!(self.story, crate::stories::Story::Drain) {
            return;
        }
        for proc in procs {
            if let Proc::Engine(host) = proc {
                if !self.injected && crate::world::pending_turn(&self.store.borrow()) {
                    self.injected = true;
                    host.service.shutdown();
                }
                if self.injected && !self.views.borrow().engine_drained && host.service.is_empty() {
                    assert_eq!(host.service.end(), Some(&jig_test_service::End::Drained));
                    let mut views = self.views.borrow_mut();
                    views.engine_drained = true;
                    views.commits_waiting_at_drain = self.store.borrow().pending.len();
                    assert_eq!(views.commits_waiting_at_drain, 0);
                    host.service = jig_test_service::startup(
                        &crate::fixtures::engine_document(),
                        &crate::fixtures::files(false),
                        None,
                        &crate::fixtures::seeds(),
                        None,
                        None,
                    )
                    .expect("restart after drained");
                    host.revision.0 += 1_000_000;
                }
            }
        }
    }
}
