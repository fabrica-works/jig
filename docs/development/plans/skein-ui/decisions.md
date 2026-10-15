# skein-ui: decisions

The decisions this plan rests on, numbered; the supervisor's rulings
during the run follow them, from 101; the choices left to the user close
the file. The user accepted every default the designer proposed
(2026-10-15).

## The plan's own

1. **Scope, the user's (2026-10-15).** The aim: skein's generic UI layer,
   `skein-ui` and the crates around it, sound and complete as a generic
   layer any portable, multi-platform application builds on. The plan
   covers:
   - **(a)** everything skein's design asks for: `skein-ui`,
     `skein-ui-text`, `skein-ui-web`, `skein-ui-bundle`,
     `skein-fake-person` and `skein-fake-person-browser`; the vocabulary
     added now; the act refusal; scenes; the browser parts; the bundler;
     the face and the harness; the UI example with its reference shell
     and browser tier; the gate's wasm32 builds; and the browser kit's
     words in a name and press at a point (skein's `browser.md`, 5 and
     6), which the face and the harness use (`ui-web.md`, 14.2);
   - **(b)** jig's changes the extraction needs, so that jig builds on
     skein-ui with its gate green: the generalizations of jig's
     `STATUS.md` row "jig on skein-ui", step 1, and jig taking the moved
     crates, and the closing repin onto skein's last landing.

   jig's new primitives are `later.md`'s, "waits for the analysis of
   jig's boundary with applications". A jig change the extraction
   forces, which the design describes, is in: jig's outline rules
   leaving `outline.rs` (jig's `view.md`, 3.1: "The outline is
   skein-ui's; jig adds no rule"), the provisional state dropped, the
   text shell's link region made a setting.
2. **The user's rule: refactor in place, then move, then build.** jig's
   code is refactored in place, under jig's gate, until the generic
   parts extract by a mechanical rename and crate move; never rewritten
   in skein. The order of both `STATUS.md` rows: (1) generalize in jig
   (jig 01 to 03); (2) move as is into skein, skein landing first (skein
   01), then jig repins and takes the crates (jig 04); (3) build what is
   new (skein 02 onwards), and jig takes it (jig 05).
3. **No downstream repin.** smith keeps its pins: its run `parity` is
   active on another machine, and nothing this plan lands in skein is a
   crate smith uses or changes one it uses, but the browser kit's
   additions (`skein-browser`, which smith does not depend on:
   `rg -l skein-browser ~/src/smith --glob Cargo.toml` prints nothing).
   The repository downstream of jig keeps its pins for its own run,
   active on another machine; its repin onto this plan's jig renames
   the tree's types it uses (`jig_view::Tree` and the rest become
   `skein_ui`'s) and drops `State::Provisional`, more than a repin may
   change (development.md, 5.7), and waits for its next plan
   (later.md). The README says so as a rule.
4. **jig moves skein only, never smith.** jig's `Cargo.lock` pins skein
   at `74b7fdaf3` and smith at `29fa97c9`. At the base, in a worktree of
   jig at `origin/main`, `heavy sh -c 'cargo update -p skein-lib && cargo
   check --workspace --all-targets'` moved skein to `2f282f7` with smith
   unmoved, and passed (2026-10-15, 88 s). Each jig repin runs `heavy
   cargo update -p skein-lib`, which moves the whole skein source and
   leaves smith's. If its `cargo check` then fails inside a smith crate,
   the increment stops and reports a question: moving smith brings
   smith's `parity` changes and more (117 commits at the base), which is
   not this plan's.
5. **The generic core is gathered in `jig_view::ui` before it moves.**
   jig 01 to 03 gather the generic parts of `jig-view` into one module,
   `crates/jig-view/src/ui/`, which names nothing outside itself but
   `skein_lib` (a test checks it, code-organization.md, 4), and change
   every user in jig to import them as `jig_view::ui::…`. The move (skein
   01, jig 04) is then: copy `src/ui/` to `crates/skein-ui/src/`, with
   `crate::ui::` read `crate::`; in jig, `jig_view::ui::` and
   `crate::ui::` read `skein_ui::`. `jig-shell-text` and
   `jig-fake-person` name only `jig_view::ui`, `skein-lib` and
   `skein-io` before they move, and move whole, renamed.
6. **jig's owner over skein-ui's opaque owner** (jig's `view.md`, 3).
   `jig_view::ui::Owner { number: u32 }` is the tree's; `jig_view::Owner`
   stays jig's enum, `Jig` or `Application { number: u16 }`, with
   `Owner::opaque(self) -> ui::Owner` (an application's number to the
   same number, jig to 65536) and `Owner::from_opaque(ui::Owner) ->
   Option<Owner>`, both matched whole. Every user changes in the same
   increment, with no alias (jig 01.1).
7. **The meeting point in the browser comes first** (development.md,
   5.3). The browser's sides (the bundler, the page part, the example's
   view, server and reference shell, the harness and the face) meet in
   one thin story, "a person opens the example and sees its heading",
   in skein 11, before any part grows past it. Each later browser
   session needs that story at its start, and grows it.

8. **The bundle's digest encoding** (ui-web.md, 4.2, "the encoding the
   plan's"): for each file of the digest directory, in the byte order of
   its path relative to the directory, `/` between segments: the path's
   bytes, one zero byte, the file's length as 8 bytes little-endian,
   then its bytes; SHA-256 over all of it; its first 16 lowercase hex
   digits name the directory. The zero byte and the length make the
   encoding unambiguous: no two directories give the same bytes.
9. **SHA-256 is the bundler's own** (skein 08.1), about 70 lines checked
   against FIPS 180-4's examples. skein's workspace takes dependencies
   from outside only where its design names them (`Cargo.toml`'s comment
   on programming-model.md, 2.1): the glue and its library are named
   (ui.md, 2); a digest crate is not. `ring`, in the lock through
   rustls, would be a second outside dependency of a tool.
10. **The press order takes the platform's hit test** (skein 04.2):
    skein-ui orders the marks under a point (ui.md, 11.3), and the shell
    finds which marks are under it, since a text mark's box needs the
    platform's fonts.
11. **The page part decides on the host, and the platform half replays
    it** (skein 09): the mapping, the patches applied and the acts read
    from events are pure, tested against golden HTML; the platform half
    executes `page::Op`s on the document. The browser tier then tests
    the replay, not the decisions.
12. **The example's test build's addresses** (examples.md, 9.4, "the
    plan's"): `/test/panic` and `/test/small-store`, read only under
    `cfg(debug_assertions)` (skein 14.2, 14.3).

13. **jig 04 pins skein 01.3's landing, not skein's newest `main`.** The
    move is mechanical only if jig takes the moved code alone: skein 02
    to 06 change skein-ui after the move, which jig 05 takes with its
    repairs. So no session changes skein-ui, skein-ui-text or
    skein-fake-person between skein 01.1 and 01.3 (each needs 01.3), and
    jig 04 runs `cargo update -p skein-lib --precise` to the commit
    whose `Step:` trailer is `skein-01.3`.
14. **The web mapping before the vocabulary** (skein 09 before skein
    02): the browser's meeting point (decision 7) needs the page part,
    not the new roles; so skein 09 maps the vocabulary skein 01 moved,
    and each of skein 02's additions reaches the mapping and the page
    part with its golden rows, as ui.md 4.3's light path does.

## The supervisor's rulings

None yet; they start at 101.

## Rulings

101. **jig 02's Shapes against main** (ready check of jig-02.1).
     jig 02's session file corrected against jig `d2a7a714e` (01.3 landed): line numbers moved by 01.1 and 01.3; the Files of 02.1, 02.2 and 02.3 name the `crate::` users inside jig-view, the 18 users of `jig_view::Limits` outside it, the callers of `Limits::{of, of_words, compose, envelope}`, every user of the `outline` path, and the test modules' lists; moved tests name `ui::Owner { number: N }`; `worst_case` and `jig_worst_case` return `Option<u64>`, `jig_capacity` `Option<u32>`; the `jig_capacity` field goes with `Phrasebook`, its method becoming jig's free function; the chip's `title` becomes `name`; 02.4's new test is dropped (the span test at `tests/phrases.rs:92` becomes the reference hole's) and it needs 01.3, not 02.1; 02.3 moves `page()` and `finish()` into `ui/tests/fixtures.rs` as the fixture's builder, with the kept tests' nodes renumbered; 02.2's proof adds `binary(fuzzy_memory)` under the fuzzy profile. A further user of a moved or renamed item that these Files miss: change it in the same increment, and list it in the commit body.

102. **jig 03's Shapes against main** (ready check of jig-03.1).
     jig 03's session file corrected against jig `d2a7a714e` (01.3 landed): line numbers moved by 01.1 and 01.3; 03.1's landmark test opens a button inside each landmark and binds that; `bind`'s role check is an exhaustive `match` (`clippy.toml` disallows `matches!`), after the drawing's assertion, and is about 50 lines; 03.2's Files name the makers of `Settings` (`tests/client-protocol/src/native/world.rs`, `crates/jig-shell-text/src/tests/terminal.rs`) and the callers of `render`, and its proof adds `jig-client-protocol-world`'s native terminal test; 03.3 imports from `jig_view::ui` only what 01.3 moved, jig 02 moving the rest. A further user of `Settings`, `render` or a moved item that these Files miss: change it in the same increment, and list it in the commit body.

103. **skein 01's Shapes against main** (ready check of skein-01.1).
     skein 01's session file corrected against jig `f15431139` (jig 02.5 landed) and skein `9366d5d`: `lib.rs` comes from jig's `ui.rs`, not `ui/mod.rs`; the six doc comments citing jig's `view.md` name their skein sections; each increment runs `cargo fmt` after the rename; `skein-sim` goes in `crates/skein-ui-text/Cargo.toml`, with `[workspace.dependencies] skein-ui-text` in skein's; skein-fake-person drops jig's three `disallowed_*` allows and takes a copy of `tests/clippy.toml`, as skein-fake-channel does; the sizes are those counted at jig `f15431139`. A further name of jig, citation or lint the copy trips that these lines miss: fix it in the same increment, and list it in the commit body.

104. **skein 10's Shapes against main** (ready check of skein-10.1).
     skein 10's session file corrected against skein `a37b22a` (skein 01.1 and 01.2 landed): the list's record is `Notes`, since skein-codegen reserves `List`; 10.1's Files add the drift test's `golden/v1/`, take `skein-codec` by path, and write the drift copy's switch without `is_some_and`, which the step-code `clippy.toml` disallows; the sketch's `Duties` holds `skein_lib::Queue`s, not `Vec`s; 10.2's Files add the server's `clippy.toml` and `[lints]` tables, copied from the echo shell's, and cite `tests/browser/src/serve.rs:118-238`. skein 12's `Requests` mirrors `Duties`: its own ready check settles its types. A further use of a reserved record name or a disallowed type or method the copy trips: fix it in the same increment, and list it in the commit body.

105. **A gate's single flaky world-test failure is retried once** (process, skein 01's block of 2026-10-15T11:39:20Z).
     skein 01.3 runs `land` once more on its unchanged tree (commit 70f09d1 rebased as `land` does). The failed test, `terminal_is_controlling_and_the_persons_interrupt_reaches_only_the_child` (skein's `tests/world/tests/end_to_end.rs:158`), is a world test of skein's terminal that the move does not reach, and it passed in 81 of the 82 gate logs of this machine's runs. For the rest of this run, a gate whose only failure is this test is retried once on the unchanged tree, without a block, and the retry is named in the commit body; a second failure of it in a row blocks, kind process. Its diagnosis is deferred: `later.md` has its row.

106. **skein 03's Shapes against main** (ready check of skein-03.1).
     skein 03's session file corrected against skein `a37b22a` (skein 01.1 and 01.2 landed): it cites skein-ui's lines, not jig's; the rule's `rg` searches `crates testing`; 03.1 adds `mod look;` to `src/tests.rs`, its reference test holds a `&[Names]` per row, and the sketch derives `Debug` on `Names`, `Clone, Copy, PartialEq, Eq, Debug` on `Fault`, with `#[must_use]` on `check`, `style`, `region` and `takes`; 03.2 flips `src/tests/tree.rs:142`'s drawing row to `true` and removes `a_drawing_cannot_be_bound`, gives `src/tests/binding.rs`'s text area `State::SubmitsOnEnter` and moves its drawing test to `src/tests/view.rs` on a bound drawing, and adds `Tree::disabled`, the one copy of the rule that an option of a disabled select is disabled, which skein 05.2, 06.2 and 06.3 call. The text shell needs no change in 03.2: a drawing is never numbered. A further test or copy of a rule these Files miss: change it in the same increment, and list it in the commit body.

107. **skein 04's and 05's Shapes against main** (ready checks of skein-04.1 and skein-05.1).
     skein 04's and 05's session files corrected against skein `a37b22a` (skein 01.1 and 01.2 landed). skein 04: 04.1's Files list skein's `Limits` literals (with `tests/ui-text` in its `rg`, for skein 05.1's copy), `pub mod scene` and `mod scene;`; a polyline's points count against `scene_bytes`; 04.4's `worst_case` counts the scene's tokens too; the sketch's types derive `Clone, Copy, PartialEq, Eq, Hash, Debug` (`Scene` `Debug`), with `#[must_use]` on `new`, `refused` and `marks`. skein 05: 05.1 copies the fixtures as they are on `main`, names `Cargo.lock`, and writes `expect` for the harness's two `unwrap`s; 05.2 needs skein 03.2 and calls its `Tree::disabled`, and its reference test uses `src/tests/command.rs:9`'s `numbers`; 05.4's test fills texts with newlines and control bytes, gives each node only the states its role takes, and its proof runs `skein-ui-text-tests`, since `Terminal::new` sizes its buffers by `worst_case`; 05.5 needs skein 02.4 and adds only the first line without its status region; the citations are skein's lines. skein 02.4's Files add `crates/skein-ui-text/src/command.rs`, whose match over `Role` takes the four roles. A further literal, copy or citation these Files miss: change it in the same increment, and list it in the commit body.

108. **skein 09's Shapes against main** (ready check of skein-09.1).
     skein 09's session file corrected against skein `a37b22a`: skein-ui-web, ordinary Rust (ui-web.md, 2), takes its own `clippy.toml` and `[lints]` tables, copied from skein-ui-bundle's, not the workspace's, and 09.1 names `Cargo.lock`; 09.2 to 09.4 name their module lines in `src/lib.rs` and `src/tests.rs`; 09.3 needs 09.2 (`Look`, `dom::Element`); `Page::new` takes the view's `Limits`, for the one `Scene` the page holds. A further lint, module line or caller of `Page::new` these Files miss: change it in the same increment, and list it in the commit body.

109. **skein 06's Shapes against main** (ready check of skein-06.1).
     skein 06's session file corrected against skein `dc71a49` (skein 01.3 landed): it cites skein-fake-person's lines; the face and the random person call `skein_ui::takes`, which skein 03.2 re-exports at skein-ui's `lib.rs:17`; 06.1's reference test reads the tree of a `View` fixture; 06.2 chooses the random person's act by `takes` (a toggle, else a press, else a submit or an input), drops `field` and `choice`, and turns the sweep at `tests.rs:308-370` into one that every bound node is acted on; `passive` holds no select, since the builder binds no passive role since jig 03.1; 06.3 takes `Tree::disabled` and replaces `passive` and 06.1's match with `takes`, the rule's `rg` checking that no copy of the table is left. skein 02.4's Files add the fake person's `passive`, `field` and `choice`, whose matches over `Role` take its four roles. A further copy of ui.md 6's table or caller these Files miss: change it in the same increment, and list it in the commit body.

110. **jig 04's Shapes against main** (ready check of jig-04.1).
     jig 04's session file corrected against jig `dcf493fc0` and skein `a37b22a`: the repin's `git log` matches `Plan: skein-ui` and `Step: skein-01.3` together (`--all-match`); 04.1 removes `src/ui.rs`, `pub mod ui;` and `ui_boundary` with `src/ui/`, rewrites `crate::{…, ui}` imports too, swaps `jig-view` for `skein-ui` in jig-shell-text and jig-fake-person, and lets `tests/boundary` allow `skein-ui`; its Shape is no change of behaviour, with jig-view's tests no longer reaching what skein-ui keeps private (`Node::text_bytes` a test helper, `Builder.tree`, `Tree.nodes`, the `ui` fixture's `LIMITS`); 04.2 and 04.3 name their `tests/boundary` edits. A further user of `jig_view::ui`, a private item of skein-ui or a boundary test these Files miss: change it in the same increment, and list it in the commit body.

111. **jig 04's pin held through each dependency** (ready check of jig-04.2 and jig-04.3).
     jig 04.2 and 04.3 checked against skein `dc71a49` (skein 01.3): adding a skein dependency moves every skein entry of jig's `Cargo.lock` to skein's newest `main`, whose skein-ui has skein 04.1's `Limits` fields, so each of 04.1, 04.2 and 04.3 reruns the rule's `cargo update -p skein-lib --precise` after adding its dependency, until `grep -o 'skein.git#[0-9a-f]*' Cargo.lock | sort -u` prints skein 01.3's commit alone; 04.2 and 04.3 run `cargo fmt` after their renames. The downstream repository's users of `jig_fake_person` join its `later.md` row. A further dependency or rename these Files miss: the same, listed in the commit body.

112. **skein 09.3 goes on without the primary shortcut (interim)** (design gap, skein 09's block of 2026-10-15T12:34:37Z).
     Interim ruling 112, until the designer's answer to the run's `question.md` (sent 2026-10-15T12:36:14Z): the question is a design gap, so the designer decides it (development.md, 10.4). Meanwhile skein 09.3 goes on with the work that does not rest on the answer: every row of `each_role_maps_to_its_element` and `each_state_and_property_maps_to_its_attribute`, and the golden page, except a shortcut with `Modifier::Primary`; `Plain`, `Alt` and Escape are written as ui-web.md, 6, says ("`Alt+K`, `K`, `Escape`"). It keeps the Shape's `mapping::element(made: &Made, look: &Look<'_>, prefix: &[u8]) -> Element`, adds no entry point, no field of `Look` and no convention type, and writes no `Control+K` or `Meta+K`. It commits nothing of 09.3 and does not land it until the answer's unblock, which says how the primary modifier reaches the mapping; then 09.3 adds the primary row and lands. Nothing else of the plan changes.

113. **the primary modifier, a fact of the platform** (design gap, skein 09's block of 2026-10-15T12:34:37Z).
     Ruling 113 lifts ruling 112's hold: the designer's answer, landed as skein `1b49e88` (the design) and jig `041267fd8` (the session files), follows.


     **Decision: the supervisor's third reading.** The primary modifier is
     one of the platform's facts. The shell reads it once, at start, and
     passes it to `Page::new`, as it passes `limits` and `prefix`. The page
     passes it to `mapping::element`, and `Page::act` matches keys by it. It
     is not part of `Look`. The mapping gets no second entry point and no
     default. It never writes both forms.

     - **Why not the look:** the look is the application's data, the same on
       every platform (ui-web.md, 3.2). The primary modifier belongs to the
       platform.
     - **Why not a second entry point that defaults to Control:** a shell
       that called the default would write `Control+K` on a Mac and nothing
       would flag it. With one entry point, every caller has to pass the
       modifier.
     - **Why not both forms:** ui.md 6.1 gives each platform one primary
       modifier. Writing both would tell a screen reader about a shortcut
       that does nothing there: on a Mac, Control+K in a field is the
       platform's kill to the end of the line. And 09.5 would still need the
       input to match keys.

     ### Landed

     - skein `1b49e88`, "Name the primary modifier a fact of the platform":
       ui.md 6.1; ui-web.md 5.1, 6, 7 and 14.
     - jig `041267fd8`, "skein-ui: the primary modifier through skein 09
       and 11.3": `skein/09-page.md` (09.3, 09.4, 09.5, the sketch) and
       `skein/11-first-story.md` (11.3).
     - Nothing deferred: no `later.md` row.

     ### The text it rests on (skein `1b49e88`)

     - ui.md 6.1: "**The primary modifier is a fact of the platform,** which
       a shell reads from its platform once, at start, and maps and matches
       shortcuts by: not the look's, which is the application's and the same
       on every platform, and not both modifiers, since each platform has
       one." And: "A key presses only with its shortcut's modifier held and
       no other: none, the primary one alone, or Alt alone. Any other set,
       Shift held among it, or Control on Apple's platforms, is no shortcut."
     - ui-web.md 5.1, the table: the page now takes "…, the primary
       modifier; patches and the tree", and the platform's facts now give
       "…, the person's offset and languages, the primary modifier". Below
       the table: "**The primary modifier** (ui.md, 6.1) is Meta, a
       browser's name for Command, when `navigator.platform` names one of
       Apple's platforms (it starts with `Mac`, `iPhone`, `iPad` or `iPod`),
       and Control otherwise, an unreadable platform among it. The shell
       reads it once, at start, and gives it to the page part when it makes
       it; the page part writes shortcuts (section 6) and matches keys
       (section 7) by it, its pure half reading no platform."
     - ui-web.md 6: "*a shortcut* is `aria-keyshortcuts`, in ARIA's form
       (`Alt+K`, `K`, `Escape`), a primary one by the primary modifier the
       page part was given (5.1): `Control+K`, or `Meta+K`, never both."
     - ui-web.md 7, "Shortcuts": "The page part's platform half gives each
       `keydown` as the event holds it, and its pure half decides: *the
       modifier,* from the event's `ctrlKey`, `metaKey`, `altKey` and
       `shiftKey`, by the primary modifier (5.1): none held, none; the
       primary one alone, the primary; Alt alone, Alt; any other set, no
       shortcut; *the key,* the event's `key`, lowercased, when it is a
       letter `a` to `z` or a digit; else the letter or digit its `code`
       names (`KeyK`, `Digit5`), as when Alt on macOS composes another
       character, or a layout gives other letters; *a key that presses a
       node* has its default prevented, so that the browser does not act on
       it as well."

     The key's `code` fallback and the prevented default answer points the
     question did not ask. They change the same `Event::Key` that 09.5 has
     not started, and Chromium on Linux never exercises them: Option+K gives
     `key` `˚` on macOS, so an Alt shortcut read from `key` would never fire
     there.

     ### What each step does

     - **09.3,** once ruling 112's hold lifts: `mapping::element(made:
       &Made, look: &Look<'_>, prefix: &[u8], primary: Primary) -> Element`,
       with `pub enum Primary { Control, Meta }` in `mapping.rs`. In
       `each_state_and_property_maps_to_its_attribute`, add a row for a
       primary `k` under `Primary::Control` (`Control+K`) and one under
       `Primary::Meta` (`Meta+K`). Every other row and the golden page run
       under `Primary::Control`, and the golden page includes one primary
       shortcut. Don't put the modifier in `Look`, add a second entry point
       or give a default.
     - **09.4:** `Page::new(look, limits, prefix, primary)`. The page keeps
       `primary` for the mapping and for 09.5's keys.
     - **09.5:** `Event::Key { key: Box<[u8]>, code: Box<[u8]>, control:
       bool, meta: bool, alt: bool, shift: bool, in_field: bool }`.
       `page::web::Dom` copies the `keydown`'s fields unchanged and prevents
       its default when `act` gives an act. `each_event_gives_its_act` gets
       key rows under each `Primary`:
       - the primary modifier with `k` presses the node that names a
         primary `k`;
       - the other modifier gives none;
       - Shift on top, or Control with Alt, gives none;
       - Alt with `key` `˚` and `code` `KeyK` presses the node that names
         Alt `k`.

       The four flags must not go into a struct: skein-ui-web runs pedantic
       clippy, and `struct_excessive_bools` refuses a struct of four
       `bool`s.
     - **11.3:** `Facts { now, wall, seed, offset, languages, primary }`.
       `primary` comes from `navigator.platform` through a pure
       `platform::primary(platform: Option<&str>) -> mapping::Primary`,
       tested on the host by
       `the_primary_modifier_is_meta_on_apples_platforms_alone`: `MacIntel`,
       `iPhone` and `iPad` give `Meta`; `Linux x86_64`, `Win32`, the empty
       text and `None` give `Control`. The shell passes `facts.primary` to
       `Page::new`.
     - **Unchanged:** skein 11's face finds nodes through
       `mapping::accessible`, which no shortcut touches. skein 14.1 serves
       the golden page, now under `Control`, and compares computed roles,
       which `aria-keyshortcuts` does not change. skein 02's additions call
       the new signature as they find it.

     ### Checked against main

     - skein `cb07925`, `crates/skein-ui/src/tree.rs:166`: `pub enum
       Modifier { Plain, Primary, Alt }`. `crates/skein-ui-web/src/look.rs:6`:
       `Look { names, pictures, draw }`. `crates/skein-ui-web/Cargo.toml:23`:
       `pedantic = { level = "warn", priority = -1 }`.
     - web-sys `=0.3.103` (skein's `Cargo.toml:64`), with feature
       `Navigator`, has `gen_Navigator.rs:306`: `pub fn platform(this:
       &Navigator) -> Result<String, JsValue>`. This is stable, unlike
       `user_agent_data`, which needs `web_sys_unstable_apis`.
       `gen_KeyboardEvent.rs` has `alt_key` (41), `ctrl_key` (48),
       `shift_key` (55), `meta_key` (62), `key` (90) and `code` (97).
     - Commands: `grep -n 'pub enum Modifier'
       crates/skein-ui/src/tree.rs`; `grep -n 'pedantic'
       crates/skein-ui-web/Cargo.toml`; `grep -n 'pub fn platform\|pub fn
       user_agent_data' ~/.cargo/registry/src/*/web-sys-0.3.103/src/features/gen_Navigator.rs`.

114. **skein 11's Shapes against main** (ready check of skein-11.1).
     skein 11's session file corrected against skein `5b4e135` and jig `fb802523a` (11.3 as jig `041267fd8` left it): 11.1 takes `testing/skein-fake-person-browser/clippy.toml`, a copy of skein-fake-person's; the face numbers the kit's `u64` nodes itself and keeps each one's `Seen`; a heading's level is not checked (`Seen` has none); `Toggle` is a press at the centre, `Drawing` a failure of the step; `Input` types only into a field whose `Seen.value` is empty and fails the step naming a field holding words, since the kit has no select-all (later.md has its row; new test `an_input_into_a_field_holding_words_fails_naming_it`); `Go` is `Go::Address` of `http://<the listening address>` followed by the step's path, `Play` yielding `Next<Box<[u8]>>`; 11.2 reads the listening line on the server's stderr and opens `http://<address>/`, cites `browser_real.rs:49-94, 154-604`, puts the copy of `chromium()` in `tests/ui-web/src/lib.rs`, and names `tests/ui-web/Cargo.toml`, as 11.4 does; 11.3 names skein-ui-web's `Cargo.toml` (web-sys features), `lib.rs` and `tests/fetch.rs`, and drops the example view's `lib.rs`, its answered list a report already. A further file, lint table or dependency these Files miss: add it in the same increment, and list it in the commit body.

115. **skein 02's Shapes against main** (ready check of skein-02.1).
     skein 02's session file corrected against skein `3711eb6` (skein 09.4 landed) and jig `415ac928c`: 02.1's users of `picture` take skein-ui-web's `mapping.rs:337` and `tests/mapping.rs:204`, `Source` cloned since it holds a `Box`, and the second-focus flag set back in `Builder::reuse` and checked in `splice`; a description's id is `skein-ui-description-<node>`, as the landed label and menu ids are; 02.2's Files take skein-ui-web's `page.rs`, which sets the items' `aria-setsize` from their list's count and sends them again when it changes (new test `a_changed_count_sets_each_items_setsize_again`), since the mapping maps one node alone; 02.3's Escape on the web landed with 09.3, keyed on the literal `27`, which `Shortcut::ESCAPE` replaces; 02.4 names the mapping's five matches over `Role`, `Select` in `focusable`, the fake person's lines 121 and 173, and puts an option, a select and the headers in the text shell's `Act::Press` arm, and adds `a_changed_sort_is_one_attribute_patch`; 02.5 names the mapping's state list and `state_attribute` and a `Required` row; Done when's `rg` names `Role::`, `State::` and `Rule::` paths, and its nextest takes `-p skein-ui-web`. A further match over `Role` or `State`, or a user of `picture`, these Files miss: change it in the same increment, and list it in the commit body.

116. **skein 02 against 09.5** (re-check of skein-02.1 after skein-09.5).
     skein 02's session file re-checked after skein 09.5 (`25b00c3`): `mapping::element`'s calls are at skein-ui-web's `page.rs:222, 292` and `focusable` at 374; the act tests of 02.3 and 02.4 go in `src/tests/page/events.rs`, beside 09.5's; 02.4's Files take `page/web.rs`, whose `change` gives `Event::Change` (web.rs:578) and gives `Event::Choose` for a `<select>`, with the web-sys features it uses, and its Proof adds the wasm32 clippy once. A further listener or event these Files miss: change it in the same increment, and list it in the commit body.

117. **skein 12's Shapes against main** (ready check of skein-12.1).
     skein 12's session file corrected against skein `e99b8fc` (skein 11.3 landed) and jig `2f31b4235`: the harness stops and starts the server as its own steps, `harness::Step::StopServer` and `StartServer`, within one `run`, not by methods between two runs, since each `run` makes its own runtime and Chromium profile and ends both children; the server comes back at its listening line's address, put in place of `args[0]`, so no test names a fixed port; `carry` takes no `Fetch`: `platform::Browser<R>` (re-exported beside `clocks` and `facts`) holds the shell's `fetch::web::Fetch<R>` and sends through its `exchange`, so each exchange has one record; removes go after the writes and before the exchanges; `StreamDuty::Open` carries headers; the add's record key is `add-<16 hex>` under the store's prefix; 12.1 to 12.4 name skein-ui-web's `lib.rs`, `tests.rs` and the web-sys features they use; 12.3 names the server's `main.rs`, `tests.rs` and `tests/ui-web/tests/serve.rs` (`route` takes `&mut State` and the body), and the view's `limits.rs` (`bindings` 66), `tests.rs` (462 nodes) and both goldens; 12.4 names the stream's way through the transport and the platform, the server's deadline, `worst_case` and fan-out, and the view's `Duties` fixture. A further caller of `route`, user of `Duties` or golden these Files miss: change it in the same increment, and list it in the commit body.

118. **skein 14's Shapes against main** (ready check of skein-14.1).
     skein 14's session file corrected against skein `0fd10aa` (skein 02.4 landed) and jig `81744cca6`: 14.1's facts test goes in the kit's world package, `tests/browser/tests/browser_facts.rs`, over its `Rig` and `serve::Pages`, since the harness only spawns `Setup.server`; its page is built in the test from `mapping::element`, each element its own node, the table's parts in `<table><tr>`, the option in its select, the menu open, not the golden `mapping.html`, whose loose table parts Chromium drops; a fixed fact moves `accessible` with its `ROLES` copy and the face's users; the panic story's `Setup.expected` holds `Uncaught`, the kit's text for the trap, not the panic's words; `stories::panic` takes `failed: Find` (the designer's question of the same time may amend it, and 14.4's `arrives`); 14.2 names skein-ui-web's `lib.rs` and `tests.rs`, the view's `Place` users and path table, and panics by an `assert!` since `clippy::panic` is denied; 14.3 names the shell, which makes the store, and the second person waits on the first's end before playing `Gone`; 14.4 adds `harness::Step::Type`, the kit's `Type` at the caret, and names `face.rs`'s `perform`; the sketch's stories are `#[must_use]`. A further user of a `Place`, of `accessible` or of the harness's steps these Files miss: change it in the same increment, and list it in the commit body.

119. **skein 14 against 02.5** (re-check of skein-14.1 after skein-02.5).
     skein 14's session file re-checked after skein 02.5 (`54ca50f`): skein-ui-web's golden page test is at `src/tests/mapping.rs:318-347` and `ROLES` is asserted at 96; `accessible` (`mapping.rs:339`) and its users in the face (`face.rs:197-198`) are as ruling 118 names them.

120. **what skein 14's stories take** (the designer's answer of 2026-10-15T14:14Z, no step blocked).
     The designer's answer to the question of 2026-10-15T14:07:16Z (what skein 14's stories take from an application), landed as skein `7c5a482` (ui-web.md 14.2 and 14.3; examples.md 9.4) and jig `e240ff1eb` (skein/14-stories.md), follows.


     **Decision: yes, the supervisor's reading.** Each of the two stories
     takes one more `Find` from the application:
     `typing_while_patched(setup, field, second, arrives: Find)` and
     `panic(setup, address, failed: Find)`, as ruling 118 already has it.
     ui-web.md 14.2's table is amended so that its third column names
     everything a story takes, and nothing else. The write-refused row and
     the panic row now also name what the plan's signatures already took.

     - **Why `arrives`:** the first person can only learn that the patch
       reached its page by seeing it. Seeing it also proves that a patch
       came at all: with no patch, the field keeps its focus, caret and
       words anyway, and the story would pass for nothing. A fixed sleep
       waits the same time whether or not the page changed.
     - **Why `failed` is a `Find` and not skein's own text:** 5.1 gives the
       restart part "the words for "the application failed"", and 3.2 says
       those words come from the application's phrasebooks. Skein does not
       know them. A `Find`, not the bare words, keeps the panic story in line
       with the other three, whose pages the application names by `Find`
       (`Shown.sees`, `absent`). It also lets an application that replaced
       the restart part (3.3, step 8) say where its words show.
     - **The panic row now names an address, not a script,** to match the
       plan's `panic(setup, address, failed)` and examples.md 9.4. The
       module panics at start, so each reload panics again and the page
       reaches the limit with no person acting. A script's act, made once,
       would not.
     - **The reloads within the limit need nothing from the application:**
       the kit sees each load and each panic's `Uncaught`. How the story
       counts them is the plan's.

     ### Landed

     - skein `7c5a482`, "Name what skein's browser stories take from an
       application": ui-web.md 14.2 (the table's three rows and a new
       bullet) and 14.3; examples.md 9.4.
     - jig `e240ff1eb`, "skein-ui: what skein 14's stories take, through
       14.2 and 14.4": `skein/14-stories.md` (What the design asks; 14.2's
       Don't; 14.4's Shape; the sketch).
     - Nothing deferred: no `later.md` row.

     ### The text it rests on

     - ui-web.md 5.1 (unchanged), the parts' table: "| restart | the words
       for "the application failed"; its limit, three reloads in a minute by
       default; its key | …". 3.2 (unchanged): "the parts' own words in the
       person's language (section 5), from its phrasebooks".
     - ui-web.md 14.2 (skein `7c5a482`, lines 873, 875, 876):
       - "| typing while a patch arrives | the field keeps its focus, its
         caret and its words | a field; a second person's script whose act
         patches the first person's page; and what that patch shows there,
         which the page does not show before it |"
       - "| a panic | the page reloads within the restart limit, and past it
         says the application failed | an address at which its module panics
         at start, as its test build allows; and its words for "the
         application failed" (5.1), which its page then shows |"
       - "| a write refused | a write the small store refuses fails the
         step's exchanges, and none leaves | an address at which its small
         store is bounded below a step's write, as its test build allows; a
         script making that step there; and what the step would have made,
         which no page then shows |"
       - The new bullet: "**What the application names is what a story
         takes,** and nothing else. What a story must see on a person's page
         is named as what a person looks for (ui.md, 17.1), and the story
         awaits it: never for a fixed time, and never by skein's words where
         the page shows the application's, as the restart part does (5.1).
         Seeing the patch is also what shows that one came: with no patch, a
         field keeps its focus, its caret and its words anyway. The panic's
         module panics at start, so that each reload panics again and the
         page reaches the limit with no person acting; a script's act, made
         once, would not."

     ### What each step does

     - **14.2:** `stories::panic(setup, address, failed: Find)`, as the
       session file has had it since ruling 118. The Don't adds: "give
       `Restart` or the story words of skein's own: `words` and `failed` are
       the application's". The example's tier gives the words its
       phrasebook gives `Restart`, as `Find::text`.
     - **14.3:** unchanged. `write_refused(setup, address, script, absent)`
       is what the design's row now names.
     - **14.4:** `stories::typing_while_patched(setup, field, second,
       arrives)`, in this order:
       1. The first person composes in `field`.
       2. It sees `arrives` gone, a `Gone` step. The page that showed the
          field shows no patch yet.
       3. It sets a flag that the second person's `Play` waits on, as 14.3's
          second person does.
       4. The second person's act patches the first person's page.
       5. The first person sees `arrives`, a `See` step within 5 s, before
          it types again.

       The field's checks stand as they were. The example passes the note
       that its second person adds, as its list shows it. The Shape names
       `back_and_forward(setup, first, second)`, as the sketch does. The
       Don't stands: "pause by a fixed sleep: each person awaits what it
       sees".

     ### Not decided here: for 14.3's ready check

     A `Gone` step passes the first time its find is absent
     (`testing/skein-fake-person/src/lib.rs:233`: `Step::Gone { .. } =>
     found == Found::Absent`). A page that is still loading gives that too.
     So 14.3's second person, going to `/` after the first person's end,
     could pass `absent` before its page holds the server's notes. That
     order of steps is the plan's. If fixing it needs one more thing that
     the application names, it is a design question.

     ### Checked against main

     - skein `bd89b69`, before the landing:
       - `testing/skein-fake-person/src/lib.rs:34`: `pub struct Find`, with
         `Find::text` (52).
       - `lib.rs:85`: `See { find: Find, within: Duration }`, and `Gone` at
         87.
       - `testing/skein-fake-person-browser/src/harness.rs:28`: `pub
         expected: Vec<Box<[u8]>>`, and `pub trait Play` at 30.
       - `testing/skein-browser/src/browser.rs:1292`: `b"Page.loadEventFired"
         | b"Page.navigatedWithinDocument"`, and each exception's
         `exceptionDetails` text at 1299.
       - No `restart` module yet in `crates/skein-ui-web/src/`.
     - jig `e240ff1eb`, `skein/14-stories.md`:
       - 192: `pub fn typing_while_patched(setup: &Setup<'_>, field: Find,
         second: &mut dyn Play, arrives: Find) -> Outcome;`
       - 196: `pub fn panic(setup: &Setup<'_>, address: &[u8], failed: Find)
         -> Outcome;`
       - 198: `pub fn write_refused(setup: &Setup<'_>, address: &[u8],
         script: &mut dyn Play, absent: Find) -> Outcome;`
     - Commands:
       - `grep -n 'Step::Gone { .. } => found == Found::Absent\|See { find:
         Find, within: Duration }' testing/skein-fake-person/src/lib.rs`
       - `grep -n 'b"Page.loadEventFired"\|exceptionDetails'
         testing/skein-browser/src/browser.rs`
       - `git show origin/main:docs/development/plans/skein-ui/skein/14-stories.md
         | grep -n 'pub fn typing_while_patched\|pub fn panic\|pub fn
         write_refused'`

