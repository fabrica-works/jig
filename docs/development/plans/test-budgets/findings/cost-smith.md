# smith's focused suite: where the time goes, and why it fails here

The planner's sub-agent's audit, 2026-10-13, at smith `0305f2fd`
(`origin/main`), measured through `heavy --gate` with `--profile
measure` at 8 threads unless it says otherwise. Tried changes:
`tried/smith-parallel-and-uid.md` and `tried/smith-skein-base64.md`
(evidence, not to be applied as is). smith 02 takes A to C and the
failures; smith 03 takes findings 2 to 6 and 8; skein 02.4 takes 7.

## Headline

| Run | Wall | Summed | Failures |
|---|---|---|---|
| Base, `0305f2fd` (`20261013T210114-88.log`) | 21.0 s | 147.7 s | 7 |
| With A to C (`20261013T214220-141.log`) | 16.7 s | 120.1 s | 1 flaky (section 4) |

- **A.** smith on skein's `main`: `Cargo.lock` only; smith builds against
  skein `2068257` with no source change.
- **B.** The one-line fix of the user's id (section 2).
- **C.** Each real-tier process's fakes on loopback addresses of its own,
  and the `real-local` test group removed (section 3).

The summed figures leave out `smith-bench::probe_command`'s
`a_replay_arm_runs_through_the_probe_command_and_prints_one_line`,
which refuses a source tree with changes in it ("source repository is
dirty").

## 1. smith's skein pin went backwards

skein-lib's base64 decoder before skein `8a34235` (jig-protocol's
skein-02.s8) finds each symbol by scanning the 64-symbol alphabet and
walks each byte twice (`base64.rs:125`, `Decoder::push` `:203-222`):
reading the system's 224 KB trust bundle costs 245 to 253 ms a load
(rustls's `from_der` on the 150 roots, 1.5 ms). smith `6449a197` took
`8a34235`; the reliability plan's smith `83122ea0` (smith-10.10, "Repin
Skein before the raw terminal consumer") moved the pin back to
`cb3cefb`, which its message calls "current upstream main", an ancestor
of `8a34235`. Every machine-trust load pays it: `end_to_end`'s checks (9
loads, about 2.2 s of its 7.6 s alone), smith-bench's settings (7 loads)
and `smith_adapter` (1 to 4 a test), local-shell's compose,
`live_configuration` and agent-shell. Taking skein's `main` saves 27.6 s
summed and 4.3 s of wall, measured. On the reliability machine, where the
suite takes 9.7 s, it does not show.

## 2. The seven failures here

**Six in `smith-local-process-world` (`authentication` ×4, `memory`
×2): the user's id.** `tests/local-process/src/world.rs:603` builds the
simulated disk with `Filesystem::new()`, whose files belong to uid 1000
(skein-fake-machine `fs.rs:285-286`), while
`tests/local-process/src/process/configuration.rs:347` passes the real
`skein_shell::effective_user()`; skein-io's private files
(`file_layer/private.rs:72-73`, through
`smith-local-service/src/credentials.rs:221`) then refuse the token store
as `Owner { found: 1000, expected: 600 }`, so "account 0 has no
credential" and the world never settles. carbon-forge's user is uid
600; the reliability machine's is evidently 1000. Fix:
`Filesystem::for_user(skein_shell::effective_user())` at `world.rs:603`;
the six pass, three times (`20261013T213327-117.log`) and in two full
runs.

**`smith::end_to_end`'s `smith_on_a_terminal_signs_in_and_commits_in_place`:
a margin, not determinism.** It passes alone (0.377 s); at 8 threads it
takes 1.07 to 1.10 s and meets the 1 s deadline of the `Refusal` referee
(`tests/end-to-end/support/exec.rs:216`), meant for refusals at
startup, which `chat_signed_in` uses for a sign-in and a six-tool
change. Fix: that story takes the 5 s deadline `run_with_provider`
uses (`support/mod.rs:325`).

## 3. The wall-time floor: the real tier runs one test at a time

