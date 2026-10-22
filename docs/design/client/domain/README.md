# jig's client

Provisional, 2026-10-09, revised 2026-10-22. The design of jig's client:
`jig-client`, the client an application holds to reach the engine; the
API it speaks with the engine; how an application's client root holds it
beside its own kinds of object; jig's helpers; jig's reference UI, the
pages, cards and view an application may present with; and how all of
it is tested. jig as a whole is `../../README.md` (section 10 is the
client in one page); the engine's domain is `../../domain/`; jig's
messages, the clients' component and jig's default HTTP mapping are the
client's protocol's (`../../protocol/client.md`,
`../../protocol/browser.md`). The mechanics are those of skein's
`programming-model.md`, as for every part of jig.

## 1. Reading order

1. **client.md:** the client: its surface, keyed requests through the
   request table, reads, watches by handle and the link, the small
   store, signing in, its modes and jig's helpers. Read it first.
2. **contract.md:** the API: what crosses between a client and the
   engine: the operations and compound requests, documents with their
   revisions and offers, subjects, reads and task queries, `me`,
   answers, versions, and what the engine's domain owes the client.
3. **root.md:** what an application's client root does: the one shape,
   routing by asker, parts, the application's families as bytes, a
   service's root, and owners.
4. **pages.md:** the reference UI's pages: the frame, the inbox,
   conversations, the task page, a project, the deployment; what an
   application adds; the frame's parts, the sections, their region names,
   and arrangements.
5. **objects.md:** the reference UI's objects and their cards: chips,
   proposals, escalations, questions, person tasks, results, refusals,
   the conversation, and the requests that steer a task.
6. **view.md:** the reference UI's view on skein's UI kit (skein's
   `ui.md`): its primitives and jig's default arrangement of them, its
   owners, bindings, words, tokens, a decision's facts,
   conversations, shells, and jig's settings for skein's text shell;
   and the reference UI's state (section 18): its place and page, the
   working set, its requests, drafts and the confirmation, notices and
   its records.
7. **testing.md:** testing the client through its surface, the reference
   UI with skein's fake person, the joint world with the engine's
   domain, and their referee.

`../../examples.md` is the example application, `ops`, which each
document uses for its examples, with jig's test application where a
second one helps.

## 2. Structure

```
an application's client
<application>-client-domain          its client root: the client held, its owners' client sides, routing (root.md)
├── jig-client                       the client, jig_client::Client, and jig's helpers: shapes, facts, thread, words (client.md)
│   ├── jig-client-domain            the client's domain: the API's types, the link, handles and their documents, the request table's use, kept work
│   └── jig-client-protocol-link     jig's messages and the small store's records, encoded and decoded; the stream's events
│       ├── jig-client-protocol-wire jig's wire records, for both ends (../../protocol/client.md, section 3)
│       └── jig-protocol-transcript  turns decoded with smith's transcript crate (client.md, section 10)
├── jig-client-view                  the reference UI, if it presents with it: pages, places, cards, the confirmation, drafts, notices (pages.md, view.md)
└── <owner>…                         each owner's client side: a connector's parts, its families and their presentation
<application>-client-protocol-wire   its families' records, as its owners' bytes (root.md, section 7)
<application>-client-view            its view root, if it presents with skein's UI kit (view.md, section 4)
<application>-client-shell-<platform> its shells

the transport, the application's, with jig's defaults, to use or leave
jig-client-protocol-http             the default HTTP mapping's client end, for the host and wasm32 (../../protocol/client.md, 17.4)
jig-client-shell-native              a native shell over skein's HTTP client (../../protocol/client.md, section 12)
jig-client-shell-web                 jig's browser parts: the client's adapter, the printed link's sign-in, jig's look (../../protocol/browser.md, 5.1)
skein-ui, skein-ui-text, skein-ui-web, skein-ui-bundle   skein's UI kit (skein's ui.md, section 2)

for tests
jig-client-test-domain, jig-client-test-view   jig's test application's client root and view root, the reference (root.md, section 12)
jig-client-test-shell-web            jig's test application's browser shell (../../protocol/browser.md, 3.1)
jig-client-test-kit, jig-client-test-records, jig-client-test-protocol-wire   the client's test kits (testing.md)
jig-client-scripted-engine           the engine as the client's worlds play it (testing.md, section 4)
jig-client-fake-person-browser       jig's stories for a browser shell, and its attack stories (../../protocol/browser.md, section 15)
skein-fake-person, skein-fake-person-browser   skein's person, its browser face and harness (testing.md, section 3)
jig-client-world, jig-client-protocol-world, jig-client-wire-world   the client's worlds (testing.md, section 4; ../../protocol/client.md, section 14)
```

