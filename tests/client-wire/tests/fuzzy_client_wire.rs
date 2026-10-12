//! Arbitrary and mutated client bytes (skein's codec.md, section 6).
use jig_protocol_client_wire::v1::*;
use skein_lib::{Reader, Rng, Writer};
use std::{fs, path::Path};

macro_rules! check {
    ($record:ident, $bytes:expr) => {
        if let Ok(record) = $record::decode(&CEILINGS, &mut Reader::new($bytes)) {
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length"));
            record.encode(&mut writer).expect("measured room");
            assert_eq!(writer.finish().as_ref(), $bytes, stringify!($record));
        }
    };
}

fn records_1(bytes: &[u8]) {
    check!(Family, bytes);
    check!(Owner, bytes);
    check!(Keyed, bytes);
    check!(Owned, bytes);
    check!(Answer, bytes);
    check!(LinkSignIn, bytes);
    check!(SignedIn, bytes);
    check!(SignInAnswer, bytes);
    check!(SignOut, bytes);
    check!(SignOutAnswer, bytes);
    check!(DocumentText, bytes);
    check!(ExecutorAgent, bytes);
    check!(ExecutorProcedure, bytes);
    check!(ExecutorParty, bytes);
    check!(ExecutorRole, bytes);
    check!(Executor, bytes);
    check!(Phase, bytes);
    check!(RunStageClaimed, bytes);
    check!(RunStageRunning, bytes);
    check!(RunStageBackingOff, bytes);
    check!(RunStage, bytes);
    check!(PersonAddressParty, bytes);
    check!(PersonAddressRole, bytes);
    check!(PersonAddress, bytes);
    check!(ClosingStageRun, bytes);
    check!(ClosingStage, bytes);
    check!(HoldFailure, bytes);
    check!(HoldCategoryFailures, bytes);
    check!(HoldCategoryDependency, bytes);
    check!(HoldCategoryStoppedBy, bytes);
    check!(HoldCategoryUncertain, bytes);
    check!(HoldCategoryPoolLost, bytes);
    check!(HoldCategory, bytes);
    check!(OwnerChipPhaseWaiting, bytes);
    check!(OwnerChipPhaseActive, bytes);
    check!(OwnerChipPhasePerson, bytes);
    check!(OwnerChipPhaseClosing, bytes);
    check!(OwnerChipPhaseHeld, bytes);
    check!(OwnerChipPhase, bytes);
    check!(FailureClass, bytes);
}

fn records_2(bytes: &[u8]) {
    check!(RetryNoticeOwnerFailed, bytes);
    check!(RetryNoticeFailed, bytes);
    check!(RetryNoticeRefused, bytes);
    check!(RetryNotice, bytes);
    check!(CallIdentity, bytes);
    check!(RunWorkTool, bytes);
    check!(RunWork, bytes);
    check!(RunObservation, bytes);
    check!(ActiveRun, bytes);
    check!(ProposerTask, bytes);
    check!(ProposerParty, bytes);
    check!(Proposer, bytes);
    check!(ProposalKey, bytes);
    check!(PartScopeProject, bytes);
    check!(PartScope, bytes);
    check!(PartConnector, bytes);
    check!(PartResource, bytes);
    check!(PartKind, bytes);
    check!(PartPanel, bytes);
    check!(PartResult, bytes);
    check!(PartEffect, bytes);
    check!(PartOwnerEffect, bytes);
    check!(PartOutbox, bytes);
    check!(PartHold, bytes);
    check!(PartProject, bytes);
    check!(PartDeployment, bytes);
    check!(Part, bytes);
    check!(WakePolicy, bytes);
    check!(RecipientTask, bytes);
    check!(RecipientParty, bytes);
    check!(RecipientRole, bytes);
    check!(RecipientRoles, bytes);
    check!(Recipient, bytes);
    check!(TakenBy, bytes);
    check!(Revision, bytes);
    check!(ActorParty, bytes);
    check!(ActorTask, bytes);
    check!(ActorProjectDeployment, bytes);
    check!(Actor, bytes);
    check!(PartyKind, bytes);
}

