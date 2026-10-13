//! Independent party observations of the watch contract (`contract.md`, 7.2).
//! Commit numbers and changed task identities come only from root outputs;
//! deliveries are checked against the fake store's durable watermark. The
//! referee never reads the core, its views child, or the journal's held queue.
use jig_domain_core::{
    self as core,
    documents::{Document, Subject},
};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use jig_test_domain as root;
use skein_lib::{Rng, Token};
use std::collections::{BTreeMap, BTreeSet};

/// One independently expected delivery, including decisions that wrote nothing.
#[derive(Debug, Default)]
pub struct Fence {
    pending: Option<u64>,
    changed: bool,
    in_flight: bool,
}

impl Fence {
    #[must_use]
    pub fn opened(after: u64) -> Self {
        Self { pending: Some(after), changed: false, in_flight: false }
    }

    pub fn changed(&mut self, after: u64) {
        if self.pending.is_some() || self.in_flight {
            self.changed = true;
        } else {
            self.pending = Some(after);
        }
    }

    /// A protocol acknowledgement permits one replacement with the newest state.
    pub fn consumed(&mut self, last_made: u64) {
        assert!(self.in_flight, "a party consumes only a received delivery");
        self.in_flight = false;
        if self.changed {
            self.pending = Some(last_made);
            self.changed = false;
        }
    }

