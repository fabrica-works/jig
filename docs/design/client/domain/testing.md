# Testing the client

Provisional, 2026-10-09, revised 2026-10-15. How jig's client domain
and view are tested, and how they test an application's client: the
person who acts on the view, the client's world with a scripted engine,
the joint world with the engine's domain, their referee and their
faults. It applies skein's `testing-strategy.md` as
`../../domain/testing.md` does for the engine; the tiers below the
domain come with the client's protocol layer. What is still open is
listed in section 10.

## 1. In one page

- **The person acts on the view,** as a person does: finds nodes in the
  tree by role, name and text, presses them, types into them, and waits
  to see what follows. Never by style, by position, by region name, or
  through the client's state. So a story written for jig's pages runs
  on an application's arrangement of the same sections.
- **One person, every tier.** The person is a script that asks its face to
  find and act, so a scenario written once runs on the tree face in every
  world, and on a browser's accessibility tree through the browser's
  face, in the browser tier (section 3).
- **The client's world comes first.** The real domain and view, a scripted
  engine that keeps the contract (contract.md), the shell played by the
  world, and every fault of section 7. Most of the client's tests live
  there.
- **The joint world is the check.** The same people and scenarios in
  front of the engine's real domain: jig's core with its test connector,
  or an application's whole engine domain on jig's conformance world.
- **The referee watches what the person saw and what the engine made
  durable,** never the client's state: nothing shown done before it was
  durable, nothing made twice, nothing sent lost.
- **Every world builds and diffs every page,** and applies the patches to a
  model of the last tree, which must then equal the new one.

## 2. The tiers

| Tier | Real | Faked |
|---|---|---|
| step tests | one step function of the domain or of jig's view; jig's words; jig's default arrangement's banner and `after_page`'s parts last, on each of jig's pages (view.md, 3.1); `jig_view::facts` and `jig_view::check`; jig's pages printed by skein's text shell, by golden text (view.md, 15.6); its test family's drawing code, by golden scenes (view.md, 17.5) | its events, by hand |
| the client's world | the client root, its domain and its view, for jig's test application or a real one | the engine, scripted; the shell, played by the world; the person |
| the joint world | the client root, domain and view, and the engine's domain, root and all | the store, hosts and systems, as the engine's worlds fake them; the shell; the person |
| the protocol world, the real loop | with the client's protocol layer and shells; skein's text shell's terminal in front of the native one, on skein's simulated machine | |
| the browser tier | the application's binary and its bundle in Chromium, the person through skein's browser face and harness (section 3) | what the binary's configuration fakes |

Every world runs on skein's world harness: one loop, one seed, the
referee in the loop, the heap counted at every iteration
(testing-strategy.md, sections 6 and 7). The browser tier is no world:
its engine and its Chromium are real processes, children of the
harness's one loop (section 3).

What every application on skein-ui shares is tested in skein, by its
crates' step tests: the tree, the diff, the Markdown reader, words'
machinery, the outline's rules, scenes by golden scenes, and the text
shell's renderer and command reader by golden text (skein's `ui.md`,
section 17; skein's `ui-text.md`, section 8). jig's tests test what is
jig's on them.

## 3. The person

The person is skein's, `skein-fake-person` (skein's `ui.md`, 17.1):
ordinary Rust, shaped as a step machine so that the browser tier's loop
runs it too.

- **What it looks for:** a target by role and words in its name, by role
  and its whole name exactly, or by its text, within regions named
  outermost first: "the button named Accept, in the region named
  Proposal from I".
- **What it does,** a step at a time: go to an address; press; type into
  a field; send, as Enter in a composer; see something, within a time;
  see it gone, within a time; reload; pause.
- **What it reports:** what it does next; that it waits until the tree
  changes or a time passes; that it is done; or that it failed, at which
  step and why ("no button named Release in the region named T7").
- **The tree face** answers its queries on the view's tree at once, the
  way an accessibility tree is searched, and turns its acts into the
  acts a shell would report. It reads a drawing by its name and its
  fallback, as a screen reader does (view.md, section 3).
- **A random person,** for the fuzzy suite, presses any bound node but
  a drawing (view.md, 17.3) and types any words drawn from the seed, so
  a sweep reaches what no script names.
