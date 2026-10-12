# Session 25: the test worker, the reference service's spine, and the channel's protocol world

jig's test application gets a worker and the spine of its service, and
the workers' channel its protocol world. `jig-test-worker` is the test
worker's root, step code: the hub, smith's host domain as its agent
capability, and a test workspace, handing smith's payloads out to its
protocol layer to be decoded and encoded. `jig-test-service` is the
reference service (`protocol/service.md`, section 7), step code written
to be copied: a startup from configuration's bytes; the engine's layer
(the test root, its journal, its components: the accounts, the host
tools' translation, the workers' side, its test connector's system in
memory) and a test worker's layer (its root, the link, smith's host half
and its agent processes), each with its `iterate`, in one `Service` that
routes io's events by the tokens each gave. `tests/channel` then runs
the engine's and a worker's services as processes of skein's simulated
world, joined over loopback sockets that the simulator cuts, the
worker's agents smith's real agent processes over skein's fake LLM. Its
first story runs as soon as both sides answer (25.6), and grows.

## Read first

- jig's `protocol/service.md`, sections 1 to 7 and 10;
  `protocol/channel.md`, sections 1 to 3, 5.3, 6, 7, 8, 10 and 12;
  `protocol/configuration.md`, sections 2 to 4, 7 and 10;
  `protocol/llm.md`, 5.4 and section 6; `protocol/README.md`, section 4.
- jig's `domain/hosts.md`, 6.1, 6.6, 6.7, 6.8 and section 11;
  `domain/root.md`, sections 3 to 8.
- skein's `programming-model.md`, sections 2, 4.2 and 9; skein's
  `testing-strategy.md`, 2.5 and 2.7; skein's `simulator.md`, 3.3.
- `decisions.md`, 8, 10, 11, 17 and 20; `code-organization.md`, sections
  2 to 6.
