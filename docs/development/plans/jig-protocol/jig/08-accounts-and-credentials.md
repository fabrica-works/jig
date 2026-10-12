# Session 08: accounts on skein's driver's terms, grants pushed and resent, an application's credentials

The accounts child stops refreshing, numbering and keeping on its own:
skein's accounts driver does the three in the protocol layer (jig 21),
and the child wants each configured account, hears each generation
granted, an account expiring, or a failure in one of five classes, and
keeps which accounts are usable and spent. Each new generation then
reaches each live attempt that holds it, on any host, and the fleet
sends each attempt it keeps its current grants at its worker's hello
and at its adoption, which replaces the core's `unreachable!` for the
fleet's grants. Last, an application's credentials: named by the
application's number and a generation, granted through the root,
named by a connector with a run's workspace items, carried with the
assignment, and pushed and resent as an account's are.

## Read first

- jig's `domain/engine.md`, section 12 and 12.1 (accounts, an
  application's credentials), section 8 ("Grants follow their
  attempts"), section 6, step 1 (LLM accounts at the restart's first
  step) and 7.1 (the assignment's credential grants);
  `protocol/llm.md`, 5.1 to 5.4 and section 6; `protocol/channel.md`,
  5.2 (the grant record), 5.3 (after a redial), section 8 and section 13
  ("The domain sends grants after a hello"); `domain/connectors.md`,
  section 10 ("The credentials they need"); `domain/authority.md`, 8.3
  (accounts and credentials usable before a run starts);
  `client/domain/pages.md`, section 7 ("LLM accounts: each usable or
  not, and why").
- skein's `docs/design/oauth.md`, section 4 and 6.3 (the driver's
  vocabulary), and `crates/skein-oauth-accounts/src/boundary.rs` (its
  `Request`, `Event`, `Failure`, `Place`, `Unloaded`).
- `findings/domain.md`: "Order and dependencies" (the bullet on
  `children.rs:284-290`), "Defects found on the way" (item 4's two
  admission defects; `tests/accounts/tests/simulation.rs`), item 4,
  item 5 and "Files over 800 lines these items touch".
- `findings/protocol.md`, 1.6 ("Accounts") and 6.1 ("06-accounts-and-grants").

## What the design asks

- **Each configured account is wanted,** "from the restart's first
  step (section 6) to the close" (engine.md, section 12); jig keeps "no
  refresh schedule, keeping or generation numbering of jig's"
  (`protocol/README.md`, 12.3).
- **The child's vocabulary** (llm.md, 5.2): it asks want, rejected,
  release, close; it hears granted (generation, how long valid),
  failed later (with how long, "the driver's delay, or the child's
  backoff"), sign in again, **expiring** ("the account is usable until
  its token lapses, and the deployment's document says a person must
  hand in a newer one"), misconfigured, unloaded (where and why),
  cancelled, closed.
- **Usable** "while the child holds a granted generation still valid
  and the account is not spent"; rejected passes to the component, which
  refreshes once; exhausted marks it spent until a time and asks
  nothing; sign in again and unloaded are asked again at the child's
  retry interval; misconfigured stays unusable until a restart; "Close
  releases every account, and ends once the component has closed"
  (engine.md, section 12).
- **Grants follow their attempts** (engine.md, section 8): "each new
  generation of an LLM account or an application's credential goes to
  every live attempt that holds one ..., and each attempt the fleet
  keeps at a worker's hello, or adopts, is sent its current grants".
- **An application's credentials** (engine.md, 12.1): "Named, never
  seen: each by the application's number for it and a generation, with
  how long it is valid"; "Granted through the root"; "Lent to runs: a
  connector names those a run's workspace needs as it gathers its
  items; the assignment carries them as grants, each naming the
  application's table"; "Rejected and exhausted are LLM accounts' only."
- **A grant names whose table** its value comes from, jig's accounts'
  or the application's (channel.md, section 8).

## Choices this plan makes

- **The child speaks to its component through two enums of its own,**
  `ToComponent` and `FromComponent`, carried in its `Request` and
  `Event`, so jig 21 translates those two and nothing else.
- **The configured accounts are the child's construction:**
  `accounts::Domain::new(limits, accounts)`, with the charters' accounts
  (jig 07's `Charter::account`), each once. The restart's first step
  wants them: `core::want_accounts`, which a root calls once in the
  decision it makes as its restart begins, as it calls `resume_fleet`.
- **Later without a delay waits the child's backoff;** sign in again and
  unloaded wait the child's retry interval (`Limits::retry`); the
  refresh margin and the rejected interval go.
- **An unusable account at either admission check makes the run wait**
  (`remember_due`), as it does at the first check today, and a grant
  missing at preparation does too; neither holds nor fails the task.
- **The fleet keeps each attempt's grants,** by name, with the current
  generation and when it lapses on the fleet's clock, taken from its
  start or its adoption and from each new generation; it sends them
  when a hello lists the attempt and when it adopts it. A grant past its
  lapse is not sent. The value stays the protocol layer's.
- **A grant to a host is `Now`:** it follows no commit (engine.md, 5.2
  lists no grant).
- **Grants name their table** in the three copies of the grant: jig
  05 gives `host::Grant` and `fleet::Grant` their `table: GrantTable`
  (its 05.1); 08.2 gives the accounts child's `Grant` the same, keeps
  jig 05's names (`GrantTable`, and `account` for an account's number or
  the application's number for its credential), and makes the hub hand
  its agent jig's accounts' grants only, keeping the application's names
  for its workspace's operations.
- **The test connector names a credential per resource** in its
  configuration (`ResourceSpec::credential`), and gives it with each
  item it gathers; the test root gathers items from its connectors as
  it answers `Now::WorkspaceRequest`, which it does not today.

## Rules that bind it

- "Once jig 14 or jig 19 has landed a family's codec, an increment of any session
  that changes a durable record of that family changes its schema, its
  generated codec and its translation in the same increment, and the
  record's family's version moves at the first such change in this
  plan" (README, "Rules"). 08.4 changes `RunProof`, a core record.
- No increment of this plan "adds an `unreachable!` arm for a variant
  the reference service can reach; an arm the service reaches is
  routed"; nor "adds a field or predicate that its counterpart across a
  boundary lacks" (code-organization.md, section 6): the three grant
  copies and the client's account line change together.
- "A file: at most 800 lines, generated code excepted. A file of at
  most 800 lines that the increment grows past 800 is split in the same
  increment"
  (code-organization.md, section 7). Over 800 already, and touched
  here: `crates/jig-domain-fleet/src/attempt.rs` (1,110),
  `crates/jig-domain-host/src/hosted.rs` (1,340),
  `testing/jig-test-domain/src/routes.rs` (1,005),
  `testing/jig-test-domain/src/lib.rs` (1,154),
  `examples/ops/domain/src/route.rs` (1,585): each grows by at most 15
  lines here; new code goes in new modules.

## Increments

### 08.1 The accounts child on the driver's terms

Scope: engine.md, section 12; llm.md, 5.2 and 5.4. The child's
vocabulary below; each configured account wanted at the restart's first
step; usable, spent, expiring and the five failure classes, each with
its next ask; rejected passed to the component; close releasing each
account and ending on the component's closed. The core routes the
child's `Request::Component` out as `Now::Account`, as it routes the
child's requests today, and the worlds' stand-in component answers a
want with a grant. The two admission defects of item 4 are fixed. The
deployment document's account line, and the client's `OwnerAccount`
that mirrors it, carry the new state and `expiring` in place of the
refresh states and `unsaved`.
Needs: nothing beyond the session's start (jig 07's one account per
charter).
Files: `crates/jig-domain-accounts/src/{boundary.rs, domain.rs,
limits.rs, lib.rs}` (rewritten, 572 lines today);
`crates/jig-domain-core/src/lib.rs` (`Core::new`, line 377;
`quiescent`, line 649), `routing/children.rs` (`tag_account` 303-320,
`route_account` 335-345), `routing/mod.rs` (new `want_accounts`, beside
`resume_fleet` 384), `routing/run_preparation.rs` (lines 66-81, 365),
`documents/inbox_deployment.rs` (lines 110-124, 594-650, 824);
`crates/jig-domain-client/src/contract/owner_deployment.rs` (lines
85-93, 185-196); `crates/jig-view/src/owner_deployment.rs` (lines
83-90, 520) and `owner_deployment_words.rs` (the account phrases);
`testing/jig-scripted-engine/src/owner_deployment.rs`;
`testing/jig-test-domain/src/lib.rs` (`Event::RestartBegin`, 531-538);
`examples/ops/domain/src/restart.rs` (its restart's begin);
`tests/accounts/{src/lib.rs, tests/simulation.rs,
tests/fuzzy_simulation.rs, tests/memory.rs}` (rewritten);
`tests/core/src/{world.rs, effects.rs, faults.rs}`,
`tests/conformance/src/lib.rs`, `tests/ops/src/conformance/mod.rs`
(the stand-in component); `tests/client/src/joint/wire/deployment.rs`.
Users counted by the audit: `Event::Add` 23 sites (crates/jig-core/src/tests.rs:1682;
examples/ops/domain/src/tests.rs:317; tests/core/src/world.rs:424;
tests/core/src/effects.rs:202, 338; tests/core/src/faults.rs:77;
tests/core/src/stories/instructions.rs:46; tests/core/tests/runs.rs:94,
restart.rs:18, stories.rs:1406, memory.rs:30; tests/conformance/src/lib.rs:614;
tests/ops/src/conformance/mod.rs:603; tests/accounts/tests 11);
`Event::Refreshed`/`Request::Refresh` at tests/core/src/world.rs:497,
tests/core/src/effects.rs:540, tests/conformance/src/lib.rs:454,
tests/ops/src/conformance/mod.rs:512, routing/children.rs:312;
`Limits` constructed at crates/jig-core/src/tests.rs:137,
tests/core/src/world.rs:141, tests/accounts/src/lib.rs:6; `State` mapped
at inbox_deployment.rs:116, tests/client/src/joint/wire/deployment.rs:160-167
(and its tests 56, 357-361), crates/jig-core/src/tests.rs:1706;
`OwnerAccountState` 35 sites in 7 files (client contract 2, view 7,
view tests 6, scripted engine 2 and its tests 6, joint wire 6 and its
tests 6); `unsaved` 12 sites. Shared hotspots: `routing/children.rs`
(jig 05 routes `Rejected`/`Exhausted` there), `routing/run_preparation.rs`
(jig 07), `crates/jig-domain-core/src/lib.rs`, `tests/core/src/effects.rs`.
Shape: the sketch's `jig_domain_accounts` vocabulary. `Domain::new(limits:
&Limits, accounts: Box<[u32]>) -> Domain`; `usable(account) -> bool`;
`grant(account, now) -> Option<Grant>`; `waiting() -> bool` (true while
a close awaits `Closed`); `next_deadline`, `account_views`, `pop_fact`
as today. In the core: `pub fn want_accounts(core: &mut Core, env:
&Env<Limits>) -> Requests` (steps the child with `Event::Want`); the
`accounts::Event::{Rejected, Exhausted}` names stay, so a route jig 05
lands from the fleet's notices compiles unchanged. `MAX_OUT` becomes
`max_out(limits) = limits.accounts + 2` (a want of each, at the start or
the close).
Test shape:
- `tests/accounts/tests/simulation.rs` (rewritten; `World` in
  `tests/accounts/src/lib.rs` keeps `step` and `fire_at`), unit-speed
  world, under 20 ms each: `every_configured_account_is_wanted_at_the_start`,
  `a_grant_is_usable_until_its_token_lapses`,
  `later_asks_again_after_the_drivers_delay`,
  `later_without_a_delay_waits_the_childs_backoff`,
  `sign_in_again_is_unusable_and_asked_again_at_the_retry_interval`,
  `misconfigured_is_never_asked_again`,
  `unloaded_says_where_and_why_and_is_asked_again`,
  `expiring_stays_usable_and_says_a_newer_record_is_needed`,
  `a_rejection_goes_to_the_component_and_changes_nothing_else`,
  `exhausted_is_spent_until_its_time_and_asks_nothing`,
  `close_releases_every_account_and_ends_on_closed`. Reference:

  ```rust
  #[test]
  fn sign_in_again_is_unusable_and_asked_again_at_the_retry_interval() {
      let mut world = World::wanting(&[ANALYST]);
      world.step(Event::Component(FromComponent::Granted {
          account: ANALYST,
          generation: 4,
          valid: Duration::from_secs(100),
      }));
      assert_eq!(
          world.step(Event::Component(FromComponent::Failed { account: ANALYST, failure: Failure::SignInAgain })),
          vec![]
      );
      assert!(!world.domain.usable(ANALYST));
      assert_eq!(world.views()[0].state, State::SignInAgain);
      assert_eq!(world.fire_at(29), vec![]);
      assert_eq!(world.fire_at(30), vec![Request::Component(ToComponent::Want { account: ANALYST })]);
  }
  ```

  (`LIMITS.retry` 30 s; `ANALYST` an account number constant in
  `tests/accounts/src/lib.rs`; `World::wanting(accounts)` builds the
  domain with those accounts and steps `Event::Want`; `World::views()`
  is `account_views()` as a `Vec`.) `tests/accounts/tests/fuzzy_simulation.rs`:
  its `respond` (line 5) plays skein's driver: granted, later, sign in
  again, expiring, misconfigured, unloaded, cancelled, closed; its
  referee: no account usable without a valid granted generation, none
  asked while closed.
- `tests/core/tests/runs.rs`, world, under 0.5 s each:
  `a_run_waits_while_its_account_must_sign_in_again`,
  `a_run_waiting_for_its_account_starts_once_granted`,
  `an_account_unusable_at_the_second_check_waits_and_is_not_held`.
- The client's account line: `tests/client/src/joint/wire/deployment/tests.rs`
  and `testing/jig-scripted-engine/src/tests/owner_deployment.rs`: their
  6 account tests each take the new state names (the expected values
  change because the vocabulary does).
Proof: world: `heavy cargo nextest run -p jig-core-world --test runs
a_run_wait` shows a chat whose account must sign in again never
assigned, then assigned once the stand-in component grants it; once,
under 2 s.
Don't: keep any record, refresh timer or generation counter in the
child; name skein's driver's types in a domain crate (the child's
`Place` and `Unread` are its own); send `Availability` anywhere.
Size: at most 620 lines of production code (the child's 572 go).
Removes: `Event::{Add, Refreshed, Failed}`'s old shapes;
`Request::{Refresh, Keep, Cancel, Availability, Refused}`;
`Failure::{Unavailable, TimedOut, RateLimited, Refused, Unsaved}`;
`State::{Fresh, Refreshing, Retrying, Revoked}`; `Fact::unsaved`;
`Limits::{refresh_margin, rejected_interval}`; the document's and the
client's `unsaved`; the 9 tests of jig's own refresh in
`tests/accounts/tests/simulation.rs`.

### 08.2 Grants name their table, and the hub's agent hears accounts' only

Scope: channel.md, section 8 ("A grant names whose table its value comes
from: jig's accounts' or the application's"); llm.md, section 6;
hosts.md, 6.8 and section 7 ("No credentials on disk"). The accounts
child's `Grant` gains `table: GrantTable`, as jig 05.2 gives the hub's
and the fleet's; the hub admits an assignment's grants distinct by
`(table, account)` (assignment.rs:12-20 compares the account alone),
renews each by a newer generation of the same `(table, account)`
(hosted.rs:1198-1224), and hands its agent jig's accounts' grants only,
at its start (hosted.rs:495-501) and as they renew. The application's
grants are made by the hub's tests here, and by the core from 08.4.
Needs: nothing beyond the session's start (jig 05.2's `GrantTable` in
the hub and the fleet).
Files: `crates/jig-domain-accounts/src/{boundary.rs (`Grant`, `GrantTable`),
domain.rs (`grant`)}`; `crates/jig-domain-core/src/routing/run_preparation.rs`
(line 77); `crates/jig-domain-host/src/{assignment.rs (lines 12-20),
hosted.rs (lines 27, 419-431, 495-501, 1198-1224)}`; the roots'
translations of a grant: `testing/jig-test-domain/src/routes.rs:421`,
`examples/ops/domain/src/assemble.rs:104`; `tests/host/src/{turn_world.rs
(160, 182, 416, 491-495), engine.rs (288)}`, `tests/host/tests/memory.rs:135`,
`crates/jig-domain-host/src/tests.rs:85`. Users of `host::Grant` the
audit counted: hosted.rs:27, 419-431, 497-498, 1198-1224; assignment.rs:12-20;
examples/ops/domain/src/assemble.rs:104; tests/host/src/turn_world.rs:160,
182, 416-417, 491-495; tests/host/tests/memory.rs:135-136;
tests/host/src/engine.rs:288; crates/jig-host/src/tests.rs:85.
Shared hotspots: `crates/jig-domain-host/src/hosted.rs` (jig 05),
`crates/jig-domain-host/src/assignment.rs` (jig 05).
Shape: `jig_domain_accounts::GrantTable { Accounts, Application }` and
`Grant { table: GrantTable, account: u32, generation: u64, valid:
Duration }`, mirroring jig 05's `jig_domain_host::Grant` field for
field; each root translates the child's into the fleet's and the hub's.
Test shape: `crates/jig-domain-host/src/tests.rs`, unit, under 20 ms:
`an_applications_grant_is_kept_and_never_handed_to_the_agent`,
`an_accounts_grant_renews_the_agents_grant`,
`two_grants_of_one_number_in_different_tables_are_admitted`;
`tests/host/tests/turns.rs`, world, under 0.5 s:
`a_newer_account_grant_reaches_a_started_agent`.
Proof: world: `heavy cargo nextest run -p jig-host-world --test turns
a_newer_account_grant_reaches_a_started_agent` shows a renewed LLM
grant handed to the hosted agent; once, under 1 s.
Don't: let the hub hand an application's grant to its agent; read a
grant's value anywhere; rename jig 05's fields.
Size: at most 80 lines of production code.
Removes: the hub's refusal of two grants of one number in different
tables as duplicates.

### 08.3 Grants pushed and resent

Scope: engine.md, sections 8 and 12; llm.md, section 6; channel.md, 5.3
and section 13. A new generation the child lends goes to the fleet,
which sends it to each live attempt that holds its account, on any
host, the engine's own included; the fleet keeps each attempt's grants
from its start and its adoption and sends them when a hello lists the
attempt and when it adopts it; the core routes the fleet's grant as
`Now::Grant`, in place of the `unreachable!` at children.rs:287; the
assignment carries its grants as a list (`Now::RunPrepared::grants`);
both roots route `Now::Grant` to their host (the test root out to its
world; `ops`'s root to its hub on channel 0, `host::Event::Grant`).
Needs: 08.2's tables; jig 05.2's fleet grant vocabulary (`Grant`, its
name, `Event::{Start, Adopt, Grant}`, `Request::Grant`) and the hub's
`RunAssignment`, used as jig 05.2 leaves them: this increment fills
them, and changes none of their variants or fields. In the core it
changes `Now::RunPrepared`'s `grant` to `grants` and adds `Now::Grant`
(jig 05's Choices: jig 04, 06 and 08 add their own fields to
`Now::RunPrepared`).
Files: `crates/jig-domain-fleet/src/boundary.rs` (`Event::{Start, Adopt,
Grant}`, `Grant`), `grants.rs` (new: the kept grants, `keep`, `renew`,
`resend`), `attempt.rs` (the `Attempt`'s grants; at most 15 lines where
`moved` and `located` are called), `call.rs` (`grant`, line 199, goes),
`domain.rs` (line 321), `limits.rs` (`grants`), `tests.rs` (helpers 120-150);
`crates/jig-domain-core/src/routing/children.rs` (lines 287-290, 371,
`tag_account`), `routing/vocabulary.rs` (`Now::Grant`; `Now::RunPrepared::grants`),
`routing/run_preparation.rs` (line 139), `sibling_routes.rs` (line 169,
the adoption's grants); `testing/jig-test-domain/src/{lib.rs (Assignment,
line 145), routes.rs (lines 208, 421, 537)}`; `examples/ops/domain/src/{route.rs
(lines 531, 1245, 1403), assemble.rs (line 104)}`; `tests/fleet/src/world.rs`
(lines 935-990, 1252, 1302), `tests/core/src/{effects.rs, peers.rs}`,
`tests/core/tests/restart.rs`. Users counted by the audit: `fleet::Event::Start`
constructed at examples/ops/domain/src/route.rs:531,
testing/jig-test-domain/src/routes.rs:208 and the fleet worlds; `Adopt`
at crates/jig-core/src/sibling_routes.rs:169 and the fleet worlds;
`fleet::Event::Grant` matched at crates/jig-core-fleet/src/domain.rs:321,
crates/jig-core/src/routing/children.rs:371, tests/fleet/src/world.rs:1302;
exhaustive `fleet::Request` matches at crates/jig-core/src/routing/children.rs
(`tag_fleet`), tests/fleet/src/world.rs:935-990, tests/fleet/src/turns.rs:242-260;
`Now::RunPrepared` field-naming matches at examples/ops/domain/src/route.rs:1245
and testing/jig-test-domain/src/routes.rs:421 (5 `..` matches unchanged).
Shared hotspots: `routing/{children.rs, vocabulary.rs}`,
`examples/ops/domain/src/route.rs`, `testing/jig-test-domain/src/routes.rs`,
`crates/jig-domain-fleet/src/attempt.rs`, `tests/core`.
Shape: the sketch's fleet changes. The fleet keeps, per attempt, at most
`Limits::grants` entries `{ name: GrantName, current: Option<(generation,
lapses: Time)> }`; `Event::Grant { grant }` renews each attempt that
holds the grant's name and sends `Request::Grant` to each on a channel;
`fleet::max_out` grows by `tracked(limits)` for a grant and by
`slots * grants` for a hello. In the core, `tag_account`'s
`Request::Granted` steps the fleet with `fleet::Event::Grant` in the same
decision; `fleet::Request::Grant` becomes `Now::Grant { channel, run,
attempt, grant }`.
Test shape:
- `crates/jig-domain-fleet/src/tests.rs`, unit, under 20 ms each:
  `a_new_generation_reaches_each_live_attempt_of_its_account`,
  `a_new_generation_skips_attempts_of_another_account`,
  `an_adrift_attempts_grant_waits_for_its_workers_hello`,
  `an_adopted_attempt_is_sent_its_current_grant_when_listed`,
  `a_lapsed_grant_is_not_sent_at_a_hello`. Reference:

  ```rust
  #[test]
  fn an_adrift_attempts_grant_waits_for_its_workers_hello() {
      let mut harness = Harness::new();
      harness.hello(WORKER, 1, &[], &[]);
      harness.start_granted(CHAT, FIRST, 7, ANALYST_FIRST);
      harness.lost(WORKER);
      assert_eq!(harness.grant(ANALYST_SECOND), Box::from([]));
      let sent = harness.hello(WORKER, 1, &[], &[hosted(CHAT, FIRST)]);
      assert!(sent.contains(&Request::Grant { channel: WORKER, run: CHAT, attempt: FIRST, grant: ANALYST_SECOND }));
  }
  ```

  (`Harness::start_granted(run, attempt, workstream, grant)` and
  `Harness::grant(grant)` beside `start` and `adopt`, tests.rs:134-150;
  `ANALYST_FIRST` and `ANALYST_SECOND` are generations 1 and 2 of
  account 1, valid 60 s.)
- `tests/fleet/tests/simulation.rs`, world, under 0.5 s:
  `a_worker_back_within_its_grace_receives_its_attempts_grants`; the
  fleet world's parent (world.rs:935-990) records `Request::Grant`.
- `tests/core/tests/restart.rs`, world, under 0.5 s each:
  `a_new_generation_reaches_the_live_chat`,
  `a_restart_sends_an_adopted_chat_its_current_grant`.
Proof: world: `heavy cargo nextest run -p jig-core-world --test restart
a_restart_sends_an_adopted_chat_its_current_grant` shows a chat adopted
after a restart sent its account's current generation once its worker
lists it; once, under 1 s.
Don't: hold a value in the fleet or the core; make a grant `Held`; send
a grant to an attempt the fleet fenced off.
Size: at most 300 lines of production code.
Removes: the `Grant` arm of the `unreachable!` at children.rs:287 (the
other three arms are jig 05's). The per-attempt `fleet::Event::Grant`
and `call::grant` go in jig 05.2, not here.

### 08.4 An application's credentials in the accounts child and the core

Scope: engine.md, 12.1; authority.md, 8.3. The child keeps a second
table, the application's credentials, each granted by the root
(`Event::Credential`), usable while its granted generation is valid,
never rejected or exhausted; a new generation is pushed as an
account's is. The core's `WorkspacePrepared` names the credentials a
run's workspace needs; both admission checks add each one's
usability to `RunAsk::accounts`, and an unusable one makes the run wait
as an account does; `Now::RunPrepared::grants` carries the account's
grant and each credential's, naming their tables; `RunProof` keeps the
credentials' numbers so an adopted attempt is sent them (08.3).
Needs: 08.3's grants pushed and resent.
Files: `crates/jig-domain-accounts/src/{boundary.rs, domain.rs,
limits.rs}`; `crates/jig-domain-core/src/routing/vocabulary.rs`
(`Event::WorkspacePrepared`, line 397), `routing/mod.rs` (lines 334,
443), `routing/run_preparation.rs` (lines 39-81, 139), `run.rs` (line
68), `connector.rs` (line 592), `stored.rs` (`RunProof`, lines 517-534,
576), `sibling_routes.rs` (line 169), `memory.rs` (line 112),
`documents/inbox_deployment.rs` (line 824); `testing/jig-test-domain/src/routes.rs`
(lines 391, 419), `examples/ops/domain/src/route.rs` (line 1243); the
`RunProof` literals at crates/jig-core/src/tests.rs:455, 1719 and
crates/jig-core/src/tests/activity.rs:68, 220; and, if jig 19 has
landed, the core family's schema, generated codec and translation in
`crates/jig-protocol-store`. Users of `WorkspacePrepared` the audit
counted: routing/mod.rs:334, 443; testing/jig-test-domain/src/routes.rs:391,
419; examples/ops/domain/src/route.rs:1243. Shared hotspots:
`routing/{vocabulary.rs, run_preparation.rs}`, `stored.rs` (jig 06 and
jig 11 change `RunProof` too).
Shape: the sketch. `Event::WorkspacePrepared { task, attempt, writes,
credentials: Box<[u32]> }`; `RunProof { credentials: Box<[u32]>, .. }`;
`accounts::Limits::credentials`.
Test shape: `tests/accounts/tests/simulation.rs`, under 20 ms each:
`a_credential_is_usable_while_its_generation_is_valid`,
`a_credentials_new_generation_is_granted_on`; `crates/jig-domain-core/src/tests.rs`,
unit: `a_run_waits_while_a_credential_it_needs_is_unusable`.
Proof: unit layer here, since no root names a credential before 08.5:
`heavy cargo nextest run -p jig-domain-core
a_run_waits_while_a_credential_it_needs_is_unusable`; 08.5's Proof is at
the world layer; once, under 1 s.
Don't: let the child refresh, reject or exhaust a credential; keep a
credential's source in the domain.
Size: at most 160 lines of production code.
Removes: nothing.

### 08.5 The test application names its credentials

Scope: connectors.md, section 10; engine.md, 12.1. The test connector's
configuration names, per resource, the credential its workspace needs,
and each item it gathers carries it; the test root gathers its
connectors' items as it answers `Now::WorkspaceRequest`, passes the
credentials they name in `WorkspacePrepared`, and routes the
application's grants in (`Event::Credential`, through its world, which
stands in for the application's credential source).
Needs: 08.4's credentials in the core.
Files: `testing/jig-test-connector/src/{boundary.rs (ResourceSpec, line
260; Item, line 412), values.rs (lines 80-117)}`, `client_marks.rs:175`,
`tests.rs` (lines 230, 431); `testing/jig-test-domain/src/{routes.rs
(lines 385-420, 766), lib.rs (Event, line 166)}`,
`client_documents.rs:149`; `tests/test-connector/src/world.rs` (lines
153, 236); `tests/core/src/world.rs` (the fixture's resource specs);
`tests/core/tests/credentials.rs` (new). Users of `ResourceSpec {` 8
constructions; of `Item` the sites above. Shared hotspots:
`testing/jig-test-domain/src/routes.rs`, `tests/core/src/world.rs`.
Shape: `ResourceSpec { credential: Option<u32>, .. }`; `Item {
credential: Option<u32>, .. }`; `jig_test_domain::Event::Credential {
number: u32, generation: u64, valid: Duration }`, routed to
`core::Event::Account(accounts::Event::Credential { .. })`.
Test shape: `tests/core/tests/credentials.rs` (new), world, under 0.5 s
each: `a_run_needing_a_credential_waits_until_it_is_granted`,
`the_assignment_carries_the_credential_named_by_its_table`,
`a_new_credential_generation_reaches_the_live_run`,
`an_adopted_run_is_sent_its_credential_again`. Reference:

```rust
#[test]
fn the_assignment_carries_the_credential_named_by_its_table() {
    let (config, limits) = jig_core_world::world::with_forge_credential(151);
    let mut world = World::configured(151, false, config, limits);
    world.send(root::Event::Credential { number: FORGE, generation: 3, valid: Duration::from_secs(60) });
    world.delegate_on_resource(FORGE_REPOSITORY);
    let assignment = world.assigned.as_ref().expect("the delegate is assigned");
    assert!(assignment.grants.contains(&accounts::Grant {
        table: accounts::GrantTable::Application,
        account: FORGE,
        generation: 3,
        valid: Duration::from_secs(60),
    }));
}
```

(`world::with_forge_credential(seed)` in `tests/core/src/world.rs`: the
shared config whose first connector's resource `FORGE_REPOSITORY` names
credential `FORGE`; `World::delegate_on_resource(path)` beside
`delegate`, effects.rs:961.)
Proof: world: `heavy cargo nextest run -p jig-core-world --test credentials`
shows a run that needs the forge's credential waiting until the
application grants it, then assigned with it, a new generation reaching
it, and an adopted run sent it again; once, under 3 s.
Don't: give the core a credential's value or source; change `ops`
beyond what the compiler asks (`ops` names no credential).
Size: at most 120 lines of production code.
Removes: nothing.

## Interface sketches

```rust
// jig_domain_accounts (08.1, 08.3, 08.4): owned here; jig 21 translates
// ToComponent and FromComponent to skein's driver and back.
use alloc::boxed::Box;
use skein_lib::{Duration, Time};

/// Whose table holds a grant's value (channel.md, section 8), as jig 05
/// names it in the hub and the fleet. 08.2.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GrantTable {
    /// jig's accounts' table: an LLM account's access token.
    Accounts,
    /// The application's own table: one of its credentials.
    Application,
}

/// A grant's name and how long it is valid; never its value.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Grant {
    pub table: GrantTable,
    /// The account's number, or the application's number for its credential.
    pub account: u32,
    pub generation: u64,
    pub valid: Duration,
}

/// The parent's and the component's inputs to the child.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    /// The restart's first step: want every configured account. Once.
    Want,
    /// From the accounts' component.
    Component(FromComponent),
    /// A run's provider refused this generation; the fleet fenced it first.
    Rejected { account: u32, generation: u64 },
    /// A run's provider says the account is spent for `retry_after`.
    Exhausted { account: u32, retry_after: Duration },
    /// From the application's root: a credential's generation granted. 08.4.
    Credential { number: u32, generation: u64, valid: Duration },
    /// The engine closes: release every account, then close the component.
    Close,
}

