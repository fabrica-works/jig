# client-composition: code organization

The plan adds one crate, `jig-shell-text`, and changes `jig-view`,
`jig-domain-client`, jig's test application (`jig-test-domain-client`,
`jig-test-view`), the fake person, `ops`'s client root and view, and the
client's, the protocol's and `ops`'s worlds. This file names who owns
what, where tests go, and what no increment does.

## 1. Crates and modules

| Crate | Kind | What changes |
|---|---|---|
| `crates/jig-view` | step code | `tree.rs` (owners, roles, attributes), new `regions.rs`, new `outline.rs`, `frame.rs` (the parts), new `sections.rs` and `sections/` (moved from `pages/`), `pages.rs` (`heading`, `main`, `not_the_work`, `after_page`, `jig`), `binding.rs` (`Act::Drawing`), `limits.rs` and `sizes/` (companions, the arrangement) |
| `crates/jig-domain-client` | step code | `pages.rs` (`Section`, `Arrangement`, `MOST_SECTIONS`), `domain.rs` (`sections`, `shows`, `shown_*`, `configure_arrangement`, `owes`, `section_status`), `pages/routing.rs` (`open_section`), new `pages/shown.rs`, `watches.rs` (`Owner::Sections`), `boundary.rs` (`ShowSections`) |
| `crates/jig-shell-text` (new) | step code, `[lints] workspace = true` | `render.rs`, `command.rs`, `terminal.rs` (decisions.md, 14) |
| `testing/jig-fake-person` | test kit | the new roles; `Act::Drawing` accepted on drawings only |
| `testing/jig-test-domain-client`, `testing/jig-test-view` | test kit | owners; the family's main; `ShowSections`; the section slot; `configure_arrangement` |
| `examples/ops/domain-client`, `examples/ops/view` | step code | owners; its main; `ShowSections`; its arranged task page |
| `tests/client` (`jig-client-world`) | world | the outline in the referee; the arranged and section-slot stories; jig's pages as golden text; `at_text_shell` |
| `tests/client-protocol`, `tests/ops`, `tests/boundary` | worlds, checks | the terminal's story; `ops`'s stories; the text shell's dependencies |

## 2. Names

- **`Owner::Application { number }`,** "an application's owner" in prose;
  its number is "the owner number" (root.md, 15). Never "side" or
  "child" for an owner in new code.
- **Region names** are `JigRegion`'s variants in code and `jig-…` in
  prose and tests, exactly as pages.md 12.1 and 12.2 spell them.
