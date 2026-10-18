# The browser's layer

Provisional, 2026-10-14, revised 2026-10-18. What jig gives a browser
client, and where it stops: the application's server serves its pages
and its bundle by skein's serving contract (skein's `ui-web.md`, 4.4),
and jig serves no page, no bundle and reserves no path. jig gives the
default HTTP mapping's browser end over skein's browser parts, in
`jig-client-shell-web`, with the cookie that carries a browser's
credential; example guards in the mapping's engine end,
`jig-protocol-engine-clients-http`, which an application uses, wraps,
or writes its own in place of; attack stories in its test kit, checked,
never mandated; and jig's test application's browser shell,
`jig-client-test-shell-web`,
with its server and its page's policy, as the example of an
application's. Section 19 says what jig promises and where its promise
stops. jig's messages, the clients' component and the mapping's routes
and framing are `client.md`'s; the tree the parts map is skein's
(skein's `ui.md`); the reference UI is `../client/domain/view.md`'s and
`../client/domain/pages.md`'s. What is still open is listed in section
17, and what is not the layer's, or comes later, in section 16.

## 1. In one page

- **The server and its URL space are the application's.** Its server
  serves its page and its bundle as skein's `ui-web.md`, 4.4 says, its
  own routes and its page's policy, and mounts jig's clients' component
  wherever it chooses (`service.md`, section 2). jig serves no page and
  no bundle, and reserves no path (sections 9 and 12).
- **The default HTTP mapping's browser end** is jig's, optional:
  `jig-client-shell-web`'s client adapter carries `jig-client`'s
  exchanges, its stream and its small store's records over skein's
  fetch, transport and small store, as `jig-client-protocol-http` maps
  them onto HTTP, adding the browser header to each request (5.1;
  section 10; `client.md`, section 17).
- **A browser's credential is a cookie** the mapping's engine end sets,
  HTTP-only, so the client and the module never see it (section 11; its
  attributes, `client.md`, 17.3).
- **Example guards,** in `jig-protocol-engine-clients-http`: a Host
  allow-list; a required header that forces a preflight, with no CORS;
  the cookie's attributes; a referrer policy on the printed link's route.
  An application uses them, wraps them, or writes its own and takes over
  what they held (12.2).
- **The page's policy is the application's page's** (12.1): skein's
  strict policy is what its parts need; jig's test application's server
  sends a stricter one, as an example.
- **Attack stories** in jig's test kit (a cross-site request, a rebound
  Host, a framed page), which an application's browser tier may run
  against its own server: checked, never mandated (section 15).
- **The printed link, opened in a browser,** is answered by the
  application's page at the mapping's link route; the shell reads its
  secret, takes it out of the address, and signs in (section 11).
- **The reference:** jig's test application's browser shell,
  `jig-client-test-shell-web`, over skein's parts, jig's and the
  reference UI, with its server in `jig-test-service` and its page's
  policy: the example of an application's server, page policy and shell
  (3.1; section 12).
- **The rest is skein's:** the loop, the tree as elements, patches,
  scenes, history, the toolchain and the bundle, with what the reference
  UI adds to each (sections 4 to 9).
- **jig's promise stops at the credential** (section 19): a request
  presented with a valid credential is its party's, and everything after
  is checked below the API. That a credential is not used across sites,
  and that a deciding act is confirmed with its facts shown, are the
  application's.

## 2. Where it sits

```
a person, a screen reader
  ▲ elements, focus, keys, the address bar, back and forward
the browser: fetch, session storage, history, cookies, clock, date, entropy
  ▲ the glue: wasm-bindgen, js-sys, web-sys
skein-ui-web                      skein's browser parts (skein's ui-web.md, 5.1)
jig-client-shell-web              jig's: the client's adapter, the printed link's sign-in, jig's look (5.1)
  ▲ plain data: reports, patches, the transport's requests, paths, the look
<application>-client-shell-web    its start and its loop: the turn (5.2)
  ▲ its client host's functions, called directly
its client host                   its client root holding jig_client::Client, its presentation, the mapping's client end (3.2)
  ▲ the client's requests as HTTP: exchanges, one stream; the small store's records
════════════════════════════════ one origin ═════════════════════════════════
its server                        its page, its bundle, its routes, its page's policy (section 12)
  ▲ the requests under the prefix it mounts jig's mapping at
jig-protocol-engine-clients-http  the default mapping's engine end, with the example guards (12.2)
  ▲ jig's messages, each with its credential as opaque bytes
jig-protocol-engine-clients       the clients' component (client.md)
```

- **A person** reads the page by sight or with a screen reader, and acts
  on it with a pointer and keys. **The browser** is any current one,
  which the parts target; the browser tier drives Chromium (section 15).
- **`skein-ui-web`,** skein's crate: the browser parts and the default
  mapping of roles to elements, knowing no application (skein's
  `ui-web.md`, section 2). **`jig-client-shell-web`,** jig's crate,
  ordinary Rust, forbidding `unsafe`: jig's two parts and jig's look,
  over skein's parts, knowing no application. Each of its parts is
  usable alone: an application with its own presentation may take the
  client's adapter and nothing else.
- **The application's browser shell,** compiled to the module, is its
  start and its loop (3.1, section 5), with no trait between it and the
  parts or its client host. **Its client host** is its client root,
  which holds `jig-client` as a component, its presentation and its
  transport's client end, stepped as one machine (3.2).
- **Its server** is the application's: skein's HTTP server over its
  clients' listener, or any other, serving the bundle by skein's
  contract and routing every request (section 12). Under the prefix it
  mounts jig's mapping at, it hands each request to
  `jig-protocol-engine-clients-http`, which speaks jig's messages to the
  clients' component, `jig-protocol-engine-clients`, which knows no HTTP
  (`client.md`, section 2; `service.md`, section 2).

## 3. What an application writes

Its presentation, its browser shell and its server. Its extension points
in the tree are skein's (skein's `ui.md`): whatever tree it builds with
`skein-ui`, with the reference UI's primitives where it likes, or none
of them (`../client/domain/view.md`). In a browser it writes its browser
shell (3.1), copied from a reference, its client host (3.2), its look
(3.3), its stylesheets, pictures and phrasebooks (3.4), and its server
(section 12); what more it may change is 3.5.

### 3.1 Its browser shell

