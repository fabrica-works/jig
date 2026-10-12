# The client domain

Provisional, 2026-10-09, revised 2026-10-12. `jig-domain-client`: what a
party's client knows of the engine's work, and what it does with it. It
is the client's counterpart of the engine's core: a child of the
application's client root, holding jig's part of the client: the frame
and the link, jig's pages and the objects they show, the person's drafts
and confirmations, and what a page reads and watches. How the root
composes it with the application's own objects is root.md; what it shows
is pages.md and objects.md; what it asks of the engine is contract.md.
What is still open is listed in section 18.

## 1. In one page

- **A step machine like every jig domain** (programming-model.md,
  section 3): its own vocabulary, limits, worst case and world,
  `#![no_std]`, depending on skein-lib alone. It runs in a browser, a
  phone, a desktop application, a command-line tool or another service;
  only the shell is the platform's (`../../README.md`, section 10).
- **The client is complete in its domain.** Everything a party sees and
  does (one card per object, keyed requests, live and offline, paging,
  drafts, folds, confirmations) is decided here, in step functions, so
  all of it runs in worlds with no protocol, no shell and no engine but a
  scripted one (testing.md).
- **One page at a time, its objects once.** The page at the address keeps
  its own state and names the objects it shows by handle; the objects are
  a working set held once, so a decision changes every place it shows
  (section 5).
- **Requests outlive pages.** A party's request is keyed, kept in the
  small store before it is sent, sent again with the same key until its
  answer is durable, and shown as asked until then, never as done. The
  keeping and sending are the client root's request table's (root.md,
  section 4); deciding what to ask, and showing it, are this domain's
  (section 6).
- **What it shows, it watches.** A page opens watches on its subjects,
  and each delivers the subject's current document whenever it changes:
  the latest, never a backlog of changes (section 8). A run's text as it
  is written is the one stream read from an offset (section 10).
- **The link is the client's to judge.** The frame's watch is always
  open; from it and from answers that never came, the client says
  whether it is live, behind or offline, and every child of the root
  follows that judgement (section 8).
- **What the person is in the middle of is state.** Drafts, the open
  confirmation and its fields, folds and filters are the domain's, so the
  view keeps none and worlds see all of it (section 9).
- **Facts, not words.** It carries what the engine said (a phase, a
  hold's reason, amounts, wall times) as typed values. Words are the
  view's, from its phrasebook (view.md, section 7).
- **It never knows an application's types.** Where one of jig's
  documents names an application's value, the domain holds the part's
  reference and asks the root to have its owner show it (root.md,
  section 6).

## 2. The client in the system

```
person ⇄ shell ⇄ view ┐                          ┌ protocol ⇄ engine
                      └──── the client root ─────┘
                            ├── jig-domain-client   the frame, the link, jig's pages and objects
                            ├── <connector>…        each connector's side: its parts, its families
                            └── the request table   keyed requests, until durable
```

- **Two translations, one per peer.** The engine meets the client
  through the protocol layer: requests, answers, documents and watches,
  typed. The person meets it through the view: the domain's state as a
  tree, and the person's acts coming back as typed actions.
- **The client has no store of the truth.** The engine's store is the
  truth; the client holds a working set of what it was last told, and a
  small store of what it must not lose across its own restart (section
  11).
- **A party may have many clients:** tabs, devices, a command-line tool.
  Each is its own client, with its own keys and its own small store. Two
  of them deciding one thing are two parties' requests to the engine: the
  first commit decides, and the second is told who decided and how
  (`../../domain/people.md`, 5.1).

## 3. Structure

`jig-domain-client` keeps, each part in its own module:

- **the frame** (pages.md, section 2): who is signed in, the projects
  they hold roles in, what needs them, the engine's versions and the
  retention of its answers, the charters it offers for chats and goals,
  and the link (section 8);
- **the page** at the address, with its own state: filters, folds,
  windows, its lead (section 5);
- **the working set:** the objects the page shows, by key, each with its
  revision and its card's state (section 5);
- **intents:** the requests this domain asked the root to send, by
  request token, and what each is about, so its card shows it as being
  decided (section 6);
- **reads in flight** and **watches open,** each ended by one terminal
  (sections 7 and 8);
- **drafts** and **the confirmation** (section 9);
- **notices,** its timers, its facts, and the seed its jitter is drawn
  from.

Its handles are skein-lib's: a page's reads, watches and objects are
`Id`s in slabs sized at startup, and what crosses the root is their
tokens (programming-model.md, 4.2). A terminal is never stale: a slot is
retired only once its terminal has come.

## 4. Its vocabulary toward the root

### 4.1 What it hears

- **The start:** where the client is (an address), what the small store
  kept of this domain, and the person's offset from universal time, for
  showing wall times. Nothing else is taken before it.
- **Going somewhere:** the shell reports a place the client went (a load,
  back, forward, a typed address); the root routes it here when the place
  is one of jig's, and tells this domain it left otherwise.
- **An act:** what a person did, as the view decoded it from the node
  acted on (view.md, section 5): go, edit, submit, intend, choose,
  confirm, dismiss, fold, filter, more, sign in.
- **An ask:** a program's intent, by value rather than by a card's
  handle: how a service's client, which has no view, asks (root.md,
  section 11).
