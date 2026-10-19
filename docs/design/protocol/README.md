# jig's protocol layer

Provisional, 2026-10-10, revised 2026-10-19. The design of what lies
between jig's domains and bytes: the components an application's protocol
layers own to carry jig's vocabularies to the store, to workers, to LLM
providers and to clients; what an application writes around them; their
versions; the secrets they hold; and what they need from skein and smith.
jig as a whole is `../README.md`, whose section 3 says every process is
the application's and jig supplies the pieces that speak jig's
vocabularies. The mechanics are those of skein's `programming-model.md`,
section 4. What is still open is listed in section 14.

## 1. In one page

- **Translation, nothing else.** jig's protocol components turn bytes
  into jig's domain vocabularies and back. Every decision stays the
  domain's: the core's, the hub's, the client's. A component runs the
  timers and the exchanges the domain asks for, and its own deadlines,
  and says what happened (programming-model.md, section 4).
- **Components, owned by the application.** Each component is a child of
  a process's protocol layer, with its own limits, worst case, deadlines
  and most outputs per entry point, as a child domain is of a root. The
  application's protocol layer owns jig's components beside smith's,
  skein's and its own connectors', and routes between them, io and its
  root (`service.md`).
- **Five boundaries in the engine** (section 4): the store, workers,
  LLM providers and accounts, clients, and connectors' systems. The
  first four are jig's, each with components of jig's; connectors'
  protocols are the application's. At the clients' boundary jig's
  vocabulary is its messages, and the transport that carries them is
  the application's.
- **Clients, by jig's messages.** The clients' component,
  `jig-protocol-engine-clients`, takes jig's messages with an opaque
  credential and answers with jig's messages, and knows no HTTP, path,
  cookie or listener. The application's service owns the listener, its
  routes, its URL space and every HTTP and browser guard, and mounts
  the component under any prefix. jig ships a default HTTP mapping, both
  ends, with example guards, to use, wrap or leave (`client.md`,
  sections 4 and 17; `browser.md`, 12.2).
- **jig owns credentials, not sign-in flows.** The clients' component
  mints, verifies, expires and revokes credentials. Its built-ins are
  the printed link, on loopback, and services' credentials; any other
  way of signing in is the application's, on its own routes, and ends
  in a `Mint` its server sends (`client.md`, section 8).
- **Generic mechanisms are skein's,** as for smith: framed channels,
  codecs generated from schemas, the key-value store, LLM connections,
  OAuth, HTTP and server-sent events, TLS, contained processes. jig keeps
  what is about its own vocabularies.
- **Agents and their records are smith's.** A hub's agent capability is
  `smith-host-domain` or `smith-inline-agent`; jig ships none (section
  4). The hub, the core and jig's channel carry smith's payloads (the
  charter, the transcript, turns' bodies, calls' names, results) as
  smith's own versioned bytes, and never read them; the root that
  composes an agent capability hands each out to its protocol layer to
  be decoded or encoded (section 13).
- **The application's parts stay the application's.** Where jig's records
  carry an application's value (a workspace's items, what a run left, an
  application's documents), they carry it as bytes the application
  encodes, versions and decodes in its own protocol layer.
- **Nothing is parsed in a domain.** Host tools' calls and runs' results
  are decoded, and calls' answers rendered, in the engine's protocol
  layer, for runs on workers and in the engine alike (`channel.md`, 6.1
  and 6.3).
- **jig ships the store,** `jig-protocol-store`: skein-kv in the
  engine's process, jig's records and the application's in one commit,
  transcripts as files, secrets in a second store (`store.md`). The
  other components rely only on the boundary section 9 states.
- **Secrets never reach a domain.** A domain names a secret by number and
  generation; components hold values in tables and in the store's
  secrets' records, LLM accounts' records and keys among them, and
  they are filled in where they leave (section 8).
- **Configuration is read once, before the loop,** strict, composed per
  child as the root is (`configuration.md`).

## 2. Reading order

1. **README.md:** this page.
2. **service.md:** the engine's and the worker's services: the protocol
   layer as a parent of components, `iterate`, starting and ending, one
   process, the application's server, which owns the clients' listener
   and mounts the clients' component, and the reference binary,
   `jig-test`, with its service.
3. **channel.md:** jig's channel between the engine and its workers, and
   where host tools' calls are decoded.
4. **llm.md:** LLM calls in a host's own process, for
   `smith-inline-agent`'s runs, and LLM accounts: skein's accounts
   driver for sign-ins, keys from requests, program sources on their
   hosts, the translation, the table of values, and grants.
5. **configuration.md:** what an engine and a worker are told at start.
6. **client.md:** the client's protocol: jig's messages and their wire,
   the clients' component and its messages toward the application's
   transport and server, the default HTTP mapping's routes and stream,
   keyed requests, reads, watches, signing in and credentials, versions,
   the small store, shells.
7. **store.md:** the store: skein-kv in the engine, keys and records,
   commits and loads, transcripts, secrets, growth.
