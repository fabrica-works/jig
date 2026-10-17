# The application's client root

Provisional, 2026-10-09, revised 2026-10-15. What an application's
client root is and does: the top-level domain of every client of an
application on jig, its root domain (programming-model.md, 4.5), which
holds jig's client, `jig-client` (client.md), as a component, beside
the application's own children: each owner's client side with its
families, and its presentation, jig's reference UI or its own, or none.
It says what the root carries, its one shape, how it routes what the
client tells it, how jig's documents name an
application's values and how its families ride the client, what a root
never does, a service's root, the reference root, and the owners every
value and family is named by (section 15). It is the client's
counterpart of the engine's root (`../../domain/root.md`), and follows
the same rule: the root routes and translates; it never decides or
orders. What is still open is listed in section 14.

## 1. In one page

- **The application owns the client's root,** as it owns the engine's
  root. The root holds `jig-client` and its own children: each owner's
  client side, holding its families; and its presentation, built on
  skein's UI kit or anything else, jig's reference UI, or none for a
  service (section 2).
- **The client is a component, not a child to route.** `jig-client`
  holds the link, the watches by handle, the request table and the work
  kept across a restart (client.md). The root steps it with its
  children's asks and the transport's events, and routes what it tells
  back. No root routes a child of jig's, or a request table.
- **The root routes and translates; it never decides or orders.** What
  makes jig's promises to a party true (a request made once, answered
  once durable, kept across a restart; the link judged once for every
  child) is the client's, and below it the engine's.
- **One shape** (section 3): every ask of a child into the client; every
  request the client gives the transport out, in its order; everything
  the client tells the application routed by its asker.
- **The owner keeps the value** (section 6), as in the engine
  (`../../domain/core.md`, section 5). jig's documents name an
  application's values as parts, by their owner's number; the owner's
  client side reads and watches them through the application's own
  subjects, and its presentation shows them.
- **Families as bytes.** An application's requests, reads and subjects
  ride the client as bytes under their owner's number and version, on
  the same connection and through the same request table; its wire
  crate encodes and decodes them (section 7).
- **Owners, numbered alike.** Each child of the root that asks the client
  or keeps values is an owner, numbered in one enum the application's
  engine's root and its client root share. A connector's client side is
  one kind of owner, numbered as its connector is (section 15).
- **Copied, not invented.** jig's test application's client root, with
  the reference UI, is the reference an application's root is copied
  from (section 12).

## 2. What a client root is

```
<application>-client-domain     the client root: the client, its children, routing, translation
├── jig_client::Client          jig's client, a component (client.md)
├── jig-client-view             jig's reference UI, if the application presents with it (pages.md, view.md)
├── <owner>…                    each owner's client side: a connector's parts, its families and their presentation
└── …                           anything else of the application's, such as its own presentation of jig's objects
```

- **Its state:** the client, its children's states, and nothing else
  that outlives a step.
- **Its vocabulary** is what crosses its boundary: the host's events (the
  start, the transport's replies and stream, the small store written,
  time, and a presentation's own, such as a place gone to or a person's
  act) and its requests to the host (the client's to the transport, and
  its children's own, such as a place shown or a record kept). It is the
  union of the client's and its children's, wrapped: an exhaustive enum
  per direction, with one variant per child.
- **Its children are siblings,** sharing no types (programming-model.md,
  4.5). A child reaches jig's engine only through the client, by the
  root: it asks by value, and reads the documents of its own handles,
  which the root lends it (section 5).
- **One child per connector.** Each connector of the engine's root has
  one child in the client root, its client side, an owner: it shows the
  connector's parts, and holds the families of objects that are the
  connector's (for `ops`, infrastructure's side holds environments). A
  family that spans connectors (`ops`'s incidents, an observability
  matter led by a task) belongs to the one whose objects it lists, and
  shows the others' values through their parts.
- **The presentation is the application's.** Its own, built on skein's
  UI kit or on anything else; jig's reference UI, `jig-client-view`,
  held as a child; or none, for a service. Whatever it is, it reaches
  jig's documents and operations only through the client, as any child
  does.
