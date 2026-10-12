# Session 33: `jig-shell-native`, the reference service's clients' side, and the client's protocol world

The native shell carries the client link's exchanges and its stream over
skein's HTTP client, signs in by the engine's printed link, keeps its
bearer token and its small store in private files, and adds the token to
each exchange. The reference service gains its clients' side:
`jig-protocol-engine-clients` as a child of its engine's protocol layer,
the test application's records through a wire of their own, the printed
link handed to its caller. And the client's protocol world,
`tests/client-protocol`, runs the client world's stories through bytes:
the test application's client root and the fake person over
`jig-protocol-client-link` and the shell's machines, against the
engine's root and core over `jig-protocol-engine-clients` and skein's
HTTP server, joined by in-memory streams cut at random. Its first
increment is the meeting point: one thin story through bytes, before
anything else is added. "Nothing fancy": the shell is the minimal native
client the process proof (jig 26) needs, with no view of its own.

## Read first

- jig's `protocol/client.md`, sections 2, 4, 8, 10, 11, 12 (the native
  shell only) and 14 (but the browser tier).
- jig's `client/domain/testing.md`, sections 4 to 7 (the client's world,
  the joint world, the referee, faults); `client/domain/root.md`, section
  11; `protocol/service.md`, sections 2, 3 and 7.
- skein's `http.md`, sections 3 and 4; skein's `io.md`, sections 4 and
  5.3 (addresses, private files); skein's `testing-strategy.md`, 2.4 and
  2.5.
