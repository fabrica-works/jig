# Session 20: `jig-protocol-config`, jig's part of a configuration

A new step crate turns jig's member of an application's configuration,
read with skein's JSON reader, into jig's typed values, or into the
problems it found, each naming its path in the document: an engine's
part (the deployment, its owners, providers, rules, projects, charters,
LLM endpoints and accounts, workers, clients and connectors) and a
worker's part, with the profiles that give every limit jig's children and
the channel take, overrides by name, and the check across the whole. It
reads no file: it names the files the application's startup reads, with
skein's `read_private_file` for secrets, and parses the forms of their
bytes. It links neither smith nor skein's LLM crates, so an application
without agents never links them (jig's `protocol/configuration.md`,
section 2). Nothing in jig parses configuration today; the reference
service's startup (jig 25) and its binary (jig 26) are its first users.

## Read first

- jig's `protocol/configuration.md`, whole; `protocol/llm.md`, 4 and
  5.4; `protocol/channel.md`, 4.1, 9, 10.1, 11 and 13 ("Configuration
  owes"); `protocol/client.md`, sections 4 and 8; `domain/engine.md`,
  section 14; `domain/hosts.md`, 6.6; `domain/root.md`, sections 6 and
  10; `domain/authority.md`, section 7.
- skein's `json.md`, sections 3 and 5.2; skein's `io.md`, section 4
  (`Addr`, `Name`); skein's `shell.md`, section 6 and 6.1; skein's
  `lib.md`, section 6 (`Secret`); programming-model.md, 6.3 and
  section 10.
- `decisions.md`, 1, 4 and 5; `code-organization.md`, sections 2 to 7.
- `findings/protocol.md`, 4.6 (the startup kit), 5.2 (the lint regimes)
  and 6.2 ("20-config"); `findings/skein.md`, sections 1 and 2.
- The code: `crates/jig-domain-core/src/lib.rs` (`Config`, `Settings`,
  as jig 07 leaves them with its charter table),
  `crates/jig-domain-core/src/routing/vocabulary.rs` (`Limits`, with its
  children's), `crates/jig-domain-core/src/memory.rs` (`worst_case`),
  each child's `limits.rs` and `worst_case`,
  `crates/jig-domain-host/src/limits.rs` (`Limits`, `worst_case`);
  skein's `crates/skein-json/src/document.rs` (`Document`, `Compact`,
  `Kind`) and `crates/skein-json/src/tokenizer.rs:81-115`; ops's
  `examples/ops/domain/src/translate.rs:302-340` (skein's tokenizer
  pumped over whole bytes); smith's
  `crates/smith-agent-shell/src/config.rs:4` ("Durations in JSON are
  milliseconds"), `:41-170` (its document: profile, endpoints, models)
  and `:1021` (`memory_bytes`, `grace_ms`).

## What the design asks

- **Read once, strict** (configuration.md, 1): "Unknown fields, missing
  fields, values past their bounds and duplicate names refuse the start,
  saying which".
- **Typed, then checked whole** (1): "names unique, numbers that refer to
  something configured, limits whose worst case fits the process's
  memory. A configuration that fails it never starts the loop."
- **No secret in the file** (1, 7): "A secret is a file of its own, which
  the configuration names, read at startup as a private file"; "a file
  readable by anyone but the process's user, or reached through a link,
  refuses the start, naming it."
- **`jig-protocol-config`** (2): "functions from a parsed JSON value to
  jig's typed configuration and back to the problems it found, each
  naming its path in the document. It reads no file. It yields jig's own
  plain values only, and links neither smith nor skein's LLM crates".
- **The engine's part** (3): deployment, owners, providers, rules,
  projects, charters, hosts, llm, clients, connectors, limits; **a
  worker's** (4): engine, name and secret, slots, graces, agent, limits.
- **Limits** (6): "Policy above, numbers below ... the numbers are in
  `jig-protocol-config`'s profiles, as module documentation and
  constants"; "A deployment names a profile and overrides any limit by
  its name"; "The root's worst case ..., the components' and the
  journal's, summed, must fit the memory the configuration states ... A
  profile whose sum does not fit refuses the start, saying by how much."
- **Charters' budgets** (6): "each charter's turns and time are above its
  reserve's, and its default budget holds its reserve and one of its
  main model's most costly completions ..., or the engine's start is
  refused, naming the charter."
- **The deployment's message bound** (6): it "holds every kind of
  message a task's inbox admits, as a run renders it with its sender's
  label"; agents' bounds are "the fleet's to check at its hello, never
  configuration's".
- **Formats an operator writes** (10): an address, one of `{"socket":
  "127.0.0.1:7100"}`, `{"host": "issuer.example", "port": 443}` or
  `{"in_process": "<name>"}`; "A digest file holds one SHA-256 digest
  as 64 lowercase hexadecimal digits, and nothing else but a final
  newline"; "A secret file holds the secret's bytes as they are, and
  nothing else but a final newline, which is not part of the secret."
- **The stop bound** (channel.md, 10.1): "its ping interval, its silence
  deadline, its grace, then the longer of its agents' stop graces and a
  delivery's deadline, then a save."

## Choices this plan makes

- **The document is skein-json's `Document`,** made by pumping skein's
  tokenizer over the whole bytes in one call (`document`), as ops's
  `translate::json::parse` does (`examples/ops/domain/src/translate.rs:302-340`);
  parsing then walks it by index from jig's member's value.
- **Problems are collected, bounded:** up to `PROBLEMS` (32), with a
  count of those past it, so one start says every mistake it can.
- **Paths** read as an operator wrote them: `hosts.workers[2].digest`.
- **Numbers are JSON integers; a duration is milliseconds, its member
  named with `_ms`; a byte count is named with `_bytes`,** as smith's
  agent configuration does (`crates/smith-agent-shell/src/config.rs:4`,
  `:1021`), so one person writes both alike.
- **Socket addresses** are parsed with `core::str::from_utf8` and
  `str::parse` under one `#[expect(clippy::disallowed_methods, reason =
  "configuration is the protocol layer's parsing (configuration.md,
  10)")]`, in `src/address.rs` alone: `SocketAddr::parse_ascii` is not
  stable on 1.95, and the lint's own note says "the protocol layer
  parses".
- **Plaintext only to loopback or in process:** TLS's server side is not
  built (decisions.md, 1; skein's `tls.md`, section 8), so a workers' or
  clients' listener must be a loopback socket address or an in-process
  name, and a worker's engine likewise; the check refuses another,
  saying so. Certificate and key members come with TLS's server side
  (`later.md`).
- **Nothing for what this plan does not build:** no `engine` member under
  `hosts` (agents in the engine, decisions.md, 1), no retention horizons
  under `deployment` (decisions.md, 13); a document naming one is
  refused as unknown, so nothing configured is silently ignored.
- **Files are named, never read:** `files` lists each file the startup
  reads, with its maximum, whether it is private, and its path in the
  document; the startup (jig 25, jig 26) reads each, private ones with
  skein's `read_private_file`, and hands the bytes to `digest`, `secret`
  or `handed_in`. A refused read becomes a `Problem` at that path
  (`What::Unreadable`), the shell's refusal beside it.
- **The channel's numbers are a plain struct of the profile's,** `Link`
  (channel.md, 11's component limits); the startup builds each channel
  component's `Limits` from it and jig 22's `Sizes` (jig 25).

## Rules that bind it

- "No increment of this plan ... adds a type beside an existing one for
  the same thing: it changes the type in place, and its users in the
  same increment" (code-organization.md, section 6): the profiles hold
  the domain's own `Limits` types, never copies; a later session that
  changes a child's or the hub's `Limits` changes `src/profile.rs`'s
  constants in the same increment.
- "**A file:** at most 800 lines, generated code excepted. A file of at
  most 800 lines that the increment grows past 800 is split in the same
  increment."
  (code-organization.md, section 7): `src/profile.rs` holds one
  profile's constants per file (`src/profile/small.rs`,
  `src/profile/standard.rs`).
- "**`jig-protocol-*` crates** depend on no application's crate"
  (code-organization.md, section 3): this crate links skein-lib,
  skein-json, skein-io's vocabulary, `jig-domain-core` with its
  children, and `jig-domain-host`; never smith, skein's LLM crates or
  skein-oauth.

## Increments

### 20.1 The crate, the document, the reader and the operator's forms

Scope: the crate `crates/jig-protocol-config` (parsing, step code);
`document`; `Path`, `Problem`, `What`, `Problems`; the reader's helpers
(an object's members, each known once and none unknown; integers within
bounds; text within bounds; booleans; `_ms` durations; lists within a
count); `Address` and its three forms; the digest and secret files'
forms (configuration.md, 1 and 10).
Needs: nothing beyond the session's start (jig 02's skein: `Addr`,
`Name` and `Secret`).
Files: `crates/jig-protocol-config/{Cargo.toml, src/lib.rs,
src/document.rs, src/problem.rs, src/read.rs, src/address.rs,
src/forms.rs, src/tests.rs, src/tests/fixtures.rs, src/tests/read.rs,
src/tests/forms.rs}`; the workspace `Cargo.toml` (members; the crate,
`skein-io`); `tests/boundary/src/` (the crate's role: skein-lib,
skein-json, skein-io, `jig-domain-*`; never smith, `skein-llm*` or
skein-oauth). Shared hotspots: the workspace `Cargo.toml`,
`tests/boundary/src/`.
Shape: as the interface sketch (`document`, `Problem`, `What`, `Path`,
`Problems`, `Address`, `digest`, `secret`); `read.rs`'s `Object` walks
one object's members by index, records each member it reads, and at its
end reports every member it did not as `What::Unknown` and every name met
twice as `What::Duplicate`.
Test shape:
- `src/tests/fixtures.rs` (new; the crate's fixtures module, grown by
  each increment): `document(bytes) -> Document`, `path(&[segment])`,
  the digest `DIGEST_EMPTY` (`b"e3b0c442...b855\n"`, the digest of no
  bytes) and the secret `SECRET_A` (`b"secret-a\n"`).
- `src/tests/forms.rs` (new), reference test:

  ```rust
  #[test]
  fn a_digest_file_with_uppercase_digits_is_refused() {
      let bytes = b"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855\n";
      assert_eq!(forms::digest(bytes), Err(What::Digest));
  }
  ```

  Others: `a_digest_file_without_its_final_newline_is_read`,
  `a_digest_of_63_digits_is_refused`,
  `a_digest_followed_by_more_than_a_newline_is_refused`,
  `a_secret_files_final_newline_is_dropped`,
  `a_secret_keeps_an_inner_newline`.
- `src/tests/read.rs` (new): `an_unknown_member_is_refused_naming_its_path`,
  `a_member_named_twice_is_refused`, `a_missing_member_is_refused`,
  `an_integer_past_its_bound_is_refused`,
  `a_duration_reads_as_milliseconds`,
  `an_address_with_two_forms_is_refused`,
  `an_in_process_name_past_its_bound_is_refused`,
  `an_ipv6_socket_address_is_read`,
  `problems_past_the_bound_are_counted`. Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p jig-protocol-config` once: an
operator's mistakes in a document are each named by their path, and the
digest and secret files' forms read as configuration.md, 10 says; under
10 s with the build warm. Not higher: no service reads configuration
until jig 25.
Don't: read a file; call `str::parse` or `from_utf8` outside
`src/address.rs`.
Size: at most 400 lines of production code.
Removes: nothing.

### 20.2 Profiles and the memory check

Scope: `Profile::{Small, Standard}`; `Limits` (the core's with its
children, the hub's, the channel's `Link`, the accounts' table's
largest token); `profile(Profile) -> Limits`, each number in module
documentation; `fits`, the sum of the parts' worst cases within the
stated memory, or by how much not (configuration.md, 6;
programming-model.md, 6.3).
Needs: nothing beyond 20.1.
Files: `crates/jig-protocol-config/{src/profile.rs,
src/profile/small.rs, src/profile/standard.rs, src/fits.rs, src/lib.rs,
src/tests/profile.rs}`.
Shape: as the interface sketch (`Profile`, `Limits`, `Link`, `profile`,
`Share`, `fits`). `Small` is one person's machine (the one-process
shape: 2 workers, 2 slots each, 4 LLM accounts); `Standard` a team's;
each limit's number is the session's choice, consistent with the others
(every child's `worst_case` and the core's answers `Some`).
Test shape:
- `src/tests/profile.rs` (new), reference test:

  ```rust
  #[test]
  fn a_stated_memory_below_the_sum_is_refused_by_how_much() {
      let shares = [Share { name: b"core", bytes: 600 }, Share { name: b"hub", bytes: 500 }];
      assert_eq!(fits(1_000, &shares), Err(What::Memory { over: 100 }));
  }
  ```

  Others: `the_small_profile_fits_every_childs_worst_case`,
  `the_standard_profile_fits_every_childs_worst_case`,
  `the_small_profile_fits_the_cores_worst_case`,
  `the_small_profiles_hub_fits_its_worst_case`,
  `a_sum_past_a_u64_is_refused`, `a_sum_within_memory_is_its_total`.
Proof: unit, `heavy cargo nextest run -p jig-protocol-config -E
'test(profile)'` once: both profiles are accepted by every child's and
the core's worst case, and a memory below a sum is refused by how much;
under 10 s with the build warm.
Don't: put a profile's number in a design document; compute a worst
case here (each owner's `worst_case` does).
Size: at most 450 lines of production code (the two profiles' constants
included).
Removes: nothing.

### 20.3 The `limits` member: a profile and overrides by name

Scope: the `limits` member: `{"profile": "small", "overrides": {"<owner>.<limit>":
<integer>, ...}}`, each limit named by its owner and field
(`tasks.inbox_messages`, `hub.slots`, `link.ping_ms`); an unknown name,
a value past its type, and a profile then refused by its owner's
`worst_case` each refused, naming the override (configuration.md, 6).
Needs: nothing beyond 20.2.
Files: `crates/jig-protocol-config/{src/overrides.rs, src/lib.rs,
src/tests/overrides.rs}`.
Shape: `overrides::apply(limits: &mut Limits, name: &[u8], value: u64)
-> Result<(), What>`, one `match` per owner on its fields' names, with
no wildcard over a field: a field added to a child's `Limits` breaks
the build until it has a name here.
Test shape: `src/tests/overrides.rs` (new):
`an_override_replaces_one_limit_of_the_profile`,
`an_override_of_an_unknown_name_is_refused`,
`an_override_past_its_type_is_refused`,
`an_override_that_breaks_a_worst_case_is_refused_naming_it`,
`every_limit_of_the_small_profile_has_a_name` (overrides each field, by
a list of names written out in the test, with its own value, and expects
no problem). Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p jig-protocol-config -E
'test(overrides)'` once: any limit can be overridden by its name, and an
override that breaks a worst case is refused naming it; under 10 s with
the build warm.
Compound limits (ruling 159): a limit whose value is a record
(`tasks.retries`, `documents.versions`, `core.brief_core_budgets`) is
overridden member by member, each scalar member named by its dotted path
(`tasks.retries.transient.base_ms`, `documents.versions.minimum`), and
the test's list names each.
Don't: accept an override by position, or of a whole compound limit at
once.
Size: at most 600 lines of production code (ruling 161).
Removes: nothing.

### 20.4 The engine's protocol part

Scope: the `llm` member: endpoints (as smith's agent configuration's,
`crates/smith-agent-shell/src/config.rs:143-170`: name, number, dialect,
account, provider, address, transport, server name, trust, headers,
connect and handshake deadlines), models (name, endpoint, window,
output, prices: input, cached, output, cache writes when priced apart,
the unit), accounts (each signed in, with its client, its client
secret's file if confidential, its authorization and token URLs, its
token endpoint's address and transport, its redirect and scope; or
handed in, with its private file), and the private directory of their
records; the `hosts` member: the workers, each a name and its digest's
file, and the workers' listener's addresses; the `clients` member: its
listener, the heartbeat, whether to print a local sign-in link. The
check: names and numbers unique; a model's endpoint, an endpoint's
account configured; plaintext only to loopback or in process. `files`
for these members' files; the handed-in file's form (configuration.md,
3, 7 and 10; llm.md, 4 and 5.4; client.md, sections 4 and 8).
Needs: nothing beyond 20.3; a handed-in account's file takes decision
25's form.
Files: `crates/jig-protocol-config/{src/llm.rs, src/hosts.rs,
src/clients.rs, src/files.rs, src/forms.rs, src/check.rs, src/lib.rs,
src/tests/llm.rs, src/tests/hosts.rs, src/tests/fixtures.rs}`.
Shape: as the interface sketch (`Llm`, `Endpoint`, `Transport`,
`Model`, `Prices`, `Account`, `Source`, `SignIn`, `Hosts`, `WorkerName`,
`Clients`, `File`, `files`, `HandedIn`, `handed_in`). Models are the
core's `Model` (`crates/jig-domain-core/src/run.rs`, `Model`) where the
core's charter carries them; endpoints and accounts are this crate's
plain values, which the startup turns into smith's, skein's and the
driver's (llm.md, 4: "One list, three views").
Test shape:
- `src/tests/fixtures.rs` gains `ENGINE_SMALL`, a whole engine document
  as a byte-string literal (one in-process workers' listener, the
  workers `worker-a` and `worker-b`, one endpoint `fake`, the model
  `small`, the account `chat` handed in), `engine(bytes) ->
  Result<Engine, Problems>`, and `with(document, from, to) ->
  Box<[u8]>`, the document with the first `from` bytes replaced by `to`,
  found with `skein_lib::bytes::find` and written with
  `skein_lib::Writer` (no `Vec` in a step crate's tests); each member
  `ENGINE_SMALL` lets a test change is written once in it, on a line of
  its own.
- `src/tests/llm.rs` (new), reference test:

  ```rust
  #[test]
  fn a_model_naming_an_unknown_endpoint_is_refused() {
      let document = fixtures::with(fixtures::ENGINE_SMALL, br#""endpoint": "fake""#, br#""endpoint": "nowhere""#);
      let problems = fixtures::engine(&document).expect_err("an unknown endpoint");
      assert_eq!(
          problems.problems.get(0),
          Some(&Problem { path: fixtures::path(&["llm", "models", "0", "endpoint"]), what: What::Refers })
      );
  }
  ```

  Others: `an_endpoint_naming_an_unknown_account_is_refused`,
  `two_endpoints_with_one_name_are_refused`,
  `plaintext_to_a_public_address_is_refused`,
  `a_handed_in_account_names_its_private_file`,
  `a_cache_write_price_may_be_left_out`.
- `src/tests/hosts.rs` (new):
  `a_workers_listener_on_a_public_address_is_refused`,
  `a_worker_named_twice_is_refused`,
  `an_engine_member_under_hosts_is_refused_as_unknown`,
  `files_list_each_digest_as_private`,
  `a_handed_in_file_reads_its_token_and_lapse` (from the designer's
  form). Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p jig-protocol-config -E
'test(llm) | test(hosts)'` once: an engine's endpoints, models,
accounts, workers and listeners parse, every reference is checked, and
every secret file is named private; under 10 s with the build warm.
Don't: link skein-oauth or an LLM crate to type these values; read a
file.
Size: at most 800 lines of production code (ruling 164).
Removes: nothing.

### 20.5 The engine's domain part and the whole check

Scope: the `deployment` (its directory; the seed's source, for tests),
`owners` (provider and subject, each an owner of every seeded project),
`providers` (number and kind, exactly one of kind `deployment`), `rules`,
`projects` (number, policy with its spend per period and its roles'
authority, allotment and proposal kinds, parties' roles; no resources
or homes, which come by adoption), `charters` (each numbered: its
instructions' file, its models named from the `llm` member's, its tool
families, the hosts that may run it, its waiting time, whether its runs
resume, its default budget with its reserve), which charter chats run
under with a closed chat's grace, and `connectors` (none the core's
reserved number) members (configuration.md, 3, as the designer's answer
of 2026-10-12 amends it, jig `2f7b8800`);
`core`, the core's `Config` from them and what the shell drew; the
check: a chats' charter offered; one account per charter, shared by its
models' endpoints; each charter's turns and time above its reserve's,
and its default budget holding its reserve and one of its main model's
most costly completions, naming the charter; the deployment's message
bound; the memory `fits` with the parts the caller names
(configuration.md, 3, 6 and 8).
Needs: 20.4; and the core's step that takes a project's period and
budget from the root, a pool from its role's allotment, a chat's
authority from its person's role, and keeps the recurring procedure's
number and the engine's tool families' bits as its own constants
(the designer's answer of 2026-10-12, a step the supervisor names), so
that `Settings` keeps no `period`, `period_budget`, `person_budget`,
`chat_authority` or `recurring_connector` and `tools` is the core's
constant.
Files: `crates/jig-protocol-config/{src/domain.rs, src/charters.rs,
src/projects.rs, src/core.rs, src/check.rs, src/lib.rs,
src/tests/domain.rs, src/tests/charters.rs, src/tests/fixtures.rs}`.
Shape: as the interface sketch (`Engine`, `Deployment`, `engine`,
`core`); the domain's values in the domain's own types, as
`jig_domain_core::Config` takes them (owners as `people::InitialOwner`,
rules and policies as the authority child's `Rules` and `Policy`, each
charter as jig 07's `jig_domain_core::Charter { number, policy, hosts,
spend }`, chats' charter as `Settings::chat`). `Settings`'
`deployment_provider` is the provider of kind `deployment`;
`close_grace` the chats' charter's grace; `resume_bytes` and
`Config::hub_turns` the profile's limits; `owners` one
`people::InitialOwner` per owner and seeded project. The check refuses
a second provider of kind `deployment`, or none; a connector numbered
as the core's reserved number. The budget's check is
jig 07's: a charter's `spend` at least `jig_domain_core::reserve_floor(&policy)`,
and its policy's turns and time above its reserve's. `core` sets jig
09's `Config::workers_in_process` to whether the workers' listener is an
in-process name; if jig 09 lands after this increment, its increment
that adds the field sets it here (code-organization.md, section 6: a
type changes "and its users in the same increment"). The message bound's check reads it as the code names it when
the increment starts (today `jig_domain_tasks::Limits::message_bytes`,
beside `jig_domain_people::Limits::words`), and checks each kind
`domain/tasks.md`, 8.2 lists by the limit that bounds it.
Test shape:
- `src/tests/charters.rs` (new), reference test:

  ```rust
  #[test]
  fn a_charter_whose_budget_cannot_hold_its_reserve_is_refused() {
      // The model `small`: window 8192, output 1024, every price 1 per 1000 tokens:
      // its most costly completion is 10; a reserve of one turn needs 20.
      let document = fixtures::with(fixtures::ENGINE_SMALL, br#""spend": 20"#, br#""spend": 19"#);
      let problems = fixtures::engine(&document).expect_err("too small a budget");
      assert_eq!(problems.problems.get(0).expect("one problem").what, What::Budget { charter: 1 });
  }
  ```

  Others: `a_charter_whose_budget_holds_its_reserve_is_read`,
  `a_charter_turns_not_above_its_reserve_are_refused`,
  `a_charter_whose_models_name_two_accounts_is_refused`,
  `a_chats_charter_not_offered_is_refused`,
  `a_charter_naming_an_unknown_model_is_refused`.
- `src/tests/domain.rs` (new): `the_small_engine_document_parses_whole`,
  `a_project_numbered_twice_is_refused`,
  `an_owner_naming_an_unknown_provider_is_refused`,
  `a_message_bound_below_a_partys_words_is_refused`,
  `the_cores_config_takes_the_charters_and_projects` (checks
  `core(..)`'s charter count and project list against literals),
  `a_retention_member_is_refused_as_unknown`,
  `a_projects_resources_member_is_refused_as_unknown`,
  `two_deployment_providers_are_refused`,
  `a_connector_numbered_as_the_cores_own_is_refused`,
  `each_owner_owns_every_seeded_project`. Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p jig-protocol-config` once: a
whole engine document parses into the core's configuration, and each
charter whose budget cannot hold its reserve and one most costly
completion is refused, naming it; under 10 s with the build warm.
Don't: judge the configuration against the store (configuration.md, 9
keeps removed parts open); check an agent's bounds (the fleet's, at its
hello).
Size: at most 1,200 lines of production code (ruling 188).
Removes: nothing.

### 20.6 A worker's part

Scope: the `engine` member (an address: loopback plaintext or in
process); `name` and `secret` (its file); `slots`; `graces` (the grace
after losing the engine, the agent's stop graces, a delivery's deadline,
a save's); the stop bound from them and the profile's ping interval and
silence deadline; `agent` (its program, its arguments and its
configuration's file, smith's agent configuration, read by the worker's
startup); `limits`. The check: a silence deadline of at least three ping
intervals; a stop bound that fits a duration (configuration.md, 4;
channel.md, 10.1).
Needs: nothing beyond 20.5.
Files: `crates/jig-protocol-config/{src/worker.rs, src/check.rs,
src/files.rs, src/lib.rs, src/tests/worker.rs, src/tests/fixtures.rs}`.
Shape: as the interface sketch (`Worker`, `Graces`, `Agent`, `worker`,
`stop_bound`).
Test shape:
- `src/tests/fixtures.rs` gains `WORKER_SMALL` (engine in process at
  `workers`, name `worker-a`, 2 slots, grace 30 s, agent stop 10 s,
  delivery 20 s, save 5 s).
- `src/tests/worker.rs` (new), reference test:

  ```rust
  #[test]
  fn the_stop_bound_sums_ping_silence_grace_the_longer_stop_and_a_save() {
      let graces = Graces {
          grace: Duration::from_secs(30),
          agent_stop: Duration::from_secs(10),
          delivery: Duration::from_secs(20),
          save: Duration::from_secs(5),
      };
      // ping 5 s, silence 15 s, grace 30 s, the longer of 10 s and 20 s, save 5 s.
      let bound = stop_bound(&graces, Duration::from_secs(5), Duration::from_secs(15));
      assert_eq!(bound, Some(Duration::from_secs(75)));
  }
  ```

  Others: `a_silence_under_three_pings_is_refused`,
  `a_worker_names_its_secret_file_as_private`,
  `an_engine_at_a_public_address_is_refused`,
  `the_small_worker_document_parses_whole`,
  `an_agent_without_a_program_is_refused`. Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p jig-protocol-config -E
'test(worker)'` once: a worker's document parses, its stop bound is the
sum channel.md, 10.1 names, and its secret's file is named private;
under 10 s with the build warm.
Don't: parse smith's agent configuration (smith's, read by the worker's
startup); compute the hello's agent bounds (the worker's root's, from
smith's configuration).
Size: at most 250 lines of production code.
Removes: nothing.

## Interface sketches

```rust
// jig_protocol_config::{document, problem, address, forms}
use alloc::boxed::Box;
use skein_io::Addr;
use skein_io::digest::Digest;
use skein_json::Document;
use skein_lib::{List, Secret, Wall};

/// The document's bounds: its bytes and tokens, from the application's startup.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DocumentLimits {
    pub bytes: u32,
    pub tokens: u32,
    pub depth: u32,
}

/// The whole configuration's bytes as skein-json's document, or why not, at the document's root.
pub fn document(bytes: &[u8], limits: &DocumentLimits) -> Result<Document, Problem> { todo!() }

/// One step of a path in the document: a member's name, or an index in a list.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Segment {
    Member(Box<[u8]>),
    Index(u32),
}

/// Where in the document a problem is, from the application's whole document root.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Path {
    pub segments: List<Segment>,
}