8. **browser.md:** a browser's client: the default HTTP mapping's
   browser end, `jig-client-shell-web`, over skein's browser parts
   (skein's `ui-web.md`), and the cookie that carries a browser's
   credential; the example guards and jig's attack stories, for the
   application's server; jig's test application's browser shell, server
   and page policy, as the example of an application's; and where jig's
   promise stops.

## 3. Names

| Name | What it is |
|---|---|
| protocol layer | a process's layer between its root and io: its components, and the application's routing between them (programming-model.md, section 4) |
| component | a crate of a protocol layer that the application's protocol layer owns as a child and routes to |
| component, a shell's | the client's name (`../client/domain/README.md`, section 4): a shell's native implementation of a node (skein's `ui.md`, section 12), never a protocol layer's component |
| codec crate | a crate of records generated from a schema, used by components at both ends (skein's `codec.md`) |
| record | a value of a protocol's vocabulary, encoded to its schema |
| service | a process's `iterate` with its root and protocol layer; its `main` is its shell |
| worker's link | a worker's one channel with its engine, dialled and redialled (`channel.md`); the hub's link to the core, over a channel |
| worker's credential | what a worker's opening carries: its name, its secret, the application's payload version (`channel.md`, 4.1) |
| payload version | the number an application gives jig's components for its own encodings carried inside jig's records |
| credential grant | an LLM account's grant, or an application's credential, named by account and source (or the application's number for it) and generation, with how long it is valid; its value is beside it (section 8). An LLM account's is a credential with its scheme, `bearer` or `key`, or a program source's name, which carries no value (`llm.md`, section 6). Not the authority's grant (`../domain/README.md`, section 4) |
| table | a component's holding of secrets' values, by account, source and generation; jig's accounts' table, or the application's of its credentials |
| secrets' records | what the store keeps of secrets that change while the engine runs: minted credentials' and services' digests, LLM accounts' records and keys, and an application's own |
| jig's messages | what a client and the clients' component exchange, whatever carries them: a keyed request, a read, opening and closing a watch, signing in and out, and the stream's events; each names its owner and its version (`client.md`, 4.1) |
| the clients' component | `jig-protocol-engine-clients`, the engine's side of jig's messages: it takes them with an opaque credential from the application's transport, and a `Mint` from its server; it mints, verifies, expires and revokes credentials; it knows no HTTP (`client.md`, 4.2) |
| the transport | the application's carrier of jig's messages between `jig-client` and the clients' component: HTTP, a local socket, or an in-process channel |
| the default HTTP mapping | jig's optional transport over HTTP, both ends, `jig-client-protocol-http` and `jig-protocol-engine-clients-http`, with example guards (`client.md`, section 17; `browser.md`, 12.2) |
| a credential | what signing in leaves: minted for an identity, its digest in the secrets' records, opaque bytes to the transport (`client.md`, section 8). Neither a credential grant nor an application's credential |
| profile | a named set of every limit jig takes, consistent with one another (`configuration.md`, section 6) |
| agent capability | the hub's agent: `smith-host-domain`, agent processes, one per run; or `smith-inline-agent`, smith's domains in the host's process (`../domain/hosts.md`, section 5) |
| smith's parent face | the vocabulary both agent capabilities speak toward their parent (`smith_host_domain::parent`; smith's `domain/host.md`, 9.4) |
| smith's values | smith's charter, transcript, turns' bodies and results as typed values, which smith's parent face carries and its agent capabilities take and give; the hub carries them as smith's bytes |
| an application's credential | a credential of the application's that a run's workspace needs (a token to fetch and push), named by the application's number for it and a generation; its value in the application's own table, from a source of its own (`channel.md`, section 8) |
| the reference binary | `jig-test`, jig's test application's `main`, which an application copies for its own (`service.md`, section 7) |

The domain's names (`../domain/README.md`, section 4) and the client's
(`../client/domain/README.md`, section 4) keep their meaning here; the
client's link is a client's reach of the engine, not a worker's link.

## 4. The processes

```
engine     the root (../domain/root.md) and its protocol layer:
           the store, workers' channels, LLM calls and accounts, host tools,
           clients (the application's server and transport, jig's clients' component), connectors' systems
worker     if any: the worker's root (jig-domain-host, an agent capability, a workspace) and its protocol layer:
           the link to the engine, smith's host half and its agent processes, the workspace's io
client     the application's presentation over jig-client (../client/domain/), and its transport:
           jig's messages to the engine and back, the small store's host, the shell
```

By `../domain/hosts.md`, section 3's shapes:

| Shape | The engine's protocol layer has | Workers |
|---|---|---|
| no agents | the store, clients, connectors | none |
| agents in the engine | and smith's LLM component for `smith-inline-agent`'s runs, accounts, host tools | none |
| agents on workers | and workers' channels, accounts, host tools | the link, smith's host half and agent processes, workspaces |

An application that mixes the last two has both.

- **The agent capability is smith's:** in the engine,
  `smith-inline-agent` with no workspace; on a worker, agent processes,
  or `smith-inline-agent` as its root's choice, for one person's machine,
  with no containment (smith's `domain/host.md`, 9.3).
- **One process,** for one person's machine, development and tests: the
  engine and its workers on one loop, each with its own root and
  protocol layer, the workers' links joined to the engine's listener by
  skein's in-process addresses (section 10), over the same components,
  codecs and channels, with no TLS, and the workers' agents child
  processes, as between hosts (`service.md`, section 10). A client
  reaches the engine through the application's server, over TCP on
  loopback.

## 5. Components

| Crate | Kind | In | What it does |
|---|---|---|---|
| `jig-protocol-worker-wire` | codecs | engine, worker | the channel's records and a worker's credential (`channel.md`) |
| `jig-protocol-engine-workers` | component | engine | workers' channels: opening, authentication, liveness, translation to and from the hub's vocabulary (`channel.md`) |
| `jig-protocol-worker-link` | component | worker | the link: dialling, liveness, the grace's clock, relays across channels, translation to and from the hub's vocabulary, smith's payloads carried as bytes (`channel.md`) |
| `jig-protocol-charter` | translation | engine | the core's charter put into smith's, as smith's bytes, and a run's result read back (`../domain/hosts.md`, section 8) |
| `jig-protocol-tools` | translation | engine | the core's host tools: their schemas and descriptions, their calls decoded into the core's typed requests, their answers rendered as text (`channel.md`, 6.3) |
| `jig-protocol-accounts` | translation, table | engine | the accounts child's vocabulary to skein's accounts driver's and back, and the values lent, tokens and keys with their scheme (`llm.md`, section 5) |
| `jig-protocol-config` | parsing | engine, worker | jig's part of a configuration, parsed and checked (`configuration.md`) |
| `jig-protocol-store` | component, codecs | engine | jig's records and the application's on skein-kv, transcripts' files, the secrets' store (`store.md`) |
| `jig-client-protocol-wire` | codecs | engine, client | jig's wire records, both ends, from schemas: jig's messages, their operations and compound requests, subjects and reads, documents, the stream's events, the small store's records (`client.md`, section 3) |
| `jig-protocol-engine-clients` | component | engine | the clients' component: jig's messages from the application's transport, each with its credential as opaque bytes, and to it; a `Mint` from the application's server; streams and their pacing; credentials minted, verified, expired and revoked, the printed link's secret; translation to and from the core's vocabulary. No HTTP, path, cookie or listener (`client.md`, 4.2) |
| `jig-protocol-engine-clients-http` | mapping | engine | the default HTTP mapping's engine end: its routes under the prefix the application mounts it at, the stream as server-sent events, the credential as a cookie or a bearer, and the example guards (`client.md`, section 17; `browser.md`, 12.2) |
| `jig-client-protocol-link` | component | client | jig's messages encoded and decoded, the stream's events, the small store's records; no HTTP (`client.md`, 4.3) |
| `jig-client-protocol-http` | mapping | client | the default HTTP mapping's client end: a message as a method, a path, headers and a body; the stream's server-sent events cut into events; for the host and wasm32 (`client.md`, 17.4) |
| `jig-protocol-transcript` | decoding | engine, client | turns' bytes into the client's vocabulary (`../client/domain/client.md`, section 10) |
| `jig-client-shell-web`, `jig-client-shell-native` | shells' parts, a shell | client | jig's browser parts over skein's `skein-ui-web`: the client's adapter over skein's fetch, the printed link's sign-in, jig's look (`browser.md`, 5.1); and the native shell over skein's HTTP client, with `jig-client-protocol-http` (`client.md`, section 12) |

Beside them, smith's components, by the agent capability:
`smith-protocol-llm`, over skein's `skein-llm-connection`, in a process
that runs `smith-inline-agent`, with `smith-protocol-machine` on a
worker (`llm.md`); `smith-host-protocol` on a worker with agent
processes (smith's `protocol/hosts.md`, sections 2 and 3).

- **The client itself is not a component of these:** `jig-client`, the
  client the application holds, with its domain, `jig-client-domain`,
  and the reference UI, `jig-client-view`, are `../client/domain/`'s.
  Every client crate is named under `jig-client`, wherever it runs
  (`../README.md`, section 15).
- **Each crate is step code** under the subset (programming-model.md,
  section 10), as every crate of jig's is but its test kits and the
  client's shells and their parts. The reference UI's bundle is made at
  build by skein's bundler, `skein-ui-bundle`, a tool, not a component,
  and served by the application's server (skein's `ui-web.md`, 4.2;
  `browser.md`, 4.2 and section 12).
- **Each depends** on skein-lib, io's vocabulary, the skein components it
  drives, its codec crate, and the domain vocabulary it translates: the
  hub's for the channel's sides, the core's (or a child's) for the
  engine's others. None depends on an application's crate, and none names
  a system.
- **Which crates link smith.** Of jig's, only `jig-protocol-charter` and
  the client's decoding of turns; `jig-protocol-tools` gives its schemas
  as bytes the charter declares. The application links the rest where it
  composes them: its root, the agent capability; its protocol layer,
  smith's components and smith's codecs.

## 6. What an application writes

| Part | In | Written |
|---|---|---|
| the protocol layer's routing: io's tokens, events to the root, outputs to components, values between components, timers, smith's payloads decoded and encoded with smith's codec functions | each process | copied from the reference service (`service.md`, section 7) |
| `iterate` and `main` | each process | copied from the reference service and the reference binary, `jig-test` |
| an `agent` command, smith's agent shell, if its workers run agent processes from its own binary | the binary | copied from the reference binary (`service.md`, section 7) |
| its configuration's own members, and their parsing | each process | its own |
| its connectors' protocols: their systems' clients, credentials, and webhooks, on a listener of each connector's own at an address its configuration names, or on its server's, routed by path (section 13) | the engine | its own |
| its server: the clients' listener, its TLS, its routes and its URL space; its own pages, bundle and API; jig's clients' component, mounted under any prefix it chooses, since jig reserves no path; every HTTP and browser guard, jig's example guards as they are, wrapped, or its own | the engine | its own, after the reference service's (`service.md`, section 7; `browser.md`, section 12) |
| its transport: jig's messages carried between `jig-client` and the clients' component, each with its credential: jig's default HTTP mapping, as it is or wrapped, or its own (a local socket, an in-process channel) | engine and client | jig's default mapping, or its own (`client.md`, section 17) |
| its sign-in: every way of signing in but the printed link and services' credentials (OAuth, a directory, a chat platform's identity), on its own routes, redirects included, with its settings and secrets; the identity it verified, handed to the clients' component in a `Mint`, from its server code alone; how the minted credential travels | the engine, the client | its own (`client.md`, section 8) |
| its presentation: its routes and screens, words and look, confirmations and drafts, home and navigation, on `jig-client` and jig's helpers if they suit, with skein's UI kit, anything else, or nothing; or jig's reference UI, `jig-client-view`, as `ops` and jig's test application use it | the client | its own (`../client/domain/`) |
| its credentials for runs' workspaces: their tables, and their sources (a file, or an exchange it refreshes) on the engine | engine and worker | its own (`channel.md`, section 8) |
| its connectors' host tools: schemas, decoding, rendering | the engine | beside each tool, as `jig-protocol-tools` is for the core's (`../domain/hosts.md`, section 8) |
| its parts' encodings: workspace items, left and saved work, delivery evidence, with its payload version | engine and worker | its own codecs (`channel.md`, 6.2) |
| its periods' source | the engine | a child of its root (`configuration.md`, section 5) |
| the worker's root, with its choice of agent capability, its workspace and its io | the worker | its own (`../domain/hosts.md`, section 7) |
| its store's families: keys' encoders, codecs, its store version, its indexes | the engine | its own, by `store.md`, section 12 |
| its documents' and requests' wire, versioned per owner | engine and client | its own codec crate, generated as jig's is (`client.md`, section 11) |
| its templates: its own shapes of work, and jig's chat and goal kept, changed or dropped | the engine's configuration | its own data (`configuration.md`, 3.1) |

