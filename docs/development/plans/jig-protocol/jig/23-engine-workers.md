# Session 23: `jig-protocol-engine-workers`, the engine's side of the channel

A new step component of the engine's protocol layer owns the channels of
the workers that dial in: one skein channel per accepted connection, the
opening's checks, one channel per worker, the hello, liveness and flow,
and the translation between `jig-protocol-worker-wire`'s records and the
hub's link vocabulary, with grants' values filled in as it encodes. The
engine's root hears a worker's link in the vocabulary its own hub speaks
and routes it the same way (jig's `protocol/channel.md`, section 1). The
streams under it are its protocol layer's (jig 25); it links neither the
core nor smith. Its machine world, `tests/engine-workers`, runs it
between the root's side, scripted, and skein's scripted peer as each
worker.

## Read first

- jig's `protocol/channel.md`, sections 1 to 4, 5, 7 to 10 and 12;
  `domain/hosts.md`, section 2 and 6.1; `domain/engine.md`, section 8;
  `protocol/README.md`, section 8.
- skein's `channel.md`, sections 5 to 10; programming-model.md, sections
  4, 5.2 and 9; testing-strategy.md, 2.4.
- `decisions.md`, 6; `code-organization.md`, sections 2 to 6.
- `findings/protocol.md`, 4.1 (skein-channel and its scripted peer) and
  6.2 ("23-engine-workers").
- The code: jig 22's `crates/jig-protocol-worker-wire` (records,
  `Reason`, `schema`, `terms_gap`, `sizes`); jig 05's link vocabulary in
  `crates/jig-domain-host/src/boundary.rs` and the fleet's `Bounds`
  (jig 05's sketches), written here as jig 25.2 names the two directions:
  `host::LinkUp`, what the hub sends toward the engine (the hello as one
  value, `host::LinkHello`; turns, calls, answers, activity, notices),
  and `host::LinkDown`, what the engine sends the hub (assignments,
  messages, relayed answers, turns kept and busy, acknowledgements,
  cancels, grants); where jig 05 landed other names, use them, and
  nothing else here changes; jig 05's fleet `Request::Refuse`
  with its reason, and `crates/jig-domain-fleet/src/channel.rs:186-192`
  (a lost channel the fleet does not know is ignored). Shapes to copy:
  skein's `crates/skein-channel/src/boundary.rs` (`Request`, `Event`,
  `Lower`, `LowerEvent`, `Closed`, `Unsent`), `machine.rs:148-153`
  (`Machine::new`, `with_mode`); smith's
  `crates/smith-protocol-channel/src/component.rs:121` and `:637-652`
  (a responder judging `Opening`); skein's
  `testing/skein-fake-channel/src/lib.rs:44-133` (`ScriptedPeer`:
  `new`, `play`, `expect`, `pop_output`, `feed`, `observed`) and
  `tests/channel/tests/generic_protocol.rs:47-120` (a machine driven
  over in-memory bytes); `crates/skein-io/src/digest.rs:18` and `:153`
  (`Digest`, `digest`).

## What the design asks

- **The opening, checked in order** (channel.md, 4.1): "the name and the
  secret, the secret compared in constant time with the digest the
  configuration holds for that name; an unknown name and a wrong secret
  are one refusal, *unauthorized*"; then "the application's payload
  version, equal to its own, or *application*"; then "room for one more
  channel: the fleet's limit on workers, less the channels open, or
  *busy*. An opening under the name of a worker that has a channel always
  has room". Terms checked both ways, refused with skein's *limits*
  "naming the kind".
- **One channel per worker** (4.2): "The engine's side refuses the older
  channel with *replaced*, best effort, and its domain hears that channel
  lost, before it accepts the newer one." Channels are "named by a token
  the engine's side gives it and never reuses".
- **The hello** (4.3): "The engine's domain hears a channel only with its
  hello"; "Nothing but the hello is taken before it"; a hello the fleet
  refuses closes the channel *not admitted*, "the fleet's reason as its
  text". The opening within the handshake's deadline, the hello within
  the hello's deadline from `Accept`.
- **Translation** (section 3): "a module of small total functions ...
  exhaustive both ways"; the application's parts handed up as bytes and
  carried down whole.
