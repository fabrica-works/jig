# Retrospective: test-budgets

2026-10-14. The run of jig's `test-budgets` on carbon-forge, claimed at
2026-10-13T23:09:32Z and done at 2026-10-14T02:49:05Z: development.md's
third run. Written by a fresh session that took no part in the run
(development.md, section 15).

Six read-only sub-agents, one per source, read:
- `log.md`;
- the git history of skein, smith and jig, with the trailers;
- the coordinator's Codex rollout;
- the 20 implementer rollouts;
- the supervisor's and the designer's Claude Code transcripts;
- the earlier retrospectives and their metrics, and the history of the
  process documents since.

This session read the plan with `decisions.md` and `later.md`, `done.md`,
`supervision.md`, `context.md`, the ledger, `logs/heavy.tsv`, and the
transcript of the session that designed and planned the run. Times are
UTC. This document proposes; the user decides, and the designer applies
what is adopted.

## 1. In one page

- **Done in 3 h 40 min, and every "Done when" item holds.**
  - 39 steps landed: 36 planned and 3 added, with none dropped.
  - 21 escalations, answered in a median of 1.8 minutes, and 4 steers.
  - No design question, and nothing waited on the user.
- **The suites are back within budget, with room.**
  - Medians of three gate-lane runs on carbon-forge, against an aim of
    12 s:
    - skein 5.590 s, from 9.5–11.4 s;
    - smith 9.021 s, from 21.0 s (24.1 s in its last gate);
    - jig 4.728 s, from 15.0 s.
  - Each default profile has its 15 s timeout again.
  - Turning off nextest's double spawn bought 3.2 s in skein and 3.3 s in
    jig at their first landings. Fewer and cheaper tests, and regenerated
    codec tables, bought the rest.
  - skein's read of what has arrived is built, and smith's and jig's
    readers use it.
  - The hand-written change was small: about +400 −140 lines of
    production code and +2.4K −0.8K of tests. 25K of the lines added were
    regenerated families.
- **The work found two defects and a question:**
  - a TLS failure that overrode a complete HTTP response (skein-01.3);
  - the echo race behind smith's window story, the flake the plan's
    findings had recorded (smith-02.s1);
  - what jig's recovery records after a worker's clean exit, not yet
    checked against jig's design. The repin found it when fewer
    deliveries moved the one-process stories' cuts past the run's end.
- **The cheapest run per step so far, but the planning cost more than
  the run.**
  - The run used 8.65M tokens per landed step. jig-protocol used 21.5M,
    counted once, and tls-cleanup 9.9M.
  - The session that designed and planned this run used about 412M input
    tokens, 274M of them in 20 sub-agents, most of them audits. That is
    more than the run's 340M, and `plan metrics` counts none of it.
- **What cost time:**
  - **Plan defects:** 11 of the 21 escalations. Nine were a Files line
    that missed a user, a copy or a derived literal of what the step
    changed. That is 0.28 a step, flat over three runs.
  - **The generator:** skein-02.2 changed the code generator's output and
    proved it in skein alone. smith's and jig's clippy refused it, and
    two added steps landed on the longest chain.
  - **jig 05.1:** five stops in an hour on the critical path.
    - Rulings 114, 115 and 117 searched for a value.
    - The search that found all four copies of smith's goldens came only
      with 118.
    - Three answers were partly reversed, all of them the supervisor's
      own narrow rulings.
  - **The queue, worst in the first hour:**
    - In that hour, tasks waited 3.9 seconds for each second they ran.
    - Four jig sessions landed at once. Over the run 17 gates ran again
      after main moved, 14 of them in jig.
    - The implementers' wait returns after at most 60 s, so each minute
      in the queue cost a request at the thread's whole context. Empty
      polls took about 36% of the implementers' input.
- **The last retrospective's proposals were not decided.**
  - None of its 17 was adopted. This plan was asked for at 20:53Z,
    before that retrospective landed at 21:22Z.
  - Six of its items recurred:
    - trailers git cannot read;
    - a supervisor at 425K with no handover;
    - gates run again after main moved;
    - empty polls;
    - a coordinator re-typing its reports, then compacting;
    - metrics that miscount.
- **Thirteen proposals** (section 8).
  - The first makes the next run wait for the user's decision on them.
  - The next four target the plan defects and the gate lane.

## 2. Outcome

The plan's "Done when", against the three `origin/main`s at skein
`9539e00`, smith `527884dc` and jig `0031b05e3`:

| # | Item | Result |
|---|---|---|
| 1 | Every step landed or dropped | Holds: 39 of 39 landed. `plan status` still says that no commit on `origin/main` names jig-02.1. Its commit `2497c8299` has `Plan:` and `Step:` lines that git does not read as trailers (ruling 102; section 5, item 8). |
| 2 | Each focused suite has its 15 s timeout | Holds: line 9 of each `.config/nextest.toml`, under `[profile.default]`. skein (`bcf5e06`) and jig (`1525519e7`) moved 30 s back to 15 s. smith's line never changed: smith ran without it here under ruling 121's gate until smith-04.1 (`29fa97c9`) returned it to `scripts/check.sh`. |
| 3 | Each budget landing passed its own gate here | Holds. skein-02.5: `scripts/check.sh passed: 2104 tests / 6.039 s`. smith-04.1: 1995 / 10.089 s. jig-06.1: 3514 / 8.113 s. |
| 4 | Each suite met the aim | Holds: `Median: 5.590 s`, `9.021 s` and `4.728 s` in the three bodies. jig-06.1's own gate ran at 8.113 s against its median of 4.728 s: one gate under load varies by about 70%, which the 3 s margin is for. |
| 5 | The double spawn is off in every gate and the queue | Holds: a line in each `check.sh` and in `heavy:88`. |
| 6 | The read of what has arrived is built and used | Holds by the record. skein's megabyte story passed once on `bcf5e06` (`logs/skein-01/done-when-main-nextest.log`), smith's once on `6e863b4f` (`logs/smith-01/done-main-proof.log`). Both tests are on `main`; this session did not run them again. |
| 7 | The aims are these items | Holds. Decision 5's fallback, each crate's unit tests in one process, was not needed. |

