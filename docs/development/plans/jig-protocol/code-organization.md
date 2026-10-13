# jig-protocol: code organization

What every session of this plan builds into, and the rules that bind them
all. Session files give each increment's paths and signatures; this file
gives the crates, their kinds and names, the owner of each shared
interface, the plan's "Don't" list and its size limits.

## 1. Names

- **Crates are named by layer** (jig's `docs/design/README.md`, section
  15). jig 01 renames the crates that exist; every later session uses the
  new names, and new crates take theirs at once:

  | As built | After jig 01 |
  |---|---|
  | `jig-core` | `jig-domain-core` |
  | `jig-core-accounts`, `-authority`, `-brief`, `-fleet`, `-notes`, `-people`, `-tasks`, `-views` | `jig-domain-accounts`, `-authority`, `-brief`, `-fleet`, `-notes`, `-people`, `-tasks`, `-views` |
  | `jig-host` | `jig-domain-host` |
  | `jig-client` | `jig-domain-client` |
  | `jig-client-view` | `jig-view` |
  | `jig-charter` | `jig-protocol-charter` |
  | `jig-client-transcript` | `jig-protocol-transcript` |
  | `jig-test-client-domain` | `jig-test-domain-client` |
  | `jig-test-client-view` | `jig-test-view` |
  | `jig-ops-client-domain` | `jig-ops-domain-client` |
  | `jig-ops-client-view` | `jig-ops-view` |
  | `jig-ops-connectors` | `jig-ops-domain-connectors` |

  Directories follow the crates' names (`crates/jig-domain-core/`); the
  Rust paths follow too (`jig_domain_core`). Test kits, worlds, `ops`'s
  domains and `jig-boundary` keep their names.
- **This plan's session files write the new names** throughout, even for
  increments that start before jig 01 lands: none does (README, "Rules").

## 2. Crates this plan adds

| Crate | Directory | Kind | Holds | Session that makes it |
|---|---|---|---|---|
| `jig-protocol-config` | `crates/jig-protocol-config` | step code | jig's part of a configuration: parsing, profiles, the whole check (`protocol/configuration.md`) | jig 20 |
| `jig-protocol-accounts` | `crates/jig-protocol-accounts` | step code | the accounts child's vocabulary to skein's driver's and back, the access tokens' table, handed-in accounts (`protocol/llm.md`, 5 and 6) | jig 21 |
| `jig-protocol-tools` | `crates/jig-protocol-tools` | step code | the core's host tools: schemas, descriptions, decoding, rendering (`protocol/channel.md`, 6.3) | jig 10 |
| `jig-protocol-worker-wire` | `crates/jig-protocol-worker-wire` | codecs | the workers' channel's records, generated, and the sizes function (`protocol/channel.md`, 3, 5, 11) | jig 22 |
| `jig-protocol-engine-workers` | `crates/jig-protocol-engine-workers` | component | the engine's side of the workers' channel | jig 23 |
| `jig-protocol-worker-link` | `crates/jig-protocol-worker-link` | component | the worker's side, the link | jig 24 |
| `jig-protocol-store` | `crates/jig-protocol-store` | component, codecs | the store (`protocol/store.md`) | jig 13 |
| `jig-protocol-client-wire` | `crates/jig-protocol-client-wire` | codecs | the client's records, generated (`protocol/client.md`, 3) | jig 30 |
| `jig-protocol-engine-clients` | `crates/jig-protocol-engine-clients` | component | the engine's side of the client's protocol | jig 31 |
| `jig-protocol-client-link` | `crates/jig-protocol-client-link` | component | the client's side, for the host only | jig 32 |
| `jig-shell-native` | `crates/jig-shell-native` | ordinary Rust | the native shell over skein's HTTP client | jig 33 |
| `jig-test-worker` | `testing/jig-test-worker` | step code | the test worker's root: the hub, `smith-host-domain`, a test workspace | jig 25 |
| `jig-test-service` | `testing/jig-test-service` | step code | the reference service: the test application's engine and a test worker, each with its protocol layer and `iterate`, in one process | jig 25 |
| `jig-test` | `testing/jig-test` | ordinary Rust, a binary | the reference binary: `main`, configuration, the store on a directory, the `agent` command | jig 26 |
| `jig-test-protocol-client-wire` | `testing/jig-test-protocol-client-wire` | codecs | the test application's family's client records, generated | jig 33 |
| `jig-test-store` | `testing/jig-test-store` | step code | the test application's part of the store and its routing, which the reference service uses | jig 18 |
| `jig-test-store-files` | `testing/jig-test-store-files` | ordinary Rust | the in-memory stand-in for transcripts' files, for the domain worlds (`protocol/store.md`, 14) | jig 18 |
| worlds | `tests/channel`, `tests/store`, `tests/store-codecs`, `tests/client-protocol`, `tests/one-process`, `tests/worker-wire`, `tests/engine-workers`, `tests/worker-link`, `tests/protocol-accounts`, `tests/client-wire`, `tests/engine-clients` | ordinary Rust | the protocol worlds; the store's codecs' drift and fuzzing; the channel's components' machine worlds; the accounts' component against skein's fake issuer; the client's wire and the engine's clients' side | jig 25, 18, 14, 33, 26, 22, 23, 24, 21, 30, 31 |

