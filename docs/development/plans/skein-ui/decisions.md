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
