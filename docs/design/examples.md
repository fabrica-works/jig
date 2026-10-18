# The example application

Provisional, 2026-10-06, revised 2026-10-18. This is jig's example
application, `ops`: a small system for running services in production,
on two connectors, with its agents inside its engine. It exists to be
tested and to be copied.

- Its root is the reference root every application starts from
  (`README.md`, 6.7).
- Its domain is the subject of jig's conformance world (`README.md`,
  6.6).
- As a second application, it is what keeps jig's vocabulary from
  being one application's in disguise.

What is still open is listed in section 11.

## 1. In one page

- **Production management, kept small.**
  - Agents watch services' logs and metrics, triage alerts, and run
    incidents.
  - Procedures restart, scale and provision.
  - People approve what is risky, and ask for environments in a chat.
- **Chosen for what it shares with an application that writes code,
  which is nothing:**
  - no code, no workspaces;
  - two connectors where such an application may have one;
  - a stream of events, mostly noise, where it gets occasional hints;
  - effects on shared and scarce resources that cost money, where its
    effects are on objects it owns;
  - agents inside the engine, where its agents are on workers.
- **Two connectors:**
  - **observability:** services' logs, metrics and alerts;
  - **infrastructure:** services, environments and quotas.
- **Agents run inside the engine** (`README.md`, section 9, the second
  shape). Their tools are reads, so `ops` has no worker. One story has no
  agent at all (the first shape).
- **Its client is jig's reference UI on `jig-client`,** with two
  families of its own, incidents and environments, and its server
  mounts jig's clients' component with jig's default HTTP mapping
  (section 8).
- **Its systems are fakes:** one fake production that both connectors
  see. A real backend may be the local machine's own services
  (section 9).
- **It stays an example.** It is small, its systems are fakes, and no
  application imports it. An application that drives production writes
  connectors of its own.

## 2. Layout

```
examples/ops/domain                   jig-ops-domain                   the engine's root: the core and two connectors, routed
examples/ops/domain-observability     jig-ops-domain-observability     the observability connector
examples/ops/domain-infrastructure    jig-ops-domain-infrastructure    the infrastructure connector
examples/ops/domain-connectors        jig-ops-domain-connectors        its connectors' numbers, which the engine's root and its client share
examples/ops/client-domain            jig-ops-client-domain            its client's domain: jig-client held, with incidents and environments on it
examples/ops/client-protocol-wire     jig-ops-client-protocol-wire     its families' wire records, for both ends, from schemas
examples/ops/client-view              jig-ops-client-view              its presentation: the reference UI, with incidents' and environments' views
testing/jig-ops-fake-production       jig-ops-fake-production          the fake production: services, quotas, logs, metrics
tests/ops                             jig-ops-world                    its worlds, on jig's conformance world
```

`ops` standalone (9.1) adds its connectors' protocols,
`jig-ops-protocol-observability` and `jig-ops-protocol-infrastructure`;
its service, `jig-ops-service`, with its server; its binary, `jig-ops`;
its browser shell, `jig-ops-client-shell-web`; and the fake
production's own binary, `jig-ops-production`.

## 3. Its connectors

### 3.1 Observability

- **Resources:**
  - services, named by environment and service;
  - their log and metric streams;
  - alert rules.
- **Topics:** a service's alerts, and its health. The connector
  classifies each alert for each subscriber: it wakes the subscriber, is
  kept in its inbox, or is dropped. A storm of alerts reaches a task as
  one wake, batched by its wake policy.
- **Reads for agents:**
  - a service's logs, filtered, in a window of time;
  - a metric's series;
  - the alerts that fired.

  Each answer is bounded in bytes and charged to the connector's request
  budget.
- **Facts:** a service's error rate and health. They are read afresh
  before any decision that depends on them, including another
  connector's (3.3).
- **Effects:** silencing an alert for a while. It is keyed, and owned by
  the task that made it.
- **Incidents,** its own records: opened by its watch from a triage's
  verdict, each with the alerts it covers, the service, a severity, and
  its lead, the task the watch made to lead it, once the core has said
  which (7.1).

### 3.2 Infrastructure

- **Resources:**
  - services: their replicas, version, and state;
  - environments, named by pool and name;
  - quotas: a pool's capacity, a team's budget in money.
- **Effects,** each with its recovery class (`domain/connectors.md`,
  4.3):
  - restarting a service: keyed in the fake production, which takes an
    operation's id with each restart; on a backend that offers none,
    such as systemd's units, unrecoverable, so an uncertain restart
    holds its task for the person on call;
  - scaling it: a set of its replicas, conditional on the count decided
    from;
  - rolling it back to a version: conditional, only from the version
    decided on;
  - creating an environment: keyed;
  - tearing an environment down: conditional on the environment being
    the one created under its key.
