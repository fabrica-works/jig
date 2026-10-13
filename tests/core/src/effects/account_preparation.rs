//! Accounts component and transcript fixture responses for admission stories.
use super::{World, accounts, core, root};
use skein_lib::Duration;
impl World {
    pub(super) fn transcript_loaded(&mut self, now: core::Now) {
        if self.fail_account_before_workspace {
            self.fail_account_before_workspace = false;
            self.events.push_back(root::Event::Core(core::Event::Account(accounts::Event::Component(
                accounts::FromComponent::Failed { account: 1, failure: accounts::Failure::SignInAgain },
            ))));
        }
        if let core::Now::StartPreparation { task, from, .. } = &now {
            if from.is_none() {
                self.events.push_back(root::Event::TranscriptLoaded { task: *task, rows: Box::new([]), done: true });
            } else {
                self.store.ask(super::StoreOut::Now(now));
            }
        } else {
            panic!("preparation route");
        }
    }
}

impl World {
    pub(super) fn account_request(&mut self, request: accounts::ToComponent) {
        let response = match request {
            accounts::ToComponent::Want { account: 1 } if !self.hold_accounts => {
                Some(accounts::FromComponent::Granted { account: 1, generation: 1, valid: Duration::from_secs(60) })
            }
            accounts::ToComponent::Rejected { account, generation } => Some(accounts::FromComponent::Granted {
                account,
                generation: generation.checked_add(1).expect("fixture generation has a successor"),
                valid: Duration::from_secs(60),
            }),
            accounts::ToComponent::Close => Some(accounts::FromComponent::Closed),
            accounts::ToComponent::Want { .. } | accounts::ToComponent::Release { .. } => None,
        };
        if let Some(response) = response {
            self.events.push_back(root::Event::Core(core::Event::Account(accounts::Event::Component(response))));
        }
    }
}
