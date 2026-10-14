# The browser's layer

Provisional, 2026-10-14, revised 2026-10-14. jig's browser parts,
`jig-shell-web`, and everything a browser needs that every jig
application reuses: what an application writes, the toolchain and the
bundle, the application's loop over the parts and where the invariants
live, the tree as elements, patches and acts, scenes as SVG, addresses
as paths, transport and the small store in a tab, signing in by cookie,
the engine's serving under its policy, jig's stylesheet, the limits,
the browser tier, and what stays fixed (section 19). The client's
protocol is `client.md`; the tree the parts map is
`../client/domain/view.md`. What is still open is listed in section 17,
and what waits in section 16.

## 1. In one page

- **A toolkit, not a framework:** `jig-shell-web` is jig's browser
  parts, ordinary Rust, each usable alone, with plain-data inputs and
  outputs; it runs no loop and holds no application (5.1).
- **The application owns its start and its loop:** its browser shell,
  `<application>-shell-web`, over jig's parts and its client host, is
  copied from the reference shell, `jig-test-shell-web` (section 3).
- **The parts hold the invariants that protect requests,** whatever the
  loop's order (5.3); the rest of the order is the reference's (5.2).
- **Open by steps,** from a restyle to a part replaced (3.5).
- **The tree as elements** by one default table, overridden for an
  application's own nodes or replaced by its widget (section 6); diffed
  in place, so focus, caret, selection, composition and scroll stay
  (section 7); scenes drawn as SVG, the fallback kept (section 8).
- **Its paths are the application's,** by its own codec; jig's default
  codec gives jig's places theirs; jig keeps `/jig/` alone (section 9).
- **Built by jig's bundler:** no wasm-pack, no npm (section 4).
- **Served by the engine:** what the application's startup gives, behind
  a fixed front door: the `Host` check, the policy's floor, the cookie's
  attributes, the browser header (sections 11, 12 and 12.1).
- **Tested** on the host part by part, and in Chromium through jig's
  harness, its face and jig's stories for a browser shell (section 15).
- **What stays fixed,** with its reason and what to do instead: section
  19. **Not the layer's:** an application's streams (section 16).

## 2. Where it sits

```
a person, a screen reader
  ▲ elements, focus, keys, the address bar, back and forward
the browser: fetch, session storage, history, clock, date, entropy
  ▲ the glue: wasm-bindgen, js-sys, web-sys
jig-shell-web            jig's browser parts, each usable alone (5.1)
  ▲ plain data: reports, patches, the link's requests, paths, the look
<application>-shell-web  its start and its loop: the turn (5.2)
  ▲ its client host's functions, called directly
the client host          client root, view root, jig-protocol-client-link
  ▲ the wire's bytes: exchanges, one stream
════════════════════════════ one origin ════════════════════════════
jig-protocol-engine-clients   the front door, what it is given, the API
```

- **A person** reads the page by sight or with a screen reader, and acts
  on it with a pointer and keys. **The browser** is any current one,
  which the parts target; the browser tier drives Chromium (section 15).
- **`jig-shell-web`,** jig's crate, ordinary Rust: the platform's parts
  (`../client/domain/view.md`, section 11), knowing no application.
  **The application's browser shell,** compiled to the module, is its
  start and its loop (3.1, section 5), with no trait between it and the
  parts or its client host (3.2).
- **`jig-protocol-engine-clients`,** the engine's side of the client's
  protocol (`client.md`, section 2), serves what the startup gives it
  behind a fixed front door, and signs in by cookie (sections 11 and 12).

## 3. What an application writes

Its extension points are the tree's: its own pages and arrangements, its
style tokens and stylesheet, its region names, its pictures, its drawings
with their drawing code (scenes), drawings acted on, its shortcuts, its
phrasebooks. In a browser it writes its browser shell, copied from the
reference, and its stylesheet; what more it may change is 3.5. Its
binary gives the engine what it serves, as `jig-test` does (section 12).

### 3.1 Its browser shell

- **`<application>-shell-web`,** its own crate, ordinary Rust, as shells
  are. Its start, through `wasm-bindgen`'s start attribute, which
  `jig-shell-web` re-exports so its version is jig's, makes the parts
  with the look (3.3) and starts its client host (section 5); its loop
  is the turn (5.2). It orders the platform's work and decides nothing:
  what to show, send and keep is its client host's; carrying a request
  safely is a part's (5.3).
- **The reference shell,** `jig-test-shell-web`, written to be copied
  and commented as such, as the client root, view root, binary and
  service are (`../client/domain/root.md`, section 12; `service.md`,
  section 7): about 200 lines beside its client host (a guide).

### 3.2 The client host

- **The application's client root, view root,
  `jig-protocol-client-link` and the routing between them,** one step
  machine. It is the application's, since its routing matches the
  application's types: its root's events and requests, its families'
  records in the link's.
- **Shared by every shell** the application uses (its terminal client,
  `../client/domain/view.md`, section 15, runs the same host), and
  **copied from jig's test application's,** as a client root is copied
  from its reference (`../client/domain/root.md`, section 12).
