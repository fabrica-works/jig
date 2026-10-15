# jig's client domain

Provisional, 2026-10-09, revised 2026-10-15. The design of jig's client:
the domain a party's client runs, the pages and cards it shows of jig's
primitives, how an application composes it with its own kinds of object,
what it asks of the engine, its view, and how all of it is tested. jig as
a whole is `../../README.md` (section 10 is the client in one page); the
engine's domain is `../../domain/`. The mechanics are those of skein's
`programming-model.md`, as for every part of jig.

## 1. Reading order

1. **client.md:** the client domain: what it holds, its vocabulary, pages
   and the working set, keyed requests, reads and windows, watches and
   the link, drafts, the small store, signing in. Read it first.
2. **root.md:** what an application's client root does: the one shape,
   the request table, routing, parts and slots, the application's own
   objects and pages, a service's root, and owners.
3. **pages.md:** jig's pages: the frame, the inbox, conversations, the
   task page, a project, the deployment; what an application adds; the
   frame's parts, the sections, their region names, and arrangements.
4. **objects.md:** jig's objects and their cards: chips, proposals,
   escalations, questions, person tasks, results, refusals, the
   conversation, and the requests that steer a task.
5. **contract.md:** what crosses between a client and the engine: keyed
   requests, documents, reads and watches, versions, and what the
   engine's domain owes the client.
