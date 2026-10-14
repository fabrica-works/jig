# Session 01: smith reads what has arrived

smith takes skein 01's `Read::Arrived` and moves its readers of bytes
nothing frames onto it: a tool's output and standard error, git's, a
host's standard error, the headless prompt on standard input, and the
scripted person of the local process world. Each demands `Fill(1)`
today and pays one iteration of its loop per byte: a tool that prints a
megabyte costs about a million (jig's `jig-protocol/smith-settle.md`,
6). The terminal's keys stay exact. Three increments, the first with
the repin.

## Read first

- skein's `lib.md`, section 7 ("A read of what has arrived"; "What a
  delivery holds is how the bytes arrived"; "Exact reads stay ...").
- skein's `io.md`, 3.3 ("A read of what has arrived is met by the first
  delivered that finds a byte in the intake").
- smith's `docs/development/workflow.md`, section 5 (`smoke-tools` before
  every landing that changes code).
- `findings/reads.md`, section 4 (smith's readers, with `file:line`, at
  smith `6449a197`; the lines below are smith `0305f2fd`'s).

## What the design asks

- "It is the read of bytes nothing frames: a child's output, a body read
  to its end. It waits for no more than one byte, so a reader of such a
  stream pays one delivery for what arrived together, not one per byte"
  (skein's lib.md, 7).
- "A side above that reads what has arrived makes the same of the bytes
  whatever the cuts; only their order is the stream's" (lib.md, 7).
- "The side above's memory counts a delivery of its maximum" (lib.md, 7).
- "Exact reads stay ... where the side above must not take more than it
  can act on in one step: ... a terminal's keys when each may be an
  event of its own" (lib.md, 7).

## Choices this plan makes

- **Each reader names its own maximum, 4,096 bytes,** io's `receive` in
  the shipped profile (`crates/smith-agent-service/src/profile.rs:424`):
  a larger delivery would never come from one receive. One `const` per
  crate, beside its reader, so no crate gains a dependency:
  `OUTPUT_READ` in `smith-protocol-machine` (`process.rs`),
  `STDERR_READ` in `smith-host-protocol` (`process.rs`), `GIT_READ` in
  `smith-local-protocol` (`git_child.rs`), `PROMPT_READ` in
  `smith-local-shell` (`front/headless/input.rs`), and `OUTPUT_READ` in
  the local-process world's person (`tests/local-process/src/terminal.rs`).
- **A delivery's memory:** each machine whose reader moves holds a box
  of up to its maximum for the step that reads it. Where the machine's
  worst case is computed in code (its `worst_case` function, or the
  memory test's bound), the increment adds that maximum to it once per
  stream it reads, citing lib.md, 7. smith's `protocol/limits.md`
  derives those bounds: if a memory test fails for want of the term in a
  derivation `limits.md` names, it is a question (below).
- **The terminal's keys stay `Fill(1)`:**
  `crates/smith-local-service/src/process.rs:264,291,860` feed each byte
  to the terminal, whose step may push one event per byte and reserves
  room for one (`terminal_max_out`); a delivery of two lines would
  overflow it.
- **The scripted person's piped output goes back to io:**
  `Terminal::piped` (`tests/local-process/src/terminal.rs:159-170`)
  adopts its output pipe into io and reads it with `Arrived`, as its
  spawned child's pipes are read (`read`, `:320-322`); the raw `PipeRead`
  loop (`PIPED_OUTPUT`, `piped_complete` `:172-195`) goes. jig's
  `smith-settle.md`, 4 kept it out of io only because "an exact read
  waits for its count and a line read for its line's end".
- **The machine world's flood fixture answers a demand as lib's intake
  would:** `World::flood`'s child output (`tests/machine/src/lib.rs:542-600`)
  hands the machine one byte per delivery whatever it demanded, and
  `flood` refuses more than 4,096 bytes (`:214`). It takes the read the
  machine demanded below and delivers `min(left, max)` bytes for
  `Arrived { max }` and `n` for `Fill(n)`; its bound becomes a named
  `FLOOD_MOST: usize = 1 << 20`; it counts the deliveries it made, for
  the story below.
- **No benchmark probe** (decisions.md, 9, and "For the user"): what a
  model sees does not change; the change is the loop's cost, which the
  stories below count.

## Rules that bind it

- "**smith's reliability plan lands in smith and skein from
  debian-16gb-hel1-1** ... a conflict in a file both changed is resolved
  by keeping both changes, and one that cannot be is a question"
  (README, "Rules").
- No increment "adds a `_ =>` arm to a match on `Read`, or an
  `unreachable!` arm for `Read::Arrived` in a production side below"
  (code-organization.md, section 4).
- "**Every smith landing that changes code runs `smoke-tools` live**
  (smith's workflow.md, 5; "Live services" below), test code included:
  once through `heavy --gate` before `land`, its run identifier and
  verdict in the commit body" (README, "Rules").

## Increments

### 01.1 smith takes skein 01; a tool's and git's output read as they arrive
Scope: skein 01's `Read::Arrived` in smith; the tool runner's and git's
  children's pipes (`findings/reads.md`, 4, rows 1 and 3).
Needs: skein 01.1, 01.3 and 02.2 landed (the read, skein's readers, and
  the generator's worst cases); smith 02.1 (smith's gate passes here).
Files:
  - the first commit, the repin (development.md, 5.7): `Cargo.lock`
    (`heavy cargo update -p skein-lib`, which moves the skein source to
    its `main`); smith's three generated families, regenerated by skein's
    generator from skein's checkout at the same commit, one command per
    family: `heavy cargo run -q --manifest-path ~/src/rust/skein/Cargo.toml
    -p skein-codegen -- <smith>/crates/smith-charter/schema/charter-v2.schema
    <smith>/crates/smith-charter/src/generated`, and the same for
    `smith-transcript` (`transcript-v3.schema`) and `smith-channel`
    (`channel-v2.schema`), after which `tests/codecs/tests/drift.rs`
    passes and `git diff --stat -- '*/golden/*'` prints nothing; and the
    arms the new variant requires, `crates/smith-events/src/read.rs:180-190`
    and `crates/smith-protocol-machine/src/search.rs:290-300` (each puts
    `Arrived` with `Nothing` and `Line`: a tokenizer never demands it);
  - `crates/smith-protocol-machine/src/process.rs` (`:230`, `:571`;
    `OUTPUT_READ`), `crates/smith-protocol-machine/src/tests.rs`;
  - `crates/smith-local-protocol/src/git_child.rs` (`:130`, `:163`;
    `GIT_READ`) and its tests;
  - `tests/machine/src/lib.rs` (`flood` `:213-216`, the child output
    `:542-600`; `FLOOD_MOST`; the deliveries counted),
    `tests/machine/tests/stories.rs`.
Shape: each `Read::Fill(1)` named becomes `Read::Arrived { max: <its
  crate's const> }`; `Process::byte` (`process.rs:255`) and git's loop
  over the box (`git_child.rs:141-164`) stay as they are; the flood
  fixture as "Choices" says.
Test shape: in `tests/machine/tests/stories.rs`, beside
  `a_command_that_floods_its_output_keeps_its_head_and_tail` (`:141`),
  on the same `World::flood`: `a_megabyte_of_output_takes_few_deliveries`:
  a command writes 1,048,576 bytes; the head and tail are the flood's,
  and the fixture counts at most 1,024 deliveries to the machine
  (`Fill(1)`: 1,048,576). In `crates/smith-protocol-machine/src/tests.rs`:
  `a_tools_output_is_the_same_whatever_its_deliveries` (one box of 64
  bytes against 64 boxes of one: the same head, tail and count).
  `a_command_that_floods_its_output_keeps_its_head_and_tail` keeps its
  assertions. Ceilings: the story under 0.5 s alone, the unit test under
  20 ms.
Proof: world, `heavy cargo nextest run -p smith-machine-world -E
  'test(a_megabyte_of_output_takes_few_deliveries)'` once: it passes, its
  assertion on the deliveries being the evidence (no count is reported);
  `smoke-tools` once before `land`.
Don't: change the head and tail bounds, the capture, or a search's
  parsing; read the terminal's keys with `Arrived`.
Size: at most 40 lines of production code, besides `Cargo.lock` and
  generated code; the fixture's change is test kit.
Removes: the two readers' `Fill(1)`.

### 01.2 A host's standard error and the headless prompt
Scope: `findings/reads.md`, 4, rows 2 and 4.
Needs: 01.1.
Files: `crates/smith-host-protocol/src/process.rs` (`read_stderr`
  `:473-484`, `keep_tail` `:486-501`; `STDERR_READ`),
  `crates/smith-host-protocol/src/tests.rs` (`:242`, which expects the
  `Fill(1)`); `crates/smith-local-shell/src/front/headless/input.rs`
  (`demand` `:55-60`; `PROMPT_READ`) and its tests.
Shape: both demands become `Read::Arrived { max: <its crate's const> }`.
  `keep_tail` takes the last `capacity` bytes of the old tail and the
  delivery in one pass, as now; it is called once per delivery instead of
  once per byte.
Test shape: `crates/smith-host-protocol/src/tests.rs`:
  `a_hosts_standard_error_keeps_its_last_bytes_whatever_its_deliveries`
  (a tail of 8 from 20 bytes given as one box and as 20); the headless
  input's tests: `a_prompt_read_as_it_arrives_is_the_same_whatever_its_cuts`;
  the test at `tests.rs:242` expects the new demand.
Proof: unit, `heavy cargo nextest run -p smith-host-protocol -p
  smith-local-shell -E 'test(/whatever_its/)'` once, both passing;
  `smoke-tools` once before `land`.
Don't: change `detail_bytes`, the prompt's maximum or its failure.
Size: at most 25 lines of production code.
Removes: the two readers' `Fill(1)`.

### 01.3 The scripted person reads its child as it writes
Scope: the local process world's person (`tests/local-process/src/terminal.rs`).
Needs: 01.1.
Files: `tests/local-process/src/terminal.rs` (`piped` `:159-170`,
  `piped_complete` `:172-195`, `PIPED_OUTPUT`, `read` `:320-322`,
  `event` `:324-360`); `tests/local-process/tests/terminal.rs`
  (`a_piped_person_shows_whole_output_in_at_most_sixteen_reads_then_closes`
  `:8`).
Shape: `read` demands `Read::Arrived { max: OUTPUT_READ }`; `piped`
  adopts `output` into io as a read pipe and calls `read` on it; the
  person no longer submits or completes the output's own `PipeRead`s.
  The 65,536-byte bound on what it observes stays (`:353`).
Test shape: the local-process world's tests unchanged, but
  `a_piped_person_shows_whole_output_in_at_most_sixteen_reads_then_closes`,
  which keeps its name, its 60,000 bytes and its bound of sixteen: the
  reads it counts are now io's `PipeRead` submissions for the output, and
  the close it waits for is io's `Close` of the output's descriptor.
Proof: world, `heavy --gate cargo nextest run -p smith-local-process-world
  -p smith --profile measure` once: every test passes; the
  `smith-local-process-world::simulation` binary's summed time (3.90 s
  alone at this plan's base, `findings/measurements.md`, 5)
  and `smith_check_json_lists_the_effective_limits_by_name`'s time, in
  the report; `smoke-tools` once before `land`.
  One cue may change (ruling 112):
  `slash_new_starts_a_chat_and_slash_resume_reopens_it`'s
  (`tests/local-process/tests/simulation.rs:289`), whose text spans two
  output pieces whose order depends on the reads' sizes; it becomes the
  shortest text shown only once the new chat has answered, whatever that
  order, with every assertion unchanged.
Don't: change the person's script, its other cues or its bounds; change
  the person's scheduling to keep a cue's order; read the terminal's own
  keys differently.
Size: test kit only: at most 60 lines changed; it removes more than it
  adds.
Removes: `PIPED_OUTPUT` and `piped_complete`.

## Questions to raise when reached

- A memory test (`tests/*/tests/memory.rs`) that fails because a
  delivery of a reader's maximum is not in a bound smith's
  `protocol/limits.md` derives: the implementer reports the test, the
  bound and the term missing, and changes no derivation `limits.md`
  names.

## Done when

- `git -C ~/src/rust/smith grep -n 'Read::Fill(1)' -- crates/smith-protocol-machine crates/smith-local-protocol crates/smith-host-protocol/src/process.rs crates/smith-local-shell/src/front tests/local-process/src`
  prints nothing.
- `heavy cargo nextest run -p smith-machine-world -E 'test(a_megabyte_of_output_takes_few_deliveries)'`
  passes on smith's `main`.
