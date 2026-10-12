# Session 05: the hub's link vocabulary

`jig-domain-host` and `jig-domain-fleet` get the contract of jig's
`domain/hosts.md`, section 2, as one vocabulary toward the link, on a
worker as in the engine: two enumerations, `LinkDown` from the engine
and `LinkUp` toward it; the hello as one value, with the answers kept
until acknowledged and the agent's bounds; relayed answers typed; the
assignment's fields the domain will fill (the start's messages, the
run's identity, grants naming their table); turns that say whether they
open a window and carry the messages refused since the previous turn,
and an answer with the unread fence; the agent's notices and jig's typed
activity, routed where the core has `unreachable!` today; and the
fleet's check of a hello's bounds against the engine's charters. The
views mark a turn's text cut at a gap. This
session owns the hub's and the fleet's boundaries (decisions.md, 6):
jig 04, 06 and 08 use the fields it lands, and never edit those
boundaries; jig 22 to 24 carry them on jig's channel.

Paths carry jig 01's names (`crates/jig-domain-host/` was
`crates/jig-host/`, `crates/jig-domain-fleet/` was
`crates/jig-core-fleet/`, `crates/jig-domain-core/` was
`crates/jig-core/`, `crates/jig-domain-views/` was
`crates/jig-core-views/`, `crates/jig-domain-accounts/` was
`crates/jig-core-accounts/`, `crates/jig-domain-client/` was
`crates/jig-client/`, `crates/jig-view/` was `crates/jig-client-view/`).
Line numbers are as built at jig `1cdfeb62`; jig 01's renames may move
them by a few lines.

## Read first

- jig's `domain/hosts.md`, section 2 (the contract), 6.1 (the link),
  6.2 (hosted runs), 6.4 (turns), 6.5 (facts), 6.8 (smith's payloads),
  section 9 (messages and delivery) and section 11 (the world).
- jig's `domain/engine.md`, 7.1 (the assignment), section 8 (the fleet:
  "A host's agent must run the engine's charters"), section 11 ("A run's
  text as it is written").
- jig's `protocol/channel.md`, 4.3 (the hello), 5.1 to 5.3 (the
  records), 6.4 (facts and text), section 7 (names), section 8 (grants).
- smith's `domain/host.md`, sections 2 and 6 (the start, turns kept),
  and `protocol/limits.md`, 2.1 and 2.3 (the bounds a hello states).
