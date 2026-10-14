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

105. **jig 09.1: the shell's name in the renderer's bound, interim** (design gap, jig 09's block of 2026-10-14T13:55:46Z).
     Interim, until the designer's answer (development.md, 10.4): what bounds the shell's name in the renderer's print is a design gap (view.md 15.1: "Its output is bounded: the most lines and bytes the view's largest tree prints, counted from the view's limits"; 15.3: TITLE is "the shell's name when there is none"), and it is the designer's. jig 09.1 goes on with the work that does not rest on the answer: the role renderer (`render`, the role table of view.md 15.3, the numbers of 15.4), `src/tests/fixtures.rs`, the two goldens and the four tests, the boundary test. `render` takes the name as `&[u8]`, as the plan's Shape gives it. What rests on the answer, and waits for it: how `worst_case` and `Print::new` count the name's bytes, and any check of the name in `Settings` or `Terminal::new` (jig 10). 09.1 does not land before the answer; with its other work done, it waits for the final unblock, which says how the name is counted.

106. **jig 09.1: the shell's name in the renderer's bound** (design gap, jig 09's block of 2026-10-14T13:55:46Z).
     The designer's answer, landed in jig 346aaa4c7 (view.md 15.1) and 6e3fa0440 (jig/09-text-shell.md, jig/10-terminal.md); jig-09.1 may land once its work follows it:

     # Answer

     Answer to question.md's "jig 09.1: the shell's name in the renderer's
     bound, 2026-10-14T13:55:46Z", from the designer.


     Reading 1, the implementer's proposal, holds. The shell's name is at most
     a node's text, `limits.text_per_node` bytes; `Terminal::new` refuses a
     longer one; the renderer's bound stays counted from the view's limits
     alone. No signature of jig 09 changes: `render(tree, name, print)`,
     `worst_case(limits)`, `Print::new(limits)`.

     Why: on the first line the name takes the place of a node's text, the
     main's level-1 heading (view.md 15.3: "TITLE is the text of the main's
     first level-1 heading, or the shell's name when there is none"), so it is
     bounded as that text is. Reading 2 would make the bound the view's limits
     and a second number, against 15.1's "counted from the view's limits", and
     thread that number through `worst_case` and `Print::new` for a name of a
     few bytes (`ops`, `jig`). Checked on jig `main` (6e3fa044):

     - `crates/jig-view/src/limits.rs:16-17`: `text_per_node` is "Name, runs,
       field value and external target together on a node."
     - `crates/jig-view/src/limits.rs:112-115`: `text_per_node` is at least
       twice a phrase's bytes (`phrase.checked_mul(2)?`), so jig's limits
       leave a name far more than it needs.
     - `crates/jig-view/src/tree.rs:328-330`: `Tree` keeps only `nodes`, no
       `Limits`, so
       `render` knows the name's bound only if its `Print` keeps it.
     - `clippy.toml:87`: `core::debug_assert` is disallowed, "assertions stay
       on in production: assert!".

     The design now says so, `client/domain/view.md`, 15.1. The renderer:
     "Its output is bounded: the most lines and bytes the view's largest tree
     prints, counted from the view's limits. The shell's name, which the first
     line prints when the main has no level-1 heading (15.3), is at most a
     node's text, so the same limits count it." Its settings: "the name its
     own lines start with (its own, and at most the bytes the view's limits
     give a node's text: the terminal refuses a longer one)".

107. **jig 10's Shapes against main** (ready check of jig-10.1).
     jig/10-terminal.md amended against origin/main, for 10.1 and 10.2 (the ready check of jig-10.1):
     - 10.1's tests drive a raw `skein_sim::Sim`, as skein-sim's own tests do (`processes.rs:1237-1262` at jig's skein pin): `skein_world::World` keeps its `sim` private (`world.rs:65-67`) and opens no inherited pipe; the dev-dependencies are `skein-sim` alone.
     - 10.1's Files add `mod terminal;` in `crates/jig-shell-text/src/tests.rs`; `Settings` and `Terminal` derive `Debug`, and `Settings.interval` is a `skein_lib::Duration` (`clippy.toml:74` disallows `core::time::Duration`).
     - 10.2's Files add `"jig-shell-text"` to the allowlist of `client_world_joins_only_the_client_and_test_roles` (`tests/boundary/src/tests/graph.rs:296-337`), and `fn view_limits(view: &Self::View) -> jig_view::Limits` in `App` (`tests/client/src/harness/mod.rs`, beside `patch_limit`), implemented at `tests/client/src/test.rs:138`, `tests/client/tests/referee.rs:34`, `tests/ops/src/client/app.rs:143` and `tests/ops/src/joint/fixture.rs:185`, so that the world can make a `Print`.
     - 10.2's read-back check goes in `walk_people`, at `world.rs:692`, where the person's act is chosen and `person.at()` names the step, not in `act` (`:327`), which the faults at `:353-362` also call.
     Checked: on origin/main, `world.rs:692` is `Next::Do(Doing::Act { node, act }) => self.act(id, node, act),` inside `fn walk_people` (`:648`); `:358` is `Fault::StalePress { client, node } => self.act(client, node, Act::Press)`; `harness/mod.rs:13` is `pub trait App: Sized {` and `:37` `fn patch_limit(view: &Self::View) -> u32;`; the four `impl … App for` lines are at the lines above; `graph.rs:296` is `fn client_world_joins_only_the_client_and_test_roles`; `crates/jig-shell-text/src/tests.rs` holds `mod command;` and `mod fixtures;` only; `clippy.toml:74` is `{ path = "core::time::Duration", reason = "skein_lib::Duration" }`. The ready check compiled 10.1's sketch with `skein-io` under clippy, and 10.2's `render` and `command` calls with `view_limits` in a stub.

108. **jig 06's Shapes against main** (ready check of jig-06.1).
     jig/06-arranged.md amended against origin/main, for 06.1 and 06.3 (the ready check of jig-06.1):
     - 06.1: a project's page shows every section its subject offers when `Overview`, `Goals`, `Tasks` and `Notes` are each in `client.sections()`, and `Settings` or each of its five (client.md 5.1: "a page that would show `Settings` and one of the five shows `Settings` alone"; `arrange` keeps `Settings` whole, `pages/shown.rs:114-143`): its tabs' ten are never all listed.
     - 06.1: the address's own section's region is marked `State::Current` after `pages::work::section` returns, at `sections/task.rs:138`, `sections/project.rs:88` and the Settings loop at `sections/project.rs:119`; `sections::section` only dispatches. Files add `section` to the two files, and a start given an `Arrangement` beside `tests/cards.rs:165-183`'s `start`.
     - 06.3: the Files' lines at main (`pages.rs:164-244`, the card sites in `sections/`), the conversation's card sites `turns/call.rs:58-59` and `turns/activity.rs:60-61,76-77`, and `Limits::of(&CLIENT)` in its test (06.2's callers are `cards_sizes.rs:481,489,492`). Whether a card the shown activity's chat names is owed went to the designer (question.md of this check); 06.3 starts only after the answer's ruling. 06.1 and 06.2 do not rest on it.
     Checked: on origin/main, `sections/task.rs:138`, `sections/project.rs:88` and `:119` each call `crate::pages::work::section(b, words, …, region(…))`; `tests/cards.rs:165-166` are `pub(super) fn start() -> (Domain, Token) {` and `let mut d = Domain::new(&CLIENT, 1);`; `limits.rs:67` is `pub fn of(client: &jig_domain_client::Limits) -> Option<Self> {`; `pages.rs:199` is `pub fn after_page(…)`; `turns/objects.rs:9` is `pub(crate) fn shows_object(state: &Conversation, key: ObjectKey) -> bool {`. The ready check compiled `Domain::owes` and an `after_page` without the scan under clippy; jig-view's 332 tests and the two worlds' 384 passed, none of their stories showing a plan and its chat together.

109. **jig 06.3: a card a shown chat holds is not owed** (design gap, the ready check of jig-06.1, question of 2026-10-14T14:31:25Z).
     The designer's answer, landed in jig 38c0a1c7 (view.md 4.1) and f90bf452 (jig/06-arranged.md); 06.3 may start once 06.2 has landed, and follows it:

     # Answer

     Answer to question.md's "jig 06.3: an owed card that the shown
     activity's chat also shows, 2026-10-14T14:31:25Z", from the designer.


     Reading 1, the ready check's proposal, holds: the card is not owed.
     The shown conversation is a section that shows it, so it stays in the
     chat, once. An owed card is one that has nowhere else to show (view.md
     4.1: "no section of the page and no card slot of the application's page
     shows it"). The colon's second case was written for jig's own
     arrangement, one section per address, where a page showing the plan
     shows no conversation. `Domain::owes` is false while a conversation the
     page shows names the object. `turns::shows_object` moves into
     `jig-domain-client` as `Conversation::shows(key)`, which `owes` and the
     view both call. The conversation's card sites keep building their cards
     as they do today, with no `owes` check.

     Why not reading 2: moving the card from the chat to `jig-owed` would
     take it away from the call that made it, against view.md 9 ("Calls to
     the engine show as the cards of what they made, live"). It would also
     leave the chat with no card at that call.

     Checked on jig `main` (f90bf452):

     - `crates/jig-domain-client/src/conversation/routing.rs:94-96`: the
       conversation's window is already a source of what it names:
       `let keys = domain.conversation.names(env.limits.objects);` and
       `let source = Source::Window { generation: domain.generation, number:
       ACTIVITY_READ };`. `Conversation::names` (`conversation.rs:630`, "Names
       cards produced by calls and heard results, deduplicated by object
       key") reads the same lines as `turns::shows_object`
       (`crates/jig-view/src/turns/objects.rs:9-48`). That function uses
       only `jig_domain_client` types, so it moves whole.
     - `crates/jig-view/src/turns/call.rs:57-59` and
       `turns/activity.rs:59-61,73-77`: the chat builds a card for each object
       a call, result or decision names that the working set holds, with no
       check of where else it shows.
     - `crates/jig-view/src/pages.rs:164`: "A proposal's own watch can still
       name it after the plan stops listing it". That is why the second case
       exists: the proposal keeps a source after it leaves the plan's rows.
     - `crates/jig-view/src/sections/task.rs:310-321`: the activity's
       proposals beside the log already skip one the conversation shows
       (`!crate::turns::shows_object(client.conversation(), key)`).
     - `tests/client/src/stories.rs:480-484`: `proposal_fixture`'s committed
       turn has a call naming the proposal (`list([ObjectKey::Proposal(p.key)])`),
       so 06.4's story is this case.

     The design now says so, in `client/domain/view.md`, 4.1, `after_page`:
     "A card is owed while it is deciding, leaving, or unknown whether made
     (client.md, sections 5 and 6) and no section of the page and no card
     slot of the application's page shows it: its object named by none of
     the page's sources, or a proposal that left the plan's rows while the
     page shows that plan (client.md, section 5) and no conversation the page
     shows names it. A card a shown conversation holds, at the call that made
     its object or a line deciding it (section 9), stays there, on a page
     that shows the plan beside it too, and is never owed."

110. **jig-06.1 landed at cdfce095a without parsed Step and Plan trailers** (process, jig 06's block of 2026-10-14T14:42:10Z).
     jig-06.1 is landed at cdfce095a, and nobody rewrites it (implementer's card 8.4: a landed commit missing its body is never rewritten). Its last paragraph's lines were joined by a literal `\n`, so git parses one trailer, `Proof: unit, test(arranged), three arranged page tests passed\nPlan: client-composition\nStep: jig-06.1`, and no `Plan` or `Step` key; its `Gate-Tree` and `Gate-Result` trailers parse. For this commit only, the coordinator's landed check (development.md, 7.3, step 2) takes `git log -1 --format=%B cdfce095a | grep -c 'Step: jig-06.1'` printing 1 in place of the `%(trailers:key=Step,valueonly)` check; the other two checks stand. This ruling records cdfce095a as the commit of the plan client-composition's step jig-06.1, for the metrics (development.md, 15.2) that count steps by their `Plan` and `Step` trailers. From now on, a commit's paragraphs and trailers are written with real newlines: `git commit -F <file>`, or one `-m` per paragraph; a `\n` inside one `-m` stays two characters.

111. **jig 10.3: the terminal's pipes through World::run_with_inputs** (plan defect, jig 10's block of 2026-10-14T14:48:28Z).
     Neither a skein change nor a hosted child: skein-world already gives a test the simulator. `World::run_with_inputs` (skein `testing/skein-world/src/world.rs:369` at jig's pin 74b7fdaf3: `pub fn run_with_inputs<F>(self, inputs: F) -> Outcome<P, M>` where `F: FnMut(&mut Sim, &[Pid], &[P])`) calls its closure on each iteration, before the processes' turns. 10.3's `run` (`tests/client-protocol/src/native/world.rs:199-228`) calls it in place of `world.run()`; in the closure it opens the client process's two pipes once (`Sim::open_inherited_read`, `open_inherited_write`, skein-sim `processes.rs:268,273`), hands them to the client through a `Cell<Option<(Fd, Fd)>>` field that the client takes on its next iterate and gives to `Terminal::adopt` (that field carries `#[expect(clippy::disallowed_types, reason = "…")]`, since `clippy.toml:35` refuses `Cell`), and writes the person's lines and reads the pages with `Sim::peer_feed`, `peer_drain` and `peer_close` (`processes.rs:296,306,316`). The client's `next_deadline` includes `Terminal::next_deadline` (an idle world with nothing due fails), and the client closes both descriptors before it settles (`assert_no_open_fds`). skein stays read only and its pin unchanged; the session file's "10.3 stops and asks" is answered by this; 10.3's Test shape, Proof and Size are unchanged.

112. **jig 07's Shapes against main** (ready check of jig-07.1).
     jig/07-section-slots.md amended against origin/main, for 07.1, with 07.2's sketch (the ready check of jig-07.1):
     - 07.1's line numbers in `domain.rs` and `pages/project.rs` move to origin/main's (after jig 06); `pages/shown.rs` is extended, not new.
     - The opens and sources with `Owner::Page` written in open or source under the section's owner (`Sections { generation }` on an application's page): `pages/task.rs:562-568` (through `routing::open_once`, `routing.rs:47-71`), `pages/project.rs:138-140`, `conversation/chips.rs:71`, `pages/replies.rs:18`; else a plan's Tree opened on an application's page outlives `an_empty_list_closes_every_sections_watch`.
     - `ShowSections` keeps the first `MOST_SECTIONS` (19) and refuses a longer list with `refuse(domain, Limit::Objects)`, as `chips.rs:28` does.
     - 07.2: `SectionStatus` derives `Clone, Copy, Debug, PartialEq, Eq` (`missing_debug_implementations`), and `pages.rs` re-exports it, since `shown` is `pub(crate)`.
     Checked: `git grep -n 'MOST_SECTIONS' origin/main -- crates/jig-domain-client/src` prints `domain.rs:123` (`sections: List::with_capacity(crate::pages::MOST_SECTIONS)`) and `pages.rs:34` (`pub const MOST_SECTIONS: u32 = 19;`); `git grep -n -E 'refuse\(domain, Limit::' origin/main` prints `conversation/chips.rs:28` with `Limit::Objects`. The ready check compiled the sketch's event and owner under clippy; its only errors were the non-exhaustive matches 07.1's Files list.

113. **jig 07.2 lands at up to 215 production lines** (plan defect, jig 07's block of 2026-10-14T15:30:33Z).
     jig 07.2 lands at the reported size: at most 215 lines of production code, as code-organization.md 6 counts them (`git diff -M --numstat origin/main...HEAD -- <the increment's Files>`, added less removed, once), against its Size of 120 and the ask line of 180. The reason the report gives holds, and the cheaper shape it names breaks client.md 5.1 ("What a page says of itself, a section says on an application's page"): one section's refusal would show in every section, and a plan's Tree failure would mark an unrelated deployment section. The per-section status, kept for each accepted section, and the 19-bit mask over `MOST_SECTIONS` (19) stay as the report describes. Unchanged: code-organization.md 6's file limits (`domain.rs` and `pages/routing.rs` grow by at most 40 lines in session 07), the other increments' Sizes and their ask lines, and `SectionStatus`'s fields. The commit body gives the count and names this ruling.

114. **jig 08 starts at 08.2** (steer of the check of 2026-10-14T15:33Z).
     Plan order, from the check of 2026-10-14T15:33Z: jig 08 starts at 08.2 once its ready check has set it ready, in the lane's free slot, while jig 07 runs; the same session takes 08.1 when 08.2 and jig 07.3 have both landed (its own Needs, "jig 07.1 and 07.3"), after its own ready check, then 08.3. jig/08-ops.md gives 08.2 "Needs: jig 06.1 and 06.2", both landed (cdfce095a, c6339177b); the README's "jig 08 once 07 has" holds for 08.1 only. Unchanged: each increment's Needs, Shape and Proof; jig 07's order; 09.3 waits for 07.3, 10.4 for 08.3.
     Ruling 113's "`domain.rs` and `pages/routing.rs` grow by at most 40 lines in session 07" reads, for `domain.rs`, as jig/07-section-slots.md:77 gives it: at most 80 lines.

115. **jig 09.3 fixes for_world and lands at up to 200 lines of test kit code** (plan defect, jig 09's block of 2026-10-14T15:59:40Z).
     jig 09.3 fixes `jig_test_view::Limits::for_world` in the test kit, keeps the complete six-kind inbox fixture and the five named tests, and lands at up to 200 lines of test kit code. `for_world` states its own contract: "Storage declared by a world, rather than every combination admitted by the domain" and "Only the world's declared counts need text and patch capacities in u32" (`testing/jig-test-view/src/limits.rs:9-11,37`); its call of `jig_view::Limits::of_words` at `:28` breaks it, since `of_words` multiplies the complete envelope's nodes by `text_per_node` in u32 (`crates/jig-view/src/limits.rs:124`), and the inbox's limits (rows 4, objects 8) overflow that product. So:
     - `for_world` takes the envelope's node count and per-node fields (`depth`, `text_per_node`, `styles_per_node`, `markdown_depth`) without its `text_bytes` product; it keeps its u64 ceiling (`full_nodes`) and sets `nodes`, `bindings`, `slots`, `patches` and `text_bytes` from the world's declared `nodes`, as now. Default: the change is in `testing/jig-test-view/src/limits.rs` alone. If `for_world` cannot reach those fields without a second copy of `with_phrase_bytes`'s formula, add one function to `crates/jig-view/src/limits.rs` that `with_phrase_bytes` itself uses, at most 25 production lines, so the formula keeps one copy and `of_words`, `of` and `compose` return what they return now. Never copy the formula into the kit.
     - `testing/jig-test-view/src/tests/world_limits.rs`'s tests keep their names and assertions and pass unchanged; add one test there, one behaviour: `for_world` returns a profile when the complete envelope's text product does not fit u32 (the inbox's limits, or smaller ones that overflow the same product).
     - The fixture lives in the new `tests/client/src/stories/text_pages.rs`, a `pub mod` of `tests/client/src/stories.rs` (942 lines on main; 09.3 adds at most 60 there, code-organization.md 6). Remove the temporary diagnostic in `tests/client/src/test.rs` before the gate.
     - Size: at most 200 lines of test kit code, the fixture and `for_world`'s fix together, counted as code-organization.md 6 counts them ("production code, and test kit code where the Size says 'test kit code' or 'test code'; not tests or golden files"): `tests/client/tests/text_pages.rs` does not count. Each test function stays at most 40 lines, a table's rows not counted. Past 200, it asks. The report gives the count; the commit body gives it and names ruling 115.
     - Unchanged: `jig_test_view::Limits::of`; `examples/ops/view/src/limits.rs`'s envelope; the eight other golden pages; Don't (golden files by hand); 09.3's Proof. A further caller of `for_world` that the fix changes is listed in the commit body.

116. **jig 08's Shapes against main** (ready check of jig-08.2).
     jig/08-ops.md amended against origin/main 52af9057a, for 08.2 and 08.1 (the ready check of jig-08.2; 08.1's Needs, jig 07.1 and 07.3, have landed):
     - 08.2: `configure_arrangement` is `examples/ops/domain-client/src/domain.rs:198-204`; `compose` is `examples/ops/view/src/limits.rs:46-53`, its `Arrangement::none()` at `:49`; `arrangement` is re-exported at `examples/ops/view/src/lib.rs:16`. The joint world's second ops client, `tests/ops/src/joint/fixture.rs:87-100`, configures the arrangement after `:95`, its bounds at `:65-78` as `app.rs`'s `view_bounds`; without it the joint world shows the arranged page with only the address's own section, and `tests/ops/tests/fuzzy_joint.rs` still passes. `frame::navigation` takes `observability::NAV`, as `root.rs:91` passes it to `jig_view::jig`. ops's main calls `jig_view::pages::excess` after its sections, as jig's task page does (`crates/jig-view/src/pages.rs:121`): `excess` (`:56`) becomes `pub`, so the arranged page keeps the "more rows" and refusal status jig's page shows.
     - 08.1: `examples/ops/domain-client/src/limits.rs` joins its Files, as at jig 07.3 (2954c546c): `:18-19` `checked_mul(4)` becomes `checked_mul(5)`, and `:59-61` `observability_payload` takes `List::<Section>::worst_case(1)`. `route.rs` changes at `:285-288` only. The lead's plan shows on both incident pages, as the lead's chip does (examples.md 8: "its chip, its report and its proposals as jig's cards, and its plan as jig's plan section, in a section slot"): after `show_incident`'s chip (`observability.rs:87`) and `native_incident`'s (`:430`); the domain's `sections` names a native `Value::Incident`'s lead as `objects` does (`observability/documents.rs:186-190`), its match naming each `Value` variant. The section slot counts at `examples/ops/view/src/limits.rs:40`.
     - Both sketches, so corrected, pass `heavy cargo clippy -p jig-ops-domain-client -p jig-ops-view -p jig-ops-world --all-targets -- -D warnings`, and the three packages' 94 tests pass at the 1,024-node world bound. temper uses none of ops's crates. A further user found of a changed name is listed in the commit body.

117. **jig 08.2 finds and fixes the joint world's missing acknowledgement** (cross-session scope, jig 08's block of 2026-10-14T16:17:55Z).
     jig 08 finds and fixes the joint world's missing acknowledgement within 08.2, and keeps ruling 116's arrangement in the joint client (`tests/ops/src/joint/fixture.rs`). On main, the fuzzy suite passed in each gate before 08.2 (the last, 09.3's: "109 tests / 15.527 s"); with 08.2's arrangement, `p_acceptance_survives_its_durable_decision_commit` fails at `tests/ops/src/joint/front.rs:855`, where `protocol.check()` runs after `engine.drain()` and before the crash count is read (`:856-861`). So the failure is 08.2's to explain. In order, each once, logged under `logs/jig-08/`:
     1. Run the failing test alone on the branch, then with the joint fixture's `configure_arrangement` line removed (uncommitted, restored after): the report says whether the arrangement alone triggers it.
     2. Name the delivery behind `Token(9223372036854775817)`: which watch the shell consumed (`front.rs:933` or `:984`), which root input would have acknowledged it, and whether the engine's crash count changed in the same drain.
     3. Then, by what step 2 shows:
        - the acknowledgement is lost in a defect of the front, the session or the ops client: fix it at its cause, with one test at the cause's layer that fails without the fix;
        - the acknowledgement is lost only because the engine crashed in that drain: a change to what the referee checks across a crash (the check's order, an exemption, a reset at the disconnect) is made only if design text says that the obligation does not hold across a crash, quoted in the report and the commit body; without such text, report it as a question (it goes to the designer), and change nothing;
        - the cause is in a jig crate under `crates/`: report it before changing it.
     - The fix counts apart from 08.2's Size: at most 40 lines; past 40, it asks. The commit body names the cause and this ruling, ruling 117.
     - Never: remove or weaken `protocol.check()`; drop the arrangement from the joint client; change the test's seed, script or name; move it, or raise a ceiling.
     - Time: about 40 minutes from this answer to the fix or a report; past that, report what steps 1 and 2 found.
     - Unchanged: 08.2's Shape and Proof; 08.1 waits for 08.2, then its ready check (ruling 116); 08.3 and 10.4 as planned.

118. **jig 08.2: only the engine's restart excuses a taken delivery** (design gap, jig 08's block of 2026-10-14T16:23:54Z).
     The designer's answer, landed in jig 74bce1ac2 (protocol/client.md 7, domain/engine.md 11, client/domain/testing.md 6) and 6dd741350 (jig/08-ops.md, 08.2); jig 08 applies its Instructions, then gates and lands 08.2:

     # Answer

     Answer to question.md's "jig 08.2: acknowledgement obligation across a
     crashed stream, 2026-10-14T16:23:54Z", from the designer.


     The restart ends the obligation; the world never keeps or replays an
     acknowledgement across it. A stream's end, or any other end of a watch,
     does not end it. So the referee forgets its consumptions when the engine
     restarts, not when a stream disconnects, and `protocol.check()` stays
     where it is (`tests/ops/src/joint/front.rs:855`).

     Why: telling the core a delivery was taken is the core's pacing. The
     core is told so that it can make the watch's next delivery, or end the
     watch. While the engine runs, a watch ended with its delivery in flight
     waits for that delivery to be settled. Checked on jig `main` (dcbfa3a88):

     - `crates/jig-domain-core/src/routing/views.rs:10`: "| Unwatch | child
       closes after outstanding terminals | held watch ending |".
     - `crates/jig-domain-views/src/watch.rs:160`: closing a watch while it
       is sending gives `State::ClosingDelivery { end: why }`, and `:148`
       finishes it only at the delivery's `Delivered`.
     - `crates/jig-protocol-engine-clients/src/watches.rs:456-460`: a closing
       stream sends `Unwatch`, then `Delivered` for a delivery still pending.
       The engine's own side settles every delivery when a stream ends.

     So the supervisor's reading, "a watch ended by its stream's end owes
     nothing more", would excuse what the real engine still does. The
     restart is what ends the obligation: the core restarted cold holds no
     watch. engine.md 5.4 lists what the store keeps of the core, and names
     no watch. `watch.rs:135` ignores a `Delivered` that names a watch the
     views child does not hold. Replaying one would tell the new core
     nothing.

     The design now says so:

     - `protocol/client.md`, section 7, a new bullet: "**Each delivery is
       settled once, while the engine runs.** The core makes a watch's next
       delivery, or ends the watch, only once the engine's side has told it
       of the last one: taken, once its event has gone to the connection; or
       dropped, when its stream ended first. So a watch's end, by the client,
       by the engine or with its stream, never excuses telling the core of a
       delivery the client took. A restart does: the core restarted cold
       holds no watch (`../domain/engine.md`, section 11), and is told
       nothing of deliveries made before it; the client watches again
       (section 4), and its first delivery is the document whole."
     - `domain/engine.md`, section 11: "A watch ended with a delivery in
       flight ends once that delivery is taken or dropped", and a new bullet,
       "**Watches are never stored.** A restart (section 6) starts with none,
       and is told nothing of the deliveries made before it; each client
       opens its watches again".
     - `client/domain/testing.md`, section 6, a referee rule: "**A delivery
       taken is told:** whenever the engine has no input left to take, each
       delivery a client took has reached the engine's core as taken
       (`../../protocol/client.md`, section 7), whatever ended the delivery's
       watch meanwhile. Only the engine's restart in between excuses it: the
       restarted core holds no watch to tell."

     Both of the ops joint world's crash paths reach one place, the `cold`
     hook of its peers (`tests/ops/src/joint/engine.rs:202-206`, which
     already clears `peers.packets`):

     - the crash drawn inside `drain`: `testing/jig-conformance/src/harness.rs:388`
       calls `self.crash()`, which calls `A::cold` at `:303`;
     - `FrontFault::Restart`: `front.rs:899` calls `engine.crash()`, which is
       the same function.

     The referee forgets there.

     Checked once, on a scratch copy of be4741f95 (removed since), with
     `Referee::restarted` clearing `consumed` and `cold` calling it (6 lines):

     - `heavy cargo nextest run -p jig-ops-world --profile fuzzy
       p_acceptance_survives_its_durable_decision_commit`: "PASS [ 0.711s]"
       (log `logs/20261014T162902-86.log`);
     - `heavy cargo nextest run -p jig-ops-world --profile fuzzy`: "22 tests
       run: 22 passed" (log `logs/20261014T162912-86.log`).

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