smith's `.config/nextest.toml` runs `package(smith-real-world) or
binary(end_to_end)` in a group of one thread: 55 tests, about 15.8 s at
8 threads, longer than the rest of the suite once that is cheaper. What
the stories share is three fixed TCP ports on 127.0.0.1:

- **34443, the provider over TLS:** `tests/agent-process/src/fake.rs:121`;
  in settings at `tests/end-to-end/support/mod.rs:269`, `exec.rs:31`,
  `tests/local-process/src/process/configuration.rs:292`.
- **34444, the issuer:** `tests/local-process/src/process.rs:106,127`,
  `exec.rs:240`, `mod.rs:349`, `login.rs:99`; in settings at
  `mod.rs:276-277`, `end_to_end.rs:207-208,233,906`,
  `configuration.rs:328-352`.
- **2345, the product's sign-in redirect listener:** `mod.rs:277`,
  `end_to_end.rs:208,906`, `process.rs:135`, `configuration.rs:337`.

Nothing else is shared: scratch directories (`local-process/src/world.rs:56`)
and `root.der` (`tests/real/src/lib.rs:98`) are per process, and
`assert_no_children` reads `/proc/self` (`settled.rs:208`).

Tried (about 50 lines in 8 files and `nextest.toml`): each real-tier
process binds its fakes on `127.1.0.0` plus its process id (below 2^22
here, so live processes never share one; every 127/8 address is local;
the product takes any loopback issuer, `sign_in.rs:110`, and any
endpoint). The redirect must be `127.0.0.1:<port>` (skein-oauth
`machine.rs:642-650`), so each process takes a free port from `bind(0)`
once: a small window between releasing the port and the product binding
it. `Scratch::new` turns this on for the real tiers; simulated tiers keep
127.0.0.1. End-to-end, real-world and the six (61 tests) then run in
2.6 s of wall, three times, with no collision; the full suite passes.

What a 155 ms story costs alone (`20261013T211703-89.log`): `Scratch::new`
10 ms (five git processes, `real/src/lib.rs:99-103`); the product ready
in 20 ms; the agent child's spawn, start, TLS handshake and first request
48 ms; the turn and exit about 16 ms; the test's own checks 50 ms
(`assert_binary_expected` 22 ms, resolving the settings again,
`settled.rs:68-74`; `read_trace` 28 ms, resolving them a third time,
`mod.rs:388-392`, reading 12.5 KB, 8 ms, and checking conformance, 5
ms).

## 4. The rest, ranked by summed gate seconds per line

1. **smith on skein's `main`** (section 1): 27.6 s summed, 4.3 s wall.
2. **agent-world's messages world formats a 33,554,433-byte text with
   `{:?}`** (`tests/agent/src/world.rs:1004`) for a log line: leaving
   the transcript out of the line, 1.25 s to 0.02 s alone, about 3.2 s
   summed (8 lines).
3. **smith-protocol-llm's `random_outcomes_at_random_caps_fit_their_derived_render_bound`**
   (`tests.rs:74-140`) renders one Read outcome 256 times (about 0.9
   million `Decimal::of`): checking it once at the smallest drawn bound,
   1.74 s to 0.27 s, about 3.8 s summed.
4. **`crates/smith-protocol-llm/src/framing.rs:16-24` parses `{}` once
   per tool,** about half of every `profile::derive`, which runs at each
   product start: parsing once and cloning (9 lines), about 2.6 s summed,
   the product's start 18.4 ms to 14.3 ms.
5. **Focused sweeps settle more worlds than cover every state:**
   `tests/run/tests/simulation.rs:655-819` (315 and 117 worlds; 7 and 5
   cover every state), `tests/session/tests/simulation.rs:516-538` (200;
   2 do): keeping the covering seeds focused, the sweeps in the fuzzy
   suite, about 2.8 s summed (10 lines).
6. **The process worlds derive the same values again:**
   `tests/agent-process/src/lib.rs:150` `limits()` per record and per line
   (`world.rs:1208-1219`, `events.rs:157`), `specialize`
   (`configuration.rs:19`), `lower_configuration()` at many sites, a
   verdict formatted at `referee.rs:668`: each once per process, about
   2.1 s summed (25 lines).
7. **skein-lib's search for a one-byte needle** (`bytes.rs:40-91`, used
   by `smith-domain-tools/src/window.rs:89-102`): the two-way search is 23
   times slower than a plain loop on one byte; a fast path, about 2.4 s
   summed (estimated). skein's.
8. **smith-agent-service's tests:** `sinks.rs:1316,1432` record 10,000
   events where 64 show the behaviour; `calls.rs:489-492` derives once
   per case: about 1.4 s summed (15 lines).
9. **Smaller, estimated:** tools-world's memory fill (`tests/tools/tests/memory.rs:116`,
   `src/memory.rs:85`), about 1.6 s, if 64 kits are not deliberate;
   skein-json's escaped writes (`writer.rs:571-585`), about 1.3 s,
   skein's; run-world's `Debug` traces (`tests/run/src/world.rs:897,1009`),
   about 1 s; end-to-end's `Scratch` with three git processes instead of
   five, and none for about 24 refusal and check stories, about 0.7 s;
   end-to-end's settings resolved once a story, about 0.3 s; rows, about
   0.5 s.

Projection: about 120 s less about 23 s, about 97 s summed: 12 to 13 s
of wall, with section 3's parallel real tier, and the longest unit tests
started first (agent-shell's dense test, about 4 s at 8 threads, now
forms the tail).

## Looked at, not worth changing

- **`Read::Fill(1)` in smith's tests:** no test pays measurably: the
  simulated terminal reads 40 to 470 bytes, and end-to-end stories take
  163 to 324 iterations each, their tools' output under 1 KB. smith 01's
  reason is production's, not the suite's.
- agent-shell's dense test (about 1.8 s alone after A; it writes a line
  over 12 MB by design), root memory, nesting, smith-bench's runner
  timeouts (real timers by design), `forget_oldest`'s scan (an index would
  change the memory bound).

## Surprises

- **A flaky test:** `smith::end_to_end`'s
  `a_window_size_change_reaches_the_status_line` failed in 3 of 6 runs at
  8 threads, once with widths `[85, 20]` against `[80, 20]`. Likely, not
  verified: the product starts unclipped
  (`smith-local-shell/src/front/interactive.rs:78`, `u16::MAX`), and the
  test's first resize (`window.rs:84-85`) happens inside `world.run`
  (`exec.rs:246`), after the binary was spawned (`exec.rs:188`), so the
  first status line can print at full width. Fix: size the terminal
  before the product starts.
- **A profile setting smith lacks:** skein's `Cargo.toml` (lines 105-111)
  sets `[profile.dev.package.rustls] debug-assertions = false` and says a
  workspace using skein-tls should copy it; smith's does not.