- **`<application>-client-shell-web`,** its own crate, ordinary Rust, as
  shells are (skein's `ui-web.md`, 3.1). Its start, through
  `wasm-bindgen`'s start attribute, which `skein-ui-web` re-exports so
  its version is skein's pin, makes skein's parts and those of jig's it
  takes, with the look (3.3), and starts its client host (section 5);
  its loop is the turn (5.2). It orders the platform's work and decides
  nothing: what to show, send and keep is its client host's; carrying
  a request safely is a part's (5.3).
- **jig's reference shell,** `jig-client-test-shell-web`, jig's test
  application's, is written after skein's reference shell (the UI
  example's `skein-ui-example-shell-web`: skein's `ui-web.md`, 3.1;
  skein's `examples.md`, section 9), and adds `jig-client` through the
  client's adapter, the printed link's sign-in, and the reference UI
  with jig's look. It is written to be copied and commented as such:
  about 200 lines beside its client host (a guide).
- **Which to copy:** an application that takes the reference UI copies
  jig's; one with its own presentation may copy skein's and add the
  client's adapter alone (3.5, step 1).
- **The example of an application's three:** with its server in
  `jig-test-service` (section 12; `service.md`, section 7) and its
  page's policy (12.1), jig's reference shows a server, a page policy
  and a shell an application may copy, each of which it may write
  otherwise.

### 3.2 The client host

- **The application's:** its client root, `<application>-client-domain`,
  which holds `jig_client::Client` as a component beside its own
  children (`../client/domain/root.md`); its presentation, the reference
  UI, its own view root, or both (`../client/domain/view.md`); and its
  transport's client end, jig's `jig-client-protocol-http` in the
  reference (`client.md`, 17.4). They step as one machine, what skein's
  `ui-web.md`, 3.1 calls the application's state machine. The client
  runs no loop of its own and never calls the application: the client
  root steps it with the events the shell's reports carry, and its
  transport requests, mapped onto HTTP, are among the host's step's
  duties.
- **Shared by every shell** the application uses: its terminal, over
  skein's text shell (`../client/domain/view.md`, section 15), runs the
  same host; a native shell steps the same client.
- **Its functions,** which the browser shell calls, are skein's
  reference's (skein's `ui-web.md`, 3.1): *start,* *step,* *build,* the
  last tree, the path a link node leads to without acting, the next
  deadline, whether closing the tab would lose requests, and the
  language its phrasebook chose. jig's reference adds:
  - to *start,* the printed link's secret, when the page opened at the
    link's route (section 11), with which the host signs the client in
    by the link, and otherwise by the transport's cookie;
  - to *step*'s duties, the client's transport requests as HTTP, in the
    order the client gave them, and a service's credential to show once
    (section 11);
  - to the next deadline, the client's, and to the closing question, the
    client's pending requests.
- **jig's test application's client host,** its client root
  `jig-client-test-domain`, its view root `jig-client-test-view` with
  the reference UI, and `jig-client-protocol-http`, is the one jig's
  reference shell drives, and is copied with it
  (`../client/domain/root.md`, section 12).
- **Effects** (skein's `ui.md`, section 13) are the application's: its
  client host gives those its presentation asks with a step's duties,
  and steps with each report; the reference UI asks none.

### 3.3 The look

skein's (skein's `ui-web.md`, 3.2; skein's `ui.md`, section 8), as plain
data the application's browser shell gives skein's page part when it
makes it. With the reference UI, jig's part of it, which
`jig-client-shell-web` gives whole:

- **jig's prefix and names:** the prefix `jig` and the names of jig's
  style tokens and region names (section 13), from `jig-client-view`,
  under jig's owner, whose number as skein-ui's opaque owner is jig's
  convention (`../client/domain/root.md`, section 15);
- **jig's pictures,** which ship with `jig-client-shell-web`, under the
  bundle's `pictures/jig/` (4.2);
- **jig's phrases** for the parts' own words (section 5).

The application's browser shell adds its own owners' prefixes, names
and pictures, its drawing code (its client host's `draw`, routing a
drawing by its kind's owner: section 8; skein's `ui.md`, 11.1), and its
elements and components (6.1), and gives the page part the whole. An
application without the reference UI gives its own alone. **Its name**
is in the title the shell sets (section 7) and in the bundle's file
names (4.2).

### 3.4 Its stylesheet, pictures and phrasebooks

- **Its stylesheet,** CSS, and **its pictures' directory** go to skein's
  bundler with its module (section 4); its stylesheet loads after
  `skein-ui.css`, and after `jig.css` where it takes the reference UI
  (section 13), or where its own page puts it (3.5, step 4).
- **Its phrasebooks are in its module,** data its presentation takes
  (skein's `ui.md`, section 9; `../client/domain/view.md`, section 7).

### 3.5 What it may change

skein's steps (skein's `ui-web.md`, 3.3), with what is jig's in each,
and two of jig's: its page's policy (step 8) and its transport (step
10); each needing one before it only where it says.

1. **Copy a reference shell** and change its names: jig's, with the
   reference UI; or skein's, with `jig-client-shell-web`'s client
   adapter added (3.1).
2. **Restyle:** its stylesheet after `jig.css`, which may restyle jig's
   tokens and regions (section 13); its pictures (4.2).
3. **Its paths,** wholly its codec's: with the reference UI, its codec
   for jig's places as is, under a prefix of its own, with its own
   segment names, or its own (section 9).
4. **Its page:** its own `index.html`, copied by skein's bundler with
   its marker (skein's `ui-web.md`, 4.3), loading the starter by a
   script file, with nothing inline: the policy refuses at run what
   breaks that (12.1), and the browser tier fails on any violation.
5. **Its files:** skein's (skein's `ui-web.md`, 3.3, step 5), served by
   its server, root files among them (section 12).
6. **Its elements and components** (6.1): another element for a role
   carrying one of its style tokens; a component for such a node or a
   drawing of its kinds.
7. **Its JavaScript,** from the origin only: `wasm-bindgen`'s snippets,
   its own modules the glue imports, or files its page loads (step 4);
   never from another origin, never inline (skein's `ui-web.md`,
   section 17).
8. **Its page's policy,** its own above skein's strict policy (12.1):
   named origins for its data and other sites' pages, never a source of
   code or style beyond the origin.
9. **A part replaced or left out:** any of skein's or jig's. One that
   holds an invariant (the transport, the inbox, the page) moves it to
   the application (5.3), which skein's stories for a browser shell
   check; one that carries jig's client (the client's adapter, fetch,
   the sign-in part) leaves the mapping's requests, the browser header
   and the printed link to the application (sections 10 and 11), which
   jig's stories for a browser shell check (section 15). There is no
   shell to write whole: the application's browser shell is its own.
10. **Its transport:** jig's default mapping as it is, its guards
    wrapped (a check of its own added before them, a header added to
    its answers), or its own transport between `jig-client` and the
    clients' component at both ends, which takes over what the guards
    held (12.2) and which jig's attack stories check, if it runs them
    (section 15).

## 4. The toolchain and the bundle

skein's (skein's `ui-web.md`, section 4): the toolchain, skein's
bundler, `skein-ui-bundle`, the bundle, its page, its digest directory
and the starter. What is jig's follows.

### 4.1 The toolchain

