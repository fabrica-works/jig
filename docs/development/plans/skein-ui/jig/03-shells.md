# Session jig 03: the text shell and the fake person name only `jig_view::ui`

The text shell's status line comes from a setting instead of jig's link
region, its output byte for byte as before; the fake person names only
`jig_view::ui`; and the roles jig binds are checked against ui.md 6's
act table by the builder. After it, `jig-shell-text` and
`jig-fake-person` move whole (skein 01.2, 01.3).

## Read first

- skein's `docs/design/ui-text.md`, sections 3 and 5 (the first line);
  `docs/design/ui.md`, sections 6 and 17.1.
- jig's `docs/design/client/domain/view.md`, section 15.3.
- This plan's `decisions.md`, 5; `findings/audit.md`, 2.
- skein's `programming-model.md`, sections 10 and 10.2.

## What the design asks

The text shell's settings carry the status region, one of the
application's region names, or none (ui-text.md, 3); jig's settings name
jig's link region (view.md, 15.3). A binding on a role ui.md 6's table
does not list is the owner's bug, which the builder asserts (ui.md, 6).

## Choices this plan makes

- `Settings` gains `status: Option<ui::Region>`; `render(tree, name,
  status, print)`.
- The builder's role assertion lands here, in `jig_view::ui`, since no
  view of jig binds another role (findings/audit.md); a drawing stays
  refused as at the base until skein 03.2 gives it the `Drawing` act.

## Rules that bind it

- **Golden files change only where this session's file says:** none in
  03.2.
- **The crates name only `jig_view::ui`, `skein_lib` and `skein_io`
  once jig 02.5 has landed too:** jig 02's increments change these
  crates' imports as they move items (every user in the same
  increment); this session removes the rest, `JigRegion` and jig's
  `Owner`.

## Increments

### 03.1 The builder asserts the roles a binding may take
Scope: ui.md 6 (the act table).
Needs: jig 01.3.
Files: `crates/jig-view/src/ui/builder.rs` (`bind`, 251); the fake
  person's test that binds seven landmarks
  (`testing/jig-fake-person/src/tests.rs:549-550`), which binds buttons
  in their place.
Shape: `bind` asserts the innermost open node's role is a text field, a
  text area, a check, a choice, a button, a link, a tab or a menu item
  (an option comes with skein 02.4, a drawing with skein 03.2).
Test shape: `ui/tests/tree.rs` (moved there by jig 01.3): a table test
  `bind_takes_only_the_roles_a_person_acts_on`, a row per role with its
  expected outcome as a literal, using `std::panic::catch_unwind` per
  row. Under 20 ms.
Proof: world. `heavy cargo nextest run -p jig-client-world -p
  jig-ops-world -p jig-client-protocol-world` passes once: no view of
  jig's or of its examples binds another role. Bound: 4 minutes.
Don't: refuse acts in `View::act` (skein 03.2).
Size: about 20 lines.
Removes: nothing.

### 03.2 The status region, a setting
Scope: ui-text.md 3 and 5; view.md 15.3.
Needs: jig 01.3 (`jig_view::ui`).
Files: `crates/jig-shell-text/src/{terminal.rs (Settings 14-20, 240),
  render.rs (2, 43-86)}`, `src/tests.rs:43`,
  `src/tests/fixtures.rs` (its owners at 17, 176, 181 and 198 become
  `ui::Owner { number: 1 }`, as 03.3 does for the person); the users that make `Settings`
  (`tests/client/src/harness/world.rs`,
  `tests/client-protocol/src/native.rs`).
Shape: `Settings { name, longest, interval, status: Option<Region> }`;
  `render(tree: &Tree, name: &[u8], status: Option<Region>, print: &mut
  Print)`; jig's users pass `Some(JigRegion::Link.region())`.
Test shape: the goldens unchanged (`tests/golden/roles.txt:1` and
  `bytes.txt:1` print `== TITLE == `); `src/tests.rs`'s status test
  takes its region from the fixture; one new
  `no_status_region_ends_the_first_line_after_its_space`.
Proof: world. `heavy cargo nextest run -p jig-client-world -E
  'binary(text_pages) | test(/at_the_text_shell/)'` passes once: jig's
  pages print their link's state from the setting. Bound: 2 minutes.
Don't: change any other line the renderer prints.
Size: about 30 lines.
Removes: `JigRegion` from the crate.

### 03.3 The fake person names only `jig_view::ui`
Scope: ui.md 17.1.
Needs: jig 01.3.
Files: `testing/jig-fake-person/src/*.rs`, `src/tests.rs:52, 87`.
Shape: imports from `jig_view::ui`; its tests' owners
  `ui::Owner { number: 1 }`; doc comments cite skein's `ui.md`, 17.1.
Test shape: the crate's tests, unchanged but for paths.
Proof: world. `heavy cargo nextest run -p jig-client-world -p
  jig-ops-world -E 'test(/stories/)'` passes once. Bound: 3 minutes.
Don't: change its behaviour (skein 06 does).
Size: about 10 lines, plus paths.
Removes: nothing.

## Questions to raise when reached

None known.

## Done when

`rg -n 'JigRegion' crates/jig-shell-text/src` and `rg -n
'Owner::(Jig|Application)' crates/jig-shell-text/src
testing/jig-fake-person/src` print nothing,
and `heavy cargo nextest run -p jig-shell-text -p jig-fake-person`
passes with the goldens unchanged.
