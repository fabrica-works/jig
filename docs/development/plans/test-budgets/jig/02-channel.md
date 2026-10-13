# Session 02: the channel world

The channel world (`tests/channel`, `jig-channel-world`) is 26 focused
tests and 21.5 s of the gate's 119 s summed time on jig's `main`
(`d91a89c45`): about 830 ms each in the gate, 6.8 s alone in the audit's
runs (9.1 s in the planner's full `-j 1` run, on a busier machine). Most
of it is the referee: it formats both roots' whole state with `{:?}`
(about 22 KB for the engine's) on every iteration of the world, and 91
to 93 % of those dumps are the same as the last. The rest is the
engine's configuration document parsed again for each limit a fixture
or a probe wants, and a store observer that parses the whole log and
checks every frame's checksum on every sync. Three increments; measured
together in the audit, 6.83 s to 2.72 s alone, the fuzzy sweep 7.02 s to
1.46 s.

## Read first

- jig's `docs/development/workflow.md`, section 2 ("Checks on change";
  "Traces on failure"; "Fixtures once").
- skein's `testing-strategy.md`, section 8.
- `findings/cost-jig-channel.md`: the profile of five stories, each
  change, its measured effect, and what it left alone;
  `findings/tried/jig-channel.md`.
- jig's `jig-protocol/decisions.md`, ruling 268 (the world's first
  reductions).

## What the design asks

- "**Checks on change.** Check what changed when it changes. Check the
  whole state at the end, unless waiting would lose the bug the check
  guards" (workflow.md, 2).
- "**Traces on failure.** Keep typed observations and format them only on
  failure" (workflow.md, 2).
- "**Fixtures once.** Build a phrasebook, configuration or worst-case
  size once per test process or world when it is a fixture, then share
  it" (workflow.md, 2).

## Choices this plan makes

- **A root's state changes only through a root input, so the referee
  checks it when one moved.** `jig-test-service` counts each input
  handed to a root and each pass that took output from one, and
  `Service::root_steps()` returns the count; the referee keys
  `roots_observed` on it instead of `revision.1`, which every `iterate`
  bumps (`tests/channel/src/processes.rs:72`). The count is kept where
  the roots are stepped: `Engine::step` and the parties' `try_step`
  (`engine.rs`), `restore_record` (`route.rs`), `inbox_census`
  (`store.rs`), the worker's root step (`worker.rs`, on `Worker`, not
  `WorkerRuntime`, which callers build by struct literal); a `release`
  pass counts when it returned requests (`engine.rs`, `release`). The
  referee also checks both roots of each host once at the world's end,
  whatever the count: a change a `release` made with no request is caught
  there.
- **The fixture's store limits are parsed once per test process**
  (`fixtures::store_limits`, a `OnceLock`, as `tests/client` does), and
  the probe peer keeps the limits it is built with (`relay.rs:88`).
- **The store observer applies each frame once:** `frames(bytes, after)`
  skips, by its length, a frame at or below the last applied, whose
  checksum was checked when it was applied.

## Rules that bind it

- No increment "drops an assertion, or changes an expected value, except
  a size its session file names with the reason the assertion holds at
  it" (code-organization.md, section 4): the referee's rule, its search
  and `a_grant_value_planted_in_a_root_s_state_is_caught` stay.
- "**Traces on failure**" (workflow.md, 2).
- `testing/jig-test-service` is shared with jig 01.2, which edits
  `startup.rs` only; this session edits `lib.rs`, `engine.rs`,
  `route.rs`, `store.rs` and `worker.rs` (code-organization.md, section
  2).

## Increments

