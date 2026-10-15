# Session skein 03: the look's names, and the act refusal

skein-ui gets the look's names and their check, which every shell's
parts take (ui.md, 8); and the view refuses an act on a disabled node,
on an option of a disabled select, or one its node's role does not
take (ui.md, 6), with the drawing bound as any node a person acts on.
`View::address` and `View::external` (ui.md, 6.2) and `number` (ui.md,
6) exist at the base (skein's `crates/skein-ui/src/view.rs:91`,
`binding.rs:37`, as skein 01.1 copied them), and stay as they are.

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
  `rg -n 'fn takes\b' crates testing` prints one line at the session's
  end.
- **A refusal gives nothing, never an error** (ui.md, 6): `View::act`
  keeps its signature, `Option<BoundAct>`.

## Increments

### 03.1 The look's names and their check
Scope: ui.md 8.
Needs: skein 01.3 (nothing in skein-ui changes between skein 01.1 and
  01.3).
Files: new `crates/skein-ui/src/look.rs`; `crates/skein-ui/src/lib.rs`;
  new `src/tests/look.rs`, and `mod look;` in `src/tests.rs`.
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
  `a_style_finds_its_name_by_number`. Reference, its first three rows,
  each row its own `&[Names]` so that the `Prefix` and `SamePrefix`
  rows fit (`SamePrefix` names the second owner):
  ```rust
  #[test]
  fn each_fault_of_the_look_is_named() {
      let owner = Owner { number: 1 };
      let one = |styles| Names { owner, prefix: b"notes", styles, regions: &[] };
      let rows: [(&[Names<'_>], Option<Fault>); 3] = [
          (&[one(&[b"notes-bar"])], None),
          (&[one(&[b"bar"])], Some(Fault::OffPrefix { owner, index: 0 })),
          (&[one(&[b"notes-bar", b"notes-bar"])], Some(Fault::Twice { owner, index: 1 })),
      ];
      for (names, expected) in rows {
          assert_eq!(check(names), expected, "{names:?}");
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
  78-88), builder.rs (bind, 253-260; its role list, `acted_on`, 453,
  gains `Drawing`), tree.rs (Tree::act, 367)}`; `src/tests/tree.rs`
  (the table test `bind_takes_only_the_roles_a_person_acts_on`: its row
  `(Role::Drawing, false)`, 142, becomes `true`; `a_drawing_cannot_be_bound`,
  89-95, goes with the assertion); `src/tests/binding.rs` (the loop at
  13-22 gives the text area `State::SubmitsOnEnter`, so that the
  `Submit` at 34 is one its role takes; `an_act_on_a_drawing_past_its_bytes_is_dropped`,
  59, binds a drawing, not a button, and moves to `src/tests/view.rs`
  as `drawing_data_past_its_bound_is_dropped`); new `src/tests/view.rs`
  and `mod view;` in `src/tests.rs`. The text shell needs no change: it
  calls no `View::act`, and its `command.rs` never meets a drawing,
  since a drawing is never numbered (ui-text.md 5; `render.rs:148`
  prints it as a container).
Shape: `binding::takes(role: Role, submits_on_enter: bool, act: &Act)
  -> bool`, ui.md 6's table, matched whole over `Role` and `Act`;
  `View::act` gives `None` when the node is gone or unbound, disabled,
  an option whose nearest select ancestor is disabled, `takes` is
  false, or a `Drawing`'s data is past `drawing_bytes`; `bind` takes a
  drawing (its "never bound" assertion goes). The disabled rule is one
  function, `Tree::disabled(&self, id: NodeId) -> bool`: the node
  disabled, or an option whose nearest select ancestor is disabled,
  found by a scan back in preorder over each node's `size`. `View::act`,
  the text shell's numbers and commands (skein 05.2) and the fake
  person (skein 06.2, 06.3) call it: no copy of the rule elsewhere.
Test shape: `src/tests/view.rs`: a table test
  `each_role_takes_only_its_acts`, one row per bound role and act with
  the expected outcome as a literal (`Some` or `None`), built on one
  fixture page; and `an_option_of_a_disabled_select_takes_no_press`,
  `a_disabled_button_takes_no_press`,
  `drawing_data_past_its_bound_is_dropped`. Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p skein-ui -p skein-ui-text -p
  skein-fake-person` once: the view refuses as ui.md 6 says, and the
  text shell's and the fake person's tests, over the builder's new
  `bind`, still pass. The first shell that reports acts to it in a
  browser is skein 11's.
Don't: refuse in the builder what the view refuses (a disabled node
  stays bindable); change `BoundAct`.
Size: about 80 lines.
Removes: the builder's assertion that a drawing is never bound, and
  its test `a_drawing_cannot_be_bound`.

## Interface sketch

```rust
// crates/skein-ui/src/look.rs: the look's names (ui.md, 8). The
// application owns the data; the shells' parts borrow it.

/// One owner's names: its prefix, its tokens' and its region names'.
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fault {
    Prefix { owner: Owner },
    SamePrefix { owner: Owner },
    OffPrefix { owner: Owner, index: u32 },
    Twice { owner: Owner, index: u32 },
}
#[must_use]
pub fn check(names: &[Names<'_>]) -> Option<Fault>;
impl Names<'_> {
    #[must_use]
    pub fn style(&self, style: Style) -> Option<&[u8]>;
    #[must_use]
    pub fn region(&self, region: Region) -> Option<&[u8]>;
}

// crates/skein-ui/src/binding.rs: ui.md 6's table, the one copy.
#[must_use]
pub fn takes(role: Role, submits_on_enter: bool, act: &Act) -> bool;
```

## Questions to raise when reached

None known.

## Done when

`rg -n 'fn takes|fn check' crates/skein-ui/src/binding.rs
crates/skein-ui/src/look.rs` prints one line each, and `heavy cargo
nextest run -p skein-ui` passes.
