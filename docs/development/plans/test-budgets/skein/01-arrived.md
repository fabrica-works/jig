# Session 01: a read of what has arrived

lib's stream vocabulary gains `Read::Arrived { max }`: met as soon as
the side below holds a byte, by all it holds then, up to `max`. The
sides below in skein meet it (lib's intake and `Held`, io's sockets,
joined ends and pipes, TLS's two faces, the HTTP body and an event's
data), the worlds read with it, and skein's own readers of bytes nothing
frames move onto it. smith 01 and jig 05 then move theirs. Today such a
reader demands `Fill(1)` and pays one iteration of its loop per byte:
smith's tool that prints a megabyte costs about a million iterations
(jig's `jig-protocol/smith-settle.md`, 6). Three increments: the read,
met everywhere; the worlds; the readers.

## Read first

- skein's `lib.md`, section 7: the contract of a stream, "A read of what
  has arrived", and `Intake`.
- skein's `io.md`, section 2 ("The startup check") and 3.3 (the contract
  io keeps; "Delivered").
- skein's `http.md`, 3.3 (the body reads below "in the demand's own
  shape") and 4 (`sse::Data`); `tls.md`, 3.1 and 3.2.
- skein's `testing-strategy.md`, 2.4 (the side below a machine world
  plays).
- `findings/reads.md`: the sides below, the exhaustive matches on
  `Read`, the readers of unframed bytes, with `file:line`.

## What the design asks

- **The read** (lib.md, 7): "A read of what has arrived (`Arrived`) is
  met as soon as the side below holds a byte, by all it holds then, up to
  the read's maximum: at least one byte, at most the maximum." "A maximum
  past what the side below can hold is met with what it holds: the side
  below never waits to fill it." A maximum of zero is the caller's bug,
  asserted.
- **What a delivery holds is how the bytes arrived** (lib.md, 7): "A side
  above that reads what has arrived makes the same of the bytes whatever
  the cuts; only their order is the stream's."
- **Stacked sides below** (lib.md, 7): "reads below no more than it
  needs: a read of what has arrived, where its framing allows one (an
  HTTP body within what is left of its chunk or its length, http.md,
  3.3), or by its framing (TLS, a record at a time, tls.md, 3.2)."
- **io** (io.md, 3.3): "met by the first delivered that finds a byte in
  the intake: in the up pass after the down pass that stated it, if the
  intake holds one, or else in the pass its next receive lands ... Pipes
  meet it as sockets do." The startup check holds exact reads only: "A
  read of what has arrived is met whatever its maximum."
- **Exact reads stay** (lib.md, 7): "where the bytes are framed, or where
  the side above must not take more than it can act on in one step".

## Choices this plan makes

- **The name is `Read::Arrived { max: u32 }`,** beside `Fill`, `Scan` and
  `Line` in `crates/skein-lib/src/stream.rs`.
- **`Intake::meet(Read::Arrived { max })`** asserts `max > 0`, returns
  `None` when the intake is empty, and otherwise `Some(self.take(n))`
  with `n = self.len().min(max)`. No assertion against the capacity.
- **`Held::answer`** meets it as the intake would: all that is left up
  to `max`, or `End` when nothing is left.
- **io's End rule** (`stream.rs:999-1003`): an `Arrived` outstanding at
  the end is a read that can never be met, as a `Fill` is, since the
  intake would have met it if it held a byte.
- **TLS's `largest(read)`** (`client/above.rs:166-171`), which its
  callers assert within `Limits::read`, is 1 for `Arrived`: one byte
  meets it, whatever `max`.
- **The HTTP body** (`body.rs:172-200`, `piece`) passes `Arrived { max }`
  down as `Arrived { max: at_most(max, left) }`, never past the body's
  framing; `answers` (`body.rs:282-295`) takes a delivery of 1 to `max`
  bytes for it.
- **The tokenizer drivers** that answer a tokenizer's reads from a slice
  (`skein-json/src/tokenizer.rs:144-155`, `skein-oauth/src/json.rs:46-56`,
  `skein-llm/src/openai/json.rs:67-77`) put `Arrived` with `Nothing` and
  `Line`: a tokenizer never demands it (json.md, 3.2).