- **Prices:** an environment is priced at its maximum, for the time asked
  for; one torn down early returns the rest.
- **Write holds:** a service being remediated is held by the task that
  remediates it, so two incidents never restart it at once.
- **Scarce resources:** a pool's environments are a quota. A task that
  asks for a slot when the pool is full waits for one, rather than being
  refused (`domain/tasks.md`, section 6).
- **Procedures:**
  - **remediate:** make the effect, then wait until the service is
    healthy or a deadline passes;
  - **provision:** create the environment, then wait until it is ready;
  - **tear down:** at release, when the task holding an environment
    closes, the environment is torn down.

  Each is level-triggered: it decides from the facts as they are.
- **Drift:** someone restarts or scales a service by hand, or deletes an
  environment `ops` made. Each is found by reading afresh, and holds the
  task that relies on it, saying what changed (`domain/connectors.md`,
  section 11). Nothing is written over.

### 3.3 Between the two

The infrastructure connector's effects need observability's facts:

- a restart in production needs a healthy replica elsewhere;
- a scale-down needs its load below a threshold for half an hour.

These are requirements whose facts are one connector's while the effect
is another's (`domain/authority.md`, section 10). The policy names
observability as their judge; the core asks it for its verdict through
the root, and observability judges from its own facts
(`domain/connectors.md`, section 7). Both are *observed:* infrastructure
cannot check observability's facts as it applies the effect, so each
holds when the effect is decided, within a freshness of a minute, and
load may rise before the scale-down lands. `ops`'s policy accepts that;
a policy that could not would mark them as needing a guard, and these
effects would be refused.

A remediation's own wait for the service to be healthy afterwards is
its procedure's mechanics, not a requirement.

## 4. Its root

```
jig-ops-domain                    the engine's root
├── core                          jig's core
├── observability                 jig-ops-domain-observability
├── infrastructure                jig-ops-domain-infrastructure
├── calendar                      the deployment's funding periods (protocol/configuration.md, section 5)
├── host                          jig-domain-host: hosted runs on the engine's slots (domain/hosts.md, section 6)
└── agent                         smith-inline-agent: smith's domains, one per run in flight (domain/hosts.md, 5.2)
```

- **It has the one shape** of `README.md`, 6.3. Every write and every
  output goes into the journal.
- **It routes, and nothing else:**
  - between the core and each connector, in their vocabularies;
  - between the core and jig's host, which hosts the agents, and
    between the host and its agent, `smith-inline-agent`, with the same
    functions a worker's root uses between its host and smith's host
    domain;
  - the verdicts the core asks of observability on infrastructure's
    effects (3.3);
  - between the calendar and the core: each period it says.
- **Its periods** are its calendar's, a child that keeps the
  deployment's calendar from its configuration: a monthly period from a
  day, at a fixed offset from UTC, a day past a short month's end being
  its last day; the projects a period opens, each with its budget,
  listed by the calendar's configuration, so a project not listed has no
  period until configuration names it and the engine restarts. It arms
  its deadline and says each period to the core with the next one's
  start; the protocol layer only fires the deadline
  (`protocol/configuration.md`, section 5).
- **It is the reference root.** Its routes, its translations and its
  sum of worst cases are written to be copied, and are commented as
  such.

## 5. Its executors

- **Agents, in the engine.** Each charter has a model, reads, a budget
  and a result contract:
  - **triage:** a cheap model, with observability's reads; its result
    is a verdict from a closed list, noise or incident, the incident's
    with its fields: the alerts it covers, the service, a severity. It
    delegates nothing;
  - **incident lead:** a coordinator, with both connectors' reads; it
    delegates remediation and finishes with a report;
  - **environment chat:** a person's assistant, which plans an
    environment and asks for it.
- **Procedures:**
  - observability's **watch:** a standing task subscribed to a set of
    services' alerts. For each batch of alerts that wakes it, it makes a
    triage task from the triage's batch observability keeps for the
    watch; on a triage's verdict of incident, it opens the incident and
    makes its lead from the lead's batch kept beside it (each its
    charter, budget and words). Neither is one of the configuration's
    templates (`protocol/configuration.md`, 3.1);
  - infrastructure's **remediate, provision and tear down** (3.2);
  - the core's **recurring** tasks, each making its recurring batch at
    each period (`domain/tasks.md`, section 10).
- **People:**
  - the person on call, who approves effects beyond policy and answers
    questions during an incident;
  - developers, who ask for environments.

