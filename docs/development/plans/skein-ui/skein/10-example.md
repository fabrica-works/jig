# Session skein 10: the example's view and server, thin

The UI example's first two crates: its view, step code with its records
family, its state, its list page and its codec of places; and its
server, the binary `skein-ui-example` on skein's HTTP and `drive`,
serving a bundle under `/bundle/`, the page at any other path, `GET
/api/notes`, and the strict policy on every answer. Its view's tests
run the fake person's tree face and the text shell over its pages. The
first story in Chromium is skein 11's; the form, the dialog, the chart
and the component are skein 12's and 13's.

## Read first

- skein's `docs/design/examples.md`, sections 9, 9.1 to 9.3 and 9.5;
  `ui-web.md`, sections 4.4 and 9; `ui.md`, sections 5, 6.2, 9 and 15.
- skein's `docs/design/codec.md`, sections 3 and 6 (a family and its
  drift check); `http.md`, sections 4.3 and 5; `shell.md`, sections 6.1
  and 12.
- This plan's `code-organization.md`, 1, 3, 5 and 6; `findings/audit.md`,
  3 (HTTP and `drive`).
- skein's `programming-model.md`, sections 6.3, 7 and 10;
  `testing-strategy.md`, sections 2.9 and 8.

## What the design asks

One owner, prefix `notes`; the list's page at `/` (a banner with the
name and a status saying whether the page is live, a main with a
level-1 heading and the notes as a list keyed by note) (examples.md,
9.1). Its records, a codec family of its own: a note, an add and the
list (9.2). The server: `GET /api/notes`, the bundle under `/bundle/`
immutable, the page at any other GET with an `ETag` and `no-cache`, the
strict policy and `nosniff` on every answer, bounded, listening where its
arguments say and saying so once (9.3; ui-web.md, 4.4).

## Choices this plan makes

- **The places:** `Place::{List, Note { number: u64 }, Missing}`;
  `/` is the list, `/notes/<n>` a note (decimal, no leading zero), any
  other path `Missing`; the codec ignores the query, and prints no
  trailing slash.
