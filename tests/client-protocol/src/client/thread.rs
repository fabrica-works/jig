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
        let body = include_bytes!("../../../../crates/jig-protocol-transcript/tests/data/record_turn_full.bin");
        Self::reading_chat_with(seed, Box::from(body.as_slice()), fixtures::domain_limits(), fixtures::owner_limits())
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