/// One problem: where, and what.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Problem {
    pub path: Path,
    pub what: What,
}

/// What is wrong at a path.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum What {
    /// Not JSON, or past the document's bounds.
    Json,
    /// A member jig does not know.
    Unknown,
    /// A member jig needs.
    Missing,
    /// A member named twice in one object.
    Duplicate,
    /// Another kind of value than the member takes.
    Kind,
    /// A value past its bound.
    Bound,
    /// An address with none or several forms, or a form that does not read.
    Form,
    /// A name or number already used.
    Unique,
    /// A name or number that refers to nothing configured.
    Refers,
    /// Plaintext to an address that is neither loopback nor in process.
    Plaintext,
    /// A digest file not in configuration.md, 10's form.
    Digest,
    /// A handed-in account's file not in its form.
    HandedIn,
    /// A charter's turns or time not above its reserve's.
    Reserve { charter: u32 },
    /// A charter's default budget cannot hold its reserve and one most costly completion.
    Budget { charter: u32 },
    /// A charter's models' endpoints name more than one account.
    Accounts { charter: u32 },
    /// The deployment's message bound below a kind of message the inbox admits.
    Message,
    /// A silence deadline under three ping intervals.
    Silence,
    /// A stop bound past a duration.
    StopBound,
    /// The stated memory below the worst cases' sum, by this much.
    Memory { over: u64 },
    /// An override of a name jig does not know, or past its type.
    Override,
    /// A file the startup could not read; the shell's refusal says why.
    Unreadable,
}

