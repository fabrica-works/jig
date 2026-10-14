# Session 05: jig takes skein's and smith's changes, and reads what has arrived

jig takes smith's and skein's `main` once smith has taken skein's read
of what has arrived (smith 01.1) and skein's codec generator checks a
family's limits once (skein 02.2). It regenerates every family, which
makes every store open cheaper (a family's worst case was checked again
at every node of its type tree: 3,092 checks in jig's generated code),
and moves jig's three readers of bytes nothing frames onto the new read:
the client link's event data, the native shell's bodies of unknown
length, the engine's chunked request bodies. Two increments, the first
with the repin.

## Read first

- skein's `lib.md`, section 7 ("A read of what has arrived"); `http.md`,
  3.3 (a body read within its framing).
- skein's `codec.md`, 4 and 6; `findings/cost-jig-units.md`, 2.
- `findings/reads.md`, section 4 (rows 12 to 14, and jig's test drivers).
- development.md, 5.7 (a repin's steps and what it may change).

## What the design asks

- "It is the read of bytes nothing frames: a child's output, a body read
  to its end" (skein's lib.md, 7); "A side above that reads what has
  arrived makes the same of the bytes whatever the cuts".
- Generated worst cases: one check of the family's limits per public
  call (skein 02.2; codec.md, 4: "a call costs what its value holds, not
  the family's size").

## Choices this plan makes

- **One repin, smith and skein together** (development.md, 5.7: "jig
  moves smith and skein together: it resolves one revision of each, for
  itself and for smith's crates"): after smith 01.1, jig's lock takes
  smith's `main` and the skein revision smith's `Cargo.lock` then holds,
  on which smith's crates were last built (as jig-protocol's ruling 281
  did downstream).
- **Every family regenerates** as jig-protocol's jig-repin.s3 and s4 did:
  `SKEIN_CODEGEN_WRITE=1` with the drift tests that take it
  (`tests/store-codecs/tests/drift.rs`, `tests/client-wire/tests/drift.rs`,
  `tests/client-wire/tests/test_client_wire.rs`), and for
  `crates/jig-protocol-worker-wire` skein's generator from skein's
  checkout at the same commit: `heavy cargo run -q --manifest-path
  ~/src/rust/skein/Cargo.toml -p skein-codegen --
  <jig>/crates/jig-protocol-worker-wire/schema/worker-wire-v2.schema
  <jig>/crates/jig-protocol-worker-wire/src/generated`; the golden
  `.bin` files do not change.
- **The readers' maxima:** the client link's event data reads `Arrived
  { max }` with the event's `chunk` (`stream.rs:118-121`'s
  `sse::Limits`); the native shell, `env.limits.read`; the engine's
  clients, `limits.http.read`. Each already bounds what it keeps.
- **A delivery's memory** (lib.md, 7: "The side above's memory counts a
  delivery of its maximum"): where the crate's worst case is computed in
  code, it gains one delivery of the reader's maximum, and the memory
  test that covers the crate keeps passing; a bound jig's design derives
  that lacks the term is a question.

## Rules that bind it

- No increment "adds a `_ =>` arm to a match on `Read`, or an
  `unreachable!` arm for `Read::Arrived` in a production side below"
  (code-organization.md, section 4); `clear_demands`' existing
  `Down::Demand { .. }` arm (`stream.rs:107-116`) matches `Down`, not
  `Read`, and stays.
- development.md, 5.7: "At most about 200 lines besides `Cargo.lock`, and
  no change of design"; a break that needs more is a question.
- "**Edits generated code by hand**" is on the plan's "Don't" list.

## Increments

### 05.1 jig takes smith's and skein's mains and regenerates its families
Scope: skein 02.2 in jig's families; skein 01 and smith 01.1 in jig's
  graph.
Needs: smith 01.1 and skein 02.2 landed.
Files: `Cargo.lock`: `heavy cargo update -p smith-charter` (one package
  moves its whole git source, development.md, 5.7, step 1), then `heavy
  cargo update -p skein-lib --precise <the skein commit smith's
  Cargo.lock then holds>` (the `#` part of the `source` line under
  `name = "skein-lib"` in `~/src/rust/smith/Cargo.lock`), so that smith's crates build in jig's graph on
  the skein they were built on; the generated families,
  `crates/*/src/generated/*.rs` and `testing/*/src/generated/*.rs`; what
  `heavy cargo check --workspace --all-targets` then names, within 5.7's
  limits: among it, smith's transcript records gained `model` and
  `effort` since jig's pin (smith `dfc0fe57` to `main`), which jig builds
  by literal in two tests, `crates/jig-protocol-transcript/src/tests.rs:42-50`
  and `tests/ops/src/conformance/scripts.rs:494-499`; and the copy of smith's
  full-turn golden that jig keeps, `crates/jig-protocol-transcript/tests/data/record_turn_full.bin`,
  127 bytes, which smith `47e99f6e` grew to 142
  (`crates/smith-transcript/golden/v3/record_turn_full.bin`): it is
  copied again from smith's revision in jig's lock, byte for byte, and
  the three sizes that name it (`crates/jig-protocol-transcript/src/tests.rs:446`,
  `tests/client/src/joint/wire/turns.rs:24`, `activity.rs:378`) move
  from 127 to 142 (ruling 114).
Shape: no hand edit of generated code; the drift tests pass with
  `SKEIN_CODEGEN_WRITE` unset.
Test shape: none new; the drift tests, unchanged, and the golden files,
  unchanged (`git diff --stat origin/main -- '*/golden/*'` prints
  nothing).
Proof: unit and world, `heavy --gate cargo nextest run -p
  jig-protocol-store -p jig-client-protocol-world -p jig-view --profile
  measure -j 1` once: summed times in the report (at this plan's base,
  alone: 2.72 s, 1.76 s and 4.69 s; the audit: client-protocol −0.51 s,
  `jig-view` −0.40 s, from the generated worst cases); then jig's gate
  through `land`.
Don't: move a package of another source; edit a generated file, a
  schema or a golden by hand.
Size: `Cargo.lock` and generated code; at most 200 other lines
  (development.md, 5.7). Whether jig's own transcript carries the
  record's new `model` and `effort` is a question, not this increment's:
  the two tests take the values smith's own tests use.
Removes: nothing.

### 05.2 jig's readers of unframed bytes read what has arrived
Scope: skein's lib.md, 7 for `findings/reads.md`, 4, rows 12 to 14.
Needs: 05.1.
Files: `crates/jig-protocol-client-link/src/stream.rs` (`data` `:101-103`;
  `clear_demands` `:107-116`, whose `Down::Demand { .. }` arm stays a
  read of lines only); `crates/jig-shell-native/src/connections.rs`
  (`read` `:349-362`: a body of unknown length);
  `crates/jig-protocol-engine-clients/src/connection.rs` (`read`
  `:64-74`: a chunked or absent body); `tests/engine-clients/src/peer.rs:146`
  (the test peer's own body read); each crate's tests.
Shape: each `Read::Fill(1)` named becomes `Read::Arrived { max }` with
  the maximum "Choices" names; a known length stays `Fill(remaining)`;
  the base64 decoder, the body's intake and the peer's buffer take a
  delivery of any size as they take one byte today.
Test shape: one test per reader, each giving the same input whole and a
  byte at a time and expecting the same result:
  `an_events_data_read_as_it_arrives_decodes_the_same_whatever_its_cuts`
  (client link), `a_body_of_unknown_length_read_as_it_arrives_is_the_same_whatever_its_cuts`
  (native shell), `a_chunked_request_body_read_as_it_arrives_is_the_same_whatever_its_cuts`
  (engine clients).
Proof: world, `heavy --gate cargo nextest run -p
  jig-client-protocol-world -p jig-engine-clients-world -p
  jig-shell-native -p jig-protocol-client-link -p
  jig-protocol-engine-clients --profile measure` once: every test
  passing, the summed time in the report.
Don't: read a line, a header or a length-framed body with `Arrived`;
  change a reader's bound or its failure.
Size: at most 30 lines of production code.
Removes: the three readers' `Fill(1)`.

## Done when

- `git -C ~/src/rust/jig grep -n 'Read::Arrived' -- crates/jig-protocol-client-link/src/stream.rs crates/jig-shell-native/src/connections.rs crates/jig-protocol-engine-clients/src/connection.rs`
  prints a line in each of the three files.
- `git -C ~/src/rust/jig diff --stat d91a89c45 origin/main -- '*/golden/*'`
  prints nothing.
