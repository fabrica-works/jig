# The engine's and the worker's services

Provisional, 2026-10-10, revised 2026-10-12. What an application writes
around its roots to make running processes: the protocol layer that
composes jig's components with its own, `iterate`, which carries every
event between io, the components and the root, and `main`, which reads
configuration, opens the store, starts the loop and ends it. It is to the
protocol layer what `../domain/root.md` is to the domain: one shape every
application copies, from the reference binary and its service (section
7). The mechanics are skein's programming-model.md, sections 2 and 4.
What is still open is listed in section 9.

## 1. In one page

- **Every process is the application's** (`../README.md`, section 3): its
  root, its protocol layer, its `iterate` and its `main`. jig supplies
  components of the protocol layer, as it supplies children of the root.
- **The protocol layer is a parent of components.** Each of jig's (the
  store's, the channel's, the accounts', the client's), each of smith's
  and skein's a process uses (smith's LLM component and skein's
  connections for runs in the process, smith's host half for agent
  processes), and each of the application's (its connectors' protocols,
  its credentials' sources) is a child with its own limits, worst case,
  deadlines and most outputs per entry point. The protocol layer routes
  between them, io and the root, and decides nothing.
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
- **The reference is jig's test application's:** its binary, `jig-test`,
  and its service, `jig-test-service`, step code written to be copied,
  like the reference root (section 7).

## 2. The engine's protocol layer

```
<application>-service                 iterate: io ↔ components ↔ root
├── <application>-domain              the root, with the journal (../domain/root.md)
└── the protocol layer
    ├── jig-protocol-store            commits, loads, transcripts, secrets (README.md, section 9)
    ├── jig-protocol-engine-workers   workers, if any (channel.md)
    ├── jig-protocol-accounts         LLM accounts, if it runs agents (llm.md)
    │   └── skein-oauth-accounts
    ├── smith-protocol-llm            smith-inline-agent's completions, for the engine's own slots (shape 2) (llm.md)
    │   └── skein-llm-connection
    ├── jig-protocol-engine-clients   parties' connections: the client's routes and files (client.md)
    ├── <connector>…                  each connector's protocol, and its webhooks' listener: the application's
    ├── <credentials>                 the application's credentials for runs' workspaces: its table, its sources
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
  the client's; a completion `smith-inline-agent` asks for, or its cancel,
  to smith's LLM component, with its slot's result contract (`llm.md`,
  section 3); an account wanted or released to the accounts'; an outbox
  entry to make, to its connector's protocol.
- **Calls decoded, answers rendered, in the routing:** a host tool's call
  the core hands out is decoded, and an answer rendered, by functions the
  routing calls, `jig-protocol-tools`' for the core's tools and the
  application's for its connectors', in the step that routes them
  (`channel.md`, 6.3); they keep no state.
- **smith's payloads, decoded and encoded in the routing:** the root
  hands out each of smith's payloads between its own hub and
  `smith-inline-agent`, and the routing decodes or encodes it with
  `smith-protocol-channel`'s functions, the charter against the engine's
  endpoints, keeping no state, and hands it back (`../domain/hosts.md`,
  6.8). As it decodes a charter, it keeps the slot's result contract
  beside smith's LLM component (`llm.md`, section 3).
- **Routing between components:** secrets' values go from the accounts'
  table, and from the application's table of its credentials, to the
  components that fill them in (`llm.md`, section 6; `channel.md`,
  section 8), never through the root.
- **Timers.** The protocol layer reports the earliest deadline of the root
  and every component, and fires each where it fires its own
  (programming-model.md, section 9); it reports the same apart from io's
  own, as the policy deadline (section 4).
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
   file (`llm.md`, 5.4), and the application's credentials' sources are
   read or started.
4. **The listeners opened:** the workers', the clients', and each
   connector's own for its webhooks. Workers may dial in and say hello
   during the restart, which needs them (`../domain/engine.md`, section
   6); clients are answered not ready until decisions open
   (`../client/domain/contract.md`, section 4).
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
- **The policy deadline.** skein's `Host` asks each service for its
  earliest deadline other than io's own, the one the teardown invariant
  watches (skein's `shell.md`, section 12, `next_policy_deadline`): a
  service's `iterate` reports there its root's and its components'
  deadlines, as it reports them for its loop; in one process (section
  10), over every root and protocol layer it owns.

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
    ├── jig-protocol-worker-link  the link to the engine; smith's payloads carried as bytes (channel.md)
    ├── smith-host-protocol       one channel per agent process, over its pipes; grants filled into its start
    ├── agent processes           each in a process group of its own, a contained tree once skein has them (README.md, section 10)
    ├── <credentials>             the application's table, filled from grants: the application's
    └── <workspace>               the workspace's io: the application's
```

- **No store.** A worker keeps nothing durable (`../domain/hosts.md`,
  section 4).
