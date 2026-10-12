# Session 12: the store's indexes for parties' reads, and the inbox's census

skein-kv has no secondary index and loads forwards only, so where a party's
read pages in an order no family's key gives, jig's domain keeps an index:
small rows saved and erased in the same decision as the records they point
at, keyed newest first where a page reads so (`protocol/store.md`, 7.4).
This session adds the five indexes store.md 15.3 names (a party's inbox
updates by time, a party's conversations, a project's goals with each ended
goal's phase counts, ended delegates by requester, a subtree's delegates)
and the inbox's census index, and makes the restart's inbox census read
the indexes instead of every ended task (15.2). The core's stored pages for
these scopes take the index's keys as their positions, so the store reads
one interval forwards; the core world's read transport serves them from the
index rows, and jig 16 serves them from the real store.

## Read first

- jig's `protocol/store.md`, sections 4.3, 4.4 (the last row), 7.2 (the
  inbox's census), 7.3 (the table of scopes), 7.4 and 15, items 2 and 3.
- jig's `client/domain/contract.md`, section 6; `client/domain/pages.md`,
  section 3 (the inbox's groups and their order); `domain/people.md`,
  section 6 (what an inbox holds, when a chat is forgotten).
- `decisions.md`, 9 (an ended goal's index row keeps its subtree's phase
  counts).
- `findings/domain.md`, items 11.2 and 11.3; `findings/store.md`, sections
  2.6 (the census) and 3 (the scope table, rows 9 to 14), 7.1 (D4, D5).
- jig 11's file: `RestoreRange`, the tasks child's key order.

## What the design asks

- **Indexes** (store.md 7.4): "Where a scope's order is not a family's key
  order, the domain keeps an index: a family of small rows its owner saves
  and erases in the same decision as the records they point at, with keys
  reversed in time where a page reads newest first. Each index is its
  owner's, tested in its world."
- **Which** (15.3): "each party's inbox by time, each party's
  conversations, each project's goals, each task's ended delegates, a
  subtree's delegates for its history".
- **The scopes** (7.3): "an inbox group | the index of the party's inbox, by
  time: … updates newest first; a group, and the filter, select within
  it"; "conversations | the index of the party's conversations, newest
  first"; "a project's goals | the index of the project's goals, live then
  ended newest first"; "a task's ended delegates | the index of ended
  delegates by requester"; "a task's history, or a subtree's | the history
  and milestone families; a subtree's by the index of its delegates".
- **The census** (7.2, 15.2): "the inbox's census, a step of its own once
  people's read positions are restored: the index of what parties' inboxes
  show (7.4), never every ended task".
- **Goals' counts** (decision 9): "an ended goal's index row keeps its
  subtree's phase counts, so a project's goals page needs no subtree join".

## Choices this plan makes

- **The index families are the tasks child's keys** (store.md 4.4's last
  row), appended after `Activity` in `tasks::Key`, each with a small row in
  `tasks::Stored`. Rows that need what only the core knows (a result's
  commit position, a chat's charter, a reply's party) are written by the
  core in the decision it routes, as `inbox::forget_chat` already erases
  `tasks::Key::Ended` (`crates/jig-domain-core/src/inbox.rs:493-516`); the
  others by the tasks child.
- **Newest first is a key of its own:** `tasks::Newest(u64)` holds a
  number's complement, so a key's derived order, and jig 13's bytes, read
  newest first forwards.
- **A stored page's positions are its index's keys** for the inbox's
  updates, conversations, goals and ended delegates:
  `Position::Owner(Key::Tasks(<index key>))`. The store reads one interval
  forwards from the cursor; a page's row carries the index row as `value`
  and the record it names as `owner` (and, for a reply, its activity line
  as `related`). `Position::Inbox` stays for the needs groups, which are
  live and never stored.
- **Conversations page both ways** (`crates/jig-domain-core/src/reads/cursors.rs:84-120`:
  `ConversationsBefore` reads toward newer ones), so they keep two families
  of the same row: newest first and oldest first.
- **Needs are not indexed:** every need is live work the core holds whole
  (`client/domain/contract.md`, 6: "live state, then the store past the
  engine's limit"; `rows.rs` `needs_store` never loads a needs group).
- **The census** reads every party's conversations (ended chats not yet
  forgotten) and each party's results after its read position, joined to
  their ended tasks; live tasks and pending person proposals are projected
  from memory, since `LoadCore` restored them.
