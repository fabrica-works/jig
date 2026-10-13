//! Exhaustive routing of the root's journal, with values kept in their owners (service.md, 4).
use crate::{End, Engine, PartyOutput, StoreRequest, stop};
use jig_domain_core as core;
use jig_test_connector as connector;
use jig_test_domain as root;

pub(super) fn request(engine: &mut Engine, request: root::Request) {
    match request {
        root::Request::Commit { number, writes } => engine.stores.push(StoreRequest::Commit { number, writes }),
        root::Request::Restart(step) => restart(engine, step),
        root::Request::Deliver(delivery) => deliver(engine, delivery),
        root::Request::Now(now) => immediate(engine, now),
        root::Request::Busy { event } => engine.outputs.push(PartyOutput::Busy(event)),
        root::Request::ReadResult { .. } => stop::unavailable(engine, b"workers"),
        root::Request::Stop => stop::invalid_restore(engine),
    }
}

fn deliver(engine: &mut Engine, delivery: root::Delivery) {
    match delivery {
        root::Delivery::Restart(step) => restart(engine, step),
        root::Delivery::System { connector: number, call } => {
            let read = match &call {
                connector::SystemRequest::ReadFact { .. } => true,
                connector::SystemRequest::Apply { .. } | connector::SystemRequest::Look { .. } => false,
            };
            let event = system(engine, number).answer(call);
            engine.events.push(root::Event::Connector { number, event: connector::Event::System(event) });
            if read {
                restart_read(engine, number);
            }
        }
        root::Delivery::LostRead { connector, resource, call, .. } => {
            let event = system(engine, connector).answer(call);
            engine.events.push(root::Event::WriterRead { connector, resource, event });
        }
        root::Delivery::Projection { connector, feed } => {
            if feed.closing {
                engine
                    .events
                    .push(root::Event::Core(core::Event::ProjectionSettled { goal: feed.goal.number, connector }));
            }
        }
        root::Delivery::Fleet(event) => engine.events.push(root::Event::Core(core::Event::Fleet(event))),
        root::Delivery::Procedure { .. } => stop::unavailable(engine, b"procedures"),
        root::Delivery::Core(held) => core_held(engine, held),
        held @ (root::Delivery::ApplicationDocument { .. }
        | root::Delivery::ApplicationRead { .. }
        | root::Delivery::Application { .. }) => engine.outputs.push(PartyOutput::Held(held)),
        root::Delivery::Link { .. }
        | root::Delivery::Assigned { .. }
        | root::Delivery::Message { .. }
        | root::Delivery::CallAnswer { .. }
        | root::Delivery::CallBusy { .. } => stop::unavailable(engine, b"workers"),
    }
}

fn restart(engine: &mut Engine, step: core::RestartStep) {
    match step {
        core::RestartStep::ReadAfresh { connector: number } => {
            let paths = system(engine, number).paths();
            let count = u32::try_from(paths.len()).expect("bounded system resources");
            if count == 0 {
                engine.events.push(root::Event::RestartDone(step));
                return;
            }
            engine.restart_reads = Some((number, count));
            for resource in paths {
                engine.events.push(root::Event::Connector {
                    number,
                    event: connector::Event::Restart(connector::RestartStep::FreshRead { resource }),
                });
            }
        }
        core::RestartStep::LoadCore | core::RestartStep::LoadInbox | core::RestartStep::RestoreConnector { .. } => {
            engine.stores.push(StoreRequest::Restart(step));
        }
        core::RestartStep::AdoptRuns | core::RestartStep::SettleOutbox { .. } | core::RestartStep::Open => {
            stop::invalid_restore(engine);
        }
    }
}
fn restart_read(engine: &mut Engine, number: u16) {
    if let Some((owner, left)) = engine.restart_reads
        && owner == number
    {
        let left = left.checked_sub(1).expect("one answer per restart read");
        engine.restart_reads = if left == 0 { None } else { Some((owner, left)) };
        if left == 0 {
            engine.events.push(root::Event::RestartDone(core::RestartStep::ReadAfresh { connector: number }));
        }
    }
}