- **Its functions,** which the browser shell calls (names the plan's):
  - *`start`:* the seed, the path and query the page opened at, the
    small store's records, the person's offset and languages;
  - *`step`:* one event (a link event, the small store written or its
    write failed, a place gone to as a path, an act on a node, a
    deadline, signed out, a sign-out lost), giving its duties in order:
    the small store's writes and removals; exchanges sent and abandoned;
    the stream opened and closed; an address shown, new or in place; a
    sign-in with a provider coming back to a path; a sign-out; a
    service's credential shown once;
  - *`build`:* patches, when what is shown moved
    (`../client/domain/view.md`, section 6);
  - the last tree; the path a link node leads to, without acting; the
    next deadline; whether closing the tab would lose requests; the
    language its phrasebook chose.

### 3.3 The look

What the application's browser shell gives jig's page part when it makes
it, as plain data, with no trait: **the names of its style tokens and
region names,** each starting with its name and a hyphen (section 13);
**its pictures' paths,** each of its owners' pictures a file in the
bundle's `pictures/<name>/`, or none (4.2); **its drawing code,** its
view root's `draw`, a function routing a drawing by its kind's owner
(section 8; `../client/domain/view.md`, section 17); and **its elements
and widgets,** if any (6.1). **Its name** is in the title the shell sets
(section 7) and in the bundle's file names (4.2).

### 3.4 Its stylesheet, pictures and phrasebooks

- **Its stylesheet,** CSS, and **its pictures' directory** go to the
  bundler with its module (section 4); its stylesheet loads after jig's
  (section 13), or where its own page puts it (3.5, step 4).
- **Its phrasebooks are in its module,** data its view root takes
  (`../client/domain/view.md`, section 7).

### 3.5 What it may change

Steps from the standard, each needing one before it only where it says.

1. **Copy the reference shell** and change its names: the standard, the
   same for every application.
2. **Restyle:** its stylesheet after jig's, which may restyle jig's
   tokens and regions (section 13); its pictures (4.2).
3. **Its paths:** jig's default codec as is, under a prefix of its own,
   renamed segment by segment, or its own codec (section 9).
4. **Its page:** its own `index.html` (its head: title, icons, fonts,
   metadata, preloads; its mount element; its stylesheets in its order),
   copied unparsed (4.2). It loads its module from the origin by a
   script file, with nothing inline: the policy refuses at run what
   breaks that (12.1), and the browser tier fails on any violation.
5. **Its files:** any of a type the engine admits, under
   `/jig/client/<name>/`; and root files, paths outside `/jig/` named at
   start (`/favicon.ico`, `/robots.txt`) to answer them (section 12).
6. **Its elements and widgets** (6.1): another element for a role
   carrying one of its style tokens; a widget for such a node or a
   drawing of its kinds: a code editor for a text area, a map library.
7. **Its JavaScript,** from the origin only: `wasm-bindgen`'s snippets,
   its own modules the glue imports (4.2), or files its page loads
   (step 4); never from another origin, never inline (section 19).
8. **Policy additions** (12.1): named origins for data and other sites'
   pages, and the origin's own media and manifest, above the floor.
9. **A part replaced or left out:** any but the engine's front door
   (section 12). One that holds an invariant (the transport, the inbox,
   the page) moves it to the application (5.3), and jig's stories for a
   browser shell check it (section 15). There is no shell to write
   whole: the application's browser shell is its own.

## 4. The toolchain and the bundle

### 4.1 The toolchain

- **`wasm32-unknown-unknown`, stable Rust,** and the glue crates,
  `wasm-bindgen`, `js-sys` and `web-sys` (only the features the parts
  use), pinned to exact versions in jig's workspace: the only `unsafe`
  the module runs, with the code their macros generate
  (`programming-model.md`, 2.1). `jig-shell-web` forbids `unsafe`; an
  application's browser shell writes none of its own, and holds only
  what the glue's macros generate for its start and for bindings to its
  own JavaScript (3.5, step 7).
- **The glue** is generated by `wasm-bindgen`'s own library
  (`wasm-bindgen-cli-support`), called from the bundler, so the
  generator's version is locked with the macros' in the same
  `Cargo.lock`; `--target web`, an ES module. No other build tool: no
  wasm-pack, no npm, no binaryen.
- **The release module** is built with a profile that optimizes for
  size. jig's is `web`: it inherits `release`, with `opt-level = "s"`,
  `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`,
  `overflow-checks = true` and `strip = true`. An application's
  workspace defines its own the same way.
- **The browser tier's module** is built with the dev profile, as code
  under test; the bundler is a tool, built optimized
  (`../../development/workflow.md`, section 2).
- **The parts' two halves:** each part's pure half builds for the host
  too (section 15); its platform half, like an application's browser
  shell, for wasm32 only.

### 4.2 The bundler and the bundle

- **`jig-bundle`,** jig's tool crate, ordinary Rust, a library with a
  small binary, makes the bundle from the module (a `.wasm` file cargo
  built), the application's name, its stylesheet and its pictures'
  directory, and, where it has them, its page and its files (3.5, steps
  4 and 5). The bundle is a directory, written whole:
  - `index.html`, the page, the bundler's or the application's, and
    `boot.js`, the starter (4.3);
  - `<name>.js`, `<name>_bg.wasm` and `snippets/`: the glue, the module
    and the modules the glue imports (3.5, step 7);
  - `jig.css`, jig's stylesheet, which ships with `jig-shell-web`, and
    `<name>.css`, the application's (section 13);
  - `pictures/jig/`, jig's pictures, and `pictures/<name>/`, the
    application's, so the two never clash; `<name>/`, its files.
- **Only what the engine serves:** a file whose type is not in the
  engine's closed list (section 12) refuses the bundle, naming it.
- **jig's texts that are not Rust:** the page and the starter, which the
  bundler writes, and jig's stylesheet. Nothing inline.
- **Its size is a guide:** jig's test application's module, built with
  `web`, at most 4 MiB before compression, measured when first built and
  kept with jig's budgets (`../../development/workflow.md`). Past it,
  what grew is reported.

### 4.3 The page and the starter

- **The bundler's page,** `index.html`, answered at every path outside
  `/jig/` but the root files, and at the link's route (section 12):
  `<meta charset="utf-8">`, a viewport, a title of the application's
  name, jig's stylesheet then the application's, the starter as
  `<script type="module">` by `src`, an empty mount element the
  reference shell finds by its id, and a `<noscript>` saying the client
  needs WebAssembly and scripts.
- **Its references are paths from the origin's root,** under
  `/jig/client/`, since the page is answered at every address.
- **The starter,** `boot.js`: two statements,
  `import init from "./<name>.js"; init();`. The glue finds the module
  beside it.

## 5. The loop, the application's

The application's browser shell runs it over jig's parts (5.1), in the
reference's order (5.2); the parts hold what protects requests (5.3).

- **Starting,** the reference's: the starter loads the glue, the glue
  the module, whose start makes the parts and reads through them, once,
  the clock, the date and what its client host's `start` takes (3.2).
  At the link's route it signs in first (section 11); then it starts its
  client host, builds, and applies the first patches, inserting the
  whole tree under the mount element.
- **A panic is fail-stop:** the restart part reloads the page, which
  starts the client again from its small store, at most three times in
  a minute (jig's default, the application's parameter), counted by the
  date under a session storage key of its own, outside the small store's
  prefix. Past it, the page says the client failed, starts nothing, and
  keeps the small store; a reload past the minute starts it again.
- **The parts' own words** (a credential shown once, the client failed)
  are jig's phrases (`../client/domain/view.md`, section 7), which the
  browser shell gives them.

### 5.1 The parts

Each is ordinary Rust, usable alone, taking and giving plain data; the
transport is built over fetch, the small store and the hidden tab, the
page over scenes as SVG, each usable without it. Their module names are
the plan's.

| Part | Takes | Gives |
|---|---|---|
| the inbox | the application's turn function, at start | the reports, in order; it calls the turn when one is waiting and no turn runs |
| the page | a mount element, the names, pictures, `draw`, the mapping (jig's table, the application's elements and widgets); patches and the tree | elements kept in place; acts on node ids, reported to the inbox; a drawing's box resized, reported; the link nodes it holds |
| scenes as SVG | a scene, the names | SVG elements: the page's default for a drawing |
| fetch | exchanges (a path, bytes), abandons; the stream opened and closed | link events: answered, failed, the stream opened, a piece, its end; one read outstanding, the next asked by the loop |
| the small store | records by key, a prefix (`jig.` by default) | the records read at start; a write done or failed, synchronously |
| the transport | the link's requests of a step, whole, and a navigation away | the store's writes first, their outcome reported; then exchanges and the stream through fetch (an opening while hidden, through the hidden tab), or every exchange of the step failed when a write failed; the navigation only after the step's writes, none when one failed |
| history | a path to show, as a new entry or in place | the path and query the page opened at; a place gone to by back or forward, as a path; the scroll kept per entry, given back after patches |
| the timer | a deadline | one timer, reported due |
| the platform's facts | nothing | the monotonic clock, wall time, a seed from the cryptographic generator, the person's offset and languages |
| title and closing | a title, the language; whether closing loses requests | the document's title and `lang`; `beforeunload` registered only while it would |
| restart | the words for "the client failed"; its limit, three reloads in a minute by default | the panic hook: a reload within the limit, counted under its own session storage key; past it, the page says so and starts nothing |
| the hidden tab | its delay, a minute by default; the tab hidden or shown; an opening of the stream while hidden | a deadline, past which fetch closes the stream; the opening held until shown, then made through fetch |
| sign-in | the page's path and query at start; a provider and a path; signing out; a credential; its dialog's words | the link's sign-in exchange, the address replaced first; a provider's navigation, for the transport to make; the sign-out exchange; the credential shown once in a dialog of its own, never kept |

- **A report** is one platform happening, as data: a link event; the
  small store written or its write failed; an act on a node or a
  drawing's box resized; a place gone to; the timer due; the tab hidden
  or shown; the end of a sign-in's or a sign-out's exchange. A part that
  reports is given the inbox.

### 5.2 The turn

The reference's; an application's loop may order it otherwise within 5.3.

```
the turn, which the inbox calls when a report waits and no turn runs:
  now, wall ← the platform's facts                 once per turn
  while the inbox holds a report:                  raised meanwhile too
    a drawing resized: to the page part, no step
    the tab hidden or shown: to the hidden tab part, no step
    the timer due: to the hidden tab part, then a step as any report
    step ← the client host steps with the report, now and wall
    away ← sign-in: a provider's navigation, if the step signs in by one
    the transport carries the step's link requests, then away  (5.3)
    fetch reads the stream on, after a piece of it
    history shows the step's address, printed by the client host
    sign-in signs out, or shows a credential once, if the step asks
  patches ← the client host builds
  the page applies them                            an unknown node: a panic
  a new entry: its top, its heading focused; back or forward: its scroll
  the page sets each link's href to the path the client host prints
  title and closing: heading · name, the language, the closing warning
  the timer: the earliest of the client host's and hidden tab's deadlines
```

### 5.3 Where the invariants live

The parts hold the invariants that protect requests, so no order of the
loop breaks them; the reference documents the rest.

- **The small store written before a step's exchanges leave, and a
  navigation away only after:** the transport's, as the native shell
  holds its exchanges until the small store's write completes, failing
  them if it failed (`client.md`, section 10). An application driving
  fetch and the small store without it takes the rule on itself, which
  jig's stories for a browser shell check (section 15).
- **No re-entry:** the inbox never calls the turn while one runs; a
  report raised during a turn (a write's outcome, a focus moved, an
  element removed) is taken by the same turn, in order. The reference
  keeps its state in one cell, whose second borrow panics, fail-stop.
- **Patches applied to the tree they are diffed from:** the page part
  refuses, as a panic, fail-stop, a patch naming a node it does not
  hold, so a build whose patches are dropped fails at the next apply.
- **The rest of the order** (links' `href`s, title, timer and closing
  warning after the build) is the reference's (5.2): out of order, a
  turn shows something stale until the next, a closing warning among
  it; no request leaves unkept or twice.
- **Why not types.** A typestate that made the wrong order unwritable
  would make jig's invariants the shape of every application's loop, a
  framework by its types, and would not stop an application calling the
  platform itself; a part that holds its invariant asks nothing of the
  loop but to be used. Documented order alone would leave an
  application's code ordering what jig promises, against
  `../README.md`, section 6.

## 6. The tree as elements

Each node is one element, by its role, through the page part's default
mapping, this table; the browser's face finds the person's roles through
the same table (section 15). An application overrides it for its own
nodes only (6.1).

| Role | Element (ARIA role where the element has none) |
|---|---|
| page | the mount element the page part is given (4.3) |
| banner, navigation, main, complementary, content info | `<header>`, `<nav>`, `<main>`, `<aside>`, `<footer>` |
| form, search | `<form>` (named), `<search>` |
| region | `<section>`, named |
| heading | `<h1>` to `<h6>`; deeper, `role="heading"` with `aria-level` |
| paragraph, text | `<p>`, `<span>` |
| list, item | `<ul>`, `<li>` |
| table, row, cell, column header, row header | `<table>`, `<tr>`, `<td>`, `<th scope="col">`, `<th scope="row">` |
| term, description | `role="term"`, `role="definition"` |
| button | `<button type="button">` |
| link | `<a href="<path>">`; a checked external Markdown link, `<a href="<url>" rel="noopener noreferrer" target="_blank">` |
| text field, text area | `<input type="text">`, `<textarea>` |
| choice, choice group | `<input type="radio">` with its label, `role="radiogroup"` |
| check | `<input type="checkbox">` with its label |
| menu, menu item | `role="menu"` behind its button, `role="menuitem"` |
| disclosure | `<details>` with `<summary>` |
| dialog | `<dialog>`, shown modal |
| status, alert, log | `role="status"`, `role="alert"`, `role="log"` |
| tree, tree item | `role="tree"`, `role="treeitem"` |
| tabs, tab | `role="tablist"`, `<a role="tab">` |
| progress | `role="progressbar"` with its text as `aria-valuetext` |
| time | `<time>` |
| code, quote, separator | `<pre><code>`, `<blockquote>`, `<hr>` |
| image | `<img src="/jig/client/pictures/<owner>/<file>" alt="<name>">`, the owner `jig` or the application's name (4.2); a picture the look lacks: its name as text |
| figure | `<figure>` |
| slot | `<div>`, its children the owner's nodes |
| drawing | `<figure>` named by its name: the scene's `<svg aria-hidden="true">`, then the fallback's elements, hidden from sight but not from assistive technology while the scene is drawn, shown when it is not (section 8) |

- **An ARIA role alone** in the table is a `<div>` with that role.
- **Attributes:**
  - *a name* is `aria-label` where the element's own text does not give
    it; a button, a tab and a menu item show their name as their text;
  - *states:* disabled is `disabled` where the element takes it, else
    `aria-disabled`; expanded `aria-expanded`; busy `aria-busy`; current
    `aria-current`, `page` on a link; invalid `aria-invalid`; pressed
    `aria-pressed`, a check's `checked`; selected `aria-selected`, a
    choice's `checked`; provisional `data-provisional`; submits on enter
    `enterkeyhint="send"`;
  - *a region name* is `data-region="<its name>"`; *style tokens* are
    classes, by their names (section 13); *a shortcut* is
    `aria-keyshortcuts`.
- **Text runs:** plain as text, emphasis `<em>`, strong `<strong>`, code
  `<code>`, a link run `<a>`.
- **Text is text:** every name, run, value and mark's text is written as
  text, never read as markup (`../client/domain/view.md`, section 8);
  bytes that are not UTF-8 show as U+FFFD.
- **A node's id is kept on its element,** as `data-node`. The parts' own
  elements (the mount element, a scene's SVG, a menu's button, the
  sign-in part's dialog for a credential) carry none.
- **One listener per kind of event,** on the mount element, finds the
  node of the element acted on, by its nearest `data-node`.

### 6.1 An application's elements and widgets

- **Another element,** for a role carrying one of the application's
  style tokens (an `<article>` for a region carrying `ops-incident`),
  keeps the node's accessible role and name: the page part adds ARIA's
  role where the element's own differs. States, `data-node`, listeners
  and patches stay the page part's; jig's nodes keep the table.
- **A widget,** for a node of such a role and token, or a drawing of one
  of its kinds, is the application's ordinary Rust, given a host element
  the page part keeps in the node's place, with its `data-node`; the
  node, on insert and each patch to it; and the inbox. It builds what it
  likes inside, reports `Act`s on its node (an `Input` with a field's
  whole value, a `Drawing` with its owner's bytes, ...), and is removed
  with its node. A drawing keeps its fallback beside the host element,
  hidden from sight only, as a drawn scene's (section 8).
- **Inside the host element is the widget's:** the page part's listeners
  leave its events to it; while the focus is inside, a shortcut with no
  modifier is not taken, as in a field.
- **What a widget builds is the application's to keep accessible and
  safe:** the node's role and name (a code editor is a text area to a
  screen reader), checked by its tier; never a node's text read as
  markup (`../client/domain/view.md`, section 8; the policy, 12.1); its
  styles from the origin's stylesheets, since the policy refuses a
  `<style>` or `style` attribute it injects (12.1).

## 7. Patches and acts

- **Patches applied in place** by the page part
  (`../client/domain/view.md`, section 6):
  - *an insert* creates a subtree's elements under its parent, before
    its sibling or at the end; each link gets its `href` in the turn
    (section 9), each drawing its scene (section 8) or widget (6.1);
  - *a remove* removes the element and its subtree;
  - *a move* moves them, and gives back the focus and the selection of a
    field it moved;
  - *a text* replaces a node's runs;
  - *an attribute* sets or removes what changed: a drawing whose data
    changed is drawn again, and a role whose element differs makes the
    new element, the children moved into it;
  - *a value* writes a field's value, which comes only from a domain
    write.
- **Never replaced while it can be updated:** focus, caret, selection,
  composition and scroll stay. A removal that takes the focus moves it
  to the nearest remaining ancestor that can take it, else the main.
- **Acts,** each reported to the inbox with its node's id
  (`../client/domain/view.md`, section 5), none on a disabled node:
  - a press on a button, a tab, a menu item or any other bound node
    but a drawing: `Press`;
  - a plain press on a link: `Press`, and the page does not navigate. A
    press with a modifier, or the middle button, on a link is the
    platform's: it opens the link's path in a new tab, a new client; so
    is any press on an external link;
  - an input's whole value, on each `input` event outside a composition,
    and once at its end: `Input`;
  - Enter, outside a composition, in a field that submits on enter:
    `Submit`, its whole value sent first as `Input`; Shift+Enter in such
    a text area inserts a line break where the domain allows it, and
    does not submit;
  - a check's or a choice's change: `Toggle`; the page part then puts
    the element's checked state back to the tree's, which the next patch
    moves if the domain followed;
  - a press in a drawing's scene: `Drawing` (section 8).
- **Shortcuts:** the platform's keys turned into a press of the first
  bound, enabled node in preorder naming the key; a shortcut with no
  modifier never while a field has the focus
  (`../client/domain/view.md`, section 5).
- **Menus and disclosures are the page part's to fold,** with no step. A
  disclosure is `<details>` and `<summary>`. A menu is a button of the
  page part's, named by the menu's name, that opens its items with the
  platform's keys (arrows, Escape): ARIA's menu-button pattern. A press
  on an item closes the menu.
- **A dialog** is `<dialog>`, shown modal while it is in the tree. Escape
  does nothing of its own: the dialog's buttons decide.
- **A log** keeps its end in view while the person has not scrolled away
  from it (`../client/domain/view.md`, section 9).
- **Going to another page** (a new entry a step shows): after the turn's
  patches, the reference starts the page at its top, the focus on its
  main's first level-1 heading, else the main; back and forward restore
  their scroll, history's (section 9).
- **The title,** as the reference sets it: the main's first level-1
  heading's text, then ` · ` and the application's name; the name alone
  when there is none. `<html lang>` is the phrasebook's language.

## 8. Scenes as SVG

A scene is drawing code's output: a bounded list of marks in preorder, in
whole pixels from its box's top left (`../client/domain/view.md`,
section 17). Scenes as SVG is a part, the page part's default for a
drawing; an application's kind may be a widget instead (6.1).

- **The page part calls the drawing code** (3.3) with the drawing's
  kind, data and box's size in whole CSS pixels, as it measured it;
  false, for a kind or data it does not draw, leaves the fallback shown.
- **Drawn as inline SVG:** an `<svg aria-hidden="true">` of the box's
  size, first in the drawing's `<figure>` and at its top left, so a
  point in the scene is the same point in the figure's box; one element
  per mark:

  | Mark | SVG |
  |---|---|
  | group | `<g>`, translated by its offset, its marks inside |
  | line, polyline | `<line>`, `<polyline>` |
  | path | `<path>`: its move, line, quadratic, cubic and close commands |
  | rectangle, circle | `<rect>` with its corner radius, `<circle>` |
  | text | `<text>`, anchored at its start, middle or end |

- **A mark's tokens are classes,** by their names; its hint, never
  needed to decide, is a `<title>` in it, shown on hover. Colours, widths
  and fonts are the stylesheet's (section 13).
- **Refused, never cut:** a scene past the view's limits (marks and bytes
  per scene, a node's tokens per mark: `../client/domain/view.md`,
  section 10) draws as its fallback.
