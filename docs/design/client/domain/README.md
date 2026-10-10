# jig's client domain

Provisional, 2026-10-09. The design of jig's client: the domain a party's
client runs, the pages and cards it shows of jig's primitives, how an
application composes it with its own kinds of object, what it asks of the
engine, its view, and how all of it is tested. jig as a whole is
`../../README.md` (section 10 is the client in one page); the engine's
domain is `../../domain/`. The mechanics are those of skein's
`programming-model.md`, as for every part of jig.

## 1. Reading order

1. **client.md:** the client domain: what it holds, its vocabulary, pages
   and the working set, keyed requests, reads and windows, watches and
   the link, drafts, the small store, signing in. Read it first.
2. **root.md:** what an application's client root does: the one shape,
   the request table, routing, parts and slots, the application's own
   objects and pages, and a service's root.
3. **pages.md:** jig's pages: the frame, the inbox, conversations, the
   task page, a project, the deployment; what an application adds.
4. **objects.md:** jig's objects and their cards: chips, proposals,
   escalations, questions, person tasks, results, refusals, the
   conversation, and the requests that steer a task.
5. **contract.md:** what crosses between a client and the engine: keyed
   requests, documents, reads and watches, versions, and what the
   engine's domain owes the client.
6. **view.md:** the view: a semantic tree, bindings, the diff, words,
   Markdown, conversations, shells.
7. **testing.md:** the fake person, the client's world, the joint world
   with the engine's domain, and their referee.

`../../examples.md` is the example application, `ops`, which each
document uses for its examples, with temper where a second one helps.

## 2. Structure

```
an application's client
<application>-client-domain     the client root: the application's (root.md)
├── jig-client                  jig's client domain: the frame, the link, jig's pages and objects (client.md)
└── <connector>…                each connector's client side: its parts, its families and their pages
<application>-client-view       the view root: jig's view and the application's, its slots filled (view.md)
└── jig-client-view             jig's view: the tree, the builder, the diff, words, jig's pages and cards

skein-lib's request table       keyed requests: the client root's only way out for them (root.md, section 4)

below the domain, with the client's protocol layer
jig-client-wire                 jig's documents, encoded, for both ends (contract.md, section 10)
jig-client-transcript           turns decoded with smith's transcript crate (client.md, section 10)
jig-client-shell-web            the browser's shell (view.md, section 11)
jig-client-shell-native         a shell on skein's HTTP client, for Linux and the worlds

for tests
testing/jig-fake-person         the person: reads the view tree, acts on it (testing.md, section 3)
testing/jig-scripted-engine     the engine as the client's world plays it (testing.md, section 4)
tests/client                    the client's world, and the joint world (testing.md, sections 4 and 5)
```

- **The crate graph is the role graph,** as in the engine. `jig-client`
  depends on skein-lib alone. `jig-client-view` depends on skein-lib and
  `jig-client`. Neither links smith: only `jig-client-transcript`, below
  the domain, does, and it is the client's only use of smith
  (`../../README.md`, section 9).
- **The application owns the roots,** of its domain and of its view, as
  it owns the engine's (root.md). Nothing in jig's client knows an
  application's types.
- **The wire, the protocol layer and the shells** come with the client's
  protocol design, beside the engine's (`../../README.md`, section 12).
  This design says what they owe the domain, and no more. A shell is the
  one impure part of a client, ordinary Rust as skein's shell is, not
  step code.

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
  often, temper where a second example helps.
- **A party** is whoever uses a client: a person, or a service
  (`../../domain/people.md`, section 3). Where only a person makes sense
  (a view, a draft), the documents say person.
- **Each document** says in one page what it is, then its structure, its
  parts, what it owes and is owed below the domain, its world, and what
  is open.

## 4. Names

