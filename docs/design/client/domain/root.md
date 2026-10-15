# The application's client root

Provisional, 2026-10-09, revised 2026-10-15. What an application's
client root is and does: the top-level domain of every client of an
application on jig, which composes jig's client domain with the
application's own kinds of object. It says what the root carries, its
one shape, what jig asks of skein-lib's request table, how the root
routes, how jig's pages show an application's values and an
application's pages show jig's objects and sections, what a root must
never do, and the owners every value and request is routed by
(section 15). It is the client's counterpart of the engine's
root (`../../domain/root.md`), and follows the same rule: the root routes
and translates; it never decides or orders. What is still open is listed
in section 14.

## 1. In one page

- **The application owns the client's root,** as it owns the engine's.
  The root composes `jig-domain-client` (client.md) with the client's
  side of each of the application's connectors: a child domain that
  shows the connector's values, and holds the application's own kinds of
  object, families, with their cards, their pages and their requests. It
  alone faces the client's protocol layer and the shell.
- **The root routes and translates; it never decides or orders.** What
  makes jig's promises to a party true (a request made once, shown as
  asked until durable, kept across a restart; the link judged once for
  every child) is decided by `jig-domain-client` or kept by the request
  table. The root carries it.
- **One shape** (section 3): every event routed to its child; every keyed
  request into the table; every read, watch and address out; the table's
  sends out after the small store is written.
- **The request table is skein-lib's** (section 4): a generic container
  of keyed requests that keys each, keeps it in the small store before it
  is sent, sends it again with its key until its answer is durable, and
  parks it while the engine is out of reach. It is the root's only way
  out for keyed requests, as the journal is the engine's root's only way
  out (`../../README.md`, 6.2). It hands each request's progress and
  answer back with the owner the root named for it, so the root routes
  them to the child that asked.
- **The owner keeps the value** (section 6), as in the engine
  (`../../domain/core.md`, section 5). jig's documents name an
  application's values as parts; jig's pages and cards leave slots for
  them; the root routes each part to the child that owns it, which reads
  and watches it through the application's own documents, and whose view
  fills the slot.
- **Both ways.** An application's page shows jig's objects (an
  incident's proposals, its task's chip) and jig's sections (its lead
  task's plan) through slots, which `jig-domain-client` and jig's view
  fill (section 7).
- **Owners, numbered alike.** Each child that keeps values is an owner:
  `jig-domain-client`, and the application's, which it numbers in one
  enum both its roots share. A connector's client side is one kind of
  owner, numbered as its connector is (section 15).
- **Copied, not invented.** `ops`'s client root is the reference every
  client root is copied from (section 12).

## 2. What a client root is

```
<application>-domain-client     the client root: the request table, routing, translation
├── jig-domain-client           jig's client domain (client.md)
├── <connector>…                the client's side of each connector, an owner: its parts, its families and their pages
└── …                           anything else of the application's, such as a settings page of its own
```

- **Its state:** its children's states and the request table, and
  nothing else that outlives a step.
- **Its vocabulary** is what crosses its boundary: the shell's events
  (the start, the place gone to, time), the view's actions, the
  protocol's answers, documents and watches' deliveries; and its
  requests to each. It is the union of its children's, wrapped: an
  exhaustive enum per direction, with one variant per child.
- **Its children are siblings,** sharing no types (programming-model.md,
  4.5). The application's children know nothing of `jig-domain-client`,
  and `jig-domain-client` knows them only as parts' owners and as
  elsewhere.
- **One child per owner.** Each connector of the engine's root has one
  child in the client root, its client side, an owner: it shows the
  connector's parts, and holds the families of objects that are the
  connector's (for `ops`, infrastructure's side holds environments). A
  family that spans connectors (`ops`'s incidents, an observability
  matter led by a task) belongs to the one whose objects it lists, and
  shows the others' values through their parts.
- **Owners are numbered alike at both ends.** A part names its owner by
  the owner's number, which for a connector is the number the engine's
  root gives it. Both of the application's roots share one typed enum of
  its owners with their numbers, so the client root's match from a part
  to its child is exhaustive, and a new owner breaks the client's build
  where it must be shown (section 15).

- **No root above it.** A client and an engine in one process (a
  desktop application that runs its own engine, a test) keep their own
  roots, each with its protocol layer, joined by bytes over an in-process
  address, as an engine and its workers are
  (`../../protocol/service.md`, section 10).

## 3. The one shape

