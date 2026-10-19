# The client

Provisional, 2026-10-09, revised 2026-10-19. `jig-client`: the client an
application holds, as a component, to reach jig's engine. It holds the
link, the watches the application names by handle and the latest
document of each, the application's keyed requests in skein-lib's
request table, the work kept across a restart, and the application's own
families, carried as bytes. It knows no page and no HTTP: what a party
sees is the application's presentation, or jig's reference UI
(pages.md, objects.md, view.md); what the client says is jig's messages,
which the application's transport carries (`../../protocol/client.md`).
How an application's client root holds it is root.md; the API it speaks
is contract.md; jig's helpers are modules of it (section 19). What is
still open is listed in section 18.

## 1. In one page

- **A step machine the application holds** (programming-model.md,
  section 3): `jig_client::Client`, with its own vocabulary, limits,
  worst case and world. The application steps it with each event, and
  it gives back what to send and what to tell. It runs no loop of its
  own, and it never calls the application (section 4).
- **Presentation-agnostic.** It knows no page, place, card, confirmation
  or draft. A browser, a terminal, a chat bot, a one-shot command and a
  service with no presentation hold the same client (section 20). jig's
  reference UI is one presentation, built on the public client alone
  (pages.md, view.md).
- **Watches by handle.** The application names each watch and read by a
  handle. The client keeps the latest document of each watch, and says
  when it changed; the application reads it when it likes. Several
  handles on one subject share one watch on the engine (section 8).
- **Keyed requests, through skein-lib's request table.** Each request (an
  operation, a compound request, or a family's request as bytes) is kept
  in the small store before it is sent, sent again under its key until
  its answer is durable, and answered to the application once
  (section 6).
- **Work kept across a restart.** The small store holds the requests not
  yet answered, and nothing else of the client's. At start, each is
  restored and told to the application as kept (section 11).
- **The link is the client's to judge,** from the stream and its own
  watch of `me`: signed out, starting, live, offline, or out of date
  (8.2).
- **Signing in by `SignIn { with }`:** a printed link's secret, a
  service's credential, a credential the application obtained by its own
  sign-in, or one its transport carries unseen. jig owns credentials;
  every sign-in flow beyond its two is the application's (section 12).
- **The application's families travel as bytes,** under their owner's
  number and version, on the same connection and through the same
  table. The application's wire crate encodes and decodes them; the
  client never reads them (sections 4 and 6; root.md, section 7).
- **jig's messages, not HTTP.** The client gives jig's messages to a
  transport the application chooses: jig's default HTTP mapping, a local
  socket, or an in-process channel (section 16).
- **Built for the host and for wasm32,** as step code under the strict
  subset (programming-model.md, section 10).
- **Helpers beside it, optional:** shapes of work from templates, facts
  over documents, a conversation's thread, and jig's words, each built
  only from the public client (sections 10 and 19).

## 2. The client in the system

```
presentation   the application's: routes, screens, words, look, confirmations, drafts; or jig's reference UI
helpers        jig's, optional: shapes, facts, thread, words (section 19)
jig-client     the link, watches by handle, the request table, kept work, families as bytes
transport      the application's: jig's default HTTP mapping, a socket, or in-process (section 16)
the API        jig's messages: operations, subjects and reads, documents with revisions and offers (contract.md)
the engine     the application's root around jig's core, and jig's clients' component
```

- **Two translations, one per side.** Toward the engine, the client
  speaks jig's messages, encoded by `jig-client-protocol-link`. Toward
  the application, it speaks the API's typed values: operations,
  subjects, reads, documents and answers.
- **Nothing below the presentation knows a page,** and nothing of the
  client's knows HTTP. The helpers use the public client only, as the
  application does.
- **The client has no store of the truth.** The engine's store is the
  truth. The client holds the latest document of each watch, and a small
  store of what it must not lose across its own restart (section 11).
- **A party may have many clients:** tabs, devices, a command. Each is
  its own client, with its own keys and its own small store. Two of them
  deciding one thing are two requests to the engine: the first commit
  decides, and the second is told who decided and how
  (`../../domain/people.md`, 5.1).

## 3. Structure

```
jig-client                        jig_client::Client: the domain and the link, composed; the helpers as modules
├── jig-client-domain             the client's domain
└── jig-client-protocol-link      jig's messages and the small store's records, encoded and decoded; the stream's events
    ├── jig-client-protocol-wire  jig's wire records, for both ends (../../protocol/client.md, section 3)
    └── jig-protocol-transcript   turns decoded with smith's transcript crate (section 10)
```

- **`jig-client-domain`** is a step machine like every jig domain:
  `#![no_std]`, depending on skein-lib alone. It holds:
  - *the API's types:* operations, subjects, reads, documents and
    answers, as contract.md gives them;
  - *the link:* its state, the sign-in it is on, the stream (8.2 and
    section 12). The credential's value is never domain state
    (`../../protocol/README.md`, section 8): `jig-client` holds it
    beside the domain, as an engine's protocol layer holds its secrets,
    and puts it on each `Exchange` and `OpenStream`;
  - *handles:* each watch's and each read's, with its asker (section
    4), each watch mapped onto a watch of the engine's with its latest
    document (sections 7 and 8);
  - *the request table,* skein-lib's `RequestTable` (skein's `lib.md`,
    section 12), with each request's key and asker (section 6);
  - *kept work:* the requests restored at start (section 11);
  - *its timers and its seed,* from which the table's keys, the engine
    watches' names and every jitter are drawn.
- **`jig-client-protocol-link`** (`../../protocol/client.md`, 4.3)
  translates between jig's wire records
  and the domain's types, encodes and decodes jig's messages and the
  small store's records, judges the stream's hello against the versions
  the client speaks, and decodes turns through `jig-protocol-transcript`.
  It knows no HTTP.
- **`jig-client`** composes the two into `jig_client::Client`, one step
  machine, and holds jig's helpers as modules: `shapes`, `facts`,
  `thread` and `words` (section 19). It depends on `skein-ui` for its
  `words` module alone, whose phrases are skein-ui's phrasebook types;
  it builds no tree, so the client stays presentation-agnostic.
- **Builds:** the three crates and the helpers are step code under the
  strict subset, built for the host and for wasm32 (programming-model.md,
  section 10). Only `jig-protocol-transcript` links smith, the client's
  only use of smith (`../../README.md`, section 10).
