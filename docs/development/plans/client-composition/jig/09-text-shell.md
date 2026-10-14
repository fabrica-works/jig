# Session 09: jig's text shell prints a tree and reads a line

A new crate, `jig-shell-text`, holds jig's shell for a terminal or a
pipe. This session builds its two pure parts: the renderer, which prints
the view's tree as lines with a number on each node a person can act
on, and the command reader, which makes one typed line an act on a
node, a print, closing, nothing, or a refusal. Golden text fixes both:
hand-built trees for every role, state and unsafe byte, and jig's pages
as the client's world shows them. jig 10 puts the terminal around them
and plays the worlds at the text shell.

## Read first

- `client/domain/view.md`, 15 (whole: 15.1 to 15.6), 3 (roles and
  states), 13 ("Region names are not styles").
- `client/domain/testing.md`, 1 and 2 (the text shell's tiers).
- decisions.md, 6, 9 and 14.

## What the design asks

- `jig_shell_text::render`: from the view's tree alone, the page's lines
  and the last print's numbers; pure, step code under the subset, with
  no io and no clock; its output bounded by the most lines and bytes the
  view's largest tree prints, counted from the view's limits (15.1).
- How it prints: the first line `== TITLE == LINK`; then every node in
  preorder by its role (the table of 15.3); states in parentheses in
  their order; text runs; control characters and bytes that are not
  UTF-8 as U+FFFD, a tab as a space, a newline as a new line at the same
  indentation; no wrapping (15.3).
- Numbers: each bound, enabled node of a role a person acts on gets
  `[n]` from 1 in preorder; a disabled or unbound one prints `[-]`; the
  shell keeps its last print's numbers (15.4).
- `jig_shell_text::command`: from one line, the last print's numbers and
  the view's last tree, one command: acts on a node (a press, a toggle,
  or a field's input followed, when it submits on enter, by a submit),
  print, close, nothing, or a refusal with its line; pure (15.1, 15.4).
- Golden text: pages kept as files beside the tests and changed only by
  hand, built by jig's test view root from fixture states; every
  command and every refusal tested, numbers kept across a print and
  renumbered at the next (15.6).

## Choices this plan makes

- **One crate under the workspace's step-code lints**
  (`[lints] workspace = true`, as `jig-view`): the renderer, the command
  reader and, in jig 10, the terminal. The terminal needs nothing
  outside the subset (it holds skein's `List`s and `Queue`s and an io
  layer); a need outside it is a question, not a lint exception.
- **A print** is `Print { text: List<u8>, numbers: List<NodeId> }`, made
  once with `Print::new(limits: &jig_view::Limits) -> Option<Print>`
  from `render::worst_case`, and filled again by each `render`. It also
  keeps, private, the longest name `render` takes: `limits.text_per_node`
  (view.md 15.1).
- **Golden files in two places** (decisions.md, 9): the hand-built trees'
  in `crates/jig-shell-text/tests/golden/`, read by the crate's tests;
  jig's pages' in `tests/client/tests/golden/`, read by a test of the
  client's world, which builds the fixture states with jig's test view
  root. Each read with `include_bytes!`, under `#[expect(clippy::disallowed_macros,
  reason = "…")]` (`clippy.toml:93` denies it), as
  `crates/jig-view/src/tests/markdown.rs:2` does.
- **A refusal** is typed, `Refusal::{NoNumber, Gone, TakesNoWords,
  NoPage, TooLong, Unknown}`, and `refusal_line(name, refusal, n)` makes
  its line; `NoNumber` and `Gone` also ask for the page again.

## Rules that bind it

- The renderer reads nothing but the tree: no style token, region name
  or shortcut is printed (view.md, 15.3, "Text").
- Golden files change only by hand; no test writes one (decisions.md, 9).
- Aim for the session's tests to add at most 0.4 s to the focused
  suite's median (a guide: README, "Test time");
  the goldens of jig's pages run worlds, each under 0.5 s.

## Increments

### 09.1 The crate, and the renderer
Scope: view.md, 15.1 (the renderer), 15.3, 15.4 (numbers).
Needs: jig 02 (the outline; `jig-link`'s region); jig 01 (the roles).
Files: new `crates/jig-shell-text/` (`Cargo.toml` with `jig-view` and
  `skein-lib`; `src/lib.rs`, `src/render.rs`, `src/tests.rs`,
  `src/tests/fixtures.rs`, `tests/golden/roles.txt`,
  `tests/golden/bytes.txt`); the workspace's `Cargo.toml` (member and
  `[workspace.dependencies]`, beside `jig-view` at `:135`);
  `tests/boundary/src/tests/graph.rs:448-455` (the client crates that
  link no engine crate: add `jig-shell-text`), `:138-151` (`client_step`:
  add `jig-shell-text`, step code under the workspace's lints) with an
  `assert!(client_step("jig-shell-text"))` beside `:177`, and
  `tests/boundary/src/tests.rs:340-352` (a test beside the native
  shell's).
Shape: `pub fn render(tree: &Tree, name: &[u8], print: &mut Print)`;
  `pub fn worst_case(limits: &jig_view::Limits) -> Option<(u32, u64)>`
  (lines, bytes); `#[derive(Debug)] pub struct Print { pub text: List<u8>, pub numbers:
  List<NodeId> }` with `Print::new`. `name` is at most
  `limits.text_per_node` bytes (view.md 15.1: the shell's name "is at
  most a node's text, so the same limits count it"): `worst_case` adds
  one node's text to the first line for TITLE, counted as it counts any
  node's text, whether TITLE is the main's heading or `name`;
  `Print::new` keeps `limits.text_per_node` in a private field of
  `Print`, and `render` checks `name.len()` against it with `assert!`
  (`clippy.toml:87`: "assertions stay on in production"); `Terminal::new`
  refuses a longer name (jig 10.1). A node is bound when its `binding`
  or its `link` is set: `render` reads those fields, not `Tree::bound`,
  which gives nothing for `NodeId(0)` (`crates/jig-view/src/tree.rs:362-368`). TITLE is the text of the main's
  first level-1 heading, or `name`; LINK is the text of the node whose
  region is `JigRegion::Link.region()`. The role table of 15.3, one arm
  per `Role`, no `_` arm.
Test shape: `crates/jig-shell-text/src/tests.rs`, with
  `src/tests/fixtures.rs` (made here: a builder over `jig_view::Limits`
  for hand-built trees): `a_tree_with_every_role_and_state_prints_its_golden_text`
  (`tests/golden/roles.txt`), `control_bytes_and_bad_utf8_print_as_replacement`
  (`tests/golden/bytes.txt`), `a_disabled_button_prints_a_dash_not_a_number`,
  `the_first_line_is_the_main_heading_and_the_link`. In
  `tests/boundary/src/tests.rs`:
  `the_text_shell_links_only_the_view_and_foundation` (its dependencies
  are `jig-view` and `skein-lib`, and `skein-io` once jig 10.1 adds it).
  Reference:
  ```rust
  #[test]
  fn a_disabled_button_prints_a_dash_not_a_number() {
      let tree = fixtures::page(|b| {
          fixtures::button(b, b"Accept", true);
          fixtures::button(b, b"Release", false);
      });
      let mut print = Print::new(&fixtures::LIMITS).unwrap();
      render(&tree, b"ops", &mut print);
      assert_eq!(print.text.as_slice(), b"== ops == \n[1] Accept\n[-] Release (disabled)\n");
  }
  ```
  (`fixtures::LIMITS`, `fixtures::page` and `fixtures::button` are what
  this increment puts in `src/tests/fixtures.rs`: `page(main:
  fn(&mut Builder)) -> Tree` builds a page with the banner jig's outline
  needs and `main`'s nodes in a main, then gives the nodes their ids with
  `jig_view::diff` from an empty tree, as the view does (so that 09.2's
  commands find them by `Tree::bound`); `main` is a fn pointer, which the
  reference's closure, capturing nothing, coerces to: `clippy.toml:63-65`
  disallows `Fn`, `FnMut` and `FnOnce`;
  `button(b, name, enabled)` a bound button, disabled when not enabled; with no link text the first line
  ends `== `: write the expected text from view.md 15.3 when the fixture
  is written.) Each under 20 ms.
Proof: unit; 09.3 prints jig's pages from the client's world, and jig
  10 drives it.
Don't: print a style token, a region name or a shortcut; wrap lines.
Size: at most 300 lines of production code.
Removes: nothing.

### 09.2 The command reader
Scope: view.md, 15.1 (the command reader), 15.4 (commands, refusals).
Needs: 09.1.
Files: `crates/jig-shell-text/src/command.rs`, `src/lib.rs`, a new
  `src/tests/command.rs` (registered in `src/tests.rs`).
Shape: `pub fn command(line: &[u8], numbers: Option<&[NodeId]>, tree:
  &Tree, longest: u32) -> Command`, `numbers` being `None` before the
  first print; `pub enum Command` and `pub enum Refusal`
  as in the sketch; `pub fn refusal_line(name: &[u8], refusal: Refusal,
  number: u64) -> Box<[u8]>`. Leading and trailing spaces are removed
  first; a line longer than `longest` is `TooLong`; `n` presses a
  button, link, tab, menu item or other bound node, toggles a check or a
  choice, writes a field empty then submits it if it submits on enter;
  `n WORDS` writes WORDS as the field's whole value, then submits if it
  submits on enter; `p`, `q`, an empty line; the refusals of 15.4's
  table, `NoPage` for a number when `numbers` is `None`.
Test shape: `crates/jig-shell-text/src/tests/command.rs`: one test per command
  and per refusal (`a_number_presses_its_button`,
  `a_number_toggles_its_check`, `words_after_a_number_fill_and_submit_its_field`,
  `words_for_a_field_that_does_not_submit_only_fill_it`,
  `p_prints_and_q_closes`, `an_empty_line_does_nothing`,
  `a_number_never_printed_is_refused`, `a_number_whose_node_left_is_gone`,
  `words_for_a_button_are_refused`, `a_number_before_the_first_print_is_refused`,
  `a_line_past_the_longest_is_refused`, `anything_else_names_the_commands`),
  and `numbers_hold_across_a_print_and_renumber_at_the_next`. Each under
  20 ms.
Proof: unit; jig 10.2 reads every act of two client-world stories back
  through it.
Don't: add a command (15.4: "The commands are jig's").
Size: at most 150 lines of production code.
Removes: nothing.

### 09.3 jig's pages as golden text
Scope: view.md, 15.6 ("Golden text … built by jig's test view root from
  fixture states").
Needs: 09.1; jig 06 (jig's pages from sections; the test application's
  arranged client) and jig 07.3 (the family's page with its section
  slot).
Files: new `tests/client/tests/text_pages.rs` and
  `tests/client/tests/golden/{inbox,task,project,deployment,starting,signing_in,missing,out_of_date,someone_else}.txt`;
  `tests/client/Cargo.toml` (`jig-shell-text` as a dev-dependency);
  `tests/client/src/stories.rs` (fixture worlds, if a page needs one it
  lacks: the inbox with an entry of each kind from `proposal_world`'s
  and `held_chat`'s fixtures; a task's page with its conversation from
  `chat_ready`; a project's sections from `goal_world`; the deployment
  from `deployment()`'s document; the five pages not about the work
  (view.md 15.6: "the pages not about the work") from the states the
  client's world reaches: starting, before the frame's delivery;
  signing in, as `signing_in_and_starting_a_chat` starts
  (`tests/client/tests/stories.rs:7`); missing, at a missing address;
  out of date and someone else's work, as `tests/client/tests/faults.rs`
  reaches them. A page no world state reaches is the question below).
Shape: no production code. Each test runs its world to `Until::Done`,
  renders `w.tree(c)` with `jig_shell_text::render` and the name
  `b"jig"`, and compares the print's text with its golden file.
Test shape: `tests/client/tests/text_pages.rs`:
  `the_inbox_prints_as_its_golden_text`,
  `a_task_page_with_its_conversation_prints_as_its_golden_text`,
  `a_projects_sections_print_as_their_golden_text`,
  `the_deployment_prints_as_its_golden_text`,
  and `each_page_not_about_the_work_prints_as_its_golden_text` (a table
  over the five, each row labelled with its page). Each under 0.5 s; the
  table's rows under 0.5 s each.
Proof: world, `heavy cargo nextest run -p jig-client-world -E
  'binary(text_pages)'` once: jig's pages, as the client's world shows
  them, print as their golden text.
Don't: generate a golden file by a test or a script. Each file is
  written by hand: it may be drafted from one print, and each of its
  lines is then checked against view.md 15.3's table and the fixture's
  state; the commit body says so (code-organization.md, 5, the golden
  files' exception).
Size: no production code; at most 120 lines of test code besides the
  golden files.
Removes: nothing.

## Interface sketches

```rust
// crates/jig-shell-text/src/render.rs
/// One print: its text, lines ended by `\n`, and its numbers' nodes, `[1]` first.
#[derive(Debug)]
pub struct Print { pub text: List<u8>, pub numbers: List<NodeId>, name_bytes: u32 }
impl Print {
    /// `name_bytes` is `limits.text_per_node`: the longest name `render` takes.
    pub fn new(limits: &jig_view::Limits) -> Option<Print>;
}
/// `name` is at most the print's `name_bytes` (an `assert!`).
pub fn render(tree: &Tree, name: &[u8], print: &mut Print);
/// The most lines and bytes a tree within these limits prints, TITLE
/// counted as one node's text more.
pub fn worst_case(limits: &jig_view::Limits) -> Option<(u32, u64)>;

// crates/jig-shell-text/src/command.rs
/// What one typed line asks.
pub enum Command {
    /// Acts on one node: `first`, then a submit when the field submits on enter.
    Act { node: NodeId, first: jig_view::Act, submit: bool },
    /// Print the page again, at once.
    Print,
    /// Close the terminal.
    Close,
    /// An empty line.
    Nothing,
    /// A refusal, with the number the line named.
    Refused { refusal: Refusal, number: u64 },
}
pub enum Refusal { NoNumber, Gone, TakesNoWords, NoPage, TooLong, Unknown }
pub fn command(line: &[u8], numbers: Option<&[NodeId]>, tree: &Tree, longest: u32) -> Command;
pub fn refusal_line(name: &[u8], refusal: Refusal, number: u64) -> Box<[u8]>;
```

Who owns what: this session owns `render`, `Print`, `command`, `Command`
and `Refusal`; jig 10 uses them by name.

## Questions to raise when reached

- If no state of the client's world reaches one of the five pages not
  about the work, 09.3 prints that page from a domain state built as
  `crates/jig-view/src/tests/pages.rs:42` (`special_pages`) builds it,
  rendered through jig's test view root, and the report names the page;
  no question unless neither reaches it.

## Done when

- `heavy cargo nextest run -p jig-shell-text -p jig-client-world -E
  'package(jig-shell-text) | binary(text_pages)'` passes on `main`, once.