/// What the accounts' component says (llm.md, 5.2), in the child's words.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FromComponent {
    /// A generation lent, and each newer one while wanted; its value went to
    /// the protocol layer's table.
    Granted { account: u32, generation: u64, valid: Duration },
    /// A handed-in account's token nears its lapse: usable until then.
    Expiring { account: u32, generation: u64 },
    /// The account's want ended, in one of five classes.
    Failed { account: u32, failure: Failure },
    /// The terminal of `ToComponent::Close`.
    Closed,
}

/// Why an account's want ended (engine.md, section 12).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Failure {
    /// Wanted again after `after`, or after the child's backoff.
    Later { after: Option<Duration> },
    /// Revoked, expired or without a record: wanted again at the retry interval.
    SignInAgain,
    /// The provider rejected the deployment's client: never wanted again.
    Misconfigured,
    /// Its record could not be read: wanted again at the retry interval.
    Unloaded { at: Place, why: Unread },
    /// The close cancelled it.
    Cancelled,
}

/// Where loading stopped, in the child's words (skein's `Place`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Place {
    Directory,
    File,
}

/// Why loading stopped, in the child's words (skein's `Unloaded`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unread {
    Unreadable,
    Refused,
    Failed,
    Stalled,
    TooLarge,
}

/// What the child asks of its component (llm.md, 5.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ToComponent {
    /// Hold the account and lend its token: answered `Granted` or `Failed`.
    Want { account: u32 },
    /// A run's provider rejected this generation.
    Rejected { account: u32, generation: u64 },
    /// Stop holding the account; no answer.
    Release { account: u32 },
    /// Close the component; `Closed` is its terminal.
    Close,
}

