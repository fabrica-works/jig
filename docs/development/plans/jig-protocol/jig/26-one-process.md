# Session 26: one process, the reference binary, and the process and live proofs

jig's first shape (`protocol/README.md`, section 4) runs an engine and
its worker in one process, joined by skein's in-process addresses, with
a native client on loopback. This session checks it at three layers.
The one-process world, `tests/one-process`, runs the reference service
holding both layers in one process of skein's simulated world, the
store on the simulator's crashing disk, smith's real agent processes
over skein's fake LLM, native clients over loopback; it kills the
process at drawn moments, drains it by a signal, and judges it with the
conformance world's referee seen through bytes. The reference binary,
`jig-test`, is the template an application copies for its `main`: it
reads its configuration with skein's startup kit, opens the store on a
directory, runs the engine and one test worker in one process, and runs
smith's agent shell as its own `agent` command. The process proof starts
it on a scratch directory beside skein's fake LLM peer, signs a native
client in by the printed link, has an agent run answer it, kills the
binary, starts it again and finds the work. The live proof does the same
once with a real model, the user's Codex login borrowed read only.

## Read first

- jig's `protocol/service.md`, sections 3, 5, 6, 7, 8 and 10;
  `protocol/README.md`, section 4; `protocol/configuration.md`, sections
  3, 4, 7, 8 and 10; `protocol/llm.md`, 5.4; `protocol/client.md`,
  section 8; `protocol/channel.md`, 10.1 and 10.2.
- jig's `domain/engine.md`, 6.1 and section 6 (claims of workers in the
  engine's process settled at once); `domain/hosts.md`, 6.6 and 6.7.
- This plan's `jig/09-stopping.md` (`Config::workers_in_process`) and
  `jig/27-channel-stories-and-store.md`, 27.2 (the engine's drain).
- skein's `io.md`, 3.5 and section 4 (joined sockets never reach the
  simulator); skein's `shell.md`, sections 5, 6 and 6.1; skein's
  `testing-strategy.md`, 2.7 and 2.9; skein's `fake-llm.md`.
- `decisions.md`, 5, 11 and 12; README "Rules" (the process proof's
  script) and "Live services".
