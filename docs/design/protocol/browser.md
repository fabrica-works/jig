# The browser's shell

Provisional, 2026-10-14. `jig-shell-web`, and everything a browser needs
that every jig application reuses: what an application writes for its
browser client, the toolchain and the bundle, the shell's loop, the tree
as elements, patches and acts, scenes drawn as SVG, addresses as paths,
transport and the small store in a tab, signing in by cookie, the
engine's serving of the bundle under its policy, jig's stylesheet, the
limits, and the browser tier. The client's protocol is `client.md`; the
tree the shell maps is `../client/domain/view.md`. What is still open is
listed in section 17, and what waits in section 16.

## 1. In one page

- **Every application's, written once:** the browser's shell, the
  bundler, the engine's serving of the bundle under its policy, signing
  in by cookie, jig's paths, scenes as SVG, jig's stylesheet, and the
  browser tier's face and harness.
- **What an application writes:** one small crate,
  `<application>-shell-web`, giving the shell its client host and its
  look; a stylesheet; its pictures (section 3).
- **No framework; the tree as elements.** The client and its view are
  step code, unchanged. The shell maps each node to one of HTML's own
  elements, with ARIA's role where HTML has none, by one closed table
  (section 6). The only JavaScript jig writes is a two-statement starter.
- **Diffed in place:** patches are applied to the elements they name,
  so focus, caret, selection, composition and scroll stay (section 7).
- **Drawings from scenes, as SVG:** an owner's drawing code turns a
  drawing's data and its box's size into marks in whole pixels, which
  the shell draws as inline SVG styled by tokens; the fallback stays for
  a screen reader, the fake person and every other shell (section 8).
- **The platform's parts:** fetch, session storage, history, its clock,
  date and entropy, its keys, menus, disclosures and dialogs.
- **Built by jig's bundler** with `wasm-bindgen`'s glue from its own
  library, pinned with its macros: no wasm-pack, no npm (section 4).
- **Served by the engine under a strict policy:** the bundle from memory
  on its one origin, the page at every address, a content security
  policy admitting only the origin's files, a `Host` check; signing in
  by an HTTP-only, same-site-strict cookie (sections 11 and 12).
- **Tested in Chromium through jig's harness, and on the host**
  (section 15).
- **What is not the layer's:** following data as it comes, an
  application's streams; graphics beyond a scene, an application's own
  shell (section 16).

## 2. Where it sits

```
a person, a screen reader
  ▲ elements, focus, keys, the address bar, back and forward
the browser: fetch, session storage, history, clock, date, entropy
  ▲ the glue: wasm-bindgen, js-sys, web-sys
jig-shell-web            loop, elements, scenes as SVG, small store, transport
  ▲ Client, the look     <application>-shell-web
the client host          client root, view root, jig-protocol-client-link
  ▲ the wire's bytes: exchanges, one stream
════════════════════════════ one origin ════════════════════════════
jig-protocol-engine-clients   the bundle, the page, the policy, the API
```

- **A person** reads the page by sight or with a screen reader, and acts
  on it with a pointer and keys. **The browser** is any current one, which
  the shell targets; the browser tier drives Chromium (section 15).
- **`jig-shell-web`,** jig's crate, ordinary Rust, built for
  `wasm32-unknown-unknown` and for the host, is the platform's part
  (`../client/domain/view.md`, section 11). It knows an application only
  through `Client` and the look.
- **The application's browser crate** is compiled to the module (3.1).
  The client host is the application's one step machine (3.2).
- **`jig-protocol-engine-clients`,** the engine's side of the client's
  protocol (`client.md`, section 2), also serves the bundle and the
  page, sets the policy, checks `Host`, and signs a browser in by cookie
  (sections 11 and 12).

## 3. What an application writes

Its extension points are the tree's: its own pages and arrangements, its
style tokens and stylesheet, its region names, its pictures, its drawings
with their drawing code (scenes), drawings acted on, its shortcuts, its
phrasebooks. It writes no HTML, no JavaScript, no element and no
listener. Its binary reads the bundle at start (section 12), as
`jig-test` does (`service.md`, section 7); its browser tier is its own
(section 15).

### 3.1 Its browser crate

- **`<application>-shell-web`:** jig's test application's is
  `jig-test-shell-web`; `ops`'s will be `jig-ops-shell-web`. It decides
  nothing: what it would decide is the client host's or the shell's.
- **It implements `jig_shell_web::Client`,** a trait of the shell,
  ordinary Rust, for its client host, which stays step code; and gives
  its look (3.3).
