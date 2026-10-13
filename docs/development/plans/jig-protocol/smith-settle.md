# jig-protocol: smith's settle test

From the designer's answer of 2026-10-13 to the supervisor's question
"smith's settle test: where `smith check --json` spends its time"
(decisions.md, ruling 186; the user chose its option (a)). The test is
smith's `tests/end-to-end/end_to_end.rs:487`,
`smith_check_json_lists_the_effective_limits_by_name`, which must settle
within 2 s of wall time (`tests/end-to-end/support/mod.rs:631`). Measured
on carbon-forge through `heavy --gate`, `--profile measure`, unoptimized
as smith's gate builds it; the phases timed by prints in a scratch
checkout, not landed. Two steps, 3 and 4; 3 first.

## 1. The test's time

On smith's `main` at `2baaa85c` (skein `7efc307`):

- **Alone:** 0.81–0.89 s; 0.65–0.82 s in the runs with timing prints.
- **In the full suite:** it fails, at 2.050 s (fail-fast stopped the run
  at 510 of 1,910 tests).
- **Its parts:** the real loop runs 0.63–0.81 s; the product exits
  0.17–0.29 s into it, and the loop runs 0.45–0.5 s more before it
  settles. Building the world and dropping it take under 1 ms.

## 2. Where it goes

### 2.1 The person reads the product's output a byte per iteration

About 450–900 ms, alone. smith's harness runs the product on pipes, and
its scripted person, `Terminal::piped`
(`tests/local-process/src/terminal.rs:159-170`), adopts the product's
standard output into io and reads it by demanding `Read::Fill(1)`
(:292-293), again after each byte. lib.md, 7's reads are exact, and io
meets a demand in the up pass after the down pass that made it (io.md,
2: "Open | `Demand` | stored; on the ready list"; skein
`crates/skein-io/src/pipe.rs:131-146` marks the pipe for the next pass,
which `reclaim` promotes, `layer.rs:282,363`). So each byte costs one
iteration of the real loop, with the product and the person in it:
`check --json` prints 24,940 bytes, every effective limit by name;
24,941 demands; about 25,000 iterations of 20–40 µs each. The loop
stays busy through the product's run and for 0.45–0.5 s after its
exit.

The demand is smith `1f3a9add` (2026-10-08, the local process world);
the harness's person on pipes, smith `1307cb56` (2026-10-12).

Ruling 186's 548 ms not named are, by the same count, about 22 µs more
per iteration, multiplied 25,000 times. They were not bisected: with
step 4 the loop runs 58–78 iterations, where a cost of that size adds
under 2 ms.

### 2.2 Trust roots decoded by searching the alphabet

About 190–210 ms of the product's 290–370 ms. The product resolves its
settings, and with them its trust roots
(`crates/smith-agent-shell/src/config.rs:885`,
`skein_shell::read_trust_roots`). skein-shell's `pem()`
(`crates/skein-shell/src/startup.rs:135-170`) feeds each line of
Debian's bundle (224,449 bytes, 150 certificates) to lib's
`base64::Decoder`. `Decoder::push`
(`crates/skein-lib/src/base64.rs:203-222`) walks each piece twice, on a
copy of its state to check it and then to decode it, and reserves
exactly for each piece; `State::push` (:95-145) finds a symbol's value
by searching the 64 symbols (:120-131). Unoptimized, about 850 ns a
byte. Parsing the certificates (`tls::Config::from_der`) takes about
1 ms.