- **skein's** (skein's `ui-web.md`, 4.1): `wasm32-unknown-unknown`,
  stable Rust, and the glue crates pinned to exact versions in skein's
  workspace, the only `unsafe` the module runs
  (`programming-model.md`, 2.1); no wasm-pack, no npm, no binaryen.
  `jig-client-shell-web` forbids `unsafe`, as `skein-ui-web` does; an
  application's browser shell writes none of its own (3.5, step 7).
- **One lock:** jig's workspace builds skein's bundler beside its test
  application's module, so the glue's generator is locked with the
  macros' version in jig's `Cargo.lock`; an application's workspace does
  the same in its own.
- **jig's release profile** is `web`: it inherits `release`, with
  `opt-level = "s"`, `lto = "fat"`, `codegen-units = 1`,
  `panic = "abort"`, `overflow-checks = true` and `strip = true`. An
  application's workspace defines its own the same way.
- **The browser tier's module** is built with the dev profile, as code
  under test; the bundler is a tool, built optimized
  (`../../development/workflow.md`, section 2).

### 4.2 The bundler and the bundle

- **skein's bundler writes the bundle whole** (skein's `ui-web.md`,
  4.2). An application's build gives it its bundle's prefix, its own
  (section 9), and, where it takes the reference UI:
  - **`jig.css`,** jig's stylesheet, which ships with
    `jig-client-shell-web`, first of the given stylesheets, then the
    application's, `<name>.css` (section 13);
  - **jig's pictures,** which ship with `jig-client-shell-web`, as the
    owner `jig`'s, beside each of the application's owners' under its
    prefix, so no two owners' clash.
- **jig's test application's bundle,** under its prefix, `/bundle/`, as
  skein's example's is: `index.html`, and under `/bundle/<digest>/`
  `boot.js`, `<name>.js`, `<name>_bg.wasm`, `snippets/`,
  `skein-ui.css`, `jig.css`, `<name>.css`, `pictures/jig/`, a
  `pictures/<prefix>/` for its test family's owner, and `files/`.
- **jig's one text that is not Rust** is `jig.css`. Nothing inline.
- **Its size is a guide:** jig's test application's module, built with
  `web`, kept with jig's budgets (`../../development/workflow.md`,
  section 2); past it, what grew is reported.

### 4.3 The page and the starter

skein's (skein's `ui-web.md`, 4.3). jig's test application's page is
skein's bundler's, which names the digest directory by absolute path,
`/bundle/<digest>/`, since its server answers it at every path it does
not reserve, and at the printed link's route (section 12); it loads
`skein-ui.css`, `jig.css` and then its own stylesheet, and its title is
the application's name.

## 5. The loop, the application's

The application's browser shell runs it over skein's parts and jig's
(5.1), in the reference's order (5.2); the parts hold what protects
requests (5.3). Starting and a panic's restart are skein's (skein's
`ui-web.md`, section 5), with jig's reference's here:

- **Starting:** the starter loads the glue, the glue the module, whose
  start makes the parts and reads through them, once, the clock, the
  date and what its client host's start takes (3.2). At the printed
  link's route, the sign-in part first reads the secret and takes it out
  of the address (section 11); then the shell starts its client host,
  which starts the client with the small store's records and signs it
  in, builds, and applies the first patches, inserting the whole tree
  under the mount element.
- **A panic is fail-stop:** skein's restart part reloads the page, which
  starts the application again from its small store, at most three
  times in a minute (skein's default, which jig's reference keeps),
  counted under a session storage key of its own, outside the small
  store's prefix. Past it, the page says the application failed, starts
  nothing, and keeps the small store.
- **The parts' own words** (a credential shown once, the application
  failed) are jig's phrases where the application takes the reference
  UI (`../client/domain/view.md`, section 7), part of jig's look, and
  its own otherwise.

### 5.1 The parts

skein's (skein's `ui-web.md`, 5.1), in `skein-ui-web`, and jig's two, in
`jig-client-shell-web`. Each is ordinary Rust, usable alone, taking and
giving plain data; jig's are built over skein's fetch, small store and
transport, and each is usable without the other.

- **skein's parts,** with what jig's reference gives each:

  | Part | What jig's reference gives it |
  |---|---|
  | the inbox | the turn, and its report type (below) |
  | the page | the mount element, the look (3.3) and the default mapping, which the reference UI's nodes keep (section 6) |
  | scenes as SVG | the look's names, for the application's drawings (section 8) |
  | fetch | the client's exchanges and its stream, as the client's adapter gives them (section 10) |
  | the small store | the prefix `jig.` (section 10) |
  | the transport | a step's writes, its exchanges and stream through the client's adapter, and a navigation to an application's sign-in, if any (5.3) |
  | history | the paths its codec prints (section 9) |
  | the timer | the earliest of the client host's and the hidden tab's deadlines |
  | the platform's facts | nothing |
  | title and closing | the title, the language, and whether the client holds pending requests |
  | restart | jig's words for "the application failed", and skein's default limit (section 5) |
  | the hidden tab | skein's default delay, a minute (section 10) |

- **jig's two:**

  | Part | Takes | Gives |
  |---|---|---|
  | the client's adapter | the client host's transport requests of a step, as `jig-client-protocol-http` maps them: each exchange a method, a path under the prefix, headers and a body; the stream's request and its closing; the small store's records saved and erased; fetch's and the small store's reports on them | skein's transport's plain data: the small store's writes and removals; each exchange for fetch, same-origin, with the browser header added to its headers (section 11), its answer read up to the client's largest record; the stream's request, with the browser header, each piece cut to the client's stream piece; the reports as the host's events: a response's status and body, an exchange failed, a piece of the stream, its end, the records stored or not |
  | sign-in | the page's path and query at start; a service's credential to show; its dialog's words | the printed link's secret, read from the address, the address rewritten first; a credential shown once in a dialog of its own, never kept |

- **A report** is one platform happening, as data, which each part that
  reports wraps into the application's report type by the function it
  is given (skein's `ui-web.md`, 5.1; skein's `ui.md`, section 13).
  jig's reference's report type holds skein's parts' reports and the
  host's events, from fetch's and the small store's reports through the
  client's adapter.

### 5.2 The turn

skein's turn (skein's `ui-web.md`, 5.2), with jig's lines: those naming
the client's adapter or sign-in. An application's loop may order it
otherwise within 5.3.

```
the turn, which the inbox calls when a report waits and no turn runs:
  now, wall ← the platform's facts                 once per turn
  while the inbox holds a report:                  raised meanwhile too
    a drawing resized: to the page part, no step
    the tab hidden or shown: to the hidden tab part, no step
    the timer due: to the hidden tab part, then a step as any report
    fetch's or the small store's report: a host event, by the client's adapter
    step ← the client host steps with the report, now and wall
    requests ← the client's adapter, from the step's requests as HTTP
    the transport carries the step's writes and requests, then any navigation (5.3)
    fetch reads the stream on, after a piece of it
    history shows the step's address, printed by its codec
    the step's effects, to their parts or the shell's own code
    sign-in shows a credential once, if the step asks
  patches ← the client host builds
  the page applies them, a focus request too       an unknown node: a panic
  a new entry: its top, its heading focused, unless a focus request moved
  back or forward: the scroll the entry kept
  the page sets each link's href to the path its codec prints
  title and closing: heading · name, the language, the closing warning
  the timer: the earliest of the client host's and the hidden tab's deadlines
```