- **It exports the start** through `wasm-bindgen`'s start attribute,
  which `jig-shell-web` re-exports so its version is jig's. The start
  hands the shell its `Client` and its look.

### 3.2 The client host

- **The application's client root, view root,
  `jig-protocol-client-link` and the routing between them,** one step
  machine. It is the application's, since its routing matches the
  application's types: its root's events and requests, its families'
  records in the link's.
- **Shared by every shell** the application uses: its terminal client
  (`../client/domain/view.md`, section 15) runs the same host.
- **Copied from jig's test application's,** the reference, as a client
  root is copied from its reference (`../client/domain/root.md`,
  section 12).
- **Its paths:** it parses and prints the application's, and hands
  jig's to `jig-protocol-client-link`'s `address` module (section 9).
- **`Client`,** in words (its names are the plan's):
  - *`start`:* the seed, the path the page opened at, the small store's
    records as read, the person's offset from universal time and their
    languages;
  - *`step`:* one event: a transport completion for the link (an
    exchange answered or failed, the stream opened, a piece of it, its
    end, the small store written or its write failed), a place gone to
    as a path, an act on a node, a deadline, signed out, a sign-out
    lost. It gives its duties in order: the small store's writes and
    removals; exchanges sent and abandoned; the stream opened and
    closed; an address shown as a path, a new entry or in place; a
    sign-in with a provider coming back to a path; a sign-out; a
    service's credential shown once;
  - *`build`:* patches, when what is shown moved
    (`../client/domain/view.md`, section 6);
  - the last tree; the path a link node leads to, without acting; the
    next deadline; whether closing the tab would lose requests; the
    look.

### 3.3 The look

What the application gives its shells when they are built:

- **its name,** which the title and the bundle's file names carry;
- **the names of its style tokens and region names,** each starting with
  its name and a hyphen, from its view root (section 13);
- **its pictures:** each of its owners' pictures' file in the bundle's
  `pictures/<name>/`, or none (4.2);
- **its drawing code:** its view root's `draw`, routing a drawing by its
  kind's owner (section 8; `../client/domain/view.md`, section 17).

### 3.4 Its stylesheet, pictures and phrasebooks

- **Its stylesheet,** CSS, the only text it writes that is not Rust,
  and **its pictures' directory** go to the bundler with its module
  (section 4); its stylesheet loads after jig's (section 13).
- **Its phrasebooks are in its module,** as data its view root takes at
  start (`../client/domain/view.md`, section 7).

## 4. The toolchain and the bundle

### 4.1 The toolchain

- **`wasm32-unknown-unknown`, stable Rust,** and the glue crates,
  `wasm-bindgen`, `js-sys` and `web-sys` (only the features the shell
  uses), pinned to exact versions in jig's workspace: the only `unsafe`
  the module runs (`programming-model.md`, 2.1). `jig-shell-web` forbids
  `unsafe` in its own code.
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
- **The shell's two halves:** its pure parts build for the host too
  (section 15); its platform part, over the glue crates, for wasm32 only.

### 4.2 The bundler and the bundle

- **`jig-bundle`,** jig's tool crate, ordinary Rust, a library with a
  small binary, makes the bundle from the module (a `.wasm` file cargo
  built), the application's name, its stylesheet and its pictures'
  directory. The bundle is a directory, written whole:
  - `index.html`, the page, and `boot.js`, the starter (4.3);
  - `<name>.js` and `<name>_bg.wasm`: the glue and the module;
  - `jig.css`, jig's stylesheet, which ships with `jig-shell-web`, and
    `<name>.css`, the application's (section 13);
  - `pictures/jig/`, jig's pictures, and `pictures/<name>/`, the
    application's, so the two never clash.
- **Only what the engine serves:** a picture whose type is not in the
  engine's closed list (section 12) refuses the bundle, naming it.
- **jig's texts that are not Rust:** the page and the starter, which the
  bundler writes, and jig's stylesheet. Nothing inline.
- **Its size is a guide:** jig's test application's module, built with
  `web`, at most 4 MiB before compression, measured when first built and
  kept with jig's budgets (`../../development/workflow.md`). Past it,
  what grew is reported.

### 4.3 The page and the starter

- **The page,** `index.html`, the one HTML document the engine answers
  for every address (section 12): `<meta charset="utf-8">`, a viewport,
  a title of the application's name, jig's stylesheet then the
  application's, the starter as `<script type="module">` by `src`, and a
  `<noscript>` saying the client needs WebAssembly and scripts.
- **Its references are paths from the origin's root,** under
  `/jig/client/`, since the page is answered at every address.
- **The starter,** `boot.js`: two statements,
  `import init from "./<name>.js"; init();`. The glue finds the module
  beside it.

## 5. The loop

- **Starting.** The starter loads the glue, the glue the module, and the
  module's start hands the shell its `Client` and look. The shell reads,
  once: the path the page opened at, the small store (section 10), its
  reload count (below), the clock, the date, a seed, the person's offset
  from universal time and their languages. At the link's route it signs
  in first (section 11). It then calls `start`, builds, and applies the
  first patches, which insert the whole tree under the page's root.
- **One callback, one step.** Each platform report is one event: the
  shell steps the client, then does its duties in the order the step
  gave them: the small store's writes, then exchanges and the stream,
  then the address, then a sign-in, a sign-out or a credential shown;
  then it builds, applies the patches and refreshes its links' paths
  (section 9); then it sets the title (section 7), arms its timer, and
  keeps its warning on closing (section 10).
- **Session storage is synchronous:** the shell writes the step's
  records, then steps the client with written, or the write failed,
  before the step's exchanges leave, as the native shell does
  (`client.md`, section 10).
- **No re-entry:** a report the platform makes while a step runs or
  patches apply (a focus moved, an element removed) waits, in order,
  and is taken one step each after.
- **One timer,** armed to the earliest deadline: the client's, and the
  shell's own (a hidden tab's, section 10).
