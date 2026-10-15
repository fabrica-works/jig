# Session jig 01: the owner made opaque, and the core gathered in `jig_view::ui`

The tree's owner becomes skein-ui's opaque number, with jig's owner
mapped onto it; the provisional state goes; and jig-view's generic core
(`tree`, `builder`, `diff`, the view, `markdown`) moves into one module,
`jig_view::ui`, which every user in jig imports by that path. Nothing a
person sees changes but the text shell's `(provisional)` word, which no
view sets.

## Read first

- skein's `docs/design/ui.md`, sections 3, 3.2 (the paragraph after the
  states' table), 3.3, 5 and 7.
- jig's `docs/design/client/domain/view.md`, sections 2, 3 and 9.
- This plan's `decisions.md`, 5 and 6; `code-organization.md`, 1, 2 and
  6; `findings/audit.md`, 1 and 2.
- skein's `programming-model.md`, section 10 (the subset);
  `testing-strategy.md`, section 8 (the suites).

## What the design asks

The tree's owners are opaque numbers (ui.md, 3.3); jig's owner maps onto
them, jig to 65536 and an application's number to itself, and back by a
partial function (view.md, 3). There is no provisional state (ui.md,
3.2; view.md, 9). The generic core is skein-ui's (view.md, 2); this
session gathers it where the move (skein 01) takes it whole.

## Choices this plan makes

