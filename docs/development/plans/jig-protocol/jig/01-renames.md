# Session 01: crates renamed by layer

jig's crates take the names jig's `docs/design/README.md`, section 15
gives them, so a crate's name says its layer: `jig-domain-*` for a
domain's step code, `jig-protocol-*` for the protocol layer's,
`jig-view` for the client's view, and `ops`'s crates the same under its
own name. Packages, directories, workspace entries, dependency keys,
Rust paths, include paths and `tests/boundary`'s rules move; behaviour
does not. Then `tests/boundary` gains code-organization.md, section 3's
rules for the crates this plan adds. It runs alone, first among jig's
sessions, since each later one works in the renamed crates. Four
increments, each landing with the gate green: the domain's crates; the
client's and the protocol's; `ops`'s; the boundary's rules.

## Read first

- jig's `docs/design/README.md`, section 15 (the names, the rule behind
  them, and "Every other crate keeps its name").
- jig's `AGENTS.md` and `docs/development/workflow.md`, sections 1 and 2.
- This plan's `code-organization.md`, sections 1 (the table) and 3 (who
  links what).
- `findings/protocol.md`, 5.1 (the workspace manifest), 5.3 (what
  `tests/boundary` checks, by line) and 6.1's "01-renames" (19 renames,
  `jig-inline-agent` gone).
- The code at the session's start: the workspace `Cargo.toml`, each
  crate's `Cargo.toml`, `tests/boundary/src/tests.rs`.

## What the design asks

- "**Named by layer.** A crate's name says the layer it is in": "a
  domain's step code: the core and its children, the hub, the client's
  domain" are `jig-domain-<name>`; "the protocol layer's: codecs,
  components, translations, parsing" are `jig-protocol-<name>`;
  `jig-view` is the client's view; "test kits keep their kind as their
  prefix ..., and worlds end in `-world`" (README, 15).
- "**An application follows the same rule** for its own crates, under
  its own name: `jig-ops-domain-*`, `jig-ops-protocol-*`, `jig-ops-view`"
  (README, 15).
- The table of README, 15 (code-organization.md, section 1, repeats
  it), and: "Every other crate keeps its name: the test kits, the
  worlds, `ops`'s domains, `jig-test-domain`, the reference root's test
  application, and `jig-boundary`, the check of the crate graph."
- "The design's documents use the names as built until the rename
  lands" (README, 15): the designer renames them after this session.
- code-organization.md, section 3's rules, which `tests/boundary`
  checks.

## Choices this plan makes

- **Directories follow the packages:** `crates/jig-domain-core`,
  `crates/jig-domain-<child>`, `crates/jig-domain-host`,
  `crates/jig-domain-client`, `crates/jig-view`,
  `crates/jig-protocol-charter`, `crates/jig-protocol-transcript`,
  `testing/jig-test-domain-client`, `testing/jig-test-view`; `ops`'s
  under `examples/ops/` without the `jig-ops-` prefix, as
  `examples/ops/domain-observability` is: `examples/ops/domain-client`,
  `examples/ops/view`, `examples/ops/domain-connectors`. Moved with `git
  mv`.
- **Rust paths follow:** `jig_core` → `jig_domain_core`, `jig_core_fleet`
  → `jig_domain_fleet`, `jig_host` → `jig_domain_host`, `jig_client` →
  `jig_domain_client`, `jig_client_view` → `jig_view`, `jig_charter` →
  `jig_protocol_charter`, `jig_client_transcript` →
  `jig_protocol_transcript`, and so on. A local alias (`use jig_core as
  core;`) keeps its alias; only its crate path changes.
- **`jig-domain-core` becomes an explicit workspace member** (today
  `crates/jig-core` is a member only as a path dependency,
  `Cargo.toml:96`).
