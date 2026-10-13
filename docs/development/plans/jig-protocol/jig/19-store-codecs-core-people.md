# Session 19: the store's codecs, part 2: the core's and people's families, the format row

The second half of jig 14 (split for size): the core's and the people
child's families' schemas, generated codecs and translations, then the
format row, the dispatch and the sizes check, which need every family.
19.1 to 19.4 need only jig 14's pattern (14.1), so this session runs
beside jig 14's tasks family; 19.5 needs jig 14 whole.

## Read first

jig 14's "Read first", "What the design asks", "Choices this plan makes"
and "Interface sketches" (`jig/14-store-codecs.md`): they bind this
session as they bind jig 14.

## Rules that bind it

- **"Line counts exclude generated code under `src/generated/` and its
  golden files"** (README, "Rules"), and "Generated codecs are committed
  under each codec crate's `src/generated/`, made by skein's generator from
  the crate's `schema/*.schema`, with skein's drift test, as
  `smith-transcript` does" (`code-organization.md`, section 2).
- **"A durable record's codec follows its record. Once jig 14 or jig 19 has landed a
  family's codec, an increment of any session that changes a durable
  record of that family changes its schema, its generated codec and its
  translation in the same increment, and the record's family's version
  moves at the first such change in this plan"** (README, "Rules"). In
  this session: each increment translates the family's types as they are
  on `main` when it starts, and a record another session changes later is
  that session's to follow.
- **"No increment … decodes or encodes bytes in a domain crate"**
  (`code-organization.md`, section 6): the translations are in
  `jig-protocol-store`; the domain crates gain no codec dependency.

## Increments

### 19.1 The core family, part 1: the values it carries, calls