- **The browser's face and its harness** are skein's, in
  `skein-fake-person-browser` (skein's `ui-web.md`, 14.2): the person's
  queries answered on the browser's accessibility tree, roles through
  skein-ui-web's mapping, which an application's elements and
  components keep, names by words in them or exactly, text by words
  within one text run; its acts a press, typing (the field cleared, then
  the words), Enter for a send, an option chosen with the browser kit's
  keys, a navigation to an address's path, and a reload. The harness
  runs the application's server binary and Chromium as children of one
  loop, stops the binary and starts it again within a story, fails a
  test on any trouble the story did not expect, and presses a drawing's
  mark at its point, which the person cannot. A browser's accessibility tree makes each run of a
  node's text a text of its own, so a story that runs in the browser
  tier finds text by words within one run; the tree face matches a
  node's whole text.
- **skein's stories for a browser shell,** which an application's tier
  calls with its binary, its bundle and its codec (skein's `ui-web.md`,
  14.2): typing while a delivery patches the page keeps the field's
  focus, caret and words; back and forward move between pages, and an
  address opened directly shows its page; a panic reloads within the
  restart limit; a write the small store refuses fails the step's
  exchanges.
- **jig's, in `jig-fake-person-browser`,** on skein's face and harness:
  - *the sign-in by the printed link:* the harness opens the link the
    engine's binary prints in a person's page, signed in by its cookie
    (`../../protocol/browser.md`, section 11);
  - *jig's stories for a browser shell,* functions an application's
    browser tier calls with its own binary, its bundle and its codec,
    where its pages include jig's, each a person's story through the
    face: a reload keeps a pending request, which the engine makes once;
    the engine killed and restarted shows offline, then live; a spent
    link is refused, and the sign-in page says to open the new one.

  An application's browser shell runs its own loop over skein's parts
  and jig's, so both kits' stories check its wiring: the order its loop
  keeps, and, where it drives fetch and the small store without skein's
  transport, the rule it then takes on itself (skein's `ui-web.md`, 5.3;
  `../../protocol/browser.md`, section 15).

## 4. The client's world

`tests/client`, package `jig-client-world`.

- **Clients:** each a tab or a device of a party: its client root, its
  view root, its small store, its address history. Several per party,
  and several parties.
- **The scripted engine** (`testing/jig-scripted-engine`) keeps the
  contract for jig's documents: requests answered by key, after a
  commit it scripts, and answered again alike; refusals as scripted;
  documents for every read; watches delivered level-triggered, a delivery
  in flight at a time; a run's text written out over time. It holds the
  truth the referee compares with: what it committed, and when.
- **The application's parts:** a client world for an application adds its
  families' scripted documents and requests beside jig's, through the
  scripted engine's hooks, which are ordinary Rust.
- **jig's test application:** jig's own client world runs a client root of
  jig's own, with one test family whose parts fill every kind of slot,
  and whose page fills both of an application's: a card's, and a
  section's (a task's plan), as the test connector stands in for an
  application in the engine's worlds (`../../domain/testing.md`,
  section 7). Its page also presents one of jig's objects itself, a
  proposal, from jig's domain data, bound through jig's binding
  constructors and saying `jig_view::facts`' sayings, so the referee's
  `jig_view::check` holds an application's own presentation in jig's
  world (view.md, section 4). It takes jig's default arrangement of
  every page, but in client.md 17's story of a task's page arranged
  with every section, whose client gives an arrangement naming every
  `TaskSection`. Its test family's drawing has drawing code, golden
  scenes and an act, with an ordinary node beside it doing the same
  (view.md, section 17). Its browser shell, `jig-test-shell-web`, jig's
  reference shell, written after skein's and adding jig's link, sign-in
  and client host, takes jig's steps an application may take past it,
  once each: its own page or skein's bundler's (the plan's choice); a
  prefix on jig's codec in one story; and a policy addition refused at
  start, in a host test (`../../protocol/browser.md`, 3.5 and section
  15). An element override and a component are skein's example's
  (skein's `ui-web.md`, 14.3).
- **Its stories** are client.md's, section 17, and pages.md's, section 10,
  each written once in a person's terms.
- **At the text shell.** A client of the world may be played as skein's
  text shell is (view.md, section 15): each tree it shows printed by its
  renderer, and each act the person makes typed as the line a person
  would type for it, from the print's numbers, then read back by its
  command reader, which must give that act on that node. A person's
  typing followed by sending is one line, `n WORDS`; typing alone into a
  field that does not submit on enter is `n WORDS` too; a story that
  types into a field that submits on enter without sending is not
  played so. The world signs its clients in, as a terminal's host does
  (view.md, 15.5). A story run so checks that a terminal can make every
  act it makes.

## 5. The joint world

The client's world, with the engine's domain in place of the scripted
one:

- **For jig:** jig's core world under its test root
  (`../../domain/testing.md`, section 4), its parties replaced by clients
  with the person in front.
- **For an application:** its whole engine domain on jig's conformance
  world, with its client roots and the people of its stories: `ops`'s P
  accepting a restart from the incident's page.
- **The translation** between the contract's values and the engine's
  people and views vocabulary is written once, in the world, as the
  protocol layers will carry it as bytes.
- **The engine's faults** reach the person: crashes at commits, held
  commits, restarts, as the conformance world makes them.

## 6. The referee

It watches what each person saw (the view's trees, as the person read
them) and what the engine made durable (the scripted engine's commits, or
the engine's store), never a client's state. Its rules:

- **Nothing shown done before it was durable:** no outcome of a request
  is shown before the commit that answers it.
- **Made once:** every request a person made is made once by the engine,
  whatever the faults: drops, restarts, double presses, reloads.
- **Nothing sent lost:** every word a person sent and was told was sent
  is in its task's activity, or its task ended.
- **What they saw is what they decided:** every act that names a
  revision (contract.md, 5.2) names the one the card the person acted on
  showed, or the application's presentation of the object, bound through
  jig's binding constructors (view.md, section 5).
- **A decision was seen:** every deciding act a person made was made on
  a tree that showed its decision's facts (view.md, section 4), in the
  open confirmation or within the acted node's nearest named region. The
  referee runs `jig_view::check` on every tree a person acted on, in
  jig's worlds and in each application's, so an application that
  presents jig's objects itself is held to what jig's cards show.
- **Offers kept:** a draft within the bounds its act was offered with is
  made, and one beyond them is proposed or refused as the offer said,
  unless a newer document of the object shows its facts moved in between
  (contract.md, 3.2).
- **A card leaves for a reason:** a card leaves only when its request was
  answered or its object stopped waiting, and says which. A person's own
  display acts (a filter, a fold, reading more in a window, going to
  another page) change what is shown, not what waits: the world observes
  each, and it explains that client's cards shown before it going out of
  view, while every other rule stands for them. A card that goes out of
  view with no such act, and no such reason, is a fault.
- **The link is told,** where the tree shows the link's state, by jig's
  link part (`jig-link`) or the application's own presentation of it,
  whose region its world names to the referee: while the engine was
  unreachable past a heartbeat, the person was shown offline. An
  application that shows neither is not held to it, and the person is
  not told (pages.md, section 2).
- **A delivery taken is told:** whenever the engine has no input left
  to take, each delivery a client took has reached the engine's core as
  taken (`../../protocol/client.md`, section 7), whatever ended the
  delivery's watch meanwhile. Only the engine's restart in between
  excuses it: the restarted core holds no watch to tell.
- **The page's outline:** every tree a person saw keeps skein-ui's
  outline (view.md, 3.1; skein's `ui.md`, section 14): the page node the
  root, its children landmarks or dialogs, one main, at most one banner
  and one content info, no shortcut twice, Escape only inside a dialog.
  The referee runs `skein_ui::outline::check` on it, so an application's
  tree is held to it as jig's is. jig adds no rule: that jig's default
  arrangement keeps its banner and `after_page`'s parts last is
  jig-view's step tests' to check (section 2).
- **Bounded:** the client and the view within their limits, the heap
  within their worst case.

## 7. Faults

Drawn from the seed:

- a watch dropped; a delivery slow;
- the engine restarting: offline, then answers by key;
- busy and not ready answers;
- no answer: a connection lost after the engine committed;
- a double press; two tabs deciding one thing;
- a reload: the client's own restart, its state gone but for its small
  store;
- a press on a node just removed; an object gone under its card;
- signed out with requests pending; another party signing in on a client
  with kept work;
- the engine outside the client's versions.

## 8. What an application adds

- its families' scripted documents and requests, in its client world;
- its client and view roots, in its client world and in its joint world
  on its conformance world;
- its stories, written in its people's terms;
- its trees, whatever its arrangement, held to skein-ui's outline by the
  same referee;
- where it presents jig's objects itself, jig's check run by the referee
  on every tree a person acted on (section 6; view.md, section 4), and
  its stories deciding through its own presentation;
- where it shows the link's state its own way, its region named to the
  referee, so the link is told (section 6);
- its stories at the text shell, where it has a terminal client;
- its drawing code's golden scenes, where it draws (view.md, section 17);
- its browser tier, with its binary and its bundle, over skein's harness
  and face: skein's stories for a browser shell and jig's, called with
  its own binary and bundle (section 3), and its pages, its drawings,
  its elements and components, its signing in
  (`../../protocol/browser.md`, section 15);
- its own shells' tiers, if it writes shells.

## 9. Budgets

The client's tests fit the suites' budgets (testing-strategy.md,
section 8; `../../domain/testing.md`, section 9): the focused suite runs
a few seeds per story and each fault once, the fuzzy suite sweeps random
people over random stories. At the text shell, the focused suite runs
each story once, on one seed: in jig's client world, two stories,
starting a chat and a held chat released; in an application's, those
its terminal client names. The browser tier is a suite of its own,
nextest's `--profile browser`, run by jig's gate after it builds jig's
test bundle, within a minute on the slowest machine (a rule: its
profile's global timeout, in `../../../development/workflow.md`).

## 10. Open questions

- **The joint world's cost:** whether every conformance scenario should
  also run with people in front, or only the stories about people.
- **Coverage of the roots' arms,** as for the engine's root
  (`../../domain/testing.md`, section 10).
