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

122. **what a story sees before it looks for something absent** (the designer's answer of 2026-10-15T14:21Z, no step blocked).
     The designer's answer to the question of 2026-10-15T14:15:41Z (when a write-refused story may see `absent` gone), landed as skein `1ff0531` (ui-web.md 14.2 and 14.3; examples.md 9.2 and 9.4) and jig `20a050caa` (skein/14-stories.md 14.3, 14.4 and the sketch; skein/12-transport.md 12.4), follows.


     **Decision: the supervisor's reading.** The story takes a fourth `Find`,
     `loaded`: what a page shows once it holds its server's state. The
     second person sees it with a `See` step, and only then plays `Gone` on
     `absent`. Two more decisions go with it:

     - **Typing while a patch arrives takes `loaded` too.** Its first person
       sees `loaded` before it composes, so that its `Gone` on `arrives` (my
       answer of 14:14Z) means something. That answer said "the page that
       showed the field shows no patch yet". That reason is wrong: the
       example's form shows before its list is read.
     - **For the example, `loaded` is its status saying Live.** examples.md
       9.2 now defines when the status says it: from the list's answer, read
       after the stream opened, until the stream ends.

     Why:

     - **Not a `Gone` held for a time.** That is a fixed wait, which 14.2's
       bullet refuses.
     - **Not the kit's load event** (`browser.rs:1292`). It comes before the
       list's exchange is answered.
     - **Not the example's "No notes yet."** On main the list page says it
       from the start, before any read:
       - `state.rs:48` starts with no notes;
       - `view.rs:61-63` writes phrase 3, "No notes yet.", when the notes
         are empty.

       Its meaning would also depend on the server holding no notes.
     - **Why Live works.** A page that says Live has read the list after its
       stream opened. In the story, the second person's page starts again
       after the first person's script has ended, and the refused step's
       exchange, had it left, went in that step's turn. So the list read
       after it holds the note, and the status says Live only once that read
       is answered.

     ### Landed

     - skein `1ff0531`, "Name what a browser story sees before it looks for
       something absent":
       - ui-web.md 14.2: two rows of the table, and the bullet;
       - ui-web.md 14.3;
       - examples.md 9.2 and 9.4.
     - jig `20a050caa`, "skein-ui: `loaded` through skein 14.3 and 14.4, and
       12.4's live status": `skein/14-stories.md` (14.3, 14.4, the sketch)
       and `skein/12-transport.md` (12.4's Shape and Test shape).
     - Nothing deferred: no `later.md` row.

     ### The text it rests on (skein `1ff0531`)

     - ui-web.md 14.2, line 876: "| a write refused | … | an address at
       which its small store is bounded below a step's write, as its test
       build allows; a script making that step there; what the step would
       have made, which no page then shows; and what a page shows once it
       holds its server's state |". Line 873, typing, ends the same way: "…
       which the page does not show before it; and what a page shows once it
       holds its server's state |".
     - The bullet, from line 882: "A story looks for something absent only
       once its page shows that it holds its server's state, which the
       application names too: what a page has not read yet is absent as
       well, and the page's load, which the kit sees, comes before its first
       read is answered."
     - examples.md 9.2, from line 536: "The page is live from the list's
       answer, read after the stream opened, until the stream ends: the
       banner's status says Live then, and Offline otherwise; an add's
       exchange failing leaves it as it is. A page that says Live holds the
       server's notes as of its read, and each made since, which skein's
       stories see before they look for a note that is not there (9.4)."
     - examples.md 9.4, line 619, and ui-web.md 14.3, line 906: the example
       gives "the stories that look for something absent" its status saying
       Live.

     ### What each step does

     - **12.4:**
       - **Shape:** the view is live from the list's answer, read after the
         stream opened, until the stream ends. The status says `Live` then
         and `Offline` otherwise. An add's failed exchange leaves it as it
         is: today `state.rs:70`, `Report::Failed { .. } => self.live =
         false`, ends it on any failure.
       - **Test shape:** a view test,
         `live_from_the_lists_answer_until_the_stream_ends`.
     - **14.3:**
       - **Signature:** `write_refused(setup, address, script, absent,
         loaded)`.
       - **Order:** the second person, after the first person's flag, goes
         to `/`. It sees `loaded` with a `See` step within 5 s, then
         `absent` gone with a `Gone` step within 1 s.
       - **Needs:** skein 12.4 as well as 12.3, for the live status. This is
         an order change that follows from the answer. 14.3 already waits on
         14.2, which waits on 14.1 and skein 02.5.
       - **Unchanged:** the Test shape, the Proof and the Don't.
     - **14.4:**
       - **Signature:** `typing_while_patched(setup, field, second, arrives,
         loaded)`.
       - **Order:** the first person sees `loaded` within 5 s, composes in
         `field`, sees `arrives` gone, then sets the flag. The rest is
         unchanged.
     - **The example's tier:** it gives both stories `Find::text(b"Live")`.
       `Live` is phrase 1 of the view's phrasebook (`view.rs:12`).

     ### Checked against main

     - skein `83a0cec`, before the landing:
       - `examples/ui/view/src/state.rs:48`: `State { notes, place:
         places::parse(path), live: false, … }`;
       - `state.rs:70`: `Report::Failed { .. } => self.live = false,`;
       - `view.rs:12`: `[b"Notes", b"Live", b"Offline", b"No notes yet.",
         …]`;
       - `view.rs:61-63`: `fn list` writes `phrase(state, 3)` when the notes
         are empty;
       - `testing/skein-fake-person/src/lib.rs:233`: `Step::Gone { .. } =>
         found == Found::Absent`;
       - `testing/skein-browser/src/browser.rs:1292`: `b"Page.loadEventFired"
         | b"Page.navigatedWithinDocument"`.
     - jig `20a050caa`, `skein/14-stories.md`:
       - line 198: `pub fn typing_while_patched(setup: &Setup<'_>, field:
         Find, second: &mut dyn Play, arrives: Find, loaded: Find) ->
         Outcome;`
       - line 204: `pub fn write_refused(setup: &Setup<'_>, address: &[u8],
         script: &mut dyn Play, absent: Find, loaded: Find) -> Outcome;`
     - Commands:
       - `grep -n 'Report::Failed { .. } => self.live = false\|fn
         list\|phrase(state, 3)\|live: false' examples/ui/view/src/state.rs
         examples/ui/view/src/view.rs`
       - `grep -n 'b"No notes yet."' examples/ui/view/src/view.rs`
       - `git show origin/main:docs/development/plans/skein-ui/skein/14-stories.md
         | grep -n 'pub fn typing_while_patched\|pub fn write_refused'`

123. **skein 13 against 12.3** (re-check of skein-13.1 after skein-12.3).
     skein 13's session file re-checked after skein 12.3 (`890b1a1`): the lines it cites moved: the view's `address` at `state.rs:145`, its limits at `limits.rs:11-18` (`bindings` now 66), its node count at `tests.rs:87` (461), the shell's `Look` literal at 56, `Dom::new` at 61 and its `href`s at 121-124; nothing else ruling 121 names changed.

124. **skein 14.2's renderer crash: a bounded diagnosis** (process, skein 14's block of 2026-10-15T14:41:18Z).
     skein 14.2's third `Crash` is a renderer crash: the kit reports `Crash` only on Chromium's `Inspector.targetCrashed` and `Target.targetCrashed` (`testing/skein-browser/src/browser.rs:1331-1349`), and the failure's Chromium stderr names no reason. The implementer's proposal holds: a diagnosis, below the story's layer, with `Setup.expected` holding `Uncaught` alone. Its bound: 25 minutes, and at most 5 runs through `heavy`, counted apart from the proof's runs; the proof runs once, after a change aimed at the cause the diagnosis names.

     1. **Rebase `skein-ui/skein-14-2-restart` on skein's `origin/main` first.** Its base predates skein 12.3 (`890b1a1`), whose `a_reload_keeps_a_pending_add_made_once` (`tests/ui-web/tests/browser_example.rs:21`) reloads the example's page through the kit (`Page.reload`) and passes on `main`.
     2. **Chromium's reason, one run.** In the worktree, not committed: add `--enable-logging=stderr` and `--log-level=0` to the list in `testing/skein-browser/src/command.rs` (47-59), and run the focused story once. The kit keeps the last 32,768 bytes of Chromium's stderr (`limits.rs:38`), which the failure prints. What counts: a line from the renderer's process near the crash naming a signal, `Check failed`, `FATAL`, `OOM` or `bad_message`. A stderr naming nothing is a finding too: go on to 3.
     3. **The reduction, two runs.** In the kit's world package (`Rig`, `tests/browser/tests/support/real.rs:157`; `serve::Pages`, `tests/browser/src/serve.rs:52`), one scratch test with a fixed page whose inline script, once per page, guarded by a session storage key: (a) calls an export of a minimal Wasm module that executes `unreachable`, then `location.reload()` from a `setTimeout`; (b) the same with a JavaScript `throw` in place of the trap. Each run: does the kit report `Crash`?
     4. **What follows:**
        - The cause is skein's code (the hook, the glue, the bundle; the minimal page does not crash): fix it in 14.2; the story unchanged.
        - Chromium crashes on the minimal page too, and a flag in the kit's command stops it: add the flag to `command.rs`'s list, with an assertion in `chromium_pipe_command` (91), and keep the reduction as a fact in `tests/browser/tests/browser_facts.rs`, beside 14.1's (ui-web.md, 14.1: the facts the face assumes). Name both in the commit body.
        - Chromium crashes on the minimal page, and no flag stops it, or the bound is reached without a cause: stop, with a question that quotes the stderr line and each run's result. The design says the restart part reloads the page (ui-web.md, 5: "the restart part reloads the page, which starts the application again from its small store"); a change to that is the designer's.

     Unchanged: the panic story's `Setup.expected` holds `Uncaught` and nothing else; no story or harness accepts `Crash`; the restart part reloads the page; the logging flags of step 2 leave the worktree once step 2 is done. A further copy of the kit's argument list that a flag would change (a test asserting the list): change it in the same commit, and list it in the commit body.

125. **skein 14.2's crash: the kit's queries during a page's own reload** (process, skein 14's block of 2026-10-15T14:52:32Z).
     Ruling 124's diagnosis narrowed the crash to a page's own reload under the kit: the JavaScript reduction crashes with no Wasm, hook, glue or bundle, while the kit's own reload of the example's page passes on `main` (`a_reload_keeps_a_pending_add_made_once`). Its reduction asks the kit, as soon as the page opens, to await the heading `Reloaded` (`14.2-reduction-rig.patch`: `Request::Await … within: Duration::from_secs(2)`), so the kit queries the page while the page's reload is in flight; a renderer crash from a query during a navigation is the cause this ruling looks for first. A second bounded diagnosis, by skein 14, in the same scratch: 20 minutes, and at most 4 runs through `heavy`, counted apart from the proof's.

     1. **The commands sent, one run.** The JavaScript reduction, with a scratch `eprintln!` in `queue_page` (`testing/skein-browser/src/browser.rs:384`) of each method it sends and the time, and one in the handler of `Inspector.targetCrashed` (1331). What counts: the methods sent in the 300 ms before the crash, quoted in the report.
     2. **No query during the reload, one run.** The same reduction, its Rig asking for nothing until 1 s after the page opened (a `std::thread::sleep` in the scratch Rig alone: this is a diagnosis, not a test), then the same `Await`. Crash, or `Reloaded` seen?
     3. **The control, one run.** The reduction with neither trap nor throw, the `setTimeout(() => location.reload(), 0)` alone, the `Await` at once as in ruling 124. Crash?
     4. **What follows:**
        - Run 2 passes (the queries during the reload are the cause): the kit holds its page commands that read the page (the `Accessibility.*` and `DOM.*` ones of `queue_page`'s callers) from a navigation the page starts until that navigation's `Page.loadEventFired` (browser.rs:1292), then sends them; an `Await` keeps its deadline. In 14.2, with a test `chromium_survives_a_page_that_reloads_itself` in `tests/browser/tests/browser_real.rs`, the reduction kept, both variants, and the story unchanged. Aim for at most 80 lines of production code; past 120, stop and report the shape, and this becomes a step of its own.
        - Run 2 crashes too, with the commands of run 1 none in the 300 ms before it: stop, with a question quoting runs 1 to 3. The supervisor takes it to the designer: ui-web.md, 5, says "the restart part reloads the page".
        - Any other result: stop, with a question quoting runs 1 to 3.

     Unchanged: the panic story's `Setup.expected` holds `Uncaught` and nothing else; no story or harness accepts `Crash`; the restart part reloads the page; ruling 124's rule on the scratch: logging and reduction edits leave the worktree once the runs are done, but for the test the first outcome keeps. The proof runs once, after the fix. A further caller of `queue_page` that reads the page while a navigation is in flight: hold it too, and list it in the commit body.

126. **skein 14.s1: the kit's finds across a page's own navigation** (process, skein 14's block of 2026-10-15T15:04:14Z).
     Ruling 125's run 4 named the cause: a find's root read before the page reloaded itself reached the renderer after the new load, and the renderer crashed (`logs/skein-14/14.2-ruling-125-diagnosis.md`: "The DOM root response from before navigation was used to construct the AX query during navigation"). The kit's fix is a kit defect of its own, not 14.2's restart part: it becomes step `skein-14.s1`, "the kit's finds across a page's own navigation", in skein 14's session, before 14.2, with its session file `skein/14.s1-finds-across-navigation.md`. Its start is the WIP's kit part (116 lines in `browser.rs`, one in `limits.rs`) and `chromium_survives_a_page_that_reloads_itself`, split from 14.2's WIP into its own commit. Its diagnosis first: at most 4 `heavy` commands and 25 minutes, each running the gate's browser line (`scripts/check.sh:31`) 5 times after the gate's build lines, with run 4's buffered trace, counting the crashes. Its proof: the browser line 5 times without a `Crash`, then the gate once. Its bound: aim for at most 160 lines of production code; past 220, stop and report the shape. A find that no longer asks CDP for the role and the name (browser.md 6: "exactly, both asked of CDP") is the designer's: stop. 14.2 then lands the restart part and the panic story on top of it, its proof unchanged.

     Unchanged: no test accepts `Crash`; the panic story's `Setup.expected` holds `Uncaught` alone; the restart part reloads the page (ui-web.md 5). skein 13.1's open question (a `beforeunload` prompt during a go) also touches `browser.rs`: whichever of the two lands second rebases on the first, and keeps both.

127. **skein 13.1: a go's closing warning, and other native dialogs** (design gap, skein 13's block of 2026-10-15T14:56:13Z).
     The designer's answer, landed as skein `506b940` and jig `c6982e600`, follows word for word.


     ### skein 13.1: a `beforeunload` prompt during the kit's go, 2026-10-15T14:56:13Z

     **Decision: the implementer's proposal, with a report and a default for
     everything else.**

     1. **The kit accepts the prompt as part of the go.** It accepts a
        `beforeunload` prompt that opens on a page while a go of that page
        is in flight (`Page.handleJavaScriptDialog`, `accept: true`). The
        go then goes on to its `Done`. No request answers a dialog, and no
        test answers one: a person who navigates, reloads or moves through
        the history has chosen to leave.
     2. **The go reports it.** It reports `Event::Warned { op }` before its
        `Done`, at most once, and `Warned` is not terminal. `Done` itself
        is unchanged, since a press, typing and a key share it. The face
        records whether its last go was warned. A new harness step,
        `Warned(bool)`, checks that record. Two stories in 13.1 check
        "registered only while it would" from both sides.
     3. **Any other native dialog is dismissed and reported as trouble.**
        This covers an alert, a confirm, a prompt, and a `beforeunload`
        with no go of its page in flight. The kit sends `accept: false` at
        once and reports `Trouble::Dialog`, whose text is `<type>:
        <message>`. The harness already fails a story on trouble it did not
        expect. A page close sends `Target.closeTarget`, which runs no
        `beforeunload` handler, so a close is never warned. The real suite
        checks this.
     4. **One more decision goes with it: the restart part clears the
        warning.** The warning is the window's `onbeforeunload`. Before the
        panic hook reloads, or says the application failed, it clears that
        handler. Without this, a page that panicked with an add pending
        would hit rule 3 on its own reload. A real person would be asked to
        stay on a page that can send nothing more.

     Why:

     - **Not a request that answers the prompt.** No story needs a person
       who stays. Staying is the browser's behaviour, not the
       application's. With a request, every go of a warning page would
       take two requests.
     - **Not a field on `Done`.** It would mean nothing for a press, typing
       or a key.
     - **Dismiss outside a go, rather than accept or leave the dialog
       open.** A dismissal commits the page to nothing: a confirm answers
       no, a prompt gives nothing, and the page stays where it is. A
       dialog left open would hold every operation on the page until its
       deadline. That is this proof's step 6, `Reload` refused `Timeout`.
     - **Not `Trouble` for the accepted prompt.** Trouble is something
       gone wrong. The warning during a go is the design working.
     - **Chromium shows the prompt only on a page with input since it
       loaded.** This is why 12.3's second reload meets no prompt and
       proves nothing about "only while". The story that looks for no
       warning has its person press `Add` on that page first.

     ### Landed

     - skein `506b940`, "Answer a go's closing warning, and dismiss other
       native dialogs":
       - browser.md 1, 5, 6, 7 and 9;
       - ui-web.md 5, 5.1, 14.2 and 14.3;
       - examples.md 9.2 and 9.5;
       - STATUS.md, the rows of browser.md and of jig's skein-ui plan.
     - jig `c6982e600`, "skein-ui: the closing warning and native dialogs
       through skein 13.1 and 14.2":
       - `skein/13-example-whole.md`: What the design asks; the harness's
         steps; 13.1's Scope, Files, Shape, Test shape, Proof, Don't and
         Size;
       - `skein/14-stories.md`: What the design asks; 14.2's Shape, Proof
         and Don't;
       - `later.md`: one row.
     - Deferred: a test answering a native dialog itself (staying on a page
       that warns, answering a confirm yes, giving a prompt words). Its row
       is in `later.md`.

     ### The text it rests on (skein `506b940`)

     - browser.md 6, the go row (line 226): "navigate, reload, or move
       through the history; the page's `beforeunload` prompt, if it opens
       meanwhile, accepted, and `Warned`; `Done` once the navigation
       commits: …".
     - browser.md 6, the bullet (from line 253): "**A go answers its page's
       closing warning.** A person who navigates, reloads or moves back or
       forward has chosen to leave the document. So the kit accepts a
       `beforeunload` prompt that opens on the page while a go of it is in
       flight (CDP's `Page.javascriptDialogOpening`, answered by
       `Page.handleJavaScriptDialog`), and reports `Warned` before the go's
       `Done`." Its sub-bullets: "Chromium shows the prompt only on a page
       that has had a person's input since it loaded. A test that looks for
       no warning acts on the page first."; "Closing a page closes its
       target, which runs no `beforeunload` handler, so a close is never
       warned."; "No request answers a dialog."
     - browser.md 5 (line 215): "`Warned` is not terminal: a go reports it
       at most once, before its terminal (section 6)."
     - browser.md 7 (from line 284): "a native dialog, other than the
       closing warning a go accepts (section 6): an alert, a confirm, a
       prompt, or a `beforeunload` prompt with no go of its page in flight.
       The kit dismisses it at once. Its text is the dialog's type, `: `,
       and its message (`confirm: Delete it?`)."
     - ui-web.md 5 (line 298): "Before either, the restart part clears the
       closing warning (5.1)."
     - ui-web.md 5.1, the table (lines 322 and 323): "`beforeunload`
       registered only while it would, as the window's `onbeforeunload`"
       and "the panic hook: the window's `onbeforeunload` cleared; a reload
       within the limit; …". The bullet at line 335: "**The closing
       warning** is the window's `onbeforeunload` handler. …"
     - ui-web.md 14.2:
       - line 861: "A go and a reload accept the page's closing warning, as
         a person who chose to go answers it";
       - line 870: the harness fails on "a policy violation, an exception
         or a native dialog";
       - line 880: it "**checks whether the person's last go met its page's
         closing warning**".
     - examples.md 9.2 (line 534): "Closing loses requests while an add is
       pending, so the page holds its closing warning then, and only then
       (ui-web.md, 5.1)."

     ### What each step does

     - **13.1** (jig `c6982e600`, `skein/13-example-whole.md`, 86-198):
       - **The warning:** `title/web.rs` sets
         `window.set_onbeforeunload(Some(..))` while `loses`, and
         `set_onbeforeunload(None)` otherwise. It never uses
         `addEventListener`.
       - **The kit:** `Page.javascriptDialogOpening` goes into
         `notification`. A `beforeunload` with the page's go in flight
         gets `accept: true` and `Event::Warned { op }`. Any other dialog
         gets `accept: false` and `Trouble::Dialog`. The new param is
         `Params::Dialog { accept }`. The reply is dropped.
       - **The face and the harness:** the face's record is reset at each
         `Doing::Go` and `Doing::Reload` and set by `Warned`.
         `harness::Step::Warned(bool)` checks it.
       - **Stories:**
         - `a_reload_with_an_add_pending_meets_the_closing_warning`, with
           12.3's `Setup`: StopServer, type, Add, 100 ms, StartServer,
           Reload, `Warned(true)`;
         - `a_reload_once_the_add_is_made_meets_no_closing_warning`: type,
           Add, see the link, Reload, `Warned(false)`.
       - **Kit step tests** in `tests/browser/src/tests.rs`:
         `a_go_accepts_its_pages_closing_warning` and
         `a_dialog_outside_a_go_is_dismissed_as_trouble`.
       - **Real tests** in `browser_real.rs`, on a page `/leave`:
         `chromium_accepts_the_closing_warning_on_a_reload` (and a close
         that meets none) and `chromium_dismisses_a_confirm_as_trouble`.
       - **Size:** about 700 lines, up from about 450.
       - **Unchanged:** `a_reload_keeps_a_pending_add_made_once`, which
         passes once the kit accepts its step 6. The fake browser opens no
         dialogs.
     - **14.2** (`skein/14-stories.md`, 122-136): the hook first calls
       `window.set_onbeforeunload(None)`, then reloads or says the words.
       The proof adds one `grep -c 'set_onbeforeunload(None)'
       crates/skein-ui-web/src/restart/web.rs`, which must print 1. No
       story checks this: the example's test address panics at start,
       before any turn sets a warning. The restart part must not reach the
       title and closing part.
     - **14.s1 and the rest:** unchanged. Ruling 126's note stands: 13.1
       and 14.s1 both change `browser.rs`, so whichever lands second
       rebases on the first.

     ### Checked against main

     - skein `22ea6c5`, before the landing:
       - `testing/skein-browser/src/browser.rs`:
         - 583-591: `admit_op` refuses a second go on a page, so a page has
           at most one go in flight;
         - 1281: `fn notification`;
         - 384: `fn queue_page`;
         - 767: a page's close sends `b"Target.closeTarget"`;
         - no `dialog` or `beforeunload` anywhere in `src`.
       - `boundary.rs`: `Done { op: Token }` at 52; `enum Trouble {
         Exception, Console, Log, Crash }` at 160-165.
       - `testing/skein-fake-person-browser/src/harness.rs:392-400`: any
         `Trouble` whose text no `Setup.expected` entry contains fails the
         story. `face.rs:139-145`: `Doing::Go` and `Doing::Reload` become
         `Request::Go`.
       - `examples/ui/view/src/state.rs`:
         - 208: `pub fn closing_loses`, true while a pending slot holds an
           add;
         - 256-272: `add_answered` takes the slot in the step that accepts
           the note;
         - 134-143: pending adds are sent again only on the first `Due`
           after a start.

         So 12.3's first reload meets the warning, and its second, on a
         page with no input, meets none.
       - `tests/ui-web/tests/browser_example.rs:21-48`:
         `a_reload_keeps_a_pending_add_made_once` presses `Add` before its
         first `Reload`, step 6.
       - web-sys `0.3.103`, `gen_Window.rs:1839`: `pub fn
         set_onbeforeunload(this: &Window, value: Option<&::js_sys::Function>)`.
         The `Window` feature is already in `crates/skein-ui-web/Cargo.toml:16`.
     - No native dialogs on any main: `git grep -n -E
       '\b(alert|confirm|prompt)\(|beforeunload' origin/main -- '*.js'
       '*.html' '*.rs'` in temper (`8d27a682`), jig and skein finds only
       Rust functions of those names.
     - Commands:
       - `git grep -n -i 'dialog\|beforeunload' origin/main --
         testing/skein-browser/src` (prints nothing);
       - `grep -n 'fn notification\|fn queue_page\|OpKind::Go'
         testing/skein-browser/src/browser.rs`;
       - `git show origin/main:examples/ui/view/src/state.rs | grep -n 'fn
         add_answered\|fn closing_loses\|if self.starting'`;
       - `grep -n 'pub fn set_onbeforeunload'
         ~/.cargo/registry/src/*/web-sys-0.3.103/src/features/gen_Window.rs`.

128. **browser.rs past its allowance, by rulings 126 and 127** (check of 2026-10-15T15:23Z).
     skein's `testing/skein-browser/src/browser.rs` grows past the README's allowance by rulings 126 and 127: 1,920 lines on skein's `main` after skein-14.s1 (`git show origin/main:testing/skein-browser/src/browser.rs | wc -l` → `1920`), and 13.1 adds the closing warning and native dialogs. The README's "Files over the plan's limit" now names both as expected, and its split is a `later.md` row: this plan does not split it.

129. **skein 13.2: a press inside a select's shadow tree** (kit defect, skein 13's block of 2026-10-15T15:56:05Z).
     Two parts, decided apart.

     1. **The select: a kit defect, fixed in 13.2.** A press on a native `<select>` hits its user-agent shadow child (`-internal-select-inner-element`, `13.2-diagnosis.md`), and the kit's `DOM.describeNode` of the select, asked without `pierce`, lists no shadow tree's descendants, so `subtree_has` (`browser.rs:939`) misses the hit and refuses `Covered`. browser.md 6 already says what holds: "`Covered` unless the hit test at that point lands on the node or inside it": a node's own shadow tree is inside it. The fix: `Params::Describe` (`params.rs:173`) also writes `"pierce": true`; nothing else in the press changes, and `Covered` stays for anything over the node that is not inside it. Its fact: `chromium_presses_a_select_inside_its_shadow_tree` in `tests/browser/tests/browser_facts.rs`, beside 14.1's (a select pressed at its centre, not `Covered`; and `chromium_rejects_a_covered_button`, `browser_real.rs:24`, still passes). About 5 lines of production code; the fact about 25.
     2. **The date field: a design question, to the designer now** (`question.md`): how the face types an `Input`'s whole value into an `<input type="date">` (and `time`), which `Input.insertText` leaves empty. Until its answer: do part 1, and the rest of 13.2 that does not type a date; change nothing in the kit's type, and keep the story's date step as the plan has it. Then wait, with the WIP committed; the answer comes as its own unblock.

     Unchanged: the face's select press through the kit's keys (13.2's Files, 14.1's `chromium_chooses_an_option_by_the_kits_keys`); the kit's hit test (`includeUserAgentShadowDOM: true`, `params.rs:109-110`). A further caller of `Params::Describe` that the pierce changes: name it in the commit body.