The decoder is skein `a50eb2a` (this run's skein 02.1); before it,
`pem()` decoded a symbol by a range match, in one pass. Every smith
process that reads trust roots pays it, an agent's too.

### 2.3 The rest

No other cost reaches 50 ms: spawning the product, reading its settings
(1–2 ms), its report, the loop's teardown (under 1 ms). With both steps,
the product exits 31–50 ms into the loop, its trust roots read in
17–24 ms.

## 3. skein-02.s8: base64 decodes a symbol by its table, in one pass

A step not in the plan, in skein 02's session
(`skein/02-startup-and-secrets.md`).

Shape: `crates/skein-lib/src/base64.rs`: each alphabet's symbol values
  are a 256-entry table made once (a `const`), so `State::push` takes a
  symbol's value by index (:120-131 now search the 64 symbols);
  `Decoder::push` decodes a piece in one pass, refused and poisoned at
  the first refusal as now, without first checking it on a copy of the
  state (:207-213); it reserves for a piece's bytes with `reserve`, not
  `reserve_exact` (:215). `decode` keeps its check of the whole text
  before it allocates (lib.md, 6.1: "refuses past it before setting
  anything aside").
Test shape: base64's tests unchanged
  (`crates/skein-lib/src/tests/base64.rs`); every byte value, pushed
  alone into a new decoder of each alphabet, is taken if the alphabet
  has it and refused as `Symbol { at: 0 }` if not (`=` as padding, as
  now); a refusal partway through a piece leaves the decoder refused
  with that error.
Proof: skein's gate; smith's settle test, in step 4's Proof.
Don't: change what is refused, or an error's `at`; allocate before
  `decode`'s check; build skein-lib optimized (it ships).
Size: about 40 lines of production code.

## 4. smith-repin, again from smith's main

Ruling 186 kept smith-repin's commit `bb26b6d9` (skein `4130d9b`, on
smith `8cfcef03`). smith's `main` has since taken skein `550767e` and
`7efc307` on smith's own machine (`78a6b6ef`, `8ad70589`), and `7efc307`
is skein's last code commit (`cb3cefb` after it is documents). So
`bb26b6d9` is not landed: smith-repin starts again from smith's `main`,
after skein-02.s8, takes it, and makes smith's gate pass here.

Shape: smith's `Cargo.lock` takes skein's `main` with 02.s8 in, and no
  other source; `Terminal::piped`
  (`tests/local-process/src/terminal.rs:159-170`) reads the product's
  output as `attached` reads its terminal (`terminal_iterate`,
  :187-240): kernel `PipeRead`s of 4,096 bytes on the output's
  descriptor, each submitted when the last completes, until a read of 0;
  then it closes the descriptor, and `is_empty` waits for the close. The
  output is not adopted into io; the input stays io's.
Test shape: the end-to-end tests unchanged; one local-process test: a
  piped person on a pipe written whole with 60,000 bytes, then ended,
  shows all of them in at most 16 reads.
Proof: smith's gate (`docs/development/workflow.md`, 1) through
  `heavy --gate`, its `Gate-Result` naming the test's time in the full
  suite, under 0.4 s; alone, under 0.15 s; `smoke-tools` once before
  `land` (README, "Live services").
Don't: raise the 2 s bound; read the output by an io demand (an exact
  read waits for its count and a line read for its line's end, so a
  prompt with neither would never show); change `Terminal::new`'s pipes
  or `attached`.
Size: about 60 lines of test kit, and the lock.

The closing repins, jig-repin's last step and temper-repin, then take
smith's `main` as it stands, which carries smith-repin. If 02.s8 waits,
smith-repin lands the read alone on smith's `main` at `7efc307` (the
test then runs about 0.3 s alone), and the next repin takes 02.s8.

## 5. Measured with both steps

In a scratch checkout of smith `2baaa85c` on a scratch skein `7efc307`,
with both steps' code, not landed:

- **Alone:** 0.047, 0.048 and 0.074 s; the loop settles after 58–78
  iterations, the person having shown all 24,940 bytes.
- **In the full suite** (1,910 tests, 16.6 s): 0.147 s. 1,909 passed;
  the one failure, smith-bench's
  `a_replay_arm_runs_through_the_probe_command_and_prints_one_line`,
  refuses to build an arm from a source tree with changes in it, the
  scratch checkout's.
- **With step 4's read alone,** measured with an io line read in its
  place: 0.29–0.32 s alone. A line read is not the step: in the full
  suite, people waiting on output with no line end stalled (the local
  process referee's stories, and the real stories at 5 s).

## 6. Beyond the test

smith reads a tool's output the same way, `Fill(1)` from io
(`crates/smith-protocol-machine/src/process.rs:230,571`), and a host's
standard error (`crates/smith-host-protocol/src/process.rs:481`): by the
same mechanism, one iteration of its loop per byte (not measured). A
tool that prints 1 MB costs about a million iterations. It is skein's
stream vocabulary's to change, and goes to the user (decisions.md, "For
the user"); nothing in this run waits on it.