It writes no codec of jig's records, none of the channel's rules, none of
the core's tools' decoding, no OAuth exchange or key's keeping of an LLM
account, no agent, none of `smith-inline-agent`'s runs' LLM calls, and
no credential's keeping: jig mints, verifies, expires and revokes them.

## 7. Versions

- **Pre-release: one version at a time.** Each of jig's encodings has one
  current version. A reader refuses another, saying so, and nothing is
  translated (skein's `codec.md`, section 5).
- **No upgrade in place.** Everything that crosses a boundary or
  outlives a process carries its version, which an upgrade will need; an
  upgrade stops the engine and its workers together, and a store written
  by another version refuses to open (section 14).
- **The channel** is versioned whole, agreed at its opening; the
  application's payloads inside it by the payload version a worker's
  credential carries (`channel.md`, 4.1 and 6.2).
- **smith's charter, turns and results** carry smith's versions (smith's
  `protocol/README.md`, section 8). The engine writes charters in the
  version its release's smith writes, and its workers, of the same
  release, read them.
- **What outlives a process** carries its version: the store's records,
  per family (`store.md`, section 5), and the client's small store
  (`../client/domain/client.md`, section 11).
- **jig's messages are the exception:** they are versioned per owner,
  jig's by jig and the application's by the application, each message
  naming its owner and its version, and the engine serves a range of
  versions, so a client that does not come from the engine may lag it
  (`../client/domain/contract.md`, section 8).
  - *A version not served* is answered with a typed refusal, *version
    not served*, naming the owner and the ranges served, never a bare
    failure of the transport (`client.md`, section 9).
  - *One jig pin:* an application takes `jig-client` and the engine's
    components from one revision of jig, so a change to jig's
    operations moves jig's wire version, and before a release every
    range is one version.
  - *How jig itself is released and pinned* is open (`../README.md`,
    section 14).
- **Configuration** has no version: unknown fields refuse the start
  (`configuration.md`, section 9).

## 8. Secrets

| Secret | Held by | Kept | Reaches |
|---|---|---|---|
| an LLM account's signed-in source's record: its tokens and their expiry, the client id its provider issued, the scopes granted, the ID token | skein's accounts driver; the access tokens lent, `jig-protocol-accounts`' table | the store's secrets' records, by the source's number, before the access token is lent (`llm.md`, section 5; `store.md`, section 9) | the refresh token, its provider alone; the access token, with its scheme, `smith-inline-agent`'s runs in the engine through smith's LLM component, and workers through credential grants, then their agent processes through smith's host half, or smith's LLM component on a worker that composes `smith-inline-agent` (`llm.md`, section 6) |
| an LLM account's key | `jig-protocol-accounts`' table | the store's secrets' records, under the request that carried it, before the core sees the request; that request is never kept by the client (`client.md`, section 5) | as an access token, with its scheme |
| a program source's login | the program, in its configuration directory on each host; never jig | the host's, never the engine's (`llm.md`, 5.5) | the program alone; a grant carries the source's name |
| a worker's secret | the worker; its digest on the engine | configuration's files | the opening of the worker's link, over TLS, loopback, or an in-process address (`channel.md`, 4.1) |
| a credential, a sign-in's | `jig-client`, where its transport needs it in hand (a bearer); or a browser, as a cookie the application's server sets and the client never sees; its digest on the engine | the store's secrets' records | the application's server once, in `Minted`, where it asked for it; then each of the party's messages, verified by the clients' component (`../domain/people.md`, section 3) |
| a service's credential | the service; its digest on the engine | the store's secrets' records | its sign-in only, which answers a sign-in's credential |
| the printed link's secret | the clients' component, which gives it; the application's startup, which prints it in a URL of its own | the component's memory only, until spent or lapsed: a restart gives a fresh one | the client that signs in with it, once, on loopback: a browser that opens the link, or a native client |
| the workers' listener's TLS key | the engine | configuration's files | the workers' TLS |
| the clients' listener's TLS key; the secrets of the application's sign-in | the application's server | its own configuration's files | its exchanges |
| a connector's credentials | the application's protocol layer | its files, or its own records in the secrets' store | its system |
| an application's credential, for a run's workspace | the application's protocol layer: its table on the engine, and on each worker that holds a grant of it | its own source's: a file, or an exchange it refreshes | the workspace's systems, through credential grants with the assignment (`channel.md`, section 8) |

- **Names in the domain, values beside it.** A domain names a secret by
  number and generation (`../domain/engine.md`, sections 5.4 and 12); no
  domain state, fact, record of the main store or log holds a value.
  Values move between components through the protocol layer's routing,
  never through a root.
- **Kept before used.** An account's record is durable in the secrets'
  records before the access token it bought is lent, by skein's driver
  and its owner's-store keeper; a key is durable there before the core
  sees the request that carried it. A sign-in is committed
  first, which gives it its number; its credential's digest is then
  kept under that number; and only then is the credential answered, to
  the client that signed in or to the application's server that asked
  for it. A crash between them leaves at worst a record nobody uses, or
  a sign-in no client holds a credential for: never a name with two
  values, or a credential the engine does not know.
- **Across hosts, only over TLS,** in plaintext over loopback, or over a
  joined socket in one process, and validity as durations, never times.

## 9. The store

- **jig's store is skein's key-value store in the engine** (skein's
  `kv.md`): embedded in the engine's process, with transcripts beside it
  and secrets in a second store. There is no store process and no wire:
  the store is a component of the engine's protocol layer, and its bytes
  are its records on disk. jig's part is the engine's own records (its
  families, their keys and encoding, versions, commits and loads,
  transcripts, secrets, opening, growth and retention); the application
  adds its families beside jig's, in the same commits.
- **Its design is `store.md`.** What the rest of the protocol layer
  relies on is this:
  - **commits** numbered as the journal numbers them, applied in order,
    each answered once durable, which makes every earlier one durable; a
    failed commit, or one the store refuses, answered failed, after which
    nothing is answered and the engine stops (`../domain/engine.md`, 5.1
    and 5.5);
  - **loads** by the domain's ranges, in bounded pages, seeing every
    commit answered before they were asked;
  - **transcripts** appended with the commit that accepts their turns,
    each turn kept with whether it opens a window, and read from either
    end, or from the turn that opens a task's current window
    (`../domain/engine.md`, 7.2);
  - **secrets' records** in a store of their own, in a directory only the
    engine's user may read, written by components alone, durable on
    request before what they protect is used, read at start;
  - **opening** before the root starts, refusing a store of another
    format, or one whose records disagree with its own header.

## 10. What jig needs from skein

Each is generic, and says nothing about jig.

- **Joined sockets in one process:** a listener on an in-process name
  accepting sockets joined in memory to those that connect to it, with
  the same requests, events, backpressure and ends as TCP, and never
  reaching the simulator, for the one-process shape (section 4; skein's
  `io.md`, 3.5 and section 4).
- **TLS's server side with independent output,** for the engine's
  workers' listener between hosts, and for clients' HTTPS on an
  application's server built on skein's HTTP, as the reference's is
  (skein's `tls.md`, section 8; skein's `channel.md`, section 12). Not
  needed in one process.
- **Contained process trees,** for workers' agents: spawned within a
  deadline, with an environment given whole and only named directories
  writable, stopped politely, then terminated, then killed, and proved
  empty (`../domain/hosts.md`, section 10; skein's `draft/process.md`;
  smith's `protocol/README.md`, section 9 asks the same). Without them,
  each agent process, command and check runs in a process group of its
  own, which a stop signals whole (skein's `io.md`, section 6).
- **Private files read at startup:** the startup kit's
  `read_private_file`, refusing a file that is not private to the
  process's user, or reached through a link, for configuration's secrets
  (skein's `shell.md`, 6.1; skein's `io.md`, 5.3).
- **A cryptographic generator:** skein-lib's `CryptoRng`, keyed by the
  seed the shell draws at start (skein's `lib.md`, section 8; skein's
  `shell.md`, section 5), for minted credentials, the printed link's
  secret, services' credentials, the accounts driver's states and
  verifiers, and the installation's identifier (`store.md`, 4.4).
- **The loop's host:** skein's `Host` and `drive`, with
  `next_policy_deadline` (skein's `shell.md`, section 12; `service.md`,
  section 4).
- **For `jig-client` on WebAssembly, and its records:** skein-lib and
  skein-codec built for `wasm32-unknown-unknown`, with skein-http's
  machines for the default mapping's client end, and `skein-ui` for the
  reference UI, checked by skein's gate (skein's `lib.md`, section 2;
  skein's `testing.md`, section 6); skein-lib's bounded `base64`, for
  records in the default mapping's server-sent events (`client.md`,
  17.2);
  and skein-lib's `Secret`, the generator's `secret` type, whose `Debug`
  shows its length only, for credentials' values (section 8; skein's
  `codec.md`, section 3).
- **For the reference UI and its shells, skein's UI kit** (skein's
  `ui.md`, `ui-text.md` and `ui-web.md`), which an application's own
  presentation may use or leave:
  - `skein-ui`: the tree and its vocabulary, owners as opaque numbers,
    to which jig maps its own (`../client/domain/view.md`, section 3;
    skein's `ui.md`, 3.3), the builder with slots and splicing, bindings
    and acts, the diff and the view, style tokens and the look's names,
    the phrasebook's machinery with its reference hole, Markdown,
    scenes, the outline's rules, and limits with their worst case;
  - `skein-ui-text`, the text shell, which jig's terminal client
    composes with jig's settings (skein's `ui-text.md`);
  - `skein-ui-web`, the browser parts each usable alone and their
    default mapping of roles to elements, beside which jig's own parts
    sit (`browser.md`, 5.1): their platform halves call through the glue
    crates, `wasm-bindgen`, `js-sys` and `web-sys`, pinned in skein's
    workspace, which with their dependencies are the only `unsafe` such
    a client runs (`programming-model.md`, section 2.1);
    `jig-client-shell-web` forbids it in its own code;
  - `skein-ui-bundle`, the bundler, which writes the reference UI's
    bundle, under the prefix the application's server serves it at
    (skein's `ui-web.md`, 4.2 to 4.4; `browser.md`, 4.2);
  - `skein-fake-person`, the person over the tree, and
    `skein-fake-person-browser`, its face over skein's browser kit (a
    name matched by words as well as exactly, a press at a point within
    a node's box: skein's `browser.md`, sections 5 and 6) and the
    harness, which stops the application's binary and starts it again
    within a story, with skein's stories for a browser shell, which
    jig's tier calls beside its own (skein's `ui-web.md`, 14.2;
    `../client/domain/testing.md`, section 3; `browser.md`, section
    15).
