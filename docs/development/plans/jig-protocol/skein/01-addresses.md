# Session 01: in-process addresses and joined sockets

io's addresses become skein's designed `Addr`, a socket address or an
in-process name, and io gains in-process names: a listener on a name
accepts sockets joined in memory to those that connect to it, through
the same requests, events and orders as TCP, with no kernel, no
simulator and no TLS between them. jig's one-process shape joins its
engine and its in-process workers this way (jig's `protocol/service.md`,
section 10). The first increment changes the type and its users and
takes names; the second accepts, moves bytes and closes; the third adds
close deadlines, late resets and output reservations; the fourth adds
the worlds the design lists that the first three do not. smith 01 and
jig 02 take the result.

## Read first

- skein's `io.md`: section 2 (the sketch, `Addr`, the decisions on
  `Busy` and refusals), 3 and 3.1 to 3.4 (the machines a joined socket
  reuses), 3.5 (joined sockets), section 4 (addresses and names),
  section 8 ("Joined sockets", "The same owners over either", "Memory",
  "One process, on the real ring"), section 9's last bullet, and
  section 10, "Faults between joined ends".
- skein's `kernel.md`, section 3 (the kernel's records keep socket
  addresses); `shell.md`, section 6, step 5 (startup hands an in-process
  name on unresolved).
- `programming-model.md`, sections 2 (entry points and `MAX_OUT`), 5.4
  (state machines), 6.3 (the worst case) and 10.3 (no hand-written trait
  impls in step code); `testing-strategy.md`, 2.6 (io worlds) and 2.8
  (the real loop).
- This plan's `decisions.md`, 4 (LLM and OAuth endpoints keep socket
  addresses).
- `findings/skein.md`, section 1 (the code today, the users, the risks)
  and "smith's repin" (what smith 01 inherits).

## What the design asks

