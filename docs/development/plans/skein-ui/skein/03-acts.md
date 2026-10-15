# Session skein 03: the look's names, and the act refusal

skein-ui gets the look's names and their check, which every shell's
parts take (ui.md, 8); and the view refuses an act on a disabled node,
on an option of a disabled select, or one its node's role does not
take (ui.md, 6), with the drawing bound as any node a person acts on.
`View::address` and `View::external` (ui.md, 6.2) and `number` (ui.md,
6) exist at the base (jig's `render.rs:88-95`, `binding.rs:36-51`), and
stay as they are.

## Read first

- skein's `docs/design/ui.md`, sections 6, 6.2, 8, 11.3 and 12.
- This plan's `code-organization.md`, 3 and 6.
- skein's `programming-model.md`, section 10.

## What the design asks

- **The look's names:** each owner's prefix, and its tokens' and region
  names' names, each the prefix, a hyphen, then ASCII letters, digits
  and hyphens; skein-ui holds the names and their check, which refuses
  a duplicate name or one off its owner's prefix (ui.md, 8).
- **The act refusal:** ui.md 6's table of the acts each bound role
  takes; the view gives nothing for an act on a disabled node, on an
  option of a disabled select, an act its role does not take, or a
  drawing's data past `drawing_bytes`; a binding on any other role is
  an assertion in the builder (jig 03.1 made it; this session adds the
  drawing to it).

## Choices this plan makes

- `skein_ui::look::Names` is plain data borrowed from the application:
  an owner, its prefix, its tokens' names indexed by `Style.token` and
  its region names indexed by `Region.number`; `look::check` gives the
  first fault or none, as the outline's check does.
- The act table is one function, `binding::takes(role, submits, &Act)
  -> bool`, which `View::act` and the fake person's tree face (skein
  06.3) both call: one table, so the two cannot differ.

## Rules that bind it

- **One table:** no copy of ui.md 6's table outside `binding::takes`:
  `rg -n 'fn takes\b' crates/skein-ui crates/skein-ui-text
  crates/skein-ui-web testing/skein-fake-person` prints one line at the
  session's end.
- **A refusal gives nothing, never an error** (ui.md, 6): `View::act`
  keeps its signature, `Option<BoundAct>`.

## Increments

### 03.1 The look's names and their check
Scope: ui.md 8.
Needs: skein 01.3 (nothing in skein-ui changes between skein 01.1 and
  01.3).
Files: new `crates/skein-ui/src/look.rs`; `crates/skein-ui/src/lib.rs`;
  new `src/tests/look.rs`.
Shape: as the sketch below. `check` refuses, in this order: a prefix
  that is empty or holds a byte outside `a-z`, `0-9`; two owners with
  one prefix; a name not starting with its owner's prefix and `-`, or
  holding a byte outside `a-z`, `A-Z`, `0-9`, `-` after it; a name given
  twice across every owner's tokens and region names.
  `Names::style(&self, style: Style) -> Option<&[u8]>` and
  `Names::region(&self, region: Region) -> Option<&[u8]>` find a name
  by its number, `None` for another owner's or past the table.
Test shape: `src/tests/look.rs`, a table test
  `each_fault_of_the_look_is_named`, the reference's three rows plus a
  `Prefix` row (prefix `b"Notes"`) and a `SamePrefix` row (two owners
  of prefix `b"notes"`), each row holding its own `&[Names]`, the
  expected `look::Fault` a literal; and
  `a_style_finds_its_name_by_number`. Reference, its first three rows:
  ```rust
  #[test]
  fn each_fault_of_the_look_is_named() {
      let owner = Owner { number: 1 };
      let rows: [(&[&[u8]], Option<Fault>); 3] = [
          (&[b"notes-bar"], None),
          (&[b"bar"], Some(Fault::OffPrefix { owner, index: 0 })),
          (&[b"notes-bar", b"notes-bar"], Some(Fault::Twice { owner, index: 1 })),
      ];
      for (styles, expected) in rows {
          let names = [Names { owner, prefix: b"notes", styles, regions: &[] }];
          assert_eq!(check(&names), expected, "{styles:?}");
      }
  }
  ```
  Under 20 ms.