/// The problems one start found: at most `PROBLEMS`, and how many past them.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Problems {
    pub problems: List<Problem>,
    pub past: u32,
}

pub const PROBLEMS: u32 = 32;

/// configuration.md, 10's address: a socket address or an in-process name, or a host the
/// shell resolves at startup (skein's shell.md, section 6).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Address {
    Addr(Addr),
    Host { name: Box<[u8]>, port: u16 },
}

/// A digest file's digest, from its bytes.
pub fn digest(bytes: &[u8]) -> Result<Digest, What> { todo!() }
/// A secret file's secret: its bytes, one final newline dropped.
pub fn secret(bytes: Box<[u8]>) -> Secret { todo!() }

/// A handed-in account's record as its file holds it (llm.md, 5.4), in the designer's form:
/// the access token, the wall time it lapses, and the provider's account id where it needs one.
#[derive(Debug)]
pub struct HandedIn {
    pub token: Secret,
    pub lapses: Wall,
    pub account_id: Option<Box<[u8]>>,
}
pub fn handed_in(bytes: &[u8], limits: &DocumentLimits) -> Result<HandedIn, Problems> { todo!() }
```

```rust
// jig_protocol_config::{profile, fits}
use jig_domain_core as core;
use jig_domain_host as host;
use skein_lib::Duration;

