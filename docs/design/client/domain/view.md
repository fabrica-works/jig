# The view

Provisional, 2026-10-09, revised 2026-10-15. `jig-view`: jig's client
domain shown on skein's UI kit, `skein-ui` (skein's `ui.md`). It is
optional: a service has none. It holds jig's primitives, which build
parts of a tree from the domain's state (the frame's parts, the
sections, the cards, the confirmation, the owed cards and the notices)
or handle a person's acts on them (jig's bindings, which turn those acts
into typed actions); jig's default arrangement of them into pages; and
what an application needs to present jig's objects itself: the binding
constructors, a decision's facts and jig's check. The tree, the builder,
the diff, the outline's rules, words' machinery, Markdown, scenes,
limits and the text shell are skein's, and each section that names them
points there and keeps what is jig's: its owner, its region names and
tokens, its phrases, its limits' constructors, its settings of the text
shell (section 15) and its drawings (section 17). What is still open is
listed in section 14, and what waits in section 16.

## 1. In one page

- **Built on skein-ui.** The tree and its vocabulary, owners, the
  builder, bindings and acts, the diff, words' machinery, Markdown,
  scenes, the outline's rules and limits are skein-ui's (skein's
  `ui.md`); the text shell is `skein-ui-text`'s and the browser's parts
  `skein-ui-web`'s. jig-view is what of a client's view is jig's.
- **State in, tree out.** Each of the frame's parts, each section of a
  page, each card, the confirmation, the owed cards and the notices is a
  function from the domain's state to tree nodes, through skein-ui's
  builder, one node a call. A card has one function, called wherever jig
  shows it (objects.md, section 2).
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
- **The promise, kept without mandated nodes.** A deciding act goes
  through the domain's confirmation and names the revision its binding
  was issued from, whatever the tree. What a card shows besides, an
  application presenting jig's objects itself takes from jig as data
  (`jig_view::facts`), and its worlds check it showed it
  (`jig_view::check`), never at run (section 4).
- **Accessible because tested.** Every node a person acts on, or reads
  to decide, has a role and a name, since the fake person finds nodes
  only by role, name and text (testing.md, section 3). A node it cannot
  find is a defect of the page.
- **Bindings are tokens.** A node a person acts on carries its owner's
  token, never a closure (programming-model.md, 4.2), as skein-ui's
  bindings are (skein's `ui.md`, section 6). jig's bindings say what
  each of jig's tokens means and decode an act into jig's action
  (section 5).
- **Diffed, not replaced,** by skein-ui's diff (skein's `ui.md`, section
  7): a matched node keeps its id, so the platform keeps focus, caret,
  selection and scroll; a field's value is written only when the domain
  wrote it (section 6).
- **Slots, spliced.** jig's pages and cards leave slots for an
  application's parts, and an application's pages leave them for jig's
  cards and sections; the view root fills each from its owner's view
  (section 4).
- **Owners, mapped.** jig's typed owner, jig or one of the application's
  by the number both its roots share, maps onto skein-ui's opaque owner
  (section 3; root.md, section 15).
- **Words from a phrasebook.** Phases, holds, amounts and times are said
  in jig's phrases by skein-ui's phrasebook: jig ships English, and an
  application may give its own as data (section 7).
- **Sized from the domain's limits,** by jig's constructors of skein-ui's
  limits, so a tree always fits (section 10).
- **No framework.** The view is step code under the subset, as the
  domain is; a framework's closures, signals, hooks and scheduler are
  what the subset leaves out (programming-model.md, section 10). The
  domain and its worlds would not change if one rendered the same tree.
- **skein's shells, with jig's parts.** A terminal client composes
  skein's text shell with jig's settings (section 15); a browser shell
  composes skein's browser parts with jig's sign-in part, the link's
  adapter and jig's look (section 11).

## 2. Structure

