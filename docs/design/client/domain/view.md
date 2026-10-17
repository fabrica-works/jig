# The view

Provisional, 2026-10-09, revised 2026-10-17. `jig-client-view`: the
reference UI, jig's optional presentation, for `ops` and jig's test
application. It is built on the public client alone, `jig-client` and
its helpers (client.md, sections 4 and 19), and on skein's UI kit,
`skein-ui` (skein's `ui.md`); a service has none, and an application may
present jig's work its own way instead. It holds the reference UI's
state (section 18: its place and page, the working set, the requests it
sends, drafts and the confirmation, notices and its own records);
jig's primitives, which build parts of a tree from that state and the
client's documents (the frame's parts, the sections, the cards, the
confirmation, the owed cards and the notices) or handle a person's acts
on them (jig's bindings, which turn those acts into the reference UI's
actions); and jig's default arrangement of them into pages. The tree,
the builder, the diff, the outline's rules, words' machinery, Markdown,
scenes, limits and the text shell are skein's, and each section that
names them points there and keeps what is jig's: its owner, its region
names and tokens, its phrases, its limits' constructors, its settings of
the text shell (section 15) and its drawings (section 17). What is still
open is listed in section 14, and what waits in section 16.

## 1. In one page

- **The reference UI, on the public client alone.** It uses
  `jig-client`'s surface and its helpers (`shapes`, `facts`, `thread`,
  `words`) as any application does, and nothing else of jig's: no door
  into the client of its own, no record on the wire of its own. Whatever
  it shows, an application can show, so it proves the API is enough. It
  is optional: an application presents jig's objects from jig's
  documents itself, on skein's UI kit or anything else (pages.md,
  section 9).
- **Built on skein-ui.** The tree and its vocabulary, owners, the
  builder, bindings and acts, the diff, words' machinery, Markdown,
  scenes, the outline's rules and limits are skein-ui's (skein's
  `ui.md`); the text shell is `skein-ui-text`'s and the browser's parts
  `skein-ui-web`'s. `jig-client-view` is what of the reference UI's view
  is jig's.
- **Its own state, beside the client's.** Where the person is, what the
  page holds, the requests it sent, drafts, the confirmation and
  notices are the reference UI's, in step code beside the client, which
  the application steps both of (section 18). The client holds the
  documents; the reference UI reads them by its handles.
- **State in, tree out.** Each of the frame's parts, each section of a
  page, each card, the confirmation, the owed cards and the notices is a
  function from the reference UI's state and the client's documents to
  tree nodes, through skein-ui's builder, one node a call. A card has
  one function, called wherever jig shows it (objects.md, section 2).
- **Primitives, nothing mandated.** An application generates whatever
  tree it wants with skein-ui and composes jig's primitives where it
  likes, or not at all. jig's pages are jig's default arrangement of the
  same primitives, a reference an application may use, copy or ignore
  (section 4; pages.md, section 12).
- **A semantic tree.** The tree says what each node is (a region, a
  heading, a list, a button, a text field, a log), the way an
  accessibility tree does, never how it looks (skein's `ui.md`, section
  3). Each shell maps it onto its platform.
- **One outline, skein-ui's.** Every tree keeps skein-ui's outline
  (skein's `ui.md`, section 14), which jig's referee checks; jig adds no
  rule. jig's default arrangement keeps a banner with the link, what
  needs the party and who is signed in, and `after_page`'s parts last
  (section 3.1).
- **The promise, below any tree.** A deciding act on the reference UI
  goes through its confirmation and names the revision of the document
  its binding was issued from, whatever the tree; the engine refuses a
  moved one, and the client makes it once. An application presenting
  jig's objects itself reads what a card shows from jig's documents
  with the helper `facts`, and jig's check finds, in its worlds, what it
  failed to show (section 4; testing.md, 6.2), never at run.
- **Accessible because tested.** Every node a person acts on, or reads
  to decide, has a role and a name, since the fake person finds nodes
  only by role, name and text (testing.md, section 3). A node it cannot
  find is a defect of the page.
- **Bindings are tokens.** A node a person acts on carries its owner's
  token, never a closure (programming-model.md, 4.2), as skein-ui's
  bindings are (skein's `ui.md`, section 6). jig's bindings say what
  each of jig's tokens means and decode an act into the reference UI's
  action (section 5).
- **Diffed, not replaced,** by skein-ui's diff (skein's `ui.md`, section
  7): a matched node keeps its id, so the platform keeps focus, caret,
  selection and scroll; a field's value is written only when the
  reference UI wrote it (section 6).
- **Slots, spliced.** jig's pages and cards leave slots for an
  application's parts, and an application's pages leave them for jig's
  cards and sections; the view root fills each from its owner's view
  (section 4).
- **Owners, mapped.** jig's typed owner, jig or one of the application's
  by the number both its roots share, maps onto skein-ui's opaque owner
  (section 3; root.md, section 15).
- **Words from phrasebooks.** jig's enums (phases, holds, refusals) are
  said with the helper `words`, the reference UI's own sentences with
  its phrases, both by skein-ui's phrasebook: jig ships English, and an
  application overrides either phrase by phrase, as data (section 7).
- **Sized from limits,** the reference UI's and the client's, by jig's
  constructors of skein-ui's limits, so a tree always fits (section 10).
- **No framework.** The reference UI is step code under the subset, as
  the client is; a framework's closures, signals, hooks and scheduler
  are what the subset leaves out (programming-model.md, section 10). Its
  state and its worlds would not change if one rendered the same tree.
- **skein's shells, with jig's parts.** A terminal client composes
  skein's text shell with jig's settings (section 15); a browser shell
  composes skein's browser parts with jig's, in `jig-client-shell-web`:
  the client's adapter, the printed link's sign-in and jig's look
  (section 11).

## 2. Structure

```
jig-client-view/src/
├── ui.rs           the reference UI's state machine: its events and requests, step, fire (section 18)
├── address.rs      its places, and their codec (pages.md, section 2)
├── page.rs         the page at the place: its sections, handles, windows, folds and filters (18.2; pages.md, 12.5)
├── objects.rs      the working set: objects by key, their revisions and card states (18.3)
├── intents.rs      the requests it sent, by key, and what each is about (18.4)
├── drafts.rs       fields, drafts and the confirmation's state (18.5)
├── records.rs      its own records: drafts and the place (18.8)
├── owner.rs        jig's owner, and its mapping onto skein-ui's (section 3)
├── regions.rs      jig's region names, each with its name (pages.md, 12.4)
├── styles.rs       jig's style tokens, each with its name (section 13)
├── binding.rs      jig's bindings: its meanings, their table, the binding constructors, decoding (section 5)
├── phrases/        the reference UI's phrases and their holes (jig.rs), and English (english.rs) (section 7)
├── turns.rs        a conversation's thread as nodes, and its provisional text (section 9)
├── frame.rs        the frame's parts, and jig's banner (section 4.1)
├── confirmation.rs the confirmation
├── notices.rs      the notices: kept, timed and built (18.7)
├── cards.rs        each card, and their shared parts: head, context, state
├── sections/       each section of jig's pages, and the pages' headers (section 4.1)
├── pages.rs        jig's default arrangement, `after_page` and the owed cards; the pages not about the work
├── draw.rs         jig's drawing code, which draws no kind (section 17)
├── limits.rs       its limits and worst case, jig's constructors of skein-ui's limits, and jig's scratch (sections 10 and 18.9)
└── tests.rs
```

It depends on `skein-ui` and `jig-client`, and on nothing else of
jig's: the client's surface, its documents and its helpers are all it
uses. What every application's view shares is skein-ui's, and the
reference UI uses it without repeating it: the tree and its vocabulary,
owners, the builder with slots and splicing, bindings and acts, the diff
and its patches, the view that keeps the last tree, tokens, region names
and the look's names, the phrasebook's machinery, the Markdown reader,
scenes, the outline's rules, and `Limits` with its worst case (skein's
`ui.md`, section 2).