- findings/protocol.md, items 3.4 (smith's agent shell as a command), 3.5
  (smith's agent in worlds), 4.6 (the startup kit, `Host`, `drive`), 4.9
  (the fake LLM peer, the real world, `end_to_end`) and 5.4 (nextest and
  the gate).
- The code: jig 25's `testing/jig-test-service` (`Service`, `startup`,
  `named`, `Worker::stop`) and `tests/channel`; jig 33's
  `jig_client_protocol_world::NativeClient` and
  `crates/jig-shell-native`; jig 18's store world (the store on the
  crashing disk); skein's `examples/echo/shell/src/{main.rs, lib.rs}`
  (a `main`, a `Host` over a step-code service); smith's `src/main.rs`
  (the `agent` command), `tests/end-to-end/support/mod.rs:286-340` (a
  shipped binary in skein's real world beside the fake LLM peer) and
  `tests/end-to-end/support/live.rs` (live settings);
  `.config/nextest.toml`.

## What the design asks

- **One service, one loop** (`service.md`, section 10): the
  application's `iterate` "owns the engine's protocol layer and each
  worker's as children, each with its own root and journal"; "Joined by
  in-process addresses ... so the channel's opening, authentication,
  liveness, relays and grants run as they do between hosts, in
  plaintext, with no TLS"; credentials granted "with each assignment over
  the in-process link".
- **Starting and ending together** (section 10): "configuration for all,
  then the engine's start, then each worker's. A signal drains the
  engine; once it has drained, the service ends the workers at once, with
  no grace ... The next start settles their claims at once".
- **A shutdown drains** (section 5): the listeners stop admitting; the
  core hears the engine is stopping; what is in flight finishes until
  nothing waits for a commit; each component closes, the store last; "A
  second signal turns the drain into an abort".
- **The protocol world** (section 8): "a store on the simulator's
  crashing disk, scripted parties' connections, skein's fake LLM peer,
  smith's agent processes over skein's fake LLM ..., the process killed
  and started again at drawn moments, and drained by a shutdown. Its
  referee is the conformance world's, seen through bytes".
- **The reference binary** (section 7): "its `main` reads its
  configuration with skein's startup kit, opens the store on a
  directory, and runs the engine and one test worker in one process ...,
  the worker's agents smith's real agent processes, run as its own
  `agent` command. A native client on loopback signs in by the link it
  prints."
- **The process layer** (section 8): "`jig-test`, started on a scratch
  directory with skein's fake LLM peer as a process of its own on
  loopback ... as its endpoint, a person's requests answered through a
  client on loopback, runs taken by smith's real agent processes, then
  the binary killed and restarted, and the work found where it was".
- **The live layer** (section 8): "`jig-test` with a real LLM account
  handed in (`llm.md`, 5.4), a native client signing in by the printed
  link, and a request answered by a real model, within a bound on calls."
- **A handed-in account** (`llm.md`, 5.4): "an access token and the wall
  time it lapses, in a private file the configuration names ...; the
  driver lends it until it lapses, never writing to its source."

## Choices this plan makes

- **The composition is jig 25's `Service`:** one document names the
  engine and one test worker, the workers' listener and the worker's
  engine address one in-process name (`{"in_process": "workers"}`,
  `configuration.md`, section 10). No world code composes the layers.
- **Native clients are jig 33's `NativeClient`,** in the simulated world
  and in the process and live proofs; each signs in by the link the
  service hands its caller, as a person reads it from the terminal.
- **The engine's drain is jig 27's** (27.2: the listeners stopped, the
  core told it is stopping, `Held::Drained`, each component closed, the
  store last, `End::Drained`; a second signal aborts). This session adds
  what one process needs after it (`service.md`, section 10): once the
  engine's layer has ended `Drained`, `Service::shutdown` aborts each
  worker's layer at once (`Worker::abort`), with no grace; the service
  ends once io holds nothing. jig 33.3 adds the clients' component to
  the components 27.2 closes.
- **The process and live proofs are tests outside both suites:**
  `testing/jig-test/tests/process.rs` and `tests/live.rs`, two `[[test]]`
  binaries named `process` and `live` that `.config/nextest.toml`'s
  default and fuzzy profiles filter out, each with a profile of its own,
  as smith keeps its `live` (`smith/.config/nextest.toml`). They use
  skein's real world (`skein_world::real::World`) to host skein's fake
  LLM peer and the native client beside the binary, which
  `skein_world::end_to_end::Binary` starts, as smith's end-to-end
  stories do. The scripts run them once.
- **`jig-test`'s commands:** `jig-test serve CONFIG.json` and `jig-test
  agent AGENT.json`; the printed link goes to standard output, one line;
  diagnostics to standard error.
- **The live proof's account** comes from `~/.codex/auth.json`, read
  once by `scripts/proof-live.sh`: `tokens.access_token`, its expiry (the
  token's `exp` claim) and `tokens.account_id` are written to a `0o600`
  file under the run's `logs/jig-26/`, which the generated configuration
  names as a handed-in account. Nothing writes to `~/.codex`; no refresh
  token is copied. The model is smith's small Codex tier
  (`smith/benchmarks/agents/models.toml`, `[small.codex]`), unless
  `JIG_LIVE_MODEL` names another.

## Rules that bind it

- "**The process proof's script** is written once, by jig 26, as
  `scripts/proof-one-process.sh` in jig, and later increments that prove
  at the process layer run it." (README, "Rules")
- "**An LLM provider, for the live proofs only:** the user's ChatGPT login
  at `~/.codex/auth.json`, borrowed read only and access token only. ...
  No agent signs in, and nothing writes to `~/.codex`." "**Its budget:**
  at most 5 completions and 5 minutes per live proof, at most 40
  completions in the whole run." (README, "Live services")
- "**The budgets bind the gate's wall time:** 15 s focused, 60 s fuzzy"
  (`code-organization.md`, section 5). Neither proof runs in either
  suite; `tests/one-process` aims for at most 0.6 s focused and 3 s
  fuzzy, serially, each focused test under 0.5 s. Missed: profile and
  make cheaper before landing; past twice, a question.

## Increments

### 26.1 One service, joined by an in-process name

Scope: a one-process document: the workers' listener and the worker's
engine address on one in-process name; `startup` refusing a worker's
in-process engine address that names no listener of the same document,
naming its path. `tests/one-process` (`jig-one-process-world`) on skein's
simulated world: one service process holding the engine's and the
worker's layers, its store on the simulator's machine (as jig 18's store
world lays it), its clients' listener on simulated loopback; skein's fake
LLM peer as a process; smith's agent hosted as the worker's agent
program, as `tests/channel` hosts it; one `NativeClient`. One story.
Needs: nothing beyond the session's start.
Files: `testing/jig-test-service/src/{startup.rs, tests/startup.rs}`;
`tests/one-process/{Cargo.toml, clippy.toml, src/{lib.rs, world.rs,
fixtures.rs}, tests/one_process.rs}`; the workspace `Cargo.toml`. Shared
hotspots: the workspace `Cargo.toml`, `testing/jig-test-service`.
Shape: `world::run(settings: Settings, people: Box<[Script]>) ->
Outcome`; `Outcome { verdict, chats_made, runs_answered, drained,
claims_settled_at_restart }`.
Test shape: step: `an_in_process_engine_address_naming_no_listener_is_refused`.
World (`tests/one_process.rs`), in full:

```rust
use jig_one_process_world::{fixtures, world};
use skein_world::domain::Verdict;

#[test]
fn a_person_s_chat_is_answered_by_the_in_process_worker() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), fixtures::one_person_chat(b"Hello"));
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.chats_made, 1);
    assert_eq!(outcome.runs_answered, 1);
}
```

Ceilings: the step test under 20 ms; the story under 0.5 s.
Proof: joint: `a_person_s_chat_is_answered_by_the_in_process_worker`,
run once with `heavy cargo nextest run -p jig-one-process-world -E
'test(a_person_s_chat_is_answered_by_the_in_process_worker)'`, which
shows the engine and worker meet over an in-process name and a native
client over loopback, under 0.5 s; the process layer is 26.5.
Don't: compose the layers in world code.
Size: at most 100 lines of production code; the world is test code.
Removes: nothing.

### 26.2 Draining both

Scope: `Service::shutdown` grown for one process: the engine's layer
drained by 27.2's drain, then each worker's layer aborted at once
(`Worker::abort`: its link and components aborted, its agent processes'
groups killed as io closes them, nothing handed in); the service's end
once io holds nothing; a second signal during the drain aborting each
layer; a restart from the store, the in-process worker's claims settled
at once (jig 09's `Config::workers_in_process`, which 27.2 passes on).
Needs: nothing beyond the session's start: jig 27's 27.2 brings jig 09.
Files: `testing/jig-test-service/src/{stop.rs, lib.rs, worker.rs,
tests/stop.rs}`; `tests/one-process/{src/world.rs,
tests/one_process.rs}`. Shared hotspots: `testing/jig-test-service`.
Shape: as the interface sketch.
Test shape: step: `the_worker_goes_on_while_the_engine_drains`,
`the_worker_is_aborted_only_once_the_engine_has_drained`,
`a_second_signal_aborts_both_layers`,
`the_service_ends_once_io_holds_nothing`. World:
`a_signal_mid_run_drains_then_ends_the_worker_with_nothing_waiting`,
`the_next_start_settles_the_worker_s_claims_at_once`. Ceilings as 26.1.
Proof: joint, `heavy cargo nextest run -p jig-one-process-world -E
'test(a_signal_mid_run_drains_then_ends_the_worker_with_nothing_waiting)
| test(the_next_start_settles_the_worker_s_claims_at_once)'`, run once:
a signal mid-run drains the engine, then ends the worker with nothing
waiting for a commit, and the next start settles the worker's claims at
once; under 1 s together.
Don't: give the worker a grace in one process.
Size: at most 200 lines.
Removes: nothing.