fn records_3(bytes: &[u8]) {
    check!(Party, bytes);
    check!(DecisionRejected, bytes);
    check!(DecisionReleased, bytes);
    check!(DecisionLeftHeld, bytes);
    check!(Decision, bytes);
    check!(AttemptFailure, bytes);
    check!(OwnerExecutorAgent, bytes);
    check!(OwnerExecutorProcedure, bytes);
    check!(OwnerExecutorPerson, bytes);
    check!(OwnerExecutor, bytes);
    check!(WakeRuleBatch, bytes);
    check!(WakeRule, bytes);
    check!(ResultsWake, bytes);
    check!(OwnerWakePolicy, bytes);
    check!(OwnerPersonStallNotice, bytes);
    check!(OwnerPersonWait, bytes);
    check!(OwnerPartKind, bytes);
    check!(OwnerPartKeyPath, bytes);
    check!(OwnerPartKeyNumber, bytes);
    check!(OwnerPartKey, bytes);
    check!(OwnerPart, bytes);
    check!(OwnerPriorPhaseActive, bytes);
    check!(OwnerPriorPhaseClosing, bytes);
    check!(OwnerPriorPhase, bytes);
    check!(OwnerEscalationHolderTask, bytes);
    check!(OwnerEscalationHolderPerson, bytes);
    check!(OwnerEscalationHolderRole, bytes);
    check!(OwnerEscalationHolder, bytes);
    check!(OwnerHoldEscalationWaiting, bytes);
    check!(OwnerHoldEscalationRejected, bytes);
    check!(OwnerHoldEscalation, bytes);
    check!(OwnerFailureTries, bytes);
    check!(OwnerChipHold, bytes);
    check!(OwnerNumbers, bytes);
    check!(FunderTask, bytes);
    check!(FunderPool, bytes);
    check!(FunderRecurring, bytes);
    check!(FunderPeriod, bytes);
    check!(Funder, bytes);
    check!(OwnerBudget, bytes);
}

fn records_4(bytes: &[u8]) {
    check!(OwnerResourceWriterRun, bytes);
    check!(OwnerResourceWriterEffect, bytes);
    check!(OwnerResourceWriter, bytes);
    check!(OwnerResourceLine, bytes);
    check!(OwnerChipWorkTool, bytes);
    check!(OwnerChipWork, bytes);
    check!(OwnerChipObservation, bytes);
    check!(OwnerRequesterTask, bytes);
    check!(OwnerRequesterPerson, bytes);
    check!(OwnerRequesterDeployment, bytes);
    check!(OwnerRequester, bytes);
    check!(ResultField, bytes);
    check!(ChipFields, bytes);
    check!(ChipRow, bytes);
    check!(ChipPhaseOwnerFields, bytes);
    check!(ChipPhaseActiveFields, bytes);
    check!(ChipPhaseWaitingFields, bytes);
    check!(ChipPhaseIdleFields, bytes);
    check!(ChipPhaseRunningFields, bytes);
    check!(ChipPhaseWaitingForWordsFields, bytes);
    check!(ChipPhaseBackingOffFields, bytes);
    check!(ChipPhaseProcedureFields, bytes);
    check!(ChipPhasePersonFields, bytes);
    check!(ChipPhaseClosingFields, bytes);
    check!(ChipPhaseHeldFields, bytes);
    check!(ChipPhaseEndedFields, bytes);
    check!(ChipPhaseFields, bytes);
    check!(ChipPhaseRow, bytes);
    check!(ClosingFields, bytes);
    check!(ClosingRow, bytes);
    check!(EndingTaskFields, bytes);
    check!(EndingFailedFields, bytes);
    check!(EndingCancelledFields, bytes);
    check!(EndingWithdrawnFields, bytes);
    check!(EndingDecidedFields, bytes);
    check!(EndingFields, bytes);
    check!(EndingRow, bytes);
    check!(HeldRuntimeActiveFields, bytes);
    check!(HeldRuntimeClosingFields, bytes);
    check!(HeldRuntimeFields, bytes);
}

