//! Template helper inputs read from delivered public documents.
use super::{Story, client, fixtures};
use jig_client::{Asker, Handle, Latest, Watchable, contract as c};
use skein_lib::{Duration, Wall};
const ME: Handle = Handle { asker: Asker::JIG, number: 20 };
const DELEGATE: Handle = Handle { asker: Asker::JIG, number: 24 };
const READY: Duration = Duration::from_secs(1);
impl Story {
    /// The existing owner policy with a finite first deadline and an unbounded ceiling.
    #[must_use]
    pub fn live_span_goal(seed: u64, first: Wall) -> Self {
        let mut config = jig_core_world::world::config(seed);
        let mut policy = config.core.seed_projects[0].policy.policy.clone();
        policy.roles[0].authority.budget.deadline = Some(first);
        config.core.seed_projects[0].policy.policy = policy;
        let mut story = Self::configured(seed, &fixtures::owner_limits(), config);
        story.sign_in();
        assert!(story.run_until(|s| s.client.link().state == client::Link::Live, READY));
        story
    }
    pub fn shapes_me(&mut self) -> c::Me {
        self.watch(ME, Watchable::Jig(c::Subject::Me));
        assert!(self.run_until(|s| s.me().is_some(), READY));
        self.me().expect("delivered me").clone()
    }
    #[must_use]
    pub fn wall(&self) -> Wall {
        fixtures::wall(self.now)
    }
    pub fn watch_delegate(&mut self, task: u64) {
        self.watch(DELEGATE, Watchable::Jig(c::Subject::Task { task }));
    }
    #[must_use]
    pub fn delegate_requester(&self) -> Option<c::OwnerRequester> {
        match self.client.document(DELEGATE)?.latest {
            Some(Latest::Jig(c::Delivery::Task(task))) => Some(task.requester),
            _ => None,
        }
    }
    pub fn shapes_chat(&mut self, key: client::Key) -> u64 {
        let me = self.shapes_me();
        let words = c::Text::new(b"Hello".as_slice().into(), 32).expect("bounded words");
        let ask = client::shapes::chat(&me, 1, words).expect("chat offered");
        self.send(key, client::Ask::Jig(Box::new(ask)));
        assert!(self.run_until(|s| !s.answered(key).is_empty(), READY));
        let answers = self.answered(key);
        let [
            client::Outcome::Answered(client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(
                c::OwnerOutcome::Created { first, last, given },
            )))),
        ] = answers.as_slice()
        else {
            panic!("chat created: {answers:?}");
        };
        assert_eq!(first, last);
        assert!(given.is_some());
        *first
    }
}

impl Story {
    #[must_use]
    pub fn requester_chat(seed: u64) -> Self {
        let mut config = jig_core_world::world::config(seed);
        config.core.defaults.templates[0].flags.ended_by = jig_domain_tasks::EndedBy::Requester;
        let mut story = Self::configured(seed, &fixtures::owner_limits(), config);
        story.sign_in();
        assert!(story.run_until(|s| s.client.link().state == client::Link::Live, READY));
        story
    }
}
