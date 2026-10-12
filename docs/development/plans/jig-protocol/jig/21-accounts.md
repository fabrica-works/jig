# Session 21: `jig-protocol-accounts`, the accounts child on skein's driver

A new step crate stands between the accounts child, as jig 08 puts it on
skein's driver's terms, and skein's OAuth accounts driver,
`skein-oauth-accounts`: small total functions from what the child asks
to what the driver hears and from what the driver says to what the child
hears, exhaustive both ways; the table of the access tokens the driver
lends, by account and generation, which the protocol layer's routing
reads to fill grants' values; and handed-in accounts, whose records are
read at start and handed in. Its world runs the real child, the
translation and the table over the real driver against skein's fake
issuer, so the two sides meet from the first increment that can. It
links no smith crate (jig's `protocol/llm.md`, sections 5 and 6).

## Read first

- jig's `protocol/llm.md`, sections 5 and 6; `protocol/README.md`,
  section 8; `protocol/configuration.md`, section 7;
  `domain/engine.md`, sections 12 and 12.1; `protocol/channel.md`,
  section 8.
- skein's `oauth.md`, sections 6.2 to 6.4 and 6.8; skein's `lib.md`,
  section 6 (`Secret`).
- `decisions.md`, 4 and 5; `code-organization.md`, sections 2 to 6.
- `findings/protocol.md`, 4.5 (the driver) and 6.2
  ("21-accounts-component"); `findings/domain.md`, item 4.
- The code: jig 08's `crates/jig-domain-accounts/src/boundary.rs` (the
  child's vocabulary toward its component, written here
  `accounts::ToComponent::{Want, Rejected, Release, Close}`,
  `accounts::FromComponent::{Granted, Failed, Expiring, Closed}` and
  `accounts::Failure::{Later, SignInAgain, Misconfigured, Unloaded,
  Cancelled}`; where jig 08 landed other names, use them, and nothing
  else here changes); skein's
  `crates/skein-oauth-accounts/src/boundary.rs` (`Request` `:10-31`,
  `Event` `:36-53`, `Ends`, `Failure`, `Asked`, `Refusal`, `Place`,
  `Unloaded`), `accounts.rs:10-52` (`Account`, `Endpoint`, `Keeper`),
  `component.rs:124` (`Component::new`, which skein 02 makes take a `CryptoRng` keyed by a stream of its own), `:374-392` (`hand_in`),
  `:1081-1090` (`Granted` lends `access_token`);
  `crates/skein-oauth/src/common.rs:43-53` (`Failure`) and
  `record.rs:29-38` (`SavedToken`). The world to copy: skein's
  `tests/oauth-accounts/src/world.rs:50-140` (`limits`), `:214-260`
  (`Client`, the owner with its io and files), `:646-716` (`Process`),
  `:873-886` (`run`), and `testing/skein-fake-peers/src/oauth.rs:62-83`
  (`Peer`).

## What the design asks

- **The driver does the OAuth** (llm.md, 5.1): "jig writes no OAuth
  exchange, and keeps no account's record." "The refresh token never
  leaves the driver. Only access tokens come out of it, each with its
  generation and how long it is valid."
- **The translation** (5.2), "small total functions between the accounts
  child's vocabulary and the driver's, both ways, exhaustive on both",
  with its two tables: want → grant, rejected → rejected, release →
  release, close → close; granted → granted (the token into the table);
  retry later, malformed, limit, not kept → later "with how long (the
  driver's delay, or the child's backoff)"; sign in again, expired →
  sign in again; expiring → expiring; client rejected → misconfigured;
  unloaded → unloaded "with where and why"; cancelled, and a grant
  refused as closed → cancelled; a grant refused as full → later "after
  the child's backoff"; closed → closed.
- **Unreachable in the engine** (5.2): "a sign-in's own failures (an
  invalid redirect), since the engine runs no sign-in; a grant's other
  refusals, for an account the driver was not made with. The translation
  names them, and asserts they never come."