### 26.3 Kills at drawn moments, and the referee through bytes

Scope: `tests/one-process`'s referee: the conformance world's
(`testing/jig-conformance/src/referee.rs`, `Referee::observe`), fed what
the native clients decoded, what the test system and the worker
observed, and the store's durable rows read back after the run: nothing a
party, a host or a system saw depended on a lost commit; a drained
shutdown ends with nothing waiting for a commit. Kills (`World::cut`,
`Cut::Kill`) and power losses (`Cut::PowerLoss`) at drawn moments; the
service started again from its store. Fuzzy: people, runs, kills and a
final drain over seeds.
Needs: nothing beyond 26.2.
Files: `tests/one-process/{src/{referee.rs, world.rs, fixtures.rs},
tests/{one_process.rs, fuzzy_one_process.rs}}`.
Shape: `referee::Referee` implementing `skein_world::Referee<Proc>` over
`jig_conformance::Referee`.
Test shape: focused: `a_kill_before_a_commit_loses_nothing_a_person_saw`,
`a_power_loss_mid_run_restarts_and_adopts_nothing_from_the_dead_worker`.
Fuzzy: `drawn_kills_and_drains_keep_the_conformance_rules`, seeds sized
to about 2.5 s. Ceilings: focused under 0.5 s each.
Proof: joint, `heavy cargo nextest run -p jig-one-process-world
--profile fuzzy`, run once: kills and drains at drawn moments keep the
conformance referee's rules over what crossed bytes and the durable
rows; under 3 s.
Don't: assert on a service's state (assert on what crossed bytes and the
durable rows instead).
Size: world code only.
Removes: nothing.