| Name | What it is |
|---|---|
| client | a program a party reaches the engine with: the application's, around jig's client domain |
| client root | the application's top-level client domain (root.md) |
| view root | the application's top-level view: jig's view and its own (view.md, section 4) |
| shell | the platform's code, the only impure part of a client (view.md, section 11) |
| view | the client domain's state as a tree a shell shows, and a person's acts coming back (view.md) |
| frame | what every page shows: who is signed in, where they are, what needs them, the link (pages.md, section 2) |
| page | a place with an address: the inbox, a task, a project's goals, an application's own |
| object | something a card shows, held once in a page's working set: a task, a proposal, an escalation… (objects.md) |
| card | how one object shows, wherever it appears |
| chip | a task named in passing: its number, its title, its phase, kept live |
| client side | the child of the client root that shows one connector's values, and holds its families (root.md, section 2) |
| family | an application's kind of object, with its own card and pages (root.md, section 7) |
| owner | the child that keeps a value, or that asked for a keyed request: `jig-client`, or a connector's client side; never a project's role, which is always "a project's owner" |
| part | a value of an application's that a jig document names by reference: a connector's name, a resource, a kind, a panel, an effect's description (root.md, section 6) |
| slot | the place a jig page or card leaves for a part, filled by its owner |
| document | a typed value the engine gives a client about one subject, its owner's alone (contract.md, section 5) |
| revision | the state of one thing a party decides about (a hold, a task's spec, a person task, a note), which the decision names (contract.md, 5.2) |
| read | a request for a document, or a page of a list; one answer |
| watch | a live view of a subject: its current document, delivered whenever it has changed, one delivery at a time (contract.md, section 7) |
| keyed request | a party's request: keyed by the client, pending until its answer is durable |
| intent | a request `jig-client` asked the root to send, kept so its card shows it being decided (client.md, section 6) |
| generation | the number of the page at the address, which what a page asked for names, so what comes for an old page is dropped (client.md, section 5) |
| request table | skein-lib's container of keyed requests: the client root's only way out for them (root.md, section 4) |
| link | whether the client reaches the engine: starting, live, behind or offline (client.md, section 8) |
| working set | the objects a page shows, each held once |
| window | a page's bounded view onto a list too long to hold |
| draft | words a person is writing, kept across pages and reloads |
| confirmation | what an act will do, said before it is sent |
| bounds | the engine's facts a draft of an act is compared with, and what happens beyond them, offered for setting a goal and amending (contract.md, 3.2) |
| small store | what a client keeps across its own restart: its requests not yet answered, its drafts (client.md, section 11) |
| kept work | the requests a client restored from its small store, whose asker is gone, counted per party (root.md, section 8) |
| phrasebook | the words a view says, for one language (view.md, section 7) |
| fake person | the person in the client's worlds, who reads the view tree (testing.md, section 3) |
| scripted engine | the engine as the client's world plays it (testing.md, section 4) |

The engine's names (`../../domain/README.md`, section 4) keep their
meaning here: task, goal, chat, proposal, escalation, party, role,
connector, resource.

## 5. Decisions this design takes

Questions `../../README.md` left open, and others met while writing, with
where each is answered:

- **The client root is the application's.** jig's client domain is a
  child of it, beside the application's own kinds of object, as the core
  is a child of the engine's root (root.md, section 2).
- **The client's barrier is skein-lib's.** Every keyed request, jig's and
  the application's, leaves through one generic container of skein-lib's,
  which keeps it until its answer is durable, saved before it is sent,
  sent again with its key within the engine's retention, and only for
  the party it was made for. It hands each answer back with the owner
  the root named for it, so the root routes it to the child that asked
  and keeps no routing of its own. It is to the client what the journal
  is to the engine (root.md, section 4).
- **A document per owner.** jig's documents carry jig's values only; an
  application's value they need is a part, named by reference, and read
  and followed by the application's own child through the application's
  own documents, on the same connection, each naming its scope, a
  project or the deployment: the engine admits the party in that scope,
  and the connector judges what they may see (root.md, section 6;
  contract.md, section 5 and 7.2).
- **How jig's pages show an application's objects:** through slots, which
  jig's pages and cards leave for parts, and which the application's
  roots fill from the part's owner, in the domain and in the view. Each
  connector has one client side, which shows its parts and holds its
  families: the application's own kinds of object, each with its card
  and its pages, shown in jig's slots, as jig's cards show on its pages
  (root.md, sections 2, 6 and 7).
- **Views are level-triggered.** A watch delivers its subject's current
  document whenever it changed since the last delivery, one delivery in
  flight; what changed meanwhile is coalesced, never queued, so a slow
  watcher is never told it missed anything. Only a run's provisional
  text is a stream, read from an offset. This changes the engine's views
  (contract.md, section 7; `../../domain/engine.md`, section 11).
- **What a party reads is durable.** A read's answer and a watch's
  delivery wait, as held outputs, for every commit they reflect, so a
  crash never takes back what a party saw; only a run's text as it is
  written is provisional, and marked so (contract.md, sections 6 and 7).
- **The inbox is the party's, across projects,** filtered by project in
  the client (pages.md, section 3; `../../domain/people.md`, section 6).
- **Decisions name what they saw.** A release names its hold's revision,
  an amendment its task's spec's, an answer its person task's; each
  moves only when that thing changes, and a decision whose revision
  moved is refused as moved, saying what it is now (contract.md,
  5.2).
- **Offers with bounds.** Setting a goal and amending are made, proposed
  or refused by what the person writes, so their documents offer them
  with the engine's facts a draft is compared with, and what happens
  beyond them. The client says which before the request is sent,
  reading the engine's word, never computing authority, and the engine
  judges again when it comes (contract.md, 3.2).
- **Requests the engine lacks:** closing a chat with what becomes of its
  live delegates; amending and releasing as one request; marking seen,
  with a read position for results and one per chat; a person task's
  approval, naming the state it approves; an application's own keyed
  requests, kept by the people child like jig's and by their connector
  (contract.md, section 3; `../../domain/people.md`, 5.1).
- **Turns are decoded below the domain,** by the client's protocol layer,
  with smith's transcript crate, into the client's own turns; the domain
  and the view never link smith (client.md, section 10).
- **The view is semantic and platform-neutral:** a tree shaped like an
  accessibility tree, which each shell maps onto its platform (view.md,
  section 3).
- **Words come from a phrasebook,** one per language: jig ships English,
  and an application may give its own as data (view.md, section 7).
- **An application may draw:** its own kinds of node (a chart, a map),
  whose data its view writes and whose drawing code its shells take,
  each with a fallback in words that every shell, a screen reader and the
  fake person can read (view.md, sections 3 and 11).
- **Home and navigation are the application's choice,** among jig's pages
  and its own (pages.md, section 2).
- **A service uses jig's client domain without a view** (root.md,
  section 11).
- **One person first.** The client is built first for one person, who
  holds the roles they need in the projects they use. What only several
  people raise (who may see what, a role's entries shared and taken) is
  designed where it meets the domain, and waits until several people use
  a deployment.
- **A client outside the engine's versions says so,** and keeps what is
  pending until it is updated (contract.md, section 8).

## 6. What is open

Each document lists its own; the ones that reach across:

- **Who may see what** among the parties of a project, deferred while
  the client is built for one person (`../../domain/people.md`,
  section 12);
- **a text shell** for terminals, from the same tree (view.md,
  section 14);
- **bundling** an application's parts into the answer of the jig read
  that names them, to save a round trip (contract.md, section 11);
- **search** over tasks, results, notes and transcripts;
- **notifications beyond the client,** which are an application's
  connectors (`../../domain/people.md`, section 9).
