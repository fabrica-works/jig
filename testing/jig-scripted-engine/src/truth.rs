//! The story's starting documents and atomic changes; contract.md, 5.
//! Documents carry offers, revisions, terminal facts and parts in the client's vocabulary.
use crate::{Bounds, Invalid, payload};
use jig_client::contract::{Answer, Ask, Deployment, Frame, Note, ObjectKey, Project, ReadValue};
use std::collections::BTreeMap;

/// Client documents indexed by their identities, with parties and sign-ins.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Truth {
    pub frames: BTreeMap<u64, Frame>,
    pub sign_ins: BTreeMap<u64, u64>,
    pub objects: BTreeMap<ObjectKey, ReadValue>,
    /// Project context for object documents whose client carrier names only a task.
    pub object_projects: BTreeMap<ObjectKey, u32>,
    pub projects: BTreeMap<u32, Project>,
    pub deployment: Option<Deployment>,
    pub notes: BTreeMap<u64, Note>,
}

/// One whole-document replacement made atomically with its story's decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    /// Replace a party's authoritative frame, including roles, versions and retention.
    Frame(Frame),
    /// Confirm a numbered sign-in for a party.
    SignIn { sign_in: u64, party: u64 },
    /// End a numbered sign-in.
    SignedOut { sign_in: u64 },
    /// Replace an object, preserving its identity.
    Object { project: u32, key: ObjectKey, value: ReadValue },
    /// Forget a live object; subsequent reads are unknown.
    Forget { key: ObjectKey },
    /// Replace a project document.
    Project(Box<Project>),
    /// Replace a revisioned note.
    Note(Note),
    /// Delete a note.
    DeleteNote { note: u64 },
    /// Replace deployment hosts, accounts and capacities.
    Deployment(Deployment),
}
impl Change {
    pub(crate) fn validate(&self, bounds: &Bounds) -> core::result::Result<(), Invalid> {
        let fits = match self {
            Self::Frame(value) => payload::fits(value, bounds),
            Self::Object { project, key, value } => {
                if identity(value) != Some(*key) || document_project(value).is_some_and(|number| number != *project) {
                    return Err(Invalid::Identity);
                }
                payload::fits(value, bounds)
            }
            Self::Project(value) => payload::fits(value, bounds),
            Self::Note(value) => payload::fits(value, bounds),
            Self::Deployment(value) => payload::fits(value, bounds),
            Self::SignIn { .. } | Self::SignedOut { .. } | Self::Forget { .. } | Self::DeleteNote { .. } => true,
        };
        if fits { Ok(()) } else { Err(Invalid::Payload) }
    }
}
impl Truth {
    /// Validates fixture documents, including unused list capacity and nested payloads.
    pub fn validate(&self, bounds: &Bounds) -> core::result::Result<(), Invalid> {
        for (party, frame) in &self.frames {
            if *party != frame.party.number {
                return Err(Invalid::Identity);
            }
            Change::Frame(frame.clone()).validate(bounds)?;
        }
        for party in self.sign_ins.values() {
            if !self.frames.contains_key(party) {
                return Err(Invalid::Identity);
            }
        }
        for (key, value) in &self.objects {
            let project = self.project(*key).ok_or(Invalid::Identity)?;
            Change::Object { project, key: *key, value: value.clone() }.validate(bounds)?;
        }
        for (number, project) in &self.projects {
            if *number != project.number {
                return Err(Invalid::Identity);
            }
            Change::Project(Box::new(project.clone())).validate(bounds)?;
        }
        for (number, note) in &self.notes {
            if *number != note.number {
                return Err(Invalid::Identity);
            }
            Change::Note(note.clone()).validate(bounds)?;
        }
        if let Some(deployment) = &self.deployment {
            Change::Deployment(deployment.clone()).validate(bounds)?;
        }
        Ok(())
    }
    pub(crate) fn apply(&mut self, change: &Change) -> core::result::Result<(), Invalid> {
        match change {
            Change::Frame(frame) => {
                self.frames.insert(frame.party.number, frame.clone());
            }
            Change::SignIn { sign_in, party } => {
                if !self.frames.contains_key(party) {
                    return Err(Invalid::Identity);
                }
                self.sign_ins.insert(*sign_in, *party);
            }
            Change::SignedOut { sign_in } => {
                self.sign_ins.remove(sign_in);
            }
            Change::Object { project, key, value } => {
                self.object_projects.insert(*key, *project);
                self.objects.insert(*key, value.clone());
            }
            Change::Forget { key } => {
                self.objects.remove(key);
                self.object_projects.remove(key);
            }
            Change::Project(project) => {
                self.projects.insert(project.number, (**project).clone());
            }
            Change::Note(note) => {
                self.notes.insert(note.number, note.clone());
            }
            Change::DeleteNote { note } => {
                self.notes.remove(note);
            }
            Change::Deployment(deployment) => {
                self.deployment = Some(deployment.clone());
            }
        }
        Ok(())
    }
    pub(crate) fn project(&self, key: ObjectKey) -> Option<u32> {
        document_project(self.objects.get(&key)?).or_else(|| self.object_projects.get(&key).copied())
    }
    pub(crate) fn visible(&self, party: u64, project: u32) -> bool {
        self.frames
            .get(&party)
            .is_some_and(|frame| frame.projects.iter().any(|role| role.project == project && !role.roles.is_empty()))
    }
}