- **Grants' values filled in** (section 8): from "the table the grant
  names, through its protocol layer. A grant whose generation the table
  no longer holds is left out as it is encoded."
- **Liveness and flow** (section 9): pings after the ping interval of
  sending nothing; lost after the silence deadline, or the stall
  deadline with output queued and nothing drained; each side's cap from
  the domains' limits; "A link that fills its cap anyway is lost".
- **The engine's side** (10.2): accepted connections "bounded by the
  fleet's workers plus a few opening at once"; a channel lost "goes up as
  the fleet's lost channel, once, if its hello was heard"; "Shutting
  down, new openings are refused with *stopping*, and open channels close
  once the engine's last outputs to them have drained."
- **States** (10.3): `accepted ─► opening ─► terms ─► hello ─► open ─►
  closing ─► closed`; "A domain output for a channel that is gone is
  dropped".

## Choices this plan makes

- **The streams are the protocol layer's** (channel.md, section 2: "The
  streams are the protocol layer's to make ... The components take the
  streams they are given, as skein's channel does"). The component
  never makes an io request: it hears an accepted connection by the
  layer's token, answers whether it takes it, and exchanges skein's
  `Lower` and `LowerEvent` records per connection; the layer maps them
  to io's socket records (or, later, a TLS machine's).
- **A credential that does not decode is refused *unauthorized***, like
  an unknown name, so a probe learns nothing more.
- **The digest compare** walks the four words of both digests, folding
  their differences, with no early exit; skein-io's `digest` makes the
  credential's.
- **Channel tokens start at 1** (the fleet keeps 0 for the engine's own
  hub, `crates/jig-domain-fleet/src/channel.rs:101`) and count up; a
  `u64` never wraps in a process's life.
- **A record before the hello, a second hello, or a body that does not
  decode** is refused with skein's *framing* (skein's `channel.md`,
  section 6: "A body that does not decode is the owner's to judge").
- **Every channel whose hello went up is told lost once,** the not
  admitted among them: the fleet ignores a lost channel it does not know
  (`crates/jig-domain-fleet/src/channel.rs:190-192`).
