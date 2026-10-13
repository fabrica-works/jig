# Session 32: `jig-protocol-client-link`, the client's side of the client's protocol

A component of the client's protocol layer, step code, built for the
host only. It makes one exchange per attempt and says how it ended: for
a keyed request, the request table's envelope with the answer decoded
into its owner's vocabulary; for a read, its one result. It keeps the
stream: skein's server-sent events reader inside it cuts the stream's
body into events, whose records it decodes; the hello's ranges go to
their owners; watches are opened by exchange, fed, kept alive, and
dropped together when the stream goes silent. It encodes the small
store's records per owner. Its shell (jig 33's `jig-shell-native`) gives
it each answer's body at once and the stream's body as it comes, and adds the bearer
token; the link never sees a secret. Its translation is the client
contract's types to and from `jig-protocol-client-wire`'s records, which
mirror them field for field.

## Read first

- jig's `protocol/client.md`, sections 1, 2, 4 to 7, 9, 10 and 13 (not
  section 12's browser).
- jig's `client/domain/client.md`, sections 8, 10, 11, 12 and 16;
  `client/domain/root.md`, sections 4, 5 and 8; `client/domain/contract.md`,
  sections 4, 6, 7 and 8.
- skein's `http.md`, section 4 (the server-sent events reader); skein's
  `lib.md`, sections 6.1 (`base64`) and 12 (the request table).
- `decisions.md`, 2; `code-organization.md`, sections 2, 3, 5 and 6.
- findings/protocol.md, items 1.2 (`jig-protocol-transcript`), 1.3 (the
  client's boundary and the request table) and 1.7 (the joint world's
  translation).
- The code: `crates/jig-domain-client/src/{boundary.rs, store.rs,
  versions.rs, contract/*.rs}`; `testing/jig-test-domain-client/src/{boundary.rs,
  store.rs}` (the root's `Send`, `Save`, `Answer`, `Record::Table`);
  skein-lib's `requests.rs` (`RequestOut`, `RequestEnvelope`,
  `RequestRecord`, `RequestKey`); `crates/jig-protocol-transcript/src/lib.rs`
  (`decode`, `Undecodable`); skein's `crates/skein-http/src/sse.rs`.

## What the design asks

- **One attempt per exchange** (`client.md`, section 5): done and refused
  are *final*, "with the record decoded into its owner's vocabulary";
  busy and not ready *again*; signed out *signed out*; "an exchange that
  failed, timed out or ended without a whole answer is *lost*. A final
  answer that does not decode is *lost* too, and reported". "The
  attempt's deadline is the client's, from its limits".
- **Reads** (section 6): one exchange each, answered once.
- **The stream and watches** (sections 4 and 7): the hello first; each
  watch told it is alive "at each heartbeat and each event; past twice
  the span with nothing read, it takes the stream as dropped"; a dropped
  stream drops each watch on it, each told so.
- **The shell hands bodies; the reader is inside** (section 2): the link
  "gives its shell a method, a path, headers and a body, and takes back
  whole bodies, and the stream's body as it comes, which skein's
  server-sent events reader, inside the component, cuts into events".
- **"Documents are decoded whole,** into the client's types, and checked
  against the client's limits" (section 3); turns decoded by `jig-protocol-transcript`, "a turn that
  does not decode crosses up as such, never dropped".
- **Versions** (section 9): a client outside a range sends nothing more of
  that owner, shows it is out of date, and keeps what is pending.
- **The small store** (section 10): records per owner, "wrapped in a
  record naming its owner"; written before the step's exchanges leave; at
  start, "one of a version its owner does not read is dropped, and its
  owner told".

## Choices this plan makes

- **Host only:** no wasm32 target, no browser kit (`decisions.md`, 2).
- **Identifiers of its own:** a fresh exchange token per attempt and per
  read, and a fresh wire name (`u64`) per watch opening, so a late answer
  of an earlier opening never meets a later one.
- **The hello's ranges go to their owners:** jig's range as
  `jig_domain_client::Event::Hello { versions }`, whose judgement stays in
  `jig-domain-client`'s `versions::hello` (`versions.rs:13`), which
  closes the watches, shows out of date and parks the table; a family's
  range handed up by number to the routing. The link sends no exchange of
  an owner its domain has blocked, since the domain and the table stop
  asking.