fn records_5(bytes: &[u8]) {
    check!(HeldRuntimeRow, bytes);
    check!(HoldReasonTriesFields, bytes);
    check!(HoldReasonStalledFields, bytes);
    check!(HoldReasonDependencyFields, bytes);
    check!(HoldReasonDriftFields, bytes);
    check!(HoldReasonEffectFailedFields, bytes);
    check!(HoldReasonEffectUncertainFields, bytes);
    check!(HoldReasonBudgetFields, bytes);
    check!(HoldReasonDeadlineFields, bytes);
    check!(HoldReasonStoppedFields, bytes);
    check!(HoldReasonResourceFields, bytes);
    check!(HoldReasonPoolFields, bytes);
    check!(HoldReasonProcedureFields, bytes);
    check!(HoldReasonFields, bytes);
    check!(HoldReasonRow, bytes);
    check!(OwnerChipDetailsFields, bytes);
    check!(OwnerChipDetailsRow, bytes);
    check!(OwnerChipEndingDoneFields, bytes);
    check!(OwnerChipEndingFailedFields, bytes);
    check!(OwnerChipEndingCancelledFields, bytes);
    check!(OwnerChipEndingFields, bytes);
    check!(OwnerChipEndingRow, bytes);
    check!(ResultValueReportFields, bytes);
    check!(ResultValueOwnerVerdictFields, bytes);
    check!(ResultValueVerdictFields, bytes);
    check!(ResultValueConnectorFields, bytes);
    check!(ResultValueChangeFields, bytes);
    check!(ResultValueChoiceFields, bytes);
    check!(ResultValueApprovedFields, bytes);
    check!(ResultValueFailureFields, bytes);
    check!(ResultValueWordsFields, bytes);
    check!(ResultValueFields, bytes);
    check!(ResultValueRow, bytes);
    check!(Fact, bytes);
    check!(Chip, bytes);
    check!(ChipPhase, bytes);
    check!(Closing, bytes);
    check!(Ending, bytes);
    check!(HeldRuntime, bytes);
    check!(HoldReason, bytes);
}

fn records_6(bytes: &[u8]) {
    check!(OwnerChipDetails, bytes);
    check!(OwnerChipEnding, bytes);
    check!(ResultValue, bytes);
    check!(PersonAnswerChoice, bytes);
    check!(PersonAnswerApproved, bytes);
    check!(PersonAnswerWords, bytes);
    check!(PersonAnswerCannot, bytes);
    check!(PersonAnswer, bytes);
    check!(AddressChange, bytes);
    check!(ParameterNumber, bytes);
    check!(ParameterBytes, bytes);
    check!(ParameterResource, bytes);
    check!(Parameter, bytes);
    check!(Specification, bytes);
    check!(OwnerPatternEndExact, bytes);
    check!(OwnerPatternEndOpen, bytes);
    check!(OwnerPatternEnd, bytes);
    check!(OwnerPattern, bytes);
    check!(OwnerGrant, bytes);
    check!(OwnerAuthorityExecutorCharter, bytes);
    check!(OwnerAuthorityExecutorProcedure, bytes);
    check!(OwnerAuthorityExecutorRole, bytes);
    check!(OwnerAuthorityExecutor, bytes);
    check!(OwnerDelegation, bytes);
    check!(OwnerResourceScope, bytes);
    check!(OwnerAuthority, bytes);
    check!(Amendment, bytes);
    check!(DisposalCancel, bytes);
    check!(DisposalTakeOver, bytes);
    check!(Disposal, bytes);
    check!(GoalExecutorCharter, bytes);
    check!(GoalExecutorProcedure, bytes);
    check!(GoalExecutor, bytes);
    check!(Priority, bytes);
    check!(SeenResults, bytes);
    check!(SeenReplies, bytes);
    check!(Seen, bytes);
    check!(NoteScopeProject, bytes);
    check!(NoteScopeResource, bytes);
    check!(NoteScopeGoal, bytes);
}