## 3. Numbers

`plan metrics --history` has two earlier runs. jig-protocol had 217 steps
over 29.8 h; tls-cleanup had 13 over 1.3 h. This run is between them, so
per-step numbers compare better than totals.

| Metric | test-budgets | jig-protocol | tls-cleanup | Notes |
|---|---|---|---|---|
| `wall_h`, `running_h` | 3.7, 3.7 | 29.8, 29.5 | 1.3, 1.3 | No pause, no restart. The planning took 2 h 14 min before the claim (20:51Z to 23:05Z). |
| `steps_planned`, `added`, `dropped` | 36, 3, 0 | 166, 51, 0 | 11, 2, 0 | Added: smith-02.s1 (the window story's fix), skein-02.s1 and 02.s2 (the generator's tables, twice). |
| `step_h` | 0.09 | 0.14 | 0.10 | 15 landings in the first hour, 14 in the second, 7 in the third, 3 in the last 40 minutes. From 01:53Z one session ran at a time, by the chain of dependencies. |
| `escalations` | plan defect 12, cross-session scope 7, process 4, design gap 1 | 255 | 12 | 24 headings, 21 distinct (below): 0.54 a step, against 0.97 and 0.92. |
| `escalation_wait_min` | 1.8 | 1.6 | 1.3 | Each block was answered 40 s to 2 min 49 s after it was filed. |
| `blocked_h` | 0.6 | 8.2 | 0.2 | 16% of running hours, against 28% and 15%. |
| `answers_reversed` | 3 | 38 | 2 | 0.08 a step, against 0.17 and 0.15. All three widened the supervisor's own ruling (section 5, item 4). |
| `steers` | 4 sent, 0 refused | 24 | 1 | |
| `restarts` | none | none | none | |
| `tokens_coordinator` | 71.8M in (99% cached), 162K out | 699M, 1.5M | 15.9M, 55K | 1.84M a step, against 3.22M and 1.23M. One compaction, at 368K. |
| `tokens_implementers` | 201.9M in (98% cached), 479K out | 2,761M, 9.8M | 43.2M, 175K | 5.18M a step, against 12.7M and 3.3M. Empty polls were about 74M (36%). |
| `tokens_claude` | 62.7M in the run, 0 before the claim | 1,184M counted once (5,153M by the metric) | 227.6M | 1.61M a step. The supervisor took 63.0M, the designer 0.3M. "After" is this retrospective's own tokens up to when `metrics.tsv` was written. |
| `tokens_per_step` | 8.65M | 21.5M counted once (39.8M by the metric) | 9.9M for its run (22.2M by the metric) | **Planning is missing** (below). |
| `coordinator_requests` | 398, 9 after a wait ran out | 3,567, 67 | 147, 4 | 10.2 a step, against 16.4 and 11.3. 61% of its wall time was waits. |
| `compactions` | 1 | 9 | 0 | At 02:25:49Z, in the end chain: 148 s, while ruling 119's step waited. |
| `gates_per_step` | 3.1 | 2.2 | 2.5 | **Not comparable** (below): 65 land gates, 1.67 a step. |
| `queue_wait_ratio` | 2.13 | 2.24 | 0.66 | By hour: 3.90, 1.94, 0.68, 0.01. Gate lane 1.94, build lane 2.63. |
| `suites` | first 2087 tests / 8.234 s; last 3514 / 8.113 s | | | **Not comparable:** "first" is skein's, "last" is jig's (below). |
| `proofs` | unit 15, world 12, process 9, none 1; 0 below plan | | | 37 of 39: jig-02.1's Proof is not read, and skein-02.1's is the "none". "0 below plan" compares nothing (below). |
| `report_lines` | 11 median, 14 p90 | 10, 11 | 10, 10 | |
| `lines` | code +23,036 −14,193; tests +5,030 −4,238 | | | **Not comparable:** jig-05.1's regenerated families alone changed 36K lines. Hand-written: production about +400 −140, tests and test kits about +2.4K −0.8K. |
| `pin_lag` | smith: skein 1; jig: skein 1, smith 1; temper far behind | | | smith and jig are behind only by the STATUS commits that closed the run. temper is outside the plan (decision 12). |
| `contexts` | 142K median, 209K p90; 3 handovers | 238K, 319K; 70 | not measured | One thread peaked at 248K, after its handover, inside its increment. |

**What explains the largest differences:**

