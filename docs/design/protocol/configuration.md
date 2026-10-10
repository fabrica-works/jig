# Configuration

Provisional, 2026-10-10. What an engine and a worker are told when they
start, as far as jig defines its shape: the parts of
`../domain/engine.md`, section 14 that are jig's, how an application's
configuration composes them with its own, where secrets come from, and
when configuration is read. What is still open is listed in section 9.

## 1. In one page

- **Read once, before the loop.** A process's shell reads its
  configuration file whole, within a bound, with skein's startup kit
  (skein's `shell.md`, 6.1), and parses it before io starts. Nothing is
  read again while the process runs: a change is a restart.
- **JSON, strict.** The file is a JSON document read with skein's JSON
  reader (skein's `json.md`). Unknown fields, missing fields, values past
  their bounds and duplicate names refuse the start, saying which, as
  smith's agent does with its own (smith's `crates/smith/README.md`).
- **One document per process, composed like its root.** The application's
  configuration is one object, with a member for jig's part and one for
  each of its connectors and its own children. `jig-protocol-config`
  parses jig's part; the application parses its own; its startup code
  hands each child its part, as its root hands each child its limits
  (`../domain/root.md`, section 6).
- **Typed, then checked whole.** Parsing yields plain values of jig's
  domain and protocol vocabularies. A check then runs across them: names
  unique, numbers that refer to something configured, limits whose worst
  case fits the process's memory. A configuration that fails it never
  starts the loop.
- **No secret in the file.** A secret is a file of its own, which the
  configuration names, read at startup as a private file (skein's
  `io.md`, 5.3): owned by the process's user and readable by nobody else,
  or the start is refused.
- **What lives in the store is not configuration.** Projects, their
  policies and roles, parties and their pools are the store's, changed by
  parties' requests. Configuration seeds them on a deployment's first
  start only.

## 2. Where it sits

```
the application's main (its shell)
├── reads the file, and each secret file it names            skein's startup kit
├── parses: jig's part (jig-protocol-config), its own parts   plain values
├── checks the whole, and the worst case
└── makes the root, the protocol layer's components, io       then the loop
```

- **`jig-protocol-config`** is a crate of jig's, step code: functions from
  a parsed JSON value to jig's typed configuration and back to the
  problems it found, each naming its path in the document. It reads no
  file. It yields jig's own plain values only, and links neither smith nor
  skein's LLM crates, so an application without agents never links them
  (`../README.md`, section 9); an application with agents turns the
  endpoints and charters it yields into smith's and skein's types in its
  startup (`llm.md`, section 4).
- **The application's startup** is ordinary code of its main: it reads,
  parses with `jig-protocol-config` and its own parsers, checks, and
  constructs. `ops`'s main is the reference (`service.md`, section 7).

## 3. jig's part for an engine

| Member | What it gives | Used by |
|---|---|---|
| deployment | the store's directory; the seed's source, for tests | the store; the shell |
| owners | the deployment's first owners, each a provider and a subject (`../domain/people.md`, section 3) | the people child, on a first start |
| providers | each sign-in provider's number and kind; its settings are the application's | the people child; the client's protocol |
| rules | the deployment's rules: the ceilings every project's policy is under (`../domain/authority.md`, section 6) | authority |
| projects | each project seeded on a first start: its number, its resources and their roles, its homes, its policy, its parties' roles | the core, on a first start |
| charters | each charter the deployment offers, numbered: its instructions (a file it names), its models, its tool families, the hosts that may run it, its waiting time, whether its runs resume, its default budget, which applies to a task whose batch gives none (`../domain/hosts.md`, section 8); its models' endpoints name one account, until a run needs several; and which charter chats run under | the core; `jig-charter` |
| hosts | the engine's own slots, and the inline agent's limits, if it runs agents; the workers' names, each with the file of its secret's digest; the workers' listener: its addresses, socket addresses or an in-process name (skein's `io.md`, section 4), and its certificate and key files, but for an in-process name (`channel.md`, section 10) | the fleet; `jig-protocol-engine-workers` |
| llm | endpoints, models and prices; accounts, each with its registration and its token endpoint, and the private directory of their records (`llm.md`, sections 4 and 5) | the core; smith's and skein's components and skein's accounts driver, through the application's startup; `jig-protocol-accounts` |
| clients | the clients' listener (a TCP address, or loopback on one person's machine) and its certificate and key files, but on loopback; the heartbeat; whether the engine prints a local sign-in link at start (`client.md`, sections 4 and 8) | `jig-protocol-engine-clients` |
| connectors | the application's connectors' numbers, as the core knows them (`../domain/core.md`, 3.5) | the core |
| limits | a profile, and any limit overridden by name (section 6) | every child and component of jig's |

- **Each connector's own configuration** (its system's address, its
  credentials' files, its procedures' settings, its templates) is the
  application's, in its own member.
- **The application's own children** take their configuration from the
  application's members, as its root gives them.

## 4. jig's part for a worker

