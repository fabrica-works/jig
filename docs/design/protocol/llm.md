# LLM calls and accounts in the engine

Provisional, 2026-10-10, revised 2026-10-12. How a host that runs agents
in its own process makes their LLM calls, and how every engine that runs
agents keeps its LLM accounts: the completions `smith-inline-agent`'s
runs ask for, in the engine or on a worker whose root composes it,
carried by smith's LLM component over skein's LLM connections
(`../domain/hosts.md`, 5.2 and section 10); and the accounts, signed in,
refreshed and kept by skein's OAuth accounts driver, their access
tokens' table, and the credential grants that reach runs in the engine
and on workers (`../domain/engine.md`, section 12). An application
without agents has none of this. What is still open is listed in
section 10.

## 1. In one page

- **smith's component makes the calls in a host's process.** A process
  whose runs are `smith-inline-agent`'s (smith's `domain/host.md`, 9.1)
  owns one `smith-protocol-llm` component, with skein's LLM connection
  component beneath it, as smith's own hosts do (smith's
  `protocol/llm.md`; skein's `llm-connection.md`): the engine, for its
  own slots, and a worker that composes `smith-inline-agent`. jig writes
  no prompt, and none of smith's own tools' schemas, decoding or
  rendering; its engine tools' are `jig-protocol-tools`' (`channel.md`,
  6.3). Agent processes make their own calls (section 6).
- **The root routes completions,** as `smith-inline-agent` asks them,
  to the protocol layer, which gives smith's component each with its
  slot's result contract, and their terminals back to the agent (section
  3; `../domain/root.md`, section 5). They do not pass through the hub.
- **Accounts: the policy is the core's, the mechanism skein's.** The
  accounts child decides which accounts are usable and which runs hear of
  a new token (`../domain/engine.md`, section 12). skein's OAuth accounts
  driver, a component of the engine's protocol layer, signs accounts in,
  refreshes those it holds before they expire, keeps each new record in a
  private file before lending its access token, and lends tokens by
  generation (skein's `oauth.md`, section 6). `jig-protocol-accounts`
  translates between the two and holds the values lent.
- **Names in the domain, values beside it.** The domain names a credential
  grant by account and generation, with how long it is valid. Values move
  only between components, through the protocol layer's routing: from the
  accounts' table into smith's LLM component for the engine's own runs,
  and into the channel's records for runs on workers (`channel.md`,
  section 8). No root sees a value, and `jig-protocol-accounts` links no
  smith crate.
- **Endpoints are configuration,** one list from which the charter's
  endpoint names, smith's component's endpoints and skein's connections
  are all made, so the three never disagree (`configuration.md`).