```
step(root, event):
    route the event to its child, by its owner                         (section 5)
    while a child asks for a hand-off (a part, an object or a section to show),
        route it to the child that owns it, and its answer back         (sections 6 and 7)
    every keyed request any child asks for       → table.ask(owner, ask)  (section 4)
    every read, watch, address, sign-in          → out, wrapped
    every record a child saves or erases         → out, wrapped, as the small store's
    every change of the link                     → to every child, and to the table

iterate, after the step:
    table.take(out)  → its records saved first, then its sends, in order, within a bound
    an answer        → table.answered(attempt, envelope): a final one retired, and handed,
                       unopened, to the child its owner names; the rest sent again or parked
    a progress       → to the child its owner names
    the table's deadlines fired → its retries sent
```

- **No outbound queue for keyed requests.** A child's entry points take no
  path to the engine but the table's: what leaves as a keyed request is
  what `iterate` takes from it. A root cannot send a keyed request
  unsaved without writing code to get around the table.
- **Reads and watches leave at once:** they decide nothing, and the
  engine answers them only with what is durable (contract.md, sections 6
  and 7). Neither is keyed nor saved.
- **Saves before sends.** Within one step's outputs, the small store's
  writes come first, and the shell applies them before any request
  leaves (client.md, section 11). The table emits its own records with
  its sends, in that order, so the root never orders them.

## 4. The request table

The client's barrier, in skein-lib, generic over the requests it holds
and the owner its asker names for each, as the journal is over the
writes and outputs it holds (`../../README.md`, 6.2). jig cannot keep
it: `jig-domain-client` may not hold the application's requests, and its
code may not be generic. It is not policy either: it is a data structure
with a lifecycle, as a queue is. Its design is a section of skein's
`lib.md`, written before its first jig consumer.

- **What it does:**
  - **a key per request,** drawn from the client's seed, which is fresh at
    every start, and never reused for another;
  - **a scope per request:** an opaque number its asker gives, which
    `jig-domain-client` names: the party the request is for. Only the
    requests of the party confirmed are ever sent (client.md,
    section 11);
  - **an owner per request:** a value of the root's own, which the
    table holds, hands back with the request's progress and its final
    answer, and never reads. It is an enum of the root's: one variant
    per child, carrying that child's token for the request, and one,
    kept, for a request restored from the small store whose asker is
    gone, carrying its scope. The root routes by matching on it,
    exhaustively, and keeps no table of its own from requests to
    children;
  - **kept before sent:** each request asked emits its record for the
    small store (its key, its scope, the request, and the wall time it
    was first sent), then its send; each request retired erases it;
  - **one attempt at a time,** each with its own token for the answer;
  - **the answer as an envelope** the protocol layer made from what came
    (contract.md, section 4), which the table reads and the root never
    opens:
    - *final:* the engine's durable answer, done or refused, in the
      owner's vocabulary: retired, and handed back with its owner, so the
      root hands it, unread, to the child that asked, under its token;
    - *again:* busy or not ready; sent again under its key after a
      backoff, jittered;
    - *lost:* no answer came; sent again under its key after a backoff
      while the link is live, and parked while it is not;
    - *signed out:* parked until its party is confirmed again;
  - **park and resume:** told the link is down, it sends nothing; told it
    is back, it sends the parked requests of the confirmed party; told a
    party is confirmed, it may send that party's, and no other's;
  - **a retention per request:** the engine keeps a request's answer
    under its key only for a retention, which the frame says
    (contract.md, 3.1). A request first sent longer ago than that, less
    a margin, is never sent again: it is retired, and its owner told it
    is unknown whether it was made;
  - **restored at start,** from the small store, each with the kept
    owner of its scope, parked until a party is confirmed. A kept
    request has no card to show it: when it is retired, answered or
    unknown, the root hands its answer to no child, and tells
    `jig-domain-client` one of that scope's kept requests is gone
    (section 8);
  - **progress for display:** in flight, retrying, parked, or unknown,
    handed to the request's owner as it changes, so its card can say so;
  - **bounded:** a limit on requests held, and on bytes; a request past
    it is refused at `ask`, and its owner shows that;
  - **its worst case,** priced like every lib container.
- **What it does not do:** read a request, an answer or an owner.
  Whoever owns the bytes checks their bounds before asking. Judging the
  link, and which party is confirmed, are `jig-domain-client`'s
  (client.md, sections 8 and 11); the root passes both on.
- **It keeps to the subset,** as the journal does: generic types and
  lib's own containers, no traits, no closures.

## 5. Routing