- **A request's progress:** for each request it asked the root to send,
  that it is in flight, retrying, or parked; then its one terminal, the
  answer: done or refused, durably (section 6).
- **Kept work:** after the start, each scope the request table restored
  requests for, once, with their count; then, as the table retires each
  of them, that one of that scope's is gone (section 11).
- **A read's terminal:** the document or the page asked for, or why there
  is none (section 7).
- **A watch's events:** opened; each delivery of its subject's document;
  alive; and its one terminal, ended, saying why (section 8).
- **Time:** its deadlines, fired by the loop.

### 4.2 What it asks for

- **Send** a keyed request: the ask, under a token of its own. The root
  puts it into the request table, which keys it, keeps it and sends it
  (root.md, section 4).
- **Read** a document or a page, under a token; one terminal.
- **Open** and **close** a watch on a subject, under a token; one
  terminal.
- **Show an address,** as a new entry in the history or in place of the
  current one.
- **Save** and **erase** its own records in the small store (section 11).
- **Sign in** with one of the application's providers, coming back to an
  address (section 12).
- **Show a part:** a page of this domain names an application's value in
  a slot, and the root routes it to the part's owner, with the page's
  generation; and **parts left,** when the page goes (root.md,
  section 6).
- **The link,** whenever its state changes, which the root passes to
  every child and to the request table (section 8).
- **The party confirmed,** once the frame's first delivery names them,
  which the root passes to the request table, so that only that party's
  requests are sent (section 11).
- **Facts,** content-free, for the world and for operators.

### 4.3 What it never asks

No request carries an application's value, and no event brings one. A
part's value goes from the engine to its owner, and from its owner to the
view, without passing through this domain. Where one of jig's acts needs
an application's values (a procedure's parameters, a resource's options),
the act is the application's own request, made from a form in a slot
(root.md, section 6).

## 5. Pages and the working set

- **One page at a time.** The page at the address is one of jig's (the
  inbox, conversations, a task, a project and its sections, the
  deployment; pages.md) or one of the application's, which this domain
  knows only as elsewhere. Pages that are not about the work (starting,
  signing in, a page missing, a client too old) are pages too.
- **Going to a page** numbers a new generation; closes the old page's
  watches; abandons its reads, whose terminals are dropped when they
  come; tells the root its parts have left; clears the working set; then
  opens the new page's watches and reads, and asks for its address to be
  shown. Requests in flight, the frame and drafts carry over.