### 26.4 The reference binary

Scope: `testing/jig-test` (ordinary Rust, a binary `jig-test`, with a
`clippy.toml` copied from `testing/jig-test-system/clippy.toml`):
`jig-test serve CONFIG.json`: the configuration read with
`skein_shell::read_file`; the files `jig_test_service::named` names read
with `skein_shell::read_private_file` (skein 02); the worker's agent
configuration parsed with `smith_agent_shell::config::read`; the seeds
(`skein_shell::seed`, skein 02's cryptographic seed, a drawn deployment
id); the worst case checked; termination signals opened and adopted;
`skein_shell::Kernel::open`; the service made by `startup` and run by
`skein_shell::drive` over a `Host` that wraps it, as skein's echo shell
does; each link from `Service::links` printed to standard output; exit 0
once drained, 1 on a failure. `jig-test agent AGENT.json`:
`smith_agent_shell::run`, as smith's `src/main.rs:17-26` does.
Needs: nothing beyond the session's start.
Files: `testing/jig-test/{Cargo.toml, clippy.toml, src/{main.rs, lib.rs,
host.rs, tests.rs}}`; the workspace `Cargo.toml`. Shared hotspots: the
workspace `Cargo.toml`.
Shape: `jig_test::configure(args) -> Result<Command, String>`;
`jig_test::Host` wrapping `jig_test_service::Service`, implementing
`skein_shell::Host`.
Test shape: unit tests in `src/tests.rs`:
`serve_without_a_configuration_path_is_a_usage_error`,
`a_configuration_file_past_its_bound_is_refused`,
`a_secret_file_readable_by_others_refuses_the_start`. Under 20 ms each.
Proof: unit, `heavy cargo nextest run -p jig-test --lib`, run once: a
start without a configuration path, with a configuration file past its
bound, or with a secret file others may read is refused (the three tests
above); under 0.1 s. The process layer is 26.5, next.
Don't: read a file once the loop runs; print a secret.
Size: at most 300 lines.
Removes: nothing.

### 26.s1 The reference service opens each project's first period

A step not in the plan, from the designer's answer on the reference
binary's period calendar (jig `b4a79914`: `protocol/configuration.md`,
5, "The smallest source"; `protocol/service.md`, 7). The real service
starts with no `core::Event::Period`, so its first chat is refused for
funding; worlds say the period themselves
(`tests/channel/src/referee.rs`, 33.4's world, ruling 275), and the process and live proofs run the
binary with no world. The core takes a period with no next start
(`crates/jig-domain-core/src/routing/vocabulary.rs:426`, `next:
Option<Wall>`).
Needs: nothing.
Files: `testing/jig-test-service/src/{startup.rs, engine.rs}` and its
  tests.
Shape: on a deployment's first start (`configuration.md`, 8: the store
  empty), once the core's restart is done and before it admits a
  party's request, the reference service's root says, for each project
  its configuration seeds, `core::Event::Period { project, period: 1,
  budget, next: None }`, `budget` the project policy's `period_spend`.
  A later start says none.
