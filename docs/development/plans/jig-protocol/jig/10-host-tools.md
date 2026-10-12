# Session 10: host tools, a call's key and a run's result outside the root

A host tool's call is decoded, and its answer rendered, in the engine's
protocol layer, so no domain and no root parses a call's input. This
session builds `jig-protocol-tools`, the core's host tools (their
schemas, descriptions, decoders and renderers), and gives
`jig-protocol-charter` the readers of a call's key and a run's result.
The core already hands an admitted call out (`Now::Call`) and takes the
typed request back (`NamedAction`, `DelegateValidated`); this session
makes an input that does not decode answered at once as an error that
decides nothing, has the worlds' harnesses decode and render where the
protocol layer will (jig 25 for the test application), and moves `ops`'s
decoding of calls, call names and results out of its root, which the
audit found done in-step (`examples/ops/domain/src/assemble.rs`,
`translate.rs`).

## Read first

- jig's `protocol/channel.md`, 6.1 (a run's result "by the path of host
  tools' calls"), 6.3 and section 7 (a call's name and key);
  `domain/hosts.md`, section 8 and 6.8; `domain/engine.md`, 7.3 (the
  engine's tools); `domain/root.md`, sections 5, 7, 11 and 12;
  `protocol/README.md`, section 5 (`jig-protocol-tools` "gives its
  schemas as bytes the charter declares"; which crates link smith) and
  section 13 ("Host tools' calls decoded in the protocol layer");
  `protocol/service.md`, section 2 ("Calls decoded, answers rendered, in
  the routing").
- `decisions.md`, 1.
- `findings/domain.md`, item 9 and 6(h) (what `ops`'s root decodes, with
  file:line, and the boundary constraint on `jig-protocol-charter`).
- `findings/protocol.md`, 1.1 (`jig-charter`), 1.6 (`Now::Call`,
  `CallKey`) and 6.1 ("09-host-tools").
- smith's `crates/smith-channel/src/generated/v2.rs:466-495, 5872-5905`
  (`CallName`: 16 bytes, big-endian) and
  `crates/smith-protocol-channel/src/result.rs` (`decode_declared`).

## What the design asks

- **Decoded beside the tool's schema** (hosts.md, section 8): "Each host
  tool's schema and description are written once, with the function
  that decodes its calls and the one that renders its answers; a call's
  input is decoded into the core's or the connector's typed request, or
  answered as an error naming what is wrong."
- **After the fence** (channel.md, 6.3): "The core then hands the
  admitted call out for decoding ...: `jig-protocol-tools`' for the
  core's own tools, the application's for its connectors' reads and
  effects. The typed request goes back to the root; an input that does
  not decode is answered at once as an error naming what is wrong, and
  decides nothing." "An answer crosses as the engine's text for the
  LLM, with whether it is an error, rendered in the engine's protocol
  layer".
- **A call's key** (channel.md, section 7): call names are decoded "by
  `jig-charter` in the engine, which reads the call's key (its
  completion and position) as the protocol layer decodes the call
  (6.3), so a connector keys the call's effects by it."
- **A run's result** (channel.md, 6.1): "`jig-charter`'s reader decodes
  it in the engine's protocol layer ..., by the path of host tools'
  calls (6.3): the core hands an answer's result out once the fleet has
  fenced it, and the typed result comes back. A result that does not
  decode fails its attempt as invalid."
- **A root never** "read[s] or rewrite[s] a value it carries between
  children" (root.md, section 11); the reference root is `ops`'s
  (root.md, section 12).

## Choices this plan makes

- **The core's vocabulary stays as it is for calls:** `Now::Call` out;
  `NamedAction`, `DelegateValidated` and the application's typed
  events back; `Held::CallAnswer` and `Now::EffectAnswer` out as parts;
  `Event::SettledCall` back, rendered. Only the decoding and rendering
  move, to the worlds' harnesses now and to jig 25's routing later.
- **An input that does not decode takes the no-commit answer path**
  each root has for a run's read: the root stores the rendered error as
  its payload and steps `fleet::Event::Relayed { to, answer }`. The test
  root's `Event::ReadAnswer` is renamed `Event::CallAnswered` for it;
  `ops`'s root gains `Event::CallAnswered`. Nothing reaches the core's
  call table, so nothing is committed.
- **Dispatch by tool name in the routing:** `jig_protocol_tools::decode`
  answers `Decoded::NotMine` for a name not its own; the routing then
  asks the application's decoders; a name neither knows is answered as
  an input that does not decode.
- **`jig-protocol-tools` depends on** `skein-lib`, `skein-json`,
  `jig-domain-core`, `jig-domain-tasks` and `jig-domain-notes` (the core
  re-exports neither child's types its requests carry), and on no smith
  crate and no application's crate.
- **`jig-protocol-charter` keeps its own types:** its readers return
  `CallKey` and `RunResult` of its own, and the routing maps them to the
  core's (`core::CallKey`, `tasks::TaskResult`), as its `charter` takes
  its own `Charter`. The call key is read with smith-channel's
  `CallName` record, the result with smith-charter's `RunResult`
  (smith's own `decode_declared` checks).
- **A run's result in `ops`:** the root keeps the hub's answer with its
  result's bytes until the fleet has fenced it; at `Now::AnswerPayload`
  it hands an ended answer's result out (`Output::ReadResult`) and steps
  the core with the typed result its world hands back
  (`Event::ResultRead`); other endings it translates in its step, as
  typed values. The core drops an answer's payload for an attempt no
  longer current where it asserted, since the reading now comes back in
  a later decision.
- **The test application's results stay typed until jig 25:** its fake
  workers answer with typed endings, so its root reads no result; jig 25
  gives the test root the hub's answer and reads its result with
  `jig_protocol_charter::result`, as `ops`'s world does from 10.5. Its
  calls' keys and inputs are read in its worlds' harness from 10.3.
- **`ops`'s tools are declared in its configuration:**
  `jig_ops_domain::Config::tools`, made by its caller from
  `jig_protocol_tools::declarations()` and `ops`'s own; `ops`'s
  connectors' decoders and renderers move to `tests/ops/src/protocol/`,
  where its world stands in for `ops`'s protocol layer (no `ops`
  protocol crate: code-organization.md, section 2 names none).

## Rules that bind it

- No increment of this plan "decodes or encodes bytes in a domain crate:
  decoding is a protocol layer's, reached by hand-out where a domain
  needs a value typed" (code-organization.md, section 6).
- No increment "links smith from `jig-domain-host`, the core or its
  children, or the channel's components"; "Only `jig-protocol-charter`
  and `jig-protocol-transcript` link smith" (code-organization.md,
  sections 3 and 6). `tests/boundary` gains `jig-protocol-tools`' rule.
- "Transitional code has an end": `ops`'s copies of the decoders that
  10.2 writes into `jig-protocol-tools` live until 10.4, which removes
  them.

## Increments

### 10.1 A call's key and a run's result, read by `jig-protocol-charter`

Scope: channel.md, section 7 and 6.1. `jig_protocol_charter::call_key`
reads smith's call name; `jig_protocol_charter::result` reads a run's
result in its charter's version, into the forms the core judges, or
says it does not decode.
Needs: nothing.
Files: `crates/jig-protocol-charter/src/{call.rs, result.rs}` (new),
`lib.rs` (lines 15-20, the exports), `Cargo.toml` (`smith-channel`);
the workspace `Cargo.toml` (`smith-channel`); `tests.rs`. Users of the
crate's public surface the audit counted: examples/ops/domain
(assemble.rs, domain.rs:51-67, limits.rs:7, 27), tests/ops/src/conformance/config.rs:93,
examples/ops/domain/src/tests.rs:50, crates/jig-charter/src/tests.rs:11:
none changes here.
Shape: see the sketch. `call_key` decodes with
`smith_channel::CallName::decode(&smith_channel::CEILINGS, ..)`, with
no trailing bytes; `result` decodes with `smith_charter::RunResult` under
`smith_charter::v2::CEILINGS`, refuses trailing bytes, a change form and
a verdict with items, as `ops`'s `decode_result` does
(assemble.rs:260-284).
Test shape: `crates/jig-protocol-charter/src/tests.rs`, unit, under 20
ms each: `a_call_key_reads_completion_and_position`,
`a_call_name_with_trailing_bytes_has_no_key`,
`a_report_result_reads_its_words`,
`a_verdict_result_reads_its_label_and_fields`,
`a_change_result_does_not_decode`,
`a_truncated_result_does_not_decode`. Reference:

```rust
#[test]
fn a_call_key_reads_completion_and_position() {
    let name = smith_channel::CallName::new(
        &smith_channel::CEILINGS,
        smith_channel::CallNameParts { activation: 7, completion: 3, position: 2 },
    )
    .expect("a call name within its ceilings");
    let mut writer = skein_lib::Writer::new(16);
    name.encode(&mut writer).expect("sixteen bytes");
    assert_eq!(
        crate::call_key(&writer.finish()),
        Some(crate::CallKey { activation: 7, completion: 3, position: 2 })
    );
}
```

Proof: unit: `heavy cargo nextest run -p jig-protocol-charter` shows names
and results smith's encoders made read back, and malformed ones
refused; this layer because no root hands a key or a result out
before 10.3; once, under 1 s.
Don't: depend on a `jig-domain-*` crate (`tests/boundary`'s rule for the
crate stays as jig 01 leaves it); read the call's activation as the
core's attempt (the routing checks it against the attempt the fleet
fenced).
Size: at most 120 lines of production code.
Removes: nothing.