- **Its handles inside are skein-lib's:** its watches, reads and
  exchanges are `Id`s in slabs sized at startup, and what crosses to the
  transport is their tokens (programming-model.md, 4.2). A terminal is
  never stale: a slot is retired only once its terminal has come.
- **Beside it, the application's to use or not:**
  `jig-client-protocol-http`, the default HTTP mapping's client end;
  `jig-client-shell-native`, a native shell over skein's HTTP client;
  and `jig-client-view`, the reference UI (README.md, section 2).

## 4. Its vocabulary toward the application

```
jig_client::Client: step(now, wall, Event, &mut Queue<Request>), fire(now, wall, &mut Queue<Request>), next_deadline()

Event, in
  from the application:
    Start { kept }                                the small store's records, as the host loaded them
    SignIn { with } | SignOut                     with: Link(secret) | Service(credential) | Held(credential) | Transport
    Watch { handle, subject, window } | Unwatch { handle }
    Read { handle, read, cursor }
    Send { key, name, request }                   an operation, a compound request, or a family's request as bytes; name optional
  from the transport:
    Reply { exchange, message } | Lost { exchange }
    Streamed { message } | StreamGone
    Stored | NotStored
Request, out
  to the transport:   Exchange { exchange, credential, message } | OpenStream { credential } | CloseStream | Save { records } | Erase { records }
  to the application: Link(state) | Changed { handle } | Ended { handle, why } | Page { handle, page }
                      Pending { key } | Answered { key, outcome } | Kept { key, request } | Dropped { count } | Text { handle }
read by the application: client.document(handle), client.text(handle), client.pending(key), client.link()
```

- **`now` and `wall`** are the monotonic instant and the wall time, as
  every step machine takes them (programming-model.md, section 9:
  `env.now` and `env.wall`).
