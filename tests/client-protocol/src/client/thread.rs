//! A worker commits the person's words after their durable receipt is visible.
use super::{Duration, Engine, Event, Front, Link, List, Net, Story, Time, fixtures};
use jig_fake_workers::{Script, Worker};
const READY: Duration = Duration::from_secs(1);
const COMMIT_WAIT: Duration = Duration::from_millis(100);
const WORKER: u64 = 1;
pub const READ_MESSAGE: u64 = 1;
impl Story {
    #[must_use]
    pub fn reading_chat(seed: u64) -> Self {
        let body = fixtures::TURN_BYTES;
        Self::reading_chat_with(seed, Box::from(body), fixtures::domain_limits(), fixtures::owner_limits())
    }
    #[must_use]
    pub fn reading_chat_with(
        seed: u64,
        body: Box<[u8]>,
        client: jig_client_domain::Limits,
        core: jig_test_domain::Limits,
    ) -> Self {
        let script = Box::new([
            Script::Wait,
            Script::Silence { for_: COMMIT_WAIT },
            Script::Turn { body, cost: 1, read: Some(READ_MESSAGE) },
            Script::Wait,
        ]);
        let peers = jig_core_world::peers::Peers::new(vec![Worker::new(WORKER, 1, vec![script])], Vec::new());
        let mut story = Self {
            front: Front::with_limits(seed, client),
            second: None,
            engine: Engine::with_peers(seed, &core, jig_core_world::world::config(seed), peers),
            net: Net::new(seed),
            now: Time::ZERO,
            seen: Vec::new(),
        };
        story.event(Event::Start { kept: List::with_capacity(0) });
        story.advance();
        story.sign_in();
        assert!(story.run_until(|s| s.client.link().state == Link::Live, READY));
        story
    }
}

impl Story {
    #[must_use]
    pub fn thread_activity(&self, handle: jig_client::Handle, task: u64) -> u64 {
        match self.client.document(handle).and_then(|d| d.latest) {
            Some(jig_client::Latest::Jig(jig_client::contract::Delivery::Task(document)))
                if document.chip.task == task =>
            {
                document.activity
            }
            _ => 0,
        }
    }
}

impl Story {
    /// Complete store ownership for this committed turn alone and joined to its activity.
    #[must_use]
    pub fn turn_row_costs(&self, archive: &jig_client::contract::ArchivedTurn, body: Box<[u8]>) -> (u64, u64) {
        use jig_domain_core::{self as core, reads};
        use jig_domain_tasks as tasks;
        let world = &self.engine.driver.world;
        let owner = jig_core_world::fixtures::live_row(world, archive.task).clone();
        let turn = core::Record::Core(core::CoreRecord::Turn(core::TurnRecord {
            opens_window: true,
            task: archive.task,
            attempt: archive.name.attempt,
            turn: archive.name.number,
            spent: archive.cumulative,
            read: archive.read,
            at: archive.at,
            transcript: body,
        }));
        let mut row = reads::Row {
            value: turn,
            owner: Some(Box::new(owner)),
            ancestors: Box::new([]),
            related: None,
            unjoined: None,
            ended_read: None,
            linked: Box::new([]),
            inbox: None,
        };
        let single = row.owned_bytes().expect("single row cost");
        let line = world
            .store
            .rows
            .values()
            .find_map(|record| match record {
                jig_test_domain::Record::Core(core::Record::Tasks(tasks::Stored::Activity(line)))
                    if line.task == archive.task
                        && matches!(line.line, tasks::Line::Turn { attempt, turn, .. }
                    if attempt == archive.name.attempt && turn == archive.name.number) =>
                {
                    Some(line.clone())
                }
                jig_test_domain::Record::Core(_) | jig_test_domain::Record::Connector { .. } => None,
            })
            .expect("durable turn activity");
        row.related = Some(row.value);
        row.value = core::Record::Tasks(tasks::Stored::Activity(line));
        (single, row.owned_bytes().expect("activity row cost"))
    }
    pub fn hide_turn_task(&mut self) {
        self.engine.driver.world.send(jig_test_domain::Event::Core(jig_domain_core::Event::People(
            jig_domain_people::Event::Roles { project: 1, holdings: Box::new([]) },
        )));
        self.advance();
    }
}