### 10.2 `jig-protocol-tools`: the core's host tools

Scope: hosts.md, section 8; channel.md, 6.3; engine.md, 7.3's table.
New crate `crates/jig-protocol-tools` (step code): each of the core's
tools (`delegate`, `message`, `cancel`, `release`, `introduce`,
`decide`, `withdraw`, `subscribe`, `unsubscribe`, `amend`,
`decide_escalation`, `propose`, `note`, `recall`) with its JSON schema,
its description and whether it writes; its decoder into
`core::NamedAction` or a delegate batch, or a `Problem` naming the
field and what is wrong; the renderer of each `core::CallPart` the core
gives and of a problem, as text for the LLM with whether it is an error;
`batch`, the delegate batch's decoder, for a configuration's templates.
The decoders are moved from `ops`'s `translate.rs` (`json` 281-504,
`nested` 506-825, `notes` 1026-1142, `decode`'s core arms 877-1011) and
the renderers from `assemble.rs` (`render`, `delegated`, `notes`,
286-416), rewritten under the subset; `ops` keeps its copies until 10.4.
Needs: nothing.
Files: `crates/jig-protocol-tools/**` (new: `lib.rs`, `declarations.rs`,
`json.rs`, `decode.rs`, `render.rs`, `limits.rs`, `tests.rs`,
`tests/fixtures.rs`); the workspace `Cargo.toml` (member and
dependency); `tests/boundary/src/tools.rs` (new, beside `tests.rs`,
which is 1,009 lines: the rule that `jig-protocol-tools` depends on no
smith crate and no application's crate), `tests/boundary/src/lib.rs`.
Shared hotspots: the workspace `Cargo.toml`, `tests/boundary` (jig 01
writes its rules).
Shape: see the sketch.
Test shape: `crates/jig-protocol-tools/src/tests.rs`, unit, under 20 ms
each, with fixtures in `src/tests/fixtures.rs` (one valid input per
tool, as the LLM writes JSON): `each_schema_parses_and_names_its_decoders_fields`,
`a_message_call_decodes_to_its_named_action`,
`a_delegate_call_decodes_its_batch_in_order`,
`a_call_missing_a_required_field_names_it`,
`a_call_that_is_not_json_is_malformed`,
`an_input_past_its_bound_is_too_large`,
`another_applications_tool_is_not_mine`,
`a_delegated_part_renders_its_task_numbers`,
`a_denied_part_renders_as_an_error_naming_what_it_lacked`. Reference:

```rust
#[test]
fn a_call_missing_a_required_field_names_it() {
    let decoded = crate::decode(b"message", br#"{"form":"words","words":"is the build green?"}"#, &fixtures::LIMITS);
    assert!(matches!(
        decoded,
        crate::Decoded::Problem(crate::Problem { field: b"target", why: crate::Why::Missing })
    ));
}
```

Proof: unit: `heavy cargo nextest run -p jig-protocol-tools` shows each
core tool's call decoded, each problem named, each part rendered; this
layer because the worlds route through it from 10.3; once, under 1 s.
Don't: link smith; depend on `jig-protocol-charter` (the routing maps a
`Declaration` to the charter's `Tool`); keep state; let a decoder panic
on any input (the fuzzy suite's decoder fuzzing is jig 22's for records,
not here).
Size: at most 1,400 lines of production code (927 moved; ruling 109).
Removes: nothing (10.4 removes `ops`'s copies).

### 10.3 Calls decoded and answers rendered by the test application's harness

Scope: channel.md, 6.3; root.md, section 5. The core world's and the
conformance world's harness stand in for the engine's protocol layer:
on `Now::Call` they read the call's key with
`jig_protocol_charter::call_key` and decode its input with
`jig_protocol_tools::decode`, sending back `NamedAction` or
`DelegateValidated`, or answering a problem (or a name that does not
decode) at once through the test root's `Event::CallAnswered`; on
`Held::CallAnswer` and `Now::EffectAnswer` for a call they decoded they
render the part with `jig_protocol_tools::render` and send
`Event::SettledCall` or `Event::CallAnswered`. A tool `decode` calls
`NotMine` keeps today's path (recorded in `host_calls`, answered by the
story). The test root renames `Event::ReadAnswer` to
`Event::CallAnswered`.
Needs: 10.1's `call_key`; 10.2's crate.
Files: `testing/jig-test-domain/src/lib.rs` (`Event::ReadAnswer`, lines
231, 472, 578, 623, 758); `tests/core/src/protocol.rs` (new: the
stand-in, at most 200 lines), `tests/core/src/{effects.rs (Now::Call
line 530, Held::CallAnswer line 820, host_call line 1239), peers.rs
(opaque_answer line 319), lib.rs}`, `tests/core/Cargo.toml`;
`tests/conformance/src/lib.rs` (line 451); `tests/core/tests/host_tools.rs`
(new). Users of `root::Event::ReadAnswer`: testing/jig-test-domain/src/lib.rs
(5), tests/core/src/observations.rs:400, tests/core/src/peers.rs:325,
tests/core/src/reads.rs:91, tests/core/src/watches.rs:203,
tests/core/tests/store.rs:172. Shared hotspots:
`testing/jig-test-domain/src/lib.rs`, `tests/core/src/effects.rs`,
`tests/conformance/src/lib.rs`.
Shape: `jig_test_domain::Event::CallAnswered { to: ReplyTo, call:
core::SettledCall }` (was `ReadAnswer`, same fields: "an answer made
with no durable call record: a run's read, or a call that does not
decode"). In the world: `World::tool_call(tool: &[u8], input: &[u8],
writes: bool) -> Token` relays a call named with smith's `CallName`
(activation the attempt, completion the next, position 0);
`protocol::Stand` keeps, per decoded call, its key, name and tool.
Test shape: `tests/core/tests/host_tools.rs` (new), world, under 0.5 s
each: `a_message_call_is_decoded_decided_and_answered_in_words`,
`a_call_whose_input_does_not_decode_is_answered_an_error_and_commits_nothing`,
`a_call_asked_again_after_an_error_is_decoded_again`,
`a_call_whose_name_does_not_decode_is_answered_an_error`,
`a_delegate_call_makes_its_batch_and_answers_their_numbers`. Reference:

```rust
#[test]
fn a_call_whose_input_does_not_decode_is_answered_an_error_and_commits_nothing() {
    let mut world = World::new(171, false);
    let commits = world.store.applied;
    world.tool_call(b"message", br#"{"form":"words","words":"is the build green?"}"#, true);
    assert_eq!(world.store.applied, commits, "a call that does not decode decides nothing");
    let (_, answer) = world.host_answers.last().expect("the call is answered at once");
    assert_eq!(
        answer.answer,
        core::SettledAnswer::Host { error: true, body: b"message: the field target is missing".as_slice().into() }
    );
}
```

(the expected words are `jig_protocol_tools::problem`'s rendering of
`Problem { field: b"target", why: Why::Missing }` for the tool `message`:
10.2 fixes its wording, and this literal follows it.)
Proof: world: `heavy cargo nextest run -p jig-core-world --test host_tools`
shows a run's `message` and `delegate` calls decoded outside the root,
decided, and answered in rendered words, and an undecodable input
answered an error with no commit; once, under 3 s.
Don't: decode in the test root; change a test that answers its calls by
hand (tools that are not the core's keep that path).
Size: at most 40 lines of production code (the test root's rename; the
stand-in is test code).
Removes: `Event::ReadAnswer` (renamed).

### 10.4 `ops`'s calls and answers out of its root

Scope: root.md, sections 11 and 12; hosts.md, section 8. `ops`'s root
passes `Now::Call`, `Held::CallAnswer`, `Now::EffectAnswer`,
`Now::NoteBusy` and `Now::SettledCallRefused` out unchanged, and takes
back the core's typed events, `Event::Effect` (as today), a new
`Event::ConnectorRead` (a decoded read of `ops`'s, which the root admits
and routes to observability as `assemble::call` does at
assemble.rs:440-460) and `Event::CallAnswered` (rendered, no commit).
`ops`'s world decodes and renders: the core's tools with
`jig-protocol-tools`, `ops`'s five (`read_logs`, `read_series`,
`read_alerts`, `effect_restart`, `propose_restart`) with decoders and
renderers moved from translate.rs:877-1011 and assemble.rs into
`tests/ops/src/protocol/`, and the key with `jig_protocol_charter::call_key`;
its scripted agent names its calls with smith's `CallName` (big-endian),
where `name()` (scripts.rs:364-369) writes little-endian today. The
root's tools come from `Config::tools`. Its procedure templates
(`translate::triage`, `lead`, translate.rs:1168-1239) parse with
`jig_protocol_tools::batch`.
Needs: 10.1's `call_key`; 10.2's crate.
Files: `examples/ops/domain/src/{translate.rs (1,310 lines: loses 249-1142
but `read_description`, `resource_role`, `hold_kinds`, `watch_result`
and the templates), assemble.rs (loses `call`, `answer`, `render`,
`delegated`, `notes`, `read_answer`'s rendering), route.rs (lines 325,
1047, 1075, 1199, 1362-1400; 1,585 lines: net shrink), boundary.rs
(`Event::{ConnectorRead, CallAnswered}`), domain.rs (`Config::tools`;
`calls` goes, line 51-67), Cargo.toml (drops `skein-json`; gains
`jig-protocol-tools` for the templates), tests.rs (`call_name` helper
397-405; `a_tool_mutation_and_its_rendered_reply_share_one_commit`,
line 412, asserts rendered bytes: its rendering moves to the world)}`;
`tests/ops/src/protocol/{mod.rs, tools.rs}` (new);
`tests/ops/src/conformance/{mod.rs (Output arms 278-291, 507-575),
scripts.rs (name 364, call 477, result 494), config.rs (line 82)}`;
`tests/ops/src/joint/{engine.rs, front.rs}`. Users the audit counted:
`translate::decode`/`Decoded`/`Problem`/`json`/`nested`/`notes` 4 sites
(assemble.rs:440-461, 468-484; translate.rs:1169-1170, 1231-1232);
`translate::call_name` 3 (assemble.rs:426; examples/ops/domain/src/tests.rs:397-405;
tests/ops/src/conformance/scripts.rs:364-369); `translate::tools` 1
(assemble.rs:66); `assemble::call` 1 (route.rs:1363); the renderers 11
(route.rs:325, 1047, 1075, 1199, 1367, 1383, 1393; assemble.rs:428,
454, 462, 497); the root's `Output`/`Event` matched exhaustively in 5
files (examples/ops/domain/src/domain.rs, examples/ops/domain/src/tests.rs,
tests/ops/src/conformance/mod.rs, tests/ops/src/joint/engine.rs,
tests/ops/src/joint/front.rs). Shared hotspots:
`examples/ops/domain/src/{route.rs, assemble.rs, translate.rs}`,
`tests/ops`.
Shape: `jig_ops_domain::Event::ConnectorRead { to: ReplyTo, task: u64,
name: Box<[u8]>, tool: Box<[u8]>, read: observability::Read }`, its `task`
copied by the world from the `Now::Call`'s authenticated run, for
`connector_read_admit` (ruling 111);
`jig_ops_domain::Event::CallAnswered { to: ReplyTo, call:
core::SettledCall }`; `Config { tools: Box<[jig_protocol_charter::Tool]>,
.. }`.
Test shape: `tests/ops/tests/stories.rs`: the alert storm
(`the_alert_storm_reaches_triage_and_the_person_accepts_the_production_restart`,
line 59) runs unchanged in what it asserts; new, world, under 0.5 s
each: `an_ops_call_that_does_not_decode_is_answered_naming_the_field`,
`an_ops_read_is_decoded_outside_the_root_and_answered`;
`examples/ops/domain/src/tests.rs`: `a_tool_mutation_and_its_rendered_reply_share_one_commit`
becomes `a_tool_mutation_and_its_reply_part_share_one_commit` (the part,
not the rendered bytes, is the root's).
Proof: world: `heavy cargo nextest run -p jig-ops-world --test stories
the_alert_storm` shows the storm's reads, delegation, proposal and
decision decoded and rendered outside `ops`'s root; once, under 5 s.
Don't: let `ops`'s root call a decoder or a renderer of a call;
change what the alert storm asserts; add an `ops` protocol crate.
Size: at most 120 lines of production code added (about 1,000 removed
from `ops`'s root).
Removes: `translate::{decode, tools, Decoded, Problem, call_name,
CallName}` and the `json`, `nested` and `notes` modules;
`assemble::{call, answer, render, delegated, notes}`; `Domain::calls`;
`ops`'s domain's dependency on `skein-json`.

### 10.5 `ops`'s results out of its root

Scope: channel.md, 6.1; root.md, section 11. `ops`'s root keeps the
hub's answer with its result's bytes (`Payload::Answer { ending }`)
until the fleet has fenced it; at `Now::AnswerPayload` an ended answer's
result goes out as `Output::ReadResult { payload, outcome }`, and comes
back as `Event::ResultRead { payload, end }`, which the root steps into
the core's `Event::AnswerPayload`; the world reads it with
`jig_protocol_charter::result` and maps it to `tasks::End::Finished` or,
undecodable, `tasks::End::Failed(Class::Invalid)`. The core's
`route_answer_payload` (runs.rs:721) drops the payload of an attempt no
longer current (`Now::DropPayload`) where it asserted (runs.rs:732).
Needs: 10.1's `result`.
Files: `crates/jig-domain-core/src/routing/runs.rs` (line 732);
`examples/ops/domain/src/{boundary.rs (Payload::Answer, line 171;
Output, Event), route.rs (lines 274, 1276-1292, 1316, 1462-1477),
assemble.rs (`ending`, `decode_result`, 244-284; line 500)}`;
`tests/ops/src/protocol/mod.rs`, `tests/ops/src/conformance/mod.rs`,
`tests/ops/src/joint/engine.rs`; `crates/jig-domain-core/src/tests.rs`.
Users of `Payload::Answer` the audit counted: route.rs:274, 1278-1279,
1316, 1466; assemble.rs:500. Shared hotspots: `examples/ops/domain/src/route.rs`
(jig 05 changes the `host_out` answer arm, route.rs:1462-1477, for
refusals), `routing/runs.rs` (jig 05).
Shape: `jig_ops_domain::Output::ReadResult { payload: Token, outcome:
Box<[u8]> }`; `jig_ops_domain::Event::ResultRead { payload: Token, end:
tasks::End }`; `Payload::Answer { task, attempt, spent, ending:
host::Ending }`.
Test shape: `crates/jig-domain-core/src/tests.rs`, unit, under 20 ms:
`an_answer_payload_for_a_replaced_attempt_is_dropped`;
`tests/ops/tests/stories.rs`, world, under 0.5 s each:
`a_run_whose_result_does_not_decode_fails_its_attempt_as_invalid`,
`a_stale_attempts_answer_is_fenced_before_its_result_is_read`.
Proof: world: `heavy cargo nextest run -p jig-ops-world --test stories
a_run_whose_result_does_not_decode` shows a scripted agent's malformed
result read outside the root and its attempt failed as invalid; once,
under 1 s.
Don't: read a result before the fleet has fenced its answer; change
the hub's `Ending`.
Size: at most 80 lines of production code (about 45 removed).
Removes: `assemble::decode_result` and the decoding in
`assemble::ending`; `ops`'s domain's dependency on `smith-charter`
unless its limits still name `smith_charter::v2::Limits` for the
charter's encoding (limits.rs:27), which stays.

## Interface sketches

```rust
// jig_protocol_tools: the core's host tools (channel.md, 6.3). Keeps no
// state; the routing calls it in the step that routes a call (service.md,
// section 2). Owned here; used by jig 25's routing and the worlds.
use alloc::boxed::Box;
use jig_domain_core as core;

/// One host tool as the charter declares it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Declaration {
    pub name: &'static [u8],
    pub description: &'static [u8],
    /// A JSON Schema of its input, as the charter declares it.
    pub schema: &'static [u8],
    pub writes: bool,
}

/// The decoders' bounds, from the core's limits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub input_bytes: u32,
    pub words: u32,
    pub batch: u32,
    pub depth: u32,
}