130. **skein 13.2: a date and a time typed by the kit's keys** (design gap, skein 13's block of 2026-10-15T15:56:05Z).
     The designer's answer, landed as skein `cef6a1a` and jig `357179a7f`, follows word for word. The README's "Files over the plan's limit" names 13.2's `keys` among the additions to `browser.rs` that it expects.


     ### skein 13.2: typing a value into a native date field, 2026-10-15T15:56:05Z

     **Decision: the supervisor's first reading, split between the kit and
     the face, with no `--lang`.**

     1. **The kit gains a request, `keys`.** It focuses the node, then sends
        each character of the text as a key down and up, with the character
        as the key and its text. `type` stays as it is: it inserts text, and
        it commits a composition left open. The kit types the characters it
        is given and does not know about dates.
     2. **The face turns a date or a time into the keys of its parts.** It
        knows the field by the role Chromium computes for it, `Date` or
        `InputTime`, which its role table already asks for. It types:
        - a date `YYYY-MM-DD` as month, day, year: `2026-10-15` as
          `10152026`;
        - a time `HH:MM` as its hour on a 12-hour clock, its minutes, then
          `A` before noon or `P` from noon: `14:30` as `0230P`, `00:00` as
          `1200A`, `12:00` as `1200P`;
        - the empty value as no key;
        - any other value fails the step, naming the field and the value.

        Every other field is still typed with `type`.
     3. **The locale is already fixed, so no `--lang`.** io's spawn gives
        Chromium the kit's environment and nothing else. In it, `LANG` and
        `LC_ALL` are `C.UTF-8`, which Chromium takes as `en-US`. browser.md
        3 now says so. A new fact checks the order of the parts against
        Chromium, so a Chromium that orders them differently fails that
        fact.
     4. **"The field cleared first" does not change.** The face still types
        only into a field that shows no value, as 11.1 does. later.md's
        select-all row stays as written.

     Why:

     - **Not the kit setting the value by a page call (the second
       reading).** The kit runs no script in the page (browser.md 1: "It
       runs no script in the page and reads no page state"; 4: "The kit
       never sends `Runtime.evaluate` or `Runtime.callFunctionOn` to run
       script"). A person's entry also never sets an attribute. A value set
       by script would test the page's `input` handler and skip the browser
       control that a person actually uses.
     - **Not dropping the date from the story (the third reading).**
       ui-web.md 14.3 asks the example's tier for "a field of each entry
       kind it uses". The date field is the one entry kind whose control is
       not a text box, so it is the one most worth checking in a real
       browser.
     - **The translation belongs in the face, not the kit.** The kit's
       vocabulary is its own, and the test translates the person's actions
       into it (browser.md 5). The face already does this for a select: it
       chooses an option with the kit's keys, and 14.1 checks that as a
       fact. The face also has what the translation needs: the value's
       format (ui.md 3.4) and the field's role from its find. The kit would
       need an extra describe to learn the input type.
     - **A new request, not a mode of `type`.** The kit has three ways to
       enter text, and `keys` sits beside `compose` as its own request. It
       is one operation with one terminal. Composing it in the face from a
       focus and one `Key` per character would put several operations on
       one page, and the kit does not promise that their steps stay in
       order.
     - **Not every field by keys.** That would change the typing in every
       landed story. It would also break `compose`'s contract, under which
       a later `Type` commits the open composition.
     - **Not `--lang`.** The environment already fixes the locale, so the
       flag would fix it a second time. On this machine I could not show
       that the flag does anything: Debian's Chromium here ships only
       `en-US.pak`, and `--lang=sv-SE` still gave `navigator.language`
       `en-US`. The fact is the check that holds on every machine.

     ### Landed

     - skein `cef6a1a`, "Type a native date or time field by keys":
       - browser.md 3, 5, 6 and 9;
       - ui-web.md 14.1 and 14.2;
       - STATUS.md, the rows of browser.md and of jig's skein-ui plan.
     - jig `357179a7f`, "skein-ui: a date and a time typed by the kit's keys
       through skein 13.2": `skein/13-example-whole.md`, What the design
       asks, and 13.2's Scope, Files, Shape, Test shape, Proof, Don't and
       Size.
     - Nothing deferred: no `later.md` row.

     ### The text it rests on (skein `cef6a1a`)

     - browser.md 3 (line 96): "The environment is minimal, and it is the
       child's whole environment (io.md, 6): `LANG` and `LC_ALL` are
       `C.UTF-8`, which Chromium takes as its `en-US`, and `TZ` is `UTC`.
       So a page renders dates and numbers the same everywhere, and a native
       date or time field shows its parts in one order (section 6)."
     - browser.md 5 (line 163): "`Keys { page: Token, op: Token, node: Node,
       text: Box<[u8]> },  // each character a key: a native date or time
       field`".
     - browser.md 6, the table (line 236): "| keys | the node focused; for
       each character of the text, in order, its key down and up, the
       character its key and its text |".
     - browser.md 6, the bullet (from line 258): "**Typing inserts; keys
       press.** `type` inserts its text as one edit, and commits a
       composition left open. A native date or time field takes no inserted
       text. Each of its parts, a number or the half of the day, takes keys,
       and a part filled moves the focus to the next. `keys` reaches it as a
       person's keyboard does." Its sub-bullets: "The kit types the
       characters it is given, in order. Which characters a field's parts
       take, and in which order, is the test's: the browser's locale sets
       them (section 3), and the face checks them as its facts (ui-web.md,
       14.1 and 14.2)." and "A key goes where the focus is. Focusing a field
       that does not have the focus puts it on the field's first part; a
       field that has it keeps it in the part it is in."
     - browser.md 9 (line 362): "The keys a native date or time field
       takes, and their order, are the face's facts, checked with its others
       in this suite (ui-web.md, 14.1)."
     - ui-web.md 14.1 (line 844): "…; a `<select>`'s option chosen by the
       kit's keys; and a date field and a time field given their values by
       the keys 14.2 names, in their parts' order under the kit's locale
       (browser.md, sections 3 and 6)."
     - ui-web.md 14.2 (line 857), the face's roles: "(a text field:
       `textbox`; `spinbutton` for a number's, `Date` for a date's and
       `InputTime` for a time's; 14.1)".
     - ui-web.md 14.2 (line 863), its acts: "type, an input's whole value
       (the field cleared, then the words), a date's or a time's by the
       kit's keys (below)".
     - ui-web.md 14.2 (from line 869): "**A date or a time is typed as a
       person types it,** into its native field's parts, since that field
       takes no inserted text (browser.md, section 6). The face knows the
       field by the role Chromium computes for it, `Date` or `InputTime`.
       It turns the value, in the element's own format (section 6), into
       the keys of the field's parts under the kit's locale, `en-US`
       (browser.md, section 3), and gives them to the kit's `keys`:" a date
       "as its month, its day, then its year: `2026-10-15` as `10152026`";
       a time "as its hour on a 12-hour clock, its minutes, then `A` before
       noon or `P` from noon: `14:30` as `0230P`, `00:00` as `1200A`,
       `12:00` as `1200P`"; "the empty value is typed as no key"; "any other
       value fails the step, naming the field and the value, since no
       person's keys put it in that field." Then (line 885): "The order is a
       fact the face assumes (14.1). A Chromium that orders the parts
       otherwise fails that fact, which names the field it typed."

     ### What each step does

     - **13.2** (jig `357179a7f`, `skein/13-example-whole.md`, 13.2's
       Files):
       - **The kit:**
         - `Request::Keys { page, op, node, text }` in `boundary.rs`, after
           `Compose`.
         - In `browser.rs`, `OpKind::Type { text, compose: bool }` becomes
           `OpKind::Type { text, entry: Entry }`, with `enum Entry { Insert,
           Compose, Keys }`.
         - After `Step::Focus`, each character gets one
           `Input.dispatchKeyEvent` `keyDown` and one `keyUp`, one at a
           time, through the existing `Params::Key`. `Done` comes after the
           last `keyUp`, or after the focus when the text is empty.
         - About 40 lines.
       - **The face:**
         - `perform`'s `Act::Input` sends `Request::Keys` for a node whose
           `Seen.role` is `Date` or `InputTime`. The emptiness check stays
           first.
         - The new pure `src/keys.rs` has `date` and `time`, each giving
           `Option<Box<[u8]>>`. `None` fails the step: `field '<name>' takes
           no value '<value>'`.
       - **Tests:**
         - kit step test `keys_type_each_character_as_its_key_down_and_up`;
         - face host tests `a_date_and_a_time_become_the_keys_of_their_parts`
           and `a_date_field_is_typed_by_the_kits_keys`;
         - the fact `chromium_takes_a_date_and_a_time_by_the_kits_keys`, on
           the facts page's existing `case-entry_date` and `case-entry_time`
           rows (`browser_facts.rs:147-148`).
       - **The story:** `a_person_fills_each_entry_kind_and_chooses_a_label`
         types `2026-10-15` as before, and the face turns it into keys.
       - **Size:** about 470 lines, up from about 250.
       - **Unchanged:** ruling 129's part 1 (the press inside a select's
         shadow tree), the kit's command, and every other field's typing.
     - **For the supervisor, not decided here:**
       - **README.md's "Files over the plan's limit"** names the three
         increments that may add to `browser.rs`: skein 07, skein-14.s1 and
         13.1. 13.2's `keys` adds about 40 lines to it as well. The README
         is yours to amend, so that the pulse's line about the file reads as
         expected.
       - **A related point, also not decided here.** The kit already sends
         `Runtime.evaluate` with the expression `document`
         (`testing/skein-browser/src/browser.rs:690`, `params.rs:52-55`),
         from skein-14.s1's finds in the current document. browser.md 4
         still says that the kit never sends it. The call runs no script
         that changes the page, but the design does not name it. If you
         want that settled, it is a question of its own.
     - **14.4 and the rest:** unchanged. Typing while a patch arrives types
       into a text field with `type`.

     ### Checked against main

     - **The code,** at skein `462e555`, whose code is the same as
       `cef6a1a`'s:
       - `testing/skein-browser/src/boundary.rs:36-38`: `Type`, `Compose`
         and `Key` (`Key { page, op, key: Key }`, the named keys alone, from
         line 107).
       - `browser.rs`:
         - 85: `Type { text: Box<[u8]>, compose: bool }`;
         - 583-594: `Request::Type` and `Request::Compose` admitted;
         - 704: `fn issue_key`;
         - 1152: `fn key_info`, the eight named keys;
         - 1858-1872: `Step::Focus` then `Input.insertText` or
           `Input.imeSetComposition`.
       - `params.rs`: `Params::Key { kind, key, code, windows, text }`.
       - `command.rs`: the arguments carry no `--lang`; the environment is
         `LANG`, `LC_ALL` (`C.UTF-8`) and `TZ` (`UTC`).
       - `crates/skein-shell/src/ring/process.rs:179-183, 265-271`:
         `pidfd_spawn` gets `envp` built from `command.env` alone, so
         Chromium has no `LANGUAGE`.
       - `crates/skein-ui-web/src/mapping.rs:367`: `Role::TextField =>
         &[b"textbox", b"spinbutton", b"Date", b"InputTime"]`.
       - `testing/skein-fake-person-browser/src/face.rs:222-230`:
         `Act::Input` refuses a field whose `seen.value` is not empty, then
         sends `Request::Type`.
       - `tests/browser/tests/support/real.rs:466-541`: the `Choose`
         scenario focuses with an empty `Type`, then sends `Key::ArrowDown`.
         This is the precedent for keys composed by the face.
     - **Chromium 147.0.7727.101** (`/usr/bin/chromium`, Debian 13), headless
       over `--remote-debugging-pipe`, with the kit's environment (`LANG`
       and `LC_ALL` `C.UTF-8`, `TZ` `UTC`). Each trial used a fresh
       `<input>`, `DOM.focus`, then `Input.dispatchKeyEvent` `keyDown` and
       `keyUp` per character, in `Params::Key`'s shape (`key`, `code`,
       `windowsVirtualKeyCode` and `text` on both):
       - `navigator.language` was `en-US`, also with `--lang=en-GB` or
         `--lang=sv-SE`, or with `LANG=de_DE.UTF-8`. `ls
         /usr/lib/chromium/locales` printed only `en-US.pak`.
       - Date fields:
         - `Input.insertText` `2026-10-15` left the value `''` and fired no
           `input` event;
         - keys `10152026` gave `2026-10-15`, with `input` events along the
           way;
         - keys `01012026` gave `2026-01-01`, and `12310001` gave
           `0001-12-31`;
         - keys `02302026` (February 30) gave `''`, as ui-web.md 6 says:
           "A value the element does not accept shows empty";
         - keys `20261015` (ISO order) gave `61015-02-02`, and
           `2026-10-15` gave `61015-02-02` too: the dash moves no part.
       - Time fields:
         - `Input.insertText` `14:30` gave `''`;
         - `0230P` gave `14:30`, `1159P` `23:59`, `1200A` `00:00`, `1200P`
           `12:00`, `0905A` `09:05`, and `0905a` `09:05`;
         - `0905` with no meridiem gave `''`. `2359P` gave `14:35`: a `2`
           fills the 12-hour field's hour at once. So the face types the
           12-hour form.
       - Fields holding a value: from the first part, the keys replace each
         part. `2020-01-02` then `10152026` gave `2026-10-15`; `09:05` then
         `0230P` gave `14:30`; `14:30` then `0905A` gave `09:05`.
       - A field that already had the focus kept it in its last part when
         `DOM.focus` came again, and the next keys went there.
       - The accessibility tree gave an empty date field role `Date` and no
         value, an empty time field `InputTime` and no value, and a filled
         one its value (`2026-10-15`, `14:30`).
     - **Commands:**
       - `chromium --version`;
       - `ls /usr/lib/chromium/locales`;
       - `grep -n 'Compose\|Key {\|pub enum Key'
         testing/skein-browser/src/boundary.rs`;
       - `grep -n 'Request::Compose\|Request::Type\|Type { text\|fn
         issue_key\|fn key_info\|Step::Focus =>'
         testing/skein-browser/src/browser.rs`;
       - `grep -n 'pidfd_spawn\|envp\|env'
         crates/skein-shell/src/ring/process.rs`;
       - `grep -n 'Date\|InputTime' crates/skein-ui-web/src/mapping.rs`;
       - `grep -n 'ENTRY_DATE\|ENTRY_TIME'
         tests/browser/tests/browser_facts.rs`;
       - the probe, a Python script over CDP's pipe (descriptors 3 and 4),
         in this session's scratch directory, which goes with the session.
         Its output is quoted above, and the 13.2 fact replaces it.

131. **skein-14.s2: a find's scope resolved in the current document** (design gap, the supervisor's question of 2026-10-15T16:15:02Z).
     The designer's answer, landed as skein `b4c7740` and jig `c8b887b29`, follows word for word. From it, the supervisor's part: step `skein-14.s2`, "a find's scope resolved in the current document", with its session file `skein/14.s2-scope-in-current-document.md`, written from the answer's "What each step does"; a new implementer, in a free place, beside skein 13, which scopes no find across a navigation. The answer's two points "for the supervisor" are deferred, each a `later.md` row: a press's or a type's node of an earlier document refused `Protocol` rather than `Gone` (the same Chromium messages come from a hidden node of the current document, so a fix tells them apart by the node's document, not by its message), and a scope removed from the current document matching nothing. The README's "Files over the plan's limit" names 14.4's small fix and 14.s2 among the additions to `browser.rs` it expects.


     ### skein-14.s1: the kit's `Runtime.evaluate` of `document`, against browser.md 4, 2026-10-15T16:15:02Z

     **Decision: the supervisor's first reading. browser.md now names the
     call. It also binds a scoped find's root to the current document, which
     needs a step.**

     1. **The kit may send `Runtime.evaluate` with the expression
        `document`, and no other expression.** That object is an unscoped
        find's root, as skein-14.s1 (`580b0ab`) already takes it.
        `Runtime.callFunctionOn` stays never. browser.md 1 and 4 now say
        so. On this point the code on `main` already matches the design.
     2. **A find's root is never a node id. It is an object that Chromium
        checks against its document.** Without a scope, that object is
        `document`'s. With a scope, it is the scope's object, resolved in
        the current document (`DOM.resolveNode`). A scope from an earlier
        document is then refused `Gone`. This second half is not on `main`,
        so it needs a step, `skein-14.s2`, which you write from "What each
        step does" below. I make this part of the answer because the probe
        below crashed Chromium 147's renderer 3 times out of 3. The probe
        found a node, reloaded, waited for the load, then sent a scoped
        `Accessibility.queryAXTree` by that node's `backendNodeId`. The kit
        sends the same command for a find `within` a node it found before
        (`browser.rs:696-702`).
     3. **The hold during a navigation is now in the design** (browser.md
        6, "A find waits out a navigation"), as `main` does it.

     Why:

     - **The rule is about what the kit does to the page and what it reads
       from it, and this call does neither.** browser.md 1's rule exists so
       that a test acts and observes only as a person does. The expression
       `document` calls nothing and reads no page state. A page also cannot
       change what it names: in Chromium 147, `Object.defineProperty(window,
       "document", …)` throws `TypeError`, `var document = 1` leaves `typeof
       document` as `object`, and `let document` is a `SyntaxError`. The
       result is a handle, as a node id is.
     - **It is the root Chromium checks.** An object belongs to the
       execution context of the document it names. After a reload, Chromium
       refuses the old object with "Cannot find context with specified id",
       both while the reload is in flight and after the load. `main`'s
       `command_error` already looks again at the next poll for that message
       (`browser.rs:1271-1277`).
     - **Not the second reading, a root by DOM commands alone.**
       `DOM.getDocument` then `DOM.resolveNode` gives the same object, and
       Chromium refuses stale ids there ("No node with given id found",
       "Node with given id does not belong to the document"). But it costs
       one more command on every look of every await, at the poll's rate,
       and a new diagnosis of a race that only shows under load. All of that
       buys only the rule's letter.
     - **Not `Accessibility.getFullAXTree` filtered in the kit.** browser.md
       6 says "exactly, both asked of CDP", and a reply sized by the whole
       tree would break `Limits::message` (section 6, the find by words).
     - **The scope gets `DOM.resolveNode`, not `Runtime`.** A scope is a
       node the test found. Resolving it is a DOM command and runs no
       script. Chromium refuses a node of an earlier document there, both
       during a reload and after it, with no crash. A scope cannot come back
       after a navigation, so the result is `Gone` (browser.md 5: "A node
       since removed is `Gone`, as a stale handle is"), not another look.

     ### Landed

     - skein `b4c7740`, "Bind a browser find's root to the current
       document":
       - browser.md 1, 4, 6 and 9;
       - STATUS.md: the browser.md row, and jig's skein-ui plan row.
     - jig `c8b887b29`, "skein-ui: the kit's one evaluated expression,
       through skein b4c7740":
       - `skein/07-browser-kit.md`: the rule on the kit's script, now the
         check that holds on `main`;
       - `skein/13-example-whole.md`: 13.2's Don't, quoting browser.md 1 as
         amended. Its instruction is unchanged.
     - Nothing deferred, so no `later.md` row. The two kit defects under
       "For the supervisor" are not this answer's to defer.

     ### The text it rests on (skein `b4c7740`)

     - browser.md 1 (line 24): "It reads no page state, and the one script
       it runs in the page is the expression `document`, which names the
       page's current document as a find's root (sections 4 and 6)."
     - browser.md 4 (from line 148): "**The one expression the kit evaluates
       is `document`.** It sends `Runtime.evaluate` with that expression
       alone, for a find's root (section 6), and it never sends
       `Runtime.callFunctionOn`." Its sub-bullets: "The expression calls
       nothing and reads no page state. A page cannot change what it names:
       a window's `document` can be neither redefined nor shadowed." and
       "Its result is a handle, as a node id is: an object of the execution
       context that holds the page's current document. The kit releases it
       (`Runtime.releaseObject`) after its find's query." The domains list
       now gives `DOM` "a scope's object" and `Runtime` "the document's
       object".
     - browser.md 6, the find row (line 241): "under the scope or the
       page's current document, either asked by its object in that document
       (below)".
     - browser.md 6 (from line 256): "**A find asks only under the current
       document.** Its root is never a node id, whose document the query
       does not check, but an object that Chromium checks against the
       document that holds it:", then "without a scope, the object of
       `document`, evaluated in the page's current execution context
       (section 4);" and "with a scope, the scope's object, resolved in the
       current document (`DOM.resolveNode`). Chromium refuses a node of an
       earlier document, and the find is refused `Gone` (section 5)." Then:
       "The kit releases the object after the query. Chromium refuses an
       object once its execution context is gone, as when the page navigates
       by itself between the object's command and the query: the find then
       looks again at its next poll, and an `Await` keeps its deadline."
     - browser.md 6 (from line 269): "**A find waits out a navigation.**
       While a navigation of the page is in flight, the kit's go or the
       page's own, the kit holds its DOM and accessibility commands to the
       page, and the evaluation of `document`, until the navigation commits,
       as a go's does. A find in flight when the navigation begins starts
       again, and asks once it has committed."
     - browser.md 9 (from line 385): the real browser's pages add "a page
       that reloads itself", and its facts add "a find across a reload (a
       page that reloads itself found again after it, a scope found before it
       `Gone`)".
     - jig `skein/07-browser-kit.md` (line 39): "**The kit sends no script
       but the expression `document`** (browser.md, 4): `rg -n
       'Runtime\.callFunctionOn' testing/skein-browser` prints nothing, and
       `rg -n 'b"expression"' testing/skein-browser/src` prints one line,
       `Params::Document`'s in `params.rs`."

     ### What each step does

     - **skein-14.s2, a scope resolved in the current document** (new; you
       write its session file and place it). The default is in skein 14's
       session, and it goes before any step that scopes a find across a
       navigation.
       - **Files:** `testing/skein-browser/src/browser.rs`; the fake's
         replies in `tests/browser/src/world.rs`; step tests beside
         `tests/browser/src/tests/navigation.rs`; a real test in
         `tests/browser/tests/browser_real.rs` with a Rig scenario in
         `tests/browser/tests/support/real.rs`.
       - **Shape:**
         - `start_find`'s `Some(root)` arm (`browser.rs:696-702`) sends
           `DOM.resolveNode` with the existing `Params::Backend(root)`.
           Today it sends `Accessibility.queryAXTree` by `backendNodeId`.
         - The reply's `object.objectId` goes to `query_document` (1667)
           the way `document`'s does. Note that `result_object` (1627)
           reads `result.objectId`, while `resolveNode` answers
           `{"object":{…}}`.
         - The late-reply path (1236-1250) releases that object too.
         - In `command_error` (1269), the refusal "Node with given id does
           not belong to the document" from that command becomes
           `Refused { why: Gone }`. Today it falls through to `Protocol`.
         - "Cannot find context with specified id", on the query that uses
           the object, still looks again at the next poll (1271-1277).
           That next look resolves the scope again, and so ends `Gone`
           after a navigation.
       - **Tests:**
         - two step tests, `a_scoped_find_asks_under_its_scopes_object` and
           `a_scope_of_an_earlier_document_is_gone`;
         - one real test, `chromium_refuses_a_scope_found_before_a_reload`.
           It finds a named region, reloads (`Go::Reload`, `Done`), finds a
           heading `within` the region, and expects `Refused { why: Gone }`
           and no `Crash`. On `main` this scenario crashes the renderer
           (the probe below, 3 of 3). About 1 s.
       - **Proof:** live, one `heavy` command: the gate's browser line
         (`scripts/check.sh:31`) 5 times with no `Crash`, then `heavy
         --gate scripts/check.sh` once.
       - **Don't:**
         - send `Accessibility.queryAXTree` with a `backendNodeId`;
         - retry a `Gone` scope;
         - accept `Crash` in any test;
         - add an evaluated expression: `rg -n 'b"expression"'
           testing/skein-browser/src` must still print one line.
       - **Size:** aim for at most 40 lines of production code. Past 80,
         stop and report the shape.
       - `browser.rs` is 2,042 lines on `main` (`git show
         origin/main:testing/skein-browser/src/browser.rs | wc -l`). Its
         allowance in the README's "Files over the plan's limit" is yours to
         amend, as ruling 128 did.
     - **Other steps:** unchanged. Unscoped finds and their hold are as
       skein-14.s1 landed them.

     ### For the supervisor, not decided here

     - **A node of an earlier document in a press or a type is refused
       `Protocol`, not `Gone`.** Chromium refuses it with no crash. I sent
       each command once after a reload with a node found before it:
       - `Accessibility.getPartialAXTree`: "Frame is detached.";
       - `DOM.scrollIntoViewIfNeeded`: "Node does not have a layout object";
       - `DOM.getContentQuads`: "Could not compute content quads.";
       - `DOM.focus`: "Element is not focusable";
       - `DOM.describeNode`: answered with a result.

       `command_error` maps only "No node found" to `Gone` (1279-1284), so
       every one of these becomes `Protocol`. browser.md 5 says such a node
       is `Gone`. This is a kit defect against design text that already
       exists, and fixing or deferring it is your ruling. It is not a crash.
     - **A scope removed from the current document matches nothing, rather
       than `Gone`.** This holds by node id on `main`, and by a resolved
       object too: the node still belongs to its document, so
       `DOM.resolveNode` answers, and the query returns no nodes. 14.s2 does
       not change it. It sits against the same browser.md 5 sentence as the
       point above, and is also yours to rule on or defer.

     ### Checked against main

     - **The code,** at skein `a57d41d` (whose code is the same as
       `b4c7740`'s), `testing/skein-browser/src/`:
       - `browser.rs`:
         - 418: `queue_page` holds `Accessibility.*`, `DOM.*` and
           `Runtime.evaluate` (`command.read`), and `sendable` (435) skips
           them while `page.navigating`;
         - 1468-1490: `navigating` is set at `Page.frameRequestedNavigation`
           and `Page.frameStartedLoading` of the page's frame, which also
           calls `restart_finds` (1573). It is cleared in `loaded` (1599) at
           `Page.loadEventFired`, `Page.navigatedWithinDocument`, or a
           `BackForwardCacheRestore`;
         - 696-704: a scoped find sends `Accessibility.queryAXTree` with
           `Params::Query { root: Some(root), object: None, … }`, and an
           unscoped one sends `Runtime.evaluate` with `Params::Document`;
         - 1667-1684: `query_document` queries by `objectId`, then queues
           `Runtime.releaseObject`;
         - 1269-1284: `command_error`.
       - `params.rs`:
         - 52-55: `Params::Document` writes `"expression": "document"`, and
           it has no field;
         - 84-91: `Params::Backend` writes `backendNodeId`, and
           `Params::Query`'s `root` writes `backendNodeId`.
     - **Chromium 147.0.7727.101** (`/usr/bin/chromium`, Debian 13),
       headless over `--remote-debugging-pipe`, with the kit's environment
       (`LANG` and `LC_ALL` `C.UTF-8`, `TZ` `UTC`), one fresh profile per
       run:
       - **On a page whose CSP gives `script-src 'unsafe-inline'` and no
         `unsafe-eval`:**
         - `Runtime.evaluate` `document` gave `{type: object, subtype: node,
           className: HTMLDocument}`, and `queryAXTree` by its `objectId`
           found `heading "Before"`;
         - the page's own scripts: `Object.defineProperty(window,
           "document", …)` threw `TypeError`, `var document = 1` left
           `typeof document` as `object`, and `let document = 2` threw
           "SyntaxError: Identifier 'document' has already been declared".
       - **The old `document` object after `Page.reload`:**
         - `queryAXTree` sent right after the reload, and again after
           `Page.loadEventFired`, both gave `-32000` "Cannot find context
           with specified id";
         - a new evaluation found the heading;
         - 0 crash events.
       - **A scope found before a reload, sent by `backendNodeId`:**
         - sent right after `Page.reload`: no reply within 10 s, then
           `Inspector.targetCrashed`;
         - sent after `Page.loadEventFired`, in 3 runs: no reply within
           10 s, then `Inspector.targetCrashed`, each time;
         - the control, a reload with no query: 0 crash events.
       - **`DOM.resolveNode` of the same scope,** 2 runs:
         - in the same document it gave an `HTMLElement`, and `queryAXTree`
           by its object found the heading;
         - during the reload and after the load it gave `-32000` "Node with
           given id does not belong to the document";
         - 0 crash events.
       - **`DOM.resolveNode` of `DOM.getDocument`'s ids after a reload:**
         the `nodeId` gave "No node with given id found", and the
         `backendNodeId` gave "Node with given id does not belong to the
         document".
       - **A scope removed by a press in the same document:** by
         `backendNodeId`, `queryAXTree` gave `[]`. `DOM.resolveNode` gave
         an `HTMLElement`, and `queryAXTree` by its object gave `[]`. 0
         crash events.
       - **A press's and a type's commands with a node from before a
         reload:** as quoted under "For the supervisor". 0 crash events.
     - **Commands:**
       - `rg -n 'Runtime\.callFunctionOn' testing/skein-browser` (prints
         nothing);
       - `rg -n 'b"expression"' testing/skein-browser/src` (prints
         `params.rs:53`);
       - `rg -n 'Runtime\.evaluate' testing/skein-browser/src` (prints
         `browser.rs:418` and `browser.rs:704`);
       - `grep -n 'fn start_find\|Params::Document\|fn query_document\|fn
         command_error\|fn result_object\|fn restart_finds\|fn loaded\|fn
         sendable' testing/skein-browser/src/browser.rs`;
       - `git show origin/main:testing/skein-browser/src/browser.rs | wc -l`
         (prints `2042`);
       - the probes: Python scripts over CDP's pipe (descriptors 3 and 4),
         in this session's scratch directory, which goes with the session.
         Their output is quoted above, and 14.s2's real test replaces the
         scoped one.

