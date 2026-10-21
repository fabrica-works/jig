# Adding a dependency to a live task

Draft, 2026-10-21. A task's dependencies are fixed once it is made: an
amendment may remove one from a task still waiting, never add one
(`domain/tasks.md`, section 7), and section 13 keeps adding as open.
This draft asks for the feature: a dependency added to a task that is
waiting, active or held. It records a session with the user on
2026-10-21, which followed temper's draft `temper-on-temper.md` (6.1).
That draft encodes skein's owners' process in a deployment, and found
this the most frequent judgement the model cannot express. Its owner
designs and implements it, under skein's `docs/foundation/owners.md`.

## 1. In one page

- **Why:** in 25 hours of the owners' process, 11 of 23 dependencies
  were found while the work went on, mostly on work already under way.
  The user ruled to design the feature, not a workaround (section 2).
- **An added dependency stops a task going on; it does not send it
  back.** A waiting task waits for it as for any other. An active one
  keeps its phase, its run's work and its holds, and is kept from going
  further until the dependency is done (section 4).
- **The promise, for every executor:** while a dependency is pending,
  its task starts no run, asks no effect its connector marks as waiting
  for dependencies, and ends no way but failed or cancelled (4.2).
- **Holds stay, so the cycle check widens.** "No task holds a resource
  while waiting for another" becomes "no circle of waits", over
  dependencies, delegates and holds' queues. This check is the real
  cost, and the reason the model forbade the feature (4.3).
- **No new authority:** whoever may amend the task may add one, naming
  a task they could name in a batch (4.4).
- **The acceptance story is the reference's own:** a change already
  queued is given a dependency on another change. It leaves the queue,
  the other lands, and it comes back at its place (section 6).

## 2. Why

- **The reference.** temper's draft replays a trace of the owners'
  process (10-19 16:15Z to 10-20 17:20Z, kept in
  `~/src/rust/tmp/temper-on-temper/`). Its dependency register
  (`out/timeline.md`, "Dependency waits") counts 12 planned and 11
  discovered. Ordering is the largest class of the director's moves no
  rule prescribed (116 of 418, `out/improvisations.md`), and many of
  them are of this kind.
- **What the 11 were,** read from the register's lines:
  - *start after* (4): browser's temper-04 after accounts' repins,
    forge-kit's resume, hygiene's CLI adoption, an archive after the
    last old runs;
  - *land after* (5): hearth's breaking stacks and its files work after
    accounts' repins, a repin after the fix of a slow test, a pin
    taken only through one change, first-run's directories after the
    next repin. The work was written; only its landing had to wait;
  - *the director's word* (2): two API breaks that waited for an
    approval.

  So the main case is a change that exists and must not land yet. A
  dependency that only means "starts after" does not cover it.
- **The workarounds:** a new task that depends on the live one, a
  subscription to its landing, or holding its branch back. Each is hand
  work the deployment should do. The user ruled against them.

## 3. What exists

- **A dependency** is a live task its dependent starts after, each done
  before it starts (`domain/tasks.md`, section 3). A task that depends
  on another never starts before what that task did is visible and
  settled (5.1 there).
- **Admission checks a batch's new edges** with the existing
  dependencies and delegation waits, so that no task depends on an
  ancestor and no two subtrees wait on each other in a circle (section
  4 there). A new task has no dependents yet, so the check stays local
  to its batch.
- **Holds are taken whole:** a task never holds one resource while
  waiting for another (6.2 there). That is why holds cannot circle, and
  why the referee checks it (section 12 there).
- **A dependency that fails** or is cancelled holds its dependent (5.5
  there).
- **A finish while delegates are live is refused,** naming them (5.6
  there). The rule this draft adds has the same shape.

## 4. The direction

### 4.1 What an added dependency does

By the dependent's phase:

- **Waiting:** it waits for this one too, as if its batch had named it.
- **Active:** it keeps its phase, and its executor is kept from going on
  (below).
- **Held:** the add is allowed, as any amendment of a held task is. A
  release returns the task where it was, the dependency pending.
