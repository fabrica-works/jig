//! Shared adversarial stories (`domain/testing.md`, 5). These name jig's work
//! and faults; an application binds each action to its own kinds and resources.
use crate::referee::Violation;
use crate::{Application, Cut, Harness, Outcome};

/// An adversarial story shared by every application with the required kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenario {
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
}

/// Every shared story, in a stable order for focused cuts and sweeps.
pub const ALL: [Scenario; 9] = [
    Scenario::LateCopy,
    Scenario::OtherHand,
    Scenario::UncertainRestarts,
    Scenario::ChangedJudge,
    Scenario::ShrinkingPool,
    Scenario::NarrowingPolicy,
    Scenario::StandingTask,
    Scenario::RunBudget,
    Scenario::RacingDecisions,
];

/// One scenario action, translated at the application's outside boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
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
        Scenario::ShrinkingPool | Scenario::StandingTask | Scenario::RunBudget | Scenario::RacingDecisions => {}
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