### 5.3 Where the invariants live

skein's parts hold the invariants that protect requests, so no order of
the loop breaks them (skein's `ui-web.md`, 5.3); the client's "once"
rests on them, and the reference documents the rest.

- **The small store written before a step's exchanges leave, and a
  navigation away only after:** skein's transport's. The client gives a
  request's first exchange only once the transport says its record
  stored (`../client/domain/client.md`, section 11); the client's
  adapter hands the transport a step's requests whole, so no exchange
  of a step leaves before that step's writes, and a navigation to an
  application's sign-in leaves nothing unkept behind it. When a write
  fails, the client hears its records not stored and each of the
  step's exchanges lost, and none is sent. An application driving fetch
  and the small store without the transport takes the rule on itself,
  which skein's stories for a browser shell and jig's check (section
  15).
- **No re-entry, and patches applied to the tree they are diffed from:**
  skein's inbox's and page part's (skein's `ui-web.md`, 5.3). jig's
  reference keeps its state in one cell, whose second borrow panics,
  fail-stop.
- **The rest of the order** (links' `href`s, title, timer and closing
  warning after the build) is the reference's (5.2): out of order, a
  turn shows something stale until the next, a closing warning among
  it; no request leaves unkept or twice.
- **Why not types:** skein's reason (skein's `ui-web.md`, 5.3). A part
  that holds its invariant asks nothing of the loop but to be used;
  documented order alone would leave an application's code ordering
  what jig promises, against `../README.md`, section 6.

## 6. The tree as elements

skein's (skein's `ui-web.md`, section 6): each node one element, by its
role, through the default mapping `skein-ui-web` holds, by which skein's
face also finds roles. The reference UI's nodes take the default table,
with no override. What is jig's:

- **jig's pictures** are `<img>` elements whose source is under the
  bundle's digest directory's `pictures/jig/`, an application's under
  `pictures/<its prefix>/` (4.2).
- **The sign-in part's dialog** for a credential is one of the parts'
  own elements, outside the tree, and carries no `data-node`
  (section 11).

### 6.1 An application's elements and components

skein's (skein's `ui-web.md`, 6.1; skein's `ui.md`, section 12). jig
gives no element and no component, and the reference UI's nodes keep the
default table: an application's override names one of its own owners'
tokens or kinds, never jig's. What a component builds keeps to its
page's policy (12.1): its styles from the origin's stylesheets, its code
from the origin (3.5, step 7).

## 7. Patches and acts

skein's (skein's `ui-web.md`, section 7). What the reference UI adds:

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

skein's (skein's `ui-web.md`, section 8; skein's `ui.md`, section 11).
What is jig's:

- **The reference UI draws no kind of its own** (section 16): the
  drawing code is the application's client host's `draw`, routing a
  drawing by its kind's owner (3.3).
- **A scene's colours, widths and fonts** are the stylesheets':
  `skein-ui.css`'s defaults, then the application's, which may use jig's
  palette's series colours (section 13).

## 9. Addresses and history

- **Every path is the application's:** its codec parses and prints every
  path the page shows, below its state, as total, bounded step code,
  since a URL is a person's input (skein's `ui-web.md`, section 9).
- **jig reserves none.** The application's server reserves its own: the
  prefix it mounts jig's mapping at, its bundle's prefix, and its own
  routes, none of which its codec prints as a place (section 12). jig's
  test application reserves `/jig`, where it mounts the mapping, and
  `/bundle/`.
- **The reference UI's codec,** `jig-client-view`'s `address` module,
  gives jig's places their paths, at the root:

  | Address | Path |
  |---|---|
  | the inbox | `/inbox`; filtered by a project, `/inbox/<project>` |
  | conversations | `/conversations`; by a project, `/conversations/<project>` |
  | a task's overview | `/tasks/<task>` |
  | a task's other section | `/tasks/<task>/<section>`: `activity`, `plan`, `history`, `budget`, `panels` |
  | a project's overview | `/projects/<project>` |
  | a project's other section | `/projects/<project>/<section>`: `goals`, `tasks`, `notes`, `settings`, `resources`, `policy`, `roles`, `spend`, `recurring` |
  | the deployment | `/deployment` |

  An application takes it as is, under a prefix of its own
  (`/work/inbox`), with its own segment names, or writes its own. The
  table changes only by a change to this design; an application keeps
  its bookmarks by keeping its codec, which says which place `/` is: the
  reference's, at the root, the home. The places are the reference UI's
  (`../client/domain/pages.md`, section 2).
- **In the reference's codec,** numbers are decimal, with no leading
  zero, within their type; one trailing slash and a query (an
  application's codec may read one) are ignored; any other path is a
  place with no page, shown as missing (`../client/domain/pages.md`,
  section 8). The fragment is not reported.
- **Links' `href`s and history** are skein's parts' (skein's
  `ui-web.md`, section 9): a link node's binding resolves, without
  acting, to the address it leads to, which the client host prints as
  a path; an address a step shows is a new entry, or takes the current
  one's place, as the step says; back and forward are a place gone to,
  as a path; a path opened directly starts the application at it.

## 10. Transport and the small store

