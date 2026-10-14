# Session 03: a page is its sections, at jig's addresses

`jig-domain-client` learns that a page is made of sections, each naming
its subject: `pages::Section`, and `Domain::sections()` listing the
page's. At jig's addresses a page is still one section (the inbox's
two), so no tree changes; but every place that decided what a page
opens, reads, folds or keeps from the address decides it from the
page's sections, every window gets a number of its own, and the page
opens what its sections need, once each. jig 04 then adds the sections
an application's arrangement puts beside the address's own, and jig 07
those an application's page names.

A path in this file that names no crate (`pages/task.rs:662`) is under
`crates/jig-domain-client/src/`.

## Read first

- `client/domain/client.md`, 5 and 5.1 (sections), 8.1, 9 (its last
  bullet, folds).
- `client/domain/pages.md`, 12.2 (the sections, in their order).
- findings/domain.md, whole: it lists every site this session changes.

## What the design asks

- `pages::Section`: `Inbox { group, project }`, `Conversations {
  project }`, `Task { task, section }`, `Project { project, section }`,
  `Deployment` (client.md, 5.1).
- `Domain::sections()` lists the page's sections, in pages.md 12.2's
  order. Each section reads its subject from itself, never from the
  address.
- What a page opens is what its sections need, each watch and read
  opened once for the page's generation however many sections need it;
  the working set is their union.
- Each section's windows are its own: every window a page holds has a
  number no other window of the page has; a read's terminal goes to the
  window that asked.
- At an address of jig's, the page shows the address's own sections:
  the inbox's two groups with the address's filter, a task's or a
  project's section at the address, the conversations or the deployment.

## Choices this plan makes

- `Section` derives `Ord`, with `InboxReadGroup`, `TaskSection` and
  `ProjectSection` deriving it too: their declaration orders are pages.md
  12.2's, so `Domain::sections()` is kept sorted.