- **No root above it.** A client and an engine in one process (a desktop
  application that runs its own engine, a test) keep their own client
  root and engine root, joined by a transport of the application's: an
  in-process channel carrying jig's messages to the clients' component
  (client.md, section 16; `../../protocol/service.md`, section 10).

## 3. The one shape

```
step(root, event):
    an event of the transport's or the store's     → client.step(now, event, out)           (client.md, 4.1)
    any other event                                → its child, by its owner                 (section 5)
    a child's ask (watch, read, send, sign in)     → client.step(now, ask, out)
    for each request the client gives, in order:
        an exchange, the stream, a save or an erase → out, wrapped, in that order
        a link's state                              → every child
        changed, ended, a page, a text              → the child its handle's asker names
        pending, answered, kept                     → the child its key's asker names
        kept records dropped, how many              → the child that says so                 (section 8)
    a child's own requests (a place shown, its records) → out, wrapped

fire(root, now):
    client.fire(now, out), routed as above; each child's deadlines
```

- **No way around the client.** A child's entry points take no path to
  the engine but the asks the root gives the client: a keyed request
  leaves only through the client's request table. A root cannot send a
  keyed request unsaved without writing code to get around the client.
- **Order is the client's.** The client gives a request's record before
  its exchange, and sends it only once the record is stored (client.md,
  section 11); the root passes the client's requests out in the order
  the client gives them, and never reorders them.
- **The documents are the client's.** A child that watches a subject
  reads its latest document from the client, by its handle, and keeps
  no copy of it across steps (section 5).

## 4. The request table