- **Handles and keys are the application's.** Each names its asker and a
  number of the asker's. The asker is the owner number of the child of
  the client root that asked (root.md, section 15; jig's reference UI's
  is 65536): the client keeps it, hands it back with everything it says
  of that handle or key, and never reads it, as skein-ui keeps an owner
  (skein's `ui.md`, 3.3). The client root routes by it, and keeps no map
  from handles or keys to its children (root.md, section 5).

### 4.1 What it hears

- **From the application:**
  - *`Start { kept }`:* the small store's records, as the host loaded
    them, before any other event (section 11);
  - *`SignIn { with }` and `SignOut`* (section 12);
  - *`Watch { handle, subject, window }` and `Unwatch { handle }`:* a
    subject of jig's (contract.md, 7.1), or a family's as bytes, and the
    window its document holds (section 8);
  - *`Read { handle, read, cursor }`:* one document, or one page of a
    list from a cursor (section 7);
  - *`Send { key, name, request }`:* an operation, a compound request,
    or a family's request as bytes, with a name or none (section 6).
- **From the transport:**
  - *`Reply { exchange, message }`:* the one record that answers an
    exchange; *`Lost { exchange }`:* none came whole;
  - *`Streamed { message }`:* one event of the stream, in order: the
    hello, a delivery, a heartbeat, a watch ended; *`StreamGone`:* the
    stream ended;
  - *`Stored` and `NotStored`:* the last `Save` or `Erase` written, or
    not (section 11).
- **Time:** `fire(now, wall, ..)`, at the deadline `next_deadline()` gave
  (section 13).

### 4.2 What it asks for

- **Of the transport:**
  - *`Exchange { exchange, credential, message }`:* one of jig's messages
    (`../../protocol/client.md`, 4.1) with the credential held, or
    none; answered by one `Reply` or one `Lost`;
  - *`OpenStream { credential }` and `CloseStream`:* the one stream every
    watch's deliveries come on;
  - *`Save { records }` and `Erase { records }`:* the small store's,
    written before the requests they keep are sent (section 11).
- **Of the application,** told, never asked to act:
  - *`Link(state)`:* the link's state, whenever it changes, and when a
    sign-out does not reach the engine (8.2 and section 12): what a
    presentation shows of the link, which it reads with `client.link()`;
  - *`Changed { handle }`:* a newer document for the handle, or the
    handle behind or live again (8.1);
    *`Text { handle }`:* more of a run's text (8.3); *`Ended { handle,
    why }`:* the handle's one terminal, and why (8.1);
  - *`Page { handle, page }`:* a read's one terminal (section 7);
  - *`Pending { key }`:* the request's progress changed; *`Answered {
    key, outcome }`:* its one answer; *`Kept { key, request }`:* a
    request restored at start (sections 6 and 11); *`Dropped { count
    }`:* how many kept records were dropped at start, unread, under no
    asker (section 11).

### 4.3 What it never asks

- **It never calls the application.** What it tells is a value in the
  queue, which the application takes when it steps it. What it holds,
  the application reads:
  - `client.document(handle)`: the handle's latest document, and its
    state (8.1);
  - `client.text(handle)`: the run's text it holds for the handle, the
    one turn being written (8.3);
  - `client.pending(key)`: the request's progress, its party and its
    owner (section 6);
  - `client.link()`: the link's state, and the credential held (8.2 and
    section 12).
- **It never reads an application's value.** A family's request, read,
  subject, page and document are bytes under their owner's number, which
  the application's wire crate encodes and decodes; the client reads
  their owner, their version and their size, and nothing more. A part
  that one of jig's documents names stays a reference (root.md, section
  6).
- **It never decides for a presentation:** what to show, when to
  confirm, what to keep of a draft. It gives the facts as data: the
  documents with their revisions and offers (contract.md, 5.3), and each
  request's progress and answer.

## 5. Pages and the working set

Pages, places, navigation and the working set are the reference UI's,
`jig-client-view`: pages.md, sections 2 and 12; objects.md, section 2;
and view.md, 18.2 and 18.3. The client holds no object by key: it holds
the latest document of each watch, by handle (section 8).

### 5.1 Sections

Sections and arrangements are the reference UI's: pages.md, 12.2, 12.3
and 12.5. What a section reads and watches, it opens through the client's
handles, as every presentation does (sections 7 and 8).

## 6. Keyed requests

- **What a request is:**
  - an operation of the API (contract.md, 3.2): `Create`, `Propose`,
    `Withdraw`, `Message`, `Amend`, `Cancel`, `Release`, `Stop`, `Move`,
    `Decide`, `Answer`, `Take`, `HandBack`, `Seen`, or one of the
    project's own;
  - a compound request, `Compound { project, operations }`: operations
    admitted in order and committed in one decision, all or none, under
    one key and one answer (contract.md, 3.4);
  - a family's request: its owner's number, its scope, its version and
    its record's bytes (contract.md, 3.3).

  The client reads none of it but its owner, its version and its size
  (4.3).
- **The key is the application's name for a request:** its asker, and a
  number the asker draws from its seed, fresh for each request it means.
  A second `Send` under a key the client holds is joined to the first
  when it carries the same request, so a double press or a retry makes
  one request; with another request, it is a key conflict, a mistake of
  the application's, and loud (`../../README.md`, 6.5). The engine knows
  the request by its engine key, scoped to the party (contract.md,
  3.1): the key the table draws, or a name (next bullet). The client
  keeps it beside the application's key in the request's record, so a
  kept request comes back under its key (section 11).
- **A `Send` may carry a name:** an engine key the application gives,
  which the table admits in place of a drawn one (skein's `lib.md`,
  section 12). The request's record keeps it as the engine's key beside
  the application's key. Any ask may carry a name, a family's included.
  - *Held.* A named `Send` whose engine key the table already holds (the
    same ask under another application key, another ask, or a restored
    record of another asker) is answered held at once: nothing is kept
    and nothing is sent. The key stays held until its holder's record is
    erased; the application asks again then. Within the engine's
    retention the engine answers the same name and ask with its first
    answer: a request sent again within the retention gets its answer
    again (contract.md, 3.1).
  - *Never loud.* A name comes from outside the application (an agent's
    key), so a name held is never the application's mistake. The same
    application key with the same request is still joined, and with
    another request is still a key conflict, loud.
  - *Not joined.* The table keeps one asker per entry, and the engine
    already answers a repeated name with its first answer.
- **Through skein-lib's request table** (skein's `lib.md`, section 12),
  which the client holds. It is the client's only way out for a keyed
  request, as the journal is the engine's root's (`../../README.md`,
  6.2). It:
  - *keeps it before sending it:* the request's record leaves in a
    `Save`, and its first `Exchange` only once the transport says
    `Stored` (section 11);
  - *sends it again under its key,* after a backoff drawn from the seed,
    while its answer is busy, not ready, or lost with the link live; and
    *parks it* while the link is down or the party signed out;
  - *sends only the confirmed party's,* the party `me` names (8.2);
  - *retires it past the retention* `me` states (contract.md, 3.1): a
    request first sent longer ago than the retention, less a margin, is
    never sent again, and is answered unknown;
  - *retires it not kept* when the last write that carried its record
    is not stored, whatever its state, parked included (section 11);
  - *is bounded:* requests held, and their bytes, a family's included. A
    `Send` past them is answered at once, refused by the client's limit,
    and never queued.
- **Progress, for showing.** `Pending { key }` says the request's
  progress changed: in flight, retrying, parked (offline, signed out, or
  another party's), or held for its owner's version (8.2). Retrying
  says why: busy, not ready, lost, or a final answer that did not
  decode, which is how the client reports such an answer, once per
  answer (`../../protocol/client.md`, section 5).
  `client.pending(key)` reads it. Whether and how it shows is the
  presentation's.
- **One answer.** `Answered { key, outcome }` comes once per request:
  - *done,* with its outcome: what the engine made or decided, or, where
    another party decided first, who, how and when; for a family's
    request, its owner's outcome, as bytes;
  - *refused,* with its facts (contract.md, section 4): jig's, typed, for
    every request the people child refuses at admission, a family's
    included; the connector's own outcome, as bytes, for a family's
    request it refused;
  - *unknown,* the client's own: past the retention, whether the engine
    made it is unknown, and the presentation, which knows the request by
    its key, says where to look;
  - *not kept,* the client's own: its record could not be written, so the
    client never sent it (section 11).
  - *unconfirmed,* the client's own: sent before `me` named the party,
    so no record could say whose it is. It is answered at once, and
    nothing of it is kept or sent (section 11).
  - *held,* the client's own: the name the `Send` carries is held by
    another request. It is answered at once, and nothing of it is kept
    or sent (above).

  Done and refused come only once the engine's commit is durable
  (`../../domain/core.md`, section 11), so the client never says done
  before it is. Nothing a presentation shows as done may come before
  this answer.
- **Proposing is explicit.** A `Create` or an `Amend` beyond the party's
  authority is refused, naming what it lacked and whether a proposal
  could be accepted; `Propose` asks (contract.md, 3.5). Each document's
  offers say, for each act, made, proposable (and to whom) or refused,
  with bounds (contract.md, 5.3), so a presentation can say which before
  it sends. The client reads none of it: authority is the engine's,
  judged again when the request comes.
- **Decisions name what they saw.** An operation that decides about
  something that changes names its revision, from the document the
  application showed (contract.md, 5.2); the engine refuses one whose
  revision has moved, saying what it is. The client carries the
  revision and checks nothing.
- **One request per decision.** What must commit together is one
  request: an operation, or a compound request (closing a conversation
  with each live delegate cancelled or moved to the party; amending a
  held task and releasing it; reordering goals). The client never chains
  two requests, so nothing is left to undo when the second is refused.
- **A version not served** keeps the request: while its owner's version
  is not served (8.2), the request is kept and unsent, held for its
  owner's version, and it is sent under its key once a hello serves that
  version again. A client updated to another version of the owner drops
  it at start (section 11).

## 7. Reads and windows

- **A read** asks for one document (a task, a proposal, `me`, a
  family's) or one page of a list (a task query's lines, a task's
  activity or history, the inbox's group), and has one terminal, `Page
  { handle, page }`: the document, or the page with its cursors, or why
  there is none: unknown, gone, too large with its size, malformed,
  refused, busy, signed out, version not served, or unreachable, when
  its exchange is lost (gone, too large and malformed for a turn read
  alone, contract.md, section 6). A read is one attempt
  (`../../protocol/client.md`, section 6): the application reads again
  if it wants.
- **What is live is watched; what is past is read.** The engine holds
  live work only (`../../domain/engine.md`, 5.3), so ended tasks, history
  and transcripts are pages read from its store, a page at a time, as
  the application asks.
- **A window** is what a watch's document holds of a list, or a read's
  page: its size, its order and where it starts, by cursor. The
  application gives it; the engine bounds it in bytes and rows, never in
  rows sized for one client's screen (contract.md, 5.3, 6.1 and 7.5).
  Keeping rows across pages, as a list slides while a person scrolls, is
  the presentation's.
- **One read in flight per handle.** A `Read` on a handle whose read is
  in flight abandons that read, and so does `Unwatch` on the handle; an
  abandoned read's terminal is dropped when it comes. A full slab of
  reads answers the read at once, refused by the client's limit.
- **A family's read** names its owner, its scope (a project, the
  deployment, or the party's own) and its record's bytes (contract.md,
  7.5); its page is its owner's, as bytes.
- **A read of an owner not served** (8.2), a family's outside its
  owner's range or any read while the link is out of date, is answered
  at once and never sent: version not served, naming the owner and the
  range the last hello named of it, or none where it named none.

## 8. Watches and the link

### 8.1 Watches

- **A watch** follows one subject (contract.md, 7.1): `me`, a task, a
  tree, chips, the inbox, a task query, a run's text, the deployment, or
  a family's subject, naming its owner, its scope and its record's bytes
  (contract.md, 7.5). Its deliveries are documents, each the subject's
  whole current state within the window.
- **Level-triggered, as facts are** (`../../domain/core.md`, section 4).
  A watch never carries a change to apply to what came before, so it
  cannot miss one: what changes while a delivery is in flight is in the
  next. A slow client sees fewer deliveries, never a gap. The same holds
  toward the application: `Changed { handle }` says the latest document
  moved, and the application reads it with `client.document(handle)`
  when it likes; one that reads late reads the latest.
- **Handles share the engine's watches.** Several handles naming one
  subject and one window share one watch on the engine and one document.
  It opens with the first handle and closes with the last.
- **A watch waits for a sign-in.** A `Watch` while no sign-in is live
  (the link signed out, 8.2) is admitted, its handle opening, and
  nothing leaves for the transport: a stream opened with no credential
  ends at once (`../../protocol/client.md`, 4.2), and the client sends
  none unless the transport carries one (section 12). Once signed in,
  the client opens each such watch under a fresh name; one on `me` is
  the client's own (8.2), not a second.
- **Each opening is named afresh.** The client names each watch it opens
  on the engine with a name of its own, never reused, so a delivery for
  a watch it closed is dropped, even when the application has given its
  handle to another subject.
- **Counted until the engine has ended it.** Each of the engine's
  watches counts toward the client's limit (section 14) from the opening
  it sends until its terminal: refused, ended on the stream, its closing
  answered, or its stream dropped. The client closes a watch only once
  its opening is answered: admitted, it sends the closing; refused, the
  watch is over. A watch's opening or closing that gets no answer drops
  the stream (8.2), whose end ends every watch on it
  (`../../protocol/client.md`, section 7). So, but for a dropped
  stream's watches until the engine has ended them, the engine holds no
  watch of the client's that the client does not count, and a client
  within its limit takes no more of the party's room (contract.md, 7.2).
- **A `Watch` on an open handle** changes its subject or its window: the
  client opens the new one, keeps the handle's last document, marked
  behind, until the new one's first delivery, and closes the old once no
  handle names it.
- **A handle's states,** which `client.document(handle)` gives beside the
  document: *opening,* no document yet; *live;* *behind,* its watch
  dropped and opening again, or changed, so the document held may be
  old; *ended.* A handle that falls behind, or is live again, is told
  `Changed { handle }`, as for a newer document: level-triggered, the
  application reads both with `client.document(handle)`.
- **Dropped is not ended.** A watch the stream lost, one the engine let
  go as it stopped, or one it refused as busy, is opened again after a
  backoff, under a fresh name; its handle stays, behind, and is live
  again at its next delivery. Busy is the party's room full for now
  (contract.md, 7.2): the party's other clients, or a stream this client
  dropped, or its predecessor's (`../../protocol/client.md`, section 4),
  whose watches the engine has not yet ended, hold the rest, and it
  empties as their watches end.
- **`Ended { handle, why }`** is a handle's one terminal, for what opening
  again would not mend: the engine refused it (unknown, not allowed);
  its subject ended and left the engine; the party lost the right to see
  it; signed out; or its owner's version not served. `Unwatch`
  ends a handle with no `Ended`.
- **Bounded:** handles, the engine's watches, and the bytes of each
  document held (section 14). A `Watch` past them ends at once, refused
  by the client's limit. One of the engine's watches is `me`'s (8.2):
  while no watch on `me` is held, the application's take at most one
  fewer, so that a sign-in always opens it.

### 8.2 The link

- **The client watches `me` itself** while a sign-in is live, under a
  handle of its own, which an application's handle on `me` shares (8.1).
  `me` names the party, which the client confirms to the request table,
  so that only that party's requests are sent; and the retention of
  answers (contract.md, 3.1), from which the table cuts its retries
  (section 6). `me` ends only with the sign-in, or when jig's version is
  not served (below): a refusal of it, busy included, or any other end
  of it, is opened again after a backoff, as a dropped watch (8.1), and
  the link is offline until its next delivery.
- **Its states,** told by `Link(state)` and read with `client.link()`:
  - *signed out:* no sign-in is live (section 12);
  - *starting:* signed in, and `me`'s first delivery has not come;
  - *live:* the stream is live, and `me` has been delivered on it;
  - *offline:* the stream dropped, or went silent past twice its
    heartbeat (`../../protocol/client.md`, section 4), or `me` was
    refused or ended on it (above). Requests wait, parked, and what
    dropped, the stream and its watches or `me` alone, opens again after
    backoffs drawn from the seed: a first span, doubling to a most,
    jittered.
    `me`'s next delivery brings the link back, and with it the confirmed
    party's parked requests;
  - *out of date:* the engine serves no version of jig's that the client
    speaks. It sends nothing, and keeps every request.

  Beside its state, the link names the party confirmed, each declared
  owner whose version the last hello does not serve (below), and a
  sign-out that did not reach the engine (section 12).
- **Judged from the stream alone,** whatever the application watches. A
  request that got no answer while the link is live is sent again after
  a backoff, and does not by itself make the client offline.
- **Behind is per handle** (8.1): a presentation that says when what it
  shows may be old reads it from its handles.
- **Versions, per owner.** The stream's hello states the ranges the
  engine serves, jig's and each application owner's
  (`../../protocol/client.md`, section 9). The client speaks the one
  version of each owner its build holds: jig's, its wire's own; and each
  family owner's, which the application declares as it makes the
  client, with its limits (section 14): each owner whose families it
  sends, by its number, with its wire's version (root.md, section 15).
  Each family's request, read and subject names a declared owner, in
  its declared version; any other is the application's mistake, and
  loud, as a key conflict is (section 6).
  - *Judged at each hello, whole.* A declared owner is not served when
    the hello names no range of it, or one that leaves out its version.
    The judgement holds until the next hello, so `client.link()` names
    an owner not served from the hello on, before the application has
    asked anything of it. The client keeps, for each declared owner, its
    judgement and the range the hello named of it; of an owner it does
    not declare, nothing.
  - *An answer version not served,* naming the owner and the ranges
    served, says the same of one message.
  - *Outside a range of a family's owner,* that owner's requests stay
    kept and unsent, held for its version (section 6); its reads are
    answered at once, unsent (section 7); its watches end, none opened
    again at that hello; each as version not served. Outside jig's, the
    link is out of date, and its reads are answered so too.
  - *Served again* by a later hello, the client sends what it kept,
    under its keys. Updated, it sends what it kept in a version its new
    build speaks, and drops what it kept in another (section 11).
    `jig-client` and the engine come from one pin of jig in an
    application; how jig is released and pinned is `../../README.md`'s,
    section 14.
- **A restart of the engine** is a moment offline, then answers by key.
  Provisional text streamed before it, never committed, is dropped when
  the next delivery comes (8.3).

### 8.3 A run's text

- **The one stream.** A watch of a run follows a task's live run: its
  attempt; the turn being written, named by its attempt, its number
  within it and its completion's retry; that retry's text so far; and
  the call it is making; all provisional, and marked so (contract.md,
  7.3).
- **Read from an offset, one turn at a time.** The client keeps the
  text it holds of the turn being written, within the bound on a
  provisional turn's bytes, takes each delivery's text past its offset,
  and says `Text { handle }`. It holds that one turn alone, the newest
  delivered: a delivery naming a later turn, or a later retry of the
  same turn, takes the place of what it holds, from its own offset; one
  naming an earlier turn, or an earlier retry, changes nothing, and
  nothing is told. Names are ordered by attempt, then number, then
  retry.
- **Text of a later turn never resets an earlier one's.** A later turn's
  text starts under its own name: it is never appended to the earlier
  turn's, nor taken as that text cut back to its start, and what the
  application read under the earlier turn's name stays that turn's.
  Keeping each turn's text until its committed turn takes its place is
  the thread's (section 10), fed the run's text at each `Text`.
- **Opened again,** the watch starts from the beginning of the turn being
  written: the client keeps what it holds, and takes what lies past it.
- **Committed turns are not in it.** A turn reaches the application
  committed only through its task's activity, read once the task's
  document says the activity has moved (contract.md, 7.3).
  `jig_client::thread` puts the two together (section 10).
- **Kinds.** The wire names a delivery's kind: `text`, `reasoning` or
  `input` (protocol/client.md, 7.3). `client.text(handle)` is kind `text`
  only, the one the engine sends; the others are held the same way, by
  turn, retry and offset, once a producer exists, and no surface for
  them is added before.
- **Only for a run asked for it.** A run whose task is not a conversation,
  or whose host names no call, has no text or no call respectively, and
  its words appear as each turn commits.

## 9. Drafts and confirmations

Drafts and the confirmation are the presentation's: the application's
own, or the reference UI's (view.md, 18.5; objects.md, section 2). What
a confirmation says is data in jig's documents (contract.md, 5.3 and
5.4); what holds below it is the revision a decision names and the
request table's once (section 6); jig's check is testing.md's, 6.2.

## 10. Conversations

`jig_client::thread`, a helper (section 19): a conversation's thread,
for any presentation of a chat or of any agent task's activity.

- **A model the application holds,** `Thread`: step code with limits of
  its own, fed what the client gives: the pages of the task's activity
  read (section 7), the task's document and its run's text from their
  handles (8.1 and 8.3), and the task's pending `Message`s with their
  answers (section 6), each alone or at its place in a compound
  request, whose answer numbers its words by that place (contract.md,
  3.4). It merges, in order, committed turns, the
  party's words, work summaries, proposals and provisional text, across
  windows and gaps. It sends nothing and reads nothing itself.
- **Committed, read in pages from the end.** A task's activity is a list
  the engine keeps in the order it committed: the words parties wrote to
  it, each run's beginning and end, its turns, the results and news it
  heard, the decisions made about it (contract.md, section 6). The
  application reads it a page at a time, newest first, as it needs
  older lines.
- **Turns, decoded below the client's surface.** A turn is bytes of
  smith's vocabulary, which the engine never reads
  (`../../domain/engine.md`, 7.2). `jig-client-protocol-link` decodes it
  with `jig-protocol-transcript` into the client's own turn: its
  messages; their blocks of text, calls and results, each call named by
  its tool with its input as written and its outcome summarised; what
  it used and spent. A call to the engine carries, as its objects and
  parts, what the core wrote beside the turn under its name
  (`../../domain/engine.md`, 7.2), never what its answer's words say.
  Neither the application nor the helpers link smith.
- **A job is a call's outcome,** worded like any other call's. A call
  that answered `running` reads "started as job N", never "is running":
  a job ends with its activation (smith's `protocol/transcript.md`,
  section 4). A `job` call names its job by number and what it
  answered. `JobsFull` and `UnknownJob` read as invalid calls. A call
  not run reads with its why: stopped, outside the choice, or after a
  failed write. The turn's count of jobs is not shown.
- **A turn that does not decode** reaches the application as such, with
  why: another version of smith's transcript (naming the one found),
  malformed, or past the client's bounds (`../../protocol/client.md`,
  section 3). It is a line of the thread, never dropped, and no other
  turn is read in its place.
- **Live, then committed.** While a run is live, its text is the
  thread's provisional line, kept under the turn's name until the
  committed turn of the same attempt and number takes its place: the
  thread keeps each turn's line, since the client holds only the turn
  being written (8.3). A turn lost with its host never commits: its
  provisional text is dropped when the run ends, and the next run
  starts after the last committed turn.
- **A person's words, in three states:** being sent, while their
  `Message` is pending; sent, once its answer names their message in
  the task's inbox; read, once a committed turn says it read through
  that message.
- **Bounded:** the lines it holds, and their bytes. A window past them
  keeps what fits, and says how many lines it left out.
- **How a thread shows** (a turn's calls folded to a line each, a call to
  the engine shown as what it made) is the presentation's; the reference
  UI's is view.md, section 9.

## 11. The small store

- **What the client keeps across its own restart:** the requests not yet
  answered, as the request table's records, each with the key the engine
  knows it by, its party, the application's key, the request (jig's
  record, or a family's bytes under its owner's number and version), and
  the wall time of its first sending; and the name of its stream, which
  the client that follows sends gone before it opens its own
  (`../../protocol/client.md`, sections 4 and 10). Nothing else of the
  client's: everything shown is read again. A presentation's own records
  (the reference UI's drafts and its place) are the application's, which
  it keeps beside the client's (root.md, section 8; view.md, 18.8).
- **Kept before sent.** A request's record leaves in a `Save`, and the
  client gives the request's first `Exchange` only once the transport
  says `Stored`. `NotStored` (a quota full, a disk failing) retires the
  requests the write carried, unsent, each answered *not kept*, by its
  key (skein's `lib.md`, 12, not kept): parked, or with its `Exchange`
  waiting for the write, since a parked request's record leaves at its
  admission too. Its erase leaves as any retired request's, since an
  earlier write may hold its record; if that erase is not stored either,
  a restart tells the request `Kept` again, under its key. A record is
  whole in each write, so a request whose record a later write carries
  again is decided by that later write, and one whose record the write
  did not carry is sent once its own write is stored. So a restart never
  loses a request the engine may have made.
- **Restored at `Start { kept }`,** before any other event: each
  request's record admitted to the table under its key, parked until
  the link is live, its party confirmed and the retention known; and
  each told to the application as `Kept { key, request }`, which the
  client root routes by the key's asker (root.md, section 8).
- **It names its party.** Each record carries the party it is for, and
  its request is sent, under its original key, only once `me` confirms
  that party. While another party is signed in, it stays parked and
  unsent, until its party returns or the retention passes and it is
  answered unknown. Whether, and how, to warn of someone else's kept
  work is the presentation's (the reference UI's: pages.md, section 8).
  A `Send` before `me` has named the party is answered unconfirmed at
  once, and nothing of it is kept (section 6).
- **Versioned** with jig's store version, so a client updated since reads
  what an older one kept. A record of a version it does not read is
  dropped, and the application told (`../../protocol/client.md`, section
  10). A family's request is kept in its owner's version: one whose
  owner the build does not declare, or whose version is not the one
  declared (8.2), is such a record. So is one past the records the
  table holds (section 14). The client tells only how many, `Dropped {
  count }`, once at start, under no asker: nothing of a record it
  does not read is read, its key's asker included. The client root
  gives the count to the presentation that says so (root.md, section
  8): the reference UI's notice (view.md, 18.7).
- **Where it lives is the host's:** a browser tab's session storage, a
  file written whole and synced, a world's memory. A host whose store is
  copied (a browser tab duplicated) sends the same keys from both
  copies, which the engine makes once, while each copy draws new keys
  from a seed of its own; the copy sends the original's stream gone,
  once, which the original takes as a drop (8.2).
- **It is never the truth.** A request kept is answered by the engine by
  its key, made once whatever the client did before.

## 12. Signing in

- **jig owns credentials, not sign-in flows** (contract.md, section 2).
  Its own two ways of signing in are the printed link's one-time
  secret, on loopback, and a service's credential. Every other way is
  the application's sign-in, on its own routes, ending in a `Mint` its
  server gives the clients' component (`../../protocol/client.md`,
  section 8).
- **`SignIn { with }`:**
  - *`Link(secret)`:* the printed link's secret. The client sends the
    component's sign-in with it, and holds the credential the answer
    gives, unless the transport keeps it;
  - *`Service(credential)`:* a service's credential, sent the same way,
    and answered with a sign-in's credential, held the same way;
  - *`Held(credential)`:* a credential the application obtained by its own
    sign-in, which the client holds as it is;
  - *`Transport`:* the transport carries a credential the client never
    sees, as a browser carries a cookie its server set.

  The `credential` on `Exchange` and `OpenStream` is the one held, or
  none. Signed in, the client opens the stream,
  watches `me` (8.2), and opens the watches asked before (8.1); under
  `Transport` it is signed in from its `SignIn`. So a host may sign in
  after `Start { kept }`, its children watching from their start.
- **The credential is never in the small store.** It is the link's, and
  `client.link()` gives it: a host that keeps a sign-in across its own
  restart keeps the credential where its platform keeps secrets, and
  gives it back with `Held`.
- **Signed out:** an answer, a read's terminal, a watch's end or the
  stream saying so drops the credential held, ends every handle
  (`Ended`, signed out), parks every request, and tells `Link` signed
  out. The application signs in again as it likes, and the parked
  requests are sent once the same party is confirmed, within the
  retention (`../../domain/people.md`, 5.1).
- **`SignOut`** sends the component's sign-out with the credential held,
  or none under `Transport`, to end the sign-in on the engine. Answered
  signed out, the client drops the credential and is signed out, as
  above.
- **A sign-out that does not reach the engine** (lost, or answered not
  ready while the engine stops) leaves the party signed in, as they
  still are on the engine: the client keeps the credential and its
  handles, and the link says the sign-out did not reach the engine.
  `SignOut` again sends it again, which is safe: a sign-out whose
  sign-in has already ended is answered signed out, the end asked for.
  Signed out meanwhile, whatever ended the sign-in, the client is signed
  out, as above, and the link no longer names the sign-out.
- **A sign-out's answer is its sign-in's.** The client knows which of
  its sign-ins each exchange belongs to. Once a newer sign-in is
  live, an earlier one's sign-out answer is dropped, and the newer
  stands. Nothing else the earlier sign-in owes reaches the newer one:
  going signed out parks every request, abandons every read and ends
  every handle, so an earlier attempt's answer is stale to the request
  table, an abandoned read's terminal is dropped, and an ended watch's
  deliveries are dropped by their names (sections 6 to 8).
- **A service's credential, when an owner makes one,** reaches the
  application once, in that request's `Answered` outcome; the client
  keeps no copy of it, and the small store holds no answer
  (`../../protocol/client.md`, section 8).
- **A bot relaying a person's decision** holds that person's own
  credential, from the application's sign-in (`Held`), and the decision
  is the person's request, naming the revision the bot showed them: jig
  adds no acting for another (`../../domain/people.md`, section 3).
- **Keys are the party's across sign-ins,** so the parked requests are
  sent again once the same party is confirmed, within the engine's
  retention.

## 13. Time and notices

- **Deadlines are the client's:** the backoffs of requests and of
  opening the stream and its watches again, the stream's heartbeat
  tolerance, and the retention's cutoff, on the monotonic clock
  (programming-model.md, section 9). `next_deadline()` gives the
  earliest, and `fire(now, wall, ..)` runs what is due.
- **Wall times are for records and for showing.** The request table
  records the wall time of a request's first sending, so its cutoff holds
  across a restart; documents carry wall times, which a presentation
  shows (contract.md, 5.3).
- **Notices,** a presentation's short-lived words for what has no other
  place, are the reference UI's (view.md, 18.7). The client tells
  only the facts they are made from: answers and the link.

## 14. Limits and the worst case

Each is configured and part of the worst case:

- handles; the engine's watches open, `me`'s included, each until its
  terminal and one kept for `me` (8.1); reads in flight; exchanges in
  flight;
- a path's segments: the most segments of a resource's path or a
  pattern a document may carry, a bound of its own, never a window's
  rows; the application sets it at least the most its engine admits in
  a name (contract.md, 5.3), and a document past it is refused at
  decoding (section 16);
- an authority's entries: the most entries of one of an authority's
  lists (its grants, its executors, its notes' scopes) or of an amending
  offer's (its grants listed, its implications) a document may carry, a
  bound of its own, never a window's rows; the application sets it at
  least the most its engine admits in each of them (contract.md, 5.3),
  and a document past it is refused at decoding (section 16);
- bytes of a document held, of a page handed over, and of a run's
  provisional turn, each priced from the wire's worst cases at the
  client's limits (`../../protocol/client.md`, section 13);
- the request table's: requests held, and their bytes, a family's
  included (skein's `lib.md`, section 12); the records restored at
  start, no more than the table holds and the stream's name (section
  11);
- backoffs' first and most spans, the heartbeat's tolerance, the
  retention's margin;
- the family owners the application declares, each once, with its
  version (8.2): no more than a hello's ranges hold beside jig's
  (`../../protocol/client.md`, section 9).

`worst_case` sums the containers' own and the bytes each entity may hold
(programming-model.md, 6.3). Each helper's limits are its own (section
19); the reference UI's derive from the client's and its own (view.md,
18.9).

## 15. What it never knows

- **HTTP and its platform:** no path, cookie, header, host or event
  stream; no browser, window or terminal. The transport and the host are
  the application's (section 16).
- **A page:** what is shown, where and how; what a person is in the
  middle of.
- **An application's values:** its families' requests, subjects, pages
  and documents are bytes under their owner's number; a part named in
  one of jig's documents is a reference (root.md, section 6).
- **Templates:** a template is the application's data, which `me` serves
  and a helper or the application fills into an ordinary `Create` or
  `Propose` (contract.md, 3.6; `../../protocol/configuration.md`, 3.1).
  The client sends what it is given.
- **The engine's internals:** tasks by number, documents by subject and
  revision, outcomes and refusals as the contract gives them. Authority
  is never computed here: what a party may do is what the engine's
  documents offer, and the engine checks it again.

## 16. Below the client

What the transport and the clients' component owe the client, which
the client's protocol designs (`../../protocol/client.md`):

- **The transport is the application's:** jig's default HTTP mapping
  (`jig-client-protocol-http` at the client's end,
  `jig-protocol-engine-clients-http` at the engine's;
  `../../protocol/client.md`, section 17), a local socket for a command
  line, or an in-process channel for one process or a world.
- **One terminal per exchange:** each `Exchange` answered by one `Reply`,
  with the record that came, or by one `Lost` when none came whole
  within the exchange's deadline, the transport's. The transport makes
  one attempt; retrying is the client's.
- **The stream in order:** its events from the hello on, each a
  `Streamed`, in the order the component wrote them; then one `StreamGone`,
  whatever ended it.
- **The small store in order:** each `Save` and `Erase` written, synced
  as its platform allows, and reported `Stored` or `NotStored` before the
  next.
- **The credential as given:** the one on `Exchange` and `OpenStream`
  carried with its message, or, under `Transport`, the transport's own.
- **The clients' component,** at the engine's end, takes jig's messages
  with their credentials, answers each exchange with one record, and
  writes the stream (`../../protocol/client.md`, 4.2). It knows no
  HTTP.
- **Documents decoded whole,** into the client's types, and checked
  against the client's limits before they reach its domain
  (programming-model.md, 4.4); a document past them is refused at
  decoding, as too large, never cut silently. A path is held whole,
  within the client's bound on its segments: only the view cuts what it
  shows of a long one (view.md, section 7). An authority is held whole,
  within the client's bound on its entries, and a draft is made from it
  whole: only the view cuts what it shows of a long one (view.md,
  section 7).

## 17. The world

`jig-client-world` (`tests/client`) runs the client through its surface
alone, as an application holds it, with the engine scripted (testing.md,
section 4), and joins the engine's domain in the joint world (testing.md,
section 5). Its stories: signing in by the printed link and starting a
chat by `shapes::chat`; a held chat amended and released in one
compound request; a proposal accepted while two handles watch it; two
clients deciding one thing; a restart with requests pending, the kept
work told and sent; the engine restarting under a pending request; the
stream dropped, every handle behind, then live; a handle given another
subject while a delivery for the old one is in flight; a store write
that fails; another party signing in on a client with kept work; a
family's request, read and watch as bytes; a version not served; a
sign-out lost, then sent again; a one-shot client that sends, waits for
the durable answer and stops.

Its referee is testing.md's, 6.1 and 6.2; the reference UI's stories,
with a person in front of a tree, are view.md's, 18.10.

## 18. Open questions

- **Several clients of one party,** such as two tabs: whether pending
  requests should be shared between them, which they are not; and the
  party's room for watches on the engine (contract.md, 7.2), which they
  share while each counts its own, so that one may find its watches
  busy, behind until the other's end.
- **A window's turn:** how a thread shows a turn that opens a window (a
  compaction's summary of what came before it), beside the earlier
  window's turns it summarises.
- **Signing out with the engine out of reach:** whether a client should
  keep the sign-out and send it once the engine is reached, or a host
  forget its credential there, leaving the sign-in live on the engine
  until it expires. A browser's cookie only its server clears
  (section 12).
- **The generic half in skein:** keyed requests until durable,
  level-triggered watches and the link are jig's while they have one
  user; whether they move to skein, by a rename and a crate move, once
  a second user wants them.

## 19. Helpers

- **Sugar over the public client.** jig's helpers are modules of
  `jig-client`, built only from plumbing (the API's operations, subjects
  and reads, contract.md) and from the client's public surface, as an
  application's own code is. A helper an application ignores costs it
  nothing, and any application may write its own instead.
- **`shapes`:** functions composing plumbing requests from templates,
  the application's data that `me` serves (contract.md, 3.6;
  `../../protocol/configuration.md`, 3.1):
  - `shapes::chat(me, project, words)` fills the chat template into a
    `Create`;
  - `shapes::close_chat(task, disposals)` gives one compound request:
    each live delegate cancelled or moved to the party, then the chat
    cancelled with the template's grace;
  - `shapes::set_goal(..)` fills the goal template, compares the draft
    with the project's creating bounds in `me` (contract.md, 5.3), a
    deadline given as a span within both, for the engine to judge, and
    gives one of four: *made,* a `Create`, within the first bound;
    *proposable,* a `Propose` to the roles the offer names, between the
    two bounds where the offer says so; *refused,* the offer's refusal,
    between them where it says so; or *not offered,* past the second
    bound or with an executor not listed. Refused and not offered send
    nothing;
  - each fills what a template does not give from `me` and its own
    inputs: the authority `me` offers for creating (contract.md, 5.3),
    one level below the role, made one level shallower again for each
    level of the template's `shallower` past the first, its spend the
    budget named (any for `AsAllowed`, whose amount the engine gives)
    and its deadline the task's: the template's span sent as a span,
    which the engine makes a wall (contract.md, 3.2), or the creator's
    wall where the template has it given, so that no shape reads a
    clock; no wake policy, so the executor's own;
    no parameters, inputs, dependencies or grants named after it;
  - a template it cannot fill gives `NotOffered::Template`, and nothing
    is sent: a contract no `Create` names (an approval, or a verdict
    whose choices its document cut), a value its helper takes no input
    for (a chat's executor, budget or deadline the creator's to give),
    or a `shallower` of 0, which no task's authority may be.
- **`facts`:** pure functions over the documents the client holds, the
  same for an application's own presentation as for the reference UI:
  - *a chat's status,* from its chip's phase and its host's last report,
    what it waits on, and `me`, for the chat's project:

    | Status | The chat is |
    |---|---|
    | starting | waiting to start, due, preparing or claimed; or its host preparing or starting its run |
    | writing | running, writing |
    | a tool | running a tool its host names: which, and since when |
    | working | running, neither writing nor in a tool its host names, or nothing reported yet; and since when |
    | waiting for you | waiting on the signed-in party: a question, a proposal or a person task in its tree; and for what |
    | waiting for them | waiting on another party: whom |
    | parked | idle: its run waiting, holding its slot, or parked |
    | retrying | backing off: until when, and why |
    | held | held: why |
    | closing; ended | closing; then ended, and when |

    The first that applies, in this order: held, closing and ended, each
    a phase that says why it waits; then a party it waits on, in any
    active stage; then its stage, and, while its run is claimed or
    running, what its host last reported. Writing and a tool are read
    only from such a report, never from a run being placed
    (`../../domain/hosts.md`, 6.5). A run that waits holds its slot only
    until it parks, and is idle meanwhile (`../../domain/engine.md`,
    7.4). The party it waits on is the signed-in one where it names
    them, or a role `me` says they hold in the chat's project; any other
    is another party. How each status is said is the presentation's
    (the reference UI's: pages.md, section 4);
  - *an act's offer and effect:* the engine's admission of the act, as
    the document offers it (made, proposable and to whom, bounded, or
    refused; contract.md, 5.3), and what a decision will do, where the
    act is offered with it (contract.md, 5.4); read, never inferred;
  - *a refusal and why:* an offer's refusal, with its facts
    (contract.md, section 4); a request's comes with them in its answer
    (section 6);
  - *whether an act is dangerous:* a cancel, a proposal's rejection and
    a note's deletion are, and no other act. A presentation that
    confirms marks a dangerous act so (the reference UI: objects.md,
    section 2);
  - *a decision's facts:* what a deciding act on an object would do, as
    data read from its document: whether the act is dangerous; its
    effect, where the act is offered with one; and, for accepting a
    proposal, what it needs and who funds it, with what the funder has
    left and will have after (contract.md, 5.4). jig's check holds a
    tree a person decided on to them (testing.md, 6.2).
- **`thread`:** a conversation's thread (section 10).
- **`words`:** jig's English for its enums (phases, holds, refusals, what
  a task waits on) and for a decision's facts, as phrases with holes an
  application overrides phrase by phrase, built with skein-ui's
  phrasebook types, `jig-client`'s one use of skein-ui (section 3); the
  reference UI says them through skein-ui's phrasebook (view.md,
  section 7), and jig's check says a decision's facts in them, with the
  application's overrides (testing.md, 6.2).
- **What jig adopts.** Sugar starts in an application. jig adopts it
  when a second consumer uses it, or early when it is hard to interpret
  consistently (what "waiting for you" means, how committed turns and
  provisional text interleave), and once its shape has settled.
  Correctness (revisions, once, atomicity) is never sugar: it is
  plumbing.

## 20. Modes

- **Long-lived:** a browser or a terminal. The host drives its loop, and
  the application presents.
- **A service:** the same, with a service's credential (`SignIn { with:
  Service }`) and no presentation; its client root holds the service's
  own logic (root.md, section 11). Its small store is a file, written
  and synced before any request it keeps is sent, so a restart of the
  service sends what is pending again, under its keys.
- **One-shot:** sign in, read or send, wait for the durable answer, and
  stop, with the small store in a file: a request sent by a run that
  stopped before its answer is sent again by the next run, under its
  key.
- **The client runs no loop of its own in any mode.** The host steps it,
  fires its deadlines and carries its requests; the modes differ in the
  host alone, and each follows from the client's shape.
