# Objects and cards

Provisional, 2026-10-09, revised 2026-10-15. The objects jig's client
shows, each as one card wherever it appears: tasks as chips, proposals,
escalations, questions, person tasks, results, refusals, the
conversation; and the requests a party makes about them, each saying
what it will do before it is sent.
What each card shows comes from the engine's documents (contract.md);
how it is held is client.md, section 5; how it is drawn is view.md. What
is still open is listed in section 13.

## 1. In one page

- **One object, one card.** Each object that asks something of a party (a
  proposal, an escalation, a question, a person task) and each result has
  one card, shown wherever it is relevant, deciding the same thing. A
  goal a chat proposes shows in the conversation and in the inbox;
  accepted in one, it is gone from the other.
- **A card says what deciding will do,** before it is sent: what an
  acceptance makes and who funds it, what a release starts again, what a
  cancel ends.
- **A card shows its own request,** as being decided until its answer,
  then as decided, refused or decided first by someone else, saying by
  whom and how.
- **A card offers only what fits:** what the object's document says the
  party may do now. Authority is the engine's to judge, and it judges
  again.
- **Facts in, words out.** A card's facts are typed values from the
  engine; its words are the view's phrasebook's (view.md, section 7); an
  application's values are parts in its slots (root.md, section 6).
- **No undo.** What can be reversed is reversed by another request (a
  stopped task released, an amendment amended back); what cannot (a
  cancel, an acceptance, a rejection) says so before it is sent.

## 2. Cards

- **Its states:** open; deciding, while a request about it is pending;
  refused, saying why, and staying; leaving, once decided, answered,
  withdrawn or ended, by whom and how, then gone after a linger
  (client.md, section 5).
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
  of jig's objects itself instead, from jig's domain data, in nodes of
  its own (view.md, section 4). jig's promise holds without jig's nodes:
  - *held by construction:* every decision goes through the domain's
    confirmation (client.md, section 9), and names the revision of the
    data its binding was issued from, since the application binds its
    nodes through jig's binding constructors (view.md, section 5); a
    revision that moved is refused (contract.md, 5.2); made once,
    nothing lost and nothing shown done before it is durable are the
    domain's and the engine's, whatever the tree;
  - *exported as data:* what the card would show, its state, a refusal
    and why, what a decision will do and whether it is dangerous, comes
    as jig's sayings from `jig_view::facts`, for the application to show
    where it likes;
  - *checked, not mandated:* `jig_view::check`, which its worlds run
    through the referee on every tree a person acted on, holds that a
    decision was made on a tree showing its facts, in the open
    confirmation or within the acted node's nearest named region
    (testing.md, sections 6 and 8). The referee checks jig's cards the
    same way, wherever they show.
- **Its revisions.** An act that decides about something that changes
  names that thing's revision as the card showed it: a release, its
  hold's; an amendment, the spec's; an answer, the person task's. One
  that moved under an open confirmation is refused in place, and one that
  moved in the engine is refused as moved (client.md, section 6;
  contract.md, 5.2).
- **Its confirmation** opens on any act that decides something, and says
  what will happen, from the object's document; a dangerous act (a
  cancel, a rejection) is marked so.
- **Changed while read.** A card follows its object live: a proposal
  passed up, a held task's reason that changed, a person task taken by
  someone else.
- **How it stays live.** A card follows its object through the watch of
  the page's subject that covers it: its task's document, a tree, the
  inbox, chips (contract.md, 7.1). An object no watch carries whole (a
  proposal, an escalation, a question, a person task, a result, a note)
  is read, and read again whenever a watch that names it says it moved:
  its task's activity moved, its entry in the inbox changed, its chip's
  phase changed. Once no watch names it, it is read once more for how it
  ended (client.md, section 5).

## 3. The chip, and phases in words

A task named anywhere (a plan, an entry, a history line, a turn) is a
chip: its number, its title, its executor's kind, its phase, kept live
while it shows, linking to its page.

| The engine's phase | The client says |
|---|---|
| waiting | waiting for T2, T3: each a chip, with its phase; or for a hold: "waiting for a slot in staging" |
| active, an agent: idle | waiting for news, and what would wake it, from its wake policy |
| due, preparing, claimed | starting |
| running | running: writing, or a tool (which, when its host names it), its attempt, for how long |
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
  left this period, 80 after".
