# Session skein 07: the browser kit's words in a name and press at a point

skein's browser kit finds a node by words in its name as well as by its
whole name, and presses at a point the test names as well as at the
box's centre, as browser.md 5 and 6 ask; its fake browser and its real
Chromium checks learn both. The browser face (skein 11.1) finds by words
through it, and the harness presses a drawing's mark at a point (skein
13.4).

## Read first

- skein's `docs/design/browser.md`, sections 5, 6 and 9.
- This plan's `findings/audit.md`, 3 (`skein-browser`).
- skein's `testing-strategy.md`, sections 2.9 and 8; `testing.md`,
  section 7 (the browser profile).

## What the design asks

`Query { role, name, matching: Match, within, boxes }`, `Match { Exact,
Words }`: `Words` matches a name containing the query's bytes, unbroken,
case kept; asked of CDP without a name, the role's nodes' names
compared in the kit, each name read whole before a kept one is cut
(browser.md, 5 and 6). `Press { page, op, node, at: Option<Point> }`,
`Point { x, y }` in whole CSS pixels from the box's top left: refused
`Hidden` with a point outside the box, `Covered` unless the hit test at
that point lands on the node or inside it (browser.md, 6). The real
browser checks a sentence found by a few of its words and a named
figure's inline SVG shape pressed at a point (browser.md, 9).

## Rules that bind it

- **`testing/skein-browser/src/browser.rs` is 1,774 lines at the base,**
  past the plan's file limit: this session adds at most 60 lines to it
  (`git diff --numstat origin/main --
  testing/skein-browser/src/browser.rs` shows at most 60 added, at each
  increment's end), and puts the words' comparison and the point's
  check in a new module, `testing/skein-browser/src/find.rs`. Pulse
  lists `browser.rs` as over the limit; that line is expected.
- **The kit sends no script** (browser.md, 4): `rg -n
  'Runtime\.(evaluate|callFunctionOn)' testing/skein-browser` prints
  nothing.
- **The browser suite's budget,** two minutes for the kit's tests and
  the UI example's tier together (testing.md, 7): 07.3 adds two
  scenarios of about 1 s each.

## Increments

### 07.1 A name matched exactly or by words
Scope: browser.md 5 (`Query`, `Match`), 6 (find by words).
Needs: nothing.
Files: `testing/skein-browser/src/boundary.rs` (`Query`, 84-89 at the
  base; new `Match`); `src/browser.rs` (the find's command,
  `Accessibility.queryAXTree` at 648; the reply's names); new
  `src/find.rs`; every `Query { … }` literal (`rg -n 'Query \{'
  testing/skein-browser tests/browser`); `tests/browser/src/world.rs`
  (the fake browser answers a query without a name with every node of
  the role); `tests/browser/src/tests.rs`.
Shape: `Query.matching: Match`; `Match::{Exact, Words}`; `find::words(
  name: &[u8], query: &[u8]) -> bool` (the query's bytes unbroken in the
  name, case kept; an empty query matches every name).
Test shape: in `testing/skein-browser/src/` step tests:
  `words_match_a_name_holding_them_unbroken`, a table test (rows:
  `b"3 failing"` in `b"Checks: 3 failing, 2 passed"` true; `b"3
  passed"` false; `b"Failing"` false, case kept; `b""` true); in
  `tests/browser/src/tests.rs`: `a_find_by_words_asks_without_a_name`
  (the fake browser sees `queryAXTree` with the role and no name).
  Each under 20 ms; the world test under 0.5 s.
Proof: world. `heavy cargo nextest run -p skein-browser-world` once:
  the kit's machine world against the fake browser finds by words.
  Real Chromium is 07.3's.
Don't: add a pattern beyond words (browser.md, 11).
Size: about 80 lines.
Removes: nothing.

### 07.2 A press at a point
Scope: browser.md 5 (`Press`, `Point`), 6 (press).
Needs: nothing.
Files: `testing/skein-browser/src/boundary.rs` (`Request::Press`, 35;
  new `Point`); `src/browser.rs` (the press's steps, the box's centre at
  1582-1587; `press_hit` 1406); `src/find.rs` (`point_in`); every
  `Request::Press { … }` (`rg -n 'Request::Press' testing/skein-browser
  tests/browser`); `tests/browser/src/world.rs` (the fake browser's hit
  test at a point).
Shape: `Request::Press { page, op, node, at: Option<Point> }`; `Point
  { x: u32, y: u32 }`; `find::point_in(rect: Rect, at: Point) -> bool`;
  `at: None` is the box's centre as at the base.
Test shape: `a_point_outside_the_box_is_hidden` (step test, refused
  `Hidden` before any mouse event); `a_press_at_a_point_hits_there`
  (world: the fake browser records the hit test's point). Each under
  20 ms or 0.5 s.
Proof: world. `heavy cargo nextest run -p skein-browser-world` once:
  the fake browser's hit test at the point. Real Chromium is 07.3's.
Don't: dispatch a mouse event on a refused press (the referee's rule,
  browser.md 9).
Size: about 60 lines.
Removes: nothing.

### 07.3 Real Chromium finds by words and presses an SVG's shape
Scope: browser.md 9 (the real browser's checks).
Needs: 07.1, 07.2.
Files: `tests/browser/tests/browser_real.rs` (`Scenario`, 128-135;
  its scenarios 590-632); `tests/browser/src/serve.rs` (two fixed pages,
  `page`, 21-32).
Shape: two pages: a sentence `Checks: 3 failing, 2 passed`; a figure
  named `Error rate` holding `<svg aria-hidden="true">` with one
  `<rect>` at a known offset, whose press changes a heading by the
  page's own script file (`/press.js`, served by `serve.rs`, no inline
  script).
Test shape: `chromium_finds_a_sentence_by_a_few_of_its_words`,
  `chromium_presses_an_svg_shape_through_its_figure_at_a_point`, each
  as the existing scenarios are written (`browser_real.rs:590-632`).
  About 1 s each.
Proof: live (a local Chromium): `heavy cargo nextest run -p
  skein-browser-world --profile browser` once; the two scenarios pass
  with the kit's earlier six. Bound: 2 minutes.
Don't: skip on a missing Chromium (it fails, naming it).
Size: about 40 lines of page text; tests about 60.
Removes: nothing.

## Interface sketch

```rust
// testing/skein-browser/src/boundary.rs: the kit's vocabulary, extended
// (browser.md, 5). The test sends these; the kit answers with one
// terminal event each.
pub struct Query { pub role: Box<[u8]>, pub name: Box<[u8]>, pub matching: Match, pub within: Option<Node>, pub boxes: bool }
/// Exact: the name is the query's. Words: the name holds the query's bytes.
pub enum Match { Exact, Words }
/// Whole CSS pixels from the node's box's top left.
pub struct Point { pub x: u32, pub y: u32 }
// Request::Press { page: Token, op: Token, node: Node, at: Option<Point> }
```

## Questions to raise when reached

None known.

## Done when

`heavy cargo nextest run -p skein-browser-world` passes, and
`heavy cargo nextest run -p skein-browser-world --profile browser`
passes once.
