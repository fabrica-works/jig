# client-composition: the domain's code against client.md 5.1

The planning's code audit of `jig-domain-client` at jig `2ae75cac2`,
against `client/domain/client.md`, 5.1 and 14. Paths are under
`crates/jig-domain-client/src/` unless they say otherwise. Nothing of
sections exists in the code: no `Section`, `Arrangement`, `ShowSections`
or `configure_arrangement`, and no `jig_view::sections`.

## 1. Decided from the address, not from a section's subject

- **Opening:** `routing::navigate` sets `domain.address` for a jig place
  only (`pages/routing.rs:105-106`; an application's page,
  `:117-120`, keeps the last jig address, so every read below is stale
  there). `enter` opens the address's own section only (`:149-187`).
- **Task:** `task::document` (`pages/task.rs:464-467`) chooses its parts
  (`:494`), the tree or the ended task (`:506-514`) and the activity
  (`:515-538`) from the address's section; so do `fold` (`:572-579`),
  `history` (`:619`), `open_ended` (`:631-638`) and `more` (`:676`).
  `conversation/routing.rs:30-37` (`document`, the activity) and
  `drafts.rs:215-221` (`sync`) read it too.
- **Project:** `project::number` (`pages/project.rs:135-140`) feeds
  `document` (`:194`), `tasks` (`:259`), `more` (`:298`), `board`
  (`:370`), `task_rows` (`:434`), `owner_document` (`:556`),
  `pages/project/notes.rs:11,24`, `pages/project/goals.rs:69,90` and
  `pages/routing.rs:992`; `enter` (`:143`), `document` (`:201-204`) and
  `show`'s part scope (`:246-251`) read the address.
- **Inbox and conversations:** `open_updates`
  (`pages/conversations.rs:219`), `filter_inbox` (`:274-283`) and `more`
  (`:394-399`).
- **Deployment:** `pages/deployment.rs:23,51`.
- **Parts' scope:** `application::current_project` (`application.rs:514-521`).
- **Domain:** `frame_summary` (`domain.rs:190-216`; `frame.rs:109-113`),
  the new chat's submit (`domain.rs:697-701`), `terminal_watch`
  (`:1088-1106`), `frame_answer` (`:783-788`).
- **The view:** `retained_outside_plan` (`crates/jig-view/src/pages.rs:140-153`).
- About the address itself, unchanged: `store.rs:108`, `signin.rs:38`,
  `pages/routing.rs:114`.

## 2. Read and window numbers

A read's number is its `Source::Window` number; a window's token is the
`Act::More` token.

| Page | Reads | Tokens |
|---|---|---|
| inbox | 10, 12 (`pages/conversations.rs:309-314`) | 10, 12 (`:342-344`) |
| conversations | 11, 14, 17 (`pages/routing.rs:171`; `pages/conversations.rs:348-351,404`) | 11 |
| task | 1 activity (`pages/task.rs:522`), 2 history (`pages/routing.rs:182`; `pages/task.rs:626,685`), 15 person task (`:528-535`), 16 ended (`:511,659,698`; `pages/routing.rs:959`), 23 final task (`domain.rs:1115`) | `HISTORY_WINDOW` 12, `ENDED` 13 (`pages/task.rs:662-664`); `ACTIVITY` 1 (`conversation/routing.rs:14`) |
| project | 0 project (`pages/project.rs:168`), 20 goals (`:155,307`), 3 notes (`pages/project/notes.rs:20`; `pages/project.rs:319`), 23 spend (`:161,330`; `pages/routing.rs:988`) | 20, 21, 22 (`pages/project.rs:292-296`) |
| an application's objects | `u16::MAX` (`application.rs:11`) | |

**Collisions:** token 12 is both `HISTORY_WINDOW` (`pages/task.rs:662`)
and `INBOX_UPDATES_WINDOW` (`pages/conversations.rs:344`); `Act::More`
hands every token to the four `more` functions (`domain.rs:683-688`).
Read 23 is both the spend's and the final task's.

## 3. Watch owners

- `watches::Owner` is `Frame`, `Page { generation }`, `Application {
  generation }` (`watches.rs:19-26`).
- `application::close` (`application.rs:72-80`) closes every
  `Application` watch, of any generation, whenever `show` sees another
  set of objects (`:33`) and on `left` (`:82`).
- `close_page` (`domain.rs:494-502`) closes `Page` and `Application`
  watches.
