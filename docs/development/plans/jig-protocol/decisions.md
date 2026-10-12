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
27. **A resumed window's turn count** is bounded by the domain's limit on
    a transcript's turns; the application's resource names by
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
37. **An LLM grant's value, as smith takes it,** is smith's envelope of
    the account's id and its access token (smith's agent service), which
    the engine's protocol layer builds from the table (decision 26).
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

## For the user

- **Retention** (decision 13): when a deployment's store must start
  shrinking ended work.
- **The browser's shell and wasm32** (decision 2): when, and whether jig
  ships them.
- **Live text from workers** (decision 14): wanted for the chat web
  interface; a smith design and plan.