/// A named set of every limit jig's children and the channel take (configuration.md, 6).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Profile {
    /// One person's machine: the one-process shape.
    Small,
    /// A team's deployment.
    Standard,
}

/// The channel components' own numbers (channel.md, 11).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Link {
    pub chunk: u32,
    pub name: u32,
    pub secret: u32,
    pub opening: u32,
    pub handshake: Duration,
    pub hello: Duration,
    pub ping: Duration,
    pub silence: Duration,
    pub stall: Duration,
    pub backoff: Duration,
    pub ceiling: Duration,
    pub proved: Duration,
    pub busy: Duration,
    pub facts: u32,
}

/// Every limit a process of jig's takes, by owner.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub core: core::Limits,
    pub hub: host::Limits,
    pub link: Link,
    /// The accounts' table's largest token (llm.md, 5.3).
    pub token: u32,
}

pub fn profile(profile: Profile) -> Limits { todo!() }

/// One owner's worst case, named for the problem that refuses a start.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Share {
    pub name: &'static [u8],
    pub bytes: u64,
}

/// The shares' sum within the stated memory, or by how much not.
pub fn fits(memory: u64, shares: &[Share]) -> Result<u64, super::What> { todo!() }
```

```rust
// jig_protocol_config::{engine, worker}: jig's part, parsed and checked
use alloc::boxed::Box;
use skein_json::Document;
use skein_lib::Duration;

