# Objects and cards

Provisional, 2026-10-09, revised 2026-10-21. The objects the reference
UI (`jig-client-view`) shows, each as one card wherever it appears:
tasks as chips, proposals, escalations, questions, person tasks,
results, refusals, the conversation; and the requests a party makes
about them, each saying what it will do before it is sent, and the
operations it sends. What each card shows comes from the engine's
documents (contract.md), a decision's effect among them, read with the
helper `facts` (client.md, section 19); how the reference UI holds it is
view.md, section 18; how it is drawn is view.md. An application that
does not take the reference UI presents these objects from the same
documents itself (section 2). What is still open is listed in
section 13.

## 1. In one page

- **One object, one card.** Each object that asks something of a party (a
  proposal, an escalation, a question, a person task) and each result has
  one card, shown wherever it is relevant, deciding the same thing. A
  goal a chat proposes shows in the conversation and in the inbox;
  accepted in one, it is gone from the other.
- **A card says what deciding will do,** before it is sent: what an
  acceptance makes and who funds it, what a release starts again, what a
  cancel ends. Each is data in the object's document (contract.md, 5.4),
  so the card reads it and computes none of it.
- **A card shows its own request,** as being decided until its answer,
  then as decided, refused or decided first by someone else, saying by
  whom and how.
- **A card offers only what fits:** what the object's document says the
  party may do: made, proposable and to whom, or refused and why.
  Authority is the engine's to judge, and it judges again.
- **One act, one request.** A card's act sends one of jig's operations,
  or one compound request where the person's one act is several (amend
  and release; close a chat with what becomes of its delegates), never
  two requests in turn (section 11; contract.md, 3.4).
- **Facts in, words out.** A card's facts are typed values from the
  engine's documents; its words are `words`' for jig's enums and the
  reference UI's phrases for its sentences (view.md, section 7); an
  application's values are parts in its slots (root.md, section 6).
- **The reference UI's cards; anyone's facts.** A card is how the
  reference UI presents an object. An application may present the same
  object its own way, from the same document, with the same facts and
  its own confirmation; jig's promises hold below any tree, and jig's
  check holds its worlds to what jig's cards show (section 2).
- **No undo.** What can be reversed is reversed by another request (a
  stopped task released, an amendment amended back); what cannot (a
  cancel, an acceptance, a rejection) says so before it is sent.

## 2. Cards

- **Its states** are the reference UI's (view.md, 18.3), shown on the
  card wherever it is.
- **Its place** (the inbox, a conversation, a task page, an
  application's page) changes its line of context only: in the inbox,
  its project and its task's lineage; in a conversation, nothing more.
  Never its actions.
- **Whole, wherever it shows.** A card is where jig's promises show: its
  state, its acts, its refusals, its confirmation. jig's card shows its
  object whole, wherever jig's card function builds it (view.md, section
  4), in jig's sections or in an application's slot; an application
  chooses where it goes, restyles it and fills its slots.
- **Or presented by the application.** An application may present one
  of jig's objects itself instead, in nodes of its own on a page of the
  reference UI, or with no reference UI at all, from the object's
  document as the client delivers it. jig's promise holds without jig's
  nodes:
  - *held below any tree:* a decision names the revision of the document
    it was decided on, and a revision that moved is refused
    (contract.md, 5.2); made once, nothing lost and nothing reported
    answered before it is durable are the client's and the engine's
    (client.md, section 6), whatever the tree;
  - *given as data:* what a decision will do (what an acceptance grants
    and costs, and the pool after; what a release does; what a cancel
    ends) is in the object's document (contract.md, 5.4), and the helper
    `facts` reads it, with the card's other facts (a refusal and why,
    whether the act is dangerous), for the application to say,
    with `words` or its own words, where it likes (client.md, section
    19);
  - *the application's to show:* the reference UI confirms every
    deciding act (below); an application presenting an object itself
    confirms as it chooses (contract.md, 5.4), and shows the rest of what
    a card shows its own way;
  - *checked, not mandated:* jig's check (testing.md, 6.2), which an
    application's worlds run, never at run, finds a decision whose
    revision no document the client delivered gave, and, on a tree, a
    deciding act made on a tree that did not show its facts. The referee
    checks jig's cards the same way, wherever they show.

  On a page of the reference UI, an application's own nodes may bind
  their acts through the reference UI's binding constructors, so they
  go through its confirmation and its intents as a card's do (view.md,
  section 5).
