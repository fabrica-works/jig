# LLM calls and accounts in the engine

Provisional, 2026-10-10. How an engine that runs agents in its own process
makes their LLM calls, and how every engine that runs agents keeps its LLM
accounts: the completions its inline agent asks for, carried by smith's
LLM component over skein's LLM connections (`../domain/hosts.md`, 5.2 and
section 10); and the accounts, signed in, refreshed and kept by skein's
OAuth accounts driver, their access tokens' table, and the credential
grants that reach runs in the engine and on workers
(`../domain/engine.md`, section 12). An application without agents has
none of this. What is still open is listed in section 10.

## 1. In one page

- **smith's component makes the calls.** The engine's protocol layer owns
  one `smith-protocol-llm` component, with skein's LLM connection
  component beneath it, as smith's agent process does (smith's
  `protocol/llm.md`; skein's `llm-connection.md`). Every inline run's
  completions go through them. jig writes no prompt, and none of smith's
  own tools' schemas, decoding or rendering; its engine tools' are
  `jig-protocol-tools`' (`channel.md`, 6.3).
- **The root passes completions through.** The inline agent hands up the
  completions its runs' smith domains ask for, as they ask: each carries
  its run's result contract (smith's `protocol/llm.md`). The engine's root
  routes them out through the journal, as every output, to the protocol
  layer, and the protocol layer's answers back to the inline agent
  (`../domain/root.md`, section 5). They do not pass through the hub.
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
  accounts' table into smith's LLM component for inline runs, and into the
  channel's records for runs on workers (`channel.md`, section 8). No
  root sees a value, and `jig-protocol-accounts` links no smith crate.
- **Endpoints are configuration,** one list from which the charter's
  endpoint names, smith's component's endpoints and skein's connections
  are all made, so the three never disagree (`configuration.md`).
- **The budget is smith's to hold** within each run: it reserves each
  completion's most before making it (`../domain/hosts.md`, section 8).
  The engine sees spend in the turns it commits.

## 2. Where it sits

```
engine's root
├── core: accounts                  want, rejected, release; granted, failed, availability   (names only)
├── jig-inline-agent                completions asked, each with its run's contract, and answered
└── …
engine's protocol layer
├── jig-protocol-accounts           the child's vocabulary to the driver's and back; the access tokens' table
│     └── skein-oauth-accounts      sign-in, refresh, records kept in private files, tokens lent by generation
├── smith-protocol-llm              prompts, tools' schemas, decoding, rendering; its grants' table
│     └── skein-llm-connection      connections, TLS, deadlines, the memory pool
└── jig-protocol-engine-workers     credential grants' values into assignments and grant records
```

- **One of each per engine,** shared by every inline run, so connections
  and memory are pooled across runs, and each account refreshes once
  however many runs use it.
