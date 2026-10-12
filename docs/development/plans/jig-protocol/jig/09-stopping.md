# Session 09: the engine stopping, and in-process workers' claims at the next start

The core hears that the engine is stopping, and from then on takes
nothing that would make a commit: a party's request is answered not
ready, a run's turn is answered busy and its answer left unacknowledged
so its host keeps them, a run's call is answered busy, a connector's
events stay with the connector, no run is placed and no procedure
stepped. It ends each open watch saying the engine is stopping, and says
once, as a held output, when nothing it holds waits for a commit, so the
service can close its components (jig 27 and jig 26 do). Nothing makes
the core stop today (`findings/domain.md`, item 1). And at the next
start, claims on workers in the engine's own process are settled at
once, as the engine's own slots' are, when the configuration says its
workers are in process.

## Read first

- jig's `domain/engine.md`, 6.1 (stopping), 5.2 (what waits for a
  commit), section 6, step 3 (claims at a restart) and section 8 ("a
  call the fleet cannot pass up is answered at once, unserved, or busy
  while the engine stops"); `protocol/service.md`, section 5 (the
  drain's five steps), section 8 (the referee's "a drained shutdown ends
  with nothing waiting for a commit") and section 10 (one process);
  `protocol/channel.md`, 5.2 (a turn busy, a relayed call busy, an
  answer left unacknowledged sent again after the next hello);
  `protocol/configuration.md`, section 3 (the `hosts` member: "a
  listener on an in-process name tells the core its workers are in its
  process"); `client/domain/contract.md`, 7.2 (a watch "Ended, each
  saying why: ... the engine is stopping").
- `findings/domain.md`, item 1 and "Note on 10.8 and 11.6".
- `findings/protocol.md`, 6.1 ("04-stopping").

## What the design asks

- **Heard before anything closes** (engine.md, 6.1; service.md, section
  5, step 2), from the core's root.
- **Nothing that would make a commit** (engine.md, 6.1): "a party's
  request is answered not ready, and an application's request likewise;
  a run's turn or call is answered busy, and its answer left
  unacknowledged, so its host keeps them and sends them again, to this
  engine or the next; a connector's events and outcomes stay with the
  connector, as when the journal is full; no run is placed, and no
  procedure stepped."
- **Watches:** "It ends every watch saying the engine is stopping".
- **Said once:** "says once when nothing it holds waits for a commit, so
  the service can close its components" (engine.md, 6.1); "What is in
  flight finishes: the commits made before are answered, and their held
  outputs released, until nothing waits for a commit, which the core
  says" (service.md, section 5, step 3).
- **In-process workers** (engine.md, section 6, step 3): "Claims placed
  in the engine died with it, and are settled at once as lost, or as not
  started if they committed nothing ...; so are claims on workers in the
  engine's own process, which no worker outlives ..., when its
  configuration says its workers are in process."

## Choices this plan makes

- **The stop is the core's event `Event::Stopping`,** heard in any
  state, the restart's script included; a second one changes nothing.
  `Core::stopping()` says whether it was heard.
- **"Nothing waits" is the held output `Held::Drained`,** made once, by
  the decision that ends the last watch (the stop's own, when none is
  open): the journal releases a held output only once the commits
  before it are durable, and no commit follows the stop, so its release
  is the moment nothing waits.
- **Each input is answered with the vocabulary it has, or one new
  `Now`:** a party's keyed request `Again::NotReady` (a held
  `PeopleReply`, as a sign-in before restoration is today); a turn
  `fleet::Event::TurnBusy` with its body dropped; an answer's payload
  dropped and never acknowledged; a call `Now::CallBusy { to }`, which
  the root answers through the fleet with its own busy answer (its
  payload, like a read's answer), so the fleet and the hub keep their
  vocabularies; a read answered as before; a new watch ended at once as
  stopping; a hello refused (`Held::Refuse`), so its worker dials the
  next engine.
- **The root holds its connectors back:** while `Core::stopping()`, the
  test root hands a connector's event, outcome or timer back as
  `Request::Busy { event }`, and its world keeps it, as a component
  holds its peer back when the journal cannot take an event
  (service.md, section 2, "Room").
- **Timers that would commit wait:** while stopping, `fire` fires only
  `Timer::View` and `Timer::Account`; `resume_fleet` places nothing; an
  `Activate` is dropped with its context (the next start rebuilds it
  from the store). jig 07's `Core::take_due` gives nothing while
  stopping: whichever of jig 07.2 and jig 09.1 lands second adds that
  line.
- **The accounts child is not closed by the stop:** its close is the
  service's step 4 (jig 08's `accounts::Event::Close`, sent by jig 27.2).
- **"Workers in process" is a flag of the core's `Config`,**
  `workers_in_process`: with it, a worker's claim restored at a restart
  is adopted as the engine's (settled lost, or not started), and the
  restart's admission counts it as the engine's. jig 20 sets it from
  configuration; each fixture of today passes false.
- **`ops` does not stop in this plan:** its root passes `Now::CallBusy`
  and `Held::Drained` out unchanged, as it passes the others it does not
  route, and its worlds never send a stop.

## Rules that bind it

- No increment of this plan "adds an `unreachable!` arm for a variant
  the reference service can reach; an arm the service reaches is
  routed" (code-organization.md, section 6): the test root routes
  `Now::CallBusy` and `Held::Drained`.
- No increment "adds a test that checks several behaviours under one
  name" (code-organization.md, section 6): each refused input has its own
  test.
- "A file: at most 800 lines, generated code excepted. A file of at
  most 800 lines that the increment grows past 800 is split in the same
  increment"
  (code-organization.md, section 7). The core's gate goes in a new
  `crates/jig-domain-core/src/stopping.rs`; `lib.rs` (707 lines) gains
  at most 10.

## Increments

### 09.1 The core stops taking what would commit

Scope: engine.md, 6.1. `Event::Stopping`; `Core::stopping()`; the gate
below, by input; a new watch ended at once; `Now::CallBusy`.
Watches already open and `Held::Drained` are 09.2's.
Needs: nothing.
Files: `crates/jig-domain-core/src/stopping.rs` (new: the gate, one
function per kind of input); `crates/jig-domain-core/src/lib.rs`
(`Core::stopping`, a `stopping: bool` field), `routing/vocabulary.rs`
(`Event::Stopping`, `Now::CallBusy`), `routing/mod.rs` (`room` line 301,
`step_one` line 402, `resume_fleet` line 384, `fire` line 517),
`routing/children.rs` (`tag_fleet`: `Turned` 291, `Answered` 295,
`Relay` 269), `routing/people.rs` (`sign_in`, line 69),
`routing/application_routes.rs` (`start`); the exhaustive `core::Now`
matches: `crates/jig-domain-core/src/tests/completion_retry.rs:52`,
`examples/ops/domain/src/route.rs` (1245-1400), `testing/jig-test-domain/src/routes.rs`
(421-542), `tests/core/src/client_peer.rs:159`, `tests/ops/src/joint/engine.rs:342`,
`tests/ops/src/joint/front.rs:751`, `tests/client/src/joint/session.rs:796`;
`testing/jig-test-domain/src/{lib.rs, routes.rs}` (`Event::Stop`,
`Payload::Busy`, `Delivery::CallBusy`, `Request::Busy`); the exhaustive
matches of the test root's `Request`: `tests/core/src/effects.rs:605`,
`tests/core/src/world.rs:531`, `tests/core/tests/store.rs:100`,
`tests/conformance/src/lib.rs`; `tests/core/src/effects.rs` (records
busy turns and calls, keeps handed-back events);
`tests/core/tests/stopping.rs` (new). Users of
`core::Event` matched exhaustively: `room` and `step_one` only. Shared
hotspots: `routing/{mod.rs, vocabulary.rs, children.rs}` (jig 05, 07,
08), `crates/jig-domain-core/src/lib.rs` (jig 07),
`testing/jig-test-domain/src/{lib.rs, routes.rs}`,
`examples/ops/domain/src/route.rs`.
Shape: the gate, by input, while `core.stopping`:

| Input | Answer |
|---|---|
| `Event::People(people::Event::{Ask, ChatSeen, SignOut})`, `Event::SignIn`, `Event::Application` | `Held::PeopleReply { reply: Again(NotReady) }`, or the application's not ready; the child not stepped |
| `Event::Read`, `ReadLoaded`, the historical loads | as today |
| `Event::Watch`, `WatchRun` | `Held::WatchEnded { why: views::End::Stopping }`, nothing admitted |
| `fleet::Request::Turned` (from `Event::Fleet(Turn)`) | `fleet::Event::TurnBusy` stepped; `Now::DropPayload { payload: body }` |
| `fleet::Request::Answered` | `Now::DropPayload { payload }`; no `Activation`, no acknowledgement |
| `fleet::Request::Relay`, `Event::{NamedAction, NamedAnswer, DelegateValidated, DelegateInputFailed, SettledCall}` | `Now::CallBusy { to }` |
| `Event::Fleet(fleet::Event::Hello { channel, .. })` | `Held::Refuse { channel }`; the fleet not stepped |
| `Event::{Activate, PreparedAgent, StartBrief, BriefNotes, BriefAssembled, WorkspacePrepared, ClaimPrepared, ClaimRefused, PreparationFailed, ProcedureStep, ProcedureHoldings, StartRecurring, Period}` | `Decided` alone: nothing placed, nothing stepped |
| `fire(Timer::{People, Tasks, Fleet, Brief})`; `resume_fleet` | `Decided` alone |
| any other input (accounts, views, notes reads, the restart's script) | as today |

In the test root: `Event::Stop`, decided as `core::Event::Stopping`;
`Domain::stopping()`; `Now::CallBusy { to }` answered by stepping
`fleet::Event::Relayed { to, answer }` with a `Payload::Busy`, and a
relayed `Payload::Busy` delivered as `Delivery::CallBusy { channel,
task, attempt, name }`; `Request::Busy { event: Box<Event> }` for
`Event::{Connector, ConnectorTimer, WriterRead, ProjectionEffect,
EffectCall}` while stopping.
Test shape: `tests/core/tests/stopping.rs` (new), world, under 0.5 s
each, on `jig_core_world::effects::World` (effects.rs:161) with
`World::stop()` (sends `root::Event::Stop`):
`a_partys_request_after_the_stop_is_answered_not_ready`,
`a_turn_after_the_stop_is_answered_busy_and_not_committed`,
`an_answer_after_the_stop_is_left_unacknowledged`,
`a_call_after_the_stop_is_answered_busy`,
`a_new_watch_after_the_stop_is_ended_as_stopping`,
`no_run_is_placed_after_the_stop`,
`a_read_after_the_stop_is_answered_as_before`,
`a_connector_event_after_the_stop_is_handed_back_busy`. Reference:

```rust
#[test]
fn a_turn_after_the_stop_is_answered_busy_and_not_committed() {
    let mut world = World::new(161, false);
    world.stop();
    let commits = world.store.applied;
    world.turn(1, 1, None, b"a turn the engine will not take");
    assert_eq!(world.store.applied, commits, "nothing is committed after the stop");
    assert_eq!(world.busy_turns, [(world.task(), world.attempt(), 1)]);
}
```

(`World::busy_turns` records each `Held::TurnBusy` delivered, beside
`cancellations` in effects.rs:723; `World::task()` and `World::attempt()`
read `assigned`.)
Proof: world: `heavy cargo nextest run -p jig-core-world --test stopping`
shows each kind of input after the stop answered as the table says, and
no commit applied after it; once, under 4 s.
Don't: change the people child's phase (its `ready()` also answers
`person()`, which reads and watches need); close the accounts child; add
a busy answer to the fleet's or the hub's vocabulary (the root's
payload carries it; channel.md 5.2's busy relayed record is jig 22's).
Size: at most 280 lines of production code.
Removes: nothing.

### 09.2 Open watches ended, and nothing waiting said once

Scope: engine.md, 6.1 ("It ends every watch saying the engine is
stopping, and says once ..."); service.md, section 5, step 3. The stop's
decision ends each open watch with `views::End::Stopping`, within the
views' output bound per decision, the rest at the next steps the root
makes; the decision that ends the last makes `Held::Drained`, once;
the test root releases it to its world as it releases the core's held
outputs.
Needs: 09.1's stop.
Files: `crates/jig-domain-core/src/stopping.rs`,
`routing/watch_marks.rs` (`end_invalid`, line 376: a stopping arm),
`routing/vocabulary.rs` (`Held::Drained`); the exhaustive `core::Held`
matches: `crates/jig-domain-core/src/tests/completion_retry.rs`,
`testing/jig-test-domain/src/{client_documents.rs, routes.rs}`,
`tests/core/src/{client_peer.rs, effects.rs}`,
`tests/ops/src/joint/{engine.rs, front.rs}`,
`tests/client/src/joint/session.rs`,
`examples/ops/domain/src/{client_documents.rs, route.rs}`;
`tests/core/tests/stopping.rs`. Shared hotspots: `routing/vocabulary.rs`,
`tests/client/src/joint/session.rs` (1,553 lines: the arm only).
Shape: `Held::Drained`; `Core::draining: Option<bool>` is internal (the
watches still to end). `views::End::Stopping` exists
(crates/jig-domain-views/src/boundary.rs:33) and each `views::End`
consumer maps it already (findings/domain.md, item 1).
Test shape: `tests/core/tests/stopping.rs`, world, under 0.5 s each:
`every_open_watch_ends_saying_the_engine_is_stopping`,
`drained_is_released_once_after_the_last_commit_is_durable`,
`a_second_stop_says_drained_no_second_time`;
`tests/client/src/joint/stories/fault_stories.rs`, joint, under 0.5 s:
`a_watch_ended_as_the_engine_stops_reaches_the_client_as_stopping`.
Proof: world: `heavy cargo nextest run -p jig-core-world --test stopping
drained_is_released_once_after_the_last_commit_is_durable` shows, with
a commit held in the store, `Drained` released only after that commit
is applied, and once; once, under 1 s.
Don't: release `Drained` through the door (it waits for the commits
before it, as a read's answer does).
Size: at most 80 lines of production code.
Removes: nothing.

### 09.3 The conformance world's drained shutdown

Scope: service.md, section 8 ("a drained shutdown ends with nothing
waiting for a commit"), on jig's test root in the domain's conformance
world. The referee gains the rule: once `Drained` is released, no
commit is applied and the commits applied before it are durable; a
story stops the engine mid-work, drains, crashes, restarts, and finds
the work where it was.
Needs: 09.2's `Drained`.
Files: `testing/jig-conformance/src/referee.rs` (`Observed::Drained`,
line 259; `observe`, line 846), `tests/conformance/src/lib.rs` (its
delivery loop, line 440: `Held::Drained` observed; `Request::Busy`
kept), `tests/conformance/src/actions.rs` (`stop`),
`tests/conformance/tests/restart.rs`. Shared hotspots:
`tests/conformance`.
Shape: `Observed::Drained { at: u64 }`; the referee's violation
`CommitAfterDrained { number: u64 }`.
Test shape: `tests/conformance/tests/restart.rs`, world, under 0.5 s:
`a_drained_shutdown_ends_with_nothing_waiting_for_a_commit`;
`tests/conformance/tests/referee.rs`, under 20 ms:
`a_commit_after_drained_is_a_violation`.
Proof: world: `heavy cargo nextest run -p jig-conformance-world --test
restart a_drained_shutdown` shows a stop mid-work drained, crashed and
restarted with the referee finding no commit after `Drained` and each
party's answer durable; once, under 1 s.
Don't: add a fuzzy story here (the fuzzy budget goes to the worlds of
jig 18, 26, 27 and 33).
Size: at most 40 lines of production code (the referee is a test kit:
its lines count as test code).
Removes: nothing.

### 09.4 In-process workers' claims settled at the next start

Scope: engine.md, section 6, step 3; configuration.md, section 3.
`Config::workers_in_process`; with it, the adoption a restart queues
for a worker's claim (sibling_routes.rs:169) names `HostKind::Engine`,
so the fleet settles it at once, lost if it worked, not started if not
(fleet domain.rs:308-316), and `restart_admit_claims` (restart.rs:43)
counts it as the engine's. The test root takes the flag in its
configuration.
Needs: nothing.
Files: `crates/jig-domain-core/src/{lib.rs (Config; Core::new),
sibling_routes.rs (line 169), restart.rs (lines 51-57)}`; the core
`Config` constructions `crates/jig-domain-core/src/tests.rs:227` and
`tests/core/src/world.rs:265` (both pass false; the 22 callers of
`world::config` inherit it); `tests/core/tests/restart.rs`. Shared
hotspots: `crates/jig-domain-core/src/lib.rs` (jig 07),
`tests/core/src/world.rs`.
Shape: `pub workers_in_process: bool` in `jig_domain_core::Config`.
Test shape: `tests/core/tests/restart.rs` (helpers `hello` line 10,
`account` line 17), world, under 0.5 s each:
`a_restart_with_workers_in_process_settles_their_claims_at_once`,
`a_restart_with_workers_in_process_assigns_without_a_grace`,
`a_restart_with_workers_elsewhere_still_waits_for_them`.
Proof: world: `heavy cargo nextest run -p jig-core-world --test restart
a_restart_with_workers_` shows a worker's claim settled at once and its
task assigned again without waiting the grace when workers are in
process, and adopted within the grace when they are not; once, under
2 s.
Don't: change the fleet (its `Adopt` arm for the engine settles the
claim as it is).
Size: at most 30 lines of production code.
Removes: nothing.

## Interface sketches

```rust
// jig_domain_core::routing::vocabulary: owned here; jig 27's service sends
// `Stopping` and closes its components on `Drained`.
use skein_lib::ReplyTo;

pub enum Event {
    /// From the root, once its service drains (engine.md, 6.1). From then on
    /// the core takes nothing that would make a commit. No terminal; a
    /// repeat changes nothing.
    Stopping,
    // as today
}

pub enum Held {
    /// Nothing the core holds waits for a commit: made once after
    /// `Stopping`, by the decision that ends the last watch; released when
    /// every commit before it is durable. The service then closes its
    /// components.
    Drained,
    // as today
}

pub enum Now {
    /// A call the core will not decide while the engine stops: the root
    /// answers the fleet's call `to` with its busy answer, and the run asks
    /// it again, here or at the next engine.
    CallBusy { to: ReplyTo },
    // as today
}

// jig_domain_core (lib.rs)
impl Core {
    /// Whether the engine is stopping. A root leaves connectors' events and
    /// outcomes with their connectors while it is.
    #[must_use]
    pub fn stopping(&self) -> bool {
        todo!()
    }
}
pub struct Config {
    /// The engine's workers are in its process (a workers' listener on an
    /// in-process name): their claims are settled at once at a restart.
    pub workers_in_process: bool,
    // deployment, seed, owners, authority, settings, projects,
    // permission_roles, connectors: as jig 07 leaves them.
}
```

- **Who owns what:** the core decides what it refuses and when nothing
  waits; a root routes the stop in, `Drained` out, a busy call to its
  host, and holds its connectors back on `stopping()`. The reference
  service (jig 27.2) refuses openings and clients first and closes its
  components on `Drained` (service.md, section 5, steps 1 and 4).
- **What ends what:** `Stopping` ends with `Drained`; a second signal's
  abort is the service's, which drops what is in flight.

## Questions to raise when reached

- **A run's answer during the drain:** engine.md, 6.1 says it is "left
  unacknowledged"; service.md, section 5 says "a run's turn, answer or
  call is answered busy". The fleet has no busy for an answer, and a
  host sends an unacknowledged answer again after its next hello
  (channel.md, 5.2), so 09.1 leaves it unacknowledged. Raise it only if
  a host is found to need a busy answer.

## Done when

- A stopped core takes no commit and says drained once: `heavy cargo
  nextest run -p jig-core-world --test stopping` passes.
- The referee holds a drained shutdown: `heavy cargo nextest run -p
  jig-conformance-world --test restart a_drained_shutdown` passes.
- In-process workers' claims settle at once: `heavy cargo nextest run
  -p jig-core-world --test restart a_restart_with_workers_` passes.
- The gate passed on this session's last landing: `git -C ~/src/rust/jig
  log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step:.jig-09\.' origin/main | head -1` names
  `scripts/check.sh passed`.