6. **view.md:** jig's view on skein's UI kit (skein's `ui.md`): jig's
   primitives (the frame's parts, the sections, the cards, the
   confirmation, the owed cards, the notices) and jig's default
   arrangement of them, jig's owners, bindings, words, region names and
   tokens, a decision's facts and jig's check, conversations, shells,
   and jig's settings for skein's text shell.
7. **testing.md:** skein's fake person in jig's worlds, the client's
   world, the joint world with the engine's domain, and their referee.

`../../examples.md` is the example application, `ops`, which each
document uses for its examples, with temper where a second one helps.

## 2. Structure

```
an application's client
<application>-domain-client     the client root: the application's (root.md)
├── jig-domain-client           jig's client domain: the frame, the link, jig's pages and objects (client.md)
└── <connector>…                each connector's client side: its parts, its families and their pages
<application>-view              the view root: its tree, from jig's primitives and its own nodes, its slots filled (view.md)
├── jig-view                    jig's view: its primitives and default arrangement, bindings, words, facts, check
└── skein-ui                    skein's UI kit: the tree, the builder, the diff, the view (skein's ui.md)
<application>-shell-web         its browser shell: its start and its loop over skein's browser parts, jig's and its client host (../../protocol/browser.md, section 3)

skein-lib's request table       keyed requests: the client root's only way out for them (root.md, section 4)

below the domain, with the client's protocol layer
jig-protocol-client-wire        jig's documents, encoded, for both ends (contract.md, section 10)
jig-protocol-client-link        the connection to the engine: exchanges, the stream, envelopes (../../protocol/client.md)
jig-protocol-transcript         turns decoded with smith's transcript crate (client.md, section 10)
skein-ui-web                    skein's browser parts, a kit: each usable alone, none running a loop (skein's ui-web.md, 5.1)
jig-shell-web                   jig's browser parts beside them: the sign-in part, the link's adapter, jig's look (../../protocol/browser.md, 5.1)
jig-shell-native                a shell on skein's HTTP client, for Linux and the worlds
skein-ui-text                   skein's text shell: the tree as lines, a line as an act; jig's settings (view.md, section 15)
skein-ui-bundle                 skein's bundler, a tool: the bundle a browser loads (skein's ui-web.md, 4.2)

for tests
skein-fake-person               skein's person: reads the tree, acts on it (testing.md, section 3)
skein-fake-person-browser       skein's face in a browser, and the harness (skein's ui-web.md, 14.2)
testing/jig-fake-person-browser jig's stories for a browser shell, on skein's face and harness (../../protocol/browser.md, section 15)
testing/jig-scripted-engine     the engine as the client's world plays it (testing.md, section 4)
tests/client                    the client's world, and the joint world (testing.md, sections 4 and 5)
```

- **The crate graph is the role graph,** as in the engine.
  `jig-domain-client` depends on skein-lib alone. `jig-view` depends on
  `skein-ui` and `jig-domain-client`. Neither links smith: only
  `jig-protocol-transcript`, below the domain, does, and it is the
  client's only use of smith (`../../README.md`, section 9).
- **The application owns the roots,** of its domain and of its view, as
  it owns the engine's (root.md), and its tree: it builds whatever tree
  it wants with skein-ui, from jig's primitives and its own nodes, and
  jig mandates no node in it (view.md, section 4). Nothing in jig's
  client knows an application's types.
- **The wire, the protocol layer and the shells** are the client's
  protocol's (`../../protocol/client.md`; jig's browser parts and the
  reference shell, `../../protocol/browser.md`), beside the engine's, and
  skein's UI kit's (skein's `ui-text.md` and `ui-web.md`). This design
  says what they owe the domain, and no more, but for jig's settings of
  skein's text shell (view.md, section 15). A shell is the one impure
  part of a client, ordinary Rust as skein's shell is, not step code.

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
| view root | the application's top-level view: its tree, from jig's primitives and its own nodes (view.md, section 4) |
| shell | the platform's code, the only impure part of a client (view.md, section 11) |
| view | the client domain's state as a tree a shell shows, and a person's acts coming back (view.md) |
| frame | the parts jig's pages show around their main: the link, what needs the party, who is signed in, the projects, going to a task, the navigation (pages.md, 12.1) |
| frame's part | one of the frame's, one of jig's primitives with a region name, placed anywhere or nowhere; jig's banner, in jig's default arrangement, holds the first five on every page |
| page | a place with an address: the inbox, a task, a project's goals, an application's own; made of sections, arranged |
| section | one part of one of jig's pages, which the domain opens what for and the view builds whole: a task's plan, the inbox's needs (client.md, 5.1; pages.md, 12.2) |
| arrangement | how a page places jig's primitives and an application's own nodes; jig's pages are jig's default arrangement, and an application's may show more sections together, or be its own (pages.md, 12.3) |
| jig's primitives | the functions of jig's view that build part of a tree or handle its acts: the frame's parts, the sections, the cards, the confirmation, the owed cards, the notices, jig's bindings and their decoders; each placed anywhere or nowhere (view.md, section 4) |
| jig's default arrangement | jig's own arrangement of its primitives: its banner, its pages, `after_page`; a default and a reference, never mandated (pages.md, 12.3) |
| a decision's facts | what a card's promise rests on (the object's state, a refusal and why, what a decision will do and whether it is dangerous), as jig's sayings, from `jig_view::facts`: data an application shows wherever it presents the object (view.md, section 4) |
| jig's check | `jig_view::check`: a deciding act was made on a tree that showed its decision's facts, in the open confirmation or within the acted node's nearest named region; the referee runs it on every tree a person acted on (view.md, section 4; testing.md, section 6) |
| outline | skein-ui's rules for the landmarks a tree keeps: each child of the page node a landmark or a dialog, exactly one main, at most one banner and one content info; jig adds none (skein's `ui.md`, section 14; view.md, 3.1) |
| landmark | a role saying which part of a page a node is: banner, navigation, main, complementary, content info, form, search |
| region name | a region's language-free, stable name, which a stylesheet places it by (skein's `ui.md`, section 8; pages.md, 12.4) |
| object | something a card shows, held once in a page's working set: a task, a proposal, an escalation… (objects.md) |
| card | how one object shows, wherever it appears |
| chip | a task named in passing: its number, its title, its phase, kept live |
| client side | the child of the client root that shows one connector's values, and holds its families (root.md, section 2) |
| family | an application's kind of object, with its own card and pages (root.md, section 7) |
| owner | the child that keeps a value, or that asked for a keyed request: `jig-domain-client`, or one of the application's, numbered (root.md, section 15): a connector's client side; never a project's role, which is always "a project's owner" |
| owner number | the number an application gives one of its owners, shared by both its roots, the wire and the small store; a connector's is its connector's (root.md, section 15) |
| part | a value of an application's that a jig document names by reference: a connector's name, a resource, a kind, a panel, an effect's description (root.md, section 6) |
| slot | the place a jig page or card leaves for a part, or an application's page for one of jig's cards or sections, filled by its owner |
| shortcut | a key a bound node answers to, as a press (view.md, section 5) |
| scene | a drawing as its owner's drawing code draws it: a bounded list of marks in whole pixels, which each shell that draws maps onto its platform (skein's `ui.md`, section 11; view.md, section 17) |
| component, a shell's | a shell's native implementation of a node, chosen from the look by the node's role and one of its owner's style tokens, or by a drawing's kind; the node and its fallback stay in the tree (skein's `ui.md`, section 12). Not a protocol layer's component (`../../protocol/README.md`, section 3) |
| client host | an application's client root, view root and `jig-protocol-client-link`, with the routing between them: one step machine, shared by its shells (`../../protocol/browser.md`, 3.2) |
| browser shell | an application's crate, `<application>-shell-web`: its start and its loop over skein's browser parts, jig's and its client host, copied from jig's reference shell, `jig-test-shell-web` (`../../protocol/browser.md`, 3.1) |
| text shell | skein's shell for a terminal or a pipe, `skein-ui-text`: the tree printed as lines, a typed line read as an act; jig's terminal client gives it jig's settings (skein's `ui-text.md`; view.md, section 15) |
| document | a typed value the engine gives a client about one subject, its owner's alone (contract.md, section 5) |
| revision | the state of one thing a party decides about (a hold, a task's spec, a person task, a note), which the decision names (contract.md, 5.2) |
| read | a request for a document, or a page of a list; one answer |
| watch | a live view of a subject: its current document, delivered whenever it has changed, one delivery at a time (contract.md, section 7) |
| keyed request | a party's request: keyed by the client, pending until its answer is durable |
| intent | a request `jig-domain-client` asked the root to send, kept so its card shows it being decided (client.md, section 6) |
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
| phrasebook | the words a view says, for one language, by skein's machinery (skein's `ui.md`, section 9; view.md, section 7) |
| fake person | skein's person, `skein-fake-person`, in the client's worlds: reads the tree by role, name and text, and acts on it (skein's `ui.md`, 17.1; testing.md, section 3) |
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
- **The view is semantic and platform-neutral:** skein's tree (skein's
  `ui.md`, section 3), shaped like an accessibility tree, which each
  shell maps onto its platform (view.md, section 3).