    /// Check the prerequisite visible at this delivery's construction.
    ///
    /// # Errors
    /// Rejects an unsolicited delivery or one preceding its durable fence.
    pub fn delivered(&mut self, durable: u64) -> Result<(), &'static str> {
        if self.in_flight {
            return Err("watch delivery overtook its acknowledgement");
        }
        let Some(after) = self.pending else { return Err("unsolicited watch delivery") };
        if durable < after {
            return Err("watch delivery preceded its durable fence");
        }
        self.pending = None;
        self.in_flight = true;
        Ok(())
    }

    /// Check that the party received the final changed state.
    ///
    /// # Errors
    /// Rejects an open watch whose initial or replacement delivery is missing.
    pub fn finish(&self) -> Result<(), &'static str> {
        if self.pending.is_some() || self.changed {
            Err("a changed open watch missed its last delivery")
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
enum Watched {
    Jig(Subject),
    Application { number: u16, subject: jig_test_connector::ClientSubject },
}

#[derive(Debug)]
struct Consumer {
    subject: Watched,
    fence: Fence,
    automatic: bool,
    queued: bool,
}

/// Scripted parties retain received documents and draw acknowledgement pace
/// from their own seed. Manual consumers let stories deliberately withhold it.
#[derive(Debug)]
pub struct Parties {
    rng: Rng,
    made: u64,
    automatic: bool,
    consumers: BTreeMap<u64, Consumer>,
    projects: BTreeMap<u64, BTreeSet<u32>>,
    task_projects: BTreeMap<u64, u32>,
    proposal_projects: BTreeMap<u64, u32>,
    pub deliveries: Vec<(u64, u64, Document)>,
    pub application_deliveries: Vec<(u64, u64, u16, jig_test_connector::ClientDocument)>,
    pub endings: Vec<(u64, views::End)>,
    pub streams: Vec<Stream>,
}

impl Parties {
    #[must_use]
    pub fn new(seed: u64, made: u64, automatic: bool) -> Self {
        Self {
            rng: Rng::new(seed),
            made,
            automatic,
            consumers: BTreeMap::new(),
            projects: BTreeMap::new(),
            task_projects: BTreeMap::new(),
            proposal_projects: BTreeMap::new(),
            deliveries: Vec::new(),
            application_deliveries: Vec::new(),
            endings: Vec::new(),
            streams: Vec::new(),
        }
    }

    pub fn manual(&mut self, watcher: u64) {
        self.consumers.get_mut(&watcher).expect("open watch").automatic = false;
    }

    pub fn inbound(&mut self, event: &root::Event) {
        match event {
            root::Event::WatchApplication { watcher, number, subject, .. } => {
                assert!(
                    self.consumers
                        .insert(
                            watcher.raw(),
                            Consumer {
                                subject: Watched::Application { number: *number, subject: *subject },
                                fence: Fence::opened(self.made),
                                automatic: self.automatic,
                                queued: false
                            }
                        )
                        .is_none(),
                    "independent application watch admission"
                );
            }
            root::Event::Core(core::Event::Watch { watcher, subject, .. }) => {
                assert!(
                    self.consumers
                        .insert(
                            watcher.raw(),
                            Consumer {
                                subject: Watched::Jig(subject.clone()),
                                fence: Fence::opened(self.made),
                                automatic: self.automatic,
                                queued: false,
                            }
                        )
                        .is_none(),
                    "watch tokens are independent admissions"
                );
            }
            root::Event::Core(core::Event::WatchRun { watcher, task, .. }) => {
                assert!(
                    self.consumers
                        .insert(
                            watcher.raw(),
                            Consumer {
                                subject: Watched::Jig(Subject::Run { task: *task }),
                                fence: Fence::opened(self.made),
                                automatic: self.automatic,
                                queued: false,
                            }
                        )
                        .is_none(),
                    "run positions belong to independent watch admissions"
                );
            }
            root::Event::Core(core::Event::Delivered { watcher }) => {
                if let Some(consumer) = self.consumers.get_mut(&watcher.raw()) {
                    consumer.queued = false;
                    consumer.fence.consumed(self.made);
                }
            }
            root::Event::Core(_)
            | root::Event::Link { .. }
            | root::Event::ResultRead { .. }
            | root::Event::Credential { .. }
            | root::Event::ReadApplication { .. }
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

    pub fn commit(&mut self, number: u64, writes: &[root::Write]) {
        self.made = number;
        for write in writes {
            match write {
                root::Write::Save(root::Record::Core(core::Record::Tasks(
                    tasks::Stored::Live(row) | tasks::Stored::Ended(row),
                ))) => {
                    self.task_projects.insert(row.number, row.project);
                }
                root::Write::Save(root::Record::Core(core::Record::Tasks(
                    tasks::Stored::PersonProposal(row) | tasks::Stored::DecidedProposal(row),
                ))) => {
                    self.proposal_projects.insert(row.number, row.project);
                }
                root::Write::Save(_) | root::Write::Erase(_) => {}
            }
        }
        for consumer in self.consumers.values_mut() {
            let touched = writes.iter().any(|write| match write {
                root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) => {
                    task_subject(&consumer.subject, row.number)
                        || matches!(&consumer.subject, Watched::Jig(Subject::Project { project } | Subject::Tasks { project, .. }) if *project == row.project)
                        || match row.requester {
                            tasks::Party::Person(party) => party_subject(&consumer.subject, party),
                            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => false,
                        }
                }
                root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) => {
                    task_subject(&consumer.subject, row.number)
                        || matches!(&consumer.subject, Watched::Jig(Subject::Project { project } | Subject::Tasks { project, .. }) if *project == row.project)
                        || match row.requester {
                            tasks::Party::Person(party) => party_subject(&consumer.subject, party),
                            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => false,
                        }
                }
                root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(row) | tasks::Stored::DecidedProposal(row)))) => {
                    project_subject(&consumer.subject, row.project, &self.task_projects, &self.proposal_projects)
                        || party_subject(&consumer.subject, row.proposer)
                        || self.projects.iter().any(|(party, projects)| {
                            projects.contains(&row.project) && party_subject(&consumer.subject, *party)
                        })
                }
                root::Write::Save(root::Record::Core(core::Record::People(
                    people::Stored::Person { number, .. }
                    | people::Stored::ReadResults { party: number, .. }
                    | people::Stored::ReadChat { party: number, .. },
                ))) => party_subject(&consumer.subject, *number),
                root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Roles {
                    project,
                    holdings,
                }))) => {
                    project_subject(&consumer.subject, *project, &self.task_projects, &self.proposal_projects)
                        || holdings.iter().any(|holding| party_subject(&consumer.subject, holding.person))
                        || self.projects.iter().any(|(party, projects)| {
                            projects.contains(project) && party_subject(&consumer.subject, *party)
                        })
                }
                root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Policy { project, .. }))) =>
                    project_subject(&consumer.subject, *project, &self.task_projects, &self.proposal_projects),
                root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(row) | tasks::Stored::ClosedLedger(row)))) => match row.funder {
                    tasks::Funder::Pool { project, .. } | tasks::Funder::Period { project, .. } | tasks::Funder::Recurring { project, .. } =>
                        project_subject(&consumer.subject, project, &self.task_projects, &self.proposal_projects),
                    tasks::Funder::Task(_) => false,
                },
                root::Write::Erase(root::Key::Core(core::Key::People(people::Key::ReadChat { party, .. }))) => {
                    party_subject(&consumer.subject, *party)
                }
                root::Write::Save(root::Record::Connector {
                    number,
                    record: jig_test_connector::Record::ApplicationObject { object, .. },
                }) => match &consumer.subject {
                    Watched::Application {
                        number: owner,
                        subject: jig_test_connector::ClientSubject::Object { object: watched, .. },
                    } => *owner == *number && *watched == *object,
                    Watched::Application {
                        subject:
                            jig_test_connector::ClientSubject::Panel { .. }
                            | jig_test_connector::ClientSubject::Result { .. }
                            | jig_test_connector::ClientSubject::Effect { .. }
                            | jig_test_connector::ClientSubject::Outbox { .. }
                            | jig_test_connector::ClientSubject::Hold { .. }
                            | jig_test_connector::ClientSubject::Project { .. }
                            | jig_test_connector::ClientSubject::Deployment,
                        ..
                    }
                    | Watched::Jig(_) => false,
                },

                root::Write::Save(_) | root::Write::Erase(_) => false,
            });
            if touched {
                consumer.fence.changed(number);
            }
        }
    }

    pub fn released(&mut self, delivery: &root::Delivery, durable: u64) {
        match delivery {
            root::Delivery::ApplicationDocument { watcher, number, document } => {
                let consumer = self.consumers.get_mut(&watcher.raw()).expect("admitted application watch");
                consumer.fence.delivered(durable).expect("application watch's durable prerequisite");
                self.application_deliveries.push((watcher.raw(), durable, *number, document.as_ref().clone()));
            }
            root::Delivery::Core(core::Held::Delivery { watcher, document }) => {
                let consumer = self.consumers.get_mut(&watcher.raw()).expect("admitted party watch");
                consumer.fence.delivered(durable).unwrap_or_else(|why| {
                    panic!("watch durable prerequisite {watcher:?} at {durable}: {why}; {document:?}")
                });
                match document.as_ref() {
                    Document::Task(row) => {
                        self.task_projects.insert(row.chip.number, row.project);
                    }
                    Document::Proposal(row) => {
                        self.proposal_projects.insert(row.number, row.project);
                    }
                    Document::Escalation(row) => {
                        self.task_projects.insert(row.task.chip.number, row.task.project);
                    }
                    Document::Tasks(value) => {
                        for line in &value.rows {
                            self.task_projects.insert(line.chip.number, value.project);
                        }
                    }
                    Document::Frame(_)
                    | Document::Tree(_)
                    | Document::Chips(_)
                    | Document::Inbox(_)
                    | Document::Goals(_)
                    | Document::Conversations(_)
                    | Document::Run(_)
                    | Document::Deployment(_)
                    | Document::Question(_)
                    | Document::PersonTask(_)
                    | Document::Result(_)
                    | Document::Project(_)
                    | Document::Note(_) => {}
                }
                if let Document::Frame(frame) = document.as_ref() {
                    self.projects.insert(frame.party, frame.projects.iter().map(|role| role.project).collect());
                }
                self.deliveries.push((watcher.raw(), durable, *document.clone()));
            }
            root::Delivery::Core(core::Held::WatchEnded { watcher, why }) => {
                assert!(durable >= self.made, "watch end follows its deciding commit");
                self.consumers.remove(&watcher.raw()).expect("ending an open watch");
                self.endings.push((watcher.raw(), *why));
            }
            root::Delivery::Core(_)
            | root::Delivery::ApplicationRead { .. }
            | root::Delivery::Application { .. }
            | root::Delivery::Restart(_)
            | root::Delivery::Fleet(_)
            | root::Delivery::Projection { .. }
            | root::Delivery::Message { .. }
            | root::Delivery::Assigned { .. }
            | root::Delivery::Link { .. }
            | root::Delivery::CallBusy { .. }
            | root::Delivery::CallAnswer { .. }
            | root::Delivery::System { .. }
            | root::Delivery::LostRead { .. }
            | root::Delivery::Procedure { .. } => {}
        }
    }

    pub fn now(&mut self, now: &core::Now) {
        if let core::Now::RunText { watcher, turn, from, to, text, cut, call } = now {
            self.streams.push(Stream {
                watcher: watcher.raw(),
                turn: *turn,
                from: *from,
                to: *to,
                text: text.clone(),
                cut: *cut,
                call: call.clone(),
            });
        }
        if let core::Now::WatchRefused { .. } = now {
            panic!("scripted party's watch was refused: {now:?}");
        }
    }

    pub fn tick(&mut self) -> Vec<root::Event> {
        self.consumers
            .iter_mut()
            .filter_map(|(watcher, consumer)| {
                if consumer.automatic && consumer.fence.in_flight && !consumer.queued && self.rng.below(5) == 0 {
                    consumer.queued = true;
                    Some(root::Event::Core(core::Event::Delivered { watcher: Token::new(*watcher) }))
                } else {
                    None
                }
            })
            .collect()
    }

    #[must_use]
    pub fn quiescent(&self) -> bool {
        self.consumers.values().all(|consumer| !consumer.automatic || !consumer.fence.in_flight)
    }

    pub fn finish(&self) {
        for consumer in self.consumers.values() {
            consumer.fence.finish().expect("open watch receives its final state");
        }
    }

    pub fn restart(&mut self, durable: u64) {
        self.consumers.clear();
        self.made = durable;
    }
}

