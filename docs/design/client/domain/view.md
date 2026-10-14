# The view

Provisional, 2026-10-09, revised 2026-10-14. `jig-view`: the client
domain's state as a tree a shell shows, a person's acts coming back as
typed actions, and the diff between the last tree and the next. It is
optional: a service has none. It says what the tree is and the outline
every page keeps, how the frame's parts, the sections, jig's pages and
the cards are built, how an application arranges them and how its
slots are filled, bindings, the diff, words and their languages,
Markdown, conversations, what a shell does, and jig's text shell
(section 15). What is still open is listed in section 14, and what
waits in section 16.

## 1. In one page

- **State in, tree out.** Each of the frame's parts, each section of a
  page and each card is a function from the domain's state to tree
  nodes, through a builder, one node a call. A card has one function,
  called wherever it shows (objects.md, section 2).
- **Pages as a library.** jig's pages are compositions of those parts
  and sections, written with the same functions an application's view
  root calls. The view root arranges each page: jig's as it is, its own
  arrangement of the same parts and sections, or, at its own address,
  its page, which shows jig's cards and sections in slots (section 4;
  pages.md, section 12).
- **A semantic tree.** The tree says what each node is (a region, a
  heading, a list, a button, a text field, a log), the way an
  accessibility tree does, never how it looks. Each shell maps it onto
  its platform: HTML with its roles in a browser, a platform's widgets, or
  text.
- **Layout as semantics.** Every page keeps one outline of landmarks (a
  banner, navigation, the main, asides), and each of jig's regions
  carries a stable region name, so a shell and a stylesheet place them
  without depending on the order of nodes (section 3.1).
- **Accessible because tested.** Every node a person acts on, or reads to
  decide, has a role and a name, since the fake person finds nodes only
  by role, name and text (testing.md, section 3). A node it cannot find is
  a defect of the page.
- **Bindings are tokens.** A node a person acts on carries its owner's
  token, never a closure (programming-model.md, 4.2). The shell reports
  an act with the node's id; the view finds the node in its last tree and
  its owner's view decodes the token into an action.
- **Diffed, not replaced.** The view keeps the last tree, builds the next
  when the domain's count of what is shown moved, and emits patches. A
  matched node keeps its id, so the platform keeps focus, caret,
  selection and scroll; a field's value is written only when the domain
  wrote it.
- **Slots, spliced.** jig's pages and cards leave empty nodes for an
  application's parts, and an application's pages leave them for jig's
  cards and sections; the view root fills each from its owner's view,
  and the builder splices them in (section 4).
- **Owners, numbered.** A key, a style token, a region name, a binding,
  a slot, a drawing, a picture and a phrase each name their owner: jig,
  or one of the application's owners, by the number both its roots
  share (root.md, section 15).
- **Words from a phrasebook.** Phases, holds, amounts and times are said in
  words from a phrasebook per language: jig ships English, and an
  application may give its own as data (section 7).
- **Sized from the domain's limits,** so a tree always fits, and a diff's
  patches are bounded by the two trees.
- **No framework.** The view is step code under the subset, as the
  domain is; a framework's closures, signals, hooks and scheduler are
  what the subset leaves out (programming-model.md, section 10). The
  domain and its worlds would not change if one rendered the same tree.
- **A text shell of jig's.** The tree printed as lines and a typed line
  read as an act decide nothing of an application's, so they are jig's
  shell for a terminal (section 15).

## 2. Structure

```
jig-view/src/
├── tree.rs         the tree: nodes, ids, owners, keys, roles, states, style tokens, region names, shortcuts, pictures, text
├── outline.rs      the page's outline, and its check (section 3.1)
├── regions.rs      jig's region names (pages.md, 12.4)
├── builder.rs      the builder: nodes, keys, names, bindings, slots; splicing
├── binding.rs      bindings: an owner and a token; the shell's acts; decoding jig's
├── diff.rs         matching, ids, patches
├── render.rs       building when what is shown moved, diffing, keeping the last tree
├── phrases.rs      jig's phrases, their holes, the phrasebook's shape, English
├── words.rs        phases, holds, executors, amounts, spans, times, in phrases
├── markdown.rs     the Markdown subset, read into nodes
├── turns.rs        a conversation's turns and its provisional text
├── frame.rs        the frame's parts, and jig's banner (section 4.1)
├── confirm.rs      the confirmation
├── notices.rs
├── cards.rs        each card, and their shared parts: head, context, state
├── sections/       each section of jig's pages, and the pages' headers (section 4.1)
├── pages.rs        jig's pages: its arrangement of the parts and sections; the pages not about the work
├── limits.rs       limits derived from the domain's; the worst case
└── tests.rs
```

