# Session 34: the engine's clients' side: documents

The documents half of jig 31 (split for size): jig's documents
encoded from the core's vocabulary into `jig-protocol-client-wire`'s
records, exhaustive both ways, the frame, tasks and chips, runs,
conversations, the inbox, goals and the deployment, pages and single
reads. It needs only jig 31.1, so it runs beside jig 31's asks, reads
and stream.

## Read first

jig 31's "Read first", "What the design asks", "Choices this plan
makes" and "Interface sketches" (`jig/31-engine-clients.md`): they bind
this session as they bind jig 31.

## Rules that bind it

- "**`jig-protocol-*` crates** depend on no application's crate. Only
  `jig-protocol-charter` and `jig-protocol-transcript` link smith"
  (`code-organization.md`, section 3). This crate links neither smith nor
  any client crate but `jig-protocol-client-wire`.
- "**Step code** follows skein's `programming-model.md`, section 10, and
  jig's `AGENTS.md`: no traits, no generic items, no `dyn`, no
  closures." (`code-organization.md`, section 2)
- "No increment of this plan adds an `unreachable!` arm for a variant the
  reference service can reach; an arm the service reaches is routed."
  (`code-organization.md`, section 6)
- **No secret reaches a domain:** no `core::Event` the component sends
  holds a token, a link's secret or a digest; the test
  `no_core_event_holds_a_token_or_digest` (31.1) checks it.

## Increments

### 34.1 Documents: the frame, tasks and chips