- **Objects by key, once.** Each object a card shows is held once, under
  the key the engine names it by: a task, a proposal, an escalation, a
  question, a person task, a result. One object may show in several
  places on a page (an escalation in a task's overview and in its plan);
  it is still one object, with one card.
- **Sources, not reference counts.** Each object records which of the
  page's watches and windows name it. Each delivery replaces what its
  source named: an object no source names any longer goes, unless a
  request about it is pending or it is leaving.
- **Each object carries the revisions of what may be decided about it:**
  a held task its hold's, a task its spec's, a person task its own, a
  note its own. Each moves only when what is decided on changes, never
  with spend or a delegate's phase. An act names the one it decides
  (section 6).
- **A card's state** (objects.md, section 2):
  - *open;*
  - *deciding:* a request about it is pending, shown as asked, never as
    done;
  - *refused:* the answer refused it; the card says why, and stays;
  - *leaving:* it was decided, answered, withdrawn or ended, by this
    person or another; the card says what happened, by whom and when,
    and leaves after a linger.
- **Leaving, whichever comes first.** A card leaves on its request's
  answer ("you accepted it"), or when a delivery no longer names it. In
  the second case, a read of the object's document says how it ended
  ("Pat released it at 10:14") before it goes. While a request about it
  is pending, it stays deciding until the answer, so the person learns
  how their own request ended.
- **Full is a refusal the page shows.** A delivery naming more objects
  than the working set holds keeps what fits in its order, and the page
  says how many more there are. Nothing grows.

## 6. Keyed requests

What this domain does with a request; the request table, which keys it,
keeps it and sends it, is the root's (root.md, section 4).

- **Asked once per intent.** A card's act opens its confirmation
  (section 9); confirming it asks the root to send one request, unless one
  for the same act on the same object is pending, which the second press
  joins. A double press, a retry and a reload therefore all carry the
  request's one key.
- **Shown as asked.** The card is deciding from the confirmation until
  the answer: "being decided", "retrying", "waiting for the engine",
  as the request's progress says. Nothing is shown as done before its
  answer, which comes only once its commit is durable
  (`../../domain/core.md`, section 11).
- **Said before it is sent.** Where the engine makes a proposal of what is
  beyond the party (a goal past their allotment, an amendment that
  widens), the object's document says so among the acts it offers, and
  the confirmation tells the person it will be proposed, and to whom.
  Setting a goal and amending are offered with bounds (contract.md,
  3.2): the form and the confirmation compare the draft with the facts
  the offer gives (a number no larger, a grant among those listed), and
  say whether it will be made or, beyond them, what the offer says:
  proposed, and to whom; or refused, and why (as when nobody in the
  project may accept the proposal), which the confirmation says in
  place, sending nothing. That is the engine's word read, never
  authority computed: the engine judges the request again when it comes.
- **The answer, applied.** Done: the card shows the outcome (made, or
  proposed), and leaves if its object stopped waiting. Refused: the card
  says why, from the refusal's facts (contract.md, section 4), and offers
  what fits: look at the object again, where it moved; change what was
  asked, where a limit or the party's authority refused it.
- **Unknown.** A request the table could not send again within the
  engine's retention (root.md, section 4) may or may not have been made:
  the card says so, and where to look.
- **Words are a request too.** A composer's words are sent the same way,
  shown as being sent while pending, as sent once answered, and as read
  once a committed turn took them (section 10).
- **Asks name what they saw.** A release, and a decision about a held
  task, name its hold's revision; an amendment, the task's spec's; an
  answer to a person task, its own; a note's correction or deletion, the
  note's. The engine refuses one whose revision has moved, saying what it
  is now, and the card asks the person to look again (contract.md,
  section 3). Other acts (accepting a proposal, which never changes;
  stopping; cancelling) name none.
- **One request per act.** What the person sees as one act is one
  request: amending and releasing a held task, words that release a
  stopped chat, closing a chat with what becomes of its live delegates.
  The client never chains two requests, so it never has to undo the
  first when the second is refused (contract.md, section 3).
- **Bounded.** The intents this domain keeps, and the table's room, are
  limits; an act past them is refused in place ("too many requests in
  flight"), never queued.

## 7. Reads and windows

- **A read** asks for one document (a task, a proposal, a result) or one
  page of a list (an inbox's group, a task's history, a conversation's
  turns, a project's ended goals), and has one terminal: the document or
  page, or why there is none: unknown, refused, gone, or unreachable.
- **What is live is watched; what is past is read.** The engine holds
  live work only (`../../domain/engine.md`, 5.3), so ended tasks, history
  and transcripts are pages read from its store, a page at a time, as the
  person asks.
- **A window** is a page's bounded view onto a list too long to hold: its
  rows, and at each end either the end of the list or a cursor to read
  more from. The subset has no generic items, so each kind of row has its
  window (objects, conversations, turns, history lines), each a few lines
  over skein-lib's `List`, sharing the edges and the sliding rule:
  - a window at its limit that reads more drops rows from its other end,
    keeping the cursor to read them again;
  - a live list adds rows only while its window reaches the end they are
    added at.
- **Abandoned with their page.** A read the page no longer wants is
  marked abandoned, and its terminal is dropped when it comes. A full
  slab of reads refuses the read in place, and the page says so.

## 8. Watches and the link

### 8.1 Watches

- **A watch** follows one subject (contract.md, section 7): the frame, a
  task, a task's tree, the chips a page names, a party's inbox, a
  project's goals, a project's tasks, a party's conversations, a run's
  live text, the deployment. Its deliveries are documents, each the subject's whole
  current state within the watch's window, and each replaces the last.
- **Level-triggered, as facts are** (`../../domain/core.md`, section 4).
  A watch never carries a change to apply to what came before, so it
  cannot miss one: what changed while a delivery was in flight is in the
  next. A slow client sees fewer deliveries, never a gap.
- **Its states:** opening (asked; its first delivery to come), live,
  reopening (dropped; armed to open again after a backoff), closing (the
  client closed it; retired on its terminal).
- **Alive.** A watch with nothing to deliver is kept alive below the
  domain; past its heartbeat with nothing heard, the domain takes it as
  dropped.
- **Ended,** each saying why: the client closed it; it was dropped; the
  engine refused it (unknown, not allowed, busy); its subject ended and
  left the engine (a task ended and was forgotten); signed out.

### 8.2 The link

What the frame shows everywhere (pages.md, section 2):

- **starting:** before the frame's first delivery;
- **live:** the frame's watch is live;
- **offline:** the frame's watch dropped, or went silent past its
  heartbeat. Requests wait, parked, and watches reopen after backoffs
  drawn from the seed: a first span, doubling to a most, jittered. The
  frame's watch opening again brings the link back, which sends the
  confirmed party's parked requests again.

And, beside the link, the page's own state:

- **behind:** a watch of the page reopened and its first delivery has
  not come: what the page shows may be old, and says so. Whoever owns the
  page says so: this domain for its pages, an application's child for
  its own (root.md, section 5).

- **The link is judged from the frame's watch alone,** which is always
  open, so it does not depend on which page is shown or whose it is, and
  its heartbeat is the client's check that the engine is there. A request
  that got no answer while the link is live is sent again after a
  backoff, and does not by itself make the client offline (root.md,
  section 4).
- **Every child follows the link.** This domain tells the root each
  change; the root passes it to the application's children, which open
  their watches while it is live and reopen them when it comes back, and
  to the request table, which parks and resumes (root.md, section 5).
- **A restart of the engine** is a moment offline, then answers by key:
  provisional text streamed before it, never committed, is gone from the
  page as the next delivery replaces it (section 10).

## 9. Drafts and confirmations

- **A field** is something a person types into, named by what it is for:
  the new chat's words, a task's composer, the open confirmation's
  reason, an amendment's fields, a new goal's. It keeps the text, and a
  count of the domain's own writes (a composer cleared once its words
  are sent, a draft restored), so the view writes a field's value only
  when that count moved and the person's typing is never overwritten
  (view.md, section 6).
- **Composers keep their drafts per task,** across pages and reloads,
  bounded by the drafts kept; the oldest untouched goes first.
- **One confirmation at a time:** the act, the object and the revision
  the act names, as it was when the confirmation opened, the choices
  made (an option of a person task, what becomes of each live delegate
  on closing), and what is missing (a reason; a draft its offer's
  bounds say would be refused, section 6). What it says comes from the
  object (what an acceptance funds, what a cancel ends, what a release
  will do), so the view can say it without asking anything (objects.md).
- **A revision that moves under an open confirmation** marks it changed:
  a held task's hold, under its release. Confirming it then is refused in
  place, saying what changed, rather than sent. What the act does not
  name (spend, a delegate's phase) changes the confirmation's words, not
  its validity. Bounds that move under an open confirmation, as a newer
  document gives them, are compared with the draft again, and the
  confirmation says what it will now be: made, proposed, or refused.
- **A confirmation's fields** are cleared when it closes, confirmed or
  dismissed.

## 10. Conversations

What a chat's page leads with, and every agent task's activity
(objects.md, section 10):

- **Committed, read in pages from the end.** A task's activity is a list
  the engine keeps in the order it committed: the words parties wrote to
  it, each run's beginning and end, its turns, the results and news it
  heard, the decisions made about it (contract.md, section 6). The
  client reads it a page at a time, newest first, as the person scrolls
  back.
- **Turns, decoded below the domain.** A turn is bytes of smith's
  vocabulary, which the engine never reads (`../../domain/engine.md`,
  7.2). The client's protocol layer decodes it with smith's transcript
  crate (smith's `protocol/transcript.md`) into the client's own turn: its
  messages; their blocks of text, calls and results, each call named by
  its tool with its input as written and its outcome summarised; what it
  used and spent. The domain holds that turn, typed and bounded, and
  links nothing of smith's.
- **Live, then replaced.** While a run is live, the page watches it: the
  turn being written, named by its attempt, its number within it and its
  completion's retry, its text from the offset the client has, and the
  call it is making. What is shown live is provisional, marked so, and
  kept under that name until the committed turn of the same attempt and
  number replaces
  it: the task's watch says its activity has moved, and the client reads
  what is new (contract.md, 7.3). Text of a later turn never resets an
  earlier one; a later retry of the same turn replaces the text of
  its earlier retries. A watch opened again starts from the beginning of the
  turn being written: the client keeps the text it holds of that turn, and
  takes what goes past it. A turn lost with its host never commits: its
  provisional text goes when the run ends, and the next run starts after
  the last committed turn. A run whose host gives no text and names no
  call (contract.md, 7.3) shows none live: its words appear as each turn
  commits.
- **A person's words, in three states:** being sent, while their request
  is pending; sent, once its answer says which message they are in the
  task's inbox; read, once a committed turn says it read through that
  message.
- **Folded by default:** a turn's calls show as one line each, opened on
  demand; a call to the engine shows as the card of what it made (a
  delegate's chip, a proposal's card), live.

## 11. The small store

- **What a client keeps across its own restart:** the requests not yet
  answered, with their keys (the request table's, written by the root);
  the person's drafts; where the client was. Nothing else: everything
  shown is read again.
- **Records per owner.** Each child of the client root saves and erases
  its own records under its own keys, wrapped by the root, as each child
  of the engine's root does in the engine's store
  (`../../domain/engine.md`, 5.6). The shell keeps them where its
  platform allows: a browser tab's session storage, a file, a world's
  memory.
- **Kept before sent.** Within one step's outputs, the shell writes the
  small store before any request leaves, so a restart never loses a
  request the engine may have made (root.md, section 4). A write that
  fails (a quota full) stops the requests of that step from leaving, and
  their owners are told they were not sent.
- **It names its party.** Each request kept carries the party it is for
  (root.md, section 4). At start, what was kept is restored, and the
  parked requests are sent with their original keys only once the
  frame's first delivery confirms that party. A different party sees a
  warning, and the kept work stays parked and hidden until its party
  returns, or the engine's retention passes and it is told unknown.
- **The warning is this domain's.** The request table keeps the kept
  requests, and the root keeps nothing of its own, so it tells this
  domain whose work was kept: each scope the restored requests name,
  with their count, and each kept request as the table retires it,
  answered or unknown (root.md, section 8). This domain keeps a count
  per scope, bounded, shows someone else's work (pages.md, section 8)
  while the confirmed party differs from a scope it still counts, and
  leaves that page when no other party's count is left.
- **Its platform's limits:** a browser tab's session storage goes with
  the tab, so the shell warns before a tab with requests pending is
  closed; a tab duplicated copies it, and both send the same keys, which
  the engine makes once, while each draws new keys from a seed of its
  own.
- **It is never the truth.** A request it kept is answered by the engine
  by its key, made once whatever the client did before.

## 12. Signing in

- **The client holds no secret in its domain.** Who the party is comes
  from the frame's delivery. A sign-in's secret (a cookie, a token, a
  service's credential) is the shell's and the protocol's
  (`../../domain/people.md`, section 3).
- **Signed out:** an answer, a read or a watch saying so closes every
  watch, parks every request in the small store, and shows the sign-in
  page, which lists the application's providers. Choosing one asks to
  sign in, coming back to where the person was; the exchange (a
  browser's navigation, a device code for a terminal) is the shell's.
- **Signing out,** from the frame, is the shell's exchange too: it ends
  the sign-in on the engine (`../../domain/people.md`, section 3), and the
  client then shows the sign-in page, as when signed out.
- **Keys are the party's across sign-ins,** so the parked requests are
  sent again once the same party is confirmed (`../../domain/people.md`,
  5.1), within the engine's retention.
- **A service signs in at start** with the credential the deployment
  issued it, through its shell, and never sees a sign-in page.

## 13. Time and notices

- **Wall times are for showing:** the engine's records carry wall times,
  and "waiting 4 min" is the view's, from an object's wall time and the
  step's. While a page shows any, a tick once a minute counts a change to
  what is shown, so the view builds again.
- **Deadlines are the domain's:** backoffs, heartbeats, a card's linger, a
  notice's life, the coalescing of drafts' saves, on the monotonic clock
  (programming-model.md, section 9).
- **Notices** are short-lived, for what has no card: a request answered
  for an object no longer shown, a refusal with nothing to show it on,
  the link going and coming back. A card's own outcome is on the card.

## 14. Limits and the worst case

Each is configured and part of the worst case:

- objects in a page's working set; rows per window; turns a
  conversation's window holds; nodes of a plan held;
- intents kept, reads in flight, watches open (the frame's included);
- drafts kept; notices; the scopes of kept work counted, no fewer than
  the requests the table holds;
- bytes a person may write (the engine's own bound on words); bytes of
  any one text shown (a spec, a result, a turn); bytes of a provisional
  turn;
- backoffs' first and most spans, the heartbeat, the linger, a notice's
  life, the drafts' coalescing.

`worst_case` sums the containers' own and the bytes each entity may hold
(programming-model.md, 6.3), and bounds what the view builds: the view's
limits derive from these, so a tree always fits (view.md, section 10).

## 15. What it never knows

- **Bytes:** addresses as text, documents' encoding, the small store's,
  turns' bytes. Words, specs and results are carried, never read.
- **Its platform:** no browser, no window, no terminal; no scroll, no
  focus. Keeping a node in place on screen as a conversation streams is
  the shell's.
- **The engine's internals:** tasks by number, objects by key and
  revision, outcomes and refusals as the contract gives them. Authority
  is never computed here: what a card offers is what the engine's
  document says the party may do, and the engine checks it again.
- **The application's types:** its parts, its families and its
  documents are its children's (root.md).

## 16. Below the domain

What the client's protocol layer and its shells owe this domain, which
the client's protocol designs (`../../protocol/client.md`):

- **One attempt per request, read and watch.** Retries, backoffs and what
  a failure means are the domain's and the table's; the protocol layer
  runs one attempt and reports how it ended. For a keyed request, it
  makes the answer's envelope: final, with the engine's durable answer
  decoded into its owner's vocabulary; again, for busy or not ready;
  lost, when no answer came; signed out (contract.md, section 4).
- **Documents decoded whole,** into the domain's types, before they reach
  it (programming-model.md, 4.4); turns through `jig-protocol-transcript`
  (section 10).
- **Watches kept alive,** a delivery at a time, with a heartbeat when
  nothing changes.
- **Addresses** to and from what the platform shows: a browser's URL, a
  command line's arguments.
- **The small store,** encoded, written before requests leave in each
  step's outputs, and read at start.
- **Signing in:** each provider's exchange; the secret kept by the shell.

## 17. The world

`jig-domain-client` has its world, with the engine scripted and the
person acting on the view (testing.md, section 4), and joins the
engine's domain in the joint world (testing.md, section 5). Its stories:
signing in and starting a chat; a held chat released; a proposal
accepted in the inbox while its chat shows it; two tabs deciding one
thing; a reload with requests pending; the engine restarting under a
pending request; a watch dropped and the page behind; a page left with
reads in flight; a draft kept across a reload; a goal beyond the
person's allotment, proposed as the confirmation said it would be.

Its referee: nothing is shown as done before the engine made it durable;
every request the person made is made once, whatever the faults; no word
a person typed and sent is lost; every act that names a revision names
the one the person saw; nothing beyond the client's limits is held.

## 18. Open questions

- **Several clients of one person,** such as two tabs: whether drafts or
  pending requests should be shared between them, which today they are
  not.
- **Writing while offline:** whether a person may write and confirm
  while offline, their requests parked until the link is back, as today;
  or be told at once that nothing will be sent until then.
- **Prefetching:** reading a task's document when its chip is hovered or
  near the screen, to open its page at once.
- **A window's turn:** how a conversation shows a turn that opens a
  window (a compaction's summary of what came before it), beside the
  earlier window's turns it summarises.
