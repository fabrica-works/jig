//! One root decision per input, with store terminals taken before decisions (service.md, 4).
use crate::{End, PartyOutput, StoreAnswer, StoreRequest, route, stop, system::System};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_test_domain as root;
use skein_lib::{Duration, Env, Queue, Time, Wall};

#[expect(missing_debug_implementations, reason = "protocol account children hold credentials")]
pub struct Engine {
    pub(crate) domain: root::Domain,
    pub(crate) accounts: Option<crate::accounts::Accounts>,
    pub(crate) workers: Option<crate::workers::Workers>,
    pub(crate) values: crate::values::Values,
    pub(crate) tools: crate::tools::Tools,
    aborted: Option<()>,
    pub(crate) env: Env<root::Limits>,
    pub(crate) stores: Queue<StoreRequest>,
    pub(crate) answers: Queue<StoreAnswer>,
    pub(crate) parties: Queue<core::Event>,
    pub(crate) outputs: Queue<PartyOutput>,
    pub(crate) events: Queue<root::Event>,
    pub(crate) requests: Queue<root::Request>,
    pub(crate) systems: [System; 2],
    pub(crate) end: Option<End>,
    pub(crate) begin: bool,
    pub(crate) max_out: u32,
    pub(crate) restart_reads: Option<(u16, u32)>,
    pub(crate) drain: Option<stop::Drain>,
    blocked: bool,
    pending: Option<Box<root::Event>>,
    ready: bool,
}

impl Engine {
    pub(crate) fn new(
        config: root::Config,
        limits: &root::Limits,
        systems: [System; 2],
        accounts: Option<crate::accounts::Accounts>,
        workers: Option<crate::workers::Workers>,
        values: crate::values::Values,
    ) -> Self {
        let max_out = root::max_out(limits);
        let capacity = max_out.checked_mul(2).expect("engine queues fit u32");
        let mut events = Queue::with_capacity(
            capacity.checked_mul(route_out(accounts.as_ref(), workers.as_ref())).expect("translated event room"),
        );
        if accounts.is_some() || workers.is_some() {
            events.push(root::Event::Credential { number: 1, generation: 1, valid: Duration::from_nanos(u64::MAX) });
        }
        Self {
            domain: root::Domain::new(config, limits),
            accounts,
            workers,
            values,
            tools: crate::tools::Tools::new(&limits.core),
            aborted: None,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
            stores: Queue::with_capacity(capacity),
            answers: Queue::with_capacity(capacity),
            parties: Queue::with_capacity(capacity),
            outputs: Queue::with_capacity(capacity),
            events,
            requests: Queue::with_capacity(max_out),
            systems,
            end: None,
            begin: true,
            max_out,
            restart_reads: None,
            drain: None,
            blocked: false,
            pending: None,
            ready: true,
        }
    }
    pub const fn store_requests(&mut self) -> &mut Queue<StoreRequest> {
        &mut self.stores
    }
    pub const fn store_answers(&mut self) -> &mut Queue<StoreAnswer> {
        &mut self.answers
    }
    pub const fn parties_in(&mut self) -> &mut Queue<core::Event> {
        &mut self.parties
    }
    pub const fn parties_out(&mut self) -> &mut Queue<PartyOutput> {
        &mut self.outputs
    }