- decisions.md, 6 and 14; later.md (a run's live text from workers).
- findings/domain.md, item 6 (6.0 to 6.3, 6(f), 6(g)), item 8 and its
  "Dependencies" and "Order and dependencies" sections.
- findings/protocol.md, 1.5 (the hub's boundary) and 6.1 (the old
  draft's session 07).

## What the design asks

- **One vocabulary toward the link** (hosts.md, 6.1): "section 2's
  whole, on a worker as in the engine: the hello as one value, which a
  worker's root assembles from the hub, its workspace, its agent
  capability's bounds and its link's graces; notices relayed from the
  agent; facts as outputs, best effort".
- **The assignment** (hosts.md, 2; engine.md, 7.1): "the start's
  messages (section 9); the run's identity; credential grants", each
  grant naming "whose table the protocol layer fills its value from".
  The hub carries the identity as bytes it never reads (hosts.md, 6.8).
- **Turns** (hosts.md, 2 and 6.4): "numbered within their attempt, each
  with its spend, the last message it read, whether it opens a window,
  and the messages refused since the run's previous turn, each by name
  with smith's reason". "No refusal crosses on its own: the hub keeps
  each, its name and smith's reason, with the run's next turn, or its
  answer if none follows".
- **The answer** (hosts.md, 2): "with the turns it took, what it spent,
  the last message it read (the unread fence, section 9), the messages
  refused since its last turn".
- **Notices** (hosts.md, 2): "relayed from the run's agent: a grant its
  provider rejected, an account exhausted until a time".
- **Activity** (hosts.md, 6.5): "carries no content: the run's phase;
  its long operations, each begun with what it is (its checks, an LLM
  completion, a command) and its span, and ended ...; and counts from
  smith's channel facts, each a kind, its time and a count".
- **The hello** (hosts.md, 2): "the host's slots, how long it may take
  to stop a run it lost contact for, the workstreams it holds, the runs
  it hosts and those whose turns or answers wait for acknowledgement;
  and its agent's bounds"; turns and answers "kept until acknowledged:
  sent at once, sent again after every hello" (6.4).
- **A relayed answer** (channel.md, 5.2): "the engine's answer: its
  text for the LLM and whether it is an error, or busy, to ask again
  after a backoff".
- **Lost contact** (hosts.md, 6.2 and 6.6; channel.md, 10.1): "The hub
  arms no timers"; past the grace, the hub cancels its runs for lost
  contact, on its root's word.
- **The fleet's check** (engine.md, 8): "The fleet refuses a hello, as
  not admitted and naming why, whose agent's bounds ... do not hold every
  run the engine may place there".
- **Text by retry** (engine.md, 11; hosts.md, 6.5): "a later retry
  replaces an earlier one's text, and since facts may be dropped, text
  that starts past what the engine holds of its retry marks that turn's
  text cut."

## Choices this plan makes

- The hub's link-facing messages are two enumerations, `LinkDown` and `LinkUp`, named as jig 22 to 25 use them; the hub's events from its agent, its workspace and its top level stay `Event` variants.
- The hello is `host::Hello`, assembled by the hub from what it hosts and what its root gives it at `Event::Opened`; the fleet's `Hello` and `Bounds` mirror it field for field.
- After a hello the root sends the hub's kept turns, then its kept answers, read by query (`turn_at`, `answer_at`), as turns are read today; the worker's link resends its waiting relays itself (jig 24).
- The hub's start is one struct, `Start`, so that a later field touches its constructor and its readers only.
- The core's part of the assignment stays `Now::RunPrepared`'s named fields: its two consumers name fields and the other matches use `..`; jig 04, 06 and 08 add theirs there and to the test root's `Assignment`.
- The fleet's `Assignment` stays two tokens: the start's messages and the identity are the root's assembly, which the fleet never reads.
- The fleet's grant vocabulary (a grant's name, a start's grants, an adoption's holds, a generation for each holder) lands here; jig 08 makes the fleet keep and send grants without editing that boundary.
- `ops`'s root fills the identity with zeros and the start's messages with none until jig 06 (identity) and jig 04 (messages) give them their sources.
- Refusals reach the core by message number only (`Box<[u64]>`); smith's reason stops at the root, which keeps it in its payload.
- The core's routes in 05.3 take the new turn and answer fields and keep today's behaviour (refusals dropped, as `Bounced => continue` drops them; the fence unchecked; the window flag unused) until jig 04 and jig 06 change those routes.
- A long operation shows on the run's chip as `Writing` for an LLM completion and `Active` for checks or a command; a count refreshes the observation's time and changes no activity.
- The engine's own host says its hello once, at construction, through its root; channel 0 stays registered from `engine_slots`.
- A run's text keeps entering the core as `core::Event::RunText`: no host in this plan gives text (later.md, "A run's live text and named calls from workers").

## Rules that bind it

- From code-organization.md, section 6: "No increment of this plan: ...
  adds a field or predicate that its counterpart across a boundary
  lacks; ... decodes or encodes bytes in a domain crate ...; links smith
  from `jig-domain-host`, the core or its children, or the channel's
  components; adds an `unreachable!` arm for a variant the reference
  service can reach; an arm the service reaches is routed;".
- From decisions.md, 6: "jig 05 changes the hub's and the fleet's
  boundaries in one session, so the sessions that use the new fields
  (jig 04, 06, 08) never edit them."
- From code-organization.md, section 4: "the core's vocabulary toward
  the root (`routing/vocabulary.rs`) | `jig-domain-core` | each session
  adds its variants; hotspot".
- For this session: a file already over 800 lines (named in each
  increment's Files) must not grow by more than 20 lines in an
  increment; new code goes in a module of its own (code-organization.md,
  section 7). `git diff --stat` in the commit body shows it.

## Increments

### 05.1 One vocabulary toward the link: two directions, the hello, answers kept, relayed answers typed

Scope: the hub's messages with the engine become two enumerations,
`LinkDown` (from the engine: an assignment, a message, a relayed answer,
a turn kept, the answer acknowledged, a cancel, a grant) and `LinkUp`
(toward the engine: a turn, a call, the answer), which the roots and the
channel's components (jig 22 to 24) translate; the hello is one value,
`Hello`, which the hub assembles when its root says the link is open,
from what it hosts and what the root gives it (its stop bound, its
workspace's workstreams, its agent's bounds), and which goes first; the
hub keeps each answer, and its run's slot, until the engine acknowledges
it, and the root sends kept turns and answers again after the hello; a
relayed answer is the engine's text and whether it is an error, or busy,
to ask again after a backoff (hosts.md, 2, 6.1, 6.2, 6.4, 6.6;
channel.md, 4.3, 5.1 to 5.3; findings/domain.md, 6.1, gaps 1 and 2). The
fleet's hello carries the agent's bounds, which 05.6 judges. Lost
contact stays the link's: the hub arms no timer, and past the grace its
root cancels each run for lost contact (`Event::CancelAll { reason:
Reason::Contact }`, as built).
Needs: nothing.
Files:
- `crates/jig-domain-host/src/boundary.rs` (675 lines): `Event`
  (:12-148: `Inbound` :14, `Assign` :33, `AcknowledgeTurn` :48, `Cancel`
  :61, `Grant` :65, `Relayed` :72, `Report` :91, `Unacknowledged`
  :96-98), `Request` (:152-262: `Relay` :153, `Answer` :187, `Turn`
  :194, `Hosting` :227-229), `Request::to_agent` (:378-401), `Reply`
  (:506), `Hosting` (:555-560), `Phase` (:564-574). New module
  `crates/jig-domain-host/src/link.rs` holds `LinkDown`, `LinkUp`,
  `Relayed`, `Hello`, `Bounds` and `ModelBound`.
- `crates/jig-domain-host/src/domain.rs` (`step`, :215-250;
  `unacknowledged`, :41-43; the queries `retained_turns` :168, `turn`
  :186, `turn_at` :192), `crates/jig-domain-host/src/hosted.rs` (1,340
  lines: `relayed` :320-345, `report` :395-403, `answer` :963-995,
  `refuse` :1230), `crates/jig-domain-host/src/turns.rs` (`admit`, the
  link copy it pushes), `crates/jig-domain-host/src/limits.rs`,
  `crates/jig-domain-host/src/lib.rs` (:28-32). New module
  `crates/jig-domain-host/src/answers.rs` keeps the answers.
- `jig_domain_host::Request`'s exhaustive matches (6):
  `examples/ops/domain/src/route.rs:1424-1520` (1,585 lines);
  `tests/host/src/engine.rs:188-221`; `tests/host/src/parent.rs:263-295`;
  `tests/host/src/turn_world.rs:470-480`; `tests/host/src/world.rs:725-780`
  (1,048 lines); `tests/host/tests/memory.rs:100-110`.
- The link-facing `Event` constructions and matches:
  `crates/jig-domain-host/src/tests.rs` (408 lines);
  `tests/host/src/fixtures.rs:6-12` (`assign`, `inbound`);
  `tests/host/src/turn_world.rs:150-191,305-345,399`;
  `tests/host/src/engine.rs:349,354`; `tests/host/tests/memory.rs:217`;
  `tests/host/src/world.rs:620-629,883,908`; `ops`'s `Held` arms
  (`examples/ops/domain/src/route.rs:249-317`) and its relayed answers
  (`examples/ops/domain/src/assemble.rs:230-242` `host_answer`,
  :497-512 `relayed`; `examples/ops/domain/src/route.rs:1379-1386`;
  `examples/ops/domain/src/tests.rs:477`); `ops`'s scripted agent, which
  reads a relayed answer's error mark (`tests/ops/src/conformance/scripts.rs`).
- Users of `Report`, `Hosting` and `Unacknowledged`:
  `examples/ops/domain/src/route.rs:317,1504`;
  `tests/host/src/engine.rs:207,223` (`reconnected`);
  `tests/host/src/parent.rs:288`;
  `tests/host/src/turn_world.rs:152,175,307,345,473`;
  `tests/host/src/world.rs:620,628,730,774,908`;
  `tests/host/tests/memory.rs:50,107,305-306`.
- The fleet: `Hello` (`crates/jig-domain-fleet/src/boundary.rs:475-484`),
  its 17 literals: `crates/jig-domain-fleet/src/tests.rs:125` (the
  helper `hello()`, :120-132), :412, :1240;
  `crates/jig-domain-core/src/tests.rs:1600,1629` (2,936 lines);
  `crates/jig-domain-core/src/tests/deployment_connectors.rs:141`;
  `tests/core/src/world.rs:431`; `tests/core/src/effects.rs:209`;
  `tests/core/src/peers.rs:61`; `tests/core/src/stories/instructions.rs:53`;
  `tests/core/tests/restart.rs:13`; `tests/core/tests/runs.rs:87`;
  `tests/fleet/src/world.rs:878`; `tests/fleet/src/turns.rs:289`;
  `tests/fleet/tests/host_kinds.rs:130,168`; `tests/fleet/tests/memory.rs:105`;
  the translators from a kit's hello, `tests/core/src/peers.rs:58-66`
  and `tests/fleet/src/world.rs:870-879`; `channel::hello`
  (`crates/jig-domain-fleet/src/channel.rs:99-181`, the destructure at
  :140).
Shape:
- `jig_domain_host::LinkDown`: `Assign { reply_to: ReplyTo, assignment: Assignment }`,
  `Message { run, attempt, name: Token, sender: Box<[u8]>, words: Box<[u8]> }`,
  `Relayed { run, attempt, call: Token, answer: Relayed }`,
  `TurnKept { run, attempt, turn: u32 }`, `Acknowledge { run, attempt }`,
  `Cancel { run, attempt }`, `Grant { run, attempt, grant: Grant }`
  (`run` and `attempt` are `Token`s); `Event::Down(LinkDown)` replaces
  `Event::{Inbound, Assign, AcknowledgeTurn, Cancel, Grant, Relayed}`
  and `Event::Unacknowledged`.
- `jig_domain_host::LinkUp`: `Turn { run, attempt, turn: Turn }`,
  `Call { run, attempt, call: Box<[u8]>, delivery: Token, tool: Box<[u8]>, writes: bool, input: Box<[u8]>, deadline: Duration }`,
  `Answer { reply_to: ReplyTo, run, attempt, answer: Answer }`;
  `Request::Up(LinkUp)` replaces `Request::{Turn, Relay, Answer}` (05.3
  changes `Turn` and `Answer`; 05.4 adds `Activity`, `Rejected` and
  `Exhausted`). The hub's requests to its agent, its workspace and its
  root (`CancelRelay`, `Prepare`, `Abort`, `Save`, `Release`) stay as
  built, and so does `Request::Bounced` until 05.3 removes it.
- `jig_domain_host::Relayed { Answer { error: bool, text: Box<[u8]> }, Busy }`;
  the agent hears `Reply::Relayed { error: bool, text: Box<[u8]> }`, or
  `Reply::Busy` for `Relayed::Busy`; `ops` builds `Relayed::Answer` from
  the core's `SettledAnswer::Host { error, body }`, and `host_answer`'s
  mark byte goes.
- `Event::Opened { stop_bound: Duration, workstreams: Box<[u64]>, bounds: Bounds }`
  replaces `Event::Report`: from the root once the link is open; the
  hub answers `Request::Hello { hello: Hello }`, with
  `Hello { slots: u32, stop_bound: Duration, workstreams: Box<[u64]>, hosting: Box<[Hosting]>, bounds: Bounds }`,
  `Phase::Answered` for a run whose answer waits, and
  `Bounds { models: Box<[ModelBound]>, turns: u32, spend: u64, time: Duration, waiting: Duration, checks: Duration, message_bytes: u32, inbox: u32, unacknowledged: u32 }`,
  `ModelBound { name: Box<[u8]>, window: u32, output: u32 }`
  (channel.md, 5.1).
- `LinkDown::Acknowledge`: the answer of that attempt is durable or not
  wanted; the hub forgets it and frees its slot.
  `Domain::kept_answers() -> u32` and
  `Domain::answer_at(index: u32) -> Option<LinkUp>` (a `LinkUp::Answer`)
  serve the resend after a hello, as `retained_turns` and `turn_at`
  (now giving `LinkUp::Turn`) do: the root sends the hello, then each
  kept turn, then each kept answer.
- `jig_domain_fleet::Hello` gains `bounds: Bounds`, and the fleet's
  `Bounds` and `ModelBound` mirror the hub's field for field; the fleet
  takes them and judges nothing until 05.6.
- `ops`'s root: its `Held` arms send `host::Event::Down(..)`
  (`Held::Acknowledge { run, attempt, .. }` becomes
  `LinkDown::Acknowledge { run, attempt }`); `host_out` matches
  `Request::Up(..)`; `Request::Hello` is taken and dropped (the engine's
  own hello is 05.6's).
- The core world's and the fleet world's hellos take
  `bounds: fixtures::bounds()`, from new fixtures modules
  `tests/core/src/fixtures.rs` and `tests/fleet/src/fixtures.rs`
  (code-organization.md, section 5), whose values hold each charter of
  the world's configuration; later sessions extend these modules.
Test shape:
- `tests/host/tests/turns.rs`, with `turn_world::World::reconnect`
  (:305, which sends `Event::Opened`), `commit_turn` (:330, which sends
  `LinkDown::TurnKept`) and `acknowledge_answer` (:339, which sends
  `LinkDown::Acknowledge`):
  `the_hello_carries_the_roots_stop_bound_workstreams_and_bounds`,
  `the_hello_lists_a_run_whose_answer_waits_as_answered`,
  `an_answer_is_sent_again_after_a_hello_until_acknowledged`,
  `an_unacknowledged_answer_keeps_its_runs_slot`,
  `a_relayed_error_reaches_the_agent_as_an_error`,
  `a_relayed_busy_reaches_the_agent_as_busy`. Each under 0.5 s.
- `tests/host/tests/memory.rs` keeps its worst case with the kept
  answers counted.
- `crates/jig-domain-fleet/src/tests.rs`: the helper `hello()` gains
  `bounds`; no new test (05.6 judges them).
Proof: world, `heavy cargo nextest run -p jig-host-world --test turns -E 'test(an_answer_is_sent_again_after_a_hello_until_acknowledged)'`,
once: an answer made while the link was down reaches the engine after
the next hello, the hello first, and is kept until acknowledged; under
0.5 s. (The hub world is the highest layer with a hub; jig's channel
comes with jig 22 to 24.)
Don't: arm a timer in the hub; tell the hub the link dropped; let the
hub choose its stop bound, workstreams or bounds (the root gives them);
move the requests to the agent or the workspace into `LinkUp`.
Size: at most 350 lines of production code.
Removes: `Event::Report`, `Event::Unacknowledged`, `Request::Hosting`,
`Domain::unacknowledged`; the link-facing `Event` and `Request`
variants, moved into `LinkDown` and `LinkUp`; `host_answer`'s mark byte.

### 05.2 The assignment's hub and fleet fields

Scope: the hub's assignment carries the run's identity (16 bytes it
never reads) and the start's messages, and refuses as invalid a start
whose messages are beyond its limits; the agent's start carries both;
grants, in the hub and in the fleet, name whose table their value comes
from; the fleet's boundary gets the grant vocabulary jig 08 fills: a
grant's name, the grants a start carries and those an adoption holds,
and a new generation for each attempt that holds its name (hosts.md, 2
and 6.8; engine.md, 7.1 and section 8, "Grants follow their attempts";
channel.md, 5.2, 5.3 and 8).
Needs: nothing.
Files:
- `crates/jig-domain-host/src/boundary.rs` (675 lines): `RunAssignment`
  (:406-417), `Assignment` (:420-427), `Request::Start` (:178-186),
  `ToAgent::Start` (:282-290), `Request::to_agent` (:382-383),
  `Invalid` (:597-602), `Grant` (:671-675).
- `crates/jig-domain-host/src/hosted.rs` (1,340 lines): `assign`
  (:116-131, the destructure at :123), `assign_runtime` (:174, the
  destructure at :202), `start_prepared` (:409-450).
- `crates/jig-domain-host/src/assignment.rs` (28 lines, the admission
  check at :8), `crates/jig-domain-host/src/limits.rs` (`Limits`, :8-50;
  `worst_case`), `crates/jig-domain-host/src/lib.rs` (re-exports, :28-32).
- `crates/jig-domain-fleet/src/boundary.rs` (615 lines): `Grant`
  (:95), `Event::Start` (:137), `Event::Adopt` (:160), `Event::Grant`
  (:194), `Request::Grant` (:328); `crates/jig-domain-fleet/src/domain.rs`
  (:301, :308, :321); `call::grant` (`crates/jig-domain-fleet/src/call.rs:200`).
- `fleet::Event::Start` constructions: `examples/ops/domain/src/route.rs:531`;
  `testing/jig-test-domain/src/routes.rs:208`;
  `crates/jig-domain-core/src/tests/chip_phase.rs:82`;
  `crates/jig-domain-fleet/src/tests.rs:135,238`;
  `crates/jig-domain-fleet/src/tests/progress.rs:93`;
  `tests/fleet/src/world.rs:1093`; `tests/fleet/tests/host_kinds.rs:40,111,182`.
- `fleet::Event::Adopt` constructions: `crates/jig-domain-core/src/sibling_routes.rs:169`;
  `crates/jig-domain-fleet/src/tests.rs:146,1203,1214`;
  `tests/fleet/src/turns.rs:303`; `tests/fleet/src/world.rs:1271`;
  `tests/fleet/tests/host_kinds.rs:72,88`.
- `fleet::Event::Grant` matches: `crates/jig-domain-fleet/src/domain.rs:321`,
  `crates/jig-domain-core/src/routing/children.rs:371`,
  `tests/fleet/src/world.rs:1302`.
- `jig_domain_host::Grant` literals: `examples/ops/domain/src/assemble.rs:104`;
  `tests/host/tests/memory.rs:136`; `tests/host/src/turn_world.rs:491`
  (a parameter).
- `RunAssignment` constructions (7): `crates/jig-domain-host/src/tests.rs:79`;
  `tests/host/src/turn_world.rs:154,176`; `tests/host/src/engine.rs:287`;
  `tests/host/tests/memory.rs:134`; `tests/host/src/world.rs:549`
  (1,048 lines); `examples/ops/domain/src/assemble.rs:98`.
- `Assignment` literals (9): `crates/jig-domain-host/src/tests.rs:144,387`;
  `tests/host/src/fixtures.rs:7` (`assign`); `tests/host/src/turn_world.rs:186`;
  `tests/host/src/engine.rs:295`; `tests/host/tests/memory.rs:159,295,370`;
  `examples/ops/domain/src/assemble.rs:97`.
- `Request::Start` and `ToAgent::Start`: the full destructure at
  `tests/host/src/turn_world.rs:416`; `..` matches at
  `crates/jig-domain-host/src/tests.rs:68,106,146,360`,
  `tests/host/src/parent.rs:272`, `tests/host/src/world.rs:829`,
  `tests/host/tests/memory.rs:103`, `tests/ops/src/conformance/scripts.rs:94`,
  `examples/ops/domain/src/tests.rs:372` (835 lines),
  `examples/ops/domain/src/route.rs:1516` (1,585 lines).
- `Invalid`'s matches: `crates/jig-domain-host/src/tests.rs:188`,
  `tests/host/src/engine.rs`.
- The assignment hotspot: `examples/ops/domain/src/assemble.rs`
  (`assignment`, :48-112).
Shape:
- `jig_domain_host::RunAssignment` gains `identity: [u8; 16]`.
- `jig_domain_host::Assignment` gains `messages: Box<[StartMessage]>`,
  with `StartMessage { name: Token, sender: Box<[u8]>, words: Box<[u8]> }`,
  oldest first.
- `jig_domain_host::Start { workspace: Option<Token>, charter: Box<[u8]>,
  activation: u64, identity: [u8; 16], turns: Box<[Box<[u8]>]>,
  answered: Box<[AnsweredCall]>, messages: Box<[StartMessage]>,
  grants: Box<[Grant]> }`; `Request::Start { owner: Token, start: Box<Start> }`
  and `ToAgent::Start { owner: Token, start: Box<Start> }`.
- `jig_domain_host::Invalid::Messages`: more messages than
  `Limits::inbox`, or one whose sender and words exceed
  `Limits::event_bytes`.
- `jig_domain_host::Limits::inbox: u32`, doc "Messages a run's inbox
  takes: the most a start carries, and the most refusals a turn or an
  answer carries (05.3)"; `worst_case` counts the start's messages.
- `GrantTable { Accounts, Application }` in both crates;
  `jig_domain_host::Grant` and `jig_domain_fleet::Grant` gain
  `table: GrantTable`. Today's grants are `GrantTable::Accounts`.
- The fleet's grant vocabulary, as jig 08's sketch names it:
  `jig_domain_fleet::GrantName { table: GrantTable, account: u32 }`;
  `Event::Start` gains `grants: Box<[Grant]>`; `Event::Adopt` gains
  `holds: Box<[GrantName]>` and `grants: Box<[Grant]>`; `Event::Grant`
  becomes `Grant { grant: Grant }`, a new generation for each live
  attempt that holds its name, replacing the per-attempt
  `Grant { run, attempt, grant }` that nothing sends; `Request::Grant`
  is unchanged. Until jig 08's grants pushed and resent, the fleet keeps
  none of them and drops `Event::Grant`, the roots and the core pass
  empty lists, and `call::grant` goes.
- `ops`'s `assemble::assignment` sets `identity: [0; 16]` (removed by
  jig 06's identity from the conversation's first attempt) and
  `messages: Box::new([])` (removed by jig 04's start's messages, which
  also removes `offered_inbox`, `examples/ops/domain/src/route.rs:341-356`).
Test shape:
- `tests/host/src/fixtures.rs`: `assign` (:6) fills `messages` empty;
  a new `assign_with(reply_to, assignment, messages)` fills them;
  `tests/host/src/turn_world.rs`'s `StartedState` keeps the start's
  `identity` and `messages`.
- `tests/host/tests/turns.rs` (existing kind; fixtures
  `turn_world::World::assign_conversation`, :174):
  `a_start_carries_its_messages_before_any_later_message`,
  `a_start_carries_its_identity_unchanged`,
  `a_start_beyond_the_inbox_is_refused_as_invalid_messages`. Each under
  0.5 s.
- `crates/jig-domain-host/src/tests.rs` (408 lines):
  `a_grant_keeps_its_table_through_the_start`. Under 20 ms.
Proof: world, `heavy cargo nextest run -p jig-host-world --test turns`,
once: an agent starts with the messages and identity its assignment
named, and a start carrying more messages than the run's inbox is
refused; under 0.5 s per test. (The hub world is the highest layer with
a hub; jig's channel comes with jig 22 to 24.)
Don't: read the identity's bytes in the hub; change the fleet's
`Assignment` tokens or `Now::RunPrepared`; keep or send a grant in the
fleet (jig 08 does).
Size: at most 170 lines of production code.
Removes: the per-attempt `fleet::Event::Grant { run, attempt, grant }`
and `call::grant` (the zero identity, the empty start in `ops` and the
fleet's empty grant lists are named above with the increments that
remove them).

### 05.3 Turns and answers carry refusals, the window flag and the fence

Scope: a turn says whether it opens a window and carries the messages
refused since the run's previous turn; the answer carries the last
message the run read and the refusals since its last turn; the hub keeps
each refusal, its own and its agent's, for the next turn or the answer,
and resends it with them after a hello; nothing crosses for a refusal on
its own (hosts.md, 2, 6.4, section 9; channel.md, 5.1 and 5.3;
findings/domain.md, 6.3 and 6(f)). The roots carry the new fields to
the core's events, whose routes keep today's behaviour until jig 04 and
jig 06.
Needs: nothing.
Files:
- `crates/jig-domain-host/src/boundary.rs`: `Event::Bounced` (:118),
  `Event::Finished` (:54), `Request::Bounced` (:219-224, :394),
  `FromAgent::Bounced` (:338-342), `FromAgent::Finished` (:328-333),
  `Event::from_agent` (:359-372), `Turn` (:449-455), `Answer` (:464-469),
  `Bounce` (:545-552).
- `crates/jig-domain-host/src/hosted.rs`: the five producers of
  `Request::Bounced` (:260, :284, :600-614 `bounced`, :1000 `hold`,
  :1012-1017 `bounce_held`), `answer` (:963-995), `next_answer`
  (:1226), `turned` (:1249-1297), `finished` (:1319-1340). New module
  `crates/jig-domain-host/src/refusals.rs` keeps each run's list.
- `crates/jig-domain-host/src/turns.rs` (`copy`, :170-172),
  `crates/jig-domain-host/src/domain.rs` (:226), `crates/jig-domain-host/src/lib.rs`.
- `Request::Bounced` consumers (9): `crates/jig-domain-host/src/tests.rs:333,364`;
  `tests/host/src/engine.rs:196`; `tests/host/src/parent.rs:287`;
  `tests/host/src/turn_world.rs:476`; `tests/host/src/world.rs:728,773`;
  `tests/host/tests/memory.rs:106` (`Asked::Bounced` :49, :276);
  `examples/ops/domain/src/route.rs:1491-1502`.
- `jig_domain_host::Turn` literals (5): `crates/jig-domain-host/src/turns.rs:171`;
  `tests/host/src/turn_world.rs:218,548`; `tests/host/tests/memory.rs:394`;
  `tests/ops/src/conformance/scripts.rs:216`; and
  `crates/jig-domain-host/src/tests.rs:154`.
- `Event::Finished`/`FromAgent::Finished` producers (8):
  `crates/jig-domain-host/src/boundary.rs:365`;
  `crates/jig-domain-host/src/tests.rs:397`; `tests/host/src/fixtures.rs:23`
  (`finished`); `tests/host/src/turn_world.rs:269`;
  `tests/host/tests/memory.rs:398`; `tests/ops/src/conformance/scripts.rs:139,176,187,242`;
  consumer `tests/host/src/world.rs:600`.
- `jig_domain_host::Answer` readers: `crates/jig-domain-host/src/tests.rs:126,188,209,312,338`;
  `tests/host/src/engine.rs` (`ending`, :397); `examples/ops/domain/src/route.rs:1462-1476`.
- The fleet: `Event::Bounced` (:259-265), `Request::Bounced`
  (:446-451), `Bounce` (:531-538) in
  `crates/jig-domain-fleet/src/boundary.rs`; `call::bounced`
  (`crates/jig-domain-fleet/src/call.rs:163-179`);
  `crates/jig-domain-fleet/src/domain.rs:351-353`;
  `crates/jig-domain-fleet/src/lib.rs:68`;
  `crates/jig-domain-fleet/src/tests.rs:8,626,736,751-752,798-815`;
  `tests/fleet/src/world.rs:367,907,1022,1314` (1,320 lines);
  `tests/fleet/src/workers.rs:44`; `tests/fleet/src/turns.rs:265`;
  `tests/fleet/tests/memory.rs:305`.
- The core (hotspot `crates/jig-domain-core/src/routing/vocabulary.rs`):
  `Event::TurnPayload` (:425), `Event::AnswerPayload` (:427-435),
  `Event::AcceptedTurn` (:437-445); the arms in
  `crates/jig-domain-core/src/routing/mod.rs` (:347, :469, and the
  `TurnPayload`/`AnswerPayload` arms);
  `crates/jig-domain-core/src/routing/runs.rs` (784 lines:
  `route_accepted_turn` :681, `route_answer_payload` :721,
  `route_turn_payload` :760); `crates/jig-domain-core/src/routing/children.rs`
  (the `fleet::Request::Bounced` arm, :284-286, and :377);
  `crates/jig-domain-core/src/tests/activity.rs` (5 sites, :30);
  `tests/core/tests/effect_fences.rs` (2 sites).
- `ops`'s root: `Payload::Turn` and `Payload::Answer`
  (`examples/ops/domain/src/boundary.rs:170-171`), their sites
  `examples/ops/domain/src/route.rs:274,1269,1278-1279,1297,1302,1314,1316,1438,1454,1466`
  and `examples/ops/domain/src/assemble.rs:499-500`.
- The test application's root (1,154 and 1,005 lines):
  `root::Event::Turn` and `Answer`
  (`testing/jig-test-domain/src/lib.rs:244-260`, handled :594-599),
  `Payload` (:518), `testing/jig-test-domain/src/routes.rs:453-500`;
  their constructions (7): `tests/core/src/effects.rs:1217,1230`;
  `tests/core/src/peers.rs:82`; `tests/core/src/world.rs:544,645`;
  `tests/core/tests/projections.rs:280`; `tests/core/tests/store.rs:76`;
  `tests/core/tests/fuzzy_watches.rs:30`.
- `testing/jig-fake-workers/src/lib.rs`: `Up::Turn` (:107) and
  `Up::Answer` (:111), their translation at `tests/core/src/peers.rs:82-97`.
- `tests/core/tests/runs.rs`: `a_bounced_message_stays_unread_for_a_later_read`
  (:214-229) and its call at :244.
Shape:
- `jig_domain_host::MessageRefusal { TooLarge, Full, NameInUse, Ending }`
  replaces `Bounce`; `Refused { name: Token, reason: MessageRefusal }`.
- `jig_domain_host::Turn { turn: u32, spent: u64, read: Option<Token>,
  opens_window: bool, refused: Box<[Refused]>, body: Box<[u8]> }`: an
  agent's turn comes with `refused` empty (a non-empty one faults the
  run, `AgentFailure::Rules`); the hub moves its kept list into it as it
  stages it (`turned`), and `turns.rs` keeps and resends it with the turn.
- `jig_domain_host::Answer { turns: u32, spent: u64, read: Option<Token>,
  refused: Box<[Refused]>, ending: Ending }`; `Event::Finished` and
  `FromAgent::Finished` gain `read: Option<Token>`.
- `FromAgent::Refused { owner: Token, name: Token, reason: MessageRefusal }`
  and `Event::Refused` replace `Bounced`; `Request::Bounced` goes; the
  five producers push to the run's list
  (`refusals::keep(&mut Refusals, run, attempt, Refused)`), bounded by
  `Limits::inbox`; one past it is dropped and counted
  (`Domain::refusals_lost`).
- The fleet loses `Event::Bounced`, `Request::Bounced`, `Bounce` and
  `call::bounced`.
- `ops`'s `Payload::Turn` gains `opens_window: bool, refused: Box<[(u64, host::MessageRefusal)]>`;
  `Payload::Answer` gains `read: Option<u64>, refused: Box<[(u64, host::MessageRefusal)]>`.
- The test root's `Event::Turn` and `Payload::Turn` gain
  `opens_window: bool, refused: Box<[u64]>`; `Event::Answer` and
  `Payload::Answer` gain `read: Option<u64>, refused: Box<[u64]>`.
- The core: `Event::TurnPayload { .., refused: Box<[u64]> }`;
  `Event::AnswerPayload { .., read: Option<u64>, refused: Box<[u64]> }`;
  `Event::AcceptedTurn { .., opens_window: bool }`. In this increment
  `route_turn_payload` drops `refused` (jig 04's refusals committed with
  turns takes it), `route_answer_payload` drops `read` and `refused`
  (jig 04), and `route_accepted_turn` drops `opens_window` (jig 06's
  current window takes it); the `fleet::Request::Bounced` arm goes.
- `jig_fake_workers::Up::Turn` gains `opens_window: bool` (its first
  turn of an assignment with an empty transcript) and `Up::Answer` gains
  `read: Option<u64>` (its last turn's read); `tests/core/src/peers.rs`
  passes both and `refused: Box::new([])` (jig 04 scripts refusals).
- The core world's `World::turn` (`tests/core/src/effects.rs:1215`)
  sets `opens_window` for turn 1 of an assignment whose transcript is
  empty, and `refused` empty; `World::end` (:1228) passes the last
  turn's read.
- `ops`'s scripted agent (`tests/ops/src/conformance/scripts.rs:216`)
  sets `opens_window` on the first turn of a fresh start.
Test shape:
- `tests/host/tests/turns.rs` (existing kind), with
  `turn_world::World` (`tests/host/src/turn_world.rs:90`, `turn` :215,
  `reconnect` :305, `commit_turn` :330): new world helpers
  `refuse(name, reason)` (the agent's `FromAgent::Refused`) and
  `relay_message(name)`. Tests:
  `a_refusal_rides_with_the_runs_next_turn`,
  `a_refusal_with_no_later_turn_rides_with_the_answer`,
  `a_refusal_is_resent_with_its_turn_after_a_hello`,
  `the_hubs_own_full_refusal_rides_with_the_next_turn`,
  `the_answer_names_the_last_message_its_run_read`,
  `a_window_turn_crosses_with_its_flag`. Each under 0.5 s.
- The hub world's referee (`tests/host/src/world.rs`, the check named in
  `tests/host/src/lib.rs:30-32`) gains: "each refusal inside a turn or
  answer no later than the first whose fence passes it" (hosts.md, 11).
- `tests/core/tests/runs.rs`: `a_bounced_message_stays_unread_for_a_later_read`
  asserts the opposite of the design (findings/domain.md, "Defects
  found on the way") and its `fleet::Event::Bounced` goes: it is removed
  here, with its call in `runs_scenarios` (:244); jig 04 writes its
  replacement.
- `crates/jig-domain-fleet/src/tests.rs`: the bounce tests
  (:798-815) go with `Event::Bounced`.
Proof: world, `heavy cargo nextest run -p jig-host-world --test turns`,
once: a message the agent refused crosses inside the run's next turn, or
its answer when no turn follows, and again after a hello; under 0.5 s per
test.
Don't: send a refusal on its own; read a turn's body or the identity in
the hub; change what the core does with a turn's read messages (jig 04
does).
Size: at most 300 lines of production code.
Removes: `Request::Bounced`, `Event::Bounced`, `FromAgent::Bounced`,
`Bounce` (hub and fleet), `call::bounced`, the core's `Bounced` arm,
`a_bounced_message_stays_unread_for_a_later_read`.

### 05.4 Notices and typed activity, routed

Scope: the hub relays its agent's notices (a grant its provider
rejected, an account exhausted until a time) for live runs; activity is
jig's typed vocabulary end to end (phase, long operation begun with its
kind and span and ended, counts), never smith's bytes; `ops`'s root
translates both to the fleet; the core routes the fleet's `Rejected` and
`Exhausted` to the accounts child and the fleet's opaque `Told` goes,
replacing three of the four arms of the `unreachable!` at
`crates/jig-domain-core/src/routing/children.rs:287-290` (hosts.md, 2,
6.5, 6.8; channel.md, 6.4; findings/domain.md, 6.2 and 6(g)).
Needs: nothing.
Files:
- `crates/jig-domain-host/src/boundary.rs`: `Event::Facts` (:43-46),
  `FromAgent::Facts` (:324-327, :364), new notice variants;
  `crates/jig-domain-host/src/facts.rs` (`Told` :40-45, `AgentFacts`
  :48-74), `crates/jig-domain-host/src/hosted.rs` (`told`, :1312-1317;
  the lifecycle pushes at :219, :279, :414, :492, :558, :623, :975,
  :979, :983), `crates/jig-domain-host/src/domain.rs` (:156 `pop_told`,
  :224), `crates/jig-domain-host/src/limits.rs` (`told`, `fact_bytes`,
  :45-47, :90-91), `crates/jig-domain-host/src/lib.rs:34`.
- `jig_domain_host::LinkUp`'s exhaustive matches (05.1 moves them from
  `Request`'s): `examples/ops/domain/src/route.rs:1424-1520`;
  `tests/host/src/engine.rs:188-221`; `tests/host/src/parent.rs:263-295`;
  `tests/host/src/turn_world.rs:470-480`; `tests/host/src/world.rs:725-780`;
  `tests/host/tests/memory.rs:100-110`.
- `Facts`/`Told` users: `crates/jig-domain-host/src/tests.rs:287-299`;
  `tests/host/src/turn_world.rs:223,529,533`; `tests/host/src/world.rs:537,625`.
- The fleet: `Event::Told` (:274-279), `Request::Told` (:460-464),
  `call::told` (`crates/jig-domain-fleet/src/call.rs:181-192`),
  `crates/jig-domain-fleet/src/domain.rs:357`;
  `crates/jig-domain-fleet/src/progress.rs` (`Activity`, `Report`,
  `received`, :11-122); `crates/jig-domain-fleet/src/tests/progress.rs`
  (103 lines); `tests/fleet/src/world.rs:916,1029,1315`;
  `tests/fleet/src/turns.rs:267`; `tests/fleet/tests/memory.rs:160-164,312`.
- The core: `crates/jig-domain-core/src/routing/children.rs` (:286-290,
  :378-379); `crates/jig-domain-core/src/documents/chip_phase.rs`
  (:178-190); `crates/jig-domain-core/src/tests/chip_phase.rs:103-140`.
- The client's joint world over the fleet's activity:
  `tests/client/src/joint/stories.rs:25-29,183`,
  `tests/client/src/joint/wire/chips.rs:205-209`.
- `ops`'s root: `host_out` (`examples/ops/domain/src/route.rs:1411-1525`)
  and a drain of the hub's activity beside it; `ops`'s scripted agent
  (`tests/ops/src/conformance/scripts.rs`, `poll` :165-253).
Shape:
- `jig_domain_host::FromAgent::Rejected { owner: Token, account: u32, generation: u64 }`
  and `FromAgent::Exhausted { owner: Token, account: u32, retry_after: Duration }`,
  with their `Event`s; `LinkUp::Rejected { run: Token, attempt: Token, account: u32, generation: u64 }`
  and `LinkUp::Exhausted { run: Token, attempt: Token, account: u32, retry_after: Duration }`,
  sent for a live run only.
- `jig_domain_host::Activity { Phase(Phase), Long { operation: Operation, span: Duration }, LongDone,
  Count { kind: CountKind, elapsed: Duration, count: u64 } }`,
  `Operation { Checks, Completion, Command }`,
  `CountKind { Admitted, Ended, LlmStarted, LlmRetried, LlmFinished, ToolStarted, ToolFinished, CheckStarted, CheckFinished, TextArrived }`
  (jig's own, as smith's channel facts name them; the hub links no
  smith). `FromAgent::Activity { owner, activity }` and `Event::Activity`
  replace `Facts`; `LinkUp::Activity { run: Token, attempt: Token, activity: Activity }`,
  drained by `Domain::pop_activity() -> Option<LinkUp>`, replaces `Told`
  and `pop_told`; the hub also keeps an `Activity::Phase` at each phase
  change; `Limits::fact_bytes` goes.
- `jig_domain_fleet::Activity` gains `Long { operation: Operation, span: Duration }`
  (`jig_domain_fleet::Operation`, the same three);
  `Report` gains `Count { kind: CountKind, elapsed: Duration, count: u64 }`,
  which refreshes the observation's `reported_at` and changes no
  activity; `Event::Told`, `Request::Told` and `call::told` go.
- The core: `fleet::Request::Rejected { account, generation, .. }` routes
  to `accounts::Event::Rejected { account, generation }` and
  `fleet::Request::Exhausted { account, retry_after, .. }` to
  `accounts::Event::Exhausted { account, retry_after }`; the arm
  `fleet::Request::Grant { .. } => unreachable!(..)` stays, removed by
  jig 08's grants pushed and resent.
- `chip_phase`: `Activity::Long { operation: Completion, .. }` is
  `ChipWork::Writing`; `Checks` and `Command` are `ChipWork::Active`.
- `ops`'s root: `LinkUp::Rejected`/`Exhausted` become
  `fleet::Event::Rejected`/`Exhausted` on channel 0; after each hub step
  it drains `pop_activity` into `fleet::Event::Observe` (phase and long
  operations as `Report::Activity`, counts as `Report::Count`).
Test shape:
- `tests/host/tests/turns.rs`, with `turn_world::World` and the agent
  capability in `tests/host/src/parent.rs` (`take` :263, `wake` :295):
  `a_rejected_grant_reaches_the_link_for_a_live_run`,
  `an_exhausted_account_reaches_the_link_with_its_wait`,
  `a_notice_of_a_fenced_attempt_is_dropped`,
  `a_long_operation_reaches_the_link_typed`,
  `activity_past_the_queue_is_dropped_and_counted`. Each under 0.5 s.
- `crates/jig-domain-fleet/src/tests/progress.rs`:
  `a_long_completion_is_observed_as_its_kind_and_span`,
  `a_count_refreshes_the_report_time_only`. Each under 20 ms.
- `tests/core/tests/effects.rs` (4 tests), with
  `World::host_call` (`tests/core/src/effects.rs:1239`) as the pattern
  for feeding the fleet: `a_rejected_grant_reaches_the_accounts_child`.
  Under 0.5 s.
Proof: world, `heavy cargo nextest run -p jig-core-world --test effects -E 'test(a_rejected_grant_reaches_the_accounts_child)'`,
once: a host's notice that a provider rejected a grant reaches the
accounts child through the fleet instead of crashing the core; under
0.5 s.
Don't: decode smith's facts anywhere in jig's domains (the worker's
hand-out does, jig 24 and 25); show a long operation as running a tool
(the chip's tool needs a call's name).
Size: at most 300 lines of production code.
Removes: `FromAgent::Facts`, `Event::Facts`, `Told`'s bytes,
`Limits::fact_bytes`, the fleet's `Told` path, the `Rejected`,
`Exhausted` and `Told` arms of the core's `unreachable!`.

