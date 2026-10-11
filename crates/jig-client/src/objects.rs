//! The page's objects, held once by key and named by explicit source sets.
//! A source replacement first preserves objects it still names, then prunes and
//! admits new names in order. Pending and leaving cards survive without sources.
//! Documents carry facts, never authority judgments or phrases computed here.
//! Entry points name, put, reply and replace preserve one card per object;
//! `dropping` and `read_card` obtain terminal facts without ending a pending request.
//! See client/domain/client.md, 5, and client/domain/objects.md, 2–8.
//!
//! | Card | Event | Next | Emits |
//! |---|---|---|---|
//! | open/refused | last source drops it | leaving/looking | terminal read |
//! | deciding | last source drops it | deciding | nothing |
//! | deciding | durable answer | refused or leaving | notice only if hidden |
//! | leaving/looking | terminal document | leaving/ended | linger armed |
//! | leaving | linger expires | gone | nothing |
use crate::{
    contract::{Chip, Ending, Escalation, ObjectKey, Outcome, PersonTask, Proposal, Question, Refusal, Result, Task},
    intents::Progress,
    watches::Watch,
};
use alloc::boxed::Box;
use skein_lib::{Id, List, Map, Set, Slab, Time, Token};
pub mod answer;
pub mod escalation;
pub mod person_task;
pub mod proposal;
pub mod question;
pub mod result;

