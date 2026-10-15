# skein-ui: skein's UI kit, extracted from jig and completed

2026-10-15. Start commits: skein `2f282f7` ("Design skein-ui: a UI kit
applications build on without a framework"), jig `5efac1af8` ("Build
jig's client presentation on skein-ui, mandating nothing"); each session
starts from its repository's `origin/main`. smith is read only, and jig
keeps its pin of smith, `29fa97c9` (decisions.md, 4).

skein gets a generic UI layer any portable application builds on:
`skein-ui` (the semantic tree, its vocabulary, owners, the builder,
bindings and the act refusal, the diff and the view, words, Markdown,
scenes, the outline, limits), the text shell `skein-ui-text`, the
browser parts `skein-ui-web` and their bundler `skein-ui-bundle`, the
fake person `skein-fake-person` and its browser face and harness
`skein-fake-person-browser`, and the UI example with its reference shell
and browser tier. The generic parts that exist are jig's today: jig 01
to 03 generalize them in place, skein 01 moves them as they are, and jig
04 takes them; skein 02 to 14 build what is new, and jig 05 takes it.

## Inputs

- **skein's design,** at `2f282f7`: `docs/design/ui.md`, `ui-text.md`,
  `ui-web.md`; `examples.md`, 9; `testing.md`, 6 and 7; `lib.md`, 2;
  `browser.md`, 5, 6, 9 and 10; `programming-model.md`, 2.1; the rows of
  `docs/design/STATUS.md` naming skein-ui.
- **jig's design,** at `5efac1af8`: `docs/design/STATUS.md`, "jig on
  skein-ui", step (1) and (2), and the sections of
  `client/domain/view.md` each jig session names.
- **jig's workflow:** `docs/development/workflow.md`, 1 and 2; **skein's
  gate:** skein's `AGENTS.md` and `testing.md`, 7.
- **`decisions.md`:** the plan's decisions, and what the user may veto.
- **`findings/`:** `audit.md` (the code audit at the base),
  `measurements.md` (the suites at the base), `toolchain.md` (the glue's
  versions, checked at the base).

## The sessions

A session waits for what "Needs at start" names; an increment that
needs more says so in its row and its file. Dependencies are by
capability.

### jig

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `jig/01-owner-and-core.md` | jig 01: the owner made opaque, the core gathered in `jig_view::ui` | nothing | |
| `jig/02-splits.md` | jig 02: binding, limits, outline and phrases split | jig 01.3 (`jig_view::ui`) | |
| `jig/03-shells.md` | jig 03: the builder's role assertion; the text shell and the fake person name only `jig_view::ui` | jig 01.3 | |
| `jig/04-on-skein-ui.md` | jig 04: jig takes skein-ui, skein-ui-text and skein-fake-person | skein 01.3 | |
| `jig/05-repin.md` | jig 05: jig on skein-ui's last landing | every skein increment, and jig 04.3 | |

### skein

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `skein/01-move.md` | skein 01: skein-ui, skein-ui-text and skein-fake-person, moved | jig 02.5 and jig 03.1 | 01.2: jig 03.2; 01.3: jig 03.3 |
| `skein/02-vocabulary.md` | skein 02: the vocabulary added now | skein 09.5 | |
| `skein/03-acts.md` | skein 03: the look's names, and the act refusal | skein 01.3 | 03.2: skein 02.4 |
| `skein/04-scenes.md` | skein 04: scenes, and skein-ui's worst case in `tests/ui` | skein 01.3 | |
| `skein/05-text-shell.md` | skein 05: the text shell whole | skein 01.3 | 05.2: skein 02.4; 05.3: skein 02.1, 02.2; 05.4: skein 02.5; 05.5: skein 02.3, 02.5 |
| `skein/06-person.md` | skein 06: the fake person whole | skein 01.3 | 06.2, 06.3: skein 03.2 |
| `skein/07-browser-kit.md` | skein 07: the browser kit's words in a name and press at a point | nothing | |
| `skein/08-bundler.md` | skein 08: the bundler | nothing | |
| `skein/09-page.md` | skein 09: the inbox, scenes as SVG, the mapping and the page part | skein 01.3 | 09.2: skein 03.1, 04.1; 09.3: skein 03.1 |
| `skein/10-example.md` | skein 10: the example's view and server, thin | skein 01.1 | 10.2: skein 08.5; 10.3: skein 01.2, 01.3 |
| `skein/11-first-story.md` | skein 11: the first story in Chromium | skein 07.2, 09.3 | 11.2: skein 10.2; 11.3: skein 09.5, 10.1; 11.4: skein 08.5 |
| `skein/12-transport.md` | skein 12: the small store, the transport, the add and the stream | skein 11.4 | |
| `skein/13-example-whole.md` | skein 13: the example whole | skein 12.3 | 13.2: skein 02.1, 02.4, 02.5; 13.3: skein 02.1, 02.3; 13.4: skein 04.2, 04.3, 06.3, 07.2 |
| `skein/14-stories.md` | skein 14: skein's stories for a browser shell, and the facts the face assumes | skein 11.4, 02.5 | 14.3: skein 12.3; 14.4: skein 12.4, 13.1 |

19 sessions, 70 increments: jig 15 in 5 sessions, skein 55 in 14.

### How many at once, and in what order

- **At most four sessions at once** on carbon-forge (8 threads, 15 GB;
  the build lane runs 2 tasks, the gate lane 1: development.md, 11.1).
- **At the start:** jig 01, skein 07 and skein 08.
- **Then,** as each lands: jig 02 and jig 03 once jig 01.3 has; skein
  01 once jig 02.5 and 03.1 have; skein 10 once skein 01.1 has; jig 04,
  skein 03, 04, 05, 06 and 09 once skein 01.3 has; skein 11 once 07.2
  and 09.3 have; skein 02 once 09.5 has; skein 12 once 11.4 has; skein
  14 once 11.4 and 02.5 have; skein 13 once 12.3 has; jig 05 last.
- **When slots are short, start first:** jig 01, jig 02, jig 03, skein
  01, skein 09, skein 02, skein 03, skein 04, skein 08, skein 10, skein
  07, skein 11, skein 12, skein 13, skein 14, skein 05, skein 06, jig
  04, jig 05. skein 03, 05 and 06 wait within their sessions for skein
  02's increments; skein 02 goes first so they wait least.
- **The longest chain,** 27 increments: jig 01.1 to 01.3 → jig 02.1 to
  02.5 → skein 01.1 to 01.3 → skein 09.1 to 09.5 → skein 11.3, 11.4 →
  skein 12.1 to 12.3 → skein 13.1 to 13.5 → jig 05.1. skein 02 runs
  beside skein 11 to 13: 01.3 → 09.1 to 09.5 → 02.1 to 02.5 reaches
  13.2's need of 02.5 one increment before 13.2 starts at the earliest.

## Rules of this plan

- **This plan runs no repin of smith or of the repository downstream of
  jig** (decisions.md, 3). The coordinator adds no repin step, and the
  supervisor adds none; jig repins skein inside jig 04 and jig 05.
- **jig moves skein only, never smith** (decisions.md, 4): `heavy cargo
  update -p skein-lib`, with jig 04's `--precise` to skein 01.3's
  landing (its file); a failure inside a smith crate is a question.
- **Nothing changes skein-ui, skein-ui-text or skein-fake-person
  between skein 01.1 and 01.3:** every other session that changes them
  needs skein 01.3, so jig 04's pin holds the moved code alone.
- **skein lands first:** a jig increment that needs a skein capability
  repins, in a commit of its own, after that capability is on skein's
  `origin/main` (development.md, 9.5).
- **Every session reads,** besides its "Read first": its repository's
  `AGENTS.md` (and jig's `docs/development/workflow.md`), this README
  and `code-organization.md`.
- **No name of jig in skein's code, and no name of the repository
  downstream of jig in any file:** `rg -n -i 'jig' crates/skein-ui*
  testing/skein-fake-person* examples/ui tests/ui*` in skein prints
  nothing at each skein increment's end.
- **The browser suite is a suite of its own,** outside the focused and
  fuzzy suites: `--profile browser`, Chromium only there.
- **Optimized builds** only for `skein-ui-bundle`, a tool the cache
  absorbs (testing-strategy.md, 8), and skein 13.5's one build of the
  example's module with the `web` profile, outside the gate; the
  browser tier's module is built with the dev profile.
- **A browser story's Proof** is `heavy --gate scripts/check.sh` once in
  the increment's committed worktree, which builds the bundle and the
  server and runs the browser suite; nothing else is run for it.
- **Test time** (findings/measurements.md, on carbon-forge at the base):
  - skein: focused median 6.75 s, fuzzy 45.88 s, browser 3.27 s. The
    run may add, as aims, at most 3.0 s to the focused suite (to about
    9.8 s), 3.0 s to the fuzzy suite, and 60 s to the browser suite (the
    example's tier, ui-web.md, 14.1), whose budget stays two minutes.
  - jig: focused median 10.22 s, fuzzy 15.75 s. The run may add at
    most 1.3 s to either (one gate's swing): jig's generic tests move
    to skein, and jig 01 to 05 add few.
  - Each landing's report names its suites' times from its
    `Gate-Result` line, and nothing else is measured. An aim exceeded
    is a guide missed: the landing stands, and the session's next
    increment first makes the slowest tests cheaper, found by `heavy
    --gate -- cargo nextest run --workspace --profile measure -j 1`
    once, until its own gate's time is back under the aim; after a
    session's last increment the supervisor decides. Never: raising a
    `global-timeout`, a slow period or a per-test ceiling, or moving a
    focused test to the fuzzy suite.
  - **A test's own time** in a session file ("under 20 ms", "under
    0.5 s", "about 3 s") is an aim, development.md 5.6's ceiling: nothing
    measures one test, and a test the gate's log flags `SLOW` is made
    cheaper before its increment lands.
- **Shared files,** which parallel sessions may both change: skein's
  `Cargo.toml` (each session that adds a crate or a dependency: skein
  01, 04.4, 05.1, 08.1, 08.5, 09.1, 10.1, 10.2, 11.1, 11.3, 13.5);
  `crates/skein-ui/src/{tree.rs, builder.rs, diff.rs}` (skein 02, 03,
  04.1); `crates/skein-ui/src/limits.rs` (skein 04.1, 04.4);
  `crates/skein-ui-text/src/render.rs` (skein 02.4, 02.5, 05);
  `crates/skein-ui-web/src/{page.rs, page/web.rs, look.rs, mapping.rs}`
  (skein 02, 09, 13.1, 13.4, 13.5, 14.1); `crates/skein-ui-web/src/
  platform.rs` (skein 11.3, 12.1, 12.2); `scripts/check.sh` (skein
  11.4, 13.1);
  `.config/nextest.toml` (skein 11.2); `examples/ui/view/src/{state.rs,
  view.rs, places.rs}` (skein 10, 12, 13, 14); `examples/ui/server/src/
  routes.rs` (skein 10.2, 12.3, 12.4); `examples/ui/shell-web/src/lib.rs`
  (skein 11.3, 12, 13, 14.2); `tests/ui-web/src/lib.rs` (skein 10.2,
  11.2, 11.4, 14.1); `tests/ui-web/tests/browser_example.rs` (skein
  11.4, 12, 13); `testing/skein-fake-person-browser/src/harness.rs`
  (skein 11.2, 12.3, 13.3, 13.4, 14.4); in jig, `crates/jig-view/src/lib.rs` and
  `src/ui/mod.rs` (jig 01, 02, 03), `testing/jig-test-view/src/root.rs`
  and `examples/ops/view/src/root.rs` (jig 01.1, 01.3, 04.1).
- **Files over the plan's limit at the base** are not split by this
  plan: skein's `testing/skein-browser/src/browser.rs` (1,774 lines),
  to which skein 07 adds at most 60 lines. The pulse lists it while
  skein 07 runs; the supervisor reads that line as expected, and as a
  finding for any other file.

## Prerequisites the run checks at its start

The coordinator runs each once, before the first session, and writes
`blocked.md` (kind: process) on a failure:

1. `/usr/bin/chromium --version` prints `Chromium 1`, or
   `SKEIN_TEST_CHROMIUM` names a file (skein's browser suite).
2. `rustup target list --installed` lists `wasm32-unknown-unknown`.
3. crates.io answers for the glue: `cargo info
   wasm-bindgen-cli-support@0.2.126` prints its name
   (findings/toolchain.md).
4. `cargo nextest --version` prints a version.

## Live services

None. The browser tier runs a local Chromium against the example's
server on loopback.

## Repin

None for smith or the repository downstream of jig: decisions.md, 3.
smith's run is active on another machine and uses no crate this plan
adds; jig's downstream repository keeps its pins for its own active run,
and its repin onto this plan's jig renames the tree's types it uses, more
than a repin may change (development.md, 5.7). jig's repins of skein are
this plan's own, in jig 04 and jig 05.

## Pulse

```
words: legacy|compat|hack|workaround|actual|genuine
types: ^(Legacy|Old|Compat)[A-Z]|V2$
file-lines: 800
budgets: 15 60
```

## Done when

1. Every step is landed or dropped: `plan status` lists none waiting,
   ready, running or blocked.
2. skein's crates exist: `git -C ~/src/skein ls-tree --name-only
   origin/main crates/skein-ui crates/skein-ui-text crates/skein-ui-web
   crates/skein-ui-bundle testing/skein-fake-person
   testing/skein-fake-person-browser examples/ui/view examples/ui/server
   examples/ui/shell-web tests/ui tests/ui-text tests/ui-web | wc -l`
   prints 12.
3. jig's copies are gone and jig names skein's: `git -C ~/src/jig
   ls-tree origin/main crates/jig-shell-text testing/jig-fake-person
   crates/jig-view/src/ui` prints nothing, and `git -C ~/src/jig grep -c
   'skein_ui::' origin/main -- '*.rs'` prints counts.
4. jig pins skein's last landing of this plan: `git -C ~/src/jig show
   origin/main:Cargo.lock | grep -m1 'skein.git#'` names a commit that
   `git -C ~/src/skein log origin/main --grep='^Plan: skein-ui$'
   --format=%H -1` prints, or a later one.
5. The gate's wasm32 builds and browser suite: `git -C ~/src/skein grep
   -n -e wasm32-unknown-unknown -e 'profile browser' origin/main --
   scripts/check.sh` prints lines for each.
6. The browser tier passes once, on `main`, in a worktree of skein at
   `origin/main`: `heavy --gate scripts/check.sh` exits 0.
7. The plan's commits: `git -C ~/src/skein log origin/main
   --grep='^Plan: skein-ui$' --format=%s` and the same in jig list one
   or more commits for each session.
8. The test-time aims, measured once in a worktree of each repository at
   `origin/main`: `NEXTEST_DOUBLE_SPAWN=0 heavy --gate -- sh -c 'for i
   in 1 2 3; do cargo nextest run --workspace 2>&1 | grep Summary;
   cargo nextest run --workspace --profile fuzzy 2>&1 | grep Summary;
   done'`, and for skein the browser profile once more: the medians
   against the aims above. An aim missed is recorded with the three
   slowest tests by name in decisions.md's rulings, for the user; the
   run is done either way.
9. No budget was raised: `git -C ~/src/skein diff 2f282f7 origin/main --
   .config/nextest.toml | grep -E '^[-+].*(global-timeout|period)'` and
   `git -C ~/src/jig diff 5efac1af8 origin/main -- .config/nextest.toml`
   print nothing.
10. The aims stated above are these items; none other is checked.