- **The protocol layer routes,** as any consumer of smith's components
  does (smith's `protocol/README.md`, section 6): io's tokens for the
  components' entities, their earliest deadline, their timers fired where
  its own are, and values from the accounts' table to the components that
  fill them in.

## 3. Completions for inline runs

- **A completion asked:** the inline agent says which run (its token),
  and passes smith's domain's request as it is: the prompt's parts, the
  model, the most output tokens, the endpoint, the grant it uses, its
  deadlines and bounds, and the run's result contract (smith's outcome and
  delivery specifications, from the run's charter), which smith's
  component needs to write `finish`'s schema and decode its calls
  (smith's `protocol/llm.md`, sections 3 and 4). The root routes it to the
  protocol layer, which hands it to smith's component. Nothing per run is
  kept beside smith's domain.
- **One terminal per completion:** completed, with its blocks, stop and
  usage; failed, with its class and evidence; or cancelled. The protocol
  layer gives it to the root, which routes it to the inline agent's run.
  Retrying is smith's session's (smith's `domain/session.md`, section 4).
- **A cancel** follows a run's cancel: the inline agent asks for it, and
  the completion ends cancelled.
- **Text as it comes:** smith's component hands its domain the text as it
  arrives; smith's domain reports it with the turn it belongs to, its
  completion's retry and its offset; the inline agent passes it up, and the
  root routes it to the core's views (`../domain/engine.md`, section 11).
- **Unauthorized and exhausted** come back to the run as smith's failures,
  and up to the accounts child as a rejected generation or an exhausted
  account, as a worker's run reports them on the channel.

## 4. Endpoints and models

- **An endpoint** is configuration: its name, its dialect, its address and
  transport (TLS with its server name and trusted roots, or plaintext to
  loopback), the dialect's path and headers, its options, and the account
  its grants are named by (smith's `protocol/agent.md`, section 4).
- **One list, three views.** The configuration's list makes the charter's
  endpoint names (`jig-charter`), smith's component's endpoints and
  skein's connection endpoints, by index, in the application's startup. A
  run's charter names endpoints the engine knows; a worker's agents
  resolve the same names against their own configuration's list.
- **Models and their prices** are configuration too: each model, the
  endpoint that serves it, its most output tokens, and its prices in the
  deployment's unit, which the charter carries (`../domain/hosts.md`,
  section 8).

## 5. Accounts

### 5.1 skein's driver

- **One driver per engine,** `skein-oauth-accounts`, made at startup with
  each configured account (skein's `oauth.md`, 6.2): its provider's
  registration (its client, and the client's secret if it is
  confidential), its token endpoint's address and transport, a sign-in
  source, and skein's private files as its keeper, one file per account in
  a private directory the configuration names (skein's `oauth.md`, 6.4).
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
| failed: client rejected | failed, misconfigured |
| failed: unloaded | failed, unloaded, with where and why |
| failed: cancelled; a grant refused as closed | failed, cancelled |
| a grant refused as full | failed, later, after the child's backoff |
| closed | closed |

- **Unreachable in the engine:** a sign-in's own failures (an invalid
  redirect), since the engine runs no sign-in; a grant's other refusals,
  for an account the driver was not made with; and `expiring`, which only
  an access-only record raises, while every account's source is a
  sign-in (section 10 keeps API keys open). The translation names them,
  and asserts they never come.

### 5.3 The table

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

## 6. Credential grants

- **To inline runs:** when the child lends a generation, the protocol
  layer hands its value from the accounts' table to smith's LLM
  component's table, by account and generation, with when it lapses
  (smith's `protocol/llm.md`, section 7). A completion names its grant;
  one whose value is missing or lapsed fails as unauthorized and unsent,
  and smith's session asks again once a newer grant has come.
- **To runs on workers:** `jig-protocol-engine-workers` fills the value in
  from the accounts' table as it encodes an assignment or a grant record
  (`channel.md`, section 8).
- **Pushed to every live attempt** whose account it is, when a new
  generation is lent, and sent again to each attempt a worker keeps after
  its hello, or that the fleet adopts (`../domain/engine.md`, sections 8
  and 12).
- **Rejected and exhausted,** from either kind of run, reach the child
  with the account and generation, fenced by attempt.

## 7. Limits and the worst case

- **smith's component's,** from the inline agent's limits: the request's
  bytes, tools' schemas, a completion's bytes and blocks, a decoded
  call's bytes, a failure's detail (smith's `protocol/llm.md`, section 8).
- **skein's connection component's:** conversations, the inline agent's
  slots; its memory pool; connections per endpoint (skein's
  `llm-connection.md`, section 7).
- **The driver's:** accounts, exchanges at once, listeners, the refresh
  lead (skein's `oauth.md`, 6.7); the table's, from the accounts and a
  token's bytes.
- **The engine's worst case** includes all of them, beside the inline
  agent's slots' sum (`../domain/hosts.md`, 5.2).

## 8. Testing

Each within jig's suites' budgets (`../../development/workflow.md`).

- **The driver is skein's,** tested in skein's worlds; jig tests what it
  adds.
- **The translation's step tests:** every request and event each way;
  every failure class mapped; the table's generations, expiry and
  pushing out.
- **The engine's LLM path and accounts,** in `ops`'s protocol world
  (`service.md`, section 8): the root, the inline agent, smith's
  component, skein's connection component and the driver, against skein's
  fake LLM peer and fake issuer (skein's `fake-llm.md`; skein's
  `oauth.md`, section 5), over streams cut at random: completions,
  failures of each class, cancels mid-stream, a grant refreshed between
  two completions, a rotation, a rejected token, an account's file
  missing, then put in place, text reaching a watch by turn, retry and
  offset. Its referee: no token reaches a run before its record is kept; a
  name never means two values.
- **The domain worlds** keep the scripted LLM below the inline agent and
  the accounts child's own world: none of this is in them.

## 9. What it owes and is owed

- **The domain** wants every configured account, and pushes each new
  generation to every live attempt (`../domain/engine.md`, section 12).
- **smith owes** its completion requests with their run's result contract,
  and its text in flight to its domain with its turn, retry and offset
  (`channel.md`, section 13).
- **skein owes** nothing beyond what it has: its accounts driver, LLM
  client, connection component and fake peers serve as they are.
- **The store owes** nothing for accounts: their records are the driver's
  private files.

## 10. Open questions

- **Signing an account in through a client:** an owner starting an
  account's sign-in from the deployment's page, the client showing the
  driver's visit, and the provider's redirect reaching the engine's
  listener; in place of a sign-in at a terminal.
- **Fairness on the engine's loop** (`../domain/hosts.md`, section 12):
  many inline runs' completions share one connection component and one
  loop; whether each run gets a share of each iteration.
- **API keys:** accounts whose credential is a static key rather than an
  OAuth grant, never refreshed: as the driver's handed-in records with no
  expiry, or a source of skein's of their own.