```
jig-view/src/
├── owner.rs        jig's owner, and its mapping onto skein-ui's (section 3)
├── regions.rs      jig's region names, each with its name (pages.md, 12.4)
├── styles.rs       jig's style tokens, each with its name (section 13)
├── binding.rs      jig's bindings: its meanings, their table, the binding constructors, decoding (section 5)
├── phrases/        jig's phrases and their holes (jig.rs), and English (english.rs) (section 7)
├── words.rs        phases, holds, executors, amounts, spans, times, in jig's phrases
├── turns.rs        a conversation's turns and its provisional text (section 9)
├── frame.rs        the frame's parts, and jig's banner (section 4.1)
├── confirmation.rs the confirmation
├── notices.rs      the notices
├── cards.rs        each card, and their shared parts: head, context, state
├── sections/       each section of jig's pages, and the pages' headers (section 4.1)
├── pages.rs        jig's default arrangement, `after_page` and the owed cards; the pages not about the work
├── facts.rs        a decision's facts, as jig's sayings (section 4)
├── check.rs        jig's check (section 4)
├── draw.rs         jig's drawing code, which draws no kind (section 17)
├── limits.rs       jig's constructors of skein-ui's limits, and jig's scratch (section 10)
└── tests.rs
```

It depends on `skein-ui` and `jig-domain-client`. What every
application's view shares is skein-ui's, and jig-view uses it without
repeating it: the tree and its vocabulary, owners, the builder with
slots and splicing, bindings and acts, the diff and its patches, the view
that keeps the last tree, tokens, region names and the look's names, the
phrasebook's machinery, the Markdown reader, scenes, the outline's rules,
and `Limits` with its worst case (skein's `ui.md`, section 2).

The application's view root (`<application>-view`) depends on jig-view,
on skein-ui, and on its families' views.

## 3. The tree

The tree is skein-ui's (skein's `ui.md`, section 3): nodes in preorder,
with ids kept while the diff matches them; its closed roles, its states
and properties, owners, text, values and keys, images and drawings
(skein's `ui.md`, 3.1 to 3.5); and no layout. There is no provisional
state: jig says provisional text is provisional in words (section 9).
What of the tree is jig's:

- **jig's owner.** `jig_view::Owner` is `Jig`, or `Application {
  number }`, one of the application's owners by its `u16` (root.md,
  section 15); there is no other kind, so an owner that is not a
  connector's (root.md, 15.1) names its nodes as a connector's side does.
  It maps onto skein-ui's opaque owner (skein's `ui.md`, 3.3) by a total
  function, an application's number to the same number and jig to
  65536, one past every `u16`, so the two never meet; and back by a
  partial one, which gives none for a number that names no owner. Both
  are matched whole. Every key, style token, region name, binding, slot,
  drawing kind, picture and phrase jig's view builds names jig.
- **jig's names in the tree:** its keys, one kind per kind of object, so
  lists diff by object; its region names (pages.md, 12.4); its style
  tokens (section 13); its pictures, in jig's look (section 13); no
  drawing kind (section 17).
- **A field's value** carries `jig-domain-client`'s count of its own
  writes to the field (client.md, section 9) as skein-ui's count of the
  owner's writes (skein's `ui.md`, 3.4).
- **Menus and disclosures** are skein-ui's (skein's `ui.md`, section 3):
  a card or a page offers the acts past the few it shows in a menu, and
  a disclosure's content stays in the tree while a shell folds it, the
  fold the shell's and lost on a reload. A fold that leaves what it
  folds out of the tree is the domain's instead (client.md, section 9).
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
  application's (testing.md, section 6).
- **jig adds none.** An application places jig's frame's parts and
  `after_page`'s parts where it likes, or omits them; what a person
  loses when one is omitted is said in pages.md, section 2 and 12.1.
- **jig's default arrangement** (`jig_view::jig`; pages.md, 12.3) keeps
  this outline, as defaults an application may use, copy or ignore:
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
  inside the main. jig-view's tests check that jig's default arrangement
  keeps its banner, with those three regions, and `after_page`'s parts
  last, on each of jig's pages (testing.md, section 2).

## 4. Building

- **The builder is skein-ui's** (skein's `ui.md`, section 5): one node a
  call, no macros. Building past the tree's capacity is an assertion:
  the capacity comes from jig's limits (section 10), so overflowing it
  is the view's bug.
- **jig's primitives** each build one part of a tree, or nothing, where
  the application calls them, and bind what they build with jig's
  bindings (section 5); 4.1 lists them:
  - *a card* is a function over the builder, the domain and its object,
    called wherever the card shows; its place changes its line of
    context only;
  - *a section* is a function reading the domain's state for one of the
    page's sections (client.md, 5.1) and calling the cards' functions
    for the objects it names;
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
     primitives, its own nodes and its families' views: at an address of
     jig's, jig's page or its own arrangement of the page's sections,
     headers and heading; at the application's address, its page, from
     its owner's view; on a page not about the work, jig's
     (`pages::not_the_work`), whatever the address, or its own, from the
     domain's page (client.md, section 5); then the page node is closed;
  2. each slot's nodes, in two passes, from the slot's owner's view,
     which the builder splices in at the slot, since it splices only a
     tree whose nodes are all closed.

  `jig_view::jig` is jig's default: it builds the banner
  (`frame::banner`), the navigation (`frame::navigation`, with the
  application's entries in their slot), jig's main (`pages::main`) and
  `after_page`, and returns true; when the domain's page is the
  application's (elsewhere: client.md, section 5) it builds the banner
  and the navigation only and returns false, and the root builds its
  main, then calls `after_page`, or places its three parts its own way.
- **Two deep.** A family's page may leave slots for jig's cards and
  sections, which the view root fills from jig's view, and those leave
  slots for parts in turn, which the second pass fills: a section's
  cards leave their parts' slots at the same depth as a card's. Slots
  therefore nest two deep, never more: the builder splices in two
  passes, and the tree's capacity counts both.
- **A slot left empty** shows as loading, and one its owner cannot show
  says so (a section the domain does not show on this page, a part its
  owner does not know); neither breaks the page.
- **jig's objects, presented by an application.** jig's card and section
  functions are the defaults. An application may instead present one of
  jig's objects itself, from jig's domain data: it names the object as a
  card's slot does (root.md, section 7), so `jig-domain-client` holds it
  in its working set, follows it live and counts it shown on the page,
  never owed; it reads the object's document from the domain and builds
  its own nodes, binding each act through jig's binding constructors
  (section 5), whose acts jig's decoders turn into jig's actions. jig's
  promise holds without mandating its nodes:
  - *held by construction, whatever the tree:* a deciding act goes
    through the domain's confirmation (client.md, section 9); it names
    the revision of the data its binding was issued from, and a revision
    that moved is refused (contract.md, 5.2); made once, nothing lost and
    nothing shown done before it is durable are the domain's and the
    engine's, not the tree's;
  - *exported as data:* `jig_view::facts` gives the facts a card's
    promise rests on, as jig's sayings (section 7): the object's state, a
    refusal and why, what a decision will do and whether it is
    dangerous, from the object's document and the open confirmation; the
    application says them with jig's words, in nodes of its own;
  - *checked, not mandated:* `jig_view::check` holds a tree a person
    acted on to one rule: a deciding act was made on a tree that showed
    its decision's facts, in the open confirmation or within the acted
    node's nearest named region. The referee runs it on every tree a
    person acted on, in jig's worlds and in each application's, so an
    application that presents jig's objects itself is held to it in its
    own worlds (testing.md, sections 6 and 8);
  - *the application's to show:* the confirmation's step and the
    revision in jig's bindings stay fixed, since they are the domain's,
    not the tree's; the rest of a card's promise is the application's to
    show when it presents the object itself, checked by jig's check in
    its worlds, never at run.

### 4.1 The parts, sections and pages, as functions

Each takes skein-ui's builder, `jig-domain-client`'s `Domain`, jig's
words and jig's bindings (`b, client, words, bindings`), and what its
line names; each builds the regions and landmarks pages.md, section 12,
names for it, with their region names, and each has a companion that
counts the most nodes it may build (section 10). Each is a primitive:
the application calls it where it likes, or not at all.

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
  the domain does not show that section on this page (client.md, 5.1).
  The headers, each shown with any of its page's sections:
  `inbox::header`; `task::header` and `task::tabs`, naming the task;
  `project::header` and `project::tabs`, naming the project. Each builds
  nothing, returning false, while no section of its subject is shown.
- **The pages** (`pages`): `heading`, the page's level-1 heading in
  jig's words; `main(…) -> bool`, the main landmark with jig's page at
  the address in jig's arrangement (its heading, its header, its tabs
  unless the page shows every section its subject offers, then each
  section the domain shows, in their order), or a page not about the
  work; it builds nothing and returns false when the page is the
  application's (the domain's page is elsewhere: client.md, section 5);
  `not_the_work(…) -> bool`, which opens the main and builds the page
  not about the work (pages.md, section 8), returning false, and
  building nothing, when the page is about the work or the
  application's.
- **`after_page`'s parts,** each building nothing when it has nothing to
  show:
  - `pages::owed`: the cards the page still owes, in a complementary
    landmark. A card is owed while it is deciding, leaving, or unknown
    whether made (client.md, sections 5 and 6) and no section of the
    page and no card the application's page names shows it: its object
    named by none of the page's sources, or a proposal that left the
    plan's rows while the page shows that plan (client.md, section 5)
    and no conversation the page shows names it. A card a shown
    conversation holds, at the call that made its object or a line
    deciding it (section 9), stays there, on a page that shows the plan
    beside it too, and is never owed. It is decided from the domain,
    never from the tree built so far, since slots are filled after it;
  - `confirmation::confirmation`: the open confirmation, a dialog, its
    cancel button naming Escape (skein's `ui.md`, 6.1);
  - `notices::notices`: the notices, a named region, and dismissing
    them.
- **`after_page`:** jig's default, the three in that order, built last
  (section 3.1).
- **`jig(b, client, nav, words, bindings) -> bool`:** jig's default
  arrangement, as section 4 says.
- **Cards** (`cards::card`): one function per card, wherever jig shows
  it (objects.md, section 2).
- **The binding constructors** (section 5), for an application that
  presents jig's objects itself.
- **`facts`:** from the domain, one of jig's objects or the open
  confirmation, and jig's words, a decision's facts as jig's sayings,
  within jig's limits (section 4).
- **`check`:** from a tree a person acted on, the node acted on, the
  action its act decoded into and that decision's facts (`facts`), none
  or the fault, naming the node (section 4; testing.md, section 6).

## 5. Bindings

Bindings and acts are skein-ui's (skein's `ui.md`, section 6): a binding
is an owner and a token; the acts are `Press`, `Input`, `Submit`,
`Toggle` and `Drawing`, which the view refuses on a node whose role does
not take them, as it drops a stale one; the view finds an act's node in
its last tree, and the view root hands the bound act to its binding's
owner's view. Shortcuts are
skein-ui's (skein's `ui.md`, 6.1), and so is a link's address (skein's
`ui.md`, 6.2). What is jig's:

- **jig's meanings.** jig's view keeps, for the tree it last built, a
  bounded table (`JigBindings`) of what each of its tokens means
  (`JigMeaning`): go to an address, edit a field, submit, intend an act
  on an object, choose, confirm, dismiss, fold, filter, read more, sign
  in. Each family's view keeps its own.
- **Decoding:** `jig_view::decode` turns one of jig's tokens and an act
  into jig's action, or nothing; the view root wraps it with its owner
  for the domain's root.
- **The binding constructors.** `JigBindings`' constructors issue a
  binding for one of jig's meanings, which jig's cards use and an
  application presenting jig's objects itself uses alike (section 4): an
  act on an object is bound from the object as `jig-domain-client` holds
  it when the tree is built, so the decision names the revision that
  data carried. The binding's owner is jig, whatever node carries it:
  the view root hands its acts to jig's view, which decodes them as any
  of jig's.
- **A link's address, for a shell.** A link node of jig's resolves,
  without acting, to jig's address, which the application's client host
  prints as a path with its codec (`../../protocol/browser.md`, section
  9), as skein-ui's addresses are given to a shell (skein's `ui.md`,
  6.2).
- **The act on a drawing:** every owner's view matches it; jig's decodes
  it into nothing (section 17).
- **jig's shortcuts:** jig's nodes name none but Escape, which the
  confirmation's cancel button names (skein's `ui.md`, 6.1); an
  application names others on its own.
- **Two kinds of stale, both dropped, neither an error:** a node the last
  patches removed decodes to nothing, as skein-ui's view drops it; a
  node still there whose object has left the working set carries a
  handle the domain refuses.
- **Typed input is made here:** a field's digits become a number, with
  skein-ui's `number` (skein's `ui.md`, section 6), or nothing when they
  are not one, as the protocol layer makes bytes typed for the domain.

## 6. The diff

The diff is skein-ui's (skein's `ui.md`, section 7): matching by key and
by position, patches, nodes never replaced while they can be updated,
bindings the view's, and bounded by the two trees. What is jig's are the
counts it reads:

- **A field's write count** is `jig-domain-client`'s count of its own
  writes to that field (client.md, section 9): a matched field whose
  count did not move gets no value patch, whatever its text, so what the
  person is typing stays theirs.
- **The view's staleness** is `jig-domain-client`'s count of what is
  shown: the view builds the next tree, and diffs it, when it moved.

## 7. Words

The machinery is skein-ui's (skein's `ui.md`, section 9): phrases named
by their owner, typed holes, plural classes, languages given as data, a
capacity checked when a phrasebook is loaded, and saying into a bounded
buffer, with skein-lib's decimal for numbers. What is jig's:

- **Facts in, words out.** The domain carries typed facts; jig's view
  says them, from tables that match the domain's enums whole, so a new
  phase or hold is a build error until it is said.
- **jig's phrases** (`JigPhrase`, `phrases/jig.rs`) are the sentences
  jig's view says ("held: out of tries", "funds 40 from your pool: 120
  left this period, 80 after"), with typed holes: a number, an amount, a
  span, a wall time, a name, and skein-ui's reference hole, which jig
  uses for a task: its title with its number as the reference, copied
  unchanged, whose span jig builds as a chip linking to its page
  (objects.md, section 3). Each family's view has its own.
- **English is jig's** (`phrases/english.rs`): an application, or a
  deployment, may give another language, with the application's own
  phrases beside jig's.
- **The language** is the person's, as the shell reads it from the
  platform, or as the person chose it.
- **Names are copied in,** never translated: a task's title, a party's
  name, a resource's words.
- **A shell's own words.** The words jig's look gives the browser's
  parts (a credential shown once, the client failed) are jig's phrases,
  in the person's language (`../../protocol/browser.md`, section 5 and
  3.3). The text shell's markers and refusals are skein's
  (skein's `ui-text.md`, sections 5 and 6).

## 8. Markdown

The reader is skein-ui's (skein's `ui.md`, section 10): the bounded
subset, the rest shown as the characters it is and never made into
markup of the platform's, one pass with bounded stacks. What agents and
people write in jig (replies, results, specs, notes) is read into nodes
by it, its headings below the level of the region it is read into; jig's
tests feed it agents' replies and results kept as files beside them.

## 9. Conversations

- **Turns** are the client's own (client.md, section 10): each message's
  blocks become nodes, text as Markdown, each call a folded line naming
  its tool and its outcome, opened on demand to its input and its
  result.
- **Calls to the engine** show as the cards of what they made, live.
- **Provisional text** is said to be provisional in words: its region
  holds a status in jig's phrase for it, and carries jig's style token
  `jig-provisional`, its look (section 13). There is no provisional
  state (skein's `ui.md`, 3.2). It shows as it comes, and is replaced by
  the turn once it commits, matched by its key so the place on screen
  holds.
- **The conversation is a log:** its role tells a shell to keep its end
  in view as it grows, while the person has not scrolled away.

## 10. Sized from the domain's limits

`skein_ui::Limits` and `skein_ui::worst_case` are skein-ui's (skein's
`ui.md`, section 15): the most nodes, the depth, a node's text and the
text bytes, a drawing's bytes, a node's style tokens, the patches, the
bindings, the slots, Markdown's depth, a scene's marks and bytes, and
the heap skein-ui's own containers take for them. jig's are the
constructors that derive the numbers from the domain's limits
(`jig_view::limits::of`, `compose`, `of_words` and `envelope`, each
returning `skein_ui::Limits`), and jig's own scratch, which jig's worst
case adds to skein-ui's.

Each of jig's parts, sections and pages has a companion that counts the
most nodes it may build with every window full and every text at its
most bytes, and the largest page is the tree's capacity. A page showing
sections together counts their sum, so the limits take the arrangement:
`limits::of` and `limits::compose` count jig's pages with the sections
the application's `Arrangement` adds beside each address's own. A
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

- **The order that protects requests.** The host, the application's loop
  over its shell's parts and its client host, runs one step for each
  event, then does what it asked, in order: the small store written,
  then jig's link's requests sent, then the address shown; once no
  report waits, it builds and applies the patches. The parts hold the
  order, so no order of the loop breaks it: `jig-shell-native`, and in a
  browser skein's transport, hold a step's requests until its small
  store's writes are done (skein's `ui-web.md`, 5.3).
- **A panic is fail-stop:** the shell restarts the client, as a
  supervisor restarts a process, and its small store and its keys make
  what was pending safe to send again.
- **jig's shells and parts:**
  - *jig's browser parts* (`jig-shell-web`), beside `skein-ui-web`'s:
    the sign-in part (the cookie, the printed link, the providers,
    signing out, a credential shown once); the link's adapter, from
    jig's link's requests to the transport's plain data and from fetch's
    reports to the link's events; and jig's look (`jig.css`, jig's
    palette, its pictures, its phrases for the parts' words). An
    application's browser shell, copied from jig's reference,
    `jig-test-shell-web`, composes them with skein's parts and its
    client host; its bundle, under the prefix `/jig/client/`, is served
    by the engine from its one origin under a strict content security
    policy (`../../protocol/browser.md`, sections 3, 11 and 12);
  - *the native one* (`jig-shell-native`): skein's HTTP client and its
    server-sent events reader, for Linux and for the worlds that make
    bytes real; it carries jig's link, not the tree, and with no view it
    is a service's shell;
  - *the text shell:* skein's (`skein-ui-text`), which an application's
    terminal client composes with jig's settings, in front of the native
    one (section 15).
- **jig's regions are placed by their names:** in a browser, jig's
  stylesheet places them in jig's default arrangement, and an
  application's stylesheet wherever it puts them (section 13).
- **Drawings:** jig's view draws no kind of its own (section 17).
- **An application's shell,** a phone's or a desktop's, mapping the same
  tree onto its platform (skein's `ui.md`, 16.2 and 16.3), is the
  application's to write over the same domain and view. Graphics beyond
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
  (skein's `ui.md`, 4.2); what they ask of jig's phrasebook and pages
  beyond it waits with it.
- **Other browsers** than Chromium, which skein's browser kit drives
  (skein's `browser.md`, section 11).

## 15. The text shell

`skein-ui-text`, skein's text shell (skein's `ui-text.md`), prints a
tree as lines and reads each line a person types as an act on a node, a
print, or closing. It knows no application. An application's terminal
client is its client root and view root, `jig-protocol-client-link` and
`jig-shell-native`, composed with `skein-ui-text` and jig's settings
(15.1), and whatever the application prints of its own.

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
  stays in the small store, sent at the next start with its keys
  (client.md, section 11).
- **Signing in** is the host's: a terminal cannot follow a provider's
  navigation, so pressing a provider on jig's sign-in page prints the
  application's own line saying how to sign in.

### 15.6 Its tests

- **skein's:** the renderer's and the command reader's golden text, and
  the terminal on skein's simulated machine (skein's `ui-text.md`,
  section 8).
- **jig's pages, printed by skein's text shell:** golden text kept as
  files beside jig's tests and changed only by hand, built by jig's test
  view root from fixture states (the inbox with entries of each kind, a
  task page with its conversation, a project's sections, the
  deployment, the pages not about the work), with the first line's
  link.
- **The client's worlds play the text shell** (testing.md, section 4),
  and the client's protocol world runs one story with the terminal in
  front of its native client: lines in on a pipe, pages out on another
  (testing.md, section 2).

## 16. Later

Each waits for the application that needs it, and adds without changing
what sections 3 to 5, 15 and 17 settle.

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