- **Words come from a phrasebook,** one per language: jig ships English,
  and an application may give its own as data (view.md, section 7).
- **An application may draw:** its own kinds of drawing (a chart, a
  map), whose data its view writes. Its drawing code, the owner's step
  code, gives a scene, which every shell that draws maps onto its
  platform, a browser as SVG by default; a scene is one way to draw, and
  a kind may be a component of the application's instead (skein's
  `ui.md`, sections 11 and 12; skein's `ui-web.md`, 6.1). Each drawing
  has a fallback in words that every shell, a screen reader and the fake
  person can read (view.md, sections 3 and 17).
- **The browser's parts are skein's; the loop is the application's.**
  skein's UI kit gives the browser parts, each usable alone, scenes as
  SVG, the bundler, and the browser tier's face and harness (skein's
  `ui-web.md`). jig writes its sign-in part, its link's adapter and its
  look (`jig.css`, jig's palette, its pictures), the engine's serving
  behind a fixed front door and signing in by cookie, its default codec
  for its places, and jig's stories for a browser shell. An
  application's browser shell, copied from jig's reference shell, runs
  its own loop over the parts and its client host, whose codec parses
  and prints its paths; it gives its look as data, a stylesheet and
  pictures. Past the reference, it changes more a step at a time, each
  needing the ones before only where it says so: its paths, its page,
  its files, its elements and components, its own JavaScript from the
  origin, additions to the policy, a part replaced or left out but the
  front door; a part that holds an invariant, replaced or left out,
  moves it to the application (skein's `ui-web.md`, 3.3 and 5.3;
  `../../protocol/browser.md`, 3.5).
- **Home and navigation are the application's choice,** among jig's pages
  and its own, and in a browser which place `/` is (pages.md, section 2).
- **A service uses jig's client domain without a view** (root.md,
  section 11).