- **The application's parts travel as jig 22's `Part`s:** a part is named in the
  vocabulary by a token; down, the layer gives each part's bytes under
  the root's token; up, the component names each part it decodes with a
  token of its own, never reused, and the layer decodes the bytes under
  it (channel.md, section 3: "A component hands up a part's bytes as it
  decodes a record").
- **Values come with the output that names them:** `Down` carries one
  `Option<Secret>` per grant the record names, in its order; `None`
  leaves that grant out. The component moves each value into the record
  it encodes and holds none past that step.
- **The machine world plays the protocol layer:** `tests/engine-workers`
  (package `jig-engine-workers-world`, ordinary Rust) joins each
  connection to a `skein_fake_channel::ScriptedPeer` by in-memory bytes
  cut at seeded sizes; the channel's protocol world, `tests/channel`,
  stays jig 25's.

## Rules that bind it

- "**`jig-protocol-*` crates** depend on no application's crate. Only
  `jig-protocol-charter` and `jig-protocol-transcript` link smith; the
  channel's components do not." (code-organization.md, section 3)
- "No increment of this plan ... adds an `unreachable!` arm for a
  variant the reference service can reach; an arm the service reaches is
  routed." (code-organization.md, section 6): the translation is total
  both ways.
- "**Step code** follows skein's `programming-model.md`, section 10, and
  jig's `AGENTS.md`: no traits, no generic items, no `dyn`, no closures."
  (code-organization.md, section 2)

## Increments

### 23.1 The component and the opening

Scope: the crate `crates/jig-protocol-engine-workers` (component, step
code); `Limits`, `worst_case`, the entry points' most outputs; accepted
connections bounded by `workers + opening`, the rest turned away; a skein
channel per connection as responder (`StreamMode::One`); the opening's
checks in channel.md, 4.1's order; a worker replaced by a newer opening
under its name; the terms stated from `largest_body` and the peer's
checked with `terms_gap`, refused *limits* naming the kind; the
handshake's deadline (channel.md, 4.1, 4.2, 10.2 and 10.3).
Needs: nothing beyond the session's start.
Files: `crates/jig-protocol-engine-workers/{Cargo.toml, src/lib.rs,
src/boundary.rs, src/limits.rs, src/component.rs, src/connection.rs,
src/opening.rs, src/tests.rs, src/tests/fixtures.rs,
src/tests/opening.rs}`; `tests/engine-workers/{Cargo.toml, src/lib.rs,
src/net.rs, src/world.rs, src/fixtures.rs, tests/opening.rs,
tests/memory.rs}`; the workspace `Cargo.toml` (members; the crate,
`skein-channel`, `skein-io`, `skein-fake-channel`);
`tests/boundary/src/` (the crate's role: skein-lib, skein-channel,
skein-io, `jig-protocol-worker-wire`, `jig-domain-host`; never a
`jig-domain-*` crate but the hub's, nor smith). Shared hotspots: the
workspace `Cargo.toml`, `tests/boundary/src/`.
Shape: as the interface sketch: `boundary.rs` (`Event`, `Request`,
`Worker`; `Part` is jig 22's), `limits.rs` (`Limits`, `worst_case`, `MAX_OUT_DOWN`,
`MAX_OUT_FIRE`), `component.rs` (`Component::new`, `down`, `fire`,
`next_deadline`, `reclaim`), `connection.rs` (one connection's state,
`Phase::{Opening, Terms, Hello, Open, Closing}`, its machine and
deadlines), `opening.rs`: `pub(crate) enum Verdict { Accept, Refuse(Reason),
Replace { older: Token } }`, what an opening's credential earns, from
`pub(crate) fn judge(&self, credential: &[u8]) -> Verdict` checking
channel.md, 4.1's three steps in order (the name and secret, the payload
version, room), and `pub(crate) fn same(left: &Digest, right: &Digest)
-> bool`, equal digests in time that does not depend on where they
differ.

Test shape:
- `src/tests/fixtures.rs` (new): `LIMITS` (two workers, one opening at
  once, 1 s handshake, 1 s hello), `WORKERS` with `WORKER_A`
  (`b"worker-a"`, secret `b"secret-a"`) and `WORKER_B` (`b"worker-b"`,
  secret `b"secret-b"`), `PAYLOAD` (3), `credential(name, secret,
  payload) -> Box<[u8]>`.
- `src/tests/opening.rs`: `a_wrong_secret_and_an_unknown_name_refuse_alike`,
  `a_wrong_secret_with_another_payload_is_unauthorized`,
  `digests_differing_in_the_last_word_are_not_the_same`,
  `an_undecodable_credential_is_unauthorized`.
- `tests/engine-workers/src/net.rs` (new): per connection, the peer's
  bytes delivered to the component's read demands in seeded cuts, the
  component's sends fed to the peer in seeded cuts, write room granted
  unless a test withholds it, each control frame the component sends
  decoded with `skein_channel::decode_control` and kept, so a test reads
  a refusal's reason and text.
- `tests/engine-workers/src/world.rs` (new): `World::new(&Limits,
  &[Worker], seed)`, `connect(ScriptedPeer) -> Token`, `run()` (until
  nothing moves), `advance(Duration)`, `down(Event)`, `refusal(Token) ->
  Option<(u16, Box<[u8]>)>`, `root() -> &[Request]` (what reached the
  root, in order).
- `tests/engine-workers/src/fixtures.rs` (new, the world's fixtures
  module): `WORKER_A`, `SECRET_A`, `WORKER_B`, `SECRET_B`, `PAYLOAD`
  (3), `OTHER_PAYLOAD` (4), `limits()`, `workers()`, `worker(name,
  secret, payload) -> ScriptedPeer`
  (initiator, version 1, the credential encoded by
  `jig_protocol_worker_wire::Credential`).
- `tests/engine-workers/tests/opening.rs`, reference test:

  ```rust
  #[test]
  fn another_payload_version_is_refused_as_application() {
      let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
      let connection = world.connect(fixtures::worker(WORKER_A, SECRET_A, OTHER_PAYLOAD));
      world.run();
      let (reason, _) = world.refusal(connection).expect("a refusal");
      assert_eq!(reason, 257);
      assert!(world.root().is_empty());
  }
  ```

  Others: `a_wrong_secret_is_refused_as_unauthorized`,
  `a_third_worker_past_the_fleet_limit_is_refused_as_busy`,
  `a_worker_dialling_again_replaces_its_open_channel`,
  `another_version_is_refused_by_skein_as_version`,
  `terms_below_the_largest_assignment_are_refused_as_limits`,
  `an_opening_past_the_handshake_deadline_is_closed`,
  `connections_past_workers_and_openings_are_turned_away`.
- `tests/engine-workers/tests/memory.rs` (new): the component with every
  connection open at its limits, measured by `skein_world`'s counting
  allocator against `worst_case`, as `tests/host/tests/memory.rs:1-14`
  does: `the_component_at_its_limits_stays_within_its_worst_case`.
- Ceilings: step tests under 20 ms; world tests under 0.5 s each, the
  world's focused tests at most 0.6 s in all by the end of this session.
Proof: world, `heavy cargo nextest run -p jig-engine-workers-world -E
'test(opening)'` once: each wrong opening ends in its reason before the
root hears anything, and a worker dialling again replaces its channel;
under 1 minute with the build warm. Not higher: no joint world composes
the component until jig 24.5 and jig 25.
Don't: make an io request; read the configuration (the workers' digests
come in `new`); compare a secret any way but `same`.
Size: at most 500 lines of production code.
Removes: nothing.

