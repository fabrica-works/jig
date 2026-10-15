# Session jig 05: jig on skein-ui's last landing

jig repins skein onto the last landing of this plan, and repairs what
skein's additions break in jig: the new limits' fields, the new
variants where jig matches them, the act refusal and the outline's
Escape rule. jig's gate stays green; nothing a person sees in jig
changes.

## Read first

- skein's `docs/foundation/development.md`, section 5.7 (what a repin
  may change).
- skein's `docs/design/ui.md`, sections 6 (the refusals), 14 (the
  outline) and 15 (`scene_marks`, `scene_bytes`).
- This plan's `decisions.md`, 3 and 4; the README's "Rules of this
  plan".

## What the design asks

jig's view on skein-ui (jig's `client/domain/view.md`, 2), at skein's
`origin/main` (development.md, 9.5).

## Rules that bind it

- **jig moves skein only** (decisions.md, 4): `heavy cargo update -p
  skein-lib`, never smith; a `cargo check` failing inside a smith crate
  stops the increment with a question.
- **A repin's size:** at most 200 lines besides `Cargo.lock` (`git diff
  --shortstat $(git merge-base HEAD origin/main) -- . ':!Cargo.lock'`:
  the merge base, so that rulings landing meanwhile do not count), and
  no change of design
  (development.md, 5.7). More stops the increment with a question: it
  lands nothing, and the supervisor lands the gap's lines in jig's
  `STATUS.md` under "Not planned" and in `later.md`.
- **Tests adapt as skein's own adapted:** a jig test that an upstream
  change of behaviour breaks changes the way skein's commit changed its
  own, its body naming the rule.

## Increments

### 05.1 Repin skein, and repair
Scope: development.md 5.7.
Needs: every skein increment of this plan on skein's `origin/main`;
  jig 04.3.
Files: `Cargo.lock` (the repin commit, alone, first); then each file
  `heavy cargo check --workspace --all-targets` names. Expected, from
  the searches at the base:
  - each literal of skein-ui's `Limits` in jig, given `scene_marks` and
    `scene_bytes` from jig's own constructors (`rg -n 'markdown_depth'
    --type rust`: 33 literals in 30 files at jig `6c7c2dcf3`, three of
    them in `crates/jig-view/src/` (`limits.rs`, `owner_deployment.rs`,
    `owner_result.rs`), one in `tests/ops/src/joint/fixture.rs`; not
    `Limits \{`, which also matches `PhraseLimits` and jig's own
    `Limits`), with jig's numbers: no scene in jig's pages, so both 0
    (jig draws no scene until later.md's "jig's own drawing code");
    the ready check's count: 66 lines;
  - `Node.picture` read in jig, now `Node.source` (none at the base
    outside the moved crates: `rg -n '\.picture\b' --type rust`);
  - a test acting on a disabled node or with an act its role does not
    take, which skein 03.2's `View::act` now refuses (`rg -n
    '\.act\(' tests/client tests/client-protocol tests/ops --type
    rust`);
  - a page naming Escape outside a dialog (none at the base: `rg -n
    'ESCAPE|0x1b' crates/jig-view examples testing --type rust`).
Shape: no change of code but these repairs.
Test shape: none new; tests adapted as above, each named in the commit
  body with the skein commit whose rule it follows.
Proof: the gate: `heavy --gate scripts/check.sh` in the increment's
  committed worktree, once, through `land`. Bound: 15 minutes.
Don't: move smith; add an arm with `_ =>` over skein-ui's enums.
Size: at most 200 lines besides `Cargo.lock`, by the command above.
Removes: nothing.

## Questions to raise when reached

- A break that needs more than 200 lines or a change of design
  (development.md, 5.7): the increment lands nothing and asks.
- **Known from the ready check (ruling 133):** after the repin and the
  `Limits` repair, the fuzzy profile fails
  `random_people_with_crash_commits_and_reload`
  (`tests/client/tests/fuzzy_joint.rs:65`) on seed 4503, `Online(ClientId(0))
  was not met by 1.211353973s`; it passes at the old pin. The random
  person presses "Sign out", then "Sign in with Test" 1.1 ms later; the
  page ends signed out, while the referee's sign-in receipt armed the
  Online deadline (`tests/client/src/harness/referee.rs:194`). The
  repin brings skein-fake-person's new draws (`83a0cec`, `561d1ae`).
  Reproduce it with that seed, at most 3 runs through `heavy`, find
  which side is wrong (jig's client losing a sign-in pressed while its
  sign-out is in flight, or the referee arming Online for a sign-in the
  page then lost), and raise it as a question with the trace. Do not
  re-seed it, and change neither the referee nor jig's client in
  05.1.

## Done when

`git -C ~/src/jig show origin/main:Cargo.lock | grep -m1 'skein.git#'`
names a commit that `git -C ~/src/skein log origin/main
--grep='^Plan: skein-ui$' --format=%H -1` prints, or a later one.
