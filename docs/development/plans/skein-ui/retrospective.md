# Retrospective: skein-ui

2026-10-15. The run of jig's `skein-ui` plan on carbon-forge, claimed
at 2026-10-15T10:16:10Z, with `done.md` at 18:05:55Z and the supervisor's
"Done when" commands at 18:07 to 18:21. The release is still held
(section 5, item 11). It was the first run under the proposals of
client-composition and protocol that were decided that morning. Written
by a fresh session that took no part in the run (development.md, section
15).

Seven read-only sub-agents, one per source, read:
- the ledger, `log.md`, the 98 reports, the checkpoints, `context.md`
  and `done.md`;
- `supervision.md`, `decisions.md` (rulings 101 to 136) and `later.md`;
- skein's and jig's git history, with the trailers;
- the coordinator's Codex rollout;
- the 27 implementer rollouts;
- the supervisor's four Claude Code transcripts and their 22 sub-agents;
- the designer's eight sessions, the planner's ten transcripts, and the
  session that designed and planned the run.

This session read:
- the plan's `README.md` and `later.md`;
- `done.md`, `status.md`, `claude.tsv` and `logs/heavy.tsv`;
- client-composition's retrospective, and protocol's sections 8 and 10.

Times are UTC. Costs are at Opus 5.5's list prices. This document
proposes; the user decides, and the designer applies what is adopted.

## 1. In one page

- **Done in 7 h 50 min, and every "Done when" item holds.**
  - 75 steps landed: 70 planned and 5 added, with none dropped.
  - 9 escalations and 6 steers. One was a plan defect, made by a ready
    check's own correction, and no answer was reversed.
  - Seven design questions, answered in a median of 16 minutes, six of
    them by a fresh designer, with the design changed to match.
  - Nothing went to the user, except that the release is held at the
    end.
- **skein has its UI kit, and jig builds on it.**
  - There are six crates and the example: `skein-ui`, `skein-ui-text`,
    `skein-ui-web`, `skein-ui-bundle`, `skein-fake-person`,
    `skein-fake-person-browser`, and `examples/ui`.
  - jig's copies are gone, and 199 of its files name `skein_ui::`.
  - skein's gate builds for wasm32 and runs a browser tier of 35
    stories, in 23 s against a 120 s budget.
  - Every test-time aim was met. skein's focused median, 6.36 s, is
    below the base's 6.75 s, with 334 more tests.
- **The cheapest run per step so far,** at 7.78M tokens a step, against
  8.84M in client-composition.
  - The implementers' tokens a step fell by 37%.
  - The ready checks made about 138 corrections to 16 session files
    before their sessions started.
  - Every trailer was readable, and every Proof was at its layer.
- **What set the run's length was the design questions.**
  - On the chain that set the run's length, 61 of 468 minutes went to
    answers.
  - Four of the seven questions could have been answered before the
    run from text that was already there. The same was true of every
    design gap in client-composition.
- **Browser stories used the gate lane as their test loop.**
  - The implementers ran 33 gates of their own, 17 of which failed:
    39 minutes alone on the machine.
  - `land` then gated each passing tree again.
  - The queue waited 0.65 seconds for each second run, against 0.06.
- **Resumed contexts cost more than any reading.** Turns resumed after a
  question, at 123K to 184K, took 28% of the implementers' input.
- **Recurring, and waiting on skein's `kit-after-parity`:**
  - the coordinator's thread unloaded, and was loaded by hand six times;
  - its context ran to 414K before it compacted;
  - `plan downstream`'s worktrees held the release, for the second run
    in a row.
- **Ten proposals** (section 8). The first five target the run's chain
  and the implementers' tokens.

## 2. Outcome

The supervisor ran the plan's "Done when" at 18:07 to 18:21. It used
worktrees of skein at `f12f29d` and jig at `06ac4c469` (`done.md`).
This session ran items 2, 3, 4, 5 and 9 again on `origin/main`: skein
`c20bc49` and jig `17694847c`, which add only Markdown since.

| # | Item | Result |
|---|---|---|
| 1 | Every step landed or dropped | Holds: 75 of 75 landed, including 5 supplements: skein-13.s1, 14.s1 and 14.s2, and jig-05.s1 and 05.s2. |
| 2 | skein's crates exist | Holds: the `ls-tree` prints 12. |
| 3 | jig's copies are gone | Holds. The three paths print nothing, and 199 files of jig name `skein_ui::`. |
| 4 | jig pins skein's last landing | Holds: jig's `Cargo.lock` names skein `f12f29d`, skein-13.s1's commit and the plan's last on skein. jig keeps smith at `29fa97c9` (decisions.md, 4). |
| 5 | The gate's wasm32 builds and browser suite | Holds: `scripts/check.sh` lines 11, 12, 23, 25 and 31. |
| 6 | The browser tier passes on `main` | Holds: `heavy --gate scripts/check.sh` exited 0, with 35 browser tests passed in 23.0 s. |
| 7 | Commits for each session | Holds: skein 01 to 14 and jig 01 to 05, by their `Step:` trailers. |
| 8 | The test-time aims | Holds. Each is the median of three gate-lane runs; see the table below. |
| 9 | No budget raised | Holds: both diffs print nothing. |
| 10 | The aims are these items | Holds. |

| Repository | Suite | Base | Aim | Median |
|---|---|---|---|---|
| skein | focused | 6.75 s | about 9.8 s | 6.357 s |
| skein | fuzzy | 45.88 s | 48.88 s | 42.643 s |
| skein | browser | 3.27 s | 63.27 s (budget 120 s) | 22.1 to 23.0 s |
| jig | focused | 10.22 s | 11.52 s | 10.563 s |
| jig | fuzzy | 15.75 s | 17.05 s | 15.649 s |

The supervisor's first browser run left out `SKEIN_UI_EXAMPLE_BUNDLE`,
and one test stopped on the missing bundle. Its second, with the
bundle the gate builds, passed all 35.

## 3. Numbers

`plan metrics --history` has six earlier runs. The nearest in kind are:
- client-composition, jig's last, 36 steps in 4.3 h;
- protocol, the last retrospective's run, temper's, 93 steps in 16.9
  running hours;
- test-budgets, 39 steps in 3.7 h.

Per-step numbers compare better than totals.

