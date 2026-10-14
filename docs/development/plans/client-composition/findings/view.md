# client-composition: the view's code against the design

The planning's code audit of `jig-view` and its users at jig
`2ae75cac2`, against `client/domain/view.md`, 3, 3.1, 4, 4.1, 5 and 10,
and `pages.md`, 12. Paths are under `crates/jig-view/src/` unless they
say otherwise.

## 1. Owners

- `tree.rs:15-22`: `Jig`, `Side { connector }`, `Other { child }`.
  `git grep -c -E 'Owner::(Side|Other)\b'`: 66 lines in 37 files
  (jig-view 28, of which 26 in tests; jig-test-view 13; `ops`'s view 13;
  `tests/client` 8; `tests/ops` 3; jig-fake-person 1).
- Exhaustive matches on `Owner`: `testing/jig-test-view/src/root.rs:66,183`,
  `examples/ops/view/src/root.rs:109,205`, `tests/client/src/observe.rs:267`,
  `tests/client/src/harness/observation.rs:102`,
  `tests/client/src/joint/deployment_story.rs:105`.

## 2. Roles, attributes, acts

- `Role` (`tree.rs:26-97`) lacks the landmarks, menu, menu item,
  disclosure and image; `State` has all nine of view.md 3's states.
- `Node` (`tree.rs:190-206`) lacks a region name, a shortcut and a
  picture: `Node::vacant`/`new` (`:209-245`), the builder, and the diff's
  `Attribute` (`diff.rs:16-40`) and `attributes()` (`:245-259`) change
  with them. No `outline` or `regions` module (`lib.rs:12-43`).
- Exhaustive matches on `Role`: `testing/jig-fake-person/src/tree.rs:117`,
  `random.rs:80,118`. Role lists that a menu item joins:
  `tests/cards_access.rs:14`, `tests/binding.rs:10`.
- Exhaustive matches on `jig_view::Act` (17): `binding.rs:233,243,253,257,261,274,280,286,290,298`;
  `testing/jig-test-view/src/bindings.rs:55,59`;
  `examples/ops/view/src/bindings.rs:59,63,76`;
  `testing/jig-fake-person/src/tree.rs:108`, `src/tests.rs:331`.

## 3. The frame (`frame.rs:13-96`)

- One region named "Work" (`JigPhrase::Frame`) holding, in order: the
  link's status (`:17`) and behind (`:18-20`); who is signed in (`:23`);
  the needs' counts (`:24`); sign out (`:25-27`); the updates' and needs'
  links (`:28-46`); the projects (`:47-49`); "signed out" without a frame
  (`:51`); the task number's field (`:53-57`); and, nested in it, the
  "Navigation" region with the application's slot (`:58-94`).
- Split by pages.md 12.1: link `:17-20`; needs `:24,28-46` (and
  "nothing known yet" where `:51` is); party `:23,25-27`; projects
  `:47-49` with `membership` (`:119-143`); go to a task `:53-57`;
  navigation `:58-94`, a sibling of the banner. The party's and the
  needs' nodes are interleaved today, so their order changes.
- `frame::control` (`:98-117`) has 44 calls in 22 files: it stays.
- `frame_nodes` (`sizes.rs:191-200`) becomes one companion per part.
- What breaks: `tests/client/src/observe.rs:304-326` (`frame_link` finds
  the "Work" region and reads its statuses);
  `tests/client/src/observe/link_tests.rs:26-46` (builds "Work" with
  "Navigation" inside); direct calls of `frame::frame`: `pages.rs:126`,
  `tests/frame_facts.rs:9-27` (sized by `frame_nodes`), `tests/cards.rs:205`,
  `tests/cards_sizes.rs:379-388`; the phrase list at
  `tests/frame_facts.rs:186-199`. Stories that find "Live", "Offline",
  "Behind" or "Sign out" by role or text alone survive
  (`tests/client/src/joint/stories/fault_stories.rs:164`,
  `tests/client/tests/shell.rs:69`).

## 4. The outline

- No landmark anywhere. Under the page node today: the frame's region,
  each page's nodes loose, the owed cards as loose keyed items
  (`pages.rs:194`), the dialog (`confirmation.rs:75`), the notices'
  statuses and a loose dismiss button (`notices.rs:16-27,31`).
