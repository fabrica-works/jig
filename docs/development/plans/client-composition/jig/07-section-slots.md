# Session 07: an application's page shows jig's sections

An application's page names jig's sections, as it names jig's objects:
its child hands the whole list to `jig-domain-client` through the root
(`ShowSections`), the domain opens what they need under an owner of
their own, and the view root fills each section slot with jig's
section, whole. A section naming a second task is not shown, and its
slot says so; each section of jig's on an application's page says of
itself what a page says at jig's addresses (behind, rows left out, a
limit's refusal, its subject missing). jig's test application's page
shows its item's task's plan this way, and the client's world runs
client.md 17's story. `ops` follows in jig 08.

A path in this file that names no crate (`pages/task.rs:662`) is under
`crates/jig-domain-client/src/`.

## Read first

- `client/domain/root.md`, 7 ("An application's page shows jig's
  sections"), 9.
- `client/domain/client.md`, 4.1 ("Objects and sections shown"), 5.1
  (the seventh and eighth bullets: "At an application's address", "What
  a page says of itself"), 17.
- `client/domain/view.md`, 4 (slots, "Two deep", "A slot left empty"),
  10 ("a section slot counting as the section it may hold").
- `client/domain/testing.md`, 4 ("jig's test application").
- findings/domain.md, 3, 4 and 7.

## What the design asks

- `ShowSections`: the page's whole list of `Section`s each time, for the
  page's generation; an empty one names none. The domain opens and
  closes what the difference needs, under an owner of their own, so
  naming objects anew (`ShowObjects`) never closes a section's watch,
  nor naming sections an object's (client.md, 5.1).
- One subject each: a page's sections name at most one task, one
  project and one inbox filter; a section naming a second one is not
  shown, and the view's slot for it says it cannot be shown.
- On an application's page each of jig's sections says, in its own
  region, that its watches are behind, that rows past its window were
  left out, that a limit refused what it asked, and that its subject is
  missing.
- A section slot holds a reference whose owner is jig and whose token is
  the family's; the family's binding table says whether it means one of
  jig's objects or one of jig's sections (root.md, 7).
- The application's view root counts a section slot as the section it
  may hold (view.md, 10).
- The story: an application's page showing a task's plan in a section
  slot, and a second task's not shown, its slot saying so (client.md,
  17); jig's test application's page fills a card's slot and a
  section's (a task's plan) (testing.md, 4).

## Choices this plan makes

- `watches::Owner::Sections { generation }`, beside `Application {
  generation }` (`watches.rs:19-26`).
- `ShowSections` before the link is live is dropped, as `ShowObjects` is
  (`application.rs:16-18`): the family names its sections from its
  document, which comes once the link is live.
- Of several sections naming different tasks, the first in the list
  names the page's task; the later ones are not shown (the list's order
  is the family's).
