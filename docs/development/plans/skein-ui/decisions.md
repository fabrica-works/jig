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
