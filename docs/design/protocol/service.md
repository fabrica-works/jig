# The engine's and the worker's services

Provisional, 2026-10-10. What an application writes around its roots to
make running processes: the protocol layer that composes jig's
components with its own, `iterate`, which carries every event between
io, the components and the root, and `main`, which reads configuration,
opens the store, starts the loop and ends it. It is to the protocol layer
what `../domain/root.md` is to the domain: one shape every application
copies, from the reference service (section 7). The mechanics are skein's programming-model.md,
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
<application>-service                 iterate: io ↔ components ↔ root
├── <application>-domain              the root, with the journal (../domain/root.md)
└── the protocol layer
    ├── jig-protocol-store            commits, loads, transcripts, secrets (README.md, section 9)
    ├── jig-protocol-engine-workers   workers, if any (channel.md)
    ├── jig-protocol-accounts         LLM accounts, if it runs agents (llm.md)
    │   └── skein-oauth-accounts
    ├── smith-protocol-llm            the inline agent's completions, if it runs agents there (llm.md)
    │   └── skein-llm-connection
    ├── jig-protocol-engine-clients   parties' connections (client.md)
    ├── <connector>…                  each connector's protocol: the application's
    └── timers                        the root's and every component's deadlines
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
  the client's; a completion to smith's LLM component; an account wanted
  or released to the accounts'; an outbox entry to make, to its
  connector's protocol.
- **Calls decoded, answers rendered, in the routing:** a host tool's call
  the core hands out is decoded, and an answer rendered, by functions the
  routing calls, `jig-protocol-tools`' for the core's tools and the
  application's for its connectors', in the step that routes them
  (`channel.md`, 6.3); they keep no state.
- **Routing between components:** secrets' values go from the accounts'
  table to the components that fill them in (`llm.md`, section 6),
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
   configuration; skein's accounts driver loads each account's private
   file (`llm.md`, 5.4).
4. **The listeners opened:** the workers' and the clients'. Workers may
   dial in and say hello during the restart, which needs them
   (`../domain/engine.md`, section 6); clients are answered not ready
   until decisions open (`../client/domain/contract.md`, section 4).
5. **The root's restart:** `iterate` gives the root its start, and the
   core's script asks for one step at a time; the protocol layer does the
   store's and the connectors' parts of each step as the root asks; the
   core wants every configured account from its first step
   (`../domain/root.md`, section 8; `../domain/engine.md`, section 12).
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
  2. **The core hears that the engine is stopping**
     (`../domain/engine.md`, 6.1) and from then on takes nothing that
     would make a commit: a party's request is answered not ready; a run's
     turn, answer or call is answered busy, so its host keeps it; a
     connector's events and outcomes are held by the connector, as when
     the journal is full; no run is placed and no procedure stepped. It
     ends every watch saying the engine is stopping
     (`../client/domain/contract.md`, 7.2).
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
<application>-worker-service      iterate
├── <application>-worker          the worker's root: jig-host, smith's host domain, its workspace
└── the protocol layer
    ├── jig-protocol-worker-link  the link to the engine (channel.md)
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

- **The test application's service first.** Until `ops`'s service is
  built, the reference is `jig-test-service`, among jig's test kits but
  written as step code, so that it can be copied: jig's test
  application's engine protocol layer (the store's port, the accounts',
  smith's LLM component, the host tools' translation, the clients' side,
  the workers' side, its test connector's system answered in memory) and
  a test worker's, each with its `iterate`, composed in one process
  (section 10), with a startup function that takes configuration's bytes
  and returns the service. jig's protocol world runs it (section 8).
  What follows is `ops`'s, to be built after it.

`ops`'s engine is the second of `../domain/hosts.md`'s shapes: agents in
the engine, no workers.

- **Its crates:** `jig-ops-service`, its `iterate` and protocol layer:
  the store, the accounts', smith's LLM component, the host tools'
  translation (`jig-protocol-tools`, and its connectors' own), the client's
  protocol, and its two connectors' protocols to the shared
  production's observability and infrastructure APIs; and `jig-ops`, its
  `main`, reading its configuration and running the loop.
- **Its periods** come from a child of its root that keeps the
  deployment's calendar from its configuration (a monthly period, from a
  day, at a fixed offset from UTC; a day past a short month's end is its
  last day; the projects a period opens, each with its budget, listed by
  the calendar's configuration, so a project not listed has no period
  until configuration names it and the engine restarts), arms its
  deadline, and says each period to the
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
  section 12; `llm.md`, section 8; the store's design; `client.md`,
  section 14).
- **The protocol world:** the reference service (section 7), the test
  application's until `ops`'s is built, every component in it, over
  skein's simulated io (testing-strategy.md, 2.7): a store on the
  simulator's crashing disk, scripted parties' connections, skein's fake
  LLM peer and the application's systems' fakes, the process killed and
  started again at drawn moments, and drained by a shutdown. Its referee
  is the conformance world's, seen through bytes: nothing a party, a host
  or a system saw depended on a commit that was lost; and a drained
  shutdown ends with nothing waiting for a commit.
- **The real loop:** an application's binary started on a scratch
  directory, a party's requests answered, killed and restarted; it checks
  integration, as skein's real-loop tests do. `ops`'s binary is the first,
  once built.

## 9. Open questions

- **Generating the routing:** `iterate`'s routing has one shape, as the
  root's does; whether it is copied and filled, or generated.
- **Supervision:** what restarts an engine that stopped, and how often,
  is the deployment's; whether jig's guidance names one.

## 10. One process

The first shape built (`README.md`, section 4): an engine and its workers
in one process, for one person's machine, development and tests, from the
same parts as between hosts.

- **One service, one loop.** The application's `iterate` owns the
  engine's protocol layer and each worker's as children, each with its
  own root and journal, and routes io's events by the tokens each gave.
  They share io and the loop, and nothing else: no root reads another's
  state, and nothing crosses between them but bytes.
- **Joined by in-process addresses.** The engine's workers' listener and
  each worker's link use an in-process address (skein's `io.md`, 3.5),
  so the channel's opening, authentication, liveness, relays and grants
  run as they do between hosts, in plaintext, with no TLS.
- **A browser on loopback.** The engine's clients' listener is a TCP
  listener on a loopback address; the browser reaches it as it reaches
  any engine, and a person signs in by the link the engine prints at
  start (`client.md`, section 8).
- **Agents in the worker's process,** in smith's one-process shape
  (smith's `domain/host.md`, section 9): the worker's root runs each run
  as a composed smith domain, as the inline agent does, and its protocol
  layer owns smith's LLM and machine components (smith's
  `protocol/README.md`, section 4). Commands and checks run as smith's
  machine component runs them, as in every shape (`README.md`,
  section 10). A run is then no process of its own: a person's machine
  accepts that, a shared host does not.
- **Credentials as between hosts:** the engine grants them with each
  assignment over the in-process link, and the worker's protocol layer
  fills smith's LLM component's table from them (`llm.md`, section 6).
- **Starting and ending together:** configuration for all, then the
  engine's start (section 3), then each worker's (section 6); a signal
  drains the engine (section 5) while the workers stop their runs and
  hand in what they hold, and the process ends when both have.
- **Its world** is the protocol world of section 8, with workers: the
  engine's and a test worker's services in one loop, joined by in-process
  addresses. Joined sockets are io's own and never reach skein's
  simulator (skein's `io.md`, 3.5), so the link's cuts and stalls are the
  channel's protocol world's, over streams cut at random
  (`channel.md`, section 12).