- The test application's page (`testing/jig-test-view/src/family.rs:29-53`):
  a level-1 heading "Family", then a list or a status, under the page
  node (`root.rs:57-61`). `ops`'s (`examples/ops/view/src/observability.rs:43-60`,
  `infrastructure.rs:56-71,81`): a level-1 heading and a keyed item,
  under the page node (`root.rs:90-104`). `tests/page_capacity.rs:550-552`
  builds a page the same way.

## 5. `after_page` (`pages.rs:174-210`)

- For each object `shown`, passing `retained_outside_plan` (`:136-172`),
  whose card is deciding, leaving or unknown, it scans `b.tree.nodes()`
  for the object's key (`:187-192`) and builds the card when none is
  there. view.md 4.1 wants it decided from the domain.
- Sections build cards by key without checking sources
  (`pages/task.rs:167,195,268`; `pages/project.rs:233,241`;
  `pages/inbox.rs:214,275`; `pages/conversations.rs:33-35`); only
  `pages/task.rs:257-261` leaves a proposal with no source to
  `after_page`. Slots are filled after `after_page`
  (`testing/jig-test-view/src/root.rs:60` before `:63`).

## 6. Limits

- Node capacity: the largest page, or the application's if larger,
  plus the frame, the confirmation, the notices, the slots, and 1
  (`limits.rs:102-108`). `of` (`:67-70`) and `compose` (`:85-94`) take no
  arrangement; nothing counts owed cards or section slots.
- `page_nodes` (`sizes/pages.rs:211-221`) is the maximum of six pages;
  the task's page takes its largest section (`:117-119`), the project's
  likewise (`:180-188`); only the project's overview (goals and inbox)
  and `settings_in` (`:152-173`) sum.
- What pins today's formula: `tests/page_capacity.rs:511-531`,
  `testing/jig-test-view/src/limits.rs:28-41,74-84`,
  `examples/ops/view/src/limits.rs:39-46`, `tests/ops/src/client/app.rs:120`.

## 7. The pages today

- Task (`pages/task.rs:17-103`): heading `:28`; header `:29-75`; tabs
  `:77-89`; the section's region `:91`; sections already functions:
  `overview` `:145`, `activity` `:224`, `plan::plan` (`pages/task/plan.rs:19`),
  `history::history` (`pages/task/history.rs:12`), `budget` `:276`,
  `slots` `:114` (panels).
- Project (`pages/project.rs:20-113`): heading `:33` (unnamed), the name
  as text `:43`; tabs `:44-74`; the overview inline `:77-84`;
  `goals::goals`, `tasks::tasks`, `notes::notes`; `settings()` `:140`,
  its five nested regions `:89-99`. The owner path: `genuine`
  (`:259-303`) calls `pages/project/owner.rs`'s `section()`
  (`:486-557`), one section at a time; its `render()` (`:321-334`)
  builds the settings whole.
- Inbox (`pages/inbox.rs:17-73`): heading `:18`; filters `:19`
  (`filters` `:135`) and counts `:34-39`, an unnamed header; the two
  groups are regions with level-2 headings (`:40-62`).
- Conversations (`pages/conversations.rs:9-91`): one function, no
  region; its forms `:92-131`.
- Deployment (`pages/deployment.rs:9-83`): hosts `:20-42`, accounts
  `:43-64`, the capacity region `:65-78`, parts `:79-81`; the owner path
  is `owner_deployment::render` (`:12`).
- `pages/not_the_work.rs:5` opens no main.
- `work::section` (`pages/work.rs:11-19`, called from `pages/task.rs:91`,
  `pages/project.rs:75,96,299`, `pages/deployment.rs:65`) opens a
  section's region and level-2 heading: where its region name goes.

## 8. Other

- Tabs are buttons marked selected (`pages/work.rs:20-37`), always
  shown; pages.md 12.3 has them lead to each section's address, the
  address's own marked current, and left out when every offered
  section shows.
- No section slots: both view roots fill `Owner::Jig` slots with cards
  only (`testing/jig-test-view/src/root.rs:66-80`,
  `examples/ops/view/src/root.rs:110-129`).
- The project settings' review is a region with confirm and dismiss
  inside its section (`pages/project/owner.rs:952-969`), not
  `after_page`'s dialog: unchanged by this plan.