- **By owner.** Every token a child issued comes back with its terminal,
  wrapped with the child's variant: a read's, a watch's, a request's. The
  root unwraps it and hands it to that child. An event from below never
  needs the root to look inside it.
- **Addresses by owner.** The root's address is the union of its
  children's: jig's (the inbox, a task, a project's section) and each
  family's (an incident, an environment). A place gone to is routed to
  its owner, which opens its page, and every other child is told it left
  whatever page it had (client.md, section 5). Below the domain, the
  application's client host prints and parses every owner's addresses
  as paths, with its own codec: for jig's, jig's default codec,
  `jig-protocol-client-link`'s `address` module, as is, under a prefix,
  renamed, or replaced; for its families', its own. Every path is the
  application's but those under `/jig/`, the engine's own
  (`../../protocol/browser.md`, section 9).
- **Acts by owner.** The view decodes a person's act into the root's
  action, already wrapped with the owner of the node acted on
  (view.md, section 5); the root hands it to that child.
- **The link to everyone.** `jig-domain-client` judges the link, from the
  frame's watch (client.md, 8.2); the root passes each change to every
  other child, which opens its watches while it is live and reopens them
  when it comes back, and to the request table, which parks or resumes.
  The party confirmed goes to the table likewise.
- **Behind is the page's owner's.** Whoever owns the page at the address
  says whether its watches are behind; the frame shows it.
- **The frame to everyone who asks.** A child that needs to know who is
  signed in, or which project the person is in, asks `jig-domain-client`
  through the root, as a hand-off answered within the step.

## 6. Parts and slots

How jig's pages show an application's values.

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
  | hold | a connector and a held task | what drift or a failed effect was: "deleted by hand, at 10:04, by mallory" |
  | project | a connector and a project | what a connector says of a project: a resource's health, its adoption |
  | deployment | a connector | what a connector says of itself: its outbox, its request budget |

- **A slot** is where one of jig's pages or cards shows a part. The page
  or card names it, and jig's view leaves an empty node for it
  (view.md, section 4). It is one of three kinds of slot: an
  application's page leaves the other two for jig's cards and sections
  (section 7).
- **Showing a part.** When a page of `jig-domain-client` shows a part,
  it asks the root, with the page's generation and the part's scope: the
  project it shows in, or the deployment. The root routes it to the
  part's owner, the client side of its connector, which holds it in its
  own working set, reads or watches its value through the application's own
  documents, naming that scope (contract.md, section 5 and 7.2), and
  forgets it when told the page's parts have left. A connector, a
  resource or a kind needs no document: its words are the owner's
  view's, from its number or its path.
- **Filling a slot.** The view root asks the owner's view to build each
  slot's nodes, from the owner's state, and jig's view splices them in
  (view.md, section 4). A part not yet read shows as loading; a part its
  owner cannot show says so. Neither breaks jig's page.
- **A part lasts as long as its owner keeps it.** A part's value is its
  owner's, kept as long as the owner's records keep it (a procedure's
  result, while its connector keeps the procedure's record). Past that
  the part is unknown, and its slot says so, as for a part its owner
  does not know.
- **A part's acts are its owner's.** A button in a panel (silence this
  alert, extend this environment) is its owner's binding, decoded by its
  owner's view and handled by its owner, whose keyed request goes
  through the table like any other.
- **A form in a slot.** Where one of jig's acts needs an application's
  values (a procedure's parameters, amended; a resource's options,
  adopted), the act is the application's own request: its form is a
  part, its request goes to the connector it concerns, and what that
  changes in the core is a request of jig's, made by the engine's root on
  the party's behalf (`../../domain/people.md`, 5.1). No request of
  `jig-domain-client`'s carries an application's value.
- **An owner not shown is a build error.** The root's match on a part's
  owner is exhaustive, as the engine's root's match on connectors is
  (`../../README.md`, section 4).

## 7. The application's families

- **A family** is an application's kind of object: for `ops`, incidents
  and environments. Each belongs to one owner, the client side of one
  connector (section 15), with:
  - its objects, held once in its own working set, with revisions and
    card states as jig's are (client.md, section 5);
  - its cards, one per kind of object, shown on its pages and in jig's
    slots;
  - its pages, at its own addresses, with jig's frame's parts where it
    places them;
  - its requests: keyed, through the table, routed by the engine's root
    to the connector they concern (`../../domain/people.md`, 5.1);
  - its reads and watches, of the application's own documents, each
    naming its scope, a project or the deployment, in which the engine
    admits the party and its connector judges what they may see
    (contract.md, 7.2).
- **An application's page shows jig's objects** through slots of the
  same kind, the other way round. An incident's page shows its task's
  chip and the proposals waiting in its subtree: it names each
  by jig's key, the root routes the request to `jig-domain-client`,
  which holds the object in its working set as shown elsewhere and
  watches it, and jig's view fills the slot with jig's card. One object,
  one card, wherever it shows (objects.md, section 2). A family that
  presents one of jig's objects itself names it the same way, and builds
  its own nodes from the object `jig-domain-client` holds, bound through
  jig's binding constructors (view.md, sections 4 and 5).
- **An application's page shows jig's sections** through slots of the
  third kind. Its child names each by value, a `Section` of
  `jig-domain-client`'s (an incident's lead task's plan or activity, the
  inbox's needs, the conversations), and the root hands the page's whole
  list to `jig-domain-client` (`ShowSections`), as it hands the objects
  the page names. `jig-domain-client` opens what the sections need for
  the page's generation (client.md, 5.1), and the view root fills each
  slot with jig's section, whole (`jig_view::sections::section`). Both
  kinds of slot hold a reference whose owner is jig and whose token is
  the family's: the family's binding table says whether it means one of
  jig's objects or one of jig's sections. A section the domain does not
  show (a second task's) builds nothing, and
  its slot says it cannot be shown.
- **A family follows jig's rules for what it shows:** nothing shown done
  before its answer is durable; acts naming the revision they saw;
  provisional values marked. Its world checks it, with jig's referee
  (testing.md, section 6).

## 8. The small store, composed

- **Records per child,** keyed by the child's own keys, wrapped by the
  root with the child's variant, as the engine's root wraps its children's
  records (`../../domain/root.md`, section 6).
- **The table's records** are the root's, saved and erased as the table
  emits them.
- **Versioned,** each by its owner, so a client updated since reads what
  the older one kept.
- **Restored at start** by the shell, each record handed to its owner,
  the table's to the table, each request with the kept owner of its
  scope (section 4), before the first step.
- **Whose work was kept.** Once the table's records are restored, the
  root tells `jig-domain-client` each scope they name, once, with its
  count of requests, read from the records' own scopes; and whenever the
  table retires a kept request, answered or unknown past the retention,
  that one of that scope's is gone. Whether to warn of someone else's
  work is `jig-domain-client`'s to decide (client.md, section 11); the
  root passes on what the table kept, and decides nothing.

## 9. The view, composed

The view root (view.md, section 4) mirrors the domain's root:

- it builds the page's tree in the arrangement it chooses, jig's default
  or its own, from jig's primitives, its own nodes, and the page's
  owner's view (view.md, section 4);
- it fills each slot from the slot's owner's view: a part from its
  owner's, a card or a section of jig's from jig's;
- it decodes an act on a node through the node owner's bindings, wrapped
  with its owner, for the domain's root to route;
- it resolves a link node's address, without acting, through its
  binding's owner's view (view.md, section 5);
- it routes `draw` by a drawing kind's owner, exhaustively, and gives
  its owners' names for styles and regions, for the look, plain data its
  shells give their parts (view.md, sections 13 and 17.1).

## 10. What a root never does

- send a keyed request around the table, or answer one itself;
- decide whether an answer is durable, when to retry, or whether the link
  is live;
- compute what a party may do: what a card offers is what the engine's
  document says;
- read a part's value meant for another child, or hold one in its own
  state;
- keep anything across steps but its children's states and the table:
  which child asked for a request is its owner, which the table holds
  (section 4), never a map of the root's own;
- match on a child's vocabulary with a `_` arm.

## 11. A service's root

A service is a party with no view and no person (`../../domain/people.md`,
section 3): a deploy pipeline that sets goals, a monitor that answers a
person task addressed to its role.

- **Its root** composes `jig-domain-client` and the request table with
  the service's own logic, and no view. It asks by value (client.md,
  4.1), reads and watches as any client does, and opens no page.
- **Its small store** is a file, written and synced before any request
  of the same step leaves, so a restart of the service sends what was
  pending again with its keys.
- **It signs in** with the credential the deployment issued it, at start.
- **The same promises hold:** each request made once, answered once
  durable, and the service told why when refused.

## 12. The reference root

`ops`'s client root (`../../examples.md`, section 8) is the reference:
written to be copied, commented as such. Its crate has one module per
part of this document:

```
<application>-domain-client/src/
├── lib.rs          the tree, re-exports
├── boundary.rs     the root's vocabulary: events in, requests out, one variant per child
├── address.rs      the root's addresses: jig's and each connector side's
├── domain.rs       Domain, step, fire, iterate's part: the one shape (section 3)
├── route.rs        which child each event, act and hand-off goes to (section 5)
├── parts.rs        parts to their owners' sides, jig's objects and sections to jig-domain-client (sections 6 and 7)
├── store.rs        the small store's records, the children's wrapped (section 8)
├── limits.rs       the children's limits and the table's, and their sum
└── tests.rs
```

Its view root, `<application>-view`, has the same shape for the
view (view.md, section 4). A new application copies both, replaces
`ops`'s connectors' sides with its own, and lets the compiler list the
arms to fill. The client host around them, which routes between both
roots and `jig-protocol-client-link` for every shell, is copied from
jig's test application's, as its binary and its service are
(`../../protocol/browser.md`, 3.2; `../../protocol/service.md`,
section 7); so is its browser shell, its start and its loop over jig's
browser parts, from the reference shell, `jig-test-shell-web`
(`../../protocol/browser.md`, 3.1).

## 13. The world

The root has no world of its own beyond its application's: the
application's client worlds run its whole client root, view and all, on
jig's client world with jig's scripted engine and the application's
scripted documents, and in the joint world with its engine's domain
(testing.md). jig's referee checks there, from what the person saw, that
the root kept every promise above.

## 14. Open questions

- **Generating the boilerplate:** whether the client root's routing and
  wrapping, which have one shape, are best copied, as now, or generated
  from a short description, as for the engine's root
  (`../../domain/root.md`, section 14).

## 15. Owners

- **An owner** is a child of the client root that keeps values, names
  nodes of the view, or asks for keyed requests: `jig-domain-client`, or
  one of the application's. Each of the application's is a connector's
  client side, or another child of the client root (a settings page of
  its own), which sends no request, read or subject to the engine: one
  that does is an own child domain's client side, which waits for the
  application that needs it (15.1).
- **jig takes no application number.** The view names it `Owner::Jig`,
  and the wire names jig's records as jig's.
- **The application numbers its owners,** each by a `u16`, in one typed
  enum both its roots share, with the function that gives an owner's
  number and the one that gives the owner of a number, or none. An
  application whose owners are all its connectors' client sides shares
  the enum of its connectors, as `ops` shares `jig-ops-domain-connectors`'s
  `Connector`; one with another owner adds it to that enum. A connector's owner number
  is the number the engine's root gives that connector in the core's
  configuration (`../../domain/root.md`, section 6); any other owner has
  a number no connector has.
- **One number, everywhere the client names the owner:** a part's owner
  (contract.md, 5.3); the owner of a view's key, style token, region
  name, binding, slot, drawing, picture and phrase
  (`jig_view::Owner::Application { number }`, view.md, section 3); an
  application's keyed requests, reads and subjects (contract.md, 3.3, 6
  and 7.1), which the wire tags with that number
  (`../../protocol/client.md`, section 11); and the small store's
  records of the owner (client.md, section 11).
- **Never reused.** A number given to an owner is never given to
  another, since the small store keeps it across the client's updates,
  and a request sent under it may be answered after one.
- **Matched whole.** Each root's match on an owner is exhaustive, with no
  `_` arm, so a new owner breaks the build where it must be routed and
  shown; a number that names no owner is a fact, never a guess.
- **Onto skein-ui's owner.** skein-ui's owner is an opaque number,
  `skein_ui::Owner { number: u32 }`, which it compares, orders and
  hashes and never interprets (skein's `ui.md`, 3.3). jig's typed owner
  maps onto it by a total function: `Application { number }` to the
  same number, and `Jig` to 65536, one past every `u16`, so jig's and
  an application's never meet; and back by a partial one, which gives
  none for a number that names no owner (view.md, section 3). The view
  root routes what skein-ui hands back by an owner (an act's binding, a
  slot, a drawing's kind) through that partial function, then matches
  the typed owner whole.

### 15.1 Later: an application's own child domains

An application's own child domain in the engine (`../../README.md`,
section 4), whose families belong to no connector (a laboratory's
evidence and verdicts), is an owner like a connector: numbered in the
same enum, with a number no connector has; its client side a child of
the client root; its families' requests, reads and subjects naming its
number, which the engine's root routes to that child domain as it routes
a connector's. Nothing in parts, slots, bindings, styles, drawings or
region names changes for it, and nothing in what an application already
writes. It waits for an application with such a domain: the engine's
root's routing of an application's requests, reads and subjects to a
child that is not a connector, and the people child's admission of them
(`../../domain/people.md`, 5.1), are designed then.
