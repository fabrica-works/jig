# Testing the client

Provisional, 2026-10-09, revised 2026-10-22. How jig's client is tested
through its surface, and how a presentation on it is tested: the
client's world with a scripted engine, the joint world with the engine's
domain, their referee and their faults; jig's check, which an
application's worlds run; and the reference UI, tested as any
presentation on skein's UI kit is, with skein's fake person on its tree.
It applies `testing-strategy.md` as `../../domain/testing.md` does for
the engine; the tiers below the client are the client's protocol's
(`../../protocol/client.md`, section 14; `../../protocol/browser.md`,
section 15). What is still open is listed in section 10.

## 1. In one page

- **The client is tested through its surface.** Its worlds give
  `jig-client` the events an application gives it and read the requests
  it makes, as an application does (client.md, section 4). A story of
  the client is written in those terms, and needs no tree and no person.
- **Trees and the fake person are skein's and the applications'.** A
  presentation on skein's UI kit is tested as skein says, by skein's
  fake person acting on its tree (skein's `ui.md`, section 17). The
  reference UI is one: its stories run in jig's worlds, with jig's test
  application in front of the client (section 4). An application's
  presentation runs in its own worlds, on jig's.
- **The client's world comes first.** The real client, a scripted engine
  that keeps the contract (contract.md), the transport played by the
  world, and every fault of section 7. Most of the client's tests live
  there.
- **The joint world is the check.** The same stories in front of the
  engine's real domain: jig's core with its test connector, or an
  application's whole engine domain on jig's conformance world.
- **The referee watches what the application was told and what the
  engine made durable,** never the client's state: nothing reported
  answered before it was durable, ordinary deciding acts made once, and
  kept work surviving restart, with `Seen`'s exceptions explicit (6.1).
  Where there is a tree, it watches what the person saw as well
  (6.3).
- **jig's check is checked, never mandated.** In any application's
  worlds, it finds a decision whose revision was not in a document the
  client delivered to the application before it was sent; on a tree, the
  reference UI's or an application's on skein's UI kit, a deciding act
  made on a tree that did not show its facts (6.2).
- **An application's server has stories too.** The attack stories for
  the server an application writes (a cross-site request, a rebound
  Host, a framed page) are jig's, in `jig-client-fake-person-browser`,
  which an application's browser tier may run against its own
  (`../../protocol/browser.md`, section 15).
- **Every world with a tree builds and diffs every page,** and applies
  the patches to a model of the last tree, which must then equal the new
  one.

## 2. The tiers

| Tier | Real | Faked |
|---|---|---|
| step tests | `jig-client`'s step and fire, by its events and its requests; its helpers, `shapes`, `facts`, `thread` and `words`; the reference UI's step and each of its view functions; its default arrangement's banner and `after_page`'s parts last, on each of jig's pages (pages.md, 12.3); jig's check; the reference UI's pages printed by skein's text shell, by golden text (view.md, 15.6); its test family's drawing code, by golden scenes (view.md, 17.5) | their events, by hand |
| the client's world | `jig-client`, driven through its surface by a scripted application; or jig's test application, the reference UI and its view root in front of it, with the person | the engine, scripted; the transport, played by the world; the person |
| the joint world | the same, in front of the engine's domain, root and all | the store, hosts and systems, as the engine's worlds fake them; the transport; the person |
| the protocol world, the real loop | with the client's protocol layer and shells (`jig-client-protocol-http`, `jig-client-shell-native`), and the engine's clients' component with the default HTTP mapping's engine end; skein's text shell's terminal in front of the native one, on skein's simulated machine (`../../protocol/client.md`, section 14) | |
| the browser tier | the application's binary and its bundle in Chromium, the person through skein's browser face and harness (section 3) | what the binary's configuration fakes |

Every world runs on skein's world harness: one loop, one seed, the
referee in the loop, the heap counted at every iteration
(testing-strategy.md, sections 6 and 7). The browser tier is no world:
its server and its Chromium are real processes, children of the
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
runs it too. It acts on a tree, and the client has none: it plays the
reference UI's stories in jig's worlds, and an application's stories in
its own, wherever a presentation builds on skein's UI kit.

- **Its queries, acts, reports, tree face, random person and browser
  face are skein's** (skein's `ui.md`, 17.1; skein's `ui-web.md`, 14.2),
  and so is the browser tier's harness, in `skein-fake-person-browser`.
- **jig's stories find a target as a person would:** by role, name and
  text, within regions named outermost first ("the button named Accept,
  in the region named Proposal from I"); never by style, by position,
  by region name, or through the presentation's state, so a story
  written for the reference UI's pages runs on an application's
  arrangement of the same sections.
- **The browser tier's stories,** skein's for a browser shell, and
  jig's for a browser shell and its attack stories in
  `jig-client-fake-person-browser`, are `../../protocol/browser.md`'s,
  section 15.

  An application's browser shell runs its own loop over skein's parts
  and jig's, so both kits' stories check its wiring: the order its loop
  keeps, and, where it drives fetch and the small store without skein's
  transport, the rule it then takes on itself (skein's `ui-web.md`, 5.3;
  `../../protocol/browser.md`, 5.3).

## 4. The client's world

`tests/client`, package `jig-client-world`.

- **Clients:** each a tab or a device of a party: a `jig-client` with its
  small store, and the application in front of it. Several per party,
  and several parties.
- **The scripted application:** the world's stand-in for an application
  with no presentation. It gives the client its events (a start with what
  was kept, a sign-in, watches and reads by handle, operations,
  compound requests and a family's requests as bytes), carries out what
  the client asks of the application, and records each report, which the
  referee reads. The client's stories (client.md, section 17) are
  written in its terms. A random one, for the fuzzy suite, draws its
  events from the seed, so a sweep reaches what no script names.
- **The scripted engine** (`jig-client-scripted-engine`) keeps the
  contract for jig's documents: requests answered by key, after a
  commit it scripts, and answered again alike; refusals as scripted, a
  compound request's naming its first failed operation; a version not
  served answered so, naming the versions served; documents for every
  read; watches delivered level-triggered, a delivery in flight at a
  time; a run's text written out over time. It holds the truth the
  referee compares with: what it committed, and when.
- **The transport, played by the world:** it carries the client's
  exchanges and its stream to the scripted engine, and its records to
  the small store, with section 7's faults between.
- **The application's parts:** a client world for an application adds its
  families' scripted documents and requests beside jig's, through the
  scripted engine's hooks, which are ordinary Rust.
- **jig's test application,** in front of the client for the reference
  UI's stories: its client root (`jig-client-test-domain`), holding the
  client, the reference UI and one test family, and its view root
  (`jig-client-test-view`). The family's parts fill every kind of slot,
  and its page fills both of an application's: a card's, and a
  section's (a task's plan), as the test connector stands in for an
  application in the engine's worlds (`../../domain/testing.md`,
  section 7). Its page also presents one of jig's objects itself, a
  proposal, from its document as the client delivers it, saying what
  `facts` gives, with a confirmation of its own, so jig's check holds an
  application's own presentation in jig's world (6.2). It takes jig's
  default arrangement of every page, but in the story of a task's page
  arranged with every section (view.md, 18.10), whose arrangement names
  every `TaskSection`. Its test family's drawing has drawing code, golden
  scenes and an act, with an ordinary node beside it doing the same
  (view.md, section 17). Its browser shell, `jig-client-test-shell-web`,
  jig's reference shell, and its server take, once each, the steps past
  the reference that `../../protocol/browser.md`, section 15, lists.
- **Its stories:** the client's (client.md, section 17), through the
  scripted application; the reference UI's (view.md, 18.10) and
  pages.md's, section 10, each written once in a person's terms, through
  jig's test application.
- **Read cancellation at credential boundaries** uses client-domain
  step tests and the existing composed `jig-client` sign-in fixture:
  `Held` and `Transport`, successful `Link` and `Service`, response
  before and after the boundary, sign-out learned from a read, and
  reads before the first `me`. Each pending jig or family read ends
  visibly once, no read is replayed, and refusal, loss or not-ready
  without installation leaves old reads eligible. Late `Reply` and
  `Lost`, same-handle replacement, and full independent read room check
  token fencing and the output bound (client.md, sections 7, 12 and 14).
  The protocol world is used only for an ordering the lower fixture
  cannot express (`../../protocol/client.md`, section 14).
- **Shapes** preserve a template's omitted LLM intent even when its
  `resolved_default` preview names a model; the existing shapes step
  tests check that the preview never enters `Create`.
- **At the text shell.** A client of the reference UI's stories may be
  played as skein's text shell is (view.md, section 15): each tree it
  shows printed by its renderer, and each act the person makes typed as
  the line a person would type for it, from the print's numbers, then
  read back by its command reader, which must give that act on that
  node. A person's typing followed by sending is one line, `n WORDS`;
  typing alone into a field that does not submit on enter is `n WORDS`
  too; a story that types into a field that submits on enter without
  sending is not played so. The world signs its clients in, as a
  terminal's host does (view.md, 15.5). A story run so checks that a
  terminal can make every act it makes.

## 5. The joint world

The client's world, with the engine's domain in place of the scripted
one:

- **For jig:** jig's core world under its test root
  (`../../domain/testing.md`, section 4), its parties played by clients:
  the scripted application, or jig's test application with the person in
  front.
- **For an application:** its whole engine domain on jig's conformance
  world, with its client and presentation and the people of its
  stories: `ops`'s P accepting a restart from the incident's page.
- **The translation** between the client's messages and the engine's
  people and views vocabulary is the clients' component's
  (`jig-protocol-engine-clients`), which the world runs with no
  transport between, as the protocol world runs it with one.
- **The engine's faults** reach the client, and through it the person:
  crashes at commits, held commits, restarts, as the conformance world
  makes them.

## 6. The referee

It watches what each client reported to its application (its requests
to the application, and the documents it delivered by handle), what
each person saw where there is a tree (the trees, as the person read
them), and what the engine made durable (the scripted engine's commits,
or the engine's store), never a client's state.

### 6.1 On the client's surface

In every world, whatever stands in front of the client:

- **Nothing reported done before it was durable:** no answer to a
  request is reported before the commit that answers it is durable.
- **Ordinary deciding requests made once:** each is made once by the
  engine, whatever the faults: drops, restarts, a request sent twice
  under one key, reloads. New `Seen` retains no completed engine answer:
  a retry after completion checks current subject and authority and
  advances the durable watermark only forward (contract.md, 3.1).
- **Kept work survives restart:** every kept request is answered,
  reported unknown past the engine's retention, or restored and reported
  kept; every word sent and reported answered is in its task's activity,
  or its task ended. New unanswered `Seen` is silently lost on closing,
  reload or crash, and the next opening checks and sends the watermark
  again. While the client lives it keeps the same pending lifecycle and
  one answer. A request carrying a value follows client.md, section 6.
- **Kept requests stored before sent, for their party:** no kept request
  is sent before its record is stored, and no request is sent for a
  party `me` has not confirmed. `Seen` has no save of its own but still
  waits behind writes already given (client.md, sections 6 and 11).
- **Documents of what was committed:** every document the client
  delivered for a handle reflects only commits the engine made durable,
  and none after a newer one of the same subject reflects an older
  state.
- **A delivery taken is told:** whenever the engine has no input left
  to take, each delivery a client took has reached the engine's core as
  taken (`../../protocol/client.md`, section 7), whatever ended the
  delivery's watch meanwhile. Only the engine's restart in between
  excuses it: the restarted core holds no watch to tell.
- **No watch left on the engine:** whenever the engine has no input left
  to take, each watch it holds for a client is one that client has open,
  for its application's handles or for `me` (client.md, 8.1): none it
  closed, none of a stream it dropped, none whose opening it lost.
- **Bounded:** the client within its limits, the heap within its worst
  case.

Existing client-domain request step tests cover `Seen`'s consumed table
save with no `Down::Save`, parked-to-live sending, retries under the same
engine key, application-key joins and conflicts, and named `Send` held
while its name is held. Existing composed-client store tests contrast
`Seen` alone, which needs no own `Stored` before its exchange, with
ordinary and mixed requests: kept requests wait for `Stored`, failed
writes answer only the ordinary records they carried as not kept, and
held `Seen` exchanges are lost and retry. Final answers and retention's
unknown outcome still erase; acknowledging an absent-key `Seen` erase
then a later ordinary save must allow that request to send, with no
tracked write left waiting. Legacy stored pending `Seen` records decode,
restore as kept, never save again, and erase normally at terminal; the
existing link codec's legacy `Seen` record fixture remains readable.
These proofs use the existing tiers, harnesses and budgets.

### 6.2 jig's check

jig's check is its test kit's, in `jig-client-test-kit`: the referee of
`jig-client-world`, on which an application's client worlds run, runs
it in jig's worlds and in each application's. It is checked, never
mandated: nothing checks it at run, and an application that does not
run it is not held to it.

- **Decided on what was delivered:** every decision that names a
  revision named one in a document the client delivered to the
  application, for that object, before the decision was sent. A
  revision the application made up, kept from another client, or never
  received is a fault, whatever the engine answered.
- **A decision was seen,** on a tree: every deciding act a person made
  was made on a tree that showed its decision's facts (what `facts`
  gives from the object's document: client.md, section 19), in the open
  confirmation or within the acted node's nearest named region. It runs
  on every tree a person acted on, the reference UI's or an
  application's on skein's UI kit, so an application that presents
  jig's objects itself is held to what jig's cards show.
  - *Said in the application's words:* the referee says each fact as
    `words` says it in the phrasebook the application's world gives
    it: jig's English for the facts, with the application's overrides,
    phrase by phrase. A world that gives none is held to jig's English.
    The referee holds no words of the reference UI's, so an application
    that words a fact its own way is held to its own wording, and to
    no other.

### 6.3 On what the person saw

Where a presentation builds a tree, the reference UI's or an
application's on skein's UI kit:

- **Nothing shown done before it was durable:** no outcome of a request
  is shown before the commit that answers it.
- **Nothing sent lost, as the person saw it:** every word a person sent
  and was told was sent is in its task's activity, or its task ended.
- **What they saw is what they decided:** every act that names a
  revision (contract.md, 5.2) names the one the card the person acted on
  showed, or the application's presentation of the object.
- **Offers kept:** a draft within the bounds its act was offered with is
  made, and one beyond them is proposed or refused as the offer said,
  unless a newer document of the object shows its facts moved in between
  (contract.md, 5.3).
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
  unreachable past a heartbeat, the person was shown offline. A tab's
  page that starts, on a reload too, is judged afresh from then, whatever
  an earlier page owed: its stream has only its own heartbeat until a
  hello (client.md, section 8), so it tells the link, live or offline,
  within the bound of a link coming back. An application that shows
  neither is not held to it, and the person is not told (pages.md,
  section 2).
- **A sign-out told:** a sign-out the engine did not receive is never
  shown done (view.md, 18.6). After a person asks to sign out, the
  sign-in page is a fault while the engine still holds the sign-in they
  asked to end. Once the engine has ended it, by that sign-out or by an
  end of its own (an expiry, a revocation, another client's sign-out of
  it), before the ask or after it, the page shows that end, whatever
  became of the ask. The world observes each end of a sign-in where the
  engine makes it, as it observes its commits, never from a client's
  link.
- **The page's outline:** every tree a person saw keeps skein-ui's
  outline (skein's `ui.md`, section 14). The referee runs `skein_ui::outline::check` on it, so an application's
  tree is held to it as jig's is. jig adds no rule: that jig's default
  arrangement keeps its banner and `after_page`'s parts last is the
  reference UI's step tests' to check (section 2).
- **Bounded:** the reference UI and its view within their limits, the
  heap within their worst case.

## 7. Faults

Drawn from the seed:

- a watch dropped; a delivery slow;
- the engine restarting: offline, then answers by key;
- busy and not ready answers, a watch's and `me`'s among them;
- no answer: a connection lost after the engine committed;
- a request sent twice; two tabs deciding one thing; a double press,
  where there is a tree;
- a reload: the application's own restart, its state gone but for the
  client's small store and its presentation's records;
- a save the small store refuses;
- a press on a node just removed; an object gone under its card;
- signed out with requests pending; another party signing in on a client
  with kept work;
- the engine outside the client's versions.

## 8. What an application adds

- its families' scripted documents and requests, in its client world;
- its client root and its presentation, in its client world and in its
  joint world on its conformance world;
- its stories, written in its people's terms: through the client's
  surface where it has no tree, through the person where it has one;
- jig's check, run by the referee in its worlds (6.2): every decision on
  a revision the client delivered; on a tree, every deciding act made on
  a tree that showed its facts;
- its trees, whatever its arrangement, held to skein-ui's outline by the
  same referee, where it builds on skein's UI kit (6.3);
- where it shows the link's state its own way, its region named to the
  referee, so the link is told (6.3);
- its stories at the text shell, where it has a terminal client;
- its drawing code's golden scenes, where it draws (view.md, section 17);
- its browser tier, with its binary and its bundle, over skein's harness
  and face: skein's stories for a browser shell and jig's, called with
  its own binary and bundle (section 3), and its pages, its drawings,
  its elements and components, its signing in
  (`../../protocol/browser.md`, section 15);
- jig's attack stories, run against its own server, if it runs them
  (`../../protocol/browser.md`, section 15);
- its own shells' tiers, if it writes shells.

## 9. Budgets

The client's tests fit the suites' budgets (testing-strategy.md,
section 8; `../../domain/testing.md`, section 9): the focused suite runs
a few seeds per story and each fault once, the fuzzy suite sweeps random
people and random applications' events over random stories. At the text
shell, the focused suite runs each story once, on one seed: in jig's
client world, two stories, starting a chat and a held chat released; in
an application's, those its terminal client names. The browser tier's
suite and budget are `../../protocol/browser.md`'s, section 15.

## 10. Open questions

- **The joint world's cost:** whether every conformance scenario should
  also run with people in front, or only the stories about people.
- **Coverage of the roots' arms,** as for the engine's root
  (`../../domain/testing.md`, section 10).
