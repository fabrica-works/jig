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
    that makes the service: the client contract's `Made::Service` carries
    it as a secret field, beside its party (`protocol/client.md`,
    section 8).
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

## For the user

- **The code-graph daemon's priority** (ruling 144): on carbon-forge,
  `codebase-memory-mcp`'s daemon (pid 272, started at boot) re-indexed
  on every change at normal priority, taking about six of eight cores as
  gates started. The supervisor set it to `SCHED_IDLE` and the idle IO
  class at 20:35Z; a restart of the daemon or the machine loses that. A
  lasting fix is yours: start the daemon under `nice -n 19 chrt --idle 0
  ionice -c 3`, or set `auto_watch` off and index `main` through the
  queue (`heavy codebase-memory-mcp cli index_repository ...`).
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
