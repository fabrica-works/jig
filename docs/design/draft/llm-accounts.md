# LLM accounts and models

Draft, 2026-10-18. How an application on jig pays for its runs' LLMs and
chooses them: who holds an account, the sources by which a run may use
it, whose work it pays for, which account and which model each run
gets, and what skein, smith, jig and an application each own. It
applies `providers.md`'s rules, its section 7 among them, and when it
becomes design it replaces the accounts of `../protocol/llm.md`,
sections 5 and 6, and `../domain/engine.md`, section 12. Section 10
names the documents that change, section 11 the order of work, and
section 12 what is open.

## 1. In one page

- **Each application is its own client, with its own state.** It signs
  in under its own name, keeps its records in its own store, and
  refreshes them itself. No application reads another program's login
  or shares a file with one: not Codex's, not a person's `~/.claude`,
  not another application's (section 2).
- **An account is a holder, its sources and its uses.** The holder is a
  person or the deployment; the sources, in the holder's order, say how
  a run may use the account; the uses say whose work it pays for
  (section 3).
- **A source is how a run is authorized:** signed in, with the
  application's own registration; a key; a program's own login on a
  host, such as Claude Code's; or none, for a local model. A new
  provider is configuration, and a dialect or a program's adapter where
  it needs one, never a change to the domain (3.4).
- **A person may hold several accounts with one provider,** each its own
  account, told apart by the provider's subject, never by an email; a
  subject has one holder in a deployment (3.3).
- **One person, no friction.** A plan pays, by default, for its holder's
  own work and for the automatic work of each project in which its
  holder is the only party, so a person alone on a deployment signs in
  once and every run uses their subscription (section 4).
- **What a task runs on is the task's:** its model, effort and the
  models its sub-agents may use, given by its template, by whoever
  creates it, a planning agent's `delegate` included, and changed by an
  amendment, within an authority that names the models a subtree may
  use. The charter says what the agent is, and which models it allows
  (section 5).
- **Who pays is decided with the run's claim:** the account that serves
  the work's payer, and its first source usable on a host with room. A
  task keeps its account from run to run, chooses again when its payer
  changes, and waits for it at a limit, never moving to another
  (4.3).
- **Credentials live in the store.** Records and keys are in the
  secrets' store, kept before they are lent, and never in a committed
  request or answer; the domain names each by number. The engine alone
  refreshes, so no two processes race a rotating token (section 7).
- **Moving an account, later:** between deployments of one application,
  never copied (section 8). The first build has no move.
- **Each layer owns one thing:** skein the wire and OAuth's mechanics,
  smith the agent and its backends, jig the accounts' policy, the
  application its name, its providers' data and its screens (section
  9).

## 2. Each application is its own client

- **Its own registration.** An application signs a person in under its
  own name: Sign in with ChatGPT's `agent_name_hint` is the
  application's name, used consistently (`providers.md`, 2.2). The
  installation's identifier that OpenAI asks for, `ext_agent_host_id`,
  is drawn once per deployment, at its first start, as `urn:uuid:…`,
  and kept in its store; it is never a person's identifier.
- **Its own records, refreshed by itself.** A sign-in's record and a
  key are kept in the application's store (section 7), and its engine's
  accounts driver alone refreshes a record (skein's `oauth.md`, section
  6), so refreshes of one record are serialized by construction
  (`providers.md`, 2.3).
- **No borrowed login.** No application reads another program's login
  file, a Codex login or another application's records. A program the
  agent runs (3.4, *program*) keeps its own login in a configuration
  directory of the application's own, per source and per host
  (`providers.md`, 7.3), never the person's: a person's own Claude Code
  settings, hooks and plugins never load into a run.