132. **skein 13.5: Taller types its words between its two looks** (plan defect, skein 13's block of 2026-10-15T16:57:21Z).
     A plan defect, not a design gap: `Taller` is the plan's harness step (skein 13's session file, the harness's steps: "`Taller { find }` in 13.5, … taller than when the step began"); the design asks only for "the body's text area that grows" (ui-web.md 14.3, step 6). The implementer's proposal holds: `Step::Taller { find, words }`, beside `Type` in `testing/skein-fake-person-browser/src/harness/script.rs` (18). The face finds `find` with `boxes: true` and keeps its box's height; types `words` as `Step::Type` types them (the kit's `Type`, at the caret); then looks at the box again at the poll's rate until it is taller, and fails the step past 1 s, naming both heights. The story `the_body_grows_with_its_words_and_reports_them` gives `Taller` its three lines as `words`, with no `Type` of them before it. No separate baseline step, and `Type` is unchanged. The rule of skein 13's stories holds: awaited, never a fixed sleep. A further user of `Step` that a new variant breaks (an exhaustive `match`): change it in the same increment, and list it in the commit body.

133. **jig 05's Shapes against main** (ready check of jig-05.1).
     jig 05's session file corrected against skein `2f8f471` and jig `f5706d976` (ready check of jig-05.1, skein 13.5 still running): the `Limits` literals are found by `rg -n 'markdown_depth' --type rust`, 33 in 30 files, three in `crates/jig-view/src/` and one in `tests/ops`, which `Limits \{` over the listed directories missed; a repin's size is measured against `git merge-base HEAD origin/main`; the fuzzy break the check met after the repair, `random_people_with_crash_commits_and_reload` on seed 4503 (a sign-in pressed 1.1 ms after a sign-out, the page ending signed out, the referee's Online deadline armed), is a question to raise when reached, after at most 3 runs that find which side is wrong, with no re-seed and no change to the referee or jig's client in 05.1; temper's four `Limits` literals are a `later.md` row. The check's suites, through `heavy` with the repair applied: `cargo check --workspace --all-targets` clean, clippy clean, nextest default 3565 of 3565, fuzzy 108 of 109.

134. **jig 05: the referee's stale frame, and a sign-out answered after a newer sign-in** (harness defect and design gap, jig 05's block of 2026-10-15T17:27:05Z).
     The seed 4503 failure has two causes (`logs/jig-05/seed-4503-findings.md`), decided apart.

     1. **The referee's stale frame: a harness defect, step `jig-05.s1`** in jig 05's session, before 05.1 lands, with its session file `jig/05.s1-referee-stale-frame.md`: a frame delivered after its watch's terminal is no evidence of a party (today `Receipt::Party` names no watch, and `referee.rs:192-194` rearms `Online` on it). It does not rest on part 2: do it now.
     2. **The earlier sign-out's `SignedOut` after a newer sign-in: a design gap, to the designer now** (`question.md`): client.md 12 does not say which sign-in a `SignedOut` answer belongs to once a newer one is authenticated. 05.1 waits for the answer, with its commits kept unlanded; the answer comes as its own unblock.

     The repin is not deferred. Unchanged: ruling 133's rules for 05.1 (no re-seed; no change to jig's client in 05.1).

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
