# Session 02: jig on skein's and smith's new revisions

jig moves its skein and smith pins to their `origin/main`, once smith 01
has moved smith onto skein's `main` with this plan's skein 01 to 03, and
repairs what the move breaks. It is development.md, 5.7's repin step,
run as a session of this plan so that the sessions that use skein's new
pieces start from it: jig 17 (`OpenWrite`, `Cut`), jig 20
(`read_private_file`), jig 22 (`secret N`), jig 31 (`CryptoRng`,
`Secret`, `base64`). One increment.

## Read first

- skein's `development.md`, 5.7 (the step, what a repin may change, a
  conflict in `Cargo.lock`) and 9.5 (pins across repositories).
- jig's `AGENTS.md` and `docs/development/workflow.md`, section 1.
- smith 01's report (the skein revision it pinned) and smith's commits
  since jig's pin: `git -C ~/src/rust/smith log --oneline
  2a800ea2..origin/main -- crates/smith-charter crates/smith-transcript
  crates/smith-domain-run`.
- `findings/skein.md`, "Does jig use these types directly?";
  `findings/protocol.md`, 3.7 and 4.0 (what jig pins and links).

## What the design asks

Nothing of jig's design changes. jig's lock names one revision of
skein, for itself and for the smith crates it compiles
(development.md, 5.7: "jig moves smith and skein together: it resolves
one revision of each, for itself and for smith's crates").

## Choices this plan makes

- **The targets are skein's and smith's `origin/main`** when the session
  starts; the commit names both revisions.
- **No new workspace dependency:** jig names `skein-lib`, `skein-json`,
  `skein-world`, `smith-domain-run`, `smith-charter` and
  `smith-transcript` today (`Cargo.toml:67-72`), and keeps them. Each
  later session adds the skein or smith crate it uses to
  `[workspace.dependencies]`, by the same GitHub URL, so the lock keeps
  one revision of each source.
- **smith's formats under their versions.** smith's reliability plan may
  change charter 2 or transcript 3 under the same version (its README,
  rule 5). If one did, jig's copies follow it as part of this repin: the
  goldens `crates/jig-protocol-transcript/tests/data/record_turn_{smallest,full}.bin`
  copied again from smith's `crates/smith-transcript/golden/v3/` at the
  pinned revision, and `jig-protocol-charter`'s and `ops`'s uses of
  `smith_charter::v2` adapted the way smith adapted its own callers.

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
- "**jig 01 runs alone.** No other jig session starts before it has
  landed whole" (README, "Rules"): this session starts after it, and
  works in the renamed crates.

## Increments

### 02.1 The repin
Scope: jig's `Cargo.lock` moves the skein and smith git sources to
  their `origin/main`; jig builds and its gate passes on them.
Needs: jig 01 (the renamed crates); smith 01's repin of smith onto
  skein's `main` with skein 01 to 03, on smith's `origin/main`.
Files: `Cargo.lock`; repairs, if any, where jig names smith's or
  skein's code: `crates/jig-protocol-charter` (`smith-charter` v2,
  `smith-domain-run`), `crates/jig-protocol-transcript` (`smith-transcript`
  v3, its `tests/data/*.bin`), `examples/ops/domain` and `tests/ops`
  (`smith-charter`, `smith-transcript`), `tests/client`
  (`smith-transcript`). Shared hotspot: `Cargo.lock`, which each later
  session that adds a dependency also touches.
Shape: in jig's worktree for this session:
  1. `git fetch origin && git switch -c jig-protocol/jig-02-1
     origin/main`;
  2. `heavy cargo update -p skein-lib -p smith-charter`, which moves the
     skein and the smith git sources, each with its packages;
  3. `heavy cargo check --workspace --all-targets`, and each error
     repaired within the Rules' limits;
  4. `heavy cargo clippy --workspace --all-targets -- -D warnings`, once;
  5. `land`.
Test shape: no test is added. A test changes only where smith's format
  under its version moved (Choices): its golden bytes copied again, its
  expected values those smith's own tests now hold.
Proof: the gate's (development.md, 5.7: "Its `Proof:` line is the
  gate's"): jig's `scripts/check.sh`, run by `land`; the commit says
  `Proof: gate, scripts/check.sh on skein <revision>, smith <revision>`.
  Nothing else is run.
Don't: add a workspace dependency; change jig's design or behaviour;
  edit `Cargo.lock` by hand; edit skein or smith.
Size: at most 200 lines besides `Cargo.lock`; most likely none.
Removes: nothing.

## Interface sketches

None: no jig type changes.

## Questions to raise when reached

- **smith's charter or transcript changed under its version** in a way
  jig's design does not cover (a field jig must now read or fill): the
  step lands nothing, and the gap goes to jig's `docs/design/STATUS.md`,
  "Not planned", and this plan's `later.md` (development.md, 5.7).
- **Two skein revisions:** if `cargo update` leaves smith's crates on a
  skein revision other than jig's (a second git source for one URL), it
  is a question, not a lock edit.

## Done when

1. jig's lock names one skein revision and one smith revision, each its
   `origin/main` of the session's start: `git -C ~/src/rust/jig show
   origin/main:Cargo.lock | grep -o 'forgetic/skein.git#[0-9a-f]*' |
   sort -u` prints one line, and the same for `forgetic/smith.git#`.
2. That skein holds this plan's skein work: `git -C ~/src/rust/skein
   merge-base --is-ancestor <skein-03.2's commit> <jig's skein
   revision>` exits 0, as it does for skein-01.4's and skein-02.5's
   commits.
3. The gate passed: `git -C ~/src/rust/jig log -1
   --format='%(trailers:key=Gate-Result,valueonly)' <the session's
   commit>` names `scripts/check.sh passed`.