- **The lock moves for jig's own packages only:** `heavy cargo check
  --workspace --all-targets` rewrites their entries; no `cargo update`.
- **Markdown outside `docs/design/` and `docs/development/plans/`** that
  names a renamed crate moves with it, in the same increment
  (`AGENTS.md:39`, `tests/client/README.md:52`,
  `docs/development/tech-debt.md`). `docs/design/` is the designer's
  (development.md, 8.2) and is left as built (README, 15).
- **A role `tests/boundary` defines by prefix becomes a list** where the
  new prefix would take in other crates: `jig-domain-` also names the
  hub and the client's domain, so the core's family is the core and its
  eight children by name; `client_step`'s `jig-client-` prefix
  (`tests.rs:743-755`) becomes a list.
- **`jig-inline-agent`** leaves `tests/boundary` in 01.1, in the test
  that names `jig-host` beside it (`tests.rs:246`); the crate is gone
  (`91f8bfec`).

## Rules that bind it

- "**jig 01 runs alone.** No other jig session starts before it has
  landed whole: it renames the crates every jig session touches"
  (README, "Rules").
- "Directories follow the crates' names (`crates/jig-domain-core/`); the
  Rust paths follow too (`jig_domain_core`). Test kits, worlds, `ops`'s
  domains and `jig-boundary` keep their names" (code-organization.md,
  section 1).
- "**No increment of this plan** ... adds a test that checks several
  behaviours under one name" (code-organization.md, section 6): each of
  section 3's rules is a test of its own, with a fixture test of its own.

## Increments

### 01.1 The domain's crates
Scope: README, 15's rows `jig-core` → `jig-domain-core`;
  `jig-core-{accounts, authority, brief, fleet, notes, people, tasks,
  views}` → `jig-domain-*`; `jig-host` → `jig-domain-host`. Packages,
  directories, workspace `members` and `[workspace.dependencies]`,
  dependency keys and Rust paths in each crate, kit, world and
  `examples/ops`; `jig-domain-core` an explicit member; the Markdown the
  Choices name; `tests/boundary`'s roles for these crates.
Needs: nothing.
Files: `crates/jig-core*/**` and `crates/jig-host/**` (moved); the
  workspace `Cargo.toml:5-52,60-101`; `Cargo.lock`; each file that names
  them: about 295 files, among them `examples/ops/domain` (11),
  `testing/jig-test-domain` (5), `tests/{accounts, brief, client,
  conformance, core, fleet, host, notes, ops, people, tasks, views}`
  (199 together); `tests/boundary/src/tests.rs:216-241` (the core's
  family), `:243-258` (`jig-host`, `jig-inline-agent` gone, the core's
  family by list), `:260-274`, `:757-775`, `:896-939` (the client
  world's list); `AGENTS.md:39`; `docs/development/tech-debt.md`.
  Shared hotspots: each of them; the session runs alone.
Shape: the table in the sketch below, rows 01.1. The core's family, in
  `tests/boundary/src/tests.rs`:

  ```rust
  const CORE_FAMILY: [&str; 9] = [
      "jig-domain-core", "jig-domain-accounts", "jig-domain-authority", "jig-domain-brief",
      "jig-domain-fleet", "jig-domain-notes", "jig-domain-people", "jig-domain-tasks", "jig-domain-views",
  ];
  ```

  `core_and_children_use_only_the_kit_and_foundation_even_when_empty`
  reads it: `jig-domain-core` depends on `skein-lib` and the other eight;
  each of the eight on `skein-lib`.
Test shape: no new test. The boundary's tests keep their names and
  rules, their strings renamed. Each focused test keeps its ceiling.
Proof: the gate (`land` runs `scripts/check.sh`), which builds and runs
  each world on the new names; and, once before the commit, `git grep -n
  -P 'jig[-_](core([-_](accounts|authority|brief|fleet|notes|people|tasks|views))?|host)(?![-_a-z0-9])'
  -- ':!docs/design' ':!docs/development/plans'`, which prints nothing,
  and `git diff origin/main -- Cargo.lock | grep '^[-+]source'`, which
  prints nothing. Under 5 minutes.
Don't: rename a world (`jig-core-world`, `jig-host-world` and the rest
  keep their names), a kit, `jig-test-domain` or `jig-boundary`; edit
  `docs/design/`; change a test's expected value, except a crate's name in
  a boundary rule.
Size: no production code added; a mechanical diff of about 1,500 lines.
Removes: the old names of these crates; `crates/jig-core`'s path-only
  membership; `jig-inline-agent` in `tests/boundary`.

### 01.2 The client's and the protocol's crates
Scope: README, 15's rows `jig-client` → `jig-domain-client`,
  `jig-client-view` → `jig-view`, `jig-charter` → `jig-protocol-charter`,
  `jig-client-transcript` → `jig-protocol-transcript`,
  `jig-test-client-domain` → `jig-test-domain-client`,
  `jig-test-client-view` → `jig-test-view`; the same reach as 01.1.
  `jig-test-client-kit` keeps its name (a kit); its dependency keys
  follow.
Needs: nothing.
Files: those crates (moved); the workspace `Cargo.toml`; `Cargo.lock`;
  each file that names them: about 353 files, among them
  `crates/jig-client-view` (121, its own Rust paths),
  `testing/jig-scripted-engine` (57), `tests/client` (84, with the
  include paths `src/joint/archive.rs:225`,
  `src/joint/wire/activity.rs:314` and `src/joint/wire/turns.rs:18` of
  `crates/jig-client-transcript/tests/data/record_turn_full.bin`),
  `tests/ops` (13), `examples/ops/{client-domain, client-view, domain}`,
  `testing/{jig-fake-person, jig-test-client-kit}`;
  `tests/boundary/src/tests.rs:243-258` (`jig-charter`), `:708-741`,
  `:743-755` (`client_step` by list), `:777-788`, `:791-817`,
  `:819-894` (the kits' fixed lists and the two client roots' source
  paths, `:859`, `:889`), `:896-939`, `:951-977`;
  `tests/client/README.md:52`; `docs/development/tech-debt.md`.
Shape: the table, rows 01.2. `client_step` becomes:

  ```rust
  fn client_step(name: &str) -> bool {
      matches!(
          name,
          "jig-domain-client"
              | "jig-view"
              | "jig-protocol-transcript"
              | "jig-test-domain-client"
              | "jig-test-view"
              | "jig-ops-connectors"
              | "jig-ops-client-domain"
              | "jig-ops-client-view"
      )
  }
  ```

  (01.3 renames its three `ops` names.)
Test shape: `client_roles_cover_step_crates_and_exclude_the_world`
  (`tests.rs:777-788`) asserts the renamed names, and
  `!client_step("jig-client-world")` stays. No other test changes but
  its strings.
Proof: the gate; and, once before the commit, `git grep -n -P
  'jig[-_](client([-_](view|transcript))?|charter|test[-_]client[-_](domain|view))(?![-_a-z0-9])'
  -- ':!docs/design' ':!docs/development/plans'`, which prints nothing,
  and 01.1's `Cargo.lock` check. Under 5 minutes.
Don't: rename `jig-client-world` or `jig-test-client-kit`; move the
  transcript's golden files except with their crate.
Size: no production code added; a mechanical diff of about 2,000 lines.
Removes: the old names of these crates.

### 01.3 `ops`'s crates
Scope: README, 15's rows `jig-ops-client-domain` →
  `jig-ops-domain-client` (`examples/ops/domain-client`),
  `jig-ops-client-view` → `jig-ops-view` (`examples/ops/view`),
  `jig-ops-connectors` → `jig-ops-domain-connectors`
  (`examples/ops/domain-connectors`); the same reach.
Needs: nothing.
Files: those crates (moved); the workspace `Cargo.toml:25-27,86-88`;
  `Cargo.lock`; about 36 files, among them `examples/ops/domain` (7) and
  `tests/ops` (13); `tests/boundary/src/tests.rs` (`client_step`,
  `:941-977` with the source paths `:945`, `:958`, `:973`).
Shape: the table, rows 01.3.
Test shape: `ops`'s three boundary tests under the new names and paths;
  `client_roles_cover_step_crates_and_exclude_the_world` asserts
  `client_step("jig-ops-domain-connectors")`.
Proof: the gate; and, once before the commit, `git grep -n -P
  'jig[-_]ops[-_](client[-_](domain|view)|connectors)(?![-_a-z0-9])|examples/ops/(client-domain|client-view|connectors)\b'
  -- ':!docs/design' ':!docs/development/plans'`, which prints nothing,
  and README "Done when" item 3's `git grep`, which prints nothing.
  Under 5 minutes.
Don't: rename `ops`'s domains (`jig-ops-domain`,
  `jig-ops-domain-observability`, `jig-ops-domain-infrastructure`) or
  `jig-ops-fake-production`.
Size: no production code added; a mechanical diff of about 200 lines.
Removes: the old names of these crates.

### 01.4 `tests/boundary`'s rules for the plan's crates
Scope: code-organization.md, section 3, as tests of the crate graph,
  each holding "even when empty" for crates later sessions add:
  - the core's family links `skein-lib` and its own (01.1's rule, as
    is);
  - `jig-domain-host` links only `skein-lib` (`host_hub_links_only_skein`,
    as is);
  - new: no `jig-protocol-*` crate depends on an application's crate (a
    name starting `jig-ops-` or `jig-test-`, or the first application's);
  - new: among jig's kit crates (`jig-domain-*`, `jig-protocol-*`,
    `jig-view`, `jig-shell-*`), only `jig-protocol-charter`
    (`smith-domain-run`, `smith-charter`) and `jig-protocol-transcript`
    (`smith-transcript`) link smith;
  - new: `jig-test-worker` and `jig-test-service` link no smith crate
    but `smith-host-domain`, `smith-host-protocol`,
    `smith-protocol-channel` and `smith-channel`;
  - new: the client's crates (`jig-domain-client`, `jig-view`,
    `jig-protocol-client-wire`, `jig-protocol-client-link`,
    `jig-shell-native`) depend on no engine-side crate: the core's
    family, `jig-domain-host`, or a `jig-protocol-*` crate but
    `jig-protocol-client-wire`, `jig-protocol-client-link` and
    `jig-protocol-transcript`.
  `tests/boundary/src/tests.rs` (1,009 lines) is split.
Needs: nothing beyond 01.1 to 01.3 (the new names).
Files: `tests/boundary/src/tests.rs` (kept: the manifest walk and
  parsing, `tests.rs:1-203`, and their fixture tests, at most 400
  lines); new `tests/boundary/src/tests/graph.rs` (the crate-graph
  rules and their fixtures, at most 800 lines); new
  `tests/boundary/src/tests/vocabulary.rs` (the 64 vocabulary parts and
  their fixtures, `tests.rs:276-706,979-1009`).
Shape: each rule is a function over manifests (the sketch below) and a
  test over the workspace's:

  ```rust
  #[test]
  fn only_the_charter_and_the_transcript_link_smith_even_when_empty() {
      let failures = smith_breaches(&manifests());
      assert!(failures.is_empty(), "{}", failures.join("\n"));
  }
  ```

  and likewise for `application_breaches`, `test_worker_smith_breaches`
  and `client_engine_breaches`; `core_breaches` is 01.1's rule, moved
  into a function. The existing tests keep their names, moved into
  `graph.rs` or `vocabulary.rs`.
Test shape: `tests/boundary/src/tests/graph.rs`. Each new rule has a
  fixture test that plants a breach in a manifest written as text and
  reads it with `parse_manifest` (`tests.rs:61-91`), as
  `fixtures_name_the_path_and_the_breach` does (`tests.rs:665-689`).
  Reference test:

  ```rust
  #[test]
  fn a_protocol_crate_linking_smith_is_reported_unless_charter_or_transcript() {
      let tools = parse_manifest(
          PathBuf::from("fixture/tools/Cargo.toml"),
          "[package]\nname = \"jig-protocol-tools\"\n[dependencies]\nsmith-charter.workspace = true\n",
      );
      let charter = parse_manifest(
          PathBuf::from("fixture/charter/Cargo.toml"),
          "[package]\nname = \"jig-protocol-charter\"\n[dependencies]\nsmith-charter.workspace = true\n",
      );
      assert_eq!(smith_breaches(&[tools, charter]), ["fixture/tools/Cargo.toml: smith-charter"]);
  }
  ```

  The others: `only_the_charter_and_the_transcript_link_smith_even_when_empty`,
  `protocol_crates_link_no_application_crate_even_when_empty`,
  `a_protocol_crate_naming_an_ops_crate_is_reported`,
  `test_worker_and_service_link_only_the_four_smith_crates_even_when_empty`,
  `a_test_service_linking_smith_charter_is_reported`,
  `client_crates_link_no_engine_side_crate_even_when_empty`,
  `a_native_shell_naming_the_store_is_reported` (`jig-shell-native` on
  `jig-protocol-store`). Each under 20 ms, as the manifest tests run
  today.
Proof: unit, run once with `heavy cargo nextest run -p jig-boundary`:
  the workspace passes each rule, and each fixture's planted breach is
  reported with its manifest's path and dependency. Under 1 minute.
Don't: change the vocabulary's word list or `names.rs`; drop or weaken
  an existing rule; count `[dev-dependencies]`, which `parse_manifest`
  does not read (`tests.rs:76-81`).
Size: test code only: at most 250 lines added.
Removes: nothing.

## Interface sketches

The renames, with the increment that makes each (README, 15;
code-organization.md, section 1):

| As built | After | Directory after | Increment |
|---|---|---|---|
| `jig-core` | `jig-domain-core` | `crates/jig-domain-core` | 01.1 |
| `jig-core-<child>` (accounts, authority, brief, fleet, notes, people, tasks, views) | `jig-domain-<child>` | `crates/jig-domain-<child>` | 01.1 |
| `jig-host` | `jig-domain-host` | `crates/jig-domain-host` | 01.1 |
| `jig-client` | `jig-domain-client` | `crates/jig-domain-client` | 01.2 |
| `jig-client-view` | `jig-view` | `crates/jig-view` | 01.2 |
| `jig-charter` | `jig-protocol-charter` | `crates/jig-protocol-charter` | 01.2 |
| `jig-client-transcript` | `jig-protocol-transcript` | `crates/jig-protocol-transcript` | 01.2 |
| `jig-test-client-domain` | `jig-test-domain-client` | `testing/jig-test-domain-client` | 01.2 |
| `jig-test-client-view` | `jig-test-view` | `testing/jig-test-view` | 01.2 |
| `jig-ops-client-domain` | `jig-ops-domain-client` | `examples/ops/domain-client` | 01.3 |
| `jig-ops-client-view` | `jig-ops-view` | `examples/ops/view` | 01.3 |
| `jig-ops-connectors` | `jig-ops-domain-connectors` | `examples/ops/domain-connectors` | 01.3 |

```rust
// jig-boundary (tests/boundary/src/tests/graph.rs): the rules' functions,
// over the manifests `tests.rs` parses; each returns "<path>: <dependency>"
// for each breach.
use std::path::PathBuf;

/// A crate's manifest as `tests.rs` parses it (`tests.rs:4-9`).
#[derive(Debug)]
pub struct CrateManifest {
    pub path: PathBuf,
    pub name: Option<String>,
    pub dependencies: Vec<String>,
}

/// The core's family depends on `skein-lib` and its own.
#[must_use]
pub fn core_breaches(manifests: &[CrateManifest]) -> Vec<String> {
    todo!()
}

/// A `jig-protocol-*` crate depends on an application's crate.
#[must_use]
pub fn application_breaches(manifests: &[CrateManifest]) -> Vec<String> {
    todo!()
}

/// A kit crate but the charter and the transcript links smith.
#[must_use]
pub fn smith_breaches(manifests: &[CrateManifest]) -> Vec<String> {
    todo!()
}

/// `jig-test-worker` or `jig-test-service` links a smith crate but its four.
#[must_use]
pub fn test_worker_smith_breaches(manifests: &[CrateManifest]) -> Vec<String> {
    todo!()
}

/// A client's crate depends on an engine-side crate.
#[must_use]
pub fn client_engine_breaches(manifests: &[CrateManifest]) -> Vec<String> {
    todo!()
}
```

Later sessions add their crates to these functions' lists only where
code-organization.md, section 3 names them; jig 01 owns the rules
(code-organization.md, section 4).

## Questions to raise when reached

- **A name collision:** a new name already taken by a crate, a
  directory or a module. Report it; rename nothing else.
- **A crate made after this plan's start** that README, 15 does not
  name: report it; it keeps its name.

## Done when

1. The crates carry their new names (README "Done when" item 3): `git -C
   ~/src/rust/jig grep -n -E
   '"(jig-core|jig-host|jig-charter|jig-client|jig-client-view|jig-client-transcript)"'
   -- '*.toml'` prints nothing.
2. No old name is left outside the design and the plans: `git -C
   ~/src/rust/jig grep -n -P
   'jig[-_](core([-_](accounts|authority|brief|fleet|notes|people|tasks|views))?|host|client([-_](view|transcript))?|charter|test[-_]client[-_](domain|view)|ops[-_]client[-_](domain|view)|ops[-_]connectors)(?![-_a-z0-9])'
   origin/main -- ':!docs/design' ':!docs/development/plans'` prints
   nothing.
3. `jig-domain-core` is a member: `git -C ~/src/rust/jig grep -n
   '"crates/jig-domain-core"' Cargo.toml` prints its `members` line.
4. The rules hold: `heavy cargo nextest run -p jig-boundary` passes, and
   `git -C ~/src/rust/jig grep -n jig-inline-agent -- tests` prints
   nothing.
5. The four increments landed through `land`: `git -C ~/src/rust/jig log
   --format='%(trailers:key=Step,valueonly)' origin/main` lists
   `jig-01.1` to `jig-01.4`, each commit's `Gate-Result` naming
   `scripts/check.sh passed`.
