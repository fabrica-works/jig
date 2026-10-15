# Session skein 13: the example whole

The UI example carries one of each thing the parts carry (ui-web.md,
14.3; examples.md, 9): history, the timer, the title and the closing
warning, with a note's page whose region is an `<article>` of its own;
the whole form, with its entry kinds, its required title and its label
select; the dialog that Escape answers, and a focus request; the chart,
drawn as SVG, its fallback read, a bar pressed through the harness; and
the body's text area as a component that grows, with its one JavaScript
snippet, under the strict policy. Each increment adds its story to the
example's tier.

## Read first

- skein's `docs/design/examples.md`, sections 9.1, 9.2, 9.4 and 9.5;
  `ui-web.md`, sections 3.3, 4.1, 6.1, 7, 8, 9, 12, 14 and 14.3;
  `ui.md`, sections 6.1, 11 and 12.
- This plan's `decisions.md`, 7 and "For the user" (the snippet, the
  page); `code-organization.md`, 1, 3 and 6.
- skein's `programming-model.md`, sections 2.1 and 10.

## What the design asks

examples.md 9.1's pages in full; ui-web.md 3.3's steps, each once: the
reference shell (1), a stylesheet after `skein-ui.css` (2), the codec
with notes under `/notes/` and `/bundle/` the prefix (3), the page the
bundler's (4, this plan's choice), an element override and a component
(6), one snippet of at most 10 lines (7). History: an address a step
shows is a new entry or replaces the current one; back and forward a
place gone to, then the entry's scroll (ui-web.md, 9). The title is the
main's first level-1 heading, ` · `, the name (ui-web.md, 7). A
component reports only its node's role's acts (ui-web.md, 6.1).

## Choices this plan makes

- **The page is the bundler's** (ui-web.md, 3.3 step 4 and 14.3, "the
  plan's choice"; decisions.md, For the user).
- **The example's stylesheet** is `examples/ui/shell-web/notes.css`,
  given to the bundler by `scripts/check.sh` with `--stylesheet`.
- **The snippet** is `examples/ui/shell-web/grow.js`, one exported
  function `grow(element)` setting the element's height from its
  `scrollHeight`, imported with `#[wasm_bindgen(module = "/grow.js")]`.
- **The harness's own steps** (skein 11.2's `harness::Step`):
  `Key(skein_browser::Key)` in 13.3, the kit's key on the page;
  `PressAt { find, at: Point }` in 13.4, the face finding the drawing's
  figure and the kit pressing at the point (skein 07.2). Both are steps
  of a `Script`, inside a `run`.
- **The module's size** is kept in `examples/ui/shell-web/size.txt`,
  one line, the `web` build's bytes; nothing in the gate reads it (a
  guide, ui-web.md 4.2).

## Rules that bind it

- **One of each:** no second component, snippet, element override or
  drawing kind in the example (ui-web.md, 14.3): `rg -c
  'wasm_bindgen\(module' examples/ui` prints 1 after 13.5.
