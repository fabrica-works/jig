# LLM calls and accounts in the engine

Provisional, 2026-10-10. How an engine that runs agents in its own process
makes their LLM calls, and how every engine that runs agents keeps its LLM
accounts: the completions its inline agent asks for, carried by smith's
LLM component over skein's LLM connections (`../domain/hosts.md`, 5.2 and
section 10); and the accounts' refreshes, the records kept of them, the
table of their access tokens, and the credential grants that reach runs
in the engine and on workers (`../domain/engine.md`, section 12). An
application without agents has none of this. What is still open is
listed in section 10.

## 1. In one page

- **smith's component makes the calls.** The engine's protocol layer owns
  one `smith-protocol-llm` component, with skein's LLM connection
  component beneath it, as smith's agent process does (smith's
  `protocol/llm.md`; skein's `llm-connection.md`). Every inline run's
  completions go through them. jig writes no prompt, and none of smith's
  own tools' schemas, decoding or rendering; its engine tools' are
  `jig-engine-tools`' (`channel.md`, 6.3).
- **The root passes completions through.** The inline agent hands its
  runs' completions up, each with its run's result contract; the engine's
  root routes them out through the journal, as every output, to the
  protocol layer, and the protocol layer's answers back to the inline
  agent (`../domain/root.md`, section 5). They do not pass through the
  hub.
- **Accounts are the core's; their exchanges the protocol layer's.** The
  accounts child decides when an account refreshes, what a failure means
  and which runs hear of a new token (`../domain/engine.md`, section 12).
  `jig-engine-accounts`, a component of the engine's protocol layer, runs
  each refresh with skein's OAuth client, keeps each refresh's whole
  record durably before the access token it bought is lent, and holds the
  access tokens in a table, by account and generation.
- **Names in the domain, values beside it.** The domain names a credential
  grant by account and generation, with how long it is valid. Values move
  only between components, through the protocol layer's routing: from the
  accounts' table into smith's LLM component for inline runs, and into the
  channel's records for runs on workers (`channel.md`, section 8). No
  root sees a value, and `jig-engine-accounts` links no smith crate.
- **Endpoints are configuration,** one list from which the charter's
  endpoint names, smith's component's endpoints and skein's connections
  are all made, so the three never disagree (`configuration.md`).
- **The budget is smith's to hold** within each run: it reserves each
  completion's most before making it (`../domain/hosts.md`, section 8).
  The engine sees spend in the turns it commits.

## 2. Where it sits

```
engine's root
├── core: accounts                  refresh, keep, cancel, granted, availability   (names only)
├── jig-inline-agent                completions asked, each with its run's contract, and answered
└── …
engine's protocol layer
├── jig-engine-accounts             OAuth refreshes; records kept; the access tokens' table
│     └── skein's OAuth client      one exchange at a time per account, over its own TLS connection
├── smith-protocol-llm              prompts, tools' schemas, decoding, rendering; its grants' table
│     └── skein-llm-connection      connections, TLS, deadlines, the memory pool
├── jig-engine-channel              credential grants' values into assignments and grant records
└── the store                       the secrets' records (README.md, section 9)
```

- **One of each per engine,** shared by every inline run, so connections
  and memory are pooled across runs, and each account refreshes once
  however many runs use it.
- **The protocol layer routes,** as any consumer of smith's components
  does (smith's `protocol/README.md`, section 6): io's tokens for the
  components' entities, their earliest deadline, their timers fired where
  its own are, and values from the accounts' component to the components
  that fill them in.

## 3. Completions for inline runs

- **A completion asked:** the inline agent says which run (its token),
  smith's request (the prompt's parts as smith's domain gives them, the
  model, the most output tokens, the endpoint, the grant it uses), its
  deadlines, and the run's result contract (smith's outcome and delivery
  specifications, from the run's charter), which smith's component needs
  to write `finish`'s schema and decode its calls (smith's
  `protocol/llm.md`, sections 3 and 4). The root routes it to the
  protocol layer, which hands it to smith's component.