- **Where it has been:** passed up by whom, or past which holders that
  could not cover it, and why ("I may not restart services in
  production").
- **Its actions:**
  - *accept,* when the party covers what it needs; otherwise not offered,
    and the card says what they lack;
  - *reject,* with a reason, which the proposer hears;
  - *pass up,* when there is a holder above the party; at the parties the
    policy names there is none, and it is not offered.
- **Accepted,** it shows what it made: chips for a batch's tasks, "the
  goal is yours", what an effect did once it is made.
- **A party's own** comes from a goal or an amendment whose draft went
  beyond the bounds its act was offered with (section 11; pages.md,
  6.2): the form said, before it was sent, that it would be proposed,
  and to whom. Where the project's policy names nobody who could accept
  it, no proposal is ever made: the form says so and sends nothing, and
  the engine refuses such a request as lacking authority (section 9).

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
  while a task above may still decide it. A party may release, amend or
  cancel a task in their tree either way, and leave it held or pass it
  up only while its escalation waits for them; the first commit decides
  (contract.md, 5.2).
- **Its actions,** those that fit its reason:
  - *release,* saying what will happen ("its next run resumes its
    conversation; tries count again from zero");
  - *amend and release,* in one request, when releasing it as it is would
    only hold it again: a budget widened, a deadline moved, instructions
    changed, a failed dependency dropped;
  - *leave it held,* with a reason: the entry leaves the inbox, and the
    task stays held where it shows, saying why;
  - *cancel* it, and everything below it (section 11);
  - *pass up,* when there is a holder above the party.

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
| its pool lost its allocation | the pool, and its slots now (a part) | release, to wait for a slot; cancel |
| its procedure needs a decision | the procedure's reason, in its connector's words (a part) | release, its counts reset; amend and release; cancel |

- **Held while closing,** the card shows the result it has kept; releasing
  it carries on closing.
- **Widening a budget** is from the task's funder: the party's pool for a
  task they fund, the task above otherwise.
- **A person deciding an uncertain effect** decides whether to make it
  again, which only someone who looks at the system can
  (`../../domain/connectors.md`, 4.3); the card says where to look, as
  its connector's part, and that the engine will do as they say.

## 6. Question

A task the party asked for asks them something (`../../domain/tasks.md`,
8.1): a coordinator unsure of the scope, a chat waiting for an answer.

- **The task** and its words, whole, never cut;
- **its context:** the task's page, and what it did just before asking;
- **an answer,** in words: a message to the task, which wakes it.

A question from a chat shows in the conversation too, answered in either
place.

## 7. Person task

A task whose executor is the party, or a role they hold
(`../../domain/people.md`, section 8).

- **What is asked:** its spec, and who asked (a task, a procedure, a
  party).
- **The results it depends on,** side by side when there are several: a
  choice between two reports shows both.
- **Its contract,** as the form that answers it:
  - *a choice* among the options its spec lists, with words to say why;
  - *an approval* of a state its procedure names, which its connector
    shows (a part); the answer names the state it approves, and is
    refused as moved unless that state is still the contract's, so an
    approval is never given to a state the party did not see; declining
    it is *cannot do it* (contract.md, 3.2);
  - *an answer* in words;
  - and, always, *cannot do it,* with a reason: a failure.
- **Addressed to a role:** *take it,* so that two do not answer at once;
  the others see who took it; *hand it back.*
- **Its stall:** when it will become news for its requester.

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
  first reply not yet read.

## 9. Refusal

What a request asked, and why it was refused, from the refusal's facts
(contract.md, section 4):

- **the role,** or the authority it lacked, said as what was missing
  ("your pool has 12 left; this needs 40");
- **a limit:** the project's live tasks, the inbox full, the chat's words
  too long;
- **moved:** the object changed since the card showed it, and how; the
  card shows it as it is now, to look at again;
- **ended:** the task or proposal is gone, and how it ended;
- **decided first,** by whom, how and when.

What the engine makes a proposal of rather than refuse (a goal beyond the
party's allotment, an amendment that widens beyond their authority) is
not refused: the confirmation said, before it was sent, that it would be
proposed and to whom, and the answer names the proposal, which then
waits where the policy says (`../../domain/tasks.md`, section 9;
contract.md, 3.2). Where the policy names nobody who could accept it,
the request is refused as lacking authority, the refusal saying that no
proposal could be accepted ("nobody in this project may accept this");
the form says so before anything is sent (client.md, section 6).

## 10. The conversation

An agent task's activity, turn by turn, across every run it has had
(client.md, section 10):

- **the words parties wrote to it,** whole, each saying where it is:
  being sent, sent, read;
- **its replies:** what the LLM wrote, as Markdown (view.md, section 8);
- **its work,** folded to a line each, opened for detail: a tool call,
  what it read, ran or changed and its outcome; its sub-agents;
- **its calls to the engine, as cards:** a delegation as the chips of the
  tasks it made; a proposal as its card; a message to a delegate; an
  effect and what it made; a note and a link to it; a decision and what
  was decided (`../../domain/engine.md`, 7.3);
- **what it heard besides parties:** its delegates' results as result
  cards, and the news it was woken by;
- **where its runs begin and end,** as quiet lines: parked; resumed;
  started fresh from a brief carrying the conversation's tail, which
  tells the person the agent may have lost detail of what came before;
- **what each turn spent.**

What streams in before a turn commits shows as it comes, marked in
progress, and is replaced by the turn once committed.

## 11. Steering

The requests a party makes on a task (`../../domain/people.md`, 5.1),
each offered where it fits and each saying, before it is sent, what it
will do:

- **Write:** words to an agent's task, which wake it. Not offered to a
  procedure, which decides on facts, nor to a party's task, which is
  answered.
- **Amend:** its spec; its wake policy; its dependencies, removing some
  while it waits; its authority, narrowed freely and widened within the
  party's own (beyond it, proposed); its charter's instructions. A
  procedure's parameters are its connector's, amended through its form in
  a slot, as the application's own request (root.md, section 6). The
  form shows before and after, and what the task will do on hearing it: a
  live run hears it; a narrowing that takes a grant its run is using
  cancels that run, and the next starts narrower. It compares the draft
  with the bounds the act is offered with (contract.md, 3.2): a change
  that widens no authority is made; one that widens anything is judged
  whole, against the authority the party may give alone, the budget
  they may add and the grants they hold, and beyond those is proposed,
  and to whom, or refused; past the policy's ceiling it is refused, and
  nothing is sent. The form says which before it is sent. Not offered
  once the task is closing.

  An agent task runs with its charter's instructions until an amendment
  gives it its own. Amended instructions replace the task's whole
  instructions, and empty ones leave it with none, not with its
  charter's; left out, they change nothing of them. The form shows the
  task's effective instructions: its own, or its charter's while it has
  none of its own. A pending proposal's before and after show the
  effective text; an older history entry shows the instructions as the
  task held them, its own or none of its own, never which charter's
  text was in effect (`../../domain/tasks.md`, section 7).
- **Stop:** halts its live run now and holds the task, stopped by the
  party; its delegates go on, and what its run committed stays.
  Releasing it carries on from there.
- **Release:** a held task, by its reason (section 5).
- **Cancel:** ends the task and its subtree, deepest first. Before it is
  sent, the card says what it ends: the tasks below that it cancels,
  counted by phase; the live runs; the proposals withdrawn; and, by
  name, the connectors whose effects will be dropped, if not yet made,
  or settle, if in flight. The core never counts a connector's effects
  (section 3): which, and how many, each connector's panel says.
- **Close,** for a chat: asks it to finish and cancels it if it has not
  within a grace. A chat with live delegates asks first what becomes of
  each: cancelled, or made the person's, funded from their pool from now
  ("reserves 6 from your pool"). One request, with each delegate's
  disposal.
- **Take over:** makes the party the task's requester, so it outlives the
  task that asked for it; its budget reserved from their pool from then
  on, which the card shows, and refused if it does not fit.
- **Prioritise:** a goal's place on the board (pages.md, 6.2).
- **Chat about it:** a new chat naming the task (pages.md, section 4).

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
