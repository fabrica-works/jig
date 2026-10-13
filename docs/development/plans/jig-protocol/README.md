# jig-protocol: jig's protocol layer, for its first application

2026-10-12. Start commits: skein `917ea21`, smith `2a800ea2`, jig
`1cdfeb62` (each repository's `origin/main` when the plan was written;
each session starts from its repository's `origin/main`).

A plan to make jig's protocol layer real, so that an application whose
agents run on workers can run as one process on one person's machine:
the engine and a worker joined in memory, the worker's agents as child
processes, a store on disk, and a native client on loopback. It builds
skein's designed pieces this needs, takes them into smith, pays what
jig's domain owes the protocol layer, and builds jig's components, its
store, its reference service and binary, and their worlds. No browser
and no wasm32 (the user's choice, 2026-10-12).

## Inputs

- **jig's design** on jig's `main` (`docs/design/`): `protocol/README.md`
  first, then `service.md`, `channel.md`, `store.md`, `configuration.md`,
  `llm.md` (sections 5 and 6), `client.md` (but section 12's browser);
  `domain/hosts.md`, `engine.md`, `tasks.md`, `authority.md`,
  `people.md`, `connectors.md`, `root.md`, where `STATUS.md`'s rows name
  them; `README.md`, section 15 (names).
- **skein's design** on skein's `main`: `io.md`, 3.5, section 4 and
  section 5 (`OpenWrite`, `Cut`); `kernel.md`, 6.1; `simulator.md`, 3;
  `shell.md`, 5 and 6.1; `lib.md`, 6, 6.1 and 8; `codec.md`, 3;
  `oauth.md`, 6.2 to 6.4.
- **skein's foundation:** `development.md` (this plan follows it),
  `programming-model.md`, `testing-strategy.md`.
- **`decisions.md`:** what this plan settled.
- **`findings/`:** the code audit: `skein.md` (skein's pieces and smith's
  repin), `domain.md` (what jig's domain owes), `store.md` (the store),
  `protocol.md` (jig's kits, client, worlds, and the old draft).

## The sessions

A session waits for what "Needs at start" names; an increment that needs
more says so in its row and its file. Each increment also builds on the
ones before it in its session. Dependencies are by capability: "skein
01's `Addr`" means the increment of skein 01 that lands io's `Addr`.

### skein

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `skein/01-addresses.md` | io's `Addr` with in-process names, `Limits::names`, in-process listeners and joined sockets, their world | nothing | none |
| `skein/02-startup-and-secrets.md` | `read_private_file` and the cryptographic seed; `lib::Secret` and `secret N`; `lib::base64`, skein's copies moved onto it; `CryptoRng` by stream, OAuth's states and verifiers from it; the accounts driver dropping a record failed as sign in again | nothing | none |
| `skein/03-files.md` | the kernel's `Write` open and `Truncate`, io's `OpenWrite` and `Cut`, the simulator's disk keeping a file's length until synced | nothing | none |

### smith

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `smith/01-skein.md` | smith on skein with skein 01 to 03: addresses, `io::Limits::names`, the kernel's opens | skein 01, 02 and 03, landed whole | none |

### jig: the start

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `jig/01-renames.md` | crates renamed by layer; `tests/boundary`'s rules | nothing; **alone** (Rules) | none |
| `jig/02-skein.md` | jig on skein's and smith's new revisions | jig 01; smith 01 | none |

### jig: what the domain owes

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `jig/03-people-and-tasks.md` | people's and tasks' refusals and timers, a reply's own sign-in, chats forgotten, signing out, reads answered signed out, the message bound | jig 01 | none |
| `jig/05-hub-link.md` | the hub's link vocabulary: the hello with its agent's bounds and the fleet's check; turns opening a window and carrying refusals; the unread fence; facts as typed activity; notices (rejected, exhausted, told) routed where the core has `unreachable!` today; text by retry, a gap cut | jig 01 | the fleet's check of the hello's agent bounds (05.6) needs jig 07's charters and reserve, and jig 04's run inbox count |
| `jig/04-messages.md` | the start carries the inbox, the brief has no inbox section; refusals committed with turns; a refused or unread message stays and wakes its task | jig 05's assignment (05.2) | 04.3 (refusals committed) needs jig 05's turns with refusals and the answer's fence (05.3) |
| `jig/06-windows.md` | the assignment names the conversation's first attempt (recorded at every claim, a resuming charter's first one included) and the run's identity (`run_identity`, in `jig-protocol-charter`); the core keeps each task's current window; a resume reads the window; the claim records its host kind | jig 01 | 06.2 (the assignment) needs jig 05's assignment (05.2); 06.3 (the window) needs jig 05's turns opening a window (05.3); 06.5 (the claim's host kind) needs jig 07.4 |
| `jig/07-charters-and-budgets.md` | several charters, the chats' charter, a default budget, one account per charter; no run placed whose budget cannot hold its reserve and one most costly completion | jig 01 | none |
| `jig/08-accounts-and-credentials.md` | the accounts child on skein's driver's terms (wanted, expiring, five failures); grants pushed and resent, replacing the core's `unreachable!` for the fleet's grants; an application's credentials | jig 05's assignment; jig 07's one account per charter | none |
| `jig/09-stopping.md` | the engine stopping; in-process workers' claims settled at once | jig 01 | none |
| `jig/10-host-tools.md` | `jig-protocol-tools`; host tools' calls decoded and answers rendered outside the root; a run's result and a call's key read by `jig-protocol-charter` | jig 01 | none |

### jig: the store

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `jig/11-store-domain.md` | live before history in the children's keys; keys for single reads; fill and the high-water mark | jig 01 | none |
| `jig/12-store-indexes.md` | the inbox's index and its restore; conversations', goals', ended delegates' and subtrees' indexes | jig 11's keys | none |
| `jig/13-store-keys.md` | `jig-protocol-store`: the crate, its boundary, keys' encoders, tags, ranges to intervals | jig 11's keys | the intervals of indexed scopes need jig 12 |
| `jig/14-store-codecs.md` | the codecs' pattern, the component's own rows, the notes' and the tasks' families, generated, with their translations | jig 13's keys | none |
| `jig/19-store-codecs-core-people.md` | the core's and people's families' codecs; the format row, the dispatch, sizes | jig 14's pattern (14.1) | 19.1 (the core family) needs jig 14.3's tasks types; 19.5 needs jig 14 whole |
| `jig/15-store-commits-loads.md` | commits, restart loads (the census's ended rows joined), pages and gets, over skein-kv's memory mode | jig 14; jig 19 | 15.3 (restart loads) needs jig 13's census intervals (13.5, after jig 12) |
| `jig/16-store-reads.md` | parties' reads: point scopes with their joins, paged scopes | jig 15; jig 12 | none |
| `jig/17-store-files.md` | transcripts' files and the secrets' store | jig 15; jig 02 (skein's `OpenWrite` and `Cut`); jig 06 (windows) | 17.5 (turns in parties' reads) needs jig 16 |
| `jig/18-store-worlds.md` | the test application's keys and codecs; the files' stand-in; jig's worlds on the real store, `jig-fake-store` kept for `ops` only; the store's world on the crashing disk | jig 16; jig 17 | none |

### jig: the workers' channel and the reference service

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `jig/20-config.md` | `jig-protocol-config` | jig 02 (`read_private_file`); jig 07 | none |
| `jig/21-accounts.md` | `jig-protocol-accounts`, handed-in accounts | jig 08; jig 02 (`Secret`) | none |
| `jig/22-worker-wire.md` | `jig-protocol-worker-wire` | jig 05; jig 02 (`secret N`) | none |
| `jig/23-engine-workers.md` | `jig-protocol-engine-workers` | jig 22 | none |
| `jig/24-worker-link.md` | `jig-protocol-worker-link` | jig 22 | 24.5 (the two sides joined) needs jig 23.3 |
| `jig/25-test-service.md` | `jig-test-worker`, `jig-test-service`'s spine, the channel's protocol world and its first story | jig 23; jig 24; jig 10; jig 20; jig 21; jig 05 | none |
| `jig/27-channel-stories-and-store.md` | the channel world's stories and referee; stopping and draining the service; the service on the real store | jig 25; jig 04; jig 06; jig 09 | 27.3 (the store) needs jig 18 |
| `jig/26-one-process.md` | the one-process world; the reference binary `jig-test`; the process and live proofs | jig 27; jig 33 | none |

### jig: the client's protocol

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `jig/30-client-wire.md` | `jig-protocol-client-wire` | jig 03 | none |
| `jig/31-engine-clients.md` | `jig-protocol-engine-clients`: exchanges, the stream, sign-ins (the printed link, a native bearer), an application's providers routed | jig 30; jig 02 (`CryptoRng`, `Secret`, `base64`); jig 17 (the secrets' store) | 31.4 (reads and cursors) needs jig 34's pages and single reads (34.3, 34.4) |
| `jig/32-client-link.md` | `jig-protocol-client-link`, for the host: attempts and envelopes, reads, the stream, the small store | jig 30 | 32.2 (reads) needs jig 35's documents (35.2); 32.3 (the stream) needs jig 02 (`base64`) |
| `jig/34-engine-clients-documents.md` | the engine's clients' side: jig's documents encoded | jig 31.1 | none |
| `jig/35-client-link-documents.md` | the client's link: jig's documents decoded | jig 32.1 | none |
| `jig/33-native-shell.md` | `jig-shell-native`; the test application's wire; the reference service's clients' side; the client's protocol world | jig 31; jig 32; jig 34; jig 35; jig 25 | none |

### Repins (development.md, 5.7)

- **`smith-repin`:** after the last skein increment, if skein's `main`
  moved past what smith 01 pinned; otherwise it lands nothing.
- **`jig-repin`:** after `smith-repin`.
- **`temper-repin`:** after `jig-repin`. See "Repin" below.

### How many at once, and in what order

- **At the start:** skein 01, 02 and 03, and jig 01 (alone among jig's
  sessions): four sessions.
- **After jig 01:** jig 03, 05, 06, 07, 09, 10 and 11 at once; jig 02 once
  smith 01 has landed. The machine's slots bound it (development.md, 7.2
  and 11.4).
- **When slots are short, start first** the sessions others wait on, in
  this order:
  1. jig 01, skein 01, skein 03, skein 02; then smith 01;
  2. jig 05, jig 11, jig 02;
  3. jig 13, jig 06, jig 04, jig 08, jig 10, jig 07, jig 03, jig 09;
  4. jig 14, jig 19, jig 12, jig 22, jig 30;
  5. jig 15, jig 20, jig 21, jig 23, jig 24, jig 31, jig 32, jig 34, jig 35;
  6. jig 16, jig 17, jig 25, jig 27;
  7. jig 18, jig 33;
  8. jig 26; then the repins.
- **The longest chains:** jig 01 → 11 → 13 → 14 → 19 → 15 → 16 → 18 →
  27 → 26; skein 01 → smith 01 → jig 02 → 17 → 18; jig 05 → 22 → 23 → 25
  → 33 → 26.

## Rules of this plan

- **jig 01 runs alone.** No other jig session starts before it has
  landed whole: it renames the crates every jig session touches.
- **Every session reads,** besides its "Read first": its repository's
  `AGENTS.md` and workflow, this README and `code-organization.md`.
- **A durable record's codec follows its record.** Once jig 14 or jig 19 has
  landed a family's codec, an increment of any session that changes a
  durable record of that family changes its schema, its generated codec
  and its translation in the same increment; the translation's build
  breaks otherwise, which is the check. Aim to move the family's version
  at the first such change in this plan (development.md, 5.1: formats
  are pre-release); nothing checks it, and the supervisor's hourly check
  looks at schema diffs for it.
  The rule binds a record or row whose translation has landed. A record,
  field or row of a family the codec does not translate yet joins the
  facade's arm that refuses the untranslated rows (one line), and the
  family's later codec increment translates it as `main` has it
  (decisions.md, ruling 134); the version does not move for it.
- **No browser, no wasm32, no web shell,** in any session: no
  session adds one, builds for wasm32 or starts a browser itself. Editing
  skein's existing browser kit and tests where an increment's Files name
  them (skein 01.1, 02.1) adds none; skein's gate runs its browser suite
  as it does today. Checked by "Done when" item 7.
- **Line counts** exclude generated code under `src/generated/` and its
  golden files.
- **Shared hotspots with smith's reliability plan,** if it runs at the
  same time on another machine: skein's `crates/skein-io/src/kernel.rs`,
  `crates/skein-io/src/file*.rs` and `crates/skein-shell/` (its terminal
  modes, skein 02.6 of that plan). `land` rebases each landing on the
  other's; no session waits for it.
- **The design's crate names follow jig 01.** Once jig 01 has landed
  whole, the supervisor wakes the designer (`plan wake designer`) to
  rename the crates in jig's design documents (`docs/design/README.md`,
  section 15's table), a Markdown-only landing; no session waits for it.
- **A component's limits join the profiles.** Once `jig-protocol-config`'s
  profiles exist (jig 20.2), the increment that makes or changes a
  component's limits adds them to the profiles in the same increment, so
  the reference service's startup finds them. A component that landed
  before jig 20.2 has its limits added by the increment of jig 25 or 27
  that takes it (jig 25's Choices, 25.4, 27.3).
- **The process proof's script** is written once, by jig 26, as
  `scripts/proof-one-process.sh` in jig, and later increments that prove
  at the process layer run it.
- **An expected value a renamed vocabulary changes** (decisions.md,
  ruling 108). An increment whose Shape changes how a value prints (a
  renamed or re-wrapped variant in a trace or `Debug` text, a moved tag)
  updates, outside its Files, the expected literals that print it: each
  replaced by what the new code prints for the same behaviour, at most 10
  lines in all, named in its commit body. It never renames, deletes or
  loosens a test or an assertion. This replaces development.md, 8.2's "it
  changes no test's expected value or name" for these literals only; any
  other change outside Files stays a question.
- **A new variant is matched where the build needs it** (decisions.md,
  ruling 112). An increment that adds a variant adds, in the same
  increment, the arm each exhaustive match outside its Files then needs,
  in any session's crate: written as its neighbours are, at most 5 lines
  per match, named in its commit body. It is no question. Tests for those
  arms only where the increment's Test shape names them.
- **A small fix outside Files that the build or the gate needs is no
  question** (decisions.md, ruling 114). An edit outside the increment's
  Files is an implementation choice, named in its commit body, when the
  increment's change, its Proof or the gate needs it, it is at most 20
  lines, and it keeps every test's name and every assertion's expected
  value; a fixture's or a test's inputs may change so that the test
  checks the same behaviour under the new code. It may be in any
  session's files, shared test files and the test roots included. This
  replaces development.md, 8.2's "it is in your session's own files" for
  this plan; expected values change only as ruling 108 says. Anything
  else outside Files stays a question.
- **An expected value for the case the Shape changes** (decisions.md,
  ruling 122). Where an increment's Shape changes what a case answers,
  a test outside its Files that sets up that very case and expects the
  old answer is updated to the new one: its inputs and name kept, at
  most 4 assertions in all, each named in the commit body with the
  Shape's words it follows. It never loosens an assertion (no
  `matches!` in place of an equality) and never removes one.
- **A removed mechanism's tests are its increment's** (decisions.md,
  ruling 123). Where an increment's Shape removes a mechanism, its tests
  in any file are the increment's, as development.md, 8.2 says
  ("Removing a mechanism removes all of it ... tests and docs"): each
  moved to the replacement's behaviour, name kept, where the Shape names
  a replacement; removed otherwise; each named in the commit body. The
  limits of rulings 108, 114 and 122 do not bind them; no other test
  changes under this rule.
- **Test ceilings bind the tests an increment writes** (decisions.md,
  ruling 124): those it adds, and those whose inputs or setup it changes.
  A test changed only in an expected value (rulings 108, 122, 123) is not
  made cheaper by that increment, whatever one run times it at.
- **A private field is an implementation choice** (decisions.md, ruling
  125). A field an increment adds to a type other sessions use, private to
  its crate and reached by no other session's code, does not change the
  fields others see (development.md, 8.2): it is no question. It is
  bounded by a limit, priced in its owner's `worst_case`, and named in the
  commit body.
- **Files follow the code** (decisions.md, ruling 129). Where code an
  increment's Files or Shape name has moved to another file since the
  plan was written (decision 44: "landed code wins where they differ"),
  the file it now lives in is part of the increment's Files, for that
  code. It is no question; the commit body names the move.
- **A new module beside a file past 800 lines is inside Files**
  (decisions.md, ruling 116). Where an increment's Files name a file past
  800 lines, the new module that code-organization.md, section 7 sends its
  new code to ("new code goes in new modules"), beside that file or under
  it, is part of the increment's Files, at the increment's Size. It is no
  question.
- **A codec increment lands past its Size, up to twice it**
  (decisions.md, rulings 132, 151, 188 and 214). In jig 14, jig 18, jig 19, jig 20 and jig 30, an increment whose
  production code is only the schema, the facade and the translations of
  the types and rows its Scope names lands without asking up to twice its
  "Size: at most N lines", and its report gives the count by file. This
  replaces code-organization.md, section 7's "past N × 1.2, it asks
  before landing" for those increments only; past 2 × N, it still asks.
  Each file stays under 800 lines, and each test function under 40.
- **A generated enum's large variant is marked `boxed`** (decisions.md,
  ruling 138; skein's `codec.md`, 3). Once `jig-repin.s1` has landed, a
  codec increment whose generated code clippy's `large_enum_variant`
  reports marks that variant `boxed` in its schema, regenerates, and
  boxes and unboxes it in its translation. It marks no other variant, and
  adds no allowance to a generated file. Before `jig-repin.s1`, such an
  increment commits on its branch and waits for it.
- **A gate that fails only on a timeout is retried twice** (decisions.md,
  ruling 140). When `land`'s gate fails only on a suite's global timeout
  or a test's wall-clock bound, in tests outside the session's files, the
  session runs `land` again, up to three attempts in all on the same
  tree, without asking. A third such failure is a block, kind
  infrastructure. This replaces development.md, 8.5's "a second failure
  on the same tree is a question" for this plan; any other failure is as
  8.5 says.
- **A gate failure the Shape explains is fixed without asking**
  (decisions.md, ruling 145). When the gate fails, other than on a
  timeout, in a test outside the increment's Files because the
  behaviour the Shape adds reaches that test's fixture or story (a new
  control pressed, a message now judged again), the session fixes the
  fixture or the story's inputs under ruling 114 (at most 20 lines,
  every test's name and expected value kept), names in its commit body
  the Shape's words that explain it, and runs `land` again. This
  replaces development.md, 8.5's "stop on a question" for that case
  only; a failure the Shape does not explain, or a fix past ruling
  114's limits, is still a question.
- **A codec checks fields and counts; sums are the domain's**
  (decisions.md, rulings 135 and 154). In jig 14, 19 and 30, a codec's
  `limits` maps the domain's limits to the generated `Limits`, each
  field's ceiling the domain limit that bounds it (a sum where only a sum
  does; the largest of their bounds where several kinds of value share the
  field, ruling 156); its translations check no sum. No family adds a wrapper
  `Limits` type; the domain checks its sums at admission and restore.
- **A generated test's ceiling is 200 ms** (decisions.md, ruling 165). A
  test under `src/generated/`, which no session edits, lands when it runs
  within 200 ms; past that it is a question. Hand-written tests keep
  their increments' ceilings.
- **Until the suite is cheaper, a timed-out landing waits** (decisions.md,
  ruling 174). Until jig-repin.s3 has landed, a session whose three gates
  fail only on the focused suite's timeout keeps its commit and ends its
  turn; the coordinator runs its `land` again (three attempts) after each
  test-cost step of `tests-cost.md` lands. No block is written for it.
- **The client wire's translations land at their count** (decisions.md,
  ruling 194). In jig 31, 32, 34 and 35, an increment whose production
  code is only translations between the client contract's types and the
  wire's lands without asking whatever its Size, its report giving the
  count by file; each file under 800 lines, each test function under 40.
  Code that is not such a translation keeps the increment's Size.
- **The fixtures a Test shape names are inside Files** (decisions.md,
  ruling 195): in the crate's shared fixture module or a child of it (a
  new child where the root would pass 800 lines), with the root's
  re-exports; no question, and no landed test changed by it.