The default mapping's browser end: the client host's
`jig-client-protocol-http`, the mapping's client end (`client.md`,
17.4), maps `jig-client`'s requests onto HTTP, and the client's adapter
gives them to skein's fetch, small store and transport as their plain
data (skein's `ui-web.md`, section 10; 5.1).

- **Exchanges** by skein's fetch: each of the client's exchanges as
  `jig-client-protocol-http` maps it, a POST of the message's record to
  its route under the prefix the application gives (`client.md`, 17.1),
  with `Content-Type: application/octet-stream`, and the browser header,
  which the adapter adds (section 11); the adapter sends it with
  same-origin credentials, so the cookie rides with it, and abandons it
  by aborting. A body is read up to the client's largest record; past
  it, the exchange failed. The response's status and body go back to
  the host, whose client end makes them a reply, and a failure, an
  abandon or a body cut short a loss.
- **The credential** rides in the cookie alone: the reference signs the
  client in by the transport (section 11), so the client sends no
  credential with an exchange. An application whose page holds a
  credential in hand sends it as a bearer, and takes on what HTTP-only
  kept from its scripts (section 19).
- **The stream** by skein's fetch: a GET of the stream's route with the
  browser header, which the adapter adds, and same-origin credentials,
  its body read a piece at a time, each piece cut to the client's stream
  piece and handed to the host, whose client end cuts the server-sent
  events into messages (`client.md`, 17.2). The next read is asked by
  the loop after the step (5.2), so the platform paces it. Its end is
  the stream gone.
- **Each load of the page is a client,** with a stream of its own
  (`client.md`, section 4): links and back and forward within the page's
  history keep it (section 9); a reload, an address typed or opened,
  and skein's restart part's reload (section 5) start a new one, which
  sends its predecessor's stream gone, by the name the small store
  kept, before it opens its own. The browser ends the old page's
  connection when it likes, often after the new page's stream has
  opened.
- **The transport,** skein's, takes a step's requests whole, as the
  client's adapter gives them: the small store's writes first, then the
  exchanges and the stream, then a navigation, if any (5.3).
- **A hidden tab lets its stream go** a minute after it is hidden,
  skein's default, which jig's reference keeps (the application's
  parameter): fetch closes the stream and reports its end, which the
  client takes as any drop, and an opening while hidden waits until the
  tab is shown (skein's `ui-web.md`, section 10).
- **The small store** is skein's, the tab's session storage: each of the
  client's records under its key (`client.md`, section 10), the stream's
  name among them, both as base64 text, under the prefix the application
  gives, the reference's `jig.`; read whole at start. The reference UI's
  own records (its places and drafts) are its application's, kept beside
  the client's through the same transport (`../client/domain/view.md`, 18.8).
  A write the platform refuses, its quota full, is a failed write. The
  restart part's key is outside the prefix (section 5).
- **Closing a tab with requests pending** asks the platform to warn
  (`beforeunload`), through skein's title and closing part, only while
  the client holds requests not yet answered durable.

## 11. Signing in and out

The cookie is the mapping's, set by its engine end; the browser header
is added by the client's adapter (5.1); the engine end checks both
(12.2). The sign-in part is jig's (5.1), and the client's own exchanges
sign in and out (`../client/domain/client.md`, section 12; `client.md`,
section 8).

- **The cookie,** its attributes `client.md`, 17.3's (why: section 19):
  every request under the prefix carries it; the module, the client and
  the application's client host never see it, since the client is
  signed in by the transport.
- **The browser header,** `Jig-Client: browser`, on every request of a
  browser under the prefix, the stream's among them: one the cookie
  authenticates without it is answered `403`, with nothing done (12.2).
- **The printed link, opened in a browser,** through the application's
  route:
  - the clients' component gives the secret, and the application's
    startup prints the URL: under the default mapping, the link's route
    under its prefix, the secret in its query,
    `<origin><prefix>/v1/sign-in/link?secret=…` (`client.md`, section 8
    and 17.1);
  - a GET of the link's route spends nothing and draws nothing: the
    mapping's engine end leaves it to the application's page, with the
    referrer policy (12.2), and the server answers its page there (why:
    section 19);
  - the sign-in part reads the secret from the address's `secret` and at
    once rewrites the address to the application's home, `/` in the
    reference (why: section 19); the client host then signs the client
    in by the link, whose exchange is the link route's POST, the one a
    native client makes, with the browser header; the engine end answers
    it signed in, the credential in the cookie and none in the body;
  - refused (a spent or expired secret), the application prints a fresh
    link, as for any refused link (`client.md`, section 8), and the
    client is signed out, which the reference UI shows as its sign-in
    page, saying to open the link printed. Not ready or lost, the same;
    a link not spent still works opened again.
- **An application's sign-in** runs on its own routes (an OAuth or
  OpenID server, a company's directory, a chat platform's identity), as
  the application's server's code: it verifies the identity, asks the
  clients' component to mint a credential for it (`client.md`,
  section 8), and sets the mapping's cookie with it, through the engine
  end's function for that, so the cookie is the one the mapping reads.
  In a browser it is a navigation, which skein's transport makes after
  the step's writes (5.3); the path it comes back to is its own to
  check, as one path of its own, never another site's (a `//` or `/\`
  read as one) nor one of its reserved prefixes.