| Member | What it gives |
|---|---|
| engine | the engine's address, and its TLS server name and trusted roots; or plaintext to a loopback address; or an in-process name, in one process (`channel.md`, 10.1; `service.md`, section 10) |
| name, secret | the worker's name, and the file holding its secret |
| slots | how many runs it hosts at once |
| graces | the grace after losing the engine, the stop graces of an agent's tree, a delivery's deadline, a save's: with the link's ping interval and silence deadline, what its stop bound is made of (`channel.md`, 10.1; `../domain/hosts.md`, 6.6) |
| agent | the agent program and its configuration, which smith's host half starts, with the endpoints its runs' charters name, by the same names as the engine's (smith's `protocol/hosts.md`, section 3; `llm.md`, section 4) |
| limits | a profile, and overrides |

The worker's workspace's configuration is the application's. Workers are
alike: every worker's configuration is the same but for its name and
secret, and every credential its runs need comes with their assignments
(`channel.md`, section 8).

## 5. Periods

- **The core keeps no calendar** (`../domain/authority.md`, section 7). A
  project's funding period reaches it from the application's root, saying
  when the next begins if its source knows.
- **A period's source is the application's:** a child of its root that
  keeps a calendar from the application's configuration (a monthly
  period starting on a day, in a time zone), arms its deadline and says
  each period, or an operator's request that ends one. It is a decision,
  so it is a domain's, never the protocol layer's, which only fires the
  deadline. jig's configuration names none, and `ops`'s shows one
  (`service.md`, section 7).

## 6. Limits

- **Policy above, numbers below.** The design documents name limits and
  what they bound; the numbers are in `jig-protocol-config`'s profiles,
  as module documentation and constants, never in the design.
- **A profile** is a named set of every limit jig's children and
  components take, consistent with one another: a small one for one
  person's machine, a standard one for a team. A deployment names a
  profile and overrides any limit by its name.
- **Checked whole at startup.** The root's worst case (`../domain/root.md`,
  section 10), the components' and the journal's, summed, must fit the
  memory the configuration states; the terms of the workers' channel
  follow from the same limits (`channel.md`, section 11). A profile whose
  sum does not fit refuses the start, saying by how much.
- **The application's limits** are its own members, checked in the same
  sum.

## 7. Secrets

| Secret | Kind | Configured as |
|---|---|---|
| a worker's secret | shared between the engine and that worker | on the worker, a file holding it; on the engine, a file holding its digest, per worker |
| an LLM account's record | its tokens, as skein's accounts driver keeps them | a private file per account, written by a sign-in, loaded by the driver (`llm.md`, 5.4) |
| an LLM account's client secret, if its provider has one | a confidential client's | a file |
| the listeners' TLS keys | private keys | files beside their certificates |
| a sign-in provider's client secret | a confidential client's | a file (`client.md`, section 8) |
| a connector's credentials | the application's | the application's files, or its own secrets' records |

- **Read at startup,** each within its bound, from a private file, with
  skein's `read_private_file` (skein's `shell.md`, 6.1): a file readable
  by anyone but the process's user, or reached through a link, refuses
  the start, naming it.
- **Held by the protocol layer,** never the domain: the domains name
  secrets by number (`../domain/engine.md`, section 12).
- **What changes while the engine runs** is kept by its owner, never
  written back to configuration: a rotated refresh token in the account's
  private file, by skein's driver; a sign-in's digest in the store's
  secrets' records.

## 8. First start, and every start

- **A deployment's first start** is the first with an empty store: the
  shell draws the deployment's id, and the core seeds owners and projects
  from configuration into its first commit (`../domain/engine.md`, 5.4
  and section 14).
- **Every later start** reads the deployment from the store. The
  configuration's projects are ignored; its rules, charters, limits,
  hosts, endpoints and accounts apply as configured, and a narrowing
  reaches every later decision, as a policy's does (`../domain/authority.md`,
  section 6).
- **What the store holds and configuration no longer names** (a charter
  removed, a connector gone) is the domain's to judge at restart, as it
  judges a live set past limits lowered since (`../domain/engine.md`,
  section 6); what it decides is open (section 9).

## 9. Open questions

- **Reloading without a restart:** a deployment's rules, charters and
  limits changed while it runs, which the domain would hear as an event,
  as a policy change is, in place of a restart.
- **Removed parts at restart:** whether a charter or a connector that
  live work names may be removed, and what becomes of that work: a
  refused start, saying which, or the work's runs failing as not
  offered.
- **Profiles' numbers,** from the first deployments' measurements.
- **A configuration's format version,** once a deployment's configuration
  must survive a release that changes it.

## 10. Formats an operator writes

- **An address** is one of three objects:
  - `{"socket": "127.0.0.1:7100"}`, a socket address, IPv4 or IPv6;
  - `{"host": "issuer.example", "port": 443}`, a host's name, which the
    shell resolves at startup (skein's `shell.md`, section 6);
  - `{"in_process": "<name>"}`, an in-process name within skein's bound
    (skein's `io.md`, section 4), in one process only (`service.md`,
    section 10).
- **A digest file** holds one SHA-256 digest as 64 lowercase hexadecimal
  digits, and nothing else but a final newline.
- **A secret file** holds the secret's bytes as they are, and nothing
  else but a final newline, which is not part of the secret.