## 6. Its policy

- **Staging:** agents' tasks may restart and scale.
- **Production:** every effect is beyond any agent's authority, and so
  beyond its delegates': the task that wants one proposes it, for the
  person on call to accept (`domain/tasks.md`, section 9).
- **Money:** each team has a budget per month, spent by environments,
  and a person's pool within it.

These are policies as data, in authority's terms
(`domain/authority.md`). `ops` writes no rule of its own in code.

## 7. Its stories

### 7.1 An alert

```
P, the person on call
└─ W   watch the shop's services       procedure: observability's watch, subscribed to alerts
   ├─ T1  triage alerts 7–9            agent, triage: noise
   ├─ T2  triage alert 12              agent, triage: incident
   └─ I   incident: checkout errors    agent, an incident lead; tracked
      └─ R  restart checkout           infrastructure's remediate, holding the checkout service
                                       in production, so a proposal P accepted
```

- **The storm** of alerts 7 to 9 wakes W once. T1 reads the logs, finds
  a known, harmless error, and finishes: noise. W hears T1's result, and
  does nothing more.
- **Alert 12** is triaged as an incident. T2 finishes with that verdict
  and the incident's fields (alert 12, checkout, its severity), and
  makes nothing itself. W hears T2's result, as a requester hears its
  delegate's (`domain/tasks.md`, 8.1), and steps on it: it opens the
  incident from the verdict's fields and makes I, a tracked task, from
  the lead's batch, in that step, keeping in its state which incident
  the step's task will lead. The core tells observability, in the same
  decision, the number I was given, and observability records I as the
  incident's lead then, durably (`domain/connectors.md`, 6.3).
- **One lead per incident,** set once. An incident may gather several
  alerts; a second lead for an incident that has one is refused by
  observability, and reported. Until the core says which task the step
  made, the incident has no lead, and its page says so: nothing is
  inferred from names, order, watches or alerts' numbers.
- **I reads** logs and metrics, and decides on a restart.
- **The restart is in production,** beyond I's authority, and so beyond
  R's, the remediation I delegates: R's step proposes it
  (`domain/connectors.md`, 6.3). The proposal passes over I and W, a
  procedure, to P, who accepts it.
- **R holds the service,** restarts it, and waits on observability's
  health (3.3). The service recovers. R's result reaches I, which
  finishes with a report. P reads it in the client.

### 7.2 A request

```
Q, a developer
└─ C   a chat with Q                   agent, an environment chat
   └─ E  staging for payments, a week  infrastructure's provision; holds a slot of the staging pool
                                       beyond C's budget, so a proposal Q accepted, funded from Q's pool
```

- **Q asks C** for a staging environment for a week. C plans its size
  and cost, and asks for E.
- **E's cost** is beyond C's budget: C's `delegate` of E is refused,
  and C proposes it. Q accepts it, and it is funded from Q's pool.
- **E provisions:** the creation is keyed, so a restart in the middle
  finds the environment rather than creating it twice. E waits until the
  environment is ready, then tells C, which tells Q.
- **A week later** a timer E subscribed to wakes it, and E finishes. Its
  closing releases the environment, and the release tears it down.

### 7.3 The others

- **A night with no agent:** a recurring task scales staging down at
  night, if its load allows (3.3), and up in the morning. This is the
  first shape of `README.md`, section 9: procedures only.
- **Load rising after the verdict:** the night's scale-down is decided on
  a met verdict, and load rises before it lands. The scale-down is made,
  as its requirement is observed; the next night's verdict says wait.
- **A queue for the pool:** two developers ask for an environment when
  one slot is left. The second request waits for the slot, and is
  provisioned once the first environment is torn down.
- **A hand on the service:** while R's proposal waits for P, someone
  restarts the service by hand. R has made no restart, so the change is
  a fact it reads again, not drift (`domain/connectors.md`, section 11):
  it finds the service healthy, and finishes without restarting it.
- **An environment deleted by hand:** E's environment disappears. E is
  held and the client says why. Nothing is recreated without a decision.
- **A pool that shrinks:** staging's quota falls from five to two while
  four environments are held and a request waits. Nobody is evicted; the
  pool admits nobody until two are torn down; the waiting request goes
  first after that.
- **An uncertain restart:** a restart's answer is lost, and the engine
  restarts twice before its retry deadline. The fake production finds
  the restart by its operation's id, and nothing is restarted twice; on
  the systemd backend, R is held for P instead.
- **Policy narrowed:** an owner forbids restarts in production while one
  is proposed, one is committed and a triage is running. The proposal is
  refused when P accepts it, the committed one is made, and the owner
  cancels its task to stop it.