| Metric | skein-ui | client-composition | protocol | test-budgets | Notes |
|---|---|---|---|---|---|
| `wall_h`, `running_h` | 8.1, 7.6 | 4.3, 4.3 | 18.5, 16.9 | 3.7, 3.7 | No pause. `wall_h` runs to `done.md`'s last write at 18:21; to its first, 7.83 h. Landings by hour: 12, 13, 13, 12, 11, 5, 4, 4, 1. |
| `steps_planned`, `added`, `dropped` | 70, 5, 0 | 36, 0, 0 | 112, 3, 22 | 36, 3, 0 | Two supplements fixed the browser kit, two the client world, one a test's bytes. |
| `step_h` | 0.10 | 0.12 | 0.15 | 0.09 | Up to four sessions at once until 14:53, then one or two (section 5, item 1). |
| `escalations` | 9: design gap 5, process 4 | 8 | 71 | 24 | 0.12 a step, against 0.22, 0.76 and 0.62. By content: 7 design questions (5 behind blocks, 2 from the supervisor), 4 process blocks, and 1 plan defect (13.5's, ruled as one). |
| `escalation_wait_min` | 4.3 | 1.9 | −153.0 | 1.8 | Process blocks took 1.6 to 4.4 min. Design questions took 6 to 25 min. |
| `blocked_h` | 1.3 | 0.4 | −277.7 | 0.6 | 17% of running hours, against 9%. Almost all of it is design questions and 14.2's crash. |
| `answers_reversed` | 0 | 0 | 5 | 3 | No `Reverses:` line, but ruling 122 calls 120's reason wrong, and 131 settles 126's premise (section 5, item 12). |
| `steers` | 6 sent, 0 refused | 2 | 5 | 4 | Three were on test shape (section 4, item 9). |
| `restarts` | none | none | 1 | none | The six hand loads of the coordinator's thread are not counted (section 5, item 7). |
| `tokens_coordinator` | 114.1M in (99% cached), 141K out | 41.6M, 74K | 271.0M, 625K | 71.8M, 162K | 1.52M a step, against 1.16M. Its context peaked at 414K (section 5, item 6). |
| `tokens_implementers` | 219.6M in (97% cached), 795K out | 168.4M, 369K | 870.4M, 3.4M | 201.9M, 479K | 2.93M a step, against 4.68M, 9.36M and 5.18M. |
| `tokens_claude` | 247.6M in the run; 83.0M before the claim; 2.4M after | 107.4M; 187.9M; 63.1M | 385.6M in the run | 62.7M in the run | 3.30M a step in the run, against 2.98M. |
| `cost_claude` | $149: planner $38, supervisor $80, designer $29, retrospective $1 | $132: $55, $37, $9, $31 | $192: supervisor $82, designer $49, retrospective $61 | not measured | The run's part, $109, is $1.46 a step, against $1.28. The retrospective's figure is this session's when `metrics.tsv` was written, before its readers. |
| `tokens_per_step` | 7.78M | 8.84M | 13.3M | 8.65M | The lowest so far. |
| `coordinator_requests` | 589, 32 after a wait ran out | 292, 12 | 1,404, 60 | 398, 9 | 7.9 a step, against 8.1. Empty wakes were 10.7% of its input. |
| `compactions` | 1 | 0 | 4 | 1 | Codex compacted automatically at 414K (16:55). |
| `gates_per_step` | 1.3 | 1.3 | 2.3 | 3.1 by the old definition | 101 land gates for 75 landings: 2 failed, and 24 ran again after main moved. |
| `gate_lane_other` | 39 | 1 | 0 | 56 | 33 gates the implementers ran as browser proofs (17 failed), and the 6 runs of "Done when" (section 5, item 2). |
| `queue_wait_ratio` | 0.65 | 0.06 | 0.31 | 2.13 | By hour: 0.40, 0.79, 1.13, 0.36, 1.22, 0.24, 0.25, 0.15. |
| `suites` | skein: 2,109 / 5.76 s → 2,443 / 6.53 s; fuzzy 99 / 38.2 → 43.8 s; browser 6 / 3.7 → 35 / 23.3 s. jig: 3,639 / 9.28 → 3,568 / 10.89 s; fuzzy 13.6 → 15.6 s | | | | These are single gates; section 2 has the medians. skein's gate rose from 52 to 84 s, mostly the browser tier and the wasm32 builds. |
| `proofs` | unit 28, world 15, joint 8, process 2, live 16, none 1, 1 below plan | | | | Wrong by two. Every step has a Proof at its session file's layer; the metric misreads "unit/world", "source", "wasm32", "focused" and "gate". |
| `report_lines` | 10 median, 11 p90 | 10, 10 | 13, 16 | 11, 14 | 98 reports, all logged by their implementers. |
| `lines` | code +15,350 −5,988; tests +16,235 −6,665 | +3,767 −930; +4,422 −244 | | | **Not comparable:** these include skein 01's 6,563 moved lines and jig 04's −3,721. |
| `pin_lag` | jig: skein 1, smith 127; temper: skein 96, smith 196, jig 188 | jig: 9, 31; temper: jig 115 | | | No repin downstream of jig, by decision 3. smith's parity run is active elsewhere. |
| `contexts` | 129K median, 237K p90; 2 handovers | 189K, 241K; 5 | 225K, 262K; 45 | 142K, 209K; 3 | A third handover, w20 to w22, has no `Handover` line. |

**What explains the largest differences:**

- **The implementers' tokens a step fell by 37%:**
  - **Reading.** The first edit came at a median 59K, against 64K to
    95K. All 15 fresh threads read `programming-model.md` and
    `testing-strategy.md` by sections only. Successors read their
    checkpoint first.
  - **Waiting.** Empty polls fell from 24% of the input to 15.4%.
  - **Size.** Increments were small: a median of 9 minutes from a
    thread's start or last landing to the next, and 2.8 landings a
    thread.
  - **Against that,** resumed contexts took 28% (section 5, item 3), and
    browser loops made the five dearest threads (section 5, item 2).
- **The queue rose from 0.06 to 0.65, for three reasons:**
  - **The implementers' own browser gates,** 33 tasks, 39 minutes
    alone. Each pauses the build lane, and four of them fell in the
    14:00 hour, at 1.22.
  - **Four sessions at once,** where client-composition ran two.
  - **The ready checks' builds,** 164 tasks in the build lane, which
    waited 56 minutes and ran 48. Six checks started together at 11:37,
    and the 12:00 hour was at 1.13.
- **The coordinator's tokens a step rose by 31%,** though its requests
  a step fell:
  - It never handed over, and grew about 50K an hour to 414K.
  - From 14:41 to 17:18, the escalations' phase, its requests averaged
    304K: 42% of its input.
  - Relaying rulings word for word was 30% of its output. Ruling 131
    alone was 16.4K characters, and became w25's 19.6K prompt.
- **Claude Code's tokens a step rose by 11% in the run:**
  - **The supervisor, $79.86:**
    - 16 ready checks: sub-agents $34.60, and its own corrections
      $13.31;
    - six hourly checks, $15.45.
  - **The designer, $29.50:** seven questions, against three, at about
    $4.20 each.
  - **The planning fell,** from $55 to $38.

**Not comparable, and why:**

- **`wall_h`** reads `done.md`'s last write, the supervisor's "Done
  when" output at 18:21, not its creation at 18:05:55.
- **`escalations`** counts `blocked.md`'s kind at a block's last
  listing. Two headings changed kind when listed again. Two questions
  came from the supervisor, not from a block.
- **`proofs`** misreads layer words it does not know (above).
- **`lines`** counts moved code as added and removed.
- **`tokens_claude` before the claim** counts the planner's ten
  transcripts. The design phase is not counted, though its landings
  carry `Designer-Session:` trailers: 11 sub-agents, $52.50. Neither is
  the session that designed, planned and started the run, $17.48 on its
  own. `numbers.sh` reading those trailers waits on `kit-after-parity`
  (client-composition's proposal 9).

## 4. What worked

1. **Every proposal was decided before the run, and those in force
   moved what they aimed at.**
   - client-composition's section 10 was decided at 09:07, and
     protocol's at 10:13, both before the claim at 10:16.
   - Their text landed in skein `e601ee2` and `d296e3e`.
   - Kit changes wait on `kit-after-parity` (section 7).
2. **Plan defects were fixed before their sessions started.**
   - Ready checks read 16 session files against `main`, in rulings 101
     to 123 and 133. They made about 138 corrections:
     - 42 users missing from Files;
     - 33 sketches that would not compile;
     - 18 stale lines;
     - 4 sizes;
     - 41 other points.
   - Four re-checks (111, 116, 119, 123) read only what `main` had moved
     since.
   - The one plan-defect block, 13.5's `Taller` at 16:57, came from a
     check's own correction (ruling 121), and was ruled in 98 seconds.
   - None of the 9 in-step blocks came from an increment no check read.
3. **The extraction order held:** generalize in jig, move as is, then
   build.
   - jig 01 to 03 landed by 11:24.
   - skein 01 moved three crates in ten minutes, at 0.94 to 0.96 of
     their lines.
   - jig 04 removed jig's copies, 3,721 lines, by 13:00.
   - No session found a jig name in skein's code. The README's `rg`
     printed nothing at each increment.
4. **The implementers followed the new card.**
   - Each read its documents by section, used the code graph on its own
     project only, and never listed projects or indexed.
   - Each checked its tip's trailers before 78 of 86 `land` calls. The
     other 8 were re-lands of a checked commit. git reads every trailer
     of all 137 commits; client-composition's jig-06.1 kind did not
     come back.
   - Each resolved its own rebase conflicts: six, none escalated.
   - Seven failing proofs were diagnosed below their layer, in a
     focused test or a host test.
   - w20 asked at the third crash, as the card says.
5. **Test time stayed low, and the browser tier came in whole.**
   - skein's focused median fell 0.39 s, and its fuzzy median 3.2 s.
   - jig's focused median rose 0.34 s, of 1.3 s allowed.
   - The browser tier added 29 tests and about 19 s, of 60 s allowed. The
     plan's aims were run-wide only, as client-composition's proposal 10
     asked.
   - No budget was raised, and no test moved to the fuzzy suite.
6. **The coordinator kept its card.**
   - It read no code and made no graph query.
   - It set every step in a command of its own, after reading its
     checks: a median of 16 s from a report.
   - It passed on every steer's text in later spawns.
   - It ended its turn with `plan idle` ten times.
   - It wrote `done.md` 69 s after the last report, and discarded
     nothing it did not own.
7. **The supervisor stayed small, and kept watching.**
   - It handed over three times, at 286K, 263K and 295K, and never
     passed 300K.
   - It used the 1-hour cache. Its 34 gaps of 5 to 34 minutes re-wrote
     nothing; at a 5-minute cache that would have cost about $29.90.
   - Its watch ran beside each of its 22 sub-agents: it was restarted a
     median of 16 s after each launch, and caught three blocks and two
     answers while checks ran.
   - It showed each block a median of 10 s after it was written, and
     acted a median of 92 s after that.
   - All 36 rulings went through `plan ruling`.
8. **Design answers reached the design.**
   - Each of the seven answers landed a design commit with its
     `Designer-Session:` trailer, and amended the session files in the
     same landing: skein `1b49e88`, `7c5a482`, `1ff0531`, `506b940`,
     `cef6a1a` and `b4c7740`, and jig `09864655c`.
   - Two needed Chromium's behaviour, which the designer probed itself:
     a date field, and a stale find scope.
   - Every deferring ruling landed its `later.md` row in the same
     commit: 10 rows added, 1 amended.
9. **Steers fixed test shape within the hour.**
   - One behaviour per test came at 11:57.
   - Named node ids came at 13:18, refined at 15:30: ids from position
     where a test does not name the node.
   - No test body writes `NodeId(<n>)` after 13:18, and the run wrote no
     test over 40 lines.
   - skein-14.1's 64 constants came down to 3.

## 5. What did not

1. **Design questions were the largest wait on the run's chain, and four
   of the seven were in the text before the run.**
   - **What:** the chain that set the length ran from 10:16:10 to
     18:04:08:
     - jig 01 → jig 02 → skein 01 → skein 09 → 11.3–11.4 → 12.1–12.3 →
       skein 13 → jig 05.

     Of its 468 minutes:

     | Wait | Minutes |
     |---|---|
     | Work | 368 |
     | Answers | 61: the designer's 49 (rulings 113, 127 at 27, 130 at 13, 136 at 8), the supervisor's 12 |
     | Ready checks | 27 |
     | Coordinator | 8 |
     | Slots | 4 |

     In the tail, skein 13 ran one increment a thread: three of its five
     increments met a question.
   - **Which could have been settled before the run ($12.87 of the
     designer's $29.50):**
     - **09.3:** `ui-web.md`, 6 gives `Control+K` or `Meta+K` by
       platform, but the plan's `mapping::element(made, look, prefix)`
       takes no platform. Ruling 108's check read 09.3 and missed it.
     - **14.2 and 14.4:** the plan's story signatures did not match
       `ui-web.md` 14.2's table.
     - **14.3:** the plan wrote "a `Gone` step within 1 s", where the
       fake person's `lib.rs:233` passes a `Gone` the first time its
       find is absent.
     - **13.1:** `ui-web.md` 5.1 registers `beforeunload`, and the
       plan's 12.3 story reloads with an add pending, but the kit had no
       handler for a dialog. The step waited 26 minutes, with no
       interim ruling.
   - **Which only the run could show:**
     - Chromium ignoring `Input.insertText` on a date field (13.2);
     - Chromium 147's renderer crashing on a stale find root (14.s1,
       14.s2);
     - seed 4503's sign-out race after the repin (jig 05.1).
   - **Cause:**
     - **The planner's three reviews checked the plan against itself
       and the code.** They found 60 items. None read each observable
       rule of the design sections a session cites against that
       session's Shapes and stories.
     - **The ready checks read the session against `main`,** not against
       its design sections.
     - client-composition's retrospective found the same: every one of
       its design gaps was there to read (its section 5, item 8). It
       made no proposal for it.
   - **First visible:** 12:34, 09.3's block. **Acted on:** each by the
     designer, in a median of 16 minutes.
2. **Browser stories used the gate lane as their test loop.**
   - **What:**
     - From 11.4 on, 16 browser increments ran `heavy --gate
       scripts/check.sh` themselves:
       - 33 runs, 17 failed: skein-13 13, skein-14 9, skein-12 8,
         skein-11 2, skein-14.s2 1;
       - 2,340 s alone on the machine, each pausing the build lane.
     - `land` then ran the same gate on each passing tree again.
     - The implementers also ran the browser profile 65 times in the
       build lane, 28 of them failing.
     - The five dearest threads were browser sessions: w16 22.1M, w11
       19.4M, w21 19.1M for one increment, w24 17.3M, and w20 14.2M for
       one landing.
     - 13.1 ran its proofs 19 times, 12 failed; 12.3 ran them 11 times,
       7 failed.
   - **Cause:**
     - **The README made the gate a Proof:** "A browser story's Proof
       is `heavy --gate scripts/check.sh` once in the increment's
       committed worktree". An implementer whose story failed had no
       cheaper place named.
     - **The browser suite needs the bundle that only the gate builds**
       (`SKEIN_UI_EXAMPLE_BUNDLE`). The plan gave no command that builds
       it once and runs one story in the build lane, though that works:
       w16 did it at 14:29, and the supervisor at 18:21.
     - **`land` keeps no record of a `heavy --gate` run,** so a passing
       proof's tree is gated again.
     - Protocol's proposal 1, "reproduced below its layer", was followed
       where a host test could hold the fault. The README named no lower
       layer for a story.
   - **First visible:** 13:35, 11.4's first failing gate. The 14:09
     check noted the queue. **Acted on:** not.
3. **Resumed contexts took 28% of the implementers' input.**
   - **What:**
     - Twelve turns, in nine threads, resumed a context of 123K to 184K
       after a question or a yield: 60.6M input.
     - w21 resumed at 172K after 13.1's 27-minute wait, and spent 11.1M
       more on that one increment, peaking at 261K.
     - w20 resumed at 157K and 184K between its three questions on 14.2
       (7.4M), and the step then went on in w22.
     - w11 resumed at 123K and 130K, and its third turn cost 10.8M.
     - A restarted thread's first edit came at 42K to 72K.
     - 193 requests ran past 200K (19% of the input), each in its
       thread's last increment.
   - **Cause:** development.md 7.3 sends a session to a fresh thread
     past the 200K limit or after an hour idle. A session that stopped on
     a question and waits 10 to 27 minutes is resumed at whatever context
     it holds.
   - **First visible:** 13:07, the check's 2.35M input a landing.
     **Acted on:** not.
4. **Ready checks still sat on the chain: about 92 minutes in all, 27 on
   the chain.**
   - **What:**
     - A check took a median of 11 minutes and $2.23. An increment took
       a median of 9 minutes, so a check started when the last need
       began running usually ended after it landed: 11 of 16 did.
     - Sessions then waited a median of 5.4 minutes after their last
       need landed, against about 11 in client-composition.
     - Six checks started together at 11:37, all waiting on skein 01.3.
       jig 04's ran 26 minutes, past the check's 25-minute stop. Its
       session was ready at 12:05, 21 minutes after its need, then
       waited 21 more for a place under the plan's limit of four.
     - About 47 slot-minutes sat idle while a session waited for its
       check. At 11:24 to 11:29 no session ran at all.
     - The checks were 60% of the supervisor's cost ($47.91).
   - **Cause:**
     - Moving the check to "all but one landed" halved the wait, but a
       check is still about as long as the increment it waits on.
     - Only one check, jig 05's, ran a suite. The other corrections were
       compiled sketches and text.
   - **First visible:** 11:44, six sessions in `check` with three places
     free. **Acted on:** the 11:41 check named it as expected.
5. **A failure a ready check met waited for its session.**
   - **What:**
     - jig 05's check ran jig's fuzzy suite on the repinned tree, and saw
       seed 4503 fail from 17:05:45 (`heavy.tsv`).
     - Ruling 133 (17:10) named it "a question to raise when reached".
     - jig 05.1 met it at 17:26:30, and the designer was woken at
       17:28:44.
     - The answer (17:43) and two supplements followed, and 05.1 landed
       at 18:04.
     - About 21 minutes of the chain's end went to the wait (inferred).
   - **By contrast,** skein 14's check sent its question at 14:07, and
     no step of skein 14 blocked on it (rulings 120 and 122).
   - **Cause:** development.md 10.4 sends a design gap to the designer
     at once. A failing seed is not yet a gap, and nothing says what a
     check does with a failure it cannot yet name.
   - **First visible:** 17:05. **Acted on:** 17:28.
6. **The coordinator's context ran to 414K, and it relayed long
   rulings.**
   - **What:**
     - It grew about 50K an hour: 154K at 12:00, 245K at 14:00, 363K at
       16:00.
     - Codex compacted it automatically at 16:55, from 414K to 22K.
     - The hourly checks reported 213K, 261K, 331K and 380K.
     - 13 unblocks were relayed word for word, from 0.9K to 16.4K
       characters each (about 80K in all), and 30% of its output went
       to relays and spawns.
     - Ruling 131's Decision, the designer's whole answer, became w25's
       19.6K-character prompt. 7.2's template is about 0.9K.
   - **Cause:**
     - Protocol's proposal 4, compacting at about 200K, waits on
       `kit-after-parity`.
     - development.md 10.4 makes an unblock's Decision append
       `answer.md`'s whole text. The coordinator must pass on the
       Decision word for word, and the implementer then carries it in
       every request.
   - **First visible:** 13:07 (213K). **Acted on:** not; the kit's fix
     waits.
7. **The coordinator's thread unloaded, for the second run in a row.**
   - **What:**
     - Rings found no loaded thread at 11:29, 15:08, 15:58, 16:14,
       17:28 and 17:45. The 17:29 handover says "every ring since
       15:08", so more may have gone unrecorded.
     - `plan resume` at 11:29 failed the same way as in
       client-composition.
     - The supervisor loaded the thread by hand with `codex_call
       thread/resume` six times. That delayed the coordinator 4 to 50 s
       each, and cost $1.10.
   - **Cause:** client-composition's proposal 4, which waits on
     `kit-after-parity`.
   - **First visible:** 11:29. **Acted on:** each time by hand, from the
     handover's recipe.
8. **Sizes went uncounted, and two increments passed twice their Size
   without asking.**
   - **What:**
     - `code-organization.md` gives no command that counts an
       increment's lines, though development.md 5.2 asks for one. No
       report gave a count.
     - skein-09.5 landed at 2.20× its Size (400) and skein-12.4 at 2.29×
       (300). Neither asked, and neither has a ruling.
     - The plan set six Sizes above the morning's "at most about 300":
       09.5 400, 10.1 350, 10.2 450, 11.2 350, 11.3 350 and 13.1 450.
       Rulings raised 13.1 to 700, and 13.2 from 250 to 470.
     - 13.1 filled one thread to its 261K peak, and skein 13 handed over
       twice.
   - **Cause:**
     - Protocol's proposal 2 changed development.md 5.2 and 5.3.
     - The planner's consistency review (5.4) does not list a missing
       count or a Size above 300 among what it looks for.
   - **First visible:** 13:27 (09.5). The hourly checks did not name
     it. **Acted on:** not.
9. **Work left for temper reached only jig's `later.md`.**
   - **What:**
     - Three `later.md` rows hold what temper's next repin onto this jig
       must change:
       - the renamed tree types and its users of the moved crates
         (decision 3, at planning);
       - its four `Limits` literals (ruling 133);
       - its joint front's `Reply::SignOut` (ruling 136).
     - temper's `STATUS.md` has none of them. In client-composition,
       temper `2dec7fb5` wrote that run's part there.
     - Rulings 127, 133 and 136 also named temper five times in jig's
       plan: `decisions.md` lines 817, 1394 and 1408, and the two rows.
       The plan's README forbade it ("no name of the repository
       downstream of jig in any file"), but its `rg` searched only
       skein's code.
   - **Cause:**
     - development.md 5.7 sends a repin's gap to the downstream
       `STATUS.md` and to `later.md`.
     - A plan that runs no repin, and a ruling that defers work
       downstream, have no such rule. The supervisor writes only
       `decisions.md` and `later.md`.
   - **First visible:** at planning, for decision 3's row; at 15:22 for
     the names. **Acted on:** not.
10. **The successor planner ran to 446K.**
    - **What:**
      - The first planner handed over at about 270K, with the plan about
        40% written.
      - Its successor's brief said "Past about 300K tokens, write a
        handover note … and stop". It crossed 300K at 09:29 and went on
        to 446K.
      - 93 of its 164 messages came after 300K, and cost $12.20 of its
        $18.30.
    - **Cause:** nothing tells a Claude Code session its context. `plan
      watch` prints it for the supervisor alone.
    - **First visible:** in this retrospective. **Acted on:** not.
11. **`plan downstream`'s worktrees held the release, for the second run
    in a row.**
    - **What:** at 18:06:22 the coordinator found `wt/jig-downstream`,
      `wt/smith-downstream` and `wt/temper-downstream`, each with a
      modified `Cargo.lock`.
      - It discarded nothing, and wrote a process block.
      - The supervisor recorded no disposition, and the run is not
        released.
      - In client-composition, ruling 120 removed
        `wt/temper-downstream` for the same reason.
    - **Cause:** `plan downstream` checks out each worktree in the run's
      `wt/`, runs `cargo update`, and never removes it (`plan`, lines
      1214–1253). `plan release` refuses while `wt/` holds anything
      (lines 479–480).
    - **First visible:** in client-composition, its ruling 120; here at
      18:06. **Acted on:** not. client-composition's section 9 listed it,
      with no proposal.
12. **A supervisor's ruling put code on `main` that the design forbade.**
    - **What:**
      - Ruling 126 (15:07) made step skein-14.s1, whose Shape takes a
        find's root from `Runtime.evaluate` of `document`.
      - browser.md 4 says "The kit never sends `Runtime.evaluate` or
        `Runtime.callFunctionOn` to run script". The plan's own skein 07
        had made that a check.
      - 14.s1 landed at 15:23. The designer's 16:13 answer named "a
        related point". The supervisor asked the question at 16:15. The
        designer amended browser.md 4 and 6 at 16:33 (`b4c7740`).
    - **Cause:** the ruling's check quoted browser.md 6 and not 4: the
      supervisor's own miss, as its question says. A step the supervisor
      writes is checked against the design by nobody else before it
      lands.
    - **First visible:** 16:13. **Acted on:** 16:15. It was settled at
      16:37, with step 14.s2 at 16:50.
13. **Small things in the record:**
    - **The supervisor:**
      - Its last session wrote no `supervision.md` entry after 17:45.
        The end shows only in `done.md` and `blocked.md`.
      - Each handover reset its hourly cron, leaving gaps of 87 and 75
        minutes.
      - Its six hourly entries were re-typed from the sub-agents'
        output.
    - **Corrections without `Reverses:`:** ruling 122 calls 120's reason
      wrong, 132 corrects 121's `Taller`, and 131 settles 126's premise.
      `answers_reversed` counts none of them.
    - **The coordinator:**
      - It asked w21 and w23 to hand over once 13.1 and 13.2 had landed.
        Both read the next increment before they stopped.
      - w20's move to w22 has no `Handover` line.
      - `context.md` reached 61 lines, against 60.
      - Three supplement notes are not in 7.3's shape.
    - **The implementers:**
      - 23 starts used a one-second yield, which the card forbids, in 10
        threads: w6 alone 11. 36 of 70 waits of 60 s came back empty.
      - Six threads read their start-up files in one `cat` that was cut
        at 30K tokens, then read them again one by one: about 4M input.
      - 63 of 133 graph searches and outlines (47%) returned nothing,
        because the code they asked for was the session's own, not yet
        on `main`.
    - **The designer:**
      - Two answers left a point "not decided here" with no `later.md`
        row. Each came back as the next question: 14.3's `loaded`, and
        14.s1's `Runtime.evaluate`.
      - The resumed session 478c82c3's design commits name its
        predecessor, c378fb63, in their trailers.
    - **The forbidden word "actual"** is in three of skein's tests.
      The 15:23 check saw it after their sessions had ended.
    - **`decisions.md`** still has the heading "The supervisor's rulings
      … None yet; they start at 101" (lines 124–126).
    - **This retrospective's own cost** was $1 when `metrics.tsv` was
      written, before its seven readers. client-composition's was $31.

## 6. Decisions taken during the run

Rulings 101 to 136 and six steers. The supervisor marked each decision
in force in `done.md` with its design commit, or "not in the design yet".

| Rulings | What | Reached the design | Proposal |
|---|---|---|---|
| 101–104, 106–111, 114–119, 121, 123, 133 | Session files corrected against `main` by the ready checks, and re-checks | Not a design matter: the session files, amended in the run | 4 |
| 105 | A gate whose only failure is the flaky terminal test retries once | Not a design matter. `later.md` has the test's row | Section 9 |
| 112, 113 | The primary modifier is a platform fact passed to the page and the mapping, outside Look | Yes: `ui-web.md`, skein `1b49e88` | 1 |
| 118, 120, 122 | What skein 14's stories take from the application: `arrives`, `failed`, and `loaded` before absent | Yes: skein `7c5a482`, `1ff0531` | 1 |
| 124, 125 | 14.2's crash: two bounded diagnoses | Not a design matter | — |
| 126, 131 | The browser kit finds across a page's own navigation; a find's root and scope resolve in the current document | Yes, after the fact: browser.md 4 and 6, skein `b4c7740` (section 5, item 12) | — |
| 127 | A go accepts a closing warning as `Warned`; other dialogs are dismissed as `Trouble::Dialog` | Yes: skein `506b940` | 1 |
| 128 | `browser.rs` past its allowance, named in the README; its split is a `later.md` row | Not a design matter | Section 9 |
| 129, 130 | A native select is found with `pierce`; a date or time field is typed by `keys` | Yes: skein `cef6a1a` | — |
| 132 | `Taller` measures, types its words, and polls for growth | Not in the design: a harness step. It corrected ruling 121 | 4 |
| 134, 136 | The referee takes no frame after its watch ends; the shell drops an earlier sign-out's answer after a newer sign-in | 136 yes: `client.md` 12, jig `09864655c`. 134 is a harness defect | 5 |
| 135 | The chart test's control bytes are written as escapes | Not a design matter | — |
| Steers 11:57, 13:18, 15:30 | One behaviour per test; named node ids; ids from position | Already in development.md 5.6 and 14.3, and the plan's "Don't" list | — |
| Steers 14:17, 14:22, 17:31 | Start orders and an amended session file | The plan only | — |

**Decided by the user during the run:** nothing; nothing was asked.

**Still with the user:** `decisions.md`'s "For the user" lists the
plan's eight long-term choices, all accepted on 2026-10-15:
- the glue pinned at 0.2.126;
- one JavaScript snippet;
- `jig-shell-native` kept one crate;
- the example's page as the bundler's;
- no repin downstream of jig;
- the digest's encoding, SHA-256 in the bundler, and an FNV-1a ETag.

No ruling added an item.

## 7. The proposals adopted after the last retrospective

The last retrospective is protocol's, decided at 10:13 and applied in
skein `d296e3e`. client-composition's remaining proposals were decided
at 09:07 and applied in skein `e601ee2`. Both were in force at the claim.
Kit changes wait on skein's `kit-after-parity`, until smith's parity run,
still active, has its retrospective.

**protocol's fourteen:**

| Proposal | Decision | Meant to move | protocol | skein-ui | Moved |
|---|---|---|---|---|---|
| 1. A failing proof reproduced below its layer; a third symptom is a question | adopted | `gate_lane_other` from 1.09 a step toward one run a proof; implementers' tokens | 1.09 a step | 0.52 a step: 33 browser proofs, 2.1 an increment, and 6 "Done when" runs | In part. Seven diagnoses went below the layer, and w20 asked at the third crash. Stories had no lower place (section 5, item 2). |
| 2. Sizes of at most about 300, landed within 2× | adopted | size escalations from 6; `contexts` p90 from 262K; handovers a step from 0.53 | 6; 262K; 0.53 | 0 asked; 237K; 0.04 | Yes, but two landed past 2× unasked, and the plan set six Sizes above 300 (section 5, item 8). |
| 3. Limits from the shipped profile | adopted | late escalations; `steps_added` | 3 added | 5 added, none from limits | Held. jig 05.1 still repaired 33 hand-written `Limits` literals (section 9). |
| 4. The coordinator compacts at about 200K | deferred | `tokens_coordinator` | 2.91M a step | 1.52M; peak 414K | Not in force. 42% of its input came at an average of 304K. |
| 5. No code graph for the coordinator; implementers on their own project | adopted in part | coordinator and implementers' tokens | 10.7% and 7.7% of input | none; 157 calls, own project only | Yes |
| 6. A step set in a command of its own, after its checks | adopted in part | `coordinator_requests` a step from 11.7 | 11.7 | 7.9 | Yes: all 75 sets after their checks |
| 7. A session idle over an hour goes to a fresh thread | adopted | `contexts` p90; implementers' tokens | 262K | 237K; three restarts, first edits at 42–49K | Yes, for that case. Shorter waits are still resumed (section 5, item 3). |
| 8. A ring is the supervisor's; a resume names its cause | adopted in part | `running_h`; `restarts` | 1.6 h lost | no reboot, no resume | Held: no ring was logged as the user's |
| 9. The card names the 300 s `wait` | adopted in part | poll-only share | 4.35% by its own count | empty polls 15.4% (24% in client-composition) | In part: 23 one-second yields; a cell returns after about 30 s whatever the yield |
| 10. The disk rule | adopted | infrastructure escalations from 5 | 5 | 0 | Held |
| 11. `plan metrics` counts what 15.2 says | deferred | comparability | | | Not in force: `proofs` and `wall_h` misread (section 3) |
| 12. `land` treats a lost ref-lock as main moving | deferred | infrastructure escalations | 1 | 0 | Not exercised: no race |
| 13. A live proof ends at its first terminal failure; `Completions:` and `Wall:` trailers | adopted in part | `queue_wait_ratio`; live usage | | no live service | Not exercised: the browser stories run a local Chromium |
| 14. A deferral lands with its `later.md` row | adopted | deferrals without a row, from 2 | 2 | 0 of the rulings'; 2 of the designer's points "not decided here" | Mostly. Each unrecorded point came back as a question. |

**client-composition's ten:**

| Proposal | Decision | Meant to move | client-composition | skein-ui | Moved |
|---|---|---|---|---|---|
| 1. A check from "all but one landed", in the background | adopted | a session's start after its need lands, from about 11 min; `wall_h` | about 11 min; 103 min in all, 70 on the chain | 5.4 min; 92 in all, 27 on the chain | Yes, by half (section 5, item 4). The watch ran beside every check. |
| 2. The increment whose Needs land first leads its session | adopted | `wall_h`; the tail's `step_h` | 45 min lost | no session waited behind a later increment's need | Held. The tail's single session was the plan's shape. |
| 3. A check reads every increment, and runs the suites it reaches | adopted | plan-defect escalations from 0.08 a step; `blocked_h` | 0.08; 0.4 h | 0.01 (one, a check's own); 1.3 h, from design questions | Yes for defects. Suites ran in one check of 16, and that one found seed 4503. |
| 4. `plan ring` and `plan resume` load the thread | deferred | the supervisor's tokens; `restarts` | six hand loads | six hand loads | Not in force, and it recurred (section 5, item 7) |
| 5. `plan watch` shows a successor what is open | deferred | no event waits on a handover | 20 min lost | none lost | Held, by the card's text: the successor restarted open checks |
| 6. Implementers read by section; successors read the checkpoint; the longest yield | adopted | `tokens_implementers` from 4.68M a step; `contexts` | 4.68M; 189K median | 2.93M; 129K | Yes |
| 7. The planner and designer neither poll nor sleep; hand over past 300K | adopted | the planning's cost | $55; 617K | $38; 446K | In part: no polling, but the successor ran past 300K (section 5, item 10) |
| 8. The implementer checks that git reads its trailers | adopted | `proofs` counts every step; process escalations | jig-06.1 unreadable | 137 of 137 readable | Yes |
| 9. `Designer-Session:` trailers | adopted in part | comparability | $28.13 uncounted | every design commit carries one; $70 uncounted | The trailers yes; `numbers.sh` reading them waits on the kit |
| 10. Test-time shares only above one gate's swing | adopted | none directly | shares fired on noise | run-wide aims only, all met | Held |

**The same day's other changes:**
- **development.md 4.6, the light path for vocabulary additions,** was
  never used. No session needed a variant.
- **The skein-ui design's rule, "generalize in jig first, then move as
  is, then build what is new",** worked (section 4, item 3).

## 8. Proposals

Each gives a rule in one sentence, its evidence, the document it
changes, and the metric it should move. They are ordered by expected
effect. Proposals 9 and 10 change the kit, and would wait for
`kit-after-parity`, as the user decided for the others. The rest change
text only.

1. **Before the plan lands, a read-only sub-agent reads each session's
   cited design sections against its Shapes, Test shapes and stories,
   and lists each observable rule that no increment builds or tests and
   no line names as open: a key, an event, a dialog, the order of two
   answers. The designer answers each before the run.**
   - **Evidence:**
     - Four of seven questions were answerable from text there before
       the run: 09.3, 14.2/14.4, 14.3 and 13.1. That was $12.87, and
       about 27 of the chain's 61 minutes of answers (rulings 113 and
       127).
     - In client-composition, every design gap was also there to read.
     - The planner's three reviews found 60 items, none of this kind
       (section 5, item 1).
   - **Changes:** development.md, 5.4: a fourth review, the design
     trace, beside the consistency review and the literal reading; 2.2.
   - **Moves:** design questions a step (0.09) down; `blocked_h` (1.3)
     down; the designer's cost in the run down.
2. **A Proof that is the repository's gate is `land`'s own gate. The
   implementer runs no gate of its own, and runs a failing story alone
   in the build lane, with the command the plan gives to build the
   story's artifact once.**
   - **Evidence:**
     - 33 gates run as proofs, 17 failed, 2,340 s alone on the machine.
       Each passing tree was then gated again by `land`.
     - The queue was at 1.22 in the 14:00 hour.
     - The five dearest threads were browser sessions.
     - One story runs in the build lane with `SKEIN_UI_EXAMPLE_BUNDLE`,
       as w16 and the supervisor did (section 5, item 2).
   - **Changes:**
     - development.md, 5.5: the Proof line;
     - 5.2: the README names the build-lane command for one test of a
       suite that needs a built artifact;
     - implementer.md, 8.8.
   - **Moves:** `gate_lane_other` from 0.52 a step toward the "Done
     when" runs alone; `queue_wait_ratio` from 0.65 down;
     `tokens_implementers` in browser sessions down.
3. **A session that stopped on a question, or yielded, and whose context
   is past 150,000 tokens, goes on in a fresh thread from its
   checkpoint, as after a handover.**
   - **Evidence:**
     - Twelve resumed turns, at 123K to 184K, took 60.6M: 28% of the
       implementers' input.
     - w21, resumed at 172K, spent 11.1M on one increment.
     - A restart's first edit came at 42K to 72K (section 5, item 3).
     - The designer's limit is already 150K.
   - **Changes:** development.md, 7.3, step 4, and 10.3, step 3;
     implementer.md, 8.6: a question's report writes the checkpoint.
   - **Moves:** `tokens_implementers` from 2.93M a step down; `contexts`
     p90 from 237K down.
4. **The coordinator sets a session to `check` once every session it
   needs at its start is running. The supervisor checks it then, and
   once its needs land re-checks only what `main` moved, before `plan
   ready`.**
   - **Evidence:**
     - A check took a median of 11 minutes, and an increment 9. 11 of
       16 checks ended after the need landed.
     - Sessions then waited a median of 5.4 minutes: 92 in all, 27 on
       the chain.
     - The re-checks (111, 116, 119, 123) took minutes (section 5, item
       4).
   - **Changes:** development.md, 7.2 and 7.3 (the states);
     supervisor.md, 5.
   - **Moves:** the wait after a session's need lands, from a median of
     5.4 minutes to about 1; `wall_h` down.
5. **A ready check that sees a suite fail, or a rule the design must
   settle, sends it on before the session starts. It goes to the
   designer as a question, or becomes a supplement, never "a question to
   raise when reached".**
   - **Evidence:**
     - jig 05's check saw seed 4503 fail at 17:05. The designer was woken
       at 17:28, after the session met it: about 21 minutes at the
       chain's end.
     - skein 14's check sent its question at once, and nothing blocked
       (section 5, item 5).
   - **Changes:** supervisor.md, 5; development.md, 10.4.
   - **Moves:** `blocked_h` down; `wall_h` down.
6. **`code-organization.md` gives the command that counts an
   increment's hand-written production lines. Every Size is at most
   about 300. The consistency review checks both, and each landing's
   report gives its count.**
   - **Evidence:**
     - There was no count, and no report gave one.
     - 09.5 and 12.4 landed at 2.20× and 2.29× unasked.
     - Six Sizes were above 300. 13.1, raised to 700, filled one thread
       to 261K (section 5, item 8).
   - **Changes:** development.md, 5.4 (the consistency review's list);
     implementer.md, 8.6.
   - **Moves:** increments past 2× without a ruling, from 2 to 0;
     `contexts` p90 down.
7. **An answer gives the session its decision in at most 40 lines,
   quoting the design text it rests on. The unblock's Decision appends
   only that part, and the reasoning stays in `answer.md`, appended to
   `supervision.md`, and in the design commit.**
   - **Evidence:**
     - 13 unblocks of 0.9K to 16.4K characters were relayed word for
       word.
     - Ruling 131 became a 19.6K prompt.
     - Relays and spawns were 30% of the coordinator's output (section
       5, item 6).
   - **Changes:** development.md, 10.3 and 10.4.
   - **Moves:** `tokens_coordinator` from 1.52M a step down; a
     supplement's start-up context down.
8. **What a plan leaves for a repository downstream of it, by its own
   decision or by a ruling or an answer, goes to that repository's
   `STATUS.md`, under "Not planned", in the designer's next landing, as
   section 5.7 says for a repin's gap. The plan's own files name that
   repository as its README does.**
   - **Evidence:**
     - Three `later.md` rows of work for temper's next repin are not in
       temper's `STATUS.md`, where client-composition wrote its part
       (temper `2dec7fb5`).
     - Rulings named temper five times against the plan's own rule
       (section 5, item 9).
   - **Changes:** development.md, 5.7 and 10.4; supervisor.md, 3.
   - **Moves:** deferred downstream work in the downstream `STATUS.md`,
     from 0 of 3 rows to all.
9. **`plan context` prints a Claude Code session's own context. The
   planner, and the designer of a new design, run it after each
   sub-agent's report, and hand over past 300K.**
   - **Evidence:** the successor planner ran from 300K to 446K over 93
     messages, $12.20, though its brief said to stop (section 5, item
     10).
   - **Changes:** `plan`; development.md, 2.1 and 2.2.
   - **Moves:** `cost_claude`'s planning part ($38) down.
10. **`plan downstream` checks out its worktrees outside the run's
    `wt/`, and removes them when it ends.**
    - **Evidence:** the release was held at the end of this run and of
      client-composition, by `wt/*-downstream` with a regenerated
      `Cargo.lock` (section 5, item 11).
    - **Changes:** `plan` (`downstream`); development.md, 14.1.
    - **Moves:** the run released at `done.md`; no process block at the
      end.

## 9. Debts left in the code

For jig's `tech-debt.md`, and for skein's (its rows are for the
designer).

**jig:**
- **33 hand-written `skein_ui::Limits` literals in 30 files,** each with
  `scene_marks: 0` and `scene_bytes: 0`. Examples:
  `crates/jig-view/src/limits.rs:94`, `owner_deployment.rs:685`,
  `owner_result.rs:122`, `tests/client/src/model.rs:188`. One
  constructor would replace them.
- **`crates/jig-view/src/tests.rs:67`'s `text_bytes`** re-implements
  skein-ui's private node accounting.
- **Files past 800 lines grew:** `tests/client/src/joint/session.rs`
  1,538 → 1,555, and `harness/world.rs` 949 → 955 (05.s2).
- **From `later.md`:**
  - a sign-out's answer kept to its sign-in in `jig-shell-native` and
    the scripted front;
  - the client world's transport forgetting a newer sign-in;
  - jig's browser layer, its frame parts and cards as primitives, which
    wait for the boundary analysis.

**skein:**
- **`testing/skein-browser/src/browser.rs` at 2,041 lines,** grown 267
  in the run. Its split is a `later.md` row.
  - Close to the limit: `skein-ui-web/src/page.rs` 777, `page/web.rs`
    767, `tests/browser/tests/support/real.rs` 759.
- **Lint exceptions:**
  - new: `skein-ui-web/src/platform/web.rs:54` and `:90` (casts), and
    `browser.rs:720` (`disallowed_methods`);
  - moved from jig into tests, against the plan's "Don't" list:
    `skein-ui/src/tests/diff.rs:1-3`, `tests/markdown.rs:1-3`,
    `skein-ui-text/src/tests.rs:8,17`, and `allow-unwrap-in-tests` in
    both fake-person `clippy.toml`s.
- **"actual" in three tests:**
  - `skein-ui-text/src/tests/bound.rs:97-103`;
  - `skein-fake-person-browser/src/tests.rs:308-309`;
  - `skein-fake-person/src/tests/random.rs:236-237`.
- **Fixtures defined again:**
  - `page()` in 7 files, and `binding()` in 4;
  - `tests/ui-text/src/lib.rs:19,22` copies
    `skein-ui-text/src/tests/fixtures.rs:21,24`.
- **The root id is a bare `NodeId(0)`** in 13 places, among them
  `skein-ui/src/tree.rs:308` and `skein-ui-web/src/page.rs:224`.
- **Code cites a plan's rulings:** `skein-ui-text/src/render.rs:7`
  ("ruling 106") and `harness/script/growth.rs:1` ("Ruling 132"). It
  should cite the design.
- **From `later.md`:**
  - the flaky `terminal_is_controlling_and_the_persons_interrupt_
    reaches_only_the_child`;
  - the face's typing into a field holding words;
  - a test answering a native dialog;
  - a press on a node of an earlier document refused as `Protocol`, not
    `Gone`;
  - a find within a removed scope.

**skein, the kit:**
- The coordinator's thread unloads (client-composition's 4).
- The coordinator does not compact at 200K (protocol's 4).
- `plan downstream`'s worktrees (proposal 10), and its death under
  `pipefail` without temper's line.
- `numbers.sh` reads `wall_h` from `done.md`'s last write, misreads
  Proof layers, and reads no `Designer-Session:` trailer.

**Workspace:**
- temper's pins are 188 commits behind jig's `main`. Its `STATUS.md`
  lacks this plan's three rows for its next repin, now only in this
  plan's `later.md` (proposal 8).
- jig pins smith 127 commits behind smith's `main`, by decision 4.
- **The run is not released.** The three `wt/*-downstream` worktrees
  hold only `plan downstream`'s regenerated `Cargo.lock`.

## 10. Decisions

| Proposal | Decision | Where |
|---|---|---|
| 1 | open | |
| 2 | open | |
| 3 | open | |
| 4 | open | |
| 5 | open | |
| 6 | open | |
| 7 | open | |
| 8 | open | |
| 9 | open | |
| 10 | open | |