- **Time and the seed are the shell's to read:** the clock from the
  platform's monotonic clock, wall time from its date, the seed from its
  cryptographic generator (`../client/domain/view.md`, section 11).
- **A panic is fail-stop.** The shell's panic hook reloads the page,
  which starts the client again from its small store. At most three
  reloads in a minute (a rule), counted by the platform's date in
  session storage, under a key of the shell's own outside the small
  store's. Past it, the page says the client failed, starts nothing, and
  keeps the small store; a reload past the minute starts it again.
- **The shell's own words** (a credential shown once, the client failed)
  are jig's phrases (`../client/domain/view.md`, section 7), in the
  person's language.

## 6. The tree as elements

Each node is one element, by its role; the browser's face finds the
person's roles through the same table (section 15).

| Role | Element (ARIA role where the element has none) |
|---|---|
| page | the page's root `<div>` in `<body>` |
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
- **A node's id is kept on its element,** as `data-node`. The shell's
  own elements (the page's root, a scene's SVG, a menu's button, its
  dialog for a credential) carry none.
- **One listener per kind of event,** on the page's root, finds the node
  of the element acted on, by its nearest `data-node`.

## 7. Patches and acts

- **Patches applied in place** (`../client/domain/view.md`, section 6):
  - *an insert* creates a subtree's elements under its parent, before
    its sibling or at the end; each link gets its path (section 9), each
    drawing its scene (section 8);
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
- **Acts,** each reported with its node's id (`../client/domain/view.md`,
  section 5), none on a disabled node:
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
  - a check's or a choice's change: `Toggle`; the shell then puts the
    element's checked state back to the tree's, which the next patch
    moves if the domain followed;
  - a press in a drawing's scene: `Drawing` (section 8).
- **Shortcuts:** the platform's keys turned into a press of the first
  bound, enabled node in preorder naming the key; a shortcut with no
  modifier never while a field has the focus
  (`../client/domain/view.md`, section 5).
- **Menus and disclosures are the shell's to fold,** with no step. A
  disclosure is `<details>` and `<summary>`. A menu is a button of the
  shell's, named by the menu's name, that opens its items with the
  platform's keys (arrows, Escape): ARIA's menu-button pattern. A press
  on an item closes the menu.
- **A dialog** is `<dialog>`, shown modal while it is in the tree. Escape
  does nothing of its own: the dialog's buttons decide.
- **A log** keeps its end in view while the person has not scrolled away
  from it (`../client/domain/view.md`, section 9).
- **Going to another page** (the address changed by a step): the page
  starts at its top, the focus on its main's first level-1 heading,
  else the main; back and forward restore their scroll (section 9).
- **The title:** the main's first level-1 heading's text, then ` · ` and
  the application's name; the name alone when there is none.
  `<html lang>` is the language of the phrasebook the client chose.

## 8. Scenes as SVG

A scene is drawing code's output: a bounded list of marks in preorder, in
whole pixels from its box's top left (`../client/domain/view.md`,
section 17).

