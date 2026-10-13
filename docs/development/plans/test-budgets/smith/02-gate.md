# Session 02: smith's gate passes here, and its real tier runs in parallel

smith's focused suite takes 21 s on carbon-forge at smith `0305f2fd`,
and seven of its tests fail here while they pass on the reliability
machine. Most of the time is a pin: the reliability plan's `83122ea0`
moved smith's skein back to `cb3cefb`, before skein's base64 decoder
read by table (`8a34235`), and every load of the machine's trust roots
pays about 250 ms again. Six failures are the user's id (carbon-forge's
is 600, the fake disk's 1000), one a deadline meant for refusals at
startup. And the real tier (`smith-real-world` and `end_to_end`, 55
tests) runs one test at a time, because its stories share three fixed
ports. Four increments, the failures first; the audit measured the first three changes
together at 16.7 s (from 21.0 s), and the real tier's stories with the
six local-process ones, 61 tests in parallel, at 2.6 s of wall.

## Read first

- smith's `docs/development/workflow.md`, sections 1, 2 and 5
  (`smoke-tools` before every landing that changes code).
- skein's `testing-strategy.md`, section 8 (as revised: nextest without
  its double spawn).
- `findings/cost-smith.md`, sections 1 to 3, and `findings/measurements.md`,
  section 4.
- jig's `jig-protocol/decisions.md`, rulings 120 and 121 (smith's gate
  here without the timeout), and `jig-protocol/smith-settle.md`, 3
  (skein-02.s8, the decoder).

## What the design asks

- The focused suite "takes at most **15 seconds**" (smith's workflow.md,
  2; testing-strategy.md, 8), measured on carbon-forge (decisions.md, 3).
- "The gate ... starts nextest without its double spawn"
  (testing-strategy.md, 8, as revised).
- skein's `Cargo.toml`: "a workspace that depends on skein-tls copies"
  `[profile.dev.package.rustls] debug-assertions = false`.

## Choices this plan makes

- **smith takes this plan's skein base, `2068257`, at once,** with
  `--precise`, whatever skein has landed since: it carries `8a34235`, and
  smith builds on it with no source change (the audit). skein 01's new
  read reaches smith later, with smith 01.1. If smith's lock already
  holds a later skein when 02.2 starts (smith 01.1 landed first, or the
  reliability plan repinned), the repin commit is left out: a repin
  never moves a pin back (README, "Rules").
- **The simulated disk belongs to the user who runs the test:**
  `Filesystem::for_user(skein_shell::effective_user())`.
- **The terminal sign-in story's deadline is the provider stories' 5 s**
  (`support/mod.rs:325`), not the 1 s of refusals at startup.
- **The window story's terminal is sized before the product starts,** so
  the product's first status line has the story's width.
- **Each real-tier process uses loopback addresses of its own:** its
  fakes bind `127.1.0.0` plus the process's id (below 2^22 on Linux, so
  two live processes never share one; any 127/8 address is local, and
  the product takes any loopback issuer and endpoint); the sign-in
  redirect, which must be `127.0.0.1:<port>` (skein-oauth
  `machine.rs:642-650`), takes a free port from a `bind` to port 0, once a
  process. The simulated tiers keep `127.0.0.1` and their ports. The
  `real-local` test group goes. Until smith 04.1 lands: a smith gate on
  carbon-forge that fails on the redirect's port taken (`AddrInUse`) is a
  question, and the supervisor's answer may put the sign-in stories in a
  test group of one thread, the rest staying parallel.

## Rules that bind it

- "**smith's reliability plan lands in smith and skein from
  debian-16gb-hel1-1** ... Its sessions edit smith's `tests/end-to-end`,
  `tests/local-process` and `crates/smith-local-*`" (README, "Rules").
- "**Every smith landing that changes code runs `smoke-tools` live**
  ..., test code included" (README, "Rules").
- decisions.md, 10: smith lands here with ruling 121's gate (no focused
  timeout) until smith 04.1. That gate runs every focused test, so the
  seven failures fail it: 02.1 lands first, and every other smith
  increment of this plan needs it.

## Increments