- **One terminal per completion:** completed, with its blocks, stop and
  usage; failed, with its class and evidence; or cancelled. The protocol
  layer gives it to the root, which routes it to the inline agent's run.
  Retrying is smith's session's (smith's `domain/session.md`, section 4).
- **A cancel** follows a run's cancel: the inline agent asks for it, and
  the completion ends cancelled.
- **Text as it comes:** smith's component hands its domain the text as it
  arrives, the inline agent reports it with the turn it belongs to and
  its offset, and the root routes it to the core's views
  (`../domain/engine.md`, section 11). `channel.md`, section 13 says what
  smith owes for it.
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

### 5.1 The component

`jig-engine-accounts` serves the accounts child's requests
(`../domain/engine.md`, section 12):

| From the domain | What the component does |
|---|---|
| refresh an account, naming the generation it buys | one exchange with the account's token endpoint, with the refresh token it keeps; then the new record kept (5.2); then refreshed, with how long the new access token is valid; or failed, with its class |
| keep | keeps again a record whose keeping failed, never spending its refresh token's predecessor again |
| cancel | ends the exchange in flight; its terminal follows |
| granted | the domain lends a generation to runs: the value goes where the runs are (section 6) |
| availability, refused, closed | the account's state for operators and the deployment's document; nothing to send |

- **skein's OAuth client** makes the exchange (skein's `oauth.md`,
  sections 1 to 5). The component owns each exchange's connection, as
  that client's caller does: it connects to the account's token endpoint,
  runs TLS or plaintext to loopback, and closes the connection when the
  exchange ends. It does not use skein's accounts driver (skein's
  `oauth.md`, section 6), whose refreshing on its own schedule would be a
  second policy beside the accounts child's.
- **An account** is configuration: its provider's registration (its
  client, and the client's secret if it is confidential), its token
  endpoint's address and transport, and the file that may seed it (5.4).
- **One exchange per account at a time,** as the child asks, and the
  exchanges at once bounded across accounts.
- **Failures are typed** as the child takes them: unavailable, timed out,
  rate limited with how long, refused (the provider will not refresh: a
  person must sign the account in again), unsaved (bought, but not kept),
  cancelled.

### 5.2 Kept before lent

- **Every refresh's record is durable** before the access token it bought
  is lent: its generation, the access token and when it expires, and the
  refresh token, whether the provider rotated it or not. The component
  keeps it in the store's secrets' records (`README.md`, section 9), then
  answers the refresh, as skein's OAuth client's caller must (skein's
  `oauth.md`, section 1).
- **Why the whole record:** a provider may rotate the refresh token on
  every use, so a crash before keeping it would lose the account; and a
  generation lent and then forgotten would be bought again under the same
  number with another value. Kept whole, a name never means two values
  across restarts.
- **Keeping failed** is the child's *unsaved*: the access token is held,
  not lent; the child asks to keep again; the old refresh token is never
  spent again.

### 5.3 The table

- **The component holds access tokens,** by account, at most the two
  newest generations of each, with their expiry on the engine's clock,
  and each account's refresh token, which never leaves it.
- **Sized at startup** from the configured accounts; it never grows.
- **An entry goes** when its token expires, or a newer generation pushes
  it out.

### 5.4 Starting an account

- **Before the root starts,** the component reads each account's kept
  record from the secrets' store into its table (`service.md`, section 3).
- **The core hears its accounts with its own records,** in the restart's
  first step (`../domain/engine.md`, section 6): the protocol layer hands
  it each configured account with its kept generation and how long its
  access token is still valid, or with only a refresh token, which the
  core refreshes at once.
- **An account is seeded** from a file its configuration names, holding a
  refresh token an operator obtained and the generation it starts from.
  The file is read at every start: it gives the account its first record
  when the store holds none, and replaces the kept record only when its
  generation is above the kept one, so a revoked account is seeded anew
  by writing a newer file, and an old file never undoes a rotation (section 10 keeps signing an account
  in through a client open).