- **LLM accounts' mechanism:** the accounts driver, with sources added
  and removed while it runs, sign-out with revocation, the client id a
  registration is issued kept with its record, signed in with the
  subject and scopes granted, and records kept through its owner's
  store (skein's `oauth.md`, 6.2 to 6.4); a credential with its scheme,
  and the failures' classes (skein's `llm.md`, sections 2 and 2.5).
- **What it has, as it is:** the framed channel, codecs and their
  generator, the JSON reader and writer, the key-value store, the LLM
  client and connection component and their fake peer, OAuth's client and
  fake issuer, HTTP's server and client, server-sent events.
- **The store** needs skein-kv as it is, and io's `OpenWrite` and `Cut`
  (skein's `io.md`, section 5; `store.md`, section 15).
- **Nothing new for the client's API.** Its generic half (keyed
  requests until durable, level-triggered watches, the link, generic
  guards) is jig's while it has one user.

## 11. What jig needs from smith

- **What smith gives:** charter v2, with each model's window, output and
  prices with cache writes, and the budget's reserve (smith's
  `protocol/charter.md`, section 2); transcript v5,
  whose turns say whether they open a window (smith's
  `protocol/transcript.md`, section 3); channel v2, with a start's
  messages and the run's identity, messages refused at the kit's
  entrance, the answer's final fence, and content-free facts (smith's
  `protocol/channel.md`, sections 3 to 8); the host domain and its host
  half, `smith-host-protocol`, with the agent process and its shell as a
  library, `smith-agent-shell` (smith's `protocol/hosts.md`, section 3);
  the parent face (smith's `domain/host.md`, 9.4); `smith-inline-agent`,
  with its LLM component, serving many runs' completions with one table
  of grants, and its machine component; and smith's codecs between bytes
  and smith's values: `smith-protocol-channel`'s functions, and
  `smith-channel`'s records for a call's name and a fact (section 13).
- **`smith-transcript` built for `wasm32-unknown-unknown`,** as it is:
  `jig-client` decodes turns with it below its surface, through
  `jig-protocol-transcript`, whose build for that target jig's gate
  checks (`../client/domain/client.md`, section 10).
- **LLM accounts' grants:** a grant carrying a credential with its
  scheme, or a program source's name; the limited, refused and
  unauthorized notices; the program backend, which runs Claude Code
  with its own login and its built-in tools asking smith's policy; and
  the host half's sources, login checks and sign-outs (smith's
  `protocol/channel.md`, section 6; `protocol/hosts.md`, section 2;
  `protocol/programs.md`, sections 3 to 5).
- **Not asked: a completion's result contract.** A protocol layer that
  runs `smith-inline-agent` keeps each slot's beside smith's LLM
  component, as smith's own local host does (`llm.md`, section 3).
- **What jig asks of smith:**
  - **a run's live content on smith's channel:** the host asks at a run's
    start, one boolean per kind (`text`, `reasoning`, `input`), and
    jig's assignment carries the ask, set for a run whose task is a
    conversation. smith sends, best effort and counted when dropped, the
    main conversation's pieces of the kinds it builds (now `text`), each
    naming the turn it will be told as, its completion's retry, its kind
    and its offset, all before that turn's record. An ask for a kind
    smith does not build is not refused, and gets nothing. Without the
    ask, a run in an agent process shows its activity, and its words as
    each turn commits (`channel.md`, 6.4; `../domain/hosts.md`, 6.5).
    A conversation's presentation wants it. A tool begun with its call
    stays `smith-inline-agent`'s;
  - **closing smith's LLM component as an owner closes it, later:** an abort
    that ends every completion in flight, for a process that stops
    without draining (`service.md`, section 5), and a completion asked
    while it closes refused as closed, never as a local limit.

## 12. What the domain owes

What the protocol layer relies on from jig's domain is designed there:

- **the engine stopping:** `../domain/engine.md`, 6.1;
- **credential grants pushed and resent, and LLM accounts:**
  `../domain/engine.md`, sections 8 and 12; `../domain/accounts.md`;
  `llm.md`, section 5;
- **the hub's link vocabulary, whole, and smith's payloads and facts by
  hand-out:** `../domain/hosts.md`, 6.1 and 6.8;
- **what smith gives** (section 11): windows, and resuming from the
  current one (`../domain/engine.md`, 7.2); the start's messages
  (`../domain/engine.md`, 7.1; `../domain/hosts.md`, section 9), and
  messages refused or unread kept in the inbox (`../domain/tasks.md`,
  8.2 and 8.3); refusals carried with turns (`../domain/hosts.md`, 6.4);
  the hello's agent bounds checked (`../domain/engine.md`, section 8);
  no run placed that smith would refuse for its budget
  (`../domain/authority.md`, section 7);
- **an application's credentials:** `../domain/engine.md`, 12.1;
- **the client's API:** `../domain/people.md`, sections 3, 5.1 and 6;
  `../domain/tasks.md`, sections 3, 4, 7 and 11; `../domain/engine.md`,
  section 11; what the domain owes the client, item by item,
  `../client/domain/contract.md`, section 9.

What the store asks of the domain is `store.md`, section 15.

## 13. Decisions this design takes

- **jig ships components, not references to copy,** for its boundaries
  whose vocabulary is jig's: the channel's two sides, accounts, host
  tools, configuration, the client's sides. What is generic is skein's,
  what is the application's is the application's; the hard parts of
  jig's own boundaries (authentication, liveness, keeping secrets before
  use, decoding the LLM's calls) are written once. The store too: jig's
  component takes the application's records as bytes (`store.md`,
  section 12).
