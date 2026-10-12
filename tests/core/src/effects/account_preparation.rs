//! Accounts component and transcript fixture responses for admission stories.
use super::{World, accounts, core, root};
use skein_lib::Duration;
impl World {
    pub(super) fn transcript_loaded(&mut self, task: u64, from: Option<(u64, u32)>) {
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
                root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn)))
                    if turn.task == task && from.is_some_and(|start| (turn.attempt, turn.turn) >= start) =>
                {
                    Some(turn.clone())
                }
                root::Record::Core(_) | root::Record::Connector { .. } => None,
            })
            .collect();
        if let Some(start) = from
            && !rows.first().is_some_and(|row| (row.attempt, row.turn) == start && row.opens_window)
        {
            self.events.push_back(root::Event::Core(core::Event::TranscriptUnread { task }));
            return;
        }
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