The request table is the client's: `jig-client` holds skein-lib's
`RequestTable` (skein's `lib.md`, section 12), and keeps, sends and
retires every keyed request, jig's and the application's alike
(client.md, section 6). No root routes it, holds it, or wraps its
records: a child asks the client to send, and the root routes each
request's progress and answer by its key's asker (section 5).

## 5. Routing

- **By asker.** Each handle and each key a child gives the client
  carries its asker, the child's owner number (section 15), which the
  client hands back with everything it says of that handle or key and
  never reads (client.md, section 4). The root matches the asker to its
  child, exhaustively, and keeps no map from handles or keys to
  children.
- **Kept work by asker too.** A kept request comes back after a restart
  under its key, with the asker that sent it (client.md, section 11), and
  the root routes its `Kept`, its progress and its answer to that child.
  An asker that names no owner of the build is a fact, never a guess:
  its kept requests are sent and answered as any, and what the client
  tells of them reaches no child.
- **The link to everyone.** The client judges the link (client.md, 8.2);
  the root passes each `Link(state)` to every child, which shows it, or
  not, as its presentation decides.
- **The documents, lent.** A child reads the documents of its own handles
  from the client, `client.document(handle)`, which the root lends it,
  read-only, within the step. A child that needs who is signed in, or
  the projects they hold roles in, watches `me` by a handle of its own,
  which shares the client's own watch of it (client.md, 8.2).
- **Places and acts by owner.** Where a presentation has places (a
  browser's paths, a terminal's commands) and acts, the root routes a
  place gone to, and an act, to the child that owns it, and tells every
  other child it left whatever place it held. With the reference UI,
  jig's places are its own, under the codec and arrangement the
  application gives it (pages.md, section 2).

## 6. Parts and slots

How jig's documents name an application's values.

- **A part** is a value of an application's that a jig document names by
  reference. Its reference is jig's: what kind of part it is, its owner
  (the connector whose value it is, for every part jig's documents name,
  by its owner number: section 15), and the key jig knows it by. The
  kinds:

  | Part | Named by | What it shows, for `ops` |
  |---|---|---|
  | connector | a connector | its name, where jig names it: "closing: 2 delegates, and infrastructure's effects" |
  | resource | a connector and a path | "the checkout service in production" |
  | kind | a connector and an effect's or a read's kind | "restart", "scale" |
  | panel | a connector and a task | a remediation's way to healthy; an environment's readiness |
  | result | a connector and a task | what a procedure produced: "environment staging-7, ready, until Friday" |
  | effect | a connector and a proposal | what a proposed restart would do, and where |
  | outbox entry | a connector and the entry's number | an effect asked for, made or in flight, apart from any proposal's description: "restart of checkout, in flight" |
  | hold | a connector and a held task | an account of drift or of a failed effect: "deleted by hand, at 10:04, by mallory" |
  | project | a connector and a project | what a connector says of a project: a resource's health, its adoption |
  | deployment | a connector | what a connector says of itself: its outbox, its request budget |

- **Showing a part.** A presentation that shows one of jig's documents
  hands each part it names to the part's owner, the client side of its
  connector, with the scope it shows in: a project, or the deployment.
  The owner reads or watches the part's value through the application's
  own subjects, naming that scope (contract.md, 7.5), and holds it while
  it is shown. A connector, a resource or a kind needs no document: its
  words are its owner's, from its number or its path.
- **A slot** is where the reference UI shows a part: its pages and cards
  leave one for each part they name, which the part's owner's view fills
  (view.md, section 4); an application's page with the reference UI
  leaves the other kinds of slot for its cards and sections (section 7).
  A part not yet read shows as loading, and a part its owner cannot show
  says so; neither breaks the page.
- **A part lasts as long as its owner keeps it.** A part's value is its
  owner's, kept as long as the owner's records keep it (a procedure's
  result, while its connector keeps the procedure's record). Past that
  the part is unknown, and says so, as a part its owner does not know.
- **A part's acts are its owner's.** A button in a panel (silence this
  alert, extend this environment) is its owner's, handled by its owner,
  whose keyed request leaves through the client like any other.
- **A form for an application's values.** Where one of jig's acts needs
  an application's values (a procedure's parameters, amended; a
  resource's options, adopted), the act is the application's own
  request: its form is the owner's, its request reaches the connector
  it concerns, and what that changes in the core is a request of jig's,
  made by the engine's root on the party's behalf
  (`../../domain/people.md`, 5.1). No operation of jig's carries an
  application's value.
- **An owner not shown is a build error.** The root's match on a part's
  owner is exhaustive, as the engine's root's match on connectors is
  (`../../README.md`, section 4).

## 7. The application's families

- **A family** is an application's kind of object: for `ops`, incidents
  and environments. Each belongs to one owner, the client side of one
  connector (section 15), with:
  - its objects and their presentation, the owner's;
  - its requests: keyed, through the client, as bytes under the owner's
    number and its wire's version, admitted by the people child and
    routed by the engine's root to the connector they concern
    (`../../domain/people.md`, 5.1; contract.md, 3.3);
  - its reads and watches, of the application's own subjects, as bytes
    under the owner's number, each naming its scope (a project, the
    deployment, or the party's own, admitted on the credential alone),
    in which the connector judges what the party may see (contract.md,
    7.5);
  - its records, in the application's wire crate,
    `<application>-client-protocol-wire`, which encodes what the owner
    asks and decodes what the client hands back
    (`../../protocol/client.md`, section 11).
- **The client never reads a family's bytes.** It reads their owner,
  their version and their size, keeps a family's requests in its table
  as it keeps jig's, and hands back its documents, pages and outcomes as
  bytes, to the asker that asked (client.md, section 4).
- **An application's page shows jig's objects** from jig's documents,
  which its child watches by handles of its own: an incident's page
  shows its lead task's chip and the proposals waiting in its subtree.
  With the reference UI, it tells the reference UI which of jig's
  objects and sections the page names (`Named`, view.md, 18.1), and
  shows the reference UI's cards and sections in slots (pages.md,
  section 9; view.md, section 4); presenting them
  itself, it decides through the same operations, naming the revisions
  its documents gave.
- **A family follows jig's rules for what it shows:** nothing shown done
  before its answer; acts naming the revision they saw; provisional
  values marked. Its world checks it, with jig's referee (testing.md,
  section 6).
- **An application's streams** (its data followed from an offset,
  provisional, never committed state) are designed in the pattern of a
  run's text (contract.md, 7.4; client.md, 8.3), and wait for an
  application whose people follow its data as it comes.

## 8. The small store, composed

- **The client's records are the client's:** its `Save` and `Erase`
  carry the request table's records, jig's requests and the families'
  alike, which the root passes to the host as they come (client.md,
  section 11). The root neither wraps nor reads them.
