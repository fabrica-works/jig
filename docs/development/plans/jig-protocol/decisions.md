# jig-protocol: decisions

The decisions this plan rests on, numbered; the supervisor's rulings
during the run follow them, from 101; the choices left to the user close
the file.

## The plan's own

1. **Scope.** jig's protocol layer as `protocol/` designs it, for the
   one-process shape: the store, configuration, accounts and grants, host
   tools, the workers' channel, the client's protocol with the native
   shell, the reference service and binary, and the domain's owed items.
   Not in it: LLM calls in a host's own process (`llm.md`, 1 to 4),
   agents in the engine, the browser's shell and wasm32 builds, TLS
   between hosts, contained process trees, retention, `ops` beyond what
   keeps it building (`later.md`).
2. **No browser, no wasm32** (the user, 2026-10-12): the client's
   protocol is proven through `jig-shell-native` only, with one small
   process proof.
3. **skein first.** skein's designed pieces (addresses and joined
   sockets, startup reads and the seed, `Secret`, `base64`, `CryptoRng`,
   the accounts driver's drop, `OpenWrite` and `Cut`) are built by this
   plan's skein sessions, then taken by smith 01 and jig 02.
4. **LLM and OAuth endpoints stay socket addresses.** They never dial an
   in-process name, so skein's `llm-connection` and `oauth-accounts`
   keep `SocketAddr` endpoints, and turn them into `Addr::Ip` as they
   connect.
5. **The live layer borrows the user's Codex login** (the user,
   2026-10-12): read only, access token only, handed in to `jig-test`
   as an account (`llm.md`, 5.4); at most 40 completions in the run.
6. **One owner for the hub's vocabulary.** jig 05 changes the hub's and
   the fleet's boundaries in one session, so the sessions that use the
   new fields (jig 04, 06, 08) never edit them.
7. **Codecs follow records** (README, "Rules"): once a family's codec
   exists, a change to a durable record changes its codec in the same
   increment, so drift is a build error the day it happens.
8. **The worlds of jig's own application move to the real store**
   (`store.md`, 14). `ops`'s worlds stay on `jig-fake-store`, which stays
   for them alone until `ops` is developed again (`later.md`): `ops` is
   not developed by this plan.
9. **The store's design questions the audit raised,** answered by the
   designer (2026-10-12):
   - the test application ships real key encoders and codecs for its
     connector's records (`store.md`, 12);
   - an ended goal's index row keeps its subtree's phase counts, so a
     project's goals page needs no subtree join;
   - an escalation's read shows its latest kept decision;
   - the conformance world rebuilds an in-memory store by replaying its
     durable commits, for snapshots and crashes: skein-kv gains nothing;
   - the store's fill reaches the core as an event of its own with each
     commit's answer; skein-lib's journal takes the number alone;
   - delegates' historical inputs are not this plan's (`later.md`);
   - a story reads a historical escalation and a seen chat through the
     real store.
10. **The test application's system** stays ordinary Rust in the worlds
    (`testing/jig-test-system`). The reference service answers its test
    connector's system with a small step-code module of its own, in
    memory, which the service owns.
11. **The test worker's agents** are smith's real agent processes: over
    skein's fake LLM, hosted by skein's world, in the channel's and the
    one-process worlds; over a real LLM in the live proof
    (`channel.md`, section 12).
12. **The reference binary runs smith's agent shell** as its `agent`
    command, so one binary serves the engine, the worker and the agents
    (`service.md`, section 10).
13. **Retention is later.** The store's mechanisms that retention needs
    (dropping a transcript whole) are built; the domain's timers,
    summaries and horizons are not (`later.md`).
14. **smith's later asks are not in this plan** (`protocol/README.md`,
    section 11): a run's text and named calls on smith's channel, and
    the LLM component's abort.

15. **A delegate's budget may be left out:** the task then takes its
    charter's default budget (`protocol/configuration.md`, section 3).
16. **The host kind is chosen at the claim:** the core picks a free slot
    of a kind the charter allows and commits that kind with the claim
    (`domain/engine.md`, 7.1); placement goes to a host of that kind,
    and an attempt with none free waits, busy being no failure.
17. **An answer that arrives while the engine stops** is left
    unacknowledged, as `domain/engine.md`, 6.1 says; its host sends it
    again after the next hello.
18. **The chats' charter is the deployment's,** one, required
    (`Settings::chat`); a project without chats is not this plan's.
19. **`jig-protocol-charter` may depend on `jig-domain-tasks`,** the
    domain vocabulary it translates a result into (`protocol/README.md`,
    section 5), so the mapping is written once.

20. **The store's fill, above its mark,** refuses a party's new task and
    new chat as busy, and a sign-in as refused; a run's delegation goes
    on (`protocol/store.md`, 11.2).
21. **Positions for single reads** are kept within the task record's
    bound (`inbox_messages`), so a read of an older question or proposal
    answers gone (`protocol/store.md`, 15).
22. **Wall times in the store's codecs** are `u64` nanoseconds since the
    epoch; the generator has no wall type.
23. **Ended goals page newest first,** as `protocol/store.md`, 7.3 says.
24. **The store's boundary** gains its open (`Open`, `OpenRefused`) and a
    corrupt row's event (`Corrupt`), which sections 7.1 and 10 of
    `protocol/store.md` need.

25. **A handed-in account's file** is a strict JSON object:
    `access_token`, `expires_at` (Unix seconds) and, where its provider
    names one, `account_id` (`protocol/configuration.md`, section 10).
26. **A grant's value** is the access token and, where the account's
    provider names one, the account's id, from the account's
    configuration or its handed-in file (`protocol/llm.md`, section 6):
    smith's ChatGPT provider needs it.
27. **A resumed window crosses as one piece,** bounded by the resume
    limit, each turn counted with its length; no turn count is kept (the
    designer, 2026-10-12, jig `e7eb8486`; `protocol/channel.md`, 6.1;
    replacing ruling 172's list); the application's resource names by
    `Sizes::resource` (`protocol/channel.md`, section 11).
28. **The worker keeps credential values** in `jig-protocol-accounts`'
    `Table`, held by its protocol layer; `jig-protocol-worker-link` hands
    values out as it decodes them.
29. **skein's open details:** `Truncate` cannot be cancelled; its errors
    are read only, permission and no space; a missing file to `OpenWrite`
    answers `Failed { NotFound }`; `read_private_file(path, max, user) ->
    Result<Secret, Unread>` with `Unread::Unsafe`, and
    `skein_shell::crypto_seed() -> Result<Secret, i32>`; a private file's
    path crosses no symbolic link, a system directory's included.
30. **Cryptographic generators' stream numbers** are named by each
    service that holds several: the reference service lists its own
    (jig 25), one per component that draws secrets.

31. **Chats forgotten** (`domain/people.md`, section 6) are built for a
    chat that ended and was read (jig 03.4); one never read waits for
    retention (`later.md`).
32. **An answer whose fence passes its last turn's read** is admitted as
    failed, invalid.
33. **A waiting proposal or escalation past a full run inbox** waits, as
    any message does (`domain/tasks.md`, 8.2).
34. **The run's identity** is derived by `run_identity`, in
    `jig-protocol-charter` (jig 06.2); the reference service and `ops`'s
    root call it as they put an assignment together, and
    `jig-protocol-engine-workers` copies the 16 bytes the hub's assignment
    carries (it links no smith, code-organization.md, section 3; jig 23.3).
35. **A claim with no free slot** records `Worker` when the charter
    allows workers, `Engine` otherwise (jig 06.5, in place of jig 07.4's
    rule); decision 16 holds otherwise.

36. **A service's credential** reaches its owner once, in the answer
    that makes the service: on the wire beside the service's party, to
    the owner's shell, never the client's domain, so the contract's
    `Made::Service { party }` carries no secret; the service signs in with
    it as a native client does (`protocol/client.md`, section 8; the
    designer, 2026-10-13, jig `2dfb78a5`).
37. **An LLM grant's value, as smith's agent processes take it,** is
    smith's envelope of the account's id and its access token (smith's
    agent service). The engine's table and jig's channel keep the two
    apart, and the worker's layer builds the envelope as it hands the
    value to smith's host half (decision 26; the designer, 2026-10-12,
    jig `87195d80`, `protocol/llm.md`, 5.3).
38. **Sessions split for size:** jig 14 into 14 and 19; jig 25 into 25
    and 27; jig 31 into 31 and 34; jig 32 into 32 and 35. Each second
    half's file names the first's "Read first" and sketches.

39. **A crash in the domain worlds keeps the in-memory stores;** a
    snapshot's fork replays the durable commits into a fresh store
    (decision 9, made precise).
40. **The secrets' store** loads its digests at start (`Secrets::Load`,
    jig 17.1); a failed secrets' commit stops the engine with no answer;
    the sweep of orphan digests is the store's alone (jig 17), never the
    clients' component's.
