# client-composition: decisions

The decisions this plan rests on, numbered; the supervisor's rulings
during the run follow them, from 101; the choices left to the user close
the file. The user delegated every decision of the planning to the
planner (2026-10-14: "The user is away and has delegated decisions to
your best judgement").

## The plan's own

1. **Scope: jig only, the rows that name client-composition.** The rows
   of jig's `docs/design/STATUS.md` that name client-composition:
   `client/domain/README.md` 2 and 5, `client.md` 3, 4.1, 5.1, 9, 14 and
   17, `contract.md` 1, 3.3, 5.3, 6, 7.1 and 7.2, `pages.md` 2, 9 and 12,
   `root.md` 2, 6, 7, 9 and 15, `testing.md` 2, 4, 6, 8 and 9,
   `view.md` 3, 3.1, 4, 4.1, 5, 10 to 13 and 15; and, since they name the
   same items from the application's side and the same plan,
   `README.md` 10 and 15 and `examples.md` 8 (`ops`'s owners,
   arrangement, section slot and terminal client in its worlds). The
   designer found nothing needed upstream, and the planning found
   nothing either: skein at jig's pin (`74b7fdaf3`) has
   `Io::adopt_read_pipe` and `Io::adopt_write_pipe`
   (`crates/skein-io/src/layer.rs:158,167`), `skein_shell::drive`
   (`crates/skein-shell/src/drive.rs:53`), `open_termination_signals`
   (`crates/skein-shell/src/ring.rs:97`, exported by `skein_shell`) and
   skein-sim's inherited pipes (`Sim::open_inherited_read`,
   `Sim::open_inherited_write`, `crates/skein-sim/src/sim/processes.rs:268,273`).
   smith is not touched.
2. **No repin step.** jig's only downstream repository holds its pins for
   a run active on another machine, whose "Done when" keeps them
   unchanged until it ends; and its repin onto this plan's jig would
   need more than a repin may change (development.md, 5.7: at most about
   200 lines, no change of design): the owners' new names, the decoders'
   new arms, its own pages inside a main landmark, its stories scoped by
   landmarks. The user takes care of the downstream side (2026-10-14).
   The README says it as a rule for the coordinator: "This plan runs no
   repin step; its run ends when the last jig session lands."
3. **Owners first, whole, in one increment** (jig 01.1). `Owner::Side`
   and `Owner::Other` become `Owner::Application { number }` with every
   user in the same increment: no alias, no transitional constructor,
   since every user is in jig.
4. **The domain's sections before the view's, at jig's addresses first.**
   jig 03 makes a page its sections with one section per address, which
   changes no tree: the client's world (the real domain and view, the
   scripted engine) and the joint world (the real engine's domain) run
   it against both sides at every landing, as the meeting-point story
   (development.md, 5.3). jig 04 then adds the arrangement and an
   application's sections, which jig 05 and 06 show; jig 06 lands the
   arranged story, the first that runs them together.
5. **Names kept** (the designer's narrowing, extended): `jig-domain-client`'s
   `Part` keeps its field `connector`, `ops` keeps
   `jig-ops-domain-connectors`, and `jig-protocol-client-link` keeps
   `family: u16` in its exchanges. Each carries an owner number: for
   every application today the owner number is the connector's
   (`root.md`, 15: "A connector's owner number is the number the
   engine's root gives that connector"), and the test application's
   family `1` is its connector `FIRST`
   (`testing/jig-test-domain-client/src/owner.rs:3`). Renaming them
   would touch about 190 references and the wire for no change of
   behaviour. later.md keeps the item.
6. **The text shell's pages wait for the sections in the view.** Its
   renderer and command reader are tested on hand-built trees, so jig 09.1
   and 09.2 start once jig 02 has landed the outline. Its golden pages
   are built by jig's test view root from fixture states (`view.md`,
   15.6), so jig 09.3 waits for jig 06 and 07.3, which rebuild the pages
   from sections and region names; written earlier, every golden page
   would be edited by hand again.
7. **The outline is checked on change.** The referee runs
   `jig_view::outline::check` on each tree a person sees when it differs
   from the last one judged for that client (the view emitted patches),
   not at every callback (workflow.md, 2: "Checks on change").
8. **Test time** (findings/measurements.md): the focused suite's median
   at the base is 8.49 s and the fuzzy suite's 13.96 s on carbon-forge.
   The plan's new tests may add at most about 3.5 s to the focused
   suite's median (to about 12 s, the user's standing aim) and 6 s to
   the fuzzy suite's; the README gives each session its share, as a
   guide. A suite's budget, slow period or a test's ceiling is never
   raised; a share missed is made up by cheaper fixtures, built once.
9. **Golden text in two places,** each changed only by hand (`view.md`,
   15.6): the hand-built trees' (every role and state; unsafe bytes) in
   `crates/jig-shell-text/tests/golden/`, read by the crate's own tests;
   jig's pages' in `tests/client/tests/golden/`, read by a test of the
   client's world, which builds the fixture states with jig's test view
   root, as 15.6 asks, without the text shell's crate depending on the
   worlds. Each file is read with `include_bytes!`; no test writes one,
   and no environment variable regenerates one.
10. **One fixture module per crate,** as development.md, 5.2 says
    (code-organization.md, 4); the text shell's,
    `crates/jig-shell-text/src/tests/fixtures.rs`, made by jig 09.1.
11. **The limits of sections shown together** (`client.md`, 14). The
    domain keeps one state per subject (one task's, one project's, the
    inbox's, the deployment's), and a page names at most one of each
    (`client.md`, 5.1), so its state is counted once already. What a
    page with more sections adds is counted: the list of sections
    (`MOST_SECTIONS`, jig 03.1) and each section's statuses (jig 07.2)
    in the domain's worst case, and the sections' nodes summed in the
    view's limits (jig 06.2). A page whose sections need more watches or
    reads than the configured counts is refused what it asked, on the
    page or in the section, as a page is today; `Domain::new` does not
    refuse such limits, since it checks them before any arrangement is
    given (findings/domain.md, 6).
12. **`Limits::of` and `Limits::compose` take the arrangement;**
    `Limits::of_words` keeps jig's (none beside the address's own), as
    `view.md`, 10 names only the two (jig 06.2).