- **smith's own product is an application in this sense:** it signs in
  under its own name, keeps its records in its own state, and borrows
  nothing (smith's `shell.md`, section 7).

## 3. Accounts

### 3.1 What an account is

```
Account
  number      the domain's name for it
  provider    a provider of the configuration's, by number
  holder      a person, or the deployment
  subject     the provider's stable id for the account, where a source gives one; a display name beside it
  sources     in the holder's order: each a kind, its state, a name for a program's, and per host for a program's (3.4)
  uses        the payers it pays for (section 4)
```

- **A provider** is configuration: its number, its name for showing,
  its endpoints and models, the sources it offers and what each needs
  (a sign-in's registration, a program's adapter), and the facts its
  terms ask an application to show, such as a manage-usage address
  (`providers.md`, 2.6). jig's code names no provider.
- **Accounts are made at runtime,** by a person's sign-in, a key added
  or a program's source declared (section 6), and kept in the store. A
  provider whose source is *none* has one account, the deployment's,
  made at the first start that configures it. An application's
  configuration names providers, never accounts.
- **The domain sees names and states,** never a value: an account by
  number, a credential by account and generation
  (`../domain/engine.md`, section 12).

### 3.2 Holders

- **Only a person holds a plan:** a signed-in subscription or a
  program's login. A subscription serves one person (`providers.md`,
  section 1).
- **A person or the deployment holds a key:** a person's own, or the
  deployment's, which an owner adds, billed to the organization that
  owns it (`providers.md`, 2.7 and 3.1). The deployment holds a *none*
  provider's account.
- **A holder decides about their account:** the order of its sources
  and its place among their accounts, its uses, signing it out,
  removing it, and later moving it. An owner decides for the
  deployment's.

### 3.3 Several accounts with one provider

- **Each its own account.** A person may hold several subscriptions
  with one provider, under several emails, and each sign-in to a new
  subject makes a new account. A sign-in to a subject the person already
  holds renews that account's record; where the provider issues the
  renewal another client id, the replaced refresh token is revoked.
- **Told apart by the subject,** the provider's validated id (an ID
  token's `sub`), never the email: two registrations stay apart even
  when their emails match (`providers.md`, 2.2).
- **A subject has one holder.** A sign-in whose subject another holder
  holds is refused, naming the provider's rule, and its new record is
  revoked: two holders would share one subscription, and race one
  rotating token.
- **A program's logins** are one configuration directory per source on
  each host, named by the source's name (3.4), each signed in on its
  own, with its own email, so one host may carry several Claude Code
  logins of one person.

### 3.4 Sources

| Kind | What the application holds | Who runs the agent's loop | Usable where | Renewed by |
|---|---|---|---|---|
| *signed in* | a record from its own OAuth registration, in the secrets' store | smith, over skein's LLM client | on any host: its access token goes with the run as a grant | the engine's driver, before it lapses |
| *key* | a key, in the secrets' store | smith, over skein's LLM client | on any host, the same way | nobody: its holder replaces it |
| *program* | nothing: a program's own login, in the application's configuration directory for the source on each host | the program, adapted by smith's agent (`providers.md`, section 7) | on a worker whose host's half reports its login usable | the program |
| *none* | nothing | smith, over skein's LLM client | where its endpoint is reachable, such as a local model on loopback | — |

- **Later kinds fit the same table,** such as an exchange of a cloud
  provider's credentials for short-lived tokens.
- **The holder orders an account's sources.** A run takes the first
  that is usable on the host it is placed on, and passes to the next
  only when the earlier one is unusable there: a program's login on
  another host. A source that is limited or refused (3.5) holds the
  account's runs until its holder resumes it or chooses another source,
  since the order decides who pays (`providers.md`, 2.5 and 7.2). An
  Anthropic account lists Claude Code, then a key, by default.
- **A grant names its source.** A value's grant carries its value
  (`../protocol/llm.md`, section 6). A program's grant carries none: it
  names the source, by whose name the host's half finds the program's
  configuration directory, and the agent checks the login again as the
  run starts (`providers.md`, 7.2).
- **A program's source on each host.** The engine tells each worker the
  program sources it may be given, by name. The host's half checks each
  login at the hello, when told of a new source, on its holder's
  `check` (section 6) and at a configured interval, and reports what
  changed; the fleet places a run on a program's source only where its
  login is usable. The engine's own slots run `smith-inline-agent`,
  which has no program backend (`providers.md`, 7.1), so they never take
  a program's source.
