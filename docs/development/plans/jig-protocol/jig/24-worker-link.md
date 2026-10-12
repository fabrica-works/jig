# Session 24: `jig-protocol-worker-link`, the worker's side of the channel

A new step component of a worker's protocol layer owns the worker's
link: the one channel it dials and redials with a jittered backoff, its
liveness, the grace's clock after losing it, the relays in flight across
channels, the turns the engine answers busy and when to send them again,
grants' values taken out for the worker's tables, and the same
translation as the engine's side, toward the hub's link vocabulary. The
hub arms no timers and never dials (jig's `domain/hosts.md`, 6.2): the
link's policy is this component's (`protocol/README.md`, section 13). Its
machine world, `tests/worker-link`, runs it between the worker's root,
scripted, and skein's scripted peer as the engine; its last increment
joins it to jig 23's component, the thin story both sides meet in.

## Read first

- jig's `protocol/channel.md`, sections 2 to 5, 8, 9, 10.1, 10.3 and 12;
  `domain/hosts.md`, section 2 and 6.1 to 6.7; `protocol/service.md`,
  section 6.
- skein's `channel.md`, sections 5 to 10; programming-model.md, sections
  4, 5.2 and 9; testing-strategy.md, 2.4.
- `decisions.md`, 6; `code-organization.md`, sections 2 to 6.
- `findings/protocol.md`, 4.1 and 6.2 ("24-worker-link").
- The code: jig 22's `crates/jig-protocol-worker-wire` (records, `Part`,
  `Reason`, `terms_gap`, `sizes::worker_cap`); jig 23's
  `crates/jig-protocol-engine-workers` and `tests/engine-workers/src/net.rs`
  (the shape this side mirrors, and the world's byte pipes); jig 05's link
  vocabulary in `crates/jig-domain-host/src/boundary.rs` and the fleet's
  `Bounds` (jig 05's sketches), written here as jig 25.2 names the two directions:
  `host::LinkUp`, `host::LinkDown`, and the hello as one value,
  `host::LinkHello`; where jig 05 landed other names, use them, and
  nothing else here changes; jig 05's `GrantTable`;
  `crates/jig-domain-host/src/domain.rs:184-188` (`Domain::turn`: "A
  transmission copy of a retained turn"). Shapes to copy: smith's
  `crates/smith-host-protocol/src/component.rs:101-142` (an initiator
  owning a skein channel); skein's `testing/skein-fake-channel/src/lib.rs`
  (`ScriptedPeer` as responder) and `crates/skein-lib/src/rng.rs:26-44`
  (`below`, `between`).

## What the design asks

- **Dial at once, redial on loss** (channel.md, 10.1): "Redial on loss,
  after a jittered backoff that doubles up to a ceiling, starting over
  once a channel has proved itself: it carried the hello and stayed open
  for the configured span." "as for any refusal" (4.3).
- **The hub hears the link** (10.1): "up, once ready, when it asks the
  hub to report what it hosts and sends the hello first; lost, when it
  starts the grace's clock."
- **The grace's clock is the component's** (10.1): "Past the grace with
  no channel back, the component tells the worker's root, which asks the
  hub to cancel every run it hosts for lost contact ... A channel back
  within the grace stops the clock."
- **Relays outlive channels** (10.1): "The component keeps every relay
  in flight in one table across channels, bounded by the hub's slots and
  calls per run, and ends each once: answered, or cancelled by the hub."
- **Busy, sent again** (5.2): "A turn answered busy is held by the
  worker's link component, which sends it again after a jittered
  backoff, as it keeps the grace's clock; the hub arms no timers. A call
  answered busy goes back to its run".
- **After a redial, everything kept goes again, in order, after the
  hello** (5.3).
- **Grants' values taken out** (section 8): "into the worker's table of
  the same owner, by number, at most the two newest generations of each,
  and the hub hears the names"; "Validity crosses as a duration ...,
  which the reader turns into a deadline on its own."
- **Liveness and the facts' reserve** (section 9): pings, silence, stall;
  "The worker's side sends a fact only while the queue leaves the rest of
  the cap free".
- **Shutting down** (10.1): "the worker stops redialling once the hub
  has given up what it keeps".
- **States** (10.3): `idle ─► connecting ─► opening ─► open ─► closing ─►
  idle (redial after backoff)`.

## Choices this plan makes

- **The dial is the protocol layer's** (channel.md, section 2: "the
  worker's dial and TLS"): the component decides when, with `Dial`, and
  takes the connection the layer made, exchanging skein's `Lower` records
  over it; it never makes an io request, and never knows the engine's
  address.
- **Every refusal is a loss,** redialled after the backoff whatever its
  reason; the reason is counted in the component's facts for operators.
- **The jitter** is drawn from a seed the layer gives (programming-model.md,
  section 9), for redials and busy turns alike: each span is drawn
  between half and all of the current backoff.
- **A busy turn is resent by asking the root:** the component keeps the
  turn's name and its resend deadline; at the deadline it asks
  `Resend`, and the root reads the hub's retained copy
  (`Domain::turn`) and hands it back as an ordinary `Up`, or nothing if
  the hub no longer keeps it. The hub keeps the bytes
  (`domain/hosts.md`, 6.4: "Kept until acknowledged"); no second copy is
  held. A turn kept meanwhile drops its entry; a loss drops every entry,
  since the hub sends its turns again after the hello.
- **A relay keeps its call's record** until it ends, so it can go again
  after a redial, in the order the calls were first sent, before what the
  hub sends again. A relayed answer busy ends its relay like any answer
  and goes up to the hub as busy, for its run to ask again.
- **Values are handed out, not kept:** the component hands each grant's
  value to its layer (`Value`, with its deadline on this side's clock)
  before the record that names it goes to the root; the layer keeps them
  in the worker's tables: `jig-protocol-accounts`' `Table` for jig's
  accounts' (jig 21) and the application's own for its credentials. So
  the two-generation table is one type, jig 21's.
- **The machine world** is `tests/worker-link` (package
  `jig-worker-link-world`, ordinary Rust), with the byte pipes of jig
  23's `tests/engine-workers/src/net.rs` copied into its own `src/net.rs`
  as each world keeps its own (code-organization.md, section 5).

## Rules that bind it

- "**`jig-protocol-*` crates** depend on no application's crate. Only
  `jig-protocol-charter` and `jig-protocol-transcript` link smith; the
  channel's components do not." (code-organization.md, section 3)
- "No increment of this plan ... adds an `unreachable!` arm for a
  variant the reference service can reach; an arm the service reaches is
  routed." (code-organization.md, section 6)
- "**Step code** follows skein's `programming-model.md`, section 10, and
  jig's `AGENTS.md`: no traits, no generic items, no `dyn`, no closures."
  (code-organization.md, section 2)

## Increments

### 24.1 Dialling and redialling

Scope: the crate `crates/jig-protocol-worker-link` (component, step
code); `Limits`, `worst_case`, the entry points' most outputs; the states
of 10.3; `Dial` at once; the opening as initiator with the credential;
the terms stated from `largest_body` and the engine's checked with
`terms_gap`, refused *limits* naming the kind; the handshake's deadline;
every refusal and failure a loss; the backoff, jittered, doubling to its
ceiling, starting over after a proved channel (channel.md, 4.1, 4.3,
10.1, 10.3).
Needs: nothing beyond the session's start.
Files: `crates/jig-protocol-worker-link/{Cargo.toml, src/lib.rs,
src/boundary.rs, src/limits.rs, src/component.rs, src/connection.rs,
src/backoff.rs, src/tests.rs, src/tests/fixtures.rs,
src/tests/backoff.rs}`; `tests/worker-link/{Cargo.toml, src/lib.rs,
src/net.rs, src/world.rs, src/fixtures.rs, tests/dialling.rs,
tests/memory.rs}`; the workspace `Cargo.toml` (members);
`tests/boundary/src/` (the crate's role: skein-lib, skein-channel,
`jig-protocol-worker-wire`, `jig-domain-host`; never another
`jig-domain-*` crate nor smith). Shared hotspots: the workspace
`Cargo.toml`, `tests/boundary/src/`.
Shape: as the interface sketch: `boundary.rs` (`Event`, `Request`),
`limits.rs` (`Limits`, `worst_case`, `MAX_OUT_DOWN`, `MAX_OUT_FIRE`),
`component.rs` (`Component::new`, `down`, `fire`, `next_deadline`,
`dropped_facts`), `connection.rs` (`Phase::{Idle, Connecting, Opening,
Open, Closing}`), `backoff.rs`:

```rust
/// The redial backoff: doubles from `first` to `ceiling`, each span drawn between half and all.
pub(crate) struct Backoff { first: Duration, ceiling: Duration, current: Duration, rng: Rng }
impl Backoff {
    pub(crate) fn next(&mut self) -> Duration;
    pub(crate) fn proved(&mut self);
}
```

Test shape:
- `src/tests/fixtures.rs` (new): `LIMITS` (backoff 100 ms to 1.6 s,
  proved 2 s, handshake 1 s, grace 10 s, busy 200 ms),
  `credential()` (`WORKER_A`, `SECRET_A`, `PAYLOAD` 3), `SEED_ONE` (1),
  and, from 24.2, `TOKEN_ONE` (`b"token-one"`) and
  `granted_record(value)` (run 7, attempt 2, generation 3, valid 60 s).
- `src/tests/backoff.rs` (new), reference test:

  ```rust
  #[test]
  fn backoff_spans_double_to_the_ceiling_within_their_jitter() {
      let mut backoff = Backoff::new(&fixtures::LIMITS, fixtures::SEED_ONE);
      let bounds = [(50, 100), (100, 200), (200, 400), (400, 800), (800, 1600), (800, 1600)];
      for (low, high) in bounds {
          let span = backoff.next();
          assert!(span >= Duration::from_millis(low) && span <= Duration::from_millis(high), "{span:?}");
      }
  }
  ```

  Others: `a_proved_channel_starts_the_backoff_over`,
  `two_seeds_draw_different_spans`.
- `tests/worker-link/src/world.rs` (new): `World::new(&Limits, seed)`,
  `engine(ScriptedPeer)` (the next dial reaches it), `refuse_dial()`
  (the next dial is unreachable), `run()`, `advance(Duration)`,
  `down(Event)`, `dials() -> &[Time]`, `root() -> &[Request]` (what
  reached the root, in order).
- `tests/worker-link/src/fixtures.rs` (new): `limits()`,
  `engine_accepting()`, `engine_refusing(reason: u16)`,
  `engine_with_terms(kind, largest)` (each a `ScriptedPeer` responder,
  version 1).
- `tests/worker-link/tests/dialling.rs` (new), reference test:

  ```rust
  #[test]
  fn a_refused_worker_redials_after_its_backoff() {
      let mut world = World::new(&fixtures::limits(), 1);
      world.engine(fixtures::engine_refusing(258));
      world.engine(fixtures::engine_accepting());
      world.run();
      world.advance(Duration::from_millis(100));
      world.run();
      assert_eq!(world.dials().len(), 2);
      let gap = world.dials()[1].saturating_since(world.dials()[0]);
      assert!(gap >= Duration::from_millis(50) && gap <= Duration::from_millis(100), "{gap:?}");
  }
  ```

  Others: `the_worker_dials_at_once`,
  `an_unreachable_engine_is_dialled_again_after_the_backoff`,
  `each_jig_reason_is_redialled_after_the_backoff`,
  `another_version_is_redialled_after_the_backoff`,
  `engine_terms_below_the_largest_turn_are_refused_as_limits`,
  `an_opening_past_the_handshake_deadline_is_closed_and_redialled`,
  `a_channel_lost_before_it_proved_keeps_doubling`.
- `tests/worker-link/tests/memory.rs` (new):
  `the_link_at_its_limits_stays_within_its_worst_case`, as
  `tests/host/tests/memory.rs:1-14`.
- Ceilings: step tests under 20 ms; world tests under 0.5 s each.
Proof: world, `heavy cargo nextest run -p jig-worker-link-world -E
'test(dialling)'` once: a worker refused for any reason, or unreachable,
dials again after a jittered backoff that doubles, and starts over after
a proved channel; under 1 minute with the build warm. Not higher: the
engine's side joins it in 24.5.
Don't: make an io request; hold the engine's address.
Size: at most 450 lines of production code.
Removes: nothing.

### 24.2 The link up, the hello first, and the records

Scope: once open, `Ready` to the root, which hands back the hub's hello,
sent first and nothing before it; `translate.rs`, both halves: each
`host::LinkUp` variant to its up record, the application's parts' bytes
written whole; each down record to its `host::LinkDown` variant, the
parts named by tokens of the component's; each grant's value handed out
as `Value`, with its deadline, before the record that names it; `Lost`
once for each channel that was up; an `Up` while no channel is up
dropped, but a relay (24.3) (channel.md, 3, 5, 6.2, 8, 10.1).
Needs: nothing beyond 24.1.
Files: `crates/jig-protocol-worker-link/{src/translate.rs,
src/connection.rs, src/component.rs, src/boundary.rs,
src/tests/translate.rs, src/tests/fixtures.rs}`;
`tests/worker-link/{src/world.rs, src/fixtures.rs, tests/link.rs,
tests/records.rs}`.
Shape: `translate::up(up: host::LinkUp, parts: &[Part]) ->
Option<wire::Up>` (`None`: a part the vocabulary names is missing, a
routing defect: dropped and counted); `translate::down(record: wire::Down,
parts: &mut Parts, values: &mut Values) -> host::LinkDown`, total over
each record; `Values` collects the `Value` requests in record order.
Test shape:
- `src/tests/translate.rs` (new), reference test:

  ```rust
  #[test]
  fn a_granted_record_hands_its_value_out_and_the_hub_hears_the_name() {
      let mut parts = Parts::new(2);
      let mut values = Values::new(2);
      let link = translate::down(fixtures::granted_record(TOKEN_ONE), &mut parts, &mut values);
      let host::LinkDown::Grant { run, attempt, grant } = link else { panic!("a grant: {link:?}") };
      assert_eq!((run, attempt, grant.generation), (Token::new(7), Token::new(2), 3));
      let value = values.pop().expect("one value");
      assert_eq!(value.value.expose(), TOKEN_ONE);
      assert_eq!(value.expires, Time::ZERO.saturating_add(Duration::from_secs(60)));
  }
  ```

  The field names are jig 05's and jig 08's. Others, one per kind:
  `a_hub_turn_becomes_a_turn_record_with_its_refusals`,
  `a_hub_answer_writes_left_and_saved_work_from_their_parts`,
  `typed_activity_becomes_a_fact_record`,
  `an_assignment_record_hands_its_items_up_as_a_part`,
  `an_assignment_record_keeps_its_start_messages_in_order`,
  `a_message_record_becomes_the_hubs_inbound_message`.
- `tests/worker-link/tests/link.rs` (new):
  `the_hello_is_the_first_record_on_every_channel`,
  `the_root_hears_ready_once_per_channel`,
  `a_lost_channel_is_told_once`,
  `a_turn_while_the_link_is_down_is_dropped`.
- `tests/worker-link/tests/records.rs` (new):
  `each_up_record_reaches_the_engine_in_order`,
  `each_down_record_reaches_the_root_in_order`,
  `grant_values_reach_the_layer_before_the_assignment`,
  `an_undecodable_assignment_is_refused_as_framing`.
Proof: world, `heavy cargo nextest run -p jig-worker-link-world -E
'test(link) | test(records)'` once: the hello goes first on every
channel, records cross both ways as the hub's vocabulary, and a grant's
value reaches the layer before the root hears its name; under 1 minute
with the build warm.
Don't: hand a value to the root; send a record before the hello.
Size: at most 450 lines of production code.
Removes: nothing.

### 24.3 Relays across channels, and busy turns

Scope: the relay table: an entry as the hub relays a call, keyed by its
run, attempt and call name, holding its call record and the hub's token;
each entry sent again after every hello, in the order first sent, before
what the hub sends again; ended once, by its relayed answer (busy among
them, passed up as busy) or by the hub's cancel (`CancelRelay`, answered
`RelayCancelled` unless its answer came first); bounded by the hub's
slots and calls per run. Busy turns: `turn busy` records the turn's name
and a resend deadline after the busy backoff, jittered; at the deadline
`Resend` goes to the root; a turn kept, or its attempt acknowledged,
drops its entry; a loss drops every entry (channel.md, 5.2, 5.3, 10.1).
Needs: nothing beyond 24.2.
Files: `crates/jig-protocol-worker-link/{src/relays.rs, src/busy.rs,
src/component.rs, src/boundary.rs, src/limits.rs, src/tests/relays.rs,
src/tests/busy.rs, src/tests/fixtures.rs}`;
`tests/worker-link/{src/fixtures.rs, tests/relays.rs, tests/busy.rs}`.
Shape: `relays::Relays` (a `Map` by `(Token, Token, Box<[u8]>)` with an
order list, sized `slots × calls`), `busy::Busy` (a `Deadlines` of turn
names, sized `slots × turns`), with `Event::CancelRelay`,
`Request::RelayCancelled` and `Request::Resend` as the interface sketch
has them.
Test shape:
- `src/tests/relays.rs` (new), reference test:

  ```rust
  #[test]
  fn a_relay_answered_ends_once_and_a_late_cancel_finds_nothing() {
      let mut relays = Relays::new(&fixtures::LIMITS);
      relays.add(fixtures::relay(CALL_FIRST, Token::new(11))).expect("room");
      assert_eq!(relays.answered(Token::new(7), Token::new(2), CALL_FIRST), Some(Token::new(11)));
      assert_eq!(relays.cancel(Token::new(11)), None);
      assert!(relays.is_empty());
  }
  ```

  Others: `relays_go_again_in_the_order_first_sent`,
  `the_relay_table_holds_every_slot_and_call`,
  `a_turn_kept_drops_its_busy_entry`,
  `a_busy_turns_resend_falls_within_its_jitter`.
- `tests/worker-link/tests/relays.rs` (new):
  `a_relay_answered_on_a_later_channel_reaches_its_run_once`,
  `a_relay_cancelled_while_the_link_is_down_is_never_sent_again`,
  `a_busy_call_goes_up_as_busy_and_ends_its_relay`,
  `relays_go_again_after_the_hello_before_the_hubs_resends`.
- `tests/worker-link/tests/busy.rs` (new):
  `a_busy_turn_is_asked_again_after_its_backoff`,
  `a_busy_turn_kept_meanwhile_is_not_asked_again`,
  `a_loss_drops_every_busy_turn`,
  `the_hub_never_hears_a_turn_busy`.
Proof: world, `heavy cargo nextest run -p jig-worker-link-world -E
'test(relays) | test(busy)'` once: a relay answered on a later channel
reaches its run once, a cancelled one never goes again, and a turn
answered busy is asked again after a jittered backoff until it is kept;
under 1 minute with the build warm.
Don't: keep a copy of a turn's bytes; let the hub hear a turn's busy.
Size: at most 400 lines of production code.
Removes: nothing.

### 24.4 The grace, liveness, the facts' reserve and stopping

Scope: the grace's clock from `Lost`, stopped by a channel back within
it, `GracePassed` once past it; pings, silence and stall as jig 23's
side has them; the output cap and its facts' reserve from `worker_cap`,
a fact that would eat into the rest dropped and counted, a cap filled
otherwise a loss; `Stop` from the root once the hub has given up what it
keeps: no redial, the channel finished once drained, `Closed` once the
connection is gone; `Abort` (channel.md, 9, 10.1).
Needs: nothing beyond 24.3.
Files: `crates/jig-protocol-worker-link/{src/grace.rs, src/connection.rs,
src/component.rs, src/limits.rs, src/tests/fixtures.rs}`;
`tests/worker-link/{src/net.rs, src/world.rs, src/referee.rs,
tests/grace.rs, tests/liveness.rs, tests/fuzzy_worker_side.rs,
tests/memory.rs}`.
Shape: `grace.rs`, the grace's one deadline; `component.rs`'s
`dropped_facts() -> u64`.
Test shape:
- `tests/worker-link/tests/grace.rs` (new):
  `the_grace_passing_with_no_channel_tells_the_root_once`,
  `a_channel_back_just_within_the_grace_stops_its_clock`,
  `a_stop_never_redials`.
- `tests/worker-link/tests/liveness.rs` (new):
  `a_silent_engine_is_lost_at_the_silence_deadline`,
  `an_engine_that_stops_reading_is_lost_by_stall`,
  `facts_are_dropped_past_the_reserve_while_turns_still_go`.
- `tests/worker-link/src/referee.rs` (new): over `World::root()` and the
  engine's observed records, each relay ended once, each busy turn asked
  again until kept, `GracePassed` at most once per loss, the hello first
  on every channel.
- `tests/worker-link/tests/fuzzy_worker_side.rs` (new), reference test:

  ```rust
  #[test]
  fn fuzzy_engines_cuts_and_stalls_keep_relays_ending_once() {
      for seed in 0..40 {
          let mut world = World::new(&fixtures::limits(), seed);
          let mut rng = Rng::new(seed);
          for _ in 0..6 {
              world.engine(fixtures::drawn_engine(&mut rng));
              world.down(fixtures::drawn_up(&mut rng));
              world.advance(Duration::from_millis(rng.between(1, 3_000)));
          }
          world.down(Event::Stop);
          world.advance(Duration::from_secs(30));
          referee::judge(&world).expect("relays end once, the grace told once per loss");
      }
  }
  ```

- `tests/memory.rs` grows: `relays_and_busy_turns_at_their_limits_stay_within_the_worst_case`.
- Ceilings: world tests under 0.5 s each; the fuzzy test under 3 s; the
  world's focused tests at most 1 s in all by 24.4.
Proof: world, `heavy cargo nextest run -p jig-worker-link-world` once,
and `heavy cargo nextest run -p jig-worker-link-world --profile fuzzy`
once: past the grace the root is told once, within it nothing is, and
facts give way to turns at the cap; under 2 minutes with the build warm.
Don't: arm a timer in the hub's place for anything but the grace,
pings, silence, stall, the backoff and busy resends.
Size: at most 350 lines of production code.
Removes: nothing.

### 24.5 The two sides joined

Scope: the thin end-to-end story where the channel's two sides meet:
jig 23's component and this one joined by in-memory bytes cut at seeded
sizes, the hub's vocabulary scripted on both ends: a worker dials, says
hello and is admitted; an assignment with a grant of each table goes
down and its values reach the worker's layer; a turn goes up and is
kept; a call is relayed and answered; the answer goes up and is
acknowledged; the bytes are cut, the worker redials, and what was kept
goes again after the hello. Beside it, the two translations agree,
variant by variant, through bytes; and jig 23's opening scenarios run
against this component as the worker (development.md, 5.3: "a fake's
scenarios also run against the real side").
Needs: jig 23's component with its down records and grants' values (jig
23's 23.3).
Files: `tests/worker-link/{Cargo.toml (the dev-dependency on
jig-protocol-engine-workers), src/joined.rs, src/fixtures.rs,
tests/joined.rs, tests/agreement.rs}`; `tests/boundary/src/` (the world's
dependency list, if jig 01's rules list worlds' dependencies).
Shape: `joined::Joined { engine: jig_protocol_engine_workers::Component,
worker: Component, pipe: net::Pipe, engine_root: Vec<engine::Request>,
worker_root: Vec<Request> }`, with `run`, `advance`, `cut`, and the two
roots scripted: the engine's root admits each hello and acknowledges
what it receives; the worker's root answers `Ready` with a fixture hello
and `Resend` from a fixture turn list.
Test shape:
- `tests/worker-link/tests/joined.rs` (new), reference test:

  ```rust
  #[test]
  fn a_turn_crosses_and_is_kept_after_a_cut_and_a_redial() {
      let mut joined = Joined::new(&fixtures::joined_limits(), 1);
      joined.run();
      joined.engine_down(fixtures::assignment(RUN_SEVEN, ATTEMPT_TWO));
      joined.worker_up(fixtures::turn(RUN_SEVEN, ATTEMPT_TWO, 1));
      joined.cut();
      joined.advance(Duration::from_millis(200));
      joined.worker_up(fixtures::turn(RUN_SEVEN, ATTEMPT_TWO, 1));
      joined.run();
      assert_eq!(joined.engine_turns(RUN_SEVEN, ATTEMPT_TWO), [1]);
      assert_eq!(joined.engine_channels(), [Token::new(1), Token::new(2)]);
  }
  ```

  Others: `a_worker_is_admitted_and_its_hello_reaches_the_engine_root`,
  `an_assignments_grant_values_reach_the_worker_layer`,
  `a_relayed_call_is_answered_across_the_two_sides`,
  `an_answer_is_acknowledged_once`.
- `tests/worker-link/tests/agreement.rs` (new), one test per kind, each
  encoding a fixture value of the hub's vocabulary on one side and
  decoding it on the other, expecting the same value:
  `a_hello_crosses_unchanged`, `a_turn_crosses_unchanged`,
  `an_answer_crosses_unchanged`, `a_fact_crosses_unchanged`,
  `an_assignment_crosses_unchanged`, `a_grant_crosses_as_a_name_and_a_value`.
- The opening scenarios of jig 23's world against this component as the
  worker: `a_wrong_secret_worker_redials_after_unauthorized`,
  `another_payload_version_worker_redials_after_application`.
- Ceilings: under 0.5 s each; this increment's tests at most 0.3 s in
  all.
Proof: joint, `heavy cargo nextest run -p jig-worker-link-world -E
'test(joined) | test(agreement)'` once: a worker dials the engine's
side, is admitted, takes an assignment with its grants' values, and its
turn crosses and is kept once across a cut and a redial; under 1 minute
with the build warm. The highest layer: both real components, bytes in
memory.
Don't: link a domain crate but the hub's into the world for this story;
script smith's payloads as anything but opaque bytes.
Size: at most 50 lines of production code (test code excepted).
Removes: nothing.

## Interface sketches

```rust
// jig_protocol_worker_link::boundary
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_protocol_worker_wire::Part;
use skein_channel::{Lower, LowerEvent};
use skein_lib::{Secret, Time, Token};

/// What the worker's protocol layer and root route to the component.
#[derive(Debug)]
pub enum Event {
    /// From the layer: the dial `Dial` asked for made `connection`.
    Dialled { connection: Token },
    /// From the layer: the dial failed; nothing was made, nothing follows.
    Unreachable,
    /// From the layer: a lower event of `connection`'s stream.
    Lower { connection: Token, event: LowerEvent },
    /// From the layer: `connection`'s socket is gone, its last event.
    Gone { connection: Token },
    /// From the root: the hub's hello, the answer to `Ready`; sent first.
    Hello { hello: host::LinkHello },
    /// From the root: a record from the hub toward the engine, its parts' bytes.
    Up { up: host::LinkUp, parts: Box<[Part]> },
    /// From the root: the hub cancels the relay it named `call`; `RelayCancelled` answers it,
    /// unless its answer went up first.
    CancelRelay { call: Token },
    /// From the root: the hub has given up what it keeps; no redial, finish, then `Closed`.
    Stop,
    /// From the root: close now; `Closed` follows.
    Abort,
}

/// What the component asks of its layer, and tells its root.
#[derive(Debug)]
pub enum Request {
    /// To the layer: dial the engine; answered `Dialled` or `Unreachable`.
    Dial,
    /// To the layer: a lower record for `connection`'s stream.
    Lower { connection: Token, lower: Lower },
    /// To the layer: close `connection` gracefully; `Gone` follows.
    Close { connection: Token },
    /// To the layer: abort `connection`; `Gone` follows.
    Abort { connection: Token },
    /// To the root: the link is open; answer with the hub's hello.
    Ready,
    /// To the root: a record from the engine for the hub, its parts' bytes under tokens minted here.
    Down { down: host::LinkDown, parts: Box<[Part]> },
    /// To the layer: a grant's value for the worker's table of `table`, before the record naming it.
    Value { table: host::GrantTable, account: u32, generation: u64, expires: Time, value: Secret },
    /// To the root: the relay `call` ended by the hub's cancel; the terminal of `CancelRelay`.
    RelayCancelled { call: Token },
    /// To the root: hand back the hub's retained turn `turn` of this attempt, for a busy resend.
    Resend { run: Token, attempt: Token, turn: u32 },
    /// To the root: the link is lost; the grace's clock runs from now.
    Lost,
    /// To the root: the grace passed with no channel back: the hub cancels every run for lost contact.
    GracePassed,
    /// To the root: the component's terminal, after Stop or Abort.
    Closed,
}
```

```rust
// jig_protocol_worker_link::{limits, component}
use jig_protocol_worker_wire::sizes::Sizes;
use skein_lib::{Duration, Queue, Secret, Time};

/// The component's limits (channel.md, section 11), from its layer's configuration (jig 20, jig 25).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub sizes: Sizes,
    pub chunk: u32,
    pub handshake: Duration,
    pub ping: Duration,
    pub silence: Duration,
    pub stall: Duration,
    /// The redial backoff's first span and ceiling, and the span that proves a channel.
    pub backoff: Duration,
    pub ceiling: Duration,
    pub proved: Duration,
    /// The grace after losing the engine (configuration.md, section 4).
    pub grace: Duration,
    /// The busy turn's resend backoff, jittered.
    pub busy: Duration,
}

pub fn worst_case(limits: &Limits) -> Option<u64> { todo!() }
pub const MAX_OUT_DOWN: u32 = 0;
pub const MAX_OUT_FIRE: u32 = 0;

/// The worker's credential (channel.md, 4.1), encoded at each opening.
#[derive(Debug)]
pub struct Credential {
    pub name: Box<[u8]>,
    pub secret: Secret,
    pub payload: u32,
}

/// Why the component cannot be made.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unusable {
    Limits,
    Credential,
}

/// The worker's side of the channel: the link.
#[derive(Debug)]
pub struct Component;

impl Component {
    pub fn new(limits: &Limits, credential: Credential, seed: u64) -> Result<Component, Unusable> { todo!() }
    pub fn down(&mut self, now: Time, event: Event, out: &mut Queue<Request>) { todo!() }
    pub fn fire(&mut self, now: Time, out: &mut Queue<Request>) { todo!() }
    pub fn next_deadline(&self) -> Option<Time> { todo!() }
    /// Facts dropped for want of room since the component was made.
    pub fn dropped_facts(&self) -> u64 { todo!() }
}
```

- **Who hears what:** the hub hears records, grants' names, relays' ends
  (a busy call's among them), never a turn's busy; the worker's root
  hears `Ready`, `Down`, `RelayCancelled`, `Resend`, `Lost` and
  `GracePassed`, and answers with the hello, the hub's records and, at
  the end, `Stop`; the layer hears dials, lower records and values.
- **What ends what:** each relay ends once, by its answer or the hub's
  cancel; a busy entry ends when its turn is kept or the link is lost;
  the component ends once, `Closed`, after `Stop` or `Abort`.

## Questions to raise when reached

- **The hub's cancel of a relay.** Today `CancelRelay` and
  `RelayCancelled` are the hub's requests to its root
  (`crates/jig-domain-host/src/boundary.rs`, `Request::CancelRelay`,
  `Event::RelayCancelled`); on a worker this component answers them. If
  jig 05 moved them into `host::LinkUp` and `host::LinkDown`, they come
  through `Up` and `Down` and this component's own variants go.
- **The link vocabulary as one pair of enumerations** (jig 22's
  Questions): if jig 05 lands no `LinkUp`, `LinkDown` and hello as one
  value, raise it before 24.2 starts.
- **Where the worker keeps jig's accounts' values.** This session hands
  each value out (`Value`); jig 25.5 reads "each LLM grant's value out
  of the link's table". The default here: the worker's layer keeps them
  in a `jig_protocol_accounts::Table` (jig 21), one type for both sides'
  tables; if jig 25 lands needing the link to answer a value by name,
  raise it, rather than adding a second table type.
- **The proving span, the backoff's first span and ceiling, and the busy
  backoff** are profile numbers (jig 20); channel.md, 14 keeps the
  timings open. Report the values the world uses.

## Done when

1. The world's focused and fuzzy tests pass: `heavy cargo nextest run -p
   jig-worker-link-world` and `heavy cargo nextest run -p
   jig-worker-link-world --profile fuzzy` print no failure.
2. The two sides meet: `heavy cargo nextest run -p jig-worker-link-world
   -E 'test(joined) | test(agreement)'` passes.
3. The crate links neither the core nor smith: `cargo tree -p
   jig-protocol-worker-link -e normal --prefix none | grep -E
   '^(jig-domain-(core|fleet|tasks|people|accounts)|smith|jig-protocol-accounts)'`
   prints nothing.
4. The gate passed on this session's last landing: `git -C
   ~/src/rust/jig log --format='%(trailers:key=Gate-Result,valueonly)'
   --grep='^Step:.jig-24\.' origin/main | head -1` names
   `scripts/check.sh passed`.