- **The table** (5.3): "at most the two newest generations of each, with
  their expiry on the engine's clock"; "Sized at startup from the
  configured accounts; it never grows"; "An entry goes when its token
  expires, or a newer generation pushes it out."
- **Handed-in accounts** (5.4): "its record is an access token and the
  wall time it lapses, in a private file the configuration names ...
  `jig-protocol-accounts` reads it at start ... and hands it in ...; the
  driver lends it until it lapses, never writing to its source. Past
  that, or once its provider rejects it, the account fails as sign in
  again".
- **Values beside the domain** (README, 8): "no domain state, fact,
  record of the main store or log holds a value. Values move between
  components through the protocol layer's routing, never through a
  root."
- **Grants to runs on workers** (llm.md, 6): `jig-protocol-engine-workers`
  "fills the value in from the accounts' table as it encodes".

## Choices this plan makes

- **The driver's events the engine never meets are asserted never to
  come,** each with its reason: `Visit`, `SignedIn` and a failure ending
  a sign-in (the engine runs no sign-in); `Keep` (the engine's accounts
  use the private keeper, llm.md, 5.1); `Exchange(InvalidRedirect)`; a
  grant refused `Busy`, `NotWaiting`, `Redirect`, `Account`, `Source`,
  `NotAccessOnly`, `Held` or `Record` (the child wants each configured
  account once); every refusal of a `SignIn`, `Redirected` or `HandIn`
  (the layer checks a handed-in record with `skein_oauth::encode_record`
  before handing it in, 21.4).
- **Values are `skein_lib::Secret`** (jig 02): `up` moves `Granted`'s
  token into a `Secret` in the step it arrives, so nothing past the
  translation holds it bare. The table's `Debug` shows counts and
  lengths only.
- **Expiry is the engine's clock's** `now + valid`, taken in the step the
  grant arrives; the table arms no timer: its owner asks `next_expiry`
  and calls `expire`.
- **The handed-in file's bytes are parsed by jig 20,** as configuration's
  other private files are (`jig_protocol_config::handed_in`); this crate
  turns the parsed token, lapse and account id into the driver's
  access-only record and `HandIn` request, so neither crate depends on
  the other.
- **The world is `tests/protocol-accounts`** (package
  `jig-protocol-accounts-world`, ordinary Rust): the child, the
  translation and the table as one owner over skein's io and the
  simulator, with the driver and its private keeper, beside skein's fake
  issuer hosted on simulated loopback, as skein's
  `tests/oauth-accounts/src/world.rs` does. It has no fuzzy test: the
  driver's faults are skein's to fuzz (llm.md, 8: "The driver is
  skein's, tested in skein's worlds; jig tests what it adds").

## Rules that bind it

- "**`jig-protocol-*` crates** depend on no application's crate. Only
  `jig-protocol-charter` and `jig-protocol-transcript` link smith"
  (code-organization.md, section 3): this crate links skein-lib,
  skein-oauth, skein-oauth-accounts and `jig-domain-accounts`, nothing
  else of jig's.
- "No increment of this plan ... adds an `unreachable!` arm for a
  variant the reference service can reach; an arm the service reaches is
  routed." (code-organization.md, section 6): each `unreachable!` names
  why the engine cannot meet its variant, and the world's stories (21.3
  to 21.5, as their Test shapes name them) reach the others; no other
  test is wanted.
- "No increment of this plan ... computes an expected value with the
  code under test." (code-organization.md, section 6)

## Increments

### 21.1 The translation

