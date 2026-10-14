# An application's own client

Draft, 2026-10-14. How jig's client lets each application have its own
interface: its pages, its layout, its interactions and its look, on
jig's client domain, which keeps every promise. It says what
applications on jig will ask of a client, what jig's client fixes today
and what it leaves to them, the one principle that decides between the
two, the changes to make before the first application builds its first
release on jig's client, and the extensions to leave for later, shaped
now so that they add without changing what applications write against.
Nothing here is adopted: `client/domain/` describes the client. What
is still open is listed in section 8.

## 1. In one page

- **jig serves any application with planned and executed work,** not
  one kind of interface. Applications on jig will differ in what a
  person looks at first (a conversation, an inbox, a fleet, a matrix of
  experiments), in how pages are laid out, in what a person does with
  their hands (press, choose from a menu, pick a point on a map), and in
  how they look (section 2).
- **The domain is open; the view is not yet.** An application already
  brings its own objects, cards, pages, documents, requests, words,
  style tokens, drawings and shells. But jig's frame and pages are built
  whole, in one call, and cannot be rearranged; the tree has no layout
  anyone owns; and the person's acts are four, with no menu and no act on
  a drawing (section 3).
- **The principle: promises live in the domain and in cards, never in a
  page's shape.** Every promise jig makes to a party (made once, shown
  as asked until durable, refused with why, a decision naming the
  revision it saw, provisional text marked) is kept by
  `jig-domain-client`, the request table, and the card that shows each
  object. A page is an arrangement of sections and cards; jig's pages
  are its reference arrangements, which an application may use as they
  are, rearrange, or replace (section 4).
- **Before the first application's first release** (section 5), so that
  what it writes against stays put:
  1. **Pages as a library:** jig's frame, sections and cards built by
     functions an application's view root composes; jig's sections
     embeddable in an application's page, as jig's cards already are.
  2. **Layout as semantics:** landmark roles in the tree, and jig's
     regions named as a documented contract, so an application's
     stylesheet and a shell place them without depending on order.
  3. **The closed sets completed:** a menu, a disclosure and an image
     among the roles; a key a binding answers to; the shape of an act on
     a drawing, decoded by its owner.
  4. **Owners, not connectors,** in what a part, a slot, a binding and a
     style name, so a family that belongs to no connector needs no
     change of an application's code.
  5. **The text shell, jig's:** a tree printed as lines and a line read
     as an act are application-neutral, so they are a shell of jig's.
- **Later, additive** (section 6): families of an application's own
  child domains, streams of an application's data, drawings acted on,
  several applications in one client, the browser's shell.

## 2. What applications ask of a client

Five applications, of the kinds jig is for:

| Application | First looks at | Its own objects | Layout | Hands | Live data |
|---|---|---|---|---|---|
| `ops` (`examples.md`) | the inbox, incidents | incidents, environments | lists, a task's page | press, write | committed state |
| work on code, landed by review | a conversation | changes on their way to landing | a thread with a composer pinned below; a page with an aside | press, write, a menu | a run's text |
| a laboratory of experiments | a matrix of heads by experiments | experiments, evidence, verdicts, baselines | a grid; distributions | press, pick a cell | committed state |
| an operator of a fleet | the fleet, its rollouts | machines, slots, rollouts, checks | a map or a grid; timelines | pick a machine, zoom a range | logs and metrics, read as they come |
| a service | nothing | its own | none | none | none |

- **Every one keeps jig's work:** tasks, conversations, proposals,
  escalations, person tasks and holds, which is why each is on jig.
- **Most differ in their first page and their layout,** several in their
  hands, and one in its live data. None differs in what a decision
  promises.

## 3. What jig's client fixes, and what it leaves

| Axis | Left to the application | Fixed by jig |
|---|---|---|
| Content | its families: objects, cards, pages, documents, requests (`client/domain/root.md`, section 7) | a family belongs to one connector's client side (root.md, section 2) |
| Work | jig's primitives, and its families' own requests and outcomes | jig's documents carry jig's values only, by design |
| Pages | home, navigation, its own pages under jig's frame (`client/domain/pages.md`, sections 2 and 9) | jig's frame and each of jig's pages are built whole: `jig-view` builds the frame, the page and its notices in one call; the task page shows one section per address; a slot nests two deep (`client/domain/view.md`, section 4) |
| Layout | the shell's stylesheet, from roles, order and tokens (view.md, sections 3 and 13) | nothing in the tree says where a region goes, and jig's regions are not named for an application's stylesheet |
| Hands | its bindings, decoded by its view | four acts (press, input, submit, toggle); no menu, no image role; a drawing is never acted on (view.md, sections 3 and 5) |
| Look | its style tokens, and jig's restyled (view.md, section 13) | nothing |
| Shells | its own shell, or no view at all (view.md, section 11) | jig's: the browser's and the native one; a text shell is open (view.md, section 14) |
| Live data | its documents, level-triggered | a run's text is the only stream (`client/domain/contract.md`, 7.3) |

