# The engine's and the worker's services

Provisional, 2026-10-10. What an application writes around its roots to
make running processes: the protocol layer that composes jig's
components with its own, `iterate`, which carries every event between
io, the components and the root, and `main`, which reads configuration,
opens the store, starts the loop and ends it. It is to the protocol layer
what `../domain/root.md` is to the domain: one shape every application
copies, from `ops`. The mechanics are skein's programming-model.md,
sections 2 and 4. What is still open is listed in section 9.

## 1. In one page

- **Every process is the application's** (`../README.md`, section 3): its
  root, its protocol layer, its `iterate` and its `main`. jig supplies
  components of the protocol layer, as it supplies children of the root.
- **The protocol layer is a parent of components.** Each of jig's (the
  store's, the channel's, the accounts', the client's), each of smith's
  and skein's an engine uses (the LLM component and its connections), and
  each of the application's (its connectors' protocols) is a child with
  its own limits, worst case, deadlines and most outputs per entry point.
  The protocol layer routes between them, io and the root, and decides
  nothing.
- **The journal is the root's only way out,** and `iterate` its only
  reader: commits go to the store, held outputs leave once the store has
  answered, outputs addressed to a child come back as events
  (`../domain/root.md`, section 3).
- **Start in order:** configuration, then the store opened, then the
  components, then the listeners, then the core's restart script, then
  decisions (section 3).
- **End in order:** a failed commit stops at once; a shutdown drains until
  nothing waits for a commit, then closes everything, as skein's owner
  close asks (programming-model.md, 5.2).
- **`ops` is the reference:** its service and its main are written to be
  copied, like its root.

## 2. The engine's protocol layer

```
<application>-service            iterate: io ↔ components ↔ root
├── <application>-domain          the root, with the journal (../domain/root.md)
└── the protocol layer
    ├── the store                 jig's: commits, loads, transcripts, secrets (README.md, section 9)
    ├── jig-engine-channel        workers, if any (channel.md)
    ├── jig-engine-accounts       LLM accounts, if it runs agents (llm.md)
    ├── smith-protocol-llm        the inline agent's completions, if it runs agents there (llm.md)
    │   └── skein-llm-connection
    ├── the client's protocol     parties' connections (README.md, section 14)
    ├── <connector>…              each connector's protocol: the application's
    └── timers                    the root's and every component's deadlines
```

- **Routing from io:** each io event goes to the component whose token it
  carries (programming-model.md, 4.2).
- **Routing from components to the root:** what a component hears from its
  peer, translated into the domain's vocabulary (the core's or the hub's,
  wrapped, or a connector's) and with the application's own parts decoded
  by the application's codecs, becomes one root event. A component never
  steps the root itself.
- **Routing from the root to components:** what the journal releases, and
  what leaves through its door, goes to the component it names: a commit
  or a load to the store; an assignment, a relay's answer, an
  acknowledgement, a credential grant or a cancel to the channel's
  component; a party's answer, a read's answer or a watch's delivery to
  the client's; a completion to smith's LLM component; a refresh to the
  accounts'; an outbox entry to make, to its connector's protocol.
- **Calls decoded, answers rendered, in the routing:** a host tool's call
  the core hands out is decoded, and an answer rendered, by functions the
  routing calls, `jig-engine-tools`' for the core's tools and the
  application's for its connectors', in the step that routes them
  (`channel.md`, 6.3); they keep no state.
- **Routing between components:** secrets' values go from the accounts'
  component to the components that fill them in (`llm.md`, section 6),
  never through the root.
- **Timers.** The protocol layer reports the earliest deadline of the root
  and every component, and fires each where it fires its own
  (programming-model.md, section 9).
- **Room.** `iterate` hands the root an event only while the root's
  output queues have room for the most that event's entry point may
  produce (programming-model.md, section 2). Whether a decision fits the
  journal is the root's to say: it answers busy, through the door, an
  event the journal cannot take (`../domain/root.md`, section 3), and the
  component that sent it holds its peer back until it may send again.

## 3. Starting

1. **Configuration,** read and checked whole before the loop
   (`configuration.md`).
2. **The store opened,** both of its stores, and its own records read
   (`README.md`, section 9). A store that refuses (another format,
   records that disagree with its own header, a recovery that fails) ends
   the start.
3. **The components made,** with their limits and their parts of the
   configuration; the accounts' component reads its kept records from the
   secrets' store (`llm.md`, 5.4).
4. **The listeners opened:** the workers' and the clients'. Workers may
   dial in and say hello during the restart, which needs them
   (`../domain/engine.md`, section 6); clients are answered not ready
   until decisions open (`../client/domain/contract.md`, section 4).
5. **The root's restart:** `iterate` gives the root its start, and the
   core's script asks for one step at a time; the protocol layer does the
   store's and the connectors' parts of each step as the root asks, and
   hands the core its accounts with its own records in the first
   (`../domain/root.md`, section 8; `llm.md`, 5.4).
6. **Decisions open** once the core says so; from then on the root takes
   every event.

## 4. iterate

The shape of skein's `iterate` (programming-model.md, section 2), with
these passes:

```
up:    io completions → components → root events → root.step (one decision each)
       journal: store answers → committed/failed → released outputs
down:  released and door outputs → components → io submissions
       outputs addressed to a child → root events, next pass
```