- **Records from schemas,** with skein's generator, for jig's channel and
  its other codecs, as smith's are.
- **The channel on skein's channel,** one per worker, dialled by the
  worker, TLS between hosts and plaintext only over loopback or in one
  process, a name and a secret authenticating the worker, and the
  application's payload version checked at the opening (`channel.md`,
  section 4).
- **The hub's vocabulary on both ends of the channel,** so the engine's
  root routes a worker's link as it routes its own hub (`channel.md`,
  section 1).
- **The worker's link policy is its component's:** dialling, redialling
  with backoff, liveness and the grace's clock; its stop bound counted
  from the engine's view of the link (`channel.md`, 10.1).
- **Host tools' calls decoded in the protocol layer,** after the fleet has
  fenced them, by `jig-protocol-tools` for the core's tools and by the
  application's code for its connectors' (`channel.md`, 6.3). It is a
  deliberate exception to decoding before the domain
  (programming-model.md, section 4): smith's domain carries a host tool's
  input as the LLM wrote it, so a call from a run in the engine is inside
  the domain before anything could decode it, and only the fleet can tell
  a live attempt's call from a stale one. One path serves both shapes:
  fenced, then handed out to be decoded, and handed back typed. A run's
  result takes the same path (`channel.md`, 6.1).
- **A run's text keeps one contract,** by turn, retry and offset,
  whatever its source (`../domain/hosts.md`, 6.5): a host that drains
  `smith-inline-agent`'s content, or agent processes on a smith channel
  that carries it at the host's ask (section 11; `channel.md`, 6.4).
