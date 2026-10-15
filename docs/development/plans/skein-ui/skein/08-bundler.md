# Session skein 08: the bundler

`skein-ui-bundle`, a tool, makes an application's bundle: a page beside
one directory named by the digest of everything in it, holding the
starter, the glue and the module, `skein-ui.css`, the stylesheets, the
pictures and the files, from a closed list of types (ui-web.md, 4.2 to
4.4, 12). Its library is tested on the host with the glue's output
given as data; the glue's generation through `wasm-bindgen-cli-support`
is proved once on a scratch module, and in the gate from skein 11.4.

## Read first

- skein's `docs/design/ui-web.md`, sections 4.1 to 4.4 and 12; `ui.md`,
  section 2 (the crate).
- This plan's `decisions.md`, 8 and 9; `code-organization.md`, 1 and 6;
  `findings/toolchain.md`, 1.
- skein's `testing-strategy.md`, section 8 (tools built optimized).

## What the design asks

The bundle is a directory written whole: `index.html`, and a directory
named by the first 16 hex digits of SHA-256 over its files' paths and
bytes, in the order of their paths, holding `boot.js`, the glue
(`<name>.js`, `<name>_bg.wasm`, `snippets/`), `skein-ui.css` then the
stylesheets in order, `pictures/<owner prefix>/`, `files/`. The same
inputs give the same bytes. The page is the bundler's, or the
application's own with each `{bundle}` replaced by `<prefix><digest>/`
and refused without one. Only the closed list's extensions; a
stylesheet named `skein-ui.css`, or two of one name, refused, naming
the file (ui-web.md, 4.2, 4.3).

## Choices this plan makes

- **The digest's encoding** (ui-web.md, 4.2, "the plan's"): for each
  file of the directory, in the byte order of its path relative to the
  directory (`/` between segments), the path's bytes, one zero byte,
  the file's length as 8 bytes little-endian, then its bytes; SHA-256
  over all of it; the first 16 lowercase hex digits (decisions.md, 8).
- **SHA-256 is the bundler's own,** about 70 lines in
  `crates/skein-ui-bundle/src/sha256.rs`, checked against FIPS 180-4's
  examples: skein adds no dependency from outside the workspace beyond
  the glue the design names (decisions.md, 9).
- **The glue apart from the bundle:** `glue(&Path module, name) ->
  Result<Glue, Refusal>` calls `wasm-bindgen-cli-support`; `bundle(&Inputs,
  &Glue, out: &Path) -> Result<Digest, Refusal>` writes the directory
  from it. Host tests give `bundle` a `Glue` of literal bytes.
