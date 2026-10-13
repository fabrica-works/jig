# Session 16: parties' reads in the store

`jig-protocol-store` answers the core's parties' reads (`Held::ReadLoad`,
answered by `Event::ReadLoaded`): each of the 17 `reads::Scope`s is served
from jig 13's intervals over jig's families and jig 12's indexes, or by
point reads from the owner's record where jig 11 keeps the positions, a
page at a time in the step that asks, each row with its joins made by
point reads in the same step (its owner task, live or ended; its
requester ancestry within the domain's depth; its links; the record it
relates to), priced within the read's rows and bytes. Today a world fake
scans the map for each page (`tests/core/src/reads.rs:166-372`); after
jig 18 the domain worlds read through this code instead. Turns, which
live in transcripts' files, join parties' reads in jig 17.

## Read first

- jig's `protocol/store.md`, sections 3, 4.3, 7.1, 7.3, 7.4 and 15
  (items 3 and 7).
- jig's `client/domain/contract.md`, section 6 (what a read names and
  answers); jig's `domain/engine.md`, section 5.3.
- `decisions.md`, 9 (an ended goal's index row keeps its subtree's phase
  counts; an escalation's read shows its latest kept decision), 21
  (positions kept within the record's bound; an older one answers gone),
  23 (ended goals newest first).
- `findings/store.md`, sections 1.3 and 3 (each scope: what the core
  expects, the fake today, the store, the index, the position).
- jig 13's file: `Request::Read` and `Event::Read`, `intervals::scope`
  (13.4, 13.5: `None` for the scopes served from the owner's record) and
  `intervals::delegates`; jig 11's 11.3 (`TaskRecord.question_lines`,
  `hold`, `proposal_rows`); jig 12's 12.1, 12.2, 12.4 and 12.5 (each index
  row: its `value` the index row, its `owner` the task it names).
- The code: `crates/jig-domain-core/src/reads/vocabulary.rs:61-215`
  (`Scope`, `Position`, `Range`, `Row`, `Loaded`), `reads/rows.rs:340-367`
  (`row_key`), `:538-1000` (`object`, `page_row`: what the core does with
  each row and its joins) and `:1404-1546` (`position_allowed`,
  `key_kind`), `reads.rs:124-148` (`backwards`); the fake transport this
  replaces, `tests/core/src/reads.rs:166-372` (`owner`, `row`,
  `candidate`, `page`); jig 15's `pages.rs` and `gets.rs`.

## What the design asks

- **Each scope as intervals with joins** (store.md 7.3): "The component
  serves each scope as intervals over jig's families, and joins each row
  in the same step by point reads: its owner task, live or ended; its
  requester ancestry, within the domain's depth limit; its links; the
  record it relates to." The table of 7.3 says where each scope is served
  from; findings/store.md section 3 maps it to the 17 scopes.
- **Joins are priced:** "Joins are point reads in the same step, priced
  within the read's bounds: a page's rows times the joins each may need."
- **Single reads go by key** (7.3, 15 item 7): a question's line and its
  answer's, an escalation's hold, a task's proposal's row in its
  proposer's history, each named by its position in its task's record.
- **Indexes** (7.4, 15 item 3): a party's inbox by time, a party's
  conversations, a project's goals, a task's ended delegates, a subtree's
  delegates; "Reading backwards: a family whose positions are dense
  (activity) reads newest first by computing forward intervals from its
  last position, which its owner task's record names; a family read
  newest first by time is keyed reversed."
- **Notes** (7.3): "the notes' lines of the project's scopes, or of the
  one scope, filtered by description, each with its entry, by name".
- **A read past its bounds** "is cut and says so, as every load is"
  (7.3): the page ends with a cursor.
- **Reads see what is durable** (3).

## Choices this plan makes

- **A row's price** is `reads::Row::owned_bytes()`
  (`reads/vocabulary.rs:198-203`) with its joins in it; a page stops
  before the first row that would pass the read's `bytes`, or its `rows`,
  with the previous row as its cursor. A first row alone past `bytes`
  answers an empty page with no cursor, and the core answers the party
  as it does an empty page today.
- **Joins per page** are bounded by a component limit, `Limits::joins`
  (store.md 13, "the joins a page may make"); a page stops before a row
  whose joins would pass it, as for bytes.
- **Ancestry** walks the requester chain by point gets of `Live(n)` then
  `Ended(n)`, at most `tasks::Limits::depth` steps
  (`crates/jig-domain-tasks/src/limits.rs:23`), nearest first.
- **Links** are the owner's `dependencies`, `spec.inputs`, `references`,
  `delegates` and `result_followups`, each by point get, in that order
  without repeats (as the fake joins them, `tests/core/src/reads.rs:188-199`).
- **`Loaded::Gone`** answers a single read whose position the owner's
  record no longer keeps (decision 21; jig 11.3: "A read whose position
  the record no longer keeps answers `Why::Gone`"); a missing owner gives
  an empty page, and the core decides what the party sees.
- **`Loaded::Busy`** is answered only when the reads waiting on
  transcripts' frames are at their limit; in this session no read waits
  (jig 17 adds the wait).
- **Notes are served from lines, their positions the lines' keys:** a
  page walks the scopes' lines (jig 13's intervals), by scope, then by
  name within it, joins each line's entry by name, and gives the entry as
  the row's value; a row's position is its line's key, and a page's
  `next` is the last line it examined, so a filtered page goes on past
  what it skipped (store.md, 7.3; the designer, 2026-10-13, jig
  `4a7fda9e`). The core's notes paging changes to match: `row_key`
  (`reads/rows.rs:343`) and `position_allowed` take the line's key for a
  notes row, and `from_store`'s order check holds for it unchanged.
- **Ended goals page newest first** (decision 23), from jig 12's `Goal`
  family, each row's phase counts in its `GoalRow` (decision 9).

## Rules that bind it

- **Shared interfaces:** "the store's boundary (commit, restore, load,
  read, get, transcript, secrets; fill) | `jig-protocol-store`'s
  `boundary.rs` | jig 13". "One session defines each; every other
  session uses it by name and extends it only as its increment says."
  (code-organization.md, section 4.) This session adds nothing to it.