fn task_subject(watched: &Watched, task: u64) -> bool {
    let subject = match watched {
        Watched::Jig(subject) => subject,
        Watched::Application { .. } => return false,
    };
    match subject {
        Subject::Task { task: watched }
        | Subject::Tree { task: watched }
        | Subject::Run { task: watched }
        | Subject::Escalation { task: watched }
        | Subject::Question { task: watched, .. }
        | Subject::PersonTask { task: watched }
        | Subject::Result { task: watched }
        | Subject::Proposal { proposer: core::documents::Proposer::Task(watched), .. } => *watched == task,
        Subject::Chips { tasks } => tasks.as_slice().contains(&task),
        Subject::Frame { .. }
        | Subject::Inbox { .. }
        | Subject::Goals { .. }
        | Subject::Tasks { .. }
        | Subject::Conversations { .. }
        | Subject::Deployment
        | Subject::Proposal { proposer: core::documents::Proposer::Party(_), .. }
        | Subject::Project { .. }
        | Subject::Note { .. }
        | Subject::Application { .. } => false,
    }
}

fn project_subject(
    watched: &Watched,
    project: u32,
    tasks: &BTreeMap<u64, u32>,
    proposals: &BTreeMap<u64, u32>,
) -> bool {
    let Watched::Jig(subject) = watched else {
        return false;
    };
    match subject {
        Subject::Project { project: owned }
        | Subject::Goals { project: owned }
        | Subject::Tasks { project: owned, .. } => *owned == project,
        Subject::Note { .. } => true,
        Subject::Task { task }
        | Subject::Tree { task }
        | Subject::Run { task }
        | Subject::Escalation { task }
        | Subject::Question { task, .. }
        | Subject::PersonTask { task }
        | Subject::Result { task }
        | Subject::Proposal { proposer: core::documents::Proposer::Task(task), .. } => {
            tasks.get(task) == Some(&project)
        }
        Subject::Proposal { proposer: core::documents::Proposer::Party(_), number } => {
            proposals.get(number) == Some(&project)
        }
        Subject::Chips { tasks: watched } => watched.iter().any(|task| tasks.get(task) == Some(&project)),
        Subject::Frame { .. }
        | Subject::Inbox { .. }
        | Subject::Conversations { .. }
        | Subject::Deployment
        | Subject::Application { .. } => false,
    }
}

