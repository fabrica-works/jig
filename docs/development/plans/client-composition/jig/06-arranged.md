# Session 06: more sections together, counted, and the owed cards

jig's main shows every section the domain lists for an arranged page,
the tabs left out when the page shows every section its subject offers;
the view's limits count the sections an arrangement shows together; the
cards a page still owes are decided from the domain, never from the
tree built so far; and the client's world runs client.md 17's story of
a task's page arranged with every section together.

## Read first

- `client/domain/view.md`, 4.1 (`pages::main`, `after_page`), 10
  (limits that take the arrangement).
- `client/domain/pages.md`, 12.1 ("After the page"), 12.3 ("More
  sections together", "The tabs").
- `client/domain/client.md`, 5 (owed cards: "Leaving, whichever comes
  first"), 5.1, 17 (the arranged story).
- `client/domain/testing.md`, 4 ("jig's test application").
- findings/view.md, 5 and 6.

## What the design asks

- jig's main shows the address's section and those the arrangement adds,
  in pages.md 12.2's order; the tabs list every offered section, the
  address's own marked current, and are left out when the page shows
  every section its subject offers (pages.md, 12.3).
- `Limits::of` and `Limits::compose` count jig's pages with the sections
  the application's `Arrangement` adds beside each address's own; a page
  showing sections together counts their sum (view.md, 10).
- A card is owed while it is deciding, leaving, or unknown whether made,
  and no section of the page and no card slot of the application's page
  shows it: its object named by none of the page's sources, or a
  proposal that left the plan's rows while the page shows that plan. It
  is decided from the domain, never from the tree built so far (view.md,
  4.1).
- The story: a task's page arranged with every section together, a
  proposal accepted in its plan while its conversation shows it
  (client.md, 17), its client given an arrangement naming every
  `TaskSection` (testing.md, 4).

## Choices this plan makes

- `Limits::of(client, arrangement)` and `Limits::compose(client, words,
  arrangement, application_nodes, slot_nodes, slot_depth)` take
  `&Arrangement`; `Limits::of_words` keeps jig's arrangement (none), as
  the design names only the two.
- A page's bound at a task's or a project's address is its heading, its
  header and its tabs, plus the sum of the bounds of the sections the
  arrangement adds and the largest of the other sections (the address's
  own may be any). With no arrangement it is today's value.
- `retained_outside_plan` moves into `jig-domain-client` as
  `Domain::owes(id) -> bool`, which also checks the card's state; the
  view asks it.

## Rules that bind it

- The literal bounds of `crates/jig-view/src/tests/page_capacity.rs:845-853`
  do not change in this session: they are the bounds with no
  arrangement.
- A card shows once on a page: where a section builds a card by key, it
  skips an object the domain owes, as `sections/task.rs` does for a
  proposal with no source today (`pages/task.rs:257-261` at the base).
- Aim for the session's tests to add at most 0.4 s to the focused
  suite's median (a guide: README, "Test time");
  an arranged client's tree is larger, so only 06.4 builds one in a
  world (06.1's unit tests build arranged pages at `CLIENT`).

## Increments

### 06.1 jig's main shows an arranged page
Scope: pages.md, 12.3 ("More sections together", "The tabs"); view.md,
  4.1 (`pages::main`).
Needs: jig 05 (`sections::*`); jig 04.2 (an arranged page's sections).
Files: `crates/jig-view/src/pages.rs` (`main`; it already builds
  `sections::section` for each of `client.sections()`, `:120-122,129-131`);
  `crates/jig-view/src/sections/task.rs` and `sections/project.rs`
  (`tabs`, `section`); tests in `crates/jig-view/src/tests/work_pages.rs`,
  and a start given an `Arrangement` beside `start` in
  `crates/jig-view/src/tests/cards.rs:165-183` (`configure_arrangement`
  between `Domain::new(&CLIENT, 1)` and `Event::Start`), for
  `work_pages.rs`'s `entered` and `task` (`:38,65`); its `task()` fixture
  already offers the six sections (`:722-746`).
Shape: `main` builds, at a task's or a project's address: `heading`,
  `header`, `tabs` unless every section the tabs would list is shown
  (for a task, every section `pages::task::offers`; for a project,
  `Overview`, `Goals`, `Tasks` and `Notes`, each in `client.sections()`,
  and `Settings` or each of its five: `Settings` holds its five,
  client.md 5.1, "Each section once", and `arrange` keeps it whole,
  `crates/jig-domain-client/src/pages/shown.rs:114-143`, so the ten its
  tabs list are never all in the list), then
  `sections::section` for each of `client.sections()`. `tabs` marks the
  address's own section current, as in jig 05.1; when the page shows
  more than one section of its subject, the address's own section's
  region is marked `State::Current` (pages.md, 12.3: "The address's
  section is the one marked current"), so it stays marked when the tabs
  are left out. `sections::section` only dispatches; the mark goes after
  `pages::work::section` returns, at the three sites that open the
  region: `sections/task.rs:138`, `sections/project.rs:88`, and the
  Settings loop at `sections/project.rs:119` (an address at one of the
  five, with `Settings` arranged).
Test shape: `crates/jig-view/src/tests/work_pages.rs`, with a domain given
  `configure_arrangement` (every `TaskSection`) before `start`:
  `an_arranged_task_page_shows_its_six_sections_in_order` and
  `an_arranged_task_page_showing_every_section_has_no_tabs` and
  `the_addresss_own_section_is_marked_current_on_an_arranged_page`. Each
  under 20 ms.
Proof: unit; 06.4 runs an arranged page in the client's world.
Don't: show a header twice; reorder sections.
Size: at most 60 lines of production code.
Removes: nothing.

### 06.2 The view's limits count an arrangement
Scope: view.md, 10.
Needs: 06.1.
Files: `crates/jig-view/src/limits.rs:60-133` (`of`, `compose`,
  `with_phrase_bytes`); `crates/jig-view/src/sizes/pages.rs` (`task_nodes`
  `:117-119`, `project_nodes` `:180-188`, `page_nodes` `:211-221`), with a
  companion per section and header (`sections::*`'s `_nodes`
  functions, in `sizes/`); the callers of `jig_view::Limits::of` (3:
  `crates/jig-view/src/tests/cards_sizes.rs:481,489,492`) and of
  `jig_view::Limits::compose` (6: `git grep -n 'Limits::compose(' --
  '*.rs'` lists them; findings/view.md, 6, names those that pin the
  formula: `crates/jig-view/src/tests/page_capacity.rs:511-531`,
  `testing/jig-test-view/src/limits.rs:28-41,74-84`,
  `examples/ops/view/src/limits.rs:39-46`,
  `tests/ops/src/client/app.rs:120`); `testing/jig-test-view/src/limits.rs:78`
  (its `compose` call passes `&Arrangement::none()`). Unchanged, with
  their callers:
  `page_nodes` (its 8 callers: `git grep -n 'page_nodes(' -- '*.rs'`),
  and `jig_test_view::Limits::of` and `for_world` (their 24 callers in
  `testing/jig-test-view`, `tests/client/src/test.rs:115` and
  `tests/ops/src/client/app.rs:120`).
Shape: `pub fn of(client: &jig_domain_client::Limits, arrangement:
  &Arrangement) -> Option<Self>`; `pub fn compose(client, words,
  arrangement: &Arrangement, application_nodes: u32, slot_nodes: u32,
  slot_depth: u32) -> Option<Self>`; a new `pub fn arranged_nodes(client,
  slot_nodes: u32, arrangement: &Arrangement) -> Option<u32>` in
  `sizes/pages.rs`, the bound of Choices above, which `Limits` uses;
  `page_nodes(client, slot_nodes)` stays, equal to `arranged_nodes` with
  `Arrangement::none()`. Each caller of `of` and `compose` passes
  `&Arrangement::none()`; `ops` passes its own in jig 08.2.
Test shape: `crates/jig-view/src/tests/page_capacity.rs`:
  `an_arranged_task_page_counts_the_sum_of_its_sections` (the bound with
  every `TaskSection` arranged equals the heading, header and tabs plus
  each section's companion, summed in the test from the companions'
  literal values at `CLIENT`: the implementer writes the test with a
  placeholder, runs `heavy cargo nextest run -p jig-view -E
  'test(an_arranged_task_page_counts_the_sum_of_its_sections)'` once,
  takes the values its failure prints, and names them in the commit
  body) and `no_arrangement_keeps_each_page_bound` (the
  `:845-853` values, unchanged). Each under 20 ms.
Proof: unit; 06.4's story builds an arranged tree within these limits
  (the builder asserts its capacity).
Don't: change a bound with no arrangement.
Size: at most 150 lines of production code.
Removes: nothing.

### 06.3 The owed cards, from the domain
Scope: view.md, 4.1 (`after_page`); pages.md, 12.1 (the owed cards).
Needs: jig 05 (the sections' card sites moved to `sections/`).
Files: `crates/jig-view/src/pages.rs:164-244` (`retained_outside_plan`
  `:164-197`, `after_page` `:199-244`, its scan `:213-219`);
  `crates/jig-domain-client/src/domain.rs` (`owes`); the sections' card
  sites (findings/view.md, 5, at their paths after jig 05):
  `sections/task.rs:226-227,254-255,326-327` and its skip of a proposal
  with no source `:310-320`, `sections/project.rs:256-257,264-265`,
  `sections/inbox.rs:241-242,302-303`, `sections/conversations.rs:42-44`;
  the conversation's card sites, `crates/jig-view/src/turns/call.rs:58-59`
  and `turns/activity.rs:60-61,76-77` (through `turns::shows_object`,
  `turns/objects.rs:9`), as the designer's answer to the question of
  the ready check of jig-06.1 says: 06.3 starts only after that answer's
  ruling; tests in
  `crates/jig-view/src/tests/{pages.rs,task_proposals.rs}`.
Shape: `pub fn owes(&self, id: Id<Object>) -> bool` on `Domain`: the
  object is shown, its card is `Deciding`, `Leaving` or `Unknown`, and
  either no source names it, or it is a proposal the shown plan's rows
  no longer list while `shows(Section::Task { task, section: Plan })`
  for the shown task (today's `retained_outside_plan`). `after_page`
  builds a card for each object the domain owes, with no look at the
  tree. Each card site above builds no card for an object
  `client.owes(id)`.
Test shape: `crates/jig-view/src/tests/task_proposals.rs`:
  `a_proposal_leaving_the_plan_is_owed_once` (the card appears once,
  in `jig-owed`); `crates/jig-domain-client/src/tests/pages.rs`:
  `an_object_no_source_names_is_owed_while_deciding`;
  `crates/jig-view/src/tests/page_capacity.rs`:
  `the_most_owed_cards_fit_the_pages_bound` (at `CLIENT`, as many
  objects deciding with no source as `CLIENT.objects`: the tree fits
  `Limits::of(&CLIENT)`, as `limits.rs:67` takes it at the base; 06.2,
  if it lands first, gives it `&Arrangement::none()` with its other
  callers). Each under 20 ms. A test over the objects is a `for` loop:
  the lints refuse `.iter().any(..)` ("no closures: a for loop").
Proof: world, `heavy cargo nextest run -p jig-client-world -p
  jig-ops-world` once: the stories where a card leaves after its
  source stopped naming it (a proposal accepted in the inbox leaving its
  chat; two tabs deciding one thing) pass, each card seen once by the
  referee.
Don't: read the tree in `after_page`; change when a card leaves.
Size: at most 80 lines of production code.
Removes: `retained_outside_plan` in `jig-view`; the scan of
  `b.tree.nodes()` in `after_page`.

### 06.4 The arranged story
Scope: client.md, 17 ("a task's page arranged with every section
  together, a proposal accepted in its plan while its conversation shows
  it"); testing.md, 4.
Needs: 06.1, 06.2, 06.3; jig 04.3 (an arranged page opens what every
  shown section needs: without it the plan's proposals and the activity
  never load together).
Files: `tests/client/src/test.rs:103-140` (`fixture`, `new`: an
  `arranged` constructor beside `new`, its fixture built once per
  limits and arrangement); `tests/client/src/stories.rs` (a world for
  the story, from `proposal_world`'s and `chat_ready`'s fixtures,
  `:296,423`); `tests/client/tests/stories.rs` (the story).
Shape: `pub fn arranged(seed: u64) -> (domain::Domain,
  view::View<'static>)`, a `ClientConstructor` (`tests/client/src/harness/world.rs:15`)
  with `limits()` (`tests/client/src/test.rs:17`), built as `Test::new`
  (`test.rs:150`) builds through `new(&limits(), seed)` (`:125`), and
  `domain::configure_arrangement(&mut root, Arrangement { task: every
  TaskSection, project: empty })`, the view's bounds from
  `view::Limits::for_world(limits, words, family, 2048, 256)`: `new`'s
  call (`test.rs:115`) with twice its 1,024 nodes, as an arranged task
  page shows six sections at once; the builder's capacity assertion
  fails the story if it is still too small, and the report gives the
  number that fits. The story's client is
  made with `w.client_with(7, address, test::limits(), test::arranged)`
  (`world.rs:185-193`).
Test shape: `tests/client/tests/stories.rs`:
  `a_proposal_accepted_in_an_arranged_plan_leaves_while_its_chat_shows`.
  Reference:
  ```rust
  #[test]
  fn a_proposal_accepted_in_an_arranged_plan_leaves_while_its_chat_shows() {
      let mut w = stories::arranged_proposal_world(15);
      let c = w.client_with(7, stories::chat(), test::limits(), test::arranged);
      w.person(
          c,
          vec![
              Step::See { find: Find::named(Role::Log, b"Conversation"), within: WITHIN },
              Step::Press { find: Find::named(Role::Button, b"Accept").within(Box::new([plan()])) },
              Step::Press { find: Find::named(Role::Button, b"Confirm") },
              Step::Gone { find: Find::named(Role::Button, b"Accept").within(Box::new([plan()])), within: WITHIN },
              Step::See { find: Find::named(Role::Log, b"Conversation"), within: WITHIN },
          ],
      );
      w.run(Until::Done);
      assert_eq!(w.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 1);
  }
  ```
  (`plan()` is `Target::Role { role: Role::Region, name: Box::from(*b"Plan") }`;
  `arranged_proposal_world` is the world this increment adds to
  `stories.rs`, from `proposal_world`, its task an agent's with a
  conversation and a proposal in its plan; "Conversation", "Accept" and
  "Confirm" are the English phrasebook's, `english.rs:470,1164,1258`.)
  Under 0.5 s.
Proof: world, `heavy cargo nextest run -p jig-client-world -E
  'test(arranged)'` once: the story passes under the referee, which
  checks the outline and that the proposal is decided once.
Don't: give any other client of the world an arrangement.
Size: at most 60 lines (test kit code).
Removes: nothing.

## Interface sketches

```rust
// crates/jig-view/src/limits.rs
impl Limits {
    pub fn of(client: &jig_domain_client::Limits, arrangement: &Arrangement) -> Option<Self>;
    pub fn compose(
        client: &jig_domain_client::Limits,
        words: &crate::words::Words,
        arrangement: &Arrangement,
        application_nodes: u32,
        slot_nodes: u32,
        slot_depth: u32,
    ) -> Option<Self>;
}

// crates/jig-domain-client/src/domain.rs
impl Domain {
    /// Whether the page still owes this object's card (view.md, 4.1).
    pub fn owes(&self, id: Id<Object>) -> bool;
}
```

## Questions to raise when reached

- **The owed cards' count:** findings/view.md, 6 could not say whether
  the page's bound covers the owed cards. If 06.3's test of the most
  owed cards the domain's limits allow does not fit `Limits::of`, the
  increment adds them to `after_page`'s companion and the report gives
  the bound before and after (the values `the_most_owed_cards_fit_the_pages_bound`
  prints, run once). A time past the session's share is the README's
  "Test time" guide: the landing stands.

## Done when

- `git grep -n 'retained_outside_plan\|b.tree.nodes()' origin/main --
  crates/jig-view/src/pages.rs` prints nothing.
- `heavy cargo nextest run -p jig-client-world -E 'test(arranged)'`
  passes on `main`, once.
