//! Complete moved-hold facts without requiring an object slot; contract.md, 4.
use super::{Words, name};
use crate::phrases::{Hole, jig::JigPhrase};
use crate::ui::Builder;
use jig_domain_client::contract::{HoldCategory, HoldFailure, HoldReason, Part};
impl Words {
    /// Admission reason facts, without invented amounts, actors, clocks or parts.
    pub(super) fn hold_category(&self, b: &mut Builder, reason: HoldCategory) {
        match reason {
            HoldCategory::Failures { class } => {
                self.text(b, JigPhrase::TriesExhausted, &[]);
                let phrase = match class {
                    HoldFailure::Transient => JigPhrase::Temporary,
                    HoldFailure::Permanent => JigPhrase::Permanent,
                    HoldFailure::Run => JigPhrase::FailureRun,
                    HoldFailure::Agent => JigPhrase::FailureAgent,
                    HoldFailure::Lost => JigPhrase::HostLost,
                    HoldFailure::Invalid => JigPhrase::FailureInvalid,
                };
                self.text(b, phrase, &[]);
            }
            HoldCategory::Stalled => self.text(b, JigPhrase::HoldStalled, &[]),
            HoldCategory::Dependency { task } => self.text(b, JigPhrase::HoldDependency, &[Hole::Number(task)]),
            HoldCategory::Stopped => self.text(b, JigPhrase::HoldStopped, &[]),
            HoldCategory::StoppedBy { party } => {
                self.text(b, JigPhrase::HoldStopped, &[]);
                self.actor(b, jig_domain_client::contract::Actor::Party(party));
            }
            HoldCategory::Drift => self.text(b, JigPhrase::HoldDrift, &[]),
            HoldCategory::Effects => self.text(b, JigPhrase::HoldEffects, &[]),
            HoldCategory::EffectFailed => self.text(b, JigPhrase::HoldEffectFailed, &[]),
            HoldCategory::Uncertain { entry } => self.text(b, JigPhrase::HoldUncertainEntry, &[Hole::Number(entry)]),
            HoldCategory::Budget => self.text(b, JigPhrase::HoldBudget, &[]),
            HoldCategory::Deadline => self.text(b, JigPhrase::HoldDeadline, &[]),
            HoldCategory::HoldsWaited => self.text(b, JigPhrase::HoldWaited, &[]),
            HoldCategory::ResourceUnavailable => self.text(b, JigPhrase::ResourceUnavailable, &[]),
            HoldCategory::ResourceAccess => self.text(b, JigPhrase::ResourceAccessMissing, &[]),
            HoldCategory::PoolLost { pool } => self.text(b, JigPhrase::HoldPoolLost, &[Hole::Number(pool)]),
            HoldCategory::Procedure => self.text(b, JigPhrase::HoldProcedure, &[]),
        }
    }