13. **Region names on a log, a dialog and tabs, and around the
    composer** (`pages.md`, 12.1 and 12.2). The conversation's log
    carries `jig-conversation` itself, the confirmation's dialog
    `jig-confirmation`, and the tabs `jig-task-tabs` and
    `jig-project-tabs`, so the builder accepts a region name on a log, a
    dialog and tabs beside landmarks and regions (jig 01.3), where
    `view.md`, 3 says "on a landmark or a region". `jig-composer` is the region around the
    composer's field, where the code builds it today: in the
    confirmation's dialog that "Write" opens
    (`crates/jig-view/src/confirmation.rs:226-260`), not inside the
    activity's section, where pages.md 12.2 lists it. This plan does not
    move the composer: no section of the design this plan builds asks
    it (`pages.md`, 4 and 5 are not among its rows).
14. **The text shell is one crate under the step code's lints,**
    terminal included; the terminal owns its own `skein_io::Io` and
    tags its operations' tokens with bit 62, as the native shell's files
    tag theirs with bit 63 (`crates/jig-shell-native/src/files.rs:9`),
    so a host routes its completions by `Terminal::owns`. Nothing
    changes in `jig-shell-native` or in skein.
15. **Tabs are tabs:** a task's and a project's tabs become `Role::Tab`
    (in the `Role::Tabs` they sit in), built by a new `work::tab` while
    `work::control` keeps building the filters' buttons, the address's
    own marked `State::Current`, as `pages.md`, 12.3 has them lead to each
    section's address with the address's own marked current. The stories
    that pressed them as buttons press them as tabs (jig 05.1).
16. **The test family's item names a second task** (`Item.second`,
    jig 07.3), so that client.md 17's story can show a second task's
    section not shown. It is the smallest change that lets the test
    family name two tasks; every literal of `Item` sets `None` but the
    story's.
17. **Sessions:** ten, each of three or four increments, cut so that an
    implementer stays under its 200,000-token handover; the domain's
    sections (jig 03, 04) and jig 01 start at once; the view's sections
    wait for both sides (jig 05 needs 02.3 and 03.2).

## Rulings

