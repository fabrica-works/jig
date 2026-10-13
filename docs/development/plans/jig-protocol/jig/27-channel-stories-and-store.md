# Session 27: the channel world's stories, stopping, and the store in the service

The second half of jig 25 (split for size): the channel's protocol
world's stories, its referee and fuzzy sweep; stopping and draining the
reference service (jig 09); and the service's real store (jig 18) in
place of its temporary port.

## Read first

jig 25's "Read first", "What the design asks", "Choices this plan
makes" and "Interface sketches" (`jig/25-test-service.md`): they bind
this session as they bind jig 25.

## Rules that bind it

- "**Step code** follows skein's `programming-model.md`, section 10, and
  jig's `AGENTS.md`: no traits, no generic items, no `dyn`, no closures.
  `jig-test-service` and `jig-test-worker` are step code so that they
  can be copied" (`code-organization.md`, section 2).
- "**`jig-test-worker` and `jig-test-service`** may link smith's
  `smith-host-domain`, `smith-host-protocol`, `smith-protocol-channel` and
  `smith-channel`." No other smith crate in their `[dependencies]`; the
  channel's components link none (`code-organization.md`, section 3).
- "No increment of this plan ... adds an `unreachable!` arm for a variant
  the reference service can reach; an arm the service reaches is
  routed." (`code-organization.md`, section 6) Each root output the
  service routes is matched without a wildcard.
- **`tests/channel` aims for at most 0.8 s focused and 4 s fuzzy,**
  serially, each focused test under 0.5 s and each fuzzy test about 5 s.
  Missed: profile with the `measure` profile and make the tests cheaper
  before landing; past twice those figures, a question (development.md,
  5.3).

## Increments

### 27.s1 A turn's read is cumulative