/// One owner of an object's presence on the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Source {
    /// A watch that names this object.
    Watch(Id<Watch>),
    /// A concrete window of one page generation.
    Window { generation: u32, number: u16 },
    /// A disappearing card reads its terminal facts without naming itself.
    Card(Id<Object>),
}
/// The facts held for an object, with independently moving revisions inside.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Body {
    /// A named task's compact facts.
    Chip(Chip),
    /// The full task document.
    Task(Box<Task>),
    /// Whole historical owner facts for the original object identity.
    Historical(Box<crate::contract::OwnerHistorical>),
    /// A proposal document.
    Proposal(Box<Proposal>),
    OwnerProposal(Box<crate::contract::OwnerProposal>),
    /// A held task escalation.
    Escalation(Box<Escalation>),
    /// A task owner’s whole escalation document.
    OwnerEscalation(Box<crate::contract::OwnerEscalation>),
    /// A question.
    Question(Box<Question>),
    /// A person task.
    PersonTask(Box<PersonTask>),
    /// Pending owner person work.
    OwnerPersonTask(Box<crate::contract::OwnerPersonTask>),
    /// A result.
    Result(Box<Result>),
    /// A note and the routing project of its page.
    Note {
        project: u32,
        document: Box<crate::contract::Note>,
    },
}
/// Why a card leaves, with durable facts preserved for its view.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Left {
    /// Its source dropped it; the terminal document has not arrived.
    Looking,
    /// Its request received a durable outcome.
    Answered { outcome: Outcome },
    /// Its terminal document says how it ended.
    Ended { how: Ending },
    /// Its task owner supplies the complete ending independently of a result document.
    OwnerEnded { how: Box<crate::contract::OwnerChipEnding> },
    /// Its whole historical terminal facts, independent of subsequent body refreshes.
    Historical { facts: Box<crate::contract::OwnerHistorical> },
    /// Its terminal facts are no longer available.
    Unknown,
}
/// A card follows its request until its durable answer.
/// Open/refused + source drop -> leaving/looking + terminal read;
/// deciding + source drop -> deciding; answer -> refused or leaving;
/// leaving + terminal document -> leaving/ended; linger -> removed.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Card {
    /// Available for an offered act.
    Open,
    /// Asked, with the root table's progress.
    Deciding { request: Option<Token>, progress: Progress },
    /// Refused, with facts kept on its card.
    Refused { refusal: Refusal },
    /// Retention ended without an answer; inspect the current object before acting again.
    Unknown { act: crate::contract::ActKind, look: crate::contract::Read },
    /// Ended and still shown until its linger expires.
    Leaving { how: Left, until: Time },
}
/// One object and the sources that still name it.
#[derive(Debug)]
pub struct Object {
    pub key: ObjectKey,
    pub project: Option<u32>,
    pub body: Option<Body>,
    pub sources: Set<Source>,
    pub card: Card,
    /// The last durable answer, also kept when the object remains open.
    pub answer: Option<crate::contract::Answer>,
    /// Whether this card belongs to the current page, even after a source drops it.
    pub shown: bool,
    /// Navigation facts for an unread reply on this chat.
    pub reply: Option<result::Reply>,
}
/// A bounded working set, with spare slab slots for retired objects in a step.
#[derive(Debug)]
pub struct Objects {
    pub(crate) slab: Slab<Object>,
    pub(crate) index: Map<ObjectKey, Id<Object>>,
    pub(crate) sources: u32,
}
impl Objects {
    /// Allocates admission and retirement room from the owner's limits.
    #[must_use]
    pub fn new(capacity: u32, sources: u32) -> Self {
        Self {
            slab: Slab::with_capacity(capacity.checked_mul(2).expect("object capacity is validated")),
            index: Map::with_capacity(capacity),
            sources,
        }
    }
    /// The object named by a current handle.
    #[must_use]
    pub fn get(&self, id: Id<Object>) -> Option<&Object> {
        let object = self.slab.get(id)?;
        if self.index.get(&object.key) == Some(&id) { Some(object) } else { None }
    }
    /// The mutable object named by a current handle.
    #[must_use]
    pub fn get_mut(&mut self, id: Id<Object>) -> Option<&mut Object> {
        let key = self.slab.get(id)?.key;
        if self.index.get(&key) == Some(&id) { self.slab.get_mut(id) } else { None }
    }
    /// Looks up one object without duplicating it.
    #[must_use]
    pub fn find(&self, key: ObjectKey) -> Option<Id<Object>> {
        self.index.get(&key).copied()
    }
    /// Current card handles, including cards waiting for terminal facts or lingering.
    #[must_use]
    pub const fn ids(&self) -> &Map<ObjectKey, Id<Object>> {
        &self.index
    }
    /// Whether a specific source successfully names this object.
    #[must_use]
    pub fn named(&self, key: ObjectKey, source: Source) -> bool {
        let Some(id) = self.find(key) else {
            return false;
        };
        self.slab.get(id).expect("indexed object exists").sources.contains(&source)
    }
    /// The number of live objects held, excluding retired slots.
    #[must_use]
    pub fn len(&self) -> u32 {
        self.index.len()
    }
    /// Whether no live object is held.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
    /// Names an object from a source, refusing before changing a full source set.
    pub fn name(&mut self, key: ObjectKey, source: Source) -> Option<Id<Object>> {
        if let Some(id) = self.find(key) {
            let object = self.slab.get_mut(id).expect("indexed object exists");
            match object.sources.insert(source) {
                Ok(_) => {
                    object.shown = true;
                    match &object.card {
                        Card::Leaving { how: Left::Looking, .. } => object.card = Card::Open,
                        Card::Open
                        | Card::Deciding { .. }
                        | Card::Refused { .. }
                        | Card::Unknown { .. }
                        | Card::Leaving {
                            how:
                                Left::Answered { .. }
                                | Left::Ended { .. }
                                | Left::OwnerEnded { .. }
                                | Left::Historical { .. }
                                | Left::Unknown,
                            ..
                        } => {}
                    }
                    return Some(id);
                }
                Err(_) => return None,
            }
        }
        if self.index.len() == self.index.capacity() {
            return None;
        }
        let mut sources = Set::with_capacity(self.sources);
        match sources.insert(source) {
            Ok(_) => {}
            Err(_) => return None,
        }
        let object = Object {
            key,
            project: None,
            body: None,
            sources,
            card: Card::Open,
            answer: None,
            shown: true,
            reply: None,
        };
        let Ok(id) = self.slab.insert(object) else {
            return None;
        };
        let old = self.index.insert(key, id).expect("index capacity checked before inserting");
        assert!(old.is_none(), "new object key is not indexed");
        Some(id)
    }
    /// Replaces only the facts, preserving one card and its source set.
    pub fn put(&mut self, key: ObjectKey, body: Body) -> bool {
        if !body.matches(key) {
            return false;
        }
        let Some(id) = self.find(key) else {
            return false;
        };
        let object = self.slab.get_mut(id).expect("indexed object exists");
        match &body {
            Body::Historical(value) => {
                let project = match &value.value {
                    crate::contract::OwnerHistoricalValue::Task { document, .. } => Some(document.project),
                    crate::contract::OwnerHistoricalValue::PersonProposal { proposal, .. } => Some(proposal.project),
                    crate::contract::OwnerHistoricalValue::Escalation { task, .. } => Some(task.project),
                    crate::contract::OwnerHistoricalValue::PersonTask { document, .. } => document.project(),
                    crate::contract::OwnerHistoricalValue::Proposal(_)
                    | crate::contract::OwnerHistoricalValue::Question { .. } => None,
                };
                if let Some(project) = project {
                    object.project = Some(project);
                }
            }
            Body::Task(task) => object.project = Some(task.project),
            Body::OwnerEscalation(value) => object.project = Some(value.task.project),
            Body::Proposal(proposal) => object.project = Some(proposal.project),
            Body::OwnerProposal(proposal) => object.project = Some(proposal.project),
            Body::OwnerPersonTask(task) => object.project = task.project(),
            Body::Chip(_) | Body::Escalation(_) | Body::Question(_) | Body::PersonTask(_) | Body::Result(_) => {}
            Body::Note { project, .. } => object.project = Some(*project),
        }
        match body {
            Body::Chip(chip) => match &mut object.body {
                Some(Body::Task(task)) => task.chip = chip,
                Some(Body::Historical(_)) => {}
                Some(
                    Body::Chip(_)
                    | Body::Proposal(_)
                    | Body::OwnerProposal(_)
                    | Body::Escalation(_)
                    | Body::OwnerEscalation(_)
                    | Body::Question(_)
                    | Body::PersonTask(_)
                    | Body::OwnerPersonTask(_)
                    | Body::Result(_)
                    | Body::Note { .. },
                )
                | None => object.body = Some(Body::Chip(chip)),
            },
            Body::Historical(_)
            | Body::Task(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Escalation(_)
            | Body::OwnerEscalation(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. } => object.body = Some(body),
        }
        true
    }
    /// Replaces what a source names, returning how many names could not fit.
    pub fn replace(&mut self, source: Source, keys: &[ObjectKey]) -> u32 {
        for (_, id) in &self.index {
            let object = self.slab.get_mut(*id).expect("indexed object exists");
            object.sources.remove(&source);
        }
        let mut available = self.index.capacity();
        for (_, id) in &self.index {
            let object = self.slab.get(*id).expect("indexed object exists");
            if retained(object) {
                available = available.saturating_sub(1);
            }
        }
        let mut chosen = List::with_capacity(self.index.capacity());
        let mut excess = 0_u32;
        for key in keys {
            if chosen.as_slice().contains(key) {
                continue;
            }
            let reserved = match self.find(*key) {
                Some(id) => retained(self.slab.get(id).expect("indexed object exists")),
                None => false,
            };
            if reserved || available > 0 {
                chosen.push(*key).expect("unique admitted keys fit object capacity");
                if !reserved {
                    available = available.saturating_sub(1);
                }
                if let Some(id) = self.find(*key) {
                    let object = self.slab.get_mut(id).expect("indexed object exists");
                    match object.sources.insert(source) {
                        Ok(_) | Err(_) => {}
                    }
                }
            } else {
                excess = excess.saturating_add(1);
            }
        }
        self.prune();
        for key in &chosen {
            if self.name(*key, source).is_none() {
                excess = excess.saturating_add(1);
            }
        }
        excess
    }
    /// Clears page sources while pending and completed leaving objects remain held.
    pub fn clear(&mut self) {
        for (_, id) in &self.index {
            let object = self.slab.get_mut(*id).expect("indexed object exists");
            object.shown = false;
            match &object.card {
                Card::Leaving { how: Left::Looking, .. } => object.card = Card::Open,
                Card::Open
                | Card::Deciding { .. }
                | Card::Refused { .. }
                | Card::Unknown { .. }
                | Card::Leaving {
                    how:
                        Left::Answered { .. }
                        | Left::Ended { .. }
                        | Left::OwnerEnded { .. }
                        | Left::Historical { .. }
                        | Left::Unknown,
                    ..
                } => {}
            }
        }
        for (_, id) in &self.index {
            let sources = &mut self.slab.get_mut(*id).expect("indexed object exists").sources;
            for _ in 0..sources.len() {
                sources.pop_first();
            }
        }
        self.prune();
    }
    /// Removes one object only after its owner no longer needs its card.
    pub fn remove(&mut self, key: ObjectKey) {
        if let Some(id) = self.index.remove(&key) {
            self.slab.retire(id);
        }
    }
    /// Removes unsourced cards except requests still pending and cards lingering.
    pub fn prune(&mut self) {
        for _ in 0..self.index.len() {
            let mut gone = None;
            for (key, id) in &self.index {
                let object = self.slab.get(*id).expect("indexed object exists");
                if object.sources.is_empty() {
                    match object.card {
                        Card::Unknown { .. } => {
                            if !object.shown {
                                gone = Some(*key);
                                break;
                            }
                        }
                        Card::Open | Card::Refused { .. } => {
                            gone = Some(*key);
                            break;
                        }
                        Card::Deciding { .. } | Card::Leaving { .. } => {}
                    }
                }
            }
            match gone {
                Some(key) => self.remove(key),
                None => break,
            }
        }
    }
    /// Holds a note from its bounded page as a card with its own recalled revision.
    pub fn note(&mut self, project: u32, source: Source, document: crate::contract::Note) -> Option<Id<Object>> {
        let key = ObjectKey::Note { project, note: document.number };
        let id = self.name(key, source)?;
        let stored = self.put(key, Body::Note { project, document: Box::new(document) });
        assert!(stored, "the named note accepts its own document");
        Some(id)
    }
    /// Attaches a chat's reply navigation without replacing its task facts or card.
    pub fn reply(&mut self, source: Source, line: crate::contract::ConversationLine) -> Option<Id<Object>> {
        let key = ObjectKey::Task(line.chip.task);
        let Some(reply) = result::Reply::from_line(line) else {
            if let Some(id) = self.find(key) {
                self.slab.get_mut(id).expect("indexed chat exists").reply = None;
            }
            return None;
        };
        let id = self.name(ObjectKey::Task(reply.chat), source)?;
        self.slab.get_mut(id).expect("named chat exists").reply = Some(reply);
        Some(id)
    }
    /// Frees retired slots at the application's reclaim point.
    pub fn reclaim(&mut self) {
        self.slab.reclaim();
    }
}

fn retained(object: &Object) -> bool {
    if !object.sources.is_empty() {
        return true;
    }
    match object.card {
        Card::Unknown { .. } => object.shown,
        Card::Deciding { .. } | Card::Leaving { .. } => true,
        Card::Open | Card::Refused { .. } => false,
    }
}

/// The working set containers, source sets and document payloads, retired room included.
#[must_use]
pub fn worst_case(limits: &crate::Limits) -> Option<u64> {
    let slots = limits.objects.checked_mul(2)?;
    let payload = Set::<Source>::worst_case(limits.sources)?.checked_add(limits.document_bytes.checked_mul(4)?)?;
    Slab::<Object>::worst_case(slots)?
        .checked_add(Map::<ObjectKey, Id<Object>>::worst_case(limits.objects)?)?
        .checked_add(List::<ObjectKey>::worst_case(limits.objects)?)?
        .checked_add(u64::from(slots).checked_mul(payload)?)
}

/// Whether this answer ends the waiting object rather than steering a live task.
#[must_use]
pub fn stops_waiting(key: ObjectKey, outcome: &Outcome) -> bool {
    match crate::outcome::route(outcome) {
        crate::outcome::Route::Proposed => false,
        crate::outcome::Route::Chat { .. }
        | crate::outcome::Route::Goal { .. }
        | crate::outcome::Route::Words { .. }
        | crate::outcome::Route::Other => match key {
            ObjectKey::Proposal(_)
            | ObjectKey::Escalation(_)
            | ObjectKey::Question { .. }
            | ObjectKey::PersonTask(_) => true,
            ObjectKey::Task(_) | ObjectKey::Result(_) | ObjectKey::Note { .. } => false,
        },
    }
}

fn delivered_name(value: &crate::contract::Delivery, key: ObjectKey) -> Option<bool> {
    use crate::contract::Delivery;
    match value {
        Delivery::OwnerProject(_)
        | Delivery::Frame(_)
        | Delivery::Run(_)
        | Delivery::RunState(_)
        | Delivery::Deployment(_)
        | Delivery::OwnerDeployment(_) => None,
        Delivery::Conversations(chats) => {
            for row in &chats.rows {
                if key == ObjectKey::Task(row.chip.task) {
                    return Some(true);
                }
            }
            Some(false)
        }
        Delivery::Task(task) => Some(key == ObjectKey::Task(task.chip.task)),
        Delivery::OwnerProposal(proposal) => Some(key == ObjectKey::Proposal(proposal.key)),
        Delivery::OwnerInbox(inbox) => {
            for entries in [&inbox.needs, &inbox.updates] {
                for entry in entries {
                    if entry.card_key() == key {
                        return Some(true);
                    }
                }
            }
            Some(false)
        }
        Delivery::Inbox(inbox) => {
            for entry in &inbox.entries {
                if entry.object == key {
                    return Some(true);
                }
            }
            Some(false)
        }
        Delivery::OwnerTree(tree) => {
            if key == ObjectKey::Task(tree.root.task) {
                return Some(true);
            }
            for row in &tree.rows {
                if key == ObjectKey::Task(row.chip.task) {
                    return Some(true);
                }
                for proposal in &row.proposals {
                    if key == ObjectKey::Proposal(*proposal) {
                        return Some(true);
                    }
                }
            }
            Some(false)
        }
        Delivery::Tree(tree) => {
            for node in &tree.nodes {
                if key == ObjectKey::Task(node.chip.task) {
                    return Some(true);
                }
                for proposal in &node.proposals {
                    if key == ObjectKey::Proposal(*proposal) {
                        return Some(true);
                    }
                }
            }
            Some(false)
        }
        Delivery::Chips(chips) => {
            for chip in chips {
                if key == ObjectKey::Task(chip.task) {
                    return Some(true);
                }
            }
            Some(false)
        }
        Delivery::Tasks(tasks) => {
            for row in &tasks.rows {
                if key == ObjectKey::Task(row.chip.task) {
                    return Some(true);
                }
            }
            Some(false)
        }
        Delivery::Goals(goals) => {
            for row in &goals.rows {
                if key == ObjectKey::Task(row.chip.task) {
                    return Some(true);
                }
            }
            Some(false)
        }
    }
}
/// Protects disappearing cards before the source replacement prunes its names.
pub(crate) fn dropping(domain: &mut crate::Domain, watch: Id<Watch>, value: &crate::contract::Delivery) {
    match value {
        crate::contract::Delivery::Conversations(_) => {
            if !domain.conversation_rows.edges.reaches(crate::action::End::Before) {
                return;
            }
        }
        crate::contract::Delivery::Frame(_)
        | crate::contract::Delivery::Run(_)
        | crate::contract::Delivery::RunState(_)
        | crate::contract::Delivery::Deployment(_)
        | crate::contract::Delivery::OwnerDeployment(_)
        | crate::contract::Delivery::Task(_)
        | crate::contract::Delivery::Tree(_)
        | crate::contract::Delivery::OwnerTree(_)
        | crate::contract::Delivery::OwnerProject(_)
        | crate::contract::Delivery::OwnerProposal(_)
        | crate::contract::Delivery::Chips(_)
        | crate::contract::Delivery::Inbox(_)
        | crate::contract::Delivery::OwnerInbox(_)
        | crate::contract::Delivery::Goals(_)
        | crate::contract::Delivery::Tasks(_) => {}
    }
    let source = Source::Watch(watch);
    for (key, id) in &domain.objects.index {
        let object = domain.objects.slab.get_mut(*id).expect("indexed object exists");
        if object.sources.len() == 1 && object.sources.contains(&source) && delivered_name(value, *key) == Some(false) {
            match object.card {
                Card::Open | Card::Refused { .. } => {
                    object.card = Card::Leaving { how: Left::Looking, until: domain.now.saturating_add(domain.linger) };
                }
                Card::Deciding { .. } | Card::Unknown { .. } | Card::Leaving { .. } => {}
            }
        }
    }
}
pub(crate) fn document_read(key: ObjectKey) -> crate::contract::Read {
    use crate::contract::Read;
    match key {
        ObjectKey::Task(task) => Read::Task { task },
        ObjectKey::Proposal(proposal) => Read::Proposal { proposal },
        ObjectKey::Escalation(task) => Read::Escalation { task },
        ObjectKey::Question { task, message } => Read::Question { task, message },
        ObjectKey::PersonTask(task) => Read::PersonTask { task },
        ObjectKey::Result(task) => Read::Result { task },
        ObjectKey::Note { project, .. } => {
            Read::Notes { scope: crate::contract::NoteScope::Project(project), description: None, after: None }
        }
    }
}
/// Requests terminal facts for leaving cards, without inventing an ending.
pub(crate) fn read_dropped(domain: &mut crate::Domain, out: &mut skein_lib::Queue<crate::Request>) {
    for (_, id) in &domain.objects.index {
        let object = domain.objects.slab.get(*id).expect("indexed object exists");
        let looking = match &object.card {
            Card::Leaving { how, .. } => match how {
                Left::Looking => true,
                Left::Answered { .. }
                | Left::Ended { .. }
                | Left::OwnerEnded { .. }
                | Left::Historical { .. }
                | Left::Unknown => false,
            },
            Card::Open | Card::Deciding { .. } | Card::Refused { .. } | Card::Unknown { .. } => false,
        };
        if looking {
            let what = document_read(object.key);
            if !domain.reads.pending(&what) {
                let read = domain.reads.ask(domain.generation, Source::Card(*id), what, out);
                if read.is_none() {
                    domain.objects.slab.get_mut(*id).expect("indexed object exists").card =
                        Card::Refused { refusal: Refusal::Limit { which: crate::contract::Limit::Reads } };
                }
            }
        }
    }
    domain.changed();
}
/// Recognises a terminal read owned by a lingering card.
fn read_key(what: &crate::contract::Read) -> Option<ObjectKey> {
    use crate::contract::Read;
    match what {
        Read::Task { task } => Some(ObjectKey::Task(*task)),
        Read::Proposal { proposal } => Some(ObjectKey::Proposal(*proposal)),
        Read::Escalation { task } => Some(ObjectKey::Escalation(*task)),
        Read::Question { task, message } => Some(ObjectKey::Question { task: *task, message: *message }),
        Read::PersonTask { task } => Some(ObjectKey::PersonTask(*task)),
        Read::Result { task } => Some(ObjectKey::Result(*task)),
        Read::Frame
        | Read::Ended { .. }
        | Read::History { .. }
        | Read::Activity { .. }
        | Read::Inbox { .. }
        | Read::Conversations { .. }
        | Read::Spend { .. }
        | Read::Goals { .. }
        | Read::Project { .. }
        | Read::Notes { .. }
        | Read::Deployment => None,
    }
}
pub(crate) fn is_card_read(domain: &crate::Domain, token: Token) -> bool {
    let Some(read) = domain.reads.slab.get(Id::from_token(token)) else {
        return false;
    };
    match read.source {
        Source::Card(_) => true,
        Source::Watch(_) | Source::Window { .. } => {
            let Some(key) = read_key(&read.what) else {
                return false;
            };
            let Some(id) = domain.objects.find(key) else {
                return false;
            };
            match &domain.objects.get(id).expect("indexed object exists").card {
                Card::Leaving { how: Left::Looking, .. } => true,
                Card::Open
                | Card::Deciding { .. }
                | Card::Refused { .. }
                | Card::Unknown { .. }
                | Card::Leaving {
                    how:
                        Left::Answered { .. }
                        | Left::Ended { .. }
                        | Left::OwnerEnded { .. }
                        | Left::Historical { .. }
                        | Left::Unknown,
                    ..
                } => false,
            }
        }
    }
}
/// Applies a card's terminal read without replacing the whole page on failure.
pub(crate) fn read_card(
    domain: &mut crate::Domain,
    token: Token,
    result: crate::contract::ReadResult,
    out: &mut skein_lib::Queue<crate::Request>,
) {
    use crate::contract::{ReadNone, ReadResult};
    if let Some(value) = historical_read(&result) {
        let Some(read) = domain.reads.slab.get(Id::from_token(token)) else {
            return;
        };
        let Some(key) = read_key(&read.what) else {
            return;
        };
        if !value.matches(key) {
            return;
        }
    }
    let Some(receipt) = domain.reads.terminal(token) else {
        return;
    };
    if receipt.abandoned || receipt.generation != domain.generation {
        return;
    }
    let id = match receipt.source {
        Source::Card(id) => id,
        Source::Watch(_) | Source::Window { .. } => {
            let Some(key) = read_key(&receipt.what) else {
                return;
            };
            let Some(id) = domain.objects.find(key) else {
                return;
            };
            id
        }
    };
    let Some(object) = domain.objects.get(id) else {
        return;
    };
    let key = object.key;
    let leaving = match &object.card {
        Card::Leaving { how: Left::Looking, .. } => true,
        Card::Open | Card::Refused { .. } | Card::Unknown { .. } | Card::Deciding { request: None, .. } => false,
        Card::Deciding { request: Some(_), .. }
        | Card::Leaving {
            how:
                Left::Answered { .. }
                | Left::Ended { .. }
                | Left::OwnerEnded { .. }
                | Left::Historical { .. }
                | Left::Unknown,
            ..
        } => return,
    };
    let body = match result {
        ReadResult::Value(value) => read_body(key, value),
        ReadResult::None(why) => {
            match why {
                ReadNone::SignedOut => crate::domain::signed_out(domain, out),
                ReadNone::Unknown | ReadNone::Gone | ReadNone::Refused | ReadNone::Busy | ReadNone::Unreachable => {}
            }
            None
        }
    };
    let refreshed = body.is_some();
    if let Some(body) = body {
        let stored = domain.objects.put(key, body);
        if !stored {
            return;
        }
    }
    finish_read_card(domain, id, leaving, refreshed);
}
fn finish_read_card(domain: &mut crate::Domain, id: Id<Object>, leaving: bool, refreshed: bool) {
    if !leaving {
        let ending = match domain.objects.get(id).expect("terminal card exists").body.as_ref() {
            Some(body) => body.terminal(),
            None => None,
        };
        if let Some(how) = ending {
            let until = domain.now.saturating_add(domain.linger);
            domain.objects.get_mut(id).expect("terminal card exists").card = Card::Leaving { how, until };
            domain.timers.arm(crate::domain::Timer::Linger(id), until).expect("one linger timer per object fits");
            domain.changed();
            return;
        }
        let object = domain.objects.get_mut(id).expect("terminal card exists");
        match object.card {
            Card::Unknown { .. } => {
                if refreshed {
                    object.card = Card::Open;
                }
            }
            Card::Open | Card::Deciding { .. } | Card::Refused { .. } | Card::Leaving { .. } => {}
        }
        domain.changed();
        return;
    }
    let pending = match domain.objects.get(id).expect("terminal card exists").body.as_ref() {
        Some(body) => body.pending_historical(),
        None => false,
    };
    if pending {
        domain.objects.get_mut(id).expect("terminal card exists").card = Card::Open;
        domain.objects.prune();
        domain.changed();
        return;
    }
    let how = match domain.objects.get(id).expect("terminal card exists").body.as_ref() {
        Some(body) => match body.terminal() {
            Some(ending) => ending,
            None => Left::Unknown,
        },
        None => Left::Unknown,
    };
    let until = domain.now.saturating_add(domain.linger);
    domain.objects.get_mut(id).expect("terminal card exists").card = Card::Leaving { how, until };
    domain.timers.arm(crate::domain::Timer::Linger(id), until).expect("one linger timer per object fits");
    domain.changed();
}
/// The supplied whole historical value, without interpreting any other read.
pub(crate) fn historical_read(result: &crate::contract::ReadResult) -> Option<&crate::contract::OwnerHistorical> {
    use crate::contract::{ReadResult, ReadValue};
    match result {
        ReadResult::Value(value) => match value {
            ReadValue::Historical(value) => Some(value),
            ReadValue::Frame(_)
            | ReadValue::Task(_)
            | ReadValue::Proposal(_)
            | ReadValue::OwnerProposal(_)
            | ReadValue::Escalation(_)
            | ReadValue::OwnerEscalation(_)
            | ReadValue::Question(_)
            | ReadValue::PersonTask(_)
            | ReadValue::OwnerPersonTask(_)
            | ReadValue::Result(_)
            | ReadValue::Ended(_)
            | ReadValue::History(_)
            | ReadValue::Activity(_)
            | ReadValue::Inbox(_)
            | ReadValue::OwnerInbox(_)
            | ReadValue::Conversations(_)
            | ReadValue::Goals(_)
            | ReadValue::Spend(_)
            | ReadValue::OwnerSpend(_)
            | ReadValue::OwnerEnded(_)
            | ReadValue::OwnerProject(_)
            | ReadValue::Project(_)
            | ReadValue::Notes(_)
            | ReadValue::Deployment(_) => None,
        },
        ReadResult::None(_) => None,
    }
}
fn read_body(key: ObjectKey, value: crate::contract::ReadValue) -> Option<Body> {
    use crate::contract::ReadValue;
    match value {
        ReadValue::Historical(value) => {
            if value.matches(key) {
                Some(Body::Historical(value))
            } else {
                None
            }
        }
        ReadValue::Task(task) => Some(Body::Task(task)),
        ReadValue::Proposal(proposal) => Some(Body::Proposal(proposal)),
        ReadValue::OwnerProposal(proposal) => Some(Body::OwnerProposal(proposal)),
        ReadValue::Escalation(escalation) => Some(Body::Escalation(escalation)),
        ReadValue::OwnerEscalation(value) => Some(Body::OwnerEscalation(value)),
        ReadValue::Question(question) => Some(Body::Question(question)),
        ReadValue::PersonTask(task) => Some(Body::PersonTask(task)),
        ReadValue::OwnerPersonTask(task) => Some(Body::OwnerPersonTask(task)),
        ReadValue::Result(result) => Some(Body::Result(result)),
        ReadValue::Frame(_)
        | ReadValue::Inbox(_)
        | ReadValue::OwnerInbox(_)
        | ReadValue::Activity(_)
        | ReadValue::History(_)
        | ReadValue::Conversations(_)
        | ReadValue::Ended(_)
        | ReadValue::Spend(_)
        | ReadValue::OwnerEnded(_)
        | ReadValue::OwnerSpend(_)
        | ReadValue::Goals(_)
        | ReadValue::OwnerProject(_)
        | ReadValue::Project(_)
        | ReadValue::Deployment(_)
        | ReadValue::OwnerDeployment(_) => None,
        ReadValue::Notes(page) => {
            let project = match key {
                ObjectKey::Note { project, .. } => project,
                ObjectKey::Task(_)
                | ObjectKey::Proposal(_)
                | ObjectKey::Escalation(_)
                | ObjectKey::Question { .. }
                | ObjectKey::PersonTask(_)
                | ObjectKey::Result(_) => return None,
            };
            for note in page.rows.into_boxed() {
                if key == (ObjectKey::Note { project, note: note.number }) {
                    return Some(Body::Note { project, document: Box::new(note) });
                }
            }
            None
        }
    }
}

impl Body {
    fn matches(&self, key: ObjectKey) -> bool {
        match self {
            Self::Historical(value) => value.matches(key),
            Self::Chip(_)
            | Self::Task(_)
            | Self::Proposal(_)
            | Self::OwnerProposal(_)
            | Self::Escalation(_)
            | Self::OwnerEscalation(_)
            | Self::Question(_)
            | Self::PersonTask(_)
            | Self::OwnerPersonTask(_)
            | Self::Result(_)
            | Self::Note { .. } => self.key() == key,
        }
    }
    fn pending_historical(&self) -> bool {
        match self {
            Self::Historical(value) => !value.is_terminal(),
            Self::Chip(_)
            | Self::Task(_)
            | Self::Proposal(_)
            | Self::OwnerProposal(_)
            | Self::Escalation(_)
            | Self::OwnerEscalation(_)
            | Self::Question(_)
            | Self::PersonTask(_)
            | Self::OwnerPersonTask(_)
            | Self::Result(_)
            | Self::Note { .. } => false,
        }
    }