- **One decision per root event,** and at most one commit; a decision's
  commit goes to the store in the down pass of the iteration that made
  it.
- **Released outputs leave in order,** a bounded number per call
  (`../domain/root.md`, section 4).
- **A store's answer is always taken,** however full the journal, so the
  bound on commits in flight never stops the answer that frees it
  (`../domain/engine.md`, 5.1).
- **Nothing waits in `iterate`** between iterations but what its
  components and the root hold as state.

## 5. Ending

- **A failed commit stops the engine.** The journal releases nothing
  tagged with it or after it (`../domain/root.md`, section 4); `iterate`
  aborts every component, which closes without draining; `main` ends the
  process with a failure status, and a supervisor starts it again, which
  recovers from the last durable commit.
- **A shutdown drains,** from a signal or an operator:
  1. **The listeners stop admitting:** workers' openings refused as
     stopping, clients' requests answered not ready.
  2. **The core hears that the engine is stopping** (`README.md`,
     section 12) and from then on takes nothing that would make a commit:
     a party's request is answered not ready; a run's turn, answer or call
     is answered busy, so its host keeps it; a connector's events and
     outcomes are held by the connector, as when the journal is full; no
     run is placed and no procedure stepped. It ends every watch saying
     the engine is stopping (`../client/domain/contract.md`, 7.2).
  3. **What is in flight finishes:** the commits made before are answered,
     and their held outputs released, until nothing waits for a commit,
     which the core says.
  4. **Each component closes,** its peers told, and reports closed; the
     store closes last.
  5. **`main` ends the process** once everything it owns has reported
     closed (programming-model.md, 5.2).

  A second signal turns the drain into an abort: what is in flight is
  dropped, which a restart recovers from, as from a crash.
- **Runs in the engine** die with it: the next start settles their claims
  (`../domain/engine.md`, section 6). Runs on workers go on for the
  engine's grace and are adopted when their workers dial in again.

## 6. A worker's service

```
<application>-worker-service     iterate
├── <application>-worker          the worker's root: jig-host, smith's host domain, its workspace
└── the protocol layer
    ├── jig-worker-channel        the link to the engine (channel.md)
    ├── smith-host-protocol       one channel per agent process, over its pipes
    ├── contained processes       the agent's tree, from skein (README.md, section 10)
    └── <workspace>               the workspace's io: the application's
```

- **No store.** A worker keeps nothing durable (`../domain/hosts.md`,
  section 4).
- **Starting:** configuration; the components; the link dialled at once;
  the hub reports what it hosts once the link is up (`channel.md`,
  10.1).
- **Ending:** the hub cancels every run and admits no more; the worker is
  done once the engine has every turn and answer, or past its grace with
  no engine (`../domain/hosts.md`, 6.7).

## 7. `ops` as the reference

`ops`'s engine is the second of `../domain/hosts.md`'s shapes: agents in
the engine, no workers.

- **Its crates:** `jig-ops-service`, its `iterate` and protocol layer:
  the store, the accounts', smith's LLM component, the host tools'
  translation (`jig-engine-tools`, and its connectors' own), the client's
  protocol, and its two connectors' protocols to the shared
  production's observability and infrastructure APIs; and `jig-ops`, its
  `main`, reading its configuration and running the loop.
- **Its periods** come from a child of its root that keeps the
  deployment's calendar from its configuration (a monthly period, from a
  day, in a time zone), arms its deadline, and says each period to the
  core with the next one's start; the protocol layer only fires the
  deadline (`configuration.md`, section 5).
- **Commented to be copied,** as its root is: a new application copies it,
  removes what its shape does not need (the LLM path and the tools,
  without agents in the engine; the channel, without workers), and adds
  its connectors' protocols.
- **A worker's reference** is the test worker of jig's channel world
  (`channel.md`, section 12), since `ops` has none.

## 8. Testing

Each within jig's suites' budgets (`../../development/workflow.md`).

- **The domain worlds** run without any of this (`../domain/testing.md`).
- **The components' worlds** test each component alone (`channel.md`,
  section 12; `llm.md`, section 8; the store's design; the client's
  protocol).
- **`ops`'s protocol world:** its service, every component in it, over
  skein's simulated io (testing-strategy.md, 2.7): a store on the
  simulator's crashing disk, scripted parties' connections, skein's fake
  LLM peer and the fake production's APIs, the process killed and
  started again at drawn moments, and drained by a shutdown. Its referee
  is the conformance world's, seen through bytes: nothing a party, a host
  or a system saw depended on a commit that was lost; and a drained
  shutdown ends with nothing waiting for a commit.
- **The real loop:** `ops`'s binary started on a scratch directory, a
  party's requests answered, killed and restarted; it checks integration,
  as skein's real-loop tests do.

## 9. Open questions

- **Several roots in one process** (`../domain/root.md`, section 14): an
  engine and a worker in one process, for one person's machine and for
  tests, with the link direct rather than over a channel.
- **Generating the routing:** `iterate`'s routing has one shape, as the
  root's does; whether it is copied and filled, or generated.
- **Supervision:** what restarts an engine that stopped, and how often,
  is the deployment's; whether jig's guidance names one.