### 05.5 A gap in a retry's text marks its turn cut

Scope: text that starts past what the views hold of its retry, because
facts were dropped between, keeps the new text from its offset and marks
that turn's text cut; a new turn starts uncut; a later retry still
replaces an earlier one's text (engine.md, 11; hosts.md, 6.5;
channel.md, 6.4; findings/domain.md, item 8, gap 1).
Needs: nothing.
Files:
- `crates/jig-domain-views/src/text.rs` (`Run`, :30-39; `written`,
  :50-157, the `return` at :114-116; `offer` :265-296; `writing`
  :334-349; the module doc at :7).
- `crates/jig-domain-views/src/tests/completion_retry.rs` (`report` :6,
  `expected` :12; the assertion at :102).
- `tests/views/src/referee.rs` (:152-157), `tests/views/src/world.rs`
  (`text`, :121).
Shape:
- `text::Run` gains `cut: bool`: set when a report starts past `run.to`
  of the same `TurnName`, and when the retained suffix is trimmed past
  its bound; cleared by a new `TurnName`; or-ed into `Request::Stream`'s
  `cut` (:288-296) and `Writing`'s `cut` (:342-349).
- A report starting past `run.to` replaces the retained text with the
  new one from its offset (`run.from = offset`), instead of returning.
Test shape:
- `crates/jig-domain-views/src/tests/completion_retry.rs`:
  `an_unwatched_suffix_keeps_the_original_byte_bound_and_overlap_fence`
  (:97) asserts the opposite of the design at :102 (`report(&mut domain,
  1, 21, b"gap").is_empty()`): that line becomes the new test
  `text_past_a_gap_restarts_there_and_marks_the_turn_cut`, and the
  original test keeps its other assertions. New:
  `a_new_turn_after_a_cut_one_starts_uncut`. Each under 20 ms.
