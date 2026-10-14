# test-budgets: decisions

The decisions this plan rests on, numbered; the supervisor's rulings
during the run follow them, from 101; the choices left to the user close
the file.

## The plan's own

1. **Scope** (the user, 2026-10-13: "the jig-protocol plan has
   completed, with few later items, among which: Test-cost passes on
   every suite whose budget was doubled (jig's quick suite, smith's),
   then the budgets restored; skein's raw-reader plan, option (a) ...
   which I'd both like to tackle now"). The focused suites of jig, smith
   and skein (whose timeout ruling 249 doubled too, skein `886a7d4`) made
   cheaper and their 15 s timeouts restored; skein's read of what has
   arrived, taken by smith's and jig's readers. Not in it: the fuzzy
   suites, which fit their 60 s; the repository downstream of jig
   (decision 12); the live suites.
2. **The aim: at most 12 s in a gate on carbon-forge** (the user,
   2026-10-13, "≤ 12 s (80%)"), each focused suite, with its
   `global-timeout` back at 15 s. The margin is for the next plans' tests
   and for load outside the queue (one measure of jig's suite on this
   plan's base took 20.1 s at a load of 8.7, against 15.0 s in its last
   gate). Each repository's last increment measures its suite three times
   in the gate lane and restores the timeout when the median is at most
   12.0 s.
3. **Measured on carbon-forge,** the slowest machine (the user,
   2026-10-13, jig's `jig-protocol/decisions.md`, "For the user": "15 s
   focused and 60 s fuzzy hold on carbon-forge"). Every time in this plan
   is carbon-forge's, through `heavy`, unoptimized as the gate builds.
4. **Algorithms first** (the user, 2026-10-13: "exploring how to make
   test runs more efficient (e.g. detecting+converting O(N) code to O(1)
   and things like that)"). The planner's audit (`findings/cost-*.md`)
   found each increment's cause by measuring it; no increment drops an
   assertion or a behaviour a test checks, and where production code
   changes, its behaviour stays the same. Where a test is made smaller,
   its session says why its assertion holds at the smaller size.
5. **The one-process fallback runs only if needed** (the user,
   2026-10-13: "Conditional last step"; earlier, "not now; only when it is
   needed to keep the suite within its budget"). If, after a repository's
   cost sessions, its timeout increment (skein 02.5, smith 04.1, jig
   06.1) measures a median past 12.0 s, it lands nothing and reports;
   the supervisor then asks the designer (development.md, 10.4) to change
   testing-strategy.md, 8 so that each crate's unit tests run in one
   process (libtest's threads) and only the worlds' tests one process
   each, and adds the step `<repository>-tests.one-process` that makes
   the change in that repository's gate. Measured on this plan's base:
   `jig-protocol-store`'s 417 unit tests run in 0.28 s in one process
   against 2.7 s as 417 nextest processes, alone.
6. **nextest without its double spawn** (`findings/cost-jig-units.md`,
   1; the designer, in this plan's design landing, testing-strategy.md,
   8). nextest starts each test through a second process of its own,
   which guards only a terminal's job control; turned off, a test's
   process costs about half. Measured on this plan's base, paired in the
   gate lane: 992 of jig's unit tests in 1.117 s and 0.951 s with it,
   0.493 s and 0.501 s without (`findings/measurements.md`, 3). Each
   repository's `scripts/check.sh` and skein's `heavy` set
   `NEXTEST_DOUBLE_SPAWN=0`; until skein 02.1 lands, each Proof's
   measuring command sets it in front of `heavy` (README, "Rules"), so
   every increment is measured as the gate will run it.
7. **The read of what has arrived** (the user, 2026-10-13: option (a)
   of "A raw reader over io takes a byte per iteration", jig's
   `jig-protocol/decisions.md`, "For the user"). Designed in skein's
   `lib.md`, 7, `io.md`, 2 and 3.3, `http.md`, 3.3, 4 and 6, `tls.md`,
   3.1, `programming-model.md`, 4.3 and `testing-strategy.md`, 2.4, landed
   with this plan. `Read::Arrived { max }`: met as soon as the side below
   holds a byte, by all it holds then, up to `max`.
8. **Which readers move** (`findings/reads.md`, 4): every reader of
   bytes nothing frames whose step already takes a whole delivery: in
   skein, the LLM client's error body, the OAuth token response, the
   fake peers' bodies, the tests' bodies; in smith, a tool's and git's
   output, a host's standard error, the headless prompt, the scripted
   person; in jig, the client link's event data, the native shell's
   bodies of unknown length, the engine's chunked request bodies. Exact
   reads stay where the bytes are framed or the step takes one event per
   byte: the JSON tokenizer, the HTTP body near a delimiter, the channel's
   header, an event's first data byte, the terminal's keys.
9. **smith's readers are proven by tests, not a benchmark probe**: what
   a model sees does not change, only the loop's iterations, which smith
   01's stories count. smith's `workflow.md`, 5 asks a probe of "an
   improvement (prompt caching, budgets, deadlines, tool round trips,
   limits sized for real work)"; a read's cost is none of those, and no
   probe measures a loop's iterations. Every smith landing that changes
   code still runs `smoke-tools` once. Listed for the user's veto.
10. **smith's gate here** (jig's `jig-protocol/decisions.md`, ruling 121):
    smith lands on carbon-forge without the focused suite's timeout, with
    `land --gate 'cargo fmt --all --check && cargo clippy --workspace
    --all-targets -- -D warnings && NEXTEST_DOUBLE_SPAWN=0 cargo nextest
    run --workspace --profile measure && NEXTEST_DOUBLE_SPAWN=0 cargo
    nextest run --workspace --profile fuzzy'`, until smith 04.1 restores
    its timeout; from then smith lands here with `scripts/check.sh`.
11. **smith's reliability plan lands in smith and skein meanwhile,** from
    debian-16gb-hel1-1. This plan's smith sessions rebase on it through
    `land`; neither waits for the other. Its sessions add tests to the
    suite this plan makes cheaper, measured on a machine about 1.7 to 2
    times faster (smith's focused suite: 9.2 to 9.8 s in that machine's
    gates; here 16.5 s in the `measure` profile on smith `6449a197`, 21.0
    s on `0305f2fd`, `findings/measurements.md`, 1).
12. **Nothing downstream of jig is repinned** (jig's
    `jig-protocol/decisions.md`, rulings 281 and 282): the one repository
    built on jig does not build on jig's `main`, and its reconciliation
    is a plan of its own. Its repin appears in no file of this plan
    (development.md, 5.7, "Names").
13. **The codec generator's changes reach smith and jig by regenerating**
    (as jig's `jig-protocol/tests-cost.md`, steps 3 and 9 did): a family's
    worst-case functions check its limits once, and its golden tables hold
    up to 1,024 cases; smith 01.1 and jig 05.1 take skein's codegen and
    regenerate every family, golden `.bin` files unchanged.

## Rulings

(The supervisor's, from 101.)

101. **skein 02.2 regenerates all four of `tests/codec`'s families**
     (plan defect, skein 02's block of 2026-10-13T23:15:24Z). 02.2's Files
     named only `tests/codec/src/generated/`, but `tests/codec/tests/drift.rs`
     checks four schemas against their generated code (`src/secrets/v1.rs`,
     `src/generated/v1.rs`, `src/scalars/v1.rs`, `src/lints/v1.rs`), so the
     generator's change fails three drift tests unless all four regenerate.
     They regenerate, once, with the drift test's own variable
     (`SKEIN_CODEGEN_WRITE=1 cargo nextest run -p skein-codec-tests --test
     drift`, through `heavy`); nothing in them is edited by hand, and the
     golden `.bin` files stay unchanged. 02.2's Files now names the four
     families. Generated code counts toward neither the Size nor the
     20-line allowance outside Files (development.md, 8.2).
102. **A landed commit whose trailers git cannot read stays as it
     landed** (process, jig 02's block of 2026-10-13T23:24:06Z). jig
     `2497c8299`'s last body paragraph mixed `Proof log:` and `Count
     test:` lines among `Proof:`, `Plan:` and `Step:`, so git reads none of
     them as trailers; skein `375c688` put a blank line between `Proof:`
     and `Plan:`, so git reads its step but not its Proof. Each is accepted by its ancestry on `origin/main`, its
     `Gate-Result` and its body; neither is rewritten (development.md, 8.4:
     "A commit that landed without its body is never rewritten"), and no
     commit is added to carry their trailers. The ledger records them
     `landed`, with a note naming this ruling; `plan status` lists jig-02.1
     as landed in the ledger but not on main, which is this ruling, not a
     fault. The same defect twice became a standing rule in the README
     ("A commit's last paragraph is its trailers").
103. **smith 03.1 keeps the run world's trace as it is** (plan defect,
     smith 03's block of 2026-10-13T23:45:15Z). 03.1's Shape asked the run
     world's trace to keep typed entries, formatted when read; but
     `run::Event` and `run::Request` have no `Clone`, their `ReplyTo`
     fields are single-use rights, and the trace is read whole by replays
     (`tests/run/tests/fuzzy_messages.rs:22`, `simulation.rs:988`) and by
     `Debug` text (`simulation.rs:564`, "Checking {"). Keeping it typed
     needs either a new kit mechanism upstream in skein or a mirrored enum
     of every variant, both past the increment's 30 lines and outside its
     Files; the audit put the whole saving at "about 1 s" summed
     (`findings/cost-smith.md`, 9, estimated), about 0.1 s of wall. 03.1
     now changes only the agent world's log line
     (`tests/agent/src/world.rs:1004`); the run world's trace goes to
     `later.md`. No kit mechanism is added.
104. **A landed report's worktree may hold the next increment**
     (process, skein 02's block of 2026-10-13T23:54:49Z). After skein
     `c57966cd2` (skein-02.2) landed, its implementer went on to 02.3 on
     `test-budgets/skein-02-3`, as development.md, 8.6 says ("After a
     landing, send the report and go on"), and the coordinator's
     `git status --porcelain` (7.3) printed 02.3's fourteen files.
     skein-02.2 is accepted: `merge-base --is-ancestor` prints `on-main`
     and its `Step` trailer reads `skein-02.2`. The README gains "The
     coordinator's check of a landed report": the cleanliness check runs
     only while the worktree is on the landed step's branch.
105. **smith 03.1's transcript is named by its length and a bounded
     fingerprint** (plan defect, smith 03's block of
     2026-10-13T23:58:15Z). 03.1's Shape said "its length and digest";
     hashing the whole 33,554,433-byte transcript in the unoptimized
     test build still took 0.232 s against the test's 0.1 s ceiling
     (`logs/20261013T235436-155.log`), past twice it (development.md,
     8.2), and the world has no faster hasher. The line now holds the
     exact length and a hash of at most the first 512 and last 512
     bytes, labelled a fingerprint; payloads of at most 1 KiB are
     formatted as before. Two runs that differ only inside a large
     payload's middle are no longer told apart by the trace; the
     transcript's content is fixed by its story. No dependency is added.
106. **skein 01.3's worst-case tests move by one delivery** (plan
     defect, skein 01's block of 2026-10-14T00:04:36Z). 01.3's Shape
     asked the LLM client's and the token exchange's worst cases to
     "gain one delivery of their reader's `max` each", but named only
     their memory tests; `tests/llm/tests/replay_total.rs` asserts the
     client's worst case as a literal, outside Files. Its `fixed` moves
     53,113,764 to 53,130,149 (OpenAI Codex) and 53,112,204 to 53,128,589
     (Anthropic), and `removed` 53,501,856 to 53,485,471 and 53,501,040
     to 53,484,655: each by 16,385, `error_bytes` (16,384) + 1, the error
     body's `max`. These are the sizes code-organization.md, 4 allows a
     session file to name; 01.3's Shape now names them, and any such
     test under `tests/oauth-accounts/`, by the token response's `max`.
107. **skein 01.3's read-shape expectations change with its Shape**
     (plan defect, skein 01's block of 2026-10-14T00:24:21Z; it narrows
     ruling 106). Ruling 106's unblock said "Any other expected value
     that changes is a question", but 01.3's Shape itself turns
     `body_read`'s unknown-length `Fill(1)` into `Read::Arrived { max }`
     (the room, or 1 where it is 0), and
     `length_framed_body_demands_bounded_batches_and_probes_actual_end`'s
     `(None, 256, 1)` row expects that `Fill(1)`. The row now expects
     `Read::Arrived { max: 256 }`, every known-length row is unchanged,
     and a zero-room row expecting `Read::Arrived { max: 1 }` is added.
     01.3's Shape names it.
108. **smith's window story is retried once, until it is fixed**
     (cross-session scope, smith 03's block of 2026-10-14T00:24:49Z and
     smith 02's of 00:25:26Z). `a_window_size_change_reaches_the_status_line`
     saw `[85, 20]` against `[80, 20]` in two smith gates in a row
     (`logs/20261014T002211-199.log`, `logs/20261014T002321-200.log`,
     each started at a load of 11 to 13 on 8 cores), after passing in
     the 12 runs logged before them, smith 02.1's six among them. smith
     02.1's fix (the terminal sized before the product starts) did not
     remove the race, and no code changed between the passing and the
     failing runs. Until a step not in the plan fixes it, a smith gate that
     fails only on this test with these widths lands again once without
     asking (README, "smith's window story may be landed again once").
     That step follows once its cause is found.
109. **smith-02.s1 fixes the window story's echo race** (step not in
     the plan, after ruling 108). A read-only look at the code: the
     product reads its first window size synchronously long before its
     first status line, so the line is clipped at 80; but the person
     sends `Again\n` as soon as the line's prefix is shown
     (`tests/end-to-end/end_to_end.rs:43-44`), and the pty echoes input
     (cooked mode, smith's `shell.md`, 308-310), so `Again` can land
     before the line's newline. A clipped line is always 80 long, and its
     unclipped length varies with elapsed time, yet both failures show
     exactly 85 = 80 + `Again`. The fix is the test's: the person waits
     for the line's end (`smith/02.s1-window-echo.md`). No product
     change. Ruling 108's retry ends when 02.s1 lands: it landed as smith
     `15a37563`, and the README's rule was removed.
110. **skein-02.s1 moves a golden table's cases out of its test
     function** (step not in the plan; smith 01's block of
     2026-10-14T00:39:27Z, cross-session scope). skein 02.2's golden
     tables of up to 1,024 cases put one line per case inside the test
     function, so smith's regenerated `golden_0` is 233 and 445 lines,
     past clippy's `too_many_lines` (100); smith 01.1 may neither edit
     generated code nor add a lint exception (code-organization.md, 4).
     The generator emits each chunk's cases as a module item and its test
     as one call (`skein/02.s1-golden-items.md`), keeping 1,024 per
     golden table. smith 01.1 and jig 05.1 take it before regenerating.
111. **skein-02.s2 makes the table items `static`** (step not in the
     plan; jig 05's block of 2026-10-14T01:10:27Z, cross-session scope).
     skein-02.s1 emitted `const` items, as its Shape said; ruling 110's
     "if clippy objects ... emit a `static`" could not fire in skein,
     whose tables are under 100 cases, and jig's tables of 512 to 1,024
     cases fail `clippy::large_const_arrays`. Every table item becomes a
     `static`, borrowed by its test (`skein/02.s1-golden-items.md`,
     02.s2). jig 05.1 waits for it; smith 01.1 takes it if its own clippy
     fails the same way. A conditional fallback in a step's Shape is
     written as the rule instead, when the step's own gate cannot reach
     the condition.
112. **smith 01.3 changes one story's cue** (cross-session scope,
     smith 01's block of 2026-10-14T01:16:31Z). With the person reading
     what has arrived, `slash_new_starts_a_chat_and_slash_resume_reopens_it`
     (seed 55) shows the new chat's `tool finish` before its "Waiting for
     a message" notice, so its cue `First answerWaiting for a
     message\n\ntool finish` (`tests/local-process/tests/simulation.rs:289`)
     never appears and the story waits out its 60 s
     (`logs/smith-01/01.3-proof-failed.log`). The cue tested the order of
     two output pieces, which no assertion asks for. It becomes the
     shortest text shown only once the new chat has answered, whatever
     that order; the assertions stay. The person's scheduling is not bent
     to keep the old order (smith/01-arrived.md, 01.3, jig `669f3ff80`).
113. **smith 03.4 primes the cached configuration outside the heap
     meter** (plan defect, smith 03's block of 2026-10-14T01:17:27Z).
     03.4's Shape computes `lower_configuration` once per process
     (`OnceLock`); `fuzzy_store`'s first metered process initialized it,
     and its heap check counted the 3,528 retained bytes against the
     process (seed 80, `logs/20261014T011543-87.log`). `configured`
     (`tests/local-process/tests/fuzzy_store.rs:206`) obtains it once
     before the world's metered processes start; the focused worlds
     already do. Every name, expected value and heap check stays.
114. **jig 05.1 copies smith's full-turn golden again** (plan defect,
     jig 05's block of 2026-10-14T01:35:46Z). jig keeps a copy of
     smith's transcript golden `record_turn_full.bin` (127 bytes at jig's
     pin); smith `47e99f6e` ("Record activation models and effort")
     grew it to 142, the `model` and `effort` 05.1 already names, and
     jig's gate failed on the old copy
     (`logs/20261014T013335-88.log`). It is copied again, byte for byte,
     from smith's revision in jig's lock; the three sizes that name it
     move from 127 to 142, as smith's own change moved its golden
     (development.md, 5.7: tests "adapted the way the upstream commit
     adapted its own"). No byte is edited by hand.
115. **jig 05.1 refreshes both copied goldens and every literal derived
     from their sizes** (plan defect, jig 05's block of
     2026-10-14T01:44:13Z; it widens ruling 114). Ruling 114 named the
     three literals equal to 127, and missed the bounds one below and one
     above the fixture (`turns.rs:57`, 126; `:60`, 128) and the smallest
     golden, 33 bytes in jig and 38 in smith. jig's own
     `crates/jig-protocol-transcript/tests/data/README.md` says "When jig
     repins smith, refresh changed fixtures and their expectations
     together". Both goldens are copied again with that README's revision
     line, and the six literals move by their fixture's growth (+15, +5).
     The same holds for `jig-repin`.
116. **A measurement that failed on a dirty tree is not a measurement**
     (process, smith 04's block of 2026-10-14T01:47:03Z). smith 04.1's
     first run stopped after 14 of 1,995 tests: smith-bench's
     `a_replay_arm_runs_through_the_probe_command_and_prints_one_line`
     refuses a dirty source ("source repository is dirty"), and the
     workflow edit was not yet committed
     (`logs/smith-04/proof-1-failed.log`). It is discarded; 04.1's three
     runs are made on the committed, clean branch, as its Proof says, and
     are not extra measurements. Every timing run of a smith suite is
     made from a clean worktree (`git status --porcelain` printing
     nothing), as steer 2026-10-14T00:18:42Z asked of `smoke-tools`.
117. **jig 05.1 refreshes every copy of smith's turn goldens, found or
     to be found** (plan defect, jig 05's block of
     2026-10-14T01:50:37Z; the third block on it, after rulings 114 and
     115). Besides the two `.bin` copies, jig holds the full golden as a
     byte literal, `FULL_TURN` (`testing/jig-test-client-records/src/activity_reads.rs:4`),
     and a bound one below its size (`crates/jig-protocol-client-link/src/tests/pages.rs:230`,
     126); three `jig-protocol-client-link` page tests decode the old
     bytes as `Malformed` (`logs/20261014T014712-86.log`). Both are
     refreshed, the literal by a script from the golden's bytes. To end
     the round trips, any further stale copy of these two goldens, or
     literal equal to a size or one either side of it, is refreshed the
     same way without asking and listed in the commit body. Rulings 114
     and 115 listed literals found by searching for the value 127; a
     ruling on a fixture names its copies and its derived bounds by
     searching for its name and its users.
118. **jig 05.1 refreshes the charter golden's copy; the inventory is
     complete** (plan defect, jig 05's block of 2026-10-14T01:57:28Z).
     `SMALLEST` (`testing/jig-test-worker/src/tests/fixtures.rs:62`) is
     smith's `crates/smith-charter/golden/v2/record_charter_smallest.bin`,
     118 bytes at jig's pin (smith `dfc0fe57`), 119 on smith's `main`;
     three `jig-test-worker` tests fail to decode it
     (`logs/20261014T015445-86.log`). It is rewritten by a script, length
     119. A search of every non-generated `.rs` file on jig's `main` for
     each of smith's 720 goldens at `dfc0fe57` written as a byte array,
     and of every `.bin` outside a `golden/` directory, finds four copies
     in all: the two transcript `.bin` files, `FULL_TURN` and `SMALLEST`.
     `jig-repin` refreshes those four as rulings 114 to 118 say.
119. **jig-repin moves the one-process world's cuts back to mid-run**
     (plan defect, jig-repin's block of 2026-10-14T02:20:27Z). Taking
     smith `29fa97c9`, three one-process tests (`tests/one-process/tests/one_process.rs`)
     see the run finish before their cut: the power loss is placed by a
     count of effects (`crash(Cut::PowerLoss, 130)`, `world.cut(0, before,
     cut)`, `tests/one-process/src/world.rs:282`), the signal once a `Turn`
     row exists (`tests/one-process/src/referee.rs:272-281`). A run that
     takes fewer deliveries, this plan's aim, moves where such cuts fall.
     The repin moves each cut so that it lands mid-run again, by an event
     where it can and by a recounted number where it must, as development.md,
     5.7 allows ("the downstream tests that an upstream change of behaviour
     breaks"); every assertion and expected outcome stays. If the cause is
     a change of what smith does rather than of how many steps it takes,
     or a cut cannot land mid-run without changing an assertion, the repin
     stops and asks.
120. **The one-process fixture keeps its worker alive past a committed
     turn** (jig-repin's block of 2026-10-14T02:32:50Z, filed as a design
     gap; ruled a fixture defect). The traced run
     (`logs/jig-repin/traced-run.log`): the worker child spawns at service
     submission 93, its output ends at 111 and 113, its `Turn` is durable
     at 132 and `Finished` at 136. Reading what has arrived, the child is
     done before anything of its run is committed, so no cut finds a
     committed run with a live worker, which these three stories need;
     at 100 nothing is claimed yet, at 130 the worker has exited. The
     fixture (`tests/one-process/src/fixtures.rs`, `provider()` and what
     it scripts) makes the worker outlive its first committed `Turn`, by
     the smallest change the fake peers already allow (a second turn, or
     a reply that waits), and the cuts are placed inside that window; every
     assertion stays. What the recovery records after a power loss
     between a worker's clean exit and its `Finished` commit (`Refused`
     at 130) is a different state from these stories', and goes to
     `later.md`, unchecked against jig's design.
121. **jig-repin traces the fault topology once more** (process,
     jig-repin's block of 2026-10-14T02:38:02Z). With ruling 120's fixture
     (`leading_turns=1`), both signal stories pass; the power-loss cut was
     placed from a trace of the answer topology (window 145 to 154, cut
     148), but the fault topology's service finishes at 143, so the cut
     never fired. One trace of the fault topology places it; the three
     stories run once more. Nothing else changes.

## For the user

- **No benchmark probe for smith's readers** (decision 9): smith's
  workflow asks a probe of an improvement; the planner reads a loop's
  iterations as outside what a probe measures, and proves smith 01 by
  stories that count deliveries. Veto, and smith 01 gains a probe (which
  `benchmarks.md` would first have to design).

- **smith's suite here and the reliability run there** (decision 11):
  after smith 04.1 restores smith's 15 s timeout on carbon-forge, the
  reliability run's landings, gated on debian-16gb-hel1-1, can carry the
  suite past it here unseen. Options: (a) the reliability run's
  supervisor holds smith's focused suite to about 7 s on its machine
  (12 s here over the 1.7 ratio), as a rule of that plan; (b) a gate
  that bites on carbon-forge is worked on when it bites, as ruling 249
  did; (c) budgets per machine. The planner recommends (a). Nothing in
  this run waits on it.