The application's view root (`<application>-view`) depends on it,
and on its families' views.

## 3. The tree

- **Nodes in preorder,** each with:
  - an id, stable across builds while the diff matches it;
  - a role, from a closed set: page; the landmarks banner, navigation,
    main, complementary, content info, form and search (section 3.1);
    region, heading (with a level), paragraph, text, list, item, table,
    row, cell, term and description, button, link, text field, text
    area, choice, choice group, check, menu, menu item, disclosure,
    dialog, status, alert, log, tree, tree item, tabs, tab, progress,
    time, code, quote, separator, figure, image, slot, and drawing;
  - a name, where its text does not give it;
  - states: disabled, expanded, busy, current, invalid, pressed,
    selected, provisional, submits on enter;
  - style tokens, from its owner's closed set (section 13);
  - a region name, on a landmark or a region, from its owner's closed
    set: which region it is, language-free and stable (pages.md, 12.4);
  - text, on text nodes only, as runs: plain, emphasis, strong, code, a
    link;
  - a value, on fields, with the domain's count of its own writes;
  - an address, on links, which the shell turns into what its platform
    shows;
  - a key, on items of a list, saying which object it is, so lists diff
    by object;
  - a binding, on what a person acts on, and a shortcut a binding may
    answer to (section 5);
  - a picture, on an image: its owner and a number from the owner's
    closed set of pictures.
- **Closed, so a new one breaks the build** wherever roles are matched:
  each shell's mapping, the fake person's queries.
- **Owners.** A node's key, style tokens, region name, binding, slot,
  drawing's kind and picture each name their owner, and so does each
  phrase (section 7): jig, or one of the application's owners by its
  number (root.md, section 15). In code, `Owner` is `Jig`, or
  `Application { number }`; there is no other kind, so an owner that is
  not a connector's (root.md, 15.1) names its nodes as a connector's
  side does.
- **A menu** is a named group of the acts a card or a page offers past
  the few it shows: its children are menu items, each bound as a button
  is. Its items are always in the tree; a shell may show them behind
  the menu's name, opened and closed with no step of the domain.
- **A disclosure** is a region a shell may show folded, to its name,
  with no step of the domain: its content is always in the tree, so the
  fake person, a screen reader and the text shell find what a browser
  folds. Its folding is the shell's, kept across patches because the
  diff keeps its id, and lost on a reload. A fold that leaves what it
  folds out of the tree is the domain's instead (client.md, section 9).
- **An image** shows one of its owner's pictures (a brand, an icon). Its
  name is its text alternative, which every image has, and which a shell
  without the picture shows. The tree carries no picture's bytes: each
  shell takes its owners' pictures when it is built (section 11).
- **An application's own kinds of node** are drawings: a chart of an
  incident's error rate, a map of a pool's environments, a timeline.
  A drawing node carries:
  - its kind: its owner and a number from the owner's closed set;
  - its data, as bytes its owner's view writes, within a bound the owner
    declares, which only the owner's drawing code reads;
  - a name, and a fallback: ordinary nodes saying what it shows in words
    and numbers ("error rate 30 % since 10:02, 2 % before"), which a
    shell without the drawing shows, a screen reader reads, and the fake
    person finds.

  A drawing is shown, not acted on, until drawings acted on are built
  (section 16): what a person does about it is an ordinary node beside
  it. Its data is compared whole by the diff, and a change is one patch.
- **No layout.** Where things go on a screen is the shell's and its
  styles', from landmarks, region names, roles, order and tokens. The
  tree never says where a region goes; it says what each region is. A
  phone and a terminal lay out the same tree differently.

### 3.1 The page's outline

Every page's tree has one outline, whoever arranged it, so a shell
places its parts and a screen reader moves between them by landmark:

- **The page node's children,** in order:
  1. the banner, first, holding jig's link (`jig-link`), what needs the
     party (`jig-needs`) and who is signed in (`jig-party`), on every
     page (pages.md, 12.1);
  2. the navigation, main, complementary and content info landmarks, in
     the order the arrangement gives them: exactly one main, at most one
     content info, any number of the others;
  3. last, what jig's `after_page` builds: the owed cards (a
     complementary landmark, `jig-owed`), when there are any; the
     confirmation (a dialog, `jig-confirmation`), when one is open; and
     the notices (a region, `jig-notices`), when there are any.

  Nothing else hangs from the page node, so every node a person reads or
  acts on is inside a landmark, the dialog, or the notices.
- **Forms and searches** are landmarks inside the others: a new chat's
  form, the notes' search.
- **A page not about the work** (pages.md, section 8) is inside the main.
- **Each shortcut at most once** among the tree's nodes (section 5).
- **Checked.** `jig_view::outline::check(tree)` gives the first fault it
  finds, or none: `Fault` says which rule a node breaks, and names the
  node. jig-view's tests run it on each of jig's pages, and the referee
  on every tree a person sees, in jig's worlds and in each
  application's (testing.md, section 6).

## 4. Building

- **The builder** owns the tree it builds: open, close, text, a key, a
  name, a state, a token, a region name, a value, a binding, a shortcut,
  a picture, a slot; one node a call, no macros. Building past the tree's capacity is an assertion: the capacity
  comes from the domain's limits, so overflowing it is the view's bug.
- **A card is a function** over the builder, the domain and its object,
  called wherever the card shows; its place changes its line of context
  only.
- **A section is a function** reading the domain's state for one of the
  page's sections (client.md, 5.1) and calling the cards' functions for
  the objects it names. A frame's part is a function too.
- **A page is an arrangement** of the frame's parts, its sections and
  the application's own nodes, under the outline of section 3.1. jig's
  pages are jig's arrangements, built from the functions of 4.1 alone,
  so jig's own pages prove the library an application uses.
- **A slot** is a node left for another owner's nodes, holding a
  reference and nothing else. It is of one of three kinds: a part's,
  which a jig page or card leaves for an application's value (root.md,
  section 6); a card's, which an application's page leaves for one of
  jig's objects; a section's, which an application's page leaves for one
  of jig's sections (root.md, section 7).
- **The view root** builds a client's whole tree, in this order:
  1. under its page node, the banner and the navigation: jig's
     (`frame::banner`, `frame::navigation`, with the application's
     entries in their slot), or the application's own, from jig's
     frame's parts and its own nodes (a brand, its own entries);
  2. the main, with any asides: at an address of jig's, jig's page
     (`pages::main`), or the application's arrangement of the page's
     sections, headers and heading with its own nodes; at the
     application's address, its page, from its owner's view; on a page
     not about the work, jig's (`pages::not_the_work`), whatever the
     address;
  3. `after_page`: the owed cards, the confirmation and the notices;
     then the page node is closed;
  4. each slot's nodes, in two passes, from the slot's owner's view,
     which the builder splices in at the slot, since it splices only a
     tree whose nodes are all closed.

  `jig_view::jig` builds the first three in jig's arrangement and
  returns true; when the domain's page is the application's (elsewhere:
  client.md, section 5) it builds the banner and the navigation only and
  returns false, and the root builds its main, then calls
  `after_page`.
- **Two deep.** A family's page may leave slots for jig's cards and
  sections, which the view root fills from jig's view, and those leave
  slots for parts in turn, which the second pass fills: a section's
  cards leave their parts' slots at the same depth as a card's. Slots
  therefore nest two deep, never more: the builder splices in two
  passes, and the tree's capacity counts both.
- **A slot left empty** shows as loading, and one its owner cannot show
  says so (a section the domain does not show on this page, a part its
  owner does not know); neither breaks the page.
- **Cards and sections stay jig's, whole.** An application places them,
  restyles them and fills their slots; it builds no node for jig's
  objects but through jig's card and section functions, since a card is
  where a promise shows (objects.md, section 2).

### 4.1 The parts, sections and pages, as functions

Each takes the builder, `jig-domain-client`'s `Domain`, jig's words and
jig's bindings (`b, client, words, bindings`), and what its line names;
each builds the regions and landmarks pages.md, section 12, names for
it, with their region names, and each has a companion that counts the
most nodes it may build (section 10).