/// The child's outputs.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    /// To the accounts' component, through the root's protocol layer.
    Component(ToComponent),
    /// A new generation lent: the core gives it to the fleet (08.2).
    Granted { grant: Grant },
    /// The close has ended: the component closed.
    Closed,
}

/// An account's state for the deployment's document.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum State {
    /// Wanted, no generation granted yet.
    Asking,
    /// A generation granted and valid.
    Granted,
    /// Failed later: wanted again at `until`.
    Later { until: Time },
    SignInAgain,
    Misconfigured,
    Unloaded { at: Place, why: Unread },
    Cancelled,
    Closing,
}

/// One account's line, secret-free.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Fact {
    pub account: u32,
    pub state: State,
    pub generation: Option<u64>,
    pub usable: bool,
    /// A handed-in token nears its lapse: a person must hand in a newer one.
    pub expiring: bool,
    pub spent_until: Option<Time>,
    pub attention: bool,
}

/// The child's limits.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub accounts: u32,
    /// The application's credentials kept at once. 08.4.
    pub credentials: u32,
    pub backoff_base: Duration,
    pub backoff_max: Duration,
    /// How long sign in again and unloaded wait before the next want.
    pub retry: Duration,
    pub spent_attention: Duration,
    pub facts: u32,
}

// jig_domain_fleet::boundary (08.2, 08.3): the changed events. The fleet's
// `GrantTable` and `Grant` are jig 05's (05.2); the child's mirror them
// field for field (08.2), and each root translates between them.
use skein_lib::{ReplyTo, Token};

