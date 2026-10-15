# Session skein 04: scenes, and skein-ui's worst case in `tests/ui`

skein-ui gets scenes (ui.md, 11): the marks drawing code writes, bounded
by two new limits; the order in which a press takes an act; and scenes
printed as lines for golden files. Then `tests/ui` checks skein-ui's
worst case, a scene's included, against the counting allocator.

## Read first

- skein's `docs/design/ui.md`, sections 11, 11.1 to 11.4, 15 and 17.
- skein's `docs/design/testing.md`, section 6 (the layout) and 7
  (memory).
- This plan's `code-organization.md`, 1, 3 and 6.
- skein's `programming-model.md`, sections 6.3 (the worst case) and 10;
  `testing-strategy.md`, section 8.

## What the design asks

A scene is a bounded list of marks in preorder, whole pixels from the
box's top left as `u16`s, each mark with its tokens, a hint and an act;
no colour; refused whole past `scene_marks` or `scene_bytes` (ui.md,
11.2). A press takes the topmost acting mark under the point, else the
nearest acting group of the topmost mark under it, else the scene's act
with the point appended as two `u16` little-endian, else none (ui.md,
11.3). `worst_case` counts a scene (ui.md, 15). Golden scenes print one
line per mark, a group's marks indented (ui.md, 17).

## Choices this plan makes

