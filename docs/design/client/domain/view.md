# The view

Provisional, 2026-10-09. `jig-client-view`: the client domain's state as a
tree a shell shows, a person's acts coming back as typed actions, and the
diff between the last tree and the next. It is optional: a service has
none, and a command-line tool may write its own. It says what the tree
is, how pages and cards are built and how an application's slots are
filled, bindings, the diff, words and their languages, Markdown,
conversations, and what a shell does. What is still open is listed in
section 14.

## 1. In one page

- **State in, tree out.** Each page and each card is a function from the
  domain's state to tree nodes, through a builder, one node a call. A
  card has one function, called wherever it shows (objects.md,
  section 2).
- **A semantic tree.** The tree says what each node is (a region, a
  heading, a list, a button, a text field, a log), the way an
  accessibility tree does, never how it looks. Each shell maps it onto
  its platform: HTML with its roles in a browser, a platform's widgets, or
  text.
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
  application's parts; the view root fills each from the part's owner's
  view, and the builder splices them in (section 4).
- **Words from a phrasebook.** Phases, holds, amounts and times are said in
  words from a phrasebook per language: jig ships English, and an
  application may give its own as data (section 7).
- **Sized from the domain's limits,** so a tree always fits, and a diff's
  patches are bounded by the two trees.
- **No framework.** The view is step code under the subset, as the
  domain is; a framework's closures, signals, hooks and scheduler are
  what the subset leaves out (programming-model.md, section 10). The
  domain and its worlds would not change if one rendered the same tree.

## 2. Structure

```
jig-client-view/src/
├── tree.rs         the tree: nodes, ids, keys, roles, states, style tokens, text
├── builder.rs      the builder: nodes, keys, names, bindings, slots; splicing
├── binding.rs      bindings: an owner and a token; the shell's acts; decoding jig's
├── diff.rs         matching, ids, patches
├── render.rs       building when what is shown moved, diffing, keeping the last tree
├── phrases.rs      jig's phrases, their holes, the phrasebook's shape, English
├── words.rs        phases, holds, executors, amounts, spans, times, in phrases
├── markdown.rs     the Markdown subset, read into nodes
├── turns.rs        a conversation's turns and its provisional text
├── frame.rs        the frame
├── confirm.rs      the confirmation
├── notices.rs
├── cards.rs        each card, and their shared parts: head, context, state
├── pages.rs        each page of jig's
├── limits.rs       limits derived from the domain's; the worst case
└── tests.rs
```

The application's view root (`<application>-client-view`) depends on it,
and on its families' views.

## 3. The tree

- **Nodes in preorder,** each with:
  - an id, stable across builds while the diff matches it;
  - a role, from a closed set: page, region, heading (with a level),
    paragraph, text, list, item, table, row, cell, term and
    description, button, link, text field, text area, choice, choice
    group, check, dialog, status, alert, log, tree, tree item, tabs,
    tab, progress, time, code, quote, separator, figure, slot, and
    drawing;
  - a name, where its text does not give it;
  - states: disabled, expanded, busy, current, invalid, pressed,
    selected, provisional, submits on enter;
  - style tokens, from its owner's closed set (section 13);
  - text, on text nodes only, as runs: plain, emphasis, strong, code, a
    link;
  - a value, on fields, with the domain's count of its own writes;
  - an address, on links, which the shell turns into what its platform
    shows;
  - a key, on items of a list, saying which object it is, so lists diff
    by object;
  - a binding, on what a person acts on (section 5).
- **Closed, so a new one breaks the build** wherever roles are matched:
  each shell's mapping, the fake person's queries.
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

  A drawing is shown, not acted on: what a person does about it is an
  ordinary node beside it. Its data is compared whole by the diff, and a
  change is one patch.
- **No layout.** Where things go on a screen is the shell's and its
  styles', from roles, order and tokens. A phone and a terminal lay out
  the same tree differently.

## 4. Building

- **The builder** owns the tree it builds: open, close, text, a key, a
  name, a state, a token, a value, a binding, a slot; one node a call, no
  macros. Building past the tree's capacity is an assertion: the capacity
  comes from the domain's limits, so overflowing it is the view's bug.
- **A card is a function** over the builder, the domain and its object,
  called wherever the card shows; its place changes its line of context
  only.
- **A page is a function** reading the page's state and calling the
  cards' functions for the objects it names.
- **A slot** is a node a jig page or card leaves for a part (root.md,
  section 6), holding the part's reference and nothing else.
- **The view root** builds a client's whole tree:
  1. the frame, from jig's view, with the application's navigation
     entries in their slot;
  2. the page at the address, from its owner's view: jig's, or a
     family's;
  3. each slot's nodes, from the part's owner's view, which the builder
     splices in at the slot;
  4. the confirmation and the notices.

  A family's page may leave slots for jig's objects, which the view root
  fills from jig's cards (root.md, section 7), and those cards leave
  slots for parts in turn. Slots therefore nest two deep, never more: the
  builder splices in two passes, and the tree's capacity counts both.
- **A slot left empty** shows as loading, and one its owner cannot show
  says so; neither breaks the page.

## 5. Bindings

- **A binding is an owner and a token.** jig's view and each family's
  view keep, for the tree they last built, a bounded table of what each
  of their tokens means: go to an address, edit a field, submit, intend
  an act on an object, choose, confirm, dismiss, fold, filter, read more,
  sign in.
- **Decoding:** the shell reports an act on a node by its id (a press, an
  input's whole value, a submit, a toggle); the view root finds the node
  in its last tree, and hands the token to its owner's view, which
  decodes it into its owner's action, wrapped for the domain's root.
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
make, the depth of nesting, of Markdown's nesting, and the bindings each
owner keeps) are derived from the domain's: each page's function has a
companion that counts the most nodes it may build with every window full
and every text at its most bytes, and the largest is the tree's capacity.
A Markdown text builds at most a fixed number of nodes per byte. The
application's view root adds its families' counts to jig's.

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
  - *the browser's* (`jig-client-shell-web`): the client compiled to
    WebAssembly, with only the glue the toolchain generates; the tree
    mapped to HTML elements with their roles; session storage as the
    small store; served by the engine from its one origin under a strict
    content security policy;
  - *the native one* (`jig-client-shell-native`): skein's HTTP client and
    its server-sent events reader, for Linux and for the worlds that make
    bytes real; with no view, it is a service's shell.
- **Drawings are drawn by the application's code in the shell.** A shell
  is ordinary Rust, so it takes, when it is built, the application's
  drawing code for each kind of drawing it shows, which reads the
  drawing's data and draws it on the platform (a canvas or vector
  graphics in a browser). A kind a shell has no code for shows its
  fallback.
- **An application's shell:** a phone's or a desktop's, mapping the same
  tree onto its widgets, is the application's to write over the same
  domain and view.

## 12. Accessibility

- Every node a person acts on has a role and a name; cards are named
  regions; buttons are named by their words; a conversation is a log; a
  plan is a tree whose items say whether they are open; the link's state
  and notices are statuses; a confirmation is a named dialog.
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

## 14. Open questions

- **Drawings a person acts on:** zooming a chart, choosing a point on a
  map; today a drawing is shown, and acts are nodes beside it.
- **A text shell** for terminals, from the same tree, as `ops`'s terminal
  client (`../../examples.md`, section 8) would want.
- **A markup macro,** if plain calls prove unreadable for whole pages:
  one `macro_rules!` expanding to the same calls, the one place a macro
  might live.
- **Right-to-left languages,** and what they ask of the tree beyond the
  phrasebook.
- **Other browsers** than Chromium, which skein's browser kit drives.