- **Signing out** is the client's exchange to the sign-out route, which
  the engine end answers by clearing the cookie (`Max-Age=0`). Its
  answer is its sign-in's (`../client/domain/client.md`, section 12): each sign-in
  loads a new page (the printed link's, before the client starts, or an
  application's sign-in by navigation), so an earlier page's sign-out
  never answers into the client of a newer sign-in.
- **A service's credential, shown once,** in the reference: in the
  sign-in part's dialog of its own, outside the tree, as selected text,
  with a button closing it, in jig's words. The part never keeps it,
  nor does the client (`client.md`, section 8).

## 12. Served by the application's server

jig serves nothing from an origin: the application's server serves it
all, from the clients' listener it owns (`service.md`, section 2).

- **Its page and its bundle,** as skein's `ui-web.md`, 4.4 says a bundle
  is served: under its bundle's prefix, the digest directory immutable;
  its page at every GET of a path it does not reserve, revalidated by
  its `ETag`; its root files; each file's type from the bundle's closed
  list; `X-Content-Type-Options: nosniff` on every answer. skein's
  example's server keeps that contract in code an application may copy
  (skein's `examples.md`, 9.3), as jig's test application's does.
- **jig's mapping, mounted at a prefix of its own,** every path under it
  the mapping's (`client.md`, 17.1), but a GET of the link's route,
  which the mapping leaves to the page (section 11). The mapping's
  answers carry `Cache-Control: no-store` and
  `X-Content-Type-Options: nosniff`; under its prefix it answers no
  `OPTIONS` and sends no `Access-Control-` header (12.2).
- **Or a transport of its own,** in place of the mapping, carrying jig's
  messages to the clients' component and back (3.5, step 10).
- **Its own routes:** its API, its sign-ins (section 11), anything else.
- **jig's test application's server,** in `jig-test-service`, is the
  example (`service.md`, section 7).
- **One application per origin** is the deployment's to keep (section
  19).

### 12.1 The policy

The page's policy is the application's page's, sent by its server on
every answer.

- **Its floor is skein's strict policy,** what skein's parts and the
  bundle need (skein's `ui-web.md`, 4.4): scripts and styles from the
  origin only, nothing inline, WebAssembly compiled only by the
  origin's code (`'wasm-unsafe-eval'`); `script-src` and `style-src`
  take nothing more (skein's `ui-web.md`, section 17).
- **The reference's,** which jig's test application's server sends, is
  skein's starting policy (skein's `ui-web.md`, 4.4), with
  `X-Content-Type-Options: nosniff`, `Cross-Origin-Opener-Policy:
  same-origin`, `Cross-Origin-Resource-Policy: same-origin` and
  `Referrer-Policy: no-referrer`. An application copies it, and adds
  what its own data needs: named `https` origins for images, fonts,
  media, connections and other sites' pages, never a wildcard, a scheme
  alone or `'unsafe-*'`.
- **Frames:** `frame-ancestors 'none'` on every answer of the origin's,
  in the reference, so no other site frames a page that decides; an
  application that frames its own pages names its own origin there.
- **The referrer:** at the printed link's route, the page's address holds
  the secret until the sign-in part rewrites it, so its answer sends a
  policy that never sends a path to another origin (12.2); the page's
  own `referrer` meta or attributes set no wider one, which the header
  cannot overrule.

### 12.2 The example guards

The guards that protect a browser's credential belong to the transport,
not to jig's model, so they are the application's server's. jig ships
them as example code in its mapping's engine end,
`jig-protocol-engine-clients-http`, on every request under the prefix,
before any reaches the clients' component. What each stops is section
19's.

| Guard | What the example does |
|---|---|
| a Host allow-list | any `Host` off the list the application gives when it mounts the mapping answered `421`, with nothing done; the reference's list is the address its printed link names on loopback, and its certificate's name between hosts |
| the required header, with no CORS | a request the cookie authenticates without `Jig-Client: browser` answered `403`, with nothing done; a POST whose `Content-Type` is not `application/octet-stream` answered `415`, with nothing done; no `OPTIONS` answered and no `Access-Control-` header sent |
| the cookie's attributes | `client.md`, 17.3's |
| a referrer policy on the printed link's route | `Referrer-Policy: no-referrer` added to the page the application answers at the link's route |

- **The page's policy** (12.1) is the fifth guard, against injected
  scripts acting as the person and a decision tricked by a frame; it is
  the application's page's, and the reference shell's example of it.
- **No switch:** the mapping takes the allow-list and nothing that turns
  a guard off. An application uses the mapping as it is, wraps it (a
  check of its own before it, a header added to its answers), or writes
  its own transport; writing its own means taking over what the guards
  held, as replacing one of skein's browser parts does (skein's
  `ui-web.md`, 3.3), which jig's attack stories check, if it runs them
  (section 15).

## 13. Stylesheets

The reference UI's, for an application that takes it; skein's
otherwise (skein's `ui-web.md`, section 12).

- **`skein-ui.css`,** which skein's bundler writes first in every
  bundle, holds the generic rules (skein's `ui-web.md`, section 12). jig
  has no class of its own for acts.
- **Every style token and region name has a name** (skein's `ui.md`,
  section 8). jig's start with `jig-`, its prefix and a hyphen: its
  tokens' names are `../client/domain/view.md`'s, section 13, and its
  regions' `../client/domain/pages.md`'s, 12.4. An application's start
  with its owners' prefixes, which its shell gives the parts as the
  look (3.3).
- **jig's stylesheet,** `jig.css`, ships with `jig-client-shell-web` and
  loads after `skein-ui.css`. It places jig's regions by `data-region`,
  and the landmarks of the reference UI's default arrangement: the
  banner at the top; the navigation beside the main on a wide screen
  and above it on a narrow one; asides beside the main; the composer
  pinned below its conversation. It styles jig's tokens, in jig's
  palette, light or dark as the platform prefers.
- **jig's palette:** custom properties an application's stylesheet uses:
  `--jig-text`, `--jig-background`, `--jig-muted`, `--jig-accent`,
  `--jig-warning`, `--jig-danger`, `--jig-success`, and six series
  colours for drawings.
- **The application's stylesheet** loads after jig's, styles its own
  tokens and regions, and may restyle jig's (its own page: 3.5, step 4).
- **No style is a fact** (skein's `ui.md`, section 8;
  `../client/domain/view.md`, section 13).

## 14. Limits

- **The client's:** an exchange's answer, at most its largest record; a
  piece of the stream; exchanges in flight (`client.md`, section 13),
  which the client's adapter gives fetch.
- **The mapping's engine end's:** a request's bytes; calls in flight,
  and streams among them, in all, never per sign-in (`client.md`,
  sections 4 and 13); its numbers in `jig-protocol-config`'s profiles
  (`configuration.md`, section 6).
- **The tree's:** `skein_ui::Limits` (skein's `ui.md`, section 15), whose
  numbers the reference UI's constructors derive from the client's
  limits (`../client/domain/view.md`, section 10); a drawing's SVG has
  at most a scene's marks (skein's `ui-web.md`, section 13).
- **The platform's:** session storage's quota, past which a write fails.
- **The parts' own,** skein's (skein's `ui-web.md`, section 13): one
  timer; one read of the stream outstanding; the inbox's reports raised
  during a turn, only those the turn's own calls raise; three reloads in
  a minute, by default, which jig's reference keeps.
- **The bundle's size,** a guide (4.2).

## 15. Testing

Each within jig's suites' budgets (`../../development/workflow.md`,
section 2). skein's parts, its bundler, its face and its harness are
tested in skein, and in a browser on skein's UI example (skein's
`ui-web.md`, section 14); jig's tests take what is jig's.

- **On the host,** in jig's default suites:
  - the client's adapter's pure half: the host's requests as HTTP to
    the transport's plain data (the small store's writes first, the
    browser header on each exchange and on the stream's request, the
    bound on an answer, the stream's piece), fetch's and the small
    store's reports to the host's events, an answer past the bound
    failed;
  - the sign-in part's pure half: the secret read from the address, the
    address rewritten first, the credential's dialog's words;
  - the mapping's engine end, in `jig-protocol-engine-clients-http`'s
    step tests: each guard of 12.2 refusing with nothing reaching the
    component (a `Host` off the list, the header missing beside the
    cookie, a POST of another content type, an `OPTIONS`); the cookie's
    attributes on loopback and with TLS, set at a browser's sign-in and
    cleared at its sign-out; a GET of the link's route left to the page
    with the referrer policy, spending nothing;
  - the reference UI's codec: every place printed and parsed back, at
    the root, under a prefix, with other segment names; any bytes parsed
    to a place, never a panic.
- **The wasm32 builds in jig's gate:** `jig-client-shell-web`,
  `jig-client-test-shell-web` and every client crate, built for
  `wasm32-unknown-unknown` with clippy's lints as errors, since the
  parts' platform halves are compiled only there; skein's gate builds
  skein's (section 18).
- **The browser tier:** a suite of its own, `--profile browser`, run by
  jig's gate after it builds jig's test application's bundle with
  skein's bundler, within its budget (a rule: its profile's global
  timeout, `../../development/workflow.md`, section 2). Each
  test starts one server (the application's binary; for jig, the
  reference `jig-test`, serving its bundle and the mapping at `/jig`)
  and one Chromium, and plays its stories with a person each: a browser
  context, its own cookies and storage. A missing Chromium or bundle
  fails the suite, naming the command that provides it; it is never
  skipped. The bundle's directory reaches the tests in
  `JIG_TEST_BUNDLE`.
