//! The client-facing seam of the core world (client/domain/testing.md, 5).
//! Values reach this peer only after the existing store and core referee saw
//! their release. Workers and systems remain the world's independent fakes.
use jig_core as core;
use jig_test_domain as root;

/// Untranslated outputs for a client-backed peer. The protocol adapter owns
/// sign-ins, request attempts and watch tokens, and takes these owned values.
#[derive(Debug, Default)]
pub struct ClientPeer {
    pub delivered: Vec<(u64, root::Delivery)>,
    pub now: Vec<core::Now>,
}

impl ClientPeer {
    /// Whether this held output crosses the boundary to a party's client.
    #[must_use]
    pub fn accepts(delivery: &root::Delivery) -> bool {
        match delivery {
            root::Delivery::ApplicationDocument { .. }
            | root::Delivery::ApplicationRead { .. }
            | root::Delivery::Application { .. } => true,
            root::Delivery::Core(held) => match held {
                core::Held::PeopleReply { .. }
                | core::Held::Read { .. }
                | core::Held::Delivery { .. }
                | core::Held::WatchEnded { .. } => true,
                core::Held::ApplicationDelivery { .. }
                | core::Held::ApplicationWatchEnded { .. }
                | core::Held::Applied { .. }
                | core::Held::ReadLoad { .. }
                | core::Held::MakeEffect { .. }
                | core::Held::SettledCall { .. }
                | core::Held::Assign { .. }
                | core::Held::Relayed { .. }
                | core::Held::Inbound { .. }
                | core::Held::Relay { .. }
                | core::Held::CallAnswer { .. }
                | core::Held::Result { .. }
                | core::Held::Acknowledge { .. }
                | core::Held::TaskTerminalAcknowledged { .. }
                | core::Held::TaskTurnKept { .. }
                | core::Held::AcknowledgeTurn { .. }
                | core::Held::Cancel { .. }
                | core::Held::Refuse { .. }
                | core::Held::TurnBusy { .. }
                | core::Held::StopRun { .. }
                | core::Held::NotesLoad { .. }
                | core::Held::NotesWritten { .. }
                | core::Held::NotesDeleted { .. } => false,
            },
            root::Delivery::Projection { .. }
            | root::Delivery::Message { .. }
            | root::Delivery::Restart(_)
            | root::Delivery::CallAnswer { .. }
            | root::Delivery::Assigned { .. }
            | root::Delivery::Fleet(_)
            | root::Delivery::System { .. }
            | root::Delivery::LostRead { .. }
            | root::Delivery::Procedure { .. } => false,
        }
    }

    /// Whether this journal-door output belongs to the client's protocol.
    #[must_use]
    pub fn immediate(value: &core::Now) -> bool {
        match value {
            core::Now::SignInRefused { .. } | core::Now::WatchRefused { .. } | core::Now::RunText { .. } => true,
            core::Now::Relayed { .. }
            | core::Now::EffectAnswer { .. }
            | core::Now::SettledCallRefused { .. }
            | core::Now::Call { .. }
            | core::Now::DropCall { .. }
            | core::Now::Undelivered { .. }
            | core::Now::Account(_)
            | core::Now::NotesIndexed { .. }
            | core::Now::NotesRecalled { .. }
            | core::Now::NotesRefused { .. }
            | core::Now::NoteBusy { .. }
            | core::Now::DropPayload { .. }
            | core::Now::DropAssignment { .. }
            | core::Now::TurnPayload { .. }
            | core::Now::AnswerPayload { .. }
            | core::Now::AcceptedTurn { .. }
            | core::Now::RefusedPayload { .. }
            | core::Now::Activate { .. }
            | core::Now::PrepareAgent { .. }
            | core::Now::StartPreparation { .. }
            | core::Now::SeenChat { .. }
            | core::Now::HistoricalProposal { .. }
            | core::Now::HistoricalPersonTask { .. }
            | core::Now::CompleteBrief { .. }
            | core::Now::BriefCorePlanned { .. }
            | core::Now::WorkspaceRequest { .. }
            | core::Now::RunPreparationFailed { .. }
            | core::Now::RunPrepared { .. }
            | core::Now::HistoricalEscalation { .. }
            | core::Now::ProcedureDelegateOutcome { .. }
            | core::Now::RestoreRefused => false,
        }
    }
}
