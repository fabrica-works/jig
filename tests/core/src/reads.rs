//! Read receipts observed at the root/store boundary (`contract.md`, section 6).
//! Parties know the last submitted commit and the fake store's durable number.
//! They never inspect the core or the journal. Their receipts survive a crash.
use crate::{Store, effects::World};
use jig_domain_core::{self as core, documents, reads};
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};
use std::collections::{BTreeMap, VecDeque};

/// Independent prerequisite for an admitted read's one answer.
#[derive(Debug)]
pub struct Fence(pub u64);

impl Fence {
    /// Reject a terminal observed before the preceding decision was durable.
    ///
    /// # Errors
    /// Returns an error if the store has not applied the prerequisite.
    pub fn answered(&self, durable: u64) -> Result<(), &'static str> {
        if durable < self.0 { Err("read preceded its durable prerequisite") } else { Ok(()) }
    }

    /// A crash preserves every commit whose state a party already read.
    ///
    /// # Errors
    /// Returns an error if restart lost the party's durable prerequisite.
    pub fn restarted(&self, durable: u64) -> Result<(), &'static str> {
        if durable < self.0 { Err("crash retracted an observed read") } else { Ok(()) }
    }
}

/// Scripted parties and their ordinary Rust owner-range store adapter.
#[derive(Debug)]
pub struct Parties {
    made: u64,
    awaiting: BTreeMap<u64, Fence>,
    admissions: VecDeque<u64>,
    queued: VecDeque<u64>,
    flights: BTreeMap<u64, u64>,
    pub loads: Vec<(Token, reads::Range)>,
    pub hold_loads: bool,
    pub receipts: Vec<(u64, u64, reads::Answer)>,
    pub application_receipts:
        Vec<(u64, u64, u16, Result<jig_test_connector::ClientDocument, jig_test_connector::DocumentMissing>)>,
}

impl Parties {
    #[must_use]
    pub fn new(made: u64) -> Self {
        Self {
            made,
            awaiting: BTreeMap::new(),
            admissions: VecDeque::new(),
            queued: VecDeque::new(),
            flights: BTreeMap::new(),
            loads: Vec::new(),
            hold_loads: false,
            receipts: Vec::new(),
            application_receipts: Vec::new(),
        }
    }

    pub fn inbound(&mut self, event: &root::Event) {
        match event {
            root::Event::Core(core::Event::Read { .. }) | root::Event::ReadApplication { .. } => {
                let reply = self.queued.pop_front().expect("the independent caller issued this read");
                assert!(self.awaiting.insert(reply, Fence(self.made)).is_none());
                self.admissions.push_back(reply);
            }
            root::Event::Core(core::Event::ReadLoaded { load, .. }) => {
                let reply = self.flights.remove(&load.raw()).expect("one terminal for the observed load");
                self.awaiting.get_mut(&reply).expect("admitted read").0 = self.made;
            }
            root::Event::Core(_)
            | root::Event::WatchApplication { .. }
            | root::Event::Application { .. }
            | root::Event::Turn { .. }
            | root::Event::Text { .. }
            | root::Event::Answer { .. }
            | root::Event::WriterRead { .. }
            | root::Event::Timer(_)
            | root::Event::Committed { .. }
            | root::Event::Failed { .. }
            | root::Event::Connector { .. }
            | root::Event::Stop
            | root::Event::RestartBegin
            | root::Event::RestartDone(_)
            | root::Event::ConnectorTimer { .. }
            | root::Event::ProjectionEffect { .. }
            | root::Event::EffectCall { .. }
            | root::Event::CallAnswered { .. }
            | root::Event::TranscriptLoaded { .. } => {}
        }
    }

    pub fn commit(&mut self, number: u64) {
        self.made = number;
    }

    pub fn replied(&mut self, reply: ReplyTo, answer: reads::Answer, store: &Store) {
        let reply = reply.into_token().raw();
        let fence = self.awaiting.remove(&reply).expect("one answer for the admitted read");
        fence.answered(store.applied).expect("read follows every prior commit");
        if self.admissions.front() == Some(&reply) {
            self.admissions.pop_front();
        }
        self.receipts.push((reply, fence.0, answer));
    }

    pub fn application_replied(
        &mut self,
        reply: ReplyTo,
        number: u16,
        answer: Result<jig_test_connector::ClientDocument, jig_test_connector::DocumentMissing>,
        store: &Store,
    ) {
        let reply = reply.into_token().raw();
        let fence = self.awaiting.remove(&reply).expect("one application read terminal");
        fence.answered(store.applied).expect("application read follows every prior commit");
        if self.admissions.front() == Some(&reply) {
            self.admissions.pop_front();
        }
        self.application_receipts.push((reply, fence.0, number, answer));
    }