## 4. The principle

- **What jig promises is kept below the page.** `jig-domain-client`
  holds each object once, with its revisions and its card's state
  (`client/domain/client.md`, section 5); the request table keeps each
  request until its answer is durable (root.md, section 4); jig's card
  for an object shows its state, its acts and its refusals, wherever it
  shows (`client/domain/objects.md`, section 2). None of it depends on
  where the card is, next to what, or in what order.
- **So a page is an arrangement,** of sections (a task's activity, its
  plan, its budget; the inbox's groups; a conversation's thread and its
  composer) and of cards. jig's pages are jig's reference arrangements:
  an application that wants them takes them whole, as now.
- **Cards stay jig's, whole.** A card is where a promise shows, so an
  application restyles a card, chooses where it goes, and fills its
  slots, but never rebuilds it.
- **The working set follows the arrangement.** What a page watches and
  reads is the union of what its sections need, opened and closed by
  `jig-domain-client` per page generation, as now (client.md,
  section 5). A page is still one at a time.
- **The fake person is unchanged.** It finds nodes by role, name and
  text (`client/domain/testing.md`, section 3), so a story written for
  jig's pages runs on an application's arrangement of the same sections.

## 5. Before the first release

What an application writes against is its client root, its view root,
the tree's closed sets and the shells it composes. Each change below
changes one of them, so each is made before the first application
builds a release on jig's client, and each is small for that
application to take when it repins: its view root's calls, its text
shell's matches, its owners' names.

### 5.1 Pages as a library

- **The frame's parts, the sections and the cards are functions** of
  `jig-view`, each over the builder, the domain and its words, as cards
  are (view.md, section 4): the frame's parts (what needs the party,
  the project and its switch, going to a task, the link's state, who is
  signed in and signing out, the navigation's slot); each page's
  sections; each card.
- **jig's pages are compositions of them,** written with the same
  functions an application calls, so jig's own pages prove the library.