### 02.1 The seven failures here
Scope: `findings/cost-smith.md`, 2 and 4 ("A flaky test").
Needs: nothing.
Files: `tests/local-process/src/world.rs:603`;
  `tests/end-to-end/support/exec.rs` (`chat_signed_in` `:120-130`, the
  `Refusal` referee's deadline `:216`); the window story
  (`tests/end-to-end/end_to_end.rs`,
  `a_window_size_change_reaches_the_status_line`) and the terminal it is
  given (`tests/end-to-end/support/window.rs:84-85`, `exec.rs:188,246`).
Shape: `let mut disk = Filesystem::for_user(skein_shell::effective_user());`;
  `chat_signed_in` runs its story under a 5 s deadline, the refusals keep
  1 s; the window story's terminal gets its first size before the binary
  is spawned.
Test shape: the seven tests, unchanged in name and expectation;
  `a_window_size_change_reaches_the_status_line` passes in the six runs
  of the Proof.
Proof: world, `heavy --gate cargo nextest run -p
  smith-local-process-world -p smith --profile measure` once, every test
  passing; then `heavy --gate bash -c 'for i in 1 2 3 4 5 6; do cargo
  nextest run -p smith --profile measure -E "binary(end_to_end)" || exit
  1; done'` once (six runs of `end_to_end` at 8 threads, where the window
  story failed), all passing; `smoke-tools` once before `land`.
Don't: change a referee's rule, a token store's check, or the fake
  disk's default for other worlds.
Size: test code only, at most 25 lines.
Removes: nothing.

### 02.2 smith on skein `2068257`; nextest without its double spawn; rustls as it ships
Scope: `findings/cost-smith.md`, 1; testing-strategy.md, 8.
Needs: 02.1.
Files: `Cargo.lock` (`heavy cargo update -p skein-lib --precise
  206825786a507cf5498c21ee3f1c2cbe79fc33da`, a commit of its own, first;
  if `tests/codecs/tests/drift.rs` then fails, smith's families
  regenerate as smith 01.1's Files say, golden files unchanged);
  `scripts/check.sh`; `Cargo.toml` (`[profile.dev.package.rustls]
  debug-assertions = false`, with skein's comment);
  `docs/development/workflow.md`, section 2's commands.
Shape: `export NEXTEST_DOUBLE_SPAWN=0` before the first `cargo nextest`
  of `scripts/check.sh`; the profile as skein's `Cargo.toml:105-111`.
Test shape: none new.
Proof: process, the focused time in `land`'s `Gate-Result` (decisions.md,
  10's gate), against 21.0 s at this plan's base
  (`findings/measurements.md`, 1); `smoke-tools` once before `land`.
Don't: move a package of another source; change smith's source.
Size: `Cargo.lock`, a profile, one line.
Removes: nothing.

### 02.3 The real tier's stories each on addresses of their own
Scope: `findings/cost-smith.md`, 3.
Needs: 02.1.
Files (the audit's list): `tests/real/src/lib.rs` (`Scratch::new`, the
  process's addresses); `tests/agent-process/src/fake.rs:121`;
  `tests/end-to-end/support/{mod.rs,exec.rs,login.rs}` (`:269,276-277,349,388`;
  `:31,240`; `:99`); `tests/end-to-end/end_to_end.rs:207-208,233,906`;
  `tests/local-process/src/process.rs:106,127,135`;
  `tests/local-process/src/process/configuration.rs:292,328-352`;
  `.config/nextest.toml` (`[test-groups.real-local]` and its override,
  `:31-36`).
Shape: one function in `smith-real-world`, `pub fn loopback() -> Ipv4Addr`,
  `127.1.0.0` plus `std::process::id()`, and `pub fn redirect_port() ->
  u16`, a port from a `bind` to `127.0.0.1:0` taken once a process
  (`OnceLock`) and released before the product binds it; every site above
  takes its address from them in the real tiers; the simulated tiers keep
  theirs.
Test shape: `smith-real-world`'s unit test
  `two_processes_never_share_a_loopback_address` (two ids, two addresses;
  the address within 127/8); the stories, unchanged.
Proof: process, `heavy --gate cargo nextest run -p smith -p
  smith-real-world -p smith-local-process-world --profile measure` three
  times in one gate task: every test passing each time; the three walls
  in the report (the audit: 2.6 s of wall for these stories, 61 tests);
  `smoke-tools` once before `land`.
Don't: change a story, a fake's behaviour, or what the product is told
  beyond its addresses.
Size: at most 60 lines of test kit.
Removes: the `real-local` group.

### 02.4 The longest tests start first
Scope: smith's `.config/nextest.toml` (its comment: the worlds' tests
  first, "so that the run ends close to its total over the threads").
Needs: 02.2, 02.3.
Files: `.config/nextest.toml`.
Shape: one override, `priority = 20`, for the focused tests that took
  more than 1 s in this increment's first run below, named in a filter,
  above the worlds' `priority = 10`.
Test shape: none.
Proof: process, `heavy --gate cargo nextest run --workspace --profile
  measure` once before the change (it names the tests) and once after:
  the two walls in the report; `smoke-tools` once before `land`.
Don't: change a test or a timeout.
Size: at most 8 lines.
Removes: nothing.

## Done when

- `grep -n NEXTEST_DOUBLE_SPAWN ~/src/rust/smith/scripts/check.sh` prints
  the export, and `grep -c real-local ~/src/rust/smith/.config/nextest.toml`
  prints 0.
- `heavy --gate cargo nextest run --workspace --profile measure` on
  smith's `main`, on carbon-forge: every test passes.