- This plan's session files for what this session composes:
  `jig/05-hub-link.md` (the hub's `Hello`, `LinkUp`, `LinkDown`,
  `Activity`), `jig/09-stopping.md`, `jig/10-host-tools.md` (10.3 and
  10.5: calls, keys and results read outside the roots),
  `jig/20-config.md`, `jig/21-accounts.md`, `jig/23-engine-workers.md`
  and `jig/24-worker-link.md` (their boundaries' sketches).
- findings/protocol.md, items 2.1 (the test root), 2.3 (the test
  connector and its system), 2.4 (the fakes), 3.1 to 3.5 (smith's pieces
  a worker needs), 4.1 (skein's channel) and 4.9 (the fake LLM peer and
  skein's world).
- The code: `testing/jig-test-domain/src/{lib.rs, routes.rs}`;
  `crates/jig-domain-host/src/boundary.rs` as jig 05 leaves it;
  `testing/jig-test-system/src/lib.rs:200-245` (the system's answers);
  jig 10's, 20's, 21's, 23's and 24's crates; smith's
  `tests/hosts/src/lib.rs:70-420` (the host routing to copy: `iterate`,
  `route_process`, `route_channel`, `route_domain`) and
  `tests/hosts/src/world.rs:217-222, 400-430` (smith's agent hosted in a
  world); smith's `tests/agent-process/src/{lib.rs, fake.rs}`; skein's
  `examples/echo/service/src/lib.rs` (a service's `iterate` and its
  `Host` entry points), `testing/skein-world/src/world.rs`
  (`spawn_restartable`, `cut`, `host`).

## What the design asks

- **The reference service** (`service.md`, section 7): "jig's test
  application's engine protocol layer (the store, the accounts', the host
  tools' translation, the clients' side, the workers' side, its test
  connector's system answered in memory); its engine runs no agents of
  its own and a test worker's (the link, smith's host half and its agent
  processes, the test workspace), each with its `iterate`, composed in
  one process (section 10), with a startup function that takes
  configuration's bytes and returns the service."
- **A parent of components** (sections 2 and 4): io's events by token;
  what a component hears, one root event; one decision per root event;
  "A store's answer is always taken"; calls decoded and answers rendered
  in the routing; "smith's payloads, decoded and encoded in the routing";
  values between components, "never through the root"; each deadline
  fired where its owner reports it.
- **Start in order** (section 3): configuration, the store, the
  components, the listeners, the root's restart, then decisions.
- **A failed commit stops the engine** (section 5).
- **A worker's service** (section 6): no store; agents as child
  processes, "with an environment holding no credential, in a process
  group of its own"; smith's payloads "decoded and encoded in the
  routing, ... the charter against the endpoints of the worker's agent
  configuration ..., and smith's facts decoded into jig's activity"; the
  link dialled at once.
- **smith's payloads by hand-out** (`domain/hosts.md`, 6.8): the root
  "hands each payload out to its protocol layer, whose routing decodes or
  encodes it with smith's codecs (`smith-protocol-channel`'s functions,
  and `smith-channel`'s records for a call's name and a fact), keeping no
  state, and hands it back as an event of the root, in the order it went
  out"; one that does not decode "fails the start as invalid".
- **One process** (section 10): "One service, one loop", each layer with
  its own root and journal, "nothing crosses between them but bytes".
- **The channel's protocol world** (`channel.md`, section 12): both sides
  "as the reference service composes them", smith's agent processes over
  skein's fake LLM beneath the test worker, "joined by in-memory streams
  cut and rejoined at random"; the hub's world's stories through bytes,
  and its own list; its referee: "every turn and answer reaches the
  engine once and is acknowledged once, every relayed call is answered
  once, every message sent ends once (read, refused or unread), no
  attempt is placed while a worker may still run the last, and no
  credential grant's value appears in any domain's state."

## Choices this plan makes

- **Two layers and the service that owns them:** `Engine` (the test root
  and the engine's protocol layer) and `Worker` (the test worker's root
  and its protocol layer), each with its `iterate` over io's vocabulary
  and a token range of its own; `Service` owns io, its kernel's queues,
  the signal source, the engine's layer if the configuration names an
  engine, and a worker's layer for each worker it names. A document names
  an engine, workers, or both: `tests/channel` runs one of each as two
  processes; jig 26 runs both in one.
- **Startup in two calls** (`startup.rs`): `named(document)` says which
  files the caller must read (workers' digests, a worker's secret,
  handed-in accounts' files, the application's credential, charters'
  instructions, the worker's agent configuration); `startup(document,
  files, agent, seeds)` takes the document, those files' bytes, the
  worker's agent profile, and the drawn seeds, and returns the `Service`
  or the problems, each naming its path. The worlds hand bytes; jig 26's
  `main` reads them with skein's startup kit.
- **The worker's agent profile is the caller's to parse:** smith's agent
  configuration is JSON that `smith_agent_shell::config::parse` reads,
  which step code cannot run; the caller hands `startup` the channel's
  endpoints (`smith_protocol_channel::Endpoints`), the hello's agent
  bounds (jig 05's) and smith's host half's limits.
- **The test application's members** (its part of the document): the
  two connectors' configurations, the system's starting values, the
  application's one credential's file, and, for a worker, the agent
  program, its arguments and the agent's root directory.
- **Limits come from jig 20's profiles** (`jig_protocol_config::profile`),
  never from constants of the service's: each component's numbers are
  there, added by the increment that makes the component (jig 20's rule:
  a session that changes a `Limits` "changes `src/profile.rs`'s constants
  in the same increment"). The store's (jig 13 to 17), skein's accounts
  driver's (jig 21) and the clients' (jig 31.1) are named in the Needs of
  the increments that take them; where none has been added when such an
  increment starts, that increment adds them to the profiles.
- **The store's port, transitional** (README, "Increments that need
  more"): until 27.3, the root's commits, restart loads and core loads
  leave through `Engine::store_requests` and their answers come in
  through `Engine::store_answers`, which the world answers with
  `jig-fake-store`. 27.3 replaces the port with `jig-protocol-store` and
  removes it. Default: the port. If jig 18 is on `main` when 25.3
  starts, 25.3 may wire `jig-protocol-store` at once and leave the port
  out; 27.3 then keeps only its world changes.
- **The parties' port:** with no clients' member in the configuration,
  the root's outputs for parties leave through `Engine::parties_out` and
  parties' events come in through `Engine::parties_in`; `tests/channel`
  answers them with `jig-fake-parties`, translated as
  `tests/core/src/peers.rs` translates them. jig 33 adds the clients'
  side for a configuration that names one.
- **The test connector's system** (decision 10): `system.rs`, step code,
  in memory: the fault-free path of `jig-test-system`'s `answer`
  (`testing/jig-test-system/src/lib.rs:200-245`), its values in a
  `skein_lib::Map`. Faults stay the worlds'.
- **The test worker's agents** (decision 11): smith's real agent
  processes. The worker's layer spawns the program its member names with
  `smith_host_protocol::Process`; in `tests/channel`, skein's world hosts
  that program as smith's agent (`smith_agent_process_world::configured`)
  over skein's fake LLM peer, a process of the world on loopback.
- **The test workspace** prepares nothing and saves nothing: a chat's run
  has no workspace items. It answers `Prepare`, `Abort`, `Save`,
  `Release` and `Deliver` in memory, `Deliver` with the title and body as
  its evidence.
- **The drain is the engine's layer's** (27.2), as `service.md`,
  section 5 orders it; what one process adds after it (the workers ended
  at once) is jig 26's.
- **The link's translation is the test root's, once** (`link.rs`, 25.2),
  as `domain/root.md`, section 5 has it.
- **Cuts are the simulator's:** the two processes meet over simulated
  loopback sockets, whose resets and short sends cut the link mid-frame
  (`skein_sim::Faults`); kills are `World::cut` on the engine's process.
  A silent partition and a peer that stops reading are a relay process of
  the world's between the two (`tests/channel/src/relay.rs`), which goes
  quiet or stops reading for a span.

## Rules that bind it

- "**Step code** follows skein's `programming-model.md`, section 10, and
  jig's `AGENTS.md`: no traits, no generic items, no `dyn`, no closures.
  `jig-test-service` and `jig-test-worker` are step code so that they
  can be copied" (`code-organization.md`, section 2).
- "**`jig-test-worker` and `jig-test-service`** may link smith's
  `smith-host-domain`, `smith-host-protocol`, `smith-protocol-channel` and
  `smith-channel`." No other smith crate in their `[dependencies]`; the
  channel's components link none (`code-organization.md`, section 3).
- "No increment of this plan ... adds an `unreachable!` arm for a variant
  the reference service can reach; an arm the service reaches is
  routed." (`code-organization.md`, section 6) Each root output the
  service routes is matched without a wildcard.
- **`tests/channel` aims for at most 0.8 s focused and 4 s fuzzy,**
  serially, each focused test under 0.5 s and each fuzzy test about 5 s.
  Missed: profile with the `measure` profile and make the tests cheaper
  before landing; past twice those figures, a question (development.md,
  5.3).

## Increments

### 25.1 `jig-test-worker`: the test worker's root

Scope: `testing/jig-test-worker` (step code): `Domain` composing the hub
(`jig-domain-host`), smith's host domain (`smith-host-domain`) and the
test workspace (`workspace.rs`); `step`, `fire`, `next_deadline`,
`reclaim`, `max_out`, `worst_case`; the hub's `ToAgent` translated to
smith's `parent::Event` and smith's `parent::Request` to the hub's
`FromAgent` (`translate.rs`, exhaustive both ways); smith's payloads by
hand-out (`hand_out.rs`): a start's charter bytes, transcript turns and
committed calls' names handed out to be decoded, smith's `Spawn` sent
once they come back typed, an undecodable one failing the start as
invalid naming which (`domain/hosts.md`, 6.8); each of smith's turns,
calls' names and results handed out to be encoded before the hub takes
them; each of smith's channel facts (`parent::Request::Told`) handed out
to be decoded into jig's typed activity (jig 05's). The hub's link
vocabulary passes through as the root's events and requests; smith's
`process::Request` and `process::Event` pass through for the protocol
layer.
Needs: nothing beyond the session's start.
Files: `testing/jig-test-worker/{Cargo.toml, src/{lib.rs, boundary.rs,
domain.rs, translate.rs, hand_out.rs, workspace.rs, limits.rs, tests.rs,
tests/fixtures.rs}}`; the workspace `Cargo.toml`;
`tests/boundary/src/tests.rs`. Shared hotspots: the workspace
`Cargo.toml`, `tests/boundary/src/tests.rs`.
Shape: as the interface sketch.
Test shape: step tests in `src/tests.rs`, fixtures in
`src/tests/fixtures.rs`: `limits()`, `RUN`, `AGENT`, `charter_bytes()`
(`smith_agent_process_world::charter()`), `charter_value()` (those bytes
decoded with `smith_protocol_channel::decode_charter` against
`smith_agent_process_world::configuration().channel_endpoints`, a
dev-dependency), `assignment(charter)`, and `Worker` driving `step`.
Tests: `a_start_waits_for_its_charter_to_come_back_decoded`,
`an_undecodable_charter_fails_the_start_as_invalid`,
`a_turn_reaches_the_hub_only_after_it_is_encoded`,
`a_result_is_handed_out_once_and_reaches_the_hub_as_bytes`,
`a_channel_fact_reaches_the_hub_as_typed_activity`,
`a_deliver_is_answered_with_its_title_and_body`. Under 20 ms each.
Reference test, in full:

```rust
#[test]
fn a_start_waits_for_its_charter_to_come_back_decoded() {
    let mut worker = fixtures::Worker::new(&fixtures::limits());
    worker.assign(fixtures::RUN, fixtures::assignment(fixtures::charter_bytes()));
    assert_eq!(worker.handed_out(), Some(fixtures::decode_charter(fixtures::AGENT, fixtures::charter_bytes())));
    assert_eq!(worker.spawns(), 0);
    worker.decoded_charter(fixtures::AGENT, fixtures::charter_value());
    assert_eq!(worker.spawns(), 1);
}
```

Proof: unit, `heavy cargo nextest run -p jig-test-worker`, run once: a
start waits for its charter to come back decoded, an undecodable charter
fails it as invalid, and a turn, a result and a channel fact reach the
hub only as the protocol layer hands them back (the six tests above);
under 0.2 s. The joint layer opens at 25.6.
Don't: link `smith-protocol-channel` or `smith-channel` outside
`[dev-dependencies]`; encode or decode a payload in the root.
Size: at most 600 lines.
Removes: nothing.

### 25.2 The test root routes a worker's link

Scope: `jig-test-domain`'s link family (`link.rs`): a worker's hello
(jig 05's hello as one value, `host::Hello`; jig 23's and 24's
sketches write it `host::LinkHello`: use the name jig 05 lands), its
turns, calls, answers, activity and notices (`host::LinkUp`), and its
loss, with the channel's token,
translated to the core's fleet events; and the fleet's outputs for a
channel (assignments, relayed answers typed as text, error or busy,
messages, kept and busy turns, acknowledgements, cancels, grants)
translated back to `host::LinkDown`, exhaustive both ways, as
`domain/root.md`, section 5 has it; `Event::Link { channel, link }`
and `Delivery::Link { channel, down }`; the assignment keeps its typed
`RunCharter` for the routing to encode, and its grants name their
tables (jig 08). A run's result, as jig 10 has `ops`'s root do (10.5):
the root keeps the hub's answer with its result's bytes until the fleet
has fenced it; at `Now::AnswerPayload` an ended answer's result leaves
as `Request::ReadResult { payload, outcome }` and comes back as
`Event::ResultRead { payload, end }`, which the root steps into the
core's `Event::AnswerPayload`. Its scripted workers' events stay, for
the domain worlds.
Needs: nothing beyond the session's start.
Files: `testing/jig-test-domain/src/{lib.rs, routes.rs, link.rs,
tests/link.rs, tests/fixtures.rs}`; the worlds' exhaustive matches of
the root's `Delivery` and `Request`, each gaining the new arms (at most
20 lines each): `tests/core/src/{effects.rs, world.rs, observations.rs,
watches.rs, peers.rs, client_peer.rs, referee/offers.rs}`,
`tests/conformance/src/{lib.rs, negative.rs}`,
`tests/client/src/joint/session.rs`. Shared hotspots:
`testing/jig-test-domain/src/{lib.rs, routes.rs}` (both over 800 lines:
an arm this increment adds goes in `link.rs`, and neither file grows by
more than 20 lines), `tests/core/src/effects.rs`,
`tests/conformance/src/lib.rs`.
Shape: `link::hello(channel: Token, hello: host::Hello) -> core::Event`;
`link::up(channel: Token, up: host::LinkUp) -> core::Event`;
`link::lost(channel: Token) -> core::Event`;
`link::down(request: fleet::Request) -> Option<(Token, host::LinkDown)>`;
names as jig 05 and jig 22 to 24 use them.
Test shape: step tests in `src/tests/link.rs`:
`a_worker_s_hello_reaches_the_fleet_with_its_bounds`,
`a_turn_from_a_link_reaches_the_fleet_with_its_refusals`,
`an_assignment_for_a_channel_leaves_as_the_hub_s_record`,
`a_lost_link_reaches_the_fleet_as_its_channel_lost`. Under 20 ms each.
Fixtures in `src/tests/fixtures.rs` (new; `CHANNEL`, `hello()`,
`turn()`). Reference test, in full:

```rust
#[test]
fn a_lost_link_reaches_the_fleet_as_its_channel_lost() {
    let event = link::lost(fixtures::CHANNEL);
    let core::Event::Fleet(fleet::Event::Lost { channel }) = event else {
        panic!("a lost link reaches the fleet");
    };
    assert_eq!(channel, fixtures::CHANNEL);
}
```

Proof: unit, `heavy cargo nextest run -p jig-test-domain -E
'test(/link::/)'`, run once: a worker's hello, its turns with their
refusals and its loss reach the fleet, and an assignment for a channel
leaves as the hub's record (the four tests above); under 0.1 s. The
joint layer opens at 25.6.
Don't: decode a payload in the root.
Size: at most 350 lines.
Removes: nothing.

### 25.3 `jig-test-service`: startup, the engine's layer, its ports and its system

Scope: `testing/jig-test-service` (step code): `startup.rs` (`named`,
`startup`: jig's engine part parsed with jig 20's parser, the test
application's members, the problems collected with their paths, the
worst case checked against the stated memory); `engine.rs` (the `Engine`
layer: the test root, its journal, its timers; `iterate`'s up and down
passes, `MAX_OUT` reserved before each call; one decision per root
event; a store's answer always taken); `route.rs` (each root output to
its destination, exhaustive); the store's port and the parties' port;
`system.rs` (decision 10); `stop.rs` (a failed commit: nothing released
after it, `End::Stopped`); `lib.rs` (`Service`, `iterate`, and the
inherent `Host` entry points: `completions`, `submissions`,
`work_pending`, `next_deadline`, `next_policy_deadline`, `is_empty`,
`worst_case`, `operations`). Workers and the components 25.4 and 25.5
bring are refused at startup, naming their paths, until those
increments.
Needs: nothing beyond the session's start: jig 20's profiles and its
engine and worker parts are at the session's start.
Files: `testing/jig-test-service/{Cargo.toml, src/{lib.rs, boundary.rs,
startup.rs, engine.rs, route.rs, system.rs, stop.rs, limits.rs, tests.rs,
tests/fixtures.rs, tests/engine.rs}}`; the workspace `Cargo.toml`;
`tests/boundary/src/tests.rs`. Shared hotspots: the workspace
`Cargo.toml`, `tests/boundary/src/tests.rs`.
Shape: as the interface sketch.
Test shape: step tests, fixtures in `src/tests/fixtures.rs`:
`engine_document()`, `files()`, `seeds()`, `Driven` (a service with its
ports answered from queues). Tests:
`a_document_with_an_unknown_member_is_refused_naming_its_path`,
`a_named_file_missing_from_those_handed_is_refused`,
`a_worst_case_past_the_memory_is_refused_saying_by_how_much`,
`the_restart_asks_one_load_at_a_time`,
`a_store_answer_is_taken_with_the_journal_full`,
`a_commit_leaves_in_the_iteration_that_made_it`,
`a_failed_commit_releases_nothing_after_it`,
`the_system_applies_an_effect_and_reads_it_back`. Under 20 ms each.
Reference test, in full:

```rust
#[test]
fn a_failed_commit_releases_nothing_after_it() {
    let mut driven = fixtures::Driven::started(&fixtures::engine_document());
    driven.party(fixtures::start_chat(fixtures::OWNER, b"one"));
    let first = driven.commit_asked();
    driven.party(fixtures::start_chat(fixtures::OWNER, b"two"));
    let second = driven.commit_asked();
    driven.fail(first);
    driven.commit(second);
    assert_eq!(driven.released_to_parties(), 0);
    assert_eq!(driven.end(), Some(End::Stopped(Stop::CommitFailed { number: first })));
}
```

Proof: unit, `heavy cargo nextest run -p jig-test-service`, run once: a
document's mistakes are refused naming their paths, a failed commit
releases nothing after it, and the system applies an effect and reads it
back (the eight tests above); under 0.2 s. The joint layer opens at
25.6.
Don't: parse JSON outside jig 20's parser and the application's own
members; read a file.
Size: at most 600 lines.
Removes: nothing.

### 25.4 The engine's components

Scope: the accounts (`accounts.rs`): skein's driver
(`skein_oauth_accounts::Component`) as the engine's child, jig 21's
translation and table between it and the root's accounts' requests, each
handed-in account's record (from the file the caller read) handed in at
start; the host tools (`tools.rs`): each call the core hands out
(`Now::Call`) decoded with jig 10's `jig-protocol-tools` into the root's
events, and each answer rendered by it into the relayed answer's text;
a call's key given by `jig_protocol_charter::call_key` as the call is
decoded, as jig 10.3's stand-in (`tests/core/src/protocol.rs`) does it,
here in step code; a run's read answered to
the test root as `CallAnswered` (jig 10's name for today's `ReadAnswer`,
`testing/jig-test-domain/src/lib.rs:230-234`); the charter encoded by
`jig-protocol-charter` as an assignment leaves; the run's identity (`channel.md`, section 7: the first 16 bytes
of SHA-256 over the deployment's id, the task's number and its
conversation's first attempt); the workers' side (`workers.rs`): jig
23's component, its layer's part played here: io's listener on the
configured address, opened in the start's order, each accepted socket
handed to the component (`Event::Accepted`) and bound or rejected as it
answers (`Take`, `TurnAway`), each socket's stream records carried to
and from its `skein_channel` lower records; its `Hello`, `Up` and `Lost`
to the root's link family (25.2) and the root's link outputs to it as
`Event::Down`, with one value per grant the record names, from the
accounts' table (jig 21's `Table::value`) or the application's
(`values.rs`), never through the root; an LLM account's value as its
two parts, the token and the account's id, as the table holds them
(llm.md, 5.3); the component aborted on a
failed commit. Results: the test root's `Request::ReadResult` read with
`jig_protocol_charter::result` and answered `Event::ResultRead` with
`tasks::End::Finished`, or `Failed(Class::Invalid)` when it does not
decode, as jig 10.5's world does for `ops`.
Needs: nothing beyond 25.3: jig 10's `jig-protocol-tools` (decoding and
rendering) and `jig_protocol_charter::{call_key, result}`, the test
root's `CallAnswered`, jig 21's `Table` and handed-in path, and jig 23's
component are at the session's start; skein's accounts driver's limits
in jig 20's profiles (added here if jig 21 has not).
Files: `testing/jig-test-service/src/{accounts.rs, tools.rs, workers.rs,
values.rs, route.rs, startup.rs, engine.rs, limits.rs, tests/engine.rs,
tests/fixtures.rs}`. Shared hotspots: `testing/jig-test-service/src/route.rs`.
Shape: `values::Values { accounts: jig_protocol_accounts::Table,
application: skein_lib::Map<u32, Secret> }`; `tools::decode(call:
&fleet::Call) -> core::Event` and `tools::render(..)`, total.
Test shape: `a_handed_in_account_is_lent_and_its_value_stays_out_of_the_root`,
`a_call_is_decoded_and_one_that_does_not_is_answered_an_error`,
`an_assignment_leaves_with_its_charter_encoded_and_grants_filled`,
`a_grant_whose_generation_the_table_lacks_is_left_out`,
`the_run_s_identity_is_shared_by_a_conversation_s_attempts`,
`the_workers_listener_opens_before_the_restart`,
`a_result_that_does_not_decode_fails_its_attempt_as_invalid`. Under 20
ms each.
Proof: unit, `heavy cargo nextest run -p jig-test-service`, run once: a
handed-in account is lent with its value kept out of the root, a call is
decoded or answered an error, an assignment leaves with its charter
encoded and its grants filled, and an undecodable result fails its
attempt as invalid (the seven tests above); under 0.2 s. The joint layer
opens at 25.6.
Don't: hand a credential's value to the root or any domain.
Size: at most 600 lines.
Removes: nothing.

### 25.5 The worker's layer, and layers routed by token

Scope: `worker.rs` and `worker_route.rs`: the `Worker` layer: its root
(25.1) with its hello's parts (its slots, its stop bound from jig 20's
worker part, its agent's bounds from the caller's profile); jig 24's link,
its layer's part played here (io's `Connect` to the engine's address on
`Request::Dial`, answered `Dialled` or `Unreachable`, the socket's stream
records to and from its lower records), its records to the root and
back; smith's host half, one `smith_host_protocol::Process` per agent the
root asks to spawn, launched with the member's program and arguments, an
environment with no credential, the agent's root directory; each of the
root's hand-outs answered in the same iterate (`smith_protocol_channel::{decode_charter,
decode_transcript, encode_turn, encode_result}` and `smith_channel::{CallName,
Fact}` against the caller's endpoints); each grant's value jig 24's link
hands out as it decodes a record kept in the worker layer's own
`jig_protocol_accounts::Table` (jig 21), and smith's host half's start
and grants filled from that table (`Values` for `Process::send_start`,
the value for `Process::send_grant`), an LLM account's value put in
smith's form as it is handed over: the account's id's length in two
bytes, big-endian, then the id and the token (llm.md, 5.3, "Put in its
taker's form where it is handed over"); `stop`, the hub cancelling
each run, the link stopped once the hub has given up what it keeps. The
`Service`: a worker's layer per worker member, token ranges disjoint;
`iterate` routes io's events to the layer whose tokens they carry; in
one process, the workers' layers started once the engine's listeners are
open. 25.3's refusal of workers removed.
Needs: nothing beyond 25.4.
Files: `testing/jig-test-service/src/{worker.rs, worker_route.rs,
lib.rs, startup.rs, limits.rs, tests/worker.rs, tests/fixtures.rs}`.
Shared hotspots: `testing/jig-test-service/src/lib.rs`.
Shape: as the interface sketch.
Test shape: `io_events_reach_only_the_layer_whose_tokens_they_carry`,
`a_decoded_charter_comes_back_in_the_iterate_that_asked`,
`a_turn_is_encoded_against_the_worker_s_endpoints`,
`a_grant_s_value_reaches_smith_s_host_half_in_its_envelope`,
`the_agent_starts_with_no_credential_in_its_environment`,
`stopped_the_worker_cancels_its_runs_and_never_redials`. Under 20 ms
each.
Proof: unit, `heavy cargo nextest run -p jig-test-service`, run once:
io's events reach only the layer whose tokens they carry, a grant's
value reaches smith's host half in its envelope, and the agent starts
with no credential in its environment (the six tests above); under 0.2
s. The joint layer is 25.6, next.
Don't: keep a payload between iterations; run a second agent per run.
Size: at most 600 lines.
Removes: 25.3's startup refusal of workers.

### 25.6 The channel's protocol world, and its first story

Scope: `tests/channel` (`jig-channel-world`) on skein's simulated world
(`skein_world::World`, `Config::calm` here): the engine's service as a
restartable process (`spawn_restartable`), made from an engine document;
a worker's service as a process, made from a worker document, dialling
the engine on simulated loopback; skein's fake LLM peer as a process,
scripted with `smith_agent_process_world::fake::configured`; smith's
agent hosted as the worker's agent program (`World::host` with a
`HostedProgram` whose `make` is `smith_agent_process_world::configured`,
as `smith/tests/hosts/src/world.rs:217-222` does); the store's port
answered by `jig-fake-store`, held by the world across the engine's
restarts; the parties' port answered by `jig-fake-parties`; the world's
observations (turns committed, answers acknowledged, calls answered,
messages ended) taken from the service's ports and the fake store. One
story.
Needs: nothing beyond 25.5.
Files: `tests/channel/{Cargo.toml, clippy.toml, src/{lib.rs, world.rs,
processes.rs, fixtures.rs}, tests/channel.rs}`; the workspace
`Cargo.toml`. Shared hotspots: the workspace `Cargo.toml`. (jig 23's and
24's machine worlds are `tests/engine-workers` and `tests/worker-link`;
this world is new.)
Shape: `world::run(settings: Settings, script: Script) -> Outcome`;
`Outcome { verdict, turns_committed, answers_acknowledged,
calls_answered, messages_ended }`.
Test shape: `tests/channel.rs`, in full:

```rust
use jig_channel_world::{fixtures, world};
use skein_world::domain::Verdict;

#[test]
fn a_person_s_chat_runs_on_the_test_worker_and_is_acknowledged_once() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), fixtures::one_chat(b"Hello"));
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.turns_committed, 1);
    assert_eq!(outcome.answers_acknowledged, 1);
}
```

Ceiling: under 0.5 s.
Proof: joint:
`a_person_s_chat_runs_on_the_test_worker_and_is_acknowledged_once`, run
once with `heavy cargo nextest run -p jig-channel-world --test channel`,
passes, which shows a person's chat crosses the engine, the workers'
channel, the worker, smith's host half and smith's real agent over the
fake LLM, and back, once; under 0.5 s. The process layer is jig 26's.
Don't: script an agent: the agent is smith's.
Size: world code only.
Removes: nothing.

## Interface sketches

```rust
// jig_test_worker::boundary: the test worker's root, between its protocol layer and its children.
use jig_domain_host as host;
use skein_lib::Token;
use smith_host_domain as smith;