### 23.2 The hello and the up records

Scope: the hello within its deadline after both terms, the only record
taken before it; `Request::Hello` with a fresh channel token; the root's
`NotAdmitted` refusing the channel *not admitted*, the fleet's reason as
text; `translate.rs`'s up half: each up record to its `host::LinkUp`
variant, the application's parts named by tokens of the component's;
`Lost` once for each channel whose hello went up, a replaced channel's
before its successor's hello (channel.md, 3, 4.2, 4.3, 5.1, 6.2).
Needs: nothing beyond 23.1.
Files: `crates/jig-protocol-engine-workers/{src/translate.rs,
src/connection.rs, src/component.rs, src/boundary.rs,
src/tests/translate.rs, src/tests/fixtures.rs}`;
`tests/engine-workers/{src/world.rs, src/fixtures.rs, tests/hello.rs,
tests/records.rs}`.
Shape: `translate::up(record: wire::Up, parts: &mut Parts) ->
host::LinkUp`, one function per record kind, each total over its record
(`wire::Up` is this crate's enum over the decoded up records); a body
that does not decode is refused *framing* before `up` runs. `Parts`
mints `Part` tokens from a counter.
Test shape:
- `src/tests/translate.rs` (new), reference test:

  ```rust
  #[test]
  fn a_turn_record_becomes_the_hubs_turn_with_its_refusals() {
      let mut parts = Parts::new(4);
      let link = translate::up(fixtures::turn_record(), &mut parts);
      let host::LinkUp::Turn { run, attempt, turn } = link else { panic!("a turn: {link:?}") };
      assert_eq!((run, attempt), (Token::new(7), Token::new(2)));
      assert!(turn.opens_window);
      assert_eq!(turn.refused.len(), 1);
      assert!(parts.is_empty());
  }
  ```

  The field names are jig 05's `host::Turn`'s. Others, one per up kind:
  `a_hello_record_becomes_the_hubs_hello_with_its_bounds`,
  `an_answer_record_keeps_its_unread_fence`,
  `an_answer_hands_left_and_saved_work_up_as_parts`,
  `a_fact_record_becomes_typed_activity`,
  `a_call_record_keeps_its_name_bytes`,
  `rejected_and_exhausted_records_become_notices`.
- `tests/engine-workers/tests/hello.rs` (new):
  `the_root_hears_a_channel_only_with_its_hello`,
  `a_turn_before_the_hello_is_refused_as_framing`,
  `a_hello_past_its_deadline_closes_the_channel_unheard`,
  `a_hello_not_admitted_is_refused_with_the_fleets_text`,
  `a_replaced_channel_is_lost_before_its_successors_hello`,
  `channel_tokens_start_at_one_and_never_repeat`.
- `tests/engine-workers/tests/records.rs` (new):
  `each_up_record_reaches_the_root_in_order`,
  `an_undecodable_turn_body_is_refused_as_framing`.
Proof: world, `heavy cargo nextest run -p jig-engine-workers-world -E
'test(hello) | test(records)'` once: a worker's hello reaches the root
before anything else from it, each up record reaches the root as the
hub's vocabulary in order, and a not-admitted worker is refused with the
fleet's text; under 1 minute with the build warm.
Don't: hand the root a record before its channel's hello; decode an
application's part or smith's bytes.
Size: at most 450 lines of production code.
Removes: nothing.

### 23.3 The down records and grants' values

Scope: `translate.rs`'s down half: each `host::LinkDown` variant to its
record, the parts' bytes written whole, each grant's value moved in from
`Down`'s values, a grant without one left out; a `Down` for a channel
that is gone, or not yet heard, dropped (channel.md, 5.2, 6.2, 8, 10.3).
The assignment's run identity is the 16 bytes the hub's assignment
carries, made by `jig_protocol_charter::run_identity` (jig 06.2) where
the engine's protocol layer puts the assignment together; this component
copies it whole (decision 34).
Needs: 23.2; jig 05.2's identity field in the hub's `RunAssignment`
(this component copies its 16 bytes; jig 06.2's `run_identity` is the
reference service's to call, decision 34).
Files: `crates/jig-protocol-engine-workers/{src/translate.rs,
src/component.rs, src/boundary.rs, src/tests/translate.rs,
src/tests/fixtures.rs}`; `tests/engine-workers/{src/fixtures.rs,
tests/records.rs, tests/grants.rs}`.
Shape: `translate::down(down: host::LinkDown, parts: &[Part], values:
Box<[Option<Secret>]>) -> Option<wire::Down>` (`None`: a part the
vocabulary names is missing from `parts`, a routing defect: the output
is dropped and counted). A grant whose value is `None` is left out of
the assignment's or the granted record's grants; a granted record with
none left is not sent.
Test shape:
- `src/tests/translate.rs` grows: `an_assignment_keeps_its_start_messages_and_identity`,
  `an_assignments_items_come_from_their_part`,
  `a_grant_without_a_value_is_left_out`,
  `a_down_event_debug_shows_value_lengths_only` (searches the formatted
  `Event::Down` for `TOKEN_ONE`'s bytes: none), `a_relayed_busy_answer_carries_no_text`,
  `turn_kept_and_turn_busy_records_name_the_turn`.
- `tests/engine-workers/tests/grants.rs` (new):
  `a_grant_of_each_table_reaches_the_worker_with_its_value`,
  `an_assignment_drops_the_grant_the_table_no_longer_holds`,
  `a_down_record_for_a_lost_channel_is_dropped`.
Proof: world, `heavy cargo nextest run -p jig-engine-workers-world -E
'test(grants) | test(records)'` once: an assignment reaches its worker
with its start's messages, its identity and the values of the grants the
tables hold, and a grant the table no longer holds is left out; under 1
minute with the build warm.
Don't: keep a value past the step that encodes it; read a table (the
values are the layer's to give).
Size: at most 400 lines of production code.
Removes: nothing.

### 23.4 Liveness, flow, loss and stopping

Scope: a ping after the ping interval with nothing sent; a frame read
always outstanding while open, and lost on the silence deadline; the
stall deadline while frames are queued, reset on `Drained`; the output
cap from `engine_cap`, a refusal `Full` a loss; a stream that fails or
ends, a framing error and a refusal from the worker each a loss, told
once if its hello went up; `Stop` refusing new openings *stopping*;
`Close` finishing each channel once its queue drains, then `Closed` once
every connection is gone; `Abort` at once (channel.md, 9, 10.2, 10.3).
Needs: nothing beyond 23.3.
Files: `crates/jig-protocol-engine-workers/{src/connection.rs,
src/component.rs, src/limits.rs, src/tests/fixtures.rs}`;
`tests/engine-workers/{src/net.rs, src/world.rs, src/referee.rs,
tests/liveness.rs, tests/stopping.rs, tests/fuzzy_engine_side.rs,
tests/memory.rs}`.
Shape: `connection.rs`'s deadline per phase (`next_deadline` the
earliest of its connections'); `limits.rs`'s `worst_case` counts each
connection's skein channel at `engine_cap`
(`skein_channel::worst_case`).
Test shape:
- `tests/engine-workers/tests/liveness.rs` (new):
  `a_silent_worker_is_lost_at_the_silence_deadline_and_not_before`,
  `a_worker_that_stops_reading_is_lost_by_stall`,
  `an_idle_channel_sends_a_ping_after_the_interval`,
  `a_link_that_fills_its_cap_is_lost`,
  `a_stream_ending_mid_frame_loses_the_channel_once`.
- `tests/engine-workers/tests/stopping.rs` (new):
  `a_stop_refuses_a_new_opening_as_stopping`,
  `a_close_waits_for_queued_records_to_drain`,
  `an_abort_closes_every_connection_at_once`.
- `tests/engine-workers/src/referee.rs` (new): over `World::root()`,
  each channel's `Hello` at most once and first, its `Lost` at most once
  and after it, no `Up` after its `Lost`, `Closed` last.
- `tests/engine-workers/tests/fuzzy_engine_side.rs` (new), reference
  test:

  ```rust
  #[test]
  fn fuzzy_workers_cuts_and_stalls_keep_each_channel_told_once() {
      for seed in 0..40 {
          let mut world = World::new(&fixtures::limits(), &fixtures::workers(), seed);
          let mut rng = Rng::new(seed);
          for _ in 0..6 {
              world.connect(fixtures::drawn_worker(&mut rng));
              world.advance(Duration::from_millis(rng.between(1, 400)));
          }
          world.down(Event::Stop);
          world.down(Event::Close);
          world.advance(Duration::from_secs(30));
          referee::judge(world.root()).expect("each channel told once, in order");
      }
  }
  ```

  `drawn_worker` draws a good or wrong credential, cuts, garbage frames,
  a silence and a stall.
- `tests/memory.rs` grows: `every_connection_at_its_cap_stays_within_the_worst_case`.
- Ceilings: world tests under 0.5 s each; the fuzzy test under 3 s; the
  world's focused tests at most 1 s in all.
Proof: world, `heavy cargo nextest run -p jig-engine-workers-world` once
(focused), and `heavy cargo nextest run -p jig-engine-workers-world
--profile fuzzy` once: a silent or stalled worker is lost at its
deadline and told once, and a stop drains and closes; under 2 minutes
with the build warm.
Don't: queue past the cap; send a ping that reaches the root.
Size: at most 400 lines of production code.
Removes: nothing.

## Interface sketches

```rust
// jig_protocol_engine_workers::boundary
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_protocol_worker_wire::Part;
use skein_channel::{Lower, LowerEvent};
use skein_io::digest::Digest;
use skein_lib::{Secret, Token};

/// A configured worker: its name and its secret's digest (configuration.md, section 3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Worker {
    pub name: Box<[u8]>,
    pub digest: Digest,
}

/// What the engine's protocol layer and root route to the component.
#[derive(Debug)]
pub enum Event {
    /// From the layer: a listener accepted `connection`; answered Take or TurnAway in this step.
    Accepted { connection: Token },
    /// From the layer: a lower event of `connection`'s stream.
    Lower { connection: Token, event: LowerEvent },
    /// From the layer: `connection`'s socket is gone, its last event.
    Gone { connection: Token },
    /// From the root: a record for the worker on `channel`, its parts' bytes, and one value
    /// per grant it names, in order (None: the table no longer holds it; the grant is left out).
    Down { channel: Token, down: host::LinkDown, parts: Box<[Part]>, values: Box<[Option<Secret>]> },
    /// From the root: the fleet refused `channel`'s hello; `text` is its reason, at most 256 bytes.
    NotAdmitted { channel: Token, text: Box<[u8]> },
    /// From the root: shutting down; new openings are refused *stopping*.
    Stop,
    /// From the root: finish each channel once drained; `Closed` follows when every connection is gone.
    Close,
    /// From the root: close every connection now; `Closed` follows.
    Abort,
}

/// What the component asks of its layer, and tells its root.
#[derive(Debug)]
pub enum Request {
    /// To the layer: keep `connection` (io's Bind).
    Take { connection: Token },
    /// To the layer: refuse `connection` (io's Reject); no room for another opening.
    TurnAway { connection: Token },
    /// To the layer: a lower record for `connection`'s stream.
    Lower { connection: Token, lower: Lower },
    /// To the layer: close `connection` gracefully; `Gone` follows.
    Close { connection: Token },
    /// To the layer: abort `connection`; `Gone` follows.
    Abort { connection: Token },
    /// To the root: a worker's hello, the first it hears of `channel`.
    Hello { channel: Token, hello: host::LinkHello },
    /// To the root: a record from the worker's hub, its parts' bytes under the tokens it names.
    Up { channel: Token, up: host::LinkUp, parts: Box<[Part]> },
    /// To the root: a channel whose hello went up is lost; its last word for `channel`.
    Lost { channel: Token },
    /// To the root: the component's terminal, after Close or Abort.
    Closed,
}
```

```rust
// jig_protocol_engine_workers::{limits, component}
use jig_protocol_worker_wire::sizes::Sizes;
use skein_lib::{Duration, Queue, Time};

/// The component's limits (channel.md, section 11), from its layer's configuration (jig 20, jig 25).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub sizes: Sizes,
    /// The fleet's workers, and the connections opening at once beyond them.
    pub workers: u32,
    pub opening: u32,
    /// The chunk read at once.
    pub chunk: u32,
    pub handshake: Duration,
    pub hello: Duration,
    pub ping: Duration,
    pub silence: Duration,
    pub stall: Duration,
}

/// The heap the component may hold at these limits, or None if unusable.
pub fn worst_case(limits: &Limits) -> Option<u64> { todo!() }

/// The most requests one `down` or `fire` pushes.
pub const MAX_OUT_DOWN: u32 = 0;
pub const MAX_OUT_FIRE: u32 = 0;

/// Why the component cannot be made.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unusable {
    /// Limits whose worst case or schema does not fit.
    Limits,
    /// More workers than the limits hold, or a name past its bound, or named twice.
    Workers,
}

/// The engine's side of the workers' channel.
#[derive(Debug)]
pub struct Component;

impl Component {
    /// The component for these workers; `payload` is the application's payload version.
    pub fn new(limits: &Limits, workers: Box<[Worker]>, payload: u32) -> Result<Component, Unusable> { todo!() }
    /// What its layer and root route to it.
    pub fn down(&mut self, now: Time, event: Event, out: &mut Queue<Request>) { todo!() }
    /// Its deadlines: handshakes, hellos, pings, silence, stalls.
    pub fn fire(&mut self, now: Time, out: &mut Queue<Request>) { todo!() }
    pub fn next_deadline(&self) -> Option<Time> { todo!() }
    /// Frees what the last iteration settled.
    pub fn reclaim(&mut self) { todo!() }
}
```

- **Who owns what:** the component owns the channels, the workers' names,
  the deadlines and the parts' tokens it mints; the layer owns sockets,
  listeners and (later) TLS; the fleet owns workers by channel token,
  fencing and resends; the routing owns the tables' values and the
  application's codecs.
- **What ends what:** `Lost` ends a channel for the root, after its
  `Hello`; `Gone` ends a connection for the component; `Closed` ends the
  component, once, last.

## Questions to raise when reached

- **The link vocabulary as one pair of enumerations** (jig 22's
  Questions): if jig 05 lands no `LinkUp`, `LinkDown` and hello as one
  value, raise it before 23.2 starts; this session never edits the hub's
  boundary (decisions.md, 6).
- **Turn busy in the hub's vocabulary.** The worker's hub never hears a
  turn's busy (channel.md, 5.2), but this component must take one from
  the root: if jig 05's `host::LinkDown` has no turn busy, raise it
  before 23.3 lands.
- **The deadlines and openings at once** are profile numbers (jig 20),
  and channel.md, 14 keeps the timings open: report the values the world
  uses.

## Done when

1. The world's focused tests pass: `heavy cargo nextest run -p
   jig-engine-workers-world` prints no failure, and its fuzzy test
   passes: `heavy cargo nextest run -p jig-engine-workers-world --profile
   fuzzy`.
2. The crate links neither the core nor smith: `cargo tree -p
   jig-protocol-engine-workers -e normal --prefix none | grep -E
   '^(jig-domain-(core|fleet|tasks|people|accounts)|smith)'` prints
   nothing.
3. No io request is made by the component: `git grep -n
   'skein_io::Request' crates/jig-protocol-engine-workers/src` prints
   nothing.
4. The gate passed on this session's last landing: `git -C
   ~/src/rust/jig log --format='%(trailers:key=Gate-Result,valueonly)'
   --grep='^Step:.jig-23\.' origin/main | head -1` names
   `scripts/check.sh passed`.