- **The frame's parts** (`frame`, pages.md, 12.1): `link`, `needs`,
  `party`, `projects` and `go_to_task`, each a region of the banner;
  `navigation`, which also takes the application's slot, a navigation
  landmark of jig's entries with the application's slot after them; and
  `banner`, jig's banner: the landmark, with the five in that order.
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
- **`after_page`:** the cards the page still owes, the confirmation and
  the notices, built last (section 3.1). A card is owed while it is
  deciding, leaving, or unknown whether made (client.md, sections 5
  and 6) and no section of the page and no card slot of the
  application's page shows it: its object named by none of the page's
  sources, or a proposal that left the plan's rows while the page shows
  that plan (client.md, section 5) and no conversation the page shows
  names it. A card a shown conversation holds, at the call that made its
  object or a line deciding it (section 9), stays there, on a page that
  shows the plan beside it too, and is never owed. It is decided from
  the domain, never from the tree built so far, since slots are filled
  after it.
- **`jig(b, client, nav, words, bindings) -> bool`:** jig's whole
  arrangement, as above.
- **Cards** (`cards::card`): one function per card, wherever it
  shows (objects.md, section 2).

## 5. Bindings

- **A binding is an owner and a token.** jig's view and each family's
  view keep, for the tree they last built, a bounded table of what each
  of their tokens means: go to an address, edit a field, submit, intend
  an act on an object, choose, confirm, dismiss, fold, filter, read more,
  sign in.
