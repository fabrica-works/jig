# Session skein 14: skein's stories for a browser shell, and the facts the face assumes

The face's table is checked against Chromium on fixed pages; the
restart part makes a panic fail-stop; and skein's four stories for a
browser shell become functions of the harness that any application's
tier calls with its binary, its bundle and its names (ui-web.md, 14.2),
the example's tier calling each with its own.

## Read first

- skein's `docs/design/ui-web.md`, sections 5 (a panic), 5.1 (restart),
  10 (the small store's bound), 14.1 and 14.2; `examples.md`, section
  9.4 (what the example gives the stories).
- This plan's `code-organization.md`, 1 and 3; skein 11's file
  (the harness's interface sketch).
- skein's `testing-strategy.md`, section 8.

## What the design asks

The facts the face assumes, the accessible role Chromium computes for
each element and input type of section 6's table and a select's option
chosen by the kit's keys, checked on fixed pages a small server on
skein's HTTP serves in the test's loop (ui-web.md, 14.1). A panic
reloads within three in a minute, counted under `skein-ui-restarts`,
and past it the page says the application failed and starts nothing
(5). The four stories: typing while a patch arrives; back and forward;
a panic; a write refused (14.2's table). Each takes what the table's
third column names, and nothing else; what it must see on a person's
page is a `Find` the application gives, awaited, never a fixed time,
and never skein's words where the page shows the application's (14.2,
skein `7c5a482`). In the test build alone, the example gives an
address that panics at start and one that bounds its small store below
an add's record (examples.md, 9.4: "those addresses are the plan's").

## Choices this plan makes

- **The test build's addresses:** `/test/panic` and `/test/small-store`,
  read by the example's view at start under `cfg(debug_assertions)`
  only; any other build parses them as `Missing`.
- **The stories' module** is `skein_fake_person_browser::stories`; the
  example's tier calls them from `tests/ui-web/tests/browser_stories.rs`.

## Rules that bind it

- **A story function takes what the application names** (14.2's third
  column), never the example's words: `rg -n -i 'note' 
  testing/skein-fake-person-browser/src/stories.rs` prints nothing.
- **The panic story is the one story expecting trouble:** its
  `Setup.expected` holds `Uncaught` and nothing else, the words the kit
  reports for the trap after the panic hook (the kit gives an
  exception's `exceptionDetails.text`, `testing/skein-browser/src/
  browser.rs:1299`, which Chromium sets to `Uncaught`; dev builds
  abort on a panic, skein's `Cargo.toml:118`); every other story's is
  empty.
- **The browser suite's budget,** two minutes, holds the kit's tests
  and the whole tier: this session's six tests aim at 3 s each (README,
  "Test time").

## Increments

### 14.1 The facts the face assumes
Scope: ui-web.md 14.1 (the facts), 14.2 (the face's roles).
Needs: skein 11.4 (the first story); skein 02.5 (the vocabulary's
  last addition: every row of the mapping).
Files: new `tests/browser/tests/browser_facts.rs`, in the kit's world
  package (skein-browser-world, in the browser profile), whose `Rig` and
  `serve::Pages` (`tests/browser/tests/browser_real.rs`,
  `tests/browser/src/serve.rs`) serve a fixed page: the harness has no
  server of its own, it spawns `Setup.server`
  (`testing/skein-fake-person-browser/src/harness.rs:176-197`);
  `tests/browser/Cargo.toml` (dev-dependencies `skein-ui-web`,
  `skein-ui`); `crates/skein-ui-web/src/mapping.rs` (`accessible`, if a
  fact differs: the table is fixed first, ui-web.md 14.1), with its
  copy, the `ROLES` table at `src/tests/mapping.rs:8` (asserted at 96),
  and its users at `testing/skein-fake-person-browser/src/face.rs:197-198`.
Shape: one page holding an element of each row of ui-web.md 6's table
  and a select of three options, built in the test from
  `mapping::element`, as `crates/skein-ui-web/src/tests/mapping.rs:318-347`
  does, each element its own node id, a table's parts inside
  `<table><tr>`, an option inside its select, the menu open. Not the
  golden `mapping.html` served as is: its `<th>`, `<tr>` and `<td>` sit
  loose in a `<div>`, which Chromium's parser drops (11 of 11, checked
  with `--dump-dom`), every element carries `data-node="1"`, and the
  menu is hidden.
Test shape: `chromium_computes_each_roles_accessible_role` (a snapshot,
  each element's computed role compared with `mapping::accessible`, each
  differing row named); `chromium_chooses_an_option_by_the_kits_keys`.
  About 2 s each.
Proof: live, `heavy --gate scripts/check.sh` once: the two tests pass
  against the local Chromium. Bound: 15 minutes.
Don't: change the face to fit a fact without fixing the table.
Size: about 60 lines of production code, 150 of tests.
Removes: nothing.

### 14.2 The restart part, and the panic story
Scope: ui-web.md 5 (a panic), 5.1 (restart), 14.2 (a panic).
Needs: 14.1.
Files: new `crates/skein-ui-web/src/{restart.rs, restart/web.rs}`;
  `crates/skein-ui-web/src/lib.rs` (`pub mod restart;`) and
  `src/tests.rs` (`mod restart;`);
  `examples/ui/shell-web/src/lib.rs` (the panic hook at start);
  `examples/ui/view/src/places.rs` (`/test/panic` a `Place`: 34, 40),
  `src/view.rs` (its match over `Place`, 55), `src/state.rs` (start,
  48, where the test address panics) and `src/tests.rs` (the path
  table, 11-20); new `testing/skein-fake-person-browser/src/stories.rs`
  (`panic`, setting `Setup.expected` to `Uncaught`); new
  `tests/ui-web/tests/browser_stories.rs`. The workspace denies
  `clippy::panic` (`Cargo.toml:190`): the test address panics through
  an `assert!` on the path, not `panic!`.
Shape: `restart::Restart::new(limit: 3, within: 60 s, key:
  "skein-ui-restarts", words)`; its pure half decides from the dates
  kept whether to reload or to say the words; `stories::panic(setup,
  address, failed: Find)`, `failed` the application's words for "the
  application failed", which the story sees past the limit.
Test shape: host: `src/tests/restart.rs`:
  `a_fourth_panic_in_a_minute_says_the_application_failed`,
  `a_panic_past_the_minute_reloads_again`; story `a_panic_reloads_then_says_it_failed`.
  Step tests under 20 ms; the story about 3 s.
Proof: live, `heavy --gate scripts/check.sh` once: the story passes.
  Bound: 15 minutes.
Don't: keep the restarts' count in the small store's prefix; give
  `Restart` or the story words of skein's own: `words` and `failed`
  are the application's (ui-web.md, 5.1 and 14.2).
Size: about 150 lines.
Removes: nothing.

### 14.3 A write refused
Scope: ui-web.md 10 (the bound), 14.2 (a write refused).
Needs: 14.2; skein 12.3 (the add written to the small store).
Files: `examples/ui/view/src/places.rs` and `state.rs`
  (`/test/small-store`); `examples/ui/shell-web/src/lib.rs` (the shell
  makes the small store, 12.3: at `/test/small-store`, under
  `cfg(debug_assertions)`, its bound below an add's record);
  `testing/skein-fake-person-browser/src/stories.rs` (`write_refused`);
  `tests/ui-web/tests/browser_stories.rs`.
Shape: `stories::write_refused(setup, address, script, absent)`: the
  person's script, opened at `address`, makes a step that writes past
  the bound; a second person then opens the application at `/` and
  `absent` (what the refused step would have made) is not there, a
  `Gone` step within 1 s. Each person's page opens at `/` at the start
  and all play at once (`harness.rs:382-414, 486-489`), so the second
  person's `Play` waits on a flag the first's sets when it is done, then
  goes to `/` and plays `Gone`.
Test shape: story `a_refused_write_makes_no_note`. About 3 s.
Proof: live, `heavy --gate scripts/check.sh` once: the story passes.
  Bound: 15 minutes.
Don't: read the server's memory or logs: the second person's page is
  the record.
Size: about 80 lines.
Removes: nothing.

### 14.4 Typing while a patch arrives, and back and forward
Scope: ui-web.md 14.2 (the first two stories).
Needs: 14.3; skein 12.4 (the stream) and skein 13.1 (history).
Files: `testing/skein-fake-person-browser/src/stories.rs`
  (`typing_while_patched`, `back_and_forward`);
  `testing/skein-fake-person-browser/src/harness.rs` (`Step::Compose {
  find, words }`, `Step::Type { find, words }`, `Step::Back`,
  `Step::Forward`, the kit's `Compose`, `Type` and `Go::Back`,
  `Go::Forward`) and `src/face.rs` (`perform`, 135-145, which takes an
  act, `Go` and a reload alone); `tests/ui-web/tests/browser_stories.rs`.
Shape: `stories::typing_while_patched(setup, field, second, arrives)`:
  the first person composes in `field`, sees `arrives` gone (a `Gone`
  step: the page that showed the field shows no patch yet), and sets a
  flag the second person's `Play` waits on, as 14.3's second person
  does; the second's act then patches the first's page, and the first
  sees `arrives` (a `See` step within 5 s) before it types again; the
  field keeps its focus, its caret and its words (the harness's
  `Compose`, then its `Type`, the kit's `Type` inserting at the caret,
  which the person's `Input`, a whole value, cannot show); `stories::back_and_forward(setup, first, second)`:
  two addresses and what each shows, back and forward between them, and
  each opened directly.
Test shape: stories `typing_in_the_title_survives_a_note_arriving`
  and `back_and_forward_move_between_the_list_and_a_note`. About 3 s
  each.
Proof: live, `heavy --gate scripts/check.sh` once: the two stories
  pass. Bound: 15 minutes.
Don't: pause by a fixed sleep: each person awaits what it sees.
Size: about 120 lines.
Removes: nothing.

## Interface sketch

```rust
// testing/skein-fake-person-browser/src/stories.rs: skein's stories for a
// browser shell (ui-web.md, 14.2). An application's tier calls each with
// its harness's setup and what it names; each returns the harness's outcome.
/// A page and what a person sees on it.
pub struct Shown<'a> { pub address: &'a [u8], pub sees: Find }
// Each story returns `#[must_use]` (pedantic clippy's `must_use_candidate`).
#[must_use]
pub fn typing_while_patched(setup: &Setup<'_>, field: Find, second: &mut dyn Play, arrives: Find) -> Outcome;
#[must_use]
pub fn back_and_forward(setup: &Setup<'_>, first: Shown<'_>, second: Shown<'_>) -> Outcome;
#[must_use]
pub fn panic(setup: &Setup<'_>, address: &[u8], failed: Find) -> Outcome;
#[must_use]
pub fn write_refused(setup: &Setup<'_>, address: &[u8], script: &mut dyn Play, absent: Find) -> Outcome;
```

## Questions to raise when reached

None known.

## Done when

`heavy --gate scripts/check.sh` passes once with `browser_facts` and
`browser_stories` in its browser suite, and `rg -n 'pub fn' testing/skein-fake-person-browser/src/stories.rs`
prints the four stories.
