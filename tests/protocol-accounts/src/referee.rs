//! Outside grant and settlement checks shared by the stories.
use crate::{
    fixtures,
    owner::{Heard, Trace},
    world::{Process, Story},
};
use skein_lib::{Duration, Time};
use skein_world::{Host, Referee};

#[derive(Debug)]
pub struct Judge {
    story: Story,
    wake: Option<Time>,
    deadline: Option<Time>,
    rejected: bool,
    closing: bool,
    shutdown: bool,
    passed: Option<()>,
}
impl Judge {
    #[must_use]
    pub fn new(story: Story) -> Self {
        Self { story, wake: None, deadline: None, rejected: false, closing: false, shutdown: false, passed: None }
    }
}
impl Referee<Process> for Judge {
    fn act(&mut self, now: Time, processes: &mut [Process]) {
        self.deadline.get_or_insert(now.saturating_add(Duration::from_secs(180)));
        let address = processes.iter().find_map(|process| match process {
            Process::Issuer(peer) => peer.address(),
            Process::Owner(_) => None,
        });
        let close_exchange = self.story == Story::ClosedExchange && crate::world::exchange_seen(processes);
        let settled = processes.iter().any(|process| matches!(process, Process::Owner(owner) if owner.is_empty()));
        for process in processes {
            match process {
                Process::Owner(owner) => {
                    owner.address = address;
                    if fixtures::rejects(self.story) && owner.child_heard().len() == 1 && !self.rejected {
                        owner.reject();
                        self.rejected = true;
                    }
                    let wanted = fixtures::wanted(self.story);
                    if (owner.child_heard().len() >= wanted || close_exchange) && !self.closing {
                        owner.close();
                        self.closing = true;
                    }
                }
                Process::Issuer(peer) if settled && !self.shutdown => {
                    peer.shutdown();
                    self.shutdown = true;
                }
                Process::Issuer(_) => (),
            }
        }
    }
    fn observe(&mut self, now: Time, processes: &[Process]) {
        let owner = processes
            .iter()
            .find_map(|process| match process {
                Process::Owner(owner) => Some(owner),
                Process::Issuer(_) => None,
            })
            .expect("owner");
        for heard in owner.child_heard() {
            if let Heard::Granted { account, generation } = heard
                && !matches!(owner.child_heard().last(), Some(Heard::Failed { .. }))
            {
                assert!(
                    owner.table_value(*account, *generation).is_some(),
                    "every granted generation is readable until expiry"
                );
            }
        }
        if owner.closed() {
            assert_eq!(owner.trace().last(), Some(&Trace::Closed), "Closed is last");
            assert_eq!(
                owner.became_usable(),
                self.story != Story::DamagedRecord,
                "the real child accepted its grant only with a readable record"
            );
            if self.story == Story::ClosedExchange {
                assert!(crate::world::exchange_finished(processes), "Closed follows the exchange's successful answer");
            }
            let expected = fixtures::expected(self.story);
            assert_eq!(owner.child_heard(), expected);
        }
        self.wake = ((owner.unstarted()
            && processes.iter().any(|process| matches!(process, Process::Issuer(peer) if peer.address().is_some())))
            || (owner.is_empty() && !self.shutdown)
            || (owner.child_heard().len() >= fixtures::wanted(self.story) && !self.closing)
            || (self.story == Story::ClosedExchange && crate::world::exchange_seen(processes) && !self.closing)
            || (owner.child_heard().len() == 1 && !self.rejected && fixtures::rejects(self.story)))
        .then_some(now);
        self.passed = (self.shutdown && processes.iter().all(Host::is_empty)).then_some(());
    }
    fn next_deadline(&self) -> Option<Time> {
        self.wake.or(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (self.deadline.is_some_and(|due| now >= due) && self.passed.is_none())
            .then(|| "accounts world did not settle".to_owned())
    }
    fn passed(&self) -> bool {
        self.passed.is_some()
    }
}