- **Each child's own records,** a presentation's (the reference UI's
  drafts and its place) and a family's, are the child's, keyed by its
  own keys and wrapped by the root with the child's variant, as the
  engine's root wraps its children's records (`../../domain/root.md`,
  section 6). They ride the host's small store beside the client's, in
  the order the step gives them.
- **Versioned,** each by its owner, so a client updated since reads what
  the older one kept.
- **Restored at start** by the host: the client's records to the client,
  in `Start { kept }`, before its first step; each child's to the child.
- **Whose work is kept.** For each request restored, the client tells
  `Kept { key, request }`, which the root routes to the child its key's
  asker names. Whether to warn of someone else's kept work is that
  child's presentation's to decide (the reference UI's: pages.md,
  section 8); the root passes on what the client kept, and decides
  nothing.
- **Kept work dropped.** The records the client could not restore are
  told only by how many, `Dropped { count }`, under no asker (client.md,
  section 11). The root gives the count to one child, which says so:
  the reference UI, where the application has it (view.md, 18.7);
  otherwise the presentation the application chooses, or none, in a
  service (section 11).

## 9. The view, composed

The presentation is the application's (client.md, section 1): a view
root on skein's UI kit, a terminal's lines, a chat platform's messages,
or none. An application that presents with skein's UI kit and the
reference UI has a view root, `<application>-client-view`, which mirrors
the client root: it builds the page's tree in the arrangement it
chooses, from the reference UI's primitives, its own nodes and the
page's owner's view; fills each slot from the slot's owner's view; decodes an act
through the node owner's bindings, wrapped with its owner, for the root
to route; resolves a link node's place through its binding's owner's
view; and routes `draw` by a drawing kind's owner, exhaustively (view.md,
sections 4, 5 and 17).

## 10. What a root never does

- send a keyed request around the client, or answer one itself;
- reorder what the client gives the transport;
- decide whether an answer is durable, when to retry, or whether the
  link is live;
- compute what a party may do: what a presentation offers is what the
  engine's documents say;
- read a part's value, or a family's bytes, meant for another child, or
  hold one in its own state;
- keep anything across steps but the client and its children's states:
  which child asked is the asker its handles and keys carry, never a
  map of the root's own;
- match on a child's vocabulary, or on an asker, with a `_` arm.

## 11. A service's root

A service is a party with no view and no person
(`../../domain/people.md`, section 3): a deploy pipeline that sets
goals, a monitor that answers a person task addressed to its role. Its
root holds `jig-client` in the client's service mode (client.md, section
20): the service's own logic as its children, asking by value and
reading the documents of its handles; no presentation; `SignIn { with:
Service(credential) }` at start; and its small store a file. The same
promises hold: each request made once, answered once durable, and the
service told why when refused. A service's root waits for an application
whose services act as parties.

## 12. The reference root

jig's test application's client root, `jig-client-test-domain`, is the
reference: written to be copied, and commented as such. It holds
`jig-client`, the reference UI, and its test family's client sides, and
its crate has one module per part of this document:

```
<application>-client-domain/src/
├── lib.rs          the tree, re-exports
├── boundary.rs     the root's vocabulary: events in, requests out, one variant per child
├── owner.rs        the application's owners and their numbers, which are its askers (section 15)
├── domain.rs       Root, step, fire: the one shape (section 3)
├── route.rs        the child each event, act and word of the client's is routed to, by owner and asker (section 5)
├── parts.rs        parts to their owners' sides (section 6)
├── store.rs        the children's records, wrapped (section 8)
├── limits.rs       the client's limits, the children's, and their sum
└── tests.rs
```

Its view root, `jig-client-test-view`, has the same shape for the view
(view.md, section 4). A new application copies both, puts its own client
sides in place of the test family's, and lets the compiler list the arms
to fill. The host around them, which steps the root for every shell and
carries the client's messages over jig's default HTTP mapping through
`jig-client-protocol-http` (`../../protocol/client.md`, section 17), is
copied from jig's test application's, as
its binary and its service are (`../../protocol/browser.md`, 3.2;
`../../protocol/service.md`, section 7); so is its browser shell, from
`jig-client-test-shell-web` (`../../protocol/browser.md`, 3.1). `ops`'s
client root, `jig-ops-client-domain`, and its view root,
`jig-ops-client-view`, are such copies (`../../examples.md`, section 8).