- **The binary's arguments,** one per input: `skein-ui-bundle --module
  PATH --name NAME --prefix /PREFIX/ --out DIR [--stylesheet PATH]…
  [--pictures OWNER-PREFIX=DIR]… [--page PATH] [--files DIR]
  [--noscript WORDS]`; it prints the digest directory's path on success,
  and a refusal's line on stderr with exit status 1.

## Rules that bind it

- **Built optimized, never linked into a service or a module:**
  `[profile.dev.package.skein-ui-bundle] opt-level = 3` in skein's root
  `Cargo.toml`; no crate but `tests/ui-web` depends on it: `rg -l
  'skein-ui-bundle' -g Cargo.toml` lists only the root's, the crate's
  own and `tests/ui-web/Cargo.toml`.
- **No wasm-pack, npm or binaryen** (ui-web.md, 4.1); no inline script
  or style in what it writes (ui-web.md, 4.4).
- **Its tests write under a scratch directory** (`skein-scratch`), never
  the source tree.

## Increments

### 08.1 The crate, its inputs and the digest directory
Scope: ui-web.md 4.2 (the directory and its digest).
Needs: nothing.
Files: new `crates/skein-ui-bundle/{Cargo.toml, src/lib.rs,
  src/sha256.rs, src/digest.rs, src/tests.rs}`; skein's `Cargo.toml`
  (`[workspace.dependencies] skein-ui-bundle`, the optimized profile).
Shape: as the sketch: `Inputs`, `Glue`, `Digest`, `bundle` writing the
  digest directory (glue, module, snippets, stylesheets, pictures,
  files; the starter and the page come in 08.2, `skein-ui.css` in 08.3)
  into `out` after removing what `out` held.
Test shape: `src/tests.rs`, with `skein_scratch::Scratch` and a
  fixture `inputs(&Scratch) -> Inputs` and `glue() -> Glue` of literal
  bytes: `sha256_gives_fips_examples` (`"abc"` and the 448-bit message,
  digests as hex literals); `the_digest_covers_paths_and_bytes` (one
  byte of one file changed, another digest); `the_same_inputs_give_the_same_bundle`
  (two bundles, every file's bytes equal). Reference test:
  ```rust
  #[test]
  fn sha256_gives_fips_examples() {
      assert_eq!(
          hex(&sha256::digest(b"abc")),
          "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
      );
  }
  ```
  Each under 20 ms.
Proof: unit; nothing calls the library until 08.5's binary and skein
  11.4's gate.
Don't: depend on `sha2`, `ring` or another crate for the digest.
Size: about 250 lines.
Removes: nothing.

### 08.2 The page and the starter
Scope: ui-web.md 4.3.
Needs: 08.1.
Files: `crates/skein-ui-bundle/src/page.rs`; `src/tests.rs`.
Shape: `boot.js` is `import init from "./<name>.js";\ninit();\n`; the
  bundler's page as ui-web.md 4.3 lists it (charset, viewport, title of
  the name, `skein-ui.css` then each stylesheet by `<link
  rel="stylesheet" href="<prefix><digest>/…">`, `<script type="module"
  src="<prefix><digest>/boot.js">`, `<div id="app"></div>`,
  `<noscript>` with the given words), text escaped as HTML text; an own
  page copied as bytes with each `{bundle}` replaced, refused without
  one (`Refusal::NoMarker { page }`).
Test shape: a golden `src/tests/golden/index.html` (by hand) for the
  fixture's inputs; `an_own_page_gets_the_directory_at_each_marker`;
  `an_own_page_without_the_marker_is_refused`;
  `references_go_under_the_prefix`. Each under 20 ms.
Proof: unit; the browser loads it first in skein 11.4.
Don't: read the application's page as HTML.
Size: about 100 lines.
Removes: nothing.

### 08.3 `skein-ui.css`
Scope: ui-web.md 12, 8 (the fallback hidden, acting marks).
Needs: 08.1.
Files: new `crates/skein-ui-bundle/skein-ui.css` (included with
  `include_bytes!`); `src/lib.rs` writes it first in the directory.
Shape: rules for `[data-drawn] > :not(svg)` hidden from sight only
  (clipped, taking no space), the scene's SVG at its figure's top left,
  marks stroked in `currentColor` with no fill, `[data-act]`
  `pointer-events: all`, `:focus-visible` outlines kept, `@media
  (prefers-color-scheme: dark)` and `(prefers-reduced-motion: reduce)`;
  no token's class, no region placed, no palette.
Test shape: `the_stylesheet_names_no_class` (no `.` selector in the
  file's bytes outside comments); `skein_ui_css_comes_first_in_the_page`.
  Under 20 ms each.
Proof: unit; skein 13.4 shows its rules in Chromium (the fallback
  hidden while the chart is drawn).
Don't: style a token or place a region (ui-web.md, 12).
Size: about 50 lines of CSS, 10 of Rust.
Removes: nothing.

### 08.4 The closed list and each refusal
Scope: ui-web.md 4.2 (only the closed list), 4.4 (the types).
Needs: 08.1.
Files: `crates/skein-ui-bundle/src/lib.rs` (`Refusal`), new
  `src/types.rs`; `src/tests.rs`.
Shape: `types::of(path) -> Option<&'static str>`, ui-web.md 4.4's
  eleven extensions and their types; the example's server, which never
  links the bundler (ui.md, 2), keeps its own copy, which skein 10.2
  checks against this one in `tests/ui-web`; `Refusal::{Type { file }, ReservedName {
  file }, SameName { file }, NoMarker { page }, Io { path }, Glue}`.
