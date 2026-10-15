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
a panic; a write refused (14.2's table). In the test build alone, the
example gives an address that panics at start and one that bounds its
small store below an add's record (examples.md, 9.4: "those addresses
are the plan's").

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
  `Setup.expected` holds the panic's words and nothing else; every
  other story's is empty.
- **The browser suite's budget,** two minutes, holds the kit's tests
  and the whole tier: this session's six tests aim at 3 s each (README,
  "Test time").

## Increments

### 14.1 The facts the face assumes
Scope: ui-web.md 14.1 (the facts), 14.2 (the face's roles).
Needs: skein 11.4 (the first story); skein 02.5 (the vocabulary's
  last addition: every row of the mapping).
Files: new `tests/ui-web/tests/browser_facts.rs`; `tests/ui-web/src/lib.rs`
  (a fixed-page server in the test's loop, after
  `tests/browser/src/serve.rs`); `crates/skein-ui-web/src/mapping.rs`
  (`accessible`, if a fact differs: the table is fixed first, ui-web.md
  14.1).
Shape: one page holding an element of each row of ui-web.md 6's table
  (the mapping's golden page, skein 09.3, served as is) and a select of
  three options.
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
  `examples/ui/shell-web/src/lib.rs` (the panic hook at start);
  `examples/ui/view/src/places.rs` (`/test/panic`); new
  `testing/skein-fake-person-browser/src/stories.rs` (`panic`, setting
  `Setup.expected` to the panic's words); new
  `tests/ui-web/tests/browser_stories.rs`.
Shape: `restart::Restart::new(limit: 3, within: 60 s, key:
  "skein-ui-restarts", words)`; its pure half decides from the dates
  kept whether to reload or to say the words; `stories::panic(setup,
  address)`.
Test shape: host: `src/tests/restart.rs`:
  `a_fourth_panic_in_a_minute_says_the_application_failed`,
  `a_panic_past_the_minute_reloads_again`; story `a_panic_reloads_then_says_it_failed`.
  Step tests under 20 ms; the story about 3 s.
Proof: live, `heavy --gate scripts/check.sh` once: the story passes.
  Bound: 15 minutes.
Don't: keep the restarts' count in the small store's prefix.
Size: about 150 lines.
Removes: nothing.

### 14.3 A write refused
Scope: ui-web.md 10 (the bound), 14.2 (a write refused).
Needs: 14.2; skein 12.3 (the add written to the small store).
Files: `examples/ui/view/src/places.rs` and `state.rs`
  (`/test/small-store`: the store's bound below an add's record);
  `testing/skein-fake-person-browser/src/stories.rs` (`write_refused`);
  `tests/ui-web/tests/browser_stories.rs`.
Shape: `stories::write_refused(setup, address, script, absent)`: the
  person's script, opened at `address`, makes a step that writes past
  the bound; a second person then opens the application at `/` and
  `absent` (what the refused step would have made) is not there, a
  `Gone` step within 1 s.
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
  find, words }`, `Step::Back`, `Step::Forward`, the kit's `Compose`
  and `Go::Back`, `Go::Forward`); `tests/ui-web/tests/browser_stories.rs`.
Shape: `stories::typing_while_patched(setup, field, second)`: the first
  person composes in the field while the second's act patches its page;
  the field keeps its focus, its caret and its words (the kit's
  `Compose` then `Type`); `stories::back_and_forward(setup, pages)`:
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
pub fn typing_while_patched(setup: &Setup<'_>, field: Find, second: &mut dyn Play) -> Outcome;
pub fn back_and_forward(setup: &Setup<'_>, first: Shown<'_>, second: Shown<'_>) -> Outcome;
pub fn panic(setup: &Setup<'_>, address: &[u8]) -> Outcome;
pub fn write_refused(setup: &Setup<'_>, address: &[u8], script: &mut dyn Play, absent: Find) -> Outcome;
```

## Questions to raise when reached

None known.

## Done when

`heavy --gate scripts/check.sh` passes once with `browser_facts` and
`browser_stories` in its browser suite, and `rg -n 'pub fn' testing/skein-fake-person-browser/src/stories.rs`
prints the four stories.