121. **skein 13's Shapes against main** (ready check of skein-13.1).
     skein 13's session file corrected against skein `9098df1` (skein 02.5 and 03.2 landed) and jig `81744cca6`: no story reads the harness's snapshot, written only on a failure and printing no focus: the harness gains its own steps `Title(words)` (13.1, the page's root named by the title), `Focused(find)` (13.3), `Taller { find }` (13.5, a box from a query with `boxes: true`), beside `Key` and `PressAt`, each through a method of `BrowserFace`; `Key` is `skein_browser::boundary::Key` and `PressAt`'s point `skein_browser::boundary::Point`; 13.1 names skein-ui-web's `lib.rs`, `tests.rs`, web-sys features and `Look` fixture, the view's `address` and `styles_per_node` 1, and keeps the shell's landed `href`s; 13.2 names `tests/ui-web/Cargo.toml`, the view's node count, goldens and `bindings`, and the face choosing an option by the kit's keys, as 14.1's fact test does, since a closed select's option has no box to press; 13.3 grows `bindings`; 13.4 drops the page part's press in a scene, landed with 09.5, and names the view's `draw` module and its scene limits (0 today), and the face; 13.5 names the shell's `Look` literal, `Dom::new` constructors and web-sys `HtmlElement`, and the fixture; `size.txt` records the cargo `web` module's bytes. A further limit, golden or node count these Files miss: change it in the same increment, and list it in the commit body.

## For the user

Long-term choices the user may veto; each default holds unless the user
says otherwise.

- **The glue pinned at 0.2.126** (`wasm-bindgen`, `wasm-bindgen-cli-support`;
  `js-sys` and `web-sys` 0.3.103), which this machine's cache holds and
  the planning built (findings/toolchain.md), not crates.io's newest,
  0.2.129. A later move is one change of four pins and the lock.
- **One JavaScript snippet of at most 10 lines in skein's UI example,**
  called by its component (the designer's default, accepted): it proves
  that the bundler and the strict policy carry an application's
  JavaScript. jig's test application carries none, and is out of this
  plan's scope.
- **The native shell stays one crate** (`jig-shell-native`, carrying
  jig's link, not the tree): later.md.
- **The example's page is the bundler's,** not its own (ui-web.md, 14.3,
  "the plan's choice"): an application's own page is tested by the
  bundler's host tests (its marker, its refusal), and the example shows
  the standard path.
- **No repin downstream of skein but jig's** (decisions.md, 3): smith
  and the repository downstream of jig take this plan's work in their
  next plans.
- **The bundle's digest encoding** (decision 8): fixed for every
  application's bundle once it lands; a change moves every digest
  directory's name, which is harmless (the page names the new one), but
  is a change of a format applications may script against.
- **SHA-256 written in the bundler** (decision 9), not taken from `ring`
  or `sha2`: the default keeps skein's outside dependencies to those
  its design names; the veto takes `ring`, already locked.
- **The example's ETag** is FNV-1a 64 over the page's bytes (skein 10.2),
  so the server needs no digest crate either: an `ETag` is a validator,
  not a security boundary.