- `tests/views/src/world.rs`: new `World::drop_text(run, turn, bytes)`,
  which advances the source's offset (`source`, :122) without sending,
  as a dropped fact; the referee (`tests/views/src/referee.rs:155`)
  drops "the scripted host writes consecutive offsets" and expects
  `cut` on the stream that crosses a dropped stretch.
- `tests/views/tests/completion_retry.rs`:
  `a_dropped_fact_marks_the_watched_turn_cut`. Under 0.5 s.
Proof: world, `heavy cargo nextest run -p jig-views-world --test completion_retry`,
once: a watcher sees a run's text resume past a dropped stretch, marked
cut; under 0.5 s per test.
Don't: route text through the hub or the fleet (later.md); change
`TurnName`.
Size: at most 30 lines of production code.
Removes: the `else if offset > run.to { return; }` in `text::written`;
the referee's consecutive-offsets assertion.

### 05.6 The fleet admits a host only if its agent holds the engine's charters

Scope: the fleet refuses a hello, as not admitted and naming why, whose
agent's bounds do not hold every run the engine may place on a host of
its kind: a model a charter it may run names missing, or held with a
smaller window or output; turns, spend, time or waiting per run smaller
than the deployment's most for a run, or a charter's; a checks' timeout
longer than the reserve's time of a charter that allows a change; a
largest message smaller than the deployment's message bound, or an inbox
that holds fewer than a start carries; fewer turns kept unacknowledged
than the hub's bound (engine.md, 8; channel.md, 4.3). The engine's own
host says its hello once, at construction, and a refusal fails the
engine's start. The deployment's document shows the reason.
Needs: jig 07's charter table (`core::Settings::charters`, each
charter with its models, host kinds (`Charter::hosts`), turns, time,
waiting and reserve); jig 04's start count (the tasks child's
`Limits::run_inbox`).
Files:
- `crates/jig-domain-fleet/src/history.rs` (`HelloRefusal`, :8-15),
  `crates/jig-domain-fleet/src/channel.rs` (`hello`, :99-181),
  `crates/jig-domain-fleet/src/domain.rs` (`Domain::new`, :60-80),
  `crates/jig-domain-fleet/src/limits.rs`, new module
  `crates/jig-domain-fleet/src/needs.rs` (the check),
  `crates/jig-domain-fleet/src/lib.rs`.