- **A subtree's history** is found through `Delegate` rows (every task
  made under a task), erased when a person adopts the task
  (`crates/jig-domain-tasks/src/moving.rs:375`); the store collects the
  subtree (at most `tasks::Limits::tree_tasks`) and reads each member's
  history and milestone intervals in key order.

## Rules that bind it

- **A durable record's codec follows its record** (README, "Rules"): "Once
  jig 14 or jig 19 has landed a family's codec, an increment of any session that
  changes a durable record of that family changes its schema, its generated
  codec and its translation in the same increment, and the record's
  family's version moves at the first such change in this plan." Each key
  variant this session adds also extends jig 13's
  `crates/jig-protocol-store/src/keys/tasks.rs` when it exists (an
  exhaustive match: the build breaks otherwise).
- **"adds a field or predicate that its counterpart across a boundary
  lacks"** is on the plan's "Don't" list (`code-organization.md`, section
  6): each new key variant gets its tag in the client world's cursor codec
  (`tests/client/src/joint/wire/cursor.rs:793-850`) in the same increment.
- **"A file: at most 800 lines …"** (`code-organization.md`, section 7):
  `crates/jig-domain-core/src/reads/rows.rs` is 1,744 lines and
  `crates/jig-domain-tasks/src/boundary.rs` 1,776. New index code goes in
  new files (`crates/jig-domain-tasks/src/index.rs`,
  `crates/jig-domain-core/src/indexes.rs`,
  `crates/jig-domain-core/src/reads/indexed.rs`); an edit to the two
  large files only rewires them.

## Increments

### 12.1 The inbox's updates, by party and time

Scope: `tasks::Newest`; the `Update` index (a person's results and replies,
newest first), written by the core where it routes a task's ending and a
reply's activity line, a result's erased by `forget_chat` and a reply's
left to retention (store.md 7.4, "An owner erased first"); the stored part
of the Updates, Results and Replies groups read from it, its keys as
positions, skipping a row whose owner is gone (store.md 7.3, 7.4, 15.3).
Needs: nothing beyond the session's start.
Files:
- `crates/jig-domain-tasks/src/index.rs` (new: `Newest`, `Update`, `UpdateObject`), `crates/jig-domain-tasks/src/boundary.rs` (`Key::Update`, `Stored::Update`, `Stored::key`), `crates/jig-domain-tasks/src/lib.rs` (exports);
- `crates/jig-domain-core/src/indexes.rs` (new: the core's index writes), `crates/jig-domain-core/src/routing/tasks.rs` (the save interception at :670-720: a result's and a reply's `Update`; shared hotspot), `crates/jig-domain-core/src/inbox.rs` (`forget_chat` :493-516 erases the result's `Update`);
- `crates/jig-domain-core/src/reads/indexed.rs` (new), `reads/rows.rs` (`inbox_position` :145-178, `inbox_row` :1056+, `position_allowed` :1427-1500, `key_kind` :1512-1546), `crates/jig-domain-core/src/reads.rs` (`backwards` :124-148: the updates groups read forwards);
- the exhaustive `tasks::Stored` and `tasks::Key` matches of 11.2's list, the cursor codec `tests/client/src/joint/wire/cursor.rs`;
- the core world's transport `tests/core/src/reads.rs` (`candidate` :251-290, `page` :331-372);
- jig 13's `keys/tasks.rs` when it exists; jig 14's tasks codec when it exists.
Shape:
- `tasks::Key::Update { party: u64, at: Newest, task: Newest, object: UpdateObject }`;
  `tasks::Stored::Update(Update)`; `Update` and `UpdateObject` as sketched.
- The core writes an `Update` with the `Save` of an ended task whose
  requester is a person (`object: Result`), and with the `Save` of an
  activity line that is a reply to a person: `Line::Turn { reply, .. }` of a
  task the person requested, `Line::Notified { recipient: Person(_), .. }`
  (the cases `tests/core/src/reads.rs:251-290` reads today).
- A stored updates page: `Range.after` is
  `Position::Owner(Key::Tasks(Key::Update { party, .. }))`; `inbox_position`
  makes it from a live cursor; a row's `value` is the `Update` row, `owner`
  its task, `related` the reply's activity line.