- **Agents are smith's,** both capabilities speaking smith's parent face
  (section 4); jig ships no agent crate.
- **smith's payloads as bytes in jig, typed only in smith.** Siblings
  share no domain types (programming-model.md, 4.5): the hub, the core
  and the channel's components carry smith's payloads as bytes and link
  no smith crate. The root that composes an agent capability hands each
  payload out, and its protocol layer's routing decodes or encodes it,
  and decodes smith's facts into jig's activity, by the path of host
  tools' calls (`../domain/hosts.md`, 6.8; `channel.md`, 6.1).
- **smith's LLM component for runs in a host's own process,** with each
  slot's result contract kept beside it (`llm.md`, section 3).
- **A connector's webhooks where the application routes them:** on a
  listener of the connector's own, owned by its protocol component, or
  on the application's server beside its clients, routed by path, so a
  deployment may expose one origin. jig owns no listener for clients,
  so the choice is the application's.
- **A reference binary,** `jig-test`, which an application copies for
  its `main`, and the binary jig's end-to-end and live tests run
  (`service.md`, sections 7 and 8).
- **LLM accounts on skein's accounts driver and the secrets' store:**
  sign-in, refresh, sign-out and tokens lent by generation are skein's;
  records and keys are kept in the store's secrets' records before
  lent; program logins stay the program's, on each host; the accounts
  child keeps which sources are usable and who holds which generation
  (`llm.md`, section 5).