- **The shell calls the drawing code,** through the look, with the
  drawing's kind, its data, and its box's size in whole CSS pixels as the
  shell measured it. False, for a kind or data it does not draw, leaves
  the fallback shown.
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
  its box's size changes, which is the shell's alone: it calls the
  drawing code again, with no step.
- **The fallback** is the drawing node's children, ordinary elements.
  While the scene is drawn, the shell marks the figure drawn, and jig's
  stylesheet hides them from sight, not from assistive technology, and
  they take no space. The text shell and the fake person read the
  fallback, unchanged.
- **Acted on:** a press in the scene reports `Act::Drawing { data }` on
  the drawing's node, its data the act the press takes, in the order of
  `../client/domain/view.md`, 17.3: the topmost mark's under the point,
  else its nearest group's, else the scene's with the point appended,
  else none. The shell gives a mark that carries an act jig's class for
  acts, so it takes presses over its whole area, filled or not (section
  13). The view drops data past its bound on a drawing's bytes, and the
  owner's view decodes the rest (`../client/domain/view.md`, section 5).
  It is by pointer only: the ordinary node beside the drawing, doing the
  same, is the keyboard's and a screen reader's.

## 9. Addresses and history

- **jig's paths,** fixed by design: a path changes only by a change to
  this table, since bookmarks rest on them.

  | Address | Path |
  |---|---|
  | the inbox | `/inbox`; filtered by a project, `/inbox/<project>` |
  | conversations | `/conversations`; by a project, `/conversations/<project>` |
  | a task's overview | `/tasks/<task>` |
  | a task's other section | `/tasks/<task>/<section>`: `activity`, `plan`, `history`, `budget`, `panels` |
  | a project's overview | `/projects/<project>` |
  | a project's other section | `/projects/<project>/<section>`: `goals`, `tasks`, `notes`, `settings`, `resources`, `policy`, `roles`, `spend`, `recurring` |
  | the deployment | `/deployment` |

- **`/` is the application's home** (`../client/domain/pages.md`,
  section 2).
- **Numbers are decimal,** with no leading zero, within their type. One
  trailing slash is ignored; a query and a fragment are not part of the
  address. Anything else is a place the client has no page for, shown as
  missing (`../client/domain/pages.md`, section 8).
- **jig reserves the first segments** `inbox`, `conversations`, `tasks`,
  `projects`, `deployment` and `jig`. An application's paths are its
  own, under any other first segment.
- **Parsed and printed below the domain:** the client host parses and
  prints the application's paths, and hands jig's to
  `jig-protocol-client-link`'s `address` module, total, bounded step
  code, since a URL is a person's input.
- **A link's `href`:** a link node's binding resolves, without acting, to
  the address it leads to (the view root asks the binding's owner's
  view), which the client host prints as a path. The shell asks when the
  link is inserted, and again for every link it holds after each build,
  setting each `href` whose path changed, since a changed binding makes
  no patch (`../client/domain/view.md`, section 6).
- **History:** an address a step shows is a new entry, or replaces the
  current one, as the step says. The shell keeps the scroll in each
  entry's state as the page leaves it. Back and forward are a place gone
  to, as a path: one step, then, after its patches, the scroll the entry
  kept. A path opened directly (a bookmark, an address typed, a link
  opened in a new tab) starts a client at it.

## 10. Transport and the small store

- **Exchanges** by the platform's fetch: a POST of the wire's bytes to
  the path the link gives, `Content-Type: application/octet-stream`, the
  browser header (section 11), same-origin credentials; abandoned by
  aborting. A body is read up to the link's largest record; past it, the
  exchange failed.
- **The stream** by fetch, with same-origin credentials, its body read a
  piece at a time: each piece, cut to the link's stream piece, handed to
  the link as it comes, the next read after the step, so the platform
  paces it (`client.md`, section 7).
