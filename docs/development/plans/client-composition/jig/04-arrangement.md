# Session 04: the arrangement, in the domain

An application may have every task's page, or every project's, show
sections beside the address's own. `jig-domain-client` takes that
arrangement before the start, refuses a malformed one, and lists an
arranged page's sections: the address's own and the arrangement's,
each once, a task's only where its document offers it, `Settings`
standing for its five. The page opens what they all need, once each.
jig's own arrangement adds none, so jig's pages are unchanged; jig 06
shows an arranged page in the view and runs its story.

A path in this file that names no crate (`pages/task.rs:662`) is under
`crates/jig-domain-client/src/`.

## Read first

- `client/domain/client.md`, 5.1 (the sixth bullet: "At an address of
  jig's"; "Each section once"; "One subject each"), 14 (limits counted
  for the most sections shown together).
- `client/domain/pages.md`, 12.3 ("More sections together").
- findings/domain.md, 3, 6 and 8.
- jig 03's session file, its interface sketch: `Section`, `sections()`,
  `shows()`.

## What the design asks

- `pages::Arrangement` holds two lists, of `TaskSection` and of
  `ProjectSection`: the sections every task's page, and every
  project's, shows beside the address's own.
- `Domain::configure_arrangement(arrangement)`, before the start, as the
  navigation is given; it refuses after the start, a list longer than
  its kind's sections, one naming a section twice, or one naming
  `Settings` with one of its five. jig's own arrangement adds none.
- A page that would show `Settings` and one of its five shows `Settings`
  alone. A task's section its document does not offer
  (`pages::task::offered`) is not shown, wherever it was named.
- The limits count objects, windows, reads and watches for the most
  sections a page may show together (client.md, 14): decisions.md, 11
  says how this plan reads it; nothing of it is in this session.

## Choices this plan makes

- `Arrangement::none()` is jig's: both lists empty, capacity 0 (skein's `List` has no `Default`, so the type derives none).
  `configure_arrangement` returns the arrangement it refuses, as
  `configure_navigation` returns the navigation (`domain.rs:161-171`).
- The task's offered sections filter in `Domain::shows`, not in
  `sections()`: the list is fixed when the page is entered, before the
  task's document comes; `shows` reads the document each time. Until the
  document comes, a listed task section is shown (the view shows it
  loading, as today).

## Rules that bind it

- **jig's arrangement changes nothing:** with no arrangement given, no
  expected value of any existing test changes in this session.
- `crates/jig-domain-client/src/domain.rs` and `pages/routing.rs` grow
  by at most 60 lines each in this session (code-organization.md, 6).
- Aim for the session's tests to add at most 0.2 s to the focused
  suite's median: a guide (README, "Test time"), checked by each
  landing's `Gate-Result`.

## Increments

### 04.1 The arrangement, given before the start
Scope: client.md, 5.1 (`pages::Arrangement`, `configure_arrangement`).
Needs: nothing outside jig 03 (the session's start).
Files: `crates/jig-domain-client/src/pages.rs` (`Arrangement`; it
  imports `TaskSection` and `ProjectSection`, so `Section`'s
  `crate::address::TaskSection` and `crate::address::ProjectSection` at
  `:23` and `:25` lose their path: `unused_qualifications`);
  `crates/jig-domain-client/src/domain.rs:48-93` (a field), `:95-147`
  (`new`), beside `:161-171` (`configure_arrangement`, and an accessor
  `arrangement()`); the roots' pass-throughs:
  `testing/jig-test-domain-client/src/domain.rs:151-157` (beside
  `provider`) and its `lib.rs` exports;
  `examples/ops/domain-client/src/domain.rs:190-196` (beside
  `configure_navigation`) and `examples/ops/domain-client/src/lib.rs:21`.
Shape: `pub struct Arrangement { pub task: List<TaskSection>, pub project:
  List<ProjectSection> }` with `pub fn none() -> Arrangement` (empty lists, capacity 0);
  `pub fn configure_arrangement(&mut self, arrangement: Arrangement) ->
  Result<(), Arrangement>`, which refuses when `started`, when `task`
  holds more than 6 or `project` more than 10, when either names a
  section twice, or when `project` holds `Settings` and any of
  `Resources`, `Policy`, `Roles`, `Spend`, `Recurring`, checking the
  length first; `pub const fn
  arrangement(&self) -> &Arrangement`. The checks use `list.get(..i)`
  and `contains`, and an exhaustive `match` for `Settings` and its five:
  the lints refuse `matches!`, `Iterator::any` and `filter`,
  `Option::is_some_and`, slicing that may panic and `as` casts. Each root:
  `pub fn configure_arrangement(root: &mut Domain, arrangement:
  jig_domain_client::pages::Arrangement) -> Result<(),
  jig_domain_client::pages::Arrangement>`, passing it to `root.jig`.
Test shape: `crates/jig-domain-client/src/tests/pages.rs`, beside
  `navigation_keeps_the_application_order_and_leaves_out_inapplicable_pages`
  (`:76`), each its own `#[test]` (not in the group at `:1262`, so that
  the Done-when filter `test(/arrange/)` names it):
  `an_arrangement_given_before_the_start_is_kept`, and
  `each_malformed_arrangement_is_refused`, a table test: an array of
  `(label, arrangement)` rows looped, each assertion's message naming
  its row's label ("seven task sections", "Plan twice", "Settings with
  Policy"; "after the start" is the same arrangement given after
  `start`). The test under 20 ms.
