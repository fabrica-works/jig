# client-composition: an application's own client, on jig's pages as a library

2026-10-14. Start commit: jig `2ae75cac2` ("Design: an application's own
client, on jig's pages as a library"), each session starting from jig's
`origin/main`. skein (jig's pin `74b7fdaf3`) and smith are read only.

jig's client pages become a library an application arranges: the tree's
closed sets are completed (owners as jig or an application's number,
landmarks, menus, disclosures, images, region names, shortcuts,
pictures, the act on a drawing); every page keeps one outline of
landmarks, which the referee checks; a page is its sections, each a
function of `jig-view` with a stable region name, which jig's pages
arrange and an application may arrange otherwise or show on its own
pages in slots; and jig gets a text shell for a terminal or a pipe.
`ops` takes each: its owners, its arranged task page, its lead's plan in
a section slot, its terminal client in its worlds.

## Inputs

- **jig's design,** at `2ae75cac2`: the rows of `docs/design/STATUS.md`
  that name client-composition (decisions.md, 1): `client/domain/`
  `README.md` 2 and 5, `client.md` 3, 4.1, 5.1, 9, 14 and 17,
  `contract.md` 1, 3.3, 5.3, 6, 7.1 and 7.2, `pages.md` 2, 9 and 12,
  `root.md` 2, 6, 7, 9 and 15, `testing.md` 2, 4, 6, 8 and 9, `view.md` 3,
  3.1, 4, 4.1, 5, 10 to 13 and 15; `README.md` 10 and 15; `examples.md` 8.
- **jig's workflow:** `docs/development/workflow.md`, sections 1 and 2
  (the gate; the 15 s and 60 s budgets).
- **skein's foundation:** `development.md` (this plan follows it),
  `programming-model.md` (section 10, the subset), `testing-strategy.md`.
- **`decisions.md`:** what this plan settled, and the long-term choices
  the user may veto.
- **`findings/`:** `measurements.md` (the suites' times at the base);
  `view.md` and `domain.md` (the code audits: every site the sessions
  change, with its line at the base).

## The sessions

A session waits for what "Needs at start" names; an increment that needs
more says so in its row and its file. Each increment also builds on the
ones before it in its session. Dependencies are by capability.

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `jig/01-tree.md` | jig 01: the tree's closed sets | nothing | |
| `jig/02-outline.md` | jig 02: the page's outline and the frame's parts | jig 01.2 (landmark roles), 01.3 (`Region`, `JigRegion`) | |
| `jig/03-sections.md` | jig 03: a page is its sections, at jig's addresses | nothing | |
| `jig/04-arrangement.md` | jig 04: the arrangement, in the domain | jig 03 (`Section`, `shows`, `open_section`), with the client's and joint worlds passing on `main` after it (03.2's Proof: the story that runs 03 against the real view and the engine's domain) | |
| `jig/05-section-views.md` | jig 05: the sections and headers as functions of jig's view | jig 02.3 (`pages::main`); jig 03.2 (`Section`, `Domain::shows`) | |
| `jig/06-arranged.md` | jig 06: more sections together, counted, and the owed cards | jig 05; jig 04.2 (an arranged page's sections) | 06.4: jig 04.3 (an arranged page opens what each section needs) |
| `jig/07-section-slots.md` | jig 07: an application's page shows jig's sections | jig 06 (the sections whole, the view's limits with an arrangement, the owed cards from the domain); jig 04.3 | |
| `jig/08-ops.md` | jig 08: `ops` arranges jig's pages and shows its lead's plan | jig 07 (`ShowSections`, the test application's section slot to copy) | |
| `jig/09-text-shell.md` | jig 09: jig's text shell prints a tree and reads a line | jig 02 (the outline, `jig-link`) | 09.3: jig 06 and 07.3 (jig's pages from sections, the family's section slot) |
| `jig/10-terminal.md` | jig 10: the terminal, and the worlds at the text shell | jig 09.1 and 09.2 (the story that runs them against the real view is 10.2's, in the client's world; 09.3's goldens wait for jig 06 and 07.3, which 10 does not need) | 10.4: jig 08.3 (the story on the incident's page) |

### How many at once, and in what order

- **At the start:** jig 01 and jig 03. The build lane runs 2 at once and
  the gate lane 1 on carbon-forge (development.md, 11.4).
- **Then,** as each lands: jig 02 once 01.3 has landed; jig 04 once 03
  has; jig 09 once 02 has (its 09.3 waits for 06 and 07.3); jig 05 once
  02.3 and 03.2 have; jig 06 once 05 and 04.2 have; jig 07 once 06 and
  04.3 have; jig 08 once 07 has; jig 10 once 09.2 has.
- **When slots are short, start first** the sessions others wait on, in
  this order: jig 03, jig 01, jig 02, jig 04, jig 05, jig 06, jig 07,
  jig 09, jig 08, jig 10.
- **The longest chain:** jig 03 → jig 04 → (with 01 → 02 → 05) jig 06 →
  jig 07 → jig 08 → jig 10.4.

## Rules of this plan

- **This plan runs no repin step; its run ends when the last jig session
  lands.** (decisions.md, 2.) The coordinator adds no repin step, and
  the supervisor adds none during the run.
- **Every session reads,** besides its "Read first": jig's `AGENTS.md`
  and `docs/development/workflow.md`, this README and
  `code-organization.md`.
- **Test time** (decisions.md, 8): the focused suite's median is 8.49 s
  at the base and the fuzzy suite's 13.96 s (findings/measurements.md).
  Each session's tests may add, as a guide, at most this to the focused
  suite's median: jig 01 0.2 s; 02 0.3 s; 03 0.2 s; 04 0.2 s; 05 0.2 s;
  06 0.4 s; 07 0.4 s; 08 0.4 s; 09 0.4 s; 10 0.8 s (3.5 s in all, to
  about 12 s); the fuzzy suite at most 6 s in all, most of it jig 02.4's
  check in every random world. Each landing's report names the focused
  and fuzzy times from its `Gate-Result` line, and nothing else is
  measured. A session's share is passed when its landing's focused time
  less the focused time of the last landing before the session's first
  exceeds the share by more than 1 s (one gate's run varies by about
  1 s, findings/measurements.md). A share passed is a guide missed:
  the landing stands, and the session's next increment, if it has one,
  makes its tests cheaper first (fixtures built once; `--profile
  measure -j 1` once, through `heavy --gate`, to find the cost); after
  a session's last increment, the report says so and the supervisor
  decides. Never: raising a `global-timeout`, a slow period or a
  per-test ceiling, or moving a focused test to the fuzzy suite
  (Done when 8).
- **Shared files,** which parallel sessions may both change (the
  coordinator expects rebases): `crates/jig-view/src/pages.rs` (jig 02,
  03.2, 05, 06); `crates/jig-domain-client/src/domain.rs` and
  `pages/routing.rs` (jig 03, 04, 06.3, 07); `tests/client/src/stories.rs`
  and `tests/client/tests/stories.rs` (jig 06, 07, 09.3, 10.2);
  `tests/client/src/harness/world.rs` (jig 02.4, 10.2);
  `testing/jig-test-view/src/root.rs` (jig 01.1, 02.3, 07.3);
  `examples/ops/view/src/root.rs` (jig 01.1, 02.3, 08).
- **jig 05 moves five of `crates/jig-view/src/pages/`'s files into
  `sections/`** with `git mv`: `task.rs`, `inbox.rs`,
  `conversations.rs`, `deployment.rs`, `project.rs`, with `task/` and
  `project/`. A session that rebases over the move and changed one of
  those files reapplies its change at the file's new path.
- **No name of a system in code** (jig's `AGENTS.md`), and no name of
  the repository downstream of jig in any file.

## Live services

None. Every proof runs in the suite, on skein's simulated machine at
most.

## Repin

None: decisions.md, 2. jig's only downstream repository keeps its pins
for a run active on another machine, and repinning it onto this plan's
jig needs more than a repin may change (development.md, 5.7); the user
takes care of it after this run.

## Pulse

```
words: legacy|compat|hack|workaround
types: ^(Legacy|Old|Compat)[A-Z]|V2$
file-lines: 800
budgets: 15 60
```

## Done when

1. Every step is landed or dropped: `plan status` lists none waiting,
   ready, running or blocked.
2. Owners are jig or an application's number: `git -C ~/src/rust/jig
   grep -n -E 'Owner::(Side|Other)\b' origin/main -- '*.rs'` prints
   nothing.
3. The referee checks the outline in every world that uses the client's
   harness: `git -C ~/src/rust/jig grep -n 'outline::check' origin/main
   -- tests/client/src/harness` prints a line.
4. A page is its sections, in the domain and in the view: `git -C
   ~/src/rust/jig grep -n -E 'pub enum Section\b|pub struct Arrangement\b'
   origin/main -- crates/jig-domain-client/src/pages.rs` prints both, and
   `git -C ~/src/rust/jig grep -n 'pub fn section(' origin/main --
   crates/jig-view/src/sections.rs` prints one line.
5. client.md 17's two new stories and the text shell's pass, once, on
   `main`, in a worktree at `origin/main`: `heavy cargo nextest run -p
   jig-client-world -p jig-ops-world -p jig-client-protocol-world -p
   jig-shell-text -E 'test(arranged) | test(a_family_page_shows_its_tasks_plan)
   | test(/at_the_text_shell|terminal|p_accepts/) | binary(text_pages) |
   package(jig-shell-text)'`.
6. The plan's commits: `git -C ~/src/rust/jig log origin/main
   --grep='^Plan: client-composition$' --format=%s` lists one or more
   commits for each session, jig 01 to 10.
7. The aim, about 12 s for the focused suite on carbon-forge (decisions.md,
   8): in a worktree at `origin/main`, `NEXTEST_DOUBLE_SPAWN=0 heavy
   --gate -- sh -c 'for i in 1 2 3; do cargo nextest run --workspace
   2>&1 | grep Summary; done'`, once: the median of the three `Summary`
   times. An aim: past 12.0 s the supervisor records the median, and
   the three slowest tests by name from one run of
   `NEXTEST_DOUBLE_SPAWN=0 heavy --gate cargo nextest run --workspace
   --profile measure -j 1` in the same worktree, in decisions.md's
   rulings, for the user; the run is done either way.
8. No budget was raised: `git -C ~/src/rust/jig diff 2ae75cac2
   origin/main -- .config/nextest.toml` prints nothing.
9. The aims stated above are these items; none other is checked.