    /// The object named by these facts, validated before replacing a body.
    #[must_use]
    pub fn key(&self) -> ObjectKey {
        match self {
            Self::Historical(value) => value.key,
            Self::Chip(chip) => ObjectKey::Task(chip.task),
            Self::Task(task) => ObjectKey::Task(task.chip.task),
            Self::Proposal(proposal) => ObjectKey::Proposal(proposal.key),
            Self::OwnerProposal(proposal) => ObjectKey::Proposal(proposal.key),
            Self::Escalation(escalation) => ObjectKey::Escalation(escalation.task.task),
            Self::OwnerEscalation(value) => ObjectKey::Escalation(value.task.chip.task),
            Self::Question(question) => ObjectKey::Question { task: question.task.task, message: question.message },
            Self::PersonTask(task) => ObjectKey::PersonTask(task.task.task),
            Self::OwnerPersonTask(task) => ObjectKey::PersonTask(task.task.task),
            Self::Result(result) => ObjectKey::Result(result.task.task),
            Self::Note { project, document } => ObjectKey::Note { project: *project, note: document.number },
        }
    }
    /// Complete departure facts from the supplied document, never inferred from phase.
    #[must_use]
    pub fn terminal(&self) -> Option<Left> {
        match self {
            Self::Task(task) => match &task.result {
                Some(crate::contract::TaskResult::Ending(how)) => Some(Left::OwnerEnded { how: how.clone() }),
                Some(crate::contract::TaskResult::Document(result)) => Some(Left::Ended { how: result.ending.clone() }),
                None => None,
            },
            Self::Historical(value) => {
                if value.is_terminal() {
                    Some(Left::Historical { facts: value.clone() })
                } else {
                    None
                }
            }
            Self::OwnerEscalation(_) => None,
            Self::Chip(_)
            | Self::Proposal(_)
            | Self::OwnerProposal(_)
            | Self::Escalation(_)
            | Self::Question(_)
            | Self::PersonTask(_)
            | Self::OwnerPersonTask(_)
            | Self::Result(_)
            | Self::Note { .. } => {
                let how = self.ending()?;
                Some(Left::Ended { how: how.clone() })
            }
        }
    }
    /// The ending a terminal document supplies, never inferred from its phase.
    #[must_use]
    pub fn ending(&self) -> Option<&Ending> {
        match self {
            Self::Task(task) => match &task.result {
                Some(crate::contract::TaskResult::Document(result)) => Some(&result.ending),
                Some(crate::contract::TaskResult::Ending(_)) | None => None,
            },
            Self::Proposal(proposal) => proposal.ending.as_ref(),
            Self::Question(question) => question.ending.as_ref(),
            Self::PersonTask(task) => task.ending.as_ref(),
            Self::Result(result) => Some(&result.ending),
            Self::Escalation(escalation) => escalation.ending.as_ref(),
            Self::Historical(_)
            | Self::Chip(_)
            | Self::OwnerProposal(_)
            | Self::OwnerPersonTask(_)
            | Self::OwnerEscalation(_)
            | Self::Note { .. } => None,
        }
    }
}

/// An object's routing context, independent of its immutable display facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Context {
    pub object: ObjectKey,
    pub project: u32,
}
/// Copies bounded inbox routing facts before its delivery is consumed.
pub(crate) fn contexts(value: &crate::contract::Delivery, capacity: u32) -> List<Context> {
    let mut rows = List::with_capacity(capacity);
    match value {
        crate::contract::Delivery::OwnerInbox(inbox) => {
            for entries in [&inbox.needs, &inbox.updates] {
                for entry in entries {
                    if let Some(project) = entry.project
                        && rows.push(Context { object: entry.card_key(), project }).is_err()
                    {
                        break;
                    }
                }
            }
        }
        crate::contract::Delivery::Inbox(inbox) => {
            for entry in &inbox.entries {
                if rows.push(Context { object: entry.object, project: entry.project }).is_err() {
                    break;
                }
            }
        }
        crate::contract::Delivery::Tasks(tasks) => {
            for row in &tasks.rows {
                if rows.push(Context { object: ObjectKey::Task(row.chip.task), project: tasks.project }).is_err() {
                    break;
                }
            }
        }
        crate::contract::Delivery::Frame(_)
        | crate::contract::Delivery::Task(_)
        | crate::contract::Delivery::OwnerProject(_)
        | crate::contract::Delivery::OwnerProposal(_)
        | crate::contract::Delivery::Tree(_)
        | crate::contract::Delivery::OwnerTree(_)
        | crate::contract::Delivery::Chips(_)
        | crate::contract::Delivery::Goals(_)
        | crate::contract::Delivery::Conversations(_)
        | crate::contract::Delivery::Run(_)
        | crate::contract::Delivery::RunState(_)
        | crate::contract::Delivery::Deployment(_)
        | crate::contract::Delivery::OwnerDeployment(_) => {}
    }
    rows
}
pub(crate) fn apply_contexts(domain: &mut crate::Domain, rows: List<Context>) {
    for row in rows.into_boxed() {
        if let Some(id) = domain.objects.find(row.object) {
            let object = domain.objects.get_mut(id).expect("indexed object exists");
            object.project = match &object.body {
                Some(Body::OwnerPersonTask(document)) => document.project(),
                Some(Body::OwnerEscalation(document)) => Some(document.task.project),
                Some(
                    Body::Historical(_)
                    | Body::Chip(_)
                    | Body::Task(_)
                    | Body::Proposal(_)
                    | Body::OwnerProposal(_)
                    | Body::Escalation(_)
                    | Body::Question(_)
                    | Body::PersonTask(_)
                    | Body::Result(_)
                    | Body::Note { .. },
                )
                | None => Some(row.project),
            };
        }
    }
}
