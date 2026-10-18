//! Facts read from delivered documents (client/domain/client.md, 19;
//! pages.md, 4; contract.md, 5.3–5.4).
use crate::contract::{
    ActKind, Admission, Chip, ChipPhase, Effect, HeldRole, HoldCategory, Me, Offered, OwnerChipDetails, OwnerChipPhase,
    OwnerChipWork, RetryNotice, RunStage, WaitedFor, WaitsOn, Whom,
};
use alloc::boxed::Box;
use skein_lib::Wall;

/// A chat's status, in the precedence of pages.md, 4.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChatStatus {
    Starting,
    Writing,
    Tool { tool: Box<[u8]>, since: Wall },
    Working { since: Wall },
    WaitingForYou { kind: WaitedFor, entries: u32 },
    WaitingFor(Whom),
    Parked,
    Retrying { until: Wall, notice: RetryNotice },
    Held(HoldCategory),
    Closing,
    Ended { at: Wall },
}

/// Read the same facts for a task query's line and a task's page.
#[must_use]
pub fn chat_status(chip: &Chip, waits_on: &WaitsOn, project: u32, me: &Me) -> ChatStatus {
    match chip.details.as_ref() {
        ChipPhase::Owner { phase, details } => match phase {
            OwnerChipPhase::Held { reason, .. } => ChatStatus::Held(*reason),
            OwnerChipPhase::Closing(_) => ChatStatus::Closing,
            OwnerChipPhase::Ended => ChatStatus::Ended { at: details.ended_at.expect("ended chip has its end time") },
            OwnerChipPhase::Active(stage) => {
                if let Some(waiting) = waiting(waits_on, project, me) {
                    return waiting;
                }
                active(*stage, details)
            }
            OwnerChipPhase::Waiting { .. } | OwnerChipPhase::Person { .. } => ChatStatus::Starting,
        },
        ChipPhase::Active(_)
        | ChipPhase::Waiting { .. }
        | ChipPhase::Idle { .. }
        | ChipPhase::Starting
        | ChipPhase::Running { .. }
        | ChipPhase::WaitingForWords { .. }
        | ChipPhase::BackingOff { .. }
        | ChipPhase::Procedure { .. }
        | ChipPhase::Person { .. }
        | ChipPhase::Closing(_)
        | ChipPhase::Held { .. }
        | ChipPhase::Ended { .. } => unreachable!("delivered chips carry their whole owner phase"),
    }
}
fn active(stage: RunStage, details: &OwnerChipDetails) -> ChatStatus {
    match stage {
        RunStage::BackingOff { until } => {
            ChatStatus::Retrying { until, notice: details.retry.expect("backing off records its retry decision") }
        }
        RunStage::Idle => ChatStatus::Parked,
        RunStage::Due | RunStage::Preparing => ChatStatus::Starting,
        RunStage::Claimed { .. } | RunStage::Running { .. } => match &details.observation {
            Some(observation) => match &observation.work {
                OwnerChipWork::Preparing | OwnerChipWork::Starting => ChatStatus::Starting,
                OwnerChipWork::Writing => ChatStatus::Writing,
                OwnerChipWork::Tool { tool, .. } => ChatStatus::Tool { tool: tool.clone(), since: observation.since },
                OwnerChipWork::Waiting => ChatStatus::Parked,
                OwnerChipWork::Active | OwnerChipWork::Ending => ChatStatus::Working { since: observation.since },
            },
            None => match stage {
                RunStage::Claimed { .. } => ChatStatus::Starting,
                RunStage::Running { .. } => ChatStatus::Working { since: details.phase_since },
                RunStage::Idle | RunStage::Due | RunStage::Preparing | RunStage::BackingOff { .. } => {
                    unreachable!("only placed stages read observations")
                }
            },
        },
    }
}
fn waiting(waits_on: &WaitsOn, project: u32, me: &Me) -> Option<ChatStatus> {
    match waits_on {
        WaitsOn::Party { whom, kind, entries, .. } => {
            if reader(*whom, project, me) {
                Some(ChatStatus::WaitingForYou { kind: *kind, entries: *entries })
            } else {
                Some(ChatStatus::WaitingFor(*whom))
            }
        }
        WaitsOn::Delegates | WaitsOn::News | WaitsOn::Nothing => None,
    }
}
fn reader(whom: Whom, project: u32, me: &Me) -> bool {
    match whom {
        Whom::Person(person) => person == me.party.number,
        Whom::Role { project: held_project, role } => {
            if held_project != project {
                return false;
            }
            for held in &me.projects {
                let number = match held.role {
                    HeldRole::Owner => 0,
                    HeldRole::Maintainer => 1,
                    HeldRole::Member => 2,
                    HeldRole::Observer => 3,
                    HeldRole::Policy { role } => role,
                };
                if held.project == project && number == role {
                    return true;
                }
            }
            false
        }
    }
}
/// The consequence supplied with this act, if it was offered with one.
#[must_use]
pub fn effect(offers: &[Offered], act: ActKind) -> Option<&Effect> {
    for offered in offers {
        if offered.act == act {
            return offered.effect.as_ref();
        }
    }
    None
}
/// The engine's admission, without inferring any permission.
#[must_use]
pub fn offer(offers: &[Offered], act: ActKind) -> Option<&Admission> {
    for offered in offers {
        if offered.act == act {
            return Some(&offered.admission);
        }
    }
    None
}
/// Acts the reference UI marks dangerous (objects.md, 2).
#[must_use]
pub const fn dangerous(act: ActKind) -> bool {
    match act {
        ActKind::Cancel | ActKind::Reject | ActKind::DeleteNote => true,
        ActKind::Message
        | ActKind::Answer
        | ActKind::Take
        | ActKind::HandBack
        | ActKind::Accept
        | ActKind::PassUp
        | ActKind::Release
        | ActKind::LeaveHeld
        | ActKind::Amend
        | ActKind::Stop
        | ActKind::Move
        | ActKind::Reorder
        | ActKind::Seen
        | ActKind::EditNote
        | ActKind::Adopt
        | ActKind::ChangePolicy
        | ActKind::SetRoles
        | ActKind::MakeService
        | ActKind::SetPool
        | ActKind::Withdraw => false,
    }
}