(The supervisor's, from 101, during the run.)

101. **jig 02's Shapes against main** (ready check of jig-02.1).
     jig/02-outline.md amended against origin/main fdfb3a0bd, for 02.1 and 02.2 (the ready check of jig-02.1):
     - Every `Role::Region` the session opens has a name, since `crates/jig-view/src/tests/cards_access.rs:21-23` asserts it: the frame's five parts take five new phrases, `LinkPart` ("Connection"), `NeedsPart` ("What needs you"), `PartyPart` ("Who is signed in"), `ProjectsPart` ("Projects"), `GoToTaskPart` ("Go to a task"); 02.3's notices' region takes `NoticesPart` ("Notices"), and a page not about the work names its region with its level-1 heading's phrase. A further unnamed region takes a new phrase at the next free id, listed in the commit body. This replaces "One phrase is added".
     - Phrase ids: `NothingKnown` and the five parts' names are 674 to 679 (621 is `ChipDependenciesCut`, `jig.rs:1113`); `NoticesPart` is 680; `ALL` goes from 620 entries to 626 in 02.2, 627 in 02.3.
     - 02.2 changes none of `page_capacity.rs:845-853`'s literals; at `:518`, `banner_nodes + navigation_nodes` is `frame_nodes`' value plus 5, and `Limits.nodes` grows by 5.
     - A part keeps view.md 4.1's signature, and names a parameter it does not read with a leading underscore (`_bindings` in `link`, `_client` in `go_to_task`).
     - 02.1's test helpers take no closure (`clippy.toml:63-65`): `fn page() -> Builder` and `fn finish(b: Builder) -> Tree`; `Fault` and `Rule` derive `Clone, Copy, PartialEq, Eq, Debug`; the sketch's doc comments quote `after_page` in backticks.
     - Ranges: `pages.rs:123-132`, `lib.rs:52-56`; the grep for "Work" and "Navigation" also prints `english.rs:765-766`.
     Checked: `git show origin/main:crates/jig-view/src/tests/cards_access.rs | grep -n 'Role::Region | Role::Dialog'` prints `21: Role::Region | Role::Dialog | Role::Status | Role::ChoiceGroup => {`; `grep -n -E 'Self::\w+ => (67[4-9]|68[0-9]),' jig.rs` prints nothing, and `:1093` is `Self::OwnerHoldRelease => 673`; `grep -c` of each new phrase's text and variant name in `english.rs` and `jig.rs` prints 0; `clippy.toml:63-65` disallow `core::ops::Fn`, `FnMut`, `FnOnce`. The ready check compiled 02.1's reference test in this form under clippy, and it passed with node 6.

102. **jig 04's Shapes against main** (ready check of jig-04.1).
     jig/04-arrangement.md amended against origin/main 982110898, for 04.1 (the ready check of jig-04.1), with one name in 04.2:
     - `domain.rs` ranges after jig 03.1's `sections()`: the struct `:48-93`, `new` `:95-147`, `configure_navigation` `:161-171` (also in findings/domain.md:124).
     - `pages.rs` imports `TaskSection` and `ProjectSection`, and `Section`'s two qualified paths at `:23` and `:25` lose theirs (`unused_qualifications`).
     - `configure_arrangement` keeps client.md 5.1's refusals ("refuses a list longer than its kind's sections, naming one twice, or naming `Settings` with one of its five"), checking the length first; the lints refuse `matches!`, `Iterator::any` and `filter`, `Option::is_some_and`, slicing that may panic and `as` casts, and `list.get(..i)` with `contains` and an exhaustive `match` passed.
     - `Arrangement::none()` and `Domain::arrangement()` carry `#[must_use]`.
     - Each new test of 04.1 and 04.2 is its own `#[test]`, outside the group at `tests/pages.rs:1262`, so that the Done-when filter `test(/arrange/)` names it; 04.2's third test is `an_arranged_task_section_its_document_does_not_offer_is_not_shown`.
     Checked: `git show origin/main:crates/jig-domain-client/src/domain.rs | grep -n -E '^pub struct Domain|^    pub fn new|^    pub fn configure_navigation'` prints 48, 97, 162; `pages.rs` lines 23 and 25 read `section: crate::address::TaskSection` and `section: crate::address::ProjectSection`; `tests/pages.rs:1262` is `#[test] fn frame_navigation_and_non_work_pages_follow_owner_facts`, which calls the plain fn at `:76`. The ready check built 04.1's sketch with these fixes under `heavy cargo clippy ... -- -D warnings`, and its two tests passed at 2 ms each.

103. **jig 05's Shapes against main** (ready check of jig-05.1).
     jig/05-section-views.md amended against origin/main 98c6e80dc, for 05.1 (the ready check of jig-05.1), with two notes for 05.3:
     - 05.1's Files add what the move of `pages/task` forces: `pages.rs:16`'s alias to `crate::sections::task::scratch` (`task.rs:298` `pub(crate)`); `mod work` (`pages.rs:17`) and `work::{section, control, counts, cursors}` `pub(crate)`; `mod conversations` (`pages.rs:11`) and `conversations.rs:133`'s `runtime` `pub(crate)` until 05.2; the moved `super::` paths as `crate::pages::…`; and the users `cards/owner_escalation.rs:53-54` and `owner_historical.rs:134-135` of `crate::pages::task::{overview, budget}`.
     - The counts: `sizes/pages.rs:119` (`task_in`, 35 becomes 36) and `sizes.rs:247` (`confirmation_nodes`, 66 becomes 67); the page bounds are `page_capacity.rs:849-858`; a size literal or the depth bound (`limits.rs:120`) changes only by what an increment adds, named in the commit body.
     - Every `Role::Region` the session opens has a name (`cards_access.rs:21-23`): a header's region takes its page's level-1 heading's phrase and holes (`JigPhrase::TaskPage` with the number for a task's), the composer's region `JigPhrase::AnswerField`; still no new phrase.
     - A tab pressed by `Find::named` or `Find::exactly_named` with `Role::Button` becomes the same call with `Role::Tab`; the task's tabs are pressed at `engine_stories.rs:341` and `tests/stories.rs:64`; `joint/stories.rs:273-281` and `deployment_story.rs:86-89` press the project's, in 05.3's Files. A fault story whose value changes because `harness/world.rs:398,432` pick the first bound button is named in the commit body.
     - Ranges: `confirmation.rs:227-239` (the field at `:231`), `pages/task.rs:90-101`, `lib.rs:107`.
     Checked: at 98c6e80dc, `sizes/pages.rs:119` ends `.checked_add(35)?.checked_add(slot_nodes)`; `sizes.rs:247` is `.checked_add(66)` in `confirmation_nodes`; `page_capacity.rs:849` is `fn each_page_bound_keeps_its_value_at_the_client_limits`; `pages/task.rs:28` is `super::heading(b, words, JigPhrase::TaskPage, &[Hole::Number(number)])`; `confirmation.rs:231` builds `Purpose::Composer { task }` with `JigPhrase::AnswerField`; `pages/work.rs:11-19`'s `section` names its region with its title. The ready check compiled a trial move of `pages/task` under clippy, which showed the paths above.