- **A delivery for a watch whose opening is in flight** admits it: the
  stream may outrun the exchange's answer.
- **The stream's events** are base64 (`Alphabet::Standard`) in each
  event's data, decoded with `skein_lib::base64::Decoder` in pieces, then
  `StreamEvent` decoded.
- **Wire limits from the client's limits:** the link's `v1::Limits` are
  set from `jig_domain_client::Limits` (`text`, `rows`, and the lists'
  bounds) at construction, so a document past them is refused at
  decoding; a record's measured size past `Limits::document_bytes` is
  refused too.
- **An application's records cross as bytes** with their family and
  version, to and from the client's routing, which decodes them with the
  application's wire (`client.md`, section 11).
- **The small store's writes lead each step's exchanges** in the link's
  output; a shell that reports a write failed has the step's exchanges
  withheld, each attempt answered `Lost`.
- **Routes** are jig 31's: `/jig/v1/request`, `/jig/v1/read`,
  `/jig/v1/watch`, `/jig/v1/unwatch`, `/jig/v1/stream-gone`, `GET
  /jig/v1/stream`. Sign-in and sign-out are the shell's exchanges (jig
  33), not the link's.

## Rules that bind it

- "**The client's crates** (`jig-domain-client`, `jig-view`,
  `jig-protocol-client-wire`, `jig-protocol-client-link`,
  `jig-shell-native`) depend on no engine-side crate."
  (`code-organization.md`, section 3)
- "No crate of this plan links a browser or wasm32 toolchain."
  (`code-organization.md`, section 2) No `wasm32` in any manifest, cfg or
  script of this session.
- "**Step code** follows skein's `programming-model.md`, section 10, and
  jig's `AGENTS.md`: no traits, no generic items, no `dyn`, no
  closures." (`code-organization.md`, section 2)

## Increments

### 32.1 The component, attempts and envelopes

Scope: the crate `crates/jig-protocol-client-link`: `boundary.rs` (the
interface sketch), `component.rs` (`Link::new`, `step`, `fire`,
`next_deadline`, `reclaim`, `max_out`, `worst_case`), `limits.rs`,
`translate.rs` with `translate/asks.rs` (`contract::Ask` to the wire's
`Ask`, exhaustive) and `translate/answers.rs` (the wire's `Answer`, `Done`
and `Refused` with `Outcome`, `Made`, `OwnerOutcome` and refusals' facts
to `contract::Answer`, exhaustive). A `Send` becomes `Exchange` to
`/jig/v1/request` with a `Keyed` body; its answer becomes `Answer {
attempt, envelope, answer }`: `final` decoded (a family's handed up as
bytes), `busy` and `not_ready` `Again`, `signed_out` `SignedOut`; a
failed exchange, a status other than 200, a body that does not decode as
`Answer`, or the deadline passing: `Lost`; a final answer that does not
decode: `Lost` and `Fact::UndecodableAnswer`. The crate's rules in
`tests/boundary`.
Needs: nothing beyond the session's start.
Files: the new crate; the workspace `Cargo.toml`;
`tests/boundary/src/tests.rs`. Shared hotspots: the workspace
`Cargo.toml`, `tests/boundary/src/tests.rs`.
Shape: as the interface sketch; `translate::asks::ask(ask:
&contract::Ask, limits: &v1::Limits) -> Result<v1::Ask, v1::Problem>`;
`translate::answers::answer(record: v1::Answer, limits: &Limits) ->
Result<contract::Answer, Undecodable>`.
Test shape: step tests in `src/tests.rs` and `src/tests/attempts.rs`,
fixtures in `src/tests/fixtures.rs`: `limits()`, `ATTEMPT`, `KEY`,
`start_chat()` (a `contract::Ask::StartChat`), `answer_done_chat(task)`,
`answer_busy()`, `answer_signed_out()`, and `Harness` (the link and its
output queue, answering exchanges with status and bytes). Tests:
`a_send_becomes_one_post_with_its_key`,
`done_is_handed_up_final_with_its_outcome`,
`busy_is_handed_up_as_again_with_no_answer`,
`signed_out_is_handed_up_as_signed_out`,
`a_failed_exchange_is_handed_up_lost`,
`an_attempt_past_its_deadline_is_abandoned_and_lost`,
`a_final_answer_that_does_not_decode_is_lost_and_reported`. Under 20 ms
each. Reference test, in full:

```rust
#[test]
fn busy_is_handed_up_as_again_with_no_answer() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    let exchange = harness.send(fixtures::ATTEMPT, fixtures::KEY, fixtures::start_chat());
    harness.answer(exchange, 200, &fixtures::answer_busy());
    assert_eq!(
        harness.answered(fixtures::ATTEMPT),
        Some((skein_lib::RequestEnvelope::Again, None))
    );
}
```

Proof: unit, `heavy cargo nextest run -p jig-protocol-client-link`, run
once: a send becomes one post with its key, and each answer (done, busy,
signed out, a failed exchange, an attempt past its deadline, an
undecodable final answer) is handed up as the domain's (the seven
tests); under 0.2 s. The joint layer is jig 33's
`tests/client-protocol`, which needs jig 31 and this session's last
increment.
Don't: retry: retrying is the table's (`client.md`, section 5); add a
header carrying a secret.
Size: at most 600 lines.
Removes: nothing.

### 32.2 Reads

Scope: `translate/reads.rs`: `contract::Read` to the wire's `Read`,
exhaustive, cursors as their bytes. A `Read` becomes `Exchange` to
`/jig/v1/read` with an `Owned` body; its answer `ReadAnswer` becomes
`Read { read, result }`: a value decoded by 35.1 and 35.2's
functions, or `ReadResult::None` with the wire's none; a failed
exchange, a status other than 200 or the deadline passing:
`ReadNone::Unreachable`; a value that does not decode: `Unreachable`
and `Fact::UndecodableDocument`. A family's read and answer as bytes.
Needs: jig 35's documents decoded (35.1, 35.2), beyond the session's
start.
Files: `src/{translate/reads.rs, component.rs, boundary.rs}`;
`src/tests/{reads.rs, fixtures.rs}`.
Shape: `translate::reads::read(read: &contract::Read, limits:
&v1::Limits) -> Result<v1::Read, v1::Problem>`.
Test shape: `a_read_is_answered_once_with_its_value`,
`a_read_signed_out_is_answered_signed_out`,
`a_read_with_no_answer_is_unreachable`,
`a_value_past_the_client_s_limits_is_unreachable_and_reported`. Under
20 ms each.
Proof: unit, `heavy cargo nextest run -p jig-protocol-client-link`, run
once: a read is answered once with its value, a signed-out one as signed
out, and one with no answer or past the client's limits as unreachable
(the four tests); under 0.2 s.
Don't: retry a read (the domain decides).
Size: at most 400 lines.
Removes: nothing.

### 32.3 The stream and watches

Scope: `stream.rs`, `watches.rs`, `translate/watches.rs`: the stream
opened (`OpenStream`) once a watch is first asked; its body's bytes fed
to a `skein_http::sse::Reader`; each event's data base64-decoded in
pieces and decoded as `StreamEvent`; the hello's ranges handed to their
owners (jig's as `jig_domain_client::Event::Hello`); an `Open` (and a
`Continue`, as an open of its run subject from the turn and offset it
names) becomes `Exchange` to `/jig/v1/watch` with the stream's name and
the watch's fresh wire name, answered `WatchEvent::Opened` or `Ended`
with the refusal's end; a delivery decoded (35.1, 35.2) and handed
up `WatchEvent::Delivered`; run text as `Delivery::Run`; an ending as
`Ended`; each watch `Alive` at each heartbeat comment and event; past
twice the hello's heartbeat with nothing read, the stream taken as
dropped: each watch `Ended { end: Dropped }`, a `/jig/v1/stream-gone`
exchange, the stream closed; a `Close` becomes `/jig/v1/unwatch`,
answered `Ended { end: Closed }` once its ending or answer comes.
Needs: jig 02's `skein_lib::base64`, beyond the session's start.
Files: `src/{stream.rs, watches.rs, translate/watches.rs, component.rs,
boundary.rs, limits.rs}`; `src/tests/{stream.rs, fixtures.rs}`.
Shape: `stream::Stream { reader: skein_http::sse::Reader, decoder:
skein_lib::base64::Decoder, name: Option<[u8; 16]>, last_read: Time,
heartbeat: Duration }`; `watches::Watches` over `Map<Token, Opening>`.
Test shape: `the_hello_s_jig_range_reaches_the_domain`,
`the_stream_s_body_cut_at_each_byte_reads_the_same_events`,
`a_delivery_before_its_opening_s_answer_admits_it`,
`silence_past_twice_the_heartbeat_drops_each_watch`,
`a_run_s_text_is_handed_up_with_its_offsets`,
`malformed_base64_drops_the_stream_and_is_reported`. Under 20 ms each;
the cut-at-each-byte test under 50 ms. Reference test of this kind, in
full:

```rust
#[test]
fn silence_past_twice_the_heartbeat_drops_each_watch() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    let watch = harness.open(fixtures::WATCH, fixtures::frame_subject());
    harness.stream_bytes(&fixtures::hello_event(fixtures::HEARTBEAT));
    harness.advance(fixtures::past_two_heartbeats());
    assert_eq!(harness.ended(watch), Some(WatchEnd::Dropped));
}
```

Proof: unit, `heavy cargo nextest run -p jig-protocol-client-link`, run
once: the stream's body cut at each byte reads the same events, silence
past twice the heartbeat drops each watch, and malformed base64 drops
the stream (the six tests); under 0.3 s for the file.
Don't: open a second stream while one is live.
Size: at most 600 lines.
Removes: nothing.

### 32.4 The small store's records

Scope: `store.rs`: each owner's records encoded with its wire and store
version and wrapped in jig 30's `Kept`: jig's (`Place`, `Draft`) by the
link, the table's (`TableEntry`, its request jig's encoded by the link
or a family's as the routing hands it), a family's as bytes; handed to
the shell as `Write` ahead of the step's exchanges; at `Start`, each
record the shell read decoded and handed to its owner (`Kept`), one of a
version its owner does not read dropped and its owner told (`Dropped`).
Needs: nothing beyond 32.1.
Files: `src/{store.rs, translate/store.rs, component.rs, boundary.rs}`;
`src/tests/{store.rs, fixtures.rs}`.
Shape: `store::key(record: &Saved) -> Box<[u8]>` (the keeper's tag and
the owner's key); `translate::store::{jig, table}` both ways.
Test shape: `a_draft_round_trips_through_the_small_store`,
`a_table_entry_restores_its_key_and_ask`,
`a_record_of_another_store_version_is_dropped_and_told`,
`a_step_s_writes_leave_before_its_exchanges`,
`a_failed_write_withholds_the_step_s_exchanges`. Under 20 ms each.
Proof: unit, `heavy cargo nextest run -p jig-protocol-client-link`, run
once: a draft and a request-table entry round-trip through the small
store, and a failed write withholds the step's exchanges (the five
tests); under 0.2 s.
Don't: write a secret to the small store.
Size: at most 400 lines.
Removes: nothing.

### 32.s1 The client's version follows its wire's

A step not in the plan (decisions.md, ruling 213), before the joint
client's proof. `crates/jig-domain-client/src/contract/versions.rs`'
`CLIENT_VERSION` is 1 while the client wire's `VERSION` is 4 (jig 30's
steps moved it); a hello serving 4..4 would put the current client out of
date. contract.md, 8: "Each owner's documents and requests are
versioned ... each range is one version".
Shape: `CLIENT_VERSION` becomes the wire's current version; fixtures that
  name the old number follow, behaviour unchanged. A test in
  `jig-protocol-client-link`, which links both, asserts
  `contract::CLIENT_VERSION == jig_protocol_client_wire::VERSION`, so a
  later move of one without the other fails.
Test shape: that one test; the existing versions tests, their numbers
  the current version.
Proof: the gate.
Don't: remap a range in the link; make the domain depend on the wire.
Size: at most 10 lines of production code.

## Interface sketches