- **Drawn again** when the drawing's data changes (one patch), or when
  its box's size changes, which is the page part's alone: reported to
  the inbox, it calls the drawing code again, with no step.
- **The fallback** is the drawing node's children, ordinary elements.
  While the scene is drawn, the page part marks the figure drawn, and
  jig's stylesheet hides them from sight, not from assistive technology,
  and they take no space. The text shell and the fake person read the
  fallback, unchanged.
- **Acted on:** a press in the scene reports `Act::Drawing { data }` on
  the drawing's node, its data the act the press takes, in the order of
  `../client/domain/view.md`, 17.3: the topmost mark's under the point,
  else its nearest group's, else the scene's with the point appended,
  else none. A mark that carries an act gets jig's class for acts, so it
  takes presses over its whole area, filled or not (section 13). The
  view drops data past its bound on a drawing's bytes, and the owner's
  view decodes the rest (`../client/domain/view.md`, section 5). It is
  by pointer only: the ordinary node beside the drawing, doing the same,
  is the keyboard's and a screen reader's.

## 9. Addresses and history

- **The application's paths:** its client host parses and prints every
  path the page shows, with its own codec, below the domain: total,
  bounded step code, since a URL is a person's input.
- **jig's default codec,** `jig-protocol-client-link`'s `address`
  module, gives jig's places their paths, at the root:

  | Address | Path |
  |---|---|
  | the inbox | `/inbox`; filtered by a project, `/inbox/<project>` |
  | conversations | `/conversations`; by a project, `/conversations/<project>` |
  | a task's overview | `/tasks/<task>` |
  | a task's other section | `/tasks/<task>/<section>`: `activity`, `plan`, `history`, `budget`, `panels` |
  | a project's overview | `/projects/<project>` |
  | a project's other section | `/projects/<project>/<section>`: `goals`, `tasks`, `notes`, `settings`, `resources`, `policy`, `roles`, `spend`, `recurring` |
  | the deployment | `/deployment` |

  An application uses it as is, mounts it under a prefix of its own
  (`/work/inbox`), renames it segment by segment, or replaces it. The
  table changes only by a change to this design; an application keeps
  its bookmarks by keeping its codec, which says which place `/` is:
  jig's, at the root, the home (`../client/domain/pages.md`, section 2).
