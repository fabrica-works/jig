//! Shared adversarial stories (`domain/testing.md`, 5). These name jig's work
//! and faults; an application binds each action to its own kinds and resources.
use crate::referee::Violation;
use crate::{Application, Cut, Harness, Outcome};

/// An adversarial story shared by every application with the required kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenario {
    /// A keyed compound is made whole across crashes and duplicate requests.
    Compound,
    /// A grace survives two restarts and ends by completion or cancellation.
    Grace,
    /// A delayed effect copy reaches the system after its retry.
    LateCopy,
    /// Another participant reaches the conditional write's target first.
    OtherHand,
    /// Repeated cold starts must retain an uncertain attempt's retry deadline.
    UncertainRestarts,
    /// An observed judge changes facts after the verdict and before application.
    ChangedJudge,
    /// A shrinking pool retains holders while its queued task waits.
    ShrinkingPool,
    /// A narrower policy keeps committed entries and rechecks waiting proposals.
    NarrowingPolicy,
    /// A standing task makes bounded batches across many funding periods.
    StandingTask,
    /// Many sessions and sub-agent completions share one run's final allowance.
    RunBudget,
    /// A party and a run amend one task against the same revision.
    RacingDecisions,
    /// Conversations end and restart while their requesting party reads.
    ConversationEndsWhileRead,
    /// A task query is watched across ends and cold starts.
    QueryWatchedWhileTasksEnd,
}

/// Every shared story, in a stable order for focused cuts and sweeps.
pub const ALL: [Scenario; 13] = [
    Scenario::LateCopy,
    Scenario::OtherHand,
    Scenario::UncertainRestarts,
    Scenario::ChangedJudge,
    Scenario::ShrinkingPool,
    Scenario::NarrowingPolicy,
    Scenario::StandingTask,
    Scenario::RunBudget,
    Scenario::RacingDecisions,
    Scenario::ConversationEndsWhileRead,
    Scenario::QueryWatchedWhileTasksEnd,
    Scenario::Compound,
    Scenario::Grace,
];

/// One scenario action, translated at the application's outside boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Move and cancel two delegates, and ask their conversation to finish.
    Compound,
    /// Send the identical compound under its original request key.
    CompoundAgain,
    /// Ask a running task to finish within a two-second grace.
    CancelWithGrace,
    /// Finish the run in the seed half that completes inside the grace.
    Finish,
    /// Let an unseen claim pass adoption grace and retry backoff if needed.
    Ready,
    /// Delay one effect copy independently of the connector.
    Late,
    /// Ask the active run for a numbered effect.
    Effect(u32),
    /// Deliver delayed copies after the retry.
    DeliverLate,
    /// Another participant reaches the decided conditional target.
    OtherHand,
    /// Withhold released system writes while the engine runs ahead.
    HoldWrites,
    /// Let released writes reach their independent system.
    ReleaseWrites,
    /// Compare the durable uncertain deadline across each cold start.
    CheckDeadline,
    /// Change the independent requirement judge's current fact.
    ChangeJudge,
    /// Start two holders and a waiter under a two-slot pool.
    FillPool,
    /// Shrink that pool to one slot.
    ShrinkPool,
    /// Complete one independently hosted holder in order.
    FinishHolder(u32),
    /// Make a proposal that needs a person's authority.
    Propose,
    /// Narrow scenario policy through an authenticated party request.
    NarrowPolicy,
    /// Let the holder accept the waiting proposal under current policy.
    AcceptProposal,
    /// Start a recurring template under its own finite funding period.
    StartStanding,
    /// Complete the current batch, advance funding, and duplicate the notification.
    Period(u64),
    /// Resume the run's independently priced sessions and sub-agents.
    Resume,
    /// A party amends the spec revision both contenders read.
    PartyAmend,
    /// A run amends the same spec revision.
    RunAmend,
    /// Create an explicitly flagged conversation.
    Conversation { forgettable: bool },
    /// Commit one turn in each conversation.
    Turn,
    /// Mark both conversations' turns.
    MarkTurns,
    /// Finish both conversations.
    EndConversations,
    /// Mark their results.
    MarkResults,
    /// Watch the requesting party's task query.
    WatchQuery,
    /// Finish one task selected by that query.
    EndTask,
    /// Page the query's ended group through its index.
    ReadEnded,
}