```rust
// jig_protocol_client_link::boundary
use jig_domain_client::contract::{self, Answer as JigAnswer, ReadResult, Subject, VersionRange, WatchEnd, WatchEvent};
use jig_domain_client::store::{Record as JigRecord, StoreKey};
use skein_lib::{List, RequestEnvelope, RequestKey, RequestRecord, Token};

/// An application's record, in its own wire, under its family's number.
pub struct Family { pub family: u16, pub version: u32, pub bytes: Box<[u8]> }
pub enum Ask { Jig(Box<contract::Ask>), Family(Family) }
pub enum Answered { Jig(JigAnswer), Family(Family) }
pub enum Readable { Jig(contract::Read), Family(Family) }
pub enum Watchable { Jig(Subject), Family(Family) }
/// A record for the small store, by its owner.
pub enum Saved { Jig(JigRecord), Table { version: u16, record: RequestRecord<Ask> }, Family { family: u16, version: u16, key: Box<[u8]>, bytes: Box<[u8]> } }
pub enum Erased { Jig(StoreKey), Table(RequestKey), Family { family: u16, key: Box<[u8]> } }

/// Into the link: from the client's routing, then from the shell.
pub enum Event {
    /// The table's send: one attempt, one exchange.
    Send { attempt: Token, key: RequestKey, ask: Ask },
    Read { read: Token, what: Readable },
    Open { watch: Token, subject: Watchable },
    Close { watch: Token },
    Save { record: Saved },
    Erase { key: Erased },
    /// The shell: an exchange's answer with its body read to the end, or its failure.
    Answered { exchange: Token, status: u16, body: Box<[u8]> },
    Failed { exchange: Token },
    /// The shell: the stream's response head, its body's bytes as they come, its end.
    StreamOpened { status: u16 },
    StreamBytes { bytes: Box<[u8]> },
    StreamEnded,
    /// The shell: the step's store writes are durable, or failed.
    Written,
    WriteFailed,
    /// The shell, at start: the records it read.
    Start { kept: List<Box<[u8]>> },
}

/// Out of the link: to the shell first in a step, then to the routing.
pub enum Request {
    Write { key: Box<[u8]>, record: Box<[u8]> },
    Remove { key: Box<[u8]> },
    /// A POST to `path`; the shell adds its bearer token and the content type.
    Exchange { exchange: Token, path: &'static [u8], body: Box<[u8]> },
    Abandon { exchange: Token },
    OpenStream,
    CloseStream,
    /// One terminal per attempt.
    Answer { attempt: Token, envelope: RequestEnvelope, answer: Option<Answered> },
    /// One terminal per read.
    Read { read: Token, result: Readout },
    /// jig's watch events, and a family's deliveries as bytes.
    Watch { watch: Token, event: WatchEvent },
    FamilyDelivery { watch: Token, record: Family },
    Ended { watch: Token, end: WatchEnd },
    /// A served range: jig's for `jig_domain_client::Event::Hello`, a family's by number.
    Hello { family: Option<u16>, versions: VersionRange },
    Kept { record: Saved },
    Dropped { keeper: Option<u16> },
    Fact { fact: Fact },
}
pub enum Readout { Jig(ReadResult), Family { family: u16, record: Option<Box<[u8]>> } }
/// Content-free facts, for the root's facts.
pub enum Fact { UndecodableAnswer, UndecodableDocument, UndecodableEvent, StreamDropped }
```

- **Who owns what:** the table owns retries and keys; the client domain
  owns watches' states, backoffs and the hello's judgement; the link owns
  one exchange per attempt and read, the stream, and the small store's
  encoding; the shell owns the HTTP stack, the token and the storage.
- **What ends what:** an attempt ends with its answer, its failure or its
  deadline; a watch with its ending, its close's answer, or its stream's
  drop; the stream with its end, its silence or `CloseStream`.

## Questions to raise when reached

- **An answer to an exchange the link abandoned** at its deadline: dropped,
  as the attempt is already lost; nothing open expected.
- **A family's range out of date:** the design says the client "shows
  that it is out of date" per owner; the test client root has no page for
  a family out of date. Raise it at 32.3 if a family's range is ever out
  of the client's; the link hands it up regardless.

## Done when

- The component's tests pass: `heavy cargo nextest run -p
  jig-protocol-client-link` passes.
- It names no engine-side crate: `git grep -n -E
  '^(jig-domain-(core|people|tasks|fleet|views|host)|jig-protocol-engine)'
  -- crates/jig-protocol-client-link/Cargo.toml` prints nothing.
- No wasm32: `git grep -n -i wasm -- crates/jig-protocol-client-link`
  prints nothing.
- The translation has no wildcard: `heavy cargo clippy -p
  jig-protocol-client-link --all-targets -- -D warnings` passes.
- The gate passed on this session's last landing: `git -C ~/src/rust/jig
  log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step:.jig-32\.' origin/main | head -1` names
  `scripts/check.sh passed`.
