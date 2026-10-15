# Session skein 12: the small store, the transport, the add and the stream

The parts that protect requests (ui-web.md, 5.3, 10): the small store
and the transport, written before sent, tested on the host over a fake
platform; then the example's add, written to the small store and sent,
made once by its server however often a reload sends it; then the
stream and the hidden tab, so a note added in one person's page shows in
another's. Each grows skein 11's story in Chromium.

## Read first

- skein's `docs/design/ui-web.md`, sections 5.1, 5.2, 5.3, 10, 13 and
  14 (the host tests of the store and the transport); `examples.md`,
  sections 9.2 (an add; live) and 9.3; `ui.md`, section 13 (reports).
- skein's `docs/design/http.md`, sections 4 and 4.3 (the event stream);
  `lib.md`, section 6.1 (base64).
- This plan's `decisions.md`, 7; `code-organization.md`, 1, 3 and 6.

## What the design asks

The small store is the tab's session storage, records and keys in
base64, keys under the application's prefix, read whole at start, a
write failing past the platform's quota or the store's own bound. The
transport takes a step's writes, exchanges, stream and navigation whole:
writes first, their outcome reported; then exchanges and the stream;
every exchange of the step failed, none sent, and no navigation when a
write failed (ui-web.md, 10; 5.3). An add is written under a key from
the seed, then sent; the server makes one note per key; the record
leaves the store once answered (examples.md, 9.2). The stream is a GET
read a piece at a time, the next read asked after the step; one at a
time; a hidden tab lets it go after a minute (ui-web.md, 10).

## Choices this plan makes

- **The small store's prefix** for the example is `notes-`; its add's
  key is `notes-add-<16 hex digits of the key>`.
- **The platform the store and the transport go through** is one trait,
  `skein_ui_web::platform::Platform`: 12.1 makes it with `read_all`,
  `write` and `remove` (session storage), 12.2 adds `send` and
  `navigate`; the browser's implementation is `platform::web::Browser`,
  the host tests' `FakePlatform` in `crates/skein-ui-web/src/tests/
  fixtures.rs`, recording each call in order, with a quota.
- **The harness stops and starts the server** (ui-web.md, 14.2) in
  12.3, where a story first needs it: `Harness::stop_server(&mut self)`
  and `Harness::start_server(&mut self)`, called by the test between
  two `run`s, the server started again at the address it had; the pages
  stay open across runs.

## Rules that bind it

- **The order that protects requests is the parts':** the shell calls
  the transport with a step's duties whole; it never calls fetch or the
  small store for a step's requests itself (ui-web.md, 5.3): `rg -n
  '\.(write|remove|send)\(' examples/ui/shell-web/src` prints nothing.
- **Each story starts from skein 11's,** in `tests/ui-web/tests/
  browser_example.rs`, one test per story, each aiming at 3 s; its
  Proof is `heavy --gate scripts/check.sh` once in the increment's
  committed worktree, which builds the bundle and the server and runs
  the browser suite; nothing else.
- **The server makes an add once per key,** checked by a step test of
  the server and by 12.3's story.

## Increments

### 12.1 The small store
Scope: ui-web.md 10 (the small store), 13.
Needs: skein 11.4 (the first story).
Files: new `crates/skein-ui-web/src/store.rs`;
  `crates/skein-ui-web/src/{platform.rs, platform/web.rs}` (the
  `Platform` trait and `Browser`); `src/tests/store.rs`;
  `src/tests/fixtures.rs` (`FakePlatform`).
Shape: `platform::Platform { fn read_all(&self) -> Vec<Record>; fn
  write(&mut self, key: &[u8], value: &[u8]) -> bool; fn remove(&mut
  self, key: &[u8]); }` (raw text records, as session storage holds
  them); `store::Store::new(prefix, bound: Option<u64>)`;
  `read_all(&self, platform) -> Vec<Record>` (decoded, only the
  prefix's; `Record { key, value }`, mirrored field for field by the
  example's `Record`); `write(&mut self, key, value, platform) ->
  Result<(), Refused>`, `remove(&mut self, key, platform)`; a write past
  the bound or refused by the platform is `Refused`; `Browser` over
  `window.sessionStorage`.
Test shape: `src/tests/store.rs`: `records_are_base64_under_the_prefix`
  (the platform's raw text literal), `a_write_past_the_bound_is_refused`,
  `records_of_another_prefix_are_not_read`. Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p skein-ui-web` once; the shell
  writes through it in 12.3, whose story runs it in Chromium.
Don't: use `localStorage` (ui-web.md, 10: session storage).
Size: about 120 lines.
Removes: nothing.

### 12.2 The transport
Scope: ui-web.md 5.3, 10 (the transport); 14 (its host tests).
Needs: 12.1.
Files: new `crates/skein-ui-web/src/transport.rs`;
  `src/tests/transport.rs`; `crates/skein-ui-web/src/{platform.rs,
  platform/web.rs}` (`send`, `navigate`).
Shape: `Platform` gains `send` and `navigate`;
  `transport::carry(duties: Requests, store, fetch, platform, reports:
  &mut Vec<Carried>)`: each write, then each exchange, which `Fetch`
  records in flight and sends through `Platform::send`, then the
  navigation through `Platform::navigate`; on the first refused write, each
  exchange of the step reported failed and the navigation dropped.
  `Requests { writes, removes, exchanges, stream: Option<StreamDuty>,
  navigate: Option<Box<[u8]>> }`.