- **Closing:** refused, as every amendment of a closing task is.

By the dependent's executor, while active:

- **An agent task.** The add is an amendment, so a live run hears it,
  and may finish its turn. A finish with a done result is refused,
  naming the dependency, so the run parks and its unfinished work is
  saved (`domain/engine.md`, 7.4). No run is claimed while the
  dependency is pending. Its being done is a notice that wakes the task
  (`domain/tasks.md`, 8.1), and the next run starts from what the last
  one left.
- **A procedure task.** The procedure steps on the amendment, as on any
  (`domain/connectors.md`, 6.2), and reads which of its task's
  dependencies are pending. It steps again when one is done. Its
  connector says which effect kinds wait for dependencies, and the core
  refuses those while one is pending. A change procedure marks its merge
  so. It also stops being ready, and leaves its landing branch's queue,
  so that nothing ready waits behind it. It comes back at its place,
  since it keeps the time it first became ready. That last part is the
  application's connector, one rule in its design.
- **A person task.** It stays in its party's inbox, shown waiting on
  the dependency. An answer while it is pending is refused, naming it.

### 4.2 The promise

While an added dependency is pending, its task:

- starts no run;
- asks no effect of a kind its connector marks as waiting for
  dependencies;
- ends no way but failed or cancelled.

So its requester and its dependents still never see it done before what
it now depends on is done. The core keeps the promise, not the
executor: an agent's charter or a procedure may also behave, but the
refusals hold if they don't.

Only what the dependent must hear still reaches it while its run cannot
start: a cancel and its grace, and the proposals and escalations it
must decide, since no policy may keep a decision from whoever must make
it (`domain/tasks.md`, 8.3). Other messages wait in its inbox, as for a
waiting task.

### 4.3 Holds and the circle check

An active task keeps its holds while its dependency is pending.
Releasing them would run the connector's release effects (a branch
deleted, an environment torn down), and taking them again would mean
queueing whole. Keeping them breaks `domain/tasks.md`'s 6.2, so the
invariant changes:

- **No circle of waits.** The waits are:
  - a dependency, from dependent to prerequisite;
  - an end after the subtree, from a task to each live delegate;
  - a hold's queue, from waiter to holder.
- **The add is checked:** adding A after B is refused when B already
  reaches A, naming the tasks on the circle. B an ancestor of A is the
  simplest case.
- **A later wait on a hold is checked** when the holder, or a task it
  reaches, has a pending added dependency. A batch whose task would
  close a circle is refused, naming it. A task that becomes ready and
  would close one is held, saying so.
- **The bound stays:** a task waiting for a hold past its bound is held
  (`domain/tasks.md`, 6.2), the backstop if the check misses a circle.
- **Its cost** is a walk of the live waits, within the limits (live
  tasks, dependencies, holds and waiters per resource), and enters
  `domain/engine.md`'s worst case (section 13). The common case should
  stay local: a queue join with no pending added dependency in reach
  costs nothing new.

### 4.4 Who may add one, and what

- **Who:** whoever `domain/tasks.md`'s section 7 lets amend the
  dependent: its requester, a task above it, a party in whose tree it is
  or whose role allows it. A party's request, an agent's `amend` and a
  procedure's step reach the same rule.
- **The prerequisite:** a task the amender could name in a batch
  (`domain/tasks.md`, section 4). For a task amender that is its own
  delegates or a task it was introduced to (8.5 there). The amender may
  amend anything below it, so the owner judges whether the reach widens
  to its whole subtree. A director that orders two owners' changes
  needs that, or introductions.
- **No authority moves.** An added dependency narrows and widens
  nothing, and funds nothing.
- **Named with the revision it saw,** as every amendment is
  (`domain/tasks.md`, section 7).

### 4.5 The edges

- **A prerequisite already done** counts as met. The add is recorded and
  nothing waits, so an amender racing a landing is not refused.
- **A prerequisite failed or cancelled** is refused, naming it.
- **A prerequisite that fails later** holds the dependent, as a batch's
  dependency does (`domain/tasks.md`, 5.5).
