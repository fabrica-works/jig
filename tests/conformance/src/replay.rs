//! Cold-cut continuations contain only durable rows and independent neighbours.
//! Fresh and resumed runs share both the story program and concrete actions.
use crate::{Testing, actions::World, scenarios::Continuation};
use jig_conformance::scenarios::{Scenario, ScenarioApplication, Step, program};
use jig_conformance::{Cut, Harness, Outcome, referee::Violation};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug, Default)]
struct Position {
    step: usize,
    action: Option<Continuation>,
    draining: bool,
    drain_started: u64,
}

fn finish_steps(
    world: &mut World,
    scenario: Scenario,
    mut position: Position,
    position_seen: &RefCell<Position>,
) -> Result<(), Violation> {
    let steps = program(scenario);
    loop {
        if position.draining {
            position_seen.replace(position.clone());
            world.drain_from(position.drain_started)?;
            position.draining = false;
        }
        let Some(&step) = steps.get(position.step) else { break };
        match step {
            Step::Action(action) => {
                let continuation = position.action.get_or_insert_with(|| Continuation::new(action));
                if continuation.advance(world).map_err(|error| world.diagnostic(error))? {
                    position.draining = true;
                    position.drain_started = world.outcome().iterations;
                } else {
                    position.action = None;
                    position.step += 1;
                }
            }
            Step::Drain => {
                position.step += 1;
                position.draining = true;
                position.drain_started = world.outcome().iterations;
            }
            Step::Advance(nanos) => {
                world.advance(nanos);
                position.step += 1;
            }
            Step::Crash => {
                world.crash().map_err(|error| world.diagnostic(error))?;
                position.step += 1;
            }
        }
    }
    Testing::check(world, scenario).map_err(|error| world.diagnostic(error))
}

fn diagnose(
    world: &mut World,
    seed: u64,
    play: impl FnOnce(&mut World) -> Result<(), Violation>,
) -> Result<(), Violation> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| play(world))).unwrap_or_else(|cause| {
        let detail = cause
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| cause.downcast_ref::<&str>().copied())
            .unwrap_or("non-string panic");
        panic!("scenario seed {seed}: {detail}\n{}", world.trace.join("\n"));
    })
}

fn finish(
    world: &mut World,
    scenario: Scenario,
    seed: u64,
    position: Position,
    position_seen: &RefCell<Position>,
) -> Result<(), Violation> {
    diagnose(world, seed, |world| finish_steps(world, scenario, position, position_seen))
}

/// Replay every original numbered durable cut from its outside snapshot.
/// Snapshots are consumed immediately, so retained copies cannot inflate the
/// baseline's heap check. The seeded random run remains a separate fresh run.
///
/// # Errors
/// Returns the same scenario/referee failure as fresh cut replay.
pub fn every_cut(scenario: Scenario, seed: u64, visit: impl FnMut(u64, World) + 'static) -> Result<Outcome, Violation> {
    every_cut_traced(scenario, seed, None, visit)
}

/// Explicit complete traces for the independent snapshot/fresh equivalence proof.
///
/// # Errors
/// Returns a scenario/referee failure from baseline or a resumed cut.
pub fn every_cut_traced(
    scenario: Scenario,
    seed: u64,
    enabled: Option<bool>,
    visit: impl FnMut(u64, World) + 'static,
) -> Result<Outcome, Violation> {
    cuts_traced(scenario, seed, enabled, None, visit)
}

/// Compare selected durable snapshots without replaying the other cuts.
///
/// # Errors
/// Returns the same baseline or resumed scenario failure as all-cut replay.
pub fn chosen_cuts_traced(
    scenario: Scenario,
    seed: u64,
    enabled: Option<bool>,
    cuts: &[u64],
    visit: impl FnMut(u64, World) + 'static,
) -> Result<Outcome, Violation> {
    cuts_traced(scenario, seed, enabled, Some(cuts.to_vec()), visit)
}

fn cuts_traced(
    scenario: Scenario,
    seed: u64,
    enabled: Option<bool>,
    selected: Option<Vec<u64>>,
    mut visit: impl FnMut(u64, World) + 'static,
) -> Result<Outcome, Violation> {
    let config = Testing::configuration(scenario, seed).expect("testing binds every shared story");
    let mut world = enabled.map_or_else(
        || Harness::<Testing>::new(config, seed),
        |enabled| Harness::<Testing>::with_trace(config, seed, enabled),
    );
    let position = Rc::new(RefCell::new(Position::default()));
    let hook_position = Rc::clone(&position);
    let failure = Rc::new(RefCell::new(None));
    let hook_failure = Rc::clone(&failure);
    world.on_commit(move |world| {
        if hook_failure.borrow().is_some() {
            return;
        }
        let cut = world.store.applied;
        if selected.as_ref().is_some_and(|cuts| !cuts.contains(&cut)) {
            return;
        }
        let continuation = hook_position.borrow().clone();
        let result = World::from_snapshot(world.cold_snapshot()).and_then(|mut replay| {
            finish(&mut replay, scenario, seed, continuation, &RefCell::new(Position::default()))?;
            visit(cut, replay);
            Ok(())
        });
        if let Err(error) = result {
            hook_failure.replace(Some(error));
        }
    });
    finish(&mut world, scenario, seed, Position::default(), &position)?;
    if let Some(error) = failure.take() {
        return Err(error);
    }
    Ok(world.outcome())
}

/// Original fresh replay, retained for random cuts and independent comparison.
///
/// # Errors
/// Returns a scenario/referee failure.
pub fn fresh(scenario: Scenario, cut: Cut, seed: u64, enabled: bool) -> Result<World, Violation> {
    let config = Testing::configuration(scenario, seed).expect("testing binds every shared story");
    let mut world = Harness::<Testing>::with_trace(config, seed, enabled);
    world.cut(cut);
    diagnose(&mut world, seed, |world| {
        for step in program(scenario) {
            match step {
                Step::Action(action) => Testing::action(world, action).map_err(|error| world.diagnostic(error))?,
                Step::Drain => {
                    world.drain()?;
                }
                Step::Advance(nanos) => world.advance(nanos),
                Step::Crash => world.crash().map_err(|error| world.diagnostic(error))?,
            }
        }
        Testing::check(world, scenario).map_err(|error| world.diagnostic(error))?;
        Ok(())
    })?;
    Ok(world)
}

/// Complete outside state; formatting is requested by equivalence proofs only.
#[must_use]
pub fn outside(world: &World) -> String {
    format!(
        "systems={:?}\npeers={:?}\nstore={:?}\nreferee={:?}\nclock={:?}\noutcome={:?}\ntrace={:?}",
        world.systems,
        world.peers,
        world.store,
        world.referee,
        world.clock(),
        world.outcome(),
        &*world.trace
    )
}