- `crates/jig-domain-core/src/lib.rs` (`Core::new`, :575; `Config`,
  :86-103), and the module that builds the fleet's needs from jig 07's
  charter table.
- `HelloRefusal`'s users (about 30 sites in 11 files):
  `crates/jig-domain-fleet/src/channel.rs` (3),
  `crates/jig-domain-fleet/src/tests.rs` (2);
  `crates/jig-domain-core/src/documents/inbox_deployment.rs:169-174,577`
  (830 lines); `crates/jig-domain-core/src/tests.rs:1657`;
  `crates/jig-domain-core/src/tests/deployment_connectors.rs:169`;
  `crates/jig-domain-client/src/contract/owner_deployment.rs:97-102,200`
  (`OwnerHelloRefusal`); `crates/jig-view/src/owner_deployment.rs:135-156`
  and its phrases in `crates/jig-view/src/phrases/` (`jig.rs`, 2,994
  lines); `crates/jig-view/src/tests/owner_deployment.rs` (4);
  `testing/jig-scripted-engine/src/owner_deployment.rs:114` and its
  tests (3); `tests/client/src/joint/wire/deployment.rs:170-173` and its
  tests (6).
- `fleet::Domain::new`'s callers: `crates/jig-domain-core/src/lib.rs:575`,
  `crates/jig-domain-fleet/src/tests.rs`, `tests/fleet/src/world.rs`.