- A new owner needs an arm in: `close_page` (`domain.rs:497-500`),
  `application::close` (`application.rs:75-78`), `watch_event`
  (`domain.rs:848-884`), `page_behind` (`:513-519`), `retry`
  (`:962-965`), the silence timer in `fire` (`:1179-1182`) and
  `terminal_watch` (`:1046-1061,1070-1075,1083-1109`).
- The open-once checks compare with `Owner::Page` exactly
  (`pages/task.rs:417,548`; `pages/project.rs:267`).

## 4. The page's statuses

- `Page::Work { behind, excess, refused }` (`pages.rs:19`), set at
  `pages/routing.rs:160-161`.
- `refuse` (`pages/routing.rs:31-39`) shows a notice on a page that is
  not `Work`, an application's included; `excess` (`:40-46`) is dropped
  there (callers `:210`, `pages/task.rs:386,429`).
- `behind` comes from `Page` watches of the generation only
  (`domain.rs:509-536`).
- Missing: `pages/routing.rs:121-124,149-157,474-478` (a read answered
  none blanks the whole page) and `domain.rs:1089-1092`. On an
  application's page a read answered none is `refuse(Reads)`
  (`application.rs:148-155`).

## 5. Folds, the ended task, the plan's filter

- Folds are one `Map<u64, bool>` by task (`pages/task.rs:48`), pruned at
  `:194-208,217-229`; `open_ended` keeps one `ended_task` (`:655`),
  checked at `pages/routing.rs:424-428,958-963`; `filter`
  (`pages/task.rs:613-616`) has no address check.
- `retained_outside_plan` checks `Page::Work { Task { Plan } }` and the
  one `plan.owner` (`crates/jig-view/src/pages.rs:140-153,163`); false on
  an application's page.

## 6. Limits

- `limits::worst_case` (`limits.rs:62-124`) counts each page's state once
  (`:90-94`) and one of each window (`windows.rs:449-459`;
  `pages/inbox.rs:370-376`). Watches (`:85`), reads (`:105-107`) and
  objects (`:104`) are configured totals.
- `Domain::new` asserts `worst_case` (`domain.rs:97`) before
  `configure_arrangement` can run: the count is the most an application's
  page may show.

## 7. The objects an application's page names

- `Event::ShowObjects` / `ObjectsLeft` (`boundary.rs:49-52`), ignored
  before the start (`domain.rs:583-584`), dispatched at `:628-629`.
  `application::show` (`application.rs:15-66`) needs a confirmed party
  and a live link (`:16-18`: a `ShowObjects` before the link is live is
  dropped), compares sets, closes, names the objects under `u16::MAX`,
  reads them and opens the chips' and the inbox's watches (`:59-64`).
- Test root: `testing/jig-test-domain-client/src/first.rs:500` (from
  `family.rs:99-114`), `ObjectsLeft` at `first.rs:307`, variants at
  `:71-72`, forwarded at `route.rs:191-194`.
- `ops`: `examples/ops/domain-client/src/infrastructure.rs:105-106,311,509`,
  `observability.rs:85-86,273,398`, forwarded at `route.rs:246-249,285-288`.
- jig hears "went elsewhere" before the family hears its own `Went`
  (`examples/ops/domain-client/src/route.rs:133-142`;
  `testing/jig-test-domain-client/src/route.rs:94-97`).
- `configure_navigation` (`domain.rs:147-156`): refuses after the start
  or past its capacity, then replaces the field. `ops` passes it through
  (`examples/ops/domain-client/src/domain.rs:190-196`); the test root has
  no pass-through.

## 8. Other

- Every source naming objects shares `object_rows` (`pages/routing.rs:211-215`).
- `reads.pending` dedupes on what is read, not on the window
  (`reads.rs:61-69`): a second window asking the same read gets no
  terminal.
- `project::open` never dedupes (`pages/project.rs:170-174`): an
  overview beside the goals would open the goals' watch twice.
- The project's overview sets the inbox's filter but asks no inbox read
  (`pages/project.rs:148-152`).
- `Address::Inbox` has no group (`address.rs:49`); `Filter.kind` is the
  finer `InboxGroup` (`pages/inbox.rs:17-21`).
- `Conversations { project }` is ignored when the page opens
  (`pages/routing.rs:169-172`).
- Parts are retracted per generation only (`boundary.rs:88-90`;
  `pages/routing.rs:81`): a section dropped within a generation keeps its
  parts.
- `pages::task::Sections` and `pages::task::sections()` exist
  (`pages/task.rs:34-37,119`), beside the planned `Domain::sections()`.
