# Session skein 02: the vocabulary added now

skein-ui's tree gains what ui.md 3 has and jig's tree lacked: the
properties (a description, an entry kind, a focus request, a count, a
position, spans, a sort, an image's address), Escape as a shortcut with
its outline rule, the roles select, option, column header and row
header, and the state required. Each addition reaches the node, the
builder, the diff's attribute, the web mapping and the page part
(skein 09), and, where the build matches it whole, the text shell's
lines and the fake person's names, with their golden rows. The text
shell's lines for the properties are skein 05's.

## Read first

- skein's `docs/design/ui.md`, sections 3, 3.1, 3.2, 3.4, 3.5, 5, 6.1,
  7 (the focus request), 14 and 17; `ui-text.md`, section 5;
  `ui-web.md`, sections 6 and 7.
- This plan's `code-organization.md`, 2, 3 and 6; `findings/audit.md`, 2.
- skein's `programming-model.md`, section 10 (the subset);
  `testing-strategy.md`, section 8 (the suites).

## What the design asks

The roles of ui.md 3.1 and the states and properties of 3.2, closed, so
an addition breaks the build wherever they are matched (ui.md, 3). The
builder sets each on the innermost open node, and an attribute off its
roles is an assertion (ui.md, 5). An attribute patch carries what a
shell maps (ui.md, 7). At most one focus request per tree (ui.md, 7).
Escape is named only inside a dialog, a rule of the outline (ui.md, 6.1
and 14). An image's source is a picture or a checked address (ui.md,
3.5).

## Choices this plan makes

- **Escape** is `Shortcut::ESCAPE`, `Shortcut { key: 0x1b, modifier:
  Modifier::Plain }`: the struct stays, so jig's shortcuts keep their
  code; the builder takes a letter, a digit, or this one value.
- **An image's source** is `Source::{Picture(Picture), Address(Box<[u8]>)}`
  in the field `source`, which replaces `picture`; the setter
  `picture(Picture)` stays, and `image_address(&[u8])` is new. An
  address counts in the node's text bytes, as a link's external target
  does (ui.md, 15 lists the target; an image's address is the same kind
  of bytes).
- **Spans** are `Spans { rows: u16, columns: u16 }`, `None` meaning one
  each; a count and a position are `u32`; a focus request is the
  owner's count, `u64`, as a value's `written` is.

## Rules that bind it

- **One closed set, matched whole:** no `_ =>` arm over `Role`, `State`,
  `EntryKind`, `Sort` or `Source`: `git diff origin/main --
  crates/skein-ui crates/skein-ui-text crates/skein-ui-web
  testing/skein-fake-person | grep -c '^+.*_ =>'` prints 0 at each
  increment's end.
- **Every field of `Node` that a shell maps has its field in
  `Attribute`,** compared in the diff (code-organization.md, 6: no field
  its counterpart lacks): each increment's diff test
  (`a_moved_focus_request_is_an_attribute_patch` and its kind) checks
  its fields.
- **Golden files change by hand:** an existing golden's changed lines
  quoted in the commit body; a new golden named once.

## Increments

