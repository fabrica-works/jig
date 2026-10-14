# Session 05: the sections and headers as functions of jig's view

jig's pages are rebuilt from functions an application can call:
`jig_view::sections::section`, which builds one section the domain shows
as a region with its level-2 heading and its region name, and the
headers and tabs of the inbox, a task and a project. `pages::main`
becomes jig's arrangement of them: the page's heading, its header, its
tabs, then each section the domain lists. At jig's addresses the domain
lists one section (the inbox's two), so what a person reads is the same,
but for the tabs, which become tabs leading to each section's address,
the address's own marked current. jig 06 shows more sections together,
counts them in the view's limits and decides the owed cards from the
domain.

## Read first

- `client/domain/view.md`, 2 (the module tree: `sections/`), 4 ("A
  section is a function"), 4.1 ("The sections", "The pages").
- `client/domain/pages.md`, 12.2 (sections, headers, region names, the
  regions named inside sections), 12.3 ("The tabs").
- findings/view.md, 7 and 8.
- jig 03's interface sketch (`Section`, `Domain::shows`).

## What the design asks

- `sections::section(b, client, words, bindings, section: Section) ->
  bool` builds one section: a region with its level-2 heading and its
  region name (`jig-task-plan`, …), or nothing, returning false, when
  the domain does not show that section on this page (view.md, 4.1).
- The headers, each shown with any of its page's sections:
  `sections::inbox::header`; `sections::task::header` and
  `sections::task::tabs`; `sections::project::header` and
  `sections::project::tabs`; each builds nothing and returns false while
  no section of its subject is shown.
- `pages::main`: the page's heading, its header, its tabs, then each
  section the domain shows, in pages.md 12.2's order.
- The tabs list every section the task or the project offers, each
  leading to its address, the address's own marked current (pages.md,
  12.3).
- The regions named inside sections: `jig-conversation` (the activity's
  log), `jig-composer`, `jig-new-chat` (a form), `jig-new-goal` (a form),
  `jig-notes-search` (a search), `jig-hosts`, `jig-accounts`,
  `jig-connectors`, `jig-capacity` (pages.md, 12.2).

## Choices this plan makes

- **Files move to `sections/`** with `git mv`, in the increment that
  rebuilds them: `pages/task.rs` and `pages/task/` to `sections/task.rs`
  and `sections/task/`; `pages/inbox.rs`, `pages/conversations.rs`,
  `pages/deployment.rs` to `sections/`; `pages/project.rs` and
  `pages/project/` to `sections/project.rs` and `sections/project/`.
  `pages/work.rs` and `pages/not_the_work.rs` stay. `pages.rs` keeps
  `heading`, `main`, `not_the_work`, `after_page` and `jig`, public, and
  the four helpers the sections share, `cut`, `edges`, `excess` and
  `object_key`, made `pub(crate)`. `lib.rs:104`'s alias
  `pub use pages::project::owner as owner_project` follows the move.
- **A section's region is `work::section`'s** (`pages/work.rs:11-19`):
  it takes the region name as a parameter, and each of its callers
  passes its own (`pages/task.rs:91`, `pages/project.rs:75,96,299`,
  `pages/deployment.rs:65` at the base).
- **Tabs are `Role::Tab`** inside the `Role::Tabs` they already sit in,
  built by a new `work::tab`, bound to the address as the button is
  today, the address's own marked `State::Current` where it was
  `State::Selected`. `work::control` (`pages/work.rs:20-37`) is
  unchanged: the plan's filters, the history's scope and the goals'
  controls keep calling it (decisions.md, 15).
- **Page-level statuses stay in the main, after the sections,** as they
  are after the page's content today: the inbox's goal context, more
  rows, refusal, and `pages::excess`.

## Rules that bind it

- **A story keeps its steps:** a step that presses a tab by
  `Find::named(Role::Button, …)` becomes `Find::named(Role::Tab, …)`, and
  nothing else in a story changes. The literal bounds of
  `crates/jig-view/src/tests/page_capacity.rs:845-853` change only by
  the nodes an increment adds (a header's region: 1 each), named in the
  commit body.
- No new phrase; no phrase removed.
- `pages/project/owner.rs` (1,167 lines) moves whole and grows by at
  most 40 lines; it is not split by this plan (code-organization.md, 6).
- Aim for the session's tests to add at most 0.2 s to the focused
  suite's median: a guide (README, "Test time"), checked by each
  landing's `Gate-Result`.

## Increments

### 05.1 A task's sections, header and tabs
Scope: view.md, 4.1 ("The sections": the task's); pages.md, 12.2 (a
  task's rows; `jig-conversation`, `jig-composer`), 12.3 ("The tabs").
Needs: jig 02.3 (`pages::main`); jig 03.2 (`Section`, `Domain::shows`).
Files: `git mv crates/jig-view/src/pages/task.rs crates/jig-view/src/sections/task.rs`
  and `pages/task/` to `sections/task/`; new `crates/jig-view/src/sections.rs`;
  `crates/jig-view/src/lib.rs` (`pub mod sections;`);
  `crates/jig-view/src/pages.rs` (`main`'s task arm);
  `crates/jig-view/src/pages/work.rs:11-19` (`section` takes a
  `JigRegion`, with its callers `pages/task.rs:91`,
  `pages/project.rs:75,96,299`, `pages/deployment.rs:65`) and a new
  `work::tab` beside `control` (`:20-37`, unchanged), called by the
  task's tabs (`pages/task.rs:77-89`);
  `crates/jig-view/src/turns.rs:22` (the conversation's log carries
  `JigRegion::Conversation`); `crates/jig-view/src/confirmation.rs:226-260`
  (the composer's field, `Purpose::Composer`, inside a `Role::Region`
  with `JigRegion::Composer`: decisions.md, 13);
  the tests that call `crate::pages::task`:
  `crates/jig-view/src/tests/{historical_pages.rs:431,task_proposals.rs:95}`;
  every test or story that finds a task's tab as a button: `git grep -n
  -E 'Role::Button, b"(Overview|Activity|Plan|History|Budget and authority|Connectors)"'
  -- '*.rs'` (at the base: `tests/client/src/joint/{deployment_story,engine_stories,stories}.rs`,
  `tests/client/tests/stories.rs`; with `tests/ops` if it lists any).
Shape: `sections/task.rs`: `pub fn header(b, client, words, bindings) ->
  bool` (today's `:29-75`, inside a region with `JigRegion::TaskHeader`),
  `pub fn tabs(b, client, words, bindings) -> bool` (`:77-89`, its
  `Role::Tabs` with `JigRegion::TaskTabs`), and `pub(crate) fn
  section(b, client, words, bindings, task: u64, section: TaskSection)
  -> bool` (`:90-102`'s match, the region named
  `JigRegion::Task<Section>`). Each returns false and builds nothing
  when `client.shows(..)` holds no section of the task.
  `sections.rs`: `pub fn section(b: &mut Builder, client: &Domain, words:
  &Words, bindings: &mut JigBindings, section: Section) -> bool`,
  matching `Section` and calling the subject's module; the arms of
  subjects not yet moved call today's page functions inside their
  region until 05.2 and 05.3 move them. `pages::main` at a task's
  address: `heading`, `sections::task::header`, `sections::task::tabs`,
  then `sections::section` for each of `client.sections()`, then
  `excess`.
Test shape: `crates/jig-view/src/tests/work_pages.rs` (`task_sections`,
  `:94`, and its helpers): `a_task_section_not_shown_builds_nothing`
  (`sections::section` with another task's `Section` returns false and
  the builder's length is unchanged),
  `the_tabs_mark_the_addresss_own_section_current`, and
  `each_task_section_carries_its_region_name` (a table over the six
  sections, the region names as literals). Each under 20 ms.
Proof: world, `heavy cargo nextest run -p jig-client-world -p jig-ops-world`
  once: every story that visits a task's page passes, the ones that
  press a tab pressing it as a tab.
Don't: change a section's content, a phrase, or the order of a task's
  tabs.
Size: at most 150 lines of production code, moves not counted (`git
  diff --numstat -M` counts a moved file's changed lines only).
Removes: `pages::task` (moved).

### 05.2 The inbox, the conversations and the deployment as sections
Scope: view.md, 4.1; pages.md, 12.2 (their rows; `jig-new-chat`,
  `jig-hosts`, `jig-accounts`, `jig-connectors`, `jig-capacity`).
Needs: 05.1.
Files: `git mv` of `crates/jig-view/src/pages/{inbox,conversations,deployment}.rs`
  to `crates/jig-view/src/sections/`; `crates/jig-view/src/sections.rs`;
  `crates/jig-view/src/pages.rs` (`main`'s arms); the tests that call
  them: `crates/jig-view/src/tests/pages.rs:26-31`,
  `crates/jig-view/src/tests/owner_inbox.rs:103`; the deployment's owner
  path, `crates/jig-view/src/owner_deployment.rs` (`render`, called at
  `pages/deployment.rs:12`).
Shape: `sections/inbox.rs`: `pub fn header(..) -> bool` (the filters and
  counts, `pages/inbox.rs:19-39`, in a region with
  `JigRegion::InboxHeader`); `pub(crate) fn group(.., group:
  InboxReadGroup, project: Option<u32>) -> bool` (one iteration of
  `:40-62`, its region named `InboxNeeds` or `InboxUpdates`).
  `sections/conversations.rs`: `pub(crate) fn section(.., project:
  Option<u32>) -> bool` (`pages/conversations.rs:9-91` in a region with
  `JigRegion::Conversations`; its new chat's form a `Role::Form` with
  `JigRegion::NewChat`, `:92-131`). `sections/deployment.rs`:
  `pub(crate) fn section(..) -> bool`, a region with
  `JigRegion::Deployment`, its hosts' and accounts' lists each inside a
  region with `Hosts` and `Accounts`, its connectors' parts (`:79-81`)
  in one with `Connectors`, its capacity's region (`:65-78`) named
  `Capacity`; the owner path builds the same regions.
  `pages::main`: at the inbox, `heading`, `sections::inbox::header`,
  each inbox section, then the page's statuses (`:63-72`); at the
  conversations and the deployment, `heading` and the one section.
Test shape: `crates/jig-view/src/tests/pages.rs`:
  `the_inbox_header_shows_with_either_group`,
  `the_new_chat_form_is_a_form_inside_the_conversations` and
  `the_deployment_section_holds_its_four_named_regions`. Each under 20
  ms.
Proof: world, the command of 05.1's Proof, once: the inbox's,
  conversations' and deployment's stories pass (starting a chat; a
  proposal accepted in the inbox; `joint/deployment_story.rs`).
Don't: change what the inbox's filters or counts say.
Size: at most 150 lines of production code, moves not counted.
Removes: `pages::{inbox, conversations, deployment}` (moved).

### 05.3 A project's sections, header and tabs
Scope: view.md, 4.1; pages.md, 12.2 (a project's rows, the settings'
  five nested; `jig-new-goal`, `jig-notes-search`).
Needs: 05.1 (`work::section` with a region name, `sections.rs`).
Files: `git mv crates/jig-view/src/pages/project.rs crates/jig-view/src/sections/project.rs`
  and `pages/project/` to `sections/project/` (`goals.rs`, `notes.rs`,
  `owner.rs`, `owner_goals.rs`, `recurring.rs`, `tasks.rs`);
  `crates/jig-view/src/lib.rs:104` (the `owner_project` alias);
  `crates/jig-view/src/sections.rs`; `crates/jig-view/src/pages.rs`
  (`main`'s project arm); every user of jig-view's project page by
  path, `git grep -n -E 'crate::pages::project|crate::owner_project|jig_view::owner_project'
  -- '*.rs'` (at the base: `crates/jig-view/src/{binding.rs:55,75,254,258,sizes/pages.rs:186}`,
  `crates/jig-view/src/tests/owner_project.rs:546,721,911-916,1141`).
Shape: `sections/project.rs`: `pub fn header(..) -> bool` (the
  project's name, `pages/project.rs:33-43`, in a region with
  `JigRegion::ProjectHeader`), `pub fn tabs(..) -> bool` (`:44-74`, its
  tabs built by `work::tab` as in 05.1, `JigRegion::ProjectTabs`),
  `pub(crate) fn section(..,
  project: u32, section: ProjectSection) -> bool`: the overview
  (`:77-84`), goals (`goals::goals`; its new goal's form a `Role::Form`
  with `JigRegion::NewGoal`), tasks, notes (`notes::notes`; its search a
  `Role::Search` with `JigRegion::NotesSearch`), settings (`settings`,
  `:140`, with its five nested regions `:89-99` named
  `ProjectResources` … `ProjectRecurring`), or one of the five alone;
  each region named `JigRegion::Project<Section>`. The owner path
  (`genuine`, `:259-303`; `owner.rs`'s `section()`, `:486-557`, and
  `render()`, `:321-334`) builds the same regions with the same names.
Test shape: `crates/jig-view/src/tests/work_pages.rs` (`project_sections`,
  `:417`, and its helpers): `settings_hold_their_five_named_regions`,
  `the_goal_form_is_a_form_and_the_notes_search_a_search`, and
  `each_project_section_carries_its_region_name` (a table over the ten,
  names as literals). Each under 20 ms.
Proof: world, the command of 05.1's Proof, once: the stories that set a
  goal, search notes and change settings pass.
Don't: split `owner.rs`; change the settings' review (findings/view.md,
  8).
Size: at most 200 lines of production code, moves not counted.
Removes: `pages::project` (moved).

## Interface sketches

```rust
// crates/jig-view/src/sections.rs
/// One section of a page, whole, or nothing and false when the domain
/// does not show it (view.md, 4.1). Its region carries its region name.
pub fn section(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings, section: Section) -> bool;
pub mod inbox {
    /// The inbox's filters and counts, while one of its groups shows.
    pub fn header(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) -> bool;
}
pub mod task {
    /// The task's card, project, goal, priority and lineage, while one of its sections shows.
    pub fn header(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) -> bool;
    /// Every section the task's document offers, each a tab to its address.
    pub fn tabs(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) -> bool;
}
pub mod project {
    pub fn header(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) -> bool;
    pub fn tabs(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) -> bool;
}
```

Who owns what: this session owns `jig_view::sections` and its
signatures; jig 06 changes `pages::main` and the tabs' omission, jig 07
and 08 call `sections::section` from the roots' slots.

## Questions to raise when reached

- **The composer's place** (decisions.md, 13; a design gap reported to
  the user): pages.md 12.2 lists `jig-composer` inside an agent's
  activity, where the code builds no composer; this plan names the
  region around the composer's field in the confirmation's dialog. If
  the designer rules otherwise before 05.1 starts, 05.1 follows the
  ruling; otherwise no question.

## Done when

- `git grep -n -E 'fn (section|header|tabs)\(' origin/main -- crates/jig-view/src/sections.rs crates/jig-view/src/sections`
  prints `section`, and `header` and `tabs` for the task and the
  project, `header` for the inbox.
- `heavy cargo nextest run -p jig-view -E 'test(/region_name/)'` passes on
  `main`, once.