- **Step code** follows skein's `programming-model.md`, section 10, and
  jig's `AGENTS.md`: no traits, no generic items, no `dyn`, no closures.
  `jig-test-service` and `jig-test-worker` are step code so that they
  can be copied (`protocol/service.md`, section 7). Ordinary Rust crates
  take `tests/clippy.toml`'s lints, as `testing/jig-test-system` does.
- **Generated codecs** are committed under each codec crate's
  `src/generated/`, made by skein's generator from the crate's
  `schema/*.schema`, with skein's drift test, as `smith-transcript` does.
  Generated files do not count against size limits.
- **No crate of this plan links a browser or wasm32 toolchain.** No
  `jig-shell-web`, no wasm32 target in any `Cargo.toml` or script.

## 3. Who links what

The rules `tests/boundary` checks (jig 01 writes them; later sessions add
their crates to them). A dependency this section allows that
`tests/boundary`'s lists lack is added to them by the increment that adds
the dependency, at most 2 lines, named in its commit body; it is no
question (decisions.md, ruling 104).

- **The core and its children** (`jig-domain-*` but `jig-domain-host` and
  `jig-domain-client`) depend on no `jig-protocol-*` crate, no smith crate
  and no application.
- **`jig-domain-host`** links no smith crate (jig's
  `domain/hosts.md`, 6.8).
- **`jig-protocol-*` crates** depend on no application's crate, and on
  no domain crate but the vocabulary they translate (`jig-protocol-charter`
  on `jig-domain-tasks`; a component on the core's or the hub's
  boundary). Only `jig-protocol-charter` and `jig-protocol-transcript`
  link smith; the channel's components do not.
- **`jig-test-worker` and `jig-test-service`** may link smith's
  `smith-host-domain`, `smith-host-protocol`, `smith-protocol-channel` and
  `smith-channel`. `jig-test-service` may also link
  `smith-domain-run`, for `Identity` alone (decisions.md, ruling 239),
  until one of those four exports it.
- **The client's crates** (`jig-domain-client`, `jig-view`,
  `jig-protocol-client-wire`, `jig-protocol-client-link`,
  `jig-shell-native`) depend on no engine-side crate.

## 4. Shared interfaces and their owners

One session defines each; every other session uses it by name and
extends it only as its increment says.

| Interface | Where | Owner |
|---|---|---|
| the crates' names, `tests/boundary`'s rules | workspace | jig 01 |
| the hub's link vocabulary: the hello with its agent's bounds, turns (with whether they open a window and the refusals carried), the answer (with the unread fence), facts as typed activity, notices | `jig-domain-host`'s boundary | jig 05 |
| the fleet's events and requests that mirror it, and the fleet's refusal of a hello with a reason | `jig-domain-fleet`'s boundary | jig 05 |
| the assignment's fields (the start's messages, the run's identity, credential grants of both owners) in the hub and the fleet | `jig-domain-host` and `jig-domain-fleet` | jig 05 alone (decision 6); jig 04, 06 and 08 fill them and change none |
| the core's part of the assignment, `Now::RunPrepared` | `jig-domain-core` | jig 04 adds the start's messages, jig 06 the conversation's first attempt, jig 08 the grants list; each its own field, in place; hotspot |
| `RunProof`, a core record | `jig-domain-core`'s `stored.rs` | jig 04 (`offered`), 06 (`transcript_from` removed), 07 (the claim's host), 08 (`credentials`) and 11 each change their own field, in place; hotspot; its codec follows (README, "Rules") |
| the core's vocabulary toward the root (`routing/vocabulary.rs`) | `jig-domain-core` | each session adds its variants; hotspot |
| the store's boundary (commit, restore, load, read, get, transcript, secrets; fill) | `jig-protocol-store`'s `boundary.rs` | jig 13 |
| the store's keys' encoders, families' codecs and the format row | `jig-protocol-store` | jig 13, jig 14 and jig 19 |
| the workers' channel's records and sizes | `jig-protocol-worker-wire` | jig 22 |
| the client's records | `jig-protocol-client-wire` | jig 30 |
| jig's typed configuration | `jig-protocol-config` | jig 20 |
| the reference service's routing and startup | `jig-test-service` | jig 25 |