104. **jig 09's Shapes against main** (ready check of jig-09.1).
     jig/09-text-shell.md amended against origin/main, for 09.1 (the ready check of jig-09.1):
     - `fixtures::page(main: fn(&mut Builder)) -> Tree`: a fn pointer, which the reference's closure coerces to, since `clippy.toml:63-65` disallows `Fn`, `FnMut` and `FnOnce`; it gives the nodes their ids with `jig_view::diff` from an empty tree, so that 09.2's commands find them by `Tree::bound`.
     - `render` judges a node bound by its `binding` or `link` field, not `Tree::bound`, which gives nothing for `NodeId(0)` (`tree.rs:362-368`).
     - `Print` derives `Debug` (`missing_debug_implementations`).
     - Each golden read by `include_bytes!` carries `#[expect(clippy::disallowed_macros, reason = "…")]` (`clippy.toml:93`), as `crates/jig-view/src/tests/markdown.rs:2` does.
     - 09.1's Files add `tests/boundary/src/tests/graph.rs:138-151` (`client_step` names `jig-shell-text`) and an `assert!(client_step("jig-shell-text"))` beside `:177`.
     Checked: `git show origin/main:clippy.toml | sed -n 93p` prints `{ path = "core::include_bytes", reason = "data comes in events and limits" },`; `tree.rs:362-368`'s `bound` returns `None` when `id == NodeId(0)`; `graph.rs:138-151`'s `client_step` lists ten crates, no `jig-shell-text`; `diff.rs:334` is `pub fn diff(old: &Tree, new: &mut Tree, limits: &Limits, next_id: &mut u32, out: &mut Queue<Patch>)`. The ready check built 09.1's skeleton, fixtures and reference test with the fn-pointer fixture, and `heavy cargo clippy -p jig-shell-text --all-targets` was clean.

## For the user

Long-term choices the user may veto. Each was decided by the planner
under the user's delegation.

- **The shortcut's modifier is `Modifier::{Plain, Primary, Alt}`** and
  its key a byte `a`–`z` or `0`–`9` (`Shortcut { key: u8, modifier }`),
  checked by the builder. It is part of the tree every application and
  shell writes against; renaming it later breaks each of them. The
  alternative was 36 named keys, an enum longer than what it carries.
- **Region names in the tree are an owner and a `u16`**
  (`Region { owner, number }`), jig's numbered by `JigRegion`'s
  declaration order, and a shell maps a number to its name through its
  owner (`JigRegion::of(number)`, `JigRegion::name`). A stylesheet and a
  browser's shell will depend on the names, never the numbers, which may
  change before a release (formats are pre-release, development.md,
  5.1).
- **Owner numbers keep two older field names:** `Part.connector` in
  `jig-domain-client` and `family` in `jig-protocol-client-link`'s
  exchanges carry an owner number (decisions.md, 5). Every application
  writes against them; renaming them later breaks each application's
  roots and its wire. The alternative was renaming them now (about 190
  references and the client link's wire).
- **The terminal's interface:** `Terminal::{new, adopt, owns,
  completions, submissions, say, iterate, commands, close, closed,
  work_pending, next_deadline}`,
  with its operations tagged by bit 62 (decisions.md, 14). An
  application's terminal client is written against it (`ops`
  standalone, later); changing it later changes each such host. The
  alternative was the terminal speaking io's requests through a host's
  shared io, which needs the native shell to expose its io.