- **In jig's default codec,** numbers are decimal, with no leading zero,
  within their type; one trailing slash and a query (an application's
  codec may read one) are ignored; any other path is a place with no
  page, shown as missing (`../client/domain/pages.md`, section 8). The
  fragment is not reported.
- **jig reserves one first segment, `/jig/`:** the API under `/jig/v1/`,
  the bundle under `/jig/client/`. Fixed; no base path (section 19).
- **A link's `href`:** a link node's binding resolves, without acting, to
  the address it leads to (the view root asks the binding's owner's
  view), which the client host prints as a path. After each apply, the
  reference prints the path of every link node the page part holds, and
  the page part sets each `href` whose path changed, since a changed
  binding makes no patch (`../client/domain/view.md`, section 6).
- **History:** an address a step shows is a new entry, or replaces the
  current one, as the step says. The history part keeps the scroll in
  each entry's state as the page leaves it. Back and forward are a place
  gone to, as a path: one step, then, after its patches, the scroll the
  entry kept. A path opened directly (a bookmark, an address typed, a
  link opened in a new tab) starts a client at it.

## 10. Transport and the small store

- **Exchanges** by fetch: a POST of the wire's bytes to the path the
  link gives, `Content-Type: application/octet-stream`, the browser
  header (section 11), same-origin credentials; abandoned by aborting. A
  body is read up to the link's largest record; past it, the exchange
  failed.