## 5. Tests and fixtures

- **Each crate's fixtures** live in one module, `src/tests/fixtures.rs`
  for a crate's step tests, or the world's own `src/fixtures.rs`, made by
  the first session that tests the crate and extended by the others,
  never redefined.
- **Worlds** keep their layout: `tests/<world>/src/` for the world, its
  focused tests in `tests/<world>/tests/*.rs`, its fuzzy ones in
  `tests/<world>/tests/fuzzy_*.rs` (jig's `docs/development/workflow.md`,
  section 2).
- **Ceilings per test,** unoptimized on the idle machine: a unit test
  under 20 ms, a focused world test under 0.5 s, a fuzzy test about 5 s.
  A new world costs at most about 1 s focused and 5 s fuzzy, in all.
- **The budgets bind the gate's wall time:** 15 s focused, 60 s fuzzy
  (`.config/nextest.toml`). On 2026-10-12 the gate took 9.0 s focused
  and 12.0 s fuzzy: the focused suite has about 6 s left for the whole
  plan.

## 6. The plan's "Don't" list

No increment of this plan:

- adds a type beside an existing one for the same thing: it changes the
  type in place, and its users in the same increment;
- adds a field or predicate that its counterpart across a boundary lacks;
- adds a test that checks several behaviours under one name;
- adds a lint exception in tests;
- computes an expected value with the code under test;
- adds a browser, a web shell, wasm32 builds, or a dependency on a
  browser kit;
- decodes or encodes bytes in a domain crate: decoding is a protocol
  layer's, reached by hand-out where a domain needs a value typed
  (jig's `domain/hosts.md`, 6.8; `protocol/channel.md`, 6.3);
- links smith from `jig-domain-host`, the core or its children, or the
  channel's components;
- adds an `unreachable!` arm for a variant the reference service can
  reach; an arm the service reaches is routed;
- leaves a legacy or compatibility path past the increment that its
  session file names to remove it;
- writes the words `legacy`, `compat`, `hack` or `workaround` in code.

## 7. Size limits

- **A file:** at most 800 lines, generated code excepted. A file of at
  most 800 lines that the increment grows past 800 is split in the same
  increment. A file already past 800 lines is not split by this rule: it
  grows by at most what its session file says (20 lines where the session
  file names no number), and new code goes in new modules.
- **A test function:** at most 40 lines.
- **An increment:** at most about 600 lines of production code; its
  session file gives its own limit, "Size: at most N lines". That limit
  is a guide (decisions.md, ruling 103): an increment within N × 1.5 (ruling 196; 1.2 before),
  rounded down, lands, and its report's Evidence line gives its count
  and how it counted; past N × 1.5 (ruling 196; 1.2 before), it asks before landing, as a
  question. Nobody restructures code only to meet it. The file and
  test-function limits above stay rules.
- **What a Size counts** (ruling 110): lines of production code added,
  less those removed, that are neither blank nor comments (doc comments
  included), outside tests and generated code; `Cargo.toml` and
  `Cargo.lock` count for nothing. Count once, with one command, and name
  it in the report.
- **`Cargo.lock`** (ruling 110): the entries a Files-named `Cargo.toml`
  change generates belong to that change, and are no question. They add
  packages only: `git diff origin/main -- Cargo.lock` shows no existing
  package's version or source changed, unless the increment's Shape
  repins it.