- **io's `Addr`** (io.md, 2 and 4): `pub enum Addr { Ip(SocketAddr),
  InProcess(Name) }`. "An in-process name is at most `Name::MAX` bytes,
  compared as bytes and held inline, so an `Addr` stays a plain value
  that allocates nothing." The kernel's records carry socket addresses
  only (kernel.md, 3).
- **Names** (io.md, 3.5): "`Listen` takes the name for its listener until
  the listener's `Closed`, which frees it. A name already taken, or the
  empty name, is answered `Failed` with `Other`, as an address in use is.
  With all `Limits::names` held, a `Listen` on a name is refused at the
  entrance (`Busy`), as one without a slot is."
- **Connecting** (3.5): `Connecting` as over TCP; a name unbound, or held
  by a closing listener, is told `Failed` with `Refused`, then `Closed`;
  otherwise the connector waits on the listener, in its own slot and in
  the order connectors came, until accepted or aborted; a listener that
  closes refuses those still waiting.
- **Accepting** (3.5) follows 3.2: armed one at a time while the
  listener's owner has room, with a free slot and room in the accept
  batch, starved without a slot; it joins the first connector waiting,
  tells the listener's owner `Accepted` with the empty name as `peer`,
  and the connector `Connected`. "Nothing moves into an end until it is
  bound."
- **Moving bytes** (3.5): one move in flight per end, the head of its
  output copied into the peer's intake as far as that has room; "the
  reader's intake is the backpressure"; demands met by 3.3's delivered;
  independent output reservations offered as on a socket.
- **Ends** (3.5): a half-close reaches the peer after the last byte
  moved; a graceful close keeps section 3's four steps; an end whose
  peer has closed fails as `Reset` at its next move; one whose peer
  aborts, is rejected, or meets its close deadline before its flush is
  done fails as `Reset` at once, or, if closing, ends its close; "an end
  never settles".
- **Nothing happens inside a request** (3.5): `down` records and readies;
  io moves, joins, unjoins and tells in the next up pass; "A joined end
  submits nothing and holds no slot of the operation table."
- **Memory** (3.5): a pair holds two slots of `Limits::sockets`;
  `worst_case` adds the names table.
- **In the simulator** (3.5): joined sockets are io's own; the
  simulator's faults never reach them and its trace does not show them.
- **No TLS** on a joined socket (3.5).
- **Worlds** (io.md, 8): the list under "Joined sockets", the same owners
  over either, memory, and one process on the real ring.

## Choices this plan makes

- `Addr` and `Name` live in a new module, `crates/skein-io/src/addr.rs`,
  exported as `skein_io::{Addr, Name}`.
- `kernel::Addr` stays `pub type Addr = SocketAddr` (kernel.md, 3: the
  kernel's records keep socket addresses), so the kernel, the ring, the
  simulator and the conformance suite need no edit (findings/skein.md,
  section 1). io converts at its kernel edge: its listener and stream
  keep `SocketAddr` inside and wrap `Addr::Ip` where they tell one.
- `Name::MAX` is 32 bytes; `Addr` stays `Copy`.
- No `From<SocketAddr> for Addr`: hand-written trait impls are out of
  step code (programming-model.md, 10.3). Users write `Addr::Ip(..)`.
- Decision 4: `skein-llm-connection`'s and `skein-oauth-accounts`'
  `Endpoint.address` stay `kernel::Addr` (`SocketAddr`), their loopback
  checks unchanged; each wraps `Addr::Ip` where it pushes `Connect`.
  `skein-fake-peers`' `address()` keeps returning `SocketAddr`.
- `Limits::is_usable` does not ask for names: zero names is usable, for
  a service that listens on none (smith).
- The joined listener and the joined end are entities of their own,
  `Entity::Named(joined::Listener)` and `Entity::Joined(joined::End)`, in
  `crates/skein-io/src/joined.rs` (and `joined/*.rs` if it passes 800
  lines), not new states of `listener.rs` (716 lines) or `stream.rs`
  (1,715 lines).
- The `MAX_OUT_*` constants stay as they are: a joined entity tells its
  own events from its own `resume`. The world checks each call against
  them (`tests/io/src/world.rs:135-138`), and the step tests' `Rig` hands
  each call that much room.
- The joined end answers demands with the socket's delivered: the
  function stream.rs's `Open` uses (`stream.rs:935-1032`) is shared, not
  copied (code-organization.md, section 6, first item).

## Rules that bind it

- "**No increment of this plan** adds a type beside an existing one for
  the same thing: it changes the type in place, and its users in the
  same increment" (code-organization.md, section 6). io's records change
  to `Addr` in 01.1, with their users.
- "**Shared hotspots with smith's reliability plan,** if it runs at the
  same time on another machine: skein's `crates/skein-io/src/kernel.rs`,
  `crates/skein-io/src/file*.rs` and `crates/skein-shell/` (its terminal
  modes, skein 02.6 of that plan). `land` rebases each landing on the
  other's; no session waits for it" (README, "Rules"). Here also
  `records.rs`, `layer.rs` and `limits.rs`, where that plan adds io's
  `Window`.
- "**A file:** at most 800 lines, generated code excepted. A file of at
  most 800 lines that the increment grows past 800 is split in the same
  increment"
  (code-organization.md, section 7). This session's exception: a file
  already past 800 lines at the session's start (`stream.rs`, 1,715)
  grows by at most 40 lines and is not split; new code goes in
  `joined.rs`, `addr.rs` and their tests.
- **This session's own rule, until 01.3 lands:** only `crates/skein-io`
  and `tests/io` name `Addr::InProcess`; checked by `git grep -n
  'Addr::InProcess' -- ':!crates/skein-io' ':!tests/io' ':!docs'`,
  which prints nothing. 01.1 and 01.2 leave behaviour that 01.2 and 01.3
  finish, and no user may meet it.

## Increments

### 01.1 io's addresses and in-process names
Scope: io.md, 2 and 4 (`Addr`, `Name`); 3.5's "The name" and
  "Connecting" bullets and its memory bullet; `Limits::names`. A `Listen`
  on a name takes it, passing Socket, Binding and Arming at once, and is
  told `Listening` at its name in the next up pass; a taken or empty
  name is told `Failed` with `Other`, then `Closed`; with
  `Limits::names` names held, a `Listen` on a name is refused at the
  entrance (`Busy`); a listener's `Closed` frees its name. A `Connect`
  to a name is told `Connecting`; an unbound name, or one held by a
  closing listener, then `Failed` with `Refused` and `Closed`; a bound
  name: the connector waits on the listener, in order, until its owner
  aborts it (told `Closed`) or the listener closes (told `Failed` with
  `Refused`, then `Closed`). Transitional: no listener on a name arms an accept yet, so
  a connector waits until aborted or refused; 01.2 removes this. The
  users of io's address records move to `Addr::Ip(..)`.
Needs: nothing.
Files:
  - skein-io: new `src/addr.rs`; `src/lib.rs` (exports); `src/records.rs`
    (`Listen`, `Connect`, `Listening`, `Accepted`: `records.rs:9,18,22,71,79`);
    `src/limits.rs` (`names`, `worst_case`, `limits.rs:13-45,107-130`);
    `src/layer.rs` (`Tables.names`, `Entity`'s two variants, `listen` and
    `connect` at `:531-547` dispatching `Addr::Ip` to the kernel path and
    `Addr::InProcess` to `joined`, the `Entity` matches); new
    `src/joined.rs`; `src/listener.rs:577,599` and the `stream.rs` emit
    sites (`Addr::Ip` around what they tell); `src/tests.rs` (`limits()`
    at `:30-42` gains `names`; `local()` at `:46-48` stays the kernel's
    `SocketAddr`, a new `addr(port) -> Addr` and `name(&[u8]) -> Addr`
    serve the records; `listening` and `connected` at `:262-282`); new
    `src/tests/joined.rs`; `src/tests/layer.rs:15-17` and its
    `worst_case` test at `:93`; `src/tests/listener.rs:20,34,45,57,76,208,239`;
    `src/tests/stream.rs:81,95,358,608,628`.
  - Production users (findings/skein.md, section 1, list A, and decision
    4): `crates/skein-llm-connection/src/component.rs:939` and
    `crates/skein-oauth-accounts/src/component.rs:706` (`Addr::Ip` at the
    connect); `crates/skein-oauth-accounts/src/component/sign_in.rs:118-121`
    (the redirect listener); `examples/echo/protocol/src/layer.rs:9,24,58,98,186,429,434`;
    `examples/echo/service/src/lib.rs:46,185,279`;
    `examples/echo/shell/src/lib.rs:5,20` and `:89` (its `io::Limits`);
    `examples/echo/shell/src/main.rs:36,99` (parses a `SocketAddr`, wraps
    it); `testing/skein-echo-client/src/conn.rs:9,151` and
    `src/lib.rs:30,184-185,266`; `testing/skein-fake-peers/src/face.rs:49`,
    `src/llm.rs:146-149`, `src/oauth.rs:161-164`.
  - Test users (list A): `examples/echo/protocol/src/tests.rs:17,36`;
    `examples/echo/service/src/tests.rs:13,44`;
    `testing/skein-echo-client/src/tests.rs:13,53`;
    `tests/io/src/owner.rs:10,17,92,130,178,352,362,369,396,570-571`,
    `src/scenarios.rs:13,50,189`, `src/output.rs:132`;
    `tests/io/tests/memory.rs:282,285,355-356,375,467,469`;
    `tests/browser/src/serve.rs:46,60,110-112`,
    `tests/browser/tests/page_server.rs:18,21`;
    `tests/echo/src/proc.rs:7,21`, `src/referee.rs:10,170`,
    `src/scenarios.rs:18,101`; `tests/echo/tests/end_to_end.rs:7,79`,
    `tests/echo/tests/real.rs:16,33`; `tests/tls/src/native.rs:177`,
    `tests/tls/tests/fake_peer.rs:50,97`;
    `tests/oauth-accounts/src/sign_in/browser.rs:167`.
  - `io::Limits` literals that gain `names` (list C, 31): the files
    above with literals, and `crates/skein-llm-connection/src/tests.rs:16`,
    `crates/skein-oauth-accounts/src/tests.rs:21`,
    `examples/echo/service/src/tests.rs:20`,
    `testing/skein-echo-client/src/tests.rs:19`,
    `testing/skein-fake-peers/src/transport/tests.rs:16`,
    `testing/skein-fake-peers/tests/llm.rs:12`, `tests/oauth.rs:23`,
    `tests/browser/tests/browser_real.rs:97`,
    `tests/echo/src/scenarios.rs:54,82`, `tests/io/src/append.rs:60`,
    `src/output.rs:107`, `src/scenarios.rs:27`,
    `tests/io/tests/memory.rs:418,430,442,456`,
    `tests/io/tests/process_memory.rs:177,213`,
    `tests/llm-connection/src/simulated.rs:29`, `src/world.rs:93`,
    `tests/llm-connection/tests/failures.rs:23`, `fake_protocol.rs:26`,
    `memory.rs:22`, `protocol.rs:21`,
    `tests/oauth-accounts/src/world.rs:90`,
    `tests/oauth-accounts/tests/memory.rs:36`,
    `tests/tls/src/native.rs:157`, `tests/tls/tests/fake_peer.rs:19`.
    Each takes `names: 0`, but `tests/io/src/scenarios.rs:27`'s
    `limits()`, which takes `names: 2` for the joined worlds.
  - World: new `tests/io/src/joined.rs`; `tests/io/src/lib.rs`
    (`pub mod joined;`); `tests/io/src/referee.rs` (one `Expect`
    variant); new `tests/io/tests/joined.rs`.
  - Lines are findings/skein.md's, at skein `5c9eedd`; `917ea21` changed
    Markdown only. `heavy cargo check --workspace --all-targets` lists any
    site they miss.
  - Shared hotspots: `crates/skein-oauth-accounts/src/component.rs` and
    `component/sign_in.rs` (skein 02.4 and 02.5 edit them);
    `crates/skein-io/src/{records,layer,limits}.rs` (smith's reliability
    plan, skein 02.6).
Shape:
  - `skein_io::Addr` and `skein_io::Name` as the sketch below; `Request::
    Listen { owner: Token, addr: Addr }`, `Request::Connect { owner:
    Token, addr: Addr }`, `Event::Listening { owner: Token, listener:
    Token, addr: Addr }`, `Event::Accepted { owner: Token, socket: Token,
    peer: Addr }`, `Addr` now `crate::Addr`; each other field and
    variant of `records.rs` unchanged.
  - `Limits { .., names: u32 }`, documented "In-process names held at
    once by listeners (io.md, 3.5)". `worst_case` adds `Map::<Name,
    Id<Entity>>::worst_case(names)` and what holds the waiting connectors
    (at most `sockets` of them together).
  - `Tables { .., names: Map<Name, Id<Entity>> }`.
  - `layer::listen(io, owner, addr, subs)`: `Addr::Ip(socket)` as today
    with `Family::of(&socket)`; `Addr::InProcess(name)`: refused into
    `tables.refused` when the slab or the names table is full, otherwise
    a `joined::Listener` on the ready list. `layer::connect` likewise,
    the connector a `joined::End` in state `Waiting`, on the ready list.
  - `joined::Listener` states: `Listening { name, waiting: <connectors
    in order> }`, `Closing`, `Closed`; told from `resume`. A taken or
    empty name: the listener tells `Failed { error: Other }` and `Closed`
    from its first resume, and holds no name.
Test shape:
  - Step tests in new `crates/skein-io/src/tests/joined.rs`, over the
    `Rig` (`src/tests.rs:200-255`: `Rig::new`, `down`, `next`, `at`,
    `empty`), `owner(n)` (`:51`), `limits()` (`:30`) and the new `name`
    fixture in `src/tests.rs`. Reference test:

    ```rust
    #[test]
    fn a_listen_on_a_name_is_told_listening_and_submits_nothing() {
        let mut rig = Rig::new(limits());
        rig.down(Request::Listen { owner: owner(1), addr: name(b"engine") }).nothing();
        let out = rig.next();
        assert!(out.subs.is_empty(), "a joined listener submits nothing: {out:?}");
        match out.events.as_slice() {
            [Event::Listening { owner: told, addr, .. }] => {
                assert_eq!((*told, *addr), (owner(1), name(b"engine")), "told its owner, at its name");
            }
            other => panic!("a listener on a name is told Listening: {other:?}"),
        }
    }
    ```

    The others: `a_second_listen_on_a_taken_name_fails_with_other`,
    `a_listen_on_the_empty_name_fails_with_other`,
    `a_listen_past_the_names_limit_is_refused_busy`,
    `a_closed_listener_frees_its_name_for_the_next_listen`,
    `a_connect_to_an_unbound_name_is_refused_then_closed`,
    `a_connector_waits_on_a_bound_name_until_its_abort`,
    `a_closing_listener_refuses_the_connectors_waiting_on_it`;
    and in `src/tests/layer.rs`, `worst_case_grows_with_the_names_table`.
    Each under 20 ms.
  - io world: new `tests/io/src/joined.rs` holds the joined scenarios,
    built like `tests/io/src/scenarios.rs`'s (`World::new`, `spawn`,
    `Owner::new(seed, serves, dials)` with one process holding both ends,
    `Serve`, `Dial` with `Target::Named` or `Target::Addr`, `plan`,
    `limits`, `END` made `pub`). `tests/io/src/referee.rs` gains
    `Expect::ListenFails { at, serve, errors, by }`, met when the owner's
    listener of that name failed with one of `errors` (the owner keeps it,
    `owner.rs:284`). Reference scenario and test:

    ```rust
    // tests/io/src/joined.rs
    /// A dial to a name no listener holds: `Connecting`, then `Failed`
    /// with `Refused`, then `Closed` (io.md, 3.5).
    #[must_use]
    pub fn unbound(seed: u64, config: Config) -> World {
        let expect = vec![
            Expect::Fails { at: 0, conn: "lost", errors: &[Error::Refused], by: END },
            Expect::Closed { at: 0, conn: "lost", by: END, optional: false },
        ];
        let mut world = World::new(seed, config, Referee::new(seed, expect));
        let lost = plan("lost", Box::from(&b"hello"[..]), 4, Reads::Fill(4));
        let dials = vec![Dial { to: Target::Addr(name(b"nowhere")), at: Time::ZERO, plan: lost }];
        world.spawn(limits(2), Owner::new(seed, Vec::new(), dials));
        world
    }

    // tests/io/tests/joined.rs
    //! Joined sockets in io's worlds (io.md, 3.5 and 8): owners at both
    //! ends of each pair, in one process over the simulator, which never
    //! sees the pairs.

    use skein_io_world::joined::unbound;
    use skein_sim::Config;

    const SEEDS: [u64; 3] = [1, 2, 3];

    #[test]
    fn a_connect_to_an_unbound_name_is_refused() {
        for seed in SEEDS {
            let outcome = unbound(seed, Config::chaos()).run();
            assert!(outcome.trace.is_empty(), "seed {seed}: a joined world submits nothing to the kernel");
        }
    }
    ```

    The others: `a_second_listener_on_a_taken_name_fails_with_other`
    (two `Serve`s of one name in one owner),
    `named_listens_past_the_names_limit_are_refused_busy` (three, under
    `names: 2`), `a_closing_listener_refuses_its_waiting_connectors` (a
    `Serve` closing `At(ms(50))` and a `Dial` to it). Each under 0.5 s. A
    name taken again after its listener's `Closed` is the step test's: the
    owner starts its listens at once and has no later one. A joined
    world's evidence comes from the owners, the referee and
    `Outcome.logs`, never the simulator's trace, which is empty.
Proof: world, `tests/io/tests/joined.rs`: `a_connect_to_an_unbound_name_is_refused`
  and `a_closing_listener_refuses_its_waiting_connectors`, run once with
  `heavy cargo nextest run -p skein-io-world --test joined`: an owner
  hears `Refused` for a name nobody holds, and a connector waiting on a
  listener that closes is refused, with the simulator's trace empty.
  Under 1 s.
Don't: rename or redefine `kernel::Addr`; change `Endpoint.address`'s
  type in skein-llm-connection or skein-oauth-accounts; add a `From` or
  `FromStr` impl; arm an accept on a name (01.2).
Size: at most 300 lines of production code in skein-io; each user's
  edit is one or two lines.
Removes: io's records' use of `kernel::Addr`.

### 01.2 Accepting and moving bytes
Scope: io.md, 3.5's "Accepting", "Moving bytes" and "Nothing happens
  inside a request", and the table's rows for `Accept`, `Connect`,
  `Recv`, `Send` and `Shutdown`; 3.1's ownership rules. A listener on a
  name arms its accept one at a time while its owner has room, with a
  free slot and room in the accept batch (`Tables.armed`), starved
  without a slot; armed, it takes the first connector waiting, makes the
  accepted end, joins the two, tells its owner `Accepted` with `peer`
  `Addr::InProcess` of the empty name, and the connector `Connected`.
  `Bind` opens the end; `Reject` closes it unannounced and the
  connector fails as `Reset` at once. Bytes move as 3.5 says, demands
  answered by the socket's delivered, `Finish` reaching the peer as
  `End` after the last byte moved; `Abort` ends the end, told `Closed`
  in the next up pass, and fails its peer as `Reset` at once. A `Close`
  keeps section 3's four steps: what is queued moves; the peer is
  half-closed; what the peer moves is discarded until its own end; the
  pair is unjoined and `Closed` told ("a close with nothing to flush or
  drain, or an abort, is told `Closed` in the next up pass"). Transitional: no
  close deadline yet, so a close whose peer never ends waits until the
  peer closes or aborts; 01.3 adds the deadline. Removes 01.1's
  transitional: a waiting connector is now accepted.
Needs: nothing beyond 01.1.
Files: `crates/skein-io/src/joined.rs` (and `joined/end.rs` if
  `joined.rs` passes 800 lines); `src/stream.rs` (delivered shared, at
  most 40 lines changed); `src/layer.rs` (`Bind`, `Reject`, `Stream`,
  `Close`, `Abort` and `resume` reach joined entities; `armed` counted
  for both kinds); `src/tests/joined.rs`; `tests/io/src/joined.rs`,
  `tests/io/tests/joined.rs`.
Shape:
  - `joined::End` states: `Waiting { listener }`, `Announced { listener,
    peer }`, `Open(Open)`, `Closing(Closing)`, `Closed`; `Open` holds the
    owner, the peer's id, the intake, the demand, the output queue
    (`Limits::sends` boxes, `Limits::output` bytes) and the move in
    flight (the head box and its offset), the reader's end and the
    writer's finish; `Closing` holds the owner, the peer, the flush (moves
    queued, then the half-close) and the drain (the peer's moves
    discarded until its end).
  - A move: copy `min(head left, peer intake room)` bytes, keep the
    offset, drop the box once moved; ready the peer; a full intake moves
    nothing until the peer's demand drains it, which readies the mover.
  - `joined::End` and `joined::Listener` emit nothing from `down`
    (`MAX_OUT_DOWN.events` is 0).
Test shape: step tests in `src/tests/joined.rs`:
  `an_armed_listener_joins_the_first_waiting_connector`,
  `the_accepted_end_is_announced_with_the_empty_name`,
  `a_rejected_end_fails_its_connector_with_reset`,
  `a_listener_without_room_leaves_its_connectors_waiting`,
  `bytes_move_into_the_peer_only_as_far_as_its_intake_has_room`,
  `a_finish_reaches_the_peer_as_end_after_the_last_byte`,
  `an_abort_fails_the_peer_with_reset_at_once`,
  `a_close_moves_what_is_queued_before_unjoining`,
  `a_closing_end_discards_what_its_peer_moves_until_end`,
  `a_close_with_nothing_to_flush_is_closed_next_pass`,
  `a_joined_end_submits_nothing_at_any_entry_point`; each under 20 ms.
  World, `tests/io/src/joined.rs` and `tests/io/tests/joined.rs`, after
  `tests/io/src/scenarios.rs`'s `accept`, `batch`, `exchange`,
  `backpressure` and `abort`, served on a name: tests
  `a_named_listener_accepts_binds_and_rejects`,
  `two_named_listeners_share_one_accept_batch`,
  `connectors_wait_without_room_or_slot_and_one_aborts`,
  `bytes_go_both_ways_over_a_joined_pair_under_small_intakes` (intakes of
  7 bytes, so each move is cut), `a_joined_reader_that_does_not_demand_stops_its_peer`,
  `an_abort_is_heard_as_reset_by_the_joined_peer`; each under 0.5 s.
Proof: world, `bytes_go_both_ways_over_a_joined_pair_under_small_intakes`,
  run once with `heavy cargo nextest run -p skein-io-world --test
  joined`: each owner receives what the other sent, in order, over a
  pair joined in one io, under intakes that cut each move, with the
  simulator's trace empty. Under 1 s.
Don't: arm a retry or a close deadline for a joined entity in this
  increment; copy stream.rs's delivered.
Size: at most 600 lines of production code.
Removes: 01.1's waiting without an accept.

### 01.3 Close deadlines, late resets and independent output
Scope: io.md, 3.5's "Ends", the rest of them; 3.3's independent output
  reservations on a joined end, the move counting as the send in
  flight; 3.1's "`Failed` and `End` are told once each, and never after
  `Closed`". A closing end's drain ends at its close deadline
  (`Limits::close_timeout`), which unjoins the pair. An end whose peer
  has closed fails as `Reset` at its next move, its bytes lost; one
  whose peer meets its close deadline before its flush is done fails as
  `Reset` at once, or, if closing, ends its close. Removes 01.2's
  transitional: a close no longer waits on a peer that never ends.
Needs: nothing beyond 01.2.
Files: `crates/skein-io/src/joined.rs` (and `joined/*.rs`);
  `src/layer.rs` (`fire` reaches a joined end's close deadline;
  `output_request` reaches a joined end); `src/output.rs` if the
  reservation's bookkeeping is shared from there; `src/tests/joined.rs`;
  `tests/io/src/joined.rs`, `tests/io/tests/joined.rs`.
Shape: `joined::End`'s `Closing` arms its close deadline in
  `Tables.deadlines` under `Timer::Close`, cancelled when the close ends
  first; the independent output cell as `stream.rs`'s `Open` keeps it,
  its `Granted`, `Cancelled` and `Failed` terminals as 3.3 orders them.
Test shape: step tests in `src/tests/joined.rs`:
  `the_close_deadline_ends_a_close_the_peer_never_ends`,
  `an_end_whose_peer_closed_fails_reset_at_its_next_move`,
  `a_peer_past_its_close_deadline_fails_the_end_with_reset`,
  `an_output_reservation_on_a_joined_end_is_granted_then_spent`,
  `a_close_cancels_a_pending_output_reservation_before_closed`; each
  under 20 ms. World, after `tests/io/src/scenarios.rs`'s
  `refusal_mid_upload`, `close_deadline` and `closes`, on a name:
  `a_response_in_the_middle_of_an_upload_reaches_the_joined_peer`,
  `a_joined_close_the_peer_never_ends_meets_its_deadline`,
  `joined_closes_and_aborts_at_random_moments_settle`; each under 0.5 s.
Proof: world, `a_response_in_the_middle_of_an_upload_reaches_the_joined_peer`,
  run once with `heavy cargo nextest run -p skein-io-world --test
  joined`: the uploading peer reads the response and hears `End`, no
  `Reset`, after the other end closed gracefully mid-upload. Under 1 s.
Don't: give a joined end a retry deadline; reach the simulator.
Size: at most 350 lines of production code.
Removes: 01.2's close without a deadline; this session's rule on naming
  `Addr::InProcess` outside skein-io ends with this landing.

### 01.4 The joined sweep, the same owners over either, memory, the ring
Scope: io.md, section 8's rest: the random scenario over many seeds in
  the fuzzy suite; "a calm world whose owners run the same plans over a
  loopback pair and over a joined pair, and read the same bytes, ends
  and failures over each"; "every name held and every slot a joined end,
  against `worst_case`"; "a listener on loopback and one on an
  in-process name in one `drive`, an exchange over each".
Needs: nothing beyond 01.3.
Files: new `tests/io/tests/fuzzy_joined.rs`; `tests/io/src/joined.rs`
  (the random scenario, the either-pair world); `tests/io/tests/joined.rs`;
  `tests/io/tests/memory.rs` (568 lines; aim to keep it under 800, else
  the joined part goes to a new `tests/io/tests/joined_memory.rs`);
  `tests/io/tests/ring.rs`.
Shape: `joined::either(seed, plans) -> [Seen; 2]`, the same plans run
  once over `Addr::Ip` on loopback and once over a name, each `Seen`
  holding what each connection received, whether it heard `End`, and
  its failure; the test compares the two.
Test shape:
  - `tests/io/tests/joined.rs`: `the_same_owners_read_the_same_over_either_pair`.
  - `tests/io/tests/fuzzy_joined.rs`: `joined_closes_and_aborts_hold_over_many_seeds`,
    150 seeds, about 5 s.
  - `tests/io/tests/memory.rs`: `every_name_held_and_every_slot_a_joined_end_fit_worst_case`,
    with the counting allocator as the file's other tests use it.
  - `tests/io/tests/ring.rs`: `a_loopback_and_a_named_listener_exchange_in_one_drive`,
    after `an_exchange_through_io_over_the_ring` (`ring.rs:18-71`), its
    `Proc`, `Directory` and loop reused, one `Serve` on `local(0)` and
    one on a name, a `Dial` to each.
  - Ceilings: focused under 0.5 s each; the joined world's tests together at most
    about 1 s focused and 5 s fuzzy (development.md, 5.3).
Proof: world, `the_same_owners_read_the_same_over_either_pair`, run once
  with `heavy cargo nextest run -p skein-io-world --test joined`: owners
  running one set of plans read the same bytes, ends and failures over a
  loopback pair and over a joined pair. Under 1 s. The ring test runs in
  `land`'s gate; nothing else is run for the proof.
Don't: add a way to fault a joined pair (io.md, 10 leaves it open).
Size: no production code; a bug the sweep finds is fixed here in at most
  100 lines of production code, or is a question.
Removes: nothing.

## Interface sketches

```rust
// skein_io::addr (crates/skein-io/src/addr.rs), exported as skein_io::{Addr, Name}
use core::net::SocketAddr;

/// Where io listens or connects (io.md, sections 2 and 4); the owner of a
/// `Listen` or `Connect` gives it, and `Listening` and `Accepted` tell it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Addr {
    /// A socket address, IPv4 or IPv6 with a port: the kernel's.
    Ip(SocketAddr),
    /// A name in this io's own namespace, whose listener joins sockets in memory (io.md, 3.5).
    InProcess(Name),
}

/// An in-process name: at most `Name::MAX` bytes, inline, compared as bytes (io.md, 4).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Name {
    len: u8,
    bytes: [u8; 32],
}

impl Name {
    /// The longest name, in bytes.
    pub const MAX: usize = 32;
    /// The empty name: a connector's, told as an accepted joined socket's peer.
    pub const EMPTY: Name = Name { len: 0, bytes: [0; 32] };

    /// The name of `bytes`, or `None` past `Name::MAX`.
    #[must_use]
    pub fn new(bytes: &[u8]) -> Option<Name> {
        todo!()
    }

    /// Its bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        todo!()
    }
}
```

```rust
// skein_io::Limits (crates/skein-io/src/limits.rs): crosses into smith and
// each skein user that writes a literal.
use skein_lib::Duration;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub sockets: u32,
    pub refusals: u32,
    pub intake: u32,
    pub receive: u32,
    pub output: u32,
    pub sends: u32,
    pub accepts: u32,
    pub backlog: u32,
    pub close_timeout: Duration,
    pub retry: Duration,
    /// In-process names held at once by listeners (io.md, 3.5).
    pub names: u32,
}
```

```rust
// skein_io::joined (crates/skein-io/src/joined.rs): crate-private; the
// states 01.1 to 01.3 build. io owns both ends of a pair and the names
// table; each end's owner is the owner a socket's would be.

/// A listener on an in-process name (io.md, 3.5), in the slab as
/// `Entity::Named`: holds its name from its `Listen` until its `Closed`.
#[derive(Debug)]
pub(crate) struct Listener {/* its owner, its name, the connectors waiting in order, its state */}