/// An engine's part (configuration.md, 3), every reference checked.
#[derive(Debug)]
pub struct Engine {
    pub deployment: Deployment,
    pub owners: Box<[jig_domain_people::InitialOwner]>,
    pub providers: Box<[Provider]>,
    pub rules: jig_domain_authority::Rules,
    pub projects: Box<[Project]>,
    pub charters: Box<[jig_domain_core::Charter]>,
    pub chats: u32,
    pub llm: Llm,
    pub hosts: Hosts,
    pub clients: Clients,
    pub connectors: Box<[u16]>,
    pub limits: Limits,
    pub memory: u64,
}

/// The deployment's directory (the store, transcripts, the secrets' store) and, for tests, its seed.
#[derive(Debug)]
pub struct Deployment {
    pub directory: Box<[u8]>,
    pub seed: Option<u64>,
}

/// A sign-in provider: its number and kind; its settings are the application's.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Provider {
    pub number: u16,
    pub kind: ProviderKind,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProviderKind { Link, Application }

/// A project seeded on a first start, in the core's terms.
#[derive(Debug)]
pub struct Project {
    pub number: u32,
    pub roles: Box<[jig_domain_people::PermissionRole]>,
    pub policy: jig_domain_authority::Policy,
}

/// LLM endpoints, models and accounts (llm.md, 4 and 5).
#[derive(Debug)]
pub struct Llm {
    pub endpoints: Box<[Endpoint]>,
    pub models: Box<[jig_domain_core::Model]>,
    pub accounts: Box<[Account]>,
    /// The private directory of signed-in accounts' records.
    pub records: Box<[u8]>,
}