Scope: the crate `crates/jig-protocol-accounts` (translation and table,
step code); `translate.rs`: `down`, the child's requests as the
driver's; `up`, the driver's events as the child's, a granted token
handed out beside the child's event as a `Secret`; every failure class
and refusal mapped as llm.md, 5.2's tables say; the unreachable ones
asserted with their reasons (llm.md, 5.2).
Needs: nothing beyond the session's start, and jig 02's `skein_lib::Secret`.
Files: `crates/jig-protocol-accounts/{Cargo.toml, src/lib.rs,
src/translate.rs, src/tests.rs, src/tests/fixtures.rs,
src/tests/translate.rs}`; the workspace `Cargo.toml` (members; the
crate, `skein-oauth`, `skein-oauth-accounts`); `tests/boundary/src/`
(the crate's role: skein-lib, skein-oauth, skein-oauth-accounts,
`jig-domain-accounts`; never smith, the core or another child). Shared
hotspots: the workspace `Cargo.toml`, `tests/boundary/src/`.
Shape: as the interface sketch, `jig_protocol_accounts::translate`.
`up` matches every `Event` variant and, inside `Failed` and `Refused`,
every `Failure`, `skein_oauth::Failure`, `Asked` and `Refusal` variant,
with no wildcard arm; the child's `Unloaded`'s place and cause are its
own words for skein's `Place` and `Unloaded`, mapped one for one.
Test shape:
- `src/tests/fixtures.rs` (new): the accounts `ACCOUNT_CHAT` (0) and
  `ACCOUNT_DELEGATE` (1); `TOKEN_FIRST` (`b"token-first"`),
  `TOKEN_SECOND` (`b"token-second"`); `NOW` (`Time::from_nanos(5)`);
  `granted(account, token, generation, valid)`.
