# Session 35: the client's link: documents

The documents half of jig 32 (split for size): jig's documents decoded
from `jig-protocol-client-wire`'s records into the client's vocabulary,
exhaustive both ways, checked against the client's limits. It needs only
jig 32.1, so it runs beside jig 32's reads, stream and small store.

## Read first

jig 32's "Read first", "What the design asks", "Choices this plan
makes" and "Interface sketches" (`jig/32-client-link.md`): they bind
this session as they bind jig 32.

## Rules that bind it

- "**The client's crates** (`jig-domain-client`, `jig-view`,
  `jig-protocol-client-wire`, `jig-protocol-client-link`,
  `jig-shell-native`) depend on no engine-side crate."
  (`code-organization.md`, section 3)
- "No crate of this plan links a browser or wasm32 toolchain."
  (`code-organization.md`, section 2) No `wasm32` in any manifest, cfg or
  script of this session.
- "**Step code** follows skein's `programming-model.md`, section 10, and
  jig's `AGENTS.md`: no traits, no generic items, no `dyn`, no
  closures." (`code-organization.md`, section 2)

## Increments

### 35.1 Documents from records: the frame, tasks and live state

Scope: `translate/documents.rs` and `translate/live.rs`: the wire's
`Frame`, `Task`, `Chip` and its parts, `Tree`, `RunState`, `RunText`,
`CurrentCall`, `LiveCall`, `Conversations`, `Inbox`, `Goals`, `Tasks`,
`Deployment`, `OwnerDeployment`, `OwnerProject` to the contract's types,
exhaustive, field for field; each document checked against the client's
limits as it is decoded (`Text::new` within `Limits::text`, lists within
their bounds, the record's measured bytes within
`Limits::document_bytes`); one past refused as `Undecodable::TooLarge`.
Pure functions; reads and deliveries use them from 32.2 and 32.3.
Needs: nothing beyond 32.1.
Files: `src/{translate/documents.rs, translate/live.rs}`;
`src/tests/{documents.rs, fixtures.rs}`.
Shape: `translate::documents::frame(record: v1::Frame, limits: &Limits)
-> Result<contract::Frame, Undecodable>`, one function per document.
Test shape: `a_frame_record_becomes_the_contract_s_frame`,
`a_task_one_row_past_the_client_s_limit_is_refused`,
`a_run_text_keeps_its_turn_retry_and_offsets`,
`a_chip_in_each_phase_becomes_the_contract_s_chip`. Expected values
written as literals of the contract's types, shaped as the client
world's fixtures (`tests/client/src/stories.rs:17` `frame`, `:163`
`task`). Under 20 ms each. Reference test of this kind, in full:

```rust
#[test]
fn a_frame_record_becomes_the_contract_s_frame() {
    let frame = translate::documents::frame(fixtures::frame_record(), &fixtures::limits());
    assert_eq!(frame, Ok(fixtures::frame()));
}
```

Proof: unit, `heavy cargo nextest run -p jig-protocol-client-link`, run
once: a frame record and a chip in each phase become the contract's
types, a run's text keeps its retry and offsets, and a task one row past
the client's limit is refused (the four tests); under 0.2 s.
Don't: cut a document to fit (one past its limits is refused, as the
Scope says).
Size: at most 600 lines.
Removes: nothing.

### 35.2 Documents from records: pages and single reads

Scope: `translate/pages.rs` and `translate/objects.rs`: the wire's pages
(`ActivityPage`, `HistoryPage`, `InboxPage`, `ConversationPage`,
`GoalPage`, `SpendPage`, `NotePage`, `ObjectPage` and their owner
variants) and single reads' documents (`Proposal`, `OwnerProposal`,
`Escalation`, `OwnerEscalation`, `Question`, `PersonTask`,
`OwnerPersonTask`, `Result`, `Project`, `OwnerHistorical`) to the
contract's types, checked as 35.1; an activity line's turn bytes decoded
with `jig_protocol_transcript::decode`, one that does not decode carried
up as the contract's undecodable turn.
Needs: nothing beyond 35.1.
Files: `src/{translate/pages.rs, translate/objects.rs}`;
`src/tests/{pages.rs, fixtures.rs}`; the crate's `Cargo.toml`
(`jig-protocol-transcript`).
Shape: as 35.1.
Test shape: `an_activity_page_decodes_each_turn`,
`an_undecodable_turn_crosses_up_as_such`,
`a_page_keeps_its_two_cursors_as_bytes`,
`a_historical_task_keeps_its_ending`. Under 20 ms each.
Proof: unit, `heavy cargo nextest run -p jig-protocol-client-link`, run
once: an activity page decodes each turn, an undecodable turn crosses up
as such, a page keeps its two cursors as bytes, and a historical task
keeps its ending (the four tests); under 0.2 s.
Don't: decode a cursor (it is the engine's).
Size: at most 600 lines.
Removes: nothing.

## Done when

- The documents' tests pass: `heavy cargo nextest run -p
  jig-protocol-client-link` passes.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/jig log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step:.jig-35\.' origin/main | head -1` names
  `scripts/check.sh passed`.
