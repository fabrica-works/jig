# Session skein 09: the inbox, scenes as SVG, the mapping and the page part

`skein-ui-web` starts: the inbox, generic over the application's
reports; a model of the document's elements, printed as HTML; scenes
drawn as SVG into it; the default mapping of every role, state and
property to elements; the page part's pure half applying patches to the
model and turning the browser's events into acts; and its platform half,
built for wasm32 only, doing the same on the real document. It maps the
vocabulary skein 01 moved; skein 02, which needs this session, adds each
new role, state and property to the mapping and the page part with its
golden rows. Everything
but the platform half is tested on the host against golden HTML and
SVG; the platform half runs first in Chromium in skein 11.4.

## Read first

- skein's `docs/design/ui-web.md`, sections 3.2, 5, 5.1, 5.3, 6, 7, 8,
  13 and 14; `ui.md`, sections 7, 11.3, 11.4 and 16.1.
- This plan's `decisions.md`, 7; `code-organization.md`, 1, 2, 3, 5 and
  6; `findings/toolchain.md`, 1.
- skein's `programming-model.md`, sections 2.1 (`unsafe`) and 10.2
  (ordinary Rust); `testing-strategy.md`, section 8.

## What the design asks

Each part is ordinary Rust, usable alone, its pure half on the host and
wasm32, its platform half on wasm32 only (ui-web.md, 4.1, 5.1). The
inbox queues reports and calls the turn only when none runs, within the
callback that raised it (5.1, 5.3). The page part maps each node to one
element by section 6's table, applies patches in place (7), refuses a
patch naming a node it does not hold as a panic (5.3), draws a scene as
SVG with its fallback kept (8), reports acts by `data-node` (6, 7), and
holds Escape, menus and a select's choice put back (7).

## Choices this plan makes

