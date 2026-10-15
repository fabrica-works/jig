# Session jig 02: binding, limits, outline and phrases split

Each of `binding.rs`, `limits.rs`, `outline.rs` and `phrases.rs` splits
into a generic half, moved into `jig_view::ui`, and jig's half, which
stays; jig's outline rules go. The session ends with `jig_view::ui`
naming nothing outside itself, checked by a test, so that skein 01 can
move it as it is.

## Read first

- skein's `docs/design/ui.md`, sections 6, 9, 14 and 15.
- jig's `docs/design/client/domain/view.md`, sections 3.1, 5, 7 and 10.
- This plan's `decisions.md`, 5; `code-organization.md`, 2, 4 and 6;
  `findings/audit.md`, 1.
- skein's `programming-model.md`, section 10.

## What the design asks

The bindings and acts, `Limits` and `worst_case`, the outline's rules
and the phrasebook's machinery are skein-ui's (ui.md, 6, 9, 14, 15);
jig keeps its meanings, its constructors of limits and its scratch, its
phrase tables, and adds no outline rule (view.md, 3.1: "The outline is
skein-ui's; jig adds no rule"). The chip hole is skein-ui's reference
hole (ui.md, 9; view.md, 7).

## Choices this plan makes

- The outline's check in `ui/outline.rs` checks ui.md 14's six rules
  over the roles `ui::Role` has today, and the shortcut rule over
  `Shortcut` as it is; the Escape rule comes with Escape in skein 02.3.
- The reference hole is `Hole::Reference { number: u64, name }` and
  `HoleKind::Reference`, its span `ReferenceSpan { start, end, number
  }`, `Saying.references`: the chip's fields renamed (`task` →
  `number`), as ui.md 9 words it ("a name with its owner's reference
  number").

## Rules that bind it

- **Every user changes in the same increment;** no alias, no re-export
  from the old path (code-organization.md, 6).
- **Split, don't rewrite:** each generic item moves with `git mv` or cut
  and paste, unchanged but for paths, its owner type, its doc comments'
  citations, and what this file names: 02.2's `impl Limits` turned into
  jig's free functions, and 02.4's chip renamed the reference hole.
- **jig's outline rules go with their tests,** not kept elsewhere.

## Increments

### 02.1 Bindings split
Scope: ui.md 6.
Needs: jig 01.3 (`jig_view::ui`).
Files: `crates/jig-view/src/binding.rs` (generic 7-51 to
  `ui/binding.rs`); `crates/jig-view/src/tests/binding.rs` (8-75 to
  `ui/tests/binding.rs`, 77-91 stay); `ui.rs`; the test modules' lists
  in `tests.rs` and `ui/tests.rs`; `lib.rs`; users of
  `jig_view::{Act, Binding, BoundAct, number}` (`rg -lU
  'jig_view::(\{[^}]*)?\b(Act|Binding|BoundAct|number)\b'`), among
  them `crates/jig-shell-text/src/command.rs:3,48`; and users of
  `crate::{Act, Binding, BoundAct, number}` inside jig-view (`rg -lU
  'crate::(\{[^}]*)?\b(Act|Binding|BoundAct|number)\b'
  crates/jig-view/src`: 20 files at `d2a7a714e`, among them
  `confirmation/amendment.rs`, `ui/tree.rs`, `ui/builder.rs`,
  `tests/turns.rs` and `tests/work_pages/sections.rs`). The moved tests
  name the tree's owner as `ui::Owner { number: N }`, not jig's `Owner`
  (`Owner::Jig` is `ui::Owner { number: 65_536 }`, `owner.rs:18`), so
  that 02.5's test passes.
Shape: `ui/binding.rs`: `Binding { owner: ui::Owner, token: Token }`,
  `Act`, `BoundAct`, `number`, as at the base. `binding.rs` keeps
  `JigMeaning`, `Entry`, `JigBindings`, `decode`,
  `jig_bindings_worst_case`.
Test shape: the moved tests; no new test.
Proof: world. `heavy cargo nextest run -p jig-client-world -E
  'test(/acts|stories/)'` passes once: acts decode through the split
  bindings. Bound: 2 minutes.
Don't: change `JigBindings`' API.
Size: about 20 lines, plus paths.
Removes: nothing.

### 02.2 Limits split
Scope: ui.md 15; view.md 10.
Needs: jig 01.3.
Files: `crates/jig-view/src/limits.rs` (11-59 to `ui/limits.rs`, but
  line 58); `crates/jig-view/src/pages.rs:208-213` (`scratch`); `lib.rs`
  (`mod limits` becomes `pub mod limits`); the test modules' lists;
  every caller of `worst_case` (`rg -n 'worst_case\(' crates/jig-view
  testing tests examples --type rust`), among them
  `ui/tests/tree.rs:146`; every user of the moved type (`rg -lU
  'jig_view::(\{[^}]*)?\bLimits\b' --type rust`: 18 files outside
  jig-view at `d2a7a714e`, among them
  `crates/jig-shell-text/src/{render,terminal,tests/fixtures}.rs`,
  `tests/client/src/{model,observe,test}.rs` and
  `tests/ops/src/{client/app,joint/fixture}.rs`, and `crate::Limits`
  inside jig-view); every caller of `Limits::{of, of_words, compose,
  envelope}` (`rg -n 'Limits::(of|of_words|compose|envelope)\b'
  --type rust`), among them `examples/ops/view/src/limits.rs:49,58`,
  `testing/jig-test-view/src/limits.rs` and
  `testing/jig-test-view/src/tests/world_limits.rs`.
Shape: `ui::Limits` (fields as at the base) and `ui::worst_case(&Limits)
  -> Option<u64>` counting skein-ui's own containers only; jig's
  `limits.rs` keeps `impl Limits` 61-153 as free functions or an
  extension the same increment names (`limits::of`, `limits::of_words`,
  `limits::compose`, `limits::with_phrase_bytes`, `limits::envelope`,
  each taking `ui::Limits` where it took `self`), and a new
  `limits::jig_worst_case(&ui::Limits) -> Option<u64>` =
  `ui::worst_case` plus `pages::scratch`; jig's callers that checked
  the whole call `jig_worst_case`.
Test shape: `crates/jig-view/src/tests/limits.rs` (new):
  `jig_worst_case_adds_the_page_scratch_to_skein_uis` with literal
  limits from `ui/tests/fixtures.rs`; the moved tests.
Proof: world. `heavy cargo nextest run -p jig-client-world -E
  'binary(memory)'` passes once (its two tests), and `heavy cargo
  nextest run --profile fuzzy -p jig-client-world -E
  'binary(fuzzy_memory)'` passes once: the client's world checks the
  most each part held against `jig_worst_case`. Bound: 2 minutes each.
Don't: add the scene limits (skein 04); change a number.
Size: about 60 lines, plus paths.
Removes: `impl Limits` from the generic type.

### 02.3 The outline's rules, skein-ui's alone
Scope: ui.md 14; view.md 3.1.
Needs: jig 01.3.
Files: `crates/jig-view/src/outline.rs` to `ui/outline.rs`;
  `crates/jig-view/src/tests/outline.rs` (its tests of jig's rules,
  35-41, 42-55, 76-86, go; the others move; its `page()` and `finish()`,
  7-15, move to `ui/tests/fixtures.rs` as the fixture's builder, which
  01.3 did not add, with a one-node banner in place of `banner()` 16-24,
  so the kept tests' nodes renumber: 64 and 74 from 6 to 3, 102 from 7
  to 4); `tests/client/tests/referee.rs:449-475` (its sample fault, line
  462, becomes `OneMain`, a page with two mains); the test modules'
  lists; every user of the `outline` path (`rg -n
  'outline::(check|Rule|Fault)' --type rust`), among them
  `tests/client/src/harness/{world.rs:632,observation.rs:25}`,
  `examples/ops/view/src/tests.rs:576` and
  `crates/jig-view/src/tests/pages.rs:513,522,525,574`.
Shape: `ui::outline::{check(&Tree) -> Option<Fault>, Fault { rule:
  Rule, node: u32 }, Rule}` with `Rule::{PageRoot, InsideLandmark,
  OneMain, OneBanner, OneContentInfo, ShortcutOnce}` (Escape added by
  skein 02.3); `InsideLandmark` accepts banner, navigation, main,
  complementary, content info, form, search, a named region and a
  dialog; `BannerFirst`, `BannerHolds`, `AfterPageLast` removed, with
  `JigRegion`'s use in `outline.rs`.
Test shape: `ui/tests/outline.rs`, one test per rule, as rows of one
  table test `each_rule_names_its_first_fault`, each row a tree built
  with the fixture's builder and its expected `Fault` as a literal; and
  `a_page_with_form_search_and_a_dialog_is_whole`. Each under 20 ms.
Proof: world. `heavy cargo nextest run -p jig-client-world -p
  jig-ops-world -E 'test(/referee|stories/)'` passes once: every page
  of jig's and `ops`'s worlds keeps skein-ui's outline. Bound: 3
  minutes.
Don't: keep jig's rules in another function (view.md 3.1); add
  `jig_view::check` (later.md).
Size: about 80 lines.
Removes: `Rule::{BannerFirst, BannerHolds, AfterPageLast}` and their
  tests.

### 02.4 The phrasebook's machinery apart from jig's tables
Scope: ui.md 9; view.md 7.
Needs: jig 01.3 (`PhraseId`'s owner is `ui::Owner` since 01.1).
Files: `crates/jig-view/src/phrases.rs` (generic items to
  `ui/phrases.rs`; jig's `pub mod english; pub mod jig;` stay; the
  `jig_capacity` field 198 and its setting 324-325 go with
  `Phrasebook`; the method `jig_capacity()` 341-346 and
  `normal_capacity` 428-434 become jig's free function, below, in
  `phrases.rs`); `crates/jig-view/src/tests/phrases.rs` (its jig
  assertion at 210, `book.jig_capacity()` is `None` for partial words,
  stays in jig's tests, calling the free function);
  `crates/jig-view/src/tests/owner_history.rs:549`; the test modules'
  lists; users of
  `ChipSpan`, `Hole::Chip`, `HoleKind::Chip`, `Saying.chips`,
  `jig_capacity` (19 files) and `phrases::{…}` machinery.
Shape: `ui::phrases::{PhraseId, HoleKind, Hole, …, Phrasebook, Saying,
  load, say, capacity, worst_case}`; the reference hole as in "Choices"
  above, the chip's `title` renamed `name` as well as `task` →
  `number` (`phrases.rs:50`); jig's `phrases::jig_capacity(&Phrasebook)
  -> Option<u32>` computed from `jig::ALL` as `normal_capacity` does
  today (`None` for a partial book), kept beside jig's tables.
Test shape: the moved tests, the chip's renamed: the span test at
  `tests/phrases.rs:92` becomes the reference hole's, so no new test.
Proof: world. `heavy cargo nextest run -p jig-client-world -E
  'test(/stories/)'` passes once: every page says its phrases through
  the split machinery. Bound: 2 minutes.
Don't: change a phrase's text; move `phrases/jig.rs` or `english.rs`.
Size: about 50 lines, plus paths.
Removes: the chip hole's names.

### 02.5 `jig_view::ui` names nothing outside itself
Scope: decisions.md 5.
Needs: 02.1 to 02.4.
Files: a new `crates/jig-view/src/tests/ui_boundary.rs`; `tests.rs`'s
  list; any `ui/` file the test finds naming outside, among them the
  tests 01.3 moved (`ui/tests/{tree,diff}.rs` name `crate::Binding`,
  `crate::Limits`, `crate::worst_case`).
Shape: no production change but what the test finds.
Test shape: one test, `the_ui_module_names_only_itself_and_skein_lib`:
  for each file under `src/ui/` (listed by `include_str!`, one per
  file), no line holds `crate::` not followed by `ui::`, nor
  `jig_domain_client`, nor `jig_` (literals in the test). Under 20 ms.
Proof: unit; the check is the deliverable, and skein 01 runs the move
  it guards.
Don't: exempt a file.
Size: about 10 lines of production code.
Removes: nothing.

## Questions to raise when reached

None known.

## Done when

`cargo nextest run -p jig-view -E 'test(the_ui_module_names_only_itself_and_skein_lib)'`
passes, and `rg -n 'BannerFirst|AfterPageLast|ChipSpan' --type rust`
prints nothing.
