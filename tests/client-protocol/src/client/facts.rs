//! Public document access for the helper stories, on a scripted worker's reports.
use super::{
    Duration, Engine, Event, Front, Handle, Latest, Link, List, Net, Story, Time, Watchable, client, fixtures,
};
use jig_client::contract as c;
use jig_fake_workers::{Call, Script, Worker};
const ME: Handle = Handle { asker: client::Asker::JIG, number: 20 };
const TASK: Handle = Handle { asker: client::Asker::JIG, number: 21 };
const LINES: Handle = Handle { asker: client::Asker::JIG, number: 22 };
const READY: Duration = Duration::from_secs(1);
const WRITING: Duration = Duration::from_millis(100);
const WORKER: u64 = 1;
const PROJECT: u32 = 1;
const TURN: u32 = 1;
const CHAT: u64 = 1;
impl Story {
    #[must_use]
    pub fn writing_chat(seed: u64) -> Self {
        let scripts = vec![
            Box::new([
                Script::Writing,
                Script::Text { turn: TURN, offset: 0, text: Box::from(b"Writing".as_slice()) },
                Script::Silence { for_: WRITING },
                Script::Call(Call::Delegate { words: Box::from(b"ask the requester".as_slice()), budget: 10 }),
                Script::Park,
            ]) as Box<[Script]>,
            Box::new([
                Script::Call(Call::Question { target: CHAT, words: Box::from(b"Which next?".as_slice()) }),
                Script::Wait,
            ]),
        ];
        let peers = jig_core_world::peers::Peers::new(vec![Worker::new(WORKER, 1, scripts)], Vec::new());
        let mut config = jig_core_world::world::config(seed);
        let mut rules = config.core.authority.rules().clone();
        rules.maximum_run_spend = 20;
        let policy = config.core.authority.policy(PROJECT).expect("chat project policy").clone();
        config.core.authority = jig_domain_authority::Domain::new(rules, fixtures::owner_limits().core.authority)
            .expect("chat run budget leaves room for its delegate");
        let mut out = skein_lib::Queue::with_capacity(jig_domain_authority::POLICY_MAX_OUT);
        jig_domain_authority::step(
            &mut config.core.authority,
            jig_domain_authority::Event::Policy { project: PROJECT, policy },
            &mut out,
        );
        assert_eq!(out.pop(), Some(jig_domain_authority::PolicyFact::Added { project: PROJECT }));
        let mut story = Self {
            front: Front::new(seed),
            second: None,
            engine: Engine::with_peers(seed, &fixtures::owner_limits(), config, peers),
            net: Net::new(seed),
            now: Time::ZERO,
            seen: Vec::new(),
        };
        story.event(Event::Start { kept: List::with_capacity(0) });
        story.advance();
        story.sign_in();
        assert!(story.run_until(|s| s.client.link().state == Link::Live, READY));
        story.watch(ME, Watchable::Jig(c::Subject::Me));
        story
    }
    pub fn watch_chat_facts(&mut self, task: u64) {
        self.watch(TASK, Watchable::Jig(c::Subject::Task { task }));
        let query = c::TaskQuery { tracked: None, ..fixtures::task_query(PROJECT) };
        self.watch(LINES, Watchable::Jig(c::Subject::Tasks { query, window: 2 }));
    }
    #[must_use]
    pub fn me(&self) -> Option<&c::Me> {
        match self.client.document(ME)?.latest {
            Some(Latest::Jig(c::Delivery::Me(me))) => Some(me),
            _ => None,
        }
    }
    #[must_use]
    pub fn line(&self, task: u64) -> Option<&c::TaskLine> {
        fixtures::task_lines(self, LINES)?.rows.iter().find(|row| row.chip.task == task)
    }
    #[must_use]
    pub fn chat_document(&self) -> Option<&c::Task> {
        match self.client.document(TASK)?.latest {
            Some(Latest::Jig(c::Delivery::Task(task))) => Some(task),
            _ => None,
        }
    }
    #[must_use]
    pub fn chat_statuses(&self, task: u64) -> Option<(client::facts::ChatStatus, client::facts::ChatStatus)> {
        let me = self.me()?;
        let page = self.chat_document()?;
        let line = self.line(task)?;
        Some((
            client::facts::chat_status(&page.chip, &page.waits_on, page.project, me),
            client::facts::chat_status(&line.chip, &line.waits_on, line.project, me),
        ))
    }
}
