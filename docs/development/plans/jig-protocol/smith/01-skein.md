# Session 01: smith on skein's main, with skein 01 to 03

smith moves its skein pin to skein's `origin/main` once this plan's
skein 01, 02 and 03 have landed, each increment of them, and repairs
what they break: io's address records (`Addr::Ip` at its io sites),
`io::Limits::names` in its literals, and the new variants in its
exhaustive matches (the kernel's `Truncate` and `Write` opens, the
simulator's `Ask::Truncate`, io's `OpenWrite`, `OpenWriteNoFollow` and
`Cut`, and `Cut`'s event).
Nothing of smith's design changes and smith adopts none of skein's new
pieces. jig 02 then takes smith's `main`, whose crates jig compiles
against the same skein. One increment: a lock that moves without its
repairs does not build.

## Read first

- smith's `AGENTS.md`, and `docs/development/workflow.md`, sections 1
  (the gate) and 5 (the smoke on each code landing).
- skein's `development.md`, 5.7 (what a repin may change, and its
  limits): this session is that step, run as a session of this plan.
- skein's session files `skein/01-addresses.md`, `skein/02-startup-and-secrets.md`
  and `skein/03-files.md` in this plan: their Shapes say what moved.
- `findings/skein.md`: "smith's repin onto skein with items 1–6", and
  the smith users in sections 1 and 6.
- This plan's `decisions.md`, 3 and 4.

## What the design asks

Nothing of smith's design changes. smith builds on skein's `main`,
whose io addresses are `Addr::{Ip, InProcess}` (skein's `io.md`, 2 and
4), whose kernel opens existing files to write and cuts them (skein's
`kernel.md`, 6.1), and whose LLM and OAuth endpoints keep socket
addresses (decision 4).

## Choices this plan makes

- **The target is skein's `origin/main`** when the session starts, which
  holds skein 01 to 03 and whatever else has landed; the commit names its
  revision.
- **`names: 0`** in each `io::Limits` literal: smith listens on no
  in-process name.
- **`skein_io::Addr::Ip(..)`** where smith puts a socket address into
  io's `Listen` or `Connect`. `skein_llm_connection::Endpoint.address`
  and `skein_oauth_accounts::Endpoint.address` stay `SocketAddr`
  (decision 4), so smith's endpoint constructions take no edit.
- **A new match arm joins the arm that already handles its kind:**
  `kernel::Op::Truncate` beside `Op::Sync` and `Op::Write`;
  `OpenHow::{Write, WriteNoFollow}` beside `Read` and `ReadNoFollow`
  where a match lists opens; `Ask::Truncate` handled as the match
  handles `Ask::Write` (forwarded to the fake machine where `Write` is);
  `file::Request::{OpenWrite, OpenWriteNoFollow, Cut}` and
  `file::Event::Cut` beside the variants the match already treats as
  smith never issuing them. No catch-all arm.
- **smith adopts nothing new here:** `CryptoRng`, `crypto_seed`,
  `read_private_file`, `Secret`, `base64`, `OpenWrite` and `Cut` are for
  smith's own plans.
- **If smith's `main` already pins a skein with some of this** (smith's
  reliability plan repins smith on skein's `main` for its terminal
  modes, smith 10.10), this session does what is left, or lands nothing,
  and says so in its report.
- **If smith's `main` by then calls `skein_oauth_accounts::Component::new`**
  (the reliability plan's smith 10.6 adopts skein's OAuth driver), its
  `u64` seed becomes `CryptoRng::new(&seed, 0)`: in a shell, `seed` from
  `skein_shell::crypto_seed()` at startup, its error handled as the
  shell handles `seed()`'s; in a world or a test,
  `skein_world::crypto_seed(<the test's seed>)`. This is the edit skein
  02.4 made to its own callers.

## Rules that bind it

- "**What a repin may change:** the pins; items an upstream trait now
  requires; names and paths that moved; the downstream tests that an
  upstream change of behaviour breaks, adapted the way the upstream
  commit adapted its own (its body names the rule). At most about 200
  lines besides `Cargo.lock`, and no change of design. A break that
  needs more, or a decision, is a question: the step lands nothing, and
  the gap goes to the downstream repository's `STATUS.md`, under "Not
  planned", and to the plan's `later.md`" (development.md, 5.7).
- "**A conflict in `Cargo.lock`** is never merged by hand: take
  `origin/main`'s, and run the step's `cargo update` again"
  (development.md, 5.7).
- "**Every smith landing that changes code runs `smoke-tools` live,**
  with the cheapest model and effort, whatever the change touched: once,
  through `heavy --gate`, on the branch rebased on `origin/main` just
  before `land`, and once more only if it fails" (smith's
  `workflow.md`, section 5). This plan's README lists no live service for
  smith 01; smith's workflow rules here, and the report names the smoke's
  completions.

## Increments

### 01.1 The repin
Scope: smith's `Cargo.lock` moves each skein package to skein's
  `origin/main`; the edits that skein 01 to 03 make necessary; nothing
  else.
Needs: skein 01's `Addr` and `Limits::names`, skein 02's `CryptoRng`
  and `Component::new`, skein 03's `Write` open, `Truncate`,
  `Ask::Truncate`, `OpenWrite` and `Cut`: skein 01, 02 and 03 landed,
  each increment of them, on skein's `origin/main`.