- **What is generic leaves jig,** within reason: a mechanism that is not
  about jig's vocabularies goes to skein, or to smith when it is about
  agents, generalised, before jig uses it: joined sockets (section 10),
  the accounts driver, the agent in a host's process, the UI kit (the
  tree, the text shell, the browser parts, the bundler and the fake
  person: section 10), and a run's text on smith's channel (section
  11). jig keeps what is entwined with its domain: the worker's link
  policy, whose grace and stop bound are the fleet's, and the
  translations.
- **Crates named by layer, and every client crate under `jig-client`:**
  `jig-domain-*` and `jig-protocol-*` for what runs in the engine and on
  workers; `jig-client-{domain,protocol,<kind>}-*` for the client's,
  wherever it runs, with `jig-client` their base, as `skein-ui` is
  `skein-ui-*`'s (`../README.md`, section 15).
- **On one person's machine, a browser on loopback** signs in by the
  printed link: the clients' component gives a one-time secret at
  start, and the application's startup prints it in a URL of its own,
  `<prefix>/v1/sign-in/link?secret=…` under the default mapping
  (`client.md`, section 8, and 17.1).
- **Presentation, the server and the transport are the application's.**
  jig's clients' component takes jig's messages with an opaque
  credential, and knows no HTTP, path, cookie, host, CORS or server-sent
  event; jig reserves no path. The application's service owns the
  listener and its URL space, mounts the component where it chooses,
  and owns every HTTP and browser guard, since the guards that protect
  a browser's credential belong to the transport, not to jig's model.
  jig ships what an application may take or leave: its default HTTP
  mapping, both ends, with example guards, and attack stories in its
  test kit, checked and never mandated (`client.md`, section 17;
  `browser.md`, 12.2 and section 15); and its reference UI, on the
  public client alone, which proves the API is enough
  (`../client/domain/`).
