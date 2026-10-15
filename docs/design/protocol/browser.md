# The browser's layer

Provisional, 2026-10-14, revised 2026-10-15. jig's browser layer:
skein's browser parts (`skein-ui-web`, skein's `ui-web.md`), and what
jig adds to them in `jig-shell-web`: the sign-in part, the link's
adapter and jig's look. It says what an application on jig writes; what
of the toolchain, the bundle, the loop, the tree as elements, patches,
scenes and addresses is jig's beside skein's; jig's link over skein's
fetch and transport; signing in by cookie; the engine's serving under
its policy; jig's stylesheet; the limits; jig's tier; and what stays
fixed (section 19). The client's protocol is `client.md`; the tree the
parts map is skein's (skein's `ui.md`), and jig's nodes in it are
`../client/domain/view.md`'s. What is still open is listed in section
17, and what waits in section 16.

## 1. In one page

- **skein's parts and jig's:** jig's browser layer is skein's browser
  parts, `skein-ui-web` (the inbox, the page, scenes as SVG, fetch, the
  small store, the transport, history, the timer and the rest: skein's
  `ui-web.md`, 5.1), and jig's, in `jig-shell-web`: the sign-in part,
  the link's adapter and jig's look (5.1; section 13). Each is ordinary
  Rust, usable alone, with plain-data inputs and outputs; none runs a
  loop or holds an application.
- **The application owns its start and its loop:** its browser shell,
  `<application>-shell-web`, over those parts and its client host, is
  copied from jig's reference shell, `jig-test-shell-web`, which is
  written after skein's and adds jig's link, sign-in and client host
  (section 3).
- **skein's parts hold the invariants that protect requests,** whatever
  the loop's order; jig's link's "once" rests on its transport (5.3).
- **Open by steps,** skein's, with jig's places in them: files under the
  bundle's prefix, `/jig/client/`; root files; policy additions; never
  the engine's front door (3.5).
- **The tree as elements, patches and acts, scenes as SVG:** skein's
  (sections 6 to 8). jig's nodes take the default table; Escape in a
  dialog presses the button jig's confirmation names with it (section 7).
- **Its paths are the application's,** by its own codec; jig's default
  codec gives jig's places theirs; jig keeps `/jig/` alone (section 9).
- **jig's link over skein's fetch and transport:** a POST of
  `application/octet-stream` with the browser header, its answer cut at
  the link's largest record, the small store under `jig.` (section 10).
- **Built by skein's bundler** under the bundle's prefix, with `jig.css`
  and jig's pictures (section 4).
- **Served by the engine:** what the application's startup gives, behind
  a fixed front door (the `Host` check, the policy's floor, the cookie's
  attributes, the browser header); the bundle's digest directory served
  immutable, the page revalidated (sections 11, 12 and 12.1).
- **Tested** on skein's harness and face, with skein's stories for a
  browser shell and jig's (section 15).
- **What stays fixed,** with its reason and what to do instead: section
  19. **Not the layer's:** an application's streams (section 16).

## 2. Where it sits

