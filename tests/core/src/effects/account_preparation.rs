//! Accounts component and transcript fixture responses for admission stories.
use super::{World, accounts, core, root};
use skein_lib::Duration;
impl World {
    pub(super) fn transcript_loaded(&mut self, task: u64) {
        if self.fail_account_before_workspace {
            self.fail_account_before_workspace = false;
            self.events.push_back(root::Event::Core(core::Event::Account(accounts::Event::Component(
                accounts::FromComponent::Failed { account: 1, failure: accounts::Failure::SignInAgain },
            ))));
        }
        let rows: Vec<_> = self
            .store
            .rows
            .values()
            .filter_map(|record| match record {
                root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) if turn.task == task => {
                    Some(turn.clone())
                }
                root::Record::Core(_) | root::Record::Connector { .. } => None,
            })
            .collect();
        for row in rows {
            self.events.push_back(root::Event::TranscriptLoaded { task, rows: Box::new([row]), done: false });
        }
        self.events.push_back(root::Event::TranscriptLoaded { task, rows: Box::new([]), done: true });
    }
}

impl World {
    pub(super) fn account_request(&mut self, request: accounts::ToComponent) {
        let response = match request {
            accounts::ToComponent::Want { account: 1 } if !self.hold_accounts => {
                Some(accounts::FromComponent::Granted { account: 1, generation: 1, valid: Duration::from_secs(60) })
            }
            accounts::ToComponent::Close => Some(accounts::FromComponent::Closed),
            accounts::ToComponent::Want { .. }
            | accounts::ToComponent::Release { .. }
            | accounts::ToComponent::Rejected { .. } => None,
        };
        if let Some(response) = response {
            self.events.push_back(root::Event::Core(core::Event::Account(accounts::Event::Component(response))));
        }
    }
}