- **A month of alerts:** the watch runs across a period's reset; its
  allotment of triage tasks is renewed, and the triages still live keep
  the period that funded them.
- **Crashes at every commit:** jig's conformance world runs every story
  above, crashing the engine at each commit and failing commits. The
  referee checks jig's promises from outside: each creation was made
  once, each restart was made only after its proposal's acceptance was
  durable, every effect was within authority as it stood when decided,
  and the effect the fake production observed was the one the core
  decided.

## 8. Its client

`ops` holds one `jig-client` and presents it with jig's reference UI
(`README.md`, section 10). With jig's test application, it is how jig
shows that the API is enough for a whole application: everything `ops`
shows, it reads through the public client.

- **Its client crates,** written to be copied, as its root is:
  - `jig-ops-client-domain`, its client's domain: `jig_client::Client`,
    held as a component (`client/domain/client.md`), and its two
    families on it, incidents and environments, with their documents and
    their requests, which ride the same client and the same request table
    as bytes under their owners' numbers;
  - `jig-ops-client-protocol-wire`, its families' wire, which its client
    and its engine both read: observability's and infrastructure's
    subjects and documents (incidents, alerts, environments, panels), and
    infrastructure's environment requests and their outcomes; an
    application's wire as `protocol/client.md`, section 11 has it;
  - `jig-ops-client-view`, its presentation: the reference UI,
    `jig-client-view`, with its families' pages, cards and parts, on
    skein's UI kit (`client/domain/view.md`).
- **Its owners:** its two connectors' client sides, numbered as its
  connectors are, in `jig-ops-domain-connectors`, which its engine's
  root and its client share (`client/domain/root.md`, section 15).
- **The reference UI's pages,** for tasks, conversations, proposals and
  a person's inbox, with the inbox as home, and two families of its own:
  - incidents: their state, their timeline, the service's health, and
    their lead, the task observability recorded for it (7.1), shown by
    jig's own documents: its chip, its report and its proposals as the
    reference UI's cards, and its plan as the reference UI's plan
    section, in a section slot (`client/domain/pages.md`, section 9); an
    incident with no lead yet says so;
  - environments: their pool, their cost, their end, and a request to
    move the end, keyed like jig's.
- **Its arrangement:** the reference UI's frame, and its default
  arrangement of every page but a task's, which shows every section
  together, the address's own marked current: `ops`'s presentation
  builds the main from the task's header and its overview, activity,
  plan and history, and puts its budget and its connectors' panels in an
  aside (`client/domain/pages.md`, 12.3).
- **Its parts in the reference UI's slots:** infrastructure's panels for
  remediate and provision tasks, its words for a proposed restart, its
  account of drift ("deleted by hand"); both connectors' resources and
  kinds in words (`client/domain/pages.md`, section 9).
- **Its drawing:** an incident's error rate, on the incident's page, a
  drawing of observability's with its fallback in words. Its drawing
  code, in `jig-ops-client-view`, draws it as a scene (skein's `ui.md`,
  section 11; `client/domain/view.md`, section 17), tested by golden
  scenes.
- **A person in the worlds** is skein's fake person, reading the tree:
  P accepting a restart from an incident's page
  (`client/domain/pages.md`, section 10); Q asking for an environment in
  a chat, and accepting what it costs.
- **Its server** is `ops`'s, in its service, mounting jig's clients'
  component under `/jig` with jig's default HTTP mapping as it is (9.1).
  People sign in by the printed link; `ops` has no sign-in of its own.
- **Its terminal** is a presentation of the same client, on skein's text
  shell, with jig's settings: its client and its presentation, printed
  by `skein-ui-text` at most once a second (skein's `ui-text.md`;
  `client/domain/view.md`, section 15), over the native shell,
  `jig-client-shell-native`, and the default mapping's client end,
  holding its credential as a bearer token. Its client world plays it
  (`client/domain/testing.md`, section 4): P accepts the restart from
  the incident's page, typing the numbers the page printed. Its loop
  over a real terminal is `ops` standalone's (9.1), which gives it an
  engine to reach.
- **Its browser:** its browser shell, `jig-ops-client-shell-web`, copied
  from jig's test application's, `jig-client-test-shell-web`: its start
  and its loop over skein's browser parts and jig's,
  `jig-client-shell-web`, with the default mapping's client end through
  the client's adapter over fetch, and the credential in the cookie its
  server sets. Its paths are its own: the reference UI's places at the
  root, by the reference UI's codec, and incidents' and environments' by
  its own (`protocol/browser.md`, sections 3 and 9). It is `ops`
  standalone's too (9.1), whose server serves its page and bundle.