- **Decoding:** the shell reports an act on a node by its id (a press, an
  input's whole value, a submit, a toggle, or an act on a drawing); the
  view root finds the node in its last tree, and hands the token to its
  owner's view, which decodes it into its owner's action, wrapped for
  the domain's root.
- **The act on a drawing** carries bytes of the drawing's owner, at most
  the view's limit on a drawing's bytes (section 10), past which the
  view drops it. Only the owner's view decodes them, into its action or
  nothing, as typed input is made below. It is `Act::Drawing { data }`
  in the shell's acts, beside `Press`, `Input`, `Submit` and `Toggle`,
  and every owner's view matches it: jig's decodes it into nothing. No
  drawing is bound until drawings acted on (section 16), so until then
  no shell reports it.
- **A shortcut.** A bound node may name a key it answers to: a letter
  `a` to `z` or a digit `0` to `9`, with no modifier, with the
  platform's primary one (Control; Command on macOS), or with Alt
  (`Shortcut { key, modifier }`). A shell with keys turns the key into a
  press of the first node in preorder that names it, unless that node is
  disabled; a shortcut with no modifier never while a field has the
  focus. It is no new act: the text shell and the fake person ignore it.
  jig's own nodes name none; an application names them on its own. A
  tree names each at most once (section 3.1).
- **Two kinds of stale, both dropped, neither an error:** a node the last
  patches removed decodes to nothing; a node still there whose object has
  left the working set carries a handle the domain refuses.
- **Typed input is made here:** a field's digits become a number, or
  nothing when they are not one, as the protocol layer makes bytes typed
  for the domain.

## 6. The diff

- **Matching:** the roots match; below two matched nodes, keyed children
  match by key and unkeyed ones by position among siblings of the same
  role.
- **Patches:** insert a subtree, remove a node, move a keyed node whose
  order changed, set a text, set an attribute, set a value. A matched
  node keeps its id; an unmatched old one is removed, an unmatched new
  one inserted with fresh ids.
- **Never replaced while it can be updated,** so the platform keeps focus,
  caret, selection, composition and scroll across a patch.
- **Values:** a matched field whose write count did not move gets no
  value patch, whatever its text: what the person is typing stays
  theirs.
- **Bindings are the view's:** a changed binding needs no patch, since
  the shell holds only node ids and whether a node is bound.
- **Bounded:** each node of either tree takes part in at most one patch
  beyond its attributes, so a diff fits the view's limit on patches.

## 7. Words

- **Facts in, words out.** The domain carries typed facts; the view says
  them, from tables that match the domain's enums whole, so a new phase
  or hold is a build error until it is said.
- **A phrase** is jig's name for a sentence the view says ("held: out of
  tries", "funds 40 from your pool: 120 left this period, 80 after"),
  with typed holes: a number, an amount, a span, a wall time, a name, a
  chip. Phrases are a closed enum, one per owner: jig's view has jig's,
  each family's view its own.
- **A phrasebook** is a language's words for every phrase: for each, its
  segments and holes in the language's order, and its forms by a
  number's plural class. It is data, given to the view at start, as
  limits are; jig ships English, and an application, or a deployment,
  may give another, with the application's own phrases beside jig's.
- **The language** is the person's, as the shell reads it from the
  platform, or as the person chose it.
- **Names are copied in,** never translated: a task's title, a party's
  name, a resource's words.
- **No formatting:** a text is written into a bounded buffer, with
  skein-lib's decimal for numbers; its capacity is the longest its
  phrase can make in any language given, checked when the phrasebook is
  loaded.

## 8. Markdown

What agents and people write (replies, results, specs) is read into nodes
by a bounded reader of the view's own:

- **the subset:** paragraphs and line breaks; headings, below the page's
  own levels; emphasis, strong and inline code; fenced code; lists,
  nested to a limit; quotes; links whose target is http or https, shown
  as text otherwise;
- **the rest is text:** tables, HTML, images, a delimiter never closed, a
  list nested too deep are shown as the characters they are, never
  dropped, and nothing is ever made into markup of the platform's;
- **a step machine:** one pass over the lines, with a bounded stack of
  open blocks and of open emphasis; no recursion;
- **fed real text:** its tests read agents' replies and results kept as
  files beside them.

## 9. Conversations

- **Turns** are the client's own (client.md, section 10): each message's
  blocks become nodes, text as Markdown, each call a folded line naming
  its tool and its outcome, opened on demand to its input and its
  result.
- **Calls to the engine** show as the cards of what they made, live.
- **Provisional text** shows marked as provisional, as it comes, and is
  replaced by the turn once it commits, matched by its key so the place
  on screen holds.
- **The conversation is a log:** its role tells a shell to keep its end
  in view as it grows, while the person has not scrolled away.

## 10. Sized from the domain's limits

The view's limits (the most nodes a tree may have, the patches a diff may
make, the depth of nesting, of Markdown's nesting, the bindings each
owner keeps, and a drawing's bytes, which also bound an act on a
drawing) are derived from the domain's: each of jig's parts,
sections and pages has a companion that counts the most nodes it may
build with every window full and every text at its most bytes, and the
largest page is the tree's capacity. A page showing sections together
counts their sum, so the view's limits take the arrangement:
`Limits::of` and `Limits::compose` count jig's pages with the sections
the application's `Arrangement` adds beside each address's own. A
Markdown text builds at most a fixed number of nodes per byte. The
application's view root counts its own arrangement from the same
companions: the parts and sections it builds, its families' pages and
its slots' nodes, a section slot counting as the section it may hold
(`Limits::compose`).

## 11. Shells

- **The only impure code.** A shell is the platform's: its event loop,
  its windows, its HTTP stack, its storage, its clock and its entropy.
  The callbacks a platform needs are the shell's, never step code's.
- **One callback, one step.** Each thing the platform reports (an act on
  a node, an answer, a delivery, a timer, the place changing) is one
  event. The shell runs the step, then does what it asked, in order: the
  small store written, then requests sent, then patches applied.
- **Time and the seed are inputs,** read by the shell.
- **The addresses** are the platform's: a URL in a browser, arguments on a
  command line.
- **A panic is fail-stop:** the shell restarts the client, as a supervisor
  restarts a process, and its small store and its keys make what was
  pending safe to send again.
- **jig's shells:**
  - *the browser's* (`jig-shell-web`): the client compiled to
    WebAssembly, with only the glue the toolchain generates; the tree
    mapped to HTML elements with their roles; session storage as the
    small store; served by the engine from its one origin under a strict
    content security policy;
  - *the native one* (`jig-shell-native`): skein's HTTP client and
    its server-sent events reader, for Linux and for the worlds that make
    bytes real; with no view, it is a service's shell;
  - *the text shell* (`jig-shell-text`): the tree printed as lines and a
    typed line read as an act, for a terminal or a pipe, in front of
    the native one (section 15).