## 13. The world

The root has no world of its own beyond its application's: the
application's client worlds run its whole client root, its client and its
presentation, on jig's client world with jig's scripted engine and the
application's scripted subjects, and in the joint world with its
engine's domain (testing.md). jig's referee checks there that the root
kept every promise above, from what the client told the application and
what the engine made durable.

## 14. Open questions

- **Generating the boilerplate:** whether the client root's routing and
  wrapping, which have one shape, are best copied, as they are, or
  generated from a short description, as for the engine's root
  (`../../domain/root.md`, section 14).

## 15. Owners

- **An owner** is a child of the client root that keeps values, names
  nodes of a view, or asks the client for anything: jig's reference UI,
  or one of the application's. Each of the application's is a
  connector's client side, or another child of the root (its own
  presentation of jig's objects, a service's logic, a settings page of
  its own), which may ask the client for jig's operations, reads and
  subjects, but sends no request, read or subject of its own records to
  the engine: one that does is an own child domain's client side, which
  waits for the application that needs it (15.1).
- **jig takes no application number.** The reference UI's view names it
  `Owner::Jig`, and the wire names jig's records as jig's.
- **The application numbers its owners,** each by a `u16`, in one typed
  enum its engine's root and its client root share, with the function
  that gives an owner's number and the one that gives the owner of a
  number, or none. An application whose owners are all its connectors'
  client sides shares the enum of its connectors, as `ops` shares
  `jig-ops-domain-connectors`'s `Connector`; one with another owner adds
  it to that enum. A connector's owner number is the number the engine's
  root gives that connector in the core's configuration
  (`../../domain/root.md`, section 6); any other owner has a number no
  connector has.
- **One number, everywhere the client names the owner:** a part's owner
  (contract.md, 5.3); the owner of a view's key, style token, region
  name, binding, slot, drawing, picture and phrase
  (`jig_client_view::Owner::Application { number }`, view.md, section
  3); an application's keyed requests, reads and subjects (contract.md,
  3.3, 6 and 7.1), which the client carries as bytes under that number
  and the wire tags with it (`../../protocol/client.md`, section 11);
  and the asker of each handle and key the owner gives the client
  (client.md, section 4), which the client's small store keeps with each
  request (client.md, section 11).
- **Never reused.** A number given to an owner is never given to
  another, since the client's small store keeps the requests asked under
  it across the client's updates, and a request sent under it may be
  answered after one.
- **Matched whole.** Each client root's and engine root's match on an
  owner is exhaustive, with no `_` arm, so a new owner breaks the build
  where it must be routed and shown; a number that names no owner is a
  fact, never a guess.
- **Onto skein-ui's owner, and the client's asker.** skein-ui's owner,
  `skein_ui::Owner { number: u32 }`, and the client's asker are opaque
  numbers, compared and never interpreted (skein's `ui.md`, 3.3;
  client.md, section 4). jig's typed owner maps onto both by one total
  function: `Application { number }` to the same number, and `Jig` to
  65536, one past every `u16`, so jig's and an application's never
  meet; and back by a partial one, which gives none for a number that
  names no owner (view.md, section 3). The root routes what the client
  hands back by an asker, and the view root what skein-ui hands back by
  an owner (an act's binding, a slot, a drawing's kind), through that
  partial function, then matches the typed owner whole.

### 15.1 Later: an application's own child domains

An application's own child domain in the engine (`../../README.md`,
section 4), whose families belong to no connector (a laboratory's
evidence and verdicts), is an owner like a connector: numbered in the
same enum, with a number no connector has; its client side a child of
the client root; its families' requests, reads and subjects naming its
number, which the engine's root routes to that child domain as it routes
a connector's. Nothing in parts, the client, slots, bindings, styles,
drawings or region names changes for it, and nothing in what an
application already writes. It waits for an application with such a
domain: the engine's root's routing of an application's requests, reads
and subjects to a child that is not a connector, and the people child's
admission of them (`../../domain/people.md`, 5.1), are designed then.