pub(super) fn system(engine: &mut Engine, number: u16) -> &mut crate::system::System {
    let index = usize::from(number.checked_sub(1).expect("connector numbers start at one"));
    engine.systems.get_mut(index).expect("the test application owns two connectors")
}

fn immediate(engine: &mut Engine, now: core::Now) {
    match now {
        value @ (core::Now::SeenChat { .. }
        | core::Now::LoadEndedChat { .. }
        | core::Now::HistoricalPersonTask { .. }
        | core::Now::HistoricalProposal { .. }
        | core::Now::HistoricalEscalation { .. }
        | core::Now::StartPreparation { .. }) => {
            engine.stores.push(StoreRequest::Load(value));
        }
        // Configuration is parsed whole; inactive account components receive no requests.
        core::Now::Account(_) => {}
        value @ (core::Now::SignInRefused { .. } | core::Now::WatchRefused { .. } | core::Now::RunText { .. }) => {
            engine.outputs.push(PartyOutput::Now(value));
        }
        core::Now::CallBusy { .. }
        | core::Now::Relayed { .. }
        | core::Now::EffectAnswer { .. }
        | core::Now::SettledCallRefused { .. }
        | core::Now::Call { .. }
        | core::Now::DropCall { .. }
        | core::Now::Undelivered { .. }
        | core::Now::NotesIndexed { .. }
        | core::Now::NotesRecalled { .. }
        | core::Now::NotesRefused { .. }
        | core::Now::NoteBusy { .. } => stop::unavailable(engine, b"accounts"),
        core::Now::DropPayload { .. }
        | core::Now::DropAssignment { .. }
        | core::Now::TurnPayload { .. }
        | core::Now::AnswerPayload { .. }
        | core::Now::AcceptedTurn { .. }
        | core::Now::RefusedPayload { .. }
        | core::Now::Activate { .. }
        | core::Now::PrepareAgent { .. }
        | core::Now::CompleteBrief { .. }
        | core::Now::BriefCorePlanned { .. }
        | core::Now::WorkspaceRequest { .. }
        | core::Now::RunPreparationFailed { .. }
        | core::Now::RunPrepared { .. }
        | core::Now::ProcedureDelegateOutcome { .. }
        | core::Now::RestoreRefused => stop::invalid_restore(engine),
    }
}

fn core_held(engine: &mut Engine, held: core::Held) {
    match held {
        core::Held::Drained => engine.end = Some(End::Drained),
        held @ (core::Held::ReadLoad { .. } | core::Held::NotesLoad { .. }) => {
            engine.stores.push(StoreRequest::Read(held));
        }
        held @ (core::Held::Read { .. }
        | core::Held::Delivery { .. }
        | core::Held::WatchEnded { .. }
        | core::Held::PeopleReply { .. }
        | core::Held::Result { .. }) => {
            engine.outputs.push(PartyOutput::Held(root::Delivery::Core(held)));
        }
        core::Held::MakeEffect { connector: number, entry } => {
            engine.events.push(root::Event::Connector { number, event: connector::Event::Make { entry } });
        }
        core::Held::Assign { .. }
        | core::Held::Grant { .. }
        | core::Held::Relayed { .. }
        | core::Held::Inbound { .. }
        | core::Held::Relay { .. }
        | core::Held::Acknowledge { .. }
        | core::Held::TaskTerminalAcknowledged { .. }
        | core::Held::TaskTurnKept { .. }
        | core::Held::AcknowledgeTurn { .. }
        | core::Held::Cancel { .. }
        | core::Held::Refuse { .. }
        | core::Held::TurnBusy { .. }
        | core::Held::StopRun { .. } => stop::unavailable(engine, b"workers"),
        core::Held::SettledCall { .. }
        | core::Held::CallAnswer { .. }
        | core::Held::NotesWritten { .. }
        | core::Held::NotesDeleted { .. } => stop::unavailable(engine, b"accounts"),
        core::Held::Applied { .. }
        | core::Held::ApplicationDelivery { .. }
        | core::Held::ApplicationWatchEnded { .. } => stop::invalid_restore(engine),
    }
}