- **Every client crate lives under the `<project>-client` prefix,**
  jig's and an application's alike (`../../README.md`, section 15).
- **The crate graph is the role graph,** as in the engine.
  `jig-client-domain` depends on skein-lib alone; `jig-client` on it,
  its link, and `skein-ui` for its `words` alone (client.md, section 3);
  `jig-client-view` on `jig-client` and `skein-ui`. None of
  them links smith: only `jig-protocol-transcript`, below the client's
  surface, does, and it is the client's only use of smith
  (`../../README.md`, section 10).
- **The application owns the client's root,** as it owns the engine's root
  (root.md), and its presentation: its own, on skein's UI kit or
  anything else; jig's reference UI, whole or in part; or none. Nothing
  in jig's client knows an application's types.
- **The wire, the link, the transport and the shells** are the client's
  protocol's (`../../protocol/client.md`, the default HTTP mapping in its
  section 17; jig's browser parts,
  `../../protocol/browser.md`), beside
  the engine's, and skein's UI kit's (skein's `ui-text.md` and
  `ui-web.md`). This design says what they owe the client, and no more,
  but for jig's settings of skein's text shell (view.md, section 15). A
  shell is the impure part of a client, ordinary Rust as skein's shell
  is, not step code.

## 3. Conventions

- **A bare file name** names a document of this directory: `client.md`.
  jig's other documents are named by their path from here:
  `../../README.md`, `../../domain/people.md`.
- **Code cites a section** at module or type level, as
  `client/domain/client.md`, 8.2.
- **skein's and smith's documents** are named with their project's name:
  skein's `lib.md`, smith's `transcript.md`. skein's foundation documents
  by their own names: `programming-model.md`, `testing-strategy.md`.
- **An application** appears only as an example, and says so: `ops` most
  often, jig's test application where a second example helps.
- **A party** is whoever uses a client: a person, or a service
  (`../../domain/people.md`, section 3). Where only a person makes sense
  (a view, a draft), the documents say person.
- **Each document** says in one page what it is, then its structure, its
  parts, what it owes and is owed below it, its world, and what is open.

## 4. Names