    pub(crate) fn room(&self) -> bool {
        (match &self.drain {
            Some(drain) => drain.busy.room() >= self.max_out,
            None => true,
        }) && self.stores.room() >= self.max_out
            && self.outputs.room() >= self.max_out
            && self.events.room()
                >= self
                    .max_out
                    .checked_mul(route_out(self.accounts.as_ref(), self.workers.as_ref()))
                    .expect("root translated outputs")
            && match &self.workers {
                Some(workers) => workers.batch_room(self.max_out),
                None => true,
            }
            && match &self.accounts {
                Some(accounts) => accounts.batch_room(self.max_out),
                None => true,
            }
    }
    pub(crate) fn step(&mut self, event: root::Event) {
        if let Err(event) = root::try_step(&mut self.domain, &self.env, event) {
            self.pending = Some(Box::new(event));
            self.blocked = true;
        }
    }

    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        if self.end.is_some() {
            let accounts = match &self.accounts {
                Some(accounts) => accounts.next_deadline(),
                None => None,
            };
            let workers = match &self.workers {
                Some(workers) => workers.next_deadline(),
                None => None,
            };
            return crate::accounts::earliest(accounts, workers);
        }
        let mut next = self.values.accounts.next_expiry();
        for (_, deadline) in self.domain.core_deadlines() {
            if let Some(deadline) = deadline {
                next = Some(match next {
                    Some(previous) => deadline.min(previous),
                    None => deadline,
                });
            }
        }
        if !self.domain.stopping() {
            for (_, deadline) in self.domain.connector_deadlines() {
                next = wall_deadline(next, deadline, &self.env);
            }
            next = wall_deadline(next, self.domain.effect_deadline(), &self.env);
        }
        if let Some(accounts) = &self.accounts {
            next = crate::accounts::earliest(next, accounts.next_deadline());
        }
        if let Some(workers) = &self.workers {
            next = crate::accounts::earliest(next, workers.next_deadline());
        }
        next
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        let components = match &self.accounts {
            Some(accounts) => accounts.work_pending(now),
            None => false,
        } || match &self.workers {
            Some(workers) => workers.work_pending(now),
            None => false,
        };
        if self.end.is_some() || (self.begin && !self.listeners_open()) {
            return components;
        }
        let due = match self.next_deadline() {
            Some(deadline) => deadline <= now,
            None => false,
        };
        components
            || !self.answers.is_empty()
            || (self.room()
                && (self.begin
                    || !self.parties.is_empty()
                    || ((!self.events.is_empty() || self.pending.is_some()) && !self.blocked)
                    || self.ready
                    || (due && !self.blocked)))
    }
    pub(crate) fn io_room(&self) -> bool {
        self.events.room() >= jig_protocol_engine_workers::MAX_OUT_DOWN
            && match &self.workers {
                Some(workers) => workers.room(),
                None => true,
            }
            && match &self.accounts {
                Some(accounts) => accounts.up_room(),
                None => true,
            }
    }
    pub(crate) fn listeners_open(&self) -> bool {
        match &self.workers {
            Some(workers) => workers.opened(),
            None => true,
        }
    }
    pub(crate) fn abort_if_ended(&mut self) {
        match &self.end {
            Some(End::Aborted | End::Stopped(_)) => self.abort_components(),
            Some(End::Drained) | None => {}
        }
    }
    pub(crate) fn abort_components(&mut self) {
        if self.aborted.is_none() {
            self.aborted = Some(());
            if let Some(accounts) = &mut self.accounts {
                accounts.abort();
            }
            if let Some(workers) = &mut self.workers {
                workers.abort(self.env.now, &mut self.events);
            }
        }
    }
}

fn wall_deadline(next: Option<Time>, deadline: Option<Wall>, env: &Env<root::Limits>) -> Option<Time> {
    let Some(deadline) = deadline else {
        return next;
    };
    let span = Duration::from_nanos(deadline.as_nanos().saturating_sub(env.wall.as_nanos()));
    let time = env.now.saturating_add(span);
    Some(match next {
        Some(previous) => time.min(previous),
        None => time,
    })
}