```
a person, a screen reader
  ▲ elements, focus, keys, the address bar, back and forward
the browser: fetch, session storage, history, clock, date, entropy
  ▲ the glue: wasm-bindgen, js-sys, web-sys
skein-ui-web             skein's browser parts (skein's ui-web.md, 5.1)
jig-shell-web            jig's: sign-in, the link's adapter, the look (5.1)
  ▲ plain data: reports, patches, the transport's requests, paths, the look
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
- **`skein-ui-web`,** skein's crate: the browser parts and the default
  mapping of roles to elements, knowing no application (skein's
  `ui-web.md`, section 2). **`jig-shell-web`,** jig's crate, ordinary
  Rust, forbidding `unsafe`: jig's two parts and jig's look, over
  skein's parts, knowing no application. **The application's browser
  shell,** compiled to the module, is its start and its loop (3.1,
  section 5), with no trait between it and the parts or its client host
  (3.2).
- **`jig-protocol-engine-clients`,** the engine's side of the client's
  protocol (`client.md`, section 2), serves what the startup gives it
  behind a fixed front door, and signs in by cookie (sections 11 and 12).

## 3. What an application writes

Its extension points are the tree's (skein's `ui.md`): whatever tree it
builds with `skein-ui`, composing jig's primitives where it likes, in
jig's default arrangement or its own (`../client/domain/view.md`); its
style tokens and stylesheet, its region names, its pictures, its drawings
with their drawing code (scenes), drawings acted on, its shortcuts, its
phrasebooks, its components and its effects. In a browser it writes its
browser shell, copied from jig's reference, and its stylesheet; what
more it may change is 3.5. Its binary gives the engine what it serves,
as `jig-test` does (section 12).

### 3.1 Its browser shell

- **`<application>-shell-web`,** its own crate, ordinary Rust, as shells
  are (skein's `ui-web.md`, 3.1). Its start, through `wasm-bindgen`'s
  start attribute, which `skein-ui-web` re-exports so its version is
  skein's pin, makes skein's parts and jig's with the look (3.3) and
  starts its client host (section 5); its loop is the turn (5.2). It orders the platform's work
  and decides nothing: what to show, send and keep is its client host's;
  carrying a request safely is a part's (5.3).
- **jig's reference shell,** `jig-test-shell-web`, is written after
  skein's reference shell (the UI example's `skein-ui-example-shell-web`:
  skein's `ui-web.md`, 3.1; skein's `examples.md`, section 9), and adds
  jig's link, through the link's adapter, the sign-in part and its
  client host. It is written to be copied and commented as such, as the
  client root, view root, binary and service are
  (`../client/domain/root.md`, section 12; `service.md`, section 7):
  about 200 lines beside its client host (a guide). An application on
  jig copies jig's, not skein's.

### 3.2 The client host

- **The application's client root, view root,
  `jig-protocol-client-link` and the routing between them,** one step
  machine. It is the application's, since its routing matches the
  application's types: its root's events and requests, its families'
  records in the link's.
- **Shared by every shell** the application uses (its terminal client,
  over `skein-ui-text`, `../client/domain/view.md`, section 15, runs the
  same host), and **copied from jig's test application's,** as a client
  root is copied from its reference (`../client/domain/root.md`, section
  12).
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
  - *`build`:* patches, when what is shown moved (skein's `ui.md`,
    section 7; `../client/domain/view.md`, section 6);
  - the last tree; the path a link node leads to, without acting; the
    next deadline; whether closing the tab would lose requests; the
    language its phrasebook chose.
- **Effects** (skein's `ui.md`, section 13) are the application's: its
  client host gives those its views ask with a step's duties, and steps
  with each report; jig's views ask none.

### 3.3 The look

What the application's browser shell gives skein's page part when it
makes it, as plain data, with no trait (skein's `ui-web.md`, 3.2;
skein's `ui.md`, section 8):

- **each owner's prefix and names:** jig's, the prefix `jig` and the
  names of jig's style tokens and region names (section 13), from
  `jig-view`, under jig's owner, whose number as skein-ui's opaque owner
  is jig's convention (`../client/domain/view.md`, section 3); and,
  for each of the application's owners, its prefix and its names, each
  starting with the prefix and a hyphen;
- **its pictures:** jig's, which ship with `jig-shell-web`, under the
  bundle's `pictures/jig/`, and each of the application's owners' under
  `pictures/<its prefix>/`, or none (4.2);
- **its drawing code,** its view root's `draw`, a function routing a
  drawing by its kind's owner (section 8; skein's `ui.md`, 11.1);
- **its elements and components,** if any (6.1).

`jig-shell-web` gives jig's part of the look whole; the application's
browser shell adds its own owners' and gives the page part the two
together. **Its name** is in the title the shell sets (section 7) and in
the bundle's file names (4.2).

### 3.4 Its stylesheet, pictures and phrasebooks

- **Its stylesheet,** CSS, and **its pictures' directory** go to skein's
  bundler with its module (section 4); its stylesheet loads after
  `skein-ui.css` and `jig.css` (section 13), or where its own page puts
  it (3.5, step 4).
- **Its phrasebooks are in its module,** data its view root takes
  (skein's `ui.md`, section 9; `../client/domain/view.md`, section 7).

### 3.5 What it may change

skein's steps (skein's `ui-web.md`, 3.3), with what is jig's in each,
and one of jig's own, the policy's additions (step 8); each needing one
before it only where it says.

1. **Copy the reference shell,** jig's (3.1), and change its names: the
   standard, the same for every application.
2. **Restyle:** its stylesheet after `jig.css`, which may restyle jig's
   tokens and regions (section 13); its pictures (4.2).
3. **Its paths:** jig's default codec as is, under a prefix of its own,
   renamed segment by segment, or its own codec (section 9).
4. **Its page:** its own `index.html` (its head: title, icons, fonts,
   metadata, preloads; its mount element; its stylesheets in its order),
   copied unparsed by skein's bundler, referring through the bundler's
   marker to the bundle's digest directory under the bundle's prefix,
   `/jig/client/` (4.3; skein's `ui-web.md`, 4.3). It loads its module
   from the origin by a script file, with nothing inline: the policy
   refuses at run what breaks that (12.1), and the browser tier fails on
   any violation.
5. **Its files:** any of a type in the bundle's closed list (skein's
   `ui-web.md`, 4.4), in the bundle's `files/`, served under
   `/jig/client/<digest>/files/`; and root files, paths outside `/jig/`
   named at start (`/favicon.ico`, `/robots.txt`), each answering one of
   the bundle's files (section 12).
6. **Its elements and components** (6.1): another element for a role
   carrying one of its style tokens; a component for such a node or a
   drawing of its kinds: a code editor for a text area, a map library.
7. **Its JavaScript,** from the origin only: `wasm-bindgen`'s snippets,
   its own modules the glue imports (4.2), or files its page loads
   (step 4); never from another origin, never inline (section 19).
8. **Policy additions** (12.1): named origins for data and other sites'
   pages, and the origin's own media and manifest, above the floor.
9. **A part replaced or left out:** any of skein's or jig's, but never
   the engine's front door (section 12). One that holds an invariant
   (the transport, the inbox, the page) moves it to the application
   (5.3), which skein's stories for a browser shell check; one that
   carries jig's link (the link's adapter, fetch, the sign-in part)
   leaves jig's wire and the browser header to the application (sections
   10 and 11), which jig's stories check (section 15). There is no shell
   to write whole: the application's browser shell is its own.

## 4. The toolchain and the bundle

skein's (skein's `ui-web.md`, section 4): the toolchain; skein's
bundler, `skein-ui-bundle`; the bundle, its page and its digest
directory; the starter; and how a bundle is served. What is jig's
follows.

### 4.1 The toolchain

- **skein's** (skein's `ui-web.md`, 4.1): `wasm32-unknown-unknown`,
  stable Rust, and the glue crates pinned to exact versions in skein's
  workspace, the only `unsafe` the module runs
  (`programming-model.md`, 2.1); no wasm-pack, no npm, no binaryen.
  `jig-shell-web` forbids `unsafe`, as `skein-ui-web` does; an
  application's browser shell writes none of its own (3.5, step 7).
- **One lock:** jig's workspace builds skein's bundler beside the module,
  so the glue's generator is locked with the macros' version in jig's
  `Cargo.lock`; an application's workspace does the same in its own.
- **jig's release profile** is `web`: it inherits `release`, with
  `opt-level = "s"`, `lto = "fat"`, `codegen-units = 1`,
  `panic = "abort"`, `overflow-checks = true` and `strip = true`. An
  application's workspace defines its own the same way.
- **The browser tier's module** is built with the dev profile, as code
  under test; the bundler is a tool, built optimized
  (`../../development/workflow.md`, section 2).

### 4.2 The bundler and the bundle

- **skein's bundler writes the bundle whole** (skein's `ui-web.md`,
  4.2): the page, `index.html`, beside the bundle's digest directory,
  named by the digest of everything in it. jig's gate, and an
  application's build, give it:
  - **the bundle's prefix, `/jig/client/`,** under jig's one reserved
    first segment (section 9);
  - **`jig.css`,** jig's stylesheet, which ships with `jig-shell-web`,
    first of the given stylesheets, then the application's, `<name>.css`
    (section 13);
  - **jig's pictures,** which ship with `jig-shell-web`, as the owner
    `jig`'s, and the application's as each of its owners', under its
    prefix, so no two owners' clash;
  - the module (a `.wasm` file cargo built), the application's name,
    and, where it has them, its page and its files (3.5, steps 4 and 5).
- **jig's bundle,** then: `index.html`, and under `/jig/client/<digest>/`
  `boot.js`, `<name>.js`, `<name>_bg.wasm`, `snippets/`, `skein-ui.css`,
  `jig.css`, `<name>.css`, `pictures/jig/`, a `pictures/<prefix>/` for
  each of the application's owners, and `files/`.
- **Only what the engine serves:** a file of a type off the bundle's
  closed list (skein's `ui-web.md`, 4.4) refuses the bundle, naming it;
  the engine serves that list (section 12).
- **jig's one text that is not Rust** is `jig.css`. Nothing inline.
- **Its size is a guide:** jig's test application's module, built with
  `web`, at most 4 MiB before compression, measured when first built and
  kept with jig's budgets (`../../development/workflow.md`). Past it,
  what grew is reported.

### 4.3 The page and the starter

skein's (skein's `ui-web.md`, 4.3). In jig's bundle the page refers to
the digest directory by absolute path, `/jig/client/<digest>/`, since
the engine answers it at every path outside `/jig/` but the root files,
and at the link's route (section 12); it loads `skein-ui.css`, `jig.css`
and then the application's stylesheet, and its title is the
application's name.

## 5. The loop, the application's

The application's browser shell runs it over skein's parts and jig's
(5.1), in the reference's order (5.2); the parts hold what protects
requests (5.3). Starting and a panic's restart are skein's (skein's
`ui-web.md`, section 5), with jig's here:

- **Starting,** the reference's: the starter loads the glue, the glue
  the module, whose start makes the parts and reads through them, once,
  the clock, the date and what its client host's `start` takes (3.2).
  At the link's route it signs in first (section 11); then it starts its
  client host, builds, and applies the first patches, inserting the
  whole tree under the mount element.
- **A panic is fail-stop:** skein's restart part reloads the page, which
  starts the client again from its small store, at most three times in
  a minute (skein's default, which jig's reference keeps; the
  application's parameter), counted under a session storage key of its
  own, outside the small store's prefix. Past it, the page says the
  client failed, starts nothing, and keeps the small store; a reload
  past the minute starts it again.
- **The parts' own words** (a credential shown once, the client failed)
  are jig's phrases (`../client/domain/view.md`, section 7), part of
  jig's look, which the browser shell gives the parts.

### 5.1 The parts

skein's (skein's `ui-web.md`, 5.1), in `skein-ui-web`, and jig's two, in
`jig-shell-web`. Each is ordinary Rust, usable alone, taking and giving
plain data; jig's are built over skein's fetch and transport, and each
is usable without the other. Their module names are the plan's.

- **skein's parts,** with what jig's reference gives each:

  | Part | What jig's reference gives it |
  |---|---|
  | the inbox | the turn, and its report type (below) |
  | the page | the mount element, the look (3.3) and the default mapping, which jig's nodes keep (section 6) |
  | scenes as SVG | the look's names, for the application's drawings (section 8) |
  | fetch | the link's exchanges and its stream, as the link's adapter gives them; the sign-in part's exchanges (sections 10 and 11) |
  | the small store | the prefix `jig.` (section 10) |
  | the transport | a step's writes, its link requests through the link's adapter, and a sign-in's navigation away (5.3) |
  | history | the paths the client host prints by its codec (section 9) |
  | the timer | the earliest of the client host's and the hidden tab's deadlines |
  | the platform's facts | nothing |
  | title and closing | the title, the language, and whether the client host would lose requests |
  | restart | jig's words for "the client failed", and skein's default limit (section 5) |
  | the hidden tab | skein's default delay, a minute (section 10) |

- **jig's two:**

  | Part | Takes | Gives |
  |---|---|---|
  | the link's adapter | the link's requests of a step: exchanges (a path, bytes), abandons, the stream opened and closed; fetch's reports on them | the transport's plain data: each exchange a POST to its path with `Content-Type: application/octet-stream` and `Jig-Client: browser`, its answer read up to the link's largest record; the stream's request, each piece cut to the link's stream piece; fetch's reports as link events: answered, failed, the stream opened, a piece, its end |
  | sign-in | the page's path and query at start; a provider and a path; signing out; a credential; its dialog's words | the link's sign-in exchange, the address replaced first; a provider's navigation, for the transport to make; the sign-out exchange; the credential shown once in a dialog of its own, never kept |

- **A report** is one platform happening, as data, which each part that
  reports wraps into the application's report type by the function it
  is given (skein's `ui-web.md`, 5.1; skein's `ui.md`, section 13).
  jig's reference's report type holds skein's parts' reports, a link
  event (fetch's report through the link's adapter), and the end of a
  sign-in's or a sign-out's exchange.

### 5.2 The turn

skein's turn (skein's `ui-web.md`, 5.2), with jig's lines: those naming
the link's adapter or sign-in. An application's loop may order it
otherwise within 5.3.

```
the turn, which the inbox calls when a report waits and no turn runs:
  now, wall ← the platform's facts                 once per turn
  while the inbox holds a report:                  raised meanwhile too
    a drawing resized: to the page part, no step
    the tab hidden or shown: to the hidden tab part, no step
    the timer due: to the hidden tab part, then a step as any report
    fetch's report: a link event, by the link's adapter;
      on a sign-in's or a sign-out's exchange, its end, by sign-in
    step ← the client host steps with the report, now and wall
    away ← sign-in: a provider's navigation, if the step signs in by one
    requests ← the link's adapter, from the step's link requests
    the transport carries the step's writes and requests, then away (5.3)
    fetch reads the stream on, after a piece of it
    history shows the step's address, printed by the client host
    the step's effects, to their parts or the shell's own code
    sign-in signs out, or shows a credential once, if the step asks
  patches ← the client host builds
  the page applies them, a focus request too       an unknown node: a panic
  a new entry: its top, its heading focused, unless a focus request moved
  back or forward: the scroll the entry kept
  the page sets each link's href to the path the client host prints
  title and closing: heading · name, the language, the closing warning
  the timer: the earliest of the client host's and hidden tab's deadlines
