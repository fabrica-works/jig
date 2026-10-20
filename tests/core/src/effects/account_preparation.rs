//! Accounts component and transcript fixture responses for admission stories.
use super::{World, accounts, core, root};
use skein_lib::Duration;

/// The subject and display name every sign-in at the fake accounts' component answers with.
const SIGNED_IN: &[u8] = b"signed-in";
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

    /// The fake component's account sign-in and sign-out: each
    /// provider's sign-in answers with [`SIGNED_IN`]'s subject, and each revocation is confirmed.
    pub(super) fn account_signing(&mut self, signing: accounts::Signing) {
        let event = match signing {
            accounts::Signing::SignIn { source, .. } => {
                let visit = accounts::Event::Visited {
                    source,
                    address: format!("https://issuer.test/visit/{source}").into_bytes().into_boxed_slice(),
                };
                self.events.push_back(root::Event::Core(core::Event::Account(visit)));
                accounts::Event::SignedIn {
                    source,
                    subject: Box::from(SIGNED_IN),
                    display: Box::from(SIGNED_IN),
                    declined: false,
                }
            }
            accounts::Signing::SignOut { source } => accounts::Event::SignedOut { source, confirmed: true },
            accounts::Signing::Replaced { .. } => return,
        };
        self.events.push_back(root::Event::Core(core::Event::Account(event)));
    }
}