- **A document model in `skein_ui_web::dom`:** `Element { tag,
  attributes, children }` and text, printed as HTML with attributes in
  a fixed order (`data-node` first, then the mapping's order). The SVG
  writer, the mapping and the page part's pure half all write into it;
  the platform half replays what the pure half did as `page::Op`s on
  the real document. So the host tests see what the browser gets.
- **The look the parts take** is `skein_ui_web::Look`: the names
  (`skein_ui::look::Names`, skein 03.1), each owner's pictures' files by
  number, and the drawing code (`skein_ui::scene::Draw`, skein 04.1).
  Elements of its own and components join it in skein 13.1 and 13.5.
- **The roles a find asks Chromium for** are the mapping's:
  `mapping::accessible(Role) -> &'static [&'static [u8]]`, used by the
  face (skein 11.1) and checked against Chromium (skein 14.1).
- **What waits for skein 02:** a focus request (02.1), Escape (02.3)
  and a select's choice (02.4) in the page part, and each new row of
  the mapping; this session maps what `skein_ui` holds at skein 01.3.

## Rules that bind it

- **`skein-ui-web` forbids `unsafe`** (`#![forbid(unsafe_code)]`), and
  names no application.
- **The pure half and the platform half do the same:** the platform
  half executes `page::Op`s and reads events; every decision (which
  element, which attribute, which act) is the pure half's, tested on
  the host.
- **Nothing inline:** no `style` attribute, no inline script, no
  `innerHTML`; text is set as text (ui-web.md, 4.4, 6): `rg -n
  'innerHTML|set_inner_html|"style"' crates/skein-ui-web/src` prints
  nothing at each increment's end.

## Increments

### 09.1 The crate and the inbox
Scope: ui-web.md 5.1 (the inbox), 5.3 (no re-entry), 13.
Needs: skein 01.3 (the moved crates; nothing in skein-ui changes
  between skein 01.1 and 01.3).
Files: new `crates/skein-ui-web/{Cargo.toml, src/lib.rs, src/inbox.rs,
  src/tests.rs, src/tests/fixtures.rs}`; skein's `Cargo.toml`
  (`[workspace.dependencies]` `skein-ui-web`, and `wasm-bindgen =
  "=0.2.126"`, `js-sys = "=0.3.103"`, `web-sys = "=0.3.103"` with no
  features yet; findings/toolchain.md, 1).
Shape: `Inbox<R>` as the sketch; `lib.rs` re-exports
  `wasm_bindgen::prelude::wasm_bindgen` on wasm32 (ui-web.md, 3.1).
Test shape: `src/tests.rs`: `a_report_raised_in_a_turn_waits_for_that_turn`
  (the turn raises one; the turn is called once and takes both, in
  order), `a_report_outside_a_turn_calls_it_at_once`,
  `reports_keep_their_order`. Reference test:
  ```rust
  #[test]
  fn a_report_raised_in_a_turn_waits_for_that_turn() {
      let inbox: Rc<Inbox<u8>> = Inbox::new();
      let seen = Rc::new(RefCell::new(Vec::new()));
      let calls = Rc::new(Cell::new(0));
      let (s, c) = (Rc::clone(&seen), Rc::clone(&calls));
      inbox.start(Box::new(move |inbox: &Inbox<u8>| {
          c.set(c.get() + 1);
          while let Some(report) = inbox.next() {
              if report == 1 { inbox.raise(2); }
              s.borrow_mut().push(report);
          }
      }));
      inbox.raise(1);
      assert_eq!((seen.borrow().clone(), calls.get()), (vec![1, 2], 1));
  }
  ```
  Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p skein-ui-web` once, and the
  wasm32 build `heavy cargo build -p skein-ui-web --target
  wasm32-unknown-unknown` once. The reference shell runs it in skein
  11.3.
Don't: add a part's platform half here.
Size: about 80 lines.
Removes: nothing.

### 09.2 The document model, and scenes as SVG
Scope: ui-web.md 8; ui.md 11.4.
Needs: skein 04.1 (`skein_ui::scene`) and skein 03.1 (the look's names).
Files: `crates/skein-ui-web/src/{dom.rs, svg.rs, look.rs}`; new
  `src/tests/golden/scene.svg`; `src/tests/svg.rs`.
Shape: `dom::{Element, Child, print}`; `svg::draw(scene: &Scene, names:
  &[Names<'_>], width: u16, height: u16) -> Element`: an `<svg
  aria-hidden="true" width height viewBox>`, one element per mark
  (ui-web.md, 8's table), tokens as `class`, a hint as `<title>`, an act
  as `data-act` (hex); a refused scene gives no element.
Test shape: `src/tests/svg.rs`: `a_scene_draws_as_its_golden_svg` (one
  scene of each mark, compared with `golden/scene.svg`, by hand);
  `a_refused_scene_draws_nothing`; `a_marks_text_is_escaped_never_markup`
  (`<b>` in a text mark prints `&lt;b&gt;`). Each under 20 ms.
Proof: unit; skein 09.4 draws drawings through it, and skein 13.4 shows
  the example's chart in Chromium.
Don't: write a colour, a width or a font.
Size: about 200 lines.
Removes: nothing.

### 09.3 The mapping
Scope: ui-web.md 6 (the table and the attributes); ui.md 16.1.
Needs: skein 03.1 (the look's names).
Files: `crates/skein-ui-web/src/mapping.rs`; `src/tests/mapping.rs`;
  new `src/tests/golden/mapping.html`.
Shape: `mapping::element(made: &Made, look: &Look<'_>, prefix: &[u8])
  -> Element` (`made` as the diff gives a node; `prefix` the digest
  directory's path for pictures), matched whole over `Role` and
  `State` as skein 01 moved them; `mapping::accessible(Role)`, the
  accessible roles Chromium computes for each role's elements
  (`textbox` for a text field, …; `spinbutton`, a number's, comes with
  skein 02.1's entry kinds).
Test shape: `src/tests/mapping.rs`: a table test
  `each_role_maps_to_its_element`, one row per role, its expected HTML
  as a literal, each failing row named; the same for
  `each_state_and_property_maps_to_its_attribute`; one golden page
  `every_node_maps_as_the_golden_page` printing a tree with every role,
  state and property (`golden/mapping.html`, by hand). Each under 20 ms.
Proof: unit; the face finds by `accessible` in skein 11.1, and skein
  14.1 checks it against Chromium. Nothing above the mapping runs until
  09.4 applies patches through it.
Don't: give an element an id but the parts' own `skein-ui-…`
  (ui-web.md, 6).
Size: about 300 lines.
Removes: nothing.

### 09.4 The page part's pure half: patches applied
Scope: ui-web.md 7 (patches applied in place, the focus, a removal
  taking the focus), 5.3 (a patch naming no held node is a panic), 8
  (a drawing's scene and fallback).
Needs: 09.2, 09.3.
Files: `crates/skein-ui-web/src/page.rs`; `src/tests/page.rs`; new
  goldens in `src/tests/golden/`.
Shape: `Page::new(look, prefix)`; `Page::apply(&mut self, patches:
  &[Patch], ops: &mut Vec<Op>)`: insert, remove, move, text, attribute
  and value applied to its model, keyed by node id, emitting the `Op`s
  the platform half replays; a drawing drawn through `look.draw` and
  `svg::draw` at the size the last `Page::resized(node, width, height)`
  gave (no scene before the first `resized`: the fallback shows),
  `data-drawn` while drawn; the nearest remaining ancestor that can take
  the focus when a removal takes it, else the main, as an `Op::Focus`;
  `Page::html(&self) -> String` for goldens; `Page::links(&self) ->
  &[NodeId]` and `set_href(node, path)`.
Test shape: `src/tests/page.rs`, one test per kind of patch and per
  rule, each applying a fixture view's patches (two builds through
  `skein_ui::View`) and comparing `Page::html` with a golden:
  `an_insert_builds_the_golden_page`, `a_remove_takes_its_subtree`,
  `a_move_keeps_the_element`, `a_value_patch_writes_the_field`,
  `a_removal_taking_the_focus_moves_it_to_an_ancestor`,
  `a_patch_naming_no_held_node_panics`. Each under 20 ms.
Proof: joint: skein-ui's real view and diff feed the real page part
  (host half), against golden HTML. The browser is 09.5's and 11.4's.
Don't: replace an element an attribute patch can update.
Size: about 300 lines.
Removes: nothing.

### 09.5 Acts, Escape, menus, and the platform half
Scope: ui-web.md 7 (acts, shortcuts, menus and disclosures; a dialog
  the browser closes shown again), 6 (one listener per kind on the mount
  element, by `data-node`). Escape (skein 02.3) and a select's change
  (skein 02.4) come with their vocabulary.
Needs: 09.4.
Files: `crates/skein-ui-web/src/page.rs` (`Page::act`), new
  `src/page/web.rs` (`#[cfg(target_arch = "wasm32")]`);
  `crates/skein-ui-web/Cargo.toml` (web-sys features the half uses);
  `src/tests/page.rs`.
Shape: `Page::act(&mut self, event: Event, ops: &mut Vec<Op>) ->
  Option<(NodeId, Act)>`: `Event::{Click { node, modifier }, Input {
  node, value, composing }, Enter { node, shift, composing }, Change {
  node } (a check's or a choice's: `Toggle`), Cancel { dialog }, Key { key, modifier, in_field }}` to
  ui-web.md 7's acts (a plain click on a link a `Press`; a dialog's
  cancel no act, its closing prevented; a shortcut the first bound,
  enabled node in preorder naming the key);
  `page::web::Dom` executes `Op`s with web-sys and turns the mount
  element's listeners into `Event`s, reporting acts to the inbox
  through the application's wrapping function.
Test shape: `src/tests/page.rs`: a table test `each_event_gives_its_act`
  (rows per ui-web.md 7's bullet, expected `Option<(NodeId, Act)>`
  literal); `a_dialogs_cancel_keeps_it_open`;
  `a_shortcut_with_no_modifier_gives_none_while_a_field_has_focus`. Each
  under 20 ms.
Proof: unit on the host, and the wasm32 build with the lints:
  `heavy cargo clippy -p skein-ui-web --target wasm32-unknown-unknown
  -- -D warnings` once. Its first run in a browser is skein 11.4's
  story, which connects it.
Don't: write `unsafe` or a `js_sys::eval`; set `innerHTML`.
Size: about 400 lines.
Removes: nothing.

## Interface sketch

```rust
// crates/skein-ui-web/src/inbox.rs: the inbox (ui-web.md, 5.1). The
// shell makes it at start; each part is given it with a function
// wrapping its report into R. It ends with the page.
pub struct Inbox<R> { /* reports, whether a turn runs, the turn */ }
/// The application's turn, which the inbox calls.
pub type Turn<R> = Box<dyn FnMut(&Inbox<R>)>;
impl<R> Inbox<R> {
    pub fn new() -> Rc<Inbox<R>>;
    /// The application's turn, given once at start.
    pub fn start(&self, turn: Turn<R>);
    /// Queue a report; call the turn now unless one runs.
    pub fn raise(&self, report: R);
    /// The turn's next report, in order.
    pub fn next(&self) -> Option<R>;
}

// crates/skein-ui-web/src/look.rs (09.2) and page.rs (09.4): the look
// the parts take, and the page part's pure half.
pub struct Look<'a> {
    pub names: &'a [Names<'a>],
    /// Each owner's pictures' files, indexed by `Picture.number`.
    pub pictures: &'a [(Owner, &'a [&'a [u8]])],
    pub draw: Draw,
}
/// What the platform half does to the real document, in order.
pub enum Op {
    Insert { parent: NodeId, before: Option<NodeId>, element: Element },
    Remove { node: NodeId },
    Move { node: NodeId, parent: NodeId, before: Option<NodeId> },
    Replace { node: NodeId, element: Element },
    Attributes { node: NodeId, element: Element },
    Text { node: NodeId, element: Element },
    Value { node: NodeId, value: Box<[u8]> },
    Choice { node: NodeId, checked: bool },
    Href { node: NodeId, path: Box<[u8]> },
    Focus { node: NodeId },
}
pub struct Page<'a> { /* the model, by node id */ }
impl<'a> Page<'a> {
    pub fn new(look: Look<'a>, prefix: &'a [u8]) -> Page<'a>;
    pub fn apply(&mut self, patches: &[Patch], ops: &mut Vec<Op>);
    pub fn resized(&mut self, node: NodeId, width: u16, height: u16, ops: &mut Vec<Op>);
    pub fn act(&mut self, event: Event, ops: &mut Vec<Op>) -> Option<(NodeId, Act)>;
    pub fn links(&self) -> &[NodeId];
    pub fn set_href(&mut self, node: NodeId, path: &[u8], ops: &mut Vec<Op>);
    pub fn html(&self) -> String;
}
```

## Questions to raise when reached

- **The menu button's own words.** ui-web.md 7 names the menu's button
  by the menu's name; if a name alone reads ambiguously to the face
  (the button and the menu share it), the designer says which a find
  of the role `menu` lands on.

## Done when

`heavy cargo nextest run -p skein-ui-web` passes, and `heavy cargo
clippy -p skein-ui-web --target wasm32-unknown-unknown -- -D warnings`
passes once.