    pub fn load(&mut self, load: Token, range: reads::Range, store: &Store) -> Vec<root::Event> {
        let reply = self.admissions.pop_front().expect("one owner load for an admitted read");
        self.awaiting
            .get(&reply)
            .expect("admitted read")
            .answered(store.applied)
            .expect("load follows the admission's durable fence");
        assert!(self.flights.insert(load.raw(), reply).is_none());
        self.loads.push((load, range));
        if self.hold_loads { Vec::new() } else { self.complete(store) }
    }

    pub fn complete(&mut self, store: &Store) -> Vec<root::Event> {
        std::mem::take(&mut self.loads)
            .into_iter()
            .map(|(load, range)| root::Event::Core(core::Event::ReadLoaded { load, result: page(store, &range) }))
            .collect()
    }

    #[must_use]
    pub fn paused(&self) -> bool {
        self.hold_loads && !self.loads.is_empty()
    }

    pub fn restart(&mut self, durable: u64) {
        for (_, prerequisite, _) in &self.receipts {
            Fence(*prerequisite).restarted(durable).expect("a crash preserves every observed read");
        }
        for (_, prerequisite, _, _) in &self.application_receipts {
            Fence(*prerequisite).restarted(durable).expect("crash preserves every observed connector read");
        }
        self.awaiting.clear();
        self.admissions.clear();
        self.queued.clear();
        self.flights.clear();
        self.loads.clear();
        self.made = durable;
    }
}

fn owner(store: &Store, number: u64) -> Option<&tasks::TaskRecord> {
    for kind in [tasks::Key::Live(number), tasks::Key::Ended(number)] {
        if let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(value) | tasks::Stored::Ended(value)))) =
            store.rows.get(&root::Key::Core(core::Key::Tasks(kind)))
        {
            return Some(value);
        }
    }
    None
}

fn row(store: &Store, value: &core::Record, task: &tasks::TaskRecord) -> reads::Row {
    let mut ancestors = Vec::new();
    let mut linked = BTreeMap::new();
    let mut requester = task.requester;
    for _ in 0..64 {
        let tasks::Party::Task(number) = requester else { break };
        let Some(parent) = owner(store, number) else { break };
        ancestors.push(number);
        linked.insert(number, Box::new(parent.clone()));
        requester = parent.requester;
    }
    for number in task
        .dependencies
        .iter()
        .chain(task.spec.inputs.iter())
        .chain(task.references.iter())
        .chain(task.delegates.iter())
        .chain(task.result_followups.iter())
    {
        if let Some(value) = owner(store, *number) {
            linked.insert(*number, Box::new(value.clone()));
        }
    }
    let related_key = match value {
        core::Record::Tasks(tasks::Stored::Activity(tasks::ActivityLine {
            line: tasks::Line::Turn { attempt, turn, .. },
            ..
        })) => core::CoreKey::Turn { task: task.number, attempt: *attempt, turn: *turn },
        core::Record::Core(_) | core::Record::Tasks(_) | core::Record::People(_) | core::Record::Notes(_) => {
            core::CoreKey::Terminal { task: task.number, attempt: task.attempt }
        }
    };
    let related = store.rows.get(&root::Key::Core(core::Key::Core(related_key))).and_then(|record| match record {
        root::Record::Core(value) => Some(value.clone()),
        root::Record::Connector { .. } => None,
    });
    reads::Row {
        value: value.clone(),
        owner: Some(Box::new(task.clone())),
        ancestors: ancestors.into(),
        related,
        linked: linked.into_values().collect(),
        inbox: None,
    }
}

fn conversation_row(store: &Store, value: &core::Record) -> Option<reads::Row> {
    let core::Record::Tasks(tasks::Stored::Ended(owner)) = value else { return None };
    Some(row(store, value, owner))
}

