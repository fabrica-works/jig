# Session 08: `ops` arranges jig's pages and shows its lead's plan

`ops`, jig's example application, becomes the first application to
take jig's pages as a library: its incident's page shows the lead
task's plan as jig's plan section, in a section slot; and its task's
page shows every section together, its view root building the main
from the task's header and its overview, activity, plan and history,
with the budget and the connectors' panels in an aside. Its client
world runs a story on each, under the referee that holds its
arrangement to the outline.

## Read first

- `examples.md`, 8 ("its lead … its plan as jig's plan section, in a
  section slot"; "Its arrangement"; "Its owners").
- `client/domain/pages.md`, 12.3 ("Its own arrangement", "What every
  arrangement keeps").
- `client/domain/root.md`, 7 and 9; `client/domain/view.md`, 4 and 4.1.
- jig 07's session file, its interface sketch (`ShowSections`), and
  07.3's shape (the test application's section slot, which this
  session copies).

## What the design asks

- The incident's lead is shown by jig's own documents: its chip, report
  and proposals as jig's cards, and its plan as jig's plan section, in a
  section slot (examples.md, 8).
- Its arrangement: jig's frame and jig's arrangement of every page but a
  task's, which shows every section together, the address's own marked
  current: its view root builds the main from the task's header and its
  overview, activity, plan and history, and puts its budget and its
  connectors' panels in an aside (examples.md, 8; pages.md, 12.3).
- What every arrangement keeps: the outline, cards whole, a section only
  where the domain shows it (pages.md, 12.3).

## Choices this plan makes

- The aside is one `Role::Complementary`, unnamed, with no region name
  of `ops`'s own (an application's own region names are optional,
  pages.md 12.4).
- The arranged task page's heading is jig's (`pages::heading` with
  `JigPhrase::TaskPage`), as jig's main builds it.
- The arrangement is given in `ops`'s client constructor, beside its
  provider (`tests/ops/src/client/app.rs:155-166`), as an application
  gives it before the start.

## Rules that bind it

- `ops`'s existing stories keep their steps: they find by role, name and
  text, so an arranged task page serves them as jig's did (testing.md,
  3).
- Cards and sections are built only by `jig_view::cards::card` and
  `jig_view::sections::section` (objects.md, 2; code-organization.md,
  4).
- Aim for the session's tests to add at most 0.4 s to the focused
  suite's median: a guide (README, "Test time"), checked by each
  landing's `Gate-Result`.

## Increments