41. **The store's `Open` runs in memory mode too,** so the format, header
    and cut checks run after a world's crash; roots cross as `Root {
    token, fd }` until skein's io names every root by token.
42. **A transcript read as unread** fails its run as transient and the
    next starts fresh (jig 06.4).
43. **The reference service's store** uses `jig-test-store`'s routing
    (jig 18), and routes `RestoreRefused`, `SeenChat` and
    `HistoricalEscalation`, which the test root sends to `unreachable!`
    today (jig 18.3, jig 27.3).

44. **The plan's commands were not all run before it landed**
    (development.md, 5.4): most name tests, test binaries and crates that
    the plan's own increments create (their names are the plan's), so
    they can run only once those land. A command whose name differs from
    what its increment made is a plan defect the supervisor fixes when
    met. The interface sketches of skein 01–03, smith 01, jig 01, 02 and
    05 were compiled under clippy; the others are guides, and landed code
    wins where they differ.
45. **Sessions that wait partway** (jig 31 for jig 34's pages, jig 32
    for jig 35's documents, jig 05.6 for jig 04 and 07) do the
    increments that need nothing outstanding first, then yield, as
    development.md, 8.7 says; the coordinator resumes them.

## The supervisor's rulings

101. **A primary checkout behind `origin/main` is no block** (2026-10-12,
     the coordinator's block "smith 01.1: primary checkout behind
     origin/main", 2026-10-12T15:00:00Z). Other machines land in skein and
     smith while this plan runs (smith's reliability plan), so a primary
     checkout's `main` falls behind `origin/main` between this run's own
     landings there; `land` fast-forwards it at the next one
     (development.md, 9.1, step 5). From now until the run's end, when
     `plan check` prints `FAIL <repository>: main is 0 ahead and <N>
     behind origin/main`, the coordinator runs `git -C
     /srv/data/git/runner/src/<repository> merge --ff-only origin/main`,
     then `plan check` once more, and goes on; it writes nothing to
     `blocked.md` for it. A `main` with commits `origin` lacks ("<N>
     ahead", N above 0), or a merge that fails, is still a block, as
     development.md, 9.6 says. Proposed for the retrospective: `plan check
     --fix` fast-forwards a primary checkout that is only behind.
102. **`Unsafe::Mode` carries the group and other bits alone** (2026-10-12,
     the coordinator's block "skein 02.3: unsafe mode bits disagree",
     2026-10-12T15:10:21Z). A plan defect, not a design gap: 02.3's Shape
     says "startup and io name one file alike", and its reference test
     expects `Unsafe::Mode(0o040)` for a `0o640` file, but io's
     `crates/skein-io/src/file_layer/private.rs:83,99` pass the whole
     mode. The design names the bits found unsafe: io.md, 5.3, "saying
     what it found unsafe (a link, a non-directory, group or other bits,
     another owner)"; shell.md, 6.1, "as io's `Unsafe` names it (a link,
     the kind, the bits, the link count, the owner)"; and `Unsafe::Mode`'s
     comment, "The opened node has these group or other permission bits."
     So io and startup both pass `stat.mode & 0o077`. skein 02.3 makes
     io's change in the same increment, beyond its Files: those two
     production lines, and the expected values at
     `crates/skein-io/src/tests/private.rs:87,104` and
     `tests/io/src/private.rs:136,142`. No other repository names
     `Unsafe::Mode` (`git grep` on smith's, jig's and temper's mains).
103. **An increment's Size is a guide, with a fifth over** (2026-10-12,
     the coordinator's block "skein 01.1: production size exceeds
     increment ceiling", 2026-10-12T15:10:55Z). skein 01.1 counts 323
     lines of production code against "Size: at most 300 lines of
     production code in skein-io", and asks whether to restructure. No
     check reads a session file's Size line, so it is a guide
     (development.md, 4.5: "An allocation meant as a guide and worded as
     a rule makes sessions stop over a millisecond, or bend their code to
     fit it"). From now to the run's end, for every session: an increment
     within N × 1.2 of its "Size: at most N lines", rounded down, lands
     and gives its count in its report; past that, it asks first.
     code-organization.md, section 7 says so. skein 01.1 lands at up to
     360 lines. The 800-line file limit and the 40-line test function
     stay rules, which `plan pulse` checks.
104. **`jig-protocol-charter` may link `smith-channel`** (2026-10-12, the
     coordinator's block "jig 10.1: required smith-channel dependency
     fails boundary rule", 2026-10-12T15:47:06Z). A plan defect: 10.1's
     Files add `smith-channel` to the crate's `Cargo.toml`, and its Shape
     decodes with `smith_channel::CallName::decode`; code-organization.md,
     section 3 says "Only `jig-protocol-charter` and
     `jig-protocol-transcript` link smith" and "later sessions add their
     crates to them"; but jig 01.4's `smith_breaches`
     (`tests/boundary/src/tests/graph.rs:408`) lists only
     `smith-domain-run` and `smith-charter` for the crate. 10.1 adds
     `smith-channel` to that list. Its "Don't" ("`tests/boundary`'s rule
     for the crate stays as jig 01 leaves it") binds the rule on
     `jig-domain-*` crates it names, not smith's list. From now to the
     run's end, a dependency section 3 allows that `tests/boundary`'s
     lists lack is added by the increment that adds it, as section 3 now
     says.
105. **jig-10.1 landed with its Step outside the trailers** (2026-10-12,
     the coordinator's block "jig 10.1: landed commit Step line is
     outside parsed trailers", 2026-10-12T15:56:22Z). jig `75bd3ec6` has
     `Proof:`, `Plan: jig-protocol` and `Step: jig-10.1` in its body, with a
     paragraph after them, so git parses only `Gate-Tree` and
     `Gate-Result` as its trailers. A landed commit is never rewritten
     (development.md, 8.4): the ledger records jig-10.1 landed at
     `75bd3ec6`, `plan status`'s line "jig-10.1: landed in the ledger, no
     commit on origin/main says so" stays and is this, and `plan pulse`'s
     and `plan metrics`' counts read one step and one proof fewer, which
     the retrospective adds back from this ruling. From now to the run's
     end, every session checks its tip before `land`: `git log -1
     --format='%(trailers:key=Step,valueonly)'` prints its step.
106. **A world's restore pages fill across the live ranges** (2026-10-12,
     the coordinator's block "jig 11.1: restore page expectations outside
     increment Files", 2026-10-12T16:02:42Z). jig 11.1's worlds load
     `RestoreRange::LIVE[0]` to `LIVE[3]` in turn; paged one range at a
     time, the core world's pages come out as 1, 2 and 1 rows, and
     `tests/core/tests/restore_pages.rs`'s
     `paused_polls_preserve_multiple_whole_pages_and_repeated_release_is_safe`
     fails. `protocol/store.md`, 4.3 says "a page that exhausts one
     interval with room left goes on into the next in the same step",
     and section 7 calls the core's live ranges "a fixed list of
     intervals, in that order, paged and handed up typed"; 11.1's Test
     shape says "The existing restart, store and conformance stories run
     unchanged." So the world's `LoadCore` pages the four ranges' rows,
     in order, as one sequence, with today's page size, in
     `tests/core/src/effects.rs` (11.1's Files), and `restore_pages.rs`
     stays as it is.
107. **jig 03.2 routes the historical escalation in the fake world**
     (2026-10-12, the coordinator's block "jig 03.2: historical
     escalation route missing outside Files", 2026-10-12T16:03:18Z). A
     plan defect: 03.2's Shape answers a stale decision "through
     `Now::HistoricalEscalation`", and two of its three stories reach it,
     but the test root sends it to `unreachable!`
     (`testing/jig-test-domain/src/routes.rs:514`) until jig 18.3, late
     in the run. The world already answers the other historical reads
     from its fake store (`tests/core/src/historical.rs`, `proposal`;
     the root's arm at `routes.rs:529-530`), so 03.2 adds
     `HistoricalEscalation` beside them: the root's arm, an `escalation`
     read in `historical.rs` loading `CoreKey::EscalationDecision`, and
     its dispatch in `tests/core/src/effects.rs`. These test-root and
     world lines do not count toward 03.2's Size. jig 18.3 moves it to
     the real store with the others (its session file says so).
108. **Expected literals a renamed vocabulary changes are updated in
     place** (2026-10-12, the coordinator's block "jig 05.1: renamed
     vocabulary trace assertion outside Files", 2026-10-12T16:04:43Z, the
     third block on a test's expected value outside an increment's
     Files, after skein 02.3 and jig 11.1). jig 05.1's vocabulary makes
     the host print `host -> Up(Answer {` where
     `tests/host/tests/simulation.rs:175` expects `host -> Answer {`; 05.1
     updates that literal. The README's Rules now say when such a literal
     is no question.
109. **jig 10.2's Size is 1,400 lines** (2026-10-12, the coordinator's
     block "jig 10.2: tools extraction exceeds production size guide",
     2026-10-12T16:07:27Z). A plan defect: 10.2 said "at most 600 lines of
     production code (about 500 moved)", but the five ranges it moves
     hold 927 lines on jig's `main` (`git show
     origin/main:examples/ops/domain/src/translate.rs | sed -n
     '281,504p;506,825p;1026,1142p;877,1011p' | wc -l` and the same for
     `assemble.rs` 286-416: 224, 320, 117, 135 and 131 lines, 2026-10-12).
     The crate measures 1,353 lines with its declarations, problems and
     renderers; it lands whole, not split. Its dependency on
     `jig-domain-authority` (the core's `Answer` and `Finding`, which the
     core does not re-export) is a vocabulary it translates, which
     code-organization.md, section 3 allows.
110. **A Size counts code, not comments; a generated lock is no edit**
     (2026-10-12, the coordinator's block "jig 13.1: generated lock
     entries and boundary size count", 2026-10-12T16:46:00Z). Sessions
     counted Size three ways (skein 01.1 "nonblank, noncomment ... net",
     jig 10.2 every line of `src/*.rs`, jig 11.1 added lines), and jig
     13.1's count crosses its 240 (ruling 103) only if comments and blank
     lines count: 275 gross, 220 net of them. From now to the run's end,
     a Size counts nonblank, noncomment lines of production code, net,
     outside tests and generated code; and the `Cargo.lock` entries a
     Files-named `Cargo.toml` change generates belong to that change, as
     long as they move no existing package. code-organization.md,
     section 7 says both. jig 13.1 lands as it is.
111. **`ops`'s `ConnectorRead` carries its task** (2026-10-12, the
     coordinator's block "jig 10.4: ConnectorRead lacks authenticated
     task", 2026-10-12T16:51:02Z). A plan defect: 10.4's sketch of
     `Event::ConnectorRead` has no task, but the root admits a read with
     `core.connector_read_admit(run.raw(), ..)`
     (`examples/ops/domain/src/assemble.rs:442`), and the root reads no
     value carried between children (root.md, 11), so it cannot take the
     task from the call's name. The event gains `task: u64`, which `ops`'s
     world, standing in for the protocol layer, copies from the
     `Now::Call`'s authenticated run. No core or fleet lookup is added.
     The design names no such event: it is `ops`'s root's own.
112. **A new variant's arms elsewhere belong to its increment**
     (2026-10-12, the coordinator's block "jig 03.3: new refusal needs
     tools renderer outside Files", 2026-10-12T16:57:50Z). jig 03.3 adds
     `tasks::Refusal::Unproposable`; jig 10.2's `task_problem`
     (`crates/jig-protocol-tools/src/render.rs:213`), which landed during
     03.3, matches every refusal and no longer builds. 03.3 adds the arm,
     `jig_domain_tasks::Refusal::Unproposable => b"no proposal could be
     accepted"`, in the words of `domain/tasks.md`, 9 ("saying that no
     proposal could be accepted"), and one unit test of it, as it
     proposed. The README's Rules now say so for every new variant: the
     fourth block in two hours on an edit outside an increment's Files.
113. **jig 13.4's `admits` says which stored rows a scope reads**
     (2026-10-12, the coordinator's block "jig 13.4: notes positions and
     stored-row admission mismatch", 2026-10-12T17:08:12Z). A plan
     defect, not a design gap: 13.4's Shape made `reads::admits` a wrapper
     of `rows::position_allowed`, but that predicate admits a cursor's
     position (an entry's key for notes, only `Ended` for a result, any
     activity of a reply's task), while store.md 4.3 checks intervals
     against the predicate of the rows a range reads, and 7.3 serves notes
     from "the notes' lines". `admits` is that predicate, of its own; the
     core's notes paging and cursors are not changed, as jig 16 already
     says ("The core's notes paging is not changed"). 13.4 also needs jig
     11.2's split families for its spend and party-proposal scopes; its
     Needs line now says so.
114. **A small fix outside Files that the gate needs is no question**
     (2026-10-12, the coordinator's block "jig 11.2: funding saturation
     fixture outside Files", 2026-10-12T17:11:44Z). jig 11.2 frees a
     funding slot, so `tests/tasks/tests/memory.rs`'s saturation fixture
     needs a third live source before its refusal: three input lines, no
     assertion changed. The sixth block in three hours on an edit outside
     an increment's Files (rulings 104, 106, 107, 108, 112): the README's
     Rules now make such a fix, at most 20 lines, keeping every test's
     name and expected values, an implementation choice in any session's
     files.
115. **jig 12.1 proves forgetting through `Event::ForgetChat`**
     (2026-10-12, the coordinator's block "jig 12.1: test-root SeenChat
     route outside Files", 2026-10-12T17:11:44Z). The implementer's own
     proposal: marking replies seen while the chat is live, then
     forgetting it through the root's existing `ForgetChat`, proves the
     index without `Now::SeenChat`, which the test root routes from jig
     18.3 (its session file says so).
116. **A new module that section 7 calls for is inside Files**
     (2026-10-12, the coordinator's block "jig 12.1: cursor helper module
     outside Files", 2026-10-12T17:14:31Z). jig 12.1 must add the `Update`
     key to `tests/client/src/joint/wire/cursor.rs`, a file past 800
     lines that grows by at most 20 (code-organization.md, section 7:
     "new code goes in new modules"); it puts 44 lines in a new
     `cursor/index.rs` and 3 wiring lines in the file. The README's Rules
     now count such a module inside the increment's Files.
117. **A closed period's pools leave the live Project document**
     (2026-10-12, the coordinator's block "jig 11.2: closed funding
     baseline fixtures contradict live-only restore",
     2026-10-12T17:49:20Z). A plan defect: 11.2 evicts a closed ledger
     from `domain.funding` and refuses it at restore, and checked the
     readers of `FundingRecord.closed` against that, but not
     `crates/jig-domain-core/src/tests/project_baselines.rs`, whose first
     and third tests restore closed rows and expect a closed configured
     pool in the live Project document. The design reads past periods
     from the store: client/domain/pages.md, Spend, "the project's
     period, spent, reserved and left; each person's pool in it; ...
     past periods, read a page at a time"; store.md 7.3, "a project's
     spend | the funders' periods of the project". So those two tests
     restore open rows only and assert a closed period's pools absent,
     keeping their names and their other expectations; no history join
     is added to the Project document.
118. **No test forgets a chat by its own event; ruling 115 amended**
     (2026-10-12, the coordinator's block "jig 03.4: forgetting a
     never-marked chat with no replies", 2026-10-12T17:49:58Z). The case
     jig 03.4 raises is decision 31's "one never read waits for retention
     (`later.md`)", and `later.md`'s retention row holds "chats
     forgotten": 03.4 goes on as planned, adding nothing. But jig 03's
     "Done when" says "No chat is forgotten by a test's own event", and
     ruling 115 had jig 12.1 forget a chat by sending
     `core::Event::ForgetChat` from its test. jig 12.1's forgotten-chat
     test forgets the chat as 03.4 makes the domain do it: the person
     marks the ended chat's replies and its result seen. jig 12.1 had
     landed (`4a86719c`) before this ruling, with that test at
     `tests/core/tests/reads.rs:217`: jig 03.4 converts it as it converts
     `activity.rs:207`, keeping its name and assertions (ruling 114).
     jig 03's check now greps `core::Event::ForgetChat`, since the people
     child's worlds send the child's own `Event::ForgetChat`
     (`tests/people/src/world.rs:347`, `tests/people/tests/memory.rs:257`),
     which stays.
119. **jig 03's check reads the scenarios only** (2026-10-12, the
     coordinator's block "jig 03.4: Done-when grep also matches prescribed
     loader replies", 2026-10-12T18:01:14Z). Ruling 118's check still
     matched the worlds' loaders, which answer `Now::LoadEndedChat` with
     `core::Event::ForgetChat` as 03.4's Shape says. It now greps
     `tests/*/tests`, the scenarios (jig `b8eb5aef`): on jig 03.4's branch
     (`14f4a908`), `git grep -n 'core::Event::ForgetChat' --
     'tests/*/tests'` printed nothing, 2026-10-12T18:02Z.
120. **smith 01.1's gate timeouts are measured once before anything
     else** (2026-10-12, the coordinator's block "smith 01.1: focused
     gate timeout and retry exhausted", 2026-10-12T18:03:40Z). Both gates
     reached smith's 15 s global timeout with 1,823 of 2,436 and 2,157 of
     2,515 tests run; on `main` the same suite runs in 6.3 to 7.4 s
     (`Gate-Result` of smith `bd273f58`, `78222b68`, `1307cb56`), and
     jig's gates in the same hour ran at their usual times. smith 01.1
     runs the suite once under the `measure` profile, which has no
     global timeout, alone in the gate lane: under 12 s, the machine was
     the cause, and it lands again; 12 s or more, it compares its ten
     slowest tests with `main`'s and reports, changing nothing yet.
121. **smith's landings here gate without the fast suite's 15 s timeout**
     (2026-10-12, the coordinator's block "smith 01.1: measured focused
     suite exceeds budget on tip and main", 2026-10-12T18:19:26Z). Under
     ruling 120's measure, smith's focused suite took 16.199 s on `main`
     and 20.953 s on 01.1's tip, on carbon-forge. The only smith gate this
     machine passed (`tmp/gates/smith/a0b07e80…`, 09:01 today) took
     13.593 s for 2,123 tests; `main` has 2,515. smith's `Gate-Result`
     times (6.3 to 7.4 s) are smith's reliability machine's. So smith's
     gate fails on `main` here (development.md, 6: "a gate that fails on
     `main` goes to the user"). So that smith 01, which heads the chain
     through jig 02 to jig 17 and 18, need not wait: on this machine, for
     this run, smith's landings (smith 01.1, smith-repin) run `land
     --gate 'cargo fmt --all --check && cargo clippy --workspace
     --all-targets -- -D warnings && cargo nextest run --workspace
     --profile measure && cargo nextest run --workspace --profile
     fuzzy'`: every test of both suites runs and must pass; only the
     focused suite's 15 s global timeout is dropped. Restored by `land`
     with smith's `scripts/check.sh`, once smith's focused suite fits 15
     s here, or by landing smith from smith's reliability machine.
     Reported to the user at once (development.md, 10.4). Before landing,
     smith 01.1 accounts for the two `smith-bench` tests about six times
     slower on its tip than on `main`.
122. **A test outside Files expecting the answer the Shape changes is
     updated** (2026-10-12, the coordinator's block "jig 03.5: cursor
     tests outside Files expect Unknown after sign-out",
     2026-10-12T18:33:43Z). 03.5's Shape: a loaded read whose sign-in
     ends answers `SignedOut`; its session file lets `tests.rs` update
     equalities "unless their case is now signed out". Two tests in
     `crates/jig-domain-core/src/tests/conversation_cursor.rs` (:262,
     :486) remove the sign-in and still expect `Why::Unknown`: they
     expect `Why::SignedOut`, inputs and names kept. The README's Rules
     now say so for any increment, at most 4 assertions.
123. **A removed mechanism's tests move with its removal** (2026-10-12,
     the coordinator's block "jig 05.4: migrate removed Told tests
     outside Files to typed activity", 2026-10-12T18:40:51Z). 05.4's Shape
     removes `Event::Told`, `Request::Told` and `call::told`; four
     assertions in `crates/jig-domain-fleet/src/tests.rs` test them.
     development.md, 8.2 already makes them 05.4's; they move to the typed
     `Observe(Activity::Active)` behaviour, names and the owned and
     fenced-channel behaviours kept. The README's Rules now say so for
     any increment that removes a mechanism.
124. **Test ceilings bind the tests an increment writes** (2026-10-12,
     the coordinator's block "jig 03.5: existing unchanged cursor fixtures
     exceed unit ceiling", 2026-10-12T18:46:03Z). Two
     `conversation_cursor.rs` tests, changed by 03.5 only in their
     expected value (ruling 122), timed 78 and 50 ms in one focused run,
     where earlier runs timed them at 10 to 18 ms and every cursor test
     rose alike. development.md, 8.2's "A test above twice its ceiling is
     made cheaper before it lands" binds the tests an increment adds, or
     whose inputs or setup it changes; 03.5 lands as it is. The README's
     Rules now say so.
125. **jig 12.3's census keeps its progress in a private, bounded map**
     (2026-10-12, the coordinator's block "jig 12.3: bounded census
     progress field for duplicate rows", 2026-10-12T18:59:33Z). 12.3's
     Shape: "a row it is handed twice (a chat with an unread result)
     changes nothing the second time"; today a second ended-chat row adds
     its unread replies again (`people/src/inbox.rs:412-437`). 12.3 adds a
     private `Core` field, a census-progress map bounded by
     `limits.people.people`, two high-water marks per party, initialised
     by `Core::census`, updated by `restore_inbox_row`, priced in `Core`'s
     `worst_case` and emptied when the census ends; every public signature
     stays. The README's Rules now say a private field is no question.
126. **A merge retires a refused hint's number** (2026-10-12, the
     coordinator's block "jig 04.3: merged hints retire refused message
     numbers", 2026-10-12T19:00:04Z; filed as a design gap, ruled by the
     supervisor as a reading of `domain/tasks.md`, 8.2). A refused
     message "stays in the inbox, untaken", so it is still waiting, and
     news, notices, amendments and timers merge with what is "already
     waiting ... since news is a hint to read afresh". When a merge
     retires a refused word, the attempt's kept refusals drop its number:
     `TaskRecord.refused` names only messages still in the inbox, and the
     newer hint is delivered as any message is (option A). No type
     changes.
127. **jig 06.4 lands the unread transcript's events** (2026-10-12, the
     coordinator's block "jig 06.4: unread transcript has no
     preparation-transient event", 2026-10-12T19:08:39Z). A plan defect:
     06.4's Scope says a transcript the store answers as unread "fails the
     run as transient, and the next run of its task starts fresh ... the
     core's transcript load gains that answer here", and jig 17's file
     expects "the core event jig 06 lands for it", but 06.4's Shape names
     none. 06.4 adds `core::Event::TranscriptUnread { task }` and
     `tasks::Event::PreparationUnread { task }`, as its implementer
     proposed (store.md, 8.3: "the run fails as transient and the next
     starts fresh"); its Shape, Test shape and Size (120) now say so.
128. **jig 14.2's Don't reads over the whole family** (2026-10-12, the
     coordinator's block "jig 14.2: scoped codec types contradict row
     reachability constraint", 2026-10-12T19:18:22Z). 14.2's Scope names
     26 types its four rows (`ledger`, `closed_ledger`, `writer`, `pool`)
     do not reach, and its Don't said "declare a type the tasks rows do
     not reach". The Don't keeps dead types out of the schema; the 26 are
     reached by `live`, which 14.4 adds. It now reads over the family's
     rows once 14.4 has added them all; 14.2 also declares `Name` and
     `Writer`, which its `writer` and `pool` rows need, and round-trips
     the 26 directly until 14.4.
129. **Files follow the code** (2026-10-12, the coordinator's block "jig
     06.5: claim helpers moved to charters.rs outside Files",
     2026-10-12T19:23:16Z). jig 07.4 moved the host factory and
     `ClaimDone`'s construction into
     `crates/jig-domain-core/src/charters.rs:68-92`; 06.5's Files name
     their old places. Decision 44: "landed code wins where they differ".
     06.5 changes them in `charters.rs` (`Core::claim_proof`,
     `Core::claim_done`), its boundary signatures as planned. The README's
     Rules now say so for any increment.
130. **The codec increments' Sizes double** (2026-10-12, the
     coordinator's block "jig 14.2: production size 879 exceeds 450-line
     guide", 2026-10-12T19:39:05Z). jig 14.2 counts 879 lines against 450:
     schema 75, facade 120, translations 680 for the 26 types ruling 128
     confirmed and four rows, each file under 800. The plan priced the
     exhaustive translations at about half what they take. 14.2 lands at
     879 (Size 900); the codec increments not yet landed double theirs:
     14.3 and 14.4 to 1,200; 19.1 to 1,200, 19.2 to 900, 19.3 to 1,200,
     19.4 to 1,100. Each stays a guide (ruling 103).
131. **A task's `ended_below` counts over its life, saturating**
     (2026-10-12, the coordinator's block "jig 12.4: ended descendant
     count lifetime across batches and periods", 2026-10-12T19:51:53Z;
     filed as a design gap, ruled by the supervisor as a reading of the
     plan). A recurring task resets `made` to 1 at each batch
     (`crates/jig-domain-tasks/src/recurring.rs:52,67`), and a standing
     one at each period (`standing.rs:109`), so one task's ended
     descendants can outnumber `limits.tasks` over its life. 12.4's Shape
     says "how this task's ended descendants ended, summed as each
     delegate ends", and decision 9 "an ended goal's index row keeps its
     subtree's phase counts"; neither resets. The goals row 12.4 replaces
     counts every ended task below the goal, of every batch and period,
     with `saturating_add` (`crates/jig-domain-core/src/reads/rows.rs`,
     Goals arm, on jig `59010f33`). So `ended_below` keeps lifetime
     counts, each added with `saturating_add` and held at `u32::MAX`; it
     is never reset with `made`; `valid_record` does not bound it by
     `made` or `tree_tasks`. `recurring.rs` and `standing.rs` are not
     changed.
132. **A codec increment lands past its Size, up to twice it**
     (2026-10-12, the coordinator's block "jig 19.3: people codecs exceed
     revised 1200-line guide", 2026-10-12T19:57:25Z). jig 19.3 counts
     about 1,700 lines against 1,200 (ruling 130): the schema and the
     exhaustive translations of the 45 types and six rows its Scope
     names, each file under 800 lines. The second codec increment past
     its guide in twenty minutes, after jig 14.2 (ruling 130): the plan
     priced these translations low, and doubling the guides did not
     cover them. So from now to the run's end, a codec increment (jig
     14.3, 14.4, 19.1 to 19.5) whose count passes its Size × 1.2 lands
     without asking, up to twice its Size, when its production code is
     the schema, the facade and the translations of the types and rows
     its Scope names, and nothing else; its report gives the count by
     file. Past twice its Size it asks, as code-organization.md, section
     7 says. The 800-line file limit and the 40-line test function stay
     rules. jig 19.3 lands at its count. The README's Rules now say so.
133. **A family's later codec increments name its facade** (2026-10-12,
     the coordinator's block "jig 14.3: required tasks codec facade
     omitted from Files", 2026-10-12T20:01:22Z). A plan defect: 14.3's
     and 14.4's Shape is "as 14.2", whose facade
     (`crates/jig-protocol-store/src/codec/tasks.rs`, 128 lines on jig
     `3f0ce934`) maps the family's limits and dispatches encode and
     decode by row, and 14.4's Shape says "`codec::tasks::encode` and
     `decode` have one arm per" row; but only 14.2's Files named the
     facade. 19.2 and 19.4 have the same gap for `src/codec/core.rs` and
     `src/codec/people.rs`. Each now names its family's facade, for the
     limits and the encode and decode arms of the rows and bounded fields
     it adds, and nothing else of it, at the increment's Size.
134. **A row the codec does not translate yet joins its refusal arm**
     (2026-10-12, the coordinator's block "jig 12.4: codec-following rule
     before the changed record codec exists", 2026-10-12T20:02:08Z). The
     README's rule "A durable record's codec follows its record" binds a
     record whose translation has landed. On jig `115258a1`, the tasks
     facade translates `Ledger`, `ClosedLedger`, `Writer` and `Pool`
     only, and refuses `Update`, `Result`, the conversations, `Live`,
     `Ended` and the rest in one arm
     (`crates/jig-protocol-store/src/codec/tasks.rs:80-93`); 14.4's Scope
     names "`TaskRecord` (with ... jig 12's `Endings` where they have
     landed); ... jig 12's index rows that have landed (... `GoalRow`
     ...); each `tasks::Stored` variant on `main` when the increment
     starts". So jig 12.4 adds `domain::Stored::Goal(_)` to that arm, and
     neither encodes `GoalRow` nor `Endings` nor moves the tasks family to
     version 2; jig 14.4 translates them in tasks-v1. The README's rule
     now says so for every family.
135. **The people codec's limits take the core's** (2026-10-12, the
     coordinator's block "jig 19.3: people codec needs core policy
     limits", 2026-10-12T20:03:42Z). A plan defect: 14.1's pattern gives
     each family `codec::<family>::limits(&<family>::Limits)`, but the
     people family's `Policy` row is admitted against
     `core::Limits.policy_bytes` and the authority limits
     (`crates/jig-domain-core/src/routing/vocabulary.rs`, `Limits`, which
     holds `policy_bytes`, `authority` and `people`), not
     `people::Limits.amendment_bytes`. So `codec::people::limits` takes
     `&core::Limits` and answers `Option<people_v1::Limits>`: a field one
     kind of row carries takes that row's bound, and a field `Policy` and
     an ask share takes the larger, as jig 14's Choices say ("each `bytes
     N` or `list N` takes the domain limit that bounds it (an aggregate
     where the domain bounds only a sum ...)"). The translations check
     each field against its ceiling and no sum, as the tasks family's do
     (`crates/jig-protocol-store/src/codec/tasks.rs`, `limits`, on jig
     `0b04cf8d`); a full policy's and an ask's sums stay the domain's.
     No facade `Limits` type is added, and 19.5's `codec::Limits { people:
     people_v1::Limits, .. }` and `codec::limits(&core::Limits)` are
     unchanged. 19.3's Shape says so.
136. **jig-protocol-store's tests build a `core::Limits`** (2026-10-12,
     the coordinator's block "jig 19.3: dev dependencies for core Limits
     fixture", 2026-10-12T20:07:13Z). Ruling 135 has the people codec
     take `&core::Limits`, whose `authority`, `fleet`, `brief`,
     `accounts` and `views` fields name types of crates that
     `crates/jig-protocol-store/Cargo.toml` does not depend on (on jig
     `9ca1be6a` it names `jig-domain-core`, `-tasks`, `-people` and `-notes`).
     jig 19.3's Files gain that `Cargo.toml`, for five
     `[dev-dependencies]` only: `jig-domain-authority`, `jig-domain-fleet`,
     `jig-domain-brief`, `jig-domain-accounts`, `jig-domain-views`, each
     `.workspace = true`; `[dependencies]` stay as they are. The fixture
     is one `core::Limits` constant in `src/tests/fixtures.rs`, which 19.5
     reuses for `codec::limits` and `sizes::check`. Where
     `tests/boundary` refuses a dev dependency, its list gains it in the
     same increment (ruling 104).
137. **jig 03.6 owns the fuzzy case its change reaches** (2026-10-12, the
     coordinator's block "jig 03.6: fuzzy gate failure outside increment
     Files", 2026-10-12T20:09:12Z). `fuzzy_joint`'s seeds are fixed
     (`tests/client/tests/fuzzy_joint.rs:5-10`: `FIRST_SEED` 4500, 64
     seeds and the pinned ones), and `main`'s gate passes
     `random_people_with_failed_commits_and_drop_watch`, so 03.6's change
     is what reaches the failure at `tests/client/src/model.rs:101`
     (node 351, written 89: platform text `**h`, view text `100`). 03.6
     replays it and fixes where the fault is: in its own code, as its
     work; outside its Files, under ruling 114 (at most 20 lines, every
     test's name and expected value kept), named in its commit body.
     A fix past 20 lines outside Files, or one that changes an expected
     value, is reported with what the replay found, and nothing lands.
138. **skein 02.s1 and jig-repin.s1 make generated code pass clippy**
     (2026-10-12, the coordinator's block "jig 19.3: generated people
     codec fails workspace clippy", 2026-10-12T20:05:22Z; the designer's
     answer, skein `482691a7`, jig `bebfb548`). jig 19.3's generated
     `people_v1.rs` fails `large_enum_variant` (`Ask::Amend`, 432 bytes
     against 168), `collapsible_if` and `too_many_lines`; skein's
     `codec.md`, 6 has generated files pass the lints "with no
     allowances". The designer added `boxed` on a variant to the schema
     language. Two steps not in the plan follow: skein 02.s1, the
     generator's `boxed` and output the lints accept
     (`skein/02-startup-and-secrets.md`); then jig-repin.s1, jig's
     `skein-codegen` pin alone (README, "Repin"). jig 19.3 needs both. The
     README's Rules now say how any codec increment meets
     `large_enum_variant`.
139. **The client model keeps what the person typed** (2026-10-12, the
     coordinator's block "jig 03.6: replay reproduces local-edited value
     model assertion mismatch", 2026-10-12T20:17:05Z). Ruling 137's
     replay fails at `tests/client/src/model.rs:101`: the reference
     model compares a field's text with the view's whatever the person
     typed, while `client/domain/view.md`, 8 says "a matched field whose
     write count did not move gets no value patch, whatever its text:
     what the person is typing stays theirs". A test defect that 03.6's
     change reaches. 03.6 repairs the model as it proposed: an entry
     marked edited by `input`, cleared by a `Value` patch; equality
     checks identity, size, value presence and write count always, and
     text only when unedited; a `Value` patch at an unchanged write count
     fails the model. At most 30 lines in `model.rs` and one regression
     test; no other assertion changes.
140. **A gate that fails only on a timeout lands on the third try**
     (2026-10-12, the coordinator's block "jig 12.4: same-tree gate
     timeout after permitted retry", 2026-10-12T20:18:04Z). Both of jig
     12.4's gates stopped at the focused suite's 15 s global timeout
     (2,488 and 2,312 tests passed), killing tests in `jig-view` and
     `jig-scripted-engine`, which 12.4 does not touch, on one code tree,
     with no assertion or compile failure; the first began at load 16 with
     a code-graph indexer at 565 % CPU outside the queue. The README's
     "For the user" item (c) counts about two such failures an hour.
     From now to the run's end, a gate that fails only on a timeout, in
     tests outside the session's files, is retried by `land` up to three
     times in all on the same tree, without asking; a third failure is a
     block, kind infrastructure. This replaces development.md, 8.5's "a
     second failure on the same tree is a question" for this plan. The
     README's Rules now say so; jig 12.4 runs `land` once more.
141. **jig 05.6's needs take the hub's turns and price their bytes**
     (2026-10-12, the coordinator's block "jig 05.6: host turn and
     retained Needs bounds absent from construction",
     2026-10-12T20:19:39Z). A plan defect: 05.6's Shape builds
     `Needs::unacknowledged` from "the hub's `Limits::turns`"
     (`crates/jig-domain-host/src/limits.rs:51`, "Normal retained turns
     per attempt"), but `Core::new` receives `Config` and `core::Limits`
     only, and `fleet::Limits::turns` counts turns "across all attempts".
     engine.md, 8: "fewer turns kept unacknowledged than the hub's bound".
     So `core::Config` gains `hub_turns: u32`, which each root sets from
     its hub's `Limits::turns` (05.6's Files name `Config`); and the
     `Needs` the fleet keeps are bounded by a new `fleet::Limits::needs_bytes:
     u64`, checked at `Domain::new` and priced in `fleet::worst_case`.
     Each `Config` or `fleet::Limits` literal outside 05.6's Files gains
     its one line, named in the commit body, whatever their number.
142. **skein 02.s1's proof reads existing goldens only** (2026-10-12, the
     coordinator's block "skein 02.s1: lint fixture goldens contradict
     unchanged-goldens proof", 2026-10-12T20:24:29Z). The supervisor's
     defect: 02.s1's Test shape grows `lints-v1.schema`, whose drift test
     then needs new goldens under `tests/codec/golden/v1/`, while its
     Proof asked `git diff --stat origin/main -- ... tests/codec/golden`
     to print nothing. The Proof now runs with `--diff-filter=MD`: no
     existing generated file or golden changes or goes; the new lint
     goldens are added and listed in the commit body.
143. **ops's configuration carries its inline agent's bounds**
     (2026-10-12, the coordinator's block "jig 05.6: ops construction has
     no authentic agent Bounds input", 2026-10-12T20:25:10Z). A plan
     defect: 05.6 has `ops`'s construction send its hub's hello on channel
     0, but `examples/ops/domain/src/domain.rs`'s `Config` holds no
     agent bounds. hosts.md, 4: "The engine's root translates its own
     hub's link as it translates a worker's ..., its hello stating the
     bounds of its `smith-inline-agent`'s configuration". So `ops::Config`
     gains `agent_bounds: host::Bounds` (`crates/jig-domain-host/src/link.rs:73`),
     which construction puts in the hello; each producer of an `ops`
     `Config` supplies its configured agent's bounds, the worlds from
     their agent fixtures. Nothing derives them from a charter. Each
     `Config` literal outside 05.6's Files gains its one line, named in
     the commit body.
144. **The code-graph daemon runs at idle priority; jig's line restarts**
     (2026-10-12, the coordinator's block "jig 12.4: third timeout-only
     gate failure and Jig infrastructure hold", 2026-10-12T20:32:28Z).
     jig 12.4's three gates each stopped only at the focused suite's 15 s
     global timeout, in tests it does not touch, with the code-graph
     service (`codebase-memory-mcp --cbm-daemon-internal`, pid 272) at
     565 % and 569 % CPU of eight cores at the first and third gate's
     start: its watcher (`auto_watch = true`) indexes a checkout when it
     changes, so each landing that moves `main` starts an index just as
     the next queued gate starts. At 20:35Z the supervisor set every
     thread of pid 272 to `SCHED_IDLE` and nice 19, and its IO to the
     idle class (`chrt --idle`, `ionice -c 3`), which the workers it
     forks inherit: indexing now takes only CPU nothing else wants, and
     the index still follows `main`. The coordinator lifts jig's hold
     (development.md, 10.6), and jig 12.4 runs `land` again, a fourth
     attempt on its tree; ruling 140 applies afresh from it. Reported to
     the user under "For the user".
145. **A gate failure the Shape's new behaviour explains is fixed without
     asking** (2026-10-12, the coordinator's block "jig 04.4: fault
     fixture leaves a message unread before parking",
     2026-10-12T20:32:28Z; the second such block, after jig 03.6's at
     20:28Z). 04.4 makes an unread message start another attempt
     (tasks.md, 8.3: "A message a run refused or left unread (8.2) is
     judged again when the run ends, as if it arrived then"), and the
     shared fixture `tests/core/src/faults.rs:26` parks a turn with
     `read: None`, so `a_slow_worker_keeps_its_assignment_and_resumes_after_the_delay`
     sees two activations. 04.4 makes the fixture read the message before
     parking, as it proposed, under ruling 114. The README's Rules now
     make such a fix no question for any session.
146. **A root with engine slots and no hello is left as it is** (2026-10-12,
     the coordinator's block "jig 05.6: engine placement before hello",
     2026-10-12T20:37:32Z, the question jig 05's file plans to raise).
     The design's reading is clear: the engine's own slots are "hosted by
     the engine's own hub ..., its agent `smith-inline-agent` with no
     workspace, present from construction" (engine.md, 8), whose hello
     states "the bounds of its `smith-inline-agent`'s configuration"
     (hosts.md, 4), so that "no run is placed where smith would refuse
     its start". Making every root say it, and the fleet place nothing
     before it, changes the constructors and worlds of roots other than
     `ops`, beyond 05.6's Scope. Plan scope, not design: 05.6 keeps the
     planned behaviour (`ops` says its hello; other roots unchanged), and
     the tightening goes to `later.md`.
147. **jig-repin.s1 moves the whole skein source** (2026-10-12, the
     coordinator's block "jig repin.s1: cargo update moves ten other
     skein packages", 2026-10-12T20:44:30Z). The supervisor's defect:
     `cargo update -p skein-codegen --precise e723e1db` moved
     `skein-codegen` and ten other skein packages from `90326012`, which
     ruling 138's step forbade. Between those revisions, skein changed,
     besides its docs, generator and codec tests, only
     `crates/skein-llm` (which jig does not compile) and two lines of
     `testing/skein-world/src/tree.rs` (`git diff --stat 90326012
     e723e1db`). development.md, 5.7 has a repin move "its whole git
     source", and never edit `Cargo.lock` by hand. So the step takes
     cargo's move as it is: every skein package at `e723e1db`, smith's
     unchanged, the gate the check.
148. **The ops crash story's random person never signs out** (2026-10-12,
     the coordinator's block "jig 03.6: two-point sign-in repair replay
     still fails", 2026-10-12T20:49:00Z, after the blocks of 20:28Z and
     20:43Z). jig 03.6's "Sign out" control, pressed by
     `world.random_person` inside `tests/ops/tests/fuzzy_joint.rs::wander`,
     ends the sign-in that `p_story_survives_seeded_commit_crashes_and_random_presses`
     goes on to need; signing in again fails there, since the story runs
     under seeded commit crashes and the tree shows "Offline" (seed 537,
     client 0, step 1). That story checks surviving crashes and random
     presses, not signing out, which 03.6's own stories check. So in that
     story only, the random person's presses leave out the "Sign out"
     button; the client's own fuzzy tests keep it. The 14-line re-sign-in
     helper is dropped. At most 20 lines under ruling 114; seeds, names
     and expected values kept.
149. **jig 30's ceilings follow jig 14's rule; `Owned` stays unversioned**
     (2026-10-12, the coordinator's block "jig 30.1: field ceilings and
     envelope version proof", 2026-10-12T21:08:10Z). Two plan defects.
     First, 30.1 took each ceiling from "`jig_domain_client::Limits`'
     largest profile value", but no client profile exists before jig 20,
     and its fallback of 65536 for every unbounded field makes a `list`
     of `bytes` pass a `u32` worst case (4 + 65536 × 65540), which
     skein's `codec.md`, 4 refuses. 30.1 now applies jig 14's Choice
     ("each `bytes N` or `list N` takes the domain limit that bounds it
     ..., with a ceiling at least the worlds' fixtures' value and as large
     as keeps each type's worst case within a `u32`"): the largest value
     any `Limits` literal on `main` gives the bounding field; unbounded,
     `bytes`/`text` 65536 and `list` 256, halved together within a record
     until it fits. Second, its test
     `an_owned_record_of_another_family_version_is_refused` contradicts
     the sketch's unversioned `Owned { owner, version: u32, record }`:
     client.md, 9 has "an exchange names the version of each owner's
     record it carries", and the range the engine serves is checked by
     jig 31 and 32, not by generated code. So the envelopes stay as
     sketched, and the test becomes
     `an_owned_record_keeps_its_owners_version`.
150. **jig 05's "Activity is typed" check leaves out the diagnostics'
     module** (2026-10-12, the coordinator's block "jig 05.6: Done-when
     regex matches private diagnostic Facts", 2026-10-12T21:09:14Z). The
     check was meant for the opaque link variants jig 05.4 removed
     (`Event::Facts { owner, fact }`, `FromAgent::Facts { owner, fact }`
     in `crates/jig-domain-host/src/boundary.rs`) and `fact_bytes`; its
     eight matches on jig `db2887df` are the private diagnostic queues,
     `pub(crate) struct Facts` in `crates/jig-domain-{host,fleet}/src/facts.rs`,
     which 05.4 kept on purpose ("Keep lifecycle diagnostics and typed
     activity in separate bounded queues owned by Facts", jig
     `9ca1be6a`). The check now excludes `*/facts.rs`, and prints nothing
     on `db2887df`. No renaming increment.
151. **jig 30.1 flattens its refusals' recursive facts** (2026-10-12, the
     coordinator's block "jig 30.1: recursive refusal facts require flat
     representation", 2026-10-12T21:12:53Z). A plan defect: 30.1 carries
     `Refused` "with ... the refusals' facts" field for field, and those
     reach contract types that nest themselves (`Refusal::Ended` →
     `Ending` → `OwnerChipEnding` → `ResultValue` → `Chip` → `ChipPhase` →
     `OwnerChipDetails` → `OwnerChipEnding`, and `ChipPhase::Held` →
     `HoldReason::Dependency` → `Ending`; 99 types in all), while skein's
     `codec.md`, 3 has a schema's types form "a tree" that decodes "to a
     fixed depth". 30.3 already says how ("a record whose contract type
     nests itself by a peer's choice is flattened to a list of rows with
     parent indexes, and the report says how"); 30.1 applies it, and
     declares the document types its refusals reach, which 30.3 and 30.4
     reuse. 30.1's Size guide becomes 900, and the README's rule "A codec
     increment lands past its Size, up to twice it" now covers jig 30.
152. **skein 02.s2 and jig-repin.s2 clean a list of fixed-width items**
     (2026-10-12, the coordinator's block "jig 22.1: generated
     primitive-list unused variables", 2026-10-12T21:21:54Z). For
     `Hello.workstreams`, a `list 65536 u64`, skein-codegen at `e723e1db`
     writes an empty check loop and an unused `item_0` in its measure
     loop (jig 22.1's `src/generated/v1.rs:473,726`), which `-D warnings`
     refuses; codec.md, 6 has generated files pass the lints "with no
     allowances". Two steps not in the plan, as for ruling 138: skein
     02.s2 (`skein/02-startup-and-secrets.md`), then jig-repin.s2 (README,
     "Repin"). jig 22.1 commits on its branch and lands after them.
153. **jig 30's drift test may take 4 s, then 6 s** (2026-10-12, the
     coordinator's block "jig 30.1: enlarged family exceeds drift
     ceiling", 2026-10-12T21:24:53Z). Ruling 151 grew 30.1's family to
     359 declarations and 85,702 generated lines (7.87 MB); its drift
     test takes 3.755 s, generating alone 3.346 s, against a 0.3 s
     ceiling. The focused suite runs tests in parallel (2,808 tests in
     10.8 s on jig `86d24f62`), so one 4 s test costs it little wall time.
     30.1's drift ceiling is 4 s; 30.2 to 30.5's, 6 s; past that, a
     question. The focused suite's 15 s budget is unchanged. Proposed for
     the retrospective: skein-codegen's speed (about 9 ms a declaration)
     and the size of a family that mirrors the client's contract.
154. **A codec checks fields and counts; sums are the domain's**
     (2026-10-12, the coordinator's block "jig 14.4: aggregate inbox
     bytes codec boundary", 2026-10-12T21:25:54Z; the second such block,
     after jig 19.3's, ruling 135). `TaskRecord.inbox` is bounded by
     `inbox_messages` (its count), `message_bytes` (each message) and
     `inbox_bytes` (their sum). jig 14's Choice gives a field the domain
     bounds only by a sum that sum as its ceiling ("an aggregate where
     the domain bounds only a sum: ..., `inbox_bytes`, ..."), and ruling
     135 has "The translations check each field against its ceiling and
     no sum, as the tasks family's do". So no wrapper `Limits` type:
     `codec::tasks::{limits, encode, decode}` keep `tasks_v1::Limits`, and
     `inbox_bytes` is checked by the domain at admission and restore.
     `a_task_record_at_its_inbox_bound_reads_back` holds `inbox_messages`
     messages, their bytes within `inbox_bytes`, and reads back. The
     README's Rules now say so for every codec family.
155. **skein 02.s2 also splits a long record's functions** (2026-10-12,
     the coordinator's block "jig 30.1: generated 23-field encoder
     exceeds 100 lines", 2026-10-12T21:28:30Z). skein 02.s1's Shape said
     "A generated function that would pass 100 lines ... is split", but
     its generator at `e723e1db` split only a family's worst cases: jig
     30.1's `OwnerChipDetailsFields::encode`, for a 23-field record, is
     107 lines. skein 02.s2, still running, takes it: a record's
     generated functions past 100 lines are split into helpers; its
     fixture gains a 30-field record; Size 120. jig-repin.s2 then brings
     both fixes to jig. jig 30.1 marks its large variants `boxed`
     (`Fact.owner_chip_details`, `Ask.amend`), as the README's rule says,
     and lands after jig-repin.s2.
156. **A field several kinds share takes the largest bound** (2026-10-12,
     the coordinator's block "jig 14.4: shared Word bytes ceiling under
     ruling 154", 2026-10-12T21:29:56Z). `Word.words` holds an ordinary
     message's words (bounded by `message_bytes`), a result's payload
     (`result_bytes`) and a notice's or news' (the larger of the two):
     `crates/jig-domain-tasks/src/stored.rs:494-510`. As ruling 135 has
     for the people family ("a field `Policy` and an ask share takes the
     larger"), its ceiling is `max(message_bytes, result_bytes)`; a field
     one kind owns keeps its own bound (`ActivityWords.words`,
     `message_bytes`). The README's rule now says so.
157. **The record split is skein 02.s3** (2026-10-12, the coordinator's
     block "skein 02.s2: scope extension arrived during successful
     landing", 2026-10-12T21:30:36Z). skein 02.s2 landed its list fix
     (skein `be47b1b5`) before ruling 155 reached it; a second landing
     under its Step would make one step two commits. The record split
     becomes skein 02.s3 (`skein/02-startup-and-secrets.md`), and
     jig-repin.s2 pins skein 02.s3's commit, which carries both fixes.
158. **skein 02.s3 lands at 119 lines, with a fixture that splits**
     (2026-10-12, the coordinator's block "skein 02.s3: size 119 exceeds
     96-line margin", 2026-10-12T21:43:45Z). 119 net lines against
     "Size: at most 80" (ruling 103's margin, 96): the split covers four
     kinds of function, and the plan priced it low. Its Size becomes 120.
     Its implementer found the prescribed 30-field fixture leaves every
     function within 100 lines, so skein's gate would not exercise a
     split: the fixture record grows until each of its four functions
     would pass 100 lines unsplit, at most 100 fields.
159. **A compound limit's members are overridden by dotted names**
     (2026-10-12, the coordinator's block "jig 20.3: compound limits
     cannot take integer overrides", 2026-10-12T21:47:28Z; asked for the
     designer, ruled by the supervisor as the plan's syntax). configuration.md,
     6: "A deployment names a profile and overrides any limit by its
     name", so keeping `tasks.retries` (six retry policies),
     `documents.versions` and `core.brief_core_budgets` preset-only would
     contradict it. Each of their scalar members is named by its dotted
     path (`tasks.retries.transient.base_ms`) and overridden alone, and
     `every_limit_of_the_small_profile_has_a_name` lists each. 20.3's
     Don't "or of a limit's part" now reads "or of a whole compound limit
     at once".
160. **A grant's value is smith's envelope, by jig 25's default, until the
     designer answers** (2026-10-12, the coordinator's block "jig 21.2:
     grant access token and account-id representation",
     2026-10-12T21:49:42Z). llm.md, 5.3: "A grant's value is the access
     token and, where the account's provider names one ..., the account's
     id ...; the table holds both"; 21.2's sketch keeps `value: Secret`
     with a token bound only. jig 25's file already names the default
     for this question ("the engine's routing fills an LLM grant's value
     as smith's envelope, from the driver's access token and the
     account's id jig 20 parsed"; smith's agent reads a two-byte
     big-endian length, the id, then the token,
     `smith/crates/smith-agent-service/src/service.rs:759-770`). So
     21.2's `Table` keeps one `Secret` per generation, that envelope, and
     its bound is the value's: 2 + `ACCOUNT_ID_BYTES` (256, a constant of
     `jig-protocol-accounts`) + the token's; no typed `Value`. The
     question goes to the designer (question.md); a different answer
     changes 21.2 and the routing that fills the table.
161. **jig 20.3 lands at 556 lines** (2026-10-12, the coordinator's block
     "jig 20.3: size 556 exceeds 420-line margin", 2026-10-12T21:52:01Z).
     556 net lines against 350: the exhaustive destructuring 20.3's
     Shape asks for ("no wildcard over a field: a field added to a
     child's `Limits` breaks the build until it has a name here"), over
     170 scalar members, ruling 159's dotted compound members among
     them. No macro or generic helper is wanted to shrink it. Its Size
     becomes 600.
162. **Ruling 160 is reversed by decision 37** (2026-10-12, the designer's
     answer to question.md's "jig 21.2: which bytes an LLM grant's value
     is", jig `87195d80`, `b91e5a49`). A grant's value keeps its two
     parts, token and account id, both secret, in the table and on jig's
     channel; the worker's layer builds smith's envelope as it hands it
     to smith's host half. jig 21.2 had not landed on ruling 160's
     envelope (the ledger had it running), so no step is added.
163. **jig 21.s1 changes the landed table; ruling 162 corrected**
     (2026-10-12, the coordinator's block "jig 21.2: typed-pair reversal
     arrives after default table landed", 2026-10-12T21:59:20Z). jig 21.2
     landed (jig `530b795a`, 21:58:15Z) on ruling 160's envelope while
     ruling 162 was written, so 162's "no step is added" is wrong. The
     designer's answer says so for this case ("If 21.2 has landed on
     ruling 160's envelope, the same change is a step of the
     supervisor's"): jig 21.s1 (`jig/21-accounts.md`), before jig 21.3
     lands.
164. **jig 20.4 lands at 744 lines** (2026-10-12, the coordinator's block
     "jig 20.4: size 744 exceeds 660-line margin", 2026-10-12T22:11:30Z).
     744 net lines against 550: strict collected parsing of every
     endpoint, account, sign-in and transport field, as 20.4's Shape
     asks, with one set of field helpers (115 lines) shared by every
     parser. Its Size becomes 800.
165. **Generated tests have a ceiling of their own: 200 ms** (2026-10-12,
     the coordinator's block "jig 22.1: generated bound tests exceed 40ms
     threshold", 2026-10-12T22:11:30Z). skein's generator writes a bounds
     test per bounded field (codec.md, 6: "generated tests decode a field
     at its limit, and one past it"); at jig 22.1's ceilings three take
     72, 91 and 119 ms against 22.1's 20 ms, and no session may edit
     generated code. From now to the run's end, a test under
     `src/generated/` has a ceiling of 200 ms, past which it is a
     question; development.md, 8.2's "made cheaper before it lands" does
     not bind it below that. No measurement is wanted. Proposed for the
     retrospective: generated bound tests at small test limits, since
     the focused suite grew 30 % in this run's last 80 minutes, much of
     it generated.
166. **jig 21.4's Files gain the accounts world's referee** (2026-10-12,
     the coordinator's block "jig 21.4: referee scope exceeds outside-Files
     allowance", 2026-10-12T22:18:32Z). 21.4's handed-in accounts expire,
     are rejected and close, and the referee
     (`tests/protocol-accounts/src/referee.rs`) must wait through expiry
     and stop checking an expired grant: 10 lines added and 18 removed,
     past ruling 114's 20, every expected literal kept, moved into a
     fixture helper. The referee is in 21.4's Files for those three
     predicates.
167. **jig 30.1 waits for a cheaper focused suite; jig's hold lifts for
     the rest** (2026-10-12, the coordinator's block "jig 30.1: third
     focused-suite timeout and Jig hold", 2026-10-12T22:29:02Z). 30.1's
     three gates stopped at the focused suite's 15 s timeout (3,379 of
     5,399 tests run at the third) with nothing failing, its drift test
     at 6.3 to 6.7 s in the gate: its generated family alone takes the
     focused suite from about 3,900 tests (13.6 s, jig's last passing
     gates) to 5,399. Infrastructure, as ruling 140 says, across main's
     moves between attempts. The user asked (22:10Z) that the tests be
     made cheaper rather than the timeout raised, and the designer has
     that analysis (question.md, 22:14Z). So 30.1 keeps its commit and
     lands once the analysis's fixes have made room; jig 30.2 to 30.5 do
     not start before it; every other jig session goes on, ruling 140's
     retries applying.
168. **Every codec increment waits for the cheaper focused suite**
     (2026-10-12, the coordinator's block "jig 19.1: third focused-suite
     timeout", 2026-10-12T22:32:53Z). jig 19.1's generated core family
     takes the focused suite to 4,439 tests; its three gates stopped at
     15 s with nothing failing, as jig 30.1's did (ruling 167). Each
     codec increment adds hundreds of generated tests, one process each
     (the designer's analysis under way: the store's 1,760 golden tests
     take 0.79 s in one process, 17.4 s one per process). So ruling 167
     extends to them: jig 19.1 keeps its commit and lands once the
     analysis's fixes have made room; jig 19.2, 19.4, 19.5 and 30.2 to
     30.5 do not start before; every other session goes on.
169. **Six steps make the focused suite cheaper** (2026-10-12, the
     designer's answer to the supervisor's question "The focused suite's
     cost", asked at the user's request; skein `778ce715`). The suite
     spends 104.7 s of test time: about 35 s starting a process per test,
     17.4 s on generated codec tests one per process (0.79 s in one), 4.1
     s on an O(N²) generator's drift, 9.2 s on the boundary's byte loop
     over generated files, 16.5 s of real work in jig-view, about 20 s in
     the worlds. Six steps not in the plan, in `tests-cost.md`:
     jig-tests.s1 (the boundary's scan), skein-02.s4 (the generator: table
     tests, small-limit bounds, linear), jig-repin.s3 (skein-codegen alone,
     families regenerated), jig-tests.s2 (the view), jig-tests.s3 (the
     worlds), skein-tests.s1 (skein-heap). About 35 s less, with room for
     jig 30.1's family. jig 19.1 and 30.1 regenerate and land after
     jig-repin.s3, and the codec increments start again (rulings 167,
     168). Ruling 165's 200 ms ceiling ends at jig-repin.s3: a generated
     table test then has 100 ms; ruling 153's drift ceilings go back to
     the world's.
170. **jig 07.s1: the core's settings come from the root and the policy**
     (2026-10-12, the designer's answer to question.md's "jig 20.5:
     configuration's sources for the core's Settings, and projects'
     resources and homes"; jig `2f7b8800`, `e9d8dd7f`, `e41d51b7`).
     configuration.md, 3 now names a source for each value; five of
     `Settings`' fields (`period`, `period_budget`, `person_budget`,
     `chat_authority`, `recurring_connector`) are not configuration, and
     the code holds them deployment-wide where the design has them per
     project, per role or the core's own. A step not in the plan, jig
     07.s1 (`jig/07-charters-and-budgets.md`), moves them before jig 20.5
     lands; resources and homes come by adoption after the first start,
     and homes wait (`later.md`).
171. **A close drains an exchange in flight, as skein's design says**
     (2026-10-12, the coordinator's block "jig 21.5: close drains versus
     cancellation", 2026-10-12T22:44:15Z). A plan defect: 21.5 had "a
     close during an exchange cancelling it", but skein's `oauth.md`, 6.6:
     "**Close drains.** ... An exchange in flight with the issuer
     finishes, its record is kept, and then its connection closes", and
     llm.md, 5.2 maps close to close. 21.5's test is now
     `a_close_during_an_exchange_drains_it`, closing once the fake issuer
     has seen the refresh and expecting `Closed` only after it finishes;
     no cancel is added to production routing. The accounts world's
     referee (`tests/protocol-accounts/src/referee.rs`) is in 21.5's
     Files for the close trigger and a damaged record's usability (its
     `became_usable` assumes every story is granted), as ruling 166 did
     for 21.4.
172. **An assignment's resumed turns: 480 of at most 8 MiB** (2026-10-12,
     the coordinator's block "jig 22.3: assignment transcript ceiling
     exceeds u32", 2026-10-12T22:46:19Z). 22.3's sketch had `turns: list
     65536 bytes 8388608`, about 550 GB at the ceilings, which skein's
     `codec.md`, 4 refuses ("must fit a `u32` at the ceilings"). The
     profiles on `main` bound a resumed window far below: the core's
     `resume_bytes` 65,536 (small) and 262,144 (standard), the hub's
     `turn_bytes` 65,536 (`crates/jig-protocol-config/src/profile/`). By
     jig 14's ceilings rule (ruling 154), the count's ceiling is 480 and a
     turn's 8 MiB, about 3.9 GiB with the record's other fields; the sum
     stays the domain's (`resume_bytes`). If the domain's limit on a
     transcript's turns (decision 27) is above 480 in a `Limits` literal on
     `main`, the count takes that value and a turn's ceiling the largest
     power of two that keeps the record within a `u32`.
173. **The test-cost steps land without the focused suite's 15 s
     timeout** (2026-10-12, the coordinator's block "jig tests.s1: third
     focused-suite timeout", 2026-10-12T22:48:15Z). jig-tests.s1, which
     makes the suite cheaper (its proof: jig-boundary's 56 tests in 0.49 s,
     the slowest part 0.24 s), failed three gates at the 15 s timeout, the
     last with 4,083 of 4,084 tests passed and none failing: the suite it
     relieves is past its timeout before it lands. So jig-tests.s1,
     jig-tests.s2, jig-tests.s3 and jig-repin.s3 (ruling 169) land with
     `land --gate 'cargo fmt --check && cargo clippy --workspace
     --all-targets -- -D warnings && cargo nextest run --workspace
     --profile measure && cargo nextest run --workspace --profile fuzzy'`:
     every test of both suites runs and must pass; only the focused
     suite's global timeout is dropped, for those four landings. The gate
     of every other landing stays `scripts/check.sh`, and its 15 s stays,
     as the user asked (22:10Z: make the tests cheaper, not the timeout
     longer). Reported to the user at once (development.md, 10.4). jig's
     hold lifts; the test-cost steps land first when the gate lane has a
     choice.
174. **Until the suite is cheaper, a timed-out landing waits for the next
     test-cost step** (2026-10-12, the coordinator's block "jig 21.5:
     third focused-suite timeout", 2026-10-12T22:58:00Z, the third such
     block, after jig 30.1's and 19.1's). Until jig-repin.s3 has landed
     (ruling 169), a session whose three gates fail only on the focused
     suite's timeout keeps its commit and ends its turn, and the
     coordinator runs its `land` again, three attempts, after each
     test-cost step lands (jig-tests.s1 landed at 22:56Z, skein-tests.s1
     at 22:54Z). No block is written for it. The codec increments still
     wait for jig-repin.s3 (rulings 167, 168). The README's Rules say so.
175. **skein 02.s5: generated tests in the strict subset** (2026-10-12,
     the coordinator's block "smith repin: generated table tests violate
     strict consumer lints", 2026-10-12T23:09:26Z). skein 02.s4's
     generated table tests (skein `d135339c`) use `Vec`, `String`,
     `format!`, closures and `slice::join`; smith's channel family alone
     gives 510 clippy errors, and jig's root `clippy.toml` refuses the
     same (`disallowed-types` `alloc::vec::Vec`, `disallowed-macros`
     `alloc::format`, the closure combinators), so jig-repin.s3 fails
     too. skein's own gate passed because its codec fixtures take the
     test harnesses' configuration. A step not in the plan, skein 02.s5
     (`tests-cost.md`, 7), then jig-repin.s3 and smith-repin pin it.
176. **jig 06.s1: a resumed window counts each turn's length** (2026-10-12,
     the designer's answer to question.md's "jig 22.3: how an assignment
     carries a resumed window's turns"; jig `e7eb8486`, `22671972`). A
     window crosses as one piece in smith's form, `transcript: bytes
     8388608`, bounded by the resume limit with each turn counted with
     its four-byte length; this replaces ruling 172. jig 06 has landed,
     so a step not in the plan, jig 06.s1 (`jig/06-*.md`), makes the
     core's window count so. The same answer settles jig 07.s1: a chat
     fits under its person's role, the person batch unchanged.
177. **The workers' component bounds its configured names apart from the
     fleet's workers** (2026-10-13, the coordinator's block "jig 23.1:
     configured names versus concurrent worker capacity",
     2026-10-13T00:01:10Z). A plan defect: 23.1's sketch refused "More
     workers than the limits hold", so with names equal to the fleet's
     workers a third worker is unauthorized, never busy, and
     `a_third_worker_past_the_fleet_limit_is_refused_as_busy` cannot be
     reached. channel.md, 4.1: room is "the fleet's limit on workers, less
     the channels open"; it does not bound the configured names. So
     `jig-protocol-engine-workers`' `Limits` gains `names: u32`, the
     configured names its digest table holds (priced in its worst case);
     `Unusable::Workers` refuses more names than `names`; room stays the
     fleet's workers less the channels open. The world configures three
     names with two workers. The profiles gain `names` in the same
     increment (README, "A component's limits join the profiles").
178. **jig 24.1 proves backoff alone; the proved channel's world test
     moves to 24.2** (2026-10-13, the coordinator's block "jig 24.1:
     proved-channel proof precedes hello translation",
     2026-10-13T00:02:34Z). A plan defect of order: channel.md, 10.1 has a
     channel proved once "it carried the hello and stayed open for the
     span", but the hello's translation first lands in 24.2, outside
     24.1's Files. 24.1 lands its opening and redial with
     `Backoff::proved` proved by a unit test, the proving span wired but
     unreached; 24.2, with the hello, runs the proved-channel world test
     (backoff starting over after a proved channel) as part of its own
     Proof.
179. **skein 02.s6 and jig-repin.s4: limits checked where they are made**
     (2026-10-13, the designer's answer to question.md's "skein 02.s4's
     generated limits check doubles a smith test's time"; skein
     `b68a17a9`, jig `795349b2`). smith's
     `smith_check_json_lists_the_effective_limits_by_name` took 2,045 ms
     with skein 02.s4's generated code, 878 ms with the old: each `new`
     and `decode` checked the family's limits. The designer: generated
     code checks values only, against `limits.min(CEILINGS)`, and the
     family's owner checks its limits once where it makes them. Two steps
     not in the plan (`tests-cost.md`, 8 and 9): skein 02.s6, then
     jig-repin.s4; smith-repin then moves smith's skein source to 02.s6's
     commit, and smith's limits' derivations call their family's check
     where they do not already.
180. **A stored turn's transcript takes the channel's turn ceiling**
     (2026-10-13, the coordinator's block "jig 19.2: authoritative
     transcript bound absent from core Limits", 2026-10-13T00:07:03Z).
     `core::TurnRecord.transcript` (`crates/jig-domain-core/src/stored.rs:531`)
     says "bounded by journal `transcript_bytes`", but skein's
     `JournalLimits` (`crates/skein-lib/src/journal.rs:26`) has no such
     field; the only `transcript_bytes` is the hub's
     (`crates/jig-domain-host/src/limits.rs:22`), and a turn reaches the
     core over the worker channel as `Turn.body: bytes 8388608`
     (`crates/jig-protocol-worker-wire/schema/worker-wire-v1.schema:37`).
     So the store admits what the channel admits: the core family's
     transcript field has the ceiling 8,388,608, and
     `codec::core::limits` sets its limit to that ceiling, no
     `core::Limits` field bounding it (jig 14's rule: a field no domain
     limit bounds). No signature changes, `codec::limits` and 19.5's
     sketch included; 19.2 corrects the stale comment at `stored.rs:531`.
181. **A story finds its tasks, not their numbers** (2026-10-13, the
     coordinator's block "jig 07.s1: seeded-crash fixture hardcodes
     shifted task identities", 2026-10-13T00:10:48Z). jig 07.s1's period
     and policy commits, as its Shape asks, shift task numbers in
     `jig-ops-world`'s `p_story_survives_seeded_commit_crashes_and_random_presses`
     (seed 537): incident 12's lead becomes task 6 and its remediation 7,
     where the story hardcodes 5 and 6. 07.s1 changes the story to find
     incident 12's lead and its remediation delegate and use them for
     addresses and record matching, past ruling 114's 20 lines, keeping
     every behavioural assertion (one restart, one acceptance, its person,
     by, choice and kind, the procedure's phases, the incident's
     resolution) and the seed.
182. **skein 02.s7: the piece helpers; jig repins once for both** (2026-10-13,
     the designer's answer to question.md's "jig 30.2: a page whose rows
     nest chips past a u32"; skein `58cc905a`, jig `3c5f607c`,
     `d8e4b2c8`). A collection its owner bounds by a sum crosses as one
     piece, `bytes N` with each item after a `u32` length: a page's rows
     and ruling 151's nine flattened tables, bounded by the client's
     `document_bytes`, and jig 22.3's window. A step not in the plan,
     skein 02.s7 (`skein/02-startup-and-secrets.md`), adds the helpers;
     jig-repin.s4 waits for both 02.s6 and 02.s7 and pins the later, so
     jig repins once; jig 30.2 lands its other reads and pages meanwhile,
     and its pieces after jig-repin.s4.
183. **jig 05.s1: the hub's vocabulary names a relayed call by its bytes**
     (2026-10-13, the designer's answer to question.md's "jig 23.2: how a
     relayed call's delivery token meets the wire's call name"; jig
     `fadbde24`, `7ea05d7f`). channel.md, 6.3 already had the hub match a
     relayed answer by the call's bytes; the landed vocabulary
     (`crates/jig-domain-host/src/link.rs:10`, `Relayed { .., call: Token,
     .. }`) and `ops`'s `host_calls` table depart from it. A step not in
     the plan, jig 05.s1 (`jig/05-hub-link.md`), moves the vocabulary to
     the call's bytes both ways; jig 23.2 and 23.3 translate names both
     ways, 23.2 landing its other records first; jig 24.3's relay table
     holds no token.
184. **A document's pieces are bounded by the envelope that carries it:
     16 MiB** (2026-10-13, the coordinator's block "jig 30.2: invalid
     Limits literal versus document piece ceiling", 2026-10-13T00:28:23Z).
     Ruling 149 takes "at least the largest value any `Limits` literal on
     `main` gives" a field; a literal a test builds to be refused
     (`crates/jig-domain-client/src/tests/time.rs:270`, `document_bytes:
     u64::MAX`, asserted `worst_case(&invalid).is_none()` at :278) does not
     count. The valid literals give 65,536, a floor. A document crosses
     inside `Owned.record: bytes 16777216` (30.1's sketch), so no piece of
     it can be larger: D, the ceiling of every piece ruling 182 makes, is
     16,777,216, within a `u32`, and jig 20's profiles must fit it.
185. **jig 05.s1 also drops `reply_to` from the link vocabulary**
     (2026-10-13, the designer's answer to question.md's "jig 23.2: an
     answer's `reply_to` on the engine side"; jig `e49ace7c`, `8fceddd1`).
     channel.md, 3: "**The link vocabulary names only what crosses** ... A
     handle one side mints for itself (a delivery's token, a request's
     reply handle) is not in it". jig 05.s1, still running when the answer
     came, takes it: `LinkDown::Assign` and `LinkUp::Answer` lose
     `reply_to`, the hub answers and refuses an assignment by its run and
     attempt; its Size is now 250. jig 23.2 translates `Answer` and
     `Assign` once 05.s1 lands; jig 24's link makes no `ReplyTo`.
186. **smith-repin waits for the user on smith's settle test** (2026-10-13,
     the coordinator's block "smith repin: full-suite settling timeout
     after three attempts", 2026-10-13T01:07:11Z). On skein `4130d9b`
     (02.s6 and 02.s7 in), smith's
     `smith_check_json_lists_the_effective_limits_by_name` passes alone in
     1,558 ms, but fails its 2,000 ms settle bound in the full suite at
     2.049, 2.083 and 2.077 s; at smith's old pins it took 1,010 ms alone.
     02.s6 took back about 490 ms of skein 02.s4's cost; the other 548 ms
     come from skein commits since smith's pin that are not named yet,
     some of them other machines' (`adopt_write_pipe`'s new argument
     among them). smith-repin keeps its commit `bb26b6d9`; the closing jig
     and temper repins wait for it; every other step goes on. To the user,
     under "For the user".
187. **jig 05.s2: `ops`'s root keeps its own hub's turn busy** (2026-10-13,
     the designer's answer to question.md's "jig 23.3: where the engine's
     'turn busy' comes from on its way down"; jig `e484056a`, `df25be1b`).
     Turn busy is "the fleet's word to what keeps the backoff below the
     hub, never the hub's": jig 23.3's component takes it as
     `Event::TurnBusy` and encodes the wire's record; jig 24's link keeps
     the backoff on a worker. For the engine's own hub the root keeps it;
     `ops`'s root today passes `core::Held::TurnBusy` out as an output
     (`examples/ops/domain/src/route.rs:350`). A step not in the plan, jig
     05.s2 (`jig/05-hub-link.md`), gives `ops`'s root that backoff.
188. **jig 20.5 lands at 1,170 lines; jig 20 joins the "up to twice" rule**
     (2026-10-13, the coordinator's block "jig 20.5: production size 1170
     versus 660 margin", 2026-10-13T01:20:12Z; the third such block in jig
     20, after rulings 161 and 164). The strict domain vocabulary 20.5's
     Shape asks for (projects' authority, rules, policy, roles, grants,
     patterns, requirements; providers, owners, connectors; charters)
     costs 1,170 net lines against 550, every file under 800. Its Size
     becomes 1,200. From now, jig 20's later increments land without
     asking up to twice their Size, as the README's rule for codec
     increments says, now naming jig 20.
189. **jig 15.1 defines the store component's `Limits`** (2026-10-13, the
     coordinator's block "jig 15.1: component Limits sketch has no landed
     owner", 2026-10-13T01:37:56Z). A plan defect: 15.1's sketch has
     `Store::over(limits: &Limits, ..)` with `Limits` labelled "jig 13's",
     but `jig-protocol-store` on `main` has no component `Limits`; 19.5
     has `codec::Limits` and `sizes::check(core, application, journal,
     kv)`. As proposed: 15.1's `lib.rs` defines `Limits { core:
     jig_domain_core::Limits, store: skein_kv::Limits, secrets:
     skein_kv::Limits, journal: skein_lib::JournalLimits }` (store.md, 13),
     and `Store::over` derives `codec::limits` once from it (ruling 179:
     limits checked where they are made), keeping settings and state
     private. The files', queue's and reads' bounds join it in jig 17,
     which owns them.
190. **jig 08.s1: an account's next ask crosses as a wall** (2026-10-13, the
     designer's answer to question.md's "jig 30.4: an account's 'later'
     deadline in a document that carries walls only"; jig `83c781c9`,
     `78f2b285`). `OwnerAccountState::Later { until: Time }` alone in the
     owner's deployment document is on the engine's clock; client.md, 3
     now has such a deadline cross "as the wall it falls at". A step not
     in the plan, jig 08.s1, changes the contract and its producers; jig
     30.4 carries the wall after it, and lands its other documents
     meanwhile.
191. **The next ask is an `Option<Wall>`, as the document's other
     deadlines** (2026-10-13, the coordinator's block "jig 08.s1:
     clock-free core deployment document cannot project account
     deadline", 2026-10-13T01:51:25Z). The core makes the owner's
     deployment document with a clock (`make_with_clock`) or without one
     (`make`); without, no wall can be made. The document's other
     deadlines already say so: `spent_until`, `grace_until` and
     `Adrift.until` are `Option<Wall>`
     (`crates/jig-domain-client/src/contract/owner_deployment.rs:69, 84,
     97, 188`). So `Later { until: Option<Wall> }`: `Some` from
     `wall_deadline` when the document is made with a clock, `None`
     without one; the view says "asks again later" for `None`. The core's
     document gains `documents::AccountState`, mirroring the accounts
     child's state with that field. This narrows the designer's `until:
     Wall` (ruling 190) to the clock-free case only.
192. **jig 05.s3: a start the agent refuses fails as invalid** (2026-10-13,
     the coordinator's block "jig 25.1: missing agent start-refusal and
     invalid vocabulary", 2026-10-13T02:30:54Z). A plan defect: hosts.md,
     8 has "A start smith refuses all the same fails its attempt as
     invalid, naming why (6.3)", but jig 05's hub (`FromAgent`,
     `Invalid` at `crates/jig-domain-host/src/boundary.rs:243, 579`) has
     no such refusal, so jig 25.1 cannot translate smith's
     `RunResult::Refused`. A step not in the plan, jig 05.s3
     (`jig/05-hub-link.md`), adds `FromAgent::Refused` and
     `Invalid::Start`; jig 25.1 lands after it, its scaffolding before.
193. **jig 05.s3's names and wire** (2026-10-13, the coordinator's block
     "jig 05.s3: Refused name collision and wire InvalidStart
     counterparts", 2026-10-13T02:35:40Z). The supervisor's step named
     `FromAgent::Refused`, which a message's refusal already is
     (`crates/jig-domain-host/src/boundary.rs:284`): the start's is
     `StartRefused`, the old name unchanged. The answer takes the hub's
     existing `Ending::Refused(Refusal::Invalid(..))` path, the hub's
     `Answer` being a struct. `Invalid::Start { why }` crosses the worker
     wire with its words, so 05.s3's Files gain the worker wire's schema
     and regenerated code (its version moved once) and jig 23's and 24's
     translations of it both ways; Size 140.
194. **The client wire's translations land at their count** (2026-10-13,
     the coordinator's block "jig 32.1: production size 2961 versus 720
     margin", 2026-10-13T02:45:49Z). jig 32.1's exhaustive translation of
     the client contract's asks, outcomes, refusals and the recursive
     facts they carry (ruling 151's 359 declarations) is 2,961 net lines
     against "at most 600", every file under 800. The plan priced the
     client wire's translations before the recursive facts were known;
     jig 31, 34 and 35 translate the same vocabulary. So from now to the
     run's end, an increment of jig 31, 32, 34 or 35 whose production code
     is only translations between the client contract's types and the
     wire's lands at its count without asking, giving the count by file in
     its report; each file stays under 800 lines and each test function
     under 40; anything else in it keeps its Size. jig 32.1 lands at 2,961.
     The README's Rules say so.
195. **The fixtures a Test shape names are inside Files** (2026-10-13, the
     coordinator's block "jig 16.2: required fixtures omitted from Files",
     2026-10-13T02:48:57Z). 16.2's Test shape names eleven fixtures
     (`person_proposal`, `proposal_decision`, ..., `with_proposal_rows`)
     but its Files list only `reads/point.rs` and `src/tests/reads.rs`;
     code-organization.md, 5 keeps one shared fixture module per crate,
     whose root is 794 lines. From now to the run's end, the fixtures an
     increment's Test shape names belong in its Files, in the crate's
     shared fixture module or a child of it (a new child where the root
     would pass 800, ruling 116), with the root's re-exports; it is no
     question. The README's Rules say so.
196. **An increment lands up to 1.5 times its Size without asking**
     (2026-10-13, the coordinator's block "jig 17.1: production size 747
     versus 660 margin", 2026-10-13T02:51:23Z; the fourth plain size block
     since 01:20Z, after jig 20.5, 24.2 and 32.1, each landed as it was).
     jig 17.1 lands at 747 against 550. From now to the run's end, ruling
     103's margin is 1.5: an increment within N × 1.5 of its "Size: at most
     N lines" lands and gives its count by file; past that it asks. The
     codec and configuration rule (up to 2 ×, rulings 132, 151, 188) and the
     client translations' (at their count, ruling 194) stay. The 800-line
     file limit and the 40-line test function stay rules. code-organization.md,
     section 7 says so.
197. **An item made visible within its crate is no question** (2026-10-13,
     the coordinator's block "jig 35.1: reuse private recursive fact
     helpers across session Files", 2026-10-13T02:57:16Z). jig 35.1 needs
     jig 32.1's `facts.rs` `Node`, `decode` and its non-recursive helpers,
     private to that module; the alternative, translating the recursive
     facts again, is forbidden. From now to the run's end, widening an
     item's visibility within its own crate (`pub(crate)`, `pub(super)`, a
     re-export from the module's root) so that a sibling module reuses it
     is an implementation choice in any session's files, named in the
     commit body, when it changes no behaviour, signature or test. Public
     API across crates stays a question. The README's Rules say so.
198. **A session may replace what its own earlier increments landed**
     (2026-10-13, the coordinator's block "jig 17.2: replace old directory
     commit path outside Files", 2026-10-13T03:03:02Z). 17.2's common
     transcript writer replaces 17.1's `directory::commit` (45 lines in
     `directory.rs`, jig 17's own file, not in 17.2's Files); development.md,
     8.2: "Removing a mechanism removes all of it". From now to the run's
     end, an increment that replaces a mechanism its own session's earlier
     increments landed edits and removes it in those files, past ruling
     114's 20 lines, keeping every test's name and expected value unless
     its Shape changes the behaviour, named in its commit body. Another
     session's files stay under rulings 114 and 145. The README's Rules
     say so.
199. **smith's `Directory.git` field is allowed where jig builds one**
     (2026-10-13, the coordinator's block "jig 25.1: foreign Smith
     Directory git field rejected by vocabulary guard",
     2026-10-13T03:05:17Z). jig's vocabulary guard
     (`tests/boundary/src/tests/vocabulary.rs`, `names.rs`'
     `vocabulary_matches`) refuses the word `git` in jig's sources; smith's
     `smith::Directory` has a field `git: bool`, and smith's `valid_start`
     wants a `Directory` whenever a start has a workspace, so jig 25.1's
     workspace fixture (and later the test service's construction) must
     write `git: false`. The guard keeps jig's own vocabulary, not the
     field names of smith's types. 25.1 adds to the guard an allowance for
     exactly the field `git` of a `smith::Directory` literal, in the files
     that build one, each named; every other check of the word stays.
     Not a dropped check: a foreign name excepted, as jig's own names are
     not.
200. **jig 25.2's link keeps payloads in the root and hands assignments on
     typed** (2026-10-13, the coordinator's block "jig 25.2: typed
     assignment and payload-retention translation sketch",
     2026-10-13T03:22:07Z). A plan defect: 25.2's sketch had a pure
     `link::up(channel, up) -> core::Event`, which cannot keep a turn's or
     an answer's payload under the tokens the fleet's events carry, and a
     pure `link::down(fleet::Request) -> Option<(Token, host::LinkDown)>`,
     which cannot carry an assignment's typed `RunCharter` for the routing
     to encode (the README: no encoding in the root). As proposed:
     `link::up` takes `&mut Domain` and keeps the payloads before making
     the fleet's event; an assignment stays the existing typed
     `Delivery::Assigned`, which the service's routing encodes;
     `Delivery::Link` carries only the hub's records already typed. The
     test `an_assignment_for_a_channel_leaves_as_the_hub_s_record` checks
     the typed hand-off and the kept `RunCharter`. No payload encoding in
     the root.
201. **`jig-protocol-transcript` gains `Limits::new` for the client link**
     (2026-10-13, the coordinator's block "jig 35.2: client transcript
     Limits construction without Smith dependency", 2026-10-13T03:27:33Z).
     35.2 decodes an activity's turn bytes with
     `jig_protocol_transcript::decode`, whose `Limits { smith, bytes, turn
     }` the client link cannot build: only the transcript crate links
     smith (`tests/boundary`'s `client_transcript_is_the_client_only_smith_link`).
     35.2 adds `pub fn Limits::new(bytes: u32, turn: contract::TurnLimits)
     -> Limits` to the transcript crate, its smith part smith's
     `CEILINGS` (a client decodes whatever smith may have written), at
     most 8 lines, no field, behaviour or landed test changed, named in
     its commit body. The client link builds it from its own limits.
202. **jig 03.s1: a turn that does not decode, in the client's contract**
     (2026-10-13, the designer's answer to question.md's "jig 35.2: how a
     turn that does not decode crosses in the client contract"; jig
     `13153661`, `941018ab`). The contract gains `TurnRead` and
     `TurnProblem`, which `Activity::Turn` and `ArchivedTurn` carry. A step
     not in the plan, jig 03.s1 (`jig/03-*.md`), changes the contract, its
     users and the view's card first; jig 35.2 carries `TurnRead` after it,
     and lands its other pages and documents meanwhile.
203. **jig 25.3 parses the whole engine document and activates only what it
     builds** (2026-10-13, the coordinator's block "jig 25.3: startup
     component availability versus mandatory engine configuration",
     2026-10-13T03:45:51Z). A plan defect of staging: 25.3 refuses at
     startup the workers and the components 25.4 and 25.5 bring, but jig
     20's `engine` parser requires `clients`, `llm`, `charters` and
     `chats`, so no document would start 25.3's service. 25.3 parses the
     document whole with jig 20's parser; the service's own part declares
     which components it activates; components 25.4 and 25.5 bring
     (accounts, the workers' listener, the clients' listener) stay parsed
     and inactive, and an activation of one not yet built is refused,
     naming its path. jig 20's parser is unchanged.
204. **The test root's bounds become its own public API** (2026-10-13, the
     coordinator's block "jig 25.3: root output and memory bounds API
     required by service startup", 2026-10-13T03:45:59Z). 25.3's service
     must size the root from jig 20's profiles, but `jig-test-domain` has no
     `max_out` or `worst_case`; `tests/core/src/observations.rs`'
     `root_bound` computes them. 25.3 moves that computation into
     `jig-test-domain` as public `max_out(&Limits)` and `worst_case(&Limits)`
     (a module of its own), the core world calling them in place of its
     copy; the profiles gain the two connectors' limits and the journal's
     and routes' (README, "A component's limits join the profiles").
     Inside 25.3's Files, named in its commit body.
205. **The test application's connector limits are its own, by profile**
     (2026-10-13, the coordinator's block "jig 25.3: application connector
     profile limits versus protocol dependency boundary",
     2026-10-13T03:49:04Z). Ruling 204's "the profiles gain the two
     connectors' limits" would make `jig-protocol-config` depend on
     `jig-test-connector`, which code-organization.md, 3 forbids.
     configuration.md, 6: "The application's limits are its own members,
     checked in the same sum." So the test service's `limits.rs` holds the
     two connectors' limits for each of jig's profiles, chosen by the
     profile's name, and checks them in the same sum; jig's profiles gain
     only jig's own (the journal's, the routes', io's). This narrows
     ruling 204.
206. **The test root answers busy and always takes a store's answer**
     (2026-10-13, the coordinator's block "jig 25.3: root admission and
     store replies with a full journal", 2026-10-13T03:59:06Z). A defect
     against the design: service.md, 2 has the root answer "busy, through
     the door, an event the journal cannot take", and 4 "A store's answer
     is always taken, however full the journal"; `jig-test-domain`'s
     `decide` returns when `domain.journal.decision(&room)` is `None`,
     dropping the input. 25.3 adds a public `try_step(..) -> Result<(),
     Event>` to `jig-test-domain` that hands back an input the journal
     cannot take, for the service to answer busy; a store's answers
     (loads, a commit's acknowledgement or failure) are taken without a
     decision's room. `step` stays for the worlds, unchanged. Inside
     25.3's Files, named in its commit body.
207. **jig 11.s1: a first start seeds the configured projects** (2026-10-13,
     the coordinator's block "jig 25.3: first-start configured project
     roles and policy seeding", 2026-10-13T04:10:11Z). A defect against
     configuration.md, 3 ("seeds projects, their policies and their
     parties' roles on a deployment's first start only"): `Core::new`
     makes the people child with no roles rows, so after an empty load the
     configured owner's sign-in is refused, and no initial policy is
     saved; the core worlds bootstrap a store by hand. A step not in the
     plan, jig 11.s1 (`jig/11-*.md`), seeds them in one commit at a first
     start; jig 25.3 lands after it.
208. **jig 30.s1: the client wire discriminates a read's value**
     (2026-10-13, the coordinator's block "jig 32.2: read-value wire
     document discriminator missing", 2026-10-13T04:11:41Z). A plan
     defect: the client wire has `ReadAnswer::Value(Owned)` with raw bytes
     and no `ReadValue`, though `contract::ReadValue`'s 26 variants overlap
     on a read and jig 34.3 and 34.4 return `v1::ReadValue`. A step not in
     the plan, jig 30.s1 (`jig/30-client-wire.md`), adds `enum ReadValue`;
     jig's read answers carry it encoded as `Owned.record`. jig 32.2 and 34
     use it; 32.2 lands its outgoing reads meanwhile.
209. **Moving an unchanged test body into a named helper counts for
     nothing** (2026-10-13, the coordinator's block "jig 11.s1: first-owner
     seed fixture fixes and required named test bodies",
     2026-10-13T04:33:47Z). 11.s1's pre-seeded owners reach eleven client
     tests (ruling 145): its fixes are 12 lines of inputs and one expected
     person (2 to 1, ruling 122), but keeping two changed tests within the
     40-line ceiling moves their unchanged assertions into named bodies,
     and the core world's clock fixture changes 3 lines, past ruling 114's
     20 in all. From now to the run's end, moving a test's unchanged
     statements into a named helper of the same file, to keep a function
     within 40 lines, does not count toward ruling 114's 20 lines; the
     inputs and expected values changed still do. 11.s1 lands with its
     edits, each named in its commit body. The README's Rules say so.
210. **The people child forgets its configured owners at a restart**
     (2026-10-13, the coordinator's block "jig 11.s1: forget configured
     initial owners after restart begins", 2026-10-13T04:37:42Z).
     configuration.md, 3: `owners` are "the deployment's first owners ...
     on a first start", and later starts ignore the configuration's seeds;
     but the core hands its configured owners to `people::Domain::new` on
     every start, so a restart after the configuration names a new owner
     grants it (11.s1's regression
     `a_restart_does_not_grant_new_configured_owners`). 11.s1 adds a public
     `people::Domain::forget_initial_owners(&mut self)`, which drops only
     its bounded list of first owners, and the core's `restart_begin`
     calls it on a restart; a first start seeds from them as 11.s1 does;
     `step`'s worlds are unchanged. About 6 lines.
211. **jig 30.s2: run text tagged with its watch; a delivery discriminated**
     (2026-10-13, the coordinator's blocks "jig 32.3: run-text stream event
     lacks destination watch" and "jig 32.3: stream delivery wire document
     discriminator missing", 2026-10-13T04:37:42Z and 04:38:15Z). Two plan
     defects in jig 30's schema: client.md, 1 tags each stream event with
     its watch, but `StreamEvent::text` is a bare `RunText`; and a
     delivery's document has no discriminator among `contract::Delivery`'s
     16 variants, as reads had none before ruling 208. A step not in the
     plan, jig 30.s2 (`jig/30-client-wire.md`), adds `StreamText` and
     `DeliveryValue`; jig 32.3 lands its other stream work meanwhile.
212. **jig 25.3 lands at 1,293 lines** (2026-10-13, the coordinator's block
     "jig 25.3: production count1293 over900 allowance",
     2026-10-13T04:41:42Z). Against "at most 600" (900 with ruling 196):
     the service's parser and routing over every engine record, and scope
     the supervisor's rulings added (203's staged activation, 204's root
     bounds, 206's root admission), every file under 800. Its Size becomes
     1,300; it lands after jig 11.s1 and its proof.
213. **jig 32.s1: the client's version follows its wire's, checked**
     (2026-10-13, the coordinator's block "jig 32.3: client domain
     version1 rejects current wire version4", 2026-10-13T05:08:00Z).
     `contract::CLIENT_VERSION` stayed 1 while jig 30's steps moved the
     client wire to version 4. A step not in the plan, jig 32.s1
     (`jig/32-client-link.md`), moves the constant and adds a test in the
     client link that the two are equal, so the next move of one alone
     fails the gate.
214. **jig 18.1 lands at 1,102 lines; jig 18 joins the codec rule**
     (2026-10-13, the coordinator's block "jig 18.1: complete codec
     count1102 over750 allowance", 2026-10-13T05:12:51Z). jig 18.1's store
     codec for the test application's connector records carries, in its
     twelve records' `ApplicationOutcome::CoreRefused`, the core's retained
     facts, whose exhaustive translations are 463 of its 1,102 lines
     against 500; every file under 800. It lands as it is, and jig 18
     joins the README's rule for codec increments (up to twice their Size
     without asking).
215. **The map's sizes leave out what goes to the transcripts' files**
     (2026-10-13, the coordinator's block "jig 18.2: file transcript body
     included in impossible map commit sizing", 2026-10-13T05:23:05Z). A
     plan defect in jig 19.5's `sizes`: `sizes::largest` takes the core
     family's worst row, a turn with its 8,388,608-byte transcript (ruling
     180), as a map value and multiplies it by a commit's writes, so with
     the world's `journal.writes` (at least 2,000) no configuration passes
     skein-kv's `u32` commit. store.md: "**Transcripts are files, one per
     task,** each turn a frame written" there, the map holding their
     places. So `sizes` sizes the map and a commit by the rows the map
     stores, a turn's body bounded by the file's write instead; the 8 MiB
     decoder ceiling stays. 18.2 makes the change in `sizes.rs` and its
     format tests, whose expected values for the map's largest value
     follow (ruling 122), named in its commit body; the world's journal
     bound is not capped.
216. **The vocabulary guard excepts a foreign name exactly where jig must
     write it** (2026-10-13, the coordinator's block "jig 31.1: native
     core IpAddr V4/V6 variants rejected as version names",
     2026-10-13T05:26:56Z; the second such block, after ruling 199's
     `smith::Directory.git`). `tests/boundary/src/names.rs` takes every
     `V` and a digit for a version name, and so refuses
     `core::net::IpAddr::V4` and `::V6`, which an exhaustive match on a
     standard address must name. From now to the run's end, the guard
     allows a name another crate (smith, skein, the standard library)
     defines, written fully qualified where jig's code must use it, each
     allowance naming the exact path; jig's own names stay refused. 31.1
     adds the two `IpAddr` allowances and a regression that an unqualified
     `V4` of jig's is still refused, under ruling 114. The README's Rules
     say so.
217. **The small store's keepers each name the version they read; jig's
     records are discriminated** (2026-10-13, the coordinator's block "jig
     32.4: keeper version admission and stored jig record discrimination",
     2026-10-13T05:29:39Z). A plan defect: 32.4's Scope has the link
     decode each kept record at start, hand it to its owner, and drop and
     tell the owner of one "of a version its owner does not read"
     (client.md, 9), but its sketch's `Request::Dropped { keeper:
     Option<u16> }` merges jig and the request table, and the link knows no
     table's or family's version. As proposed: `Start` carries the one
     store version each keeper reads (jig's `STORE_VERSION`, the table's,
     each family's), from the routing; `Dropped` names a typed `Keeper {
     Jig, Table, Family(u16) }`; a keeper absent from that set has its
     records dropped and is told. And jig's own kept record is a
     discriminated schema type, `enum JigRecord { place: Place, draft:
     Draft }` in the client wire (its version moved once), as `ReadValue`
     and `DeliveryValue` are (rulings 208, 211), not a private byte; it
     joins 32.4's Files.
218. **`Start` hands the link each kept record with its storage key**
     (2026-10-13, the coordinator's block "jig 32.4: opaque family
     restoration loses its stored key at Start", 2026-10-13T05:34:04Z).
     32.4's sketched `Saved::Family { family, version, key, bytes }` needs
     the key, but `Start` read only the records' bytes, and a family's
     record is its own opaque bytes. As proposed: `Start { versions:
     List<StoreVersion>, kept: List<StoreRead> }`, `StoreRead { key:
     Box<[u8]>, record: Box<[u8]> }` as the shell reads them; the family's
     key comes from the keeper-tagged storage key; `w::Kept` and the
     family's bytes are untouched.
219. **The focused suite's second pass: two steps and a rule for the
     rest** (2026-10-13, the designer's answer to question.md's "The
     focused suite's cost, a second pass"; skein `cb3cefbf`, jig
     `404ebd94`). Most of the suite's time is processes, each test about 5
     ms alone and 12 ms in the gate's eight at once. Two steps not in the
     plan (`tests-cost.md`, 10 and 11): jig-tests.s4, the repository's
     tools (`jig-boundary`, `skein-codegen`) built optimized, about 8 s of
     test time; jig-tests.s5, one behaviour's cases folded into a table
     test in six unit-test crates, about 7 s. And `tests-cost.md`, 12 binds
     every step left (31 to 35, 18, 25 to 27, 33): one test per behaviour,
     cases as rows, fixtures constants or built once, test-sized limits,
     the Proof naming its tests' count; jig 34 and 35 share each document's
     wire record from one kit under `testing/`. The README's Rules say so.
     Unit tests in one process per crate is the user's to choose (For the
     user).
220. **jig 25.4 gives the test root the brief an assignment carries**
     (2026-10-13, the coordinator's block "jig 25.4: test root drops the
     charter brief sections", 2026-10-13T05:55:39Z). A plan defect:
     `domain/root.md`, 7 puts "its core sections" in an assignment and
     `domain/hosts.md`, 8 renders "the brief's sections ... as smith's
     titled sections, in order", but `jig-test-domain` discards
     `BriefCorePlanned`'s sections, leaves `CompleteBrief` unreachable and
     keeps no brief in its `Assignment`, so 25.4's service cannot encode
     the charter. 25.4's Files and Scope gain the test root's brief
     assembly: it keeps the planned core sections (bounded and priced in
     its worst case), completes the brief, and carries the sections typed
     in the assignment, with one regression; its Size grows by 200.
221. **jig 18.2's test drops a forbidden word from its name** (2026-10-13,
     the coordinator's block "jig 18.2: prescribed new test name contains
     forbidden Typed", 2026-10-13T05:57:52Z). The plan named the test
     `a_connectors_step_restores_its_records_typed`; jig's vocabulary guard
     refuses "typed" in jig's own names (rulings 199, 216 keep it so). It is
     `a_connectors_step_restores_its_records`, body, fixture and expected
     equality unchanged; 18.2's file says so.
222. **The shell hands the accounts component its runtime resources at
     startup** (2026-10-13, the coordinator's block "jig 25.4: signed-in
     account startup lacks private keeper and transport resources",
     2026-10-13T05:59:50Z). A plan defect: an active accounts component
     builds its sign-in sources over skein's private keeper and transport,
     but 25.4's `startup(document, Files, Agent, Seeds)` brings only the
     files' bytes. As proposed: `startup` gains an optional
     `AccountsRuntime` (the private keeper's root descriptor and effective
     user, the resolved issuer addresses, the prepared TLS configuration),
     which the binary's shell fills before the listeners and the restart,
     and the worlds fill with their fakes; absent, an activation of signed-in
     sources is refused naming its path (ruling 203). No other public field
     or signature changes.
223. **`jig-protocol-charter` exposes its encoding limits** (2026-10-13,
     the coordinator's block "jig 25.4: charter encoder limits unavailable
     through allowed dependencies", 2026-10-13T06:01:44Z). As ruling 201
     for the transcript crate: `jig_protocol_charter::encode` takes
     `smith_charter::v2::Limits`, which the service, linking only smith's
     host and channel crates, cannot build. 25.4 adds to the charter kit a
     public `limits() -> smith_charter::v2::Limits` returning the
     ceilings it already encodes within (re-exported as the kit's own
     type), at most 10 lines, no encoding behaviour changed, named in its
     commit body; no smith dependency is added to the service.
224. **jig 03.s2: texts UTF-8, cut at a character's boundary** (2026-10-13,
     the designer's answer to question.md's "jig 34.1: a document's text,
     bytes or UTF-8"; jig `1bc30fd0`, `e2f4ba51`). The client wire's
     `DocumentText.bytes` is `text` (UTF-8) while the core cut texts at a
     byte length and the client's `Text` took any bytes; the designer
     makes texts UTF-8 end to end. A step not in the plan, jig 03.s2
     (`jig/03-people-and-tasks.md`), changes the core's cut, the client's
     `Text`, the people child's entrance and one fixture; jig 34.1 lands
     after it, its fixtures UTF-8 meanwhile.
225. **A value the Shape makes invalid is replaced wherever a test builds
     it** (2026-10-13, the coordinator's block "jig 03.s2: UTF-8 fixture
     migration exceeds shared-test edit caps", 2026-10-13T06:17:07Z).
     03.s2's Shape makes the client's `Text::new` refuse bytes that are
     not UTF-8; about 18 client test files build texts from such bytes
     (`conversation_merge.rs` 19 inputs and 5 expected literals that echo
     them, beyond rulings 114 and 122). Those fixtures are 03.s2's: each
     refused value becomes a UTF-8 one in the same role, an expected value
     that echoes it changes with it, names and every other expected value
     kept, past rulings 114's and 122's limits. A test whose subject is the
     opaque bytes themselves goes under ruling 123. The README's Rules now
     say so for any increment whose Shape makes a value invalid.
226. **jig 31.3 lands with no service credential, the credential's shape
     asked of the designer** (2026-10-13, the coordinator's block "jig
     31.3: service credential design absent from contract and wire",
     2026-10-13T06:28:00Z). `protocol/client.md`, section 8 and decision
     36 give the owner a service's credential once, in `Made::Service`;
     the contract's `Made::Service { party }` and the wire's `MadeService
     { party }` have no field, and no step adds one. 31.3 takes its session
     file's fallback, widening no contract, wire or keeper; the designer is
     asked its shape, its digest's keeper, its check and its step, which
     is added when answered.
227. **A fixture gains what the real component requires, and an expected
     value that sees exactly that gain changes with it** (2026-10-13, the
     coordinator's block "jig 18.3: real-store deployment header
     contradicts old whole-map emptiness assertions",
     2026-10-13T06:30:51Z). The real store requires the first root
     commit's deployment header, which the fake did not;
     `tests/core/tests/store_peer.rs`'s
     `whole_opaque_rows_remain_immutable_after_later_save_erase_and_store_mutation`
     then cannot see an empty map after erasing its turn. Its fixture
     commits the real header first, and its two `later[1].rows.is_empty()`
     become exact equalities with the header's rows, built from the
     fixture's own constant; name and every other assertion kept; the
     other two raw-store tests take headers with no assertion changed.
     18.3's "Don't: change an existing test's expected values to fit the
     real store" stands for anything else. The README's Rules now say so
     for any world moving to a real component.
228. **jig 31.s1: a service's credential** (2026-10-13, the designer's
     answer to question.md's "jig 31.3: a service's credential, its shape
     and its step"; jig `2dfb78a5`, `ee79fa5f`). The credential is a
     sign-in secret's shape under the service's number, its digest in the
     secrets' store, checked once at the deployment provider's sign-in
     route, never on an exchange; the contract's `Made::Service { party }`
     stays (decision 36 settled) and the wire's `MadeService` gains
     `credential: secret 40`. A step not in the plan, jig 31.s1
     (`jig/31-engine-clients.md`), builds it after 31.3.
229. **The peers' fixture journal is sized by the root's admitted bound**
     (2026-10-13, the coordinator's block "jig 18.3: peer fixture journal
     overflows the actual real-store ceiling", 2026-10-13T06:33:20Z).
     `tests/core/src/peers.rs`'s fixture asks a journal of 10,000 writes;
     the real store's `sizes` refuses it (the largest map row times twice
     the writes past `u32`), as it should refuse a configuration no commit
     can hold. The fixture's `journal.writes` becomes the root's admitted
     bound, `core::room_max(&limits.core).writes + 2 * connector::MAX_OUT`,
     at least 2,000, and `journal.held` follows it: a fixture input under
     ruling 114 and the README's test-sized limits (ruling 219), every
     test's name and expected value kept. Ruling 215 stands: the adapter
     caps nothing. A test that then fails for want of journal room is a
     finding, raised.
230. **An expectation the fake made against the design follows the
     design** (2026-10-13, the coordinator's block "jig 18.3: fake-store
     subtree history assertions contradict documented member order",
     2026-10-13T06:39:38Z). `tests/core/tests/reads.rs`'s
     `a_subtree_history_reads_each_delegates_rows_found_by_index` expects
     the fake's global sort of (kind, task, position); `protocol/store.md`,
     7.3 reads a subtree "task by task: the task's own rows, then each
     delegate's in that index's order, each task's rows in its own
     history's order", which the real store answers. Its two ordering
     assertions become exact equalities with the designed order (the
     goal's rows, then the delegate's, each in its own order), name and
     every other assertion kept, and its direct edit of the fake's mirror
     becomes a real erase through the fixture, its `is_some` kept. This
     is not fitting a test to the real store, which 18.3's Don't forbids:
     the design, not the component, gives the expected values. The
     README's Rules now say so.
231. **The peers' fixture and those built on it are test-sized until the
     real store admits them** (2026-10-13, the coordinator's block "jig
     18.3: admitted peer journal still exceeds commit ceiling and derived
     fixtures change bounds", 2026-10-13T06:48:04Z). Under ruling 229 the
     peers' fixture asks 5,515 writes, and its people family's largest
     row (1,406,719 bytes, a codec worst case of its people bounds) times
     twice the writes is past `u32`; fixtures built on it then grow the
     core's bounds after its journal was derived. The peers' fixture's
     bounds may shrink to test-sized values (`people.amendment_bytes` 256,
     and others as needed) that every test using it still exercises, and
     each fixture that changes core bounds after building on it derives
     its journal again through one named helper of `peers.rs` after its
     last change, at least 2,000 writes. Every test's name and expected
     value kept; the adapter caps nothing (ruling 215). Ruling 229's
     finding clause is about a test that runs out of journal room, not
     about admitting a fixture. A test whose case needs a bound the real
     store cannot admit is a finding, raised.
232. **A cursor is opaque bytes, not a text** (2026-10-13, the
     coordinator's block "jig 03.s2: opaque cursor uses Text that now
     requires UTF-8", 2026-10-13T06:50:26Z). The contract's `Cursor {
     value: Text }` (`contract/reads.rs:12`) borrowed `Text` for bytes the
     design calls "the engine's, opaque bytes the client gives back"
     (`protocol/client.md`, 6) and the wire carries as `record Cursor {
     value: bytes 1024 }`; with 03.s2's `Text::new` refusing non-UTF-8, the
     joint world's cursor encoder fails. 03.s2 gives `Cursor` its own
     bounded bytes (a constructor checking the cursor's bound, as `Text`
     checks its length), its users moved in any file under ruling 225,
     no schema change and no encoding of the bytes as text; its Size
     rises from 60 to 140 lines. Any other contract field the design
     calls bytes and the wire carries as `bytes` gets the same; a field
     the design leaves unclear is asked.
233. **jig 31.s2: a refusal's ending travels as a fact** (2026-10-13, the
     coordinator's block "jig 34.1: landed 31.3 shared Ending encodes
     untagged row where receiver expects Fact", 2026-10-13T06:51:23Z).
     31.3's `translate::refusals::ending` (jig `a280d09b`) encodes an
     untagged `EndingRow` into `Ending.rows`, which the client link
     decodes as `Fact`s, so a Done ending reads back as `Fact::Chip`; no
     test crossed the two sides. A step not in the plan, jig 31.s2
     (`jig/31-engine-clients.md`), tags each piece `Fact::Ending` with a
     round-trip row, and jig 34 then reuses the shared helper and drops
     its own.
234. **jig 31.s1 makes and signs in a service; removal and reissue are
     later** (2026-10-13, the designer's answer to question.md's "jig
     31.s1: a service's removal, and a replayed MakeService's credential";
     jig `4092ca1b`, `ec78da12`). Removing a service and reissuing its
     credential are owners' asks of their own, in the design and in
     `later.md`, not in this plan; 31.s1 drops its removal. A replayed
     `MakeService` answers its party with an empty credential, known by
     the digest already kept, and never draws a second. A service's
     sign-in names the identity `make_service` recorded (the deployment,
     the service's number), which people's `signed_in` already finds.
235. **Every test fixture on the real store asks the journal its limits
     admit** (2026-10-13, the coordinator's block "jig 18.3: remaining
     non-peer fixtures have unadmitted 10000-write journals",
     2026-10-13T07:01:40Z; the fifth such block of 18.3, after rulings
     227, 229 to 231). `faults::plan_fixture` and three story fixtures
     (`stories.rs`, `fuzzy_stories.rs`, `stories/watches.rs`) set 10,000
     writes outside the peers' family. Rulings 229 and 231 widen to every
     test fixture that moves onto the real store, in any session: its
     journal comes from its final limits through the one helper (at
     least 2,000 writes), its bounds test-sized as needed, every changed
     bound named old and new in the commit body, every test's name and
     expected value kept. Default and production limits
     (`world::limits`, the profiles) are not changed, and the adapter
     caps nothing. The README's Rules now say so.
236. **jig 18.3 takes what its gate needs of 18.4** (2026-10-13, the
     coordinator's block "jig 18.3: gate needs conformance transition
     currently assigned to 18.4", 2026-10-13T07:18:05Z). A plan defect:
     `tests/conformance` imports the core world's `Store` and
     `store_writes`, which 18.3 replaces and removes, and its
     `Application` harness still takes the fake store, so 18.3's gate
     fails to compile; 18.4 moves conformance, and "Needs: nothing beyond
     18.3". 18.3 makes 18.4's `Durable` and `Application` changes and the
     conformance routing its gate needs, as 18.4 specifies them; 18.4
     keeps its new snapshot test, its joint proof and the measure. No
     temporary bridge API, no fake store restored to the core world.
237. **jig 25.4 lands at its count, 1,808 production lines** (2026-10-13,
     the coordinator's block "jig 25.4: completed planned composition
     exceeds its Size allowance", 2026-10-13T07:24:00Z). Its Size was 600,
     800 with ruling 220, 1,200 under ruling 196's allowance. The work,
     done and proven (26 tests in 0.066 s, clippy clean), composes the
     reference service's accounts driver and private keeper (556 lines),
     tools (412) and workers (309) with the planned routing, rulings 220,
     222 and 223 included; nothing outside its scope, every file under 800
     lines. Splitting finished work costs more than it saves; it lands as
     counted, the 18 per-file counts in its commit body. The Size was
     underestimated, for the retrospective; ruling 196 still binds every
     other step.
238. **The shell hands each worker its root at startup, and the agent its
     host limits** (2026-10-13, the coordinator's block "jig 25.5: worker
     roots and host-domain limits", 2026-10-13T07:59:02Z). As ruling 222
     did for accounts: `startup` gains an optional `WorkerRuntime`, one
     root descriptor per worker, prepared by the binary's shell (the
     worlds fill it with their fakes); a worker whose runtime is absent is
     refused, naming the agent's root path. `Agent` gains `host_limits:
     smith_host_domain::Limits`, supplied by the caller beside its
     protocol limits. No other public field or signature changes; no
     descriptor shared between workers.
239. **`jig-test-service` may name `smith-domain-run` for `Identity`
     alone** (2026-10-13, the coordinator's block "jig 25.5: Smith
     Identity constructor under the dependency restriction",
     2026-10-13T07:59:02Z). `smith::Start.identity` is a
     `smith_domain_run::Identity`, which none of the four smith crates
     `code-organization.md` allows re-exports, at jig's smith pin or on
     smith's `main`; a re-export in smith waits on jig's smith repin
     (ruling 186). The crate is already in jig's graph through those four,
     so a direct edge from `jig-test-service` to `smith-domain-run`, used
     for `Identity::new` alone, is allowed, with its allowance in
     `tests/boundary`'s crate-graph check naming that crate and that
     dependency. No dummy channel `Start`; no other smith type through it.
     It goes when one of the four exports `Identity`.
240. **An activity line carries its turn's bytes, as the design says;
     where they join is asked** (2026-10-13, the coordinator's block "jig
     34.3: activity archive references versus turn bytes",
     2026-10-13T08:10:39Z). 34.3's Scope and Proof follow
     `client/domain/contract.md` ("a turn as its bytes, within the page's
     byte bound, which the client's protocol layer decodes, with no read
     of its own") and `protocol/client.md`, 10; the core's
     `tasks::Line::Turn`, the wire's `OwnerActivityTurn` and the joint
     world's map carry only archive references (`decoded: None`), the
     bytes reachable by a historical read alone. The proposal (references
     only, bytes through `Past::Turn`) would put that read back in the
     client, so it is declined. Where the bytes join an activity page (the
     store's activity read, the core, or the protocol layer) is asked of
     the designer; 34.3 waits for it, and 34.4 goes on.
241. **jig 18.s1: a made store directory is synced into its parent**
     (2026-10-13, the coordinator's block "jig 18.5: newly created store
     directories lost after acknowledgement", 2026-10-13T08:11:25Z). The
     store component makes `store/` and `transcripts/` and never syncs the
     deployment's directory, so skein's crashing machine drops them, and
     with them an acknowledged commit; the store's own world starts with
     them made and missed it. A step not in the plan, jig 18.s1
     (`jig/18-store-worlds.md`), syncs the parent after each make, with a
     unit row and a store-world row from an empty directory; jig 18 runs
     it before 18.5 lands, and 18.5's failing world tests stay as they
     are.
242. **smith's settle test: two fixes, and smith-repin again from smith's
     `main`** (2026-10-13, the designer's answer to question.md's
     "smith's settle test: where `smith check --json` spends its time",
     asked at the user's request; jig `0c10d59a`, `smith-settle.md`).
     Ruling 186's slow test is the test harness's person reading the
     product's 24,940 bytes one io demand at a time (about 25,000 loop
     iterations, 0.45 to 0.9 s) and skein-lib's base64 finding each
     symbol by searching its alphabet (about 0.2 s on the system's trust
     roots, since this run's skein 02.1). Two steps: skein-02.s8 (a symbol
     by its table, one pass) and smith-repin started again from smith's
     `main`, which has taken skein `7efc307` already, carrying the
     person's read by kernel pipe reads; measured with both, the test
     runs 0.05 s alone and 0.15 s in the full suite. The order is
     skein-02.s8, smith-repin, jig-repin's last step, temper-repin; the
     2 s bound stays. smith-repin's old commit `bb26b6d9` is dropped.
     This answers ruling 186.
243. **jig 18.s1 lands the store's world kit with its row** (2026-10-13,
     the coordinator's block "jig 18.s1: supporting unlanded store world
     kit for its Proof", 2026-10-13T08:14:47Z). `tests/store` is new in
     18.5's unlanded work, so 18.s1's world row needs its kit first. As
     proposed: 18.s1 lands the world's kit and Cargo member with its one
     row, ordinary test code, and 18.5 keeps its eight crash tests'
     bodies for its own commit; 18.s1's production stays at most 40
     lines.
244. **An activity page's turns join in the store's read: jig 17.s1 and
     30.s3, then 34.3** (2026-10-13, the designer's answer to
     question.md's "jig 34.3: where a turn's bytes join its activity
     line"; jig `b375b173`, `a85e1c54`). The store's activity read walks
     the page's turn frames backwards in one pass and joins each to its
     line, within the page's byte bound: a turn alone past the bound comes
     as its size, one past what is left ends the page, a frame that does
     not read whole comes as unread. The wire's `ArchivedTurn.turn`
     becomes `TurnBytes { carried | too_large }`, and the joint world's
     per-turn fetch (`archive.rs`) goes. Steps not in the plan: jig 17.s1
     (`jig/17-store-files.md`) and jig 30.s3 (`jig/30-client-wire.md`),
     at once; 34.3 after both, amended; 34.4 goes on.
245. **jig 18.s2: the private secrets directory is synced into its
     parent** (2026-10-13, the coordinator's block "jig 18.5: newly
     created private secrets directory lost after Kept",
     2026-10-13T08:33:25Z). After 18.s1, 18.5's empty-directory world finds
     the same defect for `secrets/`, made by skein's `OpenPrivate`, whose
     design leaves the new entry's durability to the caller. A step not in
     the plan, jig 18.s2 (`jig/18-store-worlds.md`), syncs the deployment's
     directory after `OpenPrivate`, before the secrets' store opens; jig 18
     runs it before 18.5 lands. Whether skein's io should sync a parent
     it makes a directory in, and whether skein's other private-root
     users share the gap, is asked of the designer.
246. **jig 17.s1: a row says why its turn is not joined** (2026-10-13,
     the coordinator's block "jig 17.s1: page-turn transport and identity
     fields", 2026-10-13T08:34:04Z). The store's `reads::Row.related:
     Option<Record>` carries a joined turn, but cannot say "too large" or
     "unread", and the core must refuse a turn that is not its line's.
     `Row` gains one field holding only those two cases: too large, with
     the frame's task, attempt and turn beside its spent, read, at and
     bytes; unread, with the line's turn. A carried turn stays in `related`
     alone, so no state holds a turn twice. Existing `Row` literals take
     the empty value; no schema change. The core maps the three to
     `PageTurn` as 17.s1 says, `Missing::Unknown` when the identity is not
     the line's.
247. **A wire version's move carries its whole sweep** (2026-10-13, the
     coordinator's block "jig 30.s3: exact current-wire v5 to v6 imports
     and fixtures outside Files", 2026-10-13T08:35:19Z). 30.s3 moves the
     client wire from v5 to v6 and `CLIENT_VERSION` with it (ruling 213);
     24 import and qualified-name lines, more than 20 fixture lines naming
     the current version or its served range, and five expected versions
     outside its Files follow. They are the move's, in any file, past
     rulings 114's and 122's limits: each names the new version, names and
     every other expected value kept, no alias for the old version left
     behind. The README's Rules now say so for any family's version move.
248. **smith-repin holds: smith's focused suite does not fit 15 s on
     carbon-forge** (2026-10-13, the coordinator's block "smith repin:
     full focused Proof global timeout with settle passed",
     2026-10-13T08:41:01Z). With ruling 242's two fixes the settle test
     passes (0.259 s in the suite, 0.086 s alone), but smith's focused
     suite reached 930 of 1,921 tests at its 15 s budget. The code-graph
     indexer's per-session processes ran at 590 % CPU then; the
     supervisor set all 22 to idle priority, as ruling 144 did the
     daemon. The designer's measure of the same suite ran 16.6 s, so it
     is over budget here even unloaded. The user measures budgets on the
     slowest machine and raises one only after the suite and its code are
     made as cheap as they can be: smith-repin keeps its clean commit
     `ecb78e34` and lands nothing until the user chooses between a test-
     cost pass on smith's suite and another course. The closing jig and
     temper repins wait; every other step goes on.
249. **A budget that bites is doubled for the rest of this plan; its
     cause is worked on right after** (2026-10-13, the user: "If the
     budget starts biting, we will double the budget temporarily
     (probably for the rest of this plan), until we work on the root
     cause (right after the plan ends)"). A suite bites when a gate on
     carbon-forge fails on its global timeout with every test that ran
     passing. Then, for the rest of this plan, its budget is twice its
     own: 30 s focused, 120 s fuzzy. In jig and skein, the next session
     in that repository sets the profile's `global-timeout` so, as its own
     commit naming this ruling and "until the jig-protocol plan ends".
     In smith, ruling 121's gate stands (the measure profile, every test
     run and passing), and each smith landing's Proof names its focused
     suite's total, at most 30 s. smith's focused suite bites now (ruling
     248): smith-repin lands under ruling 121's gate, its settle times
     (0.259 s in a full run, 0.086 s alone) and the suite's total in its
     Proof. This reverses ruling 248's hold. After the plan, each doubled
     suite gets a test-cost pass (the suite and the code it tests made
     cheaper, as jig's focused suite's two passes did), then its budget
     goes back; listed under "For the user".
250. **skein-03.s1: io makes durable what it makes by a path** (2026-10-13,
     the designer's answer to question.md's "skein: a directory io makes,
     and its entry's durability"; skein `63e81ded`, jig `e70bd54e`, smith
     `1cdc3b20`, `22486b49`). After jig 18.s1 and 18.s2, the designer puts
     the parent's sync in io (`MakeDirectory`, and `OpenPrivate` when it
     makes its root), adds the shell's `make_root`, and has crash worlds
     start with their roots absent. That fixes `skein-oauth-accounts`'
     private root, and smith's credentials once smith takes it. smith's
     shell makes its directories with std and never syncs them; that is
     recorded for smith's own plan (`later.md`). A step not in the plan,
     skein-03.s1 (`skein/03-files.md`), at most 120 lines; nothing waits
     on it, and the final jig-repin takes it if it lands first.
251. **skein-03.s1's accounts story may restart the sign-in world**
     (2026-10-13, the coordinator's block "skein 03.s1: private accounts
     sign-in world moved helpers for acknowledged record",
     2026-10-13T08:52:55Z). With its private root absent, the existing
     `private_cuts` keeper world ends its grant expired before any store,
     so it cannot show an acknowledged record surviving. As proposed: the
     private sign-in world's owner and restart helpers in
     `tests/oauth-accounts/src/sign_in.rs` are 03.s1's Files for that
     story, test code only: restartable constructors and access to the
     store's acknowledgement, reusing the simulated browser and issuer. No
     new kit, no real browser; production stays at most 120 lines.
252. **jig 25.5 lands at 1,274 lines; jig 25's steps left ask past twice
     their Size** (2026-10-13, the coordinator's block "jig 25.5: complete
     worker composition size 1274 beyond 900", 2026-10-13T09:03:02Z). As
     with 25.4 (ruling 237), the reference service's composition runs about
     twice its Size: 25.5's worker root, link and process mapping is done
     and proven (36 tests in 0.104 s), every file under 800 lines. It lands
     as counted, the per-file counts in its commit body. For jig 25's steps
     left, ruling 196's "1.5 times" is "twice" before asking; every other
     session keeps 1.5. Its `Worker::process` (about 230 lines, one arm per
     smith request) is split into one function per request kind when the
     file is next edited (ruling 209: an unchanged move counts for
     nothing). jig 25's Sizes are underestimated, for the retrospective.
253. **skein-03.s1's two absent-root stories: every power loss at each
     file step, under 0.5 s** (2026-10-13, the coordinator's block "skein
     03.s1: complete durability cut stories above 20ms ceiling",
     2026-10-13T09:06:45Z). 03.s1's "each test under 20 ms" is its step
     tests'. Its two world stories, cut at every submission for kill and
     power loss, took 0.143 s (io) and 1.450 s (accounts). As proposed,
     with skein's focused suite at 91 % of its budget: they keep every
     power-loss cut at each file operation that makes or stores (and the
     first after the acknowledgement), drop the kill rows, which the
     existing cut worlds cover, and use owner-only test-sized capacities;
     each runs under 0.5 s; no assertion weakened. The step tests keep 20
     ms.
254. **jig-repin runs now, and jig 25.6 waits for it** (2026-10-13, the
     coordinator's block "jig 25.6: retained Smith agent incompatible with
     retained Skein pipe API", 2026-10-13T09:25:03Z). jig pins smith
     `6a6b1c3e`, older than skein `b92c18a`'s `adopt_write_pipe(fd,
     deadline)`, and skein `4130d9b`, which has it; so the real smith
     agent 25.6 needs (`smith-agent-service`) does not compile against
     jig's lock. smith-repin landed (`6449a197`), and the plan orders
     jig-repin only after it: jig-repin starts now, taking smith's `main`
     and skein's `main` (skein-03.s1 too if it has landed), adapting jig's
     smith-facing crates to smith's host protocol, charter and transcript
     as they stand, and moving jig's Cargo git URLs to GitHub as the
     workspace's AGENTS.md says. 25.6 keeps its work and resumes on it.
     No partial pin move.
255. **The reference service shows its roots' state to a referee, and
     nothing else** (2026-10-13, the coordinator's block "jig 27.1:
     service root-only formatting for the grant-value referee",
     2026-10-13T10:16:00Z). 27.1's referee searches each root's formatted
     state for every grant's bytes (channel.md, 12), and `jig-test-service`
     exposes no root. As proposed: one read-only method on `Service`
     formats each root's domain state (the engine's and each worker's),
     never a component, a value table or a credential, with the service,
     engine and worker files it needs in 27.1's Files. Editing `worker.rs`
     brings ruling 252's split of `Worker::process` into the same
     increment, counted for nothing (ruling 209).
256. **jig 31.s3: the core names an own watch subject's party** (2026-10-13,
     the designer's answer to question.md's "jig 31.5: an own watch
     subject's party after a restart"; jig `d616ae9d`, `18f6a14e`). As
     reads already do, the protocol layer hands the core the sign-in's
     number and the core names the party from the sign-ins its people
     child restores, so a restart loses nothing; no party is kept in the
     protocol layer or the secrets' records. A step not in the plan, jig
     31.s3 (`jig/31-engine-clients.md`), gives `Event::Watch` a `Watched`
     subject with no party for the frame, inbox and conversations; 31.5
     needs it for those three only.
257. **A refused start is answered at most once and never acknowledged**
     (2026-10-13, the designer's answer to question.md's "jig 27.1: is a
     refused start acknowledged once?"; jig `332724e6`, `15a95357`). The
     domains and the code already keep nothing of a refused start; only
     channel.md's referee rule left refusals in. It now reads: every turn,
     and every answer of a run the host started, reaches the engine once
     and is acknowledged once; a refused start's answer reaches it at
     most once and is never acknowledged, and the start ends once,
     refused or lost. No step changes fleet or hub; 27.1's referee takes
     the rule, with the negative test
     `a_refused_start_acknowledged_is_caught`.
258. **jig 27.s1: a turn's read is cumulative** (2026-10-13, the
     coordinator's block "jig 27.1: cumulative native read fence refused
     after its earlier commit", 2026-10-13T11:08:05Z). The channel's turn
     carries "the last message it read, by name, if any", so smith repeats
     a committed read on the next turn; jig's tasks admission refuses a
     read no longer in the inbox, and the attempt fails invalid. The
     design is clear, so no question: a step not in the plan, jig 27.s1
     (`jig/27-channel-stories-and-store.md`), admits a read naming the
     attempt's last committed read as reading nothing new; jig 27 runs it
     before 27.1 lands.
259. **The engine-clients component's default watch window is the core's
     row cap, configured** (2026-10-13, the coordinator's block "jig 31.5:
     default watch rows and the core row cap", 2026-10-13T11:12:54Z). The
     ported watch translation takes a default row window; the component
     had none, and 32 (a wire chip count) passes the small profile's core
     cap of 4, so the real core would answer even a frame busy. As
     proposed: `Settings` gains `rows: u32`, filled by the composition
     from the receiving core's `documents.rows`, checked nonzero at start,
     and passed as the default window to the watch translation. No private
     constant default. A row in the composition's tests, or the reference
     service's, checks a frame watched with the default window opens on
     the small profile.

## For the user

- **Right after this plan ends** (the user, 2026-10-13): a test-cost pass
  on every suite whose budget ruling 249 doubled (smith's focused suite
  now), then its budget restored; and skein's raw reader plan, option (a)
  of "A raw reader over io takes a byte per iteration".
- **Unit tests in one process each crate** (the designer, 2026-10-13;
  `tests-cost.md`, "The second pass"): nextest runs every test in a
  process of its own, and jig's unit-test binaries are 2,654 of its
  3,508 focused tests and 63.2 s of their 97.1 s in the gate. The same
  tests in one process take a fraction: `jig-view` 4.0 s against 13.6 s,
  `jig-client-world` 1.4 s against 10.8 s, `jig-protocol-store` 0.18 s
  against 6.1 s, `jig-domain-core` 0.36 s against 3.2 s. Running each
  unit-test binary as one process (libtest's threads, under the suite's
  same budget), the worlds staying one process per test, would save
  about a third of the suite's test time. It gives up nextest's
  per-test isolation and timeout for unit tests, in every repository
  (skein's `testing-strategy.md`, 8 names nextest). **The user,
  2026-10-13: not now; only when it is needed to keep the suite within
  its budget.** Steps 10 to 12 of `tests-cost.md` go on.

- **Optimized tool builds and the real store's test cost** (the user,
  2026-10-13): the `opt-level = 3` overrides of the repository's tools
  (jig's `jig-boundary` and `skein-codegen`, tests.s4; smith's `sha2`)
  stay, being rebuilt only occasionally and absorbed by the build cache.
  The real store in the domain worlds costs about 1.8 ms per focused
  world test (608 tests 3.5 s on the fake, 4.6 s on the real store) and
  is accepted for real behaviour; the supervisor watches the suite's
  total, and improvements are evaluated when gates approach its limit
  (about 90 % of 15 s).
- **The suites' budgets are measured on the slowest machine** (the user,
  2026-10-13): 15 s focused and 60 s fuzzy hold on carbon-forge (an
  i5-8265U laptop CPU, where skein's suite runs 2.5 to 3.5 times slower
  than on smith's machine). Before a budget is raised, the suite and the
  code it tests are searched for what can be made cheaper (algorithms,
  allocations, limits, fixtures), as the focused suite's two passes did;
  only when nothing more can be had without losing coverage or quality
  is the budget raised.

- **smith's settle test on carbon-forge** (ruling 186; the user chose
  its profiling, 2026-10-13): answered by the designer
  (`smith-settle.md`). The test's time went to its scripted person,
  which reads the product's 24,940 bytes of output one byte per
  iteration of the real loop, about 25,000 iterations; and to the
  product's trust roots, whose base64 the decoder of skein 02.1 decodes
  by searching the alphabet for each symbol, about 200 ms. Two steps
  remove both and keep the bound: skein-02.s8, then smith-repin again
  from smith's `main`. Measured together: 0.05–0.07 s alone, 0.15 s in
  the full suite. The closing jig and temper repins take smith's `main`
  after smith-repin.

- **A turn too large for a page** (the designer, 2026-10-13; ruling
  240; `protocol/store.md`, 7.3): a turn whose bytes alone pass the
  client's page bound comes with its size, and the client shows it as a
  turn too large to read. A smith turn holds up to 8 MiB (a tool's
  output among them), so a client whose page bound is smaller never
  shows such a turn's words. Options: (a) leave it: a client sets its
  page bound high enough that such turns are rare; (b) the contract
  gains a read of one turn, on the person's demand, with a bound of its
  own (the engine's `Scope::Turn` serves it); the activity page still
  makes no read of its own. The designer recommends (a) until a client
  meets such turns. Nothing in this run waits on it.

- **A raw reader over io takes a byte per iteration** (the designer,
  2026-10-13; `smith-settle.md`, 6): skein's stream reads are exact
  (`Fill`, `Scan`, `Line`; lib.md, 7), and io meets a demand in the
  next up pass (io.md, 2), so a reader of bytes that nothing frames
  demands `Fill(1)` and pays one iteration of its loop per byte. smith
  reads a tool's output and a host's standard error so: a tool that
  prints 1 MB costs about a million iterations. Options: (a)
  `lib::stream` gains a read of what has arrived, at least one byte and
  at most a bound, which io meets from its intake at once; every stream
  layer meets it, and a delivery's size then depends on how the bytes
  arrived, which the simulator already draws from its seed; (b) io meets
  a demand its intake already holds in the pass that made it, changing
  io.md, 2's order for every layer above; (c) readers keep `Fill(1)`,
  and its cost. The designer recommends (a), in skein's design, for a
  plan of its own. Nothing in this run waits on it. **The user,
  2026-10-13: (a), as a skein plan of its own; not started yet.**

- **The code-graph daemon's priority** (ruling 144): on carbon-forge,
  `codebase-memory-mcp`'s daemon (pid 272, started at boot) re-indexed
  on every change at normal priority, taking about six of eight cores as
  gates started. The supervisor set it to `SCHED_IDLE` and the idle IO
  class at 20:35Z; a restart of the daemon or the machine loses that. A
  lasting fix is yours: start the daemon under `nice -n 19 chrt --idle 0
  ionice -c 3`, or set `auto_watch` off and index `main` through the
  queue (`heavy codebase-memory-mcp cli index_repository ...`).
  **Done, 2026-10-13, at the user's request:** `codebase-memory-mcp` starts
  through `~/.local/bin/codebase-memory-mcp-idle` (`ionice -c 3 chrt
  --idle 0`), its command in Codex's and Claude's MCP configs; its daemon
  inherits the priority.
- **The gate lane bounds the run** (supervision.md, checks of 17:22 and
  18:22): gates ran 81 to 91 % of each hour, pausing the build lane, so a
  build command waited about 4 minutes and landings fell to 8 to 14 an
  hour. Three changes to skein's kit, which smith's reliability run on
  another machine also uses, so none was made: (a) `land` fetches and
  rebases inside its gate task, just before the gate starts, so a gate
  that waited in the queue does not find `main` moved (about half the
  gates were such regates); (b) the gate's crates compile in the build
  lane first, so only its tests run alone; (c) the suites' global
  timeouts get a margin over their budgets (say 1.5×), reporting SLOW
  instead of failing: 15 s equals the fast suites' budget, and load
  outside the queue failed about 2 gates an hour on it.
  One source of that load (2026-10-12, 20:15Z): the code-graph
  indexer of the Claude sessions' codebase-memory server, at 565 % CPU
  when jig 12.4's gate began, re-indexing after landings outside the
  queue. Ruling 140 retries such gates meanwhile.
- **smith's focused suite on carbon-forge** (ruling 121): it runs past
  its 15 s budget here on `main` itself (16.2 s measured, 2,515 tests),
  against 6.3 to 7.4 s on smith's reliability machine. Is carbon-forge a
  reference machine for smith's budgets, and if so, is the suite made
  cheaper, or its budget raised? Until then, smith lands here without
  the focused suite's timeout.
  **The user, 2026-10-13: wait; a budget that bites is doubled for the
  rest of this plan (ruling 249), and its cause worked on right after.**
- **Retention** (decision 13): when a deployment's store must start
  shrinking ended work.
- **The browser's shell and wasm32** (decision 2): when, and whether jig
  ships them.
- **Live text from workers** (decision 14): wanted for the chat web
  interface; a smith design and plan.
- **Signing out of a shared machine with the engine out of reach**
  (the designer, 2026-10-12; `client/domain/client.md`, 12 and 18): today
  the person stays signed in, the frame says the sign-out did not reach
  the engine, and they may ask again (jig 03.6 builds it). Should a
  native shell also forget its token there, so the machine is signed out
  while the sign-in stays live on the engine until it expires; or a
  client keep the sign-out and send it once the engine is reached? A
  browser's cookie only the engine can clear. Nothing waits on it.
