//! One root decision per input, with store terminals taken before decisions (service.md, 4).
use crate::{End, PartyOutput, StoreAnswer, StoreRequest, route, stop, system::System};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_test_domain as root;
use skein_lib::{Duration, Env, Queue, Time, Wall};

#[derive(Debug)]
pub struct Engine {
    pub(crate) domain: root::Domain,
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
    blocked: bool,
    pending: Option<Box<root::Event>>,
    ready: bool,
}

impl Engine {
    pub(crate) fn new(config: root::Config, limits: &root::Limits, systems: [System; 2]) -> Self {
        let max_out = root::max_out(limits);
        let capacity = max_out.checked_mul(2).expect("engine queues fit u32");
        Self {
            domain: root::Domain::new(config, limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
            stores: Queue::with_capacity(capacity),
            answers: Queue::with_capacity(capacity),
            parties: Queue::with_capacity(capacity),
            outputs: Queue::with_capacity(capacity),
            events: Queue::with_capacity(capacity),
            requests: Queue::with_capacity(max_out),
            systems,
            end: None,
            begin: true,
            max_out,
            restart_reads: None,
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
        self.stores.room() >= self.max_out && self.outputs.room() >= self.max_out && self.events.room() >= self.max_out
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
            return None;
        }
        let mut next = None;
        for (_, deadline) in self.domain.core_deadlines() {
            if let Some(deadline) = deadline {
                next = Some(match next {
                    Some(previous) => deadline.min(previous),
                    None => deadline,
                });
            }
        }
        for (_, deadline) in self.domain.connector_deadlines() {
            next = wall_deadline(next, deadline, &self.env);
        }
        wall_deadline(next, self.domain.effect_deadline(), &self.env)
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        if self.end.is_some() {
            return false;
        }
        let due = match self.next_deadline() {
            Some(deadline) => deadline <= now,
            None => false,
        };
        !self.answers.is_empty()
            || (self.room()
                && (self.begin
                    || !self.parties.is_empty()
                    || ((!self.events.is_empty() || self.pending.is_some()) && !self.blocked)
                    || self.ready
                    || (due && !self.blocked)))
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
    if engine.end.is_some() {
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
    if !engine.room() {
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