The application's view root (`<application>-client-view`:
`jig-ops-client-view`, `jig-client-test-view`) depends on
`jig-client-view`, on skein-ui, and on its families' views.

## 3. The tree

The tree is skein-ui's (skein's `ui.md`, section 3): nodes in preorder,
with ids kept while the diff matches them; its closed roles, its states
and properties, owners, text, values and keys, images and drawings
(skein's `ui.md`, 3.1 to 3.5); and no layout. There is no provisional
state: jig says provisional text is provisional in words (section 9).
What of the tree is jig's:

- **jig's owner.** `jig_client_view::Owner` is `Jig`, or `Application {
  number }`, one of the application's owners by its `u16` (root.md,
  section 15); there is no other kind, so an owner that is not a
  connector's (root.md, 15.1) names its nodes as a connector's side does.
  It maps onto skein-ui's opaque owner (skein's `ui.md`, 3.3), and onto
  the client's asker (client.md, section 4), by a total function, an
  application's number to the same number and jig to 65536, one past
  every `u16`, so the two never meet; and back by a partial one, which
  gives none for a number that names no owner. Both are matched whole.
  Every key, style token, region name, binding, slot, drawing kind,
  picture and phrase the reference UI builds names jig.
- **jig's names in the tree:** its keys, one kind per kind of object, so
  lists diff by object; its region names (pages.md, 12.4); its style
  tokens (section 13); its pictures, in jig's look (section 13); no
  drawing kind (section 17).
- **A field's value** carries the reference UI's count of its own writes
  to the field (18.5) as skein-ui's count of the owner's writes (skein's
  `ui.md`, 3.4).
- **Menus and disclosures** are skein-ui's (skein's `ui.md`, section 3):
  a card or a page offers the acts past the few it shows in a menu, and
  a disclosure's content stays in the tree while a shell folds it, the
  fold the shell's and lost on a reload. A fold that leaves what it
  folds out of the tree is the reference UI's state's instead (18.5).
- **An application's own kinds of node** are drawings (skein's `ui.md`,
  3.5): a chart of an incident's error rate, a map of a pool's
  environments, a timeline, each with a fallback saying what it shows in
  words and numbers ("error rate 30 % since 10:02, 2 % before"), and
  every act on it with an ordinary node beside it doing the same
  (section 17).

### 3.1 The page's outline

- **skein-ui's rules** (skein's `ui.md`, section 14) hold every tree,
  whoever arranged it: the page node is the root; each of its children
  is a landmark (a banner, navigation, main, complementary, content
  info, a form or a search, or a named region) or a dialog; exactly one
  main; at most one banner and at most one content info; each shortcut
  at most once; Escape only inside a dialog. `skein_ui::outline::check`
  gives the first fault, naming the rule and the node; the referee runs
  it on every tree a person sees, in jig's worlds and in each
  application's on skein's UI kit (testing.md, 6.3).
- **jig adds none.** An application places jig's frame's parts and
  `after_page`'s parts where it likes, or omits them; what a person
  loses when one is omitted is said in pages.md, section 2 and 12.1.
- **jig's default arrangement** (`jig_client_view::jig`; pages.md, 12.3)
  keeps this outline, as defaults an application may use, copy or
  ignore:
  1. the banner, first, holding jig's link (`jig-link`), what needs the
     party (`jig-needs`) and who is signed in (`jig-party`), on every
     page (pages.md, 12.1);
  2. the navigation, the main and the complementary landmarks the page
     has;
  3. last, `after_page`'s parts: the owed cards (a complementary
     landmark, `jig-owed`), when there are any; the confirmation (a
     dialog, `jig-confirmation`), when one is open; and the notices (a
     named region, `jig-notices`), when there are any.

  Forms and searches are inside the others (a new chat's form, the
  notes' search), and a page not about the work (pages.md, section 8) is
  inside the main. The reference UI's tests check that jig's default
  arrangement keeps its banner, with those three regions, and
  `after_page`'s parts last, on each of jig's pages (testing.md,
  section 2).

## 4. Building

- **The builder is skein-ui's** (skein's `ui.md`, section 5): one node a
  call, no macros. Building past the tree's capacity is an assertion:
  the capacity comes from jig's limits (section 10), so overflowing it
  is the view's bug.
- **jig's primitives** each build one part of a tree, or nothing, where
  the application calls them, and bind what they build with jig's
  bindings (section 5); 4.1 lists them:
  - *a card* is a function over the builder, the reference UI's state,
    the client and its object, called wherever the card shows; its place
    changes its line of context only;
  - *a section* is a function reading the reference UI's state and the
    client's documents for one of the page's sections (pages.md, 12.5)
    and calling the cards' functions for the objects it names;
  - *the frame's parts* (`frame::link`, `needs`, `party`, `projects`,
    `go_to_task` and `navigation`) are functions, each placed anywhere
    or nowhere; `frame::banner` is jig's default banner of the first
    five;
  - *`after_page`'s parts* (the owed cards, the confirmation and the
    notices) are functions, each called where the application likes, or
    not at all; `after_page` is jig's default, calling the three last.
- **A page is an arrangement** of jig's primitives and the application's
  own nodes, under skein-ui's outline (section 3.1). jig's pages are
  jig's default arrangement, built from the functions of 4.1 alone, so
  jig's own pages prove the library an application uses.
- **A slot** is a node left for another owner's nodes, holding a
  reference and, until it is filled, a busy status in the words of the
  view that left it (skein's `ui.md`, section 5). jig uses
  three kinds: a part's, which a jig page or card leaves for an
  application's value (root.md, section 6); a card's, which an
  application's page leaves for one of jig's objects; a section's, which
  an application's page leaves for one of jig's sections (root.md,
  section 7).
- **The view root** builds a client's whole tree with skein-ui's
  builder, in the arrangement it chooses:
  1. under its page node, its landmarks and dialogs, from jig's
     primitives, its own nodes and its families' views: at a place of
     jig's, jig's page or its own arrangement of the page's sections,
     headers and heading; at the application's place, its page, from
     its owner's view; on a page not about the work, jig's
     (`pages::not_the_work`), whatever the place, or its own, from the
     reference UI's page (18.2); then the page node is closed;
  2. each slot's nodes, in two passes, from the slot's owner's view,
     which the builder splices in at the slot, since it splices only a
     tree whose nodes are all closed.

  `jig_client_view::jig` is jig's default: it builds the banner
  (`frame::banner`), the navigation (`frame::navigation`, with the
  application's entries in their slot), jig's main (`pages::main`) and
  `after_page`, and returns true; when the page is the application's
  (elsewhere: 18.2) it builds the banner and the navigation only and
  returns false, and the root builds its main, then calls `after_page`,
  or places its three parts its own way.
- **Two deep.** A family's page may leave slots for jig's cards and
  sections, which the view root fills from jig's view, and those leave
  slots for parts in turn, which the second pass fills: a section's
  cards leave their parts' slots at the same depth as a card's. Slots
  therefore nest two deep, never more: the builder splices in two
  passes, and the tree's capacity counts both.
- **A slot left empty** shows as loading, and one its owner cannot show
  says so (a section the reference UI does not show on this page, a part
  its owner does not know); neither breaks the page.
- **jig's objects, presented by an application.** jig's card and section
  functions are the defaults. An application may instead present one of
  jig's objects itself, from the object's document as the client
  delivers it: on a page of the reference UI, it names the object as a
  card's slot does (root.md, section 7), so the reference UI holds it in
  its working set, follows it live and counts it shown on the page,
  never owed, and its nodes may bind their acts through jig's binding
  constructors (section 5), so they go through the reference UI's
  confirmation; with no reference UI, it watches the object's subject
  through the client and sends the decision itself. jig's promise holds
  without mandating its nodes:
  - *held below any tree:* a decision names the revision of the
    document it was decided on, and a revision that moved is refused
    (contract.md, 5.2); made once, nothing lost and nothing reported
    answered before it is durable are the client's and the engine's
    (client.md, section 6), not the tree's;
  - *given as data:* what a decision will do is in the object's document
    (contract.md, 5.4); the helper `facts` reads it, with the object's
    state, a refusal and why, and whether the act is dangerous
    (client.md, section 19); the application says them with `words` or
    its own, in nodes of its own;
  - *checked, not mandated:* jig's check, which the referee runs in
    jig's worlds and in each application's, finds a decision whose
    revision was not in a document the client delivered to the
    application before it was sent, and, on a tree a person acted on, a
    deciding act made on a tree that did not show its decision's facts,
    in the open confirmation or within the acted node's nearest named
    region (testing.md, 6.2);
  - *the application's to show:* the confirmation is the application's
    (contract.md, 5.4); the reference UI keeps jig's, and the rest of a
    card's promise is the application's to show when it presents the
    object itself, checked by jig's check in its worlds, never at run.

### 4.1 The parts, sections and pages, as functions

Each takes skein-ui's builder, the reference UI's state, the client,
jig's words and jig's bindings (`b, ui, client, words, bindings`), and
what its line names; each builds the regions and landmarks pages.md,
section 12, names for it, with their region names, and each has a
companion that counts the most nodes it may build (section 10). Each is
a primitive: the application calls it where it likes, or not at all.

- **The frame's parts** (`frame`, pages.md, 12.1): `link`, `needs`,
  `party`, `projects` and `go_to_task`, each a region the application
  places in any landmark (jig's banner, its own, its navigation, an
  aside); `navigation`, which also takes the application's slot, a
  navigation landmark of jig's entries with the application's slot
  after them; and `banner`, jig's default banner: the landmark, with the
  five in that order.
- **The sections** (`sections`, pages.md, 12.2):
  `section(…, section: Section) -> bool` builds one section of the page,
  a region with its level-2 heading, or nothing, returning false, when
  the reference UI does not show that section on this page (pages.md,
  12.5). The headers, each shown with any of its page's sections:
  `inbox::header`; `task::header` and `task::tabs`, naming the task;
  `project::header` and `project::tabs`, naming the project. Each builds
  nothing, returning false, while no section of its subject is shown.
- **The pages** (`pages`): `heading`, the page's level-1 heading in
  jig's words; `main(…) -> bool`, the main landmark with jig's page at
  the place in jig's arrangement (its heading, its header, its tabs
  unless the page shows every section its subject offers, then each
  section the reference UI shows, in their order), or a page not about
  the work; it builds nothing and returns false when the page is the
  application's (elsewhere: 18.2); `not_the_work(…) -> bool`, which
  opens the main and builds the page not about the work (pages.md,
  section 8), returning false, and building nothing, when the page is
  about the work or the application's; it takes the application's slot
  for its ways of signing in, which the sign-in page leaves after the
  printed link's, as the navigation leaves one for its entries.
- **`after_page`'s parts,** each building nothing when it has nothing to
  show:
  - `pages::owed`: the cards the page still owes, in a complementary
    landmark. A card is owed while it is deciding, leaving, or unknown
    whether made (18.3 and 18.4) and no section of the page and no card
    the application's page names shows it: its object named by none of
    the page's handles, or a proposal that left the plan's rows while
    the page shows that plan and no conversation the page shows names
    it. A card a shown conversation holds, at the call that made its
    object or a line deciding it (section 9), stays there, on a page
    that shows the plan beside it too, and is never owed. It is decided
    from the reference UI's state, never from the tree built so far,
    since slots are filled after it;
  - `confirmation::confirmation`: the open confirmation, a dialog, its
    cancel button naming Escape (skein's `ui.md`, 6.1);
  - `notices::notices`: the notices, a named region, and dismissing
    them.
- **`after_page`:** jig's default, the three in that order, built last
  (section 3.1).
- **`jig(b, ui, client, nav, words, bindings) -> bool`:** jig's default
  arrangement, as section 4 says.
- **Cards** (`cards::card`): one function per card, wherever jig shows
  it (objects.md, section 2).
- **The binding constructors** (section 5), for an application's own
  nodes on a page of the reference UI that act on jig's objects.

## 5. Bindings

Bindings and acts are skein-ui's (skein's `ui.md`, section 6): a binding
is an owner and a token; the acts are `Press`, `Input`, `Submit`,
`Toggle` and `Drawing`, which the view refuses on a node whose role does
not take them, as it drops a stale one; the view finds an act's node in
its last tree, and the view root hands the bound act to its binding's
owner's view. Shortcuts are
skein-ui's (skein's `ui.md`, 6.1), and so is a link's address (skein's
`ui.md`, 6.2). What is jig's:

- **jig's meanings.** The reference UI keeps, for the tree it last
  built, a bounded table (`JigBindings`) of what each of its tokens
  means (`JigMeaning`): go to a place, edit a field, submit, intend an
  act on an object, choose, confirm, dismiss, fold, filter, read more,
  sign out. Each family's view keeps its own.
- **Decoding:** `jig_client_view::decode` turns one of jig's tokens and
  an act into the reference UI's action, or nothing; the application
  steps the reference UI with it (18.1).
- **The binding constructors.** `JigBindings`' constructors issue a
  binding for one of jig's meanings, which jig's cards use, and which an
  application's own nodes on a page of the reference UI may carry to act
  on one of jig's objects (section 4): an act on an object is bound from
  the object as the reference UI holds it when the tree is built, so the
  decision names the revision of the document that object came from, and
  goes through the reference UI's confirmation. The binding's owner is
  jig, whatever node carries it: the view root hands its acts to jig's
  view, which decodes them as any of jig's. An application with no
  reference UI binds its own nodes with its own bindings, and sends its
  requests through the client itself.
- **A link's address, for a shell.** A link node of jig's resolves,
  without acting, to the reference UI's place, which the application
  prints as a path with its codec (pages.md, section 2), as skein-ui's
  addresses are given to a shell (skein's `ui.md`, 6.2).
- **The act on a drawing:** every owner's view matches it; jig's decodes
  it into nothing (section 17).
- **jig's shortcuts:** jig's nodes name none but Escape, which the
  confirmation's cancel button names (skein's `ui.md`, 6.1); an
  application names others on its own.
- **Two kinds of stale, both dropped, neither an error:** a node the last
  patches removed decodes to nothing, as skein-ui's view drops it; a
  node still there whose object has left the working set carries a
  handle the reference UI refuses.
- **Typed input is made here:** a field's digits become a number, with
  skein-ui's `number` (skein's `ui.md`, section 6), or nothing when they
  are not one, as the protocol layer makes bytes typed for the client.

## 6. The diff

The diff is skein-ui's (skein's `ui.md`, section 7): matching by key and
by position, patches, nodes never replaced while they can be updated,
bindings the view's, and bounded by the two trees. What is jig's are the
counts it reads:

- **A field's write count** is the reference UI's count of its own
  writes to that field (18.5): a matched field whose count did not move
  gets no value patch, whatever its text, so what the person is typing
  stays theirs.
- **The view's staleness** is the reference UI's count of what is shown,
  which moves with its own state and with each document the client
  reports changed for one of its handles: the view builds the next tree,
  and diffs it, when it moved.

## 7. Words

The machinery is skein-ui's (skein's `ui.md`, section 9): phrases named
by their owner, typed holes, plural classes, languages given as data, a
capacity checked when a phrasebook is loaded, and saying into a bounded
buffer, with skein-lib's decimal for numbers. What is jig's:

- **Facts in, words out.** The client's documents and the helpers carry
  typed facts; the reference UI says them, from tables that match jig's
  enums whole, so a new phase or hold is a build error until it is said.
- **jig's enums, in `words`.** The helper `words` (client.md, section
  19) holds jig's phrases for its enums (phases, holds, executors,
  outcomes, refusals) and their English, a phrasebook the reference UI
  says every enum with, loaded into skein-ui's machinery beside its own
  phrases. An application overrides it phrase by phrase: its own
  wording, or another language, as data, which the reference UI then
  says.
- **The reference UI's phrases** (`JigPhrase`, `phrases/jig.rs`) are the
  sentences of its pages and cards ("funds 40 from your pool: 120 left
  this period, 80 after"), with typed holes: a number, an amount, a
  span, a wall time, a name, and skein-ui's reference hole, which jig
  uses for a task: its title with its number as the reference, copied
  unchanged, whose span jig builds as a chip linking to its page
  (objects.md, section 3). Each family's view has its own.
- **English is jig's** (`phrases/english.rs`, and `words`' own): an
  application, or a deployment, may give another language, with the
  application's own phrases beside jig's.
- **The language** is the person's, as the shell reads it from the
  platform, or as the person chose it.
- **Names are copied in,** never translated: a task's title, a party's
  name, a resource's words.
- **A long path is shown cut.** A resource's path and a pattern print
  their segments as their bytes, copied in. When the phrase before them
  and their segments would pass a node's text, the view leaves out the
  first segments, shows in their place the mark `…`, a phrase of jig's
  an application may override, and then the last segments that fit, the
  last one always. Only what is shown is cut: what the view acts on (a
  filter by a resource, a draft's pattern) takes the whole path the
  client holds (client.md, section 16).
- **A shell's own words.** The words jig's look gives the browser's
  parts (a credential shown once, the application failed) are jig's
  phrases, in the person's language (`../../protocol/browser.md`, 3.3
  and section 5). The text shell's markers and refusals are skein's
  (skein's `ui-text.md`, sections 5 and 6).

## 8. Markdown

The reader is skein-ui's (skein's `ui.md`, section 10): the bounded
subset, the rest shown as the characters it is and never made into
markup of the platform's, one pass with bounded stacks. What agents and
people write in jig (replies, results, specs, notes) is read into nodes
by it, its headings below the level of the region it is read into; jig's
tests feed it agents' replies and results kept as files beside them.

## 9. Conversations

- **A conversation is a thread:** the helper `thread`'s `Thread`
  (client.md, section 19), built from the task's activity read in
  pages, the person's words pending and sent, and the run's provisional
  text, which the reference UI turns into nodes: each message's blocks,
  text as Markdown, each call a folded line naming its tool and its
  outcome, opened on demand to its input and its result. Turns are
  decoded below the client (client.md, section 10), so the reference UI
  links nothing of smith's.
- **Calls to the engine** show as the cards of what they made, live.
- **Provisional text** is said to be provisional in words: its region
  holds a status in jig's phrase for it, and carries jig's style token
  `jig-provisional`, its look (section 13). There is no provisional
  state (skein's `ui.md`, 3.2). It shows as it comes, and gives way to
  the turn once it commits, matched by its key so the place on screen
  holds.
- **The conversation is a log:** its role tells a shell to keep its end
  in view as it grows, while the person has not scrolled away.

## 10. Sized from limits

`skein_ui::Limits` and `skein_ui::worst_case` are skein-ui's (skein's
`ui.md`, section 15): the most nodes, the depth, a node's text and the
text bytes, a drawing's bytes, a node's style tokens, the patches, the
bindings, the slots, Markdown's depth, a scene's marks and bytes, and
the heap skein-ui's own containers take for them. jig's are the
constructors that derive the numbers from the reference UI's limits
(18.9) and the client's (client.md, section 14)
(`jig_client_view::limits::of`, `compose`, `of_words` and `envelope`,
each returning `skein_ui::Limits`), and jig's own scratch, which jig's
worst case adds to skein-ui's.

Each of jig's parts, sections and pages has a companion that counts the
most nodes it may build with every window full and every text at its
most bytes, and the largest page is the tree's capacity. A node's text
holds at least the phrase before a path, the mark and one segment at its
most bytes, so a path's node stays within it whatever its segments
(section 7), and the client's bound on a path's segments adds nothing to
the tree's limits. A page showing
sections together counts their sum, so the limits take the arrangement:
`limits::of` and `limits::compose` count jig's pages with the sections
the application's `Arrangement` adds beside each place's own. A
Markdown text builds at most a fixed number of nodes per byte. The
application's view root counts its own arrangement from the same
companions: the parts and sections it builds, its families' pages, its
own presentations of jig's objects and its slots' nodes, a section slot
counting as the section it may hold (`limits::compose`). A scene's
limits are counted the same way: the most marks and bytes any owner's
drawing code draws for data within its bound, in a box of any size
(section 17).

## 11. Shells

What a shell is, and each platform's, is skein's (skein's `ui.md`,
section 16): the only impure code; one callback, one step; time and the
seed as inputs; addresses as the platform's; a panic fail-stop;
landmarks placed; drawings drawn by their owner's drawing code; pictures
as files; components, a shell's native implementation of a node
(skein's `ui.md`, section 12). In a browser, skein's browser parts
(skein's `ui-web.md`). What is jig's:

- **The order that protects requests** is the client's and its
  transport's: a request is kept in the client's small store before it
  is sent, whatever the loop (client.md, section 11;
  `../../protocol/browser.md`, 5.3). The application's loop gives the
  reference UI's records to be written with the client's, in the same
  step's writes (18.8), shows the place the reference UI asked for, and,
  once no report waits, builds and applies the patches.
- **A panic is fail-stop:** the shell restarts the application, as a
  supervisor restarts a process, and the client's small store and its
  keys make what was pending safe to send again; the reference UI comes
  back to its place and its drafts from its records.
- **jig's shells and parts:**
  - *jig's browser parts* (`jig-client-shell-web`), beside
    `skein-ui-web`'s: the client's adapter, which carries the client's
    exchanges, its stream and its small store over skein's fetch,
    transport and small store through the default HTTP mapping; the
    printed link's sign-in; and jig's look (`jig.css`, jig's palette,
    its pictures, its phrases for the parts' words)
    (`../../protocol/browser.md`, sections 3 and 5). An application's
    browser shell on the reference UI is copied from jig's reference,
    `jig-client-test-shell-web`; its page and its bundle are served by
    the application's server, under its page's policy
    (`../../protocol/browser.md`, section 12);
  - *the native one* (`jig-client-shell-native`): skein's HTTP client
    and its server-sent events reader with `jig-client-protocol-http`,
    for Linux and for the worlds that make bytes real; it carries the
    client's messages, not the tree, and with no view it is a service's
    shell;
  - *the text shell:* skein's (`skein-ui-text`), which an application's
    terminal client composes with jig's settings, in front of the native
    one (section 15).
- **jig's regions are placed by their names:** in a browser, jig's
  stylesheet places them in jig's default arrangement, and an
  application's stylesheet wherever it puts them (section 13).
- **Drawings:** jig's view draws no kind of its own (section 17).
- **An application's shell,** a phone's or a desktop's, mapping the same
  tree onto its platform (skein's `ui.md`, 16.2 and 16.3), is the
  application's to write over the same client and view. Graphics beyond
  a scene (a canvas of hundreds of thousands of points, three
  dimensions, video) are not jig's: a component of the application's
  draws them (skein's `ui.md`, section 12).

## 12. Accessibility

- **The mapping is skein's:** each shell maps a node's role and name
  onto its platform's own, and a component keeps them (skein's `ui.md`,
  sections 12 and 16; skein's `ui-web.md`, section 6), so a screen
  reader reads what the fake person reads.
- **jig's nodes:** every node a person acts on has a role and a name;
  cards are named regions; buttons are named by their words; a
  conversation is a log; a plan is a tree whose items say whether they
  are open; the link's state and notices are statuses; provisional text
  says so in a status; a confirmation is a named dialog; a table's
  header cells, column headers and row headers, name its columns and
  its rows; a menu's items and a disclosure's content are in the tree
  whether shown or folded; every image's name is its text alternative.
- **In jig's default arrangement** the frame is a banner, jig's
  navigation a navigation landmark, the page a main, an aside
  complementary (section 3.1).

## 13. Styles

Style tokens are presentation hints (skein's `ui.md`, section 8): each
shell interprets the names it knows and ignores the rest, and no style
is a fact. What is jig's:

- **jig's style tokens** are a closed set (`JigStyle`): a card, a chip,
  a tone (muted, warning, danger, success), a button's weight, a dense
  list, provisional text. Each family's are its own.
- **Names for a stylesheet.** Each of jig's style tokens and region
  names has a name starting with jig's prefix, `jig-`: `jig-card`,
  `jig-chip`, `jig-muted`, `jig-warning`, `jig-danger`, `jig-success`,
  `jig-button-weight`, `jig-dense-list`, `jig-provisional`; its regions'
  as pages.md, 12.4 lists them. jig's look gives them with jig's
  pictures; an application's view root gives its own beside jig's, each
  starting with its name and a hyphen, plain data its shells give their
  parts (skein's `ui.md`, section 8; skein's `ui-web.md`, 3.2).
- **How tokens look** is the shell's: in a browser, `skein-ui.css`, then
  jig's stylesheet, which styles jig's tokens, places jig's regions and
  gives a palette of series colours an application's stylesheet uses,
  then the application's, which styles its own and may restyle jig's
  (`../../protocol/browser.md`, section 13); a native shell's theme.
- **No style is a fact.** Nothing a person must know is said by style
  alone: a danger is also in words, and so is provisional text.
- **Region names are not styles.** A region name says which region a
  node is, and a stylesheet places by it (the composer pinned below its
  thread, the budget aside); a style token says how a node looks. Both
  are closed per owner. jig's region names are listed in pages.md, 12.1
  and 12.2, and change only by a change to this design; an
  application's are its own, as its tokens are.

## 14. Open questions

- **The text shell's own words** in a language other than English are
  skein's question (skein's `ui-text.md`, section 9).
- **A markup macro,** if plain calls prove unreadable for jig's pages:
  one `macro_rules!` expanding to the same calls of skein-ui's builder,
  the one place a macro might live.
- **Right-to-left languages:** right-to-left text waits as vocabulary
  (skein's `ui.md`, 4.2); what they ask of jig's phrasebooks and pages
  beyond it waits with it.
- **Other browsers** than Chromium, which skein's browser kit drives
  (skein's `browser.md`, section 11).

## 15. The text shell

`skein-ui-text`, skein's text shell (skein's `ui-text.md`), prints a
tree as lines and reads each line a person types as an act on a node, a
print, or closing. It knows no application. An application's terminal
client is its client root (the client, with the reference UI or its own
presentation: root.md) and its view root, `jig-client-shell-native`,
composed with `skein-ui-text` and jig's settings (15.1), and whatever
the application prints of its own.

### 15.1 Its parts

- **The renderer, the command reader and the terminal** are skein's
  (skein's `ui-text.md`, section 3): the renderer and the command reader
  pure, step code under the subset, bounded by skein-ui's limits; the
  terminal ordinary Rust, on skein's io.
- **The host** is the application's: its loop over skein-shell's
  `drive` routes the terminal's io completions to it, gives it each tree
  the view root builds and the lines the application prints of its own,
  and takes each command from it.
- **jig's settings,** which the application gives whole, with no
  defaults: the name its own lines start with (its own, and at most the
  bytes jig's limits give a node's text: the terminal refuses a longer
  one); the longest line it reads (at least the longest words a person
  may write, with a number and a space before them); its print interval
  (a guide; `ops`'s is one second); and its status region, jig's link
  (`jig-link`, 15.3).

### 15.2 When it prints

skein's (skein's `ui-text.md`, section 4): the whole page, never a diff;
when the tree changed, at most once per print interval; at once after
the person's own command and on `p`; one print at a time. jig's lines
and the application's (a credential shown once, how to sign in again)
are written before the next print of the page, in the order they came,
each starting with the shell's name and `: `, so they are never taken
for the page's.

### 15.3 How it prints

skein's (skein's `ui-text.md`, section 5): each role's line, states as
words, text runs, control characters and bytes that are not UTF-8 made
harmless, lines not wrapped, style tokens and region names not printed,
since nothing a person must know is said by them alone (section 13).
What is jig's:

- **The first line,** `== TITLE == STATUS`, prints as STATUS the text of
  the status region jig's settings name, jig's link (`jig-link`): the
  link's state in jig's words, then `; ` and the page's being behind
  when it is. A tree with no such region, an arrangement that omits
  jig's link part, prints no status there.
- **Provisional text** prints as its status's words (section 9).

### 15.4 Numbers and commands

skein's (skein's `ui-text.md`, section 6): numbered nodes, the commands
and the refusals, the same for every application. An application adds
to its terminal only what it prints of its own (15.2), never a command.

### 15.5 Its loop

skein's (skein's `ui-text.md`, section 7): one line, one command; the
host's iteration; closing on `q`, the end of the input or a termination
signal. What is jig's:

- **Closing** closes the client without signing out. What is pending
  stays in the client's small store, sent at the next start with its
  keys (client.md, section 11), and the reference UI's drafts and place
  stay in its records (18.8).
- **Signing in** is the host's: a terminal follows no browser's
  navigation, so it signs the client in at start with the printed link's
  secret or a credential the application's own sign-in gave it
  (client.md, section 12), and the reference UI's sign-in page prints
  the application's own line saying how to sign in.

### 15.6 Its tests

- **skein's:** the renderer's and the command reader's golden text, and
  the terminal on skein's simulated machine (skein's `ui-text.md`,
  section 8).
- **jig's pages, printed by skein's text shell:** golden text kept as
  files beside jig's tests and changed only by hand, built by jig's test
  application's view root, `jig-client-test-view`, from fixture states
  (the inbox with entries of each kind, a task page with its
  conversation, a project's sections, the deployment, the pages not
  about the work), with the first line's link.
- **The reference UI's worlds play the text shell** (testing.md,
  section 4), and the client's protocol world runs one story with the
  terminal in front of its native client: lines in on a pipe, pages out
  on another (testing.md, section 2).

## 16. Later

Each waits for the application that needs it, and adds without changing
what sections 3 to 5, 15, 17 and 18 settle.

- **jig's own drawing kinds:** a chart every application shares, drawn
  by jig's `draw` (section 17). A kind moves into jig, as jig's, when a
  second application draws the same.

## 17. Scenes

Scenes are skein-ui's (skein's `ui.md`, section 11): a drawing is drawn
once, by its owner's drawing code, for every shell that draws, as a
bounded list of marks each such shell maps onto its platform. jig's view
draws no kind of its own (section 16).

### 17.1 Drawing code

- **skein's shape** (skein's `ui.md`, 11.1): the owner's step code,
  `draw(kind, data, width, height, scene) -> bool`, of its inputs alone,
  false for a kind or data it does not draw, whose fallback then shows.
- **Routed by owner.** The view root's `draw` routes each drawing by its
  kind's owner, exhaustively, as it routes slots (section 4). jig's
  `draw` returns false (section 16).
- **Given to a shell's parts** with the look, at start (skein's
  `ui-web.md`, 3.2).

### 17.2 The scene

skein's (skein's `ui.md`, 11.2): marks in whole pixels, each with style
tokens, a hint and an act; no colour; a scene past its bounds refused
whole, never cut. jig's: a mark's tokens are its drawing's owner's or
jig's, and jig's stylesheet gives the palette of series colours an
application's stylesheet uses (section 13); a scene's words come in its
data, written by its owner's view with its phrasebook (section 7).

### 17.3 Drawings acted on

skein's (skein's `ui.md`, 11.3): the press order, `Act::Drawing { data
}` on the drawing's node, and every act on a drawing with an ordinary
node beside it doing the same, which the fake person, a screen reader
and the text shell use. jig's: the act is decoded by the drawing's
owner's view (section 5), and jig's decodes it into nothing.

### 17.4 In each shell

skein's (skein's `ui.md`, 11.4; in a browser, skein's `ui-web.md`,
section 8): drawn again when the drawing's data or its box's size
changes; SVG, or a component, in a browser; the fallback read by the
text shell and the fake person. jig adds nothing.

### 17.5 Its tests

Golden scenes, printed as lines, fix drawing code (skein's `ui.md`,
sections 11 and 17); a scene as SVG is skein's browser parts' to test
(skein's `ui-web.md`, section 14). jig's are its test family's: its
drawing code's golden scenes and its act (testing.md, section 4).

## 18. Its state

What of the reference UI is not a tree: where the person is, what the
page holds, the requests it sent, what the person is in the middle of,
and its notices. It is step code under the subset, as the client is
(programming-model.md, section 3), with its own limits, worst case and
stories (18.9, 18.10). It keeps no copy of a watch's document: it reads
each from the client, by its handles, when the client reports it
changed. A read's page, which the client hands over once, in the read's
terminal, and keeps no copy of (client.md, 7), the reference UI keeps
until the page that read it is left or the handle's next page replaces
it (18.3). It decides nothing the engine or the client decides: what is
made, when, and once, is theirs.

### 18.1 Its surface

```
jig_client_view::Ui: step(now, wall, &Client, Event, &mut Queue<Request>), fire(now, wall, &Client, &mut Queue<Request>), next_deadline()

