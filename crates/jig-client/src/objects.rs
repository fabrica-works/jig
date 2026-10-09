//! The page's objects, held once by key and named by explicit source sets.
//! A source replacement first preserves objects it still names, then prunes and
//! admits new names in order. Pending and leaving cards survive without sources.
//! Jig 21 owns card transitions and each object's acts; client/domain/client.md, 5.
use crate::{
    contract::{Chip, Ending, Escalation, ObjectKey, PersonTask, Proposal, Question, Refusal, Result, Task},
    intents::Progress,
    watches::Watch,
};
use alloc::boxed::Box;
use skein_lib::{Id, List, Map, Set, Slab, Time, Token};
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
}
/// The facts held for an object, with independently moving revisions inside.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Body {
    /// A named task's compact facts.
    Chip(Chip),
    /// The full task document.
    Task(Box<Task>),
    /// A proposal document.
    Proposal(Box<Proposal>),
    /// A held task escalation.
    Escalation(Box<Escalation>),
    /// A question.
    Question(Box<Question>),
    /// A person task.
    PersonTask(Box<PersonTask>),
    /// A result.
    Result(Box<Result>),
}
/// A card's state; jig 21 completes transitions driven by an intent.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Card {
    /// Available for an offered act.
    Open,
    /// Asked, with the root table's progress.
    Deciding { request: Token, progress: Progress },
    /// Refused, with facts kept on its card.
    Refused { refusal: Refusal },
    /// Ended and still shown until its linger expires.
    Leaving { ending: Ending, until: Time },
}
/// One object and the sources that still name it.
#[derive(Debug)]
pub struct Object {
    pub key: ObjectKey,
    pub body: Option<Body>,
    pub sources: Set<Source>,
    pub card: Card,
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
                Ok(_) => return Some(id),
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
        let object = Object { key, body: None, sources, card: Card::Open };
        let Ok(id) = self.slab.insert(object) else {
            return None;
        };
        let old = self.index.insert(key, id).expect("index capacity checked before inserting");
        assert!(old.is_none(), "new object key is not indexed");
        Some(id)
    }
    /// Replaces only the facts, preserving one card and its source set.
    pub fn put(&mut self, key: ObjectKey, body: Body) -> bool {
        let Some(id) = self.find(key) else {
            return false;
        };
        let object = self.slab.get_mut(id).expect("indexed object exists");
        match body {
            Body::Chip(chip) => match &mut object.body {
                Some(Body::Task(task)) => task.chip = chip,
                Some(
                    Body::Chip(_)
                    | Body::Proposal(_)
                    | Body::Escalation(_)
                    | Body::Question(_)
                    | Body::PersonTask(_)
                    | Body::Result(_),
                )
                | None => object.body = Some(Body::Chip(chip)),
            },
            Body::Task(_)
            | Body::Proposal(_)
            | Body::Escalation(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::Result(_) => object.body = Some(body),
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
    /// Clears page sources while pending and leaving objects remain held.
    pub fn clear(&mut self) {
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
        Card::Deciding { .. } | Card::Leaving { .. } => true,
        Card::Open | Card::Refused { .. } => false,
    }
}

/// The working set containers, source sets and document payloads, retired room included.
#[must_use]
pub fn worst_case(limits: &crate::Limits) -> Option<u64> {
    let slots = limits.objects.checked_mul(2)?;
    let payload = Set::<Source>::worst_case(limits.sources)?.checked_add(limits.document_bytes.checked_mul(2)?)?;
    Slab::<Object>::worst_case(slots)?
        .checked_add(Map::<ObjectKey, Id<Object>>::worst_case(limits.objects)?)?
        .checked_add(List::<ObjectKey>::worst_case(limits.objects)?)?
        .checked_add(u64::from(slots).checked_mul(payload)?)
}