- `pub fn index_key(scope: &Scope, party: u64, cursor: documents::Cursor, activity: Option<u64>) -> Option<Key>`
  in `reads/indexed.rs`: the index key that continues a live window into
  the store, for the scopes this session indexes.
Test shape:
- New `crates/jig-domain-tasks/src/tests/index.rs` (declared in
  `crates/jig-domain-tasks/src/tests.rs`). Reference test, written out in
  full:

  ```rust
  //! Index keys read newest first forwards (store.md 7.4).
  use crate::{Key, Newest, UpdateObject};
  use skein_lib::Wall;

  #[test]
  fn a_later_update_sorts_before_an_earlier_one_for_one_party() {
      let earlier = Key::Update {
          party: 7,
          at: Newest::of(Wall::from_nanos(1_000).as_nanos()),
          task: Newest::of(30),
          object: UpdateObject::Result,
      };
      let later = Key::Update {
          party: 7,
          at: Newest::of(Wall::from_nanos(2_000).as_nanos()),
          task: Newest::of(20),
          object: UpdateObject::Result,
      };
      assert!(later < earlier);
      assert_eq!(Newest::of(30).value(), 30);
  }
  ```
- Also there: `a_result_sorts_before_a_reply_of_the_same_task_and_time`,
  `one_partys_updates_sort_before_the_next_partys`.
- In `tests/core/tests/reads.rs` (reuse `stories::reads::{world, request}`
  at `tests/core/src/stories/reads.rs:13,43`; `reads::{ask, latest}`;
  `World::end`, `World::turn`): `updates_page_from_the_index_newest_first_after_restart`,
  `a_forgotten_chats_result_leaves_the_updates_index`,
  `the_results_group_pages_results_only_from_the_index`. Each its own
  `#[test]`.
- `a_forgotten_chats_result_leaves_the_updates_index`: once the chat is
  forgotten, its result's `Update` row is gone, its replies' `Update` rows
  stay, and the person's updates page from the store shows neither
  (store.md 7.4).
- The transport's updates arm reads `Update` rows of `range.party` from
  `after` in key order and joins their owners; it reads no `Ended` or
  `Activity` row but those an `Update` names.
- Ceilings: unit 20 ms, world 0.5 s.
Proof: world. `cargo nextest run -p jig-core-world --test reads` once,
through `heavy`: a person's updates page from the store newest first,
across a restart, with the transport reading only the party's `Update`
rows and what they name. Bound: 1 minute.
Don't: index a need; change `Position::Inbox` for the needs groups; erase
a reply's `Update` in `forget_chat`, or load a chat's replies to find them.
Size: at most 220 lines of production code.
Removes: the transport's scan of `Ended` and `Activity` rows for the
updates groups (`tests/core/src/reads.rs:251-290`).

### 12.2 A party's conversations