/// An application adds kind/resource bindings and outside story checks to its
/// engine adapter. Optional stories return no configuration when their kinds
/// do not exist; their absence remains visible to the caller.
pub trait ScenarioApplication: Application + Sized {
    /// Bind the story to tiny limits, independent system state and scripts.
    fn configuration(scenario: Scenario, seed: u64) -> Option<Self::Config>;
    /// Translate one action; implementations may read only store and peers.
    ///
    /// # Errors
    /// Returns a broken promise observed while playing this story action.
    fn action(world: &mut Harness<Self>, action: Action) -> Result<(), Violation>;
    /// Check the story's ending from independent peers and durable records.
    ///
    /// # Errors
    /// Returns the story condition that failed at its outside boundary.
    fn check(world: &Harness<Self>, scenario: Scenario) -> Result<(), Violation>;
}

/// One outside operation in the shared story, independent of replay storage.
#[derive(Clone, Copy, Debug)]
pub enum Step {
    Action(Action),
    Drain,
    Advance(u64),
    Crash,
}

/// The same action/cut order used by fresh replay and snapshot continuation.
#[must_use]
pub fn program(scenario: Scenario) -> Vec<Step> {
    let mut steps = vec![Step::Drain, Step::Action(Action::Ready), Step::Drain];
    if matches!(scenario, Scenario::Compound | Scenario::Grace) {
        lifecycle(&mut steps, scenario);
        return steps;
    }
    if scenario == Scenario::QueryWatchedWhileTasksEnd {
        query(&mut steps);
        return steps;
    }
    if scenario == Scenario::ConversationEndsWhileRead {
        conversation(&mut steps);
        return steps;
    }
    let actions: &[Action] = match scenario {
        Scenario::LateCopy => &[Action::Late, Action::Effect(1)],
        Scenario::OtherHand => &[Action::Late, Action::Effect(1), Action::OtherHand],
        Scenario::UncertainRestarts => &[Action::HoldWrites, Action::Effect(1)],
        Scenario::ChangedJudge => {
            &[Action::HoldWrites, Action::Effect(1), Action::ChangeJudge, Action::ReleaseWrites, Action::Effect(2)]
        }
        Scenario::ShrinkingPool => &[
            Action::FillPool,
            Action::ShrinkPool,
            Action::FinishHolder(0),
            Action::FinishHolder(1),
            Action::FinishHolder(2),
        ],
        Scenario::NarrowingPolicy => &[
            Action::HoldWrites,
            Action::Effect(1),
            Action::Propose,
            Action::NarrowPolicy,
            Action::ReleaseWrites,
            Action::AcceptProposal,
            Action::Effect(3),
        ],
        Scenario::StandingTask => {
            &[Action::StartStanding, Action::Period(2), Action::Period(3), Action::Period(4), Action::Period(5)]
        }
        Scenario::RunBudget => &[Action::Resume],
        Scenario::RacingDecisions => &[Action::PartyAmend, Action::RunAmend],
        Scenario::Compound
        | Scenario::Grace
        | Scenario::QueryWatchedWhileTasksEnd
        | Scenario::ConversationEndsWhileRead => {
            unreachable!("conversation program returned")
        }
    };
    for &action in actions {
        steps.extend([Step::Action(action), Step::Drain]);
    }
    match scenario {
        Scenario::LateCopy | Scenario::OtherHand => {
            // Both intervals retain the restored attempt's original deadline.
            for _ in 0..2 {
                steps.extend([Step::Advance(2_000_000_000), Step::Drain]);
            }
            steps.extend([Step::Action(Action::DeliverLate), Step::Drain]);
        }
        Scenario::UncertainRestarts => {
            steps.push(Step::Action(Action::CheckDeadline));
            for _ in 0..3 {
                steps.extend([Step::Crash, Step::Drain, Step::Action(Action::CheckDeadline)]);
            }
            steps.push(Step::Action(Action::ReleaseWrites));
            for _ in 0..2 {
                steps.extend([Step::Advance(2_000_000_000), Step::Drain]);
            }
        }
        Scenario::ChangedJudge | Scenario::NarrowingPolicy => {
            for _ in 0..2 {
                steps.extend([Step::Advance(2_000_000_000), Step::Drain]);
            }
            if scenario == Scenario::ChangedJudge {
                steps.push(Step::Action(Action::Effect(2)));
            } else {
                steps.extend([Step::Action(Action::NarrowPolicy), Step::Drain, Step::Action(Action::AcceptProposal)]);
            }
            steps.push(Step::Drain);
        }
        Scenario::Compound
        | Scenario::Grace
        | Scenario::ShrinkingPool
        | Scenario::StandingTask
        | Scenario::RunBudget
        | Scenario::RacingDecisions
        | Scenario::ConversationEndsWhileRead
        | Scenario::QueryWatchedWhileTasksEnd => {}
    }
    steps
}

