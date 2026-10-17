//! Durable engine categories and complete refusal facts; client/domain/contract.md, 4.
use super::Words;
use crate::phrases::jig::JigPhrase;
use jig_client::contract::{AdmissionLimit, CurrentHold, DisposalIssue, HeldRole, Limit, Moved, Refusal};
use skein_ui::Builder;
use skein_ui::phrases::Hole;
impl Words {
    /// Say the supplied refusal, never deriving permission or numerical limits.
    pub fn refusal(&self, b: &mut Builder, refusal: &Refusal) {
        match refusal {
            Refusal::Role { held } => {
                self.text(b, JigPhrase::RoleNotAllowed, &[]);
                self.held_role(b, *held);
            }
            Refusal::Disposal { task, issue } => {
                let phrase = match issue {
                    DisposalIssue::Missing => JigPhrase::DisposalMissing,
                    DisposalIssue::Unknown => JigPhrase::DisposalUnknown,
                    DisposalIssue::Repeated => JigPhrase::DisposalRepeated,
                };
                self.text(b, phrase, &[Hole::Number(*task)]);
            }
            Refusal::Taken { by } => self.text(b, JigPhrase::RefusalTakenBy, &[Hole::Number(*by)]),
            Refusal::NotWaiting => self.text(b, JigPhrase::NotWaitingForYou, &[]),
            Refusal::NoFurther => self.text(b, JigPhrase::NoFurtherHolder, &[]),
            Refusal::Authority { lacking, proposable } => {
                self.lacking(b, lacking);
                if let Some(false) = proposable {
                    self.text(b, JigPhrase::NobodyMayAccept, &[]);
                }
            }
            Refusal::Limit { which } => self.limit(b, *which),
            Refusal::Moved { now, what } => {
                let phrase = match what {
                    Moved::Hold { .. } => JigPhrase::MovedHold,
                    Moved::Spec => JigPhrase::MovedSpec,
                    Moved::PersonTask => JigPhrase::MovedPerson,
                    Moved::Note => JigPhrase::MovedNote,
                    Moved::Waiting => JigPhrase::MovedWaiting,
                };
                self.text(b, phrase, &[]);
                self.text(b, JigPhrase::Revision, &[Hole::Number(now.value)]);
                match what {
                    Moved::Hold { reason } => {
                        if let Some(reason) = reason {
                            match reason {
                                CurrentHold::Document(reason) => self.hold_fact(b, reason),
                                CurrentHold::Reason(reason) => self.hold_category(b, *reason),
                            }
                        }
                    }
                    Moved::Spec | Moved::PersonTask | Moved::Note | Moved::Waiting => {}
                }
            }
            Refusal::Ended { how } => self.ending(b, how),
            Refusal::Unknown => self.text(b, JigPhrase::NoObject, &[]),
            Refusal::NotOffered => self.text(b, JigPhrase::NotOffered, &[]),
            Refusal::KeyConflict => self.text(b, JigPhrase::KeyConflict, &[]),
        }
    }
    pub(crate) fn held_role(&self, b: &mut Builder, held: Option<HeldRole>) {
        match held {
            None => self.text(b, JigPhrase::NoHeldRole, &[]),
            Some(HeldRole::Owner) => self.text(b, JigPhrase::HeldOwner, &[]),
            Some(HeldRole::Maintainer) => self.text(b, JigPhrase::HeldMaintainer, &[]),
            Some(HeldRole::Member) => self.text(b, JigPhrase::HeldMember, &[]),
            Some(HeldRole::Observer) => self.text(b, JigPhrase::HeldObserver, &[]),
            Some(HeldRole::Policy { role }) => self.text(b, JigPhrase::HeldPolicy, &[Hole::Number(u64::from(role))]),
        }
    }
    fn limit(&self, b: &mut Builder, which: Limit) {
        match which {
            Limit::Admission { which } => self.engine_limit(b, which),
            Limit::LiveTasks { project, limit } => {
                self.text(b, JigPhrase::LimitLive, &[Hole::Number(u64::from(project)), Hole::Number(u64::from(limit))]);
            }
            Limit::Inbox { task, limit } => {
                self.text(b, JigPhrase::LimitInbox, &[Hole::Number(task), Hole::Number(u64::from(limit))]);
            }
            Limit::Words { limit } => self.text(b, JigPhrase::LimitWords, &[Hole::Number(u64::from(limit))]),
            Limit::ConversationWords { limit } => {
                self.text(b, JigPhrase::LimitConversation, &[Hole::Number(u64::from(limit))]);
            }
            Limit::Fields => self.text(b, JigPhrase::LimitFields, &[]),
            Limit::Records => self.text(b, JigPhrase::LimitRecords, &[]),
            Limit::Requests => self.text(b, JigPhrase::LimitRequests, &[]),
            Limit::Reads => self.text(b, JigPhrase::LimitReads, &[]),
            Limit::Watches => self.text(b, JigPhrase::LimitWatches, &[]),
            Limit::Objects => self.text(b, JigPhrase::LimitObjects, &[]),
        }
    }
    fn engine_limit(&self, b: &mut Builder, which: AdmissionLimit) {
        let phrase = match which {
            AdmissionLimit::Words => JigPhrase::EngineLimitWords,
            AdmissionLimit::Identity => JigPhrase::EngineLimitIdentity,
            AdmissionLimit::Roles => JigPhrase::EngineLimitRoles,
            AdmissionLimit::Arithmetic => JigPhrase::EngineLimitArithmetic,
            AdmissionLimit::Duplicate => JigPhrase::EngineLimitDuplicate,
            AdmissionLimit::Empty => JigPhrase::EngineLimitEmpty,
            AdmissionLimit::Batch => JigPhrase::EngineLimitBatch,
            AdmissionLimit::ProjectLiveTasks => JigPhrase::EngineLimitProjectLiveTasks,
            AdmissionLimit::LiveTasks => JigPhrase::EngineLimitLiveTasks,
            AdmissionLimit::Tree => JigPhrase::EngineLimitTree,
            AdmissionLimit::Depth => JigPhrase::EngineLimitDepth,
            AdmissionLimit::Delegates => JigPhrase::EngineLimitDelegates,
            AdmissionLimit::Subscription => JigPhrase::EngineLimitSubscription,
            AdmissionLimit::Dependencies => JigPhrase::EngineLimitDependencies,
            AdmissionLimit::Cycle => JigPhrase::EngineLimitCycle,
            AdmissionLimit::Spec => JigPhrase::EngineLimitSpec,
            AdmissionLimit::Contract => JigPhrase::EngineLimitContract,
            AdmissionLimit::Authority => JigPhrase::EngineLimitAuthority,
            AdmissionLimit::Inputs => JigPhrase::EngineLimitInputs,
            AdmissionLimit::Attempts => JigPhrase::EngineLimitAttempts,
            AdmissionLimit::LiveDelegates => JigPhrase::EngineLimitLiveDelegates,
            AdmissionLimit::Restore => JigPhrase::EngineLimitRestore,
            AdmissionLimit::Read => JigPhrase::EngineLimitRead,
            AdmissionLimit::Turn => JigPhrase::EngineLimitTurn,
            AdmissionLimit::HoldKind => JigPhrase::EngineLimitHoldKind,
            AdmissionLimit::HoldTaken => JigPhrase::EngineLimitHoldTaken,
            AdmissionLimit::Holds => JigPhrase::EngineLimitHolds,
            AdmissionLimit::InboxFull => JigPhrase::EngineLimitInboxFull,
            AdmissionLimit::Notes => JigPhrase::EngineLimitNotes,
            AdmissionLimit::Revision => JigPhrase::EngineLimitRevision,
            AdmissionLimit::Question => JigPhrase::EngineLimitQuestion,
            AdmissionLimit::Priority => JigPhrase::EngineLimitPriority,
            AdmissionLimit::Policy => JigPhrase::EngineLimitPolicy,
            AdmissionLimit::Resource => JigPhrase::EngineLimitResource,
            AdmissionLimit::Service => JigPhrase::EngineLimitService,
        };
        self.text(b, phrase, &[]);
    }
}