/// An endpoint, as smith's agent configuration names one.
#[derive(Debug)]
pub struct Endpoint {
    pub name: Box<[u8]>,
    pub number: u32,
    pub dialect: u32,
    pub account: u32,
    pub provider: Box<[u8]>,
    pub address: Address,
    pub transport: Transport,
    pub headers: Box<[(Box<[u8]>, Box<[u8]>)]>,
    pub connect: Duration,
    pub handshake: Duration,
}

/// TLS by server name and trust, or plaintext to loopback.
#[derive(Debug)]
pub enum Transport {
    Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> },
    Plaintext,
}

/// An LLM account: its number, and signed in or handed in.
#[derive(Debug)]
pub struct Account {
    pub number: u32,
    pub source: Source,
}
#[derive(Debug)]
pub enum Source {
    SignIn(SignIn),
    /// The private file of its access token and lapse.
    HandedIn { file: Box<[u8]> },
}
#[derive(Debug)]
pub struct SignIn {
    pub client: Box<[u8]>,
    pub secret: Option<Box<[u8]>>,
    pub authorization: Box<[u8]>,
    pub token: Box<[u8]>,
    pub endpoint: Address,
    pub transport: Transport,
    pub redirect: Box<[u8]>,
    pub scope: Box<[u8]>,
}

