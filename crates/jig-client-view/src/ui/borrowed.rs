//! A tree entry borrows the root's client; the UI retains only its presentation state.
use super::Ui;
use crate::objects::{Body, Object, Objects};
use jig_client::{
    Client, Latest,
    contract::{Delivery, ObjectKey},
};
use skein_lib::Id;
pub mod conversation;
pub mod project;
pub mod task;

#[derive(Debug)]
pub struct Borrowed<'a> {
    pub(crate) ui: &'a Ui,
    pub(crate) client: &'a Client,
}
impl core::ops::Deref for Borrowed<'_> {
    type Target = Ui;
    fn deref(&self) -> &Ui {
        self.ui
    }
}
impl Ui {
    #[must_use]
    pub const fn borrow<'a>(&'a self, client: &'a Client) -> Borrowed<'a> {
        Borrowed { ui: self, client }
    }
}
impl<'a> Borrowed<'a> {
    #[must_use]
    pub fn shows(&self, section: crate::page::Section) -> bool {
        self.ui.shows(section, self.client)
    }
    #[must_use]
    pub fn frame_summary(&self) -> crate::frame::me::Summary {
        self.ui.frame_summary(self.client)
    }
    #[must_use]
    pub fn frame(&self) -> crate::frame::me::FrameState<'a> {
        self.ui.frame(self.client)
    }
    #[must_use]
    pub fn owes(&self, id: Id<Object>) -> bool {
        self.ui.owes(id, self.client)
    }
    #[must_use]
    pub fn section_status(&self, section: crate::page::Section) -> Option<crate::page::SectionStatus> {
        let mut status = self.ui.section_status(section, self.client)?;
        for (handle, held) in &self.ui.handles.watches {
            if held.owner == super::Owner::Sections
                && crate::page::shown::uses(section, &held.subject)
                && let Some(document) = self.client.document(*handle)
            {
                status.behind |= document.state == jig_client::State::Behind;
            }
        }
        Some(status)
    }
    #[must_use]
    pub fn inbox(&self) -> crate::page::inbox::Inbox {
        let mut inbox = self.ui.inbox.snapshot();
        for (handle, held) in &self.ui.handles.watches {
            if held.owner == crate::handles::Owner::Application {
                continue;
            }
            let Some(document) = self.client.document(*handle) else {
                continue;
            };
            match document.latest {
                Some(Latest::Jig(Delivery::OwnerInbox(value))) => inbox.owner(value.clone()),
                Some(Latest::Jig(Delivery::Inbox(value))) => {
                    inbox.needs = value.needs;
                    inbox.updates = value.updates;
                    inbox.results_seen = inbox.results_seen.max(value.results_seen);
                    inbox.document = Some(jig_client::contract::InboxDocument::Entry(value.clone()));
                    inbox.replace(value.entries.clone(), value.more);
                }
                Some(Latest::Jig(_) | Latest::Family(_)) | None => {}
            }
        }
        inbox
    }
    #[must_use]
    pub fn conversation_rows(&self) -> crate::windows::ConversationWindow {
        let mut rows = self.ui.conversation_rows.clone();
        for (handle, _) in &self.ui.handles.watches {
            if self.ui.chats.show_live
                && let Some(document) = self.client.document(*handle)
                && let Some(Latest::Jig(Delivery::Conversations(value))) = document.latest
            {
                crate::page::conversations::project_live(&mut rows, *value.clone());
            }
        }
        rows
    }
    #[must_use]
    pub fn deployment_page(&self) -> Deployment<'a> {
        let mut page = Deployment {
            document: self.ui.deployment_page.document.as_deref(),
            owner: self.ui.deployment_page.owner.as_deref(),
        };
        for (handle, held) in &self.ui.handles.watches {
            if held.subject != jig_client::contract::Subject::Deployment {
                continue;
            }
            if let Some(document) = self.client.document(*handle) {
                match document.latest {
                    Some(Latest::Jig(Delivery::Deployment(value))) => {
                        page.document = Some(value);
                        page.owner = None;
                    }
                    Some(Latest::Jig(Delivery::OwnerDeployment(value))) => {
                        page.owner = Some(value);
                        page.document = None;
                    }
                    Some(Latest::Jig(_) | Latest::Family(_)) | None => {}
                }
            }
        }
        page
    }
    #[must_use]
    pub fn link(&self) -> jig_client::Link {
        self.client.link().state
    }
    #[must_use]
    pub fn objects(&self) -> ObjectsBorrowed<'a> {
        ObjectsBorrowed { state: self.ui.objects(), client: self.client, handles: &self.ui.handles, ui: self.ui }
    }
}
#[derive(Debug)]
pub struct ObjectsBorrowed<'a> {
    state: &'a Objects,
    client: &'a Client,
    handles: &'a crate::handles::Handles,
    ui: &'a Ui,
}
impl core::ops::Deref for ObjectsBorrowed<'_> {
    type Target = Objects;
    fn deref(&self) -> &Objects {
        self.state
    }
}
impl ObjectsBorrowed<'_> {
    #[must_use]
    pub fn get(&self, id: Id<Object>) -> Option<Object> {
        let stored = self.state.get(id)?;
        let mut sources = skein_lib::Set::with_capacity(stored.sources.len());
        for source in &stored.sources {
            sources.insert(*source).expect("original sources fit");
        }
        let mut object = Object {
            key: stored.key,
            project: stored.project,
            body: stored.body.clone(),
            sources,
            card: stored.card.clone(),
            answer: stored.answer.clone(),
            shown: stored.shown,
            reply: stored.reply.clone(),
        };
        if let ObjectKey::Task(number) = object.key {
            for line in &self.ui.borrow(self.client).conversation_rows().rows {
                if line.chip.task == number {
                    if object.body.is_none() {
                        object.body = Some(Body::Chip(line.chip.clone()));
                    }
                    object.reply = crate::objects::result::Reply::from_line(line.clone());
                }
            }
            for row in &self.ui.borrow(self.client).inbox().rows {
                if let Some(line) = row.reply()
                    && line.chip.task == number
                {
                    if object.body.is_none() {
                        object.body = Some(Body::Chip(line.chip.clone()));
                    }
                    object.reply = crate::objects::result::Reply::from_line(line.clone());
                }
            }
        }
        for (handle, held) in &self.handles.watches {
            if (object.sources.contains(&crate::objects::Source::Watch(*handle))
                || held.owner == crate::handles::Owner::Application)
                && let Some(document) = self.client.document(*handle)
                && let Some(Latest::Jig(value)) = document.latest
                && let Some(body) = body(value, object.key)
            {
                if let Delivery::OwnerTree(tree) = value
                    && object.key == ObjectKey::Task(tree.root.task)
                    && let Some(Body::Task(_)) = object.body
                {
                    continue;
                }
                match (&mut object.body, body) {
                    (Some(Body::Task(task)), Body::Chip(chip)) => task.chip = chip,
                    (slot, body) => *slot = Some(body),
                }
            }
        }
        if let Some(Body::Task(task)) = &object.body {
            object.project = Some(task.project);
        }
        if let Some(Body::OwnerProposal(proposal)) = &object.body {
            object.project = Some(proposal.project);
        }
        Some(object)
    }
}
fn body(value: &Delivery, key: ObjectKey) -> Option<Body> {
    match (value, key) {
        (Delivery::Task(task), ObjectKey::Task(number)) if task.chip.task == number => Some(Body::Task(task.clone())),
        (Delivery::OwnerProposal(value), ObjectKey::Proposal(key)) if value.key == key => {
            Some(Body::OwnerProposal(value.clone()))
        }
        (Delivery::Chips(rows), ObjectKey::Task(number)) => {
            for chip in rows {
                if chip.task == number {
                    return Some(Body::Chip(chip.clone()));
                }
            }
            None
        }
        (Delivery::Tree(tree), ObjectKey::Task(number)) => {
            for row in &tree.nodes {
                if row.chip.task == number {
                    return Some(Body::Chip(row.chip.clone()));
                }
            }
            None
        }
        (Delivery::OwnerTree(tree), ObjectKey::Task(number)) => {
            if tree.root.task == number {
                return Some(Body::Chip(tree.root.clone()));
            }
            for row in &tree.rows {
                if row.chip.task == number {
                    return Some(Body::Chip(row.chip.clone()));
                }
            }
            None
        }
        (Delivery::Goals(goals), ObjectKey::Task(number)) => {
            for row in &goals.rows {
                if row.chip.task == number {
                    return Some(Body::Chip(row.chip.clone()));
                }
            }
            None
        }
        (Delivery::Tasks(tasks), ObjectKey::Task(number)) => {
            for row in &tasks.rows {
                if row.chip.task == number {
                    return Some(Body::Chip(row.chip.clone()));
                }
            }
            None
        }
        _ => None,
    }
}

#[derive(Debug)]
pub struct Deployment<'a> {
    pub document: Option<&'a jig_client::contract::Deployment>,
    pub owner: Option<&'a jig_client::contract::OwnerDeployment>,
}