- `jig_view::ui::Owner { pub number: u32 }` is the tree's owner;
  `jig_view::Owner` (jig's enum, unchanged) gains `opaque` and
  `from_opaque` (decisions.md, 6). Call sites write
  `Owner::Jig.opaque()`; no constant is added beside it.
- The module is `crates/jig-view/src/ui/`, with `mod.rs` re-exporting
  its items as `jig_view`'s `lib.rs` does today; `render.rs` becomes
  `ui/view.rs`.

## Rules that bind it

- **Every user changes in the same increment,** with no alias and no
  re-export of a moved item from its old path (code-organization.md, 6).
- **A move is a move:** `git mv`, then paths; no change of behaviour in
  the same increment.
- **The suites' budgets:** focused 15 s, fuzzy 60 s
  (`docs/development/workflow.md`, 2); this session adds no test time
  past one gate's swing (README, "Test time").

## Increments

### 01.1 The tree's owner is an opaque number
Scope: ui.md 3.3; view.md 3.
Needs: nothing.
Files: `crates/jig-view/src/tree.rs` (`Owner`), a new
  `crates/jig-view/src/owner.rs`, and every user of `jig_view::Owner` in
  the seven crates that depend on jig-view (findings/audit.md: jig-view
  52 files; `testing/jig-test-view`, `tests/client`, `examples/ops`,
  `tests/ops`, `crates/jig-shell-text`, `testing/jig-fake-person`,
  `tests/client-protocol`). Find them with `rg -l
  'Owner::(Jig|Application)' crates/jig-view crates/jig-shell-text
  testing/jig-test-view testing/jig-fake-person tests/client
  tests/client-protocol tests/ops examples/ops`. Shared hotspots:
  `testing/jig-test-view/src/root.rs`, `examples/ops/view/src/root.rs`.
Shape:
  - `tree.rs`: `pub struct Owner { pub number: u32 }` (derives as
    today); every field typed `Owner` (in `Style`, `Region`, `Picture`,
    `Key`, `SlotRef`, `DrawingKind`, `Binding`, `PhraseId`) keeps the
    name `owner` and takes this type.
  - `owner.rs`: `pub enum Owner { Jig, Application { number: u16 } }`
    moved from `tree.rs`, exported as `jig_view::Owner`;
    `pub const fn opaque(self) -> tree::Owner` (Jig → 65536, an
    application's `number` → `u32::from(number)`);
    `pub const fn from_opaque(owner: tree::Owner) -> Option<Owner>`
    (65536 → `Jig`, below 65536 → `Application`, else `None`); both
    `match` whole.
  - jig's views compare and route by `Owner::from_opaque(x)` where they
    matched on the tree's owner.
Test shape: `crates/jig-view/src/tests/owner.rs`, reusing
  `crates/jig-view/src/tests.rs`'s module list. Reference test:
  ```rust
  #[test]
  fn jig_maps_to_one_past_every_application_number() {
      assert_eq!(Owner::Jig.opaque(), ui::Owner { number: 65_536 });
      assert_eq!(Owner::from_opaque(ui::Owner { number: 65_536 }), Some(Owner::Jig));
  }
  ```
  Also: `an_application_number_maps_to_itself_and_back`,
  `a_number_past_jig_names_no_owner`. Each under 20 ms.
Proof: world. `heavy cargo nextest run -p jig-client-world -E
  'test(/stories/)'` passes once: the client's world builds every jig
  page and routes every binding through the opaque owner. Bound: 2
  minutes.
Don't: add `JIG_OWNER` or any constant beside `Owner::Jig.opaque()`; add
  a `From` impl.
Size: about 60 lines of production code, plus the mechanical change at
  each call site (about 250 lines).
Removes: `Owner`'s variants from the tree.

### 01.2 No provisional state
Scope: ui.md 3.2; view.md 9 and 15.3.
Needs: nothing.
Files: `crates/jig-view/src/tree.rs:139` and the states' capacity
  (`tree.rs:288`, `builder.rs:161`, `limits.rs:32`, which count nine
  states and keep nine: ui.md 3.2 has nine with `required`, which skein
  02 adds); `crates/jig-shell-text/src/render.rs:306, 323`;
  `crates/jig-shell-text/src/tests/fixtures.rs:187`;
  `crates/jig-shell-text/tests/golden/roles.txt:74`.
Shape: `State::Provisional` removed; the text shell's state list loses
  its word. No view sets it (`rg -n 'State::Provisional' --type rust`
  at the base prints only those lines); the turns' provisional text is
  already a status in words (`turns.rs:56-57, 83-84`).
Test shape: the golden `roles.txt` line 74 loses ` (provisional)`,
  edited by hand; no new test.
Proof: unit, `heavy cargo nextest run -p jig-shell-text`: the golden
  page prints every state but provisional. The highest layer for a
  removed variant that nothing sets.
Don't: change the capacity of eight or nine states (skein 02 adds
  `Required`).
Size: about 10 lines.
Removes: `State::Provisional`.

### 01.3 The core gathered in `jig_view::ui`
Scope: view.md 2; decisions.md 5.
Needs: 01.1 (the opaque owner in `tree.rs`).
Files: `git mv` of `crates/jig-view/src/{tree,builder,diff,markdown}.rs`
  into `crates/jig-view/src/ui/`, and `render.rs` to `ui/view.rs`; a new
  `ui/mod.rs`; `crates/jig-view/src/lib.rs`; the test files that test
  only these (findings/audit.md: `tests/slots.rs`, `tests/markdown.rs`
  with `tests/markdown/*.md`, `tests/tree.rs`, `tests/diff.rs`) moved to
  `ui/tests/`, with `ui/tests/fixtures.rs` holding `LIMITS` (from
  `tests/tree.rs:16`); and every `use jig_view::{…}` of these items in
  the seven crates (67 files at the base, findings/audit.md) rewritten
  to `jig_view::ui::{…}`.
Shape: `pub mod ui;` in `lib.rs`, whose root re-exports the items
  `lib.rs` re-exports today from `tree`, `builder`, `diff` and `render`
  (`Tree`, `Node`, `NodeId`, `Role`, `State`, `Owner` (the opaque one),
  `Style`, `Region`, `Picture`, `Shortcut`, `Modifier`, `Run`,
  `TextRun`, `Value`, `Key`, `SlotRef`, `DrawingKind`, `Drawing`,
  `Builder`, `Slot`, `SlotId`, `Patch`, `Attribute`, `Made`, `diff`,
  `View`, `markdown`). `lib.rs` no longer re-exports them. Inside
  `ui/`, `crate::X` for a moved item reads `crate::ui::X`; `binding`
  and `limits` stay outside until jig 02. Doc comments in `ui/` cite
  skein's `ui.md` where they cite `client/domain/view.md`, and say
  "owner" where they say "domain" (`tree.rs:199`, `builder.rs:214`,
  `diff.rs:86`, `render.rs:4, 36`).
Test shape: the moved tests, unchanged but for paths; no new test.
Proof: world. The client's world and `ops`'s pass once: `heavy cargo
  nextest run -p jig-client-world -p jig-ops-world`. Bound: 3 minutes.
Don't: change a moved file's code beyond paths and doc comments.
Size: about 40 lines of production code, plus paths.
Removes: `jig_view::{Tree, …}` at the crate's root.

## Questions to raise when reached

None known.

## Done when

`rg -n 'Owner::(Jig|Application)' crates/jig-view/src/ui` prints
nothing; `rg -n 'Provisional' --type rust crates testing tests examples`
prints only words in turns' phrases; `ls crates/jig-view/src/ui` lists
`tree.rs builder.rs diff.rs view.rs markdown.rs mod.rs tests`.