- **Its revisions:** an act names the revision of what it decides, as
  the card showed it (view.md, 18.4; contract.md, 5.2); one that moved
  under an open confirmation is refused in place (view.md, 18.5).
- **Its confirmation** (view.md, 18.5) opens on any act that decides
  something, and says what will happen, from the object's document; an
  act `facts` marks dangerous (client.md, section 19) is marked so.
- **Changed while read.** A card follows its object live: a proposal
  passed up, a held task's reason that changed, a person task taken by
  someone else.
- **How it stays live** is the reference UI's: view.md, 18.3.

## 3. The chip, and phases in words

A task named anywhere (a plan, an entry, a history line, a turn) is a
chip: its number, its title, its executor's kind, its phase, kept live
while it shows, linking to its page.

| The engine's phase | The client says |
|---|---|
| waiting | waiting for T2, T3: each a chip, with its phase; or for a hold: "waiting for a slot in staging" |
| active, an agent: idle | waiting on what its task says it waits on (a party, its delegates, news), and what would wake it, from its wake policy |
| active, kept by its dependencies | kept until T7, T8 are done: each a chip, with its phase, and who added it and when ("kept until T7 is done, added by P at 10:42"); a run still live is shown as running is, and then that it parked (`../../domain/tasks.md`, 5.7) |
| due, preparing, claimed | starting |
| running | running: writing, or a tool (which, when its host names it), on its host's last report (`../../domain/hosts.md`, 6.5), and working otherwise; its attempt, for how long |
| backing off | retrying at a time, and why: "host lost, try 2 of 3" |
| active, a procedure | its connector's words, a part: "waiting for checkout to be healthy" |
| active, a party | waiting for an answer from P, or from the person on call, since when |
| closing | closing, and what it waits for: its delegates still closing, and each connector it waits on, by name: "closing: 2 delegates, and infrastructure's effects" |
| held | held, and why (section 5) |
| done, failed, cancelled | ended so, when, and by whom for a cancel |

A closing task's effects are its connectors'
(`../../domain/connectors.md`, 4.2): the chip names each connector it
waits on to settle, in the connector's own words (a part), and never
counts their effects. How many, and which, the connector shows in its
panel on the task's page (pages.md, 5.2).

## 4. Proposal

What a task or a party may not do alone, waiting for a holder of the
authority it needs (`../../domain/tasks.md`, section 9;
`../../domain/authority.md`, section 9).

- **Who proposes,** as a chip or a party, and **why,** in their words.
- **The action,** by kind:
  - *a batch:* each task it would make, with its executor, its budget,
    its deadline, its dependencies among the others ("after R1"), and
    its grants in words, each a kind on a resource pattern (parts); and
    whether the tasks would be the accepter's own, as a goal a chat
    proposes is;
  - *an effect:* what it would do and where, as its connector describes
    it (a part);
  - *a widening* of the proposer's own authority: what it has, and what it
    asks for ("budget 10 → 25"; "also restart services in production");
  - *an amendment* beyond its amender's authority: the task, and the
    change, before and after.
- **What it needs, and who would fund it:** "funds 40 from your pool: 120
  left this period, 80 after", from the proposal's document
  (contract.md, 5.4).