- **Readers that stay exact:** the JSON tokenizer's `Fill(1)`
  (`tokenizer.rs:587`); the body's `Fill(1)` while the intake ends partway
  through a delimiter (`body.rs:184-185`); `skein-channel`'s frame header
  (`machine.rs:493-495`); `skein-llm`'s first byte of an event's data,
  which it asserts is one byte (`client.rs:966`), and its six-byte
  `[DONE]` sentinel, read by the same demand (`client.rs:632`) and at
  `:985`; the terminal fixture's
  keys (`tests/world/src/bin/terminal_fixture.rs:47,173`).

## Rules that bind it

- No increment "adds a `_ =>` arm to a match on `Read`, or an
  `unreachable!` arm for `Read::Arrived` in a side below"
  (code-organization.md, section 4): a `_ =>` arm would let the next
  variant pass unseen.
- "**smith's reliability plan lands in smith and skein from
  debian-16gb-hel1-1** ... `land` rebases each landing on its landings"
  (README, "Rules"): it edits skein's `crates/skein-io/` too.
- Files already past 800 lines grow by at most this session's Sizes and
  are not split: `crates/skein-io/src/stream.rs`,
  `crates/skein-http/src/body.rs`, `crates/skein-http/src/client.rs`,
  `crates/skein-http/src/server.rs`, `crates/skein-llm/src/client.rs`.

## Increments

### 01.1 `Read::Arrived`, met by the sides below Files names
Scope: lib.md, 7 (the read, the contract, `Intake`); io.md, 2 (the
  startup check) and 3.3; http.md, 3.3 and 4 (`sse::Data`); tls.md, 3.1.
