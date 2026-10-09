# Testing the client

Provisional, 2026-10-09. How jig's client domain and view are tested, and
how they test an application's client: the person who acts on the view,
the client's world with a scripted engine, the joint world with the
engine's domain, their referee and their faults. It applies skein's
`testing-strategy.md` as `../../domain/testing.md` does for the engine;
the tiers below the domain come with the client's protocol layer. What is
still open is listed in section 10.

## 1. In one page

- **The person acts on the view,** as a person does: finds nodes in the
  tree by role, name and text, presses them, types into them, and waits
  to see what follows. Never by style, by position, or through the
  client's state.
- **One person, every tier.** The person is a script that asks its face to
  find and act, so a scenario written once runs on the tree face in every
  world, and on a browser's accessibility tree once the browser tier
  exists.
- **The client's world comes first.** The real domain and view, a scripted
  engine that keeps the contract (contract.md), the shell played by the
  world, and every fault of section 7. Most of the client's tests live
  there.
- **The joint world is the check.** The same people and scenarios in
  front of the engine's real domain: jig's core with its test connector,
  or an application's whole engine domain on jig's conformance world.
- **The referee watches what the person saw and what the engine made
  durable,** never the client's state: nothing shown done before it was
  durable, nothing made twice, nothing sent lost.
- **Every world builds and diffs every page,** and applies the patches to a
  model of the last tree, which must then equal the new one.

## 2. The tiers

| Tier | Real | Faked |
|---|---|---|
| step tests | one step function of the domain or the view; the diff; the Markdown reader; words | its events, by hand |
| the client's world | the client root, its domain and its view, for jig's test application or a real one | the engine, scripted; the shell, played by the world; the person |
| the joint world | the client root, domain and view, and the engine's domain, root and all | the store, hosts and systems, as the engine's worlds fake them; the shell; the person |
| the protocol world, the real loop, the browser | with the client's protocol layer and shells | |

Every world runs on skein's world harness: one loop, one seed, the
referee in the loop, the heap counted at every iteration
(testing-strategy.md, sections 6 and 7).

## 3. The person

`testing/jig-fake-person`, ordinary Rust like jig's other kits, shaped as
a step machine so that a browser's loop can run it later.

- **What it looks for:** a target by role and words in its name, or by
  its text, within regions named outermost first: "the button named
  Accept, in the region named Proposal from I".
- **What it does,** a step at a time: go to an address; press; type into
  a field; send, as Enter in a composer; see something, within a time;
  see it gone, within a time; reload; pause.
- **What it reports:** what it does next; that it waits until the tree
  changes or a time passes; that it is done; or that it failed, at which
  step and why ("no button named Release in the region named T7").
- **The tree face** answers its queries on the view's tree at once, the
  way an accessibility tree is searched, and turns its acts into the
  acts the shell would report.
- **A random person,** for the fuzzy suite, presses any bound node and
  types any words drawn from the seed, so a sweep reaches what no script
  names.
- **The browser's face** comes with the browser tier, over skein's
  browser kit (skein's `browser.md`), as a crate of its own.

## 4. The client's world

`tests/client`, package `jig-client-world`.

- **Clients:** each a tab or a device of a party: its client root, its
  view root, its small store, its address history. Several per party,
  and several parties.
- **The scripted engine** (`testing/jig-scripted-engine`) keeps the
  contract for jig's documents: requests answered by key, after a
  commit it scripts, and answered again alike; refusals as scripted;
  documents for every read; watches delivered level-triggered, a delivery
  in flight at a time; a run's text written out over time. It holds the
  truth the referee compares with: what it committed, and when.
- **The application's parts:** a client world for an application adds its
  families' scripted documents and requests beside jig's, through the
  scripted engine's hooks, which are ordinary Rust.
- **jig's test application:** jig's own client world runs a client root of
  jig's own, with one test family whose parts fill every kind of slot,
  as the test connector stands in for an application in the engine's
  worlds (`../../domain/testing.md`, section 7).
- **Its stories** are client.md's, section 17, and pages.md's, section 10,
  each written once in a person's terms.

## 5. The joint world

The client's world, with the engine's domain in place of the scripted
one:

- **For jig:** jig's core world under its test root
  (`../../domain/testing.md`, section 4), its parties replaced by clients
  with the person in front.
- **For an application:** its whole engine domain on jig's conformance
  world, with its client roots and the people of its stories: `ops`'s P
  accepting a restart from the incident's page.
- **The translation** between the contract's values and the engine's
  people and views vocabulary is written once, in the world, as the
  protocol layers will carry it as bytes.
- **The engine's faults** reach the person: crashes at commits, held
  commits, restarts, as the conformance world makes them.

## 6. The referee

It watches what each person saw (the view's trees, as the person read
them) and what the engine made durable (the scripted engine's commits, or
the engine's store), never a client's state. Its rules:

- **Nothing shown done before it was durable:** no outcome of a request
  is shown before the commit that answers it.
- **Made once:** every request a person made is made once by the engine,
  whatever the faults: drops, restarts, double presses, reloads.
- **Nothing sent lost:** every word a person sent and was told was sent
  is in its task's activity, or its task ended.
- **What they saw is what they decided:** every act that names a
  revision (contract.md, 5.2) names the one the card the person acted on
  showed.
- **A card leaves for a reason:** a card leaves only when its request was
  answered or its object stopped waiting, and says which.
- **The link is told:** while the engine was unreachable past a
  heartbeat, the person was shown offline.
- **Bounded:** the client and the view within their limits, the heap
  within their worst case.

## 7. Faults

Drawn from the seed:

- a watch dropped; a delivery slow;
- the engine restarting: offline, then answers by key;
- busy and not ready answers;
- no answer: a connection lost after the engine committed;
- a double press; two tabs deciding one thing;
- a reload: the client's own restart, its state gone but for its small
  store;
- a press on a node just removed; an object gone under its card;
- signed out with requests pending; another party signing in on a client
  with kept work;
- the engine outside the client's versions.

## 8. What an application adds

- its families' scripted documents and requests, in its client world;
- its client and view roots, in its client world and in its joint world
  on its conformance world;
- its stories, written in its people's terms;
- its own shells' tiers, if it writes shells.

## 9. Budgets

The client's tests fit the suites' budgets (testing-strategy.md,
section 8; `../../domain/testing.md`, section 9): the focused suite runs
a few seeds per story and each fault once, the fuzzy suite sweeps random
people over random stories. The browser tier, when it comes, is a suite
of its own, run by the gate.

## 10. Open questions

- **The joint world's cost:** whether every conformance scenario should
  also run with people in front, or only the stories about people.
- **The browser tier's budget,** set when its first tests are built.
- **Coverage of the roots' arms,** as for the engine's root
  (`../../domain/testing.md`, section 10).