- **Agents as child processes.** The worker's root composes smith's host
  domain, which asks for an agent process per run; the protocol layer
  spawns the agent program its configuration names (`configuration.md`,
  section 4), with an environment holding no credential, in a process
  group of its own (skein's `io.md`, section 6), and gives smith's host
  half its pipes (smith's `protocol/hosts.md`, section 3). The program
  may be the application's own binary run as its `agent` command,
  smith's agent shell (`smith-agent-shell`).
- **smith's payloads, decoded and encoded in the routing,** as in the
  engine (section 2), the charter against the endpoints of the worker's
  agent configuration (`configuration.md`, section 4), and smith's facts
  decoded into jig's activity (`channel.md`, 6.1 and 6.4).
- **Credentials:** `jig-protocol-worker-link` takes grants' values out
  into the worker's tables, and the protocol layer hands LLM accounts'
  values to smith's host half, which fills them into each agent's start
  and grants (`channel.md`, section 8; smith's `protocol/hosts.md`,
  section 2); an application's credentials go to its workspace's io.
- **Or agents in the worker's process,** as the worker's root's choice,
  for one person's machine: a root that composes `smith-inline-agent`
  has smith's LLM and machine components in its protocol layer instead
  (smith's `protocol/README.md`, section 4; `llm.md`, section 2). A run
  is then no process of its own: no containment, and no watchdog on
  progress (smith's `domain/host.md`, 9.3).
- **Starting:** configuration; the components; the link dialled at once;
  the hub reports what it hosts once the link is up (`channel.md`,
  10.1).
- **Ending:** the hub cancels every run and admits no more; the worker is
  done once the engine has every turn and answer, or past its grace with
  no engine (`../domain/hosts.md`, 6.7).

## 7. The reference binary and service

- **The reference service, `jig-test-service`,** among jig's test kits
  but written as step code, so that it can be copied: jig's test
  application's engine protocol layer (the store's port, the accounts',
  smith's LLM component for its own slots' `smith-inline-agent`, the host
  tools' translation, the clients' side, the workers' side, its test
  connector's system answered in memory) and a test worker's (the link,
  smith's host half and its agent processes, the test workspace), each
  with its `iterate`, composed in one process (section 10), with a
  startup function that takes configuration's bytes and returns the
  service. jig's protocol world runs it (section 8).
- **The reference binary, `jig-test`,** beside it: its `main` reads its
  configuration with skein's startup kit (`configuration.md`), opens the
  store on a directory, and runs the engine and one test worker in one
  process (section 10), the worker's agents smith's real agent
  processes, run as its own `agent` command (section 6). A browser on
  loopback signs in by the link it prints (`client.md`, section 8). Its
  LLM endpoint is its configuration's. It is the template an application
  copies for its own `main`, and the process and live layers of jig's
  proofs (section 8).
- **`ops`'s service follows them,** as the reference for an application
  with agents in the engine and no workers.

`ops`'s engine is the second of `../domain/hosts.md`'s shapes: agents in
the engine, no workers.

- **Its crates:** `jig-ops-service`, its `iterate` and protocol layer:
  the store, the accounts', smith's LLM component for its
  `smith-inline-agent`'s runs, the host tools' translation
  (`jig-protocol-tools`, and its connectors' own), the client's
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
- **A worker's reference** is the reference service's test worker, which
  jig's channel world runs too (`channel.md`, section 12), since `ops`
  has none.

## 8. Testing

Each within jig's suites' budgets (`../../development/workflow.md`).

- **The domain worlds** run without any of this (`../domain/testing.md`).
- **The components' worlds** test each component alone (`channel.md`,
  section 12; `llm.md`, section 8; the store's design; `client.md`,
  section 14).
- **The protocol world:** the reference service (section 7), every
  component in it, over skein's simulated io (testing-strategy.md, 2.7):
  a store on the simulator's crashing disk, scripted parties'
  connections, skein's fake LLM peer, smith's scripted agent processes
  spawned on skein's fake machine beneath the test worker (smith's
  `protocol/hosts.md`, section 7), and the application's systems' fakes,
  the process killed and started again at drawn moments, and drained by
  a shutdown. Its referee is the conformance world's, seen through bytes:
  nothing a party, a host or a system saw depended on a commit that was
  lost; and a drained shutdown ends with nothing waiting for a commit.
- **The process layer** (development.md, 5.5): the reference binary,
  `jig-test`, started on a scratch directory with skein's fake LLM peer
  as a process of its own on loopback (skein's `fake-llm.md`) as its
  endpoint, a person's requests answered through a client on loopback,
  runs taken by smith's real agent processes, then the binary killed and
  restarted, and the work found where it was; it checks integration, as
  skein's real-loop tests do. `ops`'s binary proves its own shape the
  same way.
- **The live layer:** `jig-test` with a real LLM account as its endpoint
  and a real browser, a person signing in by the printed link and a
  request answered by a real model, within a bound on calls.

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
- **Agents as child processes,** as between hosts (section 6): one
  binary may serve as the service and each agent it spawns, as the
  reference binary does (section 7). Or, as a worker's root's choice, in
  the worker's process with `smith-inline-agent`, with no containment: a
  person's machine accepts that, a shared host does not.
- **Credentials as between hosts:** the engine grants them with each
  assignment over the in-process link (`channel.md`, section 8).
- **Starting and ending together:** configuration for all, then the
  engine's start (section 3), then each worker's (section 6). A signal
  drains the engine (section 5); once it has drained, the service ends
  the workers at once, with no grace, since no engine will come back
  while they live: their agent processes' groups are killed as io closes
  them, and their runs end with the service, as the engine's own slots'
  do. The next start settles their claims at once (`../domain/engine.md`,
  section 6).
- **Its world** is the protocol world of section 8, with workers: the
  engine's and a test worker's services in one loop, joined by in-process
  addresses, the test worker's agents smith's scripted agent processes.
  Joined sockets are io's own and never reach skein's simulator (skein's
  `io.md`, 3.5), so the link's cuts and stalls are the channel's protocol
  world's, over streams cut at random (`channel.md`, section 12).