Needs: nothing.
Files:
  - skein-lib: `src/stream.rs` (`Read`, its docs; `Up::Bytes`'s doc
    "Exactly what the demand's read asks for"), `src/intake.rs`
    (`meet` `:128-145` and its docs `:110-127`), `src/stream/held.rs`
    (`answer` `:35-72`), `src/tests/streams.rs`, `src/tests/held.rs`.
  - skein-io: `src/stream.rs` (`delivered`'s End rule `:999-1003`;
    `demand`'s asserts `:1017-1037` and "outstanding" `:1038-1041`),
    `src/joined/end.rs` (asserts `:77-85`), `src/pipe.rs` (`:131-147`: a
    read's maximum asserted as `stream.rs` does), `src/tests/stream.rs`,
    `src/tests/process.rs`, `src/tests/joined.rs`.
  - skein-tls: `src/client/above.rs` (`largest` `:166-171`),
    `src/client/close.rs` (`:97-100`), `src/client/native/tests.rs`
    (`:499-502`).
  - skein-http: `src/body.rs` (`piece` `:172-200`, `answers`
    `:282-295`), `src/client.rs` (`:844-848`), `src/server.rs`
    (`:1032-1036`), `src/sse.rs` (`data_demand` `:375-384`),
    `src/tests/body.rs`, `src/tests/data.rs`.
  - The tokenizer drivers: `skein-json/src/tokenizer.rs:144-155`,
    `skein-oauth/src/json.rs:46-56`, `skein-llm/src/openai/json.rs:67-77`.
  - The tests' exhaustive matches, each given the arm the side below's
    contract gives it (`findings/reads.md`, section 5, "Tests"):
    `tests/lib/src/intake.rs:19-43`, `tests/io/src/ledger.rs:241-262`,
    `tests/http/src/client_world.rs:443-458,1010-1014,1063-1080`,
    `tests/http/src/server_world.rs:613-627,630-646,1437-1441`,
    `tests/http/src/sse_world.rs:550-560`,
    `tests/json/src/world.rs:571-584,600-606`,
    `tests/tls/src/world.rs:237-251,254-270,680-687`,
    `tests/tls/src/drive.rs:72-76`, `tests/protocol/src/wire.rs:125-129`.
Shape:
  ```rust
  // crates/skein-lib/src/stream.rs
  pub enum Read {
      Nothing,
      Fill(u32),
      Scan { until: Delimiter, max: u32 },
      Line { max: u32 },
      /// What has arrived: everything the side below holds once it holds a
      /// byte, at most `max`. `max` is at least one (asserted below); a
      /// larger one than the side below can hold is met with what it holds.
      Arrived { max: u32 },
  }
  ```
  `Intake::meet` and `Held::answer` as "Choices" says; io's `demand`
  asserts `max > 0` for `Arrived` and nothing against the cap; its End
  rule and "outstanding" match treat `Arrived` as `Fill`; `pipe.rs`
  asserts the same as `stream.rs` before it stores a demand; TLS's
  `largest` gives 1; `close.rs` drops an answer to it as it drops a
  fill's; the HTTP body passes it down within `left`, and `answers`
  holds a delivery for it to 1 to `max` bytes; `sse.rs`'s `data_demand`
  passes it through as it passes a fill. Where a side below asserts a
  read within a cap (`sse.rs:383`, `wanted <= env.limits.chunk`;
  `client.rs:844-848` and `server.rs:1032-1036`, `wanted <=
  limits.read`), it asserts only `max > 0` for `Arrived`. The test
  drivers among Files that play the side below a machine which never
  demands `Arrived` (`crates/skein-tls/src/client/native/tests.rs:499-502`,
  `tests/tls/src/world.rs:680-687`, `tests/tls/src/drive.rs:72-76`,
  `tests/http/src/sse_world.rs:550-560`) put it in the arm that already
  refuses the reads that machine never makes.
Test shape: the tests below, each in its crate's tests:
  - `src/tests/streams.rs`: `a_read_of_what_has_arrived_takes_all_held_up_to_its_maximum`
    (reference below), `a_read_of_what_has_arrived_waits_for_one_byte`,
    `a_read_of_what_has_arrived_of_zero_is_the_caller_s_bug`
    (`#[should_panic]`), and `a_scan_after_a_read_of_what_has_arrived_still_finds_its_delimiter`
    (the search memo, `take` `:213-224`). `reference` (`:239-262`) gains
    the read: what it meets is `min(held, max)` when `held > 0`; the
    split tests (`:336`, `:353`, `:360`) keep exact reads only, and a new
    `every_split_delivers_the_same_bytes_in_order_to_reads_of_what_has_arrived`
    checks, for the same inputs and splits, that the concatenated
    deliveries equal the input and each holds 1 to `max` bytes.
  - `src/tests/held.rs`: `held_meets_a_read_of_what_has_arrived_with_what_is_left`.
  - skein-io `src/tests/stream.rs`:
    `a_read_of_what_has_arrived_is_met_by_the_receive_that_lands`, after
    `bytes_wait_in_the_intake_until_a_fill_demands_them` (`:104`), with
    its `ask`, `give` and `push` helpers (`:17-37`);
    `src/tests/process.rs`: `a_childs_output_is_read_as_it_arrives`,
    after `a_childs_output_is_a_stream_and_the_child_closes_after_its_pipe`
    (`:90`); `src/tests/joined.rs`:
    `a_joined_end_meets_a_read_of_what_has_arrived_with_what_its_peer_moved`.
  - skein-http `src/tests/body.rs`:
    `a_read_of_what_has_arrived_never_reads_past_the_body` (a body of a
    known length followed by the next response's bytes, and a chunked
    body: no delivery holds a byte past the body or a chunk's framing).
  - skein-tls `src/tests/machine.rs`:
    `a_read_of_what_has_arrived_takes_one_records_plaintext`.
  Reference test, in `crates/skein-lib/src/tests/streams.rs`:
  ```rust
  #[test]
  fn a_read_of_what_has_arrived_takes_all_held_up_to_its_maximum() {
      let mut intake = Intake::with_capacity(16);
      intake.append(b"hello").expect("within the cap");
      assert_eq!(intake.meet(Read::Arrived { max: 3 }), Some(boxed(b"hel")));
      assert_eq!(intake.meet(Read::Arrived { max: 64 }), Some(boxed(b"lo")));
      assert_eq!(intake.meet(Read::Arrived { max: 64 }), None);
  }
  ```
  Ceilings: each unit test under 20 ms.
Proof: unit, the tests above by name, `heavy cargo nextest run -p
  skein-lib -p skein-io -p skein-http -p skein-tls -E 'test(/arriv/)'`,
  once: each passes; and `heavy cargo clippy --workspace --all-targets
  -- -D warnings` exits 0. The world layer is 01.2's.
Don't: assert a read of what has arrived against a cap; add a `_ =>`
  arm; change what an exact read delivers; make a tokenizer, the
  channel's header or a reader in "Choices" read what has arrived.
Size: at most 120 lines of production code.
Removes: nothing.

### 01.2 The worlds read what has arrived
Scope: testing-strategy.md, 2.4 (the side below "delivers what was
  demanded (exactly, or what has arrived of it), from peer bytes cut at
  random"); http.md, 6 (the client's and the server's worlds read "with
  fills, line scans, scans ..., and reads of what has arrived, of every
  size"); a story on the real ring of a child's megabyte through a pipe.
Needs: 01.1.
Files: `tests/io/src/owner.rs` (`Reads` `:600-615`) and
  `tests/io/src/ledger.rs` (`:240-262`); `tests/http/src/client_world.rs`
  (`Reads` `:85`, `:823-835`; the delivery check `:655`),
  `tests/http/src/server_world.rs` (`:108`, `:1220`; `:945`);
  `tests/tls/src/world.rs` (`:568`; `:434`); `tests/lib/src/intake.rs`
  (the fuzzy reference `:18-43`, `check` `:52-95`) and
  `tests/lib/tests/fuzzy_intake.rs`; for the story,
  `tests/io/src/processes.rs` (`Story` `:17-24`, the child's arguments
  `:74-84`, `observe` `:112-140`, `check` `:181-`),
  `tests/conformance/ring/src/bin/process_fixture.rs` (a mode
  `megabyte`, beside `echo` and `usage`) and `tests/io/tests/ring.rs`
  (beside `group_story` `:115-126`).
Shape: each world's choice of read shapes gains `Arrived` with a drawn
  maximum (1 to the intake's cap, and in one draw of 16 past it); each
  referee that holds a delivery to "exactly the demand" holds a delivery
  for `Arrived` to 1 to `max` bytes, the bytes the reference holds next,
  in order, and never more than the side below held then. The fuzzy
  intake harness draws it among its demands, its reference meeting it
  as 01.1's does.
Test shape: no new world test beyond the one below; the worlds' existing
  sweeps draw the new read. One story on the real ring, in
  `tests/io/tests/ring.rs`, on `skein_io_world::processes` as
  `group_story` (`:115-126`) uses it:
  `a_megabyte_on_a_real_pipe_arrives_in_few_deliveries`. `Story` gains
  `Megabyte`: the fixture, run as `process_fixture megabyte`, writes
  1,048,576 bytes of a repeating pattern to its standard output and
  exits; `Process` reads its standard output with `Arrived { max: 65_536
  }` in every story (its standard error keeps its line scans), counts
  the deliveries and the bytes, and `check` asserts, for `Megabyte`,
  1,048,576 bytes in the fixture's pattern and at most 4,096 deliveries
  (one per 256 bytes or more; `Fill(1)` would take 1,048,576).
Proof: process, `heavy cargo nextest run -p skein-io-world -E
  'test(a_megabyte_on_a_real_pipe_arrives_in_few_deliveries)'` once on
  the real ring: it passes, the assertion on the deliveries being the
  evidence; no count is reported. Then `heavy cargo nextest run -p
  skein-io-world -p skein-http-world -p skein-tls-world -p
  skein-lib-tests` once, every test passing.
Don't: draw `Arrived` in a world whose machine must read exactly (the
  JSON tokenizer's); weaken an exact read's check; add a seed to a
  pinned list for this.
Size: no production code; at most about 250 lines of test code.
Removes: nothing.

### 01.3 skein's readers of unframed bytes read what has arrived
Scope: lib.md, 7 ("It is the read of bytes nothing frames: a child's
  output, a body read to its end").
Needs: 01.1.
Files: `crates/skein-llm/src/client.rs` (the error body, `:852` and
  `:873`; its cut at `error_bytes`, `:434`);
  `crates/skein-oauth-accounts/src/exchange.rs` (the token response's
  body, `:340` and `:370`); `testing/skein-fake-peers/src/oauth.rs`
  (`:250`, `:339`); `testing/skein-fake-llm-protocol/src/provider.rs`
  (`body_read` `:606-616`: an unknown length's `Fill(1)`, whose comment
  "Fill must never wait for bytes beyond the body end" goes);
  `tests/tls/tests/stack.rs:221` (the `Fill(1)` of a chunked or
  until-the-end body);
  `tests/oauth-accounts/src/world.rs:1134,1138`,
  `tests/oauth-accounts/src/sign_in/protocol.rs:291,295`,
  `tests/oauth-accounts/src/sign_in/browser.rs:76,94`.
Shape: each `Fill(1)` above becomes `Read::Arrived { max }`, with `max`
  the room its reader has left plus one where a byte past the room is
  how it tells a cut (the LLM error body, `client.error.room() + 1`; the
  token response, `document.room() + 1`), and elsewhere the room, or 1
  where the room is 0. Each reader already loops over the box it is
  given; it keeps its cut and limit checks as they are. A delivery's
  memory (lib.md, 7): the LLM client's and the token exchange's worst
  cases (`crates/skein-llm/src/client.rs`'s and
  `crates/skein-oauth-accounts/src/`'s `worst_case` functions) gain one
  delivery of their reader's `max` each, and their memory tests
  (`tests/llm/tests/memory.rs`, `tests/oauth-accounts/tests/`) keep
  passing. A test that asserts one of these worst cases as a literal
  moves by exactly that delivery, written as a new literal, and the
  commit body names each with the delivery's size (ruling 106):
  `tests/llm/tests/replay_total.rs`'s
  `standard_client_bound_excludes_per_item_replay_capacity`, whose
  `fixed` and `removed` move by 16,385 (`error_bytes` + 1) for each
  provider, and any such test under `tests/oauth-accounts/`, by the
  token response's `max`. A test that expects a `Fill(1)` that this
  Shape makes `Read::Arrived { max }` expects the new read
  (`testing/skein-fake-llm-protocol/src/provider.rs`'s
  `length_framed_body_demands_bounded_batches_and_probes_actual_end`,
  its unknown-length row; ruling 107). No other expected value changes.
Test shape: in each crate's tests, one test per reader that a response
  delivered in one piece and the same delivered a byte at a time give
  the same result: `an_error_body_read_as_it_arrives_is_the_same_whatever_its_cuts`
  (skein-llm), `a_token_response_read_as_it_arrives_is_the_same_whatever_its_cuts`
  (skein-oauth-accounts); the error body's cut at `error_bytes` still
  reported when one delivery crosses it.
Proof: world, `heavy cargo nextest run -p skein-llm -p
  skein-llm-world -p skein-oauth-accounts -p skein-oauth-accounts-world
  -p skein-tls-world -p skein-oauth-world` once, every test passing
  (the fake peers' users among them); then skein's gate through `land`.
Don't: change a reader's limit or its failure; make a reader in 01.1's
  "Choices" list read what has arrived.
Size: at most 60 lines of production code.
Removes: the `Fill(1)` loops named in Files.

## Interface sketch

```rust
// crates/skein-lib/src/intake.rs
impl Intake {
    /// Meets `read` from what is held: a fill or a scan exactly, or, for
    /// `Arrived { max }`, all that is held up to `max` once a byte is.
    pub fn meet(&mut self, read: Read) -> Option<Box<[u8]>>;
}
```

The intake owns the cut: no side below computes `min(held, max)`
itself, so the delivery's size has one definition. io, TLS and the HTTP
body call `meet`; `Held` keeps its own copy of the rule, as it keeps
one for fills and scans.

## Questions to raise when reached

- A side below whose cut of a delivery is not its intake's (a test
  kit's own buffer, a fake peer writing straight up): it meets `Arrived`
  with what it holds, up to `max`, as `Held` does; a kit that cannot
  says so in its report.

## Done when

- `git -C ~/src/rust/skein grep -n 'Arrived {' -- crates/skein-lib/src/stream.rs`
  prints the variant.
- `heavy cargo nextest run -p skein-io-world -E 'test(a_megabyte_on_a_real_pipe_arrives_in_few_deliveries)'`
  passes in skein's `main`.
- `git -C ~/src/rust/skein grep -n 'Read::Fill(1)' -- crates/skein-oauth-accounts/src/exchange.rs testing/skein-fake-peers/src/oauth.rs`
  prints nothing, and `git -C ~/src/rust/skein grep -c 'Read::Fill(1)'
  -- crates/skein-llm/src/client.rs` prints `crates/skein-llm/src/client.rs:2`
  (an event's first byte and the sentinel, which stay exact).
