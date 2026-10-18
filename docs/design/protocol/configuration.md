# Configuration

Provisional, 2026-10-10, revised 2026-10-18. What an engine and a worker
are told when they start, as far as jig defines its shape: the parts of
`../domain/engine.md`, section 14 that are jig's, the templates an
application defines its shapes of work with, how an application's
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
  smith's agent does with its own (smith's `protocol/agent.md`, section
  4).
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
  parties' requests. Configuration seeds projects, their policies and
  their parties' roles on a deployment's first start only; resources and
  homes come by adoption after it (section 3).
- **Templates are data the engine serves and never expands.** The
  application's shapes of work (jig's chat and goal among them) are
  numbered templates in jig's part; a project's policy may override some
  of their defaults; the engine serves each project's merged templates
  to the client, which fills them into ordinary requests (3.1).
- **The server is the application's.** The clients' listener, its
  certificate and key, and whatever serves a browser are the
  application's service's own members; jig's part keeps what the clients'
  component takes: the heartbeat and the printed link (section 3).

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
  constructs. The reference binary's, `jig-test`'s, is the reference
  (`service.md`, section 7).

## 3. jig's part for an engine

| Member | What it gives | Used by |
|---|---|---|
| deployment | the deployment's directory, which holds the store, transcripts and the secrets' store; retention's horizons (`store.md`, 11.1); the seed's source, for tests | the store (`store.md`); the domain's retention; the shell |
| owners | the deployment's first owners, each a provider and a subject (`../domain/people.md`, section 3), each an owner of every project seeded with it | the people child, on a first start |
| providers | the numbers and kinds that identities name (`../domain/people.md`, section 3), and nothing else: jig runs no provider's exchange. Of kind *link*, the printed link's; of kind *deployment*, exactly one, the deployment's own, for services' credentials; of kind *application*, each of the application's ways of signing in, whose identities its server verifies on its own routes and asks the clients' component to mint a credential for (`client.md`, section 8). The settings and secrets of the application's sign-in are its own members | the people child; `jig-protocol-engine-clients`, which mints a printed link's credential for the first owner it names, a service's under the deployment's provider, and a `Mint`'s only for an identity whose provider is of kind application |
| rules | the deployment's rules: the ceilings every project's policy is under (`../domain/authority.md`, section 6) | authority |
| projects | each project seeded on a first start: its number; its policy, with its spend per period, its roles (each one's authority, the spend a party in it may allot per period, the kinds of proposal it decides; `../domain/authority.md`, section 6) and its overrides of templates' defaults (3.1); its parties' roles. Its resources and homes come by adoption (below) | the core, on a first start |
| charters | each charter the deployment offers, numbered: its instructions (a file it names), its models, named from the llm member's (its main model, and those its sub-agents may use), its tool families, the hosts that may run it, its waiting time, whether its runs resume, its default budget (turns, spend and time, and a reserve of turns and time), which applies to a task whose batch gives none (`../domain/hosts.md`, section 8); one account, which its models' endpoints share (`../domain/engine.md`, 7.1) | the core; `jig-protocol-charter` |
| templates | each template the application defines, numbered and never renumbered, with its defaults; jig's chat and goal among them unless the application drops them (3.1) | the core, only to serve each project's merged templates in `me` (`../client/domain/contract.md`, 5.1); the client, which fills them in |
| hosts | the engine's own slots, if it runs agents in the engine (shape 2), and their `smith-inline-agent`'s limits; the workers' names, each with the file of its secret's digest; the workers' listener: its addresses, socket addresses or an in-process name (skein's `io.md`, section 4), and its certificate and key files, but for an in-process name (`channel.md`, section 10); a listener on an in-process name tells the core its workers are in its process, and that no worker outlives it (`../domain/engine.md`, section 6) | the fleet; `jig-protocol-engine-workers`; `smith-inline-agent`, through the application's startup |
| llm | endpoints, each with the models it serves; each model's window, output and the rest of what smith's agent configuration declares of it (smith's `protocol/agent.md`, section 4), and its prices: input, cached input, output, and cache writes when priced apart; accounts, each signed in, with its registration and its token endpoint, or handed in, with the private file of its access token (`llm.md`, 5.4), and the private directory of their records (`llm.md`, sections 4 and 5) | the core; smith's and skein's components and skein's accounts driver, through the application's startup; `jig-protocol-accounts` |
| clients | the heartbeat of the clients' streams (`client.md`, section 4); whether the clients' component gives a printed link's one-time secret at start, whose URL the application's startup prints (`client.md`, section 8), only where its clients reach it on loopback, which that startup checks against its own listener (`service.md`, section 3). The listener, its certificate and key, and what serves a browser are the application's service's own members (`service.md`, section 7) | `jig-protocol-engine-clients` |
| connectors | the application's connectors' numbers, as the core knows them (`../domain/core.md`, 3.5); none takes the number the core keeps for its own procedures (below) | the core |
| limits | a profile, and any limit overridden by name (section 6) | every child and component of jig's |

- **Each connector's own configuration** (its system's address, its
  credentials' files, the address of its webhooks' listener, its
  procedures' settings and texts) is the application's, in its own
  member.
- **An application's credentials** for its runs' workspaces (a token to
  fetch and push) are its own members: each with its number, as the
  domain names it, and its source, a file or an exchange it refreshes.
  jig's part names none; the accounts child hears each generation from
  the application's root (`../domain/engine.md`, 12.1; `channel.md`,
  section 8).