/// Run one shared scenario, including the referee at every iteration.
///
/// # Errors
/// Returns the first broken promise or an application story whose ending failed.
pub fn run<A: ScenarioApplication>(scenario: Scenario, cut: Cut, seed: u64) -> Result<Option<Outcome>, Violation> {
    let Some(config) = A::configuration(scenario, seed) else { return Ok(None) };
    let mut world = Harness::<A>::new(config, seed);
    world.cut(cut);
    for step in program(scenario) {
        match step {
            Step::Action(action) => A::action(&mut world, action).map_err(|violation| world.diagnostic(violation))?,
            Step::Drain => {
                world.drain()?;
            }
            Step::Advance(nanos) => world.advance(nanos),
            Step::Crash => world.crash().map_err(|violation| world.diagnostic(violation))?,
        }
    }
    A::check(&world, scenario).map_err(|violation| world.diagnostic(violation))?;
    Ok(Some(world.outcome()))
}

fn conversation(steps: &mut Vec<Step>) {
    steps.extend([
        Step::Action(Action::Conversation { forgettable: false }),
        Step::Drain,
        Step::Action(Action::Conversation { forgettable: true }),
        Step::Drain,
        Step::Advance(6_000_000_000),
        Step::Drain,
        Step::Advance(1_000_000_000),
        Step::Drain,
        Step::Action(Action::Turn),
        Step::Drain,
        Step::Action(Action::MarkTurns),
        Step::Drain,
        Step::Crash,
        Step::Drain,
        Step::Action(Action::EndConversations),
        Step::Drain,
        Step::Crash,
        Step::Drain,
        Step::Action(Action::MarkTurns),
        Step::Drain,
        Step::Action(Action::MarkResults),
        Step::Drain,
        Step::Crash,
        Step::Drain,
    ]);
}

const GRACE_NANOS: u64 = 2_000_000_000;
const RESTART_AFTER_NANOS: u64 = 500_000_000;
const GRACE_LEFT_NANOS: u64 = 1_000_000_000;

fn lifecycle(steps: &mut Vec<Step>, scenario: Scenario) {
    use Step::{Action as Act, Advance, Crash, Drain};
    let actions: &[Step] = if scenario == Scenario::Compound {
        &[Act(Action::Compound), Drain, Crash, Drain, Act(Action::CompoundAgain), Drain, Advance(GRACE_NANOS), Drain]
    } else {
        &[
            Act(Action::CancelWithGrace),
            Drain,
            Advance(RESTART_AFTER_NANOS),
            Crash,
            Drain,
            Advance(RESTART_AFTER_NANOS),
            Crash,
            Drain,
            Act(Action::Finish),
            Drain,
            Advance(GRACE_LEFT_NANOS),
            Drain,
        ]
    };
    steps.extend_from_slice(actions);
}

fn query(steps: &mut Vec<Step>) {
    steps.extend([Step::Action(Action::WatchQuery), Step::Drain]);
    for _ in 0..3 {
        steps.extend([
            Step::Action(Action::EndTask),
            Step::Drain,
            Step::Crash,
            Step::Drain,
            Step::Action(Action::WatchQuery),
            Step::Drain,
            Step::Action(Action::ReadEnded),
            Step::Drain,
        ]);
    }
}