    /// Show the exact current hold reason supplied with a moved refusal.
    pub(super) fn hold_fact(&self, b: &mut Builder, reason: &HoldReason) {
        match reason {
            HoldReason::Tries { failures } => {
                self.text(b, JigPhrase::TriesExhausted, &[]);
                for failure in failures {
                    self.text(b, JigPhrase::Attempt, &[Hole::Number(failure.attempt)]);
                    crate::cards::chip::failure(b, failure.class, self);
                    self.text(b, JigPhrase::Reason, &[name(failure.reason.bytes())]);
                    self.wall(b, failure.at);
                }
            }
            HoldReason::Stalled { since, waiting } => {
                self.text(b, JigPhrase::StalledOn, &[name(waiting.bytes())]);
                self.text(b, JigPhrase::Since, &[Hole::Wall(self.adjusted_wall(*since))]);
            }
            HoldReason::Dependency { task, how } => {
                self.text(b, JigPhrase::DependencyEnded, &[Hole::Number(*task)]);
                self.ending(b, how);
            }
            HoldReason::Drift { part } => {
                self.text(b, JigPhrase::DriftHold, &[]);
                self.part_fact(b, part);
            }
            HoldReason::EffectFailed { part } => {
                self.text(b, JigPhrase::EffectFailedHold, &[]);
                self.part_fact(b, part);
            }
            HoldReason::EffectUncertain { part } => {
                self.text(b, JigPhrase::EffectUncertainHold, &[]);
                self.part_fact(b, part);
            }
            HoldReason::Budget { spent, limit } => {
                self.text(b, JigPhrase::BudgetHold, &[Hole::Amount(*spent), Hole::Amount(*limit)]);
            }
            HoldReason::Deadline { at } => {
                self.text(b, JigPhrase::DeadlineHold, &[Hole::Wall(self.adjusted_wall(*at))]);
            }
            HoldReason::Stopped { by, at } => {
                self.text(b, JigPhrase::StoppedHold, &[name(by.name.bytes()), Hole::Wall(self.adjusted_wall(*at))]);
            }
            HoldReason::Resource { resource, holder } => {
                self.text(b, JigPhrase::ResourceHold, &[]);
                self.part_fact(b, resource);
                self.actor(b, *holder);
            }
            HoldReason::Pool { part } => {
                self.text(b, JigPhrase::PoolHold, &[]);
                self.part_fact(b, part);
            }
            HoldReason::Procedure { part } => {
                self.text(b, JigPhrase::ProcedureHold, &[]);
                self.part_fact(b, part);
            }
        }
    }
    fn part_fact(&self, b: &mut Builder, part: &Part) {
        let connector = match part {
            Part::Connector { connector }
            | Part::Resource { connector, .. }
            | Part::Kind { connector, .. }
            | Part::Panel { connector, .. }
            | Part::Result { connector, .. }
            | Part::Effect { connector, .. }
            | Part::OwnerEffect { connector, .. }
            | Part::Outbox { connector, .. }
            | Part::Hold { connector, .. }
            | Part::Project { connector, .. }
            | Part::Deployment { connector } => *connector,
        };
        self.text(b, JigPhrase::PartConnector, &[Hole::Number(u64::from(connector))]);
        match part {
            Part::Connector { .. } => {}
            Part::Deployment { .. } => self.text(b, JigPhrase::Deployment, &[]),
            Part::Resource { path, .. } => {
                self.text(b, JigPhrase::ResourceSegments, &[]);
                for segment in path {
                    crate::literal_segment::render(b, segment);
                }
            }
            Part::Kind { kind, .. } => self.text(b, JigPhrase::PartKind, &[Hole::Number(u64::from(*kind))]),
            Part::Panel { task, .. } => {
                self.text(b, JigPhrase::PanelPart, &[]);
                self.text(b, JigPhrase::PartTask, &[Hole::Number(*task)]);
            }
            Part::Result { task, .. } => {
                self.text(b, JigPhrase::ResultPart, &[]);
                self.text(b, JigPhrase::PartTask, &[Hole::Number(*task)]);
            }
            Part::Hold { task, .. } => {
                self.text(b, JigPhrase::HoldPart, &[]);
                self.text(b, JigPhrase::PartTask, &[Hole::Number(*task)]);
            }
            Part::Outbox { scope, entry, .. } => {
                match scope {
                    jig_domain_client::contract::PartScope::Project(project) => {
                        self.text(b, JigPhrase::PartProject, &[Hole::Number(u64::from(*project))]);
                    }
                    jig_domain_client::contract::PartScope::Deployment => self.text(b, JigPhrase::Deployment, &[]),
                }
                self.text(b, JigPhrase::OutboxEntry, &[Hole::Number(*entry)]);
            }
            Part::Project { project, .. } => self.text(b, JigPhrase::PartProject, &[Hole::Number(u64::from(*project))]),
            Part::OwnerEffect { proposal, .. } => {
                self.text(b, JigPhrase::PartOwnerProposal, &[Hole::Number(*proposal)]);
            }
            Part::Effect { proposal, .. } => {
                let (phrase, maker) = match proposal.proposer {
                    jig_domain_client::contract::Proposer::Task(task) => (JigPhrase::PartProposalTask, task),
                    jig_domain_client::contract::Proposer::Party(party) => (JigPhrase::PartProposalParty, party),
                };
                self.text(b, phrase, &[Hole::Number(maker), Hole::Number(proposal.number)]);
            }
        }
    }
}