fn records_7(bytes: &[u8]) {
    check!(NoteScope, bytes);
    check!(PolicyEditRole, bytes);
    check!(PolicyJudge, bytes);
    check!(PolicyGuardObserved, bytes);
    check!(PolicyGuard, bytes);
    check!(PolicyRequirement, bytes);
    check!(PolicyPermission, bytes);
    check!(PolicyEditProjectSpend, bytes);
    check!(PolicyEditRoleValue, bytes);
    check!(PolicyEditRequirements, bytes);
    check!(PolicyEditProjections, bytes);
    check!(PolicyEditPermissions, bytes);
    check!(PolicyEdit, bytes);
    check!(HeldRolePolicy, bytes);
    check!(HeldRole, bytes);
    check!(Holding, bytes);
    check!(StartChat, bytes);
    check!(Write, bytes);
    check!(AnswerQuestion, bytes);
    check!(AnswerPerson, bytes);
    check!(Take, bytes);
    check!(HandBack, bytes);
    check!(DecideProposal, bytes);
    check!(DecideEscalation, bytes);
    check!(Release, bytes);
    check!(Amend, bytes);
    check!(Stop, bytes);
    check!(Cancel, bytes);
    check!(Close, bytes);
    check!(TakeOver, bytes);
    check!(SetGoal, bytes);
    check!(Prioritise, bytes);
    check!(MarkSeen, bytes);
    check!(EditNote, bytes);
    check!(DeleteNote, bytes);
    check!(ChangePolicy, bytes);
    check!(SetRoles, bytes);
    check!(MakeService, bytes);
    check!(SetPool, bytes);
    check!(Ask, bytes);
}

fn records_8(bytes: &[u8]) {
    check!(OwnerProposalChoice, bytes);
    check!(OwnerEscalationChoice, bytes);
    check!(OwnerOutcomeClosing, bytes);
    check!(OwnerOutcomeMarkedSeen, bytes);
    check!(OwnerOutcomeNoteEdited, bytes);
    check!(OwnerOutcomeServiceMade, bytes);
    check!(OwnerOutcomeAdopted, bytes);
    check!(OwnerOutcomeGoalStarted, bytes);
    check!(OwnerOutcomeGoalProposed, bytes);
    check!(OwnerOutcomeStopped, bytes);
    check!(OwnerOutcomeCancelled, bytes);
    check!(OwnerOutcomeReleased, bytes);
    check!(OwnerOutcomePersonTaken, bytes);
    check!(OwnerOutcomePersonHandedBack, bytes);
    check!(OwnerOutcomePersonAnswered, bytes);
    check!(OwnerOutcomeMoved, bytes);
    check!(OwnerOutcomeProposalDecided, bytes);
    check!(OwnerOutcomeSaid, bytes);
    check!(OwnerOutcomeQuestionAnswered, bytes);
    check!(OwnerOutcomePrioritised, bytes);
    check!(OwnerOutcomeAmended, bytes);
    check!(OwnerOutcomeAmendProposed, bytes);
    check!(OwnerOutcomeRolesSet, bytes);
    check!(OwnerOutcomePolicyChanged, bytes);
    check!(OwnerOutcomePoolSet, bytes);
    check!(OwnerOutcomeEscalationDecided, bytes);
    check!(OwnerOutcomeStarted, bytes);
    check!(OwnerOutcome, bytes);
    check!(MadeChat, bytes);
    check!(MadeWords, bytes);
    check!(MadeGoal, bytes);
    check!(MadeTasks, bytes);
    check!(MadeService, bytes);
    check!(MadePersonResult, bytes);
    check!(MadeNote, bytes);
    check!(Made, bytes);
    check!(OutcomeOwner, bytes);
    check!(OutcomeMade, bytes);
    check!(OutcomeProposed, bytes);
    check!(OutcomeDecided, bytes);
}