- **A section** is `jig_domain_client::pages::Section`; the sections a
  task's document offers are `pages::task::Offered`, from
  `pages::task::offers` (jig 03.1's rename).
- **"Arranged"** names a page that shows the arrangement's sections
  beside the address's own; "jig's arrangement" is none beside them.
- **`Print`, `Command`, `Refusal`, `Terminal`, `Settings`** are the text
  shell's; a test of the worlds played at the text shell has
  `at_the_text_shell` in its name.

## 3. Shared interfaces and their owners

| Interface | Owner | Users |
|---|---|---|
| `jig_view::{Owner, Region, Picture, Shortcut, Modifier}`, `jig_view::regions::JigRegion`, `Act::Drawing` | jig 01 | every later session |
| `jig_view::outline::{check, Fault, Rule}` | jig 02.1 | jig 02.3, 02.4, 08.2 |
| `jig_view::frame::{link, needs, party, projects, go_to_task, navigation, banner}` | jig 02.2 | `pages::jig`, `ops`'s view root (jig 08.2) |
| `jig_view::pages::{heading, main, not_the_work}` | jig 02.3 | jig 05, 06, 08.2 |
| `jig_domain_client::pages::Section`, `MOST_SECTIONS`, `Domain::{sections, shows, shown_task, shown_project, shown_inbox}` | jig 03 | jig 04 to 08 |
| `pages::routing::open_section` | jig 03.4 | jig 04.3, 07.1 |
| `jig_domain_client::pages::Arrangement`, `Domain::configure_arrangement`, the roots' pass-throughs | jig 04.1 | jig 06, 08.2 |
| `jig_view::sections::{section, inbox::header, task::{header, tabs}, project::{header, tabs}}` | jig 05 | jig 06, 07.3, 08 |
| `jig_view::Limits::{of, compose}` with `&Arrangement` | jig 06.2 | every caller (findings/view.md, 6), jig 07.3, 08 |
| `Domain::owes` | jig 06.3 | `after_page`, the sections' card sites |
| `Event::ShowSections`, `watches::Owner::Sections`, `SectionStatus`, `Domain::section_status` | jig 07.1, 07.2 | jig 07.3, 08.1 |
| `jig_shell_text::{render, Print, worst_case, command, Command, Refusal, refusal_line}` | jig 09 | jig 10 |
| `jig_shell_text::{Terminal, Settings}` | jig 10.1 | jig 10.3 |
| `World::at_text_shell` | jig 10.2 | jig 10.4 |

## 4. Tests and fixtures

- **Fixtures, one module per crate,** extended, never redefined:
  `crates/jig-view/src/tests/` (`cards.rs`'s `start`, `drive`,
  `document`, `CLIENT`, `limits`; `tests/pages.rs`'s `build` and `deliver`);
  `crates/jig-domain-client/src/tests/spine.rs` (`LIMITS`, `start`,
  `drive`, `delivered`, `frame`); `testing/jig-fake-person/src/tests.rs`
  (`build`, `binding`); `tests/client/src/stories.rs` (the worlds) and
  `tests/client/src/test.rs` (the constructors); `tests/ops/src/client/stories.rs`;
  `crates/jig-shell-text/src/tests/fixtures.rs`, made by jig 09.1.
- **Ceilings** (development.md, 5.6): aim for a unit test under 20 ms
  and a focused world test under 0.5 s. Nothing measures a test alone
  but nextest's slow period (500 ms focused, 5 s fuzzy,
  `.config/nextest.toml`): a test the gate flags SLOW is made cheaper
  before the session's next landing; under the flag, a ceiling is a
  guide.
- **Golden text** (decisions.md, 9): `crates/jig-shell-text/tests/golden/`
  and `tests/client/tests/golden/`, changed only by hand.
- **Literal bounds:** `crates/jig-view/src/tests/page_capacity.rs:845-853`
  holds each page's bound as a literal; an increment that adds nodes to
  a page changes them by what it adds, and its commit body names the
  change and its reason.
- **A test keeps its name,** unless its session file renames it.

## 5. The plan's "Don't" list

No increment of this plan:

- adds a type beside an existing one for the same thing: it changes the
  type in place, and its users in the same increment;
- adds a field or predicate that its counterpart across a boundary lacks
  (`ShowSections` and its roots' requests carry the same list);
- adds a test that checks several behaviours under one name;
- adds a lint exception in tests;
- computes an expected value with the code under test;
- adds a `_ =>` arm to a match on `Role`, `Owner`, `Act`, `Section`,
  `TaskSection`, `ProjectSection`, `Rule` or `Command`;
- builds a node for one of jig's objects or sections outside
  `jig_view::cards::card` and `jig_view::sections::section` (objects.md,
  2: a card is whole wherever it shows);
- has the fake person, a story or the person's steps find a node by
  region name, style or position (testing.md, 3);
- prints, styles or places anything by region name or shortcut in
  `jig-view` or the text shell, but the renderer's first line, which
  finds the link's text in the region `jig-link` (view.md, 15.3);
- renames `Part`'s field `connector`, `jig-ops-domain-connectors`, or
  the client link's `family` (decisions.md, 5);
- builds optimized a crate that ships, a world or a kit;
- raises a `global-timeout`, a slow period or a per-test ceiling, or
  moves a focused test to the fuzzy suite;
- writes a golden file by a test or a script. A golden file is the one
  expected value that may be drafted from the code under test: from one
  print, each line then checked by hand against view.md 15.3 and the
  fixture's state (decisions.md, 9);
- writes the words `legacy`, `compat`, `hack` or `workaround` in code.

## 6. Size limits

- **A file:** at most 800 lines, generated code excepted. A file of
  production or test kit code already past 800 lines grows, in one
  session, by at most what its session file says (40 lines where it
  names no number) and is not split by this plan:
  `crates/jig-domain-client/src/domain.rs` (1,241), `pages/routing.rs`
  (1,111), `objects.rs` (1,132), `crates/jig-view/src/pages/project/owner.rs`
  (1,167), `tests/client/src/stories.rs` (880; jig 06.4, 07.4 and 09.3
  may each add 60), `tests/client/src/harness/world.rs` (866; jig 10.2
  may add 80). Test files (`src/tests/`, `src/tests.rs`,
  `tests/*/tests/`) already past 800 lines take new tests without a
  limit.
- **A test function:** at most 40 lines; a table test's rows do not
  count against it.
- **An increment:** its session file's "Size: at most N lines", a guide:
  within N × 1.5 it lands, and its report gives its count; past it, it
  asks. A Size counts lines as `git diff -M --numstat
  origin/main...HEAD -- <the increment's Files>` prints them, added less
  removed, once, blank lines and comments included: production code,
  and test kit code where the Size says "test kit code" or "test
  code"; not tests or golden files. A moved file counts its changed
  lines only.

## 7. What checks these rules

- **Checked:** a new file's limit (`plan pulse`, `file-lines: 800`);
  the budgets (each gate's `global-timeout`; README, Done when 8); every
  exhaustive match (the compiler, and clippy's `wildcard_enum_match_arm`,
  denied by the workspace's lints outside the test modules that expect
  it, such as `crates/jig-view/src/tests/pages.rs:1-6`); the owners' spelling (README, Done
  when 2); the outline on every tree a world shows (jig 02.4's referee);
  the text shell's dependencies (jig 09.1's boundary test); no optimized
  build (`git -C ~/src/rust/jig diff 2ae75cac2 origin/main -- '*Cargo.toml'
  | grep opt-level` prints nothing).
- **Held by review, unchecked:** a test function's 40 lines and a
  file's growth; a test's ceiling under the SLOW flag; no query by
  region name in a story, and a story keeping its steps; a card or
  section built only by its function; golden files checked by hand;
  behaviour unchanged at jig's addresses in jig 03 and 04; no copy of
  the address (jig 03.2); no phrase removed, one added (jig 02.2);
  jig's navigation's order; one behaviour per test name; no expected
  value computed by the code under test but the golden files; a
  boundary's two sides carrying the same fields; the names kept by
  decisions.md, 5; a commit body naming a changed literal and its
  reason; a test keeping its name. The supervisor samples them in its
  hourly check (development.md, 14).
