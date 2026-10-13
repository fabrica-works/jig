# jig's client side: where the time goes

The planner's sub-agent's audit, 2026-10-13, at jig `d91a89c45`:
`jig-view`, `jig-test-view`, `jig-client-world`, `jig-client-protocol-world`,
`jig-ops-view`, `jig-ops-world`. Medians of 30 alternating runs, before
and after, through `heavy --gate`; the machine's speed drifted (one
binary's floor 1.4 to 3.3 ms within minutes). Tried changes:
`tried/jig-client.md` (evidence, not to be applied as is). jig 03 takes
findings 1 and 2; finding 3 is skein 02.2's and jig 05.1's.

Together, on the six packages (752 tests), base and changed builds
interleaved in the gate lane: summed 34.7 s to 31.0 s, wall 4.40 s to
3.91 s (`jig-view` −1.8 s, `jig-test-view` −0.5 s, client world −1.0 s,
ops view −0.2 s).

| Test | Before | After |
|---|---|---|
| `jig-view` owner_history (typical) | 7.10 ms | 5.88 ms |
| `jig-view` owner_chip | 8.98 ms | 7.59 ms |
| client world, a navigation test | 10.07 ms | 6.64 ms |
| the joint story `signing_in…` | 49.6 ms | 45.4 ms |

The floor (an empty run of the same binary): `jig_view` 1.5 ms,
`jig_client_world` 2.2 ms (a 107 MB binary).

## 1. The phrasebook's load sorts and searches the long way (production, about 12 lines)

269 of `jig-view`'s 301 tests build the full English book (1,479
phrases), once per test process, 3.8 ms alone: 68 % of a typical test's
time above the floor. The same build runs in 92 client-world tests,
about 35 of `jig-test-view`'s, the protocol world and ops.
`crates/jig-view/src/phrases.rs:234` and `:267` sort 1,479 rows with
`sort_unstable`, unoptimized, about 0.7 ms each, though the input is
nearly sorted (30 descents); `:288` binary-searches every phrase's id
(0.85 ms in all), though both lists are sorted and of one length.
Tried: a stable `sort()` (which uses the sorted runs) and, before the
binary search, a check that `ids[position] == words.id`, the search only
on a mismatch, so error reports stay the same: the book 3.82 ms to
2.82 ms. The stable sort allocates up to n rows of scratch (about 47 KB)
during the load; with `sort_unstable` kept, the index check alone saves
about 0.6 ms. `Missing` and `Duplicate` are tested; `Unknown` is not.

## 2. `page_nodes` computes `card_nodes` about 15 times a call (production, about 50 lines in one file)

Each call costs 1.3 ms of arithmetic: `Limits::of_words` and `compose`
call it once, `jig-test-view`'s `for_world`
(`testing/jig-test-view/src/limits.rs:28,38`) twice; 3 of the client
fixture's 7.3 ms (`tests/client/src/test.rs:108-125`). It also runs in
test loops: `cards_objects.rs:1793` makes 16 calls (22.7 ms);
`family_terminals.rs:98-105` calls `view()` 9 times.
`crates/jig-view/src/sizes/pages.rs:12-142` calls `card_nodes(l)?` (76 µs
each, `sizes.rs:167`) in each helper; `owner_tree::max_nodes` and
`owner_project::max_nodes` cost about 147 µs each. Tried: `card` and
`chip` computed once in `page_nodes` and passed to private `_in`
helpers, the public functions thin wrappers over them
(`deployment_nodes` takes only `chip`, so its `None` cases stay): 1.33 ms
to 0.25 ms a call, the same value (1,166,880). About 440 calls a gate:
about 1.1 s summed, 0.14 s of wall. `page_capacity.rs:514-535` checks the
values and the overflows.

## 3. Generated worst cases check the whole limits table at every type

Every core-world construction takes 3.5 to 6 ms: `Store::new` about 45 %,
the restart drain (`constructors.rs:287`) about 40 %, `Heap::new` 0.4 to
0.8 ms. In `Store::new`, `protocol::sizes::check`
(`crates/jig-protocol-store/src/sizes.rs:75-88`) takes about 1.5 ms; the
tasks family's `Row::worst_case_bytes` 0.9 ms alone, its 474 generated
`worst_case_*` (`generated/tasks_v2.rs`, from `:1610`) each starting with
`check_limits(limits)`. About 1 ms a world.

## What runs before a test's first assertion

- `jig-view`: about 35 tests use no fixture and run at the floor; 269
  build the book (3.8 ms) and do their own work; 32 compute
  `Limits::of_words` (110 calls).
- `jig-client-world`: 137 wire tests at the floor (+0.2 ms); about 30
  session, front and engine tests build a core world (+7.7 ms); 48 build
  the client fixture (books 5.5 ms, bounds 4.7 ms; the domain's and the
  view's worst cases 0.07 and 0.01 ms); the 34 joint stories pay about
  15 ms of setup, then 40 to 130 ms of running: 6.3 s of the binary's
  10.7 s in the gate.
- The protocol world (440 ms a test in the gate): 7 ms of fixture (3.2 ms
  less with 1 and 2), 3 to 6 ms a world; five tests run two seeds, each
  starting with `ready()` (sign in, start a chat), about 50 ms
  (`tests/client-protocol/src/stories.rs:7,17`). Within a run, polling the
  engine, HTTP bytes and the core 60 to 65 %, the client's callbacks 25 to
  30 %; the secrets referee formats every domain event with `{:?}`
  (`referee.rs:45-46`), 5 to 8 %.

## Looked at, not worth changing

- The book as a `const`: it owns its data (each phrase's text is copied
  to the heap); building it from static data would save 0.9 ms more
  (`language()` 0.65 ms, `shapes()` 0.24 ms), a larger change of the type.
- `Limits::of_words(&CLIENT)`: 1,182,384 nodes, 1.21 GB of text bytes,
  but `Builder::new` (`builder.rs:48`) reserves without touching: 15 to
  41 µs.
- Store snapshots and the truth comparison (0.06 to 1.4 ms a story);
  `Domain::new` and start (0.1 ms); `World::new` (0.2 ms); hoisting the
  test loops once 2 is in (4 to 6 ms each); a typed protocol referee (a
  walk over every event type); the joint stories' run, which has no
  single hot spot; ops's three books per client
  (`tests/ops/src/client/app.rs:92-110`), ops being frozen.

## Surprises

- `jig-client-world`'s unit-test binary costs 0.5 to 2 ms more per process
  than the crate's own integration binaries (274 more page faults per
  start): about 0.5 s of gate time over 266 tests, from its size alone.