fn records_9(bytes: &[u8]) {
    check!(OutcomeDecidedFirst, bytes);
    check!(Outcome, bytes);
    check!(DisposalIssue, bytes);
    check!(Grant, bytes);
    check!(LackingBudget, bytes);
    check!(LackingGrant, bytes);
    check!(LackingGrantKind, bytes);
    check!(LackingAgain, bytes);
    check!(LackingRequired, bytes);
    check!(LackingFailed, bytes);
    check!(LackingUnguarded, bytes);
    check!(LackingTaskKind, bytes);
    check!(LackingDepth, bytes);
    check!(LackingDeadline, bytes);
    check!(LackingTasks, bytes);
    check!(Lacking, bytes);
    check!(AdmissionLimit, bytes);
    check!(LimitAdmission, bytes);
    check!(LimitLiveTasks, bytes);
    check!(LimitInbox, bytes);
    check!(LimitWords, bytes);
    check!(LimitConversationWords, bytes);
    check!(Limit, bytes);
    check!(CurrentHoldDocument, bytes);
    check!(CurrentHoldReason, bytes);
    check!(CurrentHold, bytes);
    check!(MovedHold, bytes);
    check!(Moved, bytes);
    check!(RefusalRole, bytes);
    check!(RefusalDisposal, bytes);
    check!(RefusalTaken, bytes);
    check!(RefusalAuthority, bytes);
    check!(RefusalLimit, bytes);
    check!(RefusalMoved, bytes);
    check!(RefusalEnded, bytes);
    check!(Refusal, bytes);
    check!(Done, bytes);
    check!(Refused, bytes);
    check!(Final, bytes);
}

fn decoders(bytes: &[u8]) {
    records_1(bytes);
    records_2(bytes);
    records_3(bytes);
    records_4(bytes);
    records_5(bytes);
    records_6(bytes);
    records_7(bytes);
    records_8(bytes);
    records_9(bytes);
}

fn arbitrary(rng: &mut Rng) {
    let mut buffer = [0; 256];
    for _ in 0..2000 {
        let length = usize::try_from(rng.below(257)).expect("bounded length");
        for byte in buffer.get_mut(..length).expect("within buffer") {
            *byte = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
        }
        decoders(buffer.get(..length).expect("within buffer"));
    }
}

fn mutation_seed(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    name.ends_with("_full.bin")
        && [
            "record_keyed_",
            "record_owned_",
            "record_refused_",
            "enum_ask_",
            "enum_final_",
            "enum_fact_",
            "record_hold_reason_",
        ]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

fn mutated(rng: &mut Rng) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/jig-protocol-client-wire/golden/v1");
    let mut paths: Vec<_> =
        fs::read_dir(root).expect("golden directory").map(|entry| entry.expect("golden entry").path()).collect();
    paths.sort();
    for path in paths {
        if !mutation_seed(&path) {
            continue;
        }
        let golden = fs::read(path).expect("golden bytes");
        decoders(&golden);
        for _ in 0..1 {
            let mut bytes = golden.clone();
            if !bytes.is_empty() {
                let index =
                    usize::try_from(rng.below(u64::try_from(bytes.len()).expect("length fits"))).expect("index fits");
                *bytes.get_mut(index).expect("chosen byte") = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
            }
            decoders(&bytes);
        }
    }
}

#[test]
fn client_wire_decoders_never_panic_and_reencode_the_same() {
    let mut rng = Rng::new(0xc11e_0030);
    arbitrary(&mut rng);
    mutated(&mut rng);
}
