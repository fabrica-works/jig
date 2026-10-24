# Agent guidance for jig

- **main moves only through agents' `land`,** which runs the gate,
  `scripts/check.sh`, and pushes (`docs/development/workflow.md`, section
  1). Do not run the gate yourself before `land`. A change to Markdown
  files only lands with `land --docs`. Run what compiles and the tests of
  the crates you touch through `heavy`.
- The default test suite (unit tests and the worlds' focused tests under
  `tests/`) takes at most 30 seconds (15 seconds before the user's doubling
  of 10-22, provisional); the fuzzy suite (the worlds' `tests/fuzzy_*.rs`,
  randomized tests) at most 2 minutes (1 minute before the user's doubling
  of 10-22, provisional). Keep new tests within these budgets: see the same
  document.
- **jig is a kit for agentic applications;** temper is its first
  application. Read [README.md](README.md), then
  [docs/design/README.md](docs/design/README.md).
- **jig follows skein's foundation documents:** `programming-model.md`,
  `testing-strategy.md` and `notes.md`. Read them before writing code:
  the first two are rules, and a departure needs its reason, stated in
  the design or the commit. Citations use those file names and their sections.
- **jig's design is `docs/design/`.** Inside jig, a bare file name names
  one of its documents (`README.md`, 6.2). skein's design documents are
  named with skein's name (skein's `lib.md`). Code cites a section at
  module or type level, never stamped on every field.
- **Every crate of jig is step code under the strict subset** of
  programming-model.md, section 10, except jig's test kits, which are
  ordinary Rust like every world, and the client's shells, which are
  ordinary Rust like skein's shell. jig defines no traits and no generic
  items in step code. A mechanism that must be generic, such as the
  journal (`README.md`, 6.2) or the client's request table
  (`client/domain/root.md`, section 4), belongs in skein-lib, and goes
  there before its jig consumer.
- **jig owns every invariant it promises.** An application's root only
  routes and translates (`README.md`, section 6). A change that would
  make an application's root decide or order something is a change to
  jig's design first.
- **The core never depends on smith.** Only the charter's translation
  into smith's terms, the client's decoding of turns, below its domain,
  and an application's roots that compose smith's agent capability, with
  their protocol layers, which decode and encode smith's payloads, do.
  The hub, `jig-domain-host`, carries smith's payloads as bytes it never reads;
  it, the core and its children, and the client's domain and view link
  no smith crate. An application without agents never links smith
  (`README.md`, section 9; `domain/hosts.md`, 6.8).
- **No system is named in jig's code:** no forge, repository, branch, pull
  request, issue or git. Systems are an application's connectors.
- **Work on a branch in a worktree of your own,** never in the primary
  checkout. `land` is the only push; do not push or publish otherwise.