### 02.1 A description, an entry kind, a focus request and an image's address
Scope: ui.md 3.2 (description, entry kind, focus request), 3.5 (an
  image's address), 5, 7.
Needs: skein 09.5 (the mapping and the page part, which each addition
  reaches).
Files: `crates/skein-ui/src/{tree.rs (Node, Node::vacant and new,
  text_bytes; jig's tree.rs:238-325 at the base), builder.rs (setters),
  diff.rs (Attribute, Attribute::of, the attribute comparison: jig's
  diff.rs:16-46, 260), markdown.rs (the address check beside safe_url),
  lib.rs (exports)}`; tests in `src/tests/tree.rs`, `src/tests/diff.rs`;
  `crates/skein-ui-web/src/{mapping.rs, page.rs}` and their tests and
  goldens.
  Users of `Node::picture` and `Attribute::picture`: `rg -n
  '\.picture\b|picture:' crates/skein-ui crates/skein-ui-text
  testing/skein-fake-person` (jig's diff.rs:40 and 260,
  tests/tree.rs:198-202 at the base).
Shape: `Node.description: Option<Box<[u8]>>`, `Node.entry:
  Option<EntryKind>`, `Node.focus: Option<u64>`, `Node.source:
  Option<Source>` (for `picture`); the same four in `Attribute`.
  Builder: `description(&[u8])` (any node, charged as a name is),
  `entry(EntryKind)` (a text field only), `focus(u64)` (any node; a
  second in the tree, or in a spliced fragment beside the tree's, is an
  assertion), `image_address(&[u8])` (an image only, asserting
  `markdown::safe_image_address`). `markdown::safe_image_address(&[u8])
  -> bool`: a path starting with one `/` and not `//`, or `https://`
  and a host, with no space, control byte, quote or angle bracket.
  The web: a description an element of the page part's,
  `id="skein-ui-<node>-description"`, named by `aria-describedby`
  (ui-web.md, 6); an entry kind the `<input>`'s `type`; an image's
  address its `src`; a focus request an `Op::Focus` after the apply,
  `tabindex="-1"` first on an element that cannot take it (ui-web.md,
  7).
Test shape: `src/tests/tree.rs`, with the fixtures of
  `src/tests/fixtures.rs` (`LIMITS`). Reference test:
  ```rust
  #[test]
  fn a_second_focus_request_in_one_tree_is_an_assertion() {
      let result = std::panic::catch_unwind(|| {
          let mut b = Builder::new(&LIMITS);
          b.open(Role::Page);
          b.focus(1);
          b.open(Role::Main);
          b.focus(1);
      });
      assert!(result.is_err());
  }
  ```
  Also: `a_description_counts_in_the_nodes_text_bytes`,
  `an_entry_kind_on_a_button_is_an_assertion`,
  `an_image_address_must_be_a_path_or_https` (rows: `/a.png` yes,
  `//x/a.png` no, `https://x/a.png` yes, `http://x/a.png` no,
  `/a b.png` no), and in `src/tests/diff.rs`
  `a_moved_focus_request_is_an_attribute_patch`; in skein-ui-web's
  `src/tests/mapping.rs`, a row per property in
  `each_state_and_property_maps_to_its_attribute`, and in
  `src/tests/page.rs` `a_focus_request_focuses_its_node`. Each under
  20 ms.
Proof: joint, `heavy cargo nextest run -p skein-ui -p skein-ui-text -p
  skein-fake-person -p skein-ui-web` once: the view's patches carry the
  new fields into the page part's model, against golden HTML. The text
  shell prints them in skein 05.3.
Don't: print them in the text shell (skein 05.3); add a `_ =>` arm.
Size: about 150 lines.
Removes: `Node::picture` and `Attribute::picture` (for `source`).

### 02.2 A count, a position and spans
Scope: ui.md 3.2 (count, position, spans).
Needs: skein 09.5.
Files: `crates/skein-ui/src/{tree.rs, builder.rs, diff.rs}`; tests in
  `src/tests/tree.rs` and `src/tests/diff.rs`;
  `crates/skein-ui-web/src/mapping.rs` and its tests.
Shape: `Node.count: Option<u32>` (list, table, tree), `Node.position:
  Option<u32>` (item, row, tree item; from 1, zero an assertion),
  `Node.spans: Option<Spans>` (cell; each at least 1); the three in
  `Attribute`. Builder: `count(u32)`, `position(u32)`, `spans(Spans)`.
  The web: `aria-setsize` and `aria-posinset` on items and tree items,
  `aria-rowcount` and `aria-rowindex` on a table and its rows, `colspan`
  and `rowspan` left out at 1 (ui-web.md, 6).
Test shape: `src/tests/tree.rs`: `a_count_on_an_item_is_an_assertion`,
  `a_position_of_zero_is_an_assertion`,
  `spans_on_a_paragraph_are_an_assertion`; `src/tests/diff.rs`:
  `a_changed_position_is_one_attribute_patch`; mapping rows for the
  three. Each under 20 ms.
Proof: joint, `heavy cargo nextest run -p skein-ui -p skein-ui-web`
  once: the properties reach the page part's model. skein 05.3 prints
  them.
Don't: give spans to the header roles here (02.4 does, with the roles).
Size: about 80 lines.
Removes: nothing.

### 02.3 Escape, and its outline rule
Scope: ui.md 6.1 (Escape), 14 (Escape only inside a dialog).
Needs: skein 09.5.
Files: `crates/skein-ui/src/{tree.rs (Shortcut), builder.rs (shortcut,
  jig's builder.rs:192-199 at the base), outline.rs (Rule, check)}`;
  `src/tests/outline.rs` (the table test of jig 02.3);
  `crates/skein-ui-web/src/{mapping.rs, page.rs}` and their tests.
Shape: `Shortcut::ESCAPE`; `shortcut` takes a letter, a digit, or
  `ESCAPE`; `outline::Rule::EscapeInDialog`, the fault naming the first
  node outside every dialog that names Escape. `ShortcutOnce` counts
  Escape as any shortcut. The web: `aria-keyshortcuts="Escape"`; the
  page part's `Event::Cancel { dialog }` presses the first bound,
  enabled node in the dialog naming Escape, and with none gives no act
  (ui-web.md, 7).
Test shape: two rows added to `each_rule_names_its_first_fault`
  (`src/tests/outline.rs`): Escape on a button in the main, its
  `Fault { rule: Rule::EscapeInDialog, node: <its index> }` as a
  literal; Escape on a dialog's button, `None`. In skein-ui-web's
  `src/tests/page.rs`: `escape_presses_the_dialogs_escape_button`.
  Under 20 ms each.
Proof: joint, `heavy cargo nextest run -p skein-ui -p skein-ui-web`
  once: a dialog's cancel presses its Escape node in the page part's
  model. Chromium's Escape is skein 13.3's story.
Don't: print Escape in the text shell (ui-text.md, 5: not printed).
Size: about 40 lines.
Removes: nothing.

### 02.4 Select, option, column header, row header, and a sort
Scope: ui.md 3.1 (the four roles), 3.2 (a sort; spans on headers);
  ui-text.md 5 (their lines); ui.md 17.1 (their names).
Needs: skein 09.5 (skein 01.2 and 01.3, whose matches over `Role`
  take the new variants, landed before it).
Files: `crates/skein-ui/src/{tree.rs (Role, Sort, Node.sort),
  builder.rs (sort, spans on headers, bind's role list), diff.rs}`;
  `crates/skein-ui-text/src/render.rs` (the role match, jig's
  render.rs:129-215; a row's join, 179-193; the choice's mark,
  227-228); `crates/skein-ui-text/src/command.rs` (its match over
  `Role`, 74-119: an option a `Press`, a select and the headers no
  act); `crates/skein-ui-text/src/tests/fixtures.rs` (a select of
  three options, one selected; a table with a column header carrying a
  sort and a row header) and `src/tests/golden/roles.txt`;
  `testing/skein-fake-person/src/tree.rs` (`role_name`, 118; `passive`,
  170) and `src/random.rs` (`field`, 81, and `choice`, 130: their
  matches over `Role` take the four roles); `crates/skein-ui-web/src/{mapping.rs (rows,
  `accessible`), page.rs (a select's change)}` and their tests.
Shape: `Role::{Select, Option, ColumnHeader, RowHeader}`; `Node.sort:
  Option<Sort>` on a column header, in `Attribute`; builder
  `sort(Sort)`; `spans` takes the headers too; `bind` takes an option.
  The text shell: a select prints as a group (`NAME:`, its options
  indented; unnamed, its options); an option as a choice does, `[n] (*)
  TEXT` or `[n] ( ) TEXT`; a row joins its cells' and headers' texts by
  ` | `; a header outside a row prints on its own line; a sort
  `(ascending)` or `(descending)` after its header's text. The person's
  names: `select`, `option`, `column header`, `row header`. The web:
  `<select>`, `<option>` showing its name, `<th scope="col">` and
  `<th scope="row">`, `aria-sort`; `accessible` gives `combobox`,
  `option`, `columnheader`, `rowheader`; the page part's
  `Event::Choose { select, option }` is a `Press` on the option, the
  element's choice put back by an `Op::Choice` (ui-web.md, 7).
Test shape: the golden `roles.txt`, edited by hand with the fixture's
  new lines; in `src/tests/tree.rs`, `a_sort_on_a_cell_is_an_assertion`;
  mapping rows for the four roles and the sort; in skein-ui-web's
  `src/tests/page.rs`,
  `a_select_change_presses_the_option_and_puts_the_choice_back`. Under
  20 ms each.
Proof: joint, `heavy cargo nextest run -p skein-ui -p skein-ui-text -p
  skein-fake-person -p skein-ui-web` once: the golden text page prints
  the select and the headed table, and the page part maps them.
Don't: number an option of a disabled select differently (skein 05.2);
  refuse acts by role (skein 03.2).
Size: about 150 lines.
Removes: nothing.

### 02.5 The state required
Scope: ui.md 3.2 (required); ui-text.md 5 (`(required)`).
Needs: skein 09.5.
Files: `crates/skein-ui/src/tree.rs` (State);
  `crates/skein-ui-web/src/mapping.rs` (`required` where the element
  takes it, else `aria-required`); the states' capacity
  (jig's tree.rs:288, builder.rs:161, limits.rs:32 at the base: nine,
  with `Provisional` gone and `Required` added);
  `crates/skein-ui-text/src/render.rs` (the states' order, jig's
  render.rs:299-323); its fixture and `roles.txt`.
Shape: `State::Required`, on a text field, a text area, a select, a
  check or a choice group (the builder asserts); the text shell prints
  `(required)` after `(selected)`.
Test shape: `roles.txt` by hand; `src/tests/tree.rs`:
  `required_on_a_heading_is_an_assertion`. Under 20 ms.
Proof: joint, `heavy cargo nextest run -p skein-ui -p skein-ui-text -p
  skein-ui-web` once: the golden field prints `(required)`, and the page
  part maps it.
Don't: change the capacity from nine.
Size: about 30 lines.
Removes: nothing.

## Interface sketch

```rust
// crates/skein-ui/src/tree.rs: the vocabulary's additions. A shell maps
// each; the diff carries each in `Attribute`, field for field.

/// What a text field takes: a shell offers the keyboard or control.
pub enum EntryKind { Text, Number, Email, Telephone, Url, Date, Time, Secret }
/// A column header's order.
pub enum Sort { Ascending, Descending }
/// The rows and the columns a cell or a header spans, each at least 1.
pub struct Spans { pub rows: u16, pub columns: u16 }
/// An image's source: one of its owner's pictures, or a checked address.
pub enum Source { Picture(Picture), Address(Box<[u8]>) }

impl Shortcut {
    /// Escape, named only inside a dialog (outline::Rule::EscapeInDialog).
    pub const ESCAPE: Shortcut = Shortcut { key: 0x1b, modifier: Modifier::Plain };
}

impl Builder {
    pub fn description(&mut self, bytes: &[u8]);
    pub fn entry(&mut self, kind: EntryKind);
    pub fn focus(&mut self, count: u64);
    pub fn image_address(&mut self, address: &[u8]);
    pub fn count(&mut self, count: u32);
    pub fn position(&mut self, position: u32);
    pub fn spans(&mut self, spans: Spans);
    pub fn sort(&mut self, sort: Sort);
}
```

## Questions to raise when reached

None known.

## Done when

`rg -n 'Select|ColumnHeader|RowHeader|Required|EntryKind|EscapeInDialog'
crates/skein-ui/src` prints lines for each, and `heavy cargo nextest run
-p skein-ui -p skein-ui-text -p skein-fake-person` passes.