| Name | What it is |
|---|---|
| client | `jig-client`, `jig_client::Client`: the step machine an application holds to reach the engine: the link, watches by handle, the request table, kept work, families as bytes (client.md) |
| client root | the application's top-level client domain, its root domain (programming-model.md, 4.5), which holds the client as a component beside its own children, and routes no child of jig's and no request table (root.md) |
| presentation | what a party sees and acts on: the application's own, jig's reference UI, or none; the application's |
| reference UI | `jig-client-view`: jig's pages, places, navigation, cards, confirmation, drafts, notices and view, on the public client alone and skein's UI kit, for `ops` and jig's test application; optional (pages.md, objects.md, view.md) |
| the API | jig's messages: plumbing operations, subjects and reads, documents with their revisions and offers (contract.md) |
| plumbing | jig's primitives as operations, subjects and reads; whatever must be atomic, only the engine can compute, or the engine's behaviour turns on (contract.md) |
| sugar | what is built only from plumbing: templates, helpers, and facts and models (client.md, section 19) |
| operation | one of the API's requests: `Create`, `Propose`, `Withdraw`, `Message`, `Amend`, `Cancel`, `Release`, `Stop`, `Move`, `Decide`, `Answer`, `Take`, `HandBack`, `Seen`, or the project's own (contract.md, 3.2) |
| compound request | one keyed request holding a list of operations, admitted in order and committed in one decision, all or none: `Compound { project, operations }` (contract.md, 3.4) |
| template | the application's definition of a shape of work, numbered data in its configuration, which a project's policy may override and `me` serves; the client or a helper fills it into a `Create` or a `Propose`, and the engine never expands or checks it (contract.md, 3.6; `../../protocol/configuration.md`, 3.1) |
| helper | one of jig's optional modules of `jig-client`: `shapes`, `facts`, `thread`, `words` (client.md, section 19) |
| `me` | the signed-in party's own document: the party, their projects and roles, what waits for them, their read positions, each project's merged templates, the bounds a draft is compared with (contract.md, 5.1) |
| task query | a subject or read of tasks by fixed keys, orders and a window by cursor, whose rows are task lines (contract.md, 6.1) |
| task line | the one record every list of tasks is made of (contract.md, 5.1) |
| handle | the application's name for a watch or a read: its asker and a number (client.md, section 4) |
| asker | the child of the client root a handle or a key names, by its owner number: an opaque number the client hands back and never reads (client.md, section 4; root.md, section 15) |
| key | the application's name for a keyed request, its asker's and a number; the engine knows the request by the key the request table draws (client.md, section 6) |
| keyed request | a party's request: ordinary requests kept before sending, top-level `Seen` in memory only; retried under its key and pending until answered (client.md, section 6) |
| request table | skein-lib's container of keyed requests, held by the client: its only way out for them (client.md, section 6) |
| link | whether the client reaches the engine: signed out, starting, live, offline, or out of date (client.md, 8.2) |
| behind | a handle whose document may be old: its watch dropped and opening again, or changed (client.md, 8.1) |
| small store | what a client keeps across its own restart: its kept requests not yet answered, excluding new `Seen` and requests carrying a value; each child of its root keeps its own records beside them (client.md, section 11) |
| kept work | the requests a client restored from its small store, told to the application as kept (client.md, section 11) |
| credential | what a sign-in leaves, which jig mints, verifies, expires and revokes; held by the client, or carried by the transport unseen (client.md, section 12) |
| transport | the application's carrier of jig's messages between the client and the clients' component: jig's default HTTP mapping, a socket, or in-process (client.md, section 16) |
| clients' component | `jig-protocol-engine-clients`, the engine's side: jig's messages with their credentials in, answers and the stream out; transport-agnostic (`../../protocol/client.md`, 4.2) |
| default HTTP mapping | jig's optional mapping of the component's messages onto HTTP, at both ends, with example guards (`../../protocol/client.md`, section 17; `../../protocol/browser.md`, 12.2) |
| mode | how a host runs the client: long-lived, a service, or one-shot (client.md, section 20) |
| view root | the application's top-level view on skein's UI kit: its tree, from the reference UI's primitives and its own nodes (view.md, section 4) |
| shell | the platform's code, the only impure part of a client (view.md, section 11) |
| view | a presentation's state as a tree a shell shows, and a person's acts coming back (view.md) |
| frame | the reference UI's parts around a page's main: the link, what needs the party, who is signed in, the projects, going to a task, the navigation (pages.md, 12.1) |
| frame's part | one of the frame's, one of the reference UI's primitives with a region name, placed anywhere or nowhere; jig's banner, in jig's default arrangement, holds the first five on every page |
| page | a place of the reference UI's with an address: the inbox, a task, a project's goals, an application's own; made of sections, arranged |
| section | one part of one of the reference UI's pages, which it opens handles for and builds whole: a task's plan, the inbox's needs (pages.md, 12.2) |
| arrangement | how a page places the reference UI's primitives and an application's own nodes; jig's pages are jig's default arrangement (pages.md, 12.3) |
| jig's primitives | the functions of the reference UI's view that build part of a tree or handle its acts: the frame's parts, the sections, the cards, the confirmation, the owed cards, the notices, jig's bindings and their decoders (view.md, section 4) |
| jig's default arrangement | the reference UI's own arrangement of its primitives: its banner, its pages, `after_page`; a default and a reference, never mandated (pages.md, 12.3) |
| a decision's facts | what a decision rests on (the object's state, a refusal and why, what a decision will do and whether it is dangerous): data in jig's documents (contract.md, 5.4), which the helper `facts` reads (client.md, section 19) and a presentation says, with `words` or its own words (objects.md, section 2) |
| jig's check | the test kit's: in an application's worlds, a decision sent naming a revision no document the client delivered carried; on a tree, a deciding act made on a tree that did not show its facts; never mandated (testing.md, 6.2) |
| outline | skein-ui's rules for the landmarks a tree keeps: each child of the page node a landmark or a dialog, exactly one main, at most one banner and one content info; jig adds none (skein's `ui.md`, section 14; view.md, 3.1) |
| landmark | a role saying which part of a page a node is: banner, navigation, main, complementary, content info, form, search |
| region name | a region's language-free, stable name, which a stylesheet places it by (skein's `ui.md`, section 8; pages.md, 12.4) |
| object | something a card shows, held once by the reference UI: a task, a proposal, an escalation… (objects.md) |
| card | how one object shows in the reference UI, wherever it appears |
| chip | a task named in passing: its number, its title, its phase, kept live |
| client side | the child of the client root that shows one connector's values, and holds its families (root.md, section 2) |
| family | an application's kind of object, with its own requests, subjects and presentation, carried by the client as bytes (root.md, section 7) |
| owner | a child of the client root that keeps values, names nodes of a view, or asks the client: the reference UI, or one of the application's, numbered (root.md, section 15); never a project's role, which is always "a project's owner" |
| owner number | the number an application gives one of its owners, shared by its engine's root and its client root, the wire, the client's askers and the small store; a connector's is its connector's (root.md, section 15) |
| part | a value of an application's that a jig document names by reference: a connector's name, a resource, a kind, a panel, an effect's description (root.md, section 6) |
| slot | the place a reference UI's page or card leaves for a part, or an application's page for one of the reference UI's cards or sections, filled by its owner |
| shortcut | a key a bound node answers to, as a press (view.md, section 5) |
| scene | a drawing as its owner's drawing code draws it: a bounded list of marks in whole pixels, which each shell that draws maps onto its platform (skein's `ui.md`, section 11; view.md, section 17) |
| component, a shell's | a shell's native implementation of a node, chosen from the look by the node's role and one of its owner's style tokens, or by a drawing's kind (skein's `ui.md`, section 12). Not a protocol layer's component (`../../protocol/README.md`, section 3) |
| client host | an application's client root, its presentation and its transport's client end, stepped as one machine and shared by its shells (`../../protocol/browser.md`, 3.2) |
| browser shell | an application's crate, `<application>-client-shell-web`: its start and its loop over skein's browser parts, the reference UI's and its client host, copied from jig's test application's, `jig-client-test-shell-web` (`../../protocol/browser.md`, 3.1) |
| text shell | skein's shell for a terminal or a pipe, `skein-ui-text`: the tree printed as lines, a typed line read as an act; the reference UI gives it jig's settings (skein's `ui-text.md`; view.md, section 15) |
| document | a typed value the engine gives a client about one subject, its owner's alone (contract.md, section 5) |
| revision | the state of one thing a party decides about (a hold, a task's spec, a person task, a note), which the decision names (contract.md, 5.2) |
| read | a request for a document, or a page of a list; one answer |
| watch | a live view of a subject: its current document, delivered whenever it has changed, one delivery at a time (contract.md, section 7) |
| window | what a watch's document or a read's page holds of a list: its size, its order and where it starts |
| draft | words a person is writing, kept by a presentation across pages and reloads |
| confirmation | what an act will do, said before it is sent: the presentation's, from the facts jig gives |
| bounds | the engine's facts a draft of an act is compared with, and what happens beyond them, offered for creating and amending (contract.md, 5.3) |
| phrasebook | the words a view says, for one language, by skein's machinery (skein's `ui.md`, section 9; view.md, section 7) |
| fake person | skein's person, `skein-fake-person`, in the client's worlds: reads the tree by role, name and text, and acts on it (skein's `ui.md`, 17.1; testing.md, section 3) |
| scripted engine | the engine as the client's worlds play it (testing.md, section 4) |

The engine's names (`../../domain/README.md`, section 4) keep their
meaning here: task, goal, chat, proposal, escalation, party, role,
connector, resource.

## 5. Decisions this design takes

Questions `../../README.md` left open, and others met while writing, with
where each is answered.

**The client and the application:**

- **Presentation-agnostic.** jig's client knows no page, place,
  arrangement, confirmation or draft; all presentation is the
  application's (client.md, section 1).
- **The client root is the application's,** and holds jig's client as a
  component beside its own children, as the engine's root holds the
  core (root.md, section 2). No root routes a child of jig's or a request
  table: the client keeps, sends and answers every keyed request
  itself, and the root routes what it tells back by the asker each handle
  and key carries, keeping no map of its own (root.md, section 5).
- **The client never calls the application.** It is a step machine with
  `step`, `fire` and `next_deadline`; what it tells is a value in the
  queue, and what it holds the application reads (client.md, section 4).
- **Watches by handle.** The application names each watch and read; the
  client keeps the latest document of each watch, shares one engine
  watch among the handles on one subject, and names each opening afresh,
  so nothing arrives for a handle that has moved on (client.md, 8.1).
- **The key is the application's name for a request;** the engine knows
  it by the table's drawn or caller-supplied key. A kept request's record
  stores both keys; new `Seen` keeps them in memory (client.md, section 6).
- **Kept requests before sent, by the client.** A kept request's first
  exchange leaves only once the transport says its record is stored
  (client.md, section 11).
- **The client's barrier is skein-lib's.** Every keyed request, jig's and
  the application's, leaves through skein-lib's request table, which
  the client holds: ordinary requests kept before sending, top-level
  `Seen` in memory only, both retried under their keys within the
  engine's retention and only for their party. New `Seen` retains no
  completed engine answer and retries against current state; unanswered
  marks are silently lost on closing, reload or crash (client.md,
  sections 6 and 11; contract.md, 3.1).
- **The application's families travel as bytes,** under their owner's
  number and version, on the same connection and through the same
  table; the client reads their owner, version and size only (client.md,
  section 4; root.md, section 7).
- **Derived states:** the engine computes what needs a join only it
  holds (what a task waits on, `AsAllowed`); jig's helpers derive what
  follows from documents the client has (a chat's status, the thread)
  (client.md, sections 10 and 19).
- **Templates are filled in by the client,** by `shapes` or by the
  application, and never expanded or checked by the engine; the
  application's configuration defines them, a project's policy may
  override their defaults, and `me` serves them merged (client.md,
  section 19; contract.md, 3.6; `../../protocol/configuration.md`, 3.1).
- **`me`** is the signed-in party's own document, which the client
  watches whenever a sign-in is live, to confirm its party and learn the
  retention (client.md, 8.2; contract.md, 5.1).
- **Modes:** long-lived, a service and one-shot follow from the client's
  shape, with no loop of the client's own in any (client.md, section
  20).
- **Builds for the host and for wasm32,** as step code under the strict
  subset (client.md, section 3).
- **The reference UI is optional,** on the public client alone, for
  `ops` and jig's test application; it proves the API is enough
  (pages.md, view.md).
- **A service uses the client in its service mode,** with no
  presentation (root.md, section 11).

**The API, as the client meets it:**

- **Plumbing and sugar.** The API is jig's primitives; a shape of work
  is sugar: a template, a helper, or a function over documents, built
  only from plumbing. What must be atomic, what only the engine can
  compute, and what the engine's behaviour turns on, is plumbing
  (contract.md; client.md, section 19).
- **Proposing is explicit:** a request beyond the party's authority is
  refused, naming what it lacked; `Propose` asks (client.md, section 6;
  contract.md, 3.5).
- **One request per decision:** what must commit together is one
  operation or one compound request, so the client never chains two
  (client.md, section 6).
- **A document per owner.** jig's documents carry jig's values only; an
  application's value they need is a part, named by reference, and read
  and followed by the application's own child through the application's
  own subjects, on the same connection, each naming its scope: a
  project, the deployment, or the party's own, which is admitted on the
  credential alone (root.md, sections 6 and 7; contract.md, 7.5).
- **Views are level-triggered.** A watch delivers its subject's current
  document whenever it changed since the last delivery, one delivery in
  flight; what changed meanwhile is coalesced, never queued. The client
  tells the application the same way: `Changed`, and the latest
  document (contract.md, section 7; client.md, 8.1).
- **What a party reads is durable.** A read's answer and a watch's
  delivery wait for every commit they reflect; only a run's text as it
  is written is provisional, and marked so (contract.md, sections 6 and
  7).
- **The inbox is the party's, across projects,** read filtered by
  project, kind or root (contract.md, section 6 and 7.1;
  `../../domain/people.md`, section 6).
- **Decisions name what they saw.** A decision names the revision of
  what it decides, from the document the application showed, and one
  whose revision moved is refused as moved, saying what it is
  (contract.md, 5.2).
- **Offers with bounds.** A document says, for each act, made,
  proposable (to whom) or refused, and offers creating and amending
  with the facts a draft is compared with, so a presentation
  can say which before sending; the engine judges again when the
  request comes (contract.md, 5.3).
- **A decision's effect is data:** what accepting grants and costs, what
  releasing does, what cancelling ends (contract.md, 5.4).
- **Turns are decoded below the client's surface,** by
  `jig-client-protocol-link` with `jig-protocol-transcript`; neither the
  application nor the helpers link smith (client.md, section 10).

**The server, the transport and signing in:**

- **The server and its URL space are the application's.** jig's clients'
  component is transport-agnostic, and jig reserves no path; the client
  gives jig's messages, never HTTP (client.md, section 16;
  `../../protocol/client.md`, 4.2).
- **Guards are the server's.** jig ships example guard code in its
  default HTTP mapping, and attack stories in its test kit, to use if
  they suit (`../../protocol/client.md`, section 17;
  `../../protocol/browser.md`, 12.2).
- **jig owns credentials; the application owns sign-in flows,** beyond
  the printed link and a service's credential. The client signs in by
  `SignIn { with }`, and holds a credential only where its transport
  needs it in hand (client.md, section 12).
- **A bot relays a decision with the person's own credential;** jig adds
  no acting for another (client.md, section 12).
- **Versions are per owner.** A version not served is refused, typed,
  naming the ranges served; a client outside a range keeps what is
  pending and sends nothing of that owner. The client and the engine
  come from one pin of jig (client.md, 8.2; contract.md, section 8).
- **The application's streams** are designed in the pattern of a run's
  text (contract.md, 7.4).

**The reference UI:**

- **The confirmation is the application's.** The reference UI keeps
  jig's; jig gives a decision's effect as data, and holds the revision,
  the "once" and admission below the API (client.md, section 9).
- **What jig promises is kept below the API,** never by a presentation's
  shape:
  - *held by construction, whatever presents it:* a decision names the
    revision it saw, and a moved revision is refused (contract.md, 5.2);
    ordinary deciding acts made once, kept work surviving restart, and
    nothing answered done before durable are the request table's and the
    engine's (client.md, sections 6 and 11);
  - *given as data:* a decision's facts, in jig's documents, which the
    helper `facts` reads (client.md, section 19) and a presentation
    says, with `words` or its own words, wherever it confirms
    (contract.md, 5.4; objects.md, section 2);
  - *checked, not mandated:* jig's check, in its test kit, which the
    referee runs in jig's worlds and in an application's, never at run
    (testing.md, 6.2).
- **The reference UI's view is primitives; nothing in the tree is
  mandated.** The frame's parts, the sections, the cards, the
  confirmation, the owed cards and the notices are functions of its
  view, each placed anywhere or nowhere; jig's pages, its banner and
  `after_page` are its default arrangement of them, which an
  application takes as they are, rearranges, or puts its own in their
  place (view.md,
  section 4; pages.md, section 12; root.md, section 7).
- **Home and navigation are the presentation's,** among the reference
  UI's pages and the application's own (pages.md, section 2).
- **The view is semantic and platform-neutral:** skein's tree (skein's
  `ui.md`, section 3), shaped like an accessibility tree, which each
  shell maps onto its platform (skein's `ui.md`, section 16).
- **Words come from a phrasebook,** one per language: jig ships English,
  and an application may give its own as data (client.md, section 19;
  view.md, section 7).
- **An application may draw:** its own kinds of drawing (a chart, a map),
  whose data its view writes, drawn as a scene by its drawing code or by
  a component of its own, each with a fallback in words every shell, a
  screen reader and the fake person can read (skein's `ui.md`, sections
  11 and 12; view.md, sections 3 and 17).
- **The browser's parts are skein's; the server and the loop are the
  application's.** skein's UI kit gives the browser parts, scenes as
  SVG, the bundler, and the browser tier's face and harness (skein's
  `ui-web.md`). jig adds its browser parts, `jig-client-shell-web`:
  the client's adapter, the printed link's sign-in, and jig's look,
  each usable alone. An application's browser shell, copied from jig's
  test application's, runs its own loop over the parts and its client
  host; its server serves its page and bundle
  (skein's `ui-web.md`, section 4) and mounts jig's clients' component
  under a prefix of its choosing (`../../protocol/browser.md`, 3.1 and
  section 12).
- **Layout as semantics.** Every tree keeps skein-ui's outline of
  landmarks (skein's `ui.md`, section 14), to which jig adds no rule, and
  the reference UI's regions carry stable region names (view.md, section
  3).
- **The tree's closed sets are skein's,** grown by skein's criterion
  through its light path (skein's `ui.md`, sections 3 and 4): the
  reference UI uses them and adds none.
- **Owners, not connectors.** Everything that names an application's
  owner, as root.md, section 15, lists (parts, the client's askers, the
  view's slots, bindings, styles, drawings and region names, requests,
  reads and subjects), names it by a number its engine's root and its
  client root share, a connector's being its connector's. In the tree,
  an owner is skein-ui's opaque number (skein's `ui.md`, 3.3): an
  application's owner its own number, and jig 65536, one past every
  `u16`, so the two never meet (root.md, section 15).
- **The text shell is skein's:** printing a tree and reading a line
  decide nothing of jig's or an application's (skein's `ui-text.md`); a
  terminal presents the reference UI with jig's settings, its status
  region `jig-link` (view.md, section 15).

**Building:**

- **Designed around one person.** A person holds the roles they need in
  the projects they use. What only several people raise (who may see
  what, a role's entries shared and taken) is designed where it meets
  the client; the rest is open (section 6).
- **The generic half is jig's while it has one user.** Keyed requests
  until durable, level-triggered watches and the link may move to
  skein, by a rename and a crate move, once a second user wants them
  (client.md, section 18).

## 6. What is open

Each document lists its own; the ones that reach across:

- **Who may see what** among the parties of a project
  (`../../domain/people.md`, section 12);
- **several applications in one client:** one person's inbox across
  applications that share their people. Shaped so that it adds: several
  applications' clients side by side in one host, each its own client
  root, client, link and owners, with no root above them (root.md,
  section 2); nothing in one application's contract changes;
- **bundling** an application's parts into the answer of the jig read
  that names them, to save a round trip (contract.md, section 11);
- **search** over tasks, results, notes and transcripts;
- **notifications beyond the client,** which are an application's
  connectors (`../../domain/people.md`, section 9).

What waits, and for what, is `../../STATUS.md`'s, section 3.