### 02.1 The referee checks a root's state when a root input moved
Scope: workflow.md, 2 ("Checks on change"; "Traces on failure").
Needs: nothing.
Files: `testing/jig-test-service/src/{lib.rs,engine.rs,route.rs,store.rs,worker.rs}`
  (the count, `Service::root_steps`); `tests/channel/src/referee.rs`
  (`:520-545`, `roots_observed`; the end-of-world check);
  `testing/jig-test-service/src/tests/` (the count's test).
Shape:
  ```rust
  // testing/jig-test-service/src/lib.rs
  impl Service {
      /// Inputs handed to the roots, and passes that took output from them,
      /// so far: a root's state changes only with one, so a check of the
      /// roots' state need run only when it moved.
      #[must_use]
      pub fn root_steps(&self) -> u64;
  }
  ```
  `Engine` and `Worker` each hold `root_steps: u64`, moved with
  `wrapping_add(1)` at each site "Choices" names. The referee compares
  `host.service.root_steps()` where it compared `host.revision.1`, and
  checks both roots of each host in its end-of-world pass.
Test shape: in `testing/jig-test-service/src/tests/`:
  `a_roots_step_count_moves_with_an_input_and_not_with_an_idle_iterate`.
  In `tests/channel`: the stories, unchanged, and
  `a_grant_value_planted_in_a_root_s_state_is_caught` unchanged.
Proof: world, `heavy --gate cargo nextest run -p jig-channel-world
  --profile measure -j 1 -E 'binary(channel)'` once: every test passes;
  the binary's summed time and
  `a_resumed_run_is_sent_only_its_current_window`'s, in the report (the
  audit: 6.83 s to 4.38 s, and 0.978 s to 0.473 s).
Don't: change the rule, its search, or what a dump holds; count an
  idle `iterate`; format a root's state for another reason.
Size: at most 35 lines of test kit.
Removes: the per-iteration dumps.

### 02.2 The fixtures parse the engine's document once
Scope: workflow.md, 2 ("Fixtures once").
Needs: nothing.
Files: `tests/channel/src/fixtures.rs` (`store_limits` `:422-436`,
  `peer_bounds`), `tests/channel/src/relay.rs` (`Peer` `:43`, its
  construction `:88`, the refusal `:215`, the heartbeat `:323`),
  `tests/channel/src/processes.rs` (`:47,49`).
Shape: `store_limits()` returns a copy of a `static
  OnceLock<jig_test_service::limits::Limits>` filled on first call;
  `Peer` gains `limits: skein_channel::Limits`, set where it is built
  (`:88`), and the refusal and the heartbeat use it instead of calling
  `peer_bounds()`.
Test shape: the stories, unchanged.
Proof: world, the command of 02.1's Proof, once (it waits for no other
  increment): the binary's summed time, and
  `a_worker_that_stops_reading_is_lost_by_its_stall_deadline`'s (the
  audit: 0.111 s to 0.036 s), in the report.
Don't: change a document, a limit or a fixture's value.
Size: at most 15 lines of test code.
Removes: the repeated parses.

### 02.3 The store observer applies each frame once; the transcript's checksum by table
Scope: workflow.md, 2 ("Checks on change").
Needs: nothing.
Files: `tests/channel/src/world/store.rs` (`Capture` `:31`, its parse
  `:92`; `Store` `:119`, the sync `:151`, the recovery `:159`, `:174`;
  `frames` `:251-285`); `crates/jig-protocol-store/src/transcripts/frame.rs`
  (`crc32c` `:47-56`); `crates/jig-protocol-store/src/tests/frames.rs`.
Shape: `fn frames(bytes: &[u8], after: u64) -> Vec<(u64,
  Box<[skein_kv::Op]>)>`: a frame numbered at or below `after` is skipped
  by its length, its body and checksum unread; the sync path passes
  `self.applied`, recovery and capture pass 0. `crc32c` looks up a
  `const TABLE: [u32; 256]` (CRC-32C's reflected polynomial,
  `0x82f6_3b78`) once per byte, in place of eight shifts: the same
  results, in production as in tests.
Test shape: `crc32c_matches_its_check_value`
  (`crates/jig-protocol-store/src/tests/frames.rs:39`), unchanged; one
  new test there, `crc32c_by_table_matches_the_bitwise_loop`: the
  checksum of each of the 256 one-byte inputs, and of 64 seeded random
  inputs of up to 1,000 bytes, equals the old bitwise loop's, which the
  test keeps as its reference; the stories, unchanged.
Proof: world, the command of 02.1's Proof, once: the binary's summed
  time, and the window story's, in the report.
Don't: change a frame's format, the checksum's value, or what recovery
  reads.
Size: at most 30 lines of production and test kit code.
Removes: the bitwise loop, and re-reading applied frames.

## Done when

- `heavy --gate cargo nextest run -p jig-channel-world --profile measure
  -j 1 -E 'binary(channel)'` on jig's `main`: every test passes, summed
  at most 3.5 s (6.83 s in the audit before; its three changes, with jig
  01.2's, reached 2.72 s).