Test shape: `a_first_start_opens_each_seeded_projects_period` (a chat
  requested first is funded); `a_later_start_says_no_period` (the
  store's period stands, no `Event::Period` said). Under 20 ms each.
Proof: unit, `heavy cargo nextest run -p jig-test-service` once.
Don't: keep a calendar or arm a deadline (later.md); say a period at a
  later start; take a budget past the policy's `period_spend`.
Size: at most 80 lines of production code.

### 26.5 The process proof

Scope: `testing/jig-test/tests/process.rs` (`[[test]] name = "process"`):
a scratch directory with `jig-test`'s configuration (engine and one
worker in one process, joined by an in-process name, the clients'
listener on `127.0.0.1:0`, one handed-in LLM account whose file holds the
fake peer's token, `token`, and its account id, `acc`), the worker's
agent configuration naming the fake peer's address and `jig-test agent`
as the agent program; skein's real world hosting skein's fake LLM peer
(`smith_agent_process_world::fake::configured`, a script of one answer)
and a `NativeClient` on real io; `jig-test serve` started by
`skein_world::end_to_end::Binary::start` (`Mode::Pipes`), its printed
link read from its standard output; the person signs in, starts a chat
with `b"Hello"`, and sees the run's answer in the chat; `SIGKILL` to the
binary; `jig-test serve` started again on the same directory; the person
signs in by the new link and finds the chat with its answer.
`.config/nextest.toml`: the default and fuzzy profiles' filters exclude
`binary(=process)` and `binary(=live)`; a `process` profile
(`default-filter = "binary(=process)"`, `global-timeout = "2m"`,
`test-threads = 1`). `scripts/proof-one-process.sh`: `cargo build -p
jig-test --bin jig-test`, then `cargo nextest run -p jig-test --profile
process`, exiting with its status, at most 30 lines.
Needs: 26.4; 26.s1 (the first period).
Files: `testing/jig-test/{Cargo.toml, tests/process.rs,
tests/support/mod.rs}`; `.config/nextest.toml`;
`scripts/proof-one-process.sh`. Shared hotspots: `.config/nextest.toml`.
Shape: `tests/support/mod.rs`: `scratch()`, `configuration(..)`,
`agent_configuration(..)`, `start(..) -> Binary`, `printed_link(..)`.
Test shape: one test, `a_killed_jig_test_restarts_and_the_work_is_found`,
under 60 s.
Proof: process: `heavy --gate scripts/proof-one-process.sh`, run once,
its output kept in `<run>/logs/jig-26/proof-one-process.log`; it shows a
native client's chat answered by smith's real agent process through the
real binary, and the work found after a kill; bound 2 minutes. Nothing
else is wanted.
Don't: put either binary test in the default or fuzzy suite; use a
thread in the test.
Size: at most 250 lines (test support and script).
Removes: nothing.

### 26.6 The live proof

Scope: `testing/jig-test/tests/live.rs` (`[[test]] name = "live"`): as
26.5's first half, with the configuration and the account file the
script wrote (`JIG_LIVE_CONFIG`), the agent configuration naming the
Codex endpoint (as smith's `tests/end-to-end/support/live.rs` writes
one), the charter's budget at most 5 turns; a native client signs in by
the printed link and starts a chat asking for one short answer; the run
answers with a real model; the test reads the completions the agent
made from the run's turns (at most 5). `.config/nextest.toml`: a `live`
profile (`default-filter = "binary(=live)"`, `global-timeout = "5m"`,
`test-threads = 1`, `retries = 0`). `scripts/proof-live.sh`: reads
`~/.codex/auth.json` once, read only; writes the access token and its
expiry, and the account id, to a `0o600` file under `$JIG_RUN_LOGS`
(default `./logs/jig-26`); writes the configuration naming it; runs
`cargo nextest run -p jig-test --profile live`; deletes the token file;
at most 60 lines.
Needs: nothing beyond 26.5.
Files: `testing/jig-test/tests/live.rs`; `.config/nextest.toml`;
`scripts/proof-live.sh`. Shared hotspots: `.config/nextest.toml`.
Shape: as 26.5's support.
Test shape: one test, `a_native_client_s_chat_is_answered_by_a_real_model`,
under 5 minutes.
Proof: live: `scripts/proof-live.sh`, run once, its output in
`<run>/logs/jig-26/proof-live.log`, the report naming the completions
spent (at most 5) and the wall time (at most 5 minutes). Nothing else is
wanted.
Don't: copy the refresh token or the id token; write to `~/.codex`; run
it twice.
Size: at most 150 lines.
Removes: nothing.

## Interface sketches

```rust
// jig_test_service: what 26 adds (code-organization.md, section 4: jig 25 owns the service;
// its `End` and the engine's drain are 27.2's).
pub struct Service { /* jig 25's */ }
pub struct Worker { /* jig 25's */ }
impl Service {
    /// A signal or an operator, grown for one process: the first drains the engine's
    /// layer (27.2), then, once it has ended Drained, aborts each worker's layer at once
    /// (service.md, section 10); a second aborts each layer. Workers' layers alone, with
    /// no engine, are stopped (jig 25's `Worker::stop`).
    pub fn shutdown(&mut self) { todo!() }
}
impl Worker {
    /// Ends the layer at once, with no grace: its link and components aborted, its
    /// agent processes' groups killed as io closes them, nothing handed in.
    pub fn abort(&mut self) { todo!() }
}
```

```rust
// jig_test (testing/jig-test): ordinary Rust, the reference main.
/// `jig-test serve CONFIG.json` or `jig-test agent AGENT.json`.
pub enum Command { Serve { configuration: std::path::PathBuf }, Agent { configuration: std::path::PathBuf } }
pub fn configure(args: impl Iterator<Item = std::ffi::OsString>) -> Result<Command, String> { todo!() }
/// The service as a host of skein's loop, its links printed as they come.
pub struct Host { /* jig_test_service::Service, standard output */ }
```

- **Who owns what:** the service owns io and its layers, and decides the
  order of the end; `jig-test`'s `main` owns the kernel, the clock, the
  signals and the files it reads before the loop; the proofs' worlds own
  the fake LLM peer, the native client and the binary's lifetime.
- **What ends what:** the engine's layer ends `Drained` once the core says
  nothing waits and each component has said closed; each worker's layer
  is aborted then; the service when both have, io holding nothing;
  `jig-test` exits with the service's end.

## Questions to raise when reached

- **The Codex endpoint and the account id** (26.6): the account id
  travels beside the access token to the worker, which builds smith's
  envelope of both (jig `87195d80`, llm.md, 5.3; jig 25.5); if the
  provider still refuses the grant, report what the live proof needed.
- **The deployment's id on a first start** (`configuration.md`, section
  8: "the shell draws the deployment's id"): `jig-test` draws one at each
  start and the store keeps the first; report if the store's design
  wants it drawn only when the store is empty.

## Done when

- The world passes: `heavy cargo nextest run -p jig-one-process-world -p
  jig-test-service -p jig-test` passes.
- The process proof passes once: `heavy --gate
  scripts/proof-one-process.sh` exits 0; its log is in
  `<run>/logs/jig-26/`.
- The live proof ran once within its budget: `ls
  ~/src/rust/tmp/jig/jig-protocol/logs/jig-26/proof-live.log` lists the
  log, and the report, as `grep -n -i -E 'completions|wall'
  ~/src/rust/tmp/jig/jig-protocol/sessions/jig-26.md` prints it, names
  the completions spent (at most 5) and the wall time (at most 5
  minutes).
- Neither proof is in the suites: `cargo nextest list --workspace` names
  no test of the binaries `process` or `live` (run through `heavy`).
- Nothing wrote to the user's login: `git status` is clean outside the
  session's files, and `scripts/proof-live.sh` contains no write to a
  path under `~/.codex` (`grep -n codex scripts/proof-live.sh` shows
  only the one read).
- The gate passed on this session's last landing: `git -C ~/src/rust/jig
  log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step:.jig-26\.' origin/main | head -1` names
  `scripts/check.sh passed`.