- **Where it has been:** passed up by whom, or past which holders that
  could not cover it, and why ("I may not restart services in
  production").
- **Its actions,** each a `Decide`:
  - *accept,* when the party covers what it needs; otherwise not offered,
    and the card says what they lack;
  - *reject,* with a reason, which the proposer hears;
  - *pass up,* when there is a holder above the party; at the parties the
    policy names there is none, and it is not offered.
- **Accepted,** it shows what it made: chips for a batch's tasks, "the
  goal is yours", what an effect did once it is made.
- **A party's own** comes from a goal or an amendment whose draft went
  beyond what its act was offered with (section 11; pages.md, 6.2): the
  form said, before anything was sent, that it would be proposed, and to
  whom, and the reference UI sent a `Propose`, the party its proposer
  (`shapes::set_goal` gives one for a goal). Where the project's policy
  names nobody who could accept it, the form says so and sends nothing;
  the engine refuses such a `Propose` at admission as lacking authority
  (section 9).
- **Withdrawn** by its proposer, a party's own leaves with a `Withdraw`,
  offered on the card to its proposer alone.

```
Proposal from I, incident: checkout errors                waiting 4 min
  Restart checkout in production
    infrastructure: restart, keyed; checkout's other replica is healthy
  Why: "Error rate at 30 % since 10:02; the last deploy is the cause …"
  Funds it from your authority as the person on call
  Accept · Reject…
```

## 5. Escalation, and holds by reason

A held task whose escalation reached the party
(`../../domain/tasks.md`, 5.5 and section 9).

- **The task,** as a chip, and **why it is held,** in words, with what
  tells the story (the table below).
- **What it had done:** its phase before the hold; its result, if it was
  closing.
- **Where it has been:** which holders it passed, and why; or "I has it"
  while a task above may still decide it. A party in whose tree the
  task is, or whose role in the project allows it, may release, amend or
  cancel it wherever its escalation waits; leaving it held or passing it
  up is the escalation's holder's, while it waits for them; the first
  commit decides (contract.md, 5.2; `../../domain/people.md`, 5.1).
- **Its actions,** those that fit its reason:
  - *release,* a `Release` naming its hold's revision, saying what will
    happen ("its next run resumes its conversation; tries count again
    from zero");
  - *amend and release,* one compound request, the amendment then the
    release, when releasing it as it is would only hold it again: a
    budget widened, a deadline moved, instructions changed, a failed
    dependency dropped;
  - *leave it held,* a `Decide` with a reason: the entry leaves the
    inbox, and the task stays held where it shows, saying why;
  - *cancel* it, and everything below it (section 11);
  - *pass up,* a `Decide`, when there is a holder above the party.

| Held because | The card says | It offers |
|---|---|---|
| out of tries | the class of failure, and each failed attempt's reason | release; amend its instructions and release; cancel |
| stalled | what it waited for, and since when | release; for a person task, address it to someone else; cancel |
| a dependency failed or was cancelled | which, and why | amend it to drop that dependency and release; cancel; start a chat about a replacement |
| drift | what changed on the system, by whom, when: its connector's words (a part) | release, which does what the connector says; cancel |
| an effect failed for good | the effect, and the system's refusal (a part) | release, for its executor to decide afresh; cancel |
| an effect uncertain, of a kind that cannot be recovered | the effect, and that it may or may not have been made (a part); what to look at on the system | release, making it again; release without making it again, its executor told so; cancel |
| out of budget | what it spent, and where: by run, by delegate | widen its budget and release; cancel |
| past its deadline | its deadline | move the deadline and release; cancel |
| stopped | by whom, and when | release; amend and release; cancel |
| waiting for a hold past its bound | the resource, and who holds it | release, to wait again; amend to drop it; cancel |
| a circle of waits | the circle's tasks, as chips in order, each waiting for the next; that the hold lifts on its own once the circle breaks (`../../domain/tasks.md`, 6.7) | amend a task on the circle to drop a dependency; cancel; release, judged afresh |
| its pool lost its allocation | the pool, and its slots (a part) | release, to wait for a slot; cancel |
| its procedure needs a decision | the procedure's reason, in its connector's words (a part) | release, its counts reset; amend and release; cancel |

- **Held while closing,** the card shows the result it has kept; releasing
  it carries on closing.
- **Widening a budget** is funded by whoever gives it: the party's pool
  when it is within their authority, the accepter's when it is
  proposed; from then the giver is the task's funder
  (`../../domain/authority.md`, section 7).
- **A person deciding an uncertain effect** decides whether to make it
  again, which only someone who looks at the system can
  (`../../domain/connectors.md`, 4.3); the card says where to look, as
  its connector's part, and that the engine will do as they say.

## 6. Question

A task the party asked for asks them something (`../../domain/tasks.md`,
8.1): a coordinator unsure of the scope, a chat waiting for an answer.

- **The task** and its words, whole, never cut;
- **its context:** the task's page, and what it did just before asking;
- **an answer,** in words: an `Answer` to the question's entry, which
  wakes the task.

A question from a chat shows in the conversation too, answered in either
place.

## 7. Person task

A task whose executor is the party, or a role they hold
(`../../domain/people.md`, section 8).

- **What is asked:** its spec, and who asked (a task, a procedure, a
  party).
- **The results it depends on,** side by side when there are several: a
  choice between two reports shows both.
- **Its contract,** as the form that answers it, an `Answer` naming the
  person task's revision:
  - *a choice* among the options its spec lists, with words to say why;
  - *an approval* of a state its procedure names, which its connector
    shows (a part); the answer names the state it approves, and is
    refused as moved unless that state is still the contract's, so an
    approval is never given to a state the party did not see; declining
    it is *cannot do it* (contract.md, 3.2);
  - *an answer* in words;
  - and, always, *cannot do it,* with a reason: a failure.
- **Addressed to a role:** *take it* (a `Take`), so that two do not
  answer at once; the others see who took it; *hand it back* (a
  `HandBack`).
- **Its stall:** when it will become news for its requester.
- **Kept by its dependencies** (`../../domain/tasks.md`, 5.7): the card
  names the pending ones, as chips, and offers no answer until they are
  done; it may still be taken.

The answer is the task's result: a choice reaches the tasks that depend
on it, an approval the procedure that asked for it.

## 8. Result and reply

- **A result** shows its task's ending, and its result by its contract
  (`../../domain/tasks.md`, section 3):
  - *a report:* its words;
  - *a verdict:* which one, its fields, and the follow-ups it made or
    proposed, as chips or a proposal card;
  - *a connector's result:* the task's words that summarise it, and the
    connector's own (a part);
  - *a choice:* which, and who made it.

  A failure says why; a cancel says why, by whom, and what had been done
  before it.
- **A reply** names the chat and its first words, and opens the chat at the
  first turn not yet read.

## 9. Refusal

What a request asked, and why it was refused, from the refusal's facts
(contract.md, section 4):

- **the role,** or the authority it lacked, said as what was missing
  ("your pool has 12 left; this needs 40"), and whether a proposal could
  be accepted, and, from the offer, by whom (contract.md, section 4 and
  5.3);
- **a limit:** the project's live tasks, the inbox full, the chat's words
  too long, a compound request's operations past their bound;
- **a circle:** the tasks that would wait for one another, as chips in
  order ("T5 would wait for T7, which waits for T4, which ends after its
  delegate T5");
- **a dependency:** one added that failed or was cancelled, and how; or
  a person task's answer while it is kept, and what it waits for;
- **no one to escalate to:** the party would become the requester of a
  task (a chat, a goal, a task taken over) whose escalations nobody in
  the project would hold;
- **moved:** the object changed since the card showed it, and how; the
  card shows it as it is, to look at again;
- **ended:** the task or proposal is gone, and how it ended;
- **decided first,** by whom, how and when;
- **for a compound request,** the first operation that failed and why;
  nothing of the request was made.

Proposing is explicit (contract.md, 3.5): the reference UI sends what
the offer said, and, where a `Create` or an `Amend` is refused for want
of authority and the refusal says a proposal could be accepted, offers
to propose it, with a confirmation of its own (view.md, 18.4).

## 10. The conversation

An agent task's activity, turn by turn, across every run it has had:
the helper `thread`'s `Thread`, which merges committed turns, the
party's words, work summaries, proposals and provisional text in order,
across windows and gaps (client.md, section 19):

- **the words parties wrote to it,** whole, each saying where it is:
  being sent, sent, read;
- **its replies:** what the LLM wrote, as Markdown (view.md, section 8);
- **its work,** folded to a line each, opened for detail: a tool call,
  what it read, ran or changed and its outcome; its sub-agents;
- **its calls to the engine, as cards:** a delegation as the chips of the
  tasks it made; a proposal as its card; a message to a delegate; an
  effect and what it made; a note and a link to it; a decision and what
  was decided (`../../domain/engine.md`, 7.2 and 7.3);
- **the proposals waiting in its subtree** (its task's document,
  contract.md, 5.1) that no call or decision of it shows, as their cards
  after its lines: a delegate's proposal, which none of the
  conversation's calls made, and one whose call names nothing;
- **what it heard besides parties:** its delegates' results as result
  cards, and the news it was woken by;
- **where its runs begin and end,** as quiet lines: parked; resumed;
  started fresh from a brief carrying the conversation's tail, which
  tells the person the agent may have lost detail of what came before;
- **what each turn spent.**

What streams in before a turn commits shows as it comes, marked in
progress, and gives way to the turn once committed.

## 11. Steering

The requests a party makes on a task (`../../domain/people.md`, 5.1),
each offered where it fits and each saying, before it is sent, what it
will do:

- **Write:** words to an agent's task, a `Message`, which wakes it. Not
  offered to a procedure, which decides on facts, nor to a party's task,
  which is answered. Words to a task the person stopped release it too,
  in one compound request: the `Message`, and a `Release` naming the
  hold's revision the card showed.
- **Amend:** an `Amend` naming the spec's revision: its spec; its wake
  policy; its dependencies, added and removed until it closes, the form
  saying what an add does (a waiting task waits longer; an active one is
  kept, its live run finishing its turn and its next waiting for them:
  `../../domain/tasks.md`, 5.7); its authority,
  narrowed freely and widened within the party's own (beyond it,
  proposed); its charter's instructions. A procedure's parameters are
  its connector's, amended through its form in a slot, as the
  application's own request (root.md, section 6). The form shows before
  and after, and what the task will do on hearing it: a live run hears
  it; a narrowing that takes a grant its run is using cancels that run,
  and the next starts narrower. It compares the draft with the bounds
  the act is offered with (contract.md, 5.3): a change that widens no
  authority is made; one that widens anything is judged whole, against
  the authority the party may give alone, the budget they may add and
  the grants they hold, and beyond those is proposed, and to whom (a
  `Propose` of the amendment), or refused; past the policy's ceiling it
  is refused, and nothing is sent. The form says which before it is
  sent. Not offered once the task is closing.

  An agent task runs with its charter's instructions until an amendment
  gives it its own. Amended instructions are the task's whole
  instructions, and empty ones leave it with none, not with its
  charter's; left out, they change nothing of them. The form shows the
  task's effective instructions: its own, or its charter's while it has
  none of its own. A pending proposal's before and after show the
  effective text; an older history entry shows the instructions as the
  task held them, its own or none of its own, never which charter's
  text was in effect (`../../domain/tasks.md`, section 7).
- **Stop:** a `Stop`; halts its live run at once and holds the task,
  stopped by the party; its delegates go on, and what its run committed
  stays. Releasing it carries on from there.
- **Release:** a held task, by its reason (section 5).
- **Cancel:** a `Cancel`; ends the task and its subtree, deepest first.
  Before it is sent, the card says what it ends, as the task's document
  gives it (contract.md, 5.4): the tasks below that it cancels,
  counted by phase; the live runs; the proposals withdrawn; and, in
  jig's words, that effects not yet made are withdrawn and those in
  flight settle. The core never counts a connector's effects, and its
  closing asks every connector to settle the tree's (section 3;
  `../../domain/connectors.md`, 4.2): which, and how many, each
  connector's panel says.
- **Close,** for a chat: asks it to finish and cancels it if it has not
  within a grace. A chat with live delegates asks first what becomes of
  each: cancelled, or made the person's, funded from their pool from
  then on ("reserves 6 from your pool"). `shapes::close_chat` gives one
  compound request (client.md, section 19): a `Cancel` or a `Move` for
  each live delegate, as the person chose, then a `Cancel` of the chat
  with the chat template's grace, all made or none.
- **Take over:** a `Move`; makes the party the task's requester, so it
  outlives the task that asked for it; its budget reserved from their
  pool from then on, which the card shows, and refused if it does not
  fit, or if nobody would hold its escalations.
- **Reorder:** a goal's place on the board, priority-only
  amendments in one compound request, where the project's document
  offers reordering (pages.md, 6.2; contract.md, 5.3).
- **Chat about it:** a new chat naming the task, `shapes::chat` (pages.md,
  section 4).

## 12. An application's objects

A family's objects (root.md, section 7) follow the same rules: one card
each, wherever it shows; the same states; acts naming the revisions they
saw; nothing shown done before its answer; words from the application's
phrasebook. Their cards show on the family's pages and in jig's slots,
and jig's cards show on the family's pages.

## 13. Open questions

- **Words beyond text:** files and images, in a person's words and in a
  result; the engine's words are text.
- **A chat about a live task:** whether a chat started from a task holds
  a reference to it (`../../domain/tasks.md`, 8.5), to message it.
- **A run's brief, kept,** so that a person reads why an agent did what it
  did; the engine keeps what the brief was made from, not the brief.
- **A procedure's steps, kept,** as its log on its page.