- `decisions.md`, 2 and 10; `code-organization.md`, sections 2 to 6.
- findings/protocol.md, items 1.7 (the joint world), 2.2 (the client test
  application), 2.4 (the fakes, `jig-scripted-engine`'s scenarios) and
  5.2 (lint regimes for ordinary crates).
- The code: jig 31's and jig 32's boundaries; jig 25's
  `testing/jig-test-service` (its engine's layer, routing, startup);
  `tests/client/src/harness/mod.rs` (`App`, `Front`, `FrontFault`),
  `tests/client/src/joint/{engine.rs, front.rs, fixture.rs}` (a `Front`
  over the core's world), `tests/client/src/stories.rs`;
  `testing/jig-test-domain-client/src/{boundary.rs, store.rs, family.rs,
  first.rs}`; skein's `testing/skein-world/src/stream.rs` (`Wire`),
  `crates/skein-http/src/client.rs`, `crates/skein-io/src/file.rs`
  (`OpenPrivate`, `Load`, `Store`).

## What the design asks

- **The native shell** (`client.md`, section 12): "skein's HTTP client on
  skein's io, handing the stream's body to the component, whose reader
  cuts it into events, for Linux and for the worlds that make bytes real;
  with no view, it is a service's shell."
- **Its token** (section 8): "A native client's secret is a bearer token,
  which its shell keeps where its platform keeps secrets, and sends with
  each exchange." It signs in by the printed link: "it sends the link's
  secret to the sign-in route in an exchange, and takes back its bearer
  token in the answer." Signing out is the shell's exchange.
- **Its small store** (section 10): "a file a native client writes whole
  and syncs".
- **The reference service's clients' side** (`service.md`, sections 2
  and 7): `jig-protocol-engine-clients` in the engine's protocol layer;
  an application's records "decoded by the application's wire, through
  the protocol layer's routing", reaching the root typed; the listeners
  opened before the restart; clients answered not ready until decisions
  open (section 3).
- **The protocol world** (`client.md`, section 14): "the client's world's
  stories run through bytes ... joined by in-memory streams cut at
  random. Its referee is the joint world's, and adds: every record
  crossed decodes to what was encoded; no secret reaches a domain; a slow
  stream never holds the engine up."
- **A fake's scenarios run against the real side** (development.md,
  5.3): the scripted engine's faults that cross the wire (busy, not
  ready, signed out, the versions, a dropped watch, no answer) run
  against `jig-protocol-engine-clients`.

## Choices this plan makes

- **The shell's machines apart from io:** `jig_shell_native::Connections`
  drives skein's HTTP client machines over `lib::stream` faces, one
  connection for the stream and up to `Limits::exchanges` for exchanges;
  `Shell` joins them to io's sockets and keeps the token and the small
  store in files. The worlds drive `Connections` over `Wire`s.
- **The token file and the small store's file** live in a private
  directory the shell's settings name, opened with io's `OpenPrivate`;
  each is rewritten in one piece through a synced temporary file.
- **The printed link is parsed by the shell:** its address and its
  `secret` query value (jig 31's choice); a link of another form is
  refused, naming it.
- **The test application's records cross as bytes of their own wire:**
  `testing/jig-test-protocol-client-wire`, a codec crate generated like an
  application's, holding the test family's asks, outcomes, subjects,
  reads, documents and small-store records, as
  `testing/jig-test-domain-client/src/{first.rs, family.rs}` and
  `testing/jig-test-connector/src/{application.rs, client_documents.rs}`
  have them. Its engine end's translation is `jig-test-service`'s
  `family.rs` (step code); its client end's is
  `tests/client-protocol/src/family.rs`. This crate is in
  `code-organization.md`, section 2's table (jig 33).
- **The protocol world's engine side is the joint world's,** made bytes:
  `jig_client_world::joint::engine::Driver` (the core's world under
  `jig-test-domain`) beneath `jig-protocol-engine-clients`, whose io is
  answered by the world's `net.rs` over `Wire` pairs. It is a `Front` of
  the client world's harness (`jig_client_world::Front`), so the harness's
  people, clients, faults and referee are reused unchanged.
- **The native client of the process layer is world code:**
  `jig_client_protocol_world::NativeClient`, a `skein_shell::Host` made of
  `Shell`, the link, the test client root and view, and a fake person's
  script; `tests/one-process` and `jig-test`'s process proof (jig 26) use
  it.
- **One sign-in provider, the link:** the protocol world and the native
  client sign in by the printed link whatever provider the client root
  names.
- **The parties' port stays** for a configuration with no clients' member
  (`tests/channel`, jig 25); with one, the root's outputs for parties go
  to the clients' component.

## Rules that bind it

- "**Ordinary Rust crates** take `tests/clippy.toml`'s lints, as
  `testing/jig-test-system` does." (`code-organization.md`, section 2)
  `jig-shell-native` has its own `clippy.toml` copied from
  `testing/jig-test-system/clippy.toml`.
- "**The client's crates** ... depend on no engine-side crate."
  (`code-organization.md`, section 3) `jig-shell-native` links neither a
  `jig-domain-*` crate nor an engine-side protocol crate.
- **Ceilings bind this session's world:** each focused test under 0.5 s,
  each fuzzy test about 5 s; `tests/client-protocol` aims for at most
  0.8 s focused and 4 s fuzzy for the package, serially. Missed: profile with the
  `measure` profile and make the tests cheaper before landing; past twice
  those figures, a question (development.md, 5.3).

## Increments

### 33.1 The shell's connections, and the first story through bytes

Scope: `crates/jig-shell-native` (ordinary Rust): `Connections`, carrying
the link's `Exchange`, `Abandon`, `OpenStream` and `CloseStream` over
skein HTTP client machines, each exchange with `Host`, `Content-Type:
application/octet-stream`, `Content-Length` and `Authorization: Bearer`;
each answer's body and the stream's bytes handed to the link; `sign_in(link)`
and `sign_out()`, the shell's own exchanges, the client root's `Request::SignOut` (jig
03) answered by `sign_out()` and `Event::SignedOut`. The world
`tests/client-protocol` (`jig-client-protocol-world`): `net.rs` (io for
jig 31's component: `Listen` answered, a socket per client connection,
its stream records joined to the client's connection through a `Wire`
pair, no cuts yet), `engine.rs` (the joint world's `Driver` beneath
`jig-protocol-engine-clients`, the routing between them: `Request::Core`
to the root, the root's `Held` and `Now` outputs for parties to the
component, `Request::Link` read as the person reads a terminal, secrets
answered by an in-memory table that outlives a restart),
`front.rs` (`Protocol`, a `Front<Test>`: per client, the link and
`Connections`), and one story. A family record the client root sends is
answered by the world as unknown until 33.2.
Needs: nothing beyond the session's start.
Files: `crates/jig-shell-native/{Cargo.toml, clippy.toml, src/{lib.rs,
connections.rs, sign_in.rs, tests.rs}}`; `tests/client-protocol/{Cargo.toml,
clippy.toml, src/{lib.rs, net.rs, engine.rs, front.rs, fixtures.rs},
tests/protocol.rs}`; the workspace `Cargo.toml`;
`tests/boundary/src/tests.rs`. Shared hotspots: the workspace
`Cargo.toml`, `tests/boundary/src/tests.rs`.
Shape: as the interface sketch.
Test shape: unit tests in `crates/jig-shell-native/src/tests.rs`:
`an_exchange_carries_the_bearer_token`,
`a_connection_cut_mid_body_fails_the_exchange`,
`the_stream_s_bytes_reach_the_link_in_order`,
`a_link_of_another_form_is_refused`. World, `tests/protocol.rs`, the
first story, in full:

```rust
use jig_client_protocol_world::{fixtures, protocol};
use jig_client_world::{Until, stories};

#[test]
fn a_person_signs_in_by_the_printed_link_and_starts_a_chat() {
    let mut w = protocol::world(fixtures::SEED);
    let c = w.client(fixtures::FIRST_OWNER, stories::conversation());
    w.person(c, fixtures::sign_in_then(stories::start_chat_steps()));
    let outcome = w.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert_eq!(w.front().chats_made(), 1);
}
```

Ceilings: unit tests under 20 ms each; the story under 0.5 s.
Proof: joint: `a_person_signs_in_by_the_printed_link_and_starts_a_chat`,
run once with `heavy cargo nextest run -p jig-client-protocol-world -E
'test(a_person_signs_in_by_the_printed_link_and_starts_a_chat)'`,
passes, which shows a person's request crosses both components and both
wires as bytes and is made once by the real core; under 0.5 s. The
process layer is jig 26's.
Don't: give `Connections` a socket or a file; hold the token anywhere but
the shell.
Size: at most 450 lines of production code (the shell); the world is
test code.
Removes: nothing.

### 33.2 The test application's records

Scope: `testing/jig-test-protocol-client-wire` (`family jig-test-client-wire
1`: the first connector's `ApplicationRequest` and `ApplicationOutcome`,
`ClientSubject`, `ClientDocument` and its parts, reads, and the client
side's small-store record `first::Record`), generated; its drift test in
`tests/client-wire`; the engine end's translation `jig-test-service`'s
`family.rs` (records to `jig_test_connector`'s client values and back,
the request's digest kept as it came); the client end's
`tests/client-protocol/src/family.rs` (records to
`jig_test_domain_client::first`'s values and back); the world's routing
of family records through both. 33.1's "answered as unknown" removed.
Needs: nothing beyond 33.1.
Files: `testing/jig-test-protocol-client-wire/{Cargo.toml,
schema/test-client-v1.schema, src/{lib.rs, generated/v1.rs}, golden/v1/}`;
`testing/jig-test-service/src/{family.rs, lib.rs}`;
`tests/client-protocol/src/{family.rs, engine.rs, front.rs}`;
`tests/client-wire/tests/test_client_wire.rs`; the workspace `Cargo.toml`.
Shared hotspots: the workspace `Cargo.toml`, `testing/jig-test-service`.
Shape: `jig_test_service::family::{request(record: &[u8], limits) ->
Result<connector::ApplicationRequest, Undecodable>, outcome(&connector::ApplicationOutcome)
-> Box<[u8]>, subject(..), document(..)}`, exhaustive both ways over the
connector's client values.
Test shape: step tests in `testing/jig-test-service/src/tests/family.rs`:
`an_application_request_round_trips_through_its_record`,
`a_client_document_round_trips_through_its_record`,
`a_request_that_does_not_decode_reaches_no_root`. World:
`a_person_s_part_request_is_answered_from_the_connector_s_outcome`.
Drift: `test_client_wire_schema_matches_generated_code_and_goldens`.
Ceilings as 33.1.
Proof: joint:
`a_person_s_part_request_is_answered_from_the_connector_s_outcome`, run
once with `heavy cargo nextest run -p jig-client-protocol-world -E
'test(a_person_s_part_request_is_answered_from_the_connector_s_outcome)'`:
a person's request to the test connector crosses as its family's records
and is answered from the connector's outcome; under 0.5 s.
Don't: name a jig crate in the wire crate's dependencies.
Size: at most 400 lines (schema and translations).
Removes: 33.1's unknown answer to family records.

### 33.3 The reference service's clients' side

Scope: `jig-test-service`'s `clients.rs`: `jig-protocol-engine-clients`
as a child of the engine's protocol layer when the configuration has a
clients' member (jig 20): its settings from it, the seed drawn from the
startup's seeds; its io by token; its listener opened before the
restart (`service.md`, section 3), exchanges answered not ready until
decisions open; `Request::Core` to the root; the root's
`Held::{PeopleReply, Read, Delivery, WatchEnded}` and
`Now::{WatchRefused, SignInRefused, RunText}` to it; `Event::Admitted`
after a root step that refused nothing of its watch; family records
through 33.2's `family.rs`; `Request::Secrets` to jig 17's secrets'
store (`Secrets::{Keep, Erase, Get}` under `SecretOwner::SignIn`) and
its answers back, a get's in the same iterate; `Event::SignInEnded` for
each `people::Key::SignIn(n)` a commit erases, once that commit is
durable (jig 03.2); `Request::Link` to `Service::links()`; the component
closed in 27.2's drain beside the others, and aborted on a failed
commit. Its limits from jig 20's profiles, where 31.1 put them. With no
clients' member, the parties' port, as jig 25 has it.
Needs: nothing beyond 33.2.
Files: `testing/jig-test-service/src/{clients.rs, route.rs, start.rs,
startup.rs, limits.rs, lib.rs, tests/clients.rs}`. Shared hotspots:
`testing/jig-test-service/src/route.rs`.
Shape: `Service::links(&mut self) -> &mut Queue<Box<[u8]>>`; `clients.rs`
routing functions `to_component(held: core::Held) -> Option<clients::Event>`
and `to_root(request: clients::Request) -> Routed`, exhaustive.
Test shape: step tests in `src/tests/clients.rs`:
`the_listener_opens_before_the_restart_begins`,
`a_party_s_reply_reaches_the_clients_component`,
`a_kept_digest_reaches_the_store_s_secrets`,
`an_erased_sign_in_record_ends_its_stream_once_durable`,
`the_printed_link_reaches_the_caller_once`,
`with_no_clients_member_party_outputs_go_to_the_port`. Under 20 ms each.
Proof: unit, `heavy cargo nextest run -p jig-test-service -E
'test(/clients::/)'`, run once: the clients' listener opens before the
restart, a kept digest reaches the store's secrets, and the printed link
reaches the caller once (the six tests); under 0.2 s. The joint and
process layers follow in 33.4.
Don't: let a token or digest reach the root or the parties' port.
Size: at most 500 lines.
Removes: nothing.

### 33.4 The shell on io, and a native client of the reference service

Scope: `jig_shell_native::Shell`: io's sockets to the engine's `Addr`
(one per connection `Connections` asks for), the token file and the
small store's file under a private directory (`OpenPrivate`, `Load` at
start, `Store` through a synced temporary), the store written before the
step's exchanges leave, a failed write withholding them; the inherent
`Host` entry points (`iterate`, `completions`, `submissions`,
`work_pending`, `next_deadline`, `next_policy_deadline`, `is_empty`).
`tests/client-protocol/src/native.rs`: `NativeClient`, a
`skein_shell::Host`: `Shell`, the link, the test client root and view
(`jig_client_world::Test`'s), and a fake person's script, signing in by
a link it is handed. A world test on skein's simulated world: the
reference service (engine only, a clients' member on a loopback address,
the service's store as `main` has it then: jig 25's port, or the
real store once jig 27.3 has landed) and one `NativeClient` on simulated
loopback.
Needs: nothing beyond 33.3.
Files: `crates/jig-shell-native/src/{shell.rs, files.rs, lib.rs,
tests.rs}`; `tests/client-protocol/src/{native.rs, lib.rs}`,
`tests/client-protocol/tests/native.rs`.
Shape: as the interface sketch.
Test shape: unit tests: `the_token_is_kept_and_read_back_at_start`,
`the_small_store_is_written_before_the_step_s_exchanges`,
`a_failed_store_write_withholds_the_exchanges`. World
(`tests/native.rs`), in full:

```rust
use jig_client_protocol_world::{fixtures, native};

#[test]
fn a_native_client_signs_in_by_the_service_s_link_and_starts_a_chat() {
    let outcome = native::run(fixtures::SEED, native::Script::start_chat(b"Hello"));
    assert_eq!(outcome.signed_in, 1);
    assert_eq!(outcome.chats_made, 1);
    assert!(outcome.drained);
}
```

Ceilings: unit tests under 20 ms; the world test under 0.5 s.
Proof: joint over skein's simulated io:
`a_native_client_signs_in_by_the_service_s_link_and_starts_a_chat`, run
once with `heavy cargo nextest run -p jig-client-protocol-world --test
native`, which shows the reference service's clients' side and the
native shell meet over sockets; under 0.5 s. The process layer is jig
26's.
Don't: keep the token in the small store, or anywhere a domain sees.
Size: at most 500 lines of production code.
Removes: nothing.

### 33.5 The client's world's stories through bytes, faults and the referee

Scope: `net.rs` cuts drawn from the seed (a connection reset mid-bytes,
both ends failing); the client world's stories through bytes, a few seeds
each; the faults of `client/domain/testing.md`, section 7 that cross the
wire, each run once against the real side (`FrontFault::{Busy, NotReady,
Restart, Slow, DropWatch, NoAnswer, SignedOut, Versions}` as the joint
front makes them, `tests/client/src/joint/front.rs:104-156`); the
referee: the joint world's rules, and `referee.rs`'s three: each record
crossed decodes to what was encoded (the world decodes each body it
carries and compares), no token's or digest's bytes in any event handed
to a domain on either side, and another client's exchanges answered
while one stream is not read. Fuzzy: random people over random stories,
cuts and faults drawn from the seed.
Needs: nothing beyond 33.4.
Files: `tests/client-protocol/src/{net.rs, referee.rs, front.rs,
stories.rs}`, `tests/client-protocol/tests/{protocol.rs, referee.rs,
fuzzy_protocol.rs}`; `tests/client/src/harness/` where a hook the front
needs is missing (at most 20 lines). Shared hotspots: `tests/client`.
Shape: `referee::Rules` added beside the joint referee, each a function
from the world's observations to a verdict.
Test shape: focused stories (`tests/protocol.rs`):
`a_request_whose_answer_was_lost_is_made_once`,
`a_slow_reader_gets_fewer_deliveries_and_others_are_answered`,
`a_reload_restores_a_pending_request_from_the_small_store_s_bytes`,
`signed_out_with_requests_pending_then_signed_in_again`,
`a_person_s_words_reach_a_running_run_through_bytes`,
`an_engine_outside_the_client_s_versions_shows_out_of_date`. Referee's
negative cases (`tests/referee.rs`):
`a_record_altered_on_the_wire_is_caught`,
`a_token_leaked_into_a_domain_event_is_caught`. Fuzzy
(`tests/fuzzy_protocol.rs`): `random_people_over_bytes_keep_the_referee_s_rules`,
seeds sized to about 3 s. Ceilings: focused stories under 0.5 s each;
the fuzzy test about 3 s.
Proof: joint, `heavy cargo nextest run -p jig-client-protocol-world -E
'test(a_request_whose_answer_was_lost_is_made_once)'` and `heavy cargo
nextest run -p jig-client-protocol-world --profile fuzzy`, each run
once: a request whose answer was lost is made once, and drawn people
over bytes cut at drawn moments keep the referee's rules; under 4 s
together.
Don't: change a client story's expected values to fit the bytes path (a
story that cannot pass through bytes is a question).
Size: world code only.
Removes: nothing.

## Interface sketches

```rust
// jig_shell_native: ordinary Rust. The native client's shell.
use jig_protocol_client_link::boundary as link;
use skein_lib::{Duration, Queue, Time, Token, stream};

/// The HTTP machines of one client, over stream faces: io-free, so worlds drive them.
pub struct Connections { /* … */ }
/// A connection's name in `Connections`; the shell or the world joins it to a socket.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Conn(pub u32);
pub struct Limits { pub exchanges: u32, pub body_bytes: u32, pub http: skein_http::client::Limits }
impl Connections {
    pub fn new(limits: &Limits) -> Connections { todo!() }
    /// The bearer token for each later exchange; None once signed out.
    pub fn set_token(&mut self, token: Option<Box<[u8]>>) { todo!() }
    /// Carry one of the link's requests to the shell: an exchange, the stream, an abandon.
    pub fn down(&mut self, now: Time, request: link::Request, below: &mut Queue<(Conn, stream::Down)>) { todo!() }
    /// A connection's bytes or end, into the link's events.
    pub fn up(&mut self, now: Time, conn: Conn, up: stream::Up, to_link: &mut Queue<link::Event>, below: &mut Queue<(Conn, stream::Down)>) { todo!() }
    /// The shell's own exchanges: the printed link traded for a token, and signing out.
    pub fn sign_in(&mut self, link: &[u8]) -> Result<Token, Unlinked> { todo!() }
    pub fn sign_out(&mut self) -> Token { todo!() }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Unlinked { NotHttp, NoSecret, Address }

/// Connections on skein's io, and the token and the small store in private files.
pub struct Shell { /* io, connections, files, the engine's address */ }
pub struct Settings { pub engine: skein_io::Addr, pub directory: Box<[u8]>, pub limits: Limits, pub io: skein_io::Limits }
```

```rust
// tests/client-protocol (jig-client-protocol-world): world code.
/// The engine's root and core beneath jig 31's component, the clients' links and
/// connections, joined by net.rs's Wires.
pub struct Protocol { /* engine: Engine, clients: BTreeMap<ClientId, Side>, net: Net */ }
// impl jig_client_world::Front<Test> for Protocol: bind, wire, poll, next_due, observed,
// fault, reload, received, sign_in — each through bytes.
/// A native client as a process of a skein world, simulated or real.
pub struct NativeClient { /* shell, link, root, view, person */ }
// impl skein_shell::Host for NativeClient.
```

- **Who owns what:** the shell owns sockets, files and the token; the
  link owns attempts and the stream; the world owns the joins, the cuts
  and the engine side's routing; the reference service's routing owns
  the clients' component, the family's records and the secrets' path.
- **What ends what:** a connection ends with its exchange or its cut; a
  `NativeClient` once its person's script is done and its shell has
  closed its sockets.

## Questions to raise when reached

- **The token's file beyond the worlds:** where a native client keeps it
  on Linux (proposal: a private directory its settings name, as here).
- **A story that cannot pass through bytes** as the joint world tells it:
  report it with the design text it rests on.

## Done when

- The shell and the world pass: `heavy cargo nextest run -p
  jig-shell-native -p jig-client-protocol-world -p jig-test-service`
  passes.
- The world's cost: `heavy --gate cargo nextest run -p
  jig-client-protocol-world --profile measure -j 1` reports the focused
  tests' sum; the commit body records it (aim: at most 0.8 s).
- The shell links no engine-side crate: `git grep -n -E
  '^jig-(domain|protocol-engine)' -- crates/jig-shell-native/Cargo.toml`
  prints nothing.
- No browser: `git grep -n -i -E 'wasm|jig-shell-web|cookie' --
  crates/jig-shell-native tests/client-protocol` prints nothing.
- The gate passed on this session's last landing: `git -C ~/src/rust/jig
  log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step:.jig-33\.' origin/main | head -1` names
  `scripts/check.sh passed`.