- **Tokens per step fell to 40% of jig-protocol's, for four reasons:**
  - The designer was never woken. Its wakes cost 514M in jig-protocol.
  - The supervisor ran 3.7 hours, not 29.8. Its context peaked at 425K;
    jig-protocol's second supervisor ran 550 of its requests above 500K.
  - The sessions were small and their code existed, so implementer
    contexts peaked at 142K median, against 238K.
  - The coordinator handled 10 requests a step, against 16.
- **The queue waited as much as jig-protocol's.** This plan measured
  every Proof in the gate lane by rule, and 56 of its 121 gate-lane tasks
  were measurements and live runs. The first hour, with seven sessions
  starting at once, made most of the wait (section 5, item 5).

**Not comparable, and why:**

- **Planning is not counted.** "Before the claim" reads 0 because the
  session that designed and planned the run (`171e69d3`, 20:51Z to
  23:05Z) is not in `claude.tsv`. It used 137.8M input tokens itself,
  peaking at 709K, and 274.5M in 20 sub-agents (21:07Z to 21:59Z): the
  suites' audits, two reviews of the plan, and forks. That is 412M in
  all. Counted, `tokens_per_step` would be 19.2M. jig-protocol's
  planning was not found, so no earlier run compares.
- **`tokens_claude`** is comparable with jig-protocol's 1,184M counted
  once, not with its metric. This run had no fork, so the metric counted
  each message once. The kit's double count across forks (jig-protocol's
  proposal 16) is still there.
- **`escalations`** counts a block once per section that lists it. Three
  were listed twice: the two window blocks and smith 03.4's. By content,
  the 21 were 11 plan defects, 5 cross-session scope, 4 process and 1
  design gap, which the supervisor ruled a fixture defect.