/// Into the root.
pub enum Event {
    /// What the link heard from the engine, in the hub's link vocabulary (jig 05, jig 24).
    Link(host::LinkDown),
    /// jig 24's link: open, answer with the hello; lost; past its grace; a retained
    /// turn to hand back for a busy resend; a relay the hub cancelled has ended.
    Ready,
    Lost,
    GracePassed,
    Resend { run: Token, attempt: Token, turn: u32 },
    RelayCancelled { call: Token },
    /// smith's agent process events, as smith's host half hands them up.
    Process(smith::process::Event),
    /// A hand-out answered by the routing, in the order it went out.
    Decoded { agent: Token, start: Result<smith::Start, Invalid> },
    Encoded { owner: Token, encoded: Encoded },
    Activity { owner: Token, activity: Option<host::Activity> },
    /// The test workspace's terminals are its own: none cross here.
    Timer,
}
/// Out of the root.
pub enum Request {
    /// To the link, in the hub's link vocabulary: the hello once up, then its records,
    /// a relay's cancel, and the stop once the hub has given up what it keeps.
    Hello(host::Hello),
    Link(host::LinkUp),
    CancelRelay { call: Token },
    Stop,
    /// To smith's host half: spawn, send, read, wait, reap.
    Process(smith::process::Request),
    /// smith's payloads to decode against the worker's endpoints; one answer each.
    Decode { agent: Token, charter: Box<[u8]>, transcript: Box<[Box<[u8]>]>, answered: Box<[host::AnsweredCall]> },
    /// smith's values to encode for the hub; one answer each.
    Encode { owner: Token, value: Encodable },
    /// A smith channel fact to decode into jig's activity; best effort.
    Fact { owner: Token, body: Box<[u8]> },
}
pub enum Encodable { Turn(smith::Turn), CallName(smith::CallName), Result(smith::RunResult) }
pub enum Encoded { Turn(host::Turn), CallName(Box<[u8]>), Result(Box<[u8]>), Undecodable }
/// Which of a start's payloads did not decode.
pub enum Invalid { Charter, Transcript, CallName }
```

```rust
// jig_test_service: the service, its layers, its startup and its ports.
use skein_io::kernel::{Complete, Fd, Submit};
use skein_lib::{List, Queue, Secret, Time, Wall};