Test shape: a table test `each_refusal_names_its_file` (a `.xml` file,
  a stylesheet named `skein-ui.css`, two stylesheets named `a.css`);
  `each_extension_has_its_type` (eleven rows, types literal). Under
  20 ms.
Proof: unit; skein 10.2 serves a bundle it made, by its copy of the
  table, checked row by row against this one.
Don't: add a type (ui-web.md, 16: media waits).
Size: about 60 lines.
Removes: nothing.

### 08.5 The glue, and the binary
Scope: ui-web.md 4.1 (the glue by its library), 4.2 (a small binary).
Needs: 08.1 to 08.4.
Files: `crates/skein-ui-bundle/src/glue.rs`, `src/main.rs`;
  `Cargo.toml` (`wasm-bindgen-cli-support = "=0.2.126"`, a workspace
  dependency; findings/toolchain.md, 1).
Shape: `glue(module: &Path, name: &str) -> Result<Glue, Refusal>`:
  `Bindgen::new().input_path(module).web(true)`, its JavaScript, its
  module and its snippets in memory; `main` reads the arguments of
  "Choices", calls `glue` then `bundle`.
Test shape: `the_arguments_make_inputs` (a table of argument lists and
  the `Inputs` or the usage line expected). Under 20 ms.
Proof: process, a script kept in the run's `logs/skein-08/glue.sh`
  with its output: builds a one-function `cdylib` with `wasm-bindgen =
  "=0.2.126"` in a scratch directory for `wasm32-unknown-unknown`, runs
  `cargo run -p skein-ui-bundle -- --module … --name probe --prefix
  /bundle/ --out logs/skein-08/bundle` (in the run's directory, where
  skein 10.2's proof serves it), and lists the bundle: `index.html`, and the digest
  directory with `boot.js`, `probe.js`, `probe_bg.wasm` and
  `skein-ui.css`. Once; bound 5 minutes. skein 11.4 makes it the gate's.
Don't: call a `wasm-bindgen` binary or download one.
Size: about 90 lines.
Removes: nothing.

## Interface sketch

```rust
// crates/skein-ui-bundle/src/lib.rs: the bundler's library. The gate's
// script and an application's build call the binary; tests/ui-web calls
// these.

/// What the application gives.
pub struct Inputs {
    pub name: String,
    /// One or more segments, with one leading and one trailing `/`.
    pub prefix: String,
    pub stylesheets: Vec<PathBuf>,
    /// Each owner's prefix and its pictures' directory.
    pub pictures: Vec<(String, PathBuf)>,
    pub page: Option<PathBuf>,
    pub files: Option<PathBuf>,
    pub noscript: String,
}
/// The glue wasm-bindgen generated for a module.
pub struct Glue { pub js: Vec<u8>, pub wasm: Vec<u8>, pub snippets: Vec<(String, Vec<u8>)> }
/// The digest directory's name: 16 ASCII lowercase hex digits.
pub struct Digest(pub [u8; 16]);
pub enum Refusal {
    Type { file: PathBuf },
    ReservedName { file: PathBuf },
    SameName { file: PathBuf },
    NoMarker { page: PathBuf },
    Io { path: PathBuf },
    Glue,
}
pub fn glue(module: &Path, name: &str) -> Result<Glue, Refusal>;
pub fn bundle(inputs: &Inputs, glue: &Glue, out: &Path) -> Result<Digest, Refusal>;
```

## Questions to raise when reached

None known.

## Done when

`heavy cargo nextest run -p skein-ui-bundle` passes, and `rg -n
'opt-level = 3' Cargo.toml` prints the bundler's profile line.