- **Removing** an added dependency lifts it at any phase before closing,
  and wakes the task if it waits on nothing else.
- **The director's word** needs no new kind: a person task, a question
  to the director, added as a dependency, holds a change until it is
  answered. That covers the reference's two approvals.

### 4.6 Views and history

- **History:** the add is an amendment like any other: made, by whom,
  why (`domain/tasks.md`, section 3).
- **A task's document** lists its dependencies, an added one marked
  with when and by whom.
- **What it waits on** (`domain/engine.md`, 11.3) gains a case: an
  active task with a pending added dependency waits on it.

## 5. What it touches

- **`domain/tasks.md`:**
  - 3 and 7: an amendment may add a dependency;
  - 4: the prerequisite's reach;
  - 5.1 to 5.6: the promise;
  - 6.2: no circle of waits;
  - 8.1: the notice when one is met;
  - 11: limits;
  - 12: the world and its referee;
  - 13: the open question, closed.
- **`domain/engine.md`:**
  - 7.1: no claim while one is pending;
  - 7.3: `amend`'s new field;
  - 7.4: a finish refused, so the run parks;
  - 11.3: what a task waits on;
  - 13: the check's worst case.
- **`domain/connectors.md`:**
  - 6: a procedure hears and reads its pending dependencies;
  - 4.1: the effect kinds that wait for dependencies.
- **`domain/people.md`, 5.1:** a party's amendment.
- **The client's contract and protocol:** `Amend` names dependencies to
  add, with its refusals: a circle, out of reach, a prerequisite failed.
- **The store:** the task's record keeps its added dependencies, a new
  version.
- **Downstream:** the application's change procedure stops being ready
  while one is pending, and marks its merge as waiting.

## 6. Stories

In the tasks child's world, and the engine's where a run is involved:

- **The reference's reordering** (temper's draft, 5.2): change B is
  queued first; change A, queued behind it, must land first. B is given
  a dependency on A. B leaves the queue, A lands, and B comes back at
  its place and lands. No relay, no branch held back.
- **A running agent given one:** its run finishes its turn, its finish
  is refused naming the dependency, and it parks. Nothing is claimed
  until the prerequisite is done. Then the task wakes and its next run
  goes on from what was saved.
- **Circles refused,** naming their tasks:
  - A after B, then B after A;
  - a task after its ancestor;
  - through a hold: a holder after B, while a task under B would queue
    for the holder's resource.
- **The edges:** a prerequisite already done, met at once; one failed,
  refused; one that fails later, the dependent held; one removed, the
  task woken.
- **The director's word:** a change given a dependency on a person task,
  answered, and the change goes on.
- **A held task** given one and released, waiting on it.

The referee gains: no task with a pending dependency starts a run, asks
an effect of a waiting kind, or ends done; no circle of waits forms.

## 7. Open, for the owner

- **Words to a waiting agent task:** whether a party's words wake it
  anyway, as a person talking to a goal would expect, or wait like the
  rest.
- **Pool slots:** whether an active task gives up its pool slots while a
  dependency is pending (an implementer's slot sitting idle), keeping
  only its write holds. That frees capacity and removes most circles
  through queues. It costs a whole retake on resume.
- **Circles today:** a holder ends after its delegates, and a delegate
  may wait in another holder's queue. Check whether two trees can
  already circle that way. If so, the widened check covers both.
- **The reach** for a task amender: its delegates and introductions, or
  its whole subtree (4.4).
- **The check's cost and index:** what the walk costs at the limits, and
  what the core keeps so that a queue join stays cheap.
- **What a procedure reads:** a list of pending dependencies, or only
  whether any is pending.

## 8. Not in scope

- **Parking an owner** (temper's draft, 6.2): a party's or an
  ancestor's hold on a subtree. It shares this draft's mechanics: a live
  task kept from its next run without losing its state or holds. The
  design should keep them reusable, without designing the park.
- **A gate an ancestor judges** (temper's draft, 6.7): a person task as
  a dependency covers the director's word for now.
- **Waiting on what is not a task,** such as a landing nobody in the
  deployment makes: that stays a subscription.