- `src/tests/translate.rs` (new), reference test:

  ```rust
  #[test]
  fn a_rate_limited_exchange_fails_later_with_the_drivers_delay() {
      let event = driver::Event::Failed {
          account: ACCOUNT_CHAT,
          ends: driver::Ends::Grant,
          failure: driver::Failure::Exchange(oauth::Failure::RateLimited { retry_after: Duration::from_secs(30) }),
      };
      let up = translate::up(event, fixtures::NOW);
      let expected = accounts::FromComponent::Failed {
          account: ACCOUNT_CHAT,
          failure: accounts::Failure::Later { after: Some(Duration::from_secs(30)) },
      };
      assert_eq!(up, Up::Child(expected));
  }
  ```

  Others: `want_becomes_a_held_grant`, `release_and_close_pass_through`,
  `a_rejected_generation_reaches_the_driver_unchanged`,
  `a_granted_token_is_handed_out_as_a_secret`,
  `a_timed_out_exchange_fails_later_after_the_childs_backoff`,
  `malformed_limit_and_not_kept_fail_later`,
  `refused_and_expired_fail_as_sign_in_again`,
  `an_expiring_record_reaches_the_child_as_expiring`,
  `a_client_rejected_fails_as_misconfigured`,
  `an_unloaded_file_keeps_its_place_and_cause`,
  `a_grant_refused_as_full_fails_later_with_no_delay`,
  `a_grant_refused_as_closed_is_cancelled`,
  `a_sign_in_visit_is_asserted_never_to_come` (`#[should_panic]` with
  its reason's text). Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p jig-protocol-accounts` once:
each of the child's requests reaches the driver's vocabulary, and each
driver event the engine can meet reaches the child in its class; under
10 s with the build warm. Not higher: 21.3 runs it over the real driver.
Don't: write an OAuth exchange, a refresh schedule or a generation
number of jig's (`protocol/README.md`, 12.3); add a wildcard arm.
Size: at most 220 lines of production code.
Removes: nothing.

### 21.2 The table

Scope: `table.rs`: `Table`, sized from the configured accounts and a
token's largest bytes; `granted` keeps a generation's value with its
expiry, pushing out the oldest of three; `value` reads a generation still
held and unexpired; `expire`; `next_expiry`; `worst_case` (llm.md, 5.3;
channel.md, section 8).
Needs: nothing beyond 21.1.
Files: `crates/jig-protocol-accounts/{src/table.rs, src/lib.rs,
src/tests/table.rs, src/tests/fixtures.rs}`.
Shape: as the interface sketch, `jig_protocol_accounts::table`. A value
holds the token and the account's id apart, both `Secret`, the id empty
where its provider names none (llm.md, 5.3; the designer's answer of
2026-10-12, jig `87195d80`, replacing ruling 160's envelope). A value
past its token bound or `ACCOUNT_ID_BYTES`, or for an account past the
table's, is refused (`granted` answers `Err(Unfit)`), never stored.
Test shape:
- `src/tests/table.rs` (new), reference test:

  ```rust
  #[test]
  fn a_third_generation_pushes_out_the_oldest() {
      let mut table = Table::new(2, 32);
      let expires = Time::from_nanos(1_000);
      table.granted(ACCOUNT_CHAT, 1, value(TOKEN_FIRST), expires).expect("fits");
      table.granted(ACCOUNT_CHAT, 2, value(TOKEN_SECOND), expires).expect("fits");
      table.granted(ACCOUNT_CHAT, 3, value(b"token-third"), expires).expect("fits");
      assert!(table.value(ACCOUNT_CHAT, 1, Time::ZERO).is_none());
      let second = table.value(ACCOUNT_CHAT, 2, Time::ZERO).expect("the second generation");
      assert_eq!(second.token.expose(), TOKEN_SECOND);
  }
  ```

  (`fixtures::value(token)`: a `Value` of that token and an empty id;
  `Secret::new` and `expose` as jig 02 lands them.) Others:
  `an_account_id_past_its_bound_is_refused`,
  `a_value_is_gone_at_its_expiry_and_not_before`,
  `a_stale_generation_reads_as_absent`,
  `a_value_past_the_token_bound_is_refused`,
  `the_tables_debug_never_shows_a_value` (formats the table and searches
  for `TOKEN_FIRST`'s bytes: none),
  `the_table_at_its_limits_stays_within_its_worst_case` (in
  `tests/protocol-accounts/tests/memory.rs`, with skein-world's counting
  allocator, from 21.3 on; a unit test here checks `worst_case` answers
  `None` for zero accounts).
Proof: unit, `heavy cargo nextest run -p jig-protocol-accounts` once:
two generations per account at most, each gone at its expiry, and no
value in any `Debug` output; under 10 s with the build warm.
Don't: grow the table; arm a timer.
Size: at most 160 lines of production code.
Removes: nothing.

### 21.3 The world: the child over skein's driver and fake issuer

Scope: `tests/protocol-accounts`: one owner composing jig 08's accounts
child, this crate's translation and table, and skein's driver with its
private keeper over skein's io and the simulator, beside skein's fake
issuer on simulated loopback; the first stories, where the two sides
meet: a wanted account granted, its value in the table and the child
usable; a provider's rejection, refreshed by the driver, the newer
generation in the table and the child told; a close released and closed
(llm.md, 5 and 8).
Needs: nothing beyond 21.2.
Files: `tests/protocol-accounts/{Cargo.toml, src/lib.rs, src/world.rs,
src/owner.rs, src/fixtures.rs, src/referee.rs, tests/stories.rs,
tests/memory.rs}`; the workspace `Cargo.toml` (members; `skein-sim`,
`skein-fake-peers`, `skein-fake-oauth`, `skein-world` as the world's).
Shared hotspot: the workspace `Cargo.toml`.
Shape (test code): `owner::Owner`, a `skein_world::Host` as skein's
`Client` is (`tests/oauth-accounts/src/world.rs:214-260`, `:496`), its
step order: the child's requests to the component through `down`, the
driver's events through `up` into the child and the table, file
requests to `skein_world::files::Driver`; `world::run(seed, story) ->
Outcome<Process>`, as skein's `run` (`:873-886`); `referee::Judge`: no
value in the child's facts or the owner's trace, each generation the
child hears granted readable in the table until its expiry, `Closed`
last.
Test shape:
- `tests/protocol-accounts/src/fixtures.rs` (new): `limits()` (skein's
  `limits()` at `tests/oauth-accounts/src/world.rs:50`, two accounts),
  the account `ACCOUNT_CHAT` signed in with a kept record of generation
  1, the issuer plans `ROTATE` and `REFUSE`.
- `tests/protocol-accounts/tests/stories.rs` (new), reference test:

  ```rust
  #[test]
  fn a_wanted_account_is_granted_and_its_value_lands_in_the_table() {
      let outcome = world::run(1, Story::Granted);
      let owner = outcome.owner();
      assert_eq!(owner.child_heard(), [Heard::Granted { account: ACCOUNT_CHAT, generation: 1 }]);
      assert_eq!(owner.table_value(ACCOUNT_CHAT, 1), Some(b"access-first".as_slice()));
      assert!(outcome.passed());
  }
  ```

  Others: `a_rejected_generation_is_refreshed_and_the_newer_one_lent`,
  `a_close_releases_and_ends_with_closed`,
  `a_value_never_reaches_the_childs_facts`.
- `tests/protocol-accounts/tests/memory.rs` (new):
  `the_table_at_its_limits_stays_within_its_worst_case`.
- Ceilings: each world test under 0.5 s; the world's focused tests at
  most 1 s in all.
Proof: world, `heavy cargo nextest run -p jig-protocol-accounts-world`
once: an account the child wants is granted by skein's driver from the
fake issuer, its value sits in the table and the child hears only the
name, and a rejected generation is refreshed and lent again; under 1
minute with the build warm. The highest layer before jig 25: the real
child and driver, io simulated.
Don't: script the driver; give the child a value.
Size: at most 30 lines of production code (test code excepted).
Removes: nothing.

### 21.4 Handed-in accounts

Scope: `handed_in.rs`: an account's access-only record, made from the
token, the wall time it lapses and the provider's account id (the
record's metadata) as jig 20 parses them
from the private file the configuration names, checked with
`skein_oauth::encode_record` against the driver's limits, and the
`HandIn` request the layer sends before the child's first `Want`; the
world's stories: a handed-in account granted, expiring announced to the
child once, then failed as sign in again past its lapse; a provider's
rejection of a handed-in token failing as sign in again (llm.md, 5.2 and
5.4; skein's `oauth.md`, 6.3).
Needs: nothing beyond 21.3.
Files: `crates/jig-protocol-accounts/{src/handed_in.rs, src/lib.rs,
src/tests/handed_in.rs, src/tests/fixtures.rs}`;
`tests/protocol-accounts/{src/owner.rs, src/fixtures.rs,
tests/handed_in.rs}`.
Shape: as the interface sketch, `jig_protocol_accounts::handed_in`. The
record's generation is 1 at each start, its refresh token `None`, its
`expires_at` the lapse.
Test shape:
- `src/tests/handed_in.rs` (new): `a_handed_in_record_carries_no_refresh_token`,
  `a_token_past_the_drivers_bound_is_unfit`,
  `a_lapse_already_past_is_handed_in_and_fails_at_its_first_grant`.
- `tests/protocol-accounts/tests/handed_in.rs` (new), reference test:

  ```rust
  #[test]
  fn a_handed_in_account_expires_then_fails_as_sign_in_again() {
      let outcome = world::run(1, Story::HandedIn { lapses_in: Duration::from_secs(60) });
      let heard = outcome.owner().child_heard();
      assert_eq!(
          heard,
          [
              Heard::Granted { account: ACCOUNT_HANDED, generation: 1 },
              Heard::Expiring { account: ACCOUNT_HANDED, generation: 1 },
              Heard::Failed { account: ACCOUNT_HANDED, failure: accounts::Failure::SignInAgain },
          ]
      );
  }
  ```

  Others: `a_rejected_handed_in_token_fails_as_sign_in_again`,
  `the_driver_never_writes_a_handed_in_accounts_file`.
Proof: world, `heavy cargo nextest run -p jig-protocol-accounts-world -E
'test(handed_in)'` once: a handed-in account is lent until it lapses,
the child hears it expiring once, and then sign in again; under 1 minute
with the build warm.
Don't: read a file (the layer reads it, jig 20 parses it); refresh a
handed-in account.
Size: at most 100 lines of production code.
Removes: nothing.

### 21.5 Failures in their classes, through the driver

Scope: the world's stories for the classes the child acts on: a rate
limit failing later with the issuer's delay; an issuer rejecting the
refresh token (`invalid_grant`) failing as sign in again; the
deployment's client rejected failing as misconfigured; an account's
record file damaged failing as unloaded, with where and why; a close
during an exchange draining it (ruling 171) (llm.md, 5.2 and 5.4).
Needs: nothing beyond 21.4.
Files: `tests/protocol-accounts/{src/fixtures.rs, src/world.rs,
tests/failures.rs}`.
Shape: test code only: the issuer's plans (`skein_fake_oauth`'s
`Plan`, as skein's world queues them, `tests/oauth-accounts/src/world.rs:840-868`)
and a damaged record laid on the simulated disk.
Test shape: `tests/protocol-accounts/tests/failures.rs` (new):
`a_rate_limited_refresh_fails_later_with_the_issuers_delay`,
`a_refused_refresh_token_fails_as_sign_in_again`,
`a_rejected_client_fails_as_misconfigured`,
`a_damaged_record_file_fails_as_unloaded_naming_the_file`,
`a_close_during_an_exchange_drains_it`; each written as 21.3's
reference test, its `Heard` list a literal. Each under 0.5 s.
Proof: world, `heavy cargo nextest run -p jig-protocol-accounts-world -E
'test(failures)'` once: each failure the fake issuer or the disk makes
reaches the child in the class llm.md, 5.2 gives it; under 1 minute with
the build warm.
Don't: add a fuzzy test (Choices).
Size: at most 20 lines of production code (test code excepted).
Removes: nothing.

### 21.s1 The table holds a grant's value in its parts

A step not in the plan (decisions.md, ruling 163): 21.2 landed (jig
`530b795a`) on ruling 160's envelope before decision 37 reversed it.
Scope: llm.md, 5.3: "The table holds the two apart, both secret".
Needs: 21.2.
Files: as 21.2's.
Shape: as 21.2's Shape and sketch now say: `Table::granted(account,
  generation, value: Value, expires)`, `Table::value(..) ->
  Option<&Value>`, `Value { token: Secret, account_id: Secret }`, the id
  empty where its provider names none and bounded by `ACCOUNT_ID_BYTES`
  (256); ruling 160's envelope and its bound removed.
Test shape: 21.2's tests on `Value`, through `fixtures::value(token)`;
  adds `an_account_id_past_its_bound_is_refused`.
Proof: as 21.2's, run once.
Don't: build smith's envelope anywhere in the engine.
Size: at most 80 lines of production code.
Removes: the envelope and its bound.

## Interface sketches

```rust
// jig_protocol_accounts::translate: the child's vocabulary and skein's driver's (llm.md, 5.2)
use jig_domain_accounts as accounts;
use skein_lib::{Secret, Time};
use skein_oauth_accounts as driver;

/// What the child asks of its component, as the driver hears it.
pub fn down(request: accounts::ToComponent) -> driver::Request { todo!() }

/// What the driver says, as the child hears it; `now` is the engine's clock at this step.
pub fn up(event: driver::Event, now: Time) -> Up { todo!() }

/// A driver's event, translated.
#[derive(PartialEq, Eq, Debug)]
pub enum Up {
    /// A generation lent: the child hears its name; the routing puts the value
    /// in the table in the same step, expiring at `expires`.
    Granted { event: accounts::FromComponent, account: u32, generation: u64, value: Secret, expires: Time },
    /// Every other event the engine meets: failed in a class, expiring, closed.
    Child(accounts::FromComponent),
}
```

```rust
// jig_protocol_accounts::table: the access tokens lent (llm.md, 5.3)
use skein_lib::{Secret, Time};

/// The most bytes of a provider's account id (jig 20's `account_id` bound).
pub const ACCOUNT_ID_BYTES: u32 = 256;

/// A grant's value in its two parts, as skein's LLM client takes them (llm.md, 5.3):
/// put in an agent's form only where it is handed to that agent, never here.
#[derive(Clone, Debug)]
pub struct Value {
    pub token: Secret,
    /// The provider's account id; empty where its provider names none.
    pub account_id: Secret,
}

/// The access tokens lent, by account: the two newest generations of each, with their expiry.
/// Read by the protocol layer's routing alone; its Debug shows counts and lengths.
#[derive(Debug)]
pub struct Table;

/// A value the table cannot hold: past its token or id bound, or for an account past its count.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Unfit;

impl Table {
    pub fn new(accounts: u32, token_bytes: u32) -> Table { todo!() }
    pub fn granted(&mut self, account: u32, generation: u64, value: Value, expires: Time) -> Result<(), Unfit> { todo!() }
    /// A generation still held and unexpired.
    pub fn value(&self, account: u32, generation: u64, now: Time) -> Option<&Value> { todo!() }
    pub fn expire(&mut self, now: Time) { todo!() }
    pub fn next_expiry(&self) -> Option<Time> { todo!() }
}

/// The table's heap at its limits, or None if unusable.
pub fn worst_case(accounts: u32, token_bytes: u32) -> Option<u64> { todo!() }
```

```rust
// jig_protocol_accounts::handed_in (llm.md, 5.4)
use alloc::boxed::Box;
use skein_lib::{Secret, Wall};
use skein_oauth_accounts as driver;

/// A handed-in record the driver would refuse, with the bound it passes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Unfit;

/// The `HandIn` request for `account`'s access-only record, checked against the driver's limits.
/// The provider's account id, where it needs one, becomes the record's metadata.
pub fn hand_in(
    account: u32,
    token: Secret,
    lapses: Wall,
    account_id: Option<Box<[u8]>>,
    limits: &skein_oauth::Limits,
) -> Result<driver::Request, Unfit> {
    todo!()
}
```

- **Who owns what:** the driver owns records, exchanges, refreshes and
  generations; the child owns which accounts are usable and who holds
  which generation; this crate owns the translation and the table; the
  protocol layer's routing (jig 25) owns the table instance, reads it for
  grants' values (jig 23's `Down`) and calls `expire` at
  `next_expiry`. No root holds a `Table`, a value or a reference to one.
- **What ends what:** the driver's `Closed` ends the component, and the
  child hears `Closed`; the table has no terminal and empties as its
  entries expire.

## Questions to raise when reached

- **The handed-in file's form** (llm.md, 5.4 says what it holds, an
  access token and when it lapses, but configuration.md, 10, which lists
  the forms an operator writes, has none): jig 20 parses it in the form
  the designer gives; 21.4 takes the parsed token, lapse and account id,
  and does not wait for it.
- **The provider's account id beside the access token.** smith's
  credential for the ChatGPT provider carries `account_id` beside the
  token (smith's `crates/smith-protocol-llm/src/framing.rs:47-51`), the
  driver lends `access_token` alone (`component.rs:1084-1086`) and keeps
  the record's metadata. Settled by the designer (jig `87195d80`;
  llm.md, 5.3): the table's `Value` keeps the token and the account's id
  apart, and the worker puts them in smith's form where it hands them to
  smith's host half (jig 25.5).
- **The child's backoff** for a failure with no delay, and its retry
  interval for sign in again and unloaded, are the child's limits (jig
  08), numbers of jig 20's profiles, not this crate's.

## Done when

1. The translation and table pass: `heavy cargo nextest run -p
   jig-protocol-accounts` prints no failure.
2. The world's stories pass: `heavy cargo nextest run -p
   jig-protocol-accounts-world` prints no failure.
3. No wildcard arm in the translation: `git grep -n -E '^\s*_ =>'
   crates/jig-protocol-accounts/src/translate.rs` prints nothing.
4. The crate links no smith and no core: `cargo tree -p
   jig-protocol-accounts -e normal --prefix none | grep -E
   '^(smith|jig-domain-core|jig-domain-host)'` prints nothing.
5. The gate passed on this session's last landing: `git -C
   ~/src/rust/jig log --format='%(trailers:key=Gate-Result,valueonly)'
   --grep='^Step:.jig-21\.' origin/main | head -1` names
   `scripts/check.sh passed`.