A step not in the plan (decisions.md, ruling 258), found by 27.1's
current-window story on the real agent: smith sends each turn "the last
message it read, by name, if any" (`protocol/channel.md`, the turn row),
so consecutive turns repeat a read already committed. jig's tasks domain
admits a turn's read only when it names a message still in the inbox
(`crates/jig-domain-tasks/src/admission.rs:80-84`,
`inbox.rs:433-455`), and the message the first turn read was removed when
it committed; the next turn is refused, and the attempt ends
`Failed(Invalid)` (`logs/jig-27/27.1-dev-window11.txt`).
Needs: nothing; before 27.1 lands. Run by jig 27's session as its own
commit.
Files: `crates/jig-domain-core/src/routing/run_payloads.rs` and its tests
(ruling 260); the tasks domain unchanged.
Shape: a turn's read is admitted when it names a message still in the
inbox (read now) or the attempt's last committed read (nothing new read);
any other name is refused as now. The core holds the last committed read
(`RunProof.turn.read`, `routing/run_payloads.rs:31-35`, where decision 32
compares an answer's fence): a turn whose read equals it passes no read to
the tasks child, the retained payload keeping the read as sent; a read
before it is refused (ruling 260).
Test shape: rows in the admission tests: a turn repeating the attempt's
last committed read is admitted and reads nothing new; a turn naming a
message neither in the inbox nor the attempt's last read is refused.
27.1's current-window story is the world's proof.
Proof: 27.1's current-window story passes on the real agent; the gate.
Don't: accept any earlier name than the attempt's last committed read;
change smith or the wire.
Size: at most 40 lines of production code.

### 27.s2 A stopping worker waits, not spins

A step not in the plan (decisions.md, ruling 263), found by 27.1's
mid-run shutdown story on the real agent: `jig-test-worker`'s
`Domain::work_pending` returns `... || self.stopping`
(`testing/jig-test-worker/src/domain.rs:100`), and stopping lasts until no
run is hosted, no agent lives and nothing is pending (`route`, :215-217).
While it waits on an agent's IO or a stop deadline it reports work
pending without any, so a loop spins and the simulated world's clock never
advances (`logs/jig-27/27.1-dev-scenes5.txt`).
Needs: nothing; before 27.1 lands. Run by jig 27's session as its own
commit.
Files: `testing/jig-test-worker/src/domain.rs` and its tests.
Shape: while stopping, `work_pending` is true only when the final stop
can be taken now (nothing hosted, no agent, nothing pending) or other
work is pending; otherwise the worker waits for its children's IO and
its deadlines like any idle root (`next_deadline`).
Test shape: a row: a stopping worker with a live agent and no IO reports
no work pending and names its stop deadline; one whose last agent has
ended reports the stop pending. 27.1's shutdown story is the world proof.
Proof: 27.1's mid-run shutdown story passes; the gate.
Don't: change a public boundary; poll.
Size: at most 20 lines of production code.

### 27.1 The channel's stories, its referee and the fuzzy sweep

Scope: `tests/channel/src/{referee.rs, relay.rs, stories.rs}`: the
referee's rules as `channel.md`, section 12 lists them, the last checked
by searching each root's formatted state for each grant value's bytes;
a refused start's answer reaches the engine at most once and is never
acknowledged, its start ending once, refused or lost (jig `332724e6`;
`channel.md`, 5.1);
the hub's world's stories through bytes (`tests/host/tests/simulation.rs`
and `turns.rs`: runs that progress, call, take turns, wait, end; a link
dropped for less and for more than the grace; shutdown); the channel's
own: a connection cut mid-frame and the worker redialling (the
simulator's resets, `Config::chaos`); a peer that stops reading and one
that goes quiet (the relay); a partition each side notices at a
different time (the relay), no attempt placed early; garbage frames, a
wrong secret, another version, another application version, terms that
do not fit (a `skein-fake-channel` peer process); a worker dialling again
while its old channel looks alive; the engine killed and restarted while
the worker keeps its run, adopted at its hello; an assignment whose
charter or transcript does not decode; smith's facts reaching the views
as activity, with no text. With jig 04: an assignment carrying its
task's inbox, a message refused for each of smith's reasons and
committed with the next turn or the answer, the engine restarting
between. With jig 06: a run resumed after a compaction, sent only its
current window, with its conversation's identity. Fuzzy: drawn worlds
over seeds.
Needs: jig 04's refusals committed with turns and jig 06's windows, for
their stories: both at the session's start (README).
Files: `tests/channel/src/{referee.rs, relay.rs, stories.rs, world.rs,
processes.rs, fixtures.rs}`, `tests/channel/tests/{channel.rs,
referee.rs, fuzzy_channel.rs}`.
Shape: `referee::Referee` implementing `skein_world::Referee<Proc>`.
Test shape: focused (`tests/channel.rs`), a few seeds each:
`a_link_cut_mid_frame_redials_and_each_turn_reaches_the_engine_once`,
`a_worker_that_stops_reading_is_lost_by_its_stall_deadline`,
`a_partition_seen_late_by_the_engine_places_no_attempt_early`,
`a_wrong_secret_is_refused_as_unauthorized`,
`a_restarted_engine_adopts_the_worker_s_run_at_its_hello`,
`an_undecodable_charter_is_refused_as_invalid_before_the_agent_starts`,
`a_message_refused_by_the_run_is_committed_with_its_next_turn`,
`a_resumed_run_is_sent_only_its_current_window`. Referee's negatives
(`tests/referee.rs`): `an_answer_acknowledged_twice_is_caught`,
`a_refused_start_acknowledged_is_caught`,
`a_grant_value_planted_in_a_root_s_state_is_caught`. Fuzzy
(`tests/fuzzy_channel.rs`): `drawn_channel_worlds_keep_the_referee_s_rules`,
seeds sized to about 3 s. Ceilings: focused under 0.5 s each. Reference
test of the referee's kind (`tests/referee.rs`), in full:

```rust
#[test]
fn an_answer_acknowledged_twice_is_caught() {
    let mut referee = fixtures::referee();
    referee.answered(fixtures::TASK, fixtures::ATTEMPT);
    referee.acknowledged(fixtures::TASK, fixtures::ATTEMPT);
    referee.acknowledged(fixtures::TASK, fixtures::ATTEMPT);
    assert!(matches!(referee.verdict(), Verdict::Failed(_)));
}
```

Proof: joint, `heavy cargo nextest run -p jig-channel-world` and `heavy
cargo nextest run -p jig-channel-world --profile fuzzy`, each run once:
each focused story keeps the referee's rules, the referee catches its
two planted breaches, and drawn worlds keep its rules; under 4.5 s
together.
Don't: assert a story's timing below what the simulator draws (assert
the referee's rules instead).
Size: world code only.
Removes: nothing.

### 27.2 Stopping and draining

Scope: `stop.rs`: a shutdown (io's `Shutdown` from the adopted signal
source, or `Service::shutdown`, which the worlds call) drains the
engine's layer as `service.md`, section 5 orders it: the listeners stop
admitting (jig 23's `Stop`; the parties' port's requests answered not
ready by the core); the test root's `Event::Stop`, which it decides as
the core's `Event::Stopping` (jig 09); `Now::CallBusy`, which the test
root answers through the fleet (jig 09), its busy relayed answer routed
to the workers' component as any relayed answer; a connector's event
the root hands back as `Request::Busy { event }` (jig 09) kept by the
service in a bounded queue and dropped at the end, as a restart
recovers it; released outputs leaving until the core's `Held::Drained`;
then each component closed and its one `closed` awaited (the workers',
the accounts' child by jig 08's `accounts::Event::Close` and then
skein's driver, then the store's port, the store from 27.3 on, last); the engine's layer ends `End::Drained`. A second
shutdown aborts each component and ends `End::Aborted`. A failed commit
during the drain stops as before. A worker's layer on a shutdown with no
engine in the service: `Worker::stop` (25.5). jig 09's
`Config::workers_in_process` reaches the core as jig 20 sets it (an
in-process workers' listener).
Needs: jig 09's `Event::Stopping` with the test root's `Event::Stop` and
`Request::Busy`, `Now::CallBusy`, `Held::Drained` and
`Config::workers_in_process`, at the session's start (README).
Files: `testing/jig-test-service/src/{stop.rs, lib.rs, engine.rs,
route.rs, startup.rs, boundary.rs, tests/stop.rs}`;
`tests/channel/{src/world.rs, tests/channel.rs}`. Shared hotspots:
`testing/jig-test-service/src/route.rs`.
Shape: `Service::shutdown(&mut self)`; `End::{Drained, Aborted,
Stopped(Stop)}`; `stop::Drain { phase: Phase }` with `Phase::{Stopping,
Closing { waiting: u32 }, Drained}`.
Test shape: step tests in `src/tests/stop.rs`:
`a_shutdown_stops_the_listeners_before_the_core_hears_stopping`,
`a_call_the_root_answers_busy_while_stopping_reaches_its_worker`,
`components_close_only_after_the_core_says_drained`,
`the_store_closes_last`,
`a_second_shutdown_aborts_each_component`,
`an_in_process_workers_listener_sets_workers_in_process`. Under 20 ms
each. Reference test, in full:

```rust
#[test]
fn components_close_only_after_the_core_says_drained() {
    let mut driven = fixtures::Driven::started(&fixtures::engine_document());
    driven.shutdown();
    assert_eq!(driven.closes_asked(), 0);
    driven.core_says_drained();
    assert_eq!(driven.closes_asked(), fixtures::ENGINE_COMPONENTS);
}
```

World (`tests/channel.rs`):
`a_shutdown_mid_run_drains_with_nothing_waiting_for_a_commit`, under
0.5 s.
Proof: joint:
`a_shutdown_mid_run_drains_with_nothing_waiting_for_a_commit`, run once
with `heavy cargo nextest run -p jig-channel-world -E
'test(a_shutdown_mid_run_drains_with_nothing_waiting_for_a_commit)'`,
which shows a drained engine leaves no commit waiting and the worker's
turn kept for after the restart; under 0.5 s.
Don't: close a component before `Held::Drained`, except on a second
shutdown.
Size: at most 350 lines.
Removes: nothing.

### 27.3 The store

Scope: `jig-protocol-store` (jig 13 to 17) as the engine's child, with
the test application's keys and codecs (jig 18), on the deployment's
directory, a startup root: commits, restart loads, reads, transcripts,
the secrets' records; opened before the components in the start's order
(`service.md`, section 3); a store's refusal at open ends the start. The
store's port removed. `tests/channel`'s engine on the simulator's machine
(`skein_fake_machine`), its files kept across `Cut::Kill`; `jig-fake-store`
gone from `tests/channel`.
Needs: jig 18's test application's keys and codecs, beyond the
session's start; the store's limits in jig 20's profiles (added here if
jig 13 to 17 have not).
Files: `testing/jig-test-service/src/{store.rs, engine.rs, route.rs,
startup.rs, boundary.rs, tests/engine.rs}`; `tests/channel/{Cargo.toml,
src/world.rs, src/processes.rs}`; `crates/jig-protocol-config/src/profile*.rs`
if the store's limits are not there. Shared hotspots:
`testing/jig-test-service/src/route.rs`, jig 20's profiles.
Shape: `store.rs` routing the root's commits, loads and reads to the
store's boundary (jig 13) and its answers back as root events.
Test shape: step tests: `a_store_refusal_at_open_ends_the_start`,
`a_commit_reaches_the_store_and_its_answer_the_journal`. World:
`a_restart_reads_back_only_what_was_durable`. Ceilings as above.
Proof: joint: `a_restart_reads_back_only_what_was_durable`, run once
with `heavy cargo nextest run -p jig-channel-world -E
'test(a_restart_reads_back_only_what_was_durable)'`: an engine killed
and restarted on the real store reads back the commits that were durable
before the kill and none it held; under 0.5 s.
Don't: keep a commit in the service between iterations.
Size: at most 300 lines.
Removes: the store's port (`Engine::store_requests`,
`Engine::store_answers`); `jig-fake-store` from `tests/channel/Cargo.toml`.

## Done when

- The service and the world pass: `heavy cargo nextest run -p
  jig-test-service -p jig-channel-world` passes, and `heavy cargo nextest
  run -p jig-channel-world --profile fuzzy` passes.
- The world's cost, measured once after 27.1: `heavy --gate cargo nextest
  run -p jig-channel-world --profile measure -j 1` and the same with
  `--ignore-default-filter -E 'binary(/^fuzzy_/)'`; the commit body of
  27.1 records both sums (aim: at most 0.8 s and 4 s).
- The fake store has left the channel's world (27.3): `git -C
  ~/src/rust/jig grep -n jig-fake-store -- tests/channel` prints nothing.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/jig log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step:.jig-27\.' origin/main | head -1` names
  `scripts/check.sh passed`.
