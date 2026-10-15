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
component reports only its node's role's acts (ui-web.md, 6.1). The
closing warning is the window's `onbeforeunload`, set only while closing
loses requests (ui-web.md, 5.1); a go accepts it and reports `Warned`,
and the kit dismisses any other native dialog as `Trouble` (browser.md,
6 and 7). A native date or time field takes no inserted text: the face
types its value by the kit's `keys`, in its parts' order under the
kit's locale, `en-US` (ui-web.md 14.2; browser.md 3 and 6).

## Choices this plan makes

- **The page is the bundler's** (ui-web.md, 3.3 step 4 and 14.3, "the
  plan's choice"; decisions.md, For the user).
- **The example's stylesheet** is `examples/ui/shell-web/notes.css`,
  given to the bundler by `scripts/check.sh` with `--stylesheet`.
- **The snippet** is `examples/ui/shell-web/grow.js`, one exported
  function `grow(element)` setting the element's height from its
  `scrollHeight`, imported with `#[wasm_bindgen(module = "/grow.js")]`.
- **The harness's own steps** (skein 11.2's `harness::Step`):
  `Title(words)` in 13.1, the kit's query of the page's root
  (Chromium's `RootWebArea`) named exactly by the document's title;
  `Warned(bool)` in 13.1, whether the person's last go met its page's
  closing warning, which the face records from the kit's
  `Event::Warned` of that go (skein `506b940`, browser.md 6);
  `Focused(find)` in 13.3, the found node's `Seen.states.focused`
  (`testing/skein-browser/src/boundary.rs:129`); `Key(skein_browser::
  boundary::Key)` in 13.3 (`boundary.rs:71`; the crate's `lib.rs:16`
  re-exports only `Below`, `Down`, `Event`, `Request`), the kit's key
  on the page; `PressAt { find, at: skein_browser::boundary::Point }`
  in 13.4 (`u32`s, `boundary.rs:101`, not the scene's `u16` point), the
  face finding the drawing's figure and the kit pressing at the point
  (skein 07.2); `Taller { find, words }` in 13.5 (ruling 132), beside
  `Type` (`harness/script.rs:18`): the found node's box from a query
  with `boxes: true` (the face asks with `false`, `face.rs:151`), its
  height kept, then `words` typed as `Type` types them, then the box
  looked at again at the poll's rate until it is taller, the step
  failing past 1 s with both heights. Each is a step of a `Script`, inside
  a `run`, reaching the kit through a method of `BrowserFace`, whose
  page, requests and token are private (`face.rs:17-47`). The harness
  writes its snapshot only on a failure (its `snapshot`, `harness.rs:180`), and the
  snapshot prints no focus (`browser.rs:926-947`): no story reads it.
- **The module's size** is kept in `examples/ui/shell-web/size.txt`,
  one line, the `web` build's bytes:
  `target/wasm32-unknown-unknown/web/skein_ui_example_shell_web.wasm`
  as 13.5's Proof builds it (950,464 bytes at skein `9098df1`), not the
  bundle's `_bg.wasm`; nothing in the gate reads it (a guide, ui-web.md
  4.2).

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
Scope: ui-web.md 5.1 (history, the timer, title and closing, the
  closing warning), 6.1 (an element of its own), 7 (going to another
  page; the title), 9, 14.2 (the face's go, the harness's check of the
  warning); browser.md 6 (a go answers its page's closing warning) and
  7 (a native dialog dismissed as trouble); examples.md 9.1 (a note's
  page), 9.2 (the warning while an add is pending), 9.5. Skein
  `506b940` added the warning's and the dialogs' text, answering 13.1's
  question of 2026-10-15T14:56:13Z.
Needs: skein 12.3 (notes made by an add).
Files: new `crates/skein-ui-web/src/{history.rs, history/web.rs,
  timer.rs, timer/web.rs, title.rs, title/web.rs}`;
  `crates/skein-ui-web/src/{look.rs (Look.elements), mapping.rs (the
  override keeps the role: ARIA's role added)}`;
  `examples/ui/view/src/{view.rs, places.rs}` (the note's page, its
  `notes-card` region, the link back, the missing page);
  `examples/ui/shell-web/{src/lib.rs, notes.css}`; `scripts/check.sh`
  (`--stylesheet`); `tests/ui-web/tests/browser_example.rs`;
  `crates/skein-ui-web/src/lib.rs` (`pub mod history; pub mod timer;
  pub mod title;`, beside 18-21), `src/tests.rs` (`mod history;`,
  beside 53-56), `Cargo.toml` (web-sys `History`, `PopStateEvent`,
  `BeforeUnloadEvent`, line 16), `src/tests/fixtures.rs` (its `Look`
  literal, 101, gains `elements`); the view's `src/state.rs` (`address`,
  145, maps only notes' bindings: the link back gets its place) and
  `src/limits.rs` (`styles_per_node` 0 to 1, line 12, else the
  `notes-card` style panics at skein-ui's `builder.rs:233`);
  `testing/skein-fake-person-browser/src/{harness.rs, face.rs}`
  (`Step::Title`, `Step::Warned`, the face's record of its last go);
  the kit: `testing/skein-browser/src/{boundary.rs (`Event::Warned {
  op }` beside `Done`, 52; `Trouble::Dialog`, 160-165), browser.rs
  (`Page.javascriptDialogOpening` in `notification`, 1281; the answer
  through `queue_page`, 384), params.rs (`Params::Dialog { accept }`)}`;
  its tests: `tests/browser/src/{tests.rs, referee.rs (`Warned` before
  its go's terminal, 39-48), serve.rs (a page `/leave`, 25-36)}`,
  `tests/browser/tests/{support/real.rs (two scenarios, 128-139, and
  their `Trouble` arms, 595-608), browser_real.rs}`.
Shape: `history::History` (shows a path new or in place, keeps the
  scroll in the entry's state, reports a place gone to); `timer::Timer`
  (one deadline); `title::set(heading, name, language, loses)`;
  `Look.elements: &[(Owner, Role, Style, &'static str)]`: a
  `Role::Region` node carrying the style token `notes-card` is an
  `<article role="region">`, named; links' `href`s set after each
  apply from `State::address`, as the shell already does
  (`examples/ui/shell-web/src/lib.rs:121-124`): kept, not added.
  The closing warning: `title/web.rs` calls
  `window.set_onbeforeunload(Some(..))` while `loses` (the view's
  `State::closing_loses`, `state.rs:208`) and `set_onbeforeunload(None)`
  otherwise (web-sys `gen_Window.rs:1839`); its handler asks the
  browser to warn as the WIP's listener does, now as the window's one
  handler. The kit, on
  `Page.javascriptDialogOpening` for a page: when its `type` is
  `beforeunload` and the page has a go in flight (at most one,
  `admit_op`, 583-591), it sends `Page.handleJavaScriptDialog` with
  `accept: true` and pushes `Event::Warned { op }` of that go; for any
  other dialog it sends `accept: false` and reports
  `Trouble::Dialog`, its text `<type>: <message>`, cut to
  `Limits::text`. The answer is a page command like any other, counted
  in flight; its reply is dropped. The face sets its record to `false`
  at each `Doing::Go` and `Doing::Reload`, and to `true` on `Warned` of
  its acting go; `Step::Warned(expected)` fails the story, naming both,
  when the record differs.
Test shape: host: `src/tests/history.rs`:
  `a_place_gone_to_is_reported_as_a_path`; `src/tests/mapping.rs`:
  `an_override_keeps_the_nodes_role_and_name`; view:
  `a_notes_page_shows_its_title_as_the_heading`,
  `an_unknown_note_says_there_is_none`; story
  `a_person_opens_a_note_and_comes_back_by_its_link` (on the note's
  page, the document's title reads `<the note's title> · Notes`, by
  the harness's `Title` step). The warning, in
  `tests/ui-web/tests/browser_example.rs`, beside
  `a_reload_keeps_a_pending_add_made_once` and with its `Setup`:
  - `a_reload_with_an_add_pending_meets_the_closing_warning`: see the
    `Title` field; `StopServer`; type `Groceries`; press `Add`; pause
    100 ms; `StartServer`; `Reload`; `Warned(true)`;
  - `a_reload_once_the_add_is_made_meets_no_closing_warning`, with the
    example's default `Setup`: see the `Title` field; type
    `Groceries`; press `Add`; see the link `Groceries`, the add made and
    nothing pending (`add_answered`, `state.rs:256-272`, clears the
    pending slot in the step that accepts the note); `Reload`;
    `Warned(false)`. The press on `Add` is the input Chromium asks
    before it shows a warning (browser.md 6).

  The kit's, in `tests/browser/src/tests.rs`, each event given as
  `trouble_is_reported_without_judgment` gives its own (221-253):
  `a_go_accepts_its_pages_closing_warning` (a reload, a
  `beforeunload` dialog: the answer `accept: true`, `Warned` then
  `Done`); `a_dialog_outside_a_go_is_dismissed_as_trouble` (a confirm,
  and a `beforeunload` with no go: each `accept: false` and a
  `Trouble::Dialog`, `confirm: Delete it?` the first's text). In
  `browser_real.rs`, on `/leave` (a button `Hold` setting
  `onbeforeunload` and a heading `Held`; a button `Ask` writing
  `Answer: ` and `confirm('Delete it?')`'s answer into the heading):
  `chromium_accepts_the_closing_warning_on_a_reload` (press `Hold`,
  see `Held`, reload: `Warned`, `Done`, the heading back; press `Hold`
  again, close the page: `Closed`, and no `Warned` or `Trouble`);
  `chromium_dismisses_a_confirm_as_trouble` (press `Ask`: the press
  `Done`, a `Trouble::Dialog` naming `confirm: Delete it?`, the heading
  `Answer: false`). Step tests under 20 ms; each story about 3 s; each
  real test about 2 s.
Proof: live, `heavy --gate scripts/check.sh` once: the stories and the
  kit's tests pass, `a_reload_keeps_a_pending_add_made_once` among
  them, unchanged. Bound: 15 minutes.
Don't: navigate the page on a plain press of a link (ui-web.md, 7);
  add a request that answers a dialog, or accept any dialog but a
  `beforeunload` during a go of its page (later.md); close a page by
  `Page.close`, which runs `beforeunload` (the kit's `Target.closeTarget`
  stays, 767); register the warning by `addEventListener`, which the
  restart part's `set_onbeforeunload(None)` (14.2) would not clear;
  change `a_reload_keeps_a_pending_add_made_once`; teach the fake
  browser in `tests/browser/src/world.rs` dialogs: the step tests give
  the events, and the fake's scenarios open none.
Size: about 700 lines.
Removes: nothing.

### 13.2 The form whole
Scope: examples.md 9.1 (the form), 9.5 (a required title refused in
  words; a label chosen); ui-web.md 14 (golden pages of the example),
  14.1 and 14.2 (a date and a time typed by the kit's keys); browser.md
  3 and 6 (`keys`).
Needs: 13.1; skein 02.1 (entry kinds, descriptions), 02.4 (select,
  option), 02.5 (required).
Files: `examples/ui/view/src/{state.rs, view.rs}` (body, due date,
  minutes, label select of idea, task and reference, the clear button;
  invalid fields with their descriptions); `examples/ui/server/src/lib.rs`
  (a note's fields); new `tests/ui-web/tests/pages.rs` and goldens
  `tests/ui-web/golden/{list.html, note.html}`;
  `tests/ui-web/Cargo.toml` (skein-ui-web, for `Page::html`,
  `page.rs:184`); the view's `src/tests.rs` (the tree's length, line
  87, 461 nodes at skein `890b1a1`), `src/tests/golden/{empty,list}.txt` (read at `text.rs:16-30`)
  and `src/limits.rs` (`bindings`, 66, line 14, for the form's fields);
  `testing/skein-fake-person-browser/src/face.rs` (a `Press` on an
  option of a `<select>` becomes the kit's keys, as 14.1's
  `chromium_chooses_an_option_by_the_kits_keys` chooses one: the kit
  presses through `DOM.getContentQuads`, `browser.rs:1577`, which a
  closed select's option has none of; ui-web.md 14.2: "an option
  chosen with the kit's keys, as a keyboard chooses one");
  `tests/ui-web/tests/browser_example.rs`; ruling 129: the kit's
  `testing/skein-browser/src/params.rs` (`Params::Describe` asks
  `pierce: true`, so that a hit in a node's user-agent shadow tree is
  inside the node) and the fact
  `chromium_presses_a_select_inside_its_shadow_tree` in
  `tests/browser/tests/browser_facts.rs`. The date field, typed by the
  kit's keys (the designer's answer of 2026-10-15, skein `cef6a1a`:
  browser.md 3 and 6; ui-web.md 14.1 and 14.2):
  - the kit: `testing/skein-browser/src/boundary.rs`,
    `Request::Keys { page, op, node, text }` after `Compose` (line 37);
    `browser.rs`, admitted beside `Request::Type` and `Compose`
    (583-594), whose `OpKind::Type { text, compose: bool }` (85, 1607)
    becomes `OpKind::Type { text, entry: Entry }`, `enum Entry { Insert,
    Compose, Keys }`; after `Step::Focus` (1858), `Keys` sends each
    character's `Input.dispatchKeyEvent`, `keyDown` then `keyUp`, one
    at a time, through `Params::Key` with the character as `key` and
    `text`, `code` `Digit<d>` or `Key<L>` and `windowsVirtualKeyCode`
    its uppercase ASCII code (for any other character: `code` empty,
    0), and `Done` after the last `keyUp`, or after the focus for the
    empty text; `key_info` (1152) stays the named keys' table; about
    40 lines in `browser.rs`, whose split stays `later.md`'s row;
  - the face: `testing/skein-fake-person-browser/src/face.rs`,
    `perform`'s `Act::Input` (222-230): after the emptiness check (223,
    unchanged), a node whose `Seen.role` is `Date` gets
    `Request::Keys` with `keys::date(&value)`, one whose role is
    `InputTime` gets `keys::time(&value)`, and any other node keeps
    `Request::Type`; `None` fails the step with `field '<name>' takes
    no value '<value>'`. New `src/keys.rs`, pure: `pub(crate) fn
    date(value: &[u8]) -> Option<Box<[u8]>>` takes exactly
    `DDDD-DD-DD` (digits and dashes, no range check) and gives month,
    day, year (`2026-10-15` → `10152026`); `pub(crate) fn time(value:
    &[u8]) -> Option<Box<[u8]>>` takes exactly `DD:DD` with the hour
    00-23 and the minutes 00-59, and gives the hour on a 12-hour clock
    as two digits (00 → 12, 13-23 → 01-11), the minutes, then `A`
    before 12:00 or `P` from it; both give the empty text for the
    empty value;
  - the fact: `chromium_takes_a_date_and_a_time_by_the_kits_keys` in
    `tests/browser/tests/browser_facts.rs`, with a `Scenario::Dates` on
    `/facts` in `tests/browser/tests/support/real.rs` (beside
    `Choose`, 130-160): find role `Date` named exactly
    `case-entry_date`, `Keys` `10152026`; find role `InputTime` named
    exactly `case-entry_time`, `Keys` `0230P`; then the snapshot holds
    a line that, trimmed, starts `Date case-entry_date 2026-10-15`, and
    one that starts `InputTime case-entry_time 14:30`;
  - the story types `2026-10-15` as it does now: the face turns it into
    keys.
Shape: minutes read by `skein_ui::number`, invalid when not one; the
  due date `YYYY-MM-DD`, invalid otherwise; an empty title refused in
  words (the field invalid, its description saying why); a date or a
  time typed by the kit's `keys` in its parts' order under `en-US`,
  every other field by `type` as before.
Test shape: view: `an_empty_title_is_refused_in_words`,
  `minutes_that_are_not_a_number_are_invalid`, `a_label_is_chosen_by_its_option`;
  `tests/ui-web/tests/pages.rs`: `the_list_page_maps_as_its_golden_html`
  (the view's patches through the page part's host half), by hand;
  story `a_person_fills_each_entry_kind_and_chooses_a_label`. The kit,
  in `tests/browser/src/tests.rs`: `keys_type_each_character_as_its_key_down_and_up`
  (`Keys` with `0230P` sends `DOM.focus`, then ten
  `Input.dispatchKeyEvent` in order, each character's `keyDown` then
  `keyUp`, then `Done`; the empty text sends the focus alone, then
  `Done`). The face, in `src/tests.rs`:
  `a_date_and_a_time_become_the_keys_of_their_parts` (`2026-10-15` →
  `10152026`, `0001-12-31` → `12310001`, `00:00` → `1200A`, `09:05` →
  `0905A`, `12:00` → `1200P`, `14:30` → `0230P`, `23:59` → `1159P`, the
  empty value → the empty text; `2026-1-5`, `14:30:00`, `24:00` and
  `9:05` → `None`) and `a_date_field_is_typed_by_the_kits_keys` (a seen
  node of role `Date` and an `Input` of `2026-10-15` give
  `Request::Keys` with `10152026`). The fact above. Step tests under
  20 ms; the story about 3 s; the fact about 2 s.
Proof: live, `heavy --gate scripts/check.sh` once: the story, the
  golden pages and the facts pass. Bound: 15 minutes.
Don't: check a field's words in the shell (the view's, ui.md 6); set a
  field's value by a page call (`Runtime.evaluate`,
  `DOM.setAttributeValue`): the one script the kit runs in the page is
  the expression `document`, a find's root (browser.md 1 and 4), and a
  person's entry sets no attribute; add
  `--lang` to the kit's command, whose environment already fixes the
  locale (browser.md 3); type any other field by keys: `type` still
  inserts.
Size: about 470 lines, up from about 250 (the kit's `keys`, the face's
  keys and the fact: about 220).
Removes: nothing.

### 13.3 The dialog, Escape, and a focus request
Scope: examples.md 9.1 (the dialog; the focus after an add or a
  discard); ui.md 6.1 (Escape); ui-web.md 7 (Escape, a focus request).
Needs: 13.2; skein 02.1 (a focus request), 02.3 (Escape).
Files: `examples/ui/view/src/{state.rs, view.rs}`; the view's
  `src/limits.rs` (`bindings`, for `Discard` and `Keep`), `src/tests.rs`
  and its goldens where the page's nodes move;
  `testing/skein-fake-person-browser/src/{harness.rs, face.rs}`
  (`Step::Key`, `Step::Focused`);
  `tests/ui-web/tests/browser_example.rs`.
Shape: clearing a form holding words opens a dialog with `Discard` and
  `Keep`, `Keep` naming `Shortcut::ESCAPE`; an add or a discard moves
  the title's focus request count.
Test shape: view: `clearing_words_asks_in_a_dialog`,
  `an_add_moves_the_focus_to_the_title`; story
  `escape_in_the_dialog_keeps_the_draft` (the kit's `Key::Escape`; the
  dialog gone, the words still in the title, the focus on it, by the
  harness's `Focused` step). Step tests under 20 ms; the story about
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
  `examples/ui/shell-web/notes.css` (`notes-bar`); the view's
  `src/lib.rs` (`pub mod draw;`), `src/limits.rs` (`drawing_bytes`,
  `scene_marks` and `scene_bytes` are 0, lines 11, 17, 18, so no scene
  holds a mark: 64, 6 and 256 drew the check's sketch with the worst
  case under 100 MB; size them by the view's worst-case test),
  `src/tests.rs` (line 87) and the empty and list goldens;
  `testing/skein-fake-person-browser/src/{harness.rs, face.rs}`
  (`Step::PressAt`; the face refuses a person's `Act::Drawing`,
  `face.rs:163`, so the harness presses through a face method);
  `tests/ui-web/tests/browser_example.rs`. The page part's press in a
  scene has landed with 09.5: `Event::Drawing { node, under, x, y }`
  (`crates/skein-ui-web/src/page.rs:494`) to `Act::Drawing` through
  `Scene::act_at` (577-590), the marks and `elementsFromPoint` at
  `page/web.rs:702-726`; 13.4 changes neither.
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
  `examples/ui/shell-web/src/lib.rs` (its `Look` literal, 56, gains
  `components`; `mod grow;`; the constructors given to `Dom::new`, 61)
  and `Cargo.toml` (web-sys `HtmlElement`, beside `Window`, `Document`,
  `Location`); skein-ui-web's `src/tests/fixtures.rs` (101);
  `testing/skein-fake-person-browser/src/{harness.rs, face.rs}`
  and `harness/script.rs` (`Step::Taller { find, words }`, ruling 132);
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
  text area's box taller after three lines, by the harness's `Taller`
  typing them; the
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