- **The press order takes the platform's hit test as input:** the marks
  under the point, topmost first, as the shell found them (in a browser,
  the SVG's elements under the pointer). skein-ui decides the order, not
  the geometry: a text mark's box needs the platform's fonts.
- **Building a scene** is a writer over a `Scene` reserved from the
  limits: each call adds one mark and returns `false` once the scene is
  refused; a refused scene stays refused until `clear`. Drawing code
  returns `false` then, and its fallback shows (ui.md, 11.1).
- **The printed form,** one line per mark, two spaces per group depth:
  `group +X,Y`, `line X,Y X,Y`, `polyline X,Y X,Y …`, `path M X,Y L
  X,Y Q X,Y X,Y C X,Y X,Y X,Y Z`, `rect X,Y WxH rR`, `circle X,Y rR`,
  `text X,Y start|middle|end "BYTES"`; then ` [O:N …]` for its tokens
  (owner and number), ` hint="…"` and ` act=HEX` where present. Bytes
  outside printable ASCII print as `\xHH`.

## Rules that bind it

- **A scene is step code under the subset:** no float, no recursion
  (a group is a mark followed by its count of marks), no allocation past
  `Scene::new`.
- **Refused, never cut** (ui.md, 11.2): a refused scene keeps no mark:
  `marks()` is empty until `clear`, which 04.1's tests check.
- **Memory tests are binaries of their own** (testing.md, 6):
  `tests/ui/tests/memory.rs` declares the counting allocator; no other
  test there does.

## Increments

### 04.1 Scenes and their limits
Scope: ui.md 11.1, 11.2, 15 (`scene_marks`, `scene_bytes`).
Needs: skein 01.3 (the moved crates, whose fixtures' `Limits` it
  changes; nothing in skein-ui changes between skein 01.1 and 01.3).
Files: new `crates/skein-ui/src/scene.rs`; `crates/skein-ui/src/limits.rs`
  (`Limits`, two fields; jig's limits.rs:11-26 at the base);
  `crates/skein-ui/src/lib.rs`; every literal of skein-ui's `Limits`
  in skein (`rg -n 'Limits \{' crates/skein-ui crates/skein-ui-text
  testing/skein-fake-person examples/ui`; at jig's base the three
  fixtures' `LIMITS`: `crates/jig-view/src/tests/tree.rs:16`, moved to
  `src/tests/fixtures.rs`; `crates/jig-shell-text/src/tests/fixtures.rs:4`;
  `testing/jig-fake-person/src/tests.rs:31`; and the example's
  `examples/ui/view/src/limits.rs` if skein 10.1 has landed); new
  `src/tests/scene.rs`.
Shape: as the sketch. A mark's bytes (its text, hint, act and path
  commands, a command 1 byte plus its points' 4 bytes each) count
  against `scene_bytes`; marks, groups included, against `scene_marks`;
  a mark's tokens at most `styles_per_node`.
Test shape: `src/tests/scene.rs`: `a_mark_past_scene_marks_refuses_the_scene`,
  `bytes_past_scene_bytes_refuse_the_scene`,
  `a_refused_scene_keeps_no_mark`, `a_group_holds_the_marks_counted_after_it`,
  `too_many_tokens_on_a_mark_refuse_the_scene`. Each under 20 ms.
  Reference test:
  ```rust
  #[test]
  fn a_mark_past_scene_marks_refuses_the_scene() {
      let limits = Limits { scene_marks: 1, ..LIMITS };
      let mut scene = Scene::new(&limits);
      assert!(scene.rect(Point { x: 0, y: 0 }, Point { x: 4, y: 4 }, 0, &[]));
      assert!(!scene.circle(Point { x: 2, y: 2 }, 1, &[]));
      assert_eq!(scene.marks().len(), 0);
  }
  ```
Proof: unit, `heavy cargo nextest run -p skein-ui -p skein-ui-text -p
  skein-fake-person` once: the scene's tests pass and the crates whose
  `Limits` it changed build. Unit is the highest layer: no drawing code
  calls it until skein 13.4; skein 09.2 draws it as SVG.
Don't: add a colour, a width or a font to a mark.
Size: about 220 lines.
Removes: nothing.

### 04.2 The order a press takes an act
Scope: ui.md 11.3.
Needs: 04.1.
Files: `crates/skein-ui/src/scene.rs`; `src/tests/scene.rs`.
Shape: `Scene::act_at(&self, under: &[u32], point: Point, out: &mut
  List<u8>) -> bool`: `under` the marks' indices under the point,
  topmost first; writes the act's bytes into `out` (the scene's act
  with `point.x` then `point.y` as `u16` little-endian appended) and
  gives true, or false for none.
Test shape: a table test `a_press_takes_the_act_in_the_designs_order`,
  rows: an acting mark under a plain one; a plain mark in an acting
  group; nested groups, the nearest acting one; a press on nothing, the
  scene's act with the point `[.., 3, 0, 7, 0]` for `(3, 7)`; no act
  anywhere. Expected bytes literal. Under 20 ms.
Proof: unit, the test above, once: nothing calls it until skein 13.4,
  where the browser's hit test gives `under`.
Don't: hit-test geometry here.
Size: about 50 lines.
Removes: nothing.

### 04.3 Scenes printed for golden files
Scope: ui.md 17 (golden scenes).
Needs: 04.1.
Files: `crates/skein-ui/src/scene.rs` (`impl core::fmt::Display for
  Scene`); new `src/tests/golden/scene.txt`; `src/tests/scene.rs`.
Shape: the printed form of "Choices" above, one line per mark, `\n`
  after each.
Test shape: `a_scene_prints_one_line_per_mark`, building one scene of
  every kind of mark, a nested group, tokens, a hint and an act, and
  comparing with `golden/scene.txt` (written by hand). Under 20 ms.
Proof: unit, that test, once: skein 13.4's golden chart is its first
  user.
Don't: print in the drawing code's crates (they call this).
Size: about 60 lines.
Removes: nothing.

### 04.4 `tests/ui`: skein-ui's worst case against the counting allocator
Scope: ui.md 15, 17; testing.md 6 (`tests/ui`).
Needs: 04.1.
Files: new `tests/ui/{Cargo.toml (skein-ui-tests), src/lib.rs (a page
  at the limits), tests/memory.rs}`; skein's `Cargo.toml` (member);
  `crates/skein-ui/src/limits.rs` (`worst_case` counts one scene at its
  limits). The pattern: `tests/json/tests/memory.rs` (its `Counting`
  and `Meter`, `testing/skein-heap`).
Shape: `worst_case(&Limits)` adds a scene's capacity
  (`Scene::new`'s reservations at `scene_marks` and `scene_bytes`).
  `skein_ui_tests::full_page(&Limits, &mut Builder)` builds a tree with
  every node holding the most text the limits allow.
Test shape: `tests/ui/tests/memory.rs`: `the_view_holds_no_more_than_its_worst_case`
  (two builds and a diff of full pages at a fixture's limits, a scene
  filled, Markdown read at `markdown_depth`; the meter's peak at most
  `worst_case`) and `a_full_page_reaches_half_its_worst_case` (the
  peak at least half, testing.md 7). Each under 0.5 s.
Proof: world-level: the memory test above, the highest layer skein-ui
  reaches alone. Bound: 30 s.
Don't: run the counting allocator in another binary of `tests/ui`.
Size: about 40 lines of production code, 150 of tests.
Removes: nothing.

## Interface sketch

```rust
// crates/skein-ui/src/scene.rs: drawing code's output (ui.md, 11).
// The shell owns a Scene per drawing it draws; drawing code writes it.

/// Whole pixels from the box's top left.
pub struct Point { pub x: u16, pub y: u16 }
/// Where a text mark's point is.
pub enum Anchor { Start, Middle, End }
/// A path's commands.
pub enum Command { Move(Point), Line(Point), Quadratic(Point, Point), Cubic(Point, Point, Point), Close }
/// One mark, its bytes in the scene's store.
pub enum Mark {
    Group { offset: Point, marks: u32 },
    Line { from: Point, to: Point },
    Polyline { points: u32 },
    Path { commands: u32 },
    Rectangle { corner: Point, size: Point, radius: u16 },
    Circle { centre: Point, radius: u16 },
    Text { at: Point, anchor: Anchor },
}
pub struct Scene { /* marks, their bytes, refused */ }
impl Scene {
    pub fn new(limits: &Limits) -> Scene;
    pub fn clear(&mut self);
    pub fn refused(&self) -> bool;
    pub fn marks(&self) -> &[Mark];
    /// Each writer adds one mark with its tokens; false once refused.
    pub fn group(&mut self, offset: Point, marks: u32, styles: &[Style]) -> bool;
    pub fn line(&mut self, from: Point, to: Point, styles: &[Style]) -> bool;
    pub fn polyline(&mut self, points: &[Point], styles: &[Style]) -> bool;
    pub fn path(&mut self, commands: &[Command], styles: &[Style]) -> bool;
    pub fn rect(&mut self, corner: Point, size: Point, radius: u16, styles: &[Style]) -> bool;
    pub fn circle(&mut self, centre: Point, radius: u16, styles: &[Style]) -> bool;
    pub fn text(&mut self, at: Point, anchor: Anchor, bytes: &[u8], styles: &[Style]) -> bool;
    /// The last mark's hint and act; the scene's own act for a press elsewhere.
    pub fn hint(&mut self, bytes: &[u8]) -> bool;
    pub fn act(&mut self, bytes: &[u8]) -> bool;
    pub fn scene_act(&mut self, bytes: &[u8]) -> bool;
    pub fn act_at(&self, under: &[u32], point: Point, out: &mut List<u8>) -> bool;
}
/// The application's drawing code (ui.md, 11.1), routed by the kind's owner.
pub type Draw = fn(kind: DrawingKind, data: &[u8], width: u16, height: u16, scene: &mut Scene) -> bool;
```

## Questions to raise when reached

None known.

## Done when

`heavy cargo nextest run -p skein-ui -p skein-ui-tests` passes, and
`ls crates/skein-ui/src/tests/golden/scene.txt tests/ui/tests/memory.rs`
lists both.