- **An item made visible within its crate is no question** (decisions.md,
  ruling 197): widening an item's visibility within its own crate
  (`pub(crate)`, `pub(super)`, a module root's re-export) so that a sibling
  module reuses it, in any session's files, named in the commit body, when
  it changes no behaviour, signature or test. Public API across crates
  stays a question.
- **A session may replace what its own earlier increments landed**
  (decisions.md, ruling 198): an increment that replaces its own session's
  mechanism edits and removes it in that session's files, past ruling
  114's 20 lines, every test's name and expected value kept unless the
  Shape changes the behaviour, named in the commit body. Another
  session's files stay under rulings 114 and 145.
- **Moving an unchanged test body counts for nothing** (decisions.md,
  ruling 209): moving a test's unchanged statements into a named helper of
  the same file, to keep a function within 40 lines, does not count toward
  ruling 114's 20 lines; changed inputs and expected values still do.
- **The vocabulary guard excepts a foreign name exactly where jig must
  write it** (decisions.md, rulings 199 and 216): an allowance in
  `tests/boundary`'s guard for a name another crate defines (smith, skein,
  the standard library), fully qualified, naming the exact path, with a
  regression that jig's own use of the word is still refused; no question.
  jig's own names stay refused.
- **A value the Shape makes invalid is replaced wherever a test builds it**
  (decisions.md, ruling 225). Where an increment's Shape makes a
  constructor refuse a value (03.s2: a `Text` of bytes that are not
  UTF-8), each test in any file that builds such a value is the
  increment's: the value becomes a valid one in the same role (distinct
  where it was distinct, as long where its length matters), one constant
  per file rather than repeated literals; an expected value that echoes
  that input changes with it; every name and every other expected value
  kept, no assertion loosened or removed. It does not count toward
  rulings 114's or 122's limits; the commit body names the files. A test
  whose subject is the refused value itself is a removed mechanism's,
  under ruling 123.