/// The answer and truth changes scripted for one request, before key retention.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub party: Option<u64>,
    pub ask: Ask,
    pub answer: Answer,
    pub changes: Vec<Change>,
}
/// The starting truth, payload limits and request outcomes for a scenario.
#[derive(Clone, Debug, Default)]
pub struct Script {
    pub truth: Truth,
    pub bounds: Bounds,
    pub rules: Vec<Rule>,
}
impl Script {
    pub(crate) fn validate(&self) -> core::result::Result<(), Invalid> {
        self.truth.validate(&self.bounds)?;
        for rule in &self.rules {
            if matches!(rule.answer, Answer::Refused(_)) && !rule.changes.is_empty() {
                return Err(Invalid::Script);
            }
            if !payload::fits(&rule.ask, &self.bounds) || !payload::fits(&rule.answer, &self.bounds) {
                return Err(Invalid::Payload);
            }
            for change in &rule.changes {
                change.validate(&self.bounds)?;
            }
        }
        Ok(())
    }
}
pub(crate) fn identity(value: &ReadValue) -> Option<ObjectKey> {
    match value {
        ReadValue::Task(v) => Some(ObjectKey::Task(v.chip.task)),
        ReadValue::Proposal(v) => Some(ObjectKey::Proposal(v.key)),
        ReadValue::Escalation(v) => Some(ObjectKey::Escalation(v.task.task)),
        ReadValue::Question(v) => Some(ObjectKey::Question { task: v.task.task, message: v.message }),
        ReadValue::PersonTask(v) => Some(ObjectKey::PersonTask(v.task.task)),
        ReadValue::Result(v) => Some(ObjectKey::Result(v.task.task)),
        ReadValue::Frame(_)
        | ReadValue::Ended(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Conversations(_)
        | ReadValue::Goals(_)
        | ReadValue::Spend(_)
        | ReadValue::Project(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => None,
    }
}

fn document_project(value: &ReadValue) -> Option<u32> {
    match value {
        ReadValue::Task(v) => Some(v.project),
        ReadValue::Proposal(v) => Some(v.project),
        ReadValue::Frame(_)
        | ReadValue::Escalation(_)
        | ReadValue::Question(_)
        | ReadValue::PersonTask(_)
        | ReadValue::Result(_)
        | ReadValue::Ended(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Conversations(_)
        | ReadValue::Goals(_)
        | ReadValue::Spend(_)
        | ReadValue::Project(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => None,
    }
}