- **The application's own children** take their configuration from the
  application's members, as its root gives them.
- **What the core takes that no member names:**
  - *a project's current period and its budget,* from the application's
    root as each period begins (section 5), the first at the
    deployment's first start, before the core admits a party's request;
    the store keeps the current one across restarts
    (`../domain/engine.md`, 5.4), and the policy's spend per period
    bounds its budget;
  - *a party's pool* for a period: the allotment their role gives, from
    the project's policy;
  - *a task's authority and budget, as a template gives them:* in the
    request, never in configuration the core reads. The client fills a
    template's authority (the creator's role narrowed) and its budget
    into a `Create`, and the core checks them as it checks any request
    (`../domain/authority.md`, sections 5 and 6), a budget given as
    `AsAllowed` resolved at admission (`../domain/authority.md`, section
    7);
  - *the core's recurring procedure* (`../domain/tasks.md`, section 10):
    the core's own, under a number the core keeps for its procedures,
    which no connector takes;
  - *the engine's tool families' bits* in an authority: jig's own
    numbering, the same in every deployment; a charter names the
    families it allows;
  - *a resumed conversation's bytes and the hub's turns:* limits, from
    the profile (section 6).
- **Resources and homes come by adoption,** after the first start, by an
  owner's request (`../domain/people.md`, 5.1): a resource is a
  project's once its connector has read what the application may do
  there (`../domain/connectors.md`, section 12), which configuration
  cannot say. A home is a resource its adoption names as the project's
  home for that connector. A project seeded with none has none until
  then.

### 3.1 Templates

- **A template is the application's recipe for a shape of work:** the
  defaults that turn "a chat with these words" or "a goal for this"
  into a complete `Create` (`../domain/tasks.md`, section 4). An
  application's own shapes (one for `ops`'s incidents, say) and jig's
  defaults are alike.
- **Numbered, never renumbered.** A number, above zero, is the
  template's identity across the configuration, projects' policies and
  clients; a template dropped leaves its number unused.