Proof: unit, `heavy cargo nextest run -p skein-ui -E 'test(/look/)'`
  once. Its first users are skein 09.2 (classes in SVG) and skein 09.3
  (classes and `data-region`), which connect it.
Don't: hold a look's files, pictures or drawing code here (ui-web.md,
  3.2: the parts' look holds them).
Size: about 90 lines.
Removes: nothing.

### 03.2 The act refusal, and the drawing bound
Scope: ui.md 6 (the table and the refusals), 11.3 (a drawing bound).
Needs: skein 02.4 (`Role::Select` and `Role::Option`, whose rows the
  table holds).
Files: `crates/skein-ui/src/{binding.rs (takes), view.rs (View::act,
  jig's render.rs:75-86), builder.rs (bind, jig's builder.rs:251-257;
  its role list from jig 03.1), tree.rs (Tree::act, jig's
  tree.rs:372-375)}`; `src/tests/tree.rs` (jig 03.1's table test
  `bind_takes_only_the_roles_a_person_acts_on` gains the drawing's
  row); new `src/tests/view.rs` rows. Users of `View::act` in skein:
  `rg -n '\.act\(' crates/skein-ui-text testing/skein-fake-person`
  (none expected to change: the text shell's commands act as each role
  takes, ui-text.md 6).
Shape: `binding::takes(role: Role, submits_on_enter: bool, act: &Act)
  -> bool`, ui.md 6's table, matched whole over `Role` and `Act`;
  `View::act` gives `None` when the node is gone or unbound, disabled,
  an option whose nearest select ancestor is disabled, `takes` is
  false, or a `Drawing`'s data is past `drawing_bytes`; `bind` takes a
  drawing (its "never bound" assertion goes).
Test shape: `src/tests/view.rs`: a table test
  `each_role_takes_only_its_acts`, one row per bound role and act with
  the expected outcome as a literal (`Some` or `None`), built on one
  fixture page; and `an_option_of_a_disabled_select_takes_no_press`,
  `a_disabled_button_takes_no_press`,
  `drawing_data_past_its_bound_is_dropped`. Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p skein-ui -p skein-ui-text -p
  skein-fake-person` once: the view refuses as ui.md 6 says, and the
  text shell's command tests, which act through it, still pass. The
  first shell that reports acts to it in a browser is skein 11's.
Don't: refuse in the builder what the view refuses (a disabled node
  stays bindable); change `BoundAct`.
Size: about 80 lines.
Removes: the builder's assertion that a drawing is never bound.

## Interface sketch

```rust
// crates/skein-ui/src/look.rs: the look's names (ui.md, 8). The
// application owns the data; the shells' parts borrow it.

/// One owner's names: its prefix, its tokens' and its region names'.
pub struct Names<'a> {
    pub owner: Owner,
    pub prefix: &'a [u8],
    /// Indexed by `Style.token`.
    pub styles: &'a [&'a [u8]],
    /// Indexed by `Region.number`.
    pub regions: &'a [&'a [u8]],
}
/// The first fault of a look's names; `index` counts the owner's styles,
/// then its regions.
pub enum Fault {
    Prefix { owner: Owner },
    SamePrefix { owner: Owner },
    OffPrefix { owner: Owner, index: u32 },
    Twice { owner: Owner, index: u32 },
}
pub fn check(names: &[Names<'_>]) -> Option<Fault>;
impl Names<'_> {
    pub fn style(&self, style: Style) -> Option<&[u8]>;
    pub fn region(&self, region: Region) -> Option<&[u8]>;
}

// crates/skein-ui/src/binding.rs: ui.md 6's table, the one copy.
pub fn takes(role: Role, submits_on_enter: bool, act: &Act) -> bool;
```

## Questions to raise when reached

None known.

## Done when

`rg -n 'fn takes|fn check' crates/skein-ui/src/binding.rs
crates/skein-ui/src/look.rs` prints one line each, and `heavy cargo
nextest run -p skein-ui` passes.