/// What the caller hands `startup`: the files `named` named, as read.
pub struct Files { pub entries: List<(Box<[u8]>, Secret)> }
/// smith's agent configuration, parsed by the caller (ordinary Rust).
pub struct Agent { pub endpoints: smith_protocol_channel::Endpoints, pub limits: smith_host_protocol::Limits, pub bounds: jig_domain_fleet::Bounds }
pub struct Seeds { pub deployment: [u8; 16], pub seed: u64, pub crypto: Secret }
/// The files a document names, for the caller to read.
pub fn named(document: &[u8]) -> Result<List<Box<[u8]>>, Problems> { todo!() }
/// The service, holding the engine's layer if the document names one, and a worker's per worker.
pub fn startup(document: &[u8], files: &Files, agent: Option<&Agent>, seeds: &Seeds) -> Result<Service, Problems> { todo!() }
/// One iteration: io's completions to the layer whose tokens they carry, each layer's
/// iterate, then each layer's requests to io (service.md, sections 4 and 10).
pub fn iterate(service: &mut Service, now: Time, wall: Wall) { todo!() }

pub struct Service { /* io, its queues, the signal source, Option<Engine>, List<Worker> */ }
impl Service {
    pub fn adopt_signals(&mut self, descriptor: Fd) -> Result<(), Fd> { todo!() }
    pub fn completions(&mut self) -> &mut Queue<Complete> { todo!() }
    pub fn submissions(&mut self) -> &mut Queue<Submit> { todo!() }
    pub fn work_pending(&self, now: Time) -> bool { todo!() }
    pub fn next_deadline(&self) -> Option<Time> { todo!() }
    /// The earliest deadline of each root and component, io's own excepted (service.md, 4).
    pub fn next_policy_deadline(&self) -> Option<Time> { todo!() }
    pub fn is_empty(&self) -> bool { todo!() }
    pub fn engine(&mut self) -> Option<&mut Engine> { todo!() }
    pub fn end(&self) -> Option<End> { todo!() }
    /// A signal or an operator: the first drains the engine's layer (27.2), a second aborts.
    pub fn shutdown(&mut self) { todo!() }
}
pub struct Engine { /* the test root, its journal, components, tables, timers, ports */ }
impl Engine {
    /// The store's port, until 27.3.
    pub fn store_requests(&mut self) -> &mut Queue<StoreRequest> { todo!() }
    pub fn store_answers(&mut self) -> &mut Queue<StoreAnswer> { todo!() }
    /// The parties' port, for a configuration with no clients' member.
    pub fn parties_in(&mut self) -> &mut Queue<jig_domain_core::Event> { todo!() }
    pub fn parties_out(&mut self) -> &mut Queue<PartyOutput> { todo!() }
}
pub struct Worker { /* the test worker's root, its link, its grant values' table (jig 21's), its processes */ }
impl Worker {
    /// The hub cancels each run for shutdown; the link stops once the hub has given up what it keeps.
    pub fn stop(&mut self) { todo!() }
}
/// How the engine's layer ended (27.2).
pub enum End { Drained, Aborted, Stopped(Stop) }
pub enum Stop { CommitFailed { number: u64 }, Start(Problems), Listener }
pub struct Problems { /* each problem with its path in the document */ }
pub enum StoreRequest { /* the test root's Commit, Restart and Load requests, unchanged */ }
pub enum StoreAnswer { /* Committed, Failed, Restored rows, pages, as the root's events take them */ }
pub enum PartyOutput { /* the root's held and immediate outputs for parties */ }
```

- **Who owns what:** the test root decides and translates; the engine's
  layer routes, decodes calls, reads results, encodes charters and moves
  values from tables to the workers' component; the test worker's root
  hosts and keeps, and hands smith's payloads out; the worker's layer
  decodes and encodes them and holds grant values only in its own
  `jig_protocol_accounts::Table` and smith's host half; the service owns io and routes by token,
  deciding nothing; the caller owns the kernel, the clock, the signals'
  descriptor, the files and what answers the ports.
- **What ends what:** a failed commit ends the engine's layer, stopped;
  a drain ends it once the core says `Held::Drained` and each component
  has said closed, the store last; a second shutdown aborts it; a
  worker's layer ends once its link has closed after `stop`.

## Questions to raise when reached

- **Which bytes an LLM grant's value is** (25.4, 25.5): settled by the
  designer (jig `87195d80`; llm.md, 5.3; channel.md, 8). The engine's
  table and jig's channel keep the token and the account's id apart,
  both secret; the worker's layer puts them in smith's form as it hands
  them to smith's host half: the id's length in two bytes, big-endian,
  then the id and the token, as smith's agent reads it
  (`smith/crates/smith-agent-service/src/service.rs:846-858`). The fake
  LLM's id is `acc` (`smith/tests/agent-process/src/fake.rs:41`).
- **A hub story that cannot pass through bytes** as the hub's world tells
  it: report it with the design text it rests on.

## Done when

- The two crates and the world pass: `heavy cargo nextest run -p
  jig-test-worker -p jig-test-service -p jig-test-domain -p
  jig-channel-world` passes.
- smith stays where it may: `git grep -n -E '^smith-' --
  crates/*/Cargo.toml` prints only `jig-protocol-charter`'s and
  `jig-protocol-transcript`'s lines.
- The gate passed on this session's last landing: `git -C ~/src/rust/jig
  log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step:.jig-25\.' origin/main | head -1` names
  `scripts/check.sh passed`.