## 6. Credential grants

- **To inline runs:** when the child lends a generation, the protocol
  layer hands its value from the accounts' table to smith's LLM
  component's table, by account and generation, with when it lapses
  (smith's `protocol/llm.md`, section 7). A completion names its grant;
  one whose value is missing or lapsed fails as unauthorized and unsent,
  and smith's session asks again once a newer grant has come.
- **To runs on workers:** `jig-engine-channel` fills the value in from
  the accounts' table as it encodes an assignment or a grant record
  (`channel.md`, section 8).
- **Pushed to every live attempt** whose account it is, when a new
  generation is lent, and sent again to each attempt a worker keeps after
  its hello (`channel.md`, 5.3 and section 13 say what the domain owes for
  it).
- **Rejected and exhausted,** from either kind of run, reach the child
  with the account and generation, fenced by attempt; the child may
  refresh early, or mark the account spent until a time.

## 7. Limits and the worst case

- **smith's component's,** from the inline agent's limits: the request's
  bytes, tools' schemas, a completion's bytes and blocks, a decoded
  call's bytes, a failure's detail (smith's `protocol/llm.md`, section 8).
- **skein's connection component's:** conversations, the inline agent's
  slots; its memory pool; connections per endpoint (skein's
  `llm-connection.md`, section 7).
- **The accounts' component's:** accounts, exchanges at once, a record's
  and a token's bytes, and per exchange skein's OAuth client's, HTTP's,
  TLS's and io's buffers for one connection.
- **The engine's worst case** includes all three, beside the inline
  agent's slots' sum (`../domain/hosts.md`, 5.2).

## 8. Testing

Each within jig's suites' budgets (`../../development/workflow.md`).

- **The accounts' component's machine world:** scripted OAuth exchanges
  (skein's fake issuer) and a scripted store of secrets: refreshes that
  rotate or not, fail each way, and race a cancel; keeping that fails,
  then succeeds; a restart from the kept record, with the engine crashed
  between keeping and lending; a seed file older and newer than the kept
  record; tokens expiring in the table; values reaching a scripted LLM
  component and a scripted channel. Its referee: no access token is lent
  before its record is kept; no refresh token is spent twice; a name never
  means two values.
- **The engine's LLM path,** in `ops`'s protocol world (`service.md`,
  section 8): the root, the inline agent, smith's component and skein's
  connection component against skein's fake LLM peer (skein's
  `fake-llm.md`), over streams cut at random: completions, failures of
  each class, cancels mid-stream, a grant refreshed between two
  completions, text reaching a watch by turn and offset.
- **The domain worlds** keep the scripted LLM below the inline agent:
  none of this is in them.

## 9. What it owes and is owed

- **The domain owes** (`README.md`, section 12):
  - **the run's contract with each completion:** the inline agent hands
    up, with each completion its run asks for, the run's result contract
    in smith's terms, which it holds from the run's charter;
  - **grants pushed:** the core lends each new generation to every live
    attempt whose account it is, and the fleet sends each kept attempt its
    current grant after a hello (`channel.md`, section 13).
- **smith owes** its text in flight to its domain, and its text facts'
  turn and offset (`channel.md`, section 13).
- **skein owes** nothing beyond what it has: its OAuth client, LLM client,
  connection component and fake peers serve as they are.
- **The store owes** the secrets' records: each account's last kept
  record, kept durably on request and read at start (`README.md`,
  section 9).

## 10. Open questions

- **Signing an account in through a client:** an owner starting an
  account's OAuth sign-in from the deployment's page, the engine showing
  the provider's visit, and the code exchanged on the engine's redirect;
  in place of seeding it from a file.
- **Fairness on the engine's loop** (`../domain/hosts.md`, section 12):
  many inline runs' completions share one connection component and one
  loop; whether each run gets a share of each iteration.
- **API keys:** accounts whose credential is a static key rather than an
  OAuth grant, at generation zero, never refreshed; kept as a seed file's
  record is.
