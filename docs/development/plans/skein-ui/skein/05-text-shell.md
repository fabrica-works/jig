# Session skein 05: the text shell whole

skein-ui-text prints and reads what ui-text.md asks beyond jig's shell:
its terminal tested on the simulated machine in `tests/ui-text`; options
numbered, an option of a disabled select unnumbered and refused as gone;
the properties' words (descriptions, entry kinds, a secret hidden,
positions and counts); its worst case grown to them; and the golden tree
with every role, state and property.

## Read first

- skein's `docs/design/ui-text.md`, sections 3, 5, 6, 7 and 8.
- skein's `docs/design/ui.md`, sections 3.2 and 3.4; `testing.md`,
  section 6.
- This plan's `code-organization.md`, 1, 5 and 6.
- skein's `programming-model.md`, sections 10 and 10.2;
  `testing-strategy.md`, section 8.

## What the design asks

ui-text.md 5's lines and words for every role, state and property; 6's
numbers, commands and refusals, an option of a disabled select counting
as disabled; 3's `worst_case` bounding the renderer's output at the
limits; 8's golden text and the terminal's tests in `tests/ui-text` on
skein's simulated machine.

## Choices this plan makes

- The golden tree with every role, state and property is a fixture of
  its own, `every_node(&mut Builder)` in `src/tests/fixtures.rs`, printed
  to `src/tests/golden/every.txt`; `roles.txt` stays as it is.

## Rules that bind it

- **Golden files change by hand** (ui-text.md, 8): an existing golden's
  changed lines quoted in the commit body; a new golden named once.
- **Nothing from the tree drives the terminal** (ui-text.md, 5): every
  new text the renderer prints (a description, a count) goes through the
  same replacement of control bytes as a name does.
- **Test time,** aims (README, "Test time"): each step test under
  20 ms; the terminal's tests under 0.5 s each.

## Increments

### 05.1 The terminal's tests in `tests/ui-text`
Scope: ui-text.md 8 (the terminal on the simulated machine); testing.md 6.
Needs: skein 01.3 (the moved crates; nothing in them changes between
  skein 01.1 and 01.3).
Files: `crates/skein-ui-text/src/tests/terminal.rs` moved to new
  `tests/ui-text/tests/terminal.rs`, with what it shares in
  `tests/ui-text/src/lib.rs` (a copy of `LIMITS`, `binding`, `page` and
  `button` from `crates/skein-ui-text/src/tests/fixtures.rs:5-44` as
  they are on `main` at the increment's start: with skein 04.1's two
  scene fields if it has landed); new `tests/ui-text/Cargo.toml`
  (`skein-ui-text-tests`, `skein-sim` a dependency);
  `crates/skein-ui-text/Cargo.toml` (`skein-sim` dev-dependency goes);
  `crates/skein-ui-text/src/tests.rs` (module list); skein's
  `Cargo.toml` (member) and `Cargo.lock`. The harness's `start()`
  (the moved file's lines 27-28) writes `.expect("the settings fit the
  limits")` and `.expect("a terminal adopts its pipes once")` for its two
  `.unwrap()`s: the tests crates deny `unwrap_used`, and
  `allow-unwrap-in-tests` does not reach a helper.
Shape: no production change.
Test shape: the moved tests (`crates/skein-ui-text/src/tests/terminal.rs:63-166`,
  eight tests, ui-text.md 8's list, a name longer than a node's text
  among them), unchanged but for paths; none new. Under 0.5 s each.
Proof: world: `heavy cargo nextest run -p skein-ui-text-tests` once:
  the terminal on the simulated machine reads lines and paces prints.
  Bound: 1 minute.
Don't: change a test's body beyond paths.
Size: no production code; tests moved.
Removes: `crates/skein-ui-text/src/tests/terminal.rs`.

### 05.2 Options numbered, and an option of a disabled select refused
Scope: ui-text.md 6 (numbered nodes; refusals).
Needs: skein 02.4 (`Role::Select`, `Role::Option`); skein 03.2
  (`Tree::disabled`).
Files: `crates/skein-ui-text/src/render.rs` (numbering, `number`,
  288-297), `src/command.rs` (`[n] is gone`, 66-68);
  `src/tests/command.rs`; fixture and goldens.
Shape: an option is numbered when bound and `Tree::disabled` (skein
  03.2: it or its nearest select ancestor disabled) is false, else
  `[-]`; an option whose select was disabled after the print is
  `Refusal::Gone`, through the same `Tree::disabled`, the one copy of
  the rule. `n` on an option is a `Press` since skein 02.4, which adds
  the option's arm to `command.rs`'s match over `Role`; 05.2 tests it.
Test shape: `src/tests/command.rs`: `an_option_is_chosen_by_its_number`,
  `an_option_of_a_disabled_select_prints_unnumbered`,
  `an_option_disabled_after_the_print_is_gone`. Reference test:
  ```rust
  #[test]
  fn an_option_disabled_after_the_print_is_gone() {
      let shown = fixtures::page(|b| fixtures::label_select(b, false));
      let print = numbers(&shown);
      let later = fixtures::page(|b| fixtures::label_select(b, true));
      assert_eq!(
          command(b"2", Some(print.numbers.as_slice()), &later, 80),
          Command::Refused { refusal: Refusal::Gone, number: 2 }
      );
  }
  ```
  `fixtures::page` is the moved fixture; `numbers(&Tree) -> Print` is
  `src/tests/command.rs:9`'s, as it is; `label_select(b, disabled)` (a
  select of three options, the second numbered 2) is added to the
  fixtures. Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p skein-ui-text` once. Unit is
  the highest layer for the command reader: the terminal world (05.1)
  passes lines through it and chooses nothing.
Don't: refuse in the command reader what the view refuses for its role
  (ui-text.md, 6: a command acts as each role takes).
Size: about 40 lines.
Removes: nothing.

### 05.3 The properties' words
Scope: ui-text.md 5 (properties: entry kinds, a secret's `(hidden)`,
  descriptions, `(P of C)`).
Needs: skein 02.1 and 02.2 (the properties on the node).
Files: `crates/skein-ui-text/src/render.rs` (a field's line,
  235-241; the node's line end; descriptions after it);
  fixture and a new golden `src/tests/golden/properties.txt`.
Shape: after a text field's value and before its states, `(number)`,
  `(email)`, `(telephone)`, `(url)`, `(date, YYYY-MM-DD)`, `(time,
  HH:MM)`; a secret's non-empty value `(hidden)`; a description on a
  line of its own after its node's line, indented, the descriptions of
  nodes printed within a row's or an item's line after that line in
  preorder, and a node printing no line of its own giving its
  description at its children's indentation before them; `(P of C)` or
  `(P)` at the end of the first line of an item, a row or a tree item.
Test shape: `src/tests/golden/properties.txt`, by hand, from a fixture
  `properties(&mut Builder)` holding each case of ui-text.md 5's
  properties list; `a_secret_never_prints_its_bytes` (the value's bytes
  absent from the print). Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p skein-ui-text` once. Unit is
  the highest layer: no application's view in skein sets these
  properties until skein 13.2, whose pages print through this shell.