## 9. Its systems

- **One fake production, seen by both connectors,** so that restarting a
  service through infrastructure's API changes what observability's API
  reports:
  - services that log and emit metrics from a seeded model: baseline
    noise, injected incidents, recovery after a remediation;
  - pools with quotas, and environments that take time to be ready.
- **Its faults,** drawn from the seed:
  - API errors;
  - timeouts whose effect was made but whose answer was lost;
  - quotas exhausted;
  - slow provisioning;
  - drift: hands restarting and deleting.
- **A real backend.** On Linux, a machine's own services would do:
  - logs from journald;
  - restarts and scaling through systemd units;
  - environments as local containers.

  No cloud account is needed.

### 9.1 Their APIs, and `ops` standalone

- **`ops`'s own APIs,** one per system, HTTP/1.1 with JSON bodies,
  defined by `ops` and no real product's:
  - observability's: a service's logs and metrics, read by service and
    window; alerts as they fire;
  - infrastructure's: services restarted and scaled; pools read;
    environments made, read and removed; drift as it happens.
- **Events nobody asked for** (alerts, drift) come on one stream of
  server-sent events per system, which the connector's protocol keeps
  open and opens again after a backoff; its connector hears each as a
  system event.
- **A read that fails** (an error, a timeout) reaches its connector as a
  failed read: the system unreachable for that read. A run's read is
  answered as an error; a fact is left as it was and read again at its
  next pass.
- **A refusal for the rate** is an answer saying when the rate resets,
  which reaches its connector as its own event (`domain/connectors.md`,
  section 13).
- **The connectors' protocols,** `jig-ops-protocol-observability` and
  `jig-ops-protocol-infrastructure`, are components of `ops`'s engine
  protocol layer, over skein's HTTP client and server-sent events
  reader.
- **The fake production serves them:** `jig-ops-fake-production` gains a
  server face speaking both APIs, ordinary Rust as the kit is: on
  skein's simulated io in `ops`'s protocol world, and as a binary of its
  own, `jig-ops-production`, on loopback beside `jig-ops` when `ops` runs
  standalone. `ops`'s systems are outside its process, as any
  application's are.
- **Its service,** `jig-ops-service`, is the reference for an
  application with agents in the engine and no workers, the second of
  `domain/hosts.md`'s shapes: its `iterate` and protocol layer, with the
  store, the accounts', smith's LLM component for its
  `smith-inline-agent`'s runs, the host tools' translation
  (`jig-protocol-tools`, and its connectors' own), its server with the
  clients' component under jig's default HTTP mapping (below), and its
  two connectors' protocols; and `jig-ops`, its `main`, reading its
  configuration and running the loop (`protocol/service.md`, section 7).
- **Commented to be copied,** as its root is: a new application copies
  it, removes what its shape does not need (the LLM path and the tools,
  without agents in the engine; the channel, without workers), and adds
  its connectors' protocols.
- **Its server,** in `jig-ops-service`, on a loopback address: the
  clients' listener and its URL space are `ops`'s
  (`protocol/service.md`, section 2). It mounts jig's clients' component
  under `/jig`, with jig's default HTTP mapping and its example guards
  as they are, and serves its browser's page and bundle at its other
  paths, under the reference page policy, which jig's test application's
  server sends too (`protocol/browser.md`, 12.1). Its startup prints the
  printed link's URL, `/jig/v1/sign-in/link?secret=…` on that address,
  by which a person signs in, in a browser or at its terminal (section
  8). A browser's `GET` of it spends nothing: `ops`'s server answers it
  with its page, with `Referrer-Policy: no-referrer`, and its browser
  shell `POST`s the secret, as its terminal does (`protocol/client.md`,
  17.1; `protocol/browser.md`, section 11).

## 10. Its worlds

- **Each connector's world,** with the fake production below it and the
  root scripted above it, as for any connector (`domain/connectors.md`).
- **The root's world:** the whole domain on jig's conformance world.
  Agents are scripted below the hub, per charter, speaking smith's
  parent face as `smith-inline-agent` does: the root's world checks the
  root's routes, not smith. People are scripted; the store is skein-kv
  in memory.
- **Budgets:** its tests fit within the application's share of the
  suites' time (`domain/testing.md`, section 9).

## 11. Open questions

- **The watch:** whether it is a procedure that makes triage tasks, as
  section 5 has it, or a coordinator agent, woken rarely, that triages
  itself.
- **Triage before an LLM:** whether the connector's classification,
  with a wake policy batching what it lets through, is enough before a
  cheap model triages.