/// What is wrong with an input, by field.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Why {
    Malformed,
    Missing,
    Type,
    Range,
    TooLarge,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Problem {
    /// The field's name, empty for the input as a whole.
    pub field: &'static [u8],
    pub why: Why,
}

/// A call's input, decoded (`core::NamedAction` has no equality: tests
/// match it).
#[derive(Debug)]
pub enum Decoded {
    /// One of the core's named actions: sent as `core::Event::NamedAction`.
    Action(core::NamedAction),
    /// A delegate batch: sent as `core::Event::DelegateValidated`.
    Delegate(Box<[core::Delegate]>),
    /// Its input does not decode: answered at once as an error.
    Problem(Problem),
    /// Not one of the core's tools: the application's decoders are asked.
    NotMine,
}

/// The engine's text for the LLM.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Rendered {
    pub error: bool,
    pub text: Box<[u8]>,
}

/// The core's tools, in a fixed order.
#[must_use]
pub fn declarations() -> &'static [Declaration] {
    todo!()
}
#[must_use]
pub fn decode(tool: &[u8], input: &[u8], limits: &Limits) -> Decoded {
    todo!()
}
/// A delegate batch alone, for a configuration's templates.
pub fn batch(input: &[u8], limits: &Limits) -> Result<Box<[core::Delegate]>, Problem> {
    todo!()
}
#[must_use]
pub fn render(part: &core::CallPart) -> Rendered {
    todo!()
}
#[must_use]
pub fn problem(tool: &[u8], problem: &Problem) -> Rendered {
    todo!()
}