- **The budget is smith's to hold** within each run: it reserves each
  completion's most before making it and winds down on its reserve, so a
  run never spends past its budget (smith's `domain/run.md`, 9 and 9.1).
  The core places no run smith would refuse for its budget
  (`../domain/authority.md`, section 7), and sees spend in the turns it
  commits.

## 2. Where it sits

```
engine's root
├── core: accounts                  want, rejected, release; granted, failed, availability   (names only)
├── jig-host                        the engine's own hub: its runs' turns, activity and text
├── smith-inline-agent              completions asked and cancelled, and their terminals; facts and content drained
└── …
engine's protocol layer
├── jig-protocol-accounts           the child's vocabulary to the driver's and back; the access tokens' table
│     └── skein-oauth-accounts      sign-in, refresh, records kept in private files, tokens lent by generation
├── smith-protocol-llm              prompts, tools' schemas, decoding, rendering; its grants' table; text as it comes
│     └── skein-llm-connection      connections, TLS, deadlines, the memory pool
├── the slots' contracts            each slot's result contract, from its start's charter, beside smith's component
└── jig-protocol-engine-workers     credential grants' values into assignments and grant records
```

- **One of each per engine,** shared by every run on its own slots, so
  connections and memory are pooled across runs, and each account
  refreshes once however many runs use it.
- **On a worker that composes `smith-inline-agent`,** the same, beside
  `smith-protocol-machine`, with no accounts: its grants come from jig's
  channel (`service.md`, section 6).
- **The protocol layer routes,** as any consumer of smith's components
  does (smith's `protocol/README.md`, section 6): io's tokens for the
  components' entities, their earliest deadline, their timers fired where
  its own are, and values from the accounts' table, or the worker's, to
  the components that fill them in.

## 3. Completions for `smith-inline-agent`'s runs

- **A completion asked:** `smith-inline-agent` hands up a run's lower
  request, `Complete`, scoped by the agent's token for the run, as
  smith's domain asks it: the completion's name (the turn it will be
  told as, and its retry), the grant it uses, the prompt's parts with
  its model and most output tokens, its deadlines and bounds (smith's
  `domain/host.md`, 9.1). The root routes it to the protocol layer,
  which hands it to smith's component with its slot's contract.
- **Each slot's contract, beside the component.** smith's component
  needs a completion's result contract (smith's outcome and delivery
  specifications) to write `finish`'s schema and decode its calls
  (smith's `protocol/llm.md`, sections 3 and 4), and smith's request
  carries none. So the protocol layer keeps each slot's, taken from its
  start's charter as its routing decodes it (`service.md`, section 2),
  as smith's local host does, until the run is gone. It is bounded by
  the slots and a charter's largest contract.
- **One terminal per completion:** completed, with its blocks, stop and
  usage; failed, with its class, evidence and detail; or cancelled. The
  protocol layer gives it to the root, which routes it to the agent's
  run. Retrying is smith's session's (smith's `domain/session.md`,
  section 4).
- **A cancel,** `Cancel`, follows a run's cancel or drop: the agent asks
  for it, and the completion still ends with its one terminal.
- **Only these reach the engine.** The engine's runs have no workspace, so
  `smith-inline-agent`'s machine requests (files, searches, commands,
  checks, guides) never come there; on a worker that composes it, they go
  to `smith-protocol-machine` (smith's `domain/host.md`, 9.1).
- **Activity and text, from the agent:** each pass the host's root
  drains `smith-inline-agent`'s facts and content (smith's
  `domain/host.md`, 9.1) and hands the hub a run's activity, a tool begun
  with its call's name and input among it (`../domain/hosts.md`, 6.5).
  The protocol layer asks smith's component for each completion's text
  as it arrives (smith's `protocol/events.md`, 3.5) and hands it up with
  its completion's name and offset: the run's text by turn, retry and
  offset (`channel.md`, 6.4), best effort, never holding the run.
- **Unauthorized and exhausted** come back to the run as smith's failures.
  The run's notices go up through the agent and the hub to the accounts
  child, as a rejected generation or an exhausted account, fenced by
  attempt, as a worker's run reports them on the channel.

## 4. Endpoints and models

- **An endpoint** is configuration: its name, its dialect, its address and
  transport (TLS with its server name and trusted roots, or plaintext to
  loopback), the dialect's path and headers, its options, and the account
  its grants are named by (smith's `protocol/agent.md`, section 4).
- **One list, three views.** The configuration's list makes the charter's
  endpoint names (`jig-charter`), smith's component's endpoints and
  skein's connection endpoints, by index, in the application's startup. A
  run's charter names endpoints the engine knows; a worker's agents
  resolve the same names against their own configuration's list, against
  which the worker's routing decodes charters (`configuration.md`,
  section 4).
- **Models and their prices** are configuration too, and the charter
  carries them (smith's `protocol/charter.md`, section 2;
  `../domain/hosts.md`, section 8): each model's endpoint; its window, the
  most input tokens one prompt may hold, past which smith compacts the
  conversation into a new window; its output, the most output tokens one
  completion may have, thinking included, reserved before each
  completion; and its prices in the deployment's unit, each per a number
  of tokens: input, cached input, output, and cache writes when priced
  apart, at the input's rate when not. A charter's window and output are
  at most what the agent that runs it declares, which its host's hello
  states and the fleet checks (`../domain/engine.md`, section 8).

## 5. Accounts

### 5.1 skein's driver

- **One driver per engine,** `skein-oauth-accounts`, made at startup with
  each configured account (skein's `oauth.md`, 6.2): its provider's
  registration (its client, and the client's secret if it is
  confidential), its token endpoint's address and transport, its source,
  a sign-in or a handed-in record (5.4), and skein's private files as its
  keeper, one file per account in a private directory the configuration
  names (skein's `oauth.md`, 6.4).
- **What it does, and jig does not:** one exchange per account at a time;
  a held account refreshed its refresh lead before expiry, each new
  generation granted again; every new record kept before its token is
  lent, so a rotated refresh token is never lost to a crash; a rejected
  token refreshed once, a second rejection of the same generation failed;
  failures classed (skein's `oauth.md`, sections 4 and 6.3). jig writes no
  OAuth exchange, and keeps no account's record.
- **The refresh token never leaves the driver.** Only access tokens come
  out of it, each with its generation and how long it is valid.

### 5.2 The translation

`jig-protocol-accounts` is small total functions between the accounts
child's vocabulary and the driver's, both ways, exhaustive on both:

| The accounts child asks | The driver hears |
|---|---|
| want an account | grant: hold it, and lend its access token |
| a generation rejected by its provider | rejected |
| release an account | release |
| close | close |

| The driver says | The accounts child hears |
|---|---|
| granted: the access token, its generation, how long it is valid | granted: the generation and how long it is valid; the token goes into the table (5.3) |
| failed: retry later (with how long), malformed, limit, not kept | failed, later, with how long (the driver's delay, or the child's backoff) |
| failed: sign in again, expired | failed, sign in again |
| expiring, for a handed-in account | expiring: the account is usable until its token lapses, and the deployment's document says a person must hand in a newer one |
| failed: client rejected | failed, misconfigured |
| failed: unloaded | failed, unloaded, with where and why |
| failed: cancelled; a grant refused as closed | failed, cancelled |
| a grant refused as full | failed, later, after the child's backoff |
| closed | closed |

- **Unreachable in the engine:** a sign-in's own failures (an invalid
  redirect), since the engine runs no sign-in; a grant's other refusals,
  for an account the driver was not made with. The translation names
  them, and asserts they never come.

### 5.3 The table

- **A grant's value** is the access token and, where the account's
  provider names one (a ChatGPT account does), the account's id, from
  its configuration or its handed-in file; the table holds both.
- **The access tokens lent,** by account, at most the two newest
  generations of each, with their expiry on the engine's clock, held by
  `jig-protocol-accounts`.
- **Sized at startup** from the configured accounts; it never grows.
- **An entry goes** when its token expires, or a newer generation pushes
  it out.

### 5.4 An account's first record

- **The driver loads each account's file** at start, and loads again
  before answering a grant whenever it holds no record, so a record put in
  place while the engine runs is heard without a restart (skein's
  `oauth.md`, 6.4). An account with no file is failed as expired until one
  appears.
- **The first record comes from a sign-in** by any process made with the
  same account's configuration: the driver's sign-in, with its loopback
  redirect, writes the account's file, which the engine then loads. An
  application may ship that as a command beside its engine. A sign-in at
  a terminal serves a public client's registration only: a confidential
  one is refused at its start, naming the account, until a provider
  needs one. Signing an
  account in through the client is open (section 10).
- **A file that cannot be read** fails the account as unloaded, saying
  where and why; the account stays unusable until its file is replaced.
- **A revoked account** fails as sign in again; the driver drops its dead
  record, so a new sign-in's file is loaded at the next grant the child
  asks for (skein's `oauth.md`, 6.4).
- **A handed-in account** has no sign-in and no refresh: its record is an
  access token and the wall time it lapses, in a private file the
  configuration names, which another program's login may fill (a person,
  or a test borrowing a login read-only). `jig-protocol-accounts` reads
  it at start (skein's `shell.md`, 6.1) and hands it in (skein's
  `oauth.md`, 6.3); the driver lends it until it lapses, never writing to
  its source. Past that, or once its provider rejects it, the account
  fails as sign in again, and stays unusable until a restart reads a
  newer file. It serves tests, live checks and a provider's static keys
  alike.

## 6. Credential grants

- **To runs in the engine's process:** when the child lends a generation,
  the protocol layer hands its value from the accounts' table to smith's
  LLM component's table, by account and generation, with when it lapses
  (smith's `protocol/llm.md`, section 7). A completion names its grant;
  one whose value is missing or lapsed fails as unauthorized and unsent,
  and smith's session asks again once a newer grant has come.
- **To runs on workers:** `jig-protocol-engine-workers` fills the value in
  from the accounts' table as it encodes an assignment or a grant record,
  and the worker's protocol layer hands it to smith's host half for its
  agent processes, or to its own LLM component's table when it composes
  `smith-inline-agent` (`channel.md`, section 8).
- **Pushed to every live attempt** whose account it is, when a new
  generation is lent, and sent again to each attempt a worker keeps after
  its hello, or that the fleet adopts (`../domain/engine.md`, sections 8
  and 12).
- **Rejected and exhausted,** from either kind of run, reach the child
  with the account and generation, fenced by attempt.

## 7. Limits and the worst case

- **smith's component's,** from `smith-inline-agent`'s limits: the
  request's bytes, tools' schemas, a completion's bytes and blocks, a
  decoded call's bytes, a failure's detail, a contract's bytes (smith's
  `protocol/llm.md`, section 8).
- **skein's connection component's:** conversations, the agent's slots;
  its memory pool; connections per endpoint (skein's
  `llm-connection.md`, section 7).
- **The slots' contracts:** one per slot, each at most a charter's
  largest contract.
- **Text as it comes:** a piece's bytes, and the pieces a pass may hand
  up.
- **The driver's:** accounts, exchanges at once, listeners, the refresh
  lead (skein's `oauth.md`, 6.7); the table's, from the accounts and a
  token's bytes. The engine's only.
- **A host's worst case,** the engine's or a worker's that composes
  `smith-inline-agent`, includes all of them, beside the agent's slots'
  sum (`../domain/hosts.md`, 5.2).

## 8. Testing

Each within jig's suites' budgets (`../../development/workflow.md`).

- **The driver is skein's,** tested in skein's worlds; jig tests what it
  adds.
- **The translation's step tests:** every request and event each way;
  every failure class mapped; the table's generations, expiry and
  pushing out.
- **The engine's LLM path and accounts,** in the protocol world
  (`service.md`, section 8): the root, `smith-inline-agent`, smith's
  component with the slots' contracts beside it, skein's connection
  component and the driver, against skein's fake LLM peer and fake issuer
  (skein's `fake-llm.md`; skein's `oauth.md`, section 5), over streams
  cut at random: completions, `finish` decoded under each slot's own
  contract, failures of each class, cancels mid-stream, a grant refreshed
  between two completions, a rotation, a rejected token, an account's
  file missing, then put in place, text reaching a watch by turn, retry
  and offset, and a tool begun shown with its call. Its referee: no token
  reaches a run before its record is kept; a name never means two
  values; every completion is given its own run's contract.
- **A worker that composes `smith-inline-agent`,** in the protocol world:
  grants from jig's channel in its component's table, and its runs'
  completions answered as the engine's are.
- **The domain worlds** keep the scripted LLM below `smith-inline-agent`
  and the accounts child's own world: none of this is in them.

## 9. What it owes and is owed

- **The domain** wants every configured account, and pushes each new
  generation to every live attempt (`../domain/engine.md`, section 12).
- **smith owes** nothing beyond what its `main` has for the first build,
  and later an abort and a closed terminal for its LLM component
  (`README.md`, section 11).
- **skein owes** nothing beyond what it has: its accounts driver, LLM
  client, connection component and fake peers serve as they are.
- **The store owes** nothing for accounts: their records are the driver's
  private files.

## 10. Open questions

- **Signing an account in through a client:** an owner starting an
  account's sign-in from the deployment's page, the client showing the
  driver's visit, and the provider's redirect reaching the engine's
  listener; in place of a sign-in at a terminal.
- **Fairness on a host's loop** (`../domain/hosts.md`, section 12):
  many `smith-inline-agent` runs' completions share one connection
  component and one loop; whether each run gets a share of each
  iteration.
- **A handed-in account read again while the engine runs,** so a newer
  file is heard without a restart, as a signed-in account's is.
