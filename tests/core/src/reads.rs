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
    joined_row(store, value, task, related)
}

fn joined_row(
    store: &Store,
    value: &core::Record,
    task: &tasks::TaskRecord,
    related: Option<core::Record>,
) -> reads::Row {
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
    reads::Row {
        value: value.clone(),
        owner: Some(Box::new(task.clone())),
        ancestors: ancestors.into(),
        related,
        linked: linked.into_values().collect(),
        inbox: None,
    }
}

fn conversation_row(store: &Store, range: &reads::Range, value: &core::Record) -> Option<reads::Row> {
    let conversation = match value {
        core::Record::Tasks(tasks::Stored::ConversationNewest(row)) if range.backwards => row,
        core::Record::Tasks(tasks::Stored::ConversationOldest(row)) if !range.backwards => row,
        core::Record::Core(_) | core::Record::Tasks(_) | core::Record::People(_) | core::Record::Notes(_) => {
            return None;
        }
    };
    if conversation.party != range.party {
        return None;
    }
    Some(row(store, value, owner(store, conversation.task)?))
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
            let core::Record::Tasks(tasks::Stored::Update(update)) = value else { return None };
            if update.party != range.party {
                return None;
            }
            let Some(task) = owner(store, update.task) else {
                return Some(reads::Row {
                    value: value.clone(),
                    owner: None,
                    ancestors: Box::new([]),
                    related: None,
                    linked: Box::new([]),
                    inbox: None,
                });
            };
            let mut value = row(store, value, task);
            let object = match update.object {
                tasks::UpdateObject::Result => documents::InboxObject::Result { position: task.result_position },
                tasks::UpdateObject::Reply { message, activity } => {
                    let record = store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Activity {
                        task: update.task,
                        position: activity.value(),
                    })))?;
                    let root::Record::Core(line) = record else { return None };
                    value.related = Some(line.clone());
                    documents::InboxObject::Reply { message: message.value() }
                }
            };
            value.inbox = Some(documents::Cursor::Inbox { task: task.number, object, at: Some(update.at) });
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
        reads::Scope::Conversations => conversation_row(store, range, value),
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

fn get(store: &Store, key: core::Key) -> Option<&core::Record> {
    match store.rows.get(&root::Key::Core(key))? {
        root::Record::Core(record) => Some(record),
        root::Record::Connector { .. } => None,
    }
}

fn point(store: &Store, scope: &reads::Scope) -> Option<reads::Row> {
    let (number, key, related) = match scope {
        reads::Scope::Question { task, message } => {
            let task_row = owner(store, *task)?;
            let line = task_row.question_lines.iter().find(|line| line.message == *message)?;
            (
                *task,
                core::Key::Tasks(tasks::Key::Activity { task: *task, position: line.line }),
                line.answer.map(|answer| {
                    core::Key::Tasks(tasks::Key::Activity { task: answer.task, position: answer.position })
                }),
            )
        }
        reads::Scope::Escalation { task } => {
            let task_row = owner(store, *task)?;
            (
                *task,
                core::Key::Core(core::CoreKey::EscalationDecision {
                    task: *task,
                    revision: task_row.escalation.revision(),
                }),
                Some(core::Key::Tasks(tasks::Key::Milestone { task: *task, position: task_row.hold })),
            )
        }
        reads::Scope::Proposal { proposer: documents::Proposer::Task(task), proposal } => {
            let task_row = owner(store, *task)?;
            let hint = task_row.proposal_rows.iter().find(|row| row.proposal == *proposal)?;
            (*task, core::Key::Tasks(tasks::Key::History { task: *task, revision: hint.revision }), None)
        }
        reads::Scope::Task { .. }
        | reads::Scope::Proposal { proposer: documents::Proposer::Party(_), .. }
        | reads::Scope::PersonTask { .. }
        | reads::Scope::Result { .. }
        | reads::Scope::Reply { .. }
        | reads::Scope::Turn { .. }
        | reads::Scope::EndedDelegates { .. }
        | reads::Scope::History { .. }
        | reads::Scope::Activity { .. }
        | reads::Scope::Inbox { .. }
        | reads::Scope::Conversations
        | reads::Scope::Goals { .. }
        | reads::Scope::Notes { .. }
        | reads::Scope::ScopedNotes { .. }
        | reads::Scope::Spend { .. } => return None,
    };
    Some(joined_row(store, get(store, key)?, owner(store, number)?, related.and_then(|key| get(store, key)).cloned()))
}