- **`gates_per_step`** counts every gate-lane task:
  - 65 land gates, of which 9 failed (14%, against jig-protocol's 32%);
  - 41 measurements;
  - 15 `smoke-tools` runs.

  A plan that measures each Proof has more of these by design. 17 of the
  65 land gates ran again after main moved; none failed on a timeout.
- **`suites`** prints the first landing's suites and the last's, here
  skein's and then jig's. By repository, focused, first landing → last:
  - skein: 2,087 / 8.234 s → 2,105 / 3.717 s;
  - smith: 1,996 / 16.453 s (the `measure` profile, no timeout) →
    1,995 / 10.089 s (its own gate);
  - jig: 3,813 / 11.713 s → 3,514 / 8.113 s.

  Fuzzy: skein 43.6 → 40.2 s, smith 23.2 → 14.4 s, jig 16.0 → 13.1 s.
- **`proofs`:** the metric finds a step's planned layer under "2.1",
  while the session files write "### 02.1", so it compared no step with
  its plan. Every step's Proof is at its increment's layer: the
  supervisor's hourly checks sampled them all.

## 4. What worked

1. **A plan built on measured causes.**
   - The planner's audit measured each cost before planning it
     (`findings/cost-*.md`, `measurements.md`).
   - Every suite reached the aim with room: 5.6, 9.0 and 4.7 s against 12.
   - Decision 5's fallback was not needed.
   - Implementers quoted the plan's "before" numbers instead of measuring
     again ("382 tests (502 at base)", jig-04.1; "0.474 s versus the
     audit's 1.74 s", smith-03.2).
   - Of 39 steps, only the window story's fix rested on a wrong diagnosis,
     and the audit had marked it "Likely, not verified" (section 5,
     item 9).
2. **Small, independent sessions, written against landed code.**
   - 14 sessions and 36 steps. The first seven started within 2.5
     minutes of the claim.
   - 15 landings in the first hour; `step_h` 0.09.
   - No size escalation. Implementer contexts were 142K at their median
     peak, and only three threads handed over, each between increments.
3. **Answers came fast, and the supervisor investigated before ruling.**
   - Every block was answered within 3 minutes.
   - A read-only sub-agent found the window story's cause in 4 minutes
     (00:26:59Z to 00:30:55Z): the person's echoed `Again` landing before
     the status line's newline. The 80 + 5 columns pointed at it, and the
     fix was one line of the test.
   - Another checked that skein-01.3's TLS change left no exchange
     waiting out its deadline (01:18:49Z, "safe").
4. **The coordinator kept the line moving.**
   - It started dependent sessions 41 to 146 s after their last
     prerequisite reported, and no ready session waited for a slot.
   - It checked every one of the 39 landings by ancestry and by trailer.
   - Its waits followed section 7.4: 9 empty wakes in 3.7 hours.
5. **The repins held.**
   - Every `--precise` move was checked with `merge-base --is-ancestor`
     first, and no pin moved back.
   - smith and jig ended one commit behind their upstreams, a STATUS
     commit in each case.
   - Every regenerated family kept its golden bytes.
6. **Two plans landed in smith at once without a conflict.**
   - The reliability run landed six commits in smith and one in skein
     during this run, from debian-16gb-hel1-1. Their files did not
     overlap.
   - The supervisor told the user when skein-01.1 landed, as the README
     asked (23:40Z), with the two `Read` arms the reliability run would
     need.
7. **smoke-tools stayed cheap.** 13 live runs passed, against a cap of
   30: one for each smith landing, except smith-04.1, which changed only
   its workflow.
8. **"Done when" was a list of commands.** The supervisor checked items 2
   to 5 on the three `origin/main`s in two minutes, and `done.md` names
   the evidence for the rest.
9. **The designer cost nothing.** It joined, ended its turn and was never
   woken: 0.3M tokens, against 514M of wakes in jig-protocol. Its
   jig-protocol cost came from questions, not from waiting, and this run
   had none.

## 5. What did not

1. **The last retrospective's proposals were never decided.**
   - **What:** none of jig-protocol's 17 proposals was adopted.
     - skein's development.md last changed on 2026-10-12, before
       jig-protocol ran.
     - The kit has changed only by this plan's skein-02.1.
     - This plan cites none of the proposals.
   - **Cause:**
     - The user asked for this plan at 20:53Z, while the retrospective
       was being written; it landed at 21:22Z.
     - development.md says the user decides and the designer applies.
       It does not say this must happen before the next run.
   - **What recurred:**
     - trailers git cannot read (its proposal 17);
     - the supervisor's context without a handover (1);
     - gates run again after main moved (4);
     - empty polls (8);
     - re-typed reports and a compaction in the coordinator (10);
     - metrics that miscount (16).
   - **First visible:** at 22:53Z, when the plan landed. **Acted on:** not.
2. **Plan defects: a step's Files missed what its change reaches.**
   - **What:** 11 rulings, 0.28 a step, against 0.30 and 0.38 in the two
     earlier runs. Nine of them were users or copies of what the step
     changed. Three of the nine (115, 117 and 118) came back because a
     ruling had searched too narrowly (item 4):
     - the drift test over all four codec families (101);
     - a worst-case literal in `replay_total.rs` (106), and a read-shape
       row the Shape itself changed (107);
     - a heap meter that counted a fixture cached by the step (113);
     - jig's four copies of smith's goldens and the literals derived from
       their sizes (114 to 118);
     - the one-process world's cuts, placed by a count of effects that
       the plan's own aim lowered (119).
   - **The other two were Shapes not checked against the code:**
     - a lazy trace over types without `Clone` (103);
     - a whole-transcript digest past its test's ceiling (105).
   - **Cause:**
     - The audit measured what each change would save.
     - It did not list who reads, copies or counts the thing each change
       moves.
     - One search for the thing's name, its copies and its users would
       have found each of the nine.
   - **First visible:** 23:15Z (ruling 101). **Acted on:** each in its
     ruling. The supervisor wrote the general lesson into the user's
     auto-memory at 01:52Z, not into a document.
3. **The generator's new shape was proven only in skein.**
   - **What:**
     - skein-02.2 made golden tables of up to 1,024 cases.
     - smith's regenerated `golden_0` then failed clippy's
       `too_many_lines` (233 and 445 lines against 100). That became
       skein-02.s1, ruling 110, at 00:41Z.
     - skein-02.s1's `const` items failed jig's
       `clippy::large_const_arrays`. That became skein-02.s2, ruling 111,
       at 01:12Z.
     - Ruling 110's fallback, "if clippy objects … emit a `static`",
       could not fire in skein, whose tables are small.
     - Both steps landed on the longest chain. smith 01.1 sat idle from
       00:39Z to 00:50Z, and jig 05.1 from 01:10Z to 01:21Z.
   - **Cause:**
     - A change to generated code was gated where it was made.
     - Its effect shows in the families downstream, which nobody
       regenerated before the step landed.
     - `plan downstream` type-checks, but it neither regenerates nor runs
       clippy.
   - **First visible:** 23:54Z. skein-02.2's own report had already
     worked around the line limit in skein's sample family.
     **Acted on:** 00:41Z and 01:12Z.
4. **Rulings closed more than their evidence allowed.**
   - **What:** three answers were partly reversed. Each time the
     implementer had to stop again for something the first ruling should
     have covered.
     - **106 → 107:** "Any other expected value that changes is a
       question" barred a row that 01.3's own Shape changes.
     - **114 → 115:** 114 listed the literals equal to 127 and said "Do
       not copy a second one". The session's own scope note at 01:36:04Z
       had already named the smallest golden and the bounds 126 and 128.
     - **115 → 117:** 115 still searched by value. It missed `FULL_TURN`,
       a byte-literal copy, and `pages.rs:230`.
     - Ruling 118 (01:58Z) finally compared every one of smith's 720
       goldens with jig's sources and found four copies in all.
   - **Effect:** jig 05.1 took an hour, with four failed gates, on the
     critical path. Had it landed at its first gate, jig-repin could have
     started at about 01:53Z, after smith-04.1, rather than at 02:12Z.
   - **Cause:**
     - Each ruling searched for the value it was shown, not the thing's
       name, copies and users.
     - Each closed with a prohibition rather than a rule the implementer
       could apply ("refresh any further copy and list it", ruling 117).
     - The jig-protocol supervisor's memory note, "grep main for every
       symbol a new step names", was loaded but covered symbols, not
       fixtures.
   - **First visible:** 00:24Z (block 7). **Acted on:** 01:51Z (117's
     closing rule) and 01:52Z (the memory note).
5. **The gate lane was saturated in the first hour.**
   - **What:**
     - Seven sessions started at once, four of them in jig.
     - The 88 queued tasks that ended before 00:00Z waited 14,653 s and
       ran 3,759 s.
     - jig 01 to 04 ran 25 land gates for 11 landings, 14 of them again
       because another jig landing had moved main.
     - jig-02.2's first `land` gave up after main moved five times
       (exit 6).
     - The supervisor's 00:15Z check: "31 gates for 15 landings".
   - **Cause:**
     - A gate runs alone, so landings in one repository are serial
       anyway.
     - `land` gates first and pushes after. When another code landing
       moves main in between, the passed gate is thrown away.
     - development.md's open question 13 names this. Nothing has
       changed since.
   - **First visible:** 23:18Z (the first check: 1,129 s waiting against
     427 s running). **Acted on:** not. The supervisor chose not to
     change the tool mid-run, and the ratio fell to 0.68 by 01Z as
     sessions finished.
6. **Implementers' tokens went to waiting and to reading.**
   - **Empty polls:**
     - They took about 74M of the implementers' 202M input. In
       jig-protocol they took 15.7%.
     - Codex's `wait` returns after at most 60 s, so section 8.3's "Wait
       in one call" cannot be followed. A minute in the queue costs a
       request at the thread's whole context.
     - The threads waited about 416 minutes in the queue in all.
     - Some waited 1 s or 10 s at a time.
   - **Reading:**
     - Every thread read 80K to 125K of documents and code before its
       first edit, development.md often more than once, from its start
       or from section 8 to its end.
     - The global AGENTS.md sent them to the code graph's
       `list_projects`.
     - Concatenated `cat`s overflowed the tool's output cap two to eleven
       times a thread, and the cut-off text was read again.
     - Fresh threads after a handover read about 80K again before using
       their checkpoint.
   - **First visible:** 00:15Z (6.6M input tokens a landing).
     **Acted on:** not.
7. **Gate-lane runs failed in seconds after long waits, for the
   implementer's own reasons.**
   - **What:** five runs, 24 minutes of waiting:
     - two `smoke-tools` runs refused as "source repository is dirty"
       (smith 03.1, after 278 s and 365 s);
     - a timing run on an uncommitted edit (smith-04.1, ruling 116);
     - a measurement that did not compile (jig-03.1, after 262 s);
     - a gate failing clippy that the session's own clippy had already
       failed (skein-01.2, after 524 s).
   - **Cause:** nothing says a tree must be committed, clean, built and
     clippy-clean before it enters the gate lane. smith's `smoke-tools`
     and smith-bench refuse a dirty source, but only after the wait.
   - **First visible:** 23:23Z. **Acted on:** 00:18Z (steer 2, for
     `smoke-tools`) and 01:47Z (ruling 116, for timing runs).
8. **A commit's trailers were unreadable again.**
   - **What:**
     - jig-02.1 (`2497c8299`, 23:22Z) put `Proof log:` and `Count test:`
       among its trailers. Their keys hold a space, so git reads none of
       the paragraph, and `land` started a new one for its `Gate-*`
       lines.
     - skein-02.1 (`375c688`) put a blank line above `Plan:`.
     - jig-10.1 did the same in jig-protocol, whose proposal 17 asked
       `land` to refuse it.
   - **Effect:** `plan status` will list jig-02.1 as not on main for
     good, and the metrics read 37 Proof lines of 39.
   - **First visible:** 23:18Z (the pulse, steer 1). That was too late
     for jig-02.1, already committed. **Acted on:** 23:25Z (ruling 102,
     a README rule). No later commit repeated it.
9. **The plan's fix of the window story was a wrong diagnosis.**
   - **What:**
     - smith-02.1 sized the terminal before the binary started.
       `findings/cost-smith.md` had proposed that fix under "Likely, not
       verified".
     - The story then failed two gates in a row under load (00:22Z and
       00:23Z), with `[85, 20]` against `[80, 20]`.
     - Ruling 108 allowed a retry. Ruling 109 found the echo race, and
       smith-02.s1 fixed the test at 00:43Z.
   - **Cost:** two failed gates, two escalations, one added step, about
     20 minutes of smith 02 and 03.
   - **Cause:** a finding that said it was not verified became a step's
     Shape without being verified.
   - **First visible:** 00:22Z. **Acted on:** 00:43Z.
10. **The coordinator spent its output re-typing reports.**
    - **What:**
      - Section 7.3 says to append each report to `log.md` "as
        received". In Codex a report exists only in the coordinator's
        context, so it generated each one again.
      - That was 42% of its output, about 27 minutes of model time, and
        half its context's growth.
      - About 11 sessions sent a "landed" report and then a "done" report
        for the same increment (8.6), and both were re-typed.
      - Its context reached 368K and compacted at 02:25:49Z. With the
        unblock's handling, that kept ruling 119's answer from jig-repin
        for about four minutes. It also lost the convention that put the
        thread id in the spawn prompt.
    - **Cause:** no `plan log`, and no handover for the coordinator
      (jig-protocol's proposal 10). **First visible:** from the first
      report. **Acted on:** not.
11. **The supervisor ran to 425K in one session.**
    - **What:**
      - Its context went from 43K at 23:09Z to 425K at the end.
      - It passed 300K at 01:16Z. 96 requests above 300K used 34.9M,
        55% of its input.
      - Each ruling was written four or five times: in `decisions.md`,
        the session file, `unblock.md`, `supervision.md` and the
        terminal.
    - **Effect:** for a 3.7-hour run this was bearable. In jig-protocol's
      29.8 hours, the same growth put 67% of a supervisor's input above
      500K.
    - **Cause:** jig-protocol's proposal 1, not adopted. **First
      visible:** 01:16Z. **Acted on:** not.
12. **A design gap was ruled without the designer.**
    - **What:**
      - jig-repin filed "mid-run cut still recovers Refused" as a design
        gap (02:32Z).
      - The supervisor ruled it a fixture defect (120), which let the
        stories pass. It put what recovery records after a clean exit in
        `later.md`, "unchecked against jig's design … rather than waking
        the designer, since nothing in the run waits on it".
    - **Cause:** section 10.4 gives design gaps to the designer, but
      offers no way to let the step go on while the question waits.
    - **First visible:** 02:34Z. **Acted on:** the `later.md` row.
13. **The record has small gaps:**
    - **Session starts:**
      - The coordinator logged only the first seven.
      - `spawn_agent` returned `/root/wN`, not the thread's id, so the
        ledger's thread column was filled in from the reports.
      - It also started a thread no step names, w11, at 00:39Z, to run
        skein 01's "Done when" test. Section 7.3 says such work is a
        plan defect for `blocked.md`.
    - **Rulings:** ruling 112's entry missed `decisions.md` in its first
      commit, when a Python anchor failed and the commit ran anyway.
    - **This retrospective's own cost:** it ran six readers, and its
      tokens appear in `metrics.tsv` as "after". When `metrics.tsv` was
      written they were 66.7M, more than the supervisor's 63.0M for the
      whole run.

## 6. Decisions taken during the run

Rulings 101 to 121 and four steers. The supervisor marked each in
`done.md` "not in the design yet", with the document it would belong to.

| Rulings | What | Belongs in | Proposal |
|---|---|---|---|
| 101, 110, 111 | The generator's families regenerated whole; table cases as `static` items outside the test function | skein's `codec.md`, 6 ("generated tests"), which names table tests but not where their cases live | 3; for the designer |
| 102 | A commit's last paragraph is its trailers | development.md, 8.4, and `land` | 7 |
| 104 | The coordinator checks a landed report on the landed branch only | development.md, 7.3 against 8.6 | 8 |
| 116, steer 2 | Timing and live runs from a committed, clean tree | development.md, 8.3; smith's `workflow.md`, 5 | 5 |
| 106, 107, 113, 114, 115, 117, 118, 119 | A step's Files widened to the users, copies and derived literals of what it changes | The session files, amended in the run; development.md, 5.4 and 5.7 | 2 |
| 103 | smith's run-world trace stays eager | jig's `workflow.md`, 2 ("Traces on failure"), with its `later.md` row | Section 9 |
| 105 | A large transcript named by its length and a bounded fingerprint | Test kit only | Section 9 |
| 108, 109, steer 3 | The window story's echo race, fixed in the test | Test only; smith's `shell.md` already says the terminal echoes | 13 |
| 112 | A cue spanning two output pieces replaced | Test only | — |
| 120, 121 | The one-process fixture keeps its worker alive past a turn; one more trace | Test only. The `Refused` recovery is a `later.md` row for the designer | 12; section 9 |
| Steers 1, 4 | The Proof trailer's shape; ruling 108's retry ended | Expired | — |

**Decided by the user during the run:** nothing; nothing was asked.

**Still with the user** (`decisions.md`, "For the user"):
- decision 9, the veto on a benchmark probe for smith's readers;
- decision 11, smith's suite here against the reliability run there. The
  planner and the supervisor (01:59Z) recommend option (a): the
  reliability run holds smith's focused suite to about 7 s on its
  machine.

## 7. The proposals adopted after the last retrospective

None. jig-protocol's 17 proposals were not decided before this run
(section 5, item 1). A few reached the run another way:
- **Its proposal 6's 1.5× Size margin:** carried into this plan's
  code-organization.md.
- **Its proposal 13's "a repin never moves a pin back":** carried in as a
  README rule, not a `land` check.
- **Its proposal 17's trailer check:** reached the README as ruling 102,
  after the defect recurred.
- **Its proposal 2's "grep main for every symbol":** lived in the
  supervisor's auto-memory, not in a document.

Where each proposal's metric stands, though it was not adopted:

| Proposal | Meant to move | jig-protocol | test-budgets | Moved |
|---|---|---|---|---|
| 1. Supervisor and designer hand over | `tokens_claude`, `answers_reversed` | 5.46M a step counted once; 0.17 | 1.61M; 0.08 | Down, but because no designer was woken and the run was short. The supervisor still ran to 425K. |
| 2. Check each session file against `main` before it starts | `escalations` (plan defect) | 0.30 a step | 0.28 | No. |
| 3. At most about 60 steps a plan | `step_h`, `contexts` | 0.14; 238K | 0.09; 142K | Down; this plan had 36 steps by its scope. |
| 4. `land` builds outside the gate lane | `queue_wait_ratio` | 2.24 | 2.13 | No. |
| 5. The card takes the general rules on edits outside Files | scope and process escalations | 0.21, 0.05 | 0.13, 0.10 | Mixed. |
| 6. Price increments; land within 1.5× Size | size escalations | 17 | 0 | Yes, with the margin carried in the plan. |
| 7. `plan check` times the focused suite | gate timeouts | 55 | 0 | This plan's aim, not the check. |
| 8. One long wait, no polls | `tokens_implementers` | 15.7% polls | about 36% | Worse. The tool's 60 s limit makes the rule impossible. |
| 9. An implementer hands over inside an increment past 250K | `contexts` | 319K p90 | 209K p90 | Down, from smaller increments. |
| 10. The coordinator hands over; `plan log`, `plan wait` | `tokens_coordinator`, `compactions` | 3.22M a step; 9 | 1.84M; 1 | Down per step. The compaction and the re-typing remain. |
| 11. Codex roles start without the graph's project list | tokens | 5.8% | still called | No. |
| 12. Increments that do not build on the one before are marked | tail `step_h` | — | the tail was the dependency chain | Not tested. |
| 13. `plan pulse` runs `plan downstream`; `land` refuses a pin moved back | `pin_lag` | — | 1 commit behind; `plan downstream` never ran | Held by the plan's rule. |
| 14. Questions for the user pushed at once | `blocked_h` | 8.2 h | 0.6 h | No question was asked. |
| 15. `plan pulse` prints tokens by role | `tokens_per_step` | — | the supervisor counted them by hand | No. |
| 16. `plan metrics` counts what 15.2 says | comparability | — | escalations double-counted, `proofs` unmatched, planning absent | No. |
| 17. The kit keeps the record true | — | jig-10.1 | jig-02.1 | No. |

## 8. Proposals

Each gives a rule in one sentence, its evidence, the document it changes,
and the metric it should move. They are ordered by expected effect.

1. **A plan's run starts only once the user has decided each proposal of
   the last retrospective, with the decisions written into that
   retrospective, where `plan check` looks for them.**
   - **Evidence:** none of jig-protocol's 17 proposals was decided. Six
     of its items recurred here (section 5, item 1). This plan was asked
     for 29 minutes before that retrospective landed.
   - **Changes:** development.md, sections 6 and 15.1; `plan check`.
   - **Moves:** the next retrospective's section 7, from none adopted to
     a decision on each, and through it the metrics of the six items that
     recurred.
2. **Whoever writes a step or a ruling that changes a thing (a
   generator's output, a fixture, a type, a literal, a count) names, in
   Files or in the ruling, every user of it found by searching its name,
   its copies by content and its derived values in every repository
   downstream.**
   - **Evidence:**
     - Nine of the 11 plan-defect rulings were users or copies the plan
       missed (section 5, item 2).
     - Three reversed answers searched for a value (item 4).
     - Ruling 118's script, which compares every upstream golden with
       the downstream sources, found in one pass what three rulings had
       missed.
   - **Changes:** development.md, 5.4 (the planner's checks) and 10.3
     (the supervisor's rulings), taking in the supervisor's auto-memory
     note.
   - **Moves:** `escalations` (plan defect) down from 0.28 a step;
     `answers_reversed` down.
3. **A step that changes generated code regenerates every downstream
   family in a scratch worktree and runs that repository's clippy on it
   before the step lands.**
   - **Evidence:** skein-02.s1 and 02.s2, two added steps on the longest
     chain, with 22 minutes of idle sessions (section 5, item 3).
   - **Changes:** development.md, 5.7; `plan downstream`, which could
     take a `--regenerate`.
   - **Moves:** `steps_added` and `escalations` (cross-session scope)
     down.
4. **`land` holds a per-repository lock from its rebase to its push, so
   a gate always runs on the base it lands on.**
   - **Evidence:**
     - 17 of 65 land gates ran again after main moved, 14 of them in
       jig.
     - jig-02.2's `land` gave up once.
     - Gates run one at a time anyway, so a lock costs no parallelism
       (section 5, item 5).
     - It answers development.md's open question 13 more simply than a
       single lander.
   - **Changes:** development.md, 9.1 and 13; `land`.
   - **Moves:** land gates per landing down from 1.67 to about 1.2, one
     per landing plus the failed ones; `queue_wait_ratio` and
     `tokens_implementers` down.
5. **Nothing enters the gate lane from a tree that is not committed,
   clean, built and clippy-clean.**
   - **Evidence:** five gate-lane runs failed within seconds after 24
     minutes of waiting in all: two dirty `smoke-tools` runs, a dirty
     timing run, an uncompiled measurement and a known clippy failure
     (section 5, item 7). Steer 2 and ruling 116 each covered part of
     it.
   - **Changes:** development.md, 8.1 and 8.3; `heavy --gate` could check
     `git status --porcelain` itself.
   - **Moves:** failed gate-lane runs down; `queue_wait_ratio` down.
6. **The spawn prompt tells an implementer what to read and how to
   wait.**
   - **Read:**
     - section 8 and its session file, the plan's README and
       code-organization.md, its repository's AGENTS.md and workflow,
       never development.md whole;
     - no code-graph project list;
     - after a handover, the checkpoint first.
   - **Wait:** with the tool's longest wait, 60 s, and nothing between.
   - **Evidence:**
     - 80K to 125K read before the first edit.
     - Empty polls at about 36% of the implementers' input.
     - Waits of 1 s and 10 s (section 5, item 6).
     - Section 8.3's "Wait in one call" assumes a tool that does not
       exist.
   - **Changes:** development.md, 7.2 and 8.3; Codex's global
     `~/.codex/AGENTS.md`, whose code-graph section starts every role
     with `list_projects`.
   - **Moves:** `tokens_implementers` and `contexts` down.
7. **`land` refuses a tip whose message names `Step:` but whose trailers
   git does not read.** This is jig-protocol's proposal 17, again.
   - **Evidence:** jig-10.1, jig-02.1 and skein-02.1's Proof (section 5,
     item 8).
   - **Changes:** `land`; development.md, 8.4.
   - **Moves:** `proofs` to count every step; no permanent `plan status`
     disagreement.
8. **Section 7.3 checks a landed report by its branch's tip, not by the
   worktree, which section 8.6 lets move on to the next increment.**
   - **Evidence:** ruling 104.
   - **Changes:** development.md, 7.3.
   - **Moves:** `escalations` (process) down.
9. **Implementers append their own reports with a `plan log`, and the
   coordinator reads them there and hands over at about 250K.** This is
   jig-protocol's proposal 10, narrowed.
   - **Evidence:**
     - Re-typed reports took 42% of the coordinator's output and half its
       context's growth.
     - The compaction at 368K stalled the end chain (section 5, item 10).
   - **Changes:** development.md, 7.3, 7.5 and 8.6; `plan`.
   - **Moves:** `tokens_coordinator` and `compactions` down.
10. **The supervisor hands over to a fresh session from a short brief
    when its context passes 300K.** This is jig-protocol's proposal 1,
    its first half.
    - **Evidence:** 425K at the end, and 55% of its input above 300K, in
      only 3.7 hours (section 5, item 11).
    - **Changes:** development.md, 2.5; a `plan handover supervisor`.
    - **Moves:** `tokens_claude` down.
11. **`plan metrics` counts what section 15.2 says:**
    - the planning session, named by a `Planner-Session:` trailer on the
      plan's commit;
    - escalations once each;
    - Proof layers against "### NN.M" headings;
    - suites per repository;
    - land gates apart from measurements and live runs.
    - **Evidence:** 412M of planning uncounted, three double-counted
      escalations, "0 below plan" from no comparison, and `suites` mixing
      skein and jig (section 3).
    - **Changes:** the kit; development.md, 15.2.
    - **Moves:** comparability. The planning share becomes visible, so a
      later plan can budget its audit.
12. **A block filed as a design gap goes to the designer, even when the
    supervisor lets the step go on under a ruling meanwhile.**
    - **Evidence:** ruling 120 left a recovery behaviour "unchecked
      against jig's design" in `later.md` (section 5, item 12).
    - **Changes:** development.md, 10.4.
    - **Moves:** `later.md` rows unchecked against the design down to
      none. The cost is one fresh designer wake, about 0.4M tokens.
13. **A finding the planner marks "not verified" is verified, a flaky
    test's race by reproducing or tracing it, before a step's Shape rests
    on it.**
    - **Evidence:** the audit's "Likely, not verified" fix became
      smith-02.1's Shape. Then came ruling 108's retry and smith-02.s1
      (section 5, item 9).
    - **Changes:** development.md, 5.5.
    - **Moves:** `steps_added` down.

## 9. Debts left in the code

For jig's and smith's `tech-debt.md`. skein has none yet, so its rows are
listed for the designer.

**jig:**
- **Four copies of smith's goldens:** the two transcript `.bin` files,
  `FULL_TURN` (`testing/jig-test-client-records/src/activity_reads.rs:4`)
  and `SMALLEST` (`testing/jig-test-worker/src/tests/fixtures.rs:62`),
  with literals derived from their sizes.
  - Every repin of smith refreshes them by hand (rulings 114 to 118).
  - Paid back by a drift test that compares each copy with smith's
    golden at the pinned revision, or by reading the golden from the
    pinned crate.
- **The one-process world's power-loss cut is placed by a count of
  effects** (window 126 to 130, cut 128).
  - Any change in how many steps a run takes moves it (rulings 119 to
    121).
  - Paid back by placing the cut on an event: the first committed `Turn`
    with a live worker.
- **What recovery records after a power loss between a worker's clean
  exit and its `Finished` commit** (`Refused`): not checked against the
  design (`later.md`, ruling 120).
- **From `later.md`:** the core world's observer rebuilds its snapshot at
  every commit, and the fleet world formats every event into its trace.
- **jig-02.1** (`2497c8299`) carries no trailers git can read. It will
  stay so, since a landed commit is never rewritten.

**smith:**
- **The run world's trace is formatted eagerly** (ruling 103, `later.md`):
  `run::Event` and `run::Request` have no `Clone`.
- **The agent world names a transcript over 1 KiB by its length and a
  hash of its first and last 512 bytes** (ruling 105). Two runs that
  differ only in a large payload's middle look the same in the trace.
- **smith's focused suite is at a 9.0 s median here, against the 12 s
  aim,** while the reliability plan adds tests gated on a machine about
  twice as fast. Decision 11 is open.

**skein:**
- **From `later.md`:**
  - `RequestTable`'s O(N) scans;
  - the event-stream and JSON readers' byte-at-a-time loops;
  - the simulator's lack of short receives on pipes;
  - the terminal's keys read one at a time;
  - the fuzzy suites, at 40 s of their 60 s here.
- **The generator's `codec.md`, 6 does not yet say** that a golden table's
  cases are `static` items outside the test function (rulings 110, 111).

**Workspace:** temper's pins are 264, 788 and 1,481 commits behind
skein's, smith's and jig's `main`. That is outside this plan (decision
12), and a plan of its own.