fn candidate(store: &Store, range: &reads::Range, value: &core::Record) -> Option<reads::Row> {
    match &range.scope {
        reads::Scope::Task { task } | reads::Scope::Result { task } | reads::Scope::PersonTask { task } => {
            let core::Record::Tasks(tasks::Stored::Ended(value)) = value else { return None };
            if value.number != *task {
                return None;
            }
            Some(row(store, &core::Record::Tasks(tasks::Stored::Ended(value.clone())), value))
        }
        reads::Scope::Activity { task } => {
            let core::Record::Tasks(tasks::Stored::Activity(line)) = value else { return None };
            if line.task != *task {
                return None;
            }
            Some(row(store, value, owner(store, *task)?))
        }
        reads::Scope::Reply { task, activity } => {
            let core::Record::Tasks(tasks::Stored::Activity(line)) = value else { return None };
            if line.task != *task || line.position != *activity {
                return None;
            }
            Some(row(store, value, owner(store, *task)?))
        }
        reads::Scope::Inbox {
            group: reads::Group::Updates | reads::Group::Results | reads::Group::Replies, ..
        } => {
            let (task, object, at) = match value {
                core::Record::Tasks(tasks::Stored::Ended(task)) => {
                    if task.requester != tasks::Party::Person(range.party) || task.result_position == 0 {
                        return None;
                    }
                    (task.as_ref(), documents::InboxObject::Result { position: task.result_position }, task.ended_at)
                }
                core::Record::Tasks(tasks::Stored::Activity(line)) => {
                    let task = owner(store, line.task)?;
                    let message = match &line.line {
                        tasks::Line::Turn { reply, .. } if task.requester == tasks::Party::Person(range.party) => {
                            *reply
                        }
                        tasks::Line::Notified { recipient: tasks::Party::Person(party), word }
                            if *party == range.party =>
                        {
                            word.number
                        }
                        tasks::Line::Words { .. }
                        | tasks::Line::Heard(_)
                        | tasks::Line::Notified { .. }
                        | tasks::Line::RunBegan { .. }
                        | tasks::Line::RunEnded { .. }
                        | tasks::Line::Turn { .. }
                        | tasks::Line::Decided { .. }
                        | tasks::Line::Milestone { .. } => return None,
                    };
                    (task, documents::InboxObject::Reply { message }, Some(line.at))
                }
                core::Record::Core(_) | core::Record::Tasks(_) | core::Record::People(_) | core::Record::Notes(_) => {
                    return None;
                }
            };
            let mut value = row(store, value, task);
            value.inbox = Some(documents::Cursor::Inbox { task: task.number, object, at });
            Some(value)
        }
        reads::Scope::Notes { .. } | reads::Scope::ScopedNotes { .. } => {
            let core::Record::Notes(jig_domain_notes::Record::Entry(_)) = value else { return None };
            Some(reads::Row {
                value: value.clone(),
                owner: None,
                ancestors: Box::new([]),
                related: None,
                linked: Box::new([]),
                inbox: None,
            })
        }
        reads::Scope::Conversations => conversation_row(store, value),
        reads::Scope::Turn { task, attempt, turn } => {
            let core::Record::Core(core::CoreRecord::Turn(line)) = value else { return None };
            if line.task != *task || line.attempt != *attempt || line.turn != *turn {
                return None;
            }
            Some(row(store, value, owner(store, *task)?))
        }
        reads::Scope::Proposal { .. }
        | reads::Scope::Escalation { .. }
        | reads::Scope::Question { .. }
        | reads::Scope::EndedDelegates { .. }
        | reads::Scope::History { .. }
        | reads::Scope::Inbox {
            group:
                reads::Group::Needs
                | reads::Group::Proposals
                | reads::Group::Escalations
                | reads::Group::Questions
                | reads::Group::PersonTasks,
            ..
        }
        | reads::Scope::Goals { .. }
        | reads::Scope::Spend { .. } => None,
    }
}

/// The fake transport selects durable owner rows and echoes opaque continuations.
#[must_use]
pub fn page(store: &Store, range: &reads::Range) -> reads::Loaded {
    let mut ordered = BTreeMap::new();
    for record in store.rows.values() {
        let root::Record::Core(value) = record else { continue };
        if let Some(row) = candidate(store, range, value) {
            ordered.insert(reads::row_key(&row).expect("owner-addressed row"), row);
        }
    }
    let mut candidates: Vec<_> = ordered
        .into_iter()
        .filter(|(position, _)| match &range.after {
            Some(after) => {
                if range.backwards {
                    position < after
                } else {
                    position > after
                }
            }
            None => true,
        })
        .collect();
    if range.backwards {
        candidates.reverse();
    }
    let mut rows = Vec::new();
    let mut bytes = 0;
    let mut last = None;
    let mut more = false;
    for (key, row) in candidates {
        let size = row.owned_bytes().expect("owner row pricing");
        if rows.len() == range.rows as usize || size > range.bytes - bytes {
            if rows.is_empty() {
                return reads::Loaded::Busy;
            }
            more = true;
            break;
        }
        bytes += size;
        last = Some(key);
        rows.push(row);
    }
    reads::Loaded::Page { rows: rows.into(), next: if more { last } else { None } }
}

pub fn ask(world: &mut World, number: u64, read: reads::Read) {
    world.reads.as_mut().expect("read parties").queued.push_back(20_000 + number);
    world.send(root::Event::Core(core::Event::Read {
        reply: ReplyTo::new(Token::new(20_000 + number)),
        sign_in: world.sign_in.expect("signed in"),
        read,
    }));
}

pub fn application(
    world: &mut World,
    number: u64,
    scope: jig_domain_core::documents::ApplicationScope,
    connector: u16,
    subject: jig_test_connector::ClientSubject,
) {
    world.reads.as_mut().expect("read parties").queued.push_back(20_000 + number);
    world.send(root::Event::ReadApplication {
        reply: ReplyTo::new(Token::new(20_000 + number)),
        sign_in: world.sign_in.expect("signed in"),
        scope,
        number: connector,
        subject,
    });
}

#[must_use]
pub fn latest(world: &World) -> &reads::Answer {
    &world.reads.as_ref().expect("read parties").receipts.last().expect("one terminal answer").2
}

pub fn complete(world: &mut World) {
    let events = world.reads.as_mut().expect("read parties").complete(&world.store);
    world.send_all(events);
}