```

### 5.3 Where the invariants live

skein's parts hold the invariants that protect requests, so no order of
the loop breaks them (skein's `ui-web.md`, 5.3); jig's link's "once"
rests on them, and the reference documents the rest.

- **The small store written before a step's exchanges leave, and a
  navigation away only after:** skein's transport's, as the native shell
  holds its exchanges until the small store's write completes, failing
  them if it failed (`client.md`, section 10). The link's adapter hands
  the transport a step's link requests whole, and the sign-in part its
  provider's navigation, so a request leaves only once it is kept, and a
  sign-in by a provider leaves nothing unkept behind it. An application
  driving fetch and the small store without the transport takes the rule
  on itself, which skein's stories for a browser shell and jig's check
  (section 15).
- **No re-entry, and patches applied to the tree they are diffed from:**
  skein's inbox's and page part's (skein's `ui-web.md`, 5.3). jig's
  reference keeps its state in one cell, whose second borrow panics,
  fail-stop.
- **The rest of the order** (links' `href`s, title, timer and closing
  warning after the build) is the reference's (5.2): out of order, a
  turn shows something stale until the next, a closing warning among
  it; no request leaves unkept or twice.
- **Why not types.** A typestate that made the wrong order unwritable
  would make the invariants the shape of every application's loop, a
  framework by its types, and would not stop an application calling the
  platform itself; a part that holds its invariant asks nothing of the
  loop but to be used (skein's `ui-web.md`, 5.3). Documented order alone
  would leave an application's code ordering what jig promises, against
  `../README.md`, section 6.

## 6. The tree as elements

skein's (skein's `ui-web.md`, section 6): each node one element, by its
role, through the default mapping `skein-ui-web` holds, by which skein's
face also finds roles; names, states and properties as HTML and ARIA
give them; text written as text, never read as markup; a node's id kept
as `data-node`, its region name as `data-region`. jig's nodes take the
default table, with no override. What is jig's:

- **jig's pictures** are `<img>` elements whose source is under
  `/jig/client/<digest>/pictures/jig/`, an application's under
  `pictures/<its prefix>/` (4.2).
- **The sign-in part's dialog** for a credential is one of the parts'
  own elements, outside the tree, and carries no `data-node`
  (section 11).

### 6.1 An application's elements and components

skein's (skein's `ui-web.md`, 6.1; skein's `ui.md`, section 12): another
element for a role carrying one of the application's style tokens,
keeping the node's accessible role and name; or a component, the
browser's widget, for such a node or a drawing of one of its kinds,
given a host element in the node's place and reporting only the acts its
node's role takes. jig gives no element and no component, and jig's
nodes keep the default table: an application's override names one of
its own owners' tokens or kinds, never jig's. What a component builds
keeps to jig's policy (12.1): its styles from the origin's stylesheets,
since the policy refuses a `<style>` or `style` attribute it injects,
and its code from the origin (3.5, step 7).

## 7. Patches and acts

skein's (skein's `ui-web.md`, section 7): patches applied in place, so
focus, caret, selection, composition and scroll stay; acts reported with
their node's id, none on a disabled node; shortcuts; menus and
disclosures folded by the page part; a dialog shown modal; a log keeping
its end in view; going to another page; the title. What is jig's:

- **Escape in a dialog** presses the first bound, enabled node in the
  open dialog naming Escape as its shortcut (skein's `ui.md`, 6.1).
  jig's confirmation names its cancel button with it
  (`../client/domain/view.md`, 4.1), so Escape cancels a decision and
  never makes one. A dialog of an application's names its own, or none,
  and then Escape does nothing of its own.
- **The title,** as jig's reference sets it: the main's first level-1
  heading's text, then ` · ` and the application's name; the name alone
  when there is none. `<html lang>` is the language the client host's
  phrasebook chose (3.2).

## 8. Scenes as SVG

skein's (skein's `ui-web.md`, section 8; skein's `ui.md`, section 11):
the page part calls the drawing code with a drawing's kind, data and
box; draws the scene as inline SVG at the figure's top left; marks the
figure `data-drawn` while it is drawn, and a mark that carries an act
`data-act`; keeps the fallback for assistive technology; and reports a
press as `Drawing`, in skein's press order. What is jig's:

- **jig draws no kind of its own** (section 16): the drawing code is the
  application's view root's `draw`, routing a drawing by its kind's
  owner (3.3).
- **A scene's colours, widths and fonts** are the stylesheets':
  `skein-ui.css`'s defaults, then the application's, which may use jig's
  palette's series colours (section 13).

## 9. Addresses and history

- **The application's paths:** its client host parses and prints every
  path the page shows, with its own codec, below the domain: total,
  bounded step code, since a URL is a person's input (skein's
  `ui-web.md`, section 9).
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
  the bundle under its prefix, `/jig/client/`. Fixed; no base path
  (section 19).
- **Links' `href`s and history** are skein's parts' (skein's
  `ui-web.md`, section 9): a link node's binding resolves, without
  acting, to the address it leads to (the view root asks the binding's
  owner's view, jig's or the application's), which the client host
  prints as a path, and after each apply the page part sets each `href`
  whose path changed; an address a step shows is a new entry, or
  replaces the current one, as the step says; back and forward are a
  place gone to, as a path; a path opened directly (a bookmark, an
  address typed, a link opened in a new tab) starts a client at it.

## 10. Transport and the small store

skein's fetch, small store and transport carry jig's link (skein's
`ui-web.md`, section 10); the link's adapter gives them jig's wire as
their plain data (5.1).

- **Exchanges** by skein's fetch: a POST of the wire's bytes to the path
  the link gives, `Content-Type: application/octet-stream`, the browser
  header (section 11), same-origin credentials; abandoned by aborting. A
  body is read up to the link's largest record; past it, the exchange
  failed.
- **The stream** by skein's fetch, with same-origin credentials, its
  body read a piece at a time: each piece, cut to the link's stream
  piece, handed to the link as it comes, the next read asked by the loop
  after the step (5.2), so the platform paces it (`client.md`,
  section 7).
- **The transport,** skein's, takes a step's link requests whole, as the
  link's adapter gives them: the small store's writes first, then the
  exchanges and the stream, then the navigation away, if any. When a
  write fails, every exchange of the step is reported failed, none is
  sent, and there is no navigation (`client.md`, section 10).
- **A hidden tab lets its stream go** a minute after it is hidden,
  skein's default, which jig's reference keeps (the application's
  parameter): fetch closes the stream and reports its end, which the
  link takes as any drop, and an opening while hidden waits until the
  tab is shown (skein's `ui-web.md`, section 10).
- **The small store** is skein's, the tab's session storage
  (`../client/domain/client.md`, section 11): each record under its key,
  both as base64 text, the key under the prefix jig's reference gives,
  `jig.`; read whole at start. A write the platform refuses, its quota
  full, is a failed write. The restart part's key is outside the prefix
  (section 5).
- **Closing a tab with requests pending** asks the platform to warn
  (`beforeunload`), through skein's title and closing part, only while
  the client host says it would lose them.

## 11. Signing in and out

The sign-in part is jig's (5.1). Its exchanges leave through skein's
fetch with the browser header, as the link's adapter's do; its
provider's navigation through skein's transport, after the step's writes
(5.3); its dialog for a credential is its own element, outside the tree.

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
  `/jig/v1/sign-in/provider/<provider>?back=<path>`, which skein's
  transport makes after the step's writes (5.3):
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
  the sign-out lost (`../client/domain/client.md`, section 12). A
  sign-in always loads a new page (a provider's navigation, or the
  printed link, signed in before the client host starts: section 5), so
  an earlier page's sign-out never answers into the client host of a
  newer sign-in (`../client/domain/client.md`, section 12).
- **A service's credential, shown once,** in the sign-in part's dialog
  of its own, outside the view's tree: selected text, and a button
  closing it, in jig's words. The part never keeps it.

## 12. Served by the engine

`jig-protocol-engine-clients` serves the browser's client from the
engine's one origin (`client.md`, section 4), as skein's `ui-web.md`,
4.4, says a bundle is served, behind jig's front door. What it serves is
data; how it answers is fixed.

- **What it serves is data** the application's startup gives the
  component at start, with no traits: the bundle (its page and its
  digest directory), the root files and the policy's additions (12.1),
  counted in its worst case; or none, which serves no browser. `jig-test`
  reads the bundle's directory its configuration names
  (`configuration.md`, section 3, clients; skein's `shell.md`, 6.1); an
  application may also build or embed it.
- **The bundle's files** under the bundle's prefix and its digest
  directory, `/jig/client/<digest>/<file>`, by their paths in the
  bundle, from memory. Each file's type comes from the bundle's closed
  list, by its extension (skein's `ui-web.md`, 4.4); any other file
  given refuses the start.
- **A root file,** a path outside `/jig/` the startup names, answers one
  of the bundle's files (3.5, step 5); **the page,** every GET of any
  other path outside `/jig/`, and of the link's route. Under `/jig/`, a
  GET of anything but a file of the digest directory, the stream, the
  link's route, or a provider's sign-in route and its redirect back
  (section 11) is not found.
- **Caching,** by the digest directory (skein's `ui-web.md`, 4.4): each
  file under `/jig/client/<digest>/` with `Cache-Control: public,
  max-age=31536000, immutable`; the page and the root files with an
  `ETag`, a digest of their bytes computed at start, and
  `Cache-Control: no-cache`, a matching `If-None-Match` answered `304`;
  under `/jig/client/`, any other digest not found. A repeat load
  revalidates the page alone, and a page always loads one coherent
  bundle. The API's answers carry `Cache-Control: no-store`.
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

- **Its floor,** fixed (section 19), skein's strict policy, under which
  skein's parts and the bundle run (skein's `ui-web.md`, 4.4), closed
  further: `Content-Security-Policy: default-src 'none'; script-src
  'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; font-src
  'self'; connect-src 'self'; base-uri 'none'; form-action 'none';
  frame-ancestors 'none'`, and `X-Content-Type-Options: nosniff`,
  `Cross-Origin-Opener-Policy: same-origin`,
  `Cross-Origin-Resource-Policy: same-origin`. `script-src` and
  `style-src` take nothing more: no other source, no `'unsafe-inline'`,
  no `'unsafe-eval'`, no nonce or hash. `object-src` is never added.
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

- **`skein-ui.css`,** which skein's bundler writes first in every
  bundle, holds the generic rules (skein's `ui-web.md`, section 12): a
  drawn drawing's fallback hidden from sight only, taking no space; a
  scene's SVG at its figure's top left; a mark's default, and a mark
  carrying an act (`data-act`) taking presses over its whole area; focus
  outlines kept visible; the platform's light or dark preference and
  reduced motion followed. jig has no class of its own for acts.
- **Every style token and region name has a name** (skein's `ui.md`,
  section 8). jig's start with `jig-`, its prefix and a hyphen: its
  tokens `jig-card`, `jig-chip`, `jig-muted`, `jig-warning`,
  `jig-danger`, `jig-success`, `jig-button-weight`, `jig-dense-list`,
  `jig-provisional` (`../client/domain/view.md`, section 13); its
  regions as `../client/domain/pages.md`, 12.1 and 12.2, list them.
  An application's start with its owners' prefixes, given by its view
  root, which its shell gives the parts as the look (3.3).
- **jig's stylesheet,** `jig.css`, ships with `jig-shell-web` and loads
  after `skein-ui.css`. It places jig's regions by `data-region`, and
  the landmarks of jig's default arrangement: the banner at the top; the
  navigation beside the main on a wide screen and above it on a narrow
  one; asides beside the main; the composer pinned below its
  conversation. It styles jig's tokens, in jig's palette, light or dark
  as the platform prefers.
- **jig's palette:** custom properties an application's stylesheet uses:
  `--jig-text`, `--jig-background`, `--jig-muted`, `--jig-accent`,
  `--jig-warning`, `--jig-danger`, `--jig-success`, and six series
  colours for drawings.
- **The application's stylesheet** loads after jig's, styles its own
  tokens and regions, and may restyle jig's (its own page: 3.5, step 4).
- **No style is a fact** (skein's `ui.md`, section 8;
  `../client/domain/view.md`, section 13).

## 14. Limits

- **The link's:** an exchange's body, at most its largest record; a piece
  of the stream; exchanges in flight (`client.md`, section 13).
- **The tree's:** `skein_ui::Limits` (skein's `ui.md`, section 15),
  whose numbers jig's constructors derive from the domain's limits
  (`../client/domain/view.md`, section 10); a drawing's SVG has at most
  a scene's marks (skein's `ui-web.md`, section 13).
- **The platform's:** session storage's quota, past which a write fails.
- **The parts' own,** skein's (skein's `ui-web.md`, section 13): one
  timer; one read of the stream outstanding; the inbox's reports raised
  during a turn, only those the turn's own calls raise; three reloads in
  a minute, by default, which jig's reference keeps.
- **The bundle's size,** a guide (4.2).

## 15. Testing

Each within jig's suites' budgets (`../../development/workflow.md`).
skein's parts, its bundler, its face and its harness are tested in
skein, and in a browser on skein's UI example (skein's `ui-web.md`,
section 14); jig's tests take what is jig's.

- **On the host,** in jig's default suites: the link's adapter's pure
  half (a link request to the transport's plain data: the POST, its
  type, the browser header, the bound on its answer; fetch's reports to
  link events, an answer past the bound failed); the sign-in part's
  pure half (the secret read from the address, the address replaced
  first, the credential's dialog's words); jig's default codec (every
  place printed and parsed back, at the root, under a prefix, renamed;
  any bytes parsed to a place, never a panic).
- **The wasm32 builds in jig's gate:** `jig-shell-web`,
  `jig-test-shell-web` and every client crate, built for
  `wasm32-unknown-unknown` with clippy's lints as errors, since the
  parts' platform halves are compiled only there; skein's gate builds
  skein's (section 18).
- **The browser tier:** a suite of its own, `--profile browser`, run by
  jig's gate after it builds jig's test bundle with skein's bundler,
  within a minute on the slowest machine (a rule: its profile's global
  timeout, in `../../development/workflow.md`). Each test starts one
  engine (the application's binary; for jig, the reference `jig-test`)
  and one Chromium, and plays its stories with a person each: a browser
  context, its own cookies and storage. A missing Chromium or bundle
  fails the suite, naming the command that provides it; it is never
  skipped. The bundle's directory reaches the tests in
  `JIG_TEST_BUNDLE`.
- **The harness and the face** are skein's, in
  `skein-fake-person-browser` (skein's `ui-web.md`, 14.2; skein's
  `ui.md`, 17.1): the binary and Chromium as children of one loop; the
  person's `Find` by role, name and text; press, type, Enter, go to a
  path, reload; trouble the story did not expect failing it; the
  snapshot, a screenshot, and Chromium's and the binary's stderr's tails
  on a failure; a drawing's mark pressed at its point; the binary
  stopped and started again within a story. jig's tier gives the
  harness the prefix of the line `jig-test` prints its link on, so the
  address it opens first is the printed link, which signs the person in
  (section 11); the face goes to the paths the application's codec
  prints.
- **skein's stories for a browser shell** (skein's `ui-web.md`, 14.2):
  typing while a patch arrives keeps focus, caret and words; back and
  forward move between pages, and an address opened directly shows its
  page; a panic reloads within the restart limit; a write the small
  store refuses fails the step's exchanges. jig's tier calls them with
  jig's binary, bundle and codec.
- **jig's stories for a browser shell,** in `jig-fake-person-browser`,
  on skein's face and harness: functions an application's tier calls
  with its binary, bundle and codec, its pages including jig's, to check
  its wiring of jig's link and sign-in, since its loop is its own. Each
  signs its person in by the printed link:
  - a reload keeps a pending request, which the engine makes once;
  - the engine killed and restarted shows offline, then live;
  - a spent link is refused; the sign-in page says to open the new one.
- **jig's tier** calls both, and adds: a person opens the printed link
  and sees their inbox, under the policy, with no trouble.
- **jig's test application's shell takes jig's steps of 3.5 once each:**
  its own page or skein's bundler's (the plan's choice), under the
  bundle's prefix; a root file; a prefix on jig's codec in one story; a
  policy addition refused at start, in a host test. An element
  override, a component and a JavaScript snippet are skein's example's
  (skein's `ui-web.md`, 14.3); jig's test application carries no snippet
  by default (section 17).
- **What an application adds:** its tier, over skein's harness and face:
  skein's stories and jig's, called; its pages, drawings, components,
  sign-ins.

## 16. What is not the layer's

- **Following data as it comes** (a log tailed, a metric's latest points)
  is an application's streams (`../client/domain/contract.md`, 7.4): a
  contract and engine piece, carried by skein's parts and jig's link as
  any watch, with no change to them. It waits for an application whose
  people follow its data as it comes.
- **Graphics beyond a scene** (a canvas of hundreds of thousands of
  points, three dimensions, video) are out of jig's scope: a component
  of the application's in a browser (6.1; 3.5, step 6), its own shell
  elsewhere (skein's `ui.md`, section 12; `../client/domain/view.md`,
  section 11).
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
    that shows only there, and comes to skein's harness first (skein's
    `browser.md`, section 11; skein's `ui-web.md`, section 16); *a press
    that copies text, downloads and uploads of files:* effects (skein's
    `ui.md`, section 13), their parts skein's (skein's `ui-web.md`,
    sections 11 and 15), for an application whose people need them;
  - *`ops`'s browser shell,* `jig-ops-shell-web`: comes with `ops`
    standalone (`../examples.md`, 9.1), which gives it an engine.

## 17. Open questions

- **Right-to-left languages:** what they ask of the elements and scenes
  waits in skein's vocabulary (skein's `ui.md`, 4.2); what they ask of
  `jig.css` and jig's phrasebook, beyond `<html lang>`, waits for an
  application in such a language (`../client/domain/view.md`,
  section 14).
- **Media from the origin:** the bundle's closed list (skein's
  `ui-web.md`, 4.4) has no audio or video type; one waits for media in
  skein's vocabulary (skein's `ui.md`, 4.2) and an application serving
  its own.
- **A JavaScript snippet in jig's test application,** the user's
  decision; by default, none. skein's example carries the snippet that
  checks that the bundler and the strict policy carry an application's
  JavaScript (skein's `ui-web.md`, 14.3). One in jig's would check the
  same under jig's floor and additions, at the cost of a component and a
  snippet jig's test application has no other use for.

## 18. What it owes and is owed

- **The view** (`../client/domain/view.md`) owes: jig's part of the
  look, the names of jig's style tokens and region names under jig's
  prefix (section 13), and jig's owner's number as skein-ui's opaque
  owner, by jig's convention (section 3); a link node's address,
  without acting (section 5); the parts' own phrases among jig's
  (section 7); jig's confirmation's cancel button naming Escape as its
  shortcut (4.1), which Escape in a dialog presses (section 7 here).
- **`jig-protocol-client-link`** owes jig's default codec: its `address`
  module, parsing and printing jig's places, total and bounded, at the
  root, under a prefix, or renamed segment by segment (section 9); and
  its requests as plain data, which the link's adapter turns into the
  transport's (5.1).
- **`jig-protocol-engine-clients`** owes: what the startup gives, from
  memory; the page at every other path outside `/jig/` and at the link's
  route; the bundle's types from its closed list, and the caching by the
  digest directory under `/jig/client/`; the front door, with the
  policy's additions 12.1 admits and a start refused for any other; the
  sign-in cookie, set and cleared; providers' sign-ins by navigation,
  ending in a `303` to a checked `back` (sections 11, 12 and 12.1).
- **`configuration.md`,** section 3, owes the clients' member the
  bundle's directory, or none, which the reference startup reads.
- **An application's browser shell** is owed skein's parts, jig's, and
  jig's reference to copy; driving fetch and the small store without the
  transport, it owes the transport's order (5.3).
- **skein owes:**
  - `skein-ui`, which jig's view builds on: the tree (skein's `ui.md`,
    section 3), owners as opaque numbers (skein's `ui.md`, 3.3), the
    look (skein's `ui.md`, section 8), and Escape as a dialog's shortcut
    (skein's `ui.md`, 6.1);
  - `skein-ui-text`, the text shell jig's terminal client composes
    (skein's `ui-text.md`);
  - `skein-ui-web`: the parts and the default mapping (skein's
    `ui-web.md`, 5.1 and section 6); `wasm-bindgen`'s start attribute
    re-exported and the glue pinned (skein's `ui-web.md`, 3.1 and 4.1);
    fetch taking the headers, the body and the bound on an answer an
    application gives; the small store's prefix, the restart part's
    limit and the hidden tab's delay as the application's parameters;
  - `skein-ui-bundle`: the bundle under a prefix the application gives,
    its given stylesheets after `skein-ui.css` in the order given, each
    owner's pictures under its prefix, an application's page and files,
    and the closed list of types (skein's `ui-web.md`, 4.2 to 4.4);
  - `skein-fake-person` and its browser face and harness,
    `skein-fake-person-browser`, with skein's stories for a browser
    shell (skein's `ui.md`, 17.1; skein's `ui-web.md`, 14.2): the
    harness opening an address the binary prints, found by a prefix the
    test gives, and stopping the binary and starting it again within a
    story;
  - the wasm32 builds: lib, `skein-codec`, `skein-http`, `skein-ui` and
    `skein-ui-web` built for wasm32 and checked by its gate (skein's
    `lib.md`, section 2; skein's `testing.md`, section 6); a client
    compiled to WebAssembly in its model, with no kernel and no ring,
    the glue crates its only `unsafe`, a panic a trap the restart part
    reloads from (`programming-model.md`, 2.1).
- **smith owes** `smith-transcript` building for wasm32, which jig's
  gate checks through `jig-protocol-transcript`.

## 19. What stays fixed, and why

Each rule in the table is jig's; no value an application gives removes
it. The engine holds those of its front door and its serving
(section 12); jig's parts hold the sign-in's and the link's (sections 10
and 11); an application that replaces a part (3.5, step 9) holds its
rule itself. One application per origin is each deployment's to keep.
The rules skein's parts and bundle hold are skein's, with their reasons
(skein's `ui-web.md`, section 17): text never read as markup; scripts
and styles from the origin only, nothing inline; the bundle's closed
list of types, which the engine serves (section 12); an element override
keeping its node's accessible role and name (6.1); the transport's and
the inbox's invariants (5.3), on which jig's link's "once" rests (the
table's last row).

| Fixed | Why | Instead |
|---|---|---|
| the policy's floor (12.1) | A script that runs on the page runs as the person: the cookie is HTTP-only, but the script can send any exchange with the browser header, so code from anywhere but the origin's bundle, or inline, is the person's account. A frame of another site over the page could trick a press on a decision (accepting a restart, a hold). Styles injected inline, with fonts or images from another origin, would read the page's text out (by attribute selectors, by a font's ligatures), a credential shown once with it. | additions for data and other sites' pages (12.1); its JavaScript, from the origin (3.5, step 7); a component (6.1) |
| the `Host` check (section 12) | DNS rebinding: a page of another site, its name rebound to the loopback address, would reach the engine as its own origin; answered `421`, it reaches nothing. | its listener, and between hosts its certificate's name, in its configuration (`configuration.md`, section 3) |
| the cookie's attributes and the browser header (section 11) | Cross-site requests: a page of another site can neither send the cookie (same-site strict) nor set the header (it would need a preflight the engine never answers), so it cannot act as the person. HTTP-only keeps the secret from every script, the module's too; the `Path`, from every request but the API's. | its own provider (`client.md`, section 8), which ends with the same cookie |
| one application per origin | Session storage, `'self'` in the policy and the cookie are per origin: two applications on one origin would share the small store's storage, each other's scripts and the sign-in, and either could act as the person in the other. Hence no base path. | an origin each, by host name or port |
| `/jig/`, the one reserved first segment (section 9) | The link, the native shell, the engine's routes, the printed link, the bundle's prefix, the cookie's `Path` and the `back` check all name it; a setting would let them drift apart, a cookie not covering the API or a `back` reaching it. It costs an application one first segment. Recorded, not taken: the prefix as an application's constant, which would break nothing kept, since a printed link is short-lived and no bookmark names the API. | every other path, its own (section 9) |
| the checked `back` (section 11) | A `back` naming another site, or read as one by a browser (`//`, `/\`), would send the person, just signed in, to a page passing for the application's; one under `/jig/`, into the engine's routes rather than a page. | any path of its own, printed by its codec |
| the referrer policy (12.1) | The page opened at the link's route holds the sign-in's secret in its address until the sign-in part replaces it; a policy sending a path to another origin would send the secret with a first request to an addition's origin. | `strict-origin`, the origin alone |
| the credential never kept (section 11) | A service's credential acts as the service; kept in a tab, it would outlive its dialog, in storage any script of the origin reads and in the session's restore. | the person copies it from the dialog, which the application's stylesheet may restyle |
| a step's link requests through the transport (5.3) | A request that leaves before it is kept is lost or made twice after a reload. It is the client's "once" (`client.md`, section 10), which jig owns (`../README.md`, section 6), and which skein's transport holds for any application (skein's `ui-web.md`, section 17). | its own order for the rest of the turn (5.2); the transport replaced, the order taken with it and checked by skein's stories for a browser shell and jig's (section 15) |