/// The workers and their listener (configuration.md, 3).
#[derive(Debug)]
pub struct Hosts {
    pub workers: Box<[WorkerName]>,
    pub listener: Box<[Address]>,
}
#[derive(Debug)]
pub struct WorkerName {
    pub name: Box<[u8]>,
    /// The private file of its secret's digest.
    pub digest: Box<[u8]>,
}

/// The clients' listener, the heartbeat, and whether to print a local sign-in link.
#[derive(Debug)]
pub struct Clients {
    pub listener: Address,
    pub heartbeat: Duration,
    pub link: bool,
}

/// A file the startup reads: where the document names it, its path, its maximum, and whether private.
#[derive(Debug)]
pub struct File {
    pub at: Path,
    pub path: Box<[u8]>,
    pub max: u32,
    pub private: bool,
}

/// A worker's part (configuration.md, 4).
#[derive(Debug)]
pub struct Worker {
    pub engine: Address,
    pub name: Box<[u8]>,
    /// The private file of its secret.
    pub secret: Box<[u8]>,
    pub slots: u32,
    pub graces: Graces,
    pub stop_bound: Duration,
    pub agent: Agent,
    pub limits: Limits,
    pub memory: u64,
}

/// The worker's graces; with the profile's ping and silence, its stop bound (channel.md, 10.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Graces {
    pub grace: Duration,
    pub agent_stop: Duration,
    pub delivery: Duration,
    pub save: Duration,
}