Scope: the tasks schema's records the core carries as bytes (`End` with
`ResultFollowups`, `Problem` with `Refusal`, `Capacity`, `CapacityAction`,
`CapacityCause`, and `Phase` and `Change` as top-level records) with their
encoders; the notes schema's `Entry` likewise; the core schema's small
enumerations (`Proposer`, `ProposalChoice`, `EscalationDecision`,
`ProposalKind`, `EscalationOutcome`, `ProposalOutcome`, `Answer`, `Finding`,
`Source`, `Lacks`, `Judge`, `HostKind`), `OutboxOutcome`, `SettledCall`,
`SettledAnswer`, `DeliveryOutcome`, `CallPart`, `CallKey`, `CallRecord`;
the `Call` row.
Needs: 14.3's `Phase`, `Change`, `New` and `Proposal` in the tasks schema
(`End`'s follow-ups carry them).
Files: `crates/jig-protocol-store/{schema/core-v1.schema, schema/tasks-v1.schema, schema/notes-v1.schema, src/generated/{core_v1.rs, tasks_v1.rs, notes_v1.rs}, golden/{core,tasks,notes}/v1/, src/codec/core.rs, src/codec/core/calls.rs, src/codec/tasks/carried.rs}`, `src/tests/{fixtures.rs, codec.rs}`, `tests/store-codecs/tests/{drift.rs, fuzzy_decode.rs}`.
Shape:
- `codec::core::Limits { core: core_v1::Limits, tasks: tasks_v1::Limits, notes: notes_v1::Limits }`;
  `codec::core::limits(&core::Limits) -> Option<Limits>`; `encode` and
  `decode` over `core::CoreRecord`; this increment's `Row` holds `call`;
  19.2 adds the others (as 14.2 did).
Test shape:
- `src/tests/codec.rs`: `a_call_record_of_each_part_reads_back`,
  `a_refused_delegation_reads_back_with_its_problem`,
  `a_recalled_notes_page_reads_back`, `a_settled_call_reads_back`.
- Ceilings: unit 20 ms.
Proof: unit. `heavy cargo nextest run -p jig-protocol-store -p
jig-store-codecs-world` once: each kind of call answer reads back as saved,
its tasks and notes values decoded by their own families' code. Bound: 1
minute.
Don't: declare `End`, `Phase`, `Change` or `Problem` in the core's schema.
Size: at most 1,200 lines of production code (ruling 130).
Removes: nothing.

### 19.2 The core family, part 2: the deployment, projections, runs, turns, decisions

Scope: `Deployment`, `ProjectionRecord`, `ProjectionKey`, `ProjectionGoal`,
`ProjectionTask`, `ProjectionMilestone`, `MilestoneId`, `RunProof`,
`TurnProof`, `TerminalRecord`, `TurnRecord` (the frame's body jig 17
writes), `ProposalDecisionRecord`, `EscalationDecisionRecord`, and each
other `core::CoreRecord` variant on `main` when the increment starts (jig 06's conversation, if
it has landed).
Needs: 19.1's core schema and `codec::core::Limits`.
Files: `crates/jig-protocol-store/{schema/core-v1.schema, src/generated/core_v1.rs, golden/core/v1/, src/codec/core.rs, src/codec/core/{runs.rs, projections.rs}}`, `src/tests/{fixtures.rs, codec.rs}`.
Shape: `codec::core::encode` and `decode` have one arm per `CoreRecord`
variant and no fallback arm; `codec::core::turn(&core::TurnRecord, &Limits) -> Result<Box<[u8]>, core_v1::Problem>`
and `codec::core::read_turn(&[u8], &Limits) -> Result<core::TurnRecord, core_v1::Problem>`
for jig 17's frames.
Test shape:
- `src/tests/codec.rs`: `the_deployment_header_reads_back_with_its_id`,
  `a_run_proof_with_its_terminal_reads_back`,
  `a_goal_projection_reads_back_with_its_phase`,
  `a_turn_reads_back_with_its_bytes`, `both_decision_records_read_back`.
- Ceilings: unit 20 ms.
Proof: unit. `heavy cargo nextest run -p jig-protocol-store -p
jig-store-codecs-world` once: each core row kind of the fixtures reads back
as saved. Bound: 1
minute.
Don't: leave a `CoreRecord` variant without its arm.
Size: at most 900 lines of production code (ruling 130).
Removes: 19.1's `Problem` arm for rows not yet translated.

### 19.3 The people family, part 1: identities, roles, policy, asks

Scope: `Role`, `Holding`, `IdentityKey`, `Identity`, `RequestKey`, `Last`,
`Pattern`, `Grant`, `ResourceScope`, `Executor`, `Delegation`, `Authority`,
`Judge`, `Guard`, `Requirement`, `PermissionRole`, `PolicyRole`,
`PolicyValue`, `PolicyChange`, `Seen`, `NoteScope`, `NoteChange`,
`ResourceRole`, `ResourceName`, `Adoption`, `Disposal`, `Keep`,
`GoalExecutor`, `PersonResult`, `Proposer`, `ProposalDecision`,
`EscalationDecision`, `PersonAddress`, `AddressChange`, `Parameter`, `Spec`,
`WakeRule`, `ResultsWake`, `WakePolicy`, `Amendment`, `Ask`
(`crates/jig-domain-people/src/{boundary.rs, amendment.rs, policy_value.rs}`);
the `Person`, `SignIn`, `Roles`, `Policy`, `ReadResults` and `ReadChat` rows.
Needs: 14.1's pattern (not the tasks or core families).
Files: `crates/jig-protocol-store/{schema/people-v1.schema, src/generated/people_v1.rs, golden/people/v1/, src/codec/people.rs, src/codec/people/{policy.rs, asks.rs}, Cargo.toml}` (five `[dev-dependencies]`, ruling 136), `src/tests/{fixtures.rs, codec.rs}`, `tests/store-codecs/tests/{drift.rs, fuzzy_decode.rs}`.
Shape: as 14.2; this increment's `Row` holds the rows above; 19.4 adds
`Answer`. `codec::people::limits(&core::Limits) -> Option<people_v1::Limits>`
(ruling 135): a field one kind of row carries takes that row's bound
(`policy_bytes` and `authority` for the `Policy` row, `people`'s
`amendment_bytes` for an ask); a field `Policy` and an ask share takes the
larger. Sums stay the domain's to check, as in the tasks family. The
schema marks `Ask`'s `amend` variant `boxed`, and any other variant
clippy's `large_enum_variant` reports, which the generated enum then
holds in a `Box` (skein's `codec.md`, 3, "A variant marked `boxed`";
skein `482691a7`); its translation boxes and unboxes it. It needs a
`skein-codegen` that reads the word: the supervisor's skein step, and
jig's `skein-codegen` pin moved to it.
Test shape:
- `src/tests/codec.rs`: `a_person_reads_back_with_its_identity`,
  `a_projects_policy_reads_back`, `each_kind_of_ask_reads_back`,
  `a_read_chat_position_reads_back`.
- Ceilings: unit 20 ms.
Proof: unit. `heavy cargo nextest run -p jig-protocol-store -p
jig-store-codecs-world` once: people's rows read back as saved. Bound: 1
minute.
Don't: share a type with the tasks family's schema (no imports); add an
allowance to a generated file, or box a variant clippy does not report.
Size: at most 1,200 lines of production code (ruling 130).
Removes: nothing.

### 19.4 The people family, part 2: refusals, outcomes, answered keys

Scope: `Again`, `DisposalIssue`, `FailureClass`, `HoldReason`, `Moved`,
`Party`, `Ending`, `Lack`, `LimitKind`, `Refusal`, `ProposalChoice`,
`EscalationChoice`, `Outcome`; the `Answer` row; each `people::Stored`
variant on `main` when the increment starts.
Needs: 19.3's people schema and `codec::people` module.
Files: `crates/jig-protocol-store/{schema/people-v1.schema, src/generated/people_v1.rs, golden/people/v1/, src/codec/people.rs, src/codec/people/outcomes.rs}`, `src/tests/{fixtures.rs, codec.rs}`.
Shape: `codec::people::encode` and `decode` have one arm per
`people::Stored` variant and no fallback arm.
Test shape:
- `src/tests/codec.rs`: `an_answered_key_reads_back_with_its_ask_and_outcome`,
  `each_limit_kind_reads_back`, `a_refusal_lacking_a_budget_reads_back`.
- Ceilings: unit 20 ms.
Proof: unit. `heavy cargo nextest run -p jig-protocol-store -p
jig-store-codecs-world` once: each people row kind of the fixtures reads
back as saved. Bound:
1 minute.
Don't: leave a `people::Stored` variant without its arm.
Size: at most 1,100 lines of production code (ruling 130).
Removes: 19.3's `Problem` arm for rows not yet translated.

### 19.5 The format row, the dispatch, sizes

Scope: `codec::encode` and `codec::decode` over `core::Record` by its key's
child; `format::current` and `format::check` (store.md 5.3); `sizes::check`:
the largest key, value and commit from the domain's limits and the
application's, against skein-kv's limits, and the page that must hold one
row (store.md 5.4; `findings/store.md` 5: "A first row larger than
`max.bytes` is `Refused TooLarge`"); the memory test of each family's row
at its worst case.
Needs: the five families' `Row` translations (19.2's core, 19.4's people).
Files: `crates/jig-protocol-store/src/{codec.rs, format.rs, sizes.rs, lib.rs}`, `src/tests/{fixtures.rs, format.rs}` (`format.rs` new), `tests/store-codecs/tests/memory.rs` (new), `tests/store-codecs/Cargo.toml` (`skein-heap`).
Shape:
- `codec::Limits { core: core_v1::Limits, tasks: tasks_v1::Limits, people: people_v1::Limits, notes: notes_v1::Limits }`;
  `codec::limits(&core::Limits) -> Option<codec::Limits>`;
  `codec::encode(&core::Record, &codec::Limits) -> Option<Box<[u8]>>`;
  `codec::decode(&core::Key, &[u8], &codec::Limits) -> Option<core::Record>`
  (the key names the family).
- `format::current(application: u32, deployment: [u8; 16]) -> component::Format`;
  `format::check(found: &component::Format, wanted: &component::Format) -> Result<(), OpenRefusal>`,
  naming the first family whose version differs, or the deployment.
- `sizes::check(domain: &core::Limits, application: &Application, journal: &skein_lib::JournalLimits, store: &skein_kv::Limits) -> Result<Sizes, SizeRefusal>`
  as sketched.
- `src/tests/fixtures.rs` gains `APPLICATION: sizes::Application` and
  `DEPLOYMENT: [u8; 16]`.
Test shape:
- New `src/tests/format.rs` (format and sizes). Reference test, written
  out in full:

  ```rust
  //! The format row and the sizes at start (store.md 5.3, 5.4).
  use super::fixtures;
  use crate::boundary::{Family, OpenRefusal};
  use crate::format;

  #[test]
  fn a_format_row_of_another_tasks_version_is_refused_naming_tasks() {
      let wanted = format::current(fixtures::APPLICATION.version, fixtures::DEPLOYMENT);
      let mut found = wanted;
      found.tasks = 2;
      assert_eq!(
          format::check(&found, &wanted),
          Err(OpenRefusal::Format { family: Family::Tasks, found: 2, wanted: 1 })
      );
  }
  ```
- Also there: `a_format_row_of_another_deployment_is_refused`,
  `the_current_format_row_is_accepted`,
  `a_key_limit_below_the_largest_note_line_is_refused`,
  `a_value_limit_below_the_largest_task_record_is_refused`,
  `a_page_smaller_than_one_row_is_refused`,
  `fixture_limits_fit_their_store` (the expected sizes as literals).
- New `tests/store-codecs/tests/memory.rs` (smith's
  `tests/codecs/tests/memory.rs`). Reference test, written out in full:

  ```rust
  //! A family's full golden row decodes within its heap worst case (skein's codec.md, 6).
  use jig_protocol_store::notes_v1;
  use skein_heap::{Counting, Meter};
  use skein_lib::Reader;

  #[global_allocator]
  static HEAP: Counting = Counting;

  #[test]
  fn a_full_notes_row_decodes_within_its_heap_worst_case() {
      let golden = std::fs::read(
          std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
              .join("../../crates/jig-protocol-store/golden/notes/v1/record_row_full.bin"),
      )
      .expect("full notes row");
      let bound = notes_v1::Row::worst_case_heap(&notes_v1::CEILINGS).expect("notes heap bound");
      let meter = Meter::new();
      meter.start();
      let decoded = notes_v1::Row::decode(&notes_v1::CEILINGS, &mut Reader::new(&golden)).expect("full row");
      let measured = meter.end();
      assert!(measured.peak() <= bound, "decoding peaks within the generated bound");
      drop(decoded);
      assert_eq!(meter.held(), 0, "the decoded row's heap is released");
  }
  ```
  Also the same test for the tasks, core and people families' `Row`.
- Ceilings: unit 20 ms; memory 0.5 s.
Proof: unit. `heavy cargo nextest run -p jig-protocol-store -p
jig-store-codecs-world` once: a store of another version or deployment is
refused saying which, and limits a commit could break are refused at start.
Bound: 1 minute.
Don't: compute a size from fixtures' values; take it from the generated
`worst_case_bytes`.
Size: at most 250 lines of production code.
Removes: nothing.

### 19.s1 Collections the domain bounds by bytes, as pieces; the shipped profiles admitted

A step not in the plan, from the designer's answer on jig 27.3 (jig
`2fa034fb`: `protocol/store.md`, 5.1; skein's `codec.md`, 3, "A sum
crosses as one piece"). The domain bounds a verdict's fields by the
result's bytes (`jig-domain-tasks/src/verdict.rs:16-30`), and an
amendment's lists, an authority's and a policy's by
`people.amendment_bytes` and `policy_bytes`, each member counted at its
`size_of` (`jig-domain-people/src/amendment.rs:141-200`, admitted by
that sum alone, `domain.rs:1106-1112`). The schemas carry them as
counted lists, so `codec::limits` (`src/codec.rs:20`) draws each count
from its sum and refuses both shipped profiles at fourteen ceilings, the
same in both:
- `tasks-v1.schema`: `Verdict.fields`, `VerdictResult.fields`,
  `VerdictMessage.fields` (`verdict_fields`, `verdict_result_fields`,
  `verdict_message_fields`: 512 > 256);
- `people-v1.schema`: `Pattern.segments` (512 > 32), `Delegation.kinds`
  (1,024 > 128), `Authority.grants` and `Authority.note_resources`
  (170 > 32), `PolicyValue.permissions` (2,048 > 1,024),
  `PolicyChangeRequirements.requirements` (113 > 32),
  `PolicyChangeProjections.grants` (170 > 32),
  `PolicyChangePermissions.mappings` (1,024 > 128), `ResourceName.path`
  and `Adoption.options` (512 > 32), `Spec.parameters` (341 > 32).

Capped at the ceilings instead, the store would refuse values the domain
admits, and the largest rows would still be 4.7 MB (small) and 15.9 MB
(standard) for tasks and 71 MB and 143 MB for people (measured on
`2fa034fb`'s parent), which `sizes::check` would ask of skein-kv.
Needs: nothing.
Files: `crates/jig-protocol-store/schema/{tasks-v1.schema,
  people-v1.schema}`, their generated code and goldens;
  `src/codec/{tasks.rs, people.rs}` and the translations of the records
  above; their tests; `tests/store-codecs` (`jig-protocol-config` a
  dev-dependency).
Shape: each collection above, and any other list inside their records
  whose limit `codec::limits` draws from a byte sum, becomes one `bytes
  N` field written and split with `skein_codec::piece::{write, split}`,
  N its schema ceiling, its limit the domain's sum as `codec::limits`
  derives it; decoding counts each item against that sum as the domain
  counts it. A list the domain bounds by a count of its own stays a
  list. Each family's version moves once; regenerated, goldens added.
Test shape: `both_shipped_profiles_are_admitted_by_the_store` (in
  `tests/store-codecs`): for `Profile::Small` and `Profile::Standard`,
  `codec::limits` gives limits and `sizes::check` against skein-kv
  limits at its own largest sizes passes, printing the largest key,
  value and commit; per family, a value at the domain's sum (an
  amendment of 512 empty pattern segments, a verdict of 512 one-byte
  fields) round-trips, and one past it is refused.
Proof: unit, `heavy cargo nextest run -p jig-protocol-store -p
  jig-store-codecs-world` once: the profiles' test passes, and the
  report names each profile's largest value and commit, which jig 27.3
  takes into the profiles' store limits.
Don't: raise a counted list's ceiling; change a profile's limits
  (ruling 235); cap a derived count below what the domain admits.
Size: at most 400 lines of production code, generated code excluded.
Removes: nothing.

## Done when

- Each increment's tests pass in `heavy cargo nextest run -p jig-protocol-store -p jig-store-codecs-world`.
- The format row and the sizes are checked at start: `heavy cargo nextest
  run -p jig-protocol-store -E 'test(/format::/)'` passes, running at
  least 7 tests (19.5's `src/tests/format.rs`).