Proof: unit; jig 06.4's story gives the test application's client an
  arrangement through its root and runs it in the client's world.
Don't: let the arrangement name an inbox, conversations or deployment
  section (it has no list for them).
Size: at most 70 lines of production code.
Removes: nothing.

### 04.2 An arranged page's sections
Scope: client.md, 5.1 (the sixth bullet; "Each section once").
Needs: 04.1.
Files: `crates/jig-domain-client/src/pages/routing.rs:129-187` (`enter`,
  where jig 03.1 sets the list); `crates/jig-domain-client/src/domain.rs`
  (`shows`); `crates/jig-domain-client/src/pages/task.rs:176` (`offered`,
  used).
Shape: at `Address::Task { task, section }`, the list is `Task { task,
  section }` and `Task { task, section: s }` for each `s` of
  `arrangement.task`, sorted, each once. At `Address::Project { project,
  section }`, the same with `arrangement.project`; then, when the list
  holds `Settings`, every one of its five is removed, and when it holds
  one of the five and the address's own section is `Settings`, the five
  are removed. Other addresses are as in jig 03.1. `shows(section)` is
  false for a `Task` section whose task's document (the header object's
  `Body::Task`) is held and does not offer it (`pages::task::offered(&offers(doc),
  section)`); otherwise it is whether the list holds it.
Test shape: `crates/jig-domain-client/src/tests/pages.rs`:
  `an_arranged_task_page_lists_every_section_in_order`,
  `settings_stand_for_its_five_on_an_arranged_project_page`,
  `an_arranged_task_section_its_document_does_not_offer_is_not_shown`,
  each its own `#[test]`, as 04.1's. Each under 20 ms.
Proof: unit; jig 06 builds the arranged page and proves it in the
  client's world.
Don't: change the address's own section, or the address shown.
Size: at most 60 lines of production code.
Removes: nothing.

### 04.3 An arranged page opens each thing once
Scope: client.md, 5.1 ("What a page opens is what its sections need,
  each watch and read opened once for the page's generation").
Needs: 04.2.
Files: `crates/jig-domain-client/src/pages/routing.rs` (`open_section`,
  jig 03.4); `crates/jig-domain-client/src/pages/task.rs:463-538`
  (`document`, which chooses parts, the tree, the ended task and the
  activity by section: by every section shown, after jig 03.2);
  `crates/jig-domain-client/src/pages/project.rs` (the goals' watch
  beside the overview's, findings/domain.md, 8).
Shape: no new function. `open_section` called for each section of an
  arranged task page opens the task's watch once and each read its
  sections need once (history's, the activity's, the person task's);
  `task::document` handles every shown section's needs in one delivery
  (parts of `Panels`, the tree of `Plan`, the activity of `Activity`).
  On a project page, the overview's and the goals' sections open the
  goals' watch once.
Test shape: `crates/jig-domain-client/src/tests/pages.rs`:
  `an_arranged_task_page_opens_its_watch_once` (one `Subject::Task`
  watch among the requests at the address; the history's read once) and
  `an_arranged_project_page_opens_the_goals_watch_once`. Each under 20
  ms.
Proof: unit, the tests above: what `enter` emits is the domain's whole
  output toward the engine, and no world counts a client's watches by
  subject (the referee counts the person's keyed requests, testing.md,
  6). jig 06.4's story runs an arranged task page against the scripted
  engine.
Don't: open a watch per section where the subject is one.
Size: at most 100 lines of production code.
Removes: nothing.

## Interface sketches

```rust
// crates/jig-domain-client/src/pages.rs
/// The sections every task's page and every project's shows beside the
/// address's own, given by the application before the start.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arrangement {
    pub task: List<TaskSection>,
    pub project: List<ProjectSection>,
}
impl Arrangement {
    /// jig's arrangement: no section beside the address's own.
    #[must_use]
    pub fn none() -> Arrangement;
}

// crates/jig-domain-client/src/domain.rs
impl Domain {
    pub fn configure_arrangement(&mut self, arrangement: Arrangement) -> Result<(), Arrangement>;
    #[must_use]
    pub const fn arrangement(&self) -> &Arrangement;
}

// testing/jig-test-domain-client and examples/ops/domain-client
pub fn configure_arrangement(root: &mut Domain, arrangement: Arrangement) -> Result<(), Arrangement>;
```

Who owns what: this session owns `Arrangement` and
`configure_arrangement`; jig 06 reads `arrangement()` for the view's
limits, jig 06.4 and jig 08 configure one.

## Questions to raise when reached

None.

## Done when

- `heavy cargo nextest run -p jig-domain-client -E 'test(/arrange/)'`
  passes on `main`, once.