- **A hidden tab lets its stream go** a minute after it is hidden (a
  rule; the shell's own deadline), telling the link the stream ended, as
  any drop. An opening asked while hidden waits until the tab is shown.
  A browser allows about six connections to one origin over HTTP/1.1,
  each tab's stream holding one, so hidden tabs must not hold theirs.
- **The small store** is the tab's session storage
  (`../client/domain/client.md`, section 11): each record under its key,
  both as base64 text, the key prefixed `jig.`; read whole at start. A
  write the platform refuses, its quota full, is a failed write.
- **Closing a tab with requests pending** asks the platform to warn
  (`beforeunload`), registered only while `Client` says closing would
  lose requests.

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
  - the shell, started at that route, makes the link's sign-in exchange,
    the POST the native shell makes, with the secret from the address's
    `secret` and the browser header; the engine answers it signed in,
    the secret in the cookie and none in the body;
  - the shell then replaces the address with `/`, so the secret leaves
    the history, and starts the client;
  - refused (a spent or expired secret), the engine prints a fresh link,
    as for any refused link, and the shell starts the client at `/`,
    which shows the sign-in page. Not ready or lost, the shell does the
    same; a link not spent still works opened again.
- **A provider:** a sign-in with a provider, coming back to a path,
  navigates the page to `/jig/v1/sign-in/provider/<provider>?back=<path>`,
  once the step's small store is written:
  - for the deployment's link, the engine prints a fresh link and
    redirects (`303`) to `back`, whose sign-in page says to open the
    link the engine printed;
  - for an application's provider, the engine routes the GET, and its
    redirect back, to the provider's component (`client.md`, section 8),
    and ends with the cookie set and a `303` to `back`;
  - `back` must be a path: one leading `/`, not `//`, not under `/jig/`;
    else `/`.
- **Signing out** is the shell's exchange to `/jig/v1/sign-out`, which
  clears the cookie (`Max-Age=0`). Answered signed out, the shell tells
  the client signed out; any other end, the sign-out lost
  (`../client/domain/client.md`, section 12).
- **A service's credential, shown once,** in a dialog of the shell's own,
  outside the view's tree: selected text, and a button closing it, in
  jig's words. The shell never keeps it.

## 12. Served by the engine

`jig-protocol-engine-clients` serves the browser's client from the
engine's one origin (`client.md`, section 4):

- **The bundle's files** under `/jig/client/<file>`, by their paths in
  the bundle, from memory. The application's startup reads them from the
  directory jig's configuration names (`configuration.md`, section 3,
  clients: the bundle's directory, or none, which serves no browser),
  with skein's startup kit (skein's `shell.md`, 6.1), and hands them to
  the component, whose worst case counts them. Each file's type comes
  from a closed list, by its extension: `.html`, `.js`, `.wasm` as
  `application/wasm`, `.css`, `.svg`, `.png`, `.webp`, `.ico`,
  `.woff2`. Any other file in the directory refuses the start.
- **The page** on every GET of a path outside `/jig/`, and of the link's
  route. Under `/jig/`, a GET of anything but a file, the stream, or a
  provider's sign-in route and its redirect back (section 11) is not
  found.
- **Caching:** each file and the page with an `ETag`, a digest of its
  bytes computed at start, and `Cache-Control: no-cache`, a matching
  `If-None-Match` answered `304`; the API's answers with
  `Cache-Control: no-store`.
- **Every answer** with `Content-Security-Policy: default-src 'none';
  script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src
  'self'; font-src 'self'; connect-src 'self'; base-uri 'none';
  form-action 'none'; frame-ancestors 'none'`, and
  `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`,
  `Cross-Origin-Opener-Policy: same-origin`,
  `Cross-Origin-Resource-Policy: same-origin`.
- **Only its own `Host`:** a request whose `Host` is not the engine's own
  is answered `421`, and nothing else is done, so a page of another site
  rebound to the loopback address reaches nothing. On a loopback
  listener, its own is the address its printed link names; between
  hosts, with TLS, the name of the listener's certificate
  (`configuration.md`, section 3, clients).
- **No preflight:** an `OPTIONS` is not found.

## 13. Stylesheets

- **Every style token and region name has a name.** jig's start with
  `jig-`: its tokens `jig-card`, `jig-chip`, `jig-muted`, `jig-warning`,
  `jig-danger`, `jig-success`, `jig-button-weight`, `jig-dense-list`;
  its regions as `../client/domain/pages.md`, 12.1 and 12.2, list them.
  An application's start with its name and a hyphen, given by its view
  root as its look.
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
  tokens and regions, and may restyle jig's.
- **No style is a fact** (`../client/domain/view.md`, section 13).

## 14. Limits

- **The link's:** an exchange's body, at most its largest record; a piece
  of the stream; exchanges in flight (`client.md`, section 13).
- **The view's:** a tree's nodes, a diff's patches, a scene's marks and
  bytes, a drawing's bytes (`../client/domain/view.md`, section 10); a
  drawing's SVG has at most a scene's marks.
- **The platform's:** session storage's quota, past which a write fails.
- **The shell's own:** one timer; one read of the stream outstanding;
  waiting reports only those its own calls raise while a step runs.
