# Session jig 04: jig takes skein-ui, skein-ui-text and skein-fake-person

jig repins skein and takes the moved crates: `jig_view::ui` goes, and
jig's users name `skein_ui`; `jig-shell-text` and `jig-fake-person` go,
and their users name skein's. jig's gate stays green; nothing a person
sees changes.

## Read first

- jig's `docs/design/client/domain/view.md`, sections 2 and 15.
- skein's `docs/design/ui.md`, section 2.
- This plan's `decisions.md`, 4 and 5; `code-organization.md`, 1 and 6.
- skein's `development.md`, 5.7, step 1 (the repin's command).

## Rules that bind it

- **jig moves skein only, to skein 01's last landing** (decisions.md,
  4): `heavy cargo update -p skein-lib --precise "$(git -C ~/src/skein
  log origin/main --grep='^Step: skein-01.3$' --format=%H -1)"`, never
  smith; a `cargo check` failing inside a smith crate stops the
  increment with a question. The pin is that commit, not skein's newest
  `main`: skein's later changes to skein-ui (skein 02 to 06) are jig
  05's to take, and no session changes skein-ui, skein-ui-text or
  skein-fake-person between skein 01.1 and 01.3 (each needs skein
  01.3).
- **No re-export or alias of skein-ui's items in jig-view**
  (code-organization.md, 6).
- **The repin is a commit of its own,** at 04.1's start (development.md,
  5.3).

## Increments

### 04.1 jig-view on skein-ui
Scope: view.md 2.
Needs: skein 01.3 on skein's `origin/main` (the three crates moved).
Files: `Cargo.lock` (the repin commit, to skein 01.3's landing); `Cargo.toml`
  (`[workspace.dependencies] skein-ui = { git = … }`);
  `crates/jig-view/Cargo.toml`; `crates/jig-view/src/ui/` removed with
  its tests (`ui/tests/`, `tests/ui_boundary.rs`); every
  `jig_view::ui::` and `crate::ui::` in jig (`rg -l
  'jig_view::ui::|crate::ui::' --type rust`) reading `skein_ui::`; the
  `Cargo.toml` of each crate naming the tree's types (the seven of
  findings/audit.md) adding `skein-ui`.
Shape: no change of code but paths.
Test shape: none new; jig-view's generic tests went to skein with skein
  01.1.
Proof: joint. `heavy cargo nextest run -p jig-client-world -p
  jig-ops-world -p jig-client-protocol-world` passes once: jig's pages
  build, diff and decode on skein's crate. Bound: 4 minutes.
Don't: move smith; keep `src/ui/`.
Size: about 20 lines, plus paths.
Removes: `jig_view::ui`.

### 04.2 jig's terminal on skein-ui-text
Scope: view.md 15.
Needs: 04.1.
Files: `crates/jig-shell-text/` removed; `Cargo.toml` (member and
  dependency lines); `tests/client` and `tests/client-protocol` (23
  lines, findings/audit.md) naming `skein_ui_text`.
Shape: jig's settings, `skein_ui_text::Settings { name, longest,
  interval, status: Some(JigRegion::Link.region()) }`, made where they
  are made today.
Test shape: none new; the golden tests went with skein 01.2.
Proof: world. `heavy cargo nextest run -p jig-client-world -p
  jig-client-protocol-world -E 'binary(text_pages) |
  test(/at_the_text_shell|terminal/)'` passes once: jig's pages print
  through skein's shell as before. Bound: 2 minutes.
Don't: copy a skein-ui-text test back into jig.
Size: about 10 lines, plus paths.
Removes: `jig-shell-text`.

### 04.3 jig's worlds on skein-fake-person
Scope: jig's `client/domain/testing.md`, 2.
Needs: 04.1.
Files: `testing/jig-fake-person/` removed; `Cargo.toml`; `tests/client`
  (25 lines in 21 files), `tests/client-protocol` (12), `tests/ops` (4)
  naming `skein_fake_person`.
Shape: paths only.
Test shape: none new.
Proof: world. `heavy cargo nextest run -p jig-client-world -p
  jig-ops-world -p jig-client-protocol-world` passes once, and the
  fuzzy suite's client sweeps: `heavy cargo nextest run -p
  jig-client-world --profile fuzzy`. Bound: 5 minutes.
Don't: change a story.
Size: paths only.
Removes: `jig-fake-person`.

## Done when

`ls crates/jig-shell-text testing/jig-fake-person crates/jig-view/src/ui`
fails for each, and `rg -n 'jig_view::ui|jig_shell_text|jig_fake_person'
--type rust` prints nothing.