- **What each carries,** every field required (section 10):
  - a name, for showing;
  - the executor: an agent under a charter the template names, by its
    number among the charters member's; or *chosen*, a charter or
    procedure the creator's role may create with, the creator's choice;
  - the result contract: a report, a verdict or a connector's result,
    each with its bounds (`../domain/tasks.md`, section 3); never an
    approval, whose state only a procedure gives;
  - the authority: the creator's role narrowed, by as many levels of
    delegation as its `shallower` says;
  - the budget: an amount; `AsAllowed`, with a cap or none
    (`../domain/authority.md`, section 7); or *given*, the creator's,
    within their allotment;
  - the deadline: none; a span from creation, which the client sends
    as a span and the engine makes a wall when it makes the task
    (`../domain/authority.md`, section 7); or *given*, the creator's,
    or none;
  - tracked, with a default priority, or untracked;
  - the flags: *conversation*, *listed*, and the retention class,
    *kept* or *forgotten once read* (`../domain/tasks.md`, section 3);
  - the label: an opaque 32-bit number, which jig gives no meaning;
  - the grace its helper cancels with, or none.

  It names no wake policy, parameters, inputs, dependencies or
  resources: a task filled from it wakes and parks as its executor's
  defaults say (`../domain/tasks.md`, 8.3), and its helper names none
  of the rest (`../client/domain/client.md`, section 19).
- **jig's defaults** are two, which an application keeps, changes or
  drops, by writing them in its `templates` member or leaving them out;
  none is implied (section 10). jig's helpers find them by number:
  template 1 is the chat they fill, template 2 the goal
  (`../client/domain/client.md`, section 19). What the table leaves
  unnumbered (the chat's charter, a report's bound, the chat's grace)
  the application writes, and the start checks (below).

| Field | chat (template 1) | goal (template 2) |
|---|---|---|
| name | chat | goal |
| executor | an agent under a charter the template names: the application's charter for chats | chosen: a charter or procedure the role may create with |
| contract | a report | a report |
| authority | the role's, one level shallower in delegation | the role's, one level shallower in delegation |
| budget | `AsAllowed`, with no cap | given: the creator's, within their allotment |
| deadline | none | given: the creator's, or none |
| tracked | no | yes, with a default priority |
| conversation | yes | no |
| listed | no | yes |
| retention | kept | kept |
| label | 1 | 2 |
| grace | a span, which the helper closing a chat cancels with | none |

- **A project's policy may override,** per template number, the
  executor, the budget and its cap, the deadline and the default
  priority (`../domain/authority.md`, section 6), the template's own
  standing for each it leaves `null` (section 10). Never another field,
  and never the flags, the label or tracked: they are the shape's
  identity, which lists and the engine's behaviour key on. An override
  naming a template the configuration does not define, or one another
  override of the same policy names, is refused when the policy is
  changed (as unknown, or as a repeated identity), and a seeded
  policy's refuses the start.
- **Merged for `me`:** a project's merged templates are the
  configuration's, in its order, each with that project's override
  applied: its executor, budget and deadline where the override gives
  them, and its priority where the template is tracked. An override of
  the default priority of an untracked template is kept, and shown with
  the policy, but has no effect: the merged template stays untracked.
- **The engine never expands or checks one.** The core keeps the
  configuration's templates, and each policy's overrides, only to serve
  each project's merged templates in `me` (`../domain/engine.md`, 11.1;
  `../client/domain/contract.md`, 5.1). The client fills a template
  into an ordinary request, which the core checks as it checks one
  written by hand. A template grants nothing: whatever it makes, a
  `Create` could make within the creator's authority. A recurring task
  repeats a batch of its own, its recurring batch, never a template
  (`../domain/tasks.md`, section 10).
- **Permissions stay primitive:** which charters and executors a role may
  create with, its spend and its delegation (`../domain/authority.md`,
  section 6). No role is given or refused a template.
- **A task keeps the values it was made with.** A template changed or
  dropped at a later start touches no task already made; a policy's
  override naming a template the configuration does not define at that
  start serves nothing, and the next change of that policy must leave
  it out, as above.
- **Checked at start:** no more templates than the core's limit on them
  (section 6); numbers unique and above zero; each name within the
  documents' bound on a text; each charter a template's executor names
  configured; each contract within the tasks child's bounds; each grace
  within the tasks child's limit on a grace's span
  (`../domain/tasks.md`, section 11); each seeded policy's overrides
  naming configured templates, each at most once, and no more of them
  than the limit on templates.

## 4. jig's part for a worker