- **A program source's name** is 1 to 32 characters of `a-z`, `0-9` and
  `-`, given by its holder as they declare it, unique in the deployment,
  never changed and never reused once removed. It, not the account's
  number, names the configuration directory, so that a directory a
  person signed in on a host keeps meaning that source, and a
  developer's own fixed directory serves each fresh deployment a live
  test makes.
- **Any number of program logins per host,** one person's several
  subscriptions or several people's. Runs, and Claude Code's own tools,
  run as the host's user, so a run can read every login kept on its
  host until contained trees give each run a view without the programs'
  directory (smith's `domain/tools.md`, sections 1 and 5). That is
  accepted (section 12).
- **Signing out or removing a program's source** has each host's half
  run the program's own sign-out with its directory (`claude auth
  logout`); the answer names any host that did not.
- **Concurrency is the host's slots:** a run on a program is a smith
  run, and counts as one run, as any other does.

### 3.5 States

A source's state is one of:

- **usable;**
- **limited:** the provider says it is spent, with when it resets where
  the provider says, and never a guess (`providers.md`, 2.5). No new run
  starts on it until then, or, with no time given, until its holder
  resumes it;
- **refused:** the provider refuses the account for what is asked: not
  eligible, a scope not granted, plan usage declined at sign-in. It
  stays so until its holder acts: signs in again, with the scope, or
  chooses another source;
- **needs sign-in:** no record, or one revoked or past refreshing; for
  a program, on which host;
- **misconfigured:** the provider rejected the application's client;
- **unloaded:** its record could not be read, saying where and why.

Each source's mechanism maps its provider's failures onto these, skein
for what a provider's API answers and smith for what a program reports
(section 9). A failure a run meets is the run's: unavailable is retried
by smith's session; unsupported, a field the provider will not take, is
the request's error, never sent again unchanged (`providers.md`, 2.5).

### 3.6 Bounds

Accounts in the deployment, accounts per holder and sources per account
are limits (`../protocol/configuration.md`, section 6), within which
the accounts' table, skein's driver, the hub's grants and the secrets'
store are sized. A request that would pass one is refused, naming it.

## 4. Uses: whose work an account pays for

### 4.1 The payer

A run's task has one funder, and the chain of funders ends at a party's
pool or a project's period (`../domain/authority.md`, section 7). That
end is the run's **payer**: the party who started the work, or the
project whose automatic work it is.

### 4.2 Uses

- **A plan, by default,** pays for its holder's own work and for the
  automatic work of each project in which its holder is the only party
  with a role. A person alone on a deployment is the only party of
  every project, so everything runs on their subscription, with nothing
  to set.
- **A plan's uses may name more,** in its holder's word: a service, as
  an agent of theirs that connects with its own identity does
  (`../domain/people.md`, section 3), whose work then runs on its
  holder's subscription; or less than the default. A person other than
  its holder, and a project with another party, are refused in a plan's
  uses, naming the provider's rule (`providers.md`, 2.6 and 3.4).
- **A key's uses** are its holder's own work by default, a person's; the
  deployment's key pays for every payer. A key's holder may name any
  party or project in its uses: a key is the customer's to provision for
  its own authorized users, billed to the key's owner (`providers.md`,
  3.1).
- **A shared project's automatic work** is so paid by a key: the
  deployment's, or a person's whose uses name the project. No plan pays
  for it (above).
- **An agent that brings its own LLM** pays for its own completions;
  jig's accounts are not involved.

### 4.3 Choosing the account

- **With the run's check.** As the core checks that a task's run may
  start (`../domain/authority.md`, 8.3; `../domain/engine.md`, 7.1, step
  1), it takes, for its main model's provider, the accounts whose uses
  cover the payer: the payer's own, in their order, then those that name
  it, then the deployment's. It takes the first with a source usable on
  a host with a free slot of a kind the charter allows, and the claim's
  commit records it (7.1, step 3).
- **The task keeps it,** from run to run, so a conversation stays on one
  account's cache and replay. Its creator names an account when it is
  created or amended (5.1) only to want one in particular, within the
  account's uses.
- **A change of payer chooses again.** When a commit changes a task's
  payer (a move to another requester, a widening or an acceptance
  funded by another party: `../domain/tasks.md`, section 7;
  `../domain/authority.md`, section 7) or its main model's provider, its
  next run's check takes the kept account only if its uses still cover
  the payer, and chooses again otherwise.
- **A task whose account is unusable waits** for it, saying why
  (`../domain/authority.md`, 8.3). A limited task never moves to another
  of its holder's accounts: it waits for its own, as its provider's
  limit asks, and an amendment naming another account of that provider
  is refused while its account is limited (`providers.md`, 2.5 and
  3.4).
- **Its sub-agents,** on the direct backend, use the run's account for
  the run's provider, and take an account the same way, at the run's
  check, for a sub-agent's model of another provider. On a program's
  backend, sub-agents are the program's own (`providers.md`, 7.4).

## 5. Choosing the model

### 5.1 The task's choice

```
LLM choice
  main        a model, and an effort among those the model declares
  sub-agents  the models the run's sub-agents may use, each with an effort
  account     optional: one account, within its uses
```

- **Given by whoever creates the task:** a template's default, a
  party's `Create`, an agent's `delegate`, a procedure's step, an
  accepted proposal. A task that gives none takes its charter's
  default: the first of the charter's models, in its order, whose
  provider has a usable account serving the payer.
- **Changed by an amendment,** like any part of a task
  (`../domain/tasks.md`, section 7): the next run uses the new choice,
  and a run in flight keeps its own.
- **Within authority.** Authority gains *models*: the models, and so
  the providers, a subtree may use, narrowed on delegation as its other
  parts are (`../domain/authority.md`, sections 3 and 5), under a role's
  ceiling; every configured model by default.
- **The charter says what the agent is:** its instructions, tools,
  hosts, waiting and resuming, the models it allows, or every
  configured one, in order, and its default (above).
- **Placement filters by model.** A host's hello states the models its
  agent declares (`../domain/engine.md`, section 8), and a run is placed
  only on a host that declares its task's models; a hello is refused for
  a model it declares too small, never for one it does not declare.
- **The budget takes the task's main model:** a run's check holds its
  reserve and one of that model's most costly completions
  (`../domain/authority.md`, 8.3; `../protocol/configuration.md`,
  section 6), and smith reserves each completion's most before it makes
  it, so cost needs no rule of its own. A run on a program's backend,
  where smith cannot reserve a completion (`providers.md`, 7.6), holds
  one turn's most in its reserve, and an overrun within it is recorded
  as spend (`../domain/authority.md`, section 7).

### 5.2 A planning agent's choice

- **`delegate` names each task's choice,** as a party's `Create` does,
  so an agent that plans a tree may give each task the model and effort
  its work needs, by capability and cost.
- **Its brief offers the catalogue:** each model its authority allows,
  with its provider, its window and output, its efforts, its prices,
  what it is good at (a description the configuration gives), and
  whether an account that serves this work is usable now, or limited, or
  needs a sign-in (`../domain/engine.md`, section 9).
- **Its sub-agents' models** are smith's, chosen within the run among
  those the task's choice lists (smith's `domain/run.md`, 5.3).

### 5.3 Changing model or provider within a task

- **The same provider and account:** a conversation resumes as it is.
- **Another provider:** smith carries the conversation over, keeping
  its text, calls and results and dropping what is the provider's own
  and opaque: reasoning, signatures and item ids (skein's `llm.md`,
  section 2, refuses another provider's replay). The same holds between
  the direct backend and a program's, which smith's transcript joins.
- **Another account of one provider:** whether a provider's opaque
  replay crosses its accounts is open (section 12); until a capture says
  so, smith drops it as for another provider.

## 6. What people do

Each is a party's request through the client (`../domain/people.md`,
5.1), decided by the accounts child; each answer and each state is a
fact the application's pages show (`../client/domain/contract.md`,
section 5).

| Request | What it does |
|---|---|
| sign in | starts a provider's sign-in; the visit goes to the client that asked, and the redirect to the engine's loopback listener; makes an account, renews one of the same subject, or is refused for a subject another holder holds (3.3) |
| sign out | revokes the record at the provider, or signs a program's login out on each host, and removes the source; when revocation is not confirmed, the answer says to disconnect the application in the provider's settings (`providers.md`, 2.3) |
| add a key, replace it, remove it | adding names the account it joins, a new key-only account by default; a person's, or the deployment's by an owner |
| declare a program's login | makes the account, or adds the source to one, under the name its holder gives (3.4); each host then reports whether its login is usable, and the answer names the command to run there |
| check | has each host's half check a program source's login now (3.4) |
| order, uses, resume | the holder's: an account's place among theirs and its sources' order, its uses, a limited source resumed |
| move out, move in | later (section 8) |

- **Values never enter a request's record.** A key, and later a moved
  record, is taken out by the clients' protocol layer as it decodes the
  request, kept in the secrets' store first, and the request names it
  by number, as a minted credential's digest is kept before it is used
  (`../protocol/store.md`, section 9). Requests and their answers are
  committed and kept (`../domain/engine.md`, 5.4); a value is not.
- **The sign-in's redirect is on the engine's machine.** A provider that
  requires a loopback redirect (`providers.md`, 2.2) is signed in from a
  browser on the engine's machine, or, later, through a move (section
  8).
- **Plan usage declined** keeps the sign-in and marks the source
  refused, and the answer offers its two ways on: sign in again with
  the scope, or another source (`providers.md`, 2.2).
- **The terms' disclosures are facts:** whether an account is a plan,
  its provider's name and manage-usage address, the first use of a
  plan, so that the application can show them where its pages need them
  (`providers.md`, 2.6). The words and screens are the application's.

## 7. Where the values live

- **The secrets' store** (`../protocol/store.md`, section 9) keeps each
  signed-in source's record and each key, under the account's number,
  beside minted credentials' digests. A record holds what skein's driver
  keeps: the tokens, the client id the provider issued, the scopes
  granted and the ID token, for signing in again (`providers.md`, 2.2).
- **Kept before lent.** skein's driver takes the owner's store as its
  keeper (skein's `oauth.md`, 6.4): each new record goes out as a keep,
  the routing writes it to the secrets' store, and the driver lends its
  token only once it is durable.
- **Every signed-in source is wanted,** from the restart's first step to
  the close, so the driver refreshes each before it lapses whether or
  not a run holds it, and an idle account keeps its refresh token
  (`providers.md`, 2.3; skein's `oauth.md`, 6.3).
- **The accounts child's records** are in the store: each account's
  holder, provider, subject and display name, sources with their names
  and states but per host, order and uses. A host's program login is
  not stored: hosts report it.
- **On each host,** a program's configuration directory per source,
  under a directory the agent's configuration names, by the source's
  name (3.4). It is the program's, which keeps its login there; jig
  never reads it.
- **Configuration holds none of it.** No account, no key, no record and
  no handed-in file; an endpoint names no account, and a run's charter
  names a grant for each of its LLMs.

## 8. Moving an account

Later: designed here so that the first build leaves room for it, and
built after the first build.

- **Out:** its holder asks. The account becomes *moving*, unusable, and
  its grant is released at the driver, so that no refresh follows; live
  runs keep their access token until it lapses. Its record goes to the
  client that asked, from the secrets' store, never in a committed
  answer, and the client writes it to a private file and says so; the
  engine then erases it.
- **A cancelled move,** when the client never says so, refreshes the
  record once before it is lent again, so that a file written meanwhile
  holds a dead refresh token.
- **In:** a client sends a moved record, taken into the secrets' store
  as a key is (section 6); the engine checks that it is its
  application's (the registration's name and its provider) and that no
  other holder holds its subject (3.3), and makes the account or renews
  the one of its subject; the client then removes the file.
- **Signed-in sources only.** A key is added again; a program's login is
  per host.
- **One holder at a time.** A rotating refresh token is valid in one
  place: a copy that refreshes signs the other out (`providers.md`,
  2.3), so a record is moved, never copied.
- **It serves three needs:** a host with no browser, signed in on a
  computer with one (`providers.md`, 2.2); a deployment moved to another
  machine; and unattended live tests on a signed-in subscription, which
  move one account in and out of each run's fresh deployment. Until
  then, a live test on a signed-in subscription signs in as it starts,
  and one on a program's login finds it signed in on its host (3.4).

## 9. Who owns what

**skein: the wire and the mechanics, with no policy and no catalogue.**

- **Dialects** (`llm.md`): OpenAI's public Responses route
  (`api.openai.com/v1/responses`), `store:false` and `stream:true`, with
  none of the fields `providers.md`, 2.4 refuses, for a plan's token and
  an API key alike; Anthropic's Messages with a key. A provider's API
  that is not one of these is a dialect of its own.
- **Credentials say how they are sent:** a bearer, or a key, which the
  dialect puts where its provider takes it (`x-api-key` for Anthropic).
  A credential carries no account id.
- **Failures** carry the classes of 3.5 that a provider's API answers:
  exhausted, with a reset only where given; refused; unsupported, naming
  the field; unauthorized; and the provider's error code and request id
  as evidence (`providers.md`, 2.5).
- **OAuth** (`oauth.md`): a client id the provider issues at the
  callback, kept with the record and used after; parameters a
  registration adds to the authorization request; `resource`; `nonce`
  and the ID token's checks, its signature against the provider's
  published keys; scopes that must be granted; revocation; the error
  names a registration adds to "sign in again".
- **The driver** (`oauth.md`, section 6): accounts added and removed
  while it runs, within its limit; sign-out; the owner's store as
  keeper.
- **A lock on a private record** (`io.md`, 5.3), for an application
  whose several processes share one record, as smith's product's do.
- **Its fakes:** the fake issuer speaks a registration that issues its
  client id, and the fake LLM peer the public route, keys and a plan's
  errors.

**smith: the agent, its backends, and its own product.**

- **One agent process, two backends** (`providers.md`, section 7):
  direct, over skein's client with a value's grant; a program's,
  adapting Claude Code first, behind the same parent face, with a fake
  `claude` for the worlds.
- **The host's half** checks each program source's login, with its
  configuration directory, as 3.4 says, and the agent at each run's
  start.
- **Grants** carry a value with how it is sent, or name a program's
  source (its `protocol/channel.md`, section 6).
- **Notices:** limited, with or without a reset; refused; unauthorized;
  up to the host (its `domain/host.md`, section 7).
- **Carrying a conversation over** to another provider or backend (5.3),
  in its transcript.
- **Its product** signs in to ChatGPT under its own name, its `openai`
  preset carrying smith's registration (its `shell.md`, section 7);
  serializes its records' refreshes across its own processes, each
  refresh under the record's lock, the record read again once the lock
  is held; takes keys from its settings or the environment; runs Claude
  Code with its own configuration directories; and borrows nothing. Its
  live tests and benchmarks run on its own sign-ins and Claude Code
  directories, in a credentials directory the developer signs in once,
  or on keys.

**jig: the policy.**

- **The accounts child:** holders, sources, states, uses, several
  accounts per provider, the choice with the claim, and later moving;
  requests from parties (section 6).
- **Tasks' LLM choices** and authority's models (section 5); the
  brief's catalogue.
- **The fleet:** program sources' logins per host, and placement by
  them and by models.
- **The secrets' store** keeping records and keys; the installation's
  identifier.
- **Configuration:** providers, their endpoints, models, sign-ins'
  registrations, programs and terms' facts; the accounts' bounds.
- **The client's facts** for accounts and disclosures.

**The application: its name, its providers and its screens.**

- **Its name** in each registration, and its providers' data in its
  configuration, written out.
- **Its pages:** signing in, the disclosures in its provider's words,
  limits and errors, model choices.
- **A command that hands the terminal** to a program's own sign-in, with
  the source's configuration directory on that host (`providers.md`,
  7.3), named for what it does, never after the program (`providers.md`,
  3.2).
- **Its live tests,** on a program's login, a sign-in made as the test
  starts, or, later, a moved account.

## 10. Documents that change

- **skein:** `llm.md` (sections 1, 2, 2.5, 4.6, 4.7), `llm-connection.md`
  (credentials, sections 1 and 4), `oauth.md` (sections 1, 2, 4 and 6),
  `io.md`, 5.3 (a private record's lock), `fake-llm.md`.
- **smith:** `protocol/agent.md`, 1 and 4 (the program's backend; an
  endpoint names no account), `protocol/hosts.md`, 3 and 5.4 (the
  program's check; `borrow` removed), `protocol/channel.md`, 4, 5 and 6
  (sources reported, notices, grants), `domain/host.md`, 7 (sources and
  notices), `protocol/transcript.md` (carrying over), `shell.md`, 4.5
  and 5 to 7 (its product), `testing.md`, 2.3 and `benchmarks.md`, 10
  (live tests and benchmarks without a borrowed login).
- **jig, the domain:** `domain/engine.md`, 5.4, 7.1, 8, 9, 12 and 16;
  `domain/authority.md`, 3, 5, 6, 7, 8.3 and 8.4; `domain/tasks.md`, 3,
  4 and 7; `domain/people.md`, 3 and 5.1.
- **jig, the protocol layer:** `protocol/llm.md`, 1, 4 to 7 and 10;
  `protocol/configuration.md`, 3, 4, 6 to 10; `protocol/store.md`, 4.4
  and 9; `protocol/channel.md`, 4.3, 5.1, 5.2 and 8; `protocol/client.md`
  (requests carrying a value); `protocol/service.md`, 8 (the live
  layer); `protocol/README.md`, 8.
- **jig, the rest:** `README.md`, 8; `client/domain/contract.md`, 5, and
  `client/domain/pages.md`, 7 (accounts' facts and pages).
- **`providers.md`:** its section 4's disagreements are this draft's to
  remove; its section 5's questions on configuration directories, the
  licence, the ID token's check and the sources' default order are
  answered here.

## 11. Order of work

1. **The MIT licence** in each repository, a licence file and each
   manifest's `license`: what OpenAI's program asks of an open-source
   tool (`providers.md`, 2.1).
2. **skein:** the dialects, credentials and failures; OAuth's additions
   and the driver's; their fakes.
3. **smith:** grants, notices and reported sources; the program's
   backend, with its fake; carrying a conversation over; its product's
   sign-in and keys, its borrowed logins removed, its live tests and
   benchmarks moved to its own accounts.
4. **jig:** the accounts child as sections 3 to 7 say, records in the
   secrets' store, the requests, tasks' choices and authority's models,
   the brief's catalogue; handed-in accounts and private-file records
   removed.
5. **Each application:** its configuration, its pages and its live
   tests.

The first build serves one person: signed-in, key, program and *none*
sources, several accounts per provider, the default uses and tasks'
choices. Moving accounts, uses beyond the default and a client that relays a
redirect come after.

## 12. Open questions

- **Replay across accounts:** whether a provider's opaque replay (Codex's
  encrypted reasoning, Anthropic's thinking signatures) is accepted from
  another account of the same provider. Default: dropped (5.3).
- **A program's subject:** whether `claude auth status` names the
  account it is signed in to. Default: the holder's own display name for
  the account, and no subject check for a program's source (3.3).
- **Logins readable by every run on their host** (3.4): when a host
  serves several people, whether to wait for contained trees, or keep
  each person's logins on hosts of their own. Default: accepted, as for
  one person's several subscriptions.
- **Claude Code's built-in tools** (`providers.md`, section 5). Default:
  kept, each call that would prompt answered by smith's policy.
- **A redirect relayed by a client,** for an engine on another machine:
  the native client binds the loopback listener and hands the code to
  the engine. A remotely hosted application asks OpenAI first
  (`providers.md`, 2.1). Default: the engine's machine.
- **Models a provider lists** for an account (`providers.md`, 2.4), in
  the catalogue beside configuration's. Default: configuration's only.
- **What a model is good at:** a description in configuration, or
  structured capabilities a planner could filter on. Default: a
  description.
