//! The scripted host hears grants in journal release order.
use super::{World, accounts, core, fleet, root, tasks};
use skein_lib::{Duration, Token};

impl World {
    /// The last grant received by the host, with its fenced destination.
    #[must_use]
    pub fn received_grant(&self) -> Option<(Token, Token, Token, fleet::Grant)> {
        self.received_grant
    }

    /// Whether the last grant followed an assignment this host had received.
    #[must_use]
    pub fn grant_follows_assignment(&self) -> bool {
        self.grant_follows_assignment
    }

    /// Let the component renew when placement submits its still-held assignment.
    pub fn renew_on_assignment(&mut self, account: u32, generation: u64, form: accounts::Form) {
        self.renew_on_running =
            Some(accounts::FromComponent::Granted { form, account, generation, valid: Duration::from_secs(60) });
        self.store_delay = 5;
    }

    pub(super) fn inject_assignment_renewal(&mut self, writes: &[root::Write]) {
        if self.renew_on_running.is_none() {
            return;
        }
        let placing = writes.iter().any(|write| {
            matches!(write,
                root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task))))
                if matches!(task.phase, tasks::Phase::Active(tasks::Active::Running { .. }))
            )
        });
        if placing {
            let response = self.renew_on_running.take().expect("scripted renewal");
            self.events.push_back(root::Event::Core(core::Event::Account(accounts::Event::Component(response))));
        }
    }

    pub(super) fn grant_delivered(&mut self, channel: Token, run: Token, attempt: Token, grant: fleet::Grant) {
        self.grant_follows_assignment = self.assignments.contains(&(run.raw(), attempt.raw()));
        self.received_grant = Some((channel, run, attempt, grant));
    }
}

impl World {
    /// Announce the source's form before placement starts its run.
    pub fn announce_grant(&mut self, account: u32, generation: u64, form: accounts::Form) {
        self.send(root::Event::Core(core::Event::Account(accounts::Event::Component(
            accounts::FromComponent::Granted { account, generation, valid: Duration::from_secs(60), form },
        ))));
    }
}
