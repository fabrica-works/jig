# Session 03: the view's sizes and its phrasebook

`jig-view`, `jig-test-view` and the client's worlds are 22 s of the
gate's 119 s summed time (`findings/measurements.md`, 2), most of it
before a test's first assertion: the full English phrasebook built once
per process (3.8 ms, in 269 of `jig-view`'s 301 tests and about 130
others), and `page_nodes`, which computes a card's size about fifteen
times in one call (1.3 ms, about 440 calls a gate). Two increments, each
in `crates/jig-view` and needing nothing outside it; the audit measured
both together at 34.7 s to 31.0 s summed over the six client packages.

## Read first

- jig's `docs/design/client/domain/view.md`, section 10 (the page
  bounds) and the phrasebook's sections (`phrases.rs` cites them).
- jig's `docs/development/workflow.md`, section 2 ("Fixtures once").
- `findings/cost-jig-client.md`, 1 and 2, and `tried/jig-client.md`.

## What the design asks

- The page bounds are what `view.md`, 10 derives; a bound's value does
  not change, only how often its parts are computed.
- A phrasebook's load refuses as it does: `Limit`, `Duplicate`,
  `Missing`, `Unknown`, `Holes`, `Overflow`; and allocates what it
  allocates today: its sorts stay `sort_unstable`.

## Choices this plan makes

- **`page_nodes` computes a card's and a chip's size once** and passes
  them to private helpers; each public `*_nodes` function stays, a thin
  wrapper that computes them and calls its helper, so its callers and
  values are unchanged. `deployment_nodes` takes the chip only, so its
  `None` cases stay the same.
- **The load keeps `sort_unstable`** (a stable sort would allocate
  scratch the book's worst case does not count) and **finds a phrase's
  shape by position first:** both lists are sorted and of one length, so
  `ids[position] == words.id` holds for every phrase of a well-formed
  language; the binary search runs only when it does not, so a refusal
  names the same id it names today.

## Rules that bind it

- No increment "drops an assertion, or changes an expected value, except
  a size its session file names with the reason the assertion holds at
  it" (code-organization.md, section 4).
- "**A test is cheaper by its algorithm first**" (testing-strategy.md,
  8).

## Increments

### 03.1 A page's bound computes a card's size once
Scope: view.md, 10, unchanged in value.
Needs: nothing.
Files: `crates/jig-view/src/sizes/pages.rs` (`:1-210`: `runtime`,
  `inbox_nodes`, `conversations_nodes`, `call`, `turn`,
  `conversation_nodes`, `plan`, `overview`, `history`, `task_nodes`,
  `goals`, `tasks`, `notes`, `spend`, `settings`, `project_nodes`,
  `deployment_nodes`, `not_the_work_nodes`, `page_nodes`);
  `crates/jig-view/src/tests/page_capacity.rs`.
Shape: `page_nodes` computes `let card = card_nodes(l)?; let chip =
  chip_nodes(l)?;` once and calls `inbox_in(l, card, chip, slot_nodes)`
  and the others; each `pub fn x_nodes(l, slot_nodes)` becomes
  `x_in(l, card_nodes(l)?, chip_nodes(l)?, slot_nodes)` (or `chip` alone
  where the function used only it); the private helpers take `card` and
  `chip` as `u32` and never call `card_nodes` or `chip_nodes`.
Test shape: `page_capacity.rs:505-535` unchanged; one test,
  `each_page_bound_keeps_its_value_at_the_client_limits`, with each
  public `*_nodes` function's value at `CLIENT` (the tests' client
  limits) and `slot_nodes` 0, written as literals taken from a run of the
  old code on `main` before the change and named in the commit body.
Proof: unit, `heavy --gate cargo nextest run -p jig-view -p
  jig-test-view -p jig-client-world --profile measure -j 1` once before
  and once after: summed times in the report (the audit, with 03.2: about
  1.1 s of the gate's summed time from this increment).
Don't: change a bound's value, a `None` case, or a public signature.
Size: at most 60 lines of production code.
Removes: about 14 calls of `card_nodes` per `page_nodes`.

### 03.2 The phrasebook's load finds a shape by its position
Scope: the phrasebook's load, unchanged in what it takes and refuses.
Needs: nothing.
Files: `crates/jig-view/src/phrases.rs` (`load` `:221-300`: the search
  at `:288`); `crates/jig-view/src/tests/` (the phrasebook's tests).
Shape: in the loop over `language.phrases`, `position` counts the
  phrases; `let index = if ids.get(position) == Some(&words.id) {
  position } else { ids.binary_search(&words.id).map_err(|_|
  Invalid::Unknown { id: words.id })? };`.
Test shape: one test, `a_language_with_a_foreign_phrase_is_refused_naming_it`:
  a language with as many phrases as there are shapes, one of them with
  an id no shape has, refused as `Invalid::Unknown { id }` with that id.
Proof: unit, the command of 03.1's Proof, once after both: summed times
  in the report (the audit: about 0.6 ms less a book with this alone).
Don't: change the sorts, a refusal, or what the load allocates.
Size: at most 10 lines of production code.
Removes: the binary search for well-formed languages.

## Done when

- `heavy --gate cargo nextest run -p jig-view --profile measure -j 1` on
  jig's `main`: every test passes, summed at most 4.3 s (4.69 s at this
  plan's base, `findings/measurements.md`, 2).