/// A party's received provisional stream, outside its durable documents.
#[derive(Debug)]
pub struct Stream {
    pub watcher: u64,
    pub turn: views::TurnName,
    pub from: u32,
    pub to: u32,
    pub text: Box<[u8]>,
    pub cut: bool,
    pub call: Option<views::Call>,
}

fn party_subject(watched: &Watched, party: u64) -> bool {
    let subject = match watched {
        Watched::Jig(subject) => subject,
        Watched::Application { .. } => return false,
    };
    match subject {
        Subject::Frame { party: watched } | Subject::Inbox { party: watched } => *watched == party,
        Subject::Task { .. }
        | Subject::Tree { .. }
        | Subject::Run { .. }
        | Subject::Chips { .. }
        | Subject::Goals { .. }
        | Subject::Tasks { .. }
        | Subject::Conversations { .. }
        | Subject::Deployment
        | Subject::Proposal { .. }
        | Subject::Escalation { .. }
        | Subject::Question { .. }
        | Subject::PersonTask { .. }
        | Subject::Result { .. }
        | Subject::Project { .. }
        | Subject::Note { .. }
        | Subject::Application { .. } => false,
    }
}

/// Open the four independent party subscriptions used by watch stories.
#[must_use]
pub fn world(seed: u64, automatic: bool) -> crate::effects::World {
    let (configuration, mut limits) = crate::effects::fixture(seed, false);
    limits.core.views.watchers = 8;
    limits.core.views.watchers_per_party = 8;
    let room = core::room_max(&limits.core).expect("watch story room");
    let connectors = jig_test_connector::MAX_OUT * 2;
    limits.journal.writes = (room.writes + connectors) * 2;
    limits.journal.held = (room.held + connectors) * 2;
    let mut world = crate::effects::World::configured(seed, false, configuration, limits);
    world.watches = Some(Parties::new(seed, world.store.applied, automatic));
    let task = world.assigned.as_ref().expect("assigned chat").task;
    for (watcher, subject) in [
        (1, Subject::Frame { party: 1 }),
        (2, Subject::Task { task }),
        (3, Subject::Tree { task }),
        (4, Subject::Inbox { party: 1 }),
    ] {
        open(&mut world, watcher, subject);
    }
    world
}

pub fn application(
    world: &mut crate::effects::World,
    watcher: u64,
    scope: core::documents::ApplicationScope,
    number: u16,
    subject: jig_test_connector::ClientSubject,
) {
    world.send(root::Event::WatchApplication {
        watcher: Token::new(watcher),
        sign_in: world.sign_in.expect("signed in"),
        scope,
        number,
        subject,
        window: core::documents::Window { rows: 16, after: None },
    });
}

pub fn open(world: &mut crate::effects::World, watcher: u64, subject: Subject) {
    world.send(root::Event::Core(core::Event::Watch {
        watcher: Token::new(watcher),
        sign_in: world.sign_in.expect("signed in"),
        subject,
        window: core::documents::Window { rows: 16, after: None },
    }));
}

#[must_use]
pub fn latest(world: &crate::effects::World, watcher: u64) -> &Document {
    &world
        .watches
        .as_ref()
        .expect("party watches")
        .deliveries
        .iter()
        .rev()
        .find(|(received, _, _)| *received == watcher)
        .expect("received document")
        .2
}