/// A grant's name in the fleet: whose table, and its number there. 08.3.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GrantName {
    pub table: GrantTable,
    pub account: u32,
}

pub enum FleetEvent {
    /// As today, and the run's grants at its assignment.
    Start {
        reply_to: ReplyTo,
        run: Token,
        attempt: Token,
        workstream: u64,
        kinds: Kinds,
        assignment: Assignment,
        grants: Box<[Grant]>,
    },
    /// As today, and the grants the attempt holds: each by name, and the
    /// current generation of those that have one.
    Adopt {
        reply_to: ReplyTo,
        run: Token,
        attempt: Token,
        kept: u32,
        kind: HostKind,
        worked: bool,
        holds: Box<[GrantName]>,
        grants: Box<[Grant]>,
    },
    /// A new generation, for every live attempt that holds its name.
    Grant { grant: Grant },
}

// jig_domain_core::routing::vocabulary (08.3, 08.4)
use jig_domain_fleet as fleet;

pub enum Now {
    /// A grant for the host on `channel` (0, the engine's own): its value is
    /// filled in by the protocol layer from the table it names.
    Grant { channel: Token, run: Token, attempt: Token, grant: fleet::Grant },
    // RunPrepared { .., grants: Box<[accounts::Grant]> } in place of `grant`.
}
```

`FleetEvent` stands for `jig_domain_fleet::Event`'s three changed
variants; `Kinds`, `HostKind`, `Assignment`, `GrantTable` and `Grant`
there are the fleet's own. The child owns usability and spending; the fleet owns where
each attempt's grants go; the protocol layer owns the values. A want
ends with `Failed` or lives until `Release` or the close; the close ends
with `Closed`.

## Questions to raise when reached

- **The retry interval and the backoff** are numbers of jig 20's
  profiles; report the defaults the fixtures take (30 s, 2 s to 8 s).
- **An application credential the fleet holds but the hub cannot
  use:** a worker whose configuration has no source for the
  application's table drops the value (channel.md, section 8, "A grant
  whose generation the table no longer holds is left out"); none
  expected otherwise.

## Done when

- The child keeps nothing of the driver's: `git -C ~/src/rust/jig grep
  -n -E 'refresh_margin|rejected_interval|Unsaved|Refreshing' --
  crates/jig-domain-accounts crates/jig-domain-core` prints nothing.
- The fleet's grants are routed: `git -C ~/src/rust/jig grep -n
  'Now::Grant' -- crates/jig-domain-core/src/routing/children.rs`
  prints one line (none on 2026-10-12, when the arm is `unreachable!`).
- The worlds pass: `heavy cargo nextest run -p jig-accounts-world`
  passes, and `heavy cargo nextest run -p jig-core-world --test runs
  --test restart --test credentials` passes.
- The gate passed on this session's last landing: `git -C ~/src/rust/jig
  log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step:.jig-08\.' origin/main | head -1` names
  `scripts/check.sh passed`.