// jig_protocol_charter::call (10.1)
/// A call's name as smith gives it (smith-channel's `CallName`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CallKey {
    /// The attempt that made it; the routing checks it against the fence.
    pub activation: u64,
    pub completion: u32,
    pub position: u32,
}
#[must_use]
pub fn call_key(name: &[u8]) -> Option<CallKey> {
    todo!()
}

// jig_protocol_charter::result (10.1)
/// One field of a result.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Field {
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}
/// A run's result in the forms the core judges (hosts.md, section 8).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RunResult {
    Report { words: Box<[u8]> },
    /// `label` is the verdict's name as the charter declared it.
    Verdict { label: Box<[u8]>, fields: Box<[Field]>, words: Box<[u8]> },
    Failure { reason: Box<[u8]> },
}
/// Why a result does not decode.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unreadable {
    /// Not smith's result record, or bytes after it.
    Malformed,
    /// A change, or a verdict with items: no form the core judges.
    Form,
}
pub fn result(bytes: &[u8]) -> Result<RunResult, Unreadable> {
    todo!()
}
```

- **Who owns what:** the core decides and keeps parts; the protocol
  crates own schemas, decoding and rendering and keep nothing; a root
  routes the call out and the typed request back, and keeps a payload's
  bytes until the fleet has fenced it. A call ends with the core's
  `SettledCall` (decided) or the root's `CallAnswered` (not decoded, or
  a read); a result's reading ends with `Event::ResultRead`.

## Questions to raise when reached

- **The relayed answer's error flag in the hub:** channel.md, 5.2 gives
  the relayed record "its text for the LLM and whether it is an error";
  the hub's `Event::Relayed { answer: Box<[u8]> }` carries bytes, and
  `ops`'s root writes the flag as a leading byte
  (`assemble::host_answer`, assemble.rs:230-242). This session keeps
  that; a typed flag is jig 05's hub vocabulary or jig 22's record.

## Done when

- No root decodes a call or reads a result: `git -C ~/src/rust/jig grep
  -n -E 'RunResult::decode|fn call_name|fn decode\(tool' --
  examples/ops/domain testing/jig-test-domain
  ':!examples/ops/domain/src/tests.rs'` prints nothing (3 lines on
  2026-10-12).
- `jig-protocol-tools` links no smith: `heavy cargo nextest run -p
  jig-boundary` passes.
- The readers and the tools pass alone: `heavy cargo nextest run -p
  jig-protocol-charter -p jig-protocol-tools` passes.
- Calls decode outside the roots: `heavy cargo nextest run -p
  jig-core-world --test host_tools` and `heavy cargo nextest run -p
  jig-ops-world --test stories` pass.
- The gate passed on this session's last landing: `git -C ~/src/rust/jig
  log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step:.jig-10\.' origin/main | head -1` names
  `scripts/check.sh passed`.