/// The agent program the worker spawns for each run, and smith's agent configuration's file.
#[derive(Debug)]
pub struct Agent {
    pub program: Box<[u8]>,
    pub arguments: Box<[Box<[u8]>]>,
    pub configuration: Box<[u8]>,
}

/// jig's member of an engine's document, at the token index of its value.
pub fn engine(document: &Document, at: u32) -> Result<Engine, Problems> { todo!() }
/// jig's member of a worker's document.
pub fn worker(document: &Document, at: u32) -> Result<Worker, Problems> { todo!() }
/// The core's configuration, from the engine's part and what the shell drew.
pub fn core(engine: &Engine, deployment: [u8; 16], seed: u64) -> jig_domain_core::Config { todo!() }
/// Every file an engine's part names, for the startup to read.
pub fn files(engine: &Engine) -> Box<[File]> { todo!() }
/// The stop bound a worker states at its hello, or None past a duration.
pub fn stop_bound(graces: &Graces, ping: Duration, silence: Duration) -> Option<Duration> { todo!() }
```

- **Who owns what:** the application owns its document: it finds jig's
  member and its own, parses each, hands each child and component its
  part, reads the files `files` names (private ones with skein's
  `read_private_file`), and hands their bytes to `digest`, `secret` and
  `handed_in`; this crate owns the forms, the profiles and the checks.
  The domain types (`InitialOwner`, `PermissionRole`, `Rules`, `Policy`,
  `Charter`, `Model`) are written with the names jig 07 and the children
  have when the session starts; where they differ, those names are used.
- **What ends what:** nothing: parsing is a function of the bytes; a
  start refused for a problem never makes the loop.

## Questions to raise when reached

- **A handed-in account's file** (llm.md, 5.4: "an access token and the
  wall time it lapses, in a private file"): configuration.md, 10 gives
  no form for it. The plan's proposal for the designer: a JSON object
  `{"access_token": "<text>", "expires_at": <seconds since the Unix
  epoch>}`, with an optional `"account_id": "<text>"` for a provider
  that needs one (21's Questions), read strictly as the configuration
  is. Decision 25 adopts this form; nothing waits for it.
- **Profiles' numbers** are the session's (configuration.md, 6 and 9):
  report them, with the sums `fits` gives for each.
- **The store's, the clients' and the accounts driver's numbers** are
  not in `Limits`: the components that take them are built by jig 13 to
  17, jig 31 and skein; the reference service's startup (jig 25) needs
  them from somewhere. Raise it if jig 25 starts before a session adds
  them to the profiles.

## Done when

1. The crate's tests pass: `heavy cargo nextest run -p
   jig-protocol-config` prints no failure.
2. This crate links no smith, LLM or OAuth crate: `cargo tree -p jig-protocol-config -e normal
   --prefix none | grep -E '^(smith|skein-llm|skein-oauth)'` prints
   nothing.
3. Disallowed parsing stays in one file: `git grep -n
   'disallowed_methods' crates/jig-protocol-config/src` prints only
   `src/address.rs` lines.
4. The gate passed on this session's last landing: `git -C
   ~/src/rust/jig log --format='%(trailers:key=Gate-Result,valueonly)'
   --grep='^Step:.jig-20\.' origin/main | head -1` names
   `scripts/check.sh passed`.