- **A fixture gains what the real component requires** (decisions.md,
  ruling 227). Where a world moves to a real component that requires
  something the fake did not (the real store's deployment header), the
  fixture supplies it, and an assertion that sees exactly that addition
  becomes an exact equality with it, built from the fixture's own
  constant, never read back from the component; name and every other
  expected value kept, each named in the commit body. Any other failure
  on the real component is a finding, as the step's Don't says.
- **An expectation the fake made against the design follows the design**
  (decisions.md, ruling 230). Where a test moving to a real component
  expects what the fake did and the design says otherwise, its expected
  values become the design's, exact, quoting the design's section in the
  commit body; name and every other expected value kept. When the design
  is silent or the real component disagrees with it, it stays a finding.
- **A test fixture on the real store asks the journal its limits admit**
  (decisions.md, rulings 229, 231 and 235). A fixture that moves onto a
  real store derives its journal from its final limits through one named
  helper (at least 2,000 writes, `held` likewise) and may shrink its
  bounds to test-sized values its tests still exercise, each changed
  bound named old and new in the commit body; every test's name and
  expected value kept, no question. Default and production limits stay,
  and no adapter caps what it is given. A test whose case needs a bound
  the real store cannot admit is a finding.
- **A new test costs the suite about 12 ms before its work** (decisions.md,
  ruling 219; `tests-cost.md`, 12). In every step left, tests are one per
  behaviour, a behaviour's cases rows of one table test; fixtures are
  constants or built once in a test; limits are test-sized; the Proof
  names the step's tests' count. jig 34 and 35 take each document's wire
  record from one kit under `testing/`, each side keeping its own literal
  of its own value.

## Live services

- **An LLM provider, for the live proofs only:** the user's ChatGPT login
  at `~/.codex/auth.json`, borrowed read only and access token only. A
  live proof's script copies the access token and its expiry into a
  private file in the run's `logs/` directory, which `jig-test`'s
  configuration names as a handed-in account (jig's `protocol/llm.md`,
  5.4). No agent signs in, and nothing writes to `~/.codex`.
- **Its budget:** at most 5 completions and 5 minutes per live proof, at
  most 40 completions in the whole run. Only jig 26's increments have
  live proofs.
- **smith's `smoke-tools`,** for smith 01 and `smith-repin` only, as
  smith's `docs/development/workflow.md`, section 5 requires of every
  smith landing that changes code: the cheapest model and effort, once
  before `land`, once more only if it fails; about 10,000 tokens a run,
  borrowing the same login as smith's benchmarks do.
- **Nothing else is live:** skein's fake LLM peer serves every other
  proof.

## Repin

Every repository downstream of what the plan changes is repinned
(development.md, 5.7): smith by smith 01 and `smith-repin`, jig by jig 02
and `jig-repin`, temper by `temper-repin`. temper pins jig `2860e63`, far
behind; its repin is expected to need more than a repin may change (at
most about 200 lines, no change of design). If it does, the step lands
nothing, and the gap goes to temper's `docs/design/STATUS.md`, under
"Not planned", and to this plan's `later.md`, as section 5.7 says.

### jig-repin.s1: jig's `skein-codegen` pin alone

A step not in the plan (decisions.md, ruling 138), after skein 02.s1.
jig's lock holds `skein-codegen` at its own revision (`30dc6f7d`), apart
from skein's other packages (`90326012`).
Scope: `Cargo.lock` moves the skein source to skein 02.s1's commit:
  `skein-codegen` and the ten skein packages `cargo update` moves with it
  (ruling 147).
Shape: `heavy cargo update -p skein-codegen --precise <skein 02.s1's
  commit>`; `git diff origin/main -- Cargo.lock` changes skein entries
  only, no smith entry. Where a drift test
  (`tests/store-codecs/tests/drift.rs`) then reports generated code
  changed, the files are written again by the drift test's own
  environment variable; their golden bytes do not move.
Proof: gate, `scripts/check.sh`, run by `land`.
Don't: move a smith package; edit `Cargo.lock`, a schema or a generated
  file by hand. If `cargo update` moves a smith package, stop and
  report.
Size: nothing besides `Cargo.lock` and generated files.

### jig-repin.s2: the skein source again, after skein 02.s2

As jig-repin.s1, with skein 02.s3's commit, which follows 02.s2's
(rulings 152, 157).

## Pulse

```
words: legacy|compat|hack|workaround
types: ^(Legacy|Old|Compat)[A-Z]
file-lines: 800
budgets: 15 60
```

## Done when

1. Every step is landed or dropped: `plan status` lists none waiting,
   ready, running or blocked.
2. jig's gate passes on `main`: `git -C ~/src/rust/jig log -1
   --format='%(trailers:key=Gate-Result,valueonly)' origin/main -- . ':!docs'`
   (the last landed commit outside `docs/`) names `scripts/check.sh passed`.
3. The crates carry their new names: `git -C ~/src/rust/jig grep -n -E
   '"(jig-core|jig-host|jig-charter|jig-client|jig-client-view|jig-client-transcript)"'
   -- '*.toml'` prints nothing.
4. jig's own worlds run on the real store: `git -C ~/src/rust/jig grep -n
   jig-fake-store -- tests/core tests/conformance` prints nothing.
5. The process proof passes once: `cd ~/src/rust/jig && heavy --gate
   scripts/proof-one-process.sh` exits 0 (jig 26 records its output in the
   run's logs).
6. The live proof ran once within its budget: `ls
   ~/src/rust/tmp/jig/jig-protocol/logs/jig-26/proof-live.log` lists the
   log, and `grep -n -i completions
   ~/src/rust/tmp/jig/jig-protocol/sessions/jig-26.md` prints the line of
   jig 26's checkpoint that names the completions spent, at most 5.
7. No browser or wasm32 entered jig: `git -C ~/src/rust/jig grep -n -i -E
   'wasm32|wasm-bindgen|jig-shell-web' -- . ':!docs'` prints nothing.
8. The aims stated above are these items; none other is checked.