- `ops`'s root: its construction
  (`examples/ops/domain/src/domain.rs:80-130`, the slots assertion at
  :92) sends its hub's hello on channel 0.
Shape:
- `jig_domain_fleet::Needs { charters: Box<[CharterNeeds]>, spend: u64,
  message_bytes: u32, start_messages: u32, unacknowledged: u32 }` and
  `CharterNeeds { charter: u32, kinds: Kinds, models: Box<[ModelNeed]>,
  turns: u32, time: Duration, waiting: Duration, change_reserve: Option<Duration> }`,
  `ModelNeed { name: Box<[u8]>, window: u32, output: u32 }`; the core
  builds them from `Settings::charters` (each charter's main model and
  alternatives, and its `hosts`),
  `authority.rules().maximum_run_spend`, `tasks.message_bytes`,
  `tasks.run_inbox` and the hub's `Limits::turns`.
- `jig_domain_fleet::Domain::new(limits: &Limits, needs: Needs)`.
- `HelloRefusal::Agent { why: AgentShortfall }`, with
  `AgentShortfall { Model { charter: u32, model: u32 }, Window { charter: u32, model: u32 },
  Output { charter: u32, model: u32 }, Turns, Spend, Time, Waiting, Checks, Message, Inbox, Unacknowledged }`
  (`Copy`; a model by its position in its charter: 0 the main, then the
  alternatives).