made with: its limits; the application's navigation and home; its arrangement

Event, in
  from the application:
    Start { records, place, seed }  its own records, as the application kept them; the place the client starts at; its seed
    Went { place } | Left           a place of the reference UI's gone to; a place of the application's
    Act { action }                  a person's act on one of its nodes, as its bindings decoded it (section 5)
    Named { objects, sections }     what the application's page names of jig's, the whole list each time (pages.md, 12.5)
    Client { told }                 what the client tells under jig's asker, and the link, as the root routes them (client.md, section 4)
Request, out
  for the client:      Watch | Unwatch | Read | Send | SignOut               the client's events, under its handles and keys
  for the application: Show { place, how } | Save { records } | Erase { records } | ShowPart { part, scope } | PartsLeft
read by the application: ui.sections(), ui.shown()
```

- **A child of the client root,** as any presentation is (root.md,
  section 2). The root steps it with the person's acts and the places
  gone to, which its shell reports, and with what the client tells of
  its handles and keys and of the link, routed by asker (root.md,
  section 5).
- **It asks the client through the root:** each request for the client
  is one of the client's events, which the root gives the client in the
  same step; the reference UI never holds the client, and reads it only
  by the `&Client` the root lends it (`document`, `text`, `pending`,
  `link`).
- **It asks for nothing else of the client's, and reports nothing.** It
  never asks to sign in: signing in is the application's (18.6). It
  never continues a run's watch: the client does, from the text it
  holds (client.md, 8.3). A limit that refuses is said where it refuses,
  on the page or in place (18.2, 18.4), never as a report beside it.
- **`now` and `wall`,** as the client's (client.md, section 4): the
  monotonic instant for its deadlines (18.7), and the wall time it shows
  the engine's wall times against ("waiting 4 min").
- **Its handles and keys** carry its asker, jig's owner, 65536 (section
  3; root.md, section 15), which the client hands back with everything
  it tells of them. It numbers its handles itself, and draws its keys'
  numbers from its seed, fresh for each request it means (client.md,
  sections 4 and 6).
- **Navigation, home and arrangement** are given when it is made, as
  data, and never change after: which of jig's pages the frame offers,
  in what order, which is home, and the sections its arrangement adds
  (pages.md, section 2 and 12.5).
  A navigation naming a page twice, or an arrangement pages.md, 12.5
  refuses, is refused when it is made.
- **`ShowPart` and `PartsLeft`** ask the application to show a part in a
  slot, naming the project or the deployment it shows in, and tell it
  the page's parts have left (root.md, section 6).
- **No loop, no call.** It runs no loop of its own, and never calls the
  application or the client: it is stepped, and asks.

### 18.2 The place and the page

- **One page at a time.** The page at the place is one of the reference
  UI's (pages.md) or the application's, which it knows only as
  elsewhere, and by the sections and objects of jig's the application's
  page names (`Named`). Pages that are not about the work (starting,
  signing in, a page missing, a client out of date, someone else's
  work: pages.md, section 8) are pages too, whatever the place.
- **`me`, always.** It watches `me` from its start, whatever the page,
  by a handle of its own, which shares the client's own watch of it
  (client.md, 8.2): the frame's parts, the templates the pages fill, the
  bounds a draft is compared with, and the party whose work is kept
  come from it (contract.md, 5.1).
- **Going to a page** closes the old page's watches (`Unwatch`), forgets
  its reads and the pages they were handed (18.3), tells the application
  its parts have left (`PartsLeft`), drops from the working set every
  object no request in flight is about (18.3), then opens the new
  page's watches and reads (pages.md, 12.5), and asks for its place to
  be shown (`Show`). Requests in flight, `me`, drafts and notices carry
  over.
- **Handles, not generations.** Each watch and read a page opens has a
  handle of the reference UI's; what the client tells of a handle the
  page does not hold is dropped. The client drops what comes for a watch
  closed or a read abandoned, so a handle may serve the next page at
  once, and a `Watch` on a handle already open moves it to the new
  subject (client.md, section 7 and 8.1).
- **The link is the client's** (`Link`; client.md, 8.2), which the
  reference UI judges nothing of: starting, live and offline are the
  frame's link part's to say; signed out and out of date are pages not
  about the work (pages.md, section 8).
- **Behind** is the page's: a handle of the page whose state the client
  gives as behind (client.md, 8.1) makes it behind, and the page says
  what it shows may be old, beside the link's state. On an application's
  page, each section of jig's says it in its own region (pages.md,
  12.5).
- **Full is a refusal the page shows.** A page whose watches or reads
  would pass the reference UI's limits opens what fits, and says what it
  left out; nothing grows. A handle the client ends, or a read it
  answers, refused by the client's own limit (client.md, 7 and 8.1), is
  one the page left out.

### 18.3 The working set

- **Objects by key, once.** Each object a card shows is held once, under
  the key the engine names it by: a task, a proposal, an escalation, a
  question, a person task, a result. One object may show in several
  places on a page (an escalation in a task's overview and in its plan);
  it is one object, with one card.
- **Read from the client's documents.** Its facts are the documents the
  client holds for the page's handles (`client.document(handle)`), read
  each time the client reports one changed. The working set keeps only
  what no document of the client's holds: which handles name each
  object, its card's state, for a leaving object, how it ended, and the
  pages its reads were handed.
- **A read's page is kept by the page that read it.** A read has one
  terminal, `Page { handle, page }`, and the client keeps nothing of it
  once it is told (client.md, 7: "A read is one attempt ...: the
  application reads again if it wants"). The page keeps what the read is
  handed, under the read's handle: a project's settings, a page of a
  task's history or activity, an object's document read to say how it
  ended. A read's handle keeps one page; a window's keeps the rows its
  pages put in the window (client.md, 7: "Keeping rows across pages, as
  a list slides while a person scrolls, is the presentation's"). What it
  keeps goes when the page is left (18.2), or when a read on the same
  handle is answered with a page, which replaces it; a read answered
  with none leaves it, and the page shows why, as 18.2 says.
- **Sources, not reference counts.** Each object records which of the
  page's handles name it. Each change stands for the whole of what its
  handle names: an object no handle names leaves, unless a request about
  it is pending or it is leaving.
- **Each object carries the revisions of what may be decided about it,**
  as its document gave them: a held task its hold's, a task its spec's,
  a person task its own, a note its own. Each moves only when what is
  decided on changes, never with spend or a delegate's phase. An act
  names the one it decides (18.4).
- **A card's state** (objects.md, section 2):
  - *open;*
  - *deciding:* a request about it is pending, shown as asked, never as
    done;
  - *refused:* the answer refused it; the card says why, and stays;
  - *leaving:* it was decided, answered, withdrawn or ended, by this
    person or another; the card says what happened, by whom and when,
    and leaves after a linger.
- **Leaving, whichever comes first.** A card leaves on its request's
  answer ("you accepted it"), or when no document of the page names it.
  In the second case, a read of the object's document says how it ended
  ("Pat released it at 10:14") before it leaves. While a request about
  it is pending, it stays deciding until the answer, so the person
  learns how their own request ended.
- **Full is a refusal the page shows.** A document naming more objects
  than the working set holds keeps what fits in its order, and the page
  says how many more there are.

### 18.4 Requests

- **Asked once per intent.** A card's act opens its confirmation (18.5);
  confirming it sends one request through the client (`Send`), under a
  key of its own, unless one for the same act on the same object is
  pending, which the second press joins. A double press and a retry
  therefore carry the request's one key, and a reload carries it as the
  client keeps it (client.md, section 11).
- **One request per act.** What the person sees as one act is one
  request: one of jig's operations, or one compound request
  (contract.md, 3.4): amending and releasing a held task; words that
  release a stopped chat; closing a chat with what becomes of its live
  delegates; reordering goals. The reference UI never chains two
  requests, so it never has to undo the first when the second is
  refused.
- **Shapes through the helpers.** A chat is started with `shapes::chat`,
  a goal set with `shapes::set_goal`, a chat closed with
  `shapes::close_chat` (client.md,
  section 19), each filling the project's template as `me` serves it; the
  reference UI sends what they give, and knows no shape of its own.
- **Shown as asked.** The card is deciding from the confirmation until
  the answer: "being decided", "retrying", "waiting for the engine", as
  the client reports the request's progress (`Pending`,
  `client.pending(key)`). Nothing is shown as done before its answer,
  which the client reports only once its commit is durable (client.md,
  section 6).
- **Said before it is sent.** An object's document offers each act:
  made; proposable, and to whom; or refused, and why; setting a goal and
  amending are offered with bounds (contract.md, 5.3). The form
  and the confirmation compare the draft with the offer's facts (a
  number no larger, a grant among those listed) and say which before
  anything is sent: within them, the operation is sent; beyond what the
  party may do alone but proposable, a `Propose` is sent, the
  confirmation saying to whom; refused, it says why, as when nobody may
  accept the proposal, and sends nothing. That is the engine's word
  read, never authority computed: the engine judges the request again
  when it comes.
- **The answer, applied.** Done: the card shows the outcome (made, or
  proposed, naming the proposal), and leaves if its object stopped
  waiting. Refused: the card says why, from the refusal's facts
  (contract.md, section 4), and offers what fits: look at the object
  again, where it moved; change what was asked, where a limit or the
  party's authority refused it; propose it, where a `Create` or an
  `Amend` was refused for want of authority and the refusal says a
  proposal could be accepted (objects.md, section 9).
- **Unknown.** A request the client could not send again within the
  engine's retention may or may not have been made (client.md, section
  6): the card says so, and where to look.
- **Not kept.** A request whose record the client could not write was
  never sent (client.md, section 11): the card says so, and offers it
  again. So it does for one answered unconfirmed, sent before `me` named
  the party (client.md, section 6).
- **Words are a request too.** A composer's words are a `Message`, shown
  as being sent while pending, as sent once answered, and as read once a
  committed turn took them (section 9).
- **Asks name what they saw.** A release, and a decision about a held
  task, name its hold's revision; an amendment, the task's spec's; an
  answer to a person task, its own; a note's correction or deletion, the
  note's: each as the document the card was built from gave it. The
  engine refuses one whose revision has moved, saying what it is, and
  the card asks the person to look again (contract.md, 5.2). A
  priority-only amendment names none, and the first commit decides.
- **Kept work.** A request the client restored at start under jig's
  asker (`Kept`, root.md, section 8) has no card: the reference UI keeps
  no intent across a restart, since its records are its drafts and its
  place (18.8). Refused, or unknown whether made, it shows as a notice
  (18.7); made, it shows in what it made. Kept for another party, it is
  counted for the page someone else's work (pages.md, section 8). Kept
  work the client dropped at start, unread, is told by how many
  (`Dropped`; client.md, section 11; root.md, section 8): a notice says
  that many requests kept from before were not sent.
- **Bounded.** The intents it keeps are a limit; an act past them is
  refused in place ("too many requests in flight"), never queued; so is
  one the client refuses for want of room.

### 18.5 Drafts and the confirmation

- **A field** is something a person types into, named by what it is for:
  the new chat's words, a task's composer, the open confirmation's
  reason, an amendment's fields, a new goal's. It keeps the text, and a
  count of the reference UI's own writes (a composer cleared once its
  words are sent, a draft restored), so the view writes a field's value
  only when that count moved and the person's typing is never
  overwritten (section 6).
- **Composers keep their drafts per task,** across pages and reloads,
  among the reference UI's records (18.8), bounded by the drafts kept;
  the oldest untouched leaves first.
- **One confirmation at a time:** the act, the object and the revision
  the act names, as the object's document gave it when the confirmation
  opened, the choices made (an option of a person task, what becomes of
  each live delegate on closing), and what is missing (a reason; a draft
  its offer's bounds say would be refused, 18.4). What it says comes from
  the object's document through `facts` (client.md, section 19): what an
  acceptance grants and costs and the pool after, what a cancel ends,
  what a release will do (contract.md, 5.4), so the view says it
  without asking anything.
- **The confirmation is the reference UI's.** The engine asks for none,
  and an application confirms as it chooses (contract.md, 5.4); the
  reference UI opens one on every act that decides something, and sends
  nothing until it is confirmed.
- **A revision that moves under an open confirmation** marks it changed:
  a held task's hold, under its release. Confirming it then is refused in
  place, saying what changed, rather than sent. What the act does not
  name (spend, a delegate's phase) changes the confirmation's words, not
  its validity. Bounds that move under an open confirmation, as a newer
  document gives them, are compared with the draft again, and the
  confirmation says what it will be: made, proposed, or refused.
- **A confirmation's fields** are cleared when it closes, confirmed or
  dismissed.
- **Folds the reference UI keeps** leave what they fold out of the tree:
  a plan's level folded into counts, a call folded to its line. They are
  its state's because opening one changes what is read or built. A
  disclosure, whose content is always in the tree, is the shell's to
  fold, and the reference UI never knows it (section 3).

### 18.6 Signing in and out

- **Signed out:** the link says it (client.md, 8.2 and section 12), and
  the client ends every handle. The reference UI shows the sign-in page
  (pages.md, section 8), keeping the place to come back to; the requests
  in flight wait, parked, in the client.
- **Signing in is the application's:** the printed link's secret or its
  own sign-in's credential, which its shell or its host gives the
  client (client.md, section 12), or a browser's cookie, which the
  client never sees. Once `me` names the party, the reference UI returns
  to the place it kept.
- **Signing out,** from the frame's party part, sends the client
  `SignOut`; the reference UI shows the sign-in page once the client
  reports the party signed out.
- **A sign-out that does not reach the engine** leaves the person signed
  in, as the link says (client.md, 8.2 and section 12): the frame says the
  sign-out did not reach the engine and offers it again; nothing is
  shown done before it is (testing.md, 6.3). A sign-in the engine ends
  itself, before the sign-out or after it, ends what was asked: the link
  says signed out, and the reference UI shows the sign-in page, as
  above; it never keeps the work page once the link says signed out.
- **A service** has no reference UI, and never sees a sign-in page.

### 18.7 Time and notices

- **Wall times are for showing:** the engine's records carry wall times,
  and "waiting 4 min" is the view's, from an object's wall time and the
  step's. While a page shows any, a tick once a minute counts a change to
  what is shown, so the view builds again.
- **Deadlines are the reference UI's:** a card's linger, a notice's
  life, the coalescing of drafts' saves, on the monotonic clock
  (programming-model.md, section 9). Backoffs and heartbeats are the
  client's (client.md, section 8).
- **Notices** are short-lived, for what has no card: a request answered
  for an object not shown, a refusal with nothing to show it on, a
  kept request of jig's refused or unknown, kept work dropped at start
  (18.4), the link going and coming back. A card's own outcome is on
  the card.

### 18.8 Its records

- **Its own records:** the drafts, and where the person was (the place).
  Nothing else: requests are the client's to keep (client.md, section
  11), and everything shown is read again.
- **Kept through the application.** The reference UI asks the
  application to save and erase them (`Save`, `Erase`), and is given them
  back at `Start`. The application keeps them where it keeps the
  client's small store, under keys of the reference UI's own, apart
  from the client's records, which the client's own saves carry alone:
  in a browser, skein's small store, the tab's session storage
  (`../../protocol/browser.md`, section 10); a file, for a terminal.
- **Written with the client's.** The application gives the transport a
  step's records of the reference UI's among the step's writes, with
  the client's, so they are written before the step's exchanges leave:
  a composer cleared because its words were sent is never restored
  beside the kept request that carries them.
- **Versioned** by the reference UI, so a client updated since reads
  what an older one kept, and a record it cannot read is dropped, never
  guessed: a draft lost, a place forgotten for the home.
- **Bounded:** the drafts kept, a draft's bytes and a place's are
  limits.

### 18.9 Limits and the worst case

Each is configured and part of its worst case:

- objects in a page's working set; rows per window; turns a
  conversation's window holds; nodes of a plan held; the pages its reads
  were handed that it keeps (18.3), one per read handle, each priced at
  the client's bytes of a page handed over (client.md, 14);
- its handles (watches open, `me`'s included, and reads in flight) and
  intents kept;
- these, and the objects, windows and pages above, counted for the most
  sections a page may show together: at a place of jig's, its own with
  the arrangement's beside them; on an application's page, every
  section of one task, one project and one inbox filter, with the
  conversations and the deployment (pages.md, 12.5);
- drafts kept; notices;
- bytes a person may write (the engine's own bound on words); bytes of
  any one text shown (a spec, a result, a turn); bytes of a provisional
  turn;
- the linger, a notice's life, the drafts' coalescing.

`worst_case` sums the containers' own and the bytes each entity may hold
(programming-model.md, 6.3), and bounds what the view builds: the
view's limits derive from these and the client's, so a tree always fits
(section 10). An application's worst case adds it to the client's
(client.md, section 14).

### 18.10 Its world

The reference UI's stories, in a person's terms, run in jig's client
world with jig's test application and the fake person (testing.md,
section 4), and in the joint world with the engine's domain (testing.md,
section 5): signing in and starting a chat; a held chat released; a
proposal accepted in the inbox while its chat shows it; two tabs
deciding one thing; a reload with requests pending; the engine
restarting under a pending request; a watch dropped and the page behind;
a page left with reads in flight; a draft kept across a reload; a goal
beyond the person's authority, proposed as the confirmation said it
would be; a chat closed with one live delegate cancelled and another
made the person's; goals reordered together; a task's page arranged
with every section together, a proposal accepted in its plan while its
conversation shows it; an application's page showing a task's plan in a
section slot, and a second task's not shown, its slot saying so.

Its referee's rules are testing.md's, section 6: those on the client's
surface, jig's check, and those on what the person saw.