pub fn iterate(engine: &mut Engine, now: Time, wall: Wall) {
    engine.env.now = now;
    engine.env.wall = wall;
    if let Some(accounts) = &mut engine.accounts {
        accounts.iterate(now, wall, &mut engine.values, &mut engine.events);
    }
    if let Some(workers) = &mut engine.workers {
        workers.iterate(now, &mut engine.events);
    }
    stop::advance(engine);
    if engine.end.is_some() {
        engine.abort_if_ended();
        for _ in 0..engine.answers.len() {
            let _answer = engine.answers.pop().expect("late store terminal");
        }
        return;
    }
    for _ in 0..engine.answers.len() {
        let answer = engine.answers.pop().expect("store answer count");
        engine.blocked = false;
        store_answer(engine, answer);
        if engine.end.is_some() {
            return;
        }
    }
    if !engine.room() || (engine.begin && !engine.listeners_open()) {
        return;
    }
    if engine.begin {
        engine.begin = false;
        engine.step(root::Event::RestartBegin);
    } else if !engine.blocked && engine.pending.is_some() {
        let event = engine.pending.take().expect("held component input");
        engine.step(*event);
    } else if !engine.blocked && !engine.events.is_empty() {
        let event = engine.events.pop().expect("ready component input");
        engine.step(event);
    } else if let Some(event) = engine.parties.pop() {
        if let Err(event) = root::try_step(&mut engine.domain, &engine.env, root::Event::Core(event)) {
            engine.outputs.push(PartyOutput::Busy(Box::new(event)));
        }
    } else if !engine.blocked {
        fire(engine);
    }
    engine.ready = true;
    let mut empty = 0_u32;
    for _ in 0..engine.env.limits.routes {
        if !engine.room() || engine.end.is_some() {
            break;
        }
        root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        if !engine.requests.is_empty() || engine.domain.quiescent() {
            engine.blocked = false;
        }
        if engine.requests.is_empty() {
            empty = empty.checked_add(1).expect("two empty passes");
        } else {
            empty = 0;
        }
        for _ in 0..engine.requests.len() {
            let request = engine.requests.pop().expect("bounded root output");
            route::request(engine, request);
            if engine.end.is_some() {
                break;
            }
        }
        if empty == 2 {
            engine.ready = false;
            break;
        }
    }
    engine.domain.reclaim();
    stop::advance(engine);
}

fn store_answer(engine: &mut Engine, answer: StoreAnswer) {
    match answer {
        StoreAnswer::Committed { number, fill } => engine.step(root::Event::Committed { number, fill }),
        StoreAnswer::Failed { number } => stop::failed(engine, number),
        StoreAnswer::Core(event) => engine.step(root::Event::Core(*event)),
        StoreAnswer::Transcript { task, rows, done } => engine.step(root::Event::TranscriptLoaded { task, rows, done }),
        StoreAnswer::Restored { step, rows } => {
            for row in rows {
                if !root::restore_record(&mut engine.domain, &engine.env, row) {
                    stop::invalid_restore(engine);
                    return;
                }
            }
            if step == core::RestartStep::LoadCore {
                engine.events.push(root::Event::Core(core::Event::People(jig_domain_people::Event::Restored)));
            }
            engine.events.push(root::Event::RestartDone(step));
        }
    }
}

fn fire(engine: &mut Engine) {
    for (timer, deadline) in engine.domain.core_deadlines() {
        if let Some(deadline) = deadline
            && deadline <= engine.env.now
        {
            engine.step(root::Event::Timer(timer));
            return;
        }
    }
    if engine.domain.stopping() {
        return;
    }
    for (number, deadline) in engine.domain.connector_deadlines() {
        if let Some(deadline) = deadline
            && deadline <= engine.env.wall
        {
            engine.step(root::Event::ConnectorTimer { number });
            return;
        }
    }
    if let Some(deadline) = engine.domain.effect_deadline()
        && deadline <= engine.env.wall
    {
        engine.step(root::Event::Core(core::Event::EffectDeadline));
    }
}

fn route_out(accounts: Option<&crate::accounts::Accounts>, workers: Option<&crate::workers::Workers>) -> u32 {
    if workers.is_some() {
        jig_protocol_engine_workers::MAX_OUT_DOWN
    } else if accounts.is_some() {
        skein_oauth_accounts::MAX_OUT_DOWN.above
    } else {
        1
    }
}