| Member | What it gives |
|---|---|
| engine | the engine's address, and its TLS server name and trusted roots; or plaintext to a loopback address; or an in-process name, in one process (`channel.md`, 10.1; `service.md`, section 10) |
| name, secret | the worker's name, and the file holding its secret |
| slots | how many runs it hosts at once |
| graces | the grace after losing the engine, the stop graces of an agent's tree, a delivery's deadline, a save's: with the link's ping interval and silence deadline, what its stop bound is made of (`channel.md`, 10.1; `../domain/hosts.md`, 6.6) |
| agent | the agent program, which the worker's protocol layer spawns for each run, and which may be the application's own binary run as its `agent` command, smith's agent shell (`service.md`, section 6); and its configuration, smith's agent configuration (smith's `protocol/agent.md`, section 4), which declares the models the engine's charters name, with their window and output, and the endpoints they run on, by the same names as the engine's (`llm.md`, section 4). The worker's routing decodes charters against its endpoints, and its hello states its agent's bounds from it (`channel.md`, 4.3). A worker that composes `smith-inline-agent` gives it the same configuration, and runs no agent program |
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
  period starting on a day, at a fixed offset from UTC), arms its deadline and says
  each period, or an operator's request that ends one. It is a decision,
  so it is a domain's, never the protocol layer's, which only fires the
  deadline. jig's configuration names none, and `ops`'s shows one
  (`../examples.md`, section 4).
- **The smallest source** is the reference service's: at a
  deployment's first start (section 8), before the core admits a
  party's request, its root opens one period for each configured
  project, its budget the project's spend per period, with no next
  start, since it keeps no calendar. A later start says none: the store
  keeps the current period (`../domain/engine.md`, 5.4).

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
- **Charters' budgets:** each charter's turns and time are above its
  reserve's, and its default budget holds its reserve and one of its main
  model's most costly completions (`../domain/authority.md`, section 7),
  or the engine's start is refused, naming the charter.
- **Operations per request,** the people child's limit on a compound
  request (`../domain/people.md`, 5.1), is at least the larger of
  *delegates per task* plus one and *goals per project*, both the tasks
  child's (`../domain/tasks.md`, section 11), or the start is refused,
  naming the three. So closing a conversation, each of its live
  delegates cancelled or moved and the conversation then cancelled with
  a grace, fits one request; and so does reordering every live goal of
  a project.
- **A compound request's worst case** is `../domain/engine.md`, section
  13's, summed into the engine's worst case with the rest.
- **A template's grace** is within the tasks child's limit on a grace's
  span, or the start is refused, naming the template (3.1).
- **The deployment's message bound** holds every kind of message a
  task's inbox admits, as a run renders it with its sender's label: a
  party's words and questions by the people child's bound, which is at
  most it, and every other kind by its own (`../domain/tasks.md`, 8.2;
  `../domain/people.md`, 5.1). Whether a host's agent takes messages that
  large, and runs the engine's charters, is the fleet's to check at its
  hello, never configuration's (`../domain/engine.md`, section 8).
- **The application's limits** are its own members, checked in the same
  sum.

## 7. Secrets

| Secret | Kind | Configured as |
|---|---|---|
| a worker's secret | shared between the engine and that worker | on the worker, a file holding it; on the engine, a file holding its digest, per worker |
| an LLM account's record | its tokens, as skein's accounts driver keeps them | a private file per account, written by a sign-in, loaded by the driver (`llm.md`, 5.4) |
| a handed-in LLM account's access token | an access token and when it lapses | a private file, filled by another program's login, read at start (`llm.md`, 5.4) |
| an LLM account's client secret, if its provider has one | a confidential client's | a file |
| the workers' listener's TLS key | a private key | a file beside its certificate |
| the clients' listener's TLS key; the secrets of the application's sign-in (an OAuth client's) | the application's server's | its own members' files; jig's part names none (`service.md`, section 7; `client.md`, section 8) |
| a connector's credentials | the application's | the application's files, or its own secrets' records |
| an application's credential, for runs' workspaces | the application's | its own member's source: a file, or an exchange it refreshes (section 3) |