/// One end of a joined pair (io.md, 3.5), in the slab as `Entity::Joined`.
#[derive(Debug)]
pub(crate) struct End {/* its owner once bound, its peer, its state */}

/// Where a listener on a name is.
#[derive(Debug)]
pub(crate) enum ListenerState {
    /// Taking connectors, in order, accepting one at a time while its owner has room.
    Listening,
    /// Refusing those still waiting; told `Closed` in the next up pass.
    Closing,
    /// Terminal: its name is free.
    Closed,
}

/// Where an end is.
#[derive(Debug)]
pub(crate) enum EndState {
    /// A connector waiting on a listener, in order (01.1).
    Waiting,
    /// Accepted, announced to the listener's owner, not yet bound (01.2).
    Announced,
    /// Bound or connected: moves, demands, the output and its reservation (01.2, 01.3).
    Open,
    /// Flushing by moves, half-closing, discarding the peer's moves (01.2; its deadline, 01.3).
    Closing,
    /// Terminal.
    Closed,
}
```

A pair ends when both ends are `Closed`; a name ends with its listener's
`Closed`. A host that stacks io counts `io.is_ready()` in its
`Host::work_pending`, as the echo service does
(`examples/echo/service/src/lib.rs:236-239`): `drive` waits "not at all
while work is pending" (shell.md, 12), and joined bytes move only from
io's ready list.

## Questions to raise when reached

- **Faults between joined ends** (io.md, 10): whether io gives a world a
  way to reset a pair or cut a move short. Open: this session adds none.
- **TLS asked on an in-process address** by a skein component: none can
  be today, as LLM and OAuth endpoints keep socket addresses (decision
  4). A component that takes io's `Addr` for TLS later meets io.md 3.5's
  "No TLS"; that is its own design's question.

## Done when

1. io's records carry `Addr`, and the kernel's socket addresses: `git -C
   ~/src/rust/skein grep -n 'pub type Addr = SocketAddr' origin/main --
   crates/skein-io/src/kernel.rs` prints one line, and `git -C
   ~/src/rust/skein grep -n -E 'impl (From|FromStr)' origin/main --
   crates/skein-io/src` prints nothing.
2. The joined worlds pass: `heavy cargo nextest run -p skein-io-world
   --test joined --test ring --test memory` passes, and `heavy cargo
   nextest run -p skein-io-world --profile fuzzy --test fuzzy_joined`
   passes.
3. The four increments landed through `land`: `git -C ~/src/rust/skein
   log --format='%(trailers:key=Step,valueonly)' origin/main` lists
   `skein-01.1` to `skein-01.4`, each commit's `Gate-Result` naming
   `scripts/check.sh passed`.