Files: `Cargo.lock`, and the sites findings/skein.md counted, at smith
  `2a800ea2` (smith's `main` moves under its reliability plan; `heavy
  cargo check --workspace --all-targets` lists the sites at the
  session's start, and the list below is where to look first):
  - io's address records (4): `crates/smith-local-shell/src/local_auth.rs:214`
    (`Listen`), `crates/smith-local-shell/src/local_auth_http.rs:112`
    (`Connect`), `tests/local-process/src/oauth.rs:50,213` (`Connect`).
    The endpoint constructions `crates/smith-agent-shell/src/config.rs:420`,
    `crates/smith-local-shell/src/local_auth.rs:123`,
    `local_auth_http.rs:15,372`, `tests/agent-process/src/lib.rs:189`,
    `tests/local-process/src/process.rs:200`,
    `tests/protocol-llm/src/pool_peer.rs:98`,
    `tests/protocol-llm/tests/component_boundary.rs:88`,
    `component_peer.rs:131` and `endpoints_grants.rs:26` take no edit
    (decision 4), nor `crates/smith-agent-service/src/service.rs:1143-1144,1928-1930`,
    which pass io's values through.
  - `io::Limits` literals (10): `crates/smith-agent-service/src/profile.rs:254`
    (production), `crates/smith-agent-service/src/service.rs:2251`,
    `crates/smith-local-service/src/tests.rs:38`, `tests/hosts/src/lib.rs:120`,
    `tests/local-process/src/oauth.rs:287`, `tests/local-process/src/terminal.rs:76`,
    `tests/machine/src/lib.rs:42`, `tests/protocol-llm/src/pool_peer.rs:34`,
    `tests/protocol-llm/tests/component_boundary.rs:36`,
    `tests/protocol-llm/tests/component_peer.rs:38`.
  - Exhaustive matches (8): `kernel::Op` in `benchmarks/src/process.rs:331-350`,
    `crates/smith-local-shell/src/local_host.rs:465-490`,
    `tests/agent-process/src/world.rs:190-210`; `Ask` in
    `tests/machine/src/lib.rs:422-433`; `file::Event` in
    `crates/smith-local-protocol/src/markers.rs:105-115`,
    `crates/smith-local-protocol/src/plain.rs:122-133`,
    `crates/smith-protocol-machine/src/files.rs:129-136`; `file::Request`
    in `crates/smith-protocol-machine/src/tests.rs:579-594`.
  - Shared hotspot: `Cargo.lock` (smith's reliability plan names it).
Shape: in the worktree `plan worktree smith 01` made:
  1. `git fetch origin && git switch -c jig-protocol/smith-01-1
     origin/main`;
  2. `heavy cargo update -p skein-lib`, which moves the skein git
     source, each skein package with it;
  3. `heavy cargo check --workspace --all-targets`, and each error
     repaired as the Choices say;
  4. `heavy cargo clippy --workspace --all-targets -- -D warnings`, once;
  5. the smoke, as the Rules quote smith's workflow; then `land`.
Test shape: no test is added or changed but where a repaired match sits
  in a test. smith's codec drift tests (`tests/codecs/tests/drift.rs`)
  run unchanged in the gate: they show skein's generator still writes
  smith's code byte for byte (no smith schema has a `secret` field).
Proof: the gate's (development.md, 5.7: "Its `Proof:` line is the
  gate's"): smith's `scripts/check.sh`, run by `land`; the commit says
  `Proof: gate, scripts/check.sh on skein <revision>`. Evidence: the
  smoke's summary line, or, while smith 00.27 has not landed, "smoke:
  not run, before smith-00.27" (smith's reliability plan, README rule 1).
  Nothing else is run.
Don't: change any behaviour of smith's or any expected value of a test;
  use a new skein piece; edit skein; edit `Cargo.lock` by hand; add a
  catch-all arm.
Size: at most about 80 lines besides `Cargo.lock` (findings/skein.md
  estimates 60 to 80); past 200, it is a question (Rules).
Removes: nothing.

## Interface sketches

None: no smith type changes. skein's types are in the skein session
files.

## Questions to raise when reached

- **A break the Rules' 200 lines cannot hold,** or one that needs a
  decision (a skein behaviour smith's design relies on): the step lands
  nothing; the gap goes to smith's `docs/design/STATUS.md`, "Not
  planned", and to this plan's `later.md` (development.md, 5.7).
- **smith's channel schema carries a credential as `bytes`**
  (`crates/smith-channel/schema/channel-v2.schema:16`, `GrantValue {
  credential: bytes 8192 }`), where skein's `codec.md`, 3 now says "A
  credential is a `secret N`, never `bytes N`". Changing it changes
  smith's channel format: smith's own plan, not this session.

## Done when

1. smith's lock names a skein that holds skein 01 to 03: the revision in
   `git -C ~/src/rust/smith show origin/main:Cargo.lock | grep -A2 'name
   = "skein-io"'` is `<rev>`, and `git -C ~/src/rust/skein merge-base
   --is-ancestor <skein-03.2's commit> <rev>` exits 0, as it does for
   skein-01.4's and skein-02.5's commits.
2. smith's gate passed on it: `git -C ~/src/rust/smith log -1
   --format='%(trailers:key=Gate-Result,valueonly)' <the session's
   commit>` names `scripts/check.sh` as passed.