- `Domain::section_status(section) -> Option<SectionStatus>`, `None`
  where the section is not shown. At jig's addresses the page keeps
  saying it as today (`Page::Work`'s fields); the view's section asks
  `section_status` only on an application's page.
- jig's test family's `Item` gains `second: Option<u64>`, a second task
  whose plan its page names too, so the story can show the second not
  shown; every other literal sets `None`.

## Rules that bind it

- "No field or predicate that its counterpart across a boundary lacks"
  (code-organization.md, 5): `jig_domain_client::Event::ShowSections`
  and the test root's `first::Request::ShowSections` carry the same
  `List<Section>`.
- `crates/jig-domain-client/src/domain.rs` grows by at most 80 lines in
  this session (code-organization.md, 6).
- Aim for the session's tests to add at most 0.4 s to the focused
  suite's median: a guide (README, "Test time"), checked by each
  landing's `Gate-Result`.

## Increments

### 07.1 The domain opens the sections an application's page names
Scope: client.md, 4.1, 5.1 ("At an application's address", "One subject
  each"); root.md, 7.
Needs: jig 03 (`Section`, `open_section`); jig 04.3.
Files: `crates/jig-domain-client/src/boundary.rs:49-52` (the event);
  `crates/jig-domain-client/src/domain.rs:615-616,660-661` (ignored
  before the start; dispatched); `crates/jig-domain-client/src/watches.rs:19-26`;
  `crates/jig-domain-client/src/pages/shown.rs`, extended (146 lines
  today, from jig 03.2 and 04.2; its `mod` is `pub(crate)`, `pages.rs:16`)
  with the handling, as `application.rs` handles objects; the arms for
  the new owner (findings/domain.md, 3; lines at origin/main after jig 06):
  `domain.rs:526-534` (`close_page`), `:541-568` (`page_behind`),
  `:876-912` (`watch_event`), `:990-993` (`retry`),
  `:1074-1089,1098-1103,1111-1129` (`terminal_watch`), `:1203-1206`
  (`fire`'s silence); `application.rs:72-80` (`close` leaves `Sections`
  watches); the open-once checks, `pages/task.rs:417` and
  `pages/project.rs:247`; the opens and sources with `Owner::Page`
  written in, which open or source under the section's owner instead
  (`Sections { generation }` on an application's page, `Page` at jig's
  addresses): `pages/task.rs:562-568` (`routing::open_once`,
  `pages/routing.rs:47-71`, for the plan's `Subject::Tree`, `:526`, and
  `Run`, `:538`), `pages/project.rs:138-140` (the project's `open`),
  `conversation/chips.rs:71` (the chips) and `pages/replies.rs:18`
  (`source`, which reads only `Page` watches); the tests that
  list `Event`'s variants (`crates/jig-domain-client/src/tests/spine.rs:70`,
  `tests/time.rs:33`, `crates/jig-view/src/tests/cards.rs:96`); the
  client's world's exhaustive match on `jig_domain_client::Event`,
  `tests/client/src/observe.rs:221-230`.
Shape: `Event::ShowSections { sections: List<Section> }`. At
  `Page::Elsewhere`, with a confirmed party and a live link: the
  domain's list of sections becomes the event's, but those naming a
  second task, project or inbox filter (Choices), and its first
  `MOST_SECTIONS` (19, `pages.rs:34`; `domain.sections`' capacity,
  `domain.rs:123`): a longer list is refused with
  `crate::pages::routing::refuse(domain, Limit::Objects)`, as
  `conversation/chips.rs:28` refuses a list past its limit, and keeps its
  first 19; `open_section`
  (jig 03.4) opens, under `Owner::Sections { generation }`, what a
  section new to the list needs and no `Sections` watch already holds
  (`open_section`'s `owner` parameter, jig 03.4); the watches of the
  sections that left
  the list, and no longer needed by one that stays, close.
  `ObjectsLeft` does not touch them; going to another page (a new
  generation) closes them with the page.
Test shape: `crates/jig-domain-client/src/tests/pages.rs`, with
  `tests/spine.rs`'s helpers:
  `an_application_page_naming_a_tasks_plan_opens_its_watch`,
  `naming_objects_anew_never_closes_a_sections_watch`,
  `a_section_naming_a_second_task_is_not_shown`,
  `an_empty_list_closes_every_sections_watch`. Each under 20 ms.
Proof: unit; 07.4's story runs it against the scripted engine through
  the test root and view.
Don't: close an `Application` watch from `ShowSections`, or a
  `Sections` watch from `ShowObjects`.
Size: at most 150 lines of production code.
Removes: nothing.

### 07.2 A section says of itself what a page says
Scope: client.md, 5.1 ("What a page says of itself, a section says on an
  application's page").
Needs: 07.1.
Files: `crates/jig-domain-client/src/pages/shown.rs` (the statuses;
  `crates/jig-domain-client/src/pages.rs` re-exports them, `pub use
  shown::SectionStatus;`, since `shown` is `pub(crate)`, `pages.rs:16`);
  `crates/jig-domain-client/src/pages/routing.rs:31-46` (`refuse`,
  `excess` on an application's page); `crates/jig-domain-client/src/application.rs:148-155`
  (a read answered none on an application's page); `domain.rs:509-536`
  (behind); `crates/jig-domain-client/src/limits.rs` (`worst_case`
  counts the statuses: `MOST_SECTIONS` of them);
  `crates/jig-view/src/sections.rs` (the section builds its status in
  its region on an application's page).
Shape: `pub struct SectionStatus { pub behind: bool, pub excess: u32,
  pub refused: Option<Limit>, pub missing: bool }`; `pub fn
  section_status(&self, section: Section) -> Option<SectionStatus>` on
  `Domain`, kept for each section of an application's page: `behind`
  from its `Sections` watches, `excess` and `refused` from what its
  windows and reads were refused, `missing` when its subject's watch
  ends not allowed or its read is answered none. `sections::section`, on
  an application's page, builds them in the section's region with the
  phrases `pages::excess` and the page's status use today
  (`JigPhrase::Behind`, `MoreRows`, the limit's refusal,
  `JigPhrase::MissingPage`).
Test shape: `crates/jig-domain-client/src/tests/pages.rs`:
  `a_section_whose_task_is_not_allowed_says_it_is_missing`,
  `a_sections_dropped_watch_shows_that_section_behind`;
  `crates/jig-view/src/tests/pages.rs`:
  `a_missing_section_says_so_in_its_own_region`. Each under 20 ms.
Proof: unit, the tests above: a story would need a fault aimed at one
  section's watch or read, which the client's world draws from its seed
  only; 07.4's story runs the sections whose statuses these are.
Don't: change what a page at jig's address says, or where.
Size: at most 120 lines of production code.
Removes: nothing.

### 07.3 jig's test application fills a section slot
Scope: root.md, 7 (the third kind of slot), 9; view.md, 4, 10;
  testing.md, 4.
Needs: 07.1; jig 05 (`sections::section`); jig 06.2 (`Limits::compose`
  with an arrangement).
Files: `testing/jig-test-domain-client/src/family.rs:37-46` (`Item`'s
  `second`), `:95-115` (`Document::sections`, beside `objects`);
  `testing/jig-test-domain-client/src/first.rs:60-80` (`Request::ShowSections`),
  `:490-505` (pushed beside `ShowObjects`), `:300-310` (an empty list
  on `Left`); `testing/jig-test-domain-client/src/route.rs:185-195`
  (forwarded); `testing/jig-test-view/src/bindings.rs:7-50`
  (`Meaning::Section`, `Bindings::section`);
  `testing/jig-test-view/src/family.rs:29-57` (a section slot after the
  card slots); `testing/jig-test-view/src/root.rs:63-144` (the slot pass:
  a section's token filled by `jig_view::sections::section`, unshowable
  when it returns false); `testing/jig-test-view/src/limits.rs:62-95`
  (the section slot counted as a task's plan's bound); every literal of
  `first::Item` / `family::Item`: `git grep -n -E 'Item \{' --
  testing/jig-test-domain-client testing/jig-test-view tests/client`
  (at the base: `testing/jig-test-domain-client/src/{family.rs,tests/family_terminals.rs,tests/root.rs}`,
  `testing/jig-test-view/src/{tests.rs,tests/family_terminals.rs,tests/native_effect.rs}`,
  `tests/client/src/stories.rs`, `tests/client/tests/{fuzzy_memory.rs,referee.rs}`).
Shape: `Item.second: Option<u64>`; `Document::sections(&self) ->
  List<Section>`: an item's `Task { task, section: Plan }`, then its
  `second`'s when there is one; `first` pushes `Request::ShowSections {
  sections }` whenever it pushes `ShowObjects`, and an empty list on
  `Left`; the route forwards it as `Event::ShowSections`. The view's
  `Meaning::Section(Section)`; the family's page leaves one slot per
  section, `SlotRef { owner: Owner::Jig, token }`; the root's first pass,
  for an `Owner::Jig` slot whose token is no object's, asks
  `view.first.section(token)` and calls `jig_view::sections::section`.
Test shape: `testing/jig-test-view/src/tests.rs`:
  `the_family_page_fills_its_tasks_plan_in_a_section_slot`,
  `a_second_tasks_section_slot_says_it_cannot_be_shown`. Each under
  20 ms.
Proof: world, `heavy cargo nextest run -p jig-client-world` once: every
  story on the family's page passes with its section slot.
Don't: build a node for jig's section outside `jig_view::sections::section`.
Size: at most 100 lines of production code (test kit code).
Removes: nothing.

### 07.4 The section-slot story
Scope: client.md, 17 ("an application's page showing a task's plan in a
  section slot, and a second task's not shown, its slot saying so").
Needs: 07.2, 07.3.
Files: `tests/client/src/stories.rs` (a world whose item names a second
  task, from `incident_world`, `:765`); `tests/client/tests/stories.rs`.
Shape: no production code.
Test shape: `tests/client/tests/stories.rs`:
  `a_family_page_shows_its_tasks_plan_and_not_a_second_one`. Reference:
  ```rust
  #[test]
  fn a_family_page_shows_its_tasks_plan_and_not_a_second_one() {
      let mut w = stories::two_plans_world(16);
      let c = w.client(7, stories::incident());
      w.person(
          c,
          vec![
              Step::See { find: Find::named(Role::Region, b"Plan"), within: WITHIN },
              Step::See { find: Find::text(b"Cannot show this part"), within: WITHIN },
          ],
      );
      w.run(Until::Done);
  }
  ```
  (`two_plans_world` is the world this increment adds, from
  `incident_world`, its item naming a second task; `stories::incident()`
  is the family's address, `stories.rs:844`; "Cannot show this part" is
  the test view's `Word::Unshowable`, `testing/jig-test-view/src/words.rs:127`.)
  Under 0.5 s.
Proof: world, `heavy cargo nextest run -p jig-client-world -E
  'test(a_family_page_shows_its_tasks_plan)'` once: the story passes
  under the referee, the outline kept with the section inside the main.
Don't: add a step the story does not need.
Size: no production code; at most 50 lines of test kit code.
Removes: nothing.

## Interface sketches

```rust
// crates/jig-domain-client/src/boundary.rs
pub enum Event {
    // …
    /// An application's page names jig's sections for the page's generation, the whole list.
    ShowSections { sections: List<Section> },
}

// crates/jig-domain-client/src/watches.rs
pub enum Owner {
    Frame,
    Page { generation: u32 },
    Application { generation: u32 },
    /// The sections an application's page names.
    Sections { generation: u32 },
}

// crates/jig-domain-client/src/pages/shown.rs
/// What a section of jig's says of itself on an application's page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectionStatus { pub behind: bool, pub excess: u32, pub refused: Option<Limit>, pub missing: bool }
impl Domain {
    pub fn section_status(&self, section: Section) -> Option<SectionStatus>;
}
```

Who owns what: this session owns `Event::ShowSections`,
`Owner::Sections` and `SectionStatus`; jig 08 uses them from `ops`'s
root by name.

## Questions to raise when reached

None.

## Done when

- `heavy cargo nextest run -p jig-client-world -E 'test(a_family_page_shows_its_tasks_plan)'`
  passes on `main`, once.