- **Landmarks placed.** A shell places the outline's landmarks (section
  3.1): a browser's header, nav, main, aside and footer elements; the
  text shell prints them in the tree's order. A stylesheet places
  regions by their region names.
- **Drawings are drawn by the application's code in the shell.** A shell
  is ordinary Rust, so it takes, when it is built, the application's
  drawing code for each kind of drawing it shows, which reads the
  drawing's data and draws it on the platform (a canvas or vector
  graphics in a browser). A kind a shell has no code for shows its
  fallback.
- **Pictures are the application's files in the shell.** A shell takes,
  when it is built, each owner's pictures: the browser's will have them
  from the engine's origin, beside the application's drawings, when it
  is built (section 16). A picture a shell does not have shows as its
  image's name.
- **An application's shell:** a phone's or a desktop's, mapping the same
  tree onto its widgets, is the application's to write over the same
  domain and view.

## 12. Accessibility

- Every node a person acts on has a role and a name; cards are named
  regions; buttons are named by their words; a conversation is a log; a
  plan is a tree whose items say whether they are open; the link's state
  and notices are statuses; a confirmation is a named dialog.
- The frame is a banner, jig's navigation a navigation landmark, the page
  a main, an aside complementary (section 3.1); a menu's items and a
  disclosure's content are in the tree whether shown or folded; every
  image's name is its text alternative.
- The browser's shell maps roles to the platform's own, so a screen
  reader reads what the fake person reads.

## 13. Styles

- **Style tokens** are a closed set per owner: jig's (a card, a chip, a
  tone: muted, warning, danger, success; a button's weight; a dense
  list), and each family's own.
- **How tokens look** is the shell's: the browser's stylesheet, which jig
  ships with its tokens and the application extends with its own and may
  restyle; a native shell's theme.
- **No style is a fact.** Nothing a person must know is said by style
  alone: a danger is also in words.
- **Region names are not styles.** A region name says which region a
  node is, and a stylesheet places by it (the composer pinned below its
  thread, the budget aside); a style token says how a node looks. Both
  are closed per owner. jig's region names are listed in pages.md, 12.1
  and 12.2, and change only by a change to this design; an
  application's are its own, as its tokens are.

## 14. Open questions

- **The text shell's own words** in a language other than English: its
  markers are symbols, but its states and its refusals are English
  words (section 15).
- **A markup macro,** if plain calls prove unreadable for whole pages:
  one `macro_rules!` expanding to the same calls, the one place a macro
  might live.
- **Right-to-left languages,** and what they ask of the tree beyond the
  phrasebook.
- **Other browsers** than Chromium, which skein's browser kit drives.

## 15. The text shell

`jig-shell-text`: jig's shell for a terminal or a pipe. It prints the
view's tree as lines, and reads each line a person types as an act on a
node, a print, or closing. Nothing it decides names an application's
value, so it is jig's: an application's terminal client is its client
root and view root, `jig-protocol-client-link` and `jig-shell-native`,
composed with it, and whatever the application prints of its own.

### 15.1 Its parts

- **The renderer,** `jig_shell_text::render`: from the view's tree
  alone, the page's lines (15.3) and the last print's numbers (15.4). A
  pure function, step code under the subset, with no io and no clock.
  Its output is bounded: the most lines and bytes the view's largest
  tree prints, counted from the view's limits. The shell's name, which
  the first line prints when the main has no level-1 heading (15.3), is
  at most a node's text, so the same limits count it.