- **The bundle's size,** a guide (4.2).

## 15. Testing

Each within jig's suites' budgets (`../../development/workflow.md`).

- **On the host,** in jig's default suites: the shell's pure parts (the
  element mapping; patches applied to a model of the document, printed
  as HTML and compared with golden files of jig's test pages; the small
  store's text; scenes as SVG); the paths (every address printed and
  parsed back; any bytes parsed to a place, never a panic); golden
  scenes for drawing code (`../client/domain/view.md`, section 17).
- **The wasm32 builds in jig's gate:** the shell, jig's test
  application's browser crate and every client crate, built for
  `wasm32-unknown-unknown` with clippy's lints as errors, since the
  shell's platform code is compiled only there.
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
  roles through section 6's table, names by words in them or exactly,
  text by words within one run (a sentence that a strong word or a link
  splits is several runs); `Pending` while the kit answers. Its acts:
  press; type, an input's whole value (the field cleared, then the
  words); Enter for a send; for `Go`, a navigation to the address's
  path; a reload.
- **jig's stories,** each a sentence, a test or a few to a test:
  - a person opens the printed link and sees their inbox, under the
    policy, with no trouble;
  - a person types into a field while a delivery patches the page around
    it, and the field keeps its focus, its caret and its words;
  - a reload keeps a pending request, which the engine makes once;
  - back and forward move between pages, and an address opened directly
    shows its page;
  - the engine killed and restarted shows offline, then live;
  - a drawing is drawn, its fallback read by its name, and a press on
    its mark, through the harness, acts as the ordinary node beside it
    does;
  - a spent link is refused, and the sign-in page says to open the one
    the engine printed.
- **What an application adds:** its browser tier, with its binary and
  its bundle, over the same harness and face: its pages, its drawings,
  its signing in.

## 16. What is not the layer's

- **Following data as it comes** (a log tailed, a metric's latest points)
  is an application's streams (`../client/domain/contract.md`, 7.4): a
  contract and engine piece, carried by the browser's shell as any
  watch, with no change to the shell. It waits for an application whose
  people follow its data as it comes.
- **Graphics beyond a scene** (a canvas of hundreds of thousands of
  points, three dimensions, video) are out of jig's scope: such an
  application writes its own shell over the same domain and view
  (`../client/domain/view.md`, section 11, "An application's shell").
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
  - *`ops`'s browser crate,* `jig-ops-shell-web`: comes with `ops`
    standalone (`../examples.md`, 9.1), which gives it an engine.

## 17. Open questions

- **Escape on a dialog:** dismissing a dialog by Escape needs the tree to
  name the button Escape presses, which it cannot; until it can, Escape
  does nothing of its own (section 7).
- **Right-to-left languages:** what they ask of the elements, the
  stylesheet and scenes, beyond `<html lang>` and the phrasebook; it
  waits for an application in such a language
  (`../client/domain/view.md`, section 14).

## 18. What it owes and is owed

- **The view** (`../client/domain/view.md`) owes: a link node's address,
  without acting (section 5); drawing code and scenes, `draw` routed by
  the kind's owner and `jig_view::scene` within its limits (sections 10,
  11 and 17); the names of jig's style tokens and region names, and the
  application's from its view root (section 13); the shells' own phrases
  among jig's (section 7); the column header and row header roles
  (section 3).
- **`jig-protocol-client-link`** owes jig's paths: its `address` module,
  parsing and printing them, total and bounded (section 9).
- **`jig-protocol-engine-clients`** owes the bundle's files from memory;
  the page at every path outside `/jig/` and at the link's route; the
  caching and policy headers; the `Host` check; the sign-in cookie, set
  and cleared; the browser header required with the cookie; providers'
  sign-ins by navigation, ending in a `303` to a checked `back`
  (sections 11 and 12).
- **`configuration.md`,** section 3, owes the clients' member the
  bundle's directory, or none.
- **skein owes:** lib, `skein-codec` and `skein-http` built for wasm32
  and checked by its gate (skein's `lib.md`, section 2; skein's
  `testing.md`, section 6); its browser kit's query matching a name by
  words, and its press at a point within a node's box (skein's
  `browser.md`, sections 5 and 6); a client compiled to WebAssembly in
  its model, with no kernel and no ring, the glue crates its only
  `unsafe`, a panic a trap the shell restarts it from
  (`programming-model.md`, 2.1).
- **smith owes** `smith-transcript` building for wasm32, which jig's
  gate checks through `jig-protocol-transcript`.