- **The harness and the face** are skein's, in
  `skein-fake-person-browser` (skein's `ui-web.md`, 14.2; skein's
  `ui.md`, 17.1). jig's tier gives the harness the prefix of the line
  `jig-test` prints its link on, so the address it opens first is the
  printed link, which signs the person in (section 11); the face opens
  the paths the application's codec prints.
- **skein's stories for a browser shell** (skein's `ui-web.md`, 14.2):
  jig's tier calls them with jig's binary, bundle and codec.
- **jig's stories for a browser shell,** in `jig-client-fake-person-browser`,
  on skein's face and harness: functions an application's tier calls
  with its binary, bundle and codec, to check its wiring of
  `jig-client`, the client's adapter and the printed link, since its
  loop is its own. Each signs its person in by the printed link:
  - a reload keeps a pending request, which the engine makes once;
  - the engine killed and restarted shows offline, then live, where its
    presentation shows the link's state;
  - a spent link is refused, and the person is signed out.
- **jig's attack stories,** in the same kit: functions an application's
  tier may call with its binary, its page's address and its mapping's
  prefix, against its own server, whether it uses jig's mapping or its
  own transport. Checked, never mandated, as jig's check is
  (`../client/domain/testing.md`, 6.2). Another origin's page is
  served by jig's kit, in the story's own loop, as the browser kit's
  facts are served (skein's `ui-web.md`, 14.1); skein's harness opens it
  as an address the test gives, and the story names the policy
  violation it causes as trouble it expects (skein's `ui-web.md`, 14.2):
  - *a cross-site request:* such a page, on another port of the same
    host and on another host name, makes the signed-in person's browser
    send a request under the prefix; it is refused, and nothing is done,
    which the person's page then shows;
  - *a rebound Host:* a request naming a host off the list, as a rebound
    name would, sent by skein's HTTP client; answered `421`, with
    nothing done;
  - *a framed page:* such a page frames the application's page; the
    browser shows nothing of it, under the page's policy.
- **jig's tier** calls skein's stories, jig's and jig's attack stories,
  and adds: a person opens the printed link and sees their inbox, under
  the reference policy, with no trouble.
- **jig's test application takes skein's bundler's page** (4.3), and
  two of 3.5's steps once each: a root file (step 5); a prefix on the
  reference codec, in one story (step 3). An element override, a
  component and a JavaScript snippet are skein's example's (skein's
  `ui-web.md`, 14.3); jig's test application carries no snippet by
  default (section 17).
- **What an application adds:** its tier, over skein's harness and face:
  skein's stories and jig's called, jig's attack stories if it runs
  them, and its own pages, drawings, components and sign-ins.

## 16. What is not the layer's

- **Following data as it comes** (a log tailed, a metric's latest
  points) is an application's streams (`../client/domain/contract.md`,
  7.4): a contract and engine piece, carried by skein's parts and the
  client as any watch, with no change to them.
- **Graphics beyond a scene** (a canvas of hundreds of thousands of
  points, three dimensions, video) are out of jig's scope: a component
  of the application's in a browser (6.1; 3.5, step 6), its own shell
  elsewhere (skein's `ui.md`, section 12; `../client/domain/view.md`,
  section 11).
- **Later,** each adding without changing what sections 3 to 15 settle:
  - *jig's own drawing kinds,* a chart every application shares: a kind
    moves into the reference UI when a second application draws the
    same;
  - *tabs of one person sharing one stream, and HTTP/2* (`client.md`,
    section 15);
  - *the bundle served compressed:* skein's (skein's `ui-web.md`,
    section 15), and its server's;
  - *the browser tier in other browsers than Chromium,* which comes to
    skein's harness first (skein's `browser.md`, section 11; skein's
    `ui-web.md`, section 16); *a press that copies text, downloads and
    uploads of files:* effects (skein's `ui.md`, section 13), their parts
    skein's (skein's `ui-web.md`, sections 11 and 15), for an application
    whose people need them;
  - *`ops`'s browser shell,* `jig-ops-client-shell-web`, and its server:
    part of `ops` standalone (`../examples.md`, 9.1), which gives it an
    engine.

## 17. Open questions

- **Right-to-left languages:** what they ask of `jig.css` and the
  reference UI's phrasebook, beyond `<html lang>`
  (`../client/domain/view.md`, section 14); what they ask of the
  elements and scenes is skein's question (skein's `ui.md`, section 19).
- **Media from the origin:** the bundle's closed list (skein's
  `ui-web.md`, 4.4) has no audio or video type, and skein's vocabulary
  no media (skein's `ui.md`, 4.2): what an application serving its own
  would need of both.
- **A JavaScript snippet in jig's test application,** the user's
  decision; by default, none. skein's example carries the snippet that
  checks that the bundler and the strict policy carry an application's
  JavaScript (skein's `ui-web.md`, 14.3). One in jig's would check the
  same under the reference policy, at the cost of a component and a
  snippet jig's test application has no other use for.

## 18. What it owes and is owed

- **The reference UI** (`../client/domain/view.md`,
  `../client/domain/pages.md`) owes: jig's part of the look, the names
  of jig's style tokens and region names under jig's prefix (section
  13), and jig's owner's number as skein-ui's opaque owner (section 3);
  a link node's address, without acting; the parts' own phrases among
  jig's (section 5); jig's confirmation's cancel button naming Escape
  (section 7); and its codec, `jig-client-view`'s `address` module,
  parsing and printing jig's places, total and bounded, at the root,
  under a prefix, or with other segment names (section 9).
- **`jig-client`** (`../client/domain/client.md`) owes its transport
  requests as plain data, in the order it gives them (an exchange with
  its credential or none, the stream opened and closed, records saved
  and erased), and takes its transport's events (a reply, a loss, a
  message streamed, the stream gone, records stored or not); a sign-in
  by link and by the transport; its pending requests and its next
  deadline, read by the client host.
- **`jig-client-protocol-http`** (`client.md`, 17.4) owes: a message
  as a method, a path under the prefix the application gives, headers
  and a body; the stream's request; the stream's server-sent events cut
  into messages; the bound on an answer, from the client's limits.
- **`jig-protocol-engine-clients-http`** owes: the example guards of
  12.2, with no switch, every POST's content type among them; the
  cookie set at a browser's sign-in, with no
  credential in the body, and cleared at its sign-out; a function that
  sets the same cookie for a credential the application minted; a GET
  of the link's route left to the application's page, with the referrer
  policy; its answers with `no-store` and `nosniff`; no `OPTIONS` and
  no CORS under its prefix (sections 11, 12 and 12.2).
- **`jig-protocol-engine-clients`** owes: the printed link's secret,
  given to the application to print, and a fresh one after a refused
  link; a credential minted on the application's word (`client.md`,
  section 8).