- **Step code** "follows skein's `programming-model.md`, section 10, and
  jig's `AGENTS.md`: no traits, no generic items, no `dyn`, no closures."
  The tests too (jig 13's "Rules": no `Vec`, no lint exception).
- **"No increment of this plan ... adds a test that checks several
  behaviours under one name; ... computes an expected value with the
  code under test"** (code-organization.md, section 6): each scope's test
  states its rows and joins as literals from fixtures, never from the
  component's own output.

## Increments

### 16.1 The read path, its joins, and a task's scopes

Scope: `Request::Read { owner, range }` answered `Event::Read { owner,
result }` in the step that asks (store.md 7.1, 7.3): the scope's intervals
from `intervals::scope(range.party, &range.scope)`; the walk from jig
15's `pages`, from just past the range's `after` position, forwards (a
family keyed reversed reads newest first forwards); each row's joins
(owner, ancestry, links, related) by point gets; the price and the cut
(Choices). The scopes `Task`, `PersonTask` and `Result`: the task by key
(`Live`, then `Ended`, jig 13's intervals), its owner itself, its
terminal (`CoreKey::Terminal { task, attempt }`, the attempt its record
names) as `related`.
Needs: jig 13's scope intervals (13.4); jig 15's walk and gets.
Files: `crates/jig-protocol-store/src/{reads.rs (new), reads/joins.rs
(new), reads/point.rs (new), lib.rs, limits.rs (jig 13's: `joins`)}`;
`src/tests/{fixtures.rs, reads.rs (new)}`.
Shape:
- `reads::read(store: &mut Store, owner: Token, range: core::reads::Range,
  above: &mut Queue<Event>)`.
- `joins::row(store: &Store, value: core::Record, owner:
  Option<&tasks::TaskRecord>, related: Option<core::Key>, spent: &mut
  Spent) -> Option<core::reads::Row>` (none when the row's joins would
  pass what the page has left); `joins::owner(store, task: u64) ->
  Option<tasks::TaskRecord>` (`Live` then `Ended`); `joins::ancestry(store,
  task: &tasks::TaskRecord, depth: u32) -> (Box<[u64]>,
  Box<[Box<tasks::TaskRecord>]>)`, the `Row`'s `ancestors` and its share of
  `linked`; `Spent { rows: u32, bytes: u64, joins: u32 }`.
- `point::task(store, range: &core::reads::Range, task: u64, spent) ->
  Page` for the three scopes.
Test shape: `src/tests/reads.rs` (16.1 to 16.4 add to it; split at 800
lines into `src/tests/reads/{point.rs, paged.rs}`); fixtures from jig 15
(`opened`, `committed`, `header`, `save`, `run`, `ALICE`); this increment
adds `read(party: u64, scope: core::reads::Scope) -> core::reads::Range`
(rows 8, bytes 65 536, forwards, no `after`), `ended_of(task, requester:
tasks::Party) -> core::Record` (its attempt 1), `terminal(task, attempt)
-> core::Record`, `with_links(record, dependencies: &[u64])`, and named
numbers `GOAL = 31`, `CHILD = 32`, `GRANDCHILD = 33`. The reference test:

```rust
#[test]
fn an_ended_task_read_joins_its_terminal_and_ancestry() {
    let mut store = fixtures::opened();
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(1),
            fixtures::save(fixtures::ended_of(GOAL, tasks::Party::Person(ALICE))),
            fixtures::save(fixtures::ended_of(CHILD, tasks::Party::Task(GOAL))),
            fixtures::save(fixtures::terminal(CHILD, 1)),
        ]),
    );
    let range = fixtures::read(ALICE, reads::Scope::Task { task: CHILD });
    let events = fixtures::run(&mut store, Request::Read { owner: Token::new(7), range });
    let [Event::Read { result: reads::Loaded::Page { rows, next: None }, .. }] = &*events else {
        panic!("one page: {events:?}");
    };
    let [row] = rows.as_ref() else { panic!("one row: {rows:?}") };
    assert_eq!(row.ancestors.as_ref(), [GOAL]);
    assert_eq!(row.related, Some(fixtures::terminal(CHILD, 1)));
}
```

Tests: `an_ended_task_read_joins_its_terminal_and_ancestry`,
`a_live_requester_joins_before_an_ended_one`,
`ancestry_stops_at_the_domains_depth_limit`,
`links_join_dependencies_and_delegates_once_each`,
`a_missing_task_reads_as_an_empty_page`,
`a_page_stops_before_the_row_past_its_bytes`,
`a_page_stops_before_the_row_past_its_joins`. Each under 20 ms.
Proof: unit,
`reads::an_ended_task_read_joins_its_terminal_and_ancestry`, run once
with `heavy cargo nextest run -p jig-protocol-store -E
'test(an_ended_task_read_joins_its_terminal_and_ancestry)'`: a read of
an ended delegate comes back with its goal as its ancestor and its
terminal as its related record; under 20 ms. The joint layer (the core
world's read stories over the component) opens in jig 18.
Don't: scan a family to find a row a key names (store.md 7.3); decide
visibility (the core's `loaded` does, `reads.rs:387-411`).
Size: at most 450 lines.
Removes: nothing (the fake transport goes in jig 18).

### 16.2 Single reads by key

Scope: store.md 7.3 and 15 item 7, the point scopes left:
- `Proposal { proposer: Party(p), proposal }`: the person proposal by key
  (`PersonProposal`, then `DecidedProposal`, jig 13's intervals),
  `related` its `CoreKey::ProposalDecision`;
- `Proposal { proposer: Task(t), proposal }`: from the proposer's record
  (`intervals::scope` gives `None`), the revision its `proposal_rows`
  keeps for that proposal (jig 11.3), and the `History { task: t,
  revision }` row by key;
- `Escalation { task }`: from the task's record, `EscalationDecision {
  task, revision: escalation.revision() }` (the latest kept, decision 9),
  `related` `Milestone { task, position: hold }`;
- `Question { task, message }`: from the task's record, the
  `question_lines` entry for that message: `Activity { task, position:
  line }`, `related` `Activity { task, position: answer }`;
- `Reply { task, activity }`: the activity line by key; a `Notified` line
  needs nothing more;
- a position the record no longer keeps: `Loaded::Gone` (decision 21);
- `Turn`, and a reply whose line is a `Turn`: transitional (Removes).
Needs: jig 11's keys for single reads (11.3), and 11.2's
`DecidedProposal`.
Files: `crates/jig-protocol-store/src/reads/point.rs`, `src/tests/reads.rs`.
Shape: `point::proposal`, `point::escalation`, `point::question`,
`point::reply`, `point::turn`, each `(store, range, .., spent) -> Answer`
where `Answer::{Page(Page), Gone}`.
Test shape: `src/tests/reads.rs`; fixtures as 16.1, plus
`person_proposal(number, decided: bool)`, `proposal_decision(party,
number)`, `history_with_proposal(task, revision, proposal)`,
`escalation_decision(task, revision)`, `hold_milestone(task, position)`,
`question_line(task, position, message)`, `answer_line(task, position,
question)`, `notified_line(task, position, person)`, and the owner
records naming their positions (`with_question_lines`, `with_hold`,
`with_proposal_rows`). Tests:
`a_party_proposal_joins_its_kept_decision`,
`a_task_proposal_reads_its_proposers_history_row_by_key`,
`an_escalation_reads_the_decision_its_task_names`,
`an_escalation_joins_its_hold_milestone_by_position`,
`a_question_joins_the_line_that_answers_it`,
`a_question_older_than_the_kept_lines_reads_as_gone`,
`a_reply_reads_its_notified_line_by_key`. Each under 20 ms. No new kind
of test (16.1's reference test is the pattern).
Proof: unit,
`reads::a_task_proposal_reads_its_proposers_history_row_by_key`, run
once with `heavy cargo nextest run -p jig-protocol-store -E
'test(a_task_proposal_reads_its_proposers_history_row_by_key)'`: with
five history rows, the read returns the one revision the proposer's
`proposal_rows` names; under 20 ms.
Don't: fall back to a scan when a position is missing.
Size: at most 350 lines.
Removes: nothing. Transitional, removed by jig 17's "turns in parties'
reads": `Scope::Turn` answers an empty page, and a reply whose line is a
`Turn` answers its line with no `related` (which the core reads as
missing, `reads/rows.rs:770-799`); no world reads through the component
before jig 18, which needs jig 17.

### 16.3 Paged scopes over families

Scope: store.md 7.3 and 7.4:
- `Activity { task }`, newest first: the owner's record names its last
  position (`TaskRecord.activity`); the component reads one forward
  interval of at most `rows` positions below the cursor (or below the
  last plus one), and hands its rows up newest first, each with its owner;
- `History { task, subtree: false }`: jig 13's two intervals (history,
  then milestones), in key order, each row with its owner;
- `Spend { project }`: jig 13's intervals of the project's periods in
  `Ledger` and `ClosedLedger` (jig 11.2), in key order;
- `Notes { project }` (project 0 names the deployment's notes) and
  `ScopedNotes { scope, description }`: the lines (jig 13's intervals),
  for `ScopedNotes` those whose description holds the query
  (`skein_lib::bytes::find`, as `reads/rows.rs:884-895` does), each joined
  with its entry by name, the entry as the row's value (Choices).
Needs: jig 11's closed ledgers apart (11.2).
Files: `crates/jig-protocol-store/src/reads/paged.rs (new)`,
`src/reads.rs`, `src/tests/reads.rs`.
Shape: `paged::activity`, `paged::history`, `paged::spend`,
`paged::notes`, `paged::scoped_notes`, each `(store, range, spent) ->
Page`; `paged::backwards_interval(task: u64, last: u64, before:
Option<u64>, rows: u32) -> skein_kv::Range` (an algorithm easy to get
wrong: the lower bound saturates at position 1).
Test shape: `src/tests/reads.rs`; fixtures as above plus
`activity_line(task, position)`, `ledger_period(project, period, closed:
bool)`, `notes_entry_in(scope, name, description)` with its line. Tests:
`activity_reads_newest_first_from_the_records_last_position`,
`an_activity_cursor_continues_below_its_position`,
`a_tasks_history_joins_history_then_milestones`,
`spend_reads_live_and_closed_periods_of_the_project`,
`project_notes_read_their_scopes_lines_with_entries`,
`scoped_notes_keep_only_matching_descriptions`,
`a_notes_cursor_resumes_after_its_line`,
`notes_across_two_scopes_come_by_scope_then_name`,
`a_filtered_notes_page_goes_on_past_the_lines_it_skipped`. Each under
20 ms.
Proof: unit,
`reads::activity_reads_newest_first_from_the_records_last_position`, run
once with `heavy cargo nextest run -p jig-protocol-store -E
'test(activity_reads_newest_first_from_the_records_last_position)'`:
with positions 1 to 5 and a page of 2, the rows are 5 then 4 and the
cursor names 4; under 20 ms.
Don't: read activity by loading the task's activity family and
reversing it; read notes' entries by scanning entries; change the core's
paging of any scope but notes (Choices).
Size: at most 450 lines.
Removes: nothing.

### 16.4 Paged scopes over indexes

Scope: store.md 7.3 and 7.4, over jig 12's indexes, each row's `value`
the index row and its `owner` the task it names, with that task's joins
(jig 12's row contract):
- `EndedDelegates { task }`: the `EndedDelegate { requester: task, .. }`
  rows forwards (jig 12.5);
- `History { task, subtree: true }`: the subtree collected from
  `Delegate` rows (`intervals::delegates(requester)`), at most
  `tree_tasks` tasks, each member's history and milestone rows read in key
  order (jig 12.5); a cursor names the last row, and the walk resumes at
  its task's `Delegate` row, found by key;
- `Inbox { group, filter }`: the party's `Update` rows (jig 12.1) for the
  updates, results and replies groups, the filter selecting on the row
  or on the owner joined; `related` a reply's activity line;
- `Conversations`: the party's conversation rows, newest first (jig 12.2);
- `Goals { project }`: the project's `Goal` rows, ended goals newest first
  (decision 23), `linked` empty, the counts in the `GoalRow` (jig 12.4).
Needs: jig 12's indexes (12.1, 12.2, 12.4, 12.5) and jig 13's intervals
for them (13.5).
Files: `crates/jig-protocol-store/src/reads/indexed.rs (new)`,
`src/reads.rs`, `src/tests/reads.rs` (split at 800 lines, see 16.1).
Shape: `indexed::ended_delegates`, `indexed::subtree_history`,
`indexed::inbox`, `indexed::conversations`, `indexed::goals`, each
`(store, range, spent) -> Page`.
Test shape: `src/tests/reads.rs` (or `src/tests/reads/paged.rs` after
the split); fixtures as above plus the index rows as jig 12 keys them:
`update(party, at, task, object)`, `conversation(party, at, task)`,
`goal_row(project, ended_at, goal, endings)`, `ended_delegate(requester,
task)`, `delegate(requester, task)`. Tests:
`ended_delegates_read_their_requesters_index`,
`subtree_history_walks_each_delegates_history_in_turn`,
`a_subtree_cursor_resumes_at_its_delegates_index_row`,
`inbox_results_read_newest_first_within_the_group`,
`an_inbox_filter_keeps_one_projects_rows`,
`conversations_read_newest_first_from_the_index`,
`ended_goals_read_newest_first_with_their_counts`. Each under 20 ms.
Proof: unit, `reads::inbox_results_read_newest_first_within_the_group`,
run once with `heavy cargo nextest run -p jig-protocol-store -E
'test(inbox_results_read_newest_first_within_the_group)'`: three results
and two replies for Alice at distinct times, read as `Group::Results`,
come back newest first, results only, each with its ended task as owner;
under 20 ms.
Don't: fall back to scanning `Ended` for a scope an index serves
(findings/store.md 3 lists the fakes that did); map needs groups (never
stored, jig 12).
Size: at most 450 lines.
Removes: nothing.

## Interface sketches

jig 13's `Request::Read { owner, range: reads::Range }` and `Event::Read
{ owner, result: reads::Loaded }` are used as its file sketches them. What
this session adds:

```rust
// jig_protocol_store::reads (this session).
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use skein_lib::{Queue, Token};

pub struct Store; // jig 13's
pub enum Event {} // jig 13's
/// What a page has used of its read's bounds and the component's joins.
pub struct Spent {
    pub rows: u32,
    pub bytes: u64,
    pub joins: u32,
}
/// One scope's page before it is answered.
pub struct Page {
    pub rows: Box<[core::reads::Row]>,
    pub next: Option<core::reads::Position>,
}
/// A point scope's answer: a page, or gone when the owner no longer keeps the position (decision 21).
pub enum Answer {
    Page(Page),
    Gone,
}

/// Serve one party's read in this step.
pub fn read(store: &mut Store, owner: Token, range: core::reads::Range, above: &mut Queue<Event>) {
    todo!()
}
/// One row with its joins, or none when they would pass what the page has left.
pub fn row(
    store: &Store,
    value: core::Record,
    owner: Option<&tasks::TaskRecord>,
    related: Option<core::Key>,
    spent: &mut Spent,
) -> Option<core::reads::Row> {
    todo!()
}
/// The task by key, live first, then ended.
pub fn owner(store: &Store, task: u64) -> Option<tasks::TaskRecord> {
    todo!()
}
/// One forward interval of a task's activity positions, newest page below `before` (store.md 7.4).
pub fn backwards_interval(task: u64, last: u64, before: Option<u64>, rows: u32) -> skein_kv::Range {
    todo!()
}
```

The component owns the walk and the joins; the core owns visibility and
the documents (`reads.rs:387-411`, `reads/rows.rs`); the routing hands
`Event::Read` to the core as `Event::ReadLoaded { load: owner, result }`.

## Questions to raise when reached

- **The inbox's filter by goal** (16.4): if jig 12's `Update` row carries
  the project but not the goal, the component filters on the owner
  joined, which costs a join per row skipped; if a page then stops short
  on its joins with rows left, raise whether the row should carry the
  goal.
- **Notes' positions** (16.3, jig 13's question): settled by the
  designer (jig `4a7fda9e`; store.md, 7.3): a notes row's position is its
  line's key, and a page's `next` the last line examined; the core's
  `row_key` and `position_allowed` take it for notes (Choices).

## Done when

1. The four increments are on `origin/main`: `git -C ~/src/rust/jig log
   origin/main --format=%b | grep -c '^Step: jig-16\.'` prints 4.
2. The reads' tests pass: `heavy cargo nextest run -p jig-protocol-store
   -E 'test(/tests::reads/)'` passes, running at least 25 tests.
3. Each of the 17 scopes has its arm in the component's reads:
   `git -C ~/src/rust/jig grep -n -E 'Scope::(Task|Proposal|Escalation|Question|PersonTask|Result|Reply|Turn|EndedDelegates|History|Activity|Inbox|Conversations|Goals|Notes|ScopedNotes|Spend)' -- 'crates/jig-protocol-store/src/reads*'`
   prints at least 17 lines.
4. jig's gate passed on the last of them: `git -C ~/src/rust/jig log -1
   --format='%(trailers:key=Gate-Result,valueonly)' <its commit>` names
   `scripts/check.sh passed`.