- **Read at startup,** each within its bound, from a private file, with
  skein's `read_private_file` (skein's `shell.md`, 6.1): a file readable
  by anyone but the process's user, or reached through a link, refuses
  the start, naming it.
- **Held by the protocol layer,** never the domain: the domains name
  secrets by number (`../domain/engine.md`, section 12).
- **What changes while the engine runs** is kept by its owner, never
  written back to configuration: a rotated refresh token in the account's
  private file, by skein's driver; a minted credential's digest in the
  store's secrets' records (`store.md`, section 9).

## 8. First start, and every start

- **A deployment's first start** is the first with an empty store: the
  shell draws the deployment's id, and the core seeds owners and projects
  from configuration into its first commit (`../domain/engine.md`, 5.4
  and section 14).
- **Every later start** reads the deployment from the store. The
  configuration's projects are ignored; its rules, charters, templates,
  limits, hosts, endpoints and accounts apply as configured, and a
  narrowing reaches every later decision, as a policy's does
  (`../domain/authority.md`, section 6).
- **What the store holds and the configuration does not name** (a charter
  removed, a connector gone) is the domain's to judge at restart, as it
  judges a live set past limits lowered since (`../domain/engine.md`,
  section 6); what it decides is open (section 9). A template dropped
  is not among them: no task depends on one (3.1).

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
- **Accounts per run:** whether a run's models may use several
  accounts is `../domain/engine.md`'s (section 16).

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
- **A handed-in account's file** (`llm.md`, 5.4) is one JSON object:
  `access_token`, a string; `expires_at`, the Unix time in seconds at
  which it lapses; and `account_id`, a string, where the account's
  provider names one. Nothing else.
- **A template,** in the `templates` member (3.1), is one JSON object
  whose members are all required, `null` where it has none: `number`;
  `name`; `executor`, `{"charter": n}` or `"chosen"`; `contract`,
  `{"report": {"words": n}}`, `{"verdict": {"choices": [{"code": c,
  "words": n, "followups": n, "fields": [{"name": s, "max": n}]}]}}`
  (each choice's code, its words' most bytes, the most follow-up tasks
  it may propose, and each field it requires, with its value's most
  bytes: `../domain/tasks.md`, section 3) or `{"change": {"connector":
  c, "kind": k, "words": n}}`, a connector's result (an approval names
  a state its procedure gives, so no template makes one); `shallower`, the levels of
  delegation its authority is below the creator's role; `budget`,
  `{"amount": n}`, `{"as_allowed": cap}`, the cap a number or `null`, or
  `"given"`; `deadline`, `"none"`, `{"span_ms": n}` or `"given"`;
  `tracked`, `null` or `{"priority": n}`; `conversation` and `listed`,
  booleans; `retention`, `"kept"` or `"forgotten_once_read"`; `label`;
  and `grace_ms`, or `null`. jig's chat: `{"number": 1, "name": "chat",
  "executor": {"charter": n}, "contract": {"report": {"words": n}},
  "shallower": 1, "budget": {"as_allowed": null}, "deadline": "none",
  "tracked": null, "conversation": true, "listed": false, "retention":
  "kept", "label": 1, "grace_ms": n}`; jig's goal: `{"number": 2,
  "name": "goal", "executor": "chosen", "contract": {"report":
  {"words": n}}, "shallower": 1, "budget": "given", "deadline":
  "given", "tracked": {"priority": 1}, "conversation": false, "listed":
  true, "retention": "kept", "label": 2, "grace_ms": null}`, each `n`
  the application's (3.1). jig's defaults are written, never implied:
  the `templates` member holds each template the engine has, jig's
  among them, and an application drops one of jig's by leaving it out
  (3.1).
- **A policy's override of a template,** in its `templates` list, is one
  JSON object: `template`, its number; and `executor`, `budget`,
  `deadline` and `priority`, each in the template's form (`priority` a
  number), or `null` where the template's own stands:
  `{"template": 2, "executor": null, "budget": null, "deadline": null,
  "priority": 5}`.