Scope: two families of one row, `ConversationNewest` and
`ConversationOldest`, written by the core when a chat ends and erased by
`forget_chat`; the stored conversations read from them, newest first and,
for `ConversationsBefore`, oldest first (store.md 7.3, 15.3).
Needs: 12.1's `Newest`.
Files:
- `crates/jig-domain-tasks/src/index.rs`, `boundary.rs` (`Key::ConversationNewest`, `Key::ConversationOldest`, `Stored::ConversationNewest`, `Stored::ConversationOldest`);
- `crates/jig-domain-core/src/indexes.rs` (at a chat's ending: `chat_requester` at `crates/jig-domain-core/src/inbox.rs:103-117`), `inbox.rs` (`forget_chat` erases both);
- `crates/jig-domain-core/src/reads/indexed.rs`, `reads/cursors.rs` (`stored_edges` :84-120: a page's edge positions in the family it continues), `reads/rows.rs` (`position_allowed`, `key_kind`, `page_row` Conversations :972-989);
- `crates/jig-domain-core/src/tests/conversation_cursor.rs` (13 uses of the conversations' positions);
- the exhaustive matches and the cursor codec as in 12.1; `tests/core/src/reads.rs` (`conversation_row` :221-224);
- jig 13's `keys/tasks.rs` when it exists; jig 14's tasks codec when it exists.
Shape:
- `tasks::Key::ConversationNewest { party: u64, task: Newest }` and
  `tasks::Key::ConversationOldest { party: u64, task: u64 }`;
  `tasks::Stored::ConversationNewest(Conversation)` and
  `tasks::Stored::ConversationOldest(Conversation)`, one record type.
- A stored conversations page with `backwards: true` reads
  `ConversationNewest` from `after`; `ConversationsBefore` reads
  `ConversationOldest`. Its row: `value` the conversation row, `owner` the
  ended chat.
Test shape:
- `crates/jig-domain-tasks/src/tests/index.rs`:
  `a_newer_conversation_sorts_first_in_the_newest_family`.
- `crates/jig-domain-core/src/tests/conversation_cursor.rs` (its helpers
  `retained` and the page builder at :146, :344): the existing tests run
  with the new positions; add `a_before_cursor_continues_in_the_oldest_family`.
- `tests/core/tests/reads.rs`: `conversations_page_both_ways_from_the_index`,
  `a_forgotten_chat_leaves_both_conversation_families`.
- Ceilings: unit 20 ms, world 0.5 s.
Proof: world. `cargo nextest run -p jig-core-world --test reads` and
`cargo nextest run -p jig-domain-core conversation_cursor` once, through
`heavy`: a person pages their conversations forward and back from the
store across a restart. Bound: 1 minute.
Don't: write a conversation row for a task that is not a chat (as
`chat_requester` says).
Size: at most 150 lines of production code.
Removes: the transport's scan of the `Ended` rows for conversations
(`tests/core/src/reads.rs:221-224, 302`).

### 12.3 The census from the indexes

Scope: the `Result` index (a person's results by commit position), written
by the core with each ending and erased by `forget_chat`;
`RestoreRange::Census`, which the core names when it asks `LoadInbox`: each
party's conversations, and each party's results after its read position; the census
rows reach `restore_inbox_row` as the ended tasks the index names; live
tasks and pending person proposals are projected from memory (store.md
7.2, 15.2).
Needs: 12.2's conversation families.
Files:
- `crates/jig-domain-tasks/src/index.rs`, `boundary.rs` (`Key::Result`, `Stored::Result`);
- `crates/jig-domain-core/src/indexes.rs`, `inbox.rs` (`restore_key` :397-414 removed; `restore_inbox_row` :418-478 takes ended rows only; `forget_chat` erases the `Result` row), `restart.rs` (`RestoreRange::Census`, `Census`, `ReadResults`), `crates/jig-domain-core/src/lib.rs` (`Core::census`; the `inbox_restore_key` export at :30 removed);
- the roots: `testing/jig-test-domain/src/lib.rs` (`restore_record` :1006-1056), `examples/ops/domain/src/store.rs` (`Range::Inbox` :12 becomes `Range::Census(core::Census)`), `examples/ops/domain/src/restart.rs` (:24-25, :104-170);
- the worlds: `tests/core/src/effects.rs` (`load_restart` :1028-1048, `restored` :1074-1080), `tests/conformance/src/lib.rs` (:193-233), `tests/ops/src/conformance/mod.rs` (:341-392);
- `RestartStep::LoadInbox`'s users (`findings/domain.md` 11.2: 23 uses in 11 files) where they load; `crates/jig-domain-core/src/tests/restore_admission.rs`;
- jig 13's `keys/tasks.rs` and `intervals` when they exist; jig 14's tasks codec when it exists.
Shape:
- `tasks::Key::Result { party: u64, position: u64 }`, `tasks::Stored::Result(ResultRow)`.
- `RestoreRange::Census(Census)`; `Census { results: Box<[ReadResults]> }`;
  `ReadResults { party: u64, after: u64 }`; `Census::admits(&self, key: &Key) -> bool`
  (each conversation row of the oldest family; a `Result` row of a listed
  party past its `after`).
- `Core::census(&mut self, limits: &Limits) -> Census`: called by the root
  when the core asks `LoadInbox`; it steps people with each live task's
  waiting entries and each pending person proposal's (what
  `restore_inbox_row` did for `Live` and `PersonProposal` rows), and lists
  each person with a read position.
- `Core::restore_inbox_row(&mut self, limits: &Limits, row: &tasks::Stored) -> bool`
  keeps its signature and takes `Stored::Ended` only; a row it is handed
  twice (a chat with an unread result) changes nothing the second time.
- A restart's `LoadInbox` reads no `Live`, `PersonProposal` or unnamed
  `Ended` row.
Test shape:
- `crates/jig-domain-core/src/tests/restore_admission.rs` (fixtures at
  `crates/jig-domain-core/src/tests.rs:29,193`):
  `the_census_lists_each_party_after_its_read_position`,
  `an_ended_chat_handed_twice_to_the_census_changes_nothing`.
- `tests/core/tests/restart.rs`:
  `a_restart_reads_no_ended_task_whose_result_was_read`,
  `a_restart_restores_unread_results_and_chat_totals`. The world counts the
  ended rows it handed the census (a field of `World`) and the first test
  asserts it equals the unread ones, written as a literal.
- Ceilings: unit 20 ms, world 0.5 s.
Proof: world. `cargo nextest run -p jig-core-world --test restart --test
reads` and `cargo nextest run -p jig-conformance-world` once, through
`heavy`: after a restart, a person's inbox shows the same unread results
and chat counts while the census reads only the ended tasks it names.
Bound: 2 minutes.
Don't: keep `inbox_restore_key`; read `Live` rows twice at restart.
Size: at most 200 lines of production code.
Removes: `inbox::restore_key` and its export; the `Live` and
`PersonProposal` arms of `restore_inbox_row`; the census's loads of the
tasks family at `tests/core/src/effects.rs:1035`, `tests/conformance/src/lib.rs:202`,
`tests/ops/src/conformance/mod.rs:358`.

### 12.4 A project's goals, with their phase counts

Scope: a task's record keeps how its ended descendants ended; the `Goal`
index (a project's ended goals, newest first), written by the tasks child
when a tracked goal ends, holds its subtree's counts; the stored goals page
reads it and joins no subtree (store.md 7.3, 15.3; decision 9).
Needs: 12.1's `Newest`.
Files:
- `crates/jig-domain-tasks/src/index.rs`, `boundary.rs` (`TaskRecord.ended_below`, `Endings`, `Key::Goal`, `Stored::Goal`), `crates/jig-domain-tasks/src/closing.rs` (:320-340: a delegate's ending adds to its requester's `ended_below`; a tracked goal's ending saves its `Goal` row), `stored.rs` (`valid_record`), `owned.rs`;
- the `TaskRecord` literals (11.3's list);
- `crates/jig-domain-core/src/reads/rows.rs` (`page_row` Goals :923-971: counts from the row), `reads/indexed.rs`, `reads.rs` (`backwards`);
- `crates/jig-domain-core/src/documents/*` where `PhaseCounts` is built from `Endings`;
- the exhaustive matches and the cursor codec as in 12.1; `tests/core/src/reads.rs`; `tests/tasks/src/world.rs`;
- jig 13's `keys/tasks.rs` when it exists; jig 14's tasks codec when it exists.
Shape:
- `tasks::Endings { done: u32, failed: u32, cancelled: u32 }`;
  `TaskRecord.ended_below: Endings`: "how this task's ended descendants
  ended, summed as each delegate ends", over the task's life: never reset
  with `made`, each count added with `saturating_add` (ruling 131).
- `tasks::Key::Goal { project: u32, task: Newest }`, `tasks::Stored::Goal(GoalRow)`,
  `GoalRow { project: u32, task: u64, endings: Endings }` (the goal's own
  ending included).
- The stored goals read forwards from `after` in the `Goal` family: newest
  first, as store.md 7.3 says ("live then ended newest first"; today
  oldest first, `crates/jig-domain-core/src/reads.rs:124-148`). A row:
  `value` the `GoalRow`, `owner` the ended goal; `linked` is empty.
Test shape:
- `tests/tasks/tests/lifecycle.rs` (reuse `jig_tasks_world::{World, task, LIMITS}`):
  `a_goal_ending_saves_its_subtrees_phase_counts`,
  `a_delegates_ending_adds_to_its_requesters_ended_below`.
- `tests/core/tests/reads.rs`: `goals_page_newest_first_with_counts_from_the_index`.
- `crates/jig-domain-tasks/src/tests/index.rs`:
  `a_newer_goal_sorts_first_within_its_project`.
- Ceilings: unit 20 ms, world 0.5 s.
Proof: world. `cargo nextest run -p jig-tasks-world --test lifecycle` and
`cargo nextest run -p jig-core-world --test reads` once, through `heavy`: a
project's ended goals page newest first, each with its subtree's counts,
and the transport joins nothing below them. Bound: 1 minute.
Don't: join a subtree for a goals row.
Size: at most 180 lines of production code.
Removes: the subtree join for goals rows (`rows.rs` `linked_below` in the
Goals arm).

### 12.5 Ended delegates and a subtree's delegates

Scope: `EndedDelegate` (a task's ended delegates by requester), written by
the tasks child when a delegate ends; `Delegate` (each task made under a
task), written when the task is made and erased when a person adopts it;
the stored ended-delegates page reads the first, and a subtree's history
finds its members through the second (store.md 7.3, 15.3).
Needs: nothing beyond 12.1.
Files:
- `crates/jig-domain-tasks/src/index.rs`, `boundary.rs` (`Key::EndedDelegate`, `Key::Delegate`, `Stored::EndedDelegate`, `Stored::Delegate`);
- `crates/jig-domain-tasks/src/closing.rs` (an ending with `requester: Task(_)`), the making of a task (`crates/jig-domain-tasks/src/batch.rs`, where a new task's first `Live` save is pushed), `moving.rs` (:375, adoption erases its `Delegate` row);
- `crates/jig-domain-core/src/reads/rows.rs` (`page_row` EndedDelegates :830-838), `reads/indexed.rs`;
- the exhaustive matches and the cursor codec as in 12.1; `tests/core/src/reads.rs` (EndedDelegates and History arms, :313-314 today none);
- jig 13's `keys/tasks.rs` when it exists; jig 14's tasks codec when it exists.
Shape:
- `tasks::Key::EndedDelegate { requester: u64, task: u64 }` and
  `tasks::Key::Delegate { requester: u64, task: u64 }`;
  `tasks::Stored::EndedDelegate(Edge)` and `tasks::Stored::Delegate(Edge)`,
  `Edge { requester: u64, task: u64 }`.
- A stored ended-delegates page reads `EndedDelegate { requester: task, .. }`
  forwards; its row's `value` is the edge, `owner` the ended delegate.
- `History { subtree: true }`'s positions stay `History` and `Milestone`
  keys; the transport (and jig 16's store) collects the subtree from
  `Delegate` rows, at most `tree_tasks`, and reads each member's history
  and milestone rows in key order.
Test shape:
- `tests/tasks/tests/lifecycle.rs`: `a_delegate_ending_saves_its_ended_delegate_row`,
  `an_adopted_task_leaves_its_old_requesters_delegate_rows`.
- `tests/core/tests/reads.rs`: `ended_delegates_page_from_the_index`,
  `a_subtree_history_reads_each_delegates_rows_found_by_index`.
- Ceilings: world 0.5 s.
Proof: world. `cargo nextest run -p jig-tasks-world --test lifecycle` and
`cargo nextest run -p jig-core-world --test reads` once, through `heavy`: a
person reads a goal's ended delegates and its subtree's history from the
store, found by index. Bound: 1 minute.
Don't: write a `Delegate` row for a task made by a person or the
deployment.
Size: at most 150 lines of production code.
Removes: nothing.

## Interface sketches

```rust
// jig_domain_tasks::index (crates/jig-domain-tasks/src/index.rs)
use skein_lib::Wall;

/// A number's complement, so that a key holding it reads newest first
/// forwards (store.md 7.4). jig 13 writes its raw value as a `u64`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Newest(u64);

impl Newest {
    #[must_use]
    pub const fn of(value: u64) -> Newest { Newest(!value) }
    /// The number it orders.
    #[must_use]
    pub const fn value(self) -> u64 { !self.0 }
    /// The stored complement, as keys write it.
    #[must_use]
    pub const fn raw(self) -> u64 { self.0 }
    #[must_use]
    pub const fn from_raw(raw: u64) -> Newest { Newest(raw) }
}

/// What an inbox update opens: a result, or a reply. A result sorts first.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum UpdateObject {
    /// The task's result.
    Result,
    /// A reply: its message number and its activity line.
    Reply { message: Newest, activity: Newest },
}

/// One update in a person's inbox. Written by the core with the decision
/// that made it; a result's erased when its chat is forgotten, a reply's
/// by retention (store.md 7.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Update {
    pub party: u64,
    pub at: Wall,
    pub task: u64,
    pub object: UpdateObject,
}

/// One of a person's results, by the commit position the core gave it: the
/// census reads those past the person's read position.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ResultRow {
    pub party: u64,
    pub position: u64,
    pub task: u64,
}

/// An ended chat of a person's, until it is forgotten.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Conversation {
    pub party: u64,
    pub task: u64,
}

/// How a task's ended descendants ended (decisions.md 9).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Endings {
    pub done: u32,
    pub failed: u32,
    pub cancelled: u32,
}

/// An ended tracked goal of a project, with its subtree's endings.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GoalRow {
    pub project: u32,
    pub task: u64,
    pub endings: Endings,
}

/// A task made under a task: as `Delegate` from its making until a person
/// adopts it; as `EndedDelegate` once it ended under that requester.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Edge {
    pub requester: u64,
    pub task: u64,
}

// tasks::Key gains, after `Activity`, in this order:
//     Update { party: u64, at: Newest, task: Newest, object: UpdateObject },
//     Result { party: u64, position: u64 },
//     ConversationNewest { party: u64, task: Newest },
//     ConversationOldest { party: u64, task: u64 },
//     Goal { project: u32, task: Newest },
//     EndedDelegate { requester: u64, task: u64 },
//     Delegate { requester: u64, task: u64 },
// tasks::Stored gains Update(Update), Result(ResultRow), ConversationNewest(Conversation),
// ConversationOldest(Conversation), Goal(GoalRow), EndedDelegate(Edge), Delegate(Edge).
// Key::live is false for each. TaskRecord gains `ended_below: Endings`.
```

```rust
// jig_domain_core::restart (crates/jig-domain-core/src/restart.rs)
use alloc::boxed::Box;

/// A person's results to restore: those after their read position.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ReadResults {
    pub party: u64,
    pub after: u64,
}

/// The inbox's census (store.md 7.2): each conversation, and each listed
/// party's unread results. The store hands up the ended tasks they name.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Census {
    pub results: Box<[ReadResults]>,
}

// RestoreRange gains `Census(Census)`; `Census::admits(&self, key: &Key) -> bool`.
// impl Core { pub fn census(&mut self, limits: &Limits) -> Census; }

// jig_domain_core::reads::indexed (crates/jig-domain-core/src/reads/indexed.rs)
// /// The index key that continues a live window into the store for `scope`,
// /// or None for a scope this session does not index.
// pub fn index_key(scope: &Scope, party: u64, cursor: documents::Cursor, activity: Option<u64>) -> Option<Key>;
```

- **Owners.** The tasks child owns the keys and rows; the core writes
  `Update`, `Result` and both conversation families, the tasks child
  `Goal`, `EndedDelegate` and `Delegate`. jig 13 encodes the keys and maps
  each scope and `Census` to intervals; jig 16 serves the pages from them.
- **What ends what.** An index row ends with what it names: a chat's
  conversation rows and its result's rows with `forget_chat`; a
  `Delegate` row with an adoption. Retention (`later.md`) ends the rest,
  a forgotten chat's replies' `Update` rows among them, which pages skip
  until then (store.md 7.4).

## Questions to raise when reached

- **A forgotten chat's replies** (12.1): settled by the designer (jig
  `36de4733`, store.md 7.4, "An owner erased first"). `forget_chat`
  erases the chat's ended row and its result's index rows; its replies'
  `Update` rows stay until retention erases the activity lines they name,
  since a chat's replies are unbounded and a decision is bounded. A stored
  updates page skips a row whose owner is gone, counting it within its
  bounds.
- **Goals' order** (12.4). store.md 7.3 reads ended goals newest first;
  today's core pages them oldest first. This plan follows the design.

## Done when

- `git -C ~/src/rust/jig grep -n -E 'inbox_restore_key|restore_key\(' --
  crates tests testing examples` prints nothing.
- `git -C ~/src/rust/jig grep -n -E 'ConversationNewest|EndedDelegate|Goal \{
  project' -- crates/jig-domain-tasks/src/boundary.rs` prints the keys.
- `heavy cargo nextest run -p jig-domain-tasks -p jig-domain-core -p
  jig-tasks-world -p jig-core-world -p jig-conformance-world -p
  jig-ops-world` passes once on `main` after 12.5.