- **One person first.** The client is built first for one person, who
  holds the roles they need in the projects they use. What only several
  people raise (who may see what, a role's entries shared and taken) is
  designed where it meets the domain, and waits until several people use
  a deployment.
- **A client outside the engine's versions says so,** and keeps what is
  pending until it is updated (contract.md, section 8).
- **What jig promises is kept below the tree,** never by a page's
  shape or a node's (pages.md, section 1):
  - *held by construction, whatever the tree:* a deciding act goes
    through `jig-domain-client`'s confirmation (client.md, section 9) and
    names the revision of the data its binding was issued from, and a
    moved revision is refused (contract.md, 5.2); made once, nothing
    lost and nothing shown done before durable are the request table's
    and the engine's;
  - *exported as data:* a decision's facts, jig's sayings, which jig's
    cards show and an application shows wherever it presents the object
    itself (view.md, section 4);
  - *checked, not mandated:* jig's check, which the referee runs in jig's
    worlds and in an application's, never at run (testing.md, sections 6
    and 8).
- **jig's view is primitives; nothing in the tree is mandated.** The
  frame's parts, the sections, the cards, the confirmation, the owed
  cards and the notices are functions of jig's view, each placed
  anywhere or nowhere; jig's pages, its banner and `after_page` are its
  default arrangement of them and references, which an application
  takes as they are, rearranges, or replaces. An application's page
  shows jig's sections in slots, as it shows jig's cards, or presents
  jig's objects itself, from jig's documents, binding its nodes through
  jig's bindings (view.md, section 4; pages.md, section 12; root.md,
  section 7). What a page opens is what its sections need (client.md,
  5.1).
- **Layout as semantics.** Every tree keeps skein-ui's outline of
  landmarks (skein's `ui.md`, section 14), to which jig adds no rule, and
  jig's regions carry stable region names, so a shell and a stylesheet
  place them without depending on the order of nodes; the tree still
  says where nothing goes (view.md, section 3).
- **The tree's closed sets are skein's,** grown by skein's criterion
  through its light path (skein's `ui.md`, sections 3 and 4): jig's view
  uses them and adds none, and asks for a role, a state or a property as
  any application does.
- **Owners, not connectors.** Everything that names an application's
  owner, as root.md, section 15, lists (parts, the view's slots,
  bindings, styles, drawings and region names, requests, reads and
  subjects), names it by a number the application's roots share, a
  connector's being its connector's, so a family that belongs to no
  connector needs no change of what an application writes. In the tree,
  an owner is skein-ui's opaque number (skein's `ui.md`, 3.3), which
  jig's mapping gives: an application's owner its own number, and jig
  65536, one past every `u16`, so the two never meet (view.md, section
  3; root.md, section 15).
- **The text shell is skein's:** printing a tree and reading a line
  decide nothing of jig's or an application's (skein's `ui-text.md`);
  jig's terminal client composes it with jig's settings, its status
  region `jig-link` (view.md, section 15).

## 6. What is open

Each document lists its own; the ones that reach across:

- **Who may see what** among the parties of a project, deferred while
  the client is built for one person (`../../domain/people.md`,
  section 12);
- **several applications in one client:** one person's inbox across
  applications that share their people. Shaped so that it adds: several
  applications' clients side by side in one shell, each its own client
  root, view, tree, link and owners, with no root above them (root.md,
  section 2), the shell showing their trees together; nothing in one
  application's contract changes. It waits for applications that share
  their people;
- **bundling** an application's parts into the answer of the jig read
  that names them, to save a round trip (contract.md, section 11);
- **search** over tasks, results, notes and transcripts;
- **notifications beyond the client,** which are an application's
  connectors (`../../domain/people.md`, section 9).