Don't: print a cell's spans, a focus request, an image's source or a
  shortcut (ui-text.md, 5: not printed).
Size: about 120 lines.
Removes: nothing.

### 05.4 The worst case grown to the new words
Scope: ui-text.md 3 (`worst_case`), 8 (its bound).
Needs: 05.3; skein 02.5 (`(required)`).
Files: `crates/skein-ui-text/src/render.rs` (`worst_case`, 26-38);
  `src/tests.rs` or `src/tests/bound.rs`. Its user: `Terminal::new`
  (`terminal.rs:69-73`) sizes the print and io's output by it.
Shape: `worst_case(&Limits) -> Option<(u32, u64)>` counts, per node, a
  description's own line, the longest entry kind's words, `(required)`
  and `(P of C)` at `u32::MAX` digits, besides what it counts at the
  base.
Test shape: `the_print_of_a_full_page_fits_its_worst_case`: a page at
  a fixture's limits, every text at its most bytes of the bytes that
  grow most (newlines, each 1 byte, the indentation and a line, with
  `> ` in a quote; and control bytes, three bytes printed), every node
  with a description and every state and property its role takes (the
  builder asserts `Required` only where skein 02.5 allows it); the
  print's lines and bytes at most `worst_case`. Under 20 ms (a small
  fixture's limits).
Proof: unit, `heavy cargo nextest run -p skein-ui-text -p
  skein-ui-text-tests` once: a bound of step code, checked where it is
  computed, and the terminal, which sizes its buffers by it, still
  starting.
Don't: lower a bound to fit (the bound follows the words).
Size: about 40 lines.
Removes: nothing.

### 05.5 The golden tree with every role, state and property
Scope: ui-text.md 8 (the golden pages).
Needs: skein 02.3 (Escape), 02.4 (both sorts) and 02.5
  (`State::Required`); 05.3.
Files: `crates/skein-ui-text/src/tests/fixtures.rs` (`every_node`,
  `first_lines`, `control_bytes`); new goldens `every.txt`,
  `first-line.txt`, `bytes-everywhere.txt` in `src/tests/golden/`;
  `src/tests.rs`.
Shape: no production change; a defect a golden shows is fixed in
  `render.rs` in this increment when the fix is at most 40 lines, named
  in the commit body; a larger one is a question.
Test shape: three golden tests: `every_role_state_and_property_prints`
  (ui-text.md 8's first list: each entry kind, a secret with a value and
  one empty, a required field, a description on a group, a field, a
  row's cell and a node that prints no line, a count with positions and
  positions without one, both sorts, a cell's spans, an image of each
  source, a node naming Escape, a focus request);
  `the_first_line_without_its_status_region` (set and absent: set and
  present, and not set, are `src/tests.rs:39-68`'s); `control_bytes_print_as_replacements_everywhere`
  (a name, a run, a value, a description). Each under 20 ms.
Proof: unit, the golden tests, once: the highest layer for a renderer
  no application in skein uses until skein 10.3.
Don't: change `roles.txt` or `bytes.txt`.
Size: no production code beyond a defect's fix.
Removes: nothing.

## Questions to raise when reached

None known.

## Done when

`heavy cargo nextest run -p skein-ui-text -p skein-ui-text-tests`
passes, and `ls crates/skein-ui-text/src/tests/golden` lists
`every.txt`, `first-line.txt`, `bytes-everywhere.txt` and
`properties.txt`.