- **The server's arguments:** `skein-ui-example ADDRESS --bundle DIR
  [--memory BYTES]`; it prints `skein-ui-example: listening at
  ADDRESS` once (the echo's `drain`, `examples/echo/shell/src/lib.rs`),
  the line the harness finds by its prefix.
- **The list's words:** its level-1 heading is `Notes`, and with no
  notes a paragraph of the main says `No notes yet.`, so the first
  story has words to find before any add.

## Rules that bind it

- **The view is step code under the subset** and builds for wasm32; the
  server is a service: its domain names no io, its routes are its
  shell's (examples.md, 9.3).
- **The server never links the bundler** (ui.md, 2): its types' table
  is its own copy, checked against the bundler's in `tests/ui-web`.
- **The policy on every answer,** ui-web.md 4.4's text, one constant in
  the server; the server's test holds the text as its own literal, never
  the constant.

## Increments

### 10.1 The view: records, state, the list's page and places
Scope: examples.md 9.1 (the list's page), 9.2; ui-web.md 9 (the codec).
Needs: skein 01.1.
Files: new `examples/ui/view/{Cargo.toml (skein-ui-example-view),
  schema/notes-v1.schema, src/generated/v1.rs, src/lib.rs,
  src/state.rs, src/view.rs, src/places.rs, src/limits.rs,
  src/tests.rs, src/tests/fixtures.rs, src/tests/drift.rs, golden/v1/}`
  (the drift test writes and checks `golden/v1/*.bin`: 12 files for
  this family); skein's `Cargo.toml` (member, `[workspace.dependencies]`).
  The view takes `skein-codec` by path, as `tests/codec/Cargo.toml:10`
  does: it is not in `[workspace.dependencies]`. The family's pattern:
  `tests/codec` (`schema/`, `src/generated/`, `tests/drift.rs:10-37`);
  the view is step code under the root `clippy.toml`, which disallows
  `drift.rs:14`'s `is_some_and`: the copy writes
  `std::env::var_os("SKEIN_CODEGEN_WRITE").as_deref() ==
  Some(std::ffi::OsStr::new("1"))`.
Shape: records `Note { number: u64, title, body, due, minutes, label:
  Label }`, `Add { key: u64, title, body, due, minutes, label }`, `Notes
  { notes }` (bounded by the limits; not `List`, which skein-codegen
  reserves, with `Box`, `Limits`, `Option`, `Path`, `Problem` and
  `Result`: `testing/skein-codegen/src/check.rs:191-199`); `State::start(seed, path,
  records, offset, languages) -> State`; `State::step(report, now,
  wall, duties)`; `State::build(&mut View, out)`; `State::address(&self,
  Binding) -> Option<Place>`; `places::{parse(&[u8]) -> Place,
  print(Place, out)}`; `limits::limits() -> skein_ui::Limits` and `worst_case`.
Test shape: `src/tests.rs`, fixtures in `src/tests/fixtures.rs` (notes
  named `GROCERIES` and `DENTIST`): `places_print_and_parse_back`
  (a table: `/`, `/notes/7`, `/notes/07` → `Missing`, `/x` →
  `Missing`); `the_list_shows_each_note_by_its_key`;
  `the_empty_list_says_no_notes_yet`; `the_views_worst_case_fits_its_limits`;
  `records_match_their_generated_code` (drift). Each under 20 ms.
Proof: unit; 10.3 runs the page through the person and the text shell.
Don't: name a browser or a part of `skein-ui-web` in the view.
Size: about 350 lines (the generated code apart).
Removes: nothing.

### 10.2 The server: the bundle, the page, the list and the policy
Scope: examples.md 9.3; ui-web.md 4.4.
Needs: 10.1; skein 08.5 (the bundler's types table, for the check in
  `tests/ui-web`, and the bundle its proof wrote).
Files: new `examples/ui/server/{Cargo.toml (skein-ui-example-server),
  src/lib.rs (domain: notes in memory, bounded), src/routes.rs,
  src/bundle.rs (read at startup; types), src/main.rs, src/tests.rs}`;
  new `tests/ui-web/{Cargo.toml (skein-ui-web-tests), src/lib.rs,
  tests/serve.rs}`; a new `examples/ui/server/clippy.toml`, a copy of
  `examples/echo/shell/clippy.toml`, and the server's own
  `[lints.rust]` and `[lints.clippy]` tables, copied from
  `examples/echo/shell/Cargo.toml` (under the workspace's lints an
  echo-like `main` fails 15 times); skein's `Cargo.toml` (members). The
  patterns: `examples/echo/shell/src/{main.rs, lib.rs:26-79}` (startup,
  `drive`, the listening line); `tests/browser/src/serve.rs:118-238`
  (HTTP connections over `skein_http::server`, their answers
  included).
Shape: routes: `GET /api/notes` the list record as
  `application/octet-stream`; `GET /bundle/<digest>/<path>` the file,
  `Cache-Control: public, max-age=31536000, immutable`, its type by
  extension, 404 for another digest or path; any other `GET` the page,
  `ETag` (`"` 16 lowercase hex digits `"`, FNV-1a 64 over the page's
  bytes, computed once at startup: a digest of its bytes, ui-web.md
  4.4, with no dependency), `Cache-Control: no-cache`, 304 on a matching `If-None-Match`; on every
  answer the policy and `X-Content-Type-Options: nosniff`; past a limit
  (notes, a record's bytes, streams) the entrance's refusal.
Test shape: `src/tests.rs` step tests of the routes:
  `each_answer_carries_the_policy_and_nosniff`,
  `another_digest_is_not_found`, `the_page_answers_any_other_path`,
  `a_matching_etag_is_not_modified`; `tests/ui-web/tests/serve.rs`:
  `the_servers_types_are_the_bundlers` (row by row against
  `skein_ui_bundle::types::of`), `the_server_serves_a_bundle_it_did_not_make`
  (a bundle written by the bundler's library with a literal `Glue`,
  then the server's routes answering its page and files). Each under
  20 ms or 0.5 s.
Proof: process: a script kept in `logs/skein-10/serve.sh` with its
  output: runs `skein-ui-example 127.0.0.1:0 --bundle
  logs/skein-08/bundle` (skein 08.5's proof wrote it), reads its
  listening line, and `curl -si` the page, a
  bundle file and `/api/notes`, showing each status, the policy and
  the types. Once; bound 2 minutes.
Don't: answer `POST /api/notes` or `/api/events` yet (skein 12.3,
  12.4).
Size: about 450 lines.
Removes: nothing.

### 10.3 The view's pages through the person and the text shell
Scope: examples.md 9.5 (the view's step tests); ui.md 17.1.
Needs: skein 01.2 and 01.3; 10.1.
Files: `examples/ui/view/src/tests/{person.rs, text.rs}`, new goldens in
  `examples/ui/view/src/tests/golden/` (`list.txt`, `empty.txt`).
Shape: no production change.
Test shape: `the_list_prints_as_its_golden_text` (the list's page
  through `skein_ui_text::render`, by hand); `a_person_sees_each_note_on_the_list`
  (a `Person` with `See` steps over `TreeFace`); `the_empty_page_prints_its_words`.
  Each under 20 ms.
Proof: joint: the example's real view built through skein-ui, read by
  the real fake person and the real text shell. The browser, the layer
  above, is skein 11.4's.
Don't: test the browser here (skein 11).
Size: tests only.
Removes: nothing.

## Interface sketch

```rust
// examples/ui/view/src/lib.rs: what the reference shell calls (ui-web.md,
// 3.1, "names the plan's"). The shell owns one State; it ends with the page.
pub struct State { /* notes, form, label shown, place, adds pending, live */ }
/// The reports the state machine takes, one per platform happening.
pub enum Report {
    Act(BoundAct),
    Place(Place),
    Answered { exchange: u32, status: u16, body: Box<[u8]> },
    Failed { exchange: u32 },
    Due,
}
/// A small-store record, as skein-ui-web's `store::Record` (skein 12.1)
/// is, field for field; the shell converts one to the other.
pub struct Record { pub key: Box<[u8]>, pub value: Box<[u8]> }
/// What a step asks of the shell's parts, in order (ui-web.md, 3.1).
pub struct Duties {
    pub writes: Queue<Record>,
    pub removes: Queue<Box<[u8]>>,
    pub exchanges: Queue<Exchange>,
    pub show: Option<(Place, bool)>,
}
pub struct Exchange { pub id: u32, pub post: bool, pub path: Box<[u8]>, pub body: Box<[u8]>, pub bound: u32 }
impl State {
    pub fn start(seed: u64, path: &[u8], records: &[Record], offset: i32, languages: &[u8]) -> State;
    pub fn step(&mut self, report: Report, now: Time, wall: Wall, duties: &mut Duties);
    pub fn build(&mut self, view: &mut View, out: &mut Queue<Patch>);
    pub fn address(&self, binding: Binding) -> Option<Place>;
    pub fn deadline(&self) -> Option<Time>;
    pub fn closing_loses(&self) -> bool;
}
```

The reports and duties grow in skein 12 (the stream, the small store)
and skein 13 (history): each session adds its variants and fields, and
the shell's match over `Report` takes them whole.

## Questions to raise when reached

None known.

## Done when

`heavy cargo nextest run -p skein-ui-example-view -p
skein-ui-example-server -p skein-ui-web-tests` passes, and `heavy cargo
build -p skein-ui-example-view --target wasm32-unknown-unknown` passes.