- **Nothing inline under the strict policy;** each story fails on a
  policy violation (skein 11's rule).
- **The fallback says everything** (ui.md, 11.2): each act on the chart
  has an ordinary node doing the same, the choice group.

## Increments

### 13.1 History, the timer, the title, and a note's page
Scope: ui-web.md 5.1 (history, the timer, title and closing), 6.1 (an
  element of its own), 7 (going to another page; the title), 9;
  examples.md 9.1 (a note's page).
Needs: skein 12.3 (notes made by an add).
Files: new `crates/skein-ui-web/src/{history.rs, history/web.rs,
  timer.rs, timer/web.rs, title.rs, title/web.rs}`;
  `crates/skein-ui-web/src/{look.rs (Look.elements), mapping.rs (the
  override keeps the role: ARIA's role added)}`;
  `examples/ui/view/src/{view.rs, places.rs}` (the note's page, its
  `notes-card` region, the link back, the missing page);
  `examples/ui/shell-web/{src/lib.rs, notes.css}`; `scripts/check.sh`
  (`--stylesheet`); `tests/ui-web/tests/browser_example.rs`.
Shape: `history::History` (shows a path new or in place, keeps the
  scroll in the entry's state, reports a place gone to); `timer::Timer`
  (one deadline); `title::set(heading, name, language, loses)`;
  `Look.elements: &[(Owner, Role, Style, &'static str)]`: a
  `Role::Region` node carrying the style token `notes-card` is an
  `<article role="region">`, named; links' `href`s set
  after each apply from `State::address`.
Test shape: host: `src/tests/history.rs`:
  `a_place_gone_to_is_reported_as_a_path`; `src/tests/mapping.rs`:
  `an_override_keeps_the_nodes_role_and_name`; view:
  `a_notes_page_shows_its_title_as_the_heading`,
  `an_unknown_note_says_there_is_none`; story
  `a_person_opens_a_note_and_comes_back_by_its_link` (on the note's
  page, the document's title reads `<the note's title> · Notes`, from
  the snapshot). Step tests
  under 20 ms; the story about 3 s.
Proof: live, `heavy --gate scripts/check.sh` once: the story passes.
  Bound: 15 minutes.
Don't: navigate the page on a plain press of a link (ui-web.md, 7).
Size: about 450 lines.
Removes: nothing.

### 13.2 The form whole
Scope: examples.md 9.1 (the form), 9.5 (a required title refused in
  words; a label chosen); ui-web.md 14 (golden pages of the example).
Needs: 13.1; skein 02.1 (entry kinds, descriptions), 02.4 (select,
  option), 02.5 (required).
Files: `examples/ui/view/src/{state.rs, view.rs}` (body, due date,
  minutes, label select of idea, task and reference, the clear button;
  invalid fields with their descriptions); `examples/ui/server/src/lib.rs`
  (a note's fields); new `tests/ui-web/tests/pages.rs` and goldens
  `tests/ui-web/golden/{list.html, note.html}`;
  `tests/ui-web/tests/browser_example.rs`.
Shape: minutes read by `skein_ui::number`, invalid when not one; the
  due date `YYYY-MM-DD`, invalid otherwise; an empty title refused in
  words (the field invalid, its description saying why).
Test shape: view: `an_empty_title_is_refused_in_words`,
  `minutes_that_are_not_a_number_are_invalid`, `a_label_is_chosen_by_its_option`;
  `tests/ui-web/tests/pages.rs`: `the_list_page_maps_as_its_golden_html`
  (the view's patches through the page part's host half), by hand;
  story `a_person_fills_each_entry_kind_and_chooses_a_label`. Step tests
  under 20 ms; the story about 3 s.
Proof: live, `heavy --gate scripts/check.sh` once: the story and the
  golden pages pass. Bound: 15 minutes.
Don't: check a field's words in the shell (the view's, ui.md 6).
Size: about 250 lines.
Removes: nothing.

### 13.3 The dialog, Escape, and a focus request
Scope: examples.md 9.1 (the dialog; the focus after an add or a
  discard); ui.md 6.1 (Escape); ui-web.md 7 (Escape, a focus request).
Needs: 13.2; skein 02.1 (a focus request), 02.3 (Escape).
Files: `examples/ui/view/src/{state.rs, view.rs}`;
  `testing/skein-fake-person-browser/src/harness.rs` (`Step::Key`);
  `tests/ui-web/tests/browser_example.rs`.
Shape: clearing a form holding words opens a dialog with `Discard` and
  `Keep`, `Keep` naming `Shortcut::ESCAPE`; an add or a discard moves
  the title's focus request count.
Test shape: view: `clearing_words_asks_in_a_dialog`,
  `an_add_moves_the_focus_to_the_title`; story
  `escape_in_the_dialog_keeps_the_draft` (the kit's `Key::Escape`; the
  dialog gone, the words still in the title, the focus on it, from the
  snapshot's focused state). Step tests under 20 ms; the story about
  3 s.
Proof: live, `heavy --gate scripts/check.sh` once: the story passes.
  Bound: 15 minutes.
Don't: close the `<dialog>` by the browser's own cancel (ui-web.md, 7).
Size: about 120 lines.
Removes: nothing.

### 13.4 The chart
Scope: examples.md 9.1 (the complementary landmark), 9.4 (the drawing);
  ui.md 11; ui-web.md 8, 14.2 (a mark pressed through the harness).
Needs: 13.2; skein 07.2 (a press at a point); skein 04.2 and 04.3
  (the press order, the printed scene); skein 06.3 (the tree face's
  acts by role).
Files: new `examples/ui/view/src/draw.rs` and golden
  `examples/ui/view/src/tests/golden/chart.txt`;
  `examples/ui/view/src/view.rs` (the drawing, its fallback list, the
  choice group: all notes or one label's);
  `examples/ui/shell-web/src/lib.rs` (the look's `draw`);
  `examples/ui/shell-web/notes.css` (`notes-bar`);
  `crates/skein-ui-web/src/{page.rs, page/web.rs}` (a press in a
  scene: `Event::Scene { drawing, under, point }` to `Act::Drawing`
  through `Scene::act_at`; the platform half keeps each drawing's mark
  elements in order and maps `elementsFromPoint`'s elements to their
  indices); `testing/skein-fake-person-browser/src/harness.rs`
  (`Step::PressAt`);
  `tests/ui-web/tests/browser_example.rs`.
Shape: `draw::draw(kind, data, width, height, scene) -> bool`: a
  `rect` and a `text` per label, `notes-bar`, each bar's act its
  label's number; the data the three counts and their words.
Test shape: view: `the_chart_draws_as_its_golden_scene` (through
  `Scene`'s printed form, skein 04.3, by hand);
  `a_bars_act_shows_its_labels_notes`; story
  `a_press_on_a_bar_shows_its_labels_notes` (the harness presses the
  `task` bar at its point; the list shows only tasks, as the choice
  `Tasks` does) and `the_charts_fallback_is_read_by_its_name`. Step
  tests under 20 ms; each story about 3 s.
Proof: live, `heavy --gate scripts/check.sh` once: the two stories
  pass. Bound: 15 minutes.
Don't: put a word only in the scene (its fallback says it).
Size: about 200 lines.
Removes: nothing.

### 13.5 The component and its snippet
Scope: ui.md 12; ui-web.md 3.3 (steps 6, 7), 4.1 (the `web` profile),
  6.1 (a component); examples.md 9.4.
Needs: 13.2 (the body's text area).
Files: `crates/skein-ui-web/src/{look.rs (Look.components), page.rs and
  page/web.rs (a host element, insert, patch, remove, its reports)}`;
  new `examples/ui/shell-web/{src/grow.rs, grow.js, size.txt}`;
  `examples/ui/view/src/view.rs` (`notes-grow` on the body); skein's
  `Cargo.toml` (`[profile.web]`: inherits `release`, `opt-level = "s"`,
  `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`,
  `overflow-checks = true`, `strip = true`);
  `tests/ui-web/tests/browser_example.rs`.
Shape: `Look.components: &[(Owner, Role, Style)]` on every target,
  which nodes the page part gives a host; on wasm32 only, the shell's
  table of constructors in the same order, `&[fn() -> Box<dyn
  page::web::Component>]`, and `trait Component { fn insert(&mut self,
  host: &web_sys::HtmlElement, made: &Made, report: Rc<dyn Fn(Act)>);
  fn patch(&mut self, made: &Made); fn remove(&mut self); }`; the page
  part keeps the host with its `data-node`, and leaves events inside it
  to the component. `grow.js` at most 10 lines.
Test shape: host: `src/tests/page.rs`:
  `a_component_node_keeps_its_host_and_data_node` (the pure half's
  model); story `the_body_grows_with_its_words_and_reports_them` (the
  text area's box taller after three lines, from the kit's boxes; the
  note added with the body's words). Step tests under 20 ms; the story
  about 3 s.
Proof: live, `heavy --gate scripts/check.sh` once: the story passes;
  and `heavy cargo build -p
  skein-ui-example-shell-web --target wasm32-unknown-unknown --profile
  web` once, its module's bytes written to `size.txt`. Bound: 10
  minutes.
Don't: add a second snippet; let the component report an act its
  text area does not take.
Size: about 250 lines.
Removes: nothing.

## Questions to raise when reached

- **The snippet's height.** `grow.js` sets `element.style.height`
  through the CSSOM, which `style-src 'self'` allows (the policy refuses
  a `style` attribute in markup and `setAttribute("style", …)`, not the
  CSSOM). If the story's harness reports a policy violation for it, the
  increment stops with a question to the designer, quoting ui-web.md
  6.1's "its styles from the origin's stylesheets".

## Done when

`heavy --gate scripts/check.sh` passes once with the stories of 13.1 to
13.5 in its browser suite, and `wc -l < examples/ui/shell-web/grow.js`
prints at most 10.