- **An application's server** owes: its page and bundle as skein's
  `ui-web.md`, 4.4 says; its page's policy, at least skein's strict
  policy; jig's mapping mounted with its Host allow-list, or a transport
  of its own that takes over what the guards held; the path its own
  sign-in comes back to, checked (section 11).
- **`configuration.md`** names no bundle for jig: jig's test
  application's startup reads its bundle's directory from a member of
  its own (`service.md`, section 7).
- **An application's browser shell** is owed skein's parts, jig's, and
  jig's reference to copy; driving fetch and the small store without
  the transport, it owes the transport's order (5.3).
- **skein owes:**
  - `skein-ui`, which the reference UI builds on: the tree (skein's
    `ui.md`, section 3), owners as opaque numbers (skein's `ui.md`, 3.3),
    the look (skein's `ui.md`, section 8), and Escape as a dialog's
    shortcut (skein's `ui.md`, 6.1);
  - `skein-ui-text`, the text shell an application's terminal composes
    (skein's `ui-text.md`);
  - `skein-ui-web`: the parts and the default mapping (skein's
    `ui-web.md`, 5.1 and section 6); `wasm-bindgen`'s start attribute
    re-exported and the glue pinned (skein's `ui-web.md`, 3.1 and 4.1);
    fetch taking the headers, the body and the bound on an answer an
    application gives, for exchanges and the stream; the small store's
    prefix, the restart part's limit and the hidden tab's delay as the
    application's parameters;
  - `skein-ui-bundle`: the bundle under a prefix the application gives,
    its given stylesheets after `skein-ui.css` in the order given, each
    owner's pictures under its prefix, an application's page and files,
    and the closed list of types (skein's `ui-web.md`, 4.2 to 4.4);
  - its UI example's server, keeping the serving contract in code to
    copy (skein's `examples.md`, 9.3);
  - `skein-fake-person` and its browser face and harness,
    `skein-fake-person-browser`, with skein's stories for a browser
    shell (skein's `ui.md`, 17.1; skein's `ui-web.md`, 14.2): the harness
    opening an address the binary prints, found by a prefix the test
    gives, or one the test gives; stopping the binary and starting it
    again within a story; and failing a test only on trouble its story
    did not expect, which jig's attack stories use as it stands (section
    15);
  - the wasm32 builds: lib, `skein-codec`, `skein-http`, `skein-ui` and
    `skein-ui-web` built for wasm32 and checked by its gate (skein's
    `lib.md`, section 2; skein's `testing.md`, section 6); a client
    compiled to WebAssembly in its model, with no kernel and no ring,
    the glue crates its only `unsafe`, a panic a trap the restart part
    reloads from (`programming-model.md`, 2.1).
- **smith owes** `smith-transcript` building for wasm32, which jig's
  gate checks through `jig-protocol-transcript`.

## 19. What jig promises, and where its promise stops

- **jig's promise** is `../README.md`, section 10's: a request
  presented with a valid credential is that credential's party's, and
  everything after is checked below the API
  (`../client/domain/contract.md`, 5.2); jig mints, verifies, expires
  and revokes the credential (`client.md`, section 8).
- **Where it stops,** as `../README.md`, section 10 says what is the
  application's:
  - *a credential not used across sites* is the application's server's
    work, as keeping it out of its logs is: jig gives example guards
    (12.2) and attack stories (section 15), to use if they suit;
  - *a deciding act confirmed, with its facts shown,* is the
    application's, with jig's check to run if it suits
    (`../client/domain/testing.md`, 6.2);
  - *one application per origin* is each deployment's: session storage,
    `'self'` in the policy and the cookie are per origin, so two
    applications on one origin would share the small store's storage,
    each other's scripts and the sign-in, and either could act as the
    person in the other. An origin each, by host name or port.

What jig's parts hold whole, whatever value an application gives them,
with the reason; an application that writes its own in the part's place
(3.5, steps 9 and 10) holds the rule itself. What each guard of 12.2
stops is this table's. The rules skein's parts and bundle hold are
skein's, with their reasons (skein's `ui-web.md`, section 17): text
never read as markup; scripts and styles from the origin only, nothing
inline; the bundle's closed list of types; an element override keeping
its node's accessible role and name; the transport's and the inbox's
invariants, on which the client's "once" rests (the table's last row).

| Held | Why | Instead |
|---|---|---|
| the `Host` allow-list (12.2) | DNS rebinding: a page of another site, its name rebound to the loopback address, would reach the engine as its own origin; answered `421`, it reaches nothing. | the list it gives at the mount: its listener, and between hosts its certificate's name |
| the required header, the content type and no CORS (12.2) | Cross-origin requests: a page of another origin can neither set the header nor send a body of the mapping's type (either would need a preflight the mapping never answers, and no form can send that type) nor, from another site, send the cookie (same-site strict), so it can neither act as the person nor sign them in by the link. Same-site is not same-origin: on one person's machine, a local server on another port of the same host is the same site, which same-site strict lets send the cookie, and only the header stops it. | its own transport, holding what they held |
| the cookie's attributes (`client.md`, 17.3) | HTTP-only keeps the credential from every script, the module's too; same-site strict, from requests another site starts; the `Path`, from every request but the mapping's; `Secure`, from plaintext, and left off on loopback, since not every browser keeps a secure cookie set over plaintext, loopback included. | its own sign-in, which sets the same cookie through the mapping's function |
| the referrer policy at the link's route (12.2) | The page opened at the link's route holds the sign-in's secret in its address until the sign-in part rewrites it; a policy sending a path to another origin would send the secret with a first request to an origin the page's policy names. | `strict-origin`, the origin alone |
| the link's route safe to GET (section 11) | A GET is the one request any other site can make the person's browser send (a link, an image), with no header of its own and past every guard; one that signed in, or drew a fresh link on a refusal (`client.md`, section 8), would let any page the person visits spend the printed link or keep drawing new ones, until the link printed never works. | its own page at its own route, as safe |
| the secret taken out of the address first (section 11) | Left in the address, the secret stays in the tab's history and in what the page shows or shares, after it is spent and before. Taken out first, it leaves the address bar and the tab's history; the browser's record of visited addresses keeps it, spent once signed in. | its own page at its own sign-in route |
| the credential never kept (section 11) | A service's credential acts as the service; kept in a tab, it would outlive its dialog, in storage any script of the origin reads and in the session's restore. | the person copies it from the dialog, which the application's stylesheet may restyle |
| a step's client requests through the transport (5.3) | A request that leaves before it is kept is lost or made twice after a reload. It is the client's "once" (`../client/domain/client.md`, section 6), which jig owns (`../README.md`, section 10), and which skein's transport holds for any application (skein's `ui-web.md`, section 17). | its own order for the rest of the turn (5.2); the transport replaced, the order taken with it and checked by skein's stories for a browser shell and jig's (section 15) |