- **The command reader,** `jig_shell_text::command`: from one line, the
  last print's numbers and the view's last tree, one command (15.4): acts
  on a node (a press, a toggle, or a field's input followed, when it
  submits on enter, by a submit: jig-view's `Act`s), print, close,
  nothing, or a refusal with its line. Pure, step code under the subset.
- **The terminal,** `jig_shell_text::Terminal`: ordinary Rust, as shells
  are. It reads the input as lines, writes the output, and keeps when
  to print (15.2), on skein's io: its input adopted as a read pipe, its
  output as a write pipe. It knows no application: the application's
  host, its loop over skein-shell's `drive`, routes the terminal's io
  completions to it, gives it each tree the view builds and the lines
  the application prints of its own, and takes each command from it.
- **Its settings,** `jig_shell_text::Settings`, which the application
  gives whole, with no defaults: the name its own lines start with (its
  own, and at most the bytes the view's limits give a node's text: the
  terminal refuses a longer one), the longest line it reads (at least
  the longest words a person may write, with a number and a space
  before them), and its print interval (a guide; `ops`'s is one
  second).

### 15.2 When it prints

- **The whole page,** always: the view's last tree, never a diff.
- **When the tree changed** (the view emitted patches since the last
  print), at most once per print interval. The interval is a guide: a
  change within it is printed when it ends, as the tree is then, never
  lost.
- **At once after the person's own command,** once the steps that took
  it have built the tree; and at once on `p`.
- **One print at a time:** while the output has not taken the last
  print, a due print waits, then shows the tree as it is when it starts.
- **Its own lines and the application's** (a credential shown once, how
  to sign in again) are written before the next print of the page, in
  the order they came, each starting with the shell's name and `: `, so
  they are never taken for the page's.

### 15.3 How it prints

- **The first line** is `== TITLE == LINK`. TITLE is the text of the
  main's first level-1 heading, or the shell's name when there is none.
  LINK is the text of jig's link region (`jig-link`): the link's state in
  jig's words, then `; ` and the page's being behind when it is. Neither
  is printed again.
- **Then every node in preorder,** by its role:

  | Role | Prints |
  |---|---|
  | page | its children |
  | banner, navigation, main, complementary, content info, form, search, region, figure, dialog, disclosure | named: `NAME:` and its children indented; unnamed: its children |
  | heading | `#` once per level, a space, its text |
  | paragraph, text, time | its text; a time's text is the view's words for it |
  | list, tree, menu | named: `NAME:` and its items indented; unnamed: its items |
  | item, tree item | `- ` before its first line, its other lines indented |
  | table | one line per row |
  | row | its cells' texts joined by ` \| ` |
  | cell | its text, as its row joins it; a cell outside a row, its text on a line of its own |
  | term | its text, then `:` |
  | description | its text, indented |
  | button, tab, menu item | `[n] NAME` |
  | link | `[n] TEXT`; a checked external link of Markdown's: `TEXT <URL>`, with no number |
  | text field, text area | `[n] NAME: VALUE`, or `[n] NAME: (empty)` |
  | choice group, tabs, log | `NAME:` and its children indented, a log's entries oldest first |
  | choice | `[n] (*) TEXT` when selected, `[n] ( ) TEXT` otherwise |
  | check | `[n] [x] NAME` when pressed or selected, `[n] [ ] NAME` otherwise |
  | status | `* TEXT`; a status with no text (a slot loading, or one its owner cannot show) prints its name |
  | alert | `! TEXT` |
  | progress | `NAME: TEXT` |
  | code | each line of its text, indented four spaces more |
  | quote | each line of its text after `> ` |
  | separator | `--` |
  | slot | its children: the owner's fragment, or the words saying it is loading or cannot be shown |
  | drawing | `NAME:` and its fallback indented (section 3) |
  | image | its name |

  A disclosure prints unfolded and a menu prints its items: the text
  shell keeps no fold of its own.
- **Indented** means two spaces more than the line it belongs to.
- **States** follow a node's line as words in parentheses, in this
  order: `(disabled)`, `(expanded)`, `(busy)`, `(current)`,
  `(invalid)`, `(pressed)`, `(selected)`, `(provisional)`; a field that
  submits on enter adds `(enter sends)`. A check's and a choice's
  pressed or selected state is its mark instead.
- **Text** is a node's runs, in order: a code run between backticks,
  every other run as it is. Style tokens, region names and shortcuts
  are not printed: nothing a person must know is said by them alone
  (section 13).
- **Nothing from the engine drives the terminal.** A control character
  (U+0000 to U+001F, U+007F to U+009F) and a byte that is not UTF-8
  print as U+FFFD, except a tab, which prints as a space, and a newline,
  which starts a new line at the same indentation.
- **Lines are not wrapped:** the terminal wraps them.

### 15.4 Numbers and commands

- **Numbered nodes:** each node of a role a person acts on (a button, a
  link with an address, a text field, a text area, a choice, a check, a
  tab, a menu item) that is bound and not disabled gets `[n]`, from 1 in
  preorder, at each print; one of those roles disabled or bound to
  nothing prints `[-]` in its place. The shell keeps its last print's
  numbers: each number's node id, which stays the node's while the
  view's diff matches it (section 6).