### 08.1 The incident's lead's plan, in a section slot
Scope: examples.md, 8 (the lead's plan); root.md, 7.
Needs: jig 07.1 and 07.3 (the shapes it copies).
Files: `examples/ops/domain-client/src/observability.rs:85-86` (the
  request), `:273` (`ObjectsLeft` on `Left`: an empty `ShowSections`
  beside it) and `:398` (`ShowObjects`: `ShowSections` beside it), the
  incident's document's `sections` beside its `objects`, a native
  `Value::Incident` as `objects` names its lead
  (`examples/ops/domain-client/src/observability/documents.rs:186-190`);
  `examples/ops/domain-client/src/route.rs:285-288` (observability's arm
  forwards it; infrastructure's, `:246-249`, is unchanged);
  `examples/ops/domain-client/src/limits.rs:18-19` (`max_out(..)
  .checked_mul(4)` becomes `checked_mul(5)`, the further `ShowSections`
  handoff, as jig 07.3 did at 2954c546c) and `:59-61`
  (`observability_payload` takes `.max(List::<jig_domain_client::pages::Section>::worst_case(1)?)`);
  `examples/ops/view/src/bindings.rs` (`Meaning::Section`, a
  `section(token)` lookup); `examples/ops/view/src/observability.rs:84-89`
  (`show_incident`: a section slot after the lead's chip, `:87`) and
  `:426-431` (`native_incident`: the same, after `:430`);
  `examples/ops/view/src/root.rs:106-190` (the slot pass fills a
  section's token by `jig_view::sections::section`, unshowable when it
  returns false); `examples/ops/view/src/limits.rs:40` (the section
  slot counted as a task's plan's bound: `task_section_nodes(..,
  TaskSection::Plan)? + 4` added to `jig_cards`); `examples/ops/domain-client/src/tests.rs`
  and `examples/ops/view/src/tests.rs` for their tests.
Shape: `observability::Request::ShowSections { sections:
  List<Section> }`; the incident's sections are `[Section::Task { task:
  lead, section: TaskSection::Plan }]` when it has a lead, else empty,
  for a `Document::Incident` and a native `Value::Incident` alike (its
  match names each native `Value` variant: no `_` arm, clippy's
  `wildcard_enum_match_arm`);
  pushed wherever `ShowObjects` is, and an empty list where
  `ObjectsLeft` is; `route.rs` forwards it as
  `jig_domain_client::Event::ShowSections`. The view as jig 07.3's test
  view.
Test shape: `examples/ops/domain-client/src/tests.rs`:
  `an_incident_with_a_lead_names_its_plan_section`;
  `examples/ops/view/src/tests.rs`:
  `the_incident_page_fills_its_leads_plan_in_a_section_slot`. Each under
  20 ms.
Proof: world, 08.3's first story.
Don't: name a section for an incident with no lead.
Size: at most 80 lines of production code.
Removes: nothing.

### 08.2 `ops`'s arranged task page
Scope: examples.md, 8 ("Its arrangement"); pages.md, 12.3 ("Its own
  arrangement").
Needs: jig 06.1 and 06.2 (an arranged page; `Limits::compose` with an
  arrangement).
Files: `tests/ops/src/client/app.rs:155-166` (`new`:
  `configure_arrangement`), `:114-126` (`view_bounds`: its 1,024 nodes
  become 2,048 if the arranged page needs them; the builder's capacity
  assertion says so, and the report gives the number);
  `tests/ops/src/joint/fixture.rs:87-100` (`client()`, the joint
  world's second ops client: `client::configure_arrangement(&mut domain,
  view::arrangement())` after `:95`; its bounds at `:65-78` as
  `app.rs`'s `view_bounds`);
  `examples/ops/view/src/root.rs:80-104` (`build`: at a task's address,
  its own main and aside); `examples/ops/view/src/limits.rs:46-53`
  (`compose` with `ops`'s arrangement in place of `Arrangement::none()`
  at `:49`) and a new `pub fn arrangement() -> Arrangement` there,
  re-exported at `examples/ops/view/src/lib.rs:16` (`pub use
  limits::{Limits, arrangement, worst_case};`);
  `examples/ops/domain-client/src/domain.rs:198-204`
  (`configure_arrangement`, from jig 04.1); `crates/jig-view/src/pages.rs:56`
  (`excess` becomes `pub`, with a one-line doc comment);
  `examples/ops/view/src/tests.rs`; for the joint world's missing
  acknowledgement (ruling 117; protocol/client.md, section 7: only a
  restart excuses telling the core of a delivery taken),
  `tests/client/src/joint/referee.rs` (`Referee::restarted`, and its
  unit test) and `tests/ops/src/joint/engine.rs:202-206` (`cold` calls
  `peers.protocol.restarted()`).
Shape: `jig_ops_view::arrangement()` is `Arrangement { task: [Overview,
  Activity, Plan, History, Budget, Panels], project: [] }`; the client
  constructor gives it to `configure_arrangement`, and `ops`'s
  `Limits::of` passes it to `jig_view::Limits::compose`. `build`, when the
  domain's page is `Page::Work { address: Address::Task { task, .. } }`:
  `b.open(Role::Page)`; `jig_view::frame::banner`;
  `jig_view::frame::navigation` with `observability::NAV`, as `build`'s
  `jig_view::jig` call passes it (`root.rs:91`); `b.open(Role::Main)`;
  `jig_view::pages::heading(.., JigPhrase::TaskPage, &[Hole::Number(task)])`;
  `jig_view::sections::task::header`; `jig_view::sections::section` for
  the overview, activity, plan and history; `jig_view::pages::excess`,
  as jig's task page calls it (`crates/jig-view/src/pages.rs:121`);
  `b.close()`;
  `b.open(Role::Complementary)`; `sections::section` for the budget and
  the panels; `b.close()`; `jig_view::after_page`; `b.close()`. Every
  other page: `jig_view::jig`, as today.
Test shape: `examples/ops/view/src/tests.rs`:
  `an_ops_task_page_puts_budget_and_panels_in_an_aside` (the
  complementary holds the regions `jig-task-budget` and
  `jig-task-panels`; `outline::check` is `None`) and
  `an_ops_task_page_marks_the_addresss_own_section_current`. Each under
  20 ms. `tests/client/src/joint/referee.rs`:
  `only_a_restart_excuses_a_taken_delivery` (a consumed delivery fails
  `check` until `acknowledged`; another fails it until `restarted`,
  after which `acknowledgements()` is still 1), under 5 ms.
Proof: world, `heavy cargo nextest run -p jig-ops-world` once: `ops`'s
  stories that visit a task's plan and panels pass on the arranged page,
  the referee holding it to the outline.
Don't: build tabs on the arranged page (every section is shown); build a
  section's nodes outside `sections::section`.
Size: at most 80 lines of production code.
Removes: nothing.

### 08.3 `ops`'s stories on the lead's plan
Scope: examples.md, 8.
Needs: 08.1, 08.2.
Files: `tests/ops/tests/client_stories.rs`; `tests/ops/src/client/stories.rs`
  (fixtures, if a new one is needed).
Shape: no production code.
Test shape: `tests/ops/tests/client_stories.rs`:
  `p_accepts_the_restart_in_its_leads_plan_on_the_incidents_page`.
  Reference, beside `accept_restart` (`:32`), with its `see` and `press`
  helpers (`:16-22`):
  ```rust
  #[test]
  fn p_accepts_the_restart_in_its_leads_plan_on_the_incidents_page() {
      let mut world = stories::incident_world(503);
      let p = world.client(7, stories::incident());
      world.person(
          p,
          vec![
              see(b"R restart checkout"),
              see(b"Restart checkout in production"),
              press(b"Accept"),
              press(b"Confirm"),
              see(b"Accepted"),
          ],
      );
      assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
      assert_eq!(world.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 1);
  }
  ```
  Under 0.5 s.
Proof: world, `heavy cargo nextest run -p jig-ops-world -E
  'test(/p_accepts/)'` once: P accepts the restart from the incident's
  page, through jig's plan section in its slot.
Don't: change `accept_restart`'s steps.
Size: no production code.
Removes: nothing.

## Interface sketches

```rust
// examples/ops/domain-client/src/observability.rs
pub enum Request {
    // …
    /// The incident's page names jig's sections: its lead's plan.
    ShowSections { sections: List<jig_domain_client::pages::Section> },
}
```

## Questions to raise when reached

- If the incident's page shows the restart's proposal twice (as a card
  slot and inside the plan section), that is one object with one card
  in two places (objects.md, 1); the person's `press(b"Accept")` finds
  the first. No question unless the referee objects.

## Done when

- `heavy cargo nextest run -p jig-ops-world` passes on `main`, once.
- `git grep -n 'configure_arrangement' origin/main -- tests/ops examples/ops`
  prints the constructor's call.