- `channel::hello` judges `hello.bounds` against the charters whose
  `kinds` allow `HostKind::Worker`; on channel 0, the first hello is the
  engine's own, judged against those allowing `HostKind::Engine`, and a
  refusal is `Request::Refuse { channel: Token::new(0) }`, on which
  `ops`'s root stops; later hellos on channel 0 are refused
  `ReservedChannel` as built.
- `OwnerHelloRefusal::Agent { why: OwnerAgentShortfall }` in the
  client's contract, mirrored field for field, with a phrase per
  shortfall in `jig-view`.
Test shape:
- New fleet step module `crates/jig-domain-fleet/src/tests/needs.rs`,
  declared in `crates/jig-domain-fleet/src/tests.rs`, with its harness
  (`Harness::new`, the `hello()` helper at :120-132, `LIMITS`, `C1`).
  Reference test:

  ```rust
  use super::*;

  fn needs_window(window: u32) -> Needs {
      Needs {
          charters: Box::new([CharterNeeds {
              charter: 1,
              kinds: Kinds::Workers,
              models: Box::new([ModelNeed { name: Box::from(&b"model"[..]), window, output: 1024 }]),
              turns: 8,
              time: Duration::from_secs(60),
              waiting: Duration::from_secs(10),
              change_reserve: None,
          }]),
          spend: 100,
          message_bytes: 64,
          start_messages: 4,
          unacknowledged: 2,
      }
  }

  fn bounds_window(window: u32) -> Bounds {
      Bounds {
          models: Box::new([ModelBound { name: Box::from(&b"model"[..]), window, output: 1024 }]),
          turns: 8,
          spend: 100,
          time: Duration::from_secs(60),
          waiting: Duration::from_secs(10),
          checks: Duration::from_secs(5),
          message_bytes: 64,
          inbox: 4,
          unacknowledged: 2,
      }
  }

  #[test]
  fn a_worker_whose_model_window_is_smaller_is_refused_naming_it() {
      let mut h = Harness::with_needs(LIMITS, needs_window(8192));
      let hello = Hello {
          stop_bound: Duration::from_nanos(1),
          slots: 2,
          workstreams: Box::new([]),
          hosting: Box::new([]),
          bounds: bounds_window(4096),
      };
      assert_eq!(&*h.step(Event::Hello { channel: C1, hello }), &[Request::Refuse { channel: C1 }]);
      assert_eq!(
          h.domain.hello_refusals().get(0).unwrap().why,
          HelloRefusal::Agent { why: AgentShortfall::Window { charter: 1, model: 0 } }
      );
  }
  ```

  with `Harness::with_needs(limits, needs)` added beside
  `Harness::new`. Other tests there:
  `a_worker_lacking_a_charters_model_is_refused`,
  `a_worker_whose_inbox_holds_fewer_than_a_start_is_refused`,
  `a_worker_whose_checks_outlast_a_change_reserve_is_refused`,
  `a_charter_for_the_engine_only_needs_nothing_of_workers`,
  `a_worker_holding_every_charter_is_admitted`. Each under 20 ms.
- `tests/fleet/tests/host_kinds.rs` (:120-175, the engine and worker
  kinds): `a_worker_refused_for_its_agent_hosts_nothing`. Under 0.5 s.
- `crates/jig-domain-core/src/tests/deployment_connectors.rs` (:139-170,
  the existing pattern): `a_refused_hello_shows_its_agent_shortfall`.
  Under 20 ms.
- `crates/jig-view/src/tests/owner_deployment.rs`:
  `an_agent_shortfall_refusal_is_worded`. Under 20 ms.
Proof: world, `heavy cargo nextest run -p jig-fleet-world --test host_kinds -E 'test(a_worker_refused_for_its_agent_hosts_nothing)'`,
once: a worker whose agent cannot hold a charter's model is turned away
and no run is placed on it; under 0.5 s.
Don't: let the fleet read a charter's bytes or a model's prices; check
a hello against charters no host of its kind may run.
Size: at most 350 lines of production code.
Removes: nothing.

## Interface sketches

```rust
// crates/jig-domain-host/src/boundary.rs (jig_domain_host)
use skein_lib::{Duration, Token};

/// Whose table a grant's value comes from (channel.md, section 8).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GrantTable {
    /// jig's LLM accounts.
    Accounts,
    /// The application's credentials.
    Application,
}

/// A credential grant's name, never its value. Sent by the engine with the
/// assignment and again while the run lives; it ends with its attempt.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Grant {
    pub table: GrantTable,
    pub account: u32,
    pub generation: u64,
    pub valid: Duration,
}

/// One message a run's start carries, oldest first (hosts.md, 9).
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct StartMessage {
    pub name: Token,
    pub sender: Box<[u8]>,
    pub words: Box<[u8]>,
}

/// What the agent's start is given: sent by the hub as `Request::Start` and
/// `ToAgent::Start`; its terminal is the agent's `Started` or `Gone`.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Start {
    pub workspace: Option<Token>,
    /// smith's charter, bytes the hub never reads.
    pub charter: Box<[u8]>,
    pub activation: u64,
    /// The run's identity, 16 bytes the hub never reads (channel.md, 7).
    pub identity: [u8; 16],
    /// The current window's turns, smith's bytes, when the run resumes.
    pub turns: Box<[Box<[u8]>]>,
    /// Calls settled since the last turn (`AnsweredCall` as built).
    pub answered: Box<[crate::AnsweredCall]>,
    pub messages: Box<[StartMessage]>,
    pub grants: Box<[Grant]>,
}

/// Why a message ended refused (hosts.md, 9), as smith names it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MessageRefusal {
    TooLarge,
    Full,
    NameInUse,
    Ending,
}

/// A message refused, by its name; kept by the hub for the next turn or the
/// answer, never sent on its own.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Refused {
    pub name: Token,
    pub reason: MessageRefusal,
}

/// One turn, kept by the hub until the engine acknowledges it; sent at once
/// and again after every hello.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Turn {
    pub turn: u32,
    pub spent: u64,
    pub read: Option<Token>,
    /// A window turn, or a conversation's first.
    pub opens_window: bool,
    /// The refusals since the run's previous turn; empty from the agent.
    pub refused: Box<[Refused]>,
    pub body: Box<[u8]>,
}

/// What a long operation is (hosts.md, 6.5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Operation {
    Checks,
    Completion,
    Command,
}

/// The kinds of smith's channel facts, in jig's terms.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CountKind {
    Admitted,
    Ended,
    LlmStarted,
    LlmRetried,
    LlmFinished,
    ToolStarted,
    ToolFinished,
    CheckStarted,
    CheckFinished,
    TextArrived,
}

/// Where a hosted run is (as built, plus `Answered`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Phase {
    Preparing,
    Starting,
    Active,
    Waiting,
    Ending,
    /// Its answer waits for the engine's acknowledgement.
    Answered,
}

/// A run's activity, content-free: from the agent's root and the hub's
/// phases; best effort, after the hello; nothing decides on it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Activity {
    Phase(Phase),
    /// A long operation begun, with its span.
    Long { operation: Operation, span: Duration },
    /// The long operation ended.
    LongDone,
    /// A count of one kind of smith's channel facts, at its time.
    Count { kind: CountKind, elapsed: Duration, count: u64 },
}

/// A hosted run, for the hello (as built).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Hosting {
    pub run: Token,
    pub attempt: Token,
    pub phase: Phase,
}

/// A model the host's agent holds (channel.md, 5.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ModelBound {
    pub name: Box<[u8]>,
    pub window: u32,
    pub output: u32,
}

/// The limits a run's start must meet on this host (smith's limits.md,
/// 2.1 and 2.3), given by the root from its agent's configuration.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Bounds {
    pub models: Box<[ModelBound]>,
    pub turns: u32,
    pub spend: u64,
    pub time: Duration,
    pub waiting: Duration,
    pub checks: Duration,
    pub message_bytes: u32,
    pub inbox: u32,
    pub unacknowledged: u32,
}

/// The hello as one value: the first thing on each link, made by the hub
/// at `Event::Opened` from what it hosts and what its root gives it.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Hello {
    pub slots: u32,
    /// The longest the host may take to stop a run it lost contact for.
    pub stop_bound: Duration,
    /// The workstreams its workspace holds.
    pub workstreams: Box<[u64]>,
    /// The runs it hosts, and those whose answer waits (`Phase::Answered`).
    pub hosting: Box<[Hosting]>,
    pub bounds: Bounds,
}

/// The engine's answer to a relayed call (channel.md, 5.2).
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Relayed {
    /// Its text for the LLM, and whether it is an error.
    Answer { error: bool, text: Box<[u8]> },
    /// Ask again after a backoff.
    Busy,
}

/// From the engine, through the link (hosts.md, 2: down). Each names its
/// run and attempt; the hub drops what names an attempt it finished with.
#[derive(PartialEq, Eq, Debug)]
pub enum LinkDown {
    /// An assignment; its terminal is one `LinkUp::Answer`.
    Assign { reply_to: skein_lib::ReplyTo, assignment: crate::Assignment },
    /// A message for the run, named, in order.
    Message { run: Token, attempt: Token, name: Token, sender: Box<[u8]>, words: Box<[u8]> },
    /// The one answer to the relayed call the hub named `call`.
    Relayed { run: Token, attempt: Token, call: Token, answer: Relayed },
    /// The engine committed this turn: the hub forgets it.
    TurnKept { run: Token, attempt: Token, turn: u32 },
    /// The answer is durable or not wanted: the hub forgets it, freeing its slot.
    Acknowledge { run: Token, attempt: Token },
    /// At most one per attempt.
    Cancel { run: Token, attempt: Token },
    /// A grant refreshed while the run lives.
    Grant { run: Token, attempt: Token, grant: Grant },
}

/// Toward the engine, through the link (hosts.md, 2: up). 05.4 adds
/// `Activity`, `Rejected` and `Exhausted`.
#[derive(PartialEq, Eq, Debug)]
pub enum LinkUp {
    /// A turn, kept by the hub until `LinkDown::TurnKept`.
    Turn { run: Token, attempt: Token, turn: Turn },
    /// A call the run named; one `LinkDown::Relayed` answers it.
    Call {
        run: Token,
        attempt: Token,
        call: Box<[u8]>,
        delivery: Token,
        tool: Box<[u8]>,
        writes: bool,
        input: Box<[u8]>,
        deadline: Duration,
    },
    /// The run's one answer, kept by the hub until `LinkDown::Acknowledge`.
    Answer { reply_to: skein_lib::ReplyTo, run: Token, attempt: Token, answer: crate::Answer },
}
```