Scope: `translate/owner.rs`, `translate/chips.rs`, `translate/task.rs`:
the core's `documents::Frame`, `Task`, `Chip` and their parts
(specification, contract, authority, executor, funder, numbers, budget,
ledger, wake, subscriptions, holds, phases, resources) to the wire's
records. A port of the joint world's `wire/owner.rs`, `wire/chips.rs` and
`wire/task.rs`. Pure functions; reads and deliveries use them from 31.4
and 31.5. A document's texts cross as the wire's `text` fields, UTF-8
(client/domain/contract.md, 5.3, "Texts are UTF-8"; the designer,
2026-10-13, jig `1bc30fd0`): the engine makes no other, so a text the
wire refuses is the engine's bug, answered as a failed translation, never
passed as bytes.
Needs: nothing beyond 31.1, and a step of the supervisor's first: the
core cuts a text at a character's boundary (`documents/small.rs:9-11`),
the client's `Text::new` refuses bytes that are not UTF-8
(`contract/common.rs:13-18`) and the contract's "opaque" goes, a party's
words that are not UTF-8 are refused at their request's entrance, and
the joint world's login fixture (`tests/client/src/joint/wire/owner.rs:822,
845`) becomes UTF-8; with a core unit test that a text cut inside "é"
keeps the whole character out and says it was cut.
Files: `src/{translate.rs, translate/owner.rs, translate/chips.rs,
translate/task.rs}`; `src/tests/{documents.rs, fixtures.rs}`.
Shape: `translate::owner::frame(value: &documents::Frame, limits:
&v1::Limits) -> Result<v1::Frame, v1::Problem>`, and one function per
document, named after the joint world's.
Test shape: one test per document at a fixture value, expected records
written as literals: `a_frame_with_two_projects_becomes_its_record`,
`a_task_with_a_hold_becomes_its_record`,
`a_chip_in_each_phase_becomes_its_record`,
`a_frame_past_the_wire_s_bound_is_refused_naming_it`. Under 20 ms
each. The fixture documents are the joint world's
(`tests/client/src/joint/wire/owner.rs`'s tests), their expected records
written out as literals in `fixtures.rs`. Reference test of this kind, in
full:

```rust
#[test]
fn a_frame_with_two_projects_becomes_its_record() {
    let limits = fixtures::wire_limits();
    let record = translate::owner::frame(&fixtures::frame_two_projects(), &limits);
    assert_eq!(record, Ok(fixtures::frame_two_projects_record()));
}
```

Proof: unit, `heavy cargo nextest run -p jig-protocol-engine-clients`,
run once: a frame, a task and a chip in each phase become the wire
records written as literals, and a frame past the wire's bound is
refused naming it (the four tests); under 0.2 s.
Don't: read the client's limits (the client checks its own).
Size: at most 600 lines.
Removes: nothing.

### 34.2 Documents: runs, conversations, inbox, goals and the deployment

Scope: `translate/live.rs`, `translate/inbox.rs`, `translate/deployment.rs`:
`documents::Run`, `views::Writing` and `views::Call` to `RunState`,
`RunText`, `CurrentCall`, `LiveCall`; `Conversations`, `Inbox`, `Goals`,
`Tasks`, `Tree`, `Deployment`. A port of the joint world's
`wire/documents.rs` (its delivery half), `wire/inbox.rs`, `wire/rows.rs`
and `wire/deployment.rs`.
Needs: nothing beyond 34.1.
Files: `src/{translate/live.rs, translate/inbox.rs,
translate/deployment.rs}`; `src/tests/{documents.rs, fixtures.rs}`.
Shape: one function per document, as 34.1.
Test shape: `a_run_writing_with_a_call_becomes_its_run_state`,
`a_run_text_keeps_its_turn_retry_and_offsets`,
`an_inbox_with_both_groups_becomes_its_record`,
`a_deployment_document_becomes_its_record`. Under 20 ms each.
Proof: unit, `heavy cargo nextest run -p jig-protocol-engine-clients`,
run once: a run's state and text, an inbox with both groups and a
deployment document become their wire records (the four tests); under
0.2 s.
Don't: invent a turn's retry: it is the views' `TurnName::retry`.
Size: at most 600 lines.
Removes: nothing.

### 34.3 Documents: pages

Scope: `translate/pages.rs`, `translate/activity.rs`,
`translate/history.rs`: the core's `reads::Page` rows to the wire's
`ActivityPage`, `HistoryPage`, `InboxPage`, `ConversationPage`,
`GoalPage`, `SpendPage`, `NotePage`, `ObjectPage` and their owner
variants, with an activity line's turn bytes carried unchanged. A port of
the joint world's `wire/pages.rs`, `wire/activity.rs`, `wire/history.rs`,
`wire/ledger_pages.rs` and `wire/notes.rs`. Cursors are 31.4's: here a
page's ends are the core's positions, handed to a function 31.4 writes.
Needs: nothing beyond 34.1.
Files: `src/{translate/pages.rs, translate/activity.rs,
translate/history.rs}`; `src/tests/{pages.rs, fixtures.rs}`.
Shape: `translate::pages::page(read: &reads::Read, page: &reads::Page,
cursors: &mut Cursors, limits: &v1::Limits) -> Result<v1::ReadValue,
v1::Problem>`, `Cursors` being 31.4's encoder, a stub of one function
returning the position's bytes until 31.4, which replaces it.
Test shape: `an_activity_page_carries_each_turn_s_bytes`,
`a_history_page_keeps_its_cut_flag`,
`an_inbox_page_keeps_its_two_groups_apart`,
`a_notes_page_becomes_its_record`. Under 20 ms each.
Proof: unit, `heavy cargo nextest run -p jig-protocol-engine-clients`,
run once: an activity page carries each turn's bytes, a history page
keeps its cut flag, an inbox page keeps its two groups apart, and a
notes page becomes its record (the four tests); under 0.2 s.
Don't: decode a turn's bytes.
Size: at most 600 lines.
Removes: nothing.

### 34.4 Documents: single reads

Scope: `translate/objects.rs`: `Proposal`, `OwnerProposal`, `Escalation`,
`OwnerEscalation`, `Question`, `PersonTask`, `OwnerPersonTask`,
`Result`, `Project`, `OwnerProject`, `OwnerHistorical`, from the core's
`reads::Answer::{Document, Historical}`. A port of the joint world's
`wire/proposals.rs`, `wire/escalations.rs`, `wire/person_tasks.rs`,
`wire/results.rs` (its document half), `wire/projects.rs`,
`wire/historical.rs` and `wire/historical_goal.rs`.
Needs: nothing beyond 34.1.
Files: `src/translate/objects.rs`; `src/tests/{objects.rs, fixtures.rs}`.
Shape: one function per document, as 34.1; `translate::document(answer:
&reads::Answer, ..) -> Result<v1::ReadValue, v1::Problem>` exhaustive
over `documents::Document` and `reads::Past`.
Test shape: `a_pending_proposal_becomes_its_owner_record`,
`a_decided_escalation_keeps_its_decider_and_wall`,
`a_historical_task_keeps_its_ending`,
`a_project_document_becomes_its_record`. Under 20 ms each.
Proof: unit, `heavy cargo nextest run -p jig-protocol-engine-clients`,
run once: a pending proposal, a decided escalation, a historical task
and a project document become their wire records (the four tests); under
0.2 s.
Don't: add a document the wire lacks.
Size: at most 600 lines.
Removes: nothing.

## Done when

- The documents' tests pass: `heavy cargo nextest run -p
  jig-protocol-engine-clients` passes.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/jig log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step:.jig-34\.' origin/main | head -1` names
  `scripts/check.sh passed`.