- `pages::task::Sections` and `pages::task::sections()` (the sections a
  task's document offers) are renamed `pages::task::Offered` and
  `pages::task::offers()`, so that `Section` and `sections()` name one
  thing; `pages::task::offered` keeps its name (client.md, 5.1 names it).
- `pages::MOST_SECTIONS: u32 = 19`, the most sections one page may show:
  the inbox's 2, the conversations' 1, a task's 6, a project's 9 (its
  overview, goals, tasks and notes, and `Settings` or its five) and the
  deployment's 1. `Domain::new` sizes its list of sections by it.
- `Domain::shows(section) -> bool` is the one question the view and the
  domain's own sites ask; jig 04.2 adds the task's offered sections to
  it.
- The frame's own reads of the address stay: `frame_summary`
  (`domain.rs:190-216`, `frame.rs:109-113`) and `frame_answer`
  (`domain.rs:783-788`) say where the person is, as do `store.rs:108`,
  `signin.rs:38` and `pages/routing.rs:114`.

## Rules that bind it

- **Behaviour at jig's addresses is unchanged:** no test of
  `jig-domain-client`, `jig-view` or the worlds changes an expected
  value in this session, but the renames of 03.1 and the window numbers
  of 03.3, each named in its commit body.
- `crates/jig-domain-client/src/domain.rs` (1,241 lines) and
  `pages/routing.rs` (1,111 lines) grow by at most 60 lines each in this
  session (code-organization.md, 6).
- Aim for the session's tests to add at most 0.2 s to the focused
  suite's median: a guide (README, "Test time"), checked by each
  landing's `Gate-Result`.

## Increments

### 03.1 The page's sections, named
Scope: client.md, 5.1 (the first and fifth bullets: `Section`,
  `Domain::sections()`, an address of jig's).
Needs: nothing.
Files: `crates/jig-domain-client/src/pages.rs` (`Section`,
  `MOST_SECTIONS`, beside `Page`); `crates/jig-domain-client/src/address.rs:5,30`
  (`ProjectSection`, `TaskSection`: derive `PartialOrd, Ord`);
  `crates/jig-domain-client/src/contract/reads.rs:24` (`InboxReadGroup`:
  the same); `crates/jig-domain-client/src/domain.rs:48-91` (a field
  `sections: List<Section>`), `:96-145` (`new`), the accessor;
  `crates/jig-domain-client/src/pages/routing.rs:61-127` (`navigate`
  clears it), `:129-187` (`enter` sets it); the rename in
  `crates/jig-domain-client/src/pages/task.rs:34-37,119,176` and its
  users `crates/jig-domain-client/src/tests/pages.rs:490,2349,2436`,
  `crates/jig-domain-client/src/tests/task_context.rs:65`,
  `crates/jig-view/src/pages/task.rs:76`.
Shape: `pub enum Section` as in the sketch; `pub const MOST_SECTIONS:
  u32 = 19;` `pub fn sections(&self) -> &[Section]` on `Domain`. `enter`,
  once it has made the page `Page::Work`, sets the list from the
  address: `Inbox { project }` gives `Inbox { group: Needs, project }`
  and `Inbox { group: Updates, project }`; `Conversations { project }`
  gives `Conversations { project }`; `Task { task, section }` and
  `Project { project, section }` give theirs; `Deployment` gives
  `Deployment`. Every other page has none.
Test shape: `crates/jig-domain-client/src/tests/pages.rs`, with
  `tests/spine.rs`'s `start`, `delivered`, `frame` and `drive`:
  `the_inbox_shows_both_groups_of_its_filter`,
  `a_task_address_shows_its_own_section_alone`,
  `a_page_not_about_the_work_shows_no_section`. Reference:
  ```rust
  #[test]
  fn a_task_address_shows_its_own_section_alone() {
      let mut domain = Domain::new(&LIMITS, 2);
      let f = start(&mut domain, List::with_capacity(0));
      delivered(&mut domain, Time::ZERO, f, frame(7));
      drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Task { task: 9, section: TaskSection::Plan }) });
      assert_eq!(domain.sections(), &[Section::Task { task: 9, section: TaskSection::Plan }]);
  }
  ```
  Each under 20 ms.
Proof: unit; nothing reads the list until 03.2, which proves it in the
  worlds.
Don't: change what `enter` opens (03.4 does).
Size: at most 60 lines of production code.
Removes: the names `pages::task::Sections`, `pages::task::sections`.

### 03.2 Each section reads its subject from itself
Scope: client.md, 5.1 (the first bullet's last sentence; "One subject
  each": the sections share this domain's state for their subject).
Needs: 03.1.
Files: every site of findings/domain.md, 1, but the frame's and those
  about the address itself (Choices above):
  `crates/jig-domain-client/src/pages/task.rs:464-467,494,506-514,515-538,572-579,619,631-638,676`;
  `crates/jig-domain-client/src/conversation/routing.rs:30-37`;
  `crates/jig-domain-client/src/drafts.rs:215-221`;
  `crates/jig-domain-client/src/pages/project.rs:135-140` (`number`, and
  through it `:194,259,298,370,434,556`, `pages/project/notes.rs:11,24`,
  `pages/project/goals.rs:69,90`, `pages/routing.rs:992`),
  `:143,201-204,246-251`;
  `crates/jig-domain-client/src/pages/conversations.rs:219,274-283,394-399`;
  `crates/jig-domain-client/src/pages/deployment.rs:23,51`;
  `crates/jig-domain-client/src/application.rs:514-521`;
  `crates/jig-domain-client/src/domain.rs:697-701` (the new chat's
  project), `:1088-1106` (`terminal_watch`'s task);
  `crates/jig-view/src/pages.rs:136-172` (`retained_outside_plan`).
Shape: on `Domain`, `pub fn shows(&self, section: Section) -> bool`
  (the list holds it); `pub fn shown_task(&self) -> Option<u64>`, `pub fn
  shown_project(&self) -> Option<u32>` and `pub fn shown_inbox(&self) ->
  Option<Option<u32>>`, the subject the page's sections name, if any
  (a project's overview names its project's inbox filter,
  `Some(Some(project))`). Each site asks these instead of matching
  `domain.address`: a task's section choice becomes `shows(Section::Task
  { task, section: X })` for the section it decided on;
  `project::number` becomes `shown_project()`; the new chat's project is
  the conversations' filter, else `shown_inbox()`'s, else
  `shown_project()`, as the address's precedence is today.
  `retained_outside_plan` asks `client.shows(Section::Task { task, section:
  TaskSection::Plan })` with `client.shown_task()`.
Test shape: no new test: the domain's, the view's and the worlds' tests
  pass unchanged (the session's rule). One test,
  `crates/jig-domain-client/src/tests/pages.rs`:
  `an_application_page_shows_no_task_after_a_task_page` (go to a task,
  then `Place::Elsewhere`: `shown_task()` is `None`, where the address
  still names the task). Under 20 ms.
Proof: joint, `heavy cargo nextest run -p jig-client-world -p
  jig-ops-world` once: the client's world (the real domain and view, the
  scripted engine), the joint world (the engine's real domain) and
  `ops`'s client world pass unchanged, every page opening and showing
  what it did. This is the meeting point's story for jig 03 and 04
  (development.md, 5.3).
Don't: add a field holding a copy of the address; change what any page
  opens.
Size: at most 200 lines of production code.
Removes: the reads of `domain.address` at the sites listed.

### 03.3 Each window its own number
Scope: client.md, 5.1 ("Each section's windows are its own").
Needs: nothing (03.2 if it has landed: the `more` functions it touched).
Files: the window tokens: `crates/jig-domain-client/src/pages/task.rs:662-664`
  (`HISTORY_WINDOW` 12, `ENDED_WINDOW` 13), `pages/conversations.rs:342-346`,
  `pages/project.rs:292-296`, `conversation/routing.rs:14`. The read
  numbers passed to `read`, `ask` and `reads.ask` (findings/domain.md,
  2): `pages/routing.rs:171,182`, `pages/conversations.rs:309-314,348-351,404`,
  `pages/task.rs:511,522,528-535,626,659,685,698`, `domain.rs:1115`,
  `pages/project.rs:155,161,168,307,319,330`, `pages/project/notes.rs:20`;
  and the matches on a read's source number, `pages/routing.rs:959,988`
  (the ended task's 16, the spend's 23). `domain.rs:683-688` (`Act::More`'s
  dispatch) is unchanged. The view's users of the tokens, unchanged by
  name: `crates/jig-view/src/pages/{conversations.rs:74,inbox.rs:107-108,project.rs:193,322,task/history.rs:48,task/plan.rs:37}`,
  `pages/project/{goals.rs:54,notes.rs:74,owner.rs:537}`. Tests that name
  a window by its const follow its new value with no edit
  (`crates/jig-domain-client/src/tests/pages.rs:646,686`,
  `tests/history_facts.rs:115`, `crates/jig-view/src/tests/page_loss.rs:233`);
  no test names the final task's read by its number (`git grep -n
  'number: 23' -- crates/jig-domain-client/src/tests` prints nothing at
  the base).
Shape: `HISTORY_WINDOW` becomes 14 (no other window's token); the
  final task's read (`domain.rs:1115`) becomes 24 (no other read's
  number). Each read number becomes a `pub(crate) const <WHAT>_READ:
  u16` beside its window's token in the module that reads it (`ACTIVITY_READ
  = 1`, `HISTORY_READ = 2`, `NOTES_READ = 3`, `INBOX_NEEDS_READ = 10`,
  `CONVERSATIONS_READ = 11`, `INBOX_UPDATES_READ = 12`,
  `CONVERSATIONS_BEFORE_READ = 14`, `PERSON_TASK_READ = 15`,
  `ENDED_READ = 16`, `REPLAY_READ = 17`, `GOALS_READ = 20`,
  `SPEND_READ = 23`, `FINAL_TASK_READ = 24`, `PROJECT_READ = 0`), and
  each call names its const.
Test shape: `crates/jig-domain-client/src/tests/pages.rs`:
  `no_two_windows_of_a_page_share_a_token` (the nine `*_WINDOW` consts
  in one array, sorted, `windows(2)` all different) and
  `no_two_reads_of_a_page_share_a_number` (the same over the
  `*_READ` consts). Each under 20 ms.
Proof: joint, the command of 03.2's Proof, once: the stories that read
  more history, updates and conversations pass with the new numbers.
Don't: change a window's capacity or what `more` reads.
Size: at most 60 lines of production code.
Removes: the collisions of token 12 and read 23.

### 03.4 A page opens what its sections need
Scope: client.md, 5.1 ("What a page opens is what its sections need").
Needs: 03.2.
Files: `crates/jig-domain-client/src/pages/routing.rs:129-187`
  (`enter`); `crates/jig-domain-client/src/pages/project.rs:142-169`
  (`project::enter`) and `:170-174` (`open`, which never dedupes);
  `crates/jig-domain-client/src/pages/task.rs:540-557` (`open_once`, the
  model); `crates/jig-domain-client/src/pages/conversations.rs` (the
  inbox's start, `start_inbox`).
Shape: `enter` sets the page's sections (03.1), then for each section in
  `sections()` calls `open_section(domain, env, section, owner, out)`,
  a new `pub(crate) fn` in `pages/routing.rs` whose arms are what
  `enter` and `project::enter` open today for that section, `owner`
  being the watches' owner (`enter` passes `watches::Owner::Page {
  generation }`; jig 07.1 passes another). Every watch it opens goes
  through a check that no watch of that owner and generation has the
  same subject (as `open_once` does for the task's); every read through
  `reads.pending` (as today). `project::enter` is folded into
  `open_section`'s project arm and goes.
Test shape: `crates/jig-domain-client/src/tests/pages.rs`:
  `the_inbox_opens_its_watch_once_for_both_groups` (the requests
  `enter` emits at `Address::Inbox`: one `Subject::Inbox` watch).
  Under 20 ms. The rest is the session's rule: tests unchanged.
Proof: joint, the command of 03.2's Proof, once: every page the stories
  visit opens what it did.
Don't: open anything an address did not open at the base.
Size: at most 120 lines of production code.
Removes: `pages::project::enter` (folded into `open_section`).

## Interface sketches

```rust
// crates/jig-domain-client/src/pages.rs
/// One section of a page, naming its subject (client.md, 5.1; pages.md, 12.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Section {
    /// One group of the inbox, filtered by a project, or every project's.
    Inbox { group: crate::contract::InboxReadGroup, project: Option<u32> },
    /// The party's chats, and starting one, in a project or any.
    Conversations { project: Option<u32> },
    /// One section of one task.
    Task { task: u64, section: crate::address::TaskSection },
    /// One section of one project.
    Project { project: u32, section: crate::address::ProjectSection },
    /// The deployment's hosts, accounts, connectors and capacity.
    Deployment,
}
/// The most sections one page shows: 2 + 1 + 6 + 9 + 1.
pub const MOST_SECTIONS: u32 = 19;

// crates/jig-domain-client/src/domain.rs
impl Domain {
    pub fn sections(&self) -> &[Section];
    pub fn shows(&self, section: Section) -> bool;
    pub fn shown_task(&self) -> Option<u64>;
    pub fn shown_project(&self) -> Option<u32>;
    pub fn shown_inbox(&self) -> Option<Option<u32>>;
}
```

Who owns what: this session owns `Section`, `MOST_SECTIONS` and the
`Domain` methods above; jig 04 extends `shows` and `sections()`'s
content, jig 05 and 06 call them, jig 07 adds an application's page's
sections to the same list.

## Questions to raise when reached

- None expected. If a site of findings/domain.md, 1 decides something
  for which no section of the page names a subject (a task's watch
  ending at an application's page, for instance), it keeps today's
  behaviour, and the report names it.

## Done when

- `git grep -n 'domain.address' origin/main -- crates/jig-domain-client/src/pages/task.rs crates/jig-domain-client/src/pages/project.rs crates/jig-domain-client/src/pages/deployment.rs`
  prints nothing.
- `heavy cargo nextest run -p jig-domain-client -E 'test(/no_two_(windows|reads)_of_a_page_share/)'`
  passes on `main`, once.