- **The stream** by fetch, with same-origin credentials, its body read a
  piece at a time: each piece, cut to the link's stream piece, handed to
  the link as it comes, the next read asked by the loop after the step
  (5.2), so the platform paces it (`client.md`, section 7).
- **The transport** takes a step's link requests whole: the small
  store's writes first, then the exchanges and the stream, then the
  navigation away, if any. When a write fails, every exchange of the
  step is reported failed, none is sent, and there is no navigation
  (`client.md`, section 10).
- **A hidden tab lets its stream go** a minute after it is hidden (a
  default, the application's parameter): at the deadline, fetch closes
  the stream and reports its end, as any drop. An opening while hidden
  waits in the hidden tab part until the tab is shown. A browser allows
  about six connections to one origin over HTTP/1.1, each tab's stream
  holding one, so hidden tabs must not hold theirs.
- **The small store** is the tab's session storage
  (`../client/domain/client.md`, section 11): each record under its key,
  both as base64 text, the key under the part's prefix, `jig.` by
  default; read whole at start. A write the platform refuses, its quota
  full, is a failed write.
- **Closing a tab with requests pending** asks the platform to warn
  (`beforeunload`), only while the client host says it would lose them.

## 11. Signing in and out

- **The sign-in cookie:** `jig-sign-in=<the secret>; Path=/jig/v1/;
  HttpOnly; Secure; SameSite=Strict; Max-Age=<the sign-in's remaining
  life>`. A browser treats a loopback origin as secure, so `Secure`
  holds in plaintext there. Every exchange and the stream carry it; the
  module and the domain never see it (`client.md`, section 8).
- **The browser header,** `Jig-Client: browser`, on every exchange of a
  browser: a POST the cookie authenticates without it is answered `403`,
  with nothing done. A page of another origin can neither set it (it
  would need a preflight the engine never answers) nor send the cookie
  (same-site strict).
- **The printed link, opened in a browser:**
  - a GET of the link's route, `/jig/v1/sign-in/link`, answers the page
    and spends nothing;
  - the sign-in part reads the secret from the address's `secret` and
    at once replaces the address with `/`, so the secret leaves the
    address bar and the tab's history (the browser's record of visited
    addresses keeps it, spent once signed in); it then makes the link's
    sign-in exchange, the POST the native shell makes, with the secret
    and the browser header; the engine answers it signed in, the secret
    in the cookie, none in the body;
  - refused (a spent or expired secret), the engine prints a fresh link,
    as for any refused link, and the browser shell starts its client
    host at `/`, which shows the sign-in page. Not ready or lost, the
    same; a link not spent still works opened again.