```rust
// crates/jig-domain-fleet/src/boundary.rs and needs.rs (jig_domain_fleet)
use skein_lib::{Duration, ReplyTo, Token};

/// Whose table a grant's value comes from, as in the hub (05.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GrantTable {
    Accounts,
    Application,
}

/// A grant's name and how long it is valid; never its value (05.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Grant {
    pub table: GrantTable,
    pub account: u32,
    pub generation: u64,
    pub valid: Duration,
}

/// A grant's name in the fleet: whose table, and its number there (05.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GrantName {
    pub table: GrantTable,
    pub account: u32,
}

/// Parent to fleet (as built, but these three; jig 08 makes the fleet keep
/// and send what they carry).
pub enum Event {
    /// Place a run; its grants at its assignment. Ended by one of
    /// `Answered`, `Lost`, `Withdrawn` or `Refused`.
    Start {
        reply_to: ReplyTo,
        run: Token,
        attempt: Token,
        workstream: u64,
        kinds: crate::Kinds,
        assignment: crate::Assignment,
        grants: Box<[Grant]>,
    },
    /// A claim restored after a restart, with the grants it holds by name
    /// and the current generation of those that have one.
    Adopt {
        reply_to: ReplyTo,
        run: Token,
        attempt: Token,
        kept: u32,
        kind: crate::HostKind,
        worked: bool,
        holds: Box<[GrantName]>,
        grants: Box<[Grant]>,
    },
    /// A new generation, for each live attempt that holds its name.
    Grant { grant: Grant },
    // … the other variants as built.
}

/// A model an agent holds, mirroring the hub's `ModelBound` field for field.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ModelBound {
    pub name: Box<[u8]>,
    pub window: u32,
    pub output: u32,
}

/// The limits a run's start must meet on a host, in a hello, mirroring
/// the hub's `Bounds` field for field; the fleet admits the host by them.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Bounds {
    pub models: Box<[ModelBound]>,
    pub turns: u32,
    pub spend: u64,
    pub time: Duration,
    pub waiting: Duration,
    pub checks: Duration,
    pub message_bytes: u32,
    pub inbox: u32,
    pub unacknowledged: u32,
}

/// A model a charter names, as the fleet checks it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ModelNeed {
    pub name: Box<[u8]>,
    pub window: u32,
    pub output: u32,
}

/// What one charter needs of a host's agent.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CharterNeeds {
    pub charter: u32,
    /// The host kinds it may run on (`Kinds` as built).
    pub kinds: crate::Kinds,
    /// Its main model first, then its alternatives.
    pub models: Box<[ModelNeed]>,
    pub turns: u32,
    pub time: Duration,
    pub waiting: Duration,
    /// The reserve's time, when the charter allows a change.
    pub change_reserve: Option<Duration>,
}

/// What the engine needs of every host's agent, handed to `Domain::new`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Needs {
    pub charters: Box<[CharterNeeds]>,
    pub spend: u64,
    pub message_bytes: u32,
    pub start_messages: u32,
    pub unacknowledged: u32,
}

/// Which bound an agent fell short of.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AgentShortfall {
    Model { charter: u32, model: u32 },
    Window { charter: u32, model: u32 },
    Output { charter: u32, model: u32 },
    Turns,
    Spend,
    Time,
    Waiting,
    Checks,
    Message,
    Inbox,
    Unacknowledged,
}

/// The owner's reason for refusing a hello (as built, plus `Agent`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HelloRefusal {
    ReservedChannel,
    Workers { limit: u32 },
    StopBound { grace: Duration },
    /// The host's agent cannot run a charter the engine may place there.
    Agent { why: AgentShortfall },
}
```

The hub owns what it keeps (turns, answers, refusals, activity) and
reads only what section 2 puts beside smith's bytes; the root assembles
the hello and hands smith's payloads out; the fleet fences, and judges a
hello against the needs the core gives it at construction. A turn ends
at its acknowledgement; an answer at its acknowledgement; a refusal with
the turn or answer that carries it.

## Questions to raise when reached

- **A run's checks or command on the chip:** the client's contract has
  no word for a tool run without its call's name (contract.md, 7.3), so
  this plan shows `Active`. Raise whether the client should say what
  the run is doing.
- **The engine's hello without agents:** a root whose engine hosts no
  agents (the reference service's) says no hello on channel 0, and the
  fleet places nothing there only because `engine_slots` is 0. Raise if
  a root with engine slots and no hello should be refused at start.

## Done when

- No bounce crosses the link: `git -C ~/src/rust/jig grep -n -E 'Bounced|Bounce::' -- crates testing tests examples`
  prints nothing.
- The core routes notices: `git -C ~/src/rust/jig grep -n -E 'Request::(Rejected|Exhausted|Told)[^;]*unreachable' -- crates/jig-domain-core`
  prints nothing.
- Activity is typed: `git -C ~/src/rust/jig grep -n -E 'Facts \{|fact_bytes' -- crates/jig-domain-host crates/jig-domain-fleet ':!*/facts.rs'` (ruling 150)
  prints nothing.
- The hello replaced the report: `git -C ~/src/rust/jig grep -n -E 'Event::Report|Request::Hosting|Unacknowledged \{' -- crates testing tests examples`
  prints nothing.
- The link is two enumerations: `git -C ~/src/rust/jig grep -n -E 'pub enum (LinkUp|LinkDown|Relayed)\b' -- crates/jig-domain-host/src`
  prints three lines.
- The fleet names an agent's shortfall: `git -C ~/src/rust/jig grep -n 'HelloRefusal::Agent' -- crates/jig-domain-fleet/src`
  prints at least one line.
- The hub links no smith: `git -C ~/src/rust/jig grep -n smith -- crates/jig-domain-host/Cargo.toml crates/jig-domain-fleet/Cargo.toml`
  prints nothing.
- The gate passes on the last landed commit:
  `git -C ~/src/rust/jig log -1 --format='%(trailers:key=Gate-Result,valueonly)' origin/main`
  names `scripts/check.sh passed`.