- **An application's view root arranges each page:** at an address of
  jig's, jig's page as it is, or its own arrangement of the same
  sections (a task's every section on one page with its budget aside);
  at its own address, its page, which shows jig's objects through slots
  (root.md, section 7).
- **jig's sections in an application's page.** Today an application's
  page shows jig's cards through slots. A slot of a third kind shows one
  of jig's sections: the application's domain names it (a task's
  activity, the inbox's needs-you group, a conversation), the root
  routes it to `jig-domain-client`, which opens what that section
  watches and reads for the page's generation, and jig's view fills the
  slot. Slots still nest at most two deep.
- **The frame is the application's to arrange,** from jig's frame's
  parts and its own (a brand, its own navigation): what needs the party
  and the link's state stay shown on every page, since a person must
  always know both; the project's switch is shown where an application
  has more than one project.

### 5.2 Layout as semantics

- **Landmark roles** join the tree's roles, as an accessibility tree has
  them: banner, navigation, main, complementary, and contentinfo, with
  form and search. A shell places landmarks (a browser's header, nav,
  main, aside and footer; a terminal prints the banner first and the
  complementary last), and a screen reader moves between them.
- **jig's regions are named, as a contract:** each section and each
  region of the frame carries a stable name, listed in `pages.md` beside
  the section, so an application's stylesheet places it (the composer
  pinned below its thread, the budget aside) without depending on the
  order of nodes. A name is changed only as a design change.
- **An application's own placement** is its style tokens on its own
  regions, as now. The tree still says where nothing goes; it says what
  each region is, and the shell's stylesheet does the rest.

### 5.3 The closed sets, completed

- **Roles:** a menu and its items, for the acts a card or a page offers
  past the few it shows; a disclosure, a region a shell may show folded
  with no step of the domain, its content always in the tree (so the
  fake person and a terminal find what a browser hides); an image, with
  its text alternative.
- **A key a binding answers to:** a binding may name a key (a letter,
  with or without a modifier), and a shell that has keys turns that key
  into a press of the node. No new act; a terminal ignores it.
- **The act on a drawing, shaped now:** an act carrying bytes of the
  drawing's owner, within a bound the owner declares, which only the
  owner's view decodes, into its action or nothing, as typed input is
  made (view.md, section 5). Its drawing code reports it. Every act on a
  drawing has an ordinary node beside it doing the same, so the fake
  person, a screen reader and a terminal can make it. Built with the
  first drawing that needs it (section 6); its place in the act's type
  is taken now.

### 5.4 Owners, not connectors

- **A part, a slot, a binding, a style token and a drawing name their
  owner,** jig or one of the application's owners, by a number the
  application's two roots share, as connectors' numbers are shared now
  (root.md, section 2). A connector's client side is one kind of owner.
- **For an application whose families are all its connectors',** each
  owner's number is its connector's, and nothing else changes. A family
  of an application's own child domain (section 6) then needs no change
  in the parts, slots and bindings an application already writes.

### 5.5 The text shell, jig's

- **What a terminal client decides is application-neutral:** the tree
  printed as lines, by role, with every node a person acts on numbered;
  a typed line read as an act on a numbered node, or as printing, or
  closing; the print's rhythm. None of it names an application's value.
- **So it is a shell of jig's,** beside the native one: two pure
  functions under the subset (the renderer and the command reader),
  tested by golden text, and the loop over a terminal's input and output
  and the native shell, ordinary Rust, as shells are (view.md,
  section 11). It settles view.md, section 14's text shell, and is
  `ops`'s terminal client (`examples.md`, section 8).
- **An application's terminal** is its client root and view root
  composed with it, and anything it prints of its own before the tree.
  An application that built one first moves its renderer and command
  reader here.

## 6. Later, and additive

Each waits for the application that needs it; none changes what
section 5 settles.

- **Families of an application's own child domains:** objects that
  belong to no connector (a laboratory's evidence and verdicts), read,
  watched and requested through the owner of section 5.4, which the
  engine's root routes to that child domain as it routes a connector's
  (`README.md`, section 4: an application's own child domains
  beside its connectors).
- **Streams of an application's data:** a subject of an application's
  read from an offset, within a window, as a run's text is
  (contract.md, 7.3): a log followed, a metric's latest points. A
  stream is never a document of the engine's committed state, and is
  marked so where it shows.
- **Drawings acted on** (5.3): its decoding, its drawing code's reports
  in the browser's shell, and the fake person's act on its fallback.
- **Several applications in one client:** one person's inbox across
  applications that share a deployment's people.
- **The browser's shell** (`jig-shell-web`; view.md, section 11), on the
  tree as section 5 leaves it, with jig's stylesheet naming its landmarks
  and regions. Its plan follows section 5, so the tree it maps does not
  change under it.

## 7. In what order

1. **A design pass:** this draft into `client/domain/view.md` (the
   roles, the key, the act on a drawing, the text shell),
   `client/domain/pages.md` (sections and their names, the frame's
   parts, arrangements), `client/domain/root.md` (owners; the section
   slot) and `client/domain/contract.md` (owners in parts and
   subjects).
2. **A plan for section 5,** in jig, with `ops`'s client taking each
   change as the reference application does, and `ops`'s terminal
   client as the text shell's first consumer.
3. **The first application repins once,** after that plan: its view
   root's calls, its terminal's composition, its owners' names.
4. **The browser's shell,** in its own plan, with skein's builds for
   `wasm32-unknown-unknown`.
5. **Section 6's items,** each with the application that needs it.

## 8. Open questions

- **Landmarks, or placement tokens only:** whether jig's tree takes the
  accessibility tree's landmark roles, or leaves placement entirely to
  owners' style tokens on regions. Default: landmarks, since a screen
  reader needs them anyway, and tokens for the rest.
- **How much of the frame is required:** what needs the party and the
  link's state on every page, or only what an application chooses.
  Default: those two, always.
- **Folding without the domain:** whether a disclosure's state stays the
  shell's (lost on a reload, unseen by the domain), or is the domain's,
  as a plan's folds are now. Default: the shell's for a disclosure, the
  domain's where folding changes what is read (a plan's window).
- **The text shell's commands:** whether a terminal client's commands
  (`n`, `n words`, print, close) are jig's for every application, or an
  application's to extend. Default: jig's, with an application's own
  lines printed before the tree.
- **Section slots' depth:** whether a section in an application's page,
  whose cards leave slots for parts, needs a third level of nesting.
  Default: no; a section's cards fill their parts in the second pass.