fn single_page(store: &Store, range: &reads::Range) -> reads::Loaded {
    let Some(row) = point(store, &range.scope) else { return reads::Loaded::Gone };
    let key = reads::row_key(&row).expect("single row has a key");
    if let Some(after) = &range.after
        && if range.backwards { key >= *after } else { key <= *after }
    {
        return reads::Loaded::Page { rows: Box::new([]), next: None };
    }
    if range.rows == 0 || row.owned_bytes().expect("single row pricing") > range.bytes {
        return reads::Loaded::Busy;
    }
    reads::Loaded::Page { rows: Box::new([row]), next: None }
}

/// The fake transport selects durable owner rows and echoes opaque continuations.
#[must_use]
pub fn page(store: &Store, range: &reads::Range) -> reads::Loaded {
    if matches!(
        range.scope,
        reads::Scope::Question { .. }
            | reads::Scope::Escalation { .. }
            | reads::Scope::Proposal { proposer: documents::Proposer::Task(_), .. }
    ) {
        return single_page(store, range);
    }
    let mut ordered = BTreeMap::new();
    let bounds = index_bounds(range);
    let bounds = match bounds {
        Some((first, last)) => {
            let first = match &range.after {
                Some(reads::Position::Owner(key))
                    if root::Key::Core(key.clone()) >= first && root::Key::Core(key.clone()) <= last =>
                {
                    std::ops::Bound::Excluded(root::Key::Core(key.clone()))
                }
                Some(reads::Position::Owner(_) | reads::Position::Inbox { .. }) | None => {
                    std::ops::Bound::Included(first)
                }
            };
            (first, std::ops::Bound::Included(last))
        }
        None => (std::ops::Bound::Unbounded, std::ops::Bound::Unbounded),
    };
    let records = store.rows.range(bounds).map(|(_, value)| value);
    for record in records {
        let root::Record::Core(value) = record else { continue };
        if let Some(row) = candidate(store, range, value) {
            ordered.insert(reads::row_key(&row).expect("owner-addressed row"), row);
        }
    }
    let mut candidates: Vec<_> = ordered
        .into_iter()
        .filter(|(position, _)| match &range.after {
            Some(after) => {
                if range.backwards && range.scope != reads::Scope::Conversations {
                    position < after
                } else {
                    position > after
                }
            }
            None => true,
        })
        .collect();
    if range.backwards && range.scope != reads::Scope::Conversations {
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

fn index_bounds(range: &reads::Range) -> Option<(root::Key, root::Key)> {
    match range.scope {
        reads::Scope::Inbox {
            group: reads::Group::Updates | reads::Group::Results | reads::Group::Replies, ..
        } => {
            let key = |at, task, object| {
                root::Key::Core(core::Key::Tasks(tasks::Key::Update {
                    party: range.party,
                    at: tasks::Newest::from_raw(at),
                    task: tasks::Newest::from_raw(task),
                    object,
                }))
            };
            Some((
                key(0, 0, tasks::UpdateObject::Result),
                key(
                    u64::MAX,
                    u64::MAX,
                    tasks::UpdateObject::Reply {
                        message: tasks::Newest::from_raw(u64::MAX),
                        activity: tasks::Newest::from_raw(u64::MAX),
                    },
                ),
            ))
        }
        reads::Scope::Conversations => {
            let key = |number| {
                root::Key::Core(core::Key::Tasks(if range.backwards {
                    tasks::Key::ConversationNewest { party: range.party, task: tasks::Newest::from_raw(number) }
                } else {
                    tasks::Key::ConversationOldest { party: range.party, task: number }
                }))
            };
            Some((key(0), key(u64::MAX)))
        }
        reads::Scope::Task { .. }
        | reads::Scope::Proposal { .. }
        | reads::Scope::Escalation { .. }
        | reads::Scope::Question { .. }
        | reads::Scope::PersonTask { .. }
        | reads::Scope::Result { .. }
        | reads::Scope::Reply { .. }
        | reads::Scope::Turn { .. }
        | reads::Scope::EndedDelegates { .. }
        | reads::Scope::History { .. }
        | reads::Scope::Activity { .. }
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
        | reads::Scope::Notes { .. }
        | reads::Scope::ScopedNotes { .. }
        | reads::Scope::Spend { .. } => None,
    }
}
