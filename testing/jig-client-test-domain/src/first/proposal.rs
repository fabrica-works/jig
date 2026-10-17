//! The application's proposal confirmation and public client request (testing.md, 4).
use super::{Request, Side};
use alloc::boxed::Box;
use jig_client::{
    Asker, Key,
    contract::{Ask, DecideOn, Decision, ProposalKey, Text},
};
use skein_lib::Queue;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    pub project: u32,
    pub proposal: ProposalKey,
    pub accept: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Review(Choice),
    Confirm,
    Dismiss,
}
impl Side {
    #[must_use]
    pub const fn proposal_confirmation(&self) -> Option<Choice> {
        self.proposal
    }
}
pub(super) fn act(side: &mut Side, act: Act, out: &mut Queue<Request>) {
    match act {
        Act::Review(choice) => side.proposal = Some(choice),
        Act::Dismiss => side.proposal = None,
        Act::Confirm => {
            if let Some(choice) = side.proposal.take() {
                side.proposal_sequence =
                    side.proposal_sequence.checked_add(1).expect("application request sequence fits");
                let decision = if choice.accept {
                    Decision::Accepted
                } else {
                    Decision::Rejected { reason: Text::new(b"Rejected".as_slice().into(), 8).expect("bounded reason") }
                };
                out.push(Request::Client {
                    asked: jig_client::Event::Send {
                        key: Key { asker: Asker(3), number: side.proposal_sequence },
                        request: jig_client::Ask::Jig(Box::new(Ask::Decide {
                            project: choice.project,
                            on: DecideOn::Proposal { proposal: choice.proposal, decision },
                        })),
                    },
                });
            }
        }
    }
    side.changed();
}
