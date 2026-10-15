# Session skein 06: the fake person whole

skein-fake-person does what ui.md 17.1 asks beyond jig's person: a
press on a check or a choice is a `Toggle`; the random person chooses
options and never acts on a drawing; its names for roles are checked
by a table test; and the tree face fails a step whose act the node's
role does not take, by skein-ui's own act table.

## Read first

- skein's `docs/design/ui.md`, sections 6 and 17.1.
- This plan's `code-organization.md`, 3 and 6; `findings/audit.md`, 2
  (the person's absences).
- skein's `testing-strategy.md`, sections 4 (fakes as step machines)
  and 8.

## What the design asks

The tree face turns a press on a check or a choice into a `Toggle`, and
fails a step on a node unbound, disabled, or whose act its role does not
take (ui.md, 17.1, by section 6). Choosing an option is a `Press` on it
(ui.md, 3). The random person acts on any bound node but a drawing.
Its names for roles, one per role, are what its failures print.

## Rules that bind it

- **One act table:** the face calls `skein_ui::binding::takes` (skein
  03.2); it keeps no copy of ui.md 6's table.
- **The person stays a step machine:** `Person::next` consumes at most
  one step (jig's `lib.rs:1-14` at the base), and nothing here adds a
  clock or a thread.

## Increments

### 06.1 A press on a check or a choice is a toggle
Scope: ui.md 17.1.
Needs: skein 01.3 (`skein-fake-person` in skein).
Files: `testing/skein-fake-person/src/lib.rs` (the act of a press,
  jig's lib.rs:202 at the base: `Step::Press` always `Act::Press`);
  `src/tests.rs`.
Shape: `Person::next` asks the face for the found node's role through a
  new `Face::role(&self, NodeId) -> Option<Role>` (the tree face reads
  its tree; a face that cannot tell gives `None`, and the press stays a
  `Press`); a press on a check or a choice gives `Act::Toggle`.
Test shape: `src/tests.rs`: `a_press_on_a_check_is_a_toggle`,
  `a_press_on_a_choice_is_a_toggle`, `a_press_on_a_button_stays_a_press`.
  Reference test:
  ```rust
  #[test]
  fn a_press_on_a_check_is_a_toggle() {
      let tree = fixtures::page_with_check(b"Notify me");
      let mut person: Person<u8> = Person::new(Box::new([Step::Press {
          find: Find::named(Role::Check, b"Notify"),
      }]));
      let next = person.next(&mut TreeFace::new(&tree), Time::ZERO);
      assert_eq!(next, Next::Do(Doing::Act { node: fixtures::CHECK, act: Act::Toggle }));
  }
  ```
  (`page_with_check` and `CHECK` added to the crate's fixtures, in
  `src/tests.rs` or a new `src/tests/fixtures.rs`). Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p skein-fake-person` once. Unit
  is the highest layer until skein 13.4, whose view tests toggle the
  example's choice group through the tree face (13.4 needs skein 06.3).
Don't: decide by the node's name or states.
Size: about 30 lines.
Removes: nothing.

### 06.2 Options, the random person's acts, and the names of roles
Scope: ui.md 17.1 (the random person; names for roles).
Needs: skein 03.2 (a drawing bound; it needs skein 02.4's
  `Role::Select` and `Role::Option`).
Files: `testing/skein-fake-person/src/random.rs` (`available`, jig's
  random.rs:73-78; `choice`, 129-130), `src/tree.rs` (`role_name`,
  jig's tree.rs:117-167; `passive`, 169-183); `src/tests.rs`.
Shape: the random person acts on any bound, enabled node but a drawing;
  on an option it presses; on a check or a choice it toggles; `passive`
  holds the select (its options take the press).
Test shape: a table test `each_role_has_its_own_name`: one row per
  `Role` with its name as a literal, the rows checked whole and each
  failing row named; `the_random_person_never_acts_on_a_drawing` (a page
  of one bound drawing and one button, 200 draws, every act on the
  button); `the_random_person_presses_an_option`. Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p skein-fake-person` once. Unit
  is the highest layer: no fuzzy world in skein runs the random person;
  an application's does.
Don't: give a drawing a random act.
Size: about 30 lines.
Removes: nothing.

### 06.3 The tree face fails an act its node's role does not take
Scope: ui.md 6 (the table), 17.1 (the face's failures).
Needs: skein 03.2 (`skein_ui::binding::takes`; a drawing bound).
Files: `testing/skein-fake-person/src/tree.rs` (`TreeFace::allows`,
  jig's tree.rs:98-114 at the base); `src/random.rs` (`available`, jig's
  random.rs:73-78, which calls `passive`); `src/tests.rs`.
Shape: `allows` is: the node is in the face's subtree, bound, enabled,
  not an option of a disabled select, and `binding::takes(role,
  submits_on_enter, act)`; `passive` goes, the table covering it: the
  random person's `available` asks `takes` for the act it would make
  (a `Press`, a `Toggle` or an `Input` by the node's role).
Test shape: `typing_into_a_button_fails_as_unexpected`,
  `pressing_an_option_of_a_disabled_select_fails`,
  `a_drawing_takes_only_its_drawing_act`. Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p skein-fake-person` once; skein
  13.4's view tests run the face over the example's choices.
Don't: refuse what the view takes, or take what it refuses: one table.
Size: about 20 lines.
Removes: `passive`.

## Interface sketch

```rust
// testing/skein-fake-person/src/lib.rs: the face's trait, extended.
pub trait Face {
    /// Search by role, name and text, through the named ancestors.
    fn find(&mut self, find: &Find) -> Found;
    /// Check the control's interactive semantics when the face knows them.
    fn allows(&self, _node: NodeId, _act: &Act) -> bool { true }
    /// The node's role, so a press on a check or a choice is a toggle;
    /// none when the face cannot tell.
    fn role(&self, _node: NodeId) -> Option<Role> { None }
}
```

## Questions to raise when reached

None known.

## Done when

`heavy cargo nextest run -p skein-fake-person` passes, and `rg -n 'fn
passive' testing/skein-fake-person` prints nothing.