Test shape: `src/tests/transport.rs`: `writes_go_before_exchanges`
  (the fake platform's record, in order, as a literal),
  `a_refused_write_fails_every_exchange_of_its_step`,
  `a_refused_write_cancels_the_navigation`,
  `the_navigation_goes_after_the_writes`. Each under 20 ms.
Proof: unit over the fake platform, `heavy cargo nextest run -p
  skein-ui-web` once, the layer ui-web.md 14 names for it; 12.3's story
  runs it in Chromium.
Don't: send an exchange before the step's writes are done.
Size: about 120 lines.
Removes: nothing.

### 12.3 The add: written, sent, made once
Scope: examples.md 9.1 (the form's title and the add button), 9.2 (an
  add), 9.3 (`POST /api/notes`); ui-web.md 14.2 (the server stopped and
  started again).
Needs: 12.2.
Files: `examples/ui/view/src/{state.rs, view.rs}` (the form's title
  field and the add button, the add in the small store);
  `examples/ui/server/src/{lib.rs, routes.rs}` (`POST /api/notes`, one
  note per key); `examples/ui/shell-web/src/lib.rs` (the transport, the
  small store at start); `testing/skein-fake-person-browser/src/harness.rs`
  (`stop_server`, `start_server`); `tests/ui-web/tests/browser_example.rs`.
Shape: the title is a text field submitting on enter (required comes
  with the whole form, skein 13.2); the
  add button adds it; the add's record is written under its key, then
  sent; answered, it leaves the store; at start, each add record in the
  store is sent again. The server answers an add whose key it made
  before with the note it made then.
Test shape: view step tests `an_add_is_written_before_it_is_sent`,
  `an_answered_add_leaves_the_store`, `a_pending_add_is_sent_again_at_start`;
  server step test `a_second_add_with_one_key_makes_no_second_note`;
  story `a_reload_keeps_a_pending_add_made_once` (the server stopped; a
  note added, its exchange failing; the server started; the page
  reloaded; one note of that title on the list). Step tests under 20 ms; the story about 3 s.
Proof: live, `heavy --gate scripts/check.sh` once: the story shows the
  transport and the store in Chromium keeping an add across a reload.
  Bound: 15 minutes.
Don't: send an add that was not written.
Size: about 250 lines.
Removes: nothing.

### 12.4 The stream and the hidden tab
Scope: ui-web.md 5.1 (fetch's stream, the hidden tab), 10; examples.md
  9.1 (the banner's live status), 9.2 (live), 9.3 (`GET /api/events`).
Needs: 12.3.
Files: `crates/skein-ui-web/src/{fetch.rs, fetch/web.rs (the stream),
  hidden.rs, hidden/web.rs}`; `src/tests/{fetch.rs, hidden.rs}`;
  `examples/ui/server/src/routes.rs` (the event stream, on
  `skein_http`'s writer); `examples/ui/view/src/state.rs` (open the
  stream, then read the list; apply each note by its number; the live
  status); `examples/ui/shell-web/src/lib.rs`;
  `tests/ui-web/tests/browser_example.rs`.
Shape: `Fetch::open_stream(path, headers, bound)`, a piece handed in
  parts past the bound, `read_on()` after the step; `hidden::Hidden`
  (a deadline a minute after hidden; an opening while hidden held until
  shown); the stream's event data one note record in base64.
Test shape: `src/tests/fetch.rs`: `a_piece_past_its_bound_comes_in_parts`;
  `src/tests/hidden.rs`: `the_stream_goes_a_minute_after_the_tab_hides`,
  `an_opening_while_hidden_waits_for_shown`; view:
  `a_note_seen_twice_shows_once`; story
  `a_note_added_in_one_page_shows_in_another` (two people). Step tests
  under 20 ms; the story about 3 s.
Proof: live, `heavy --gate scripts/check.sh` once: the two-person
  story passes. Bound: 15 minutes.
Don't: hold two streams at once.
Size: about 300 lines.
Removes: nothing.

## Interface sketch

```rust
// crates/skein-ui-web/src/transport.rs: a step's requests, whole (ui-web.md,
// 10). The shell builds one per step from its state machine's duties.
/// A small-store record (store.rs, 12.1).
pub struct Record { pub key: Box<[u8]>, pub value: Box<[u8]> }
pub struct Requests {
    pub writes: Vec<Record>,
    pub removes: Vec<Box<[u8]>>,
    pub exchanges: Vec<fetch::Exchange>,
    pub stream: Option<StreamDuty>,
    pub navigate: Option<Box<[u8]>>,
}
pub enum StreamDuty { Open { path: Box<[u8]>, bound: u32 }, Close }
/// What the transport reports back into the turn, in order.
pub enum Carried {
    Written { key: Box<[u8]> },
    Refused { key: Box<[u8]> },
    Failed { exchange: u32 },
}
pub fn carry<P: Platform>(requests: Requests, store: &mut Store, fetch: &mut Fetch, platform: &mut P, reports: &mut Vec<Carried>);
/// skein_ui_web::platform::Platform: what the store and the transport
/// go through, the browser's on wasm32, the fake one in the host tests.
pub trait Platform {
    fn read_all(&self) -> Vec<Record>;
    fn write(&mut self, key: &[u8], value: &[u8]) -> bool;
    fn remove(&mut self, key: &[u8]);
    fn send(&mut self, exchange: &fetch::Exchange);
    fn navigate(&mut self, to: &[u8]);
}
```

## Questions to raise when reached

None known.

## Done when

`heavy cargo nextest run -p skein-ui-web -p skein-ui-example-view -p
skein-ui-example-server` passes, and `heavy --gate scripts/check.sh` passes once with
`a_reload_keeps_a_pending_add_made_once` and
`a_note_added_in_one_page_shows_in_another` in its browser suite.