- **Commands,** one per line, after leading and trailing spaces are
  removed:

  | Line | Does |
  |---|---|
  | `n` | acts on node n: a button, a link, a tab, a menu item or any other bound node is pressed (a link followed); a check or a choice is toggled; a text field or a text area is written empty, then submitted if it submits on enter |
  | `n WORDS` | writes WORDS, the rest of the line after the spaces that follow n, as field n's whole value, then submits it if it submits on enter |
  | `p` | prints the page again, at once |
  | `q` | closes (15.5) |
  | an empty line | nothing |

  A text area takes one line: a line break cannot be typed into it. The
  words are passed as they were typed; their bounds are the domain's,
  which marks a field invalid past them.
- **Refusals,** one line each, NAME being the shell's name:

  | When | The shell prints |
  |---|---|
  | n is not a number its last print showed | `NAME: no [n] on the page`, then the page again |
  | n was shown, but its node is gone or takes no such act now | `NAME: [n] is gone`, then the page again |
  | words for a node that is not a field | `NAME: [n] takes no words` |
  | a number before the first print | `NAME: no page yet` |
  | a line longer than the longest | `NAME: line too long` |
  | anything else | `NAME: commands: n, n words, p, q` |

- **The commands are jig's,** for every application. An application
  adds to its terminal only what it prints of its own (15.2), never a
  command.

### 15.5 Its loop

- **One line, one command.** A line is the bytes before a `\n`, without
  a `\r` just before it. Bytes with no `\n` yet wait for it; a line
  longer than the longest is refused whole, through its `\n`. The input
  is left as it is, never raw: the terminal's own line editing makes
  each line, and a pipe works as well as a terminal.
- **Each iteration of the host,** in order: the client's steps; the view
  built; each whole line read made into one command and done; a print,
  when one is due; the output written. Time is the host's clock, read
  once per iteration.
- **Closing:** `q`, the end of the input, or a termination signal, which
  the host hears. The terminal starts no new print, writes what it has
  begun, and says it is closed; the host then closes its client, without
  signing out. What is pending stays in the small store, sent at the
  next start with its keys (client.md, section 11).
- **Signing in** is the host's: a terminal cannot follow a provider's
  navigation, so pressing a provider on jig's sign-in page prints the
  application's own line saying how to sign in.

### 15.6 Its tests

- **Golden text** fixes the renderer and the command reader: pages kept
  as files beside the tests and changed only by hand, built by jig's
  test view root from fixture states: the inbox with entries of each
  kind, a task page with its conversation, a project's sections, the
  deployment, the pages not about the work, a tree with every role and
  state, and text with control characters and bytes that are not
  UTF-8. Every command and every refusal is tested, with numbers kept
  across a print and renumbered at the next.
- **The terminal** is tested on skein's simulated machine: lines split
  and joined across reads, a line too long, prints paced and one at a
  time, the input's end closing.
- **The client's worlds play the text shell** (testing.md, section 4),
  and the client's protocol world runs one story with the terminal in
  front of its native client: lines in on a pipe, pages out on another
  (testing.md, section 2).

## 16. Later

Each waits for the application that needs it, and adds without changing
what sections 3 to 5 and 15 settle.

- **Drawings acted on:** a drawing bound by its owner, its drawing code
  in a shell reporting `Act::Drawing` with the owner's bytes, and the
  owner's view decoding them (section 5). Every act on a drawing has an
  ordinary node beside it doing the same, which the fake person, a
  screen reader and the text shell act on. It adds no act, role or
  attribute. Waits for the first drawing a person acts on: zooming a
  chart, choosing a point on a map.
- **The browser's shell** (`jig-shell-web`, section 11), on the tree this
  document settles: landmarks to HTML's elements, region names to
  attributes jig's stylesheet places by, shortcuts to the platform's
  keys, menus and disclosures to their elements, pictures to files the
  engine serves from its origin beside the application's drawings
  (`../../protocol/client.md`, section 12). Its plan
  follows the one that builds sections 3.1, 4.1, 5 and 15, so the tree it
  maps does not change under it.
