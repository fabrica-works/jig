//! Closed owner facts and separate transport dispositions (contract.md, 4).
//! Uses the actual whole owner carrier; successful owner translation is kept separate from transport-only dispositions.
use jig_client_domain::contract as client;
use jig_domain_people as people;
use skein_lib::Token;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Owner(client::OwnerOutcome),
    Applied { connector: u16, token: Token },
    Again(people::Again),
    SignedOut,
    Refused(people::Refusal),
}

#[must_use]
pub fn outcome(value: &people::Outcome) -> Outcome {
    let owner = match value {
        people::Outcome::Closing { chat, cancelled, kept } => {
            client::OwnerOutcome::Closing { chat: *chat, cancelled: *cancelled, kept: *kept }
        }
        people::Outcome::NoteEdited { name } => client::OwnerOutcome::NoteEdited { name: *name },
        people::Outcome::ServiceMade { person } => client::OwnerOutcome::ServiceMade { person: *person },
        people::Outcome::Adopted { project } => client::OwnerOutcome::Adopted { project: *project },
        people::Outcome::GoalStarted { task } => client::OwnerOutcome::GoalStarted { task: *task },
        people::Outcome::GoalProposed { proposal } => client::OwnerOutcome::GoalProposed { proposal: *proposal },
        people::Outcome::Stopped { task } => client::OwnerOutcome::Stopped { task: *task },
        people::Outcome::Cancelled { task } => client::OwnerOutcome::Cancelled { task: *task },
        people::Outcome::Released { task } => client::OwnerOutcome::Released { task: *task },
        people::Outcome::PersonTaken { task } => client::OwnerOutcome::PersonTaken { task: *task },
        people::Outcome::PersonHandedBack { task } => client::OwnerOutcome::PersonHandedBack { task: *task },
        people::Outcome::PersonAnswered { task } => client::OwnerOutcome::PersonAnswered { task: *task },
        people::Outcome::Moved { task } => client::OwnerOutcome::Moved { task: *task },
        people::Outcome::Said { task, message, released } => {
            client::OwnerOutcome::Said { task: *task, message: *message, released: *released }
        }
        people::Outcome::QuestionAnswered { task, question, message } => {
            client::OwnerOutcome::QuestionAnswered { task: *task, question: *question, message: *message }
        }
        people::Outcome::Prioritised { project } => client::OwnerOutcome::Prioritised { project: *project },
        people::Outcome::Amended { task, released } => {
            client::OwnerOutcome::Amended { task: *task, released: *released }
        }
        people::Outcome::AmendProposed { task, proposal, released } => {
            client::OwnerOutcome::AmendProposed { task: *task, proposal: *proposal, released: *released }
        }
        people::Outcome::RolesSet { project } => client::OwnerOutcome::RolesSet { project: *project },
        people::Outcome::PolicyChanged { project } => client::OwnerOutcome::PolicyChanged { project: *project },
        people::Outcome::PoolSet { project, person } => {
            client::OwnerOutcome::PoolSet { project: *project, person: *person }
        }
        people::Outcome::Started { task } => client::OwnerOutcome::Started { task: *task },
        people::Outcome::MarkedSeen { seen } => client::OwnerOutcome::MarkedSeen {
            seen: match seen {
                people::Seen::Results { through } => client::Seen::Results { through: *through },
                people::Seen::Chat { task, through } => client::Seen::Replies { chat: *task, through: *through },
            },
        },
        people::Outcome::ProposalDecided { proposer, proposal, by, at, choice } => {
            client::OwnerOutcome::ProposalDecided {
                proposal: client::ProposalKey {
                    proposer: match proposer {
                        people::Proposer::Task(number) => client::Proposer::Task(*number),
                        people::Proposer::Party(number) => client::Proposer::Party(*number),
                    },
                    number: *proposal,
                },
                by: *by,
                at: *at,
                choice: match choice {
                    people::ProposalChoice::Accepted => client::OwnerProposalChoice::Accepted,
                    people::ProposalChoice::Rejected => client::OwnerProposalChoice::Rejected,
                    people::ProposalChoice::Passed => client::OwnerProposalChoice::Passed,
                    people::ProposalChoice::Withdrawn => client::OwnerProposalChoice::Withdrawn,
                    people::ProposalChoice::Stale => client::OwnerProposalChoice::Stale,
                },
            }
        }
        people::Outcome::EscalationDecided { task, revision, by, at, choice } => {
            client::OwnerOutcome::EscalationDecided {
                task: *task,
                revision: client::Revision { value: *revision },
                by: *by,
                at: *at,
                choice: match choice {
                    people::EscalationChoice::Released => client::OwnerEscalationChoice::Released,
                    people::EscalationChoice::Rejected => client::OwnerEscalationChoice::Rejected,
                    people::EscalationChoice::Passed => client::OwnerEscalationChoice::Passed,
                },
            }
        }
        people::Outcome::Applied { connector, token } => {
            return Outcome::Applied { connector: *connector, token: *token };
        }
        people::Outcome::Again(why) => return Outcome::Again(*why),
        people::Outcome::SignedOut => return Outcome::SignedOut,
        people::Outcome::Refused(why) => return Outcome::Refused(*why),
    };
    Outcome::Owner(owner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::Wall;

    #[test]
    fn committed_choices_keep_equal_number_makers_full_winner_clocks_and_every_closed_result_kind() {
        let at = Wall::from_nanos(u64::MAX);
        for maker in [people::Proposer::Task(u64::MAX), people::Proposer::Party(u64::MAX)] {
            let proposer = match maker {
                people::Proposer::Task(n) => client::Proposer::Task(n),
                people::Proposer::Party(n) => client::Proposer::Party(n),
            };
            for (before, after) in [
                (people::ProposalChoice::Accepted, client::OwnerProposalChoice::Accepted),
                (people::ProposalChoice::Rejected, client::OwnerProposalChoice::Rejected),
                (people::ProposalChoice::Passed, client::OwnerProposalChoice::Passed),
                (people::ProposalChoice::Withdrawn, client::OwnerProposalChoice::Withdrawn),
                (people::ProposalChoice::Stale, client::OwnerProposalChoice::Stale),
            ] {
                assert_eq!(
                    outcome(&people::Outcome::ProposalDecided {
                        proposer: maker,
                        proposal: 41,
                        by: 37,
                        at,
                        choice: before
                    }),
                    Outcome::Owner(client::OwnerOutcome::ProposalDecided {
                        proposal: client::ProposalKey { proposer, number: 41 },
                        by: 37,
                        at,
                        choice: after
                    })
                );
            }
        }
        for (before, after) in [
            (people::EscalationChoice::Released, client::OwnerEscalationChoice::Released),
            (people::EscalationChoice::Rejected, client::OwnerEscalationChoice::Rejected),
            (people::EscalationChoice::Passed, client::OwnerEscalationChoice::Passed),
        ] {
            assert_eq!(
                outcome(&people::Outcome::EscalationDecided {
                    task: 99,
                    revision: u64::MAX,
                    by: 37,
                    at,
                    choice: before
                }),
                Outcome::Owner(client::OwnerOutcome::EscalationDecided {
                    task: 99,
                    revision: client::Revision { value: u64::MAX },
                    by: 37,
                    at,
                    choice: after
                })
            );
        }
    }

    #[test]
    fn owner_outcomes_keep_independent_messages_counts_seen_positions_and_released_flags_without_inventing_makers() {
        for released in [false, true] {
            assert_eq!(
                outcome(&people::Outcome::Said { task: u64::MAX, message: 17, released }),
                Outcome::Owner(client::OwnerOutcome::Said { task: u64::MAX, message: 17, released })
            );
            assert_eq!(
                outcome(&people::Outcome::Amended { task: 11, released }),
                Outcome::Owner(client::OwnerOutcome::Amended { task: 11, released })
            );
            assert_eq!(
                outcome(&people::Outcome::AmendProposed { task: 11, proposal: u64::MAX, released }),
                Outcome::Owner(client::OwnerOutcome::AmendProposed { task: 11, proposal: u64::MAX, released })
            );
        }
        assert_eq!(
            outcome(&people::Outcome::QuestionAnswered { task: 11, question: 19, message: u64::MAX }),
            Outcome::Owner(client::OwnerOutcome::QuestionAnswered { task: 11, question: 19, message: u64::MAX })
        );
        assert_eq!(
            outcome(&people::Outcome::Closing { chat: u64::MAX, cancelled: u32::MAX, kept: 0 }),
            Outcome::Owner(client::OwnerOutcome::Closing { chat: u64::MAX, cancelled: u32::MAX, kept: 0 })
        );
        assert_eq!(
            outcome(&people::Outcome::MarkedSeen { seen: people::Seen::Chat { task: 11, through: u64::MAX } }),
            Outcome::Owner(client::OwnerOutcome::MarkedSeen {
                seen: client::Seen::Replies { chat: 11, through: u64::MAX }
            })
        );
        assert_eq!(
            outcome(&people::Outcome::MarkedSeen { seen: people::Seen::Results { through: 0 } }),
            Outcome::Owner(client::OwnerOutcome::MarkedSeen { seen: client::Seen::Results { through: 0 } })
        );
        assert_eq!(
            outcome(&people::Outcome::GoalProposed { proposal: u64::MAX }),
            Outcome::Owner(client::OwnerOutcome::GoalProposed { proposal: u64::MAX })
        );
        for why in [people::Again::Busy, people::Again::NotReady] {
            assert_eq!(outcome(&people::Outcome::Again(why)), Outcome::Again(why));
        }
        assert_eq!(outcome(&people::Outcome::SignedOut), Outcome::SignedOut);
        let token = Token::new(u64::MAX);
        assert_eq!(
            outcome(&people::Outcome::Applied { connector: u16::MAX, token }),
            Outcome::Applied { connector: u16::MAX, token }
        );
        assert_eq!(
            outcome(&people::Outcome::Refused(people::Refusal::Role { held: None })),
            Outcome::Refused(people::Refusal::Role { held: None })
        );
    }
}
