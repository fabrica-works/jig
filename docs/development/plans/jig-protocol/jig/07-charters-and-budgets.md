# Session 07: several charters, and the reserve before a run is placed

The core holds the charters the deployment offers in place of the one
charter, run policy and account its settings hold today: each charter
numbered, with its run policy, the hosts that may run it and its default
budget, its models on one account; it names the charter chats run under.
A run is prepared under its task's charter, granted that charter's
account, and placed on a host its charter allows; a goal may name any
charter offered. And no run is placed whose spend cannot hold its
reserve and one of its main model's most costly completions, so smith
never refuses a start for its budget. jig 05's hello check, jig 08's
accounts and jig 20's configuration read the charter table this session
defines.

## Read first

- jig's `domain/engine.md`, 7.1 (a run's charter, one account per
  charter, placed on a host its charter allows), section 8 ("A charter
  says which kinds may run it") and section 14 (configuration);
  `protocol/configuration.md`, section 3 (the charters row) and section
  6 ("Charters' budgets"); `domain/authority.md`, section 7 ("A run's
  spend holds its reserve and one completion more") and 8.3;
  `domain/tasks.md`, 5.5 (held for its budget); `domain/hosts.md`,
  section 8 ("The core places no run smith would refuse");
  `domain/people.md`, section 5's table (start a chat) and section 7;
  `client/domain/contract.md`, section 6 (*not offered*).
- `decisions.md`, 1.
- `findings/domain.md`: "Order and dependencies" (the bullet on
  `core::Settings`), "Defects found on the way" (item 3's fixtures),
  item 2 and item 3.
- smith's `crates/smith-domain-run/src/budget.rs:388-404` (`most_costly`,
  `reserve_spend`): the formula the core's floor matches.

## What the design asks

- **Each charter the deployment offers, numbered** (`configuration.md`,
  section 3): "its instructions ..., its models ..., its tool families,
  the hosts that may run it, its waiting time, whether its runs resume,
  its default budget (turns, spend and time, and a reserve of turns and
  time), which applies to a task whose batch gives none ...; its models'
  endpoints name one account, until a run needs several; and which
  charter chats run under".
- **A run under its task's charter** (engine.md, 7.1): its charter's
  instructions, models, waiting and resume; "its credential grants ...
  those its models' accounts need, one account per charter"; placed "on
  a host its charter allows".
- **The reserve** (authority.md, section 7): "The reserve's spend is its
  turns, each at the main model's most costly completion: a window of
  input at its highest input rate, plus its output at the output rate.
  The core places a run only if its spend holds the reserve's and one
  more such completion". Checked "before a run is claimed and again as
  it starts" (8.3); a task that cannot fund it "waits, or is held for
  its budget".
- **Charters' budgets are configuration's to check** (configuration.md,
  section 6): "each charter's turns and time are above its reserve's,
  and its default budget holds its reserve and one of its main model's
  most costly completions ..., or the engine's start is refused, naming
  the charter." jig 20 makes that check; the core asserts the same as it
  takes its configuration, as it asserts its other configured bounds.
- **A chat** is "a task with a chat's charter" (people.md, section 5);
  a goal is "a tracked task, with the charter or procedure ... asked
  for".

## Choices this plan makes

- **`Settings::charters: Box<[Charter]>`** holds the deployment's
  charters, where the core keeps its settings (`core.settings`, read by
  jig 05's fleet needs); `Settings::chat` names the chats' charter, one
  of them;
  `Settings::{charter, run, account, account_generation,
  account_valid}` go (the last two are read by nothing but fixtures).
- **A charter's run policy stays `RunPolicy`**, moved into the charter
  unchanged: its turns, time and reserve are the default budget's turns,
  time and reserve; the default budget's spend is `Charter::spend`.
- **One account per charter:** `Charter::account()` is
  `policy.model.account`; `Core::new` asserts that each alternative
  model names the same account.
- **The floor** is `(reserve_turns + 1) * most_costly(main)`, computed
  by the core with its own `most_costly` (u128, rounded up once, as
  smith's), never by linking smith; a unit test pins it to smith's on
  literal numbers.
- **The reserve rides `RunAsk`:** `RunAsk::reserve`, and `check_run`
  answers `Wait, Finding::RunBudget` when `budget < reserve`, so the
  five exhaustive `Finding` matches stay unchanged and the run is held
  for its budget as a run below the deployment's minimum is today.
- **The core decides which due task may start:** `Core::take_due` takes
  the first due task whose charter's account is usable (a procedure task
  always), in place of each root popping `due` and reading the one
  account; `Core::due_ready` answers the roots' quiescence.
- **Fixtures keep their budgets:** the shared model's `price_unit`
  becomes 1,000,000 (most costly 1, floor 2), as `findings/domain.md`
  proposes; the stories of the floor set their own `price_unit`.

## Rules that bind it

- No increment of this plan "adds a type beside an existing one for the
  same thing: it changes the type in place, and its users in the same
  increment" (code-organization.md, section 6).
- "A file: at most 800 lines, generated code excepted. A file of at
  most 800 lines that the increment grows past 800 is split in the same
  increment"
  (code-organization.md, section 7). `crates/jig-domain-core/src/lib.rs`
  is 707 lines: the charter table goes in a module of its own.
- "Ceilings per test, unoptimized on the idle machine: a unit test under
  20 ms, a focused world test under 0.5 s, a fuzzy test about 5 s"; the
  focused suite "has about 6 s left for the whole plan"
  (code-organization.md, section 5).

## Increments

### 07.1 The reserve before a run is placed

Scope: authority.md, section 7 and 8.3; tasks.md, 5.5. The core computes
each run's floor from its policy's main model and reserve turns, and
both run checks (`Core::run_admission` before the claim and again after
the workspace, `Core::connector_run_write_admit` for a connector's
write) pass it to authority; a run whose spend is below it is held for
its budget. Against `Settings::run` here; 07.2 moves it to the task's
charter.
Needs: nothing.
Files: `crates/jig-domain-authority/src/boundary.rs` (`RunAsk`, line
257), `check.rs` (`check_run`, line 485; the file is 1,006 lines: add at
most 6), `tests_policy.rs` (lines 638, 725: the two `RunAsk` literals);
`crates/jig-domain-core/src/run.rs` (`run_admission`, line 48; new
`most_costly`, `reserve_floor`), `connector.rs` (line 586); the fixtures
`crates/jig-domain-core/src/tests.rs:279` and `tests/core/src/world.rs:317`
(`price_unit`); `tests/core/tests/runs.rs`. Users of `RunAsk` (4): the
two above in the core and the two in `tests_policy.rs`.
Shape: `authority::RunAsk { reserve: u64, .. }`, the spend the run must
hold, beside `budget`; in `check_run`, after the existing `RunBudget`
test, `if ask.budget < ask.reserve { find(.., Answer::Wait,
Finding::RunBudget) }`. In `jig_domain_core::run`:
`pub fn most_costly(model: &Model) -> Option<u64>` and
`pub fn reserve_floor(policy: &RunPolicy) -> Option<u64>`; a policy whose
floor overflows fails `Core::new`'s pricing assertion (lib.rs:479).
Test shape:
- `crates/jig-domain-authority/src/tests_policy.rs`: a seventh cause in
  `run_cells_hold_for_readiness_propose_grants_and_refuse_hard_caps`
  (line 636): `ask.reserve = 21` with `budget: 20` gives `Answer::Wait`
  and `Finding::RunBudget`; the base gets `reserve: 0`.
- `crates/jig-domain-core/src/tests.rs`, unit, under 20 ms each:
  `most_costly_matches_smiths_rounding_on_a_large_window`,
  `most_costly_takes_the_highest_input_rate`,
  `reserve_floor_counts_the_reserve_and_one_completion_more`. Reference:

  ```rust
  #[test]
  fn most_costly_matches_smiths_rounding_on_a_large_window() {
      // smith's most_costly on the same numbers: (200_000 * 3 + 8_192 * 15) / 1_000, rounded up.
      let model = Model {
          dialect: 1,
          account: 1,
          endpoint: 1,
          name: b"large".as_slice().into(),
          window: 200_000,
          output: 8_192,
          input_price: 3,
          cached_price: 1,
          cache_write_price: None,
          output_price: 15,
          price_unit: 1_000,
      };
      assert_eq!(crate::run::most_costly(&model), Some(723));
  }
  ```
- `tests/core/tests/runs.rs`, world, under 0.5 s each, with a local
  fixture that sets `price_unit` back to 1,000 (floor 20):
  `a_chat_whose_spend_is_below_its_reserve_is_held_for_budget` (rules'
  `maximum_run_spend` 19: no assignment, the task held
  `Hold::Budget`), `a_chat_whose_spend_holds_its_reserve_is_assigned`
  (`maximum_run_spend` 20: assigned with `run.budget == 20`).
  `World::configured` (effects.rs:175) asserts that its setup's chat is
  assigned; the first story builds with `World::configured_unassigned`,
  the same setup without that assertion (at most 15 lines, beside it).
Proof: world: `heavy cargo nextest run -p jig-core-world --test runs
a_chat_whose_spend` shows a chat below the floor never assigned and held
for its budget, and one at the floor assigned; once, under 2 s.
Don't: link smith from the core; add a `Finding` variant; change any
budget in a fixture (the `price_unit` change keeps them).
Size: at most 60 lines of production code.
Removes: nothing.

### 07.2 The charters' table, and runs under their task's charter

Scope: configuration.md, section 3; engine.md, 7.1. `Settings::charters`
and `Settings::chat`; `Core::new` builds the tasks child with the
charters' numbers, asserts each charter (its number unique, its models
on one account, its turns and time above its reserve's, its default
spend at least its floor, its owned bytes within `Limits::run_bytes`),
and the count within `tasks::Limits::charters`; run preparation takes
the task's charter for its policy, resume, account and
`Now::RunPrepared::charter`; `run_policy`, `run_admission` and
`connector_run_write_admit` take the task's charter's account and
floor; the chat checks (`chat_requester`, roles, watch marks, the
deployment's inbox, parties' chat start) use `Settings::chat`; a task's
default instructions in its documents are its charter's; the deployment
document's account line counts runs whose charter names that account;
`supported_task` admits any configured charter; the worst case prices
`tasks.charters` policies; `Core::take_due` and `Core::due_ready`
replace the roots' reads of `due` and the one account. Each
configuration of today becomes one charter, number 1, with the policy
and account it has now.
Needs: nothing (07.1's floor, from the session's own order).
Files: `crates/jig-domain-core/src/charters.rs` (new: `Charter`,
`Core::charter`, `Core::charter_of`, the assertions);
`crates/jig-domain-core/src/lib.rs` (`Config`, `Settings`, `Core::new`;
707 lines, net shrink expected), `run.rs` (lines 37, 62-68),
`connector.rs` (line 592), `inbox.rs` (line 109), `roles.rs` (line 97),
`stored.rs` (lines 287, 386), `memory.rs` (lines 20, 242),
`routing/run_preparation.rs` (lines 77, 89, 134, 245),
`routing/watch_marks.rs` (line 205), `routing/parties.rs` (line 220),
`documents/inbox_deployment.rs` (lines 449, 599; 830 lines: split its
account line into `documents/accounts.rs` if it grows),
`documents/task_tree.rs` (line 786; 1,018 lines: net zero),
`documents/objects.rs` (line 345; 933 lines: net zero),
`documents/small.rs` (line 111), `acts/bounds.rs` (lines 52, 62, 64,
227: `charter` becomes `chat`, behaviour unchanged until 07.3);
`testing/jig-test-domain/src/lib.rs` (lines 430, 943);
`examples/ops/domain/src/domain.rs` (lines 165, 317); the fixtures
`crates/jig-domain-core/src/tests.rs` (lines 258-296, 1004),
`crates/jig-domain-core/src/tests/frame_facts.rs:34`,
`tests/core/src/world.rs` (lines 296-335),
`tests/ops/src/conformance/config.rs` (lines 113-117),
`tests/ops/src/conformance/mod.rs:188`; `tests/core/tests/charters.rs`
(new). Shared hotspots: `crates/jig-domain-core/src/lib.rs` (jig 09
adds the stop beside it), `routing/run_preparation.rs` (jig 08),
`testing/jig-test-domain/src/lib.rs`, `examples/ops/domain/src/domain.rs`,
`tests/core/src/world.rs` (the shared fixture behind 65 calls in 18
files: only its `config` body changes).
Users counted by the audit: `Settings.charter` 13 production uses and
1 test read (frame_facts.rs:34), 2 constructions; `Settings.run` 6
production uses, 2 constructions, 6 test reads; `Settings.account` 4 in
the core and 4 in the roots, 2 constructions; `account_generation` and
`account_valid` 2 constructions each, no reader.
Shape: see the sketch. `Core::charter(number) -> Option<&Charter>`;
`Core::charter_of(task) -> &Charter` (the task's own, which restore and
admission guarantee is configured); `Core::take_due(&mut self) ->
Option<Box<tasks::RunContext>>` (the first due context whose executor is
a procedure, or an agent whose charter's account is usable; the others
keep their order); `Core::due_ready(&self) -> bool`. The roots' release
becomes `if let Some(context) = domain.core.take_due() { .. Activate ..
}`; their quiescence uses `!self.core.due_ready()`.
Test shape:
- `crates/jig-domain-core/src/tests.rs`, unit, under 20 ms:
  `a_charter_whose_alternative_names_another_account_is_refused`
  (`#[should_panic(expected = "one account per charter")]`),
  `a_charter_whose_default_spend_misses_its_floor_is_refused`,
  `take_due_skips_a_task_whose_charters_account_is_unusable`.
- `tests/core/tests/charters.rs` (new), world, under 0.5 s each, with
  the fixture's second charter `world::research_charter()` (number 2,
  account 2, instructions `b"Research the question."`, model
  `b"research"`) and its account granted by the story:
  `a_delegate_on_another_charter_runs_its_instructions_and_account`,
  `a_task_waits_while_its_charters_account_is_unusable`,
  `a_restart_restores_a_task_on_the_second_charter`. Reference:

  ```rust
  #[test]
  fn a_delegate_on_another_charter_runs_its_instructions_and_account() {
      let (config, limits) = jig_core_world::world::two_charters(141);
      let mut world = World::configured(141, false, config, limits);
      world.grant_account(2);
      world.delegate_agent(2, b"find the regression");
      let assignment = world.assigned.as_ref().expect("the delegate is assigned");
      assert_eq!(assignment.charter, 2);
      assert_eq!(assignment.run.policy.instructions.as_ref(), b"Research the question.");
      assert_eq!(assignment.grant.account, 2);
  }
  ```

  `world::two_charters(seed) -> (root::Config, root::Limits)` (in
  `tests/core/src/world.rs`: the shared config with
  `research_charter()` added, `Executor::Charter(2)` in the ceilings
  and the chat's delegation kinds, `accounts.accounts: 2`);
  `World::grant_account(account)` and `World::delegate_agent(charter,
  words)` (in `tests/core/src/effects.rs`, beside `delegate`, line 961).
Proof: world: `heavy cargo nextest run -p jig-core-world --test charters`
shows a delegate on charter 2 assigned with charter 2's instructions and
account while the chat runs on charter 1, and a task on charter 2
restored after a restart; once, under 3 s.
Don't: move `chat_authority` or `tools` per charter (they stay
deployment-wide); change a story's expected value outside
`tests/core/tests/charters.rs`; add a second account child entry point
(jig 08 rewrites the child).
Size: at most 330 lines of production code.
Removes: `Settings::{charter, run, account, account_generation,
account_valid}`; the roots' reads of `core.due` and
`settings.account`.

### 07.3 Goals under any charter the deployment offers

Scope: people.md, section 5 (a goal "with the charter or procedure ...
asked for"); contract.md, section 6 (*not offered*). A party's goal may
name any configured charter that its role's authority allows; one naming
a charter the deployment does not offer is refused *not offered*; the
frame offers, per project, the chats' charter and each charter a goal
may name, as `OfferedCharter { chat, goals, cut }` already carries.
Needs: 07.2's table.
Files: `crates/jig-domain-core/src/acts/bounds.rs` (lines 45-70, 220-230),
`documents/small.rs` (lines 103-112);
`crates/jig-domain-core/src/tests/frame_facts.rs`;
`tests/core/tests/charters.rs`. Shared hotspot: `acts/bounds.rs`.
Shape: `goal_start`'s executor check at bounds.rs:227 becomes
`core.charter(charter).is_none() => Err(people::Refusal::NotOffered)`;
the offered list at bounds.rs:52-64 tries each configured charter, in
configuration order, within `limits.documents.lines`.
Test shape: `tests/core/tests/charters.rs`, world, under 0.5 s each:
`a_goal_on_the_second_charter_is_admitted_and_runs_it`,
`a_goal_naming_an_unoffered_charter_is_refused_not_offered`;
`crates/jig-domain-core/src/tests/frame_facts.rs`, unit:
`the_frame_offers_both_charters_for_goals`.
Proof: world: `heavy cargo nextest run -p jig-core-world --test charters
a_goal_` shows a person's goal on charter 2 admitted and run under it,
and one on charter 9 refused *not offered*; once, under 2 s.
Don't: offer a charter the role's delegation kinds exclude (the
existing `goal_start` judge stays the only gate).
Size: at most 60 lines of production code.
Removes: nothing.

### 07.4 Hosts by charter

Scope: engine.md, section 8 ("A charter says which kinds may run it")
and 7.1 step 3 (the claim records the host kind). `Charter::hosts`;
the core's `Ask::ClaimDone` carries the charter's kinds, which both
roots put in the fleet's `Start`; a claim on a charter whose hosts are
the engine's alone records `HostKind::Engine` in its `RunProof`, so a
restart settles it at once (engine.md, section 6, step 3).
Needs: 07.2's table.
Files: `crates/jig-domain-core/src/charters.rs`;
`routing/vocabulary.rs` (line 89, `Ask::ClaimDone`), `routing/tasks.rs`
(lines 647, 665; 1,414 lines: net zero), `stored.rs` (`RunProof::claimed`,
line 576), `routing/run_preparation.rs` (line 95);
`testing/jig-test-domain/src/routes.rs` (line 204-215),
`testing/jig-test-domain/src/lib.rs` (`Config.hosting`, lines 294, 306,
379); `examples/ops/domain/src/route.rs` (lines 527-538; 1,585 lines:
net zero); the fixtures `tests/core/src/world.rs:262`,
`tests/core/src/peers.rs:546`; `tests/core/tests/charters.rs`. Shared
hotspots: `routing/vocabulary.rs`, `examples/ops/domain/src/route.rs`,
`testing/jig-test-domain/src/{routes.rs, lib.rs}`.
Shape: `core::Ask::ClaimDone { task: u64, attempt: u64, kinds:
fleet::Kinds }`; `RunProof::claimed(task, attempt, transcript_from,
offered, host: fleet::HostKind)`, the host `Engine` iff
`kinds == Kinds::Engine`, `Worker` otherwise (as today).
Test shape: `tests/core/tests/charters.rs`, world, under 0.5 s each:
`a_charter_for_workers_never_runs_in_the_engines_slots`,
`a_charter_for_the_engine_never_runs_on_a_worker`,
`a_restart_settles_an_engine_charters_claim_at_once`.
Proof: world: `heavy cargo nextest run -p jig-core-world --test charters
a_charter_for_` shows each charter placed only on its kind of host;
once, under 2 s.
Don't: let a root choose kinds; change ops's charter (its fixture's one
charter gets `Kinds::Engine`, what route.rs:537 hard-codes today).
Size: at most 60 lines of production code.
Removes: `jig_test_domain::Config::hosting`; ops's hard-coded
`Kinds::Engine` in `route.rs`.

## Interface sketches

```rust
// jig_domain_core::charters (07.2): owned here; read by jig 05 (the
// fleet's hello requirements), jig 08 (the accounts wanted), jig 20 (filled
// from configuration).
use alloc::boxed::Box;
use jig_domain_fleet as fleet;

use crate::RunPolicy;

/// One charter the deployment offers (configuration.md, section 3), as the
/// core holds it from its configuration to the process's end.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Charter {
    /// Its number, unique among the deployment's charters; tasks name it.
    pub number: u32,
    /// Instructions, models (each naming the charter's one account), tool
    /// families, waiting, resume, and the default budget's turns, time and
    /// reserve.
    pub policy: RunPolicy,
    /// The kinds of host that may run it (engine.md, section 8). 07.4.
    pub hosts: fleet::Kinds,
    /// The default budget's spend, in the deployment's unit; at least the
    /// policy's reserve floor.
    pub spend: u64,
}

impl Charter {
    /// The one account its models' endpoints share (engine.md, 7.1).
    #[must_use]
    pub fn account(&self) -> u32 {
        todo!()
    }
}

// jig_domain_core (lib.rs, 07.2): the configuration, each field shown;
// jig 20 fills it, jig 09 adds `workers_in_process`.
use jig_domain_authority as authority;
use jig_domain_people as people;
use skein_lib::{List, Map};

/// Application-supplied initial core state, in jig's vocabulary.
#[derive(Debug)]
pub struct Config {
    /// Identity committed with the first decision.
    pub deployment: [u8; 16],
    /// Deterministic task scheduling seed.
    pub seed: u64,
    /// Initial authenticated owners.
    pub owners: Box<[people::InitialOwner]>,
    /// Configured authority and policy.
    pub authority: authority::Domain,
    /// Deployment-wide settings retained by the core, its charters among them.
    pub settings: Settings,
    /// Configured project identifiers for bootstrap.
    pub projects: List<u32>,
    /// Policy-controlled connector permissions per project.
    pub permission_roles: Map<u32, Box<[people::PermissionRole]>>,
    /// Numbered connectors the application routes for task lifecycle handoffs.
    pub connectors: Box<[u16]>,
}

/// Immutable deployment-wide settings used by core decisions.
#[derive(Debug)]
pub struct Settings {
    /// Time a chat has to finish after its whole close decision.
    pub close_grace: skein_lib::Duration,
    /// Provider number for the application's services.
    pub deployment_provider: u16,
    /// The charters the deployment offers, numbers unique, at most
    /// `tasks::Limits::charters`.
    pub charters: Box<[Charter]>,
    /// The charter chats run under: one of `charters`.
    pub chat: u32,
    /// Maximum committed conversation bytes resumed whole.
    pub resume_bytes: u32,
    /// Funding period identifier.
    pub period: u64,
    /// Initial project period budget.
    pub period_budget: u64,
    /// Initial person pool budget.
    pub person_budget: u64,
    /// Authority of a directly requested chat.
    pub chat_authority: authority::Authority,
    /// Connector number for recurring procedures.
    pub recurring_connector: u16,
    /// Configured one-bit authority family for each engine tool.
    pub tools: crate::ToolFamilies,
}

// jig_domain_core::run (07.1)
/// One completion of `model` at its most: a window of input at its highest
/// input rate and its output at the output rate, rounded up once.
#[must_use]
pub fn most_costly(model: &Model) -> Option<u64> {
    todo!()
}
/// What a run's spend must hold: its reserve's turns and one completion
/// more, each at the main model's most costly (authority.md, section 7).
#[must_use]
pub fn reserve_floor(policy: &RunPolicy) -> Option<u64> {
    todo!()
}

// jig_domain_authority::boundary (07.1): RunAsk's one new field.
pub struct RunAsk {
    /// The spend the run must hold: its reserve and one most costly
    /// completion. A budget below it waits as `Finding::RunBudget`.
    pub reserve: u64,
    // project, authority, numbers, budget, wall, accounts, writes: as today.
}
```

- **Who owns what:** the core owns which charter a run runs under, which
  account it is granted, and where it may run; a root assembles the
  assignment from `RunPrepared` and the fleet's `Start` from
  `ClaimDone`, and never reads the charters. jig 20 fills
  `Settings::charters` and makes configuration.md 6's check before the
  core asserts it.

## Questions to raise when reached

- **A charter's default spend: which tasks take it.** configuration.md,
  section 3: the default budget "applies to a task whose batch gives
  none". Each batch the design names gives its tasks a spend (tasks.md,
  section 4; authority.md, 8.1), and none gives a run's turns or time.
  Until answered, 07.2 carries `Charter::spend` and asserts it holds the
  floor, and a run's spend stays "what its task has left capped per run
  by the deployment" (authority.md, 8.3). Proposed: a `delegate` call
  may leave a task's budget out, and the task takes its charter's
  default spend.
- **A deployment with no chats' charter** (contract.md, section 6: "no
  chat's charter in this project"): `Settings::chat` is required here;
  proposed: keep it required until a deployment needs none.
- **A charter that both kinds of host may run:** its claim commits
  before the fleet places it, so the claim cannot record "the host kind
  it is placed on" (engine.md, 7.1, step 3); 07.4 records `Worker`, as
  today. Proposed: record it as the fleet places it.

## Done when

- The core takes several charters and names the chats' one: `git -C
  ~/src/rust/jig grep -n -E
  'settings\.(charter|run|account)\b|settings\.account_' -- crates
  testing tests examples ':!tests/fleet'` prints nothing (34 lines on
  2026-10-12; `tests/fleet` has a `Settings` of its own).
- A run below its floor is held, one at it assigned: `heavy cargo
  nextest run -p jig-core-world --test runs a_chat_whose_spend` passes.
- A delegate, a goal and a restored task run under their own charter,
  each charter only on its hosts: `heavy cargo nextest run -p
  jig-core-world --test charters` passes.
- The gate passed on this session's last landing: `git -C ~/src/rust/jig
  log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step:.jig-07\.' origin/main | head -1` names
  `scripts/check.sh passed`.