- **A provider:** a sign-in with a provider, coming back to a path, is
  the sign-in part's navigation to
  `/jig/v1/sign-in/provider/<provider>?back=<path>`, which the transport
  makes after the step's writes (5.3):
  - for the deployment's link, the engine prints a fresh link and
    redirects (`303`) to `back`, whose sign-in page says to open the
    link the engine printed;
  - for an application's provider, the engine routes the GET, and its
    redirect back, to the provider's component (`client.md`, section 8),
    and ends with the cookie set and a `303` to `back`;
  - `back` must be a path of the application's: one leading `/`, then
    printable ASCII without `\`, no empty, `.` or `..` segment (encoded
    or not), its first segment not `jig`; else `/`.
- **Signing out** is the sign-in part's exchange to `/jig/v1/sign-out`,
  which clears the cookie (`Max-Age=0`). Answered signed out, the
  browser shell steps its client host with signed out; any other end,
  the sign-out lost (`../client/domain/client.md`, section 12).
- **A service's credential, shown once,** in the sign-in part's dialog
  of its own, outside the view's tree: selected text, and a button
  closing it, in jig's words. The part never keeps it.

## 12. Served by the engine

`jig-protocol-engine-clients` serves the browser's client from the
engine's one origin (`client.md`, section 4). What it serves is data;
how it answers is fixed.

- **What it serves is data** the application's startup gives the
  component at start, with no traits: the bundle's files, the page, the
  root files and the policy's additions (12.1), counted in its worst
  case; or none, which serves no browser. `jig-test` reads the bundle's
  directory its configuration names (`configuration.md`, section 3,
  clients; skein's `shell.md`, 6.1); an application may also build or
  embed them.
- **The bundle's files** under `/jig/client/<file>`, by their paths in
  the bundle, from memory. Each file's type comes from a closed list, by
  its extension: `.html`, `.js`, `.wasm` as `application/wasm`, `.css`,
  `.svg`, `.png`, `.webp`, `.ico`, `.woff2`, `.txt` as `text/plain`,
  `.webmanifest` as `application/manifest+json`. Any other file given
  refuses the start.
- **A root file,** a path outside `/jig/` the startup names, answers one
  of the bundle's files (3.5, step 5); **the page,** every GET of any
  other path outside `/jig/`, and of the link's route. Under `/jig/`, a
  GET of anything but a file, the stream, the link's route, or a
  provider's sign-in route and its redirect back (section 11) is not
  found.
- **Caching:** each file and the page with an `ETag`, a digest of its
  bytes computed at start, and `Cache-Control: no-cache`, a matching
  `If-None-Match` answered `304`; the API's answers with
  `Cache-Control: no-store`.
- **The front door, fixed** (section 19), on every request and answer of
  the clients' listener, whatever is served: the `Host` check; the
  policy and the security headers (12.1); the cookie's attributes, and
  the browser header required with the cookie (section 11); no
  preflight, an `OPTIONS` not found; nothing under `/jig/` but jig's
  routes and the bundle. No value the startup gives removes them.
- **Only its own `Host`,** any other answered `421` with nothing done:
  on a loopback listener, the address its printed link names; between
  hosts, with TLS, the name of the listener's certificate
  (`configuration.md`, section 3, clients).

### 12.1 The policy

- **Its floor,** fixed (section 19): `Content-Security-Policy:
  default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src
  'self'; img-src 'self'; font-src 'self'; connect-src 'self'; base-uri
  'none'; form-action 'none'; frame-ancestors 'none'`, and
  `X-Content-Type-Options: nosniff`, `Cross-Origin-Opener-Policy:
  same-origin`, `Cross-Origin-Resource-Policy: same-origin`. `script-src`
  and `style-src` take nothing more: no other source, no
  `'unsafe-inline'`, no `'unsafe-eval'`, no nonce or hash. `object-src`
  is never added.
- **An application's additions,** each a named `https` origin, to
  `img-src`, `font-src`, `media-src`, `connect-src` and `frame-src`:
  data and other sites' pages, never code run by the page; `'self'` to
  `media-src` and `manifest-src`; `blob:` to `img-src` and `media-src`.
  Never a wildcard, a scheme alone (`https:`, `data:`), or `'unsafe-*'`:
  the component refuses such a start, naming one.
- **Workers from the origin** are scripts from the origin, which
  `script-src 'self'` admits with no addition; the origin frames none of
  its own pages, every answer carrying `frame-ancestors 'none'`.
- **`Referrer-Policy`:** `no-referrer` by default; an application may
  set `strict-origin`, never one that sends a path to another origin.
  Its own page (3.5, step 4) and code set no wider one, in a `referrer`
  meta or attribute, which the header cannot overrule.

## 13. Stylesheets

- **Every style token and region name has a name.** jig's start with
  `jig-`: its tokens `jig-card`, `jig-chip`, `jig-muted`, `jig-warning`,
  `jig-danger`, `jig-success`, `jig-button-weight`, `jig-dense-list`;
  its regions as `../client/domain/pages.md`, 12.1 and 12.2, list them.
  An application's start with its name and a hyphen, given by its view
  root, which its shell gives the parts as the look (3.3).
- **jig's stylesheet,** `jig.css`, ships with `jig-shell-web`. It places
  jig's landmarks and regions by `data-region`: the banner first; the
  navigation beside the main on a wide screen and above it on a narrow
  one; asides beside the main; the composer pinned below its
  conversation. It styles jig's tokens; gives every mark of a scene a
  default (strokes in the text's colour, no fill), and its class for
  acts `pointer-events: all`; places a scene's SVG at its figure's top
  left, and hides a drawn drawing's fallback from sight, taking no
  space (section 8); keeps focus outlines visible; follows the
  platform's light or dark preference and reduced motion.
- **jig's palette:** custom properties an application's stylesheet uses:
  `--jig-text`, `--jig-background`, `--jig-muted`, `--jig-accent`,
  `--jig-warning`, `--jig-danger`, `--jig-success`, and six series
  colours for drawings.
- **The application's stylesheet** loads after jig's, styles its own
  tokens and regions, and may restyle jig's (its own page: 3.5, step 4).
- **No style is a fact** (`../client/domain/view.md`, section 13).

## 14. Limits

- **The link's:** an exchange's body, at most its largest record; a piece
  of the stream; exchanges in flight (`client.md`, section 13).
- **The view's:** a tree's nodes, a diff's patches, a scene's marks and
  bytes, a drawing's bytes (`../client/domain/view.md`, section 10); a
  drawing's SVG has at most a scene's marks.
- **The platform's:** session storage's quota, past which a write fails.
- **The parts' own:** one timer; one read of the stream outstanding; the
  inbox's reports raised during a turn, only those the turn's own calls
  raise; three reloads in a minute, by default.
- **The bundle's size,** a guide (4.2).

## 15. Testing

Each within jig's suites' budgets (`../../development/workflow.md`).

- **On the host,** in jig's default suites: each part's pure half (the
  mapping and its overrides; patches applied to a model of the document,
  printed as HTML and compared with golden files of jig's test pages;
  the small store's text; scenes as SVG; the transport's order over a
  fake platform: writes before exchanges, a failed write failing the
  step's exchanges, a navigation after the writes); jig's default codec
  (every place printed and parsed back, at the root, under a prefix,
  renamed; any bytes parsed to a place, never a panic); golden scenes
  for drawing code (`../client/domain/view.md`, section 17).
- **The wasm32 builds in jig's gate:** `jig-shell-web`,
  `jig-test-shell-web` and every client crate, built for
  `wasm32-unknown-unknown` with clippy's lints as errors, since the
  parts' platform halves are compiled only there.
- **The browser tier:** a suite of its own, `--profile browser`, run by
  jig's gate after it builds jig's test bundle, within a minute on the
  slowest machine (a rule: its profile's global timeout, in
  `../../development/workflow.md`). Each test starts one engine (the
  application's binary; for jig, the reference `jig-test`) and one
  Chromium, and plays its stories with a person each: a browser context,
  its own cookies and storage. A missing Chromium or bundle fails the
  suite, naming the command that provides it; it is never skipped. The
  bundle's directory reaches the tests in `JIG_TEST_BUNDLE`.
- **The harness,** in `jig-fake-person-browser`, ordinary Rust: it runs
  the binary and Chromium as children of one loop, through skein's io;
  reads the link the binary prints; opens it in a person's page; runs
  the person's script through the face; fails a test on any trouble the
  kit reports that the story did not expect (a policy violation, an
  exception); and writes the accessibility snapshot, a screenshot and
  Chromium's stderr's tail on a failure (skein's `browser.md`, sections
  7 and 10). It also presses a drawing's mark, which the person cannot:
  the kit's press at the mark's point on the drawing's figure (section
  8; skein's `browser.md`, sections 5 and 6), after which the person
  sees what follows. An application's stories have it too.
- **The face,** in the same kit: the person's `Find`
  (`../client/domain/testing.md`, section 3) over skein's browser kit,
  roles through section 6's table, which an override keeps (6.1), names
  by words in them or exactly, text by words within one run (a sentence
  that a strong word or a link splits is several runs); `Pending` while
  the kit answers. Its acts: press; type, an input's whole value (the
  field cleared, then the words); Enter for a send; for `Go`, a
  navigation to the address's path; a reload.
- **jig's stories for a browser shell,** functions of the harness an
  application's tier calls with its binary, bundle and codec, its pages
  including jig's, to check its wiring, since its loop is its own:
  - a reload keeps a pending request, which the engine makes once;
  - a person types into a field while a delivery patches the page, and
    the field keeps its focus, its caret and its words;
  - back and forward move between pages; an address opened directly
    shows its page;
  - the engine killed and restarted shows offline, then live;
  - a spent link is refused; the sign-in page says to open the new one.
- **jig's tier** calls them, and adds: a person opens the printed link
  and sees their inbox, under the policy, with no trouble; a drawing is
  drawn, its fallback read by its name, and a press on its mark, through
  the harness, acts as the ordinary node beside it does.
- **jig's test application's shell takes steps of 3.5 once each:** an
  element override; a widget for a kind of its test family, calling one
  function of a JavaScript snippet of at most 10 lines, so the bundle
  and the policy carry an application's JavaScript; its own page or the
  bundler's (the plan's choice); a prefix on jig's codec in one story;
  a policy addition refused at start, in a host test.
- **What an application adds:** its tier, over the same harness and
  face: jig's stories, called; its pages, drawings, widgets, sign-ins.

## 16. What is not the layer's

- **Following data as it comes** (a log tailed, a metric's latest points)
  is an application's streams (`../client/domain/contract.md`, 7.4): a
  contract and engine piece, carried by jig's parts as any watch, with
  no change to them. It waits for an application whose people follow
  its data as it comes.
- **Graphics beyond a scene** (a canvas of hundreds of thousands of
  points, three dimensions, video) are out of jig's scope: a widget of
  the application's in a browser (6.1; 3.5, step 6), its own shell
  elsewhere (`../client/domain/view.md`, section 11).
- **Later,** each adding without changing what sections 3 to 15 settle:
  - *jig's own drawing kinds,* a chart every application shares: a kind
    moves into jig, as jig's, when a second application draws the same;
  - *tabs of one person sharing one stream, so that many tabs hold one
    connection, and HTTP/2* (`client.md`, section 15): wait for HTTP/2
    in skein's HTTP, or for people keeping many tabs open;
  - *the bundle served compressed,* compressed at bundling and served in
    its encoding to a browser that accepts it: waits for a bundle whose
    size over the network matters, between hosts;
  - *the browser tier in other browsers than Chromium:* waits for a bug
    that shows only there (skein's `browser.md`, section 11); *a press
    that copies text, downloads and uploads of files:* for an
    application whose people need them;
  - *`ops`'s browser shell,* `jig-ops-shell-web`: comes with `ops`
    standalone (`../examples.md`, 9.1), which gives it an engine.

## 17. Open questions

- **Escape on a dialog:** dismissing a dialog by Escape needs the tree to
  name the button Escape presses, which it cannot; until it can, Escape
  does nothing of its own (section 7).
- **Right-to-left languages:** what they ask of the elements, the
  stylesheet and scenes, beyond `<html lang>` and the phrasebook; it
  waits for an application in such a language
  (`../client/domain/view.md`, section 14).
- **Media from the origin:** the closed list (section 12) has no audio
  or video type; one waits for an application serving its own media.

## 18. What it owes and is owed

- **The view** (`../client/domain/view.md`) owes: a link node's address,
  without acting (section 5); drawing code and scenes, `draw` routed by
  the kind's owner and `jig_view::scene` within its limits (sections 10,
  11 and 17); the names of jig's style tokens and region names, and the
  application's from its view root (section 13); the parts' own phrases
  among jig's (section 7); the column header and row header roles
  (section 3).
- **`jig-protocol-client-link`** owes jig's default codec: its `address`
  module, parsing and printing jig's places, total and bounded, at the
  root, under a prefix, or renamed segment by segment (section 9).
- **`jig-protocol-engine-clients`** owes: what the startup gives, from
  memory; the page at every other path outside `/jig/` and at the link's
  route; the caching headers; the front door, with the policy's
  additions 12.1 admits and a start refused for any other; the sign-in
  cookie, set and cleared; providers' sign-ins by navigation, ending in
  a `303` to a checked `back` (sections 11, 12 and 12.1).
- **`configuration.md`,** section 3, owes the clients' member the
  bundle's directory, or none, which the reference startup reads.
- **An application's browser shell** is owed jig's parts and the
  reference to copy; driving fetch and the small store without the
  transport, it owes the transport's order (5.3).
- **skein owes:** lib, `skein-codec` and `skein-http` built for wasm32
  and checked by its gate (skein's `lib.md`, section 2; skein's
  `testing.md`, section 6); its browser kit's query matching a name by
  words, and its press at a point within a node's box (skein's
  `browser.md`, sections 5 and 6); a client compiled to WebAssembly in
  its model, with no kernel and no ring, the glue crates its only
  `unsafe`, a panic a trap the restart part reloads from
  (`programming-model.md`, 2.1).
- **smith owes** `smith-transcript` building for wasm32, which jig's
  gate checks through `jig-protocol-transcript`.

## 19. What stays fixed, and why

Each rule is jig's; no value an application gives removes it. The engine
holds those of its front door and its serving (section 12); a part holds
the rest, and an application that replaces it (3.5, step 9) holds the
rule itself. One application per origin is each deployment's to keep.

| Fixed | Why | Instead |
|---|---|---|
| the policy's floor (12.1) | A script that runs on the page runs as the person: the cookie is HTTP-only, but the script can send any exchange with the browser header, so code from anywhere but the origin's bundle, or inline, is the person's account. A frame of another site over the page could trick a press on a decision (accepting a restart, a hold). Styles injected inline, with fonts or images from another origin, would read the page's text out (by attribute selectors, by a font's ligatures), a credential shown once with it. | additions for data and other sites' pages (12.1); its JavaScript, from the origin (3.5, step 7); a widget (6.1) |
| the `Host` check (section 12) | DNS rebinding: a page of another site, its name rebound to the loopback address, would reach the engine as its own origin; answered `421`, it reaches nothing. | its listener, and between hosts its certificate's name, in its configuration (`configuration.md`, section 3) |
| the cookie's attributes and the browser header (section 11) | Cross-site requests: a page of another site can neither send the cookie (same-site strict) nor set the header (it would need a preflight the engine never answers), so it cannot act as the person. HTTP-only keeps the secret from every script, the module's too; the `Path`, from every request but the API's. | its own provider (`client.md`, section 8), which ends with the same cookie |
| one application per origin | Session storage, `'self'` in the policy and the cookie are per origin: two applications on one origin would share the small store's storage, each other's scripts and the sign-in, and either could act as the person in the other. Hence no base path. | an origin each, by host name or port |
| `/jig/`, the one reserved first segment (section 9) | The link, the native shell, the engine's routes, the printed link, the cookie's `Path` and the `back` check all name it; a setting would let them drift apart, a cookie not covering the API or a `back` reaching it. It costs an application one first segment. Recorded, not taken: the prefix as an application's constant, which would break nothing kept, since a printed link is short-lived and no bookmark names the API. | every other path, its own (section 9) |
| the closed list of served types (section 12) | With `nosniff`, a file is the type the engine says; the list holds only the types a page loads, each weighed under the policy, and keeps off the origin the rest a browser would run or render (XML, a PDF, a plugin's), which `'self'` would admit. | any file of those types, under `/jig/client/<name>/` or at a root path (3.5, step 5) |
| the checked `back` (section 11) | A `back` naming another site, or read as one by a browser (`//`, `/\`), would send the person, just signed in, to a page passing for the application's; one under `/jig/`, into the engine's routes rather than a page. | any path of its own, printed by its codec |
| the referrer policy (12.1) | The page opened at the link's route holds the sign-in's secret in its address until the sign-in part replaces it; a policy sending a path to another origin would send the secret with a first request to an addition's origin. | `strict-origin`, the origin alone |
| text never read as markup (section 6) | A node's text carries what agents, systems and other people wrote; read as markup, it would run as the person, or restyle the page to read a credential out. | Markdown, read into runs by the view (`../client/domain/view.md`, section 8); a widget building its elements from its node's data (6.1) |
| the credential never kept (section 11) | A service's credential acts as the service; kept in a tab, it would outlive its dialog, in storage any script of the origin reads and in the session's restore. | the person copies it from the dialog, which the application's stylesheet may restyle |
| the transport's and the inbox's invariants (5.3) | A request that leaves before it is kept is lost or made twice after a reload; a turn inside a turn steps the client host on a step half done. They are the client's "once" (`client.md`, section 10), which jig owns (`../README.md`, section 6). | its own order for the rest of the turn (5.2); the part replaced, the invariant taken with it and checked by jig's stories for a browser shell (section 15) |
| an element override keeping the node's accessible role and name (6.1) | A person reads the page with a screen reader as the fake person reads the tree (`../client/domain/view.md`, section 12): a role lost is a control a screen reader cannot name, and that the face, finding by role, cannot find. | any element, the page part adding ARIA's role where it differs; a widget, for what no element gives (6.1) |