- **jig owns credentials, the application owns sign-in flows.** jig
  mints, verifies, expires and revokes credentials, and keeps their
  digests; its built-in ways of signing in are the printed link and
  services' credentials. Every other way runs on the application's
  routes and ends in a `Mint`, which only its server code sends, and
  which names an identity of a provider of kind application
  (`configuration.md`, section 3). jig adds no "acting for": a bot
  relaying a person's decision holds that person's own credential
  (`../domain/people.md`, section 3; `../client/domain/contract.md`,
  section 2).
- **Versions per owner, refused by type:** a message in a version the
  engine does not serve is answered *version not served*, naming the
  owner and the ranges served; `jig-client` and the engine come from
  one jig pin (section 7).
- **Templates are configuration, served and never expanded by the
  engine** (`configuration.md`, 3.1), and lists are task queries: live
  tasks scanned in memory, ended ones through one index of the store,
  chosen by the query's keys (`store.md`, 7.3 and 7.4).
- **Configuration as strict JSON,** read once before the loop, composed
  per child, secrets in private files, limits by profile
  (`configuration.md`).
- **The store on skein-kv in the engine:** one transcript file per task,
  the current window's start kept in the map, records from schemas,
  another version refused (`store.md`).
- **Workers are alike:** configured the same but for their name and
  secret, keeping nothing between runs that the engine does not hand them
  again. Every credential a run needs, an LLM account's or an
  application's for its workspace's systems, is the engine's, granted
  with the assignment (`channel.md`, section 8), but a program's login,
  which is the program's own on its host, and is why a host keeping a
  person's logins takes only runs that person's accounts pay for, until
  a run's process tree is contained from them (`llm.md`, 5.5).
- **No upgrade in place, and no rolling upgrades** (section 7).
- **One process** (section 4): the workers joined to the engine by
  in-process addresses, their agents child processes, a browser on
  loopback, with no TLS.

## 14. Open questions

- **A sign-in's redirect relayed by a client,** for a browser on
  another machine than the engine's (`llm.md`, section 10).
- **Upgrades:** in place, across a store's format: records translated
  between versions, and rolling upgrades of the engine and its workers
  (section 7).
