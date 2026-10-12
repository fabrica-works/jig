# Code audit for jig-protocol (development.md 5.4)

Read-only audit, 2026-10-12. jig `main` = `origin/main` 2fae240a; smith `origin/main` 2a800ea2; skein `origin/main` 5c9eedd. Nothing was edited, committed or checked out in any repository. The plan's scope is STATUS.md:11 and the request: jig-protocol-{config, accounts, tools, worker-wire, engine-workers, worker-link, store, client-wire, engine-clients, client-link}, jig-shell-native, `jig-test-service` plus a test worker, the `jig-test` binary, and the protocol worlds. It excludes the web shell, wasm32 and the browser, and agents in the engine (`smith-inline-agent`, llm.md 1–4 and 7–9). Sections 1–6 follow the request's numbering. Citations are `file:line`, relative to the repository each section names.

## 0. What the plan must settle first

**Upstream: what skein has only designed (section 4.10).**
- jig's pin is skein 30dc6f7. The 8 commits after it on skein's main change Markdown only, so jig already has every line of skein's code.
- Seven pieces the design asks for exist in skein's designs but not in its code, and no skein plan for them exists (skein `docs/development/plans/` holds only `tls-cleanup`):
  - in-process addresses and joined sockets;
  - `read_private_file`;
  - `CryptoRng` and the 32-byte seed;
  - `base64` in skein-lib;
  - `Secret` and the generator's `secret N`;
  - io's `OpenWrite` and `Cut`;
  - the accounts driver dropping a record after "sign in again". skein's STATUS.md does not list this one.
- Each has a fallback, and the plan picks one or schedules skein's step, then smith's and jig's repins (development.md 5.7):
  - loopback TCP in one process for in-process addresses (channel.md allows it; service.md:331-332 and README 4 name in-process addresses);
  - `read_file` or in-loop `OpenPrivate` for `read_private_file`;
  - `Rng`, for tests only, in place of `CryptoRng`;
  - `bytes N` with a hand-guarded `Debug` in place of `Secret`;
  - for `OpenWrite`/`Cut`, **no fallback that fits store.md 8**. A transcript file cannot be reopened for writing after a restart (`skein-io/src/kernel.rs:603-607`, `O_CREAT|O_EXCL`), so the store's transcripts depend on skein, or the store's design changes.

**smith (section 3).**
- jig is already on smith's main. Three gaps:
  - **Call names and facts have no public functions.** `smith-protocol-channel` exports no function for a call's name or for a fact (`calls.rs`/`facts.rs` are `pub(crate)`, `lib.rs:37-46`). The routing must link `smith-channel`'s generated `CallName`/`Fact` directly, or the design (channel.md:651-655, hosts.md:509-520) changes.
  - **There is no "scripted agent process."** smith's worlds run the real agent against skein's fake LLM peer, hosted by `skein_world::HostedProgram` (`tests/agent-process`, `tests/hosts/src/world.rs:217-222,412-428`). channel.md:600-607 and service.md:295-296 need to read that way.
  - **smith's paused reliability plan may still change the process and parent faces** (exit evidence, lossless facts; smith STATUS.md:26,35).

**The domain is owed work before the components can be thin (sections 1.8 and 2.1).**
- **jig-charter only encodes.** It re-implements `smith_protocol_channel::encode_charter`, and has no reader for a run's result or a call's key. ops decodes both in its root, in-step:
  - `examples/ops/domain/src/assemble.rs:254-284`;
  - `translate.rs:261-269`;
  - `translate.rs:878`.
- **The test root links no smith and hands nothing out.** It carries a typed `RunCharter` and `tasks::End` (`testing/jig-test-domain/src/lib.rs:133-146,254-260`). Its world decodes host tools, playing `jig-protocol-tools` (`tests/core/src/peers.rs:98-147`).
- **The hub has no hello value and still bounces messages.** It has no agent bounds, notices, window flag or refusals carried with turns (`crates/jig-host/src/boundary.rs:406-455,545`).
- **The fleet's `Refuse` carries no reason** (`crates/jig-core-fleet/src/boundary.rs:387-389`).
- **The accounts child still refreshes** (`crates/jig-core-accounts/src/boundary.rs:12-67`), against skein's driver's want/expiring vocabulary.
- **Other gaps:**
  - no engine-stopping event;
  - reads fold signed-out into `Unknown` (`crates/jig-core/src/reads/vocabulary.rs:304-306`);
  - grants are never pushed: `fleet::Request::{Grant,Rejected,Exhausted,Told}` hit `unreachable!` (`crates/jig-core/src/routing/children.rs:287-290`);
  - the store's owed items (store.md 15).
- These are the old draft's jig 03–09 (section 6.1), reworked.

**The test application (section 2).**
- **No test worker root exists.** The design names `testing/jig-test-worker`, composing `jig-host` with `smith-host-domain`.
- **Who owns the in-memory system is a decision.** `jig-test-service` must be step code, but the system it answers in memory, `jig-test-system`, is ordinary Rust. The system either becomes step code, or stays in the world, outside the service.
- **The test root has routes nothing has reached yet.** `testing/jig-test-domain/src/routes.rs` has `unreachable!` arms a real service will reach: `:31-33` and `:513-518`, 15 in the crate.

**The fakes (section 2.4).**
- **`jig-fake-store` retires** once the real store runs over `skein_kv::Store::memory` (`skein-kv/src/store.rs:164`). The conformance harness is hard-wired to it (`testing/jig-conformance/src/harness.rs:6,72-109,153`), as is every `Application` impl in `tests/{core,conformance,ops}`.
- **These stay, but their scenarios must also run against the real side** (development.md:667-668):
  - `jig-fake-workers`;
  - `jig-fake-parties`;
  - `jig-scripted-engine`;
  - `tests/host/src/{engine,parent}.rs`.

**The workspace (section 5).**
- **`tests/boundary` is stale and incomplete:**
  - it still names `jig-inline-agent` (`tests/boundary/src/tests.rs:246`);
  - it has no rule for `jig-protocol-*`: none forbids smith, the core on the client's side, or the client's domain on the engine's side;
  - it names crates by string in about 15 places (section 5.3), which the renames break.
- **Generated codecs are scanned for forbidden words.** They are committed under `src/generated/` (skein's generator is a CLI), so the word list and the version-suffix name rule apply to schema field names.
- **The default suite's budget is shared.** It is 15 s, for the whole workspace. The process layer (`jig-test` and the fake LLM peer, about a minute) is a script, not a test.
- **The shell and the binary need their own lint setup.** `jig-shell-native` and `jig-test` are ordinary Rust, and need their own `clippy.toml`, like `testing/jig-test-system/clippy.toml`.
- **No crate has `src/tests/fixtures.rs`.**

**The design still says things the user's scope excludes** (for the designer, or for narrowing in the plan):
- `jig-test-service` composes smith's LLM component for its slots' `smith-inline-agent` (`docs/design/protocol/service.md:236-238`), which is out of scope (STATUS.md:76-80).
- "a browser on loopback", "a real browser" (`service.md:249`, `:309-311`; `protocol/README.md:133-134`), and "built for the host and for wasm32" (`client.md:85-86`) all predate the exclusion.
- How a **native** client signs in by the printed link is not designed (`client.md:266-279`).

**The old draft plan (section 6).**
- None of its sessions has landed.
- **Obsolete:** jig 02 (the smith repin is done), 10 and 27 (the inline agent), 28.2–28.3, and 35 (web).
- **Reworked:** 05, 06, 07, 11, 21, 22, 25, 26, 29, 31, 33.
- **Planned in full from `store.md`:** 34.
- **Reusable nearly whole:** 01 (19 renames remain), 03, 04, 08, 09, 20, 23, 24, 30, 32 (minus 32.4).

## 1. What exists that the protocol layer builds on

Paths are relative to `/srv/data/git/runner/src/jig` unless they say otherwise. Every crate below is `no_std` step code under `[lints] workspace = true`.

### 1.1 jig-charter (602 lines; deps skein-lib, smith-domain-run, smith-charter: `crates/jig-charter/Cargo.toml`)

- **API.** It exports `charter`, `encode` and `EndpointName`, plus the boundary records (`crates/jig-charter/src/lib.rs:15-20`).
  - `charter(Charter, Box<[Section]>, resumed: bool) -> Option<smith_domain_run::Charter>` (`crates/jig-charter/src/charter.rs:86`) maps the core's typed charter into smith's run vocabulary: models, prices with `cache_write` (`charter.rs:19`), outcome rules, host tools with effect and timeout, workspace rights, and the budget with its reserve (`charter.rs:133`).
  - `encode(run::Charter, &[EndpointName], &smith_charter::v2::Limits) -> Option<Box<[u8]>>` (`crates/jig-charter/src/encode.rs:172`) writes smith **charter v2** bytes (`encode.rs:7`; reserve at `:189-194`). It resolves the endpoint number, dialect and account to the configured wire name (`EndpointName`, `encode.rs:12`).
  - The boundary types are `Model{prices,dialect,account,endpoint,name,window,output}`, `Tool{name,description,schema,effect,timeout}`, `Budget{..reserve}` and `Charter{..}` (`crates/jig-charter/src/boundary.rs:9-161`: `Model` `:9`, `Prices` `:23`, `Tool` `:41`, `Budget` `:61`, `Reserve` `:72`, `Contract` `:117`, `Charter` `:148`).
- **It only encodes.** Nothing decodes a run's result or a call's key. The design wants jig-charter to "read every run's result back by the path of host tools' calls" (`docs/design/domain/hosts.md:570-573`; `protocol/channel.md:297-303`) and to read "the call's key (its completion and position)" (`protocol/channel.md:406-409`). Those readers are new work. Today both are done **inside ops's domain root**:
  - the result: `decode_result`, which calls `smith_charter::RunResult::decode` (`examples/ops/domain/src/assemble.rs:254-284`);
  - the call key: `call_name`, which takes `completion` from bytes 8..12 and `position` from 12..16 of the 16-byte name (`examples/ops/domain/src/translate.rs:261-269`), used at `assemble.rs:436-437`;
  - the host tools' inputs: `translate::decode(tool, input)` over skein-json (`translate.rs:878`, with `json` at `translate.rs:281`).
- **Users.** Only `examples/ops/domain`, which calls it in the root itself: `charter::charter` at `assemble.rs:63`, `charter::encode` at `assemble.rs:86`, and `Limits.charter: smith_charter::v2::Limits` at `limits.rs:27`. `tests/ops/src/conformance/config.rs:93` also uses it. The test application (`testing/jig-test-domain`) does not link it.
- **The gap.** The design has the root hand the charter out to the protocol layer to encode (`domain/root.md:178-182`; `hosts.md:495-527`). Here ops's root links `smith-charter` (`examples/ops/domain/Cargo.toml`) and encodes in the step. STATUS records this (`docs/design/STATUS.md:39`, root.md 11-12 "not planned").

### 1.2 jig-client-transcript (871 lines; deps skein-lib, jig-client, smith-transcript)

- `decode(&Limits, &[u8]) -> Result<jig_client::contract::Turn, Undecodable>` (`crates/jig-client-transcript/src/lib.rs:45`).
  - It refuses a size over `limits.bytes` as `TooLarge` (`:46-48`) and any version but **3** as `Version{found}` (`:49-52`).
  - It decodes `smith_transcript::v3::Turn` (`:53`) and classifies smith's `Problem` without naming smith's reason type (`:61-75`).
- `Limits{smith: smith::Limits, bytes, turn: TurnLimits}` (`lib.rs:25-29`); `Undecodable{Version,Malformed,TooLarge,Unknown}` (`:33-42`). Bounded tool-outcome summaries are in `summary.rs`.
- **Users.** Only the joint worlds: `tests/client/src/joint/wire/turns.rs:8`, `joint/wire/activity.rs:201,319`, `joint/archive.rs:104,210`, and `tests/ops/src/joint/fixture.rs:116`. `jig-protocol-client-link` is the first production caller the design names (`protocol/client.md:117-121`).

### 1.3 jig-client: its boundary toward a protocol layer (30,485 lines; dep skein-lib only)

- **The boundary file** is `crates/jig-client/src/boundary.rs` (98 lines). Its `Event` is at `:24-57`; its `Request` at `:61-98`.
  - **Up from the protocol (Event):**
    - `Start{address, kept: List<Record>, offset}` (`:26`);
    - `Hello{versions: VersionRange}`, the stream hello's served range (`:28`);
    - `Kept{scope, requests}` / `KeptGone{scope}`, the restored table (`:30-32`);
    - `Progress{request, progress}` and `Answered{request, answer: Answer}`, from the table (`:40-42`);
    - `Read{read, result: ReadResult}`, a read's one terminal (`:44`);
    - `Watch{watch, event: WatchEvent}` and `Ended{watch, end: WatchEnd}` (`:46-48`);
    - `SignedOut`, which a keyed envelope ends (`:54`).
  - **Down (Request):**
    - `Send{request, scope, ask: Box<Ask>}`, which enters the **root's** keyed table (`:63`);
    - `Read{read, what: Read}` and `Open{watch, subject}`, both unkeyed (`:65-67`);
    - `Continue{watch, subject}`, which resumes run text from the retained turn and offset (`:69`);
    - `Close{watch}` (`:71`);
    - `Save{record}` / `Erase{key}`, the small store (`:75-77`);
    - `SignIn{provider: u16, back}` (`:79`);
    - `Link{link}`, `Confirmed{party}` and `Retention{retention}`, all to the table (`:85-89`).
  - Entry points: `step` (`domain.rs:568`), `fire` (`:1135`), `next_deadline` (`:1209`), `reclaim` (`:1213`), `max_out` (`:557`).
- **The request table and its envelope are skein-lib's.**
  - `RequestTable<O,R>` (`/srv/data/git/runner/src/skein/crates/skein-lib/src/requests.rs:173`): `ask` (`:220`), `restore` (`:235`), `link` (`:244`), `confirmed` (`:250`), `retention` (`:256`), `take` (`:262`), `answered(now, attempt, RequestEnvelope)` (`:268`), `fire` (`:308`), `next_out -> RequestOut::{Save, Erase, Send{attempt,key,request}}` (`:582`; enum at `:72-79`).
  - `RequestEnvelope::{Final, Again, Lost, SignedOut}` (`requests.rs:83-92`) is exactly client.md 5's final/again/signed out/lost.
  - The test client root owns the table: `table: RequestTable<Owner, Ask>` (`testing/jig-test-client-domain/src/domain.rs:15,25`).
  - The root's protocol-facing boundary (`testing/jig-test-client-domain/src/boundary.rs:25-63`):
    - `Event::Answer{attempt, envelope: RequestEnvelope, answer: Option<Answered>}` (`:29`);
    - `Request::Send{attempt, key: RequestKey, ask: Ask::{Jig(Box<jig Ask>), First(..)}}` (`:57`, `:10-13`);
    - `Request::Jig(JigOut::{Read, Open, Continue, Close, ..})` (`:35-42`);
    - `Save`/`Erase` of `store::Record::{Jig, First, Table{version, record: RequestRecord<Ask>}}` (`testing/jig-test-client-domain/src/store.rs:5-14`, `VERSION = 1` at `:3`).
  - So `jig-protocol-client-link` meets the **client root**, not `jig-client` directly.
- **Contract types the wire mirrors.**
  - `Ask` (`contract/asks.rs:121`), `Outcome` (`:180`), `Answer::{Done, Refused}` (`:216-221`).
  - `Read` (`contract/reads.rs:38`), `ReadValue` (`:153`), `ReadResult::{Value, None(ReadNone)}` (`:227`), `ReadNone` including `SignedOut` (`:222`), and the opaque `Cursor` (`:11`).
  - `Subject` (`contract/watches.rs:9-30`), which includes `Run{task, turn: Option<ProvisionalTurnName>, offset}` (`:27`).
  - `Delivery` (`:34`), `RunText{attempt, began, turn, from, through, text, cut, call}` (`:95-104`).
  - `WatchEvent::{Opened, Delivered, Alive}` (`:111`), `WatchEnd::{Closed, Dropped, Unknown, NotAllowed, Busy, SubjectEnded, SignedOut, Stopping}` (`:122`).
  - `VersionRange{first, last}` and `CLIENT_VERSION = 1` (`contract/versions.rs:5-11`); `Refusal` (`contract/refusals.rs:151`); `Turn` (`contract/turn.rs:8`).
  - `contract/documents.rs` is 1,161 lines.
- **Small store.** `StoreKey::{Place, Draft{task}}` and `Record::{Place{version,party,address}, Draft{version,party,task,words,touched}}` (`store.rs:6-20`), `STORE_VERSION = 2` (`store.rs:25`), with `save`, `erase` and `coalesce` (`store.rs:74,86,117`). `Link::{Starting, Live, Offline}` is judged by the frame watch alone (`link.rs:4-11`).

### 1.4 jig-client-view (48,361 lines; deps skein-lib, jig-client)

- It is platform-neutral. Its API:
  - `View` (`crates/jig-client-view/src/render.rs:16`): `new`, `builder`, `finish(built, shown, &mut Queue<Patch>)` (`:49`), `tree`, `bound`, `act(node, Act) -> Option<BoundAct>` (`:75`), `external` (`:80`);
  - `diff(old, new, limits, next_id, out: Queue<Patch>)` (`diff.rs:325`) and `Patch` (`diff.rs:69`);
  - `binding::decode(&mut JigBindings, Token, Act) -> Option<jig_client::action::Act>` (`binding.rs:227`);
  - `Tree`, `Node`, `Role`, `Drawing` (`lib.rs:59-61`).
- **What a native shell needs below it:** apply the `Patch` queue to its own widgets, hand node acts back through `View::act`, and own storage, clock and entropy (`protocol/client.md:88-90`). It touches no protocol type.
- **Files over 800 lines:** `phrases/jig.rs` 2,994, `phrases/english.rs` 1,495, `pages/project/owner.rs` 1,171 and `owner_deployment_words.rs` 1,062, plus tests: `cards_objects.rs` 1,978, `owner_project.rs` 1,226, `turns.rs` 935 and `page_capacity.rs` 823.

### 1.5 jig-host: the hub's boundary (3,202 lines; dep skein-lib only, no smith)

- **What crosses.**
  - Up from the parent: `Event` (`crates/jig-host/src/boundary.rs:12-148`).
  - Down to the parent: `Request` (`:152-262`).
  - Toward the agent: `ToAgent` (`:267-303`) and `FromAgent` (`:308-354`), joined by `Event::from_agent` (`:359`) and `Request::to_agent` (`:378-401`).
  - Entry points: `step` (`domain.rs:215`) and `resume` (`domain.rs:250`).
- **Every smith payload is bytes:**
  - `RunAssignment.charter: Box<[u8]>` (`:415`);
  - `Assignment.turns: Box<[Box<[u8]>]>` (`:424`);
  - `AnsweredCall{name, tool: Box<[u8]>, answer}` (`:431-436`);
  - `Turn{turn, spent, read: Option<Token>, body: Box<[u8]>}` (`:450-455`);
  - `Finish::Ended{outcome: Box<[u8]>}` (`:459`);
  - `Event::Facts{owner, fact: Box<[u8]>}` (`:43-46`): smith's facts, not yet jig's typed activity;
  - `Request::Relay{run, attempt, call, delivery, tool, writes, input, deadline}` (`:154-163`) and `Event::Relayed{answer: Box<[u8]>}` (`:72-77`).
- **Grants** are names only: `Grant{account, generation, valid}` (`:671-675`), in `Request::Grant`, `Event::Grant` and `ToAgent::Grant`.
- **There is no hello value.** The hub answers `Event::Report` with `Request::Hosting{runs: Box<[Hosting]>}` (`:91`, `:227-229`), and the root builds the fleet's `Hello{stop_bound, slots, workstreams, hosting}` (`crates/jig-core-fleet/src/boundary.rs:475-485`). That hello carries **no agent bounds**.
- **Owed to the design** (STATUS row `docs/design/STATUS.md:37`; hosts.md 2, 6.1, 6.4, 6.8):
  - hub-side:
    - the hello with the agent's bounds;
    - the agent's notices (a grant rejected, an account exhausted): the hub has neither variant, though fleet has `Rejected`/`Exhausted` (`jig-core-fleet/src/boundary.rs:198-212`);
    - turns that say whether they open a window (absent from `Turn`, `:450`);
    - the messages refused, carried with turns and the answer;
    - the start's messages and the run's identity in the assignment (absent from `RunAssignment`, `:406-417`);
  - root-side (the root composing an agent capability):
    - smith's payloads by **hand-out**, decoded and encoded by `smith-protocol-channel`'s functions;
    - smith's facts decoded into jig's typed activity.
- **No agent capability is composed anywhere.** No jig crate links `smith-host-domain`, so there is no worker root yet.
- `lib.rs:7-8` still says the agent "may be a process or an inline agent".
- `hosted.rs` is 1,340 lines.

### 1.6 jig-core: the boundary toward the root (`crates/jig-core/src/routing/vocabulary.rs`, 649 lines)

- **Request** (`:18-29`): `Write(Write)`, `Ask{connector, ask}`, `Held(Box<Held>)`, `Now(Box<Now>)`, `Decided`.
- **Store writes:** `Write::{Save(Record), Erase(Key)}` (`:102-107`).
  - `Record::{Core, Tasks, People, Notes}` (`crates/jig-core/src/stored.rs:205-214`).
  - `Key::{Core, Tasks, People, Notes}`, ordered child-first (`stored.rs:218-227`).
  - The root wraps both once more beside its connectors' families (`domain/root.md:200-205`).
- **Store loads and reads** (what `jig-protocol-store` serves through the root):
  - `Held::ReadLoad{load, range: Box<reads::Range>}` (`:117`) is answered by `Event::ReadLoaded{load, result: reads::Loaded}` (`:335`).
  - `Range{party, scope, after: Option<Position>, backwards, rows, bytes}` (`reads/vocabulary.rs:174-181`).
  - `Loaded::{Page{rows: Box<[Row]>, next}, Busy, Gone}` (`reads/vocabulary.rs:208-215`).
  - `Row{value: Record, owner, ancestors, related, linked, inbox}` (`:186-199`).
  - Notes have their own loads: `Held::NotesLoad{owner, range: notes::Range}` (`:162`).
  - Single historical reads: `Now::SeenChat`, `HistoricalProposal`, `HistoricalPersonTask` and `HistoricalEscalation` (`:240-244`, `:266`), answered by events with `row` and `busy` (`:455-484`).
  - Transcripts: `Now::StartPreparation{task, transcript_waiter}` (`:238`), `Now::PrepareAgent` → `Event::PreparedAgent{transcript_waiter, busy}` (`:236`, `:451`), and `Now::AcceptedTurn` → `Event::AcceptedTurn{.., transcript: Box<[u8]>}` (`:230`, `:437-445`).
  - Restart: `RestartStep::{LoadCore, LoadInbox, RestoreConnector, AdoptRuns, ReadAfresh, SettleOutbox, Open}` and `RestartRequest` (`crates/jig-core/src/restart.rs:8-30`), with `restart_begin`, `restart_step` and `restart_done` (`restart.rs:66,105,114`). Rows go in through `restore_core -> Restored::{Deployment{commits}, Live, Archive, Rejected}` (`stored.rs:231-241`, `:329`) and `restore_task_row` (`:274`).
  - **Gaps:**
    - there are no transcript-file requests: the core carries turn bodies as bytes, and keeping them in store.md's per-task files is new;
    - live versus history, and window-scoped turns, are owed (store.md 15; `STATUS.md:74`).
- **Held, released after the commit** (`:111-167`):
  - client-facing: `Delivery{watcher, document}`, `WatchEnded`, `Read{reply, answer}`, `PeopleReply{to, sign_in, reply}` (`:140`), `Applied{to, connector, token}`, `ApplicationDelivery`;
  - fleet: `Assign{channel, run, attempt, activation, assignment}` (`:132`), `Relayed`, `Inbound`, `Acknowledge`, `AcknowledgeTurn`, `Cancel`, `Refuse`, `TurnBusy`;
  - calls: `SettledCall`, `CallAnswer`.
- **Now, with no commit** (`:184-271`):
  - `Call{to, run, attempt, call: fleet::Call}`: "the protocol layer decodes this attested tool and input at the root boundary" (`:191-192`). `fleet::Call{name, tool, writes, input, deadline}` is at `crates/jig-core-fleet/src/boundary.rs:122-131`.
  - `Account(accounts::Request)` (`:202`).
  - `RunText{watcher, turn, from, to, text, cut, call}` (`:204-212`).
  - `WatchRefused`, `SignInRefused`.
  - Root-owned payloads: `TurnPayload` / `AnswerPayload` / `AcceptedTurn` / `RefusedPayload` (`:226-232`). ops's root answers them **synchronously, in-step** (`examples/ops/domain/src/route.rs:1266-1302`, `Now::Call` at `:1362`), not by a hand-out round trip.
- **Event**, the client side (`:331-509`):
  - `Read{reply, sign_in, read}` (`:337`);
  - `Application{reply_to, sign_in, key: [u8;16], project, connector, digest: [u8;32], owner}` (`:339-347`);
  - `SignIn{reply_to, identity}` (`:363`);
  - `Watch{watcher, sign_in, subject, window}` (`:365`), `WatchRun{.., turn, offset}` (`:367`), `Unwatch` (`:369`);
  - `Delivered{watcher}` (`:371`) and `StreamDelivered` (`:373`): the "taken" signals the engine-clients component sends;
  - `RunText{.., turn, retry, offset, text, call}` (`:377-385`);
  - `SettledCall{to, key: CallKey, call}` (`:355`), the protocol-rendered answer;
  - `NamedAnswer` / `NamedAction` / `DelegateValidated` (`:486-490`), which carry the decoded host tool;
  - `Account(accounts::Event)` (`:504`).
- **CallKey** is `{task, attempt, completion, position}` (`stored.rs:447-454`). It needs the decoded smith call name, which is the call-key reader 1.1 notes is missing.
- **Accounts** (`crates/jig-core-accounts/src/boundary.rs`) still run jig's refresh schedule:
  - requests `Refresh / Keep / Cancel / Granted / Availability / Refused / Closed` (`:59-67`);
  - events `Add / Refreshed / Failed / Rejected / Exhausted / Close` (`:12-40`);
  - `Failure::{Unavailable, TimedOut, RateLimited, Refused, Unsaved, Cancelled}` (`:43-56`);
  - `Limits.refresh_margin` / `backoff_*` (`limits.rs:7-15`).
  - The design (`protocol/llm.md:171-198`; `domain/engine.md:762-792`) wants *want / rejected / release / close* down and *granted / failed{later, sign in again, misconfigured, unloaded, cancelled} / expiring / closed* up, with no refresh schedule or generation numbering of jig's. This is a domain rewrite the plan owes (README 12.3).
  - An application's credentials (engine.md 12.1) are absent.
- **Stopping.** There is no engine-stopping event in the core's vocabulary. `views::End::Stopping` exists (`crates/jig-core-views/src/boundary.rs:32-33`), and the client's `WatchEnd::Stopping` too, but nothing in `jig-core/src/routing` raises it (README 12.1 owed).
- **Reads signed out.** The core folds signed out into `Why::Unknown` ("Missing, signed out, or invisible; these are indistinguishable", `reads/vocabulary.rs:304-306`), while the client has `ReadNone::SignedOut` (`jig-client/src/contract/reads.rs:222`). This is owed (STATUS `client/domain/contract.md` 6).
- **Grants pushed and resent at hello and adoption:** fleet `Event::Grant` / `Request::Grant` exist (`jig-core-fleet/src/boundary.rs:194-197`, `:328-333`). Resend at hello and adoption is listed as owed (`STATUS.md:36`).
- Entry points: `step(core, env, Event) -> Requests` (`routing/mod.rs:374`), `fire(core, env, Timer)` (`:517`), `Core::new` (`lib.rs:467`), `reclaim` (`:585`), `quiescent` (`:615`).
- **Files over 800 lines:** `tests.rs` 2,936, `reads/rows.rs` 1,744, `effects.rs` 1,441, `routing/tasks.rs` 1,414, `documents/task_tree.rs` 1,018, `tests/acts.rs` 973, `documents/objects.rs` 933 and `documents/inbox_deployment.rs` 830.
  - Children: `jig-core-people/src/domain.rs` 1,873 and `tests.rs` 1,592; `jig-core-tasks/src/boundary.rs` 1,776, `domain.rs` 1,251, `stored.rs` 1,049, `tests/revisions.rs` 960 and `proposals.rs` 874; `jig-core-fleet/src/tests.rs` 1,268 and `attempt.rs` 1,110.
- `crates/jig-core` is not listed in `Cargo.toml` `members`; it is pulled in as a path dependency.

### 1.7 The client's joint world: the translation the wire follows (client.md 3)

- **Where it is.** `protocol/client.md:103-107` says the records are "field for field" the joint world's translation (`client/domain/testing.md:105-107`). That translation is `tests/client/src/joint/wire/` (33 modules, 11,858 lines; `joint/wire/mod.rs:1-33`), with the correlation layer beside it.
- **Pure total translations** (client contract ↔ core):
  - `asks::ask(&client::Ask) -> Result<people::Ask, Invalid>` (`joint/wire/asks.rs:21`);
  - `answers::reply(people::Reply) -> Reply` (`answers.rs:26`), and `answers::keyed(Reply) -> (RequestEnvelope, Option<client::Answer>)` (`answers.rs:46-53`): Final, Again and SignedOut, with SignedIn and Applied routed separately;
  - `reads::read(&client::Read, cursor_bytes) -> reads::Read` (`reads.rs:17`);
  - `watches::watch(watcher, sign_in, party, &Subject, rows) -> core::Event` (`watches.rs:17`), with `watches::ended(views::End) -> WatchEnd` (`:107`) and `refused(WatchRefusal)` (`:118`);
  - `documents::{frame,task,proposal,escalation,project,deployment,person_task}_read` and `*_delivery` (`documents.rs:29-118`);
  - `refusals::{refusal,lack,limit,moved,hold,ending}` (`refusals.rs:6-148`);
  - `cursor::{encode,decode,encode_tree,decode_tree,belongs_to}`: opaque cursor bytes (`cursor.rs:17-75`, 1,036 lines);
  - `activity::{kind,word,line,with_archive}` (`activity.rs:58-198`);
  - `turns::turn(bytes, limits)` through `jig-client-transcript` (`turns.rs:8`);
  - `family`: the test application's own requests.
- **Correlation, envelopes and digests** (what the components' state machines must reproduce):
  - `transport.rs`: `Transport::new(VersionRange)` (`:86`), `Pace::{Document, Stream}` (`:13-16`), `KeyedOwner::{Jig, Application{connector}}` (`:22-25`), and reply routes `Reply::{SignIn, SignOut, Keyed{client, attempt, key: [u8;16], owner}, Read, ApplicationRead, Archive}` (`:28-35`).
  - `requests.rs::send` (`:33`) and `keyed`, which builds `people::Event::Ask{reply_to, sign_in, key, ask}` (`:130-157`).
  - `application_digest`: SHA-256 over a canonical fixed-field encoding (`requests.rs:169`), standing in for client.md 5's digest of the record's bytes.
  - `packets.rs::{document, stream, ended, refused, dropped}` (`:59-198`), `read_packets.rs::read` (`:27`) and `archive.rs`, which fetches turn records (`:53-163`).
  - `session.rs`: the protocol loop, `Session::new(driver, limits, codec)` (`:68`) and `stream_heartbeat` (`:119`), 1,553 lines.
  - `engine.rs`: the `Driver` over the jig-test-domain root (`:19-86`).
  - `front.rs`: `Joint`, with faults: `version_fault` (`:78`), `restart_configuration` (`:84`), `busy_pressure` (`:90`).
  - `referee.rs` and `truth.rs`, which takes durable truth from whole store images (`truth.rs:23-29`).
- **Files over 800 lines:** `joint/session.rs` 1,553, `joint/packets.rs` 1,104, `joint/wire/cursor.rs` 1,036, `wire/owner.rs` 881, `wire/documents.rs` 865, `src/stories.rs` 878, `harness/world.rs` 840.

### 1.8 Summary of what the domain still owes before the components can be thin

| Owed | Where today | Design |
|---|---|---|
| result and call-key readers in jig-charter; host-tool decoding out of the root | ops root: `assemble.rs:254-284`, `translate.rs:261-269`, `:878` | hosts.md 6.8, 8; channel.md 6.1, 6.3; README 13 |
| hand-out of charter, turns, call names, results and facts | ops root encodes and decodes in-step (`assemble.rs:63-86`; `route.rs:1266-1302`) | root.md 5 (`:178-182`); hosts.md 6.8 |
| the hub's hello with agent bounds; notices; window flag; refused messages; run identity; start's messages | absent: `jig-host/src/boundary.rs:406-455`; fleet `Hello` at `jig-core-fleet/src/boundary.rs:475` | hosts.md 2, 6.1, 6.4 |
| accounts on skein's driver: want, release, five failure classes, `expiring` | `jig-core-accounts/src/boundary.rs:12-67` (a refresh schedule) | llm.md 5.2; engine.md 12, 12.1 |
| the engine stopping | no core event; `views::End::Stopping` unused by the core | README 12.1; engine.md 6.1 |
| reads answered signed out | `Why::Unknown` folds it in (`reads/vocabulary.rs:305`) | contract.md 6; client.md 16 |
| store: live before history, windows, transcript files | core emits `Write` / `ReadLoad` only | store.md 15 |

## 2. The test application and the fakes

Paths are relative to `/srv/data/git/runner/src/jig` unless stated. "Step
code" means `[lints] workspace = true` (the subset, `clippy.toml`);
"ordinary Rust" means a crate with its own `[lints.rust]`/`[lints.clippy]`
table. These files are over 800 lines: `testing/jig-test-domain/src/lib.rs` (1154),
`testing/jig-test-domain/src/routes.rs` (1005), `testing/jig-test-client-domain/src/first.rs` (1011),
`testing/jig-test-client-domain/src/tests/root.rs` (1037), `testing/jig-test-client-view/src/tests.rs` (820),
`testing/jig-conformance/src/referee.rs` (1088), `testing/jig-scripted-engine/src/tests.rs` (2295),
`testing/jig-scripted-engine/src/payload.rs` (1331).

### 2.1 The engine's test root: `testing/jig-test-domain`

- **What it is.** It is step code and `no_std` (`src/lib.rs:11`). Its description is
  "the testing application's root over jig's core and two connectors"
  (`Cargo.toml`). It depends on `jig-core` and its children and on
  `jig-test-connector`. It does **not** depend on `jig-host`, `jig-charter` or
  any smith crate. Its layout does not follow the reference root's
  module list (`docs/design/domain/root.md:301-321`: boundary/store/domain/route/
  translate/assemble/restart/limits). Everything is in `lib.rs` and
  `routes.rs`, and both are over 800 lines.
- **State** (`Domain`, `src/lib.rs:305-327`): `core::Core`, two
  `connector::Domain` (`first`, `second`), `Journal<Write, Delivery>`,
  `assignments`, and `payloads: Map<Token, Payload>`. `Payload`
  (`src/lib.rs:517-523`) is `Turn`, `Answer`, `SettledCall` or `InboxWord`. The
  root itself keeps turn bodies, answers, settled calls and inbox words by token
  (`src/routes.rs:49-77`, `src/routes.rs:338-519`).
- **API:**
  - `new(Config, &Limits)` (`src/lib.rs:368`);
  - `step(&mut Domain, &Env<Limits>, Event)` (`src/lib.rs:525`);
  - `release(&mut Domain, &Env<Limits>, &mut Queue<Request>)` (`src/lib.rs:858`);
  - `restore_record(.., Record) -> bool` (`src/lib.rs:1006`);
  - `core_deadlines() -> [(core::Timer, Option<Time>); 6]` (`src/lib.rs:332`),
    `connector_deadlines()` (`:345`) and `effect_deadline()` (`:351`);
  - `configure_holds` (`:356`), `ready` (`:407`), `escalation` (`:413`),
    `reclaim` (`:420`) and `quiescent` (`:427`).
- **In** (`Event`, `src/lib.rs:166-275`):
  - client application traffic: `WatchApplication` and `ReadApplication`
    (typed `connector::ClientSubject`), and `Application`. Its doc says
    "a protocol-decoded application value with its independently computed
    digest" (`:178-187`), so the protocol layer decodes the request and
    computes `digest: [u8;32]`;
  - `Core(core::Event)`: party, host (`core::Event::Fleet(..)`) and account
    events;
  - `Connector{number, event}`, the system's events;
  - `EffectCall` ("a decoded named effect call", `:221-229`) and
    `ReadAnswer`;
  - `Text`. The root hard-codes `retry: 0` (`:580-592`, `:588`), so "text by retry"
    is owed;
  - `Turn`, with an opaque `transcript: Box<[u8]>` (`:244-252`);
  - `Answer`, with a **typed** `end: tasks::End` (`:254-260`);
  - `TranscriptLoaded`, `Committed` and `Failed`;
  - `RestartBegin`/`RestartDone`, `Timer` and `ConnectorTimer`.
- **Out:**
  - `Request` (`src/lib.rs:150-161`) is one of `Restart(core::RestartStep)`,
    `Commit{number, writes: Queue<Write>}`, `Deliver(Delivery)`,
    `Now(core::Now)` or `Stop`.
  - The store vocabulary is typed: `Key`, `Record` and `Write` (`:25-87`) wrap
    `core::Key`/`core::Record` and the connector's `RecordKey`/`Record`. There
    are no codecs yet.
  - `Delivery` (`:90-129`): `ApplicationDocument`, `ApplicationRead`,
    `Application`, `Projection`, `Message` (an inbox word to its host),
    `Restart`, `Core(core::Held)`, `CallAnswer{name: Box<[u8]>, call:
    core::SettledCall, writes}` (a **typed** answer, not rendered text),
    `Assigned{channel, assignment}`, `Fleet(fleet::Event)`, `System{connector,
    call: connector::SystemRequest}`, `LostRead` and `Procedure`.
- **The assignment** (`src/lib.rs:133-146`): `charter: u32`, `run:
  Box<core::RunCharter>` (typed), `inbox`, `transcript: Box<[Box<[u8]>]>`,
  `answered: Box<[core::SettledCall]>` and `grant: jig_core_accounts::Grant`.
  - It **hands out no smith payload.** No charter is encoded and no result
    is decoded.
  - `ops`'s root, by contrast, encodes the charter itself
    (`examples/ops/domain/src/assemble.rs:63`, `:86`, with
    `jig_charter::charter`/`encode`). The design wants hand-out instead
    (`docs/design/domain/root.md:182-186`, section 5 "smith's payloads, by
    hand-out"). `STATUS.md:39` lists root.md 5, 6, 11 and 12 as not planned.
- **Host tool calls are not decoded in this root or in its protocol layer.**
  The world decodes them (`tests/core/src/peers.rs:98-147`):
  - `Call::Opaque` becomes `fleet::Event::Relay` with an opaque
    name, tool and input;
  - `Call::ProposeGoal` becomes `core::Event::NamedAction`;
  - `Call::Choose` becomes `core::Event::DelegateValidated`.

  The world is playing `jig-protocol-tools`. Answers come back as a typed
  `Delivery::CallAnswer` (`src/routes.rs:53-66`, `:338-353`), where
  `jig-protocol-tools` would render them as text (`channel.md` 6.3).
- **Run results.** The world maps `jig_fake_workers::End` to `tasks::End`
  (`tests/core/src/peers.rs:499-514`). `Now::AnswerPayload` is routed with `saved:
  None, invalid_saved: false` (`src/routes.rs:462-477`). Nothing reads
  smith's `RunResult` bytes.
- **No hub in the engine.**
  - Hosts reach the root as `core::Event::Fleet(..)`. The engine's own slots
    (channel 0) are played by `jig_fake_workers::Worker::new(0, ..)` "over a
    permanent direct link" (`testing/jig-fake-workers/src/lib.rs:6`). The
    worlds do this at `tests/core/src/faults.rs:31`,
    `tests/conformance/src/lib.rs:313` and `tests/core/tests/peers.rs:24`.
  - `Config.hosting: fleet::Kinds` (`src/lib.rs:294`) is passed in
    `fleet::Event::Start`, which only connector 1's `ClaimDone` triggers
    (`src/routes.rs:204-217`).
  - `ops` composes `jig-host` in its engine (`examples/ops/domain/Cargo.toml`
    lines 22-23; `tests/ops/src/conformance/mod.rs:332`). The test application
    does not.
- **Arms a real service will reach:**
  - `core::Now::{SeenChat, HistoricalEscalation, CompleteBrief,
    RestoreRefused}` hit `unreachable!("this route awaits the testing
    application's scripted peer")` (`src/routes.rs:513-518`);
  - `Held::ApplicationDelivery` is `unreachable!` (`src/routes.rs:31-33`);
  - there are 15 `unreachable!` in the crate.
- **What `jig-test-service` would compose around it** (`service.md:234-244`):
  - **The store.** `jig-protocol-store`, with jig's families. The test
    connector's `Record`/`RecordKey` (`testing/jig-test-connector/src/boundary.rs:629-650`)
    need application codecs and keys in the service (store.md 12).
  - **Workers.** `jig-protocol-engine-workers` must translate to the root's
    `Turn`/`Answer`/`Text` events and to `Core(Fleet(Hello|Lost|Relay|..))`.
    That boundary is not the fleet's alone.
  - **Owed domain changes.** `Assigned` must hand `RunCharter` out to be
    encoded by `jig-charter`, and `Answer` must take smith's result bytes,
    handed out to be decoded.
  - **Tools.** `jig-protocol-tools` produces `NamedAction`/`DelegateValidated`
    and renders `SettledCall`.
  - **Clients.** `jig-protocol-engine-clients` translates to
    `Core(SignIn|Request|Watch|Read..)` and to the `*Application` events.
    That needs a codec of `jig_test_connector::{ApplicationRequest,
    ClientSubject, ClientDocument, ApplicationOutcome}`, plus the digest.
  - **Accounts.** `jig-protocol-accounts` serves the accounts child, and
    `Assignment.grant`.
  - **The system.** "Its test connector's system answered in memory"
    (`service.md:239`) is today `jig-test-system`. That crate is ordinary
    Rust (std `BTreeMap`/`VecDeque`, `testing/jig-test-system/src/lib.rs:9`, own lints
    table), while the service is "step code written to be copied"
    (`service.md:37`, `:234-235`). This needs a decision: a step-code in-memory
    system, or the system kept in the world outside the service.
- **No test worker root exists.**
  - `channel.md:600-604` names `testing/jig-test-worker` (smith's scripted
    agent processes and a scripted workspace beneath the test worker's
    root), and `service.md:239` names the test worker. A real one composes
    `jig-host` with `smith-host-domain` and a test workspace.
  - The nearest existing pieces are in-world scripts in `tests/host`:
    `src/engine.rs` is a scripted fleet-side assigner in the hub's
    vocabulary, and `src/parent.rs` is a scripted workspace and agent child,
    standing for smith's parent face. `src/world.rs` (1048 lines) and
    `src/turn_world.rs` are the rest.

### 2.2 The client test application

- **`testing/jig-test-client-domain`** is step code and `no_std`
  (`src/lib.rs:4`).
  - **Root** (`src/domain.rs:10-16`): `jig_client::Domain`, `first::Side`,
    `second::Side` and `skein_lib::RequestTable<Owner, Ask>`.
  - **API:** `new(&Limits, seed)` (`:18`), `step` (`:42`), `fire` (`:45`),
    `next_deadline` (`:55`), `take` (`:67`, the table phase:
    `Save`/`Erase`/`Send`), `shown` (`:93`) and `provider` (`:151`).
  - **Boundary** (`src/boundary.rs:1-63`; "protocol decodes answers; routing
    only unwraps owners"):
    - `Event` is `Start{address, kept: List<Record>, offset}`, `Went`, `Act`,
      `Answer{attempt, envelope: RequestEnvelope, answer: Option<Answered>}`,
      `Jig(jig_client::Event)`, `First(first::Event)` or `SignedOut`;
    - `Request` is `Save`, `Erase`, `Send{attempt, key: RequestKey, ask: Ask}`,
      `Jig(JigOut::{Read, Open, Continue, Close, Answered, Unknown})`,
      `First(first::Out::{Read, Open, Close, Fact})`, `Show`,
      `SignIn{provider, back}` or `Fact`.
  - **The small store** (`src/store.rs:1-15`): `Record::{Jig, First, Table{version,
    record: RequestRecord<Ask>}}`, with `VERSION = 1`.
  - **The test family's client vocabulary**, which the application's client
    wire must encode: `src/family.rs:26` (`Subject`), `:62` (`Document`), `:117`
    (`Ask`), `:133` (`Answer`), `:149` (`ReadResult`), `:160` (`WatchEvent`) and
    `:166` (`WatchEnd`).
  - **Today's translation** to the engine's values is in the joint world,
    `tests/client/src/joint/wire/family.rs:20,58,72,78`.
  - **A stale comment.** That file's header (line 2) says it "remains
    unregistered and unexecuted", but it is registered at
    `tests/client/src/joint/wire/mod.rs:12` and used at
    `tests/client/src/joint/requests.rs:84,97,145` and
    `tests/client/src/joint/packets.rs:249,279,380`.
- **`testing/jig-test-client-view`** is step code and `no_std`. Its API is
  `View`, `build(&mut View, &Domain, &mut Queue<Patch>)` (`src/root.rs:49`) and
  `decode(&mut View, NodeId, Act) -> Option<Action>` (`src/root.rs:181`).
  `jig-shell-native` renders its patches and reports acts.
- **`testing/jig-test-client-kit`** is ordinary Rust: it re-exports
  `connector_bounds`' admission (`src/connector_values.rs:1-5`) for the scripted
  engine's hooks. Only `tests/client` uses it.
- **For jig-protocol:**
  - These three crates are the client that `jig-test` uses through the native
    shell, since the browser is excluded (`STATUS.md:103-108`).
  - The client protocol world (`client.md:390-397`, `tests/client-protocol`)
    runs this root and the fake person over `jig-protocol-client-link` and the
    native shell's machines.
  - No service client root is planned (`STATUS.md:101-102`).

### 2.3 The test connector and its system

- **`testing/jig-test-connector`** is step code and `no_std` (`src/lib.rs:25`).
  - **API:**
    - `Domain`; `step(&mut Domain, &Env<Limits>, Event, &mut Queue<Request>)`
      (`src/domain.rs:158`);
    - `fire` (`:293`), `next_deadline` (`:282`), `resume` (`:252`) and
      `closing_ready` (`:242`);
    - `MAX_OUT = 9` (`:11`).
  - **Types:**
    - `SystemRequest::{ReadFact, Apply, Look}` (`src/boundary.rs:180-189`);
    - `Record` (`src/boundary.rs:629`);
    - `Event` (`src/boundary.rs:493`);
    - its client values: `ClientDocument`, `ClientSubject` (`src/client_documents.rs`)
      and `ApplicationRequest`/`ApplicationOutcome` (`src/application.rs:17,33`).
  - `src/boundary.rs` is 744 lines.
  - **Users:** `jig-test-domain`, `jig-test-client-domain`, `jig-test-client-kit`
    and `jig-test-system`, and the worlds `core`, `conformance`,
    `test-connector` and `client`.
- **`testing/jig-test-system`** is ordinary Rust. It is "a fake system that
  receives the test connector's calls and reports what it saw"
  (`src/lib.rs:1-2`).
  - **API:**
    - `System::new` (`:149`);
    - `answer(SystemRequest, Fault) -> SystemEvent` (`:201`) and
      `deliver_late` (`:248`);
    - `observed` (`:155`) and `reads` (`:161`);
    - `other_hand` (`:185`) and `preload_history` (`:178`).
  - **Users:** `jig-conformance`, and the worlds `core` (6 files),
    `conformance` (4), `ops` (2) and `test-connector` (4).
  - **Fate:** it stays. It answers in memory, with no wire, as
    `service.md:239` wants. Its step-code status is the open point raised in
    2.1.

### 2.4 The fakes: API, users, and what happens to each

The counts are of `.rs` files under `tests/` that name each crate.

| Crate | Fakes | Public API (file:line) | Users | Once the protocol components exist |
|---|---|---|---|---|
| `jig-fake-store` (ordinary Rust, 235 lines) | the store at the domain face: ordered whole commits, held completions, failed commits, slow pages, crash retention (`src/lib.rs:1-7`) | `Store<K,R>` `:60`; `Write<K,R>` `:18`; `Pending` `:27`; `Page` `:38`; `Fault::{Hold, Fail, SlowPages}` `:49`; `new` `:96`; `fault` `:101`; `submit(number, writes, delay)` `:110`; `tick(hold) -> Result<Option<u64>,u64>` `:123`; `release_held` `:158`; `crash` `:164`; `restore(first,last)` `:174`; `page(..)` `:180`; `load(owner,..)` `:208`; `tick_pages` `:216`; `loading` `:232` | `jig-conformance` (harness.rs:6; `Harness.store: Store<A::Key, A::Record>`; the `Application` trait's `start`/`cold`/`inbound`/`deliver`/`poll`/`submitted`/`observed` take `&Store`/`&mut Store`, harness.rs:71-111); core (20 files), conformance (3), ops (6) | **Retires.** store.md:563-567 says "a fake of the store's own component retires once the component exists". domain/testing.md:193-196 has the worlds run `jig-protocol-store` over skein-kv's in-memory mode, with an in-memory stand-in for transcripts' files; STATUS.md:40 agrees. Before it goes, its fault scenarios (hold, fail, slow pages, crash) must pass against the real store (development.md:667-668). Blast radius: the conformance harness's store type and every `Application` impl (tests/core, tests/conformance, tests/ops). |
| `jig-fake-workers` (ordinary Rust, 483 lines) | hosts, meaning the hub and its link together, speaking the fleet's face in their own terms (`src/lib.rs:1-6`); also the engine's own slots (channel 0) | `Worker::new(channel, slots, scripts)` `:168`; `assign(Assignment)` `:195`; `message` `:226`; `call_answer` `:234`; `acknowledge_turn` `:244`; `acknowledge_answer` `:262`; `cancel` `:282`; `fault(Fault, now) -> Vec<Up>` `:296`; `restart_engine` `:326`; `tick(now) -> Vec<Up>` `:344`; `quiescent` `:476`; types `Assignment{task,attempt,budget,transcript}` `:17`, `Call::{Opaque, ProposeGoal, Choose}` `:43`, `Script` `:54`, `Fault::{DropChannel, Vanish, Slow}` `:77`, `End` `:88`, `Up::{Text, Hello, Lost, Turn, Call, Answer}` `:99` | core (9), conformance (3), client (4); translated at `tests/core/src/peers.rs:55-147` | **Stays** for the domain worlds (channel.md:630-631: "the application's worlds keep running its hosts without bytes"). Its scenarios must also run against the real side (development.md:667-668): the test worker root (`jig-host` over `jig-protocol-worker-link`, with smith's scripted agents in `testing/jig-test-worker`) against `jig-protocol-engine-workers` in `tests/channel` (channel.md:600-628). It does not yet model what the real side will show: the hello's agent bounds (also absent from `fleet::Hello`, `crates/jig-core-fleet/src/boundary.rs:475-483`), grants, inbox or charter in the assignment, refusals carried with turns, unread messages, windows, and results as smith bytes (`End` is typed). `ProposeGoal`/`Choose` are decoded by the world (`peers.rs:112-145`); with `jig-protocol-tools` they should come from real tool-call input. |
| `jig-fake-parties` (ordinary Rust, 448+274 lines) | parties at the engine's people face, with no core types (`src/lib.rs:1-4`) | `Party::new(project, role, first_to, script)` `:253`; `waiting` `:271`; `reply(to, Reply)` `:276`; `restart_engine` `:310`; `extend` `:322`; `tick(now) -> Option<Up>` `:328`; `quiescent` `:421`; `random_acts(seed, role, count)` `:434`; `Act` `:119`; `Ask` `:53`; `Reply` `:152`; `Up::{SignIn, Request}` `:226`; refusal facts `src/facts.rs:6-274` | core (5), conformance (3), client (3); translated at `tests/core/src/peers.rs:165ff` | **Stays** for the domain worlds. The protocol world needs "scripted parties' connections" (`service.md:293-300`), so a party's `Up` needs a byte adapter onto `jig-protocol-client-wire`, and its scenarios must run against `jig-protocol-engine-clients` (5.3). |
| `jig-fake-person` (ordinary Rust, `no_std`) | the person, on the view's tree (client/domain/testing.md:47-69); not a fake of a protocol side | `Person<A>::new(script)` `src/lib.rs:161`; `at` `:167`; `next(&mut F: Face, now) -> Next<A>` `:180`; `Step<A>` `:74`; `Find` `:33`; `Target` `:63`; `Face` trait `:149`; `TreeFace::new/subtree` `src/tree.rs:9,17,23`; `RandomPerson::new/next` `src/random.rs:24,32,39` | client (18), ops (3) | **Stays.** It drives the native shell's tree in `tests/client-protocol` (`client.md:390-394`). Its browser face is excluded. |
| `jig-scripted-engine` (ordinary Rust, 15,243 lines) | the engine at the client's typed protocol boundary (`jig_client::contract`), with no engine domain (`src/lib.rs:1-4`) | `Engine::new(Script, seed)` `:200`; `with_application` `:206`; `bounds` `:242`; `versions` `:247`; `truth` `:257`; `commits` `:262`; `pending_commits` `:266`; `script(Rule)` `:271`; `act(now, wall, Change)` `:278`; `poll(now) -> Vec<Up<A>>` `:369`; `next_due` `:400`; `send(now, wall, sign_in, key, attempt, Ask)` `src/requests.rs:13`; `read` `src/reads.rs:187`; watches `open/close/taken/continue_run/write/write_live/write_provisional/write_run_state` `src/watches.rs:26-482`; `fault(now, Fault)` `src/faults.rs:34`; `stop` `src/faults.rs:200`; `Fault::{Busy, NotReady, Lose, DropWatch, Slow, Restart, SignOut, Versions, ..}` `src/faults.rs:10`; `Envelope::{Final, Again, Lost, SignedOut}` `src/lib.rs:76` (mapped onto `skein_lib::RequestEnvelope` at `tests/client/src/scripted.rs:193`); `Up<A>` `src/lib.rs:89-121`; `Application` hooks trait `src/application.rs:41-50` | client (21), ops (4) | **Stays** for the client's world (client/domain/testing.md:78-84). By 5.3, its contract (envelopes, faults, the versions served, watch pacing, a run's text) must be checked against `jig-protocol-engine-clients` and the real core. The client protocol world runs the client world's stories through bytes (`client.md:390-397`). |
| `jig-conformance` (ordinary Rust) | not a fake: the conformance world's harness, referee and shared scenarios | `Harness<A>` `src/harness.rs:149`; `Application` trait `:43-124`; `new` `:174`; `with_trace` `:180`; `on_commit` `:222`; `cut` `:227`; `advance` `:232`; `crash` `:247`; `release_commits` `:262`; `send` `:269`; `iterate` `:278`; `drain` `:395`; `cold_snapshot` `:474`; `from_snapshot` `:495`; `Referee::new(Policy)` `src/referee.rs:367`; `observe(now, Observed)` `:846`; `check_at` `:1013`; `Observed` `:259-306`; `Promise` `:310`; `Scenario` and `ALL` (8 scenarios) `src/scenarios.rs:8-37`; `ScenarioApplication` `:83`; `run(scenario, cut, seed)` `:180`; implements `skein_world::domain::Expectations` `src/referee.rs:1068` | core (2), conformance (16), ops (11) | The **referee is reused**: the protocol world's referee is "the conformance world's, seen through bytes" (`service.md:298-300`). The **harness changes** when the real store replaces `jig-fake-store`, because it is hard-wired at `src/harness.rs:6`, `:72-109` and `:153`. |
| `jig-ops-fake-production` (ordinary Rust, 773 lines) | `ops`'s production, seen by both of its connectors (`src/lib.rs:1-5`) | `Production::new(seed, Backend)` `:257`; `observed` `:295`; `alerts` `:301`; `queue_fault` `:306`; `inject_incident` `:385`; `advance` `:417`; and more | ops only | **Out of the plan's scope**: `ops` standalone is not planned (`STATUS.md:87-89`). It stays. |

**In-world fakes in `tests/host`** (modules, not crates):
- `src/engine.rs` is a scripted fleet side, speaking the hub's vocabulary.
  Once the components exist, its stories should also run against
  `jig-protocol-worker-link` and `jig-protocol-engine-workers`.
- `src/parent.rs` scripts the workspace and agent children. It stands for
  smith's parent face, and should be checked against `smith-host-domain`'s
  parent face (hosts.md 4).

**Where the joint world fakes the wire.** The client's joint world
(`tests/client/src/joint/`) runs the real engine domain against the client
through an in-memory typed translation:
- `wire/*`;
- `packets.rs` (1104 lines);
- `session.rs` (1553 lines);
- `wire/cursor.rs` (1036 lines);
- `wire/owner.rs` (881 lines);
- `wire/documents.rs` (865 lines).

That translation is what `jig-protocol-client-wire`, `jig-protocol-engine-clients`
and `jig-protocol-client-link` replace, with byte-level checks, in
`tests/client-protocol`. Section 1 covers the translation itself.

## 3. smith's pieces a worker needs

smith checkout `/srv/data/git/runner/src/smith` is clean on `main` = origin/main = `2a800ea2`; all paths below are relative to it unless prefixed `jig/`.

### 3.0 What jig pins and links today

- `jig/Cargo.lock:652-671`: `smith-charter`, `smith-domain-run`, `smith-transcript` all at `#2a800ea2...` (= smith's main). skein in both jig's and smith's lock: `30dc6f7` (`jig/Cargo.lock`, `smith/Cargo.lock`, `skein-lib` entry); skein's main `5c9eedd` adds only design commits since (`git diff --stat 30dc6f7 5c9eedd -- crates testing tests` is empty), so no skein repin is needed for code.
- Workspace deps: `jig/Cargo.toml:70-72` (`smith-domain-run`, `smith-charter`, `smith-transcript`, by GitHub URL, no rev).
- Users: `jig/crates/jig-charter/Cargo.toml:11-12` (smith-domain-run, smith-charter); `jig/crates/jig-client-transcript/Cargo.toml:12` (smith-transcript); `jig/examples/ops/domain/Cargo.toml:26`; `jig/tests/ops/Cargo.toml:24-25`; `jig/tests/client/Cargo.toml:34`.
- **New smith deps the plan adds** (worker side, the reference service, `jig-test`): `smith-host-domain`, `smith-host-protocol`, `smith-protocol-channel`, `smith-channel` (for facts and call names, see 3.3), `smith-agent-shell` (binary only), `smith-agent-service` (profile helpers), and for worlds `smith-agent-process-world`, `smith-host-world`, optionally `smith-hosts-world`. None is in `jig/Cargo.toml` today.
- **jig-charter duplicates smith's encoder:** `jig/crates/jig-charter/src/encode.rs:172` `pub fn encode(source: run::Charter, names: &[EndpointName], limits: &wire::Limits) -> Option<Box<[u8]>>` re-implements `smith_protocol_channel::encode_charter` (`crates/smith-protocol-channel/src/charter.rs:16`). jig-charter has **no result reader** (no `RunResult`/`decode` in `jig/crates/jig-charter/src/*.rs` but tests), although `jig/docs/design/protocol/channel.md:296-297` says "`jig-charter`'s reader decodes it"; smith's reader is `decode_declared` (3.3).

### 3.1 smith-host-domain: the parent face

Crate root `crates/smith-host-domain/src/lib.rs:13-65` (no_std, step code). Re-exports: `Input, Output, End, Fault, Invalid, MessageRefusal, Signal` (`:41`); channel vocabulary `Answer, AnsweredCall, Ask, CallName, Grant, Message, Reply, RunResult, SavedReply, Start, Turn, Up, Down, Window, ...` (`:43-47`); `Domain, fire, max_out, step` (`:54`); `Fact` (`:56`); `Limits, worst_case` (`:60`); typed payload envelopes `Charter, Declared, Fields, Transcript, TurnValue` (`:62`).

- **Down (parent -> capability)**, `parent::Event`, `src/parent.rs:12-53`: `Spawn { client: Token, start: Start }`, `Message { agent, name: Token, label: Box<[u8]>, text: Box<[u8]> }`, `Answer { agent, call: Token, reply: Reply }`, `Acknowledge { agent, turn: u32 }`, `Grant { agent, grant: Grant }`, `Stop { agent }`, `Drop { agent }`.
- **Up (capability -> parent)**, `parent::Request`, `src/parent.rs:56-145`: `Started { client, agent }`, `Admitted { client }`, `Called { client, logical_run, call, name: CallName, deadline: Time, ask: Ask }`, `Withdrawn`, `Turn { client, turn: Turn }`, `Waiting { client, read: Option<Token> }`, `Long { client, operation, span }`, `LongDone`, `Rejected { client, account: u32, generation: u64 }`, `Exhausted { client, account, retry_after }`, `Told { client, body: Box<[u8]> }` (an **opaque smith_channel::Fact encoding**, see 3.3), `Answered { client, answer: Answer }`, `Unread { client, names }`, `Faulted { client, fault }`, `MessageRefused { client, name, reason }`, `Gone { client, end: End, detail }`.
- **Envelopes** `src/boundary.rs:12-27`: `Input::{Parent(parent::Event), Process(process::Event)}`, `Output::{Parent(parent::Request), Process(process::Request)}`. `Fault` `:40-51` (Exited, Rules, TooLarge, NoProgress, WallTime); `Invalid` `:68-86`; `End` `:90-102` (Busy, Invalid, Unspawned, Stopped).
- **Typed payloads, not bytes** (`src/values.rs:1-193`): `Charter::new(run::Charter, capacity) -> Option<Self>` `:122`; `Transcript::new(smith_domain::Transcript, capacity)` `:159`; `TurnValue::new(smith_domain::Turn, ...)` `:196`, `.value()/.into_value()`; `Declared::new(run::outcome::Declared, ...)` `:233`; `Fields` `:270`. So the worker's routing must decode jig's bytes into these before building `Start` (smith's `docs/design/domain/host.md:466-470`, "Values, never bytes").
- **Start** `src/channel.rs:21-42`: `identity: Option<run::Identity>` (16 raw bytes, `crates/smith-domain-run/src/identity.rs:10-16`), `messages: Box<[Message]>`, `logical_run: Token`, `activation: u64`, `workspace: Option<Token>`, `charter: Charter`, `transcript: Option<Transcript>`, `answered: Box<[AnsweredCall]>` (`:46-63`, `SavedReply::Host{error, body}|Delivery|TooLarge`), `directories: Box<[Directory]>` (`:77-95`), `grants: Box<[Grant]>`.
- `Grant { account: u32, generation: u64, valid: Duration }` `:98-106`; `CallName { activation: u64, completion: u32, position: u32 }` `:109-117`; `Turn { opens_window: bool, number: u32, spent: u64, read: Option<Token>, body: TurnValue }` `:120-134`; `Ask::{Host { tool, effect, body }, Deliver { fields }}` `:147-166` (host tool input stays the LLM's bytes); `Reply::{Host{error, body}, Delivery, Busy, Unavailable, Withdrawn, TooLarge}` `:170-191`; `RunResult::{Refused{refusal}, Accepted{outcome: Declared}, Parked, Failed{failure}}` `:485-503`; `Answer { read, turns, spent, result }` `:507-519`; process-face `Up` `:524-585`, `Down` `:588-623` (`Start{start, window}`, `Message`, `Answer`, `Acknowledge`, `Grant`, `Cancel`).
- **Domain** `src/domain.rs:131-199`: `Domain::new(&Limits)` `:142` (asserts `worst_case`), `next_deadline` `:160`, `is_due` `:166`, `reclaim` `:174`, `pop_fact() -> Option<Fact>` `:179`, `facts_lost` `:185`; `max_out(&Limits) -> u32` = `calls + 9` `:194`; `step(&mut Domain, &Env<Limits>, Input, &mut Queue<Output>)` `:202`; `fire(...)` `:371`.
- **Limits** `src/limits.rs:12-70` (agents, directories, accounts, charter_bytes, transcript_bytes, turns, turn_bytes, unacknowledged_bytes, fact_bytes, outcome_bytes, spawn_timeout, no_progress, wall_time, grace, kill_after, facts, ...); `worst_case(&Limits) -> Option<u64>` `:80`. Ready-made derivation from an agent profile: `smith_local_service::profile::host_limits(&Profile, largest_turn: u64, message_bytes: u32) -> host::Limits` (`crates/smith-local-service/src/profile.rs:10`; it hard-codes `agents: 1`, so a worker with N slots overrides `agents`).
- **Domain's own facts** `src/facts.rs:6-37`: content-free `Fact::{Started, Admitted, Answered, Faulted, Gone}`, distinct from the agent's `Told` bodies.
- **No hello / agent bounds here.** The "hello's agent bounds" jig's design wants (`jig/docs/design/protocol/configuration.md:99`) come from smith's agent configuration: `smith_agent_shell::config::Configuration.declarations: smith_agent_service::profile::Configuration` (`crates/smith-agent-shell/src/config.rs:31-37`), whose endpoints' `Model { name, window, output, ... }` are shown in `tests/agent-process/src/lib.rs:62-82`. A worker reads them with `smith_agent_shell::config::read(&Path)` `config.rs:187` / `parse(&[u8])` `config.rs:200` (serde, non-step, before the loop).

### 3.2 smith-host-protocol: the host half

`crates/smith-host-protocol/src/lib.rs:32-35` exports `Component, OpenEvent`; `Error, Limits, MaxOut, max_out, output_bytes, worst_case`; `Launch, Process, ProcessEvent, process_worst_case`; `Values, encode_start`. Deps: skein-lib, skein-channel, skein-io, smith-channel, smith-host-domain, smith-domain, smith-protocol-channel, smith-charter, smith-transcript (`crates/smith-host-protocol/Cargo.toml`).

- **Launch** `src/process.rs:19-25`: `{ program: Box<[u8]>, arguments: Box<[Box<[u8]>]>, environment: Box<[Box<[u8]>]>, root: kernel::Fd, directory: Box<[u8]> }` (credential-free env; `root` an io directory fd).
- **Process** (one per agent) `src/process.rs:55-77`: `new(agent: Token, &Limits, detail_bytes: u32, endpoints: smith_protocol_channel::Endpoints) -> Result<Process, Error>` `:89` (agent token must be < 2^63; io owner = token | 2^63, `:95-98`); `spawn(Launch, deadline: Time, below: &mut Queue<IoRequest>)` `:120`; `send_start(channel::Start, channel::Window, Values, token, above: &mut Queue<ProcessEvent>, below)` `:148`; `send_message(name, label, text, ...)` `:163`; `send_reply(call, channel::Reply, ...)` `:178`; `send_acknowledge(turn, ...)` `:192`; `send_grant(channel::Grant, credential: Box<[u8]>, ...)` `:205` (value filled here, not in the domain); `send_cancel` `:219`; `channel_down(Lower, below)` `:231`; `next_channel_down() -> Option<Lower>` `:242`; `signal(kernel::Signal, below)` `:247`; `kill_if_spawned` `:254`; `from_io(now, io::Event, above, below)` `:266`; `fire(now, above, below)` `:391`; `next_deadline()` `:405`. `process_worst_case(&Limits, detail_bytes) -> Option<u64>` `:79`.
- **ProcessEvent** `src/process.rs:29-40`: `Spawned{agent, process}`, `Unspawned{agent, detail}`, `Exited{agent}`, `Reaped{agent, detail}`, `Channel{agent, event: OpenEvent}`.
- **OpenEvent** `src/component.rs:41-76`: `Opened{version}`, `Hangup`, `Sent`, `Unsent`, `Answer{answer: channel::Answer, record: smith_channel::Answer}`, `Admitted`, `MessageRefused`, `Waiting`, `Long`, `LongDone`, `Turn{turn}`, `Fact{body}`, `Rejected`, `Exhausted`, `WriteFailed`, `Call{call, name, deadline, ask}`, `Withdraw{call}`. The half decodes turns/results into typed values and checks facts (`component.rs:490-497`: `smith_channel::Fact::decode` then passes `body` on unchanged).
- **Component** (pipe- or socket-mode channel alone) `src/component.rs:80-330`: `new(&Limits, StreamMode, Endpoints)` `:101`, `open` `:138`, `set_now` `:144`, `send_*` `:149-290`, `from_below` `:315`, `fire` `:322`. `Process` wraps it.
- **Limits** `src/limits.rs:10-22`: `{ bodies: smith_channel::Limits, charter: smith_charter::v2::Limits, transcript: smith_transcript::v3::Limits, endpoints: u32, channel: skein_channel::Limits, start_bytes: u32, calls: u32 }`; `Error` `:25-46`; `max_out` = `{ to_domain: 1, below: 6 }` `:75-76`; `output_bytes(&Limits) -> Option<u32>` `:101` (set into `channel.output_bytes`, as `tests/hosts/src/lib.rs:198-199` does).
- **Values / encode_start** `src/translate.rs:14-19` `Values { paths, credentials }` (in the domain's directory/grant order), `encode_start(start, window, values, &wire::Limits, &v2::Limits, &v3::Limits, &Endpoints) -> Result<Frame, Error>` `:28` (encodes the typed transcript with `encode_turn`, `:50`).
- **What the host's protocol layer must do (the routing to copy).** Two reference compositions: `tests/hosts/src/lib.rs:1-4` ("The host routing below is the part another host copies"), `HostService` `:70-94`, `iterate` `:207-300` (io up -> `Process::from_io` -> `route_process` `:302-322` -> `host::step` -> `route_domain` -> `next_channel_down/channel_down` -> io down; reclaim at the end), `route_channel` `:324-...` (OpenEvent -> `host::Input::Process(process::Event::Received{ message: Up::... })`; `Opened`/`WriteFailed` dropped). Production version: `crates/smith-local-service/src/process.rs:732-819` (`process::Request::Spawn` -> `Process::new` + `spawn`; `Send{Down::Start}` -> `send_start` with `Values`; `Down::Grant` -> `send_grant` with the value from the service's table; `Read/Wait/Reap` are no-ops because `Process` reads and reaps by itself, `:812-814`), `to_host_event` `:821-835`, `channel_event` `:837-887` (uses `unreachable!` for `Opened`, which jig's lints forbid in step code: copy the hosts-world version instead). A worker with N slots keeps a map of `Process` by agent token (both references hold one `Option<Process>`).
- **Design vs code gap in smith:** `docs/design/protocol/hosts.md:88-93` wants exit evidence (`exit`, `forced`) with the process end; `process::Event::Exited { owner }` (`crates/smith-host-domain/src/process.rs:31`) carries none. smith's STATUS (`docs/design/STATUS.md:26,35`) leaves exit evidence, lossless facts and the derived grace to the reliability plan's sessions 02/06 (paused): the process and parent faces may still change under jig.

### 3.3 smith-protocol-channel: codec functions

Exports `crates/smith-protocol-channel/src/lib.rs:37-46`. Endpoint table used by every function:
- `Endpoint { name: Box<[u8]>, number: u32, dialect: u32, account: u32 }` `src/translate.rs:195-205`; `Endpoints::new(List<Endpoint>)` `:215`, `fits(count, name_bytes)` `:227`, `resolve(name)` `:249`, `name_of(number, dialect)` `:260`. On a worker it comes from smith's agent config: `smith_agent_service::Config.channel_endpoints` (`crates/smith-agent-service/src/service.rs:54-61`).

Signatures (host side uses the first six):
- `decode_charter(bytes: &[u8], limits: &smith_charter::v2::Limits, endpoints: &Endpoints) -> Result<run::Charter, run::Invalid>` `src/translate.rs:272` (checks the `[0,2]` prefix `:280`; `Invalid::CharterVersion|MalformedCharter|...`).
- `encode_charter(source: &run::Charter, limits: &v2::Limits, endpoints: &Endpoints) -> Result<Box<[u8]>, Error>` `src/charter.rs:16`.
- `decode_transcript(bytes: &[Box<[u8]>], limits: &smith_transcript::v3::Limits, endpoints: &Endpoints) -> Result<Option<record::Transcript>, record::Refusal>` `src/transcript.rs:18` (first turn must be `First|Window`, others `Ordinary`, `[0,3]` prefix each, one endpoint/dialect; `:33-60`). This is exactly "the current window's turns, in order".
- `decode_turn(bytes: &[u8], number: u32, limits: &v3::Limits, endpoints: &Endpoints) -> Result<record::Turn, record::Refusal>` `src/transcript.rs:77`.
- `encode_turn(source: &record::Turn, limits: &v3::Limits, endpoints: &Endpoints) -> Result<Box<[u8]>, Error>` `src/turn.rs:17` (worker: `TurnValue::value()` -> bytes for the hub).
- `encode_result(result: &run::outcome::Declared, limits: &v2::Limits) -> Result<Box<[u8]>, Error>` `src/translate.rs:439` (worker: `RunResult::Accepted{outcome}` -> bytes); `decode_declared(bytes: &[u8], limits: &v2::Limits) -> Result<run::outcome::Declared, Error>` `src/result.rs:9` (engine: the run's result read back; `Error::InvalidResult`).
- Agent side only: `start_context` `src/translate.rs:50`, `answer_record` `:487`, `host_path` `:726`, `Component/OpenEvent` (agent's channel half), `schema/schema_for_start` `src/limits.rs:156-162`.
- `Error` `src/limits.rs:35-66` (Codec, Channel, Endpoints, Result, ResultCapacity, InvalidSavedAnswer, InvalidResult, Transcript(record::Refusal), TranscriptBody, Order, Calls, Window, Grants, ResultWrite, Answer, Frame). `record::Refusal` (`smith_domain_session::record`): Version, Endpoint, Dialect, Malformed, TooLarge, ...
- **Missing from smith-protocol-channel's public face: call names and facts.** `src/calls.rs:16-107` (`host_call`, `withdraw`, `host_reply`) and `src/facts.rs:11` (`project`) are `pub(crate)` and agent-direction. jig's design assumes smith-protocol-channel functions "to decode ... a channel fact, and to encode ... a call's name" (`jig/docs/design/protocol/channel.md:651-655`; `jig/docs/design/domain/hosts.md:509-520`). What exists: the generated records in `smith-channel` — `smith_channel::Fact::decode(&smith_channel::Limits, &mut Reader) -> Result<Fact, Problem>` with `kind(): FactKind` (`Admitted, Ended, LlmStarted, LlmRetried, LlmFinished, ToolStarted, ToolFinished, CheckStarted, CheckFinished, TextArrived`), `elapsed()`, `count()` (`crates/smith-channel/src/generated/v2.rs:3367-3470`, re-exported `crates/smith-channel/src/lib.rs:28`); `smith_channel::CallName::new(&Limits, CallNameParts { activation: u64, completion: u32, position: u32 })` + `measure/encode/decode` (`v2.rs:455-462`). So either the routing links `smith-channel` directly (my reading: acceptable, it is smith's codec crate; the README's "smith-protocol-channel's functions" is then slightly inaccurate), or jig defines the call name's bytes itself from the three integers (it is not a smith wire payload the hub must store verbatim). The run identity is 16 opaque bytes (`Identity::new([u8;16])`), no codec needed.

### 3.4 smith-agent-shell: smith's agent as a command of an application's binary

- `crates/smith-agent-shell/src/lib.rs:11`: `pub use agent_shell::{Agent, Resources, run}`; `pub mod config`, `pub mod trace`.
- **Entry for `jig-test agent CONFIG.json`:** `pub fn run(path: &Path, errors: Box<dyn Write>) -> Result<Exit, String>` `src/agent_shell.rs:370` (reads config, `skein_shell::seed()`, termination signals, stdin/stdout fds 0/1 as the channel, `Kernel::open`, `drive`). smith's own glue to copy: `src/bin/smith-agent.rs:10-24` and the `agent` arm of `src/main.rs:20-27` (`Ok(Exit::Code(c)) -> ExitCode::from(c)`, signal/err -> FAILURE).
- **For worlds:** `Agent::new(config::Configuration, Resources, Box<dyn Write>) -> Result<Agent, String>` `src/agent_shell.rs:76`; `Agent::read(path, Resources, errors)` `:69`; `Agent: skein_shell::Host` `:223`; `Resources { input, output, signals: Fd, seed, pid, trace, error, roots }` `:20-37`. `config::Configuration { service, memory, trace, profile, declarations }` `src/config.rs:31-37`.
- Footprint: smith-agent-shell depends on serde, serde_json, libc, rustls-native-certs, skein-http/-llm/-llm-connection/-tls (`crates/smith-agent-shell/Cargo.toml`); it belongs in `jig-test`'s binary (and its worlds), never a step crate.

### 3.5 smith's agent processes in worlds (there is no "scripted agent process"; the real agent runs against a scripted LLM)

- **`smith-agent-process-world`** (`tests/agent-process`, package `smith-agent-process-world`): the real agent service on skein's simulated kernel. `configured(configuration: agent::Config, seed, inherited: &skein_world::Inherited, errors: AgentErrors) -> smith_agent_shell::Agent` `src/lib.rs:87-116`; `AgentErrors::pipe(fd) -> (Self, skein_world::PipeCapture)` `:28`; `declarations(environment_bytes)` `:62` (one endpoint, model `fake`, window 1 MiB, output 512); `profile()` `:119`; `limits()` `:143`; `configuration()` `:169` (`.channel_endpoints` for `Process::new`/`decode_charter`); `charter() -> Box<[u8]>` `:370` (a valid v2 charter); `start(charter)` `:251`; `host() -> ScriptedPeer` `:231` (scripted host, for agent-only worlds).
- **Its fake LLM** `tests/agent-process/src/fake.rs`: `peer() -> skein_fake_peers::llm::Peer` `:31` (one turn "Hello"), `configured(scripts: Box<[api::Script]>, credential: skein_llm::Credential, latency) -> llm::Peer` `:48`, `configured_transport(..., Transport)` `:54` (Plaintext or TLS), `configured_anthropic` `:65`, `rejected()` `:71`, `queries(&Peer)` `:126`, `replied` `:138`. Scripts are `skein_fake_llm_domain::api::Script { cue, turns: [api::Turn { lines: [api::Line::Text{..}], finish, tokens }] }` (`:31-45`).
- **How a host world spawns it** — `smith-hosts-world` (`tests/hosts`): `make_agent(&kernel::Spawn, &Inherited) -> Proc` `src/world.rs:217-222`; registered with `harness.host(skein_world::HostedProgram { program: b"smith", make, instances: 1, operations })` `:412-419`; peer added as another proc `harness.spawn(|| Proc::Peer(Box::new(fixture::fake::peer())))` `:428`; `Proc` enum implementing `skein_shell::Host` `:15-97`. This is the template for jig's channel/one-process worlds: the worker's `Launch.program` names a program the world's factory hosts as smith's real agent.
- **`smith-host-world`** (`tests/host`): domain-tier fixtures behind the parent face, no bytes: `limits()` `src/lib.rs:116`, `start()` `:151`, `charter_value(bytes)` `:420`, `turn_value` `:454`, `transcript_value` `:489`, `fields_value` `:506`, `declared_value` `:522`; `inline` module for `smith-inline-agent` (`src/inline.rs`). Useful for `jig-protocol-worker-link` unit tests of the hand-out translation.
- **Real loop:** `smith-real-world` (`tests/real/src/lib.rs:1-4`, real kernel, TLS peers on loopback) and the shipped-binary stories `tests/end-to-end/end_to_end.rs` (binary `smith`, fake peers). These show the process-layer pattern `jig-test`'s worlds would follow.

### 3.6 smith-inline-agent (not built into jig by this plan)

Exists: `crates/smith-inline-agent/src/lib.rs:1-41`; exports `Below, Input, Lower, Output` (`src/boundary.rs:12-74`), `Domain, fire, resume, step` (`src/domain.rs:43, 399, 444, 493`), `Fact, FactKind`, `Limits, max_facts, max_out, worst_case`. Speaks the same `smith_host_domain::parent` face (`docs/design/domain/host.md:459-465`). jig's STATUS keeps agents in the engine "not planned" (`jig/docs/design/STATUS.md:76-80`), so the plan needs only to keep the worker root's capability choice open.

### 3.7 smith-charter v2 and smith-transcript v3 (codec crates jig links)

- `crates/smith-charter/src/lib.rs:10-20`: `pub mod v2` (generated); re-exports `Charter, CharterParts, RunResult, RunResultParts, Form, Field, Item, Section, Tools, HostTool, Contract, Budget, Reserve, Llm, Prices, CEILINGS, ...`. `CharterParts` fields: instructions, brief, tools, contract, conventions, budget, main, models, waiting, resume (`src/generated/v2.rs:1617-1640`); `Charter::new(&Limits, CharterParts) -> Result<Self, Problem>` (`:1655+`); generated `measure() -> u32`, `encode(&self, &mut Writer) -> Result<(), Overflow>`, `decode(&Limits, &mut Reader<'_>) -> Result<Self, Problem>` per record (e.g. `:2083-2106`); version prefix `[0, 2]` (`:4607`). `RunResultParts { form, label, text, fields, items }` `:1974-1989`. `v2::Limits` `:8`, `CEILINGS` `:42`, `worst_case_bytes/heap` `:3659, 3688`.
- `crates/smith-transcript/src/lib.rs:11-26`: `pub mod v3` (generated, `#[path = "generated/v3.rs"]`) and its re-exports; `Turn, TurnParts { kind: TurnKind, affinity, endpoint, dialect, place, usage, spent, messages }` (`src/generated/v3.rs:3863-3884`), `TurnKind::{Ordinary, Window}` `:3811-3816` (the window opener), `Turn::new` `:3897`, `measure` etc.; prefix `[0, 3]` (checked in `smith-protocol-channel/src/transcript.rs:34`).
- jig's current users: `jig/crates/jig-client-transcript/src/lib.rs:17,45` (`decode(limits, bytes) -> Result<Turn, Undecodable>` over `smith_transcript::v3`), `jig/crates/jig-charter/src/encode.rs:7,172`.

## 4. skein's pieces

Paths are relative to `/srv/data/git/runner/src/skein` (the working tree is at origin/main `5c9eedd`) unless they name `smith/` or `jig/`.

### 4.0 What jig pins, and what skein's main adds

- **jig pins skein at `30dc6f7`** (`jig/Cargo.lock:588`). smith's origin/main `2a800ea2` pins the same revision (its `Cargo.lock`). `git diff --stat 30dc6f7 5c9eedd` touches only `docs/` and `scripts/dev/workspace/AGENTS.md`. **The pinned code is skein main's code**, so nothing has to be repinned for what exists. The 8 commits on top are designs only: f6c3e82 joined sockets and in-process addresses, plus the dropped OAuth record; 72462f7 private startup read, base64, `Secret`, wasm32; dec0075 `CryptoRng`; 6d92015 the cryptographic seed; 5c9eedd `OpenWrite`/`Cut`.
- **Skein crates jig names today** (`jig/Cargo.toml` `[workspace.dependencies]`): `skein-lib`, `skein-json`, `skein-world` only. The lock also carries `skein-codec`, `skein-io`, `skein-shell`, `skein-sim` and `skein-heap`, transitively through smith-charter and skein-world. Every crate below except lib, json and world is a new workspace dependency for the plan: `skein-channel`, `skein-codec`, `skein-codegen` (dev), `skein-http`, `skein-kv`, `skein-io`, `skein-shell`, `skein-oauth`, `skein-oauth-accounts`, `skein-sim`, `skein-fake-peers`, `skein-fake-oauth`, `skein-fake-machine`, `skein-fake-channel`. smith declares `skein-codegen` as a git workspace dependency (`smith/Cargo.toml:17`).
- **A repin is coupled to smith.** Once skein builds an item in 4.10, jig's lock moves every skein crate to one revision. smith's crates, which jig compiles, must then build against that skein. So each skein step needs smith's main to follow it, or at least to compile with it.

### 4.1 skein-channel: the framed channel

`crates/skein-channel/src/`, `no_std`. A bare step machine: it owns no socket, timer, credential policy or call correlation (`lib.rs:1-9`).

- **Schema:** `Schema { magic: [u8;4], versions: List<Version> }` (`schema.rs:40`), `Version { version: u16, kinds: List<Kind> }` (`:33`), `Kind { kind: u16, direction: Direction, largest: u32 }` (`:25`), `Role::{Initiator, Responder}` (`:7`), `Limits { chunk, credential, skip, output_bytes, output_frames, kinds }` (`:47`), `Schema::check(role, &limits)` (`:69`).
- **Machine:** `Machine::new(schema, role, limits)` (`machine.rs:148`) and `Machine::with_mode(.., StreamMode::{One, Two})` (`:153`, `:108`). `StreamMode::One` is a socket; `Two` is a pair of pipes, which is what smith uses.
  - Entry points: `down(&mut self, Request, &mut Queue<Event>, &mut Queue<Lower>)` (`:190`), `up(LowerEvent, ..)` (`:205`), `poll(..)` (`:215`).
  - State: `waiting() -> Waiting { read: ReadWait, write: WriteWait }` (`:236`), for the owner's deadlines; `version()` (`:254`); `room() -> Room` (`:269`).
  - `MAX_UP = 3` and `MAX_DOWN = 3` (`boundary.rs:9,12`).
- **Requests** (`boundary.rs:16`):
  - `Open { credential }`: the initiator's opaque bounded credential.
  - `Accept { version }` and `Refuse { reason, text }`: the responder's owner judges the credential.
  - `Read`: one frame of credit.
  - `Send { token, frame }`, `Ping`, `Finish`, `Close`.
- **Events** (`boundary.rs:76`):
  - `Opening { credential, lowest, highest }`, then `Ready { version, terms }` once open.
  - `Body { kind, body }`, `Ping`, `Unsupported`, `Refused`, `Ended`.
  - `Sent`, or `Unsent { why }` (`WrongDirection`, `PeerDoesNotTake`, `TooLarge`, `Full`, `Closed`).
  - `Drained`, `OutputFailed`, and the terminal `Closed { why }`.
- **Lower** (`boundary.rs:105`): `Read(stream::Down)`, `Write(stream::OutputDown)`, `FinishWrite`, with `LowerEvent::{Read, Write, WriteFailed}`. The owner routes these to io's socket stream records itself.
- **Frames:** `frame_writer(kind, body_len) -> FrameWriter` (`frame.rs:77`): `put`, then `finish -> Frame`, so a frame is measured before `Send`. `worst_case(&schema, role, &limits) -> Option<u64>` (`limits.rs:13`).
- **Liveness is the owner's.** The machine sends and reports `Ping`. Timers, the grace period and redial live in jig's `-engine-workers` and `-worker-link` (`channel.md` 10.1 of jig's design).
- **Reference owners in smith:**
  - initiator: `smith/crates/smith-host-protocol/src/component.rs:113` (`Machine::with_mode(.., Role::Initiator, ..)`) and `:139` (`Open` with an empty credential);
  - responder: `smith/crates/smith-protocol-channel/src/component.rs:121`, with `:637-652` turning `Opening` into `Accept { version: highest.min(2) }` or `Refuse`.
  - Both run over pipes and ignore pings (`smith-host-protocol/src/component.rs:601`). jig's channel is the first to use `StreamMode::One` over a socket, with a real credential and liveness.
- **Test kit:** `testing/skein-fake-channel/src/lib.rs:1-4` is a scripted peer for an application's protocol worlds: generic opening, scripted frames, recorded `Observed { kind, body }`, byte cuts. skein's own worlds are `tests/channel/tests/{generic_protocol,fuzzy_channel,memory}.rs`.

### 4.2 skein-codec and the generator

- **Runtime** `crates/skein-codec/src/lib.rs`, `no_std`:
  - `Reason::{Short, Bound, Tag, Bool, Utf8, Version, Trailing}` (`:18`);
  - `read_len` (`:36`), `read_count` (`:48`), `read_text` (`:57`), `version_of(&[u8]) -> Result<u16, Reason>` (`:67`), `text_is_valid` (`:74`).
- **Generator: a CLI, run by hand, with its output committed.** There is no `build.rs` and no macro.
  - `testing/skein-codegen/src/main.rs:10-26`: `skein-codegen <schema> <out-dir>` writes `<out-dir>/v{N}.rs`, and writes goldens to `<crate>/golden/v{N}/*.bin`, where the crate directory is the schema directory's parent.
  - Library: `parse(&str)`, `generate(&Schema) -> Output { rust, goldens }` (`lib.rs:131`).
  - Wire types: `Type::{U8, U16, U32, U64, Bool, Duration, Fixed, Bytes, Text, List, Option, Named}` (`lib.rs:68`). **There is no `secret` type.**
- **Example crate: `smith/crates/smith-charter`.**
  - The schema is `schema/charter-v2.schema` (first line `family smith-charter 2`; records such as `record HostTool { name: text 64 .. input: bytes 65536 effect: Effect deadline: duration }`).
  - The code is `src/generated/v2.rs`, included by `src/lib.rs:11-12` as `#[path = "generated/v2.rs"] pub mod v2;`.
  - Goldens are in `golden/v2/{record,enum}_*_{full,smallest}.bin`.
  - `Cargo.toml` takes `skein-lib` and `skein-codec`, and `skein-codegen` as a dev-dependency only.
  - A drift test regenerates and compares: `smith/tests/codecs/tests/drift.rs:8-19`. skein's own is `tests/codec/tests/drift.rs`.
  - smith-transcript (`schema/transcript-v3.schema`) and smith-channel (`schema/channel-v2.schema`) follow the same layout.
- **Generated API**, from `smith-charter/src/generated/v2.rs`:
  - `pub struct Limits` (`:8`) and `pub const CEILINGS: Limits` (`:42`);
  - `Path` (`:77`) and `Problem` (`:268`);
  - per record: `XParts`, `X::new(&Limits, XParts) -> Result<X, Problem>` (`:291`), byte getters, `into_parts()`, `measure() -> u32`, `encode(&self, &mut skein_lib::Writer) -> Result<(), Overflow>`, and `decode(&Limits, &mut skein_lib::Reader<'_>) -> Result<Self, Problem>` (`:2083-2106`).
- **For jig:** each codec crate the plan writes (`-worker-wire`, `-client-wire`, `-store`'s records) copies this layout: `schema/<family>-v1.schema`, `src/generated/v1.rs`, `golden/v1/`, and a drift test in a tests crate.

### 4.3 skein-http: server, client, server-sent events

`crates/skein-http/src/`, `no_std`. Every machine is `up` and `down` over `lib::stream` (`lib.rs:5-17`), and each publishes `UP_MAX_OUT` and `DOWN_MAX_OUT`.

- **Server** (`server.rs`):
  - `Limits` (`:77`), `Server::new(&Limits)` (`:441`), `waiting()` (`:451`).
  - `up(&mut Server, &Env<Limits>, Up, &mut Queue<Event>, &mut Queue<Down>)` (`:484`) and `down(.., Request, ..)` (`:499`).
  - `Request::{Next, Respond(Response), Body(Down), Discard, Reply(Down), Close}` (`:184`).
  - `Event::{Call(Call), Ended, Body(Up), Reply(Up), Refused, Done(Reuse), Failed, Closed}` (`:220`).
  - `Call { method, target, version, headers, body }` with `header(name)` (`:263`); `Response { status, headers, body: Body::{None, Length, Chunked}, close }` (`:308`).
  - It runs one exchange at a time per connection. A listener and the per-socket stacking are the owner's.
  - In-tree example of an owner: `crates/skein-oauth-accounts/src/listener.rs`, the redirect listener in step code; also `tests/browser/src/serve.rs`.
- **Client** (`client.rs`):
  - `Client::new(&Limits)` (`:334`), `up` (`:359`) and `down` (`:373`).
  - `Request::{Call, Upload(Down), Body(Down), Discard, Close}` (`:157`) and `Event::{Response, Upload, Body, Done(Reuse), Failed, Closed}` (`:182`).
  - Plain HTTP/1.1 over any `lib::stream`, so loopback needs no TLS.
- **Server-sent events reader** (`sse.rs`):
  - `Reader::new(&Limits)` (`:232`); `retry()` and `last_event_id()` (`:258`, `:264`).
  - `up` (`:270`) and `down` (`:314`).
  - `Request::{Next, Data(Down), Close}` (`:118`) and `Event::{Opened, Data(Up), Dispatched(Dispatch{name, id}), Ended, Failed, Closed}` (`:129`).
- **Server-sent events writer** (`sse/writer.rs`):
  - `Writer::new(&Limits)` (`:180`), `up` (`:198`) and `down` (`:223`).
  - `Request::{Event(Outgoing), Comment, Finish, Close}` (`:93`), with `Outgoing { name, data, id, retry }` (`:124`).
  - `Event::{Sent, Refused(Refusal), Failed, Closed}` (`:109`).
- **Not built** (skein `docs/design/STATUS.md`, `http.md` row): reconnecting event streams, a pool per peer, and the server's deadlines. The client link reconnects itself.

### 4.4 skein-kv

`crates/skein-kv/src/`, `no_std`: an ordered byte map held in memory, with a framed append-only log and snapshots, run over io's file layer.

- **Requests** (`boundary.rs:10`):
  - `Open { owner, root: Fd }`
  - `Commit { owner, ops: Box<[Op]> }`, with `Op::{Put{key,value}, Erase{key}}` (`:19`)
  - `Get { owner, key }`
  - `Load { owner, range: Range, max: Page }`, with `Range::prefix` (`:41`) and `Page { rows, bytes }`
  - `Close { owner }`
- **Events** (`boundary.rs:68`):
  - `Opened { last }`, `Committed { number }`, `Refused { refusal }` (`TooLarge`, `Full`, `Busy`, `Unavailable`), `Failed { number }`;
  - `Got { value }`, `Loaded { rows, next }`, `Closed`;
  - `OpenFailed { failure }`, `Recovered { last }`, `RecoveryFailed`.
- **Entry points** (`store.rs`):
  - `Store::new(&Limits)` (`:135`).
  - **`Store::memory(&Limits)` (`:164`), "for protocol tests; commits are applied at once"** (kv.md:58-60). It needs no `Open` and no files. This is the in-memory mode jig's `domain/testing.md` 7 wants in the worlds.
  - `down(store, env, Request, above, below: &mut Queue<file::Request>)` (`:460`), `up(.., file::Event, ..)` (`:627`), `is_ready` (`:872`) and `fire` (`:881`): snapshot chunks.
  - `MAX_OUT { above: 64, below: 2 }` (`lib.rs:36`).
  - `Limits { key, value, ops, commit, queued, queued_bytes, budget, segment, snapshot_after, chunk, page, deadline }` (`limits.rs:7`) and `worst_case` (`limits.rs:44`).
  - `KeyWriter` and `KeyReader` for composite keys (`key.rs`; kv.md 2.1).
  - Commit numbers are contiguous from 1 and survive restarts (kv.md:50).
- **Worlds:** `tests/kv/tests/{memory,store,real,fuzzy_map}.rs`. A disk that crashes is the simulator's machine seam (`skein-world` `Machine::cut`, `testing/skein-world/src/program.rs:90-96`), and `testing/skein-fake-machine` supplies the files.
- **`OpenWrite`/`Cut`: designed only.**
  - io's file vocabulary (`crates/skein-io/src/file.rs:31`) has Create, CreateNoFollow, OpenRead, OpenReadNoFollow, OpenDirectory, Load, Scan, Store, WriteAt, ReadAt, Sync, Stat, Close, Rename, Remove, List, MakeDirectory and OpenPrivate. It has no `OpenWrite` and no `Cut`.
  - `kernel::OpenHow::Create` is `O_CREAT|O_EXCL` (`crates/skein-io/src/kernel.rs:603-607`), so a service cannot reopen an existing file to write once the loop runs.
  - The design is in io.md:553, :557 and :590-597 (commit 5c9eedd). jig's `store.md:394` and `:617` need it for transcripts' files after a restart.
  - The shell's `open_append(root, path, mode)` (`crates/skein-shell/src/ring.rs:1609`) opens append files at **startup only**.

### 4.5 skein-oauth-accounts: the accounts driver

`crates/skein-oauth-accounts/src/`, `no_std`.

- **Configuration** (`accounts.rs`):
  - `Account::{HandedIn, SignIn { registration, endpoint, keeper }}` (`:15`).
  - `Endpoint { address: Addr, transport: Transport::{Tls{server_name, trust}, Plaintext} }` (`:24`, `:31`). `Plaintext` is admitted on loopback only.
  - `Keeper::{Owner { kept: Option<SavedToken> }, Private { root: Token, directory, file }}` (`:40`), where `Private` is io.md 5.3 private files under the owner's `FileIo` root.
- **Requests** (`boundary.rs:10`):
  - `SignIn{account}` and `Redirected{account, uri}`; `Cancel`;
  - **`HandIn { account, record: SavedToken }` (`:18`)**, an access-only record;
  - `Grant{account}`, `Rejected{account, generation}`, `Release`;
  - `Kept{account, generation, keeping}`, `Close`, `Abort`.
- **Events** (`boundary.rs:36`):
  - `Visit{account, url}`, `SignedIn{account, generation}`;
  - `Granted{account, token, generation, valid: Duration}` (`:42`);
  - **`Expiring { account, generation }` (`:44`)**: a handed-in record reached `Limits.refresh_lead`, announced once;
  - `Keep{account, record}`;
  - `Failed{account, ends, failure: Failure::{NotKept, Exchange, Expired, Unloaded{at, why}}}`;
  - `Refused{account, asked, why}`. HandIn's refusals include `Source`, `NotAccessOnly` and `Record(..)` (`Refusal`, `boundary.rs:92-113`; `component.rs:374-392`);
  - `Closed`.
- **Record:** `SavedToken { key, generation, access_token, refresh_token: Option, metadata, expires_at: Wall }` (`crates/skein-oauth/src/record.rs:32`).
- **Component** (`component.rs`):
  - `Component::new(accounts: List<Account>, &Limits, seed: u64) -> Result<_, Unusable>` (`:124`).
  - `down(env, Request, up, io: &mut Queue<Lower>, files: &mut Queue<FileLower>)` (`:223`); `up(env, LowerEvent, ..)` (`:464`) for sockets; `fire` (`:507`); `filed` (`component/private.rs:143`) for file terminals.
  - `has_work` (`:921`), `next_deadline` (`:972`), `reclaim` (`:1030`).
  - `Lower = skein_io::Request` and `LowerEvent = skein_io::Event` (`:24`, `:27`); `FileLower::{Request{request, deadline}, Cancel{owner}}` (`:55`).
  - Room per entry point: `MAX_OUT_{DOWN,FIRE,UP,FILED}` (`:38-47`).
  - `Limits { accounts, file_stall, exchanges, listeners, server, refresh_lead, client, http, tls, io }` (`limits.rs:11`).
  - Unit tests for hand-in: `src/tests.rs:83-168`, including `an_access_only_record_is_announced_expiring_once_and_fails_at_expiry` (`:152`). The world is `tests/oauth-accounts/src/world.rs`.
- **Fake issuer:**
  - `testing/skein-fake-oauth/src/lib.rs`: `Issuer::new(Config, Limits)` (`:118`), `queue(Plan)` (`:150`), `step` (`:191`).
  - Hosted on loopback: `testing/skein-fake-peers/src/oauth.rs`, `Peer::new` (`:83`), `impl Host for Peer` (`:507`).
- **Gap: the record dropped on "sign in again" is designed, not built.**
  - oauth.md 6.4 "Loading again" (added in f6c3e82, 2026-10-10, docs only) says a refresh that fails as sign in again drops the record, so the next grant reloads it.
  - jig's `llm.md:226-228` relies on it for a revoked account.
  - In the code, `fail_exchange` (`component.rs:827-872`) ends the hold but leaves `State::Record` in place. Only an empty account reloads (`component/private.rs:16-35`), and no test covers a dropped record.
  - skein's STATUS.md does not list this as unbuilt. The old draft's `skein/02-record-dropped.md` is this step.

### 4.6 skein-shell: startup kit, `Host`, `drive`

- **Exports** (`crates/skein-shell/src/lib.rs:31-41`):
  - `Clock` and `Now`; `Host` and `drive`;
  - `Kernel` and `Config`, `open_root`, `open_append`, `effective_user`, `process_id`, `open_termination_signals`;
  - process helpers: `start_binary`, `hosted_pipes`, subreaper, pidfd, cgroup;
  - `seed`, `read_file` and `read_trust_roots`, `RootBounds`, `Roots`, `Trust`, `Unread`.
- **`read_file(path, max) -> Result<Box<[u8]>, Unread>`** (`startup.rs:33`), with `Unread::{Absent, NotAFile, TooLarge, Kernel}` (`:20`). It makes no privacy check.
- **`read_private_file`: designed only.** It is in shell.md:162-176 (72462f7) and absent from the code. In the meantime a component can read private files in the loop with io's `file::Request::OpenPrivate` (`crates/skein-io/src/file.rs:40`) and `Load{no_follow}`, as the accounts driver's private keeper does. But jig's configuration.md:31, :43 and :162 and llm.md 5.4 (the handed-in account's file) read at startup, before the loop.
- **`trait Host`** (`drive.rs:12`):
  - `drain`, `iterate(now, wall)` (`:18`), `completions` and `submissions`, `work_pending`;
  - `next_deadline`, and **`next_policy_deadline(&self) -> Option<Time>`** (`:32`): "every host states it explicitly", excluding io close, retry and file stalls;
  - `is_empty`, `exit`, `worst_case`, `operations`.
- **`drive(kernel, clock, host) -> Exit`** (`drive.rs:53`) does not read `next_policy_deadline`. The worlds do: `testing/skein-world/src/world.rs:462` and `real.rs:465` fail a clock advance that needs a policy deadline after the referee's last word.
- **`seed() -> Result<u64, i32>`** (`seed.rs:11`) gives a 64-bit `getrandom` seed for `Rng`. The 32-byte cryptographic seed (shell.md:117-121, 6d92015) is not built.
- **Template `main`:** `examples/echo/shell/src/main.rs`, which configures, opens `Kernel`, and runs `drive` over its `Host`. Its service is `examples/echo/service/src/lib.rs` and its protocol layer `examples/echo/protocol/src/layer.rs` (a listener and connections over io).

### 4.7 skein-lib: `CryptoRng`, `Secret`, `base64`

- **None is built.** `crates/skein-lib/src/lib.rs:42-62` exports `Deadlines`, `Decimal`, `Env`, `Id`, `Intake`, `Journal` and its kin, `List`, `Map`, `Queue`, `Reader`, `RequestTable` and its kin, `Rng`, `Set`, `Slab`, `Stack`, `Duration`, `Time`, `Wall`, `ReplyTo`, `Token`, `Writer` and `Overflow`.
- `Rng` is SplitMix64, documented "Not for secrets" (`rng.rs:3-4`).
- The designs: lib.md:102 (`Secret`), :109 (6.1 `base64`), :308 (`CryptoRng`, ChaCha20 with a 32-byte seed), and codec.md:91 and :113 (`secret N`). skein's STATUS.md says "not built".
- Private base64 copies exist but none is shared: `crates/skein-oauth/src/pkce.rs` and `jwt.rs`, and `testing/skein-browser/src/wire/base64.rs`.
- **Effect on jig:**
  - sign-ins' secrets, the deployment's link and services' credentials need `CryptoRng`. Tests can run on `Rng`; a real deployment cannot.
  - records in server-sent events need `base64` (jig `client.md` 1).
  - `secret N` fields in the wire and store codecs need the generator's `secret` type and `Secret`. Without them the codecs use `bytes N`, and `Debug` leaks the value.

### 4.8 Joined sockets and in-process addresses: designed only (verified)

- `pub type Addr = SocketAddr;` (`crates/skein-io/src/kernel.rs:291`). The requests are `io::Request::Listen { owner, addr: Addr }` and `Connect { owner, addr }` (`crates/skein-io/src/records.rs:18,22`). There is no in-process name.
- The design is io.md:428 (3.5) and :514-530 (section 4), from f6c3e82. Names live in **one io's namespace**, so the engine and its workers must share one io, on one loop.
- skein's STATUS.md, "Joined sockets", says "Waits for its plan, which follows". **No skein plan exists:** `docs/development/plans/` holds only `tls-cleanup`. The old draft's `skein/01-in-process-addresses.md` is that step.
- **Fallback that exists today:** plaintext TCP on loopback within one process. io listens on `127.0.0.1:0`, and the worker's link connects to it. jig's channel.md:20 and :76-77 allow plaintext over loopback, and the simulator models loopback sockets (`crates/skein-sim/src/net.rs`).
- Also not built, and not needed by the first shape: TLS's server side (`crates/skein-tls/src/lib.rs:26` has `pub mod client` only), and contained process trees. Process groups are built: `io::Request::Signal { child, signal, to: Target }` (`crates/skein-io/src/records.rs:42-44`) and `process.rs:1-4`.

### 4.9 The fake LLM peer, skein-world and simulated io

- **The fake LLM peer** is `testing/skein-fake-peers/src/llm.rs`, "Hosted fake LLM process" (`:1-4`).
  - `Peer::new(address: SocketAddr, transport: Transport, limits, provider::Config, Credential, provider::Limits, domain::Domain, domain::Config)` (`:67`); `address()` (`:104`), `observations()` (`:110`), `shutdown()` (`:116`).
  - `impl skein_world::Host for Peer` (`:353`). It refuses an address that is not on loopback (`face.rs:45`).
  - fake-llm.md:88-92: "the byte peer also runs as a process of its own … a simulated world and the real loop host it as they host a service."
- **Is it served as a separate OS process?** Not as shipped. No skein binary wraps it; the only `[[bin]]` targets are `skein-codegen`, `examples/echo/shell`, and `tests/world` and `tests/io`. It is a `Host` in the test's own process, on loopback, which a spawned binary dials. That is the pattern of smith's process-layer worlds: `smith/tests/end-to-end/support/mod.rs:300-334`.
  - `Binary::start(..)` starts the shipped binary.
  - `real::World::new(judge)`, then `world.spawn_with_fds(binary.descriptors(), || Process::Binary(..))`.
  - `world.spawn(|| Process::Script(Proc::Peer(..)))` adds the fake LLM, and optionally the fake issuer `IssuerProcess` with `Transport::Tls`.
  - `world.run(&clock, Duration::from_secs(6))`.
  - jig's process layer, with `jig-test` running a worker whose agent processes dial an LLM, can copy this as it stands. A standalone fake-LLM binary would be a few lines (`skein_shell::drive(&mut kernel, &clock, &mut peer)`) if one is ever wanted.
- **skein-world** is in `testing/skein-world/`, re-exporting `skein_shell::{Host, drive}` (`lib.rs:55`).
  - **Simulated:** `World::new(seed, skein_sim::Config, referee, Memory)` (`world.rs:166`); `with_machine(M)` (`:210`) for the fake file machine; `host(HostedProgram)` (`:239`) answers a process's spawn of a named program with an in-process `Host`, which is how agent processes are hosted.
    - `spawn` and `spawn_root` (`:315`, `:258`), `spawn_restartable` (`:267`), `cut(host, before, Cut)` (`:280`) for crashes at a chosen operation;
    - `run`, `run_with_faults` and `run_with_inputs` (`:347-366`).
    - `HostedProgram { program, make: fn(&Spawn, &Inherited) -> P, instances, operations }` (`program.rs:26`); `Machine` trait (`program.rs:90`).
  - **Real ring:** `real::World::new` and `new_checked` (`real.rs:148`, `:173`), `spawn_with_fds` (`:229`), `run(clock, patience)` (`:242`).
  - **End to end:** `end_to_end::{Command (:25), Mode, Binary (:76), Binary::start (:131), start_with_tree (:140)}`, with stderr capture and signals to the process, its group or its tree.
  - **Referee:** `trait Referee<P> { act, observe, next_deadline, overdue, passed, last_word }` (`referee.rs:40`).
  - **Domain-only kit:** `skein_world::domain::{Ledger, Stage, schedule, trace, referee}` (`src/domain/*`).
- **The simulator:** `crates/skein-sim`, `Sim::new(seed, Config)` with `Config::calm` and `Config::chaos`. It simulates sockets between hosted processes (`net.rs`) and passes file calls to the embedder's machine (`lib.rs:1-30`). Files come from `testing/skein-fake-machine/src/lib.rs` (`Machine::lay`).

### 4.10 What the design needs that skein's main does not build

| # | Piece | skein design | Built? | jig needs it for | Without it |
|---|---|---|---|---|---|
| 1 | Joined sockets, in-process addresses | io.md 3.5, 4 (f6c3e82) | **no**, and no skein plan | one-process shape: workers' links joined to the engine (README 4, service.md 10, channel.md 10) | plaintext TCP on loopback in one process. The design admits it (channel.md:20), but service.md:331-332 names in-process addresses for one process |
| 2 | `read_private_file` | shell.md 6.1 (72462f7) | **no** | configuration's secrets: workers' secrets and digests, the handed-in account's token file (configuration.md 7, llm.md 5.4) | `read_file` (no checks), or in-loop `OpenPrivate`/`Load` through io |
| 3 | `CryptoRng` and the 32-byte seed | lib.md 8, shell.md 5 (dec0075, 6d92015) | **no** | sign-ins' secrets, the deployment's link, services' credentials | `Rng` (insecure). Acceptable for tests only |
| 4 | `base64` in skein-lib | lib.md 6.1 (72462f7) | **no** | client records in server-sent events (client.md 1) | a jig-local copy, against "generic leaves jig" |
| 5 | `Secret`, and the codec's `secret N` | lib.md 6, codec.md 3–4 | **no** | credentials' values in the wire and store codecs (README 8) | `bytes N`; `Debug` must be hand-guarded |
| 6 | io `OpenWrite` / `Cut` | io.md 5 (5c9eedd) | **no** | transcripts' files resumed after a restart (store.md:394, :617) | a new file per task per restart, or transcripts in kv. A store design change |
| 7 | Accounts driver drops a record that failed as "sign in again" | oauth.md 6.4 (f6c3e82) | **no** (code: `component.rs:827-872`) | a revoked LLM account re-signed without a restart (llm.md:226-228) | a restart after re-sign-in |
| 8 | TLS server side | tls.md 8 | no (not planned) | between hosts only | out of scope (one process first) |
| 9 | Contained process trees | io.md 6, draft/process.md | no (not planned) | between hosts | process groups (built) |
| 10 | wasm32 builds | lib.md 2, http.md 2, codec.md 2 | no | the browser only | excluded by the user |

Built and usable as they are:

- skein-channel;
- skein-codec and skein-codegen;
- skein-http's server, client, SSE reader and SSE writer;
- skein-kv, including `Store::memory`;
- skein-oauth-accounts, with `HandIn`, `Expiring`, private keepers, and the fake issuer and its peer;
- skein-json, for strict configuration;
- the shell's `Host` and `drive` with `next_policy_deadline`;
- skein-world: simulated, real, end to end, hosted programs, cuts;
- skein-fake-peers' LLM peer on loopback, which smith's end-to-end harness uses alongside a binary;
- skein-fake-channel.

Items 1–6 are all in skein's STATUS.md as designed and unbuilt; item 7 is not, and should be added. The old draft's `skein/01..05` sessions cover 1, 7, 2, 4–5 and 3, in that order.

## 5. The workspace

### 5.1 Cargo.toml (jig `main` 2fae240a)

- **Members** `Cargo.toml:5-52`: 47 listed; `crates/jig-core` is not listed but is a member as a path dependency inside the root (it is `jig-core = { path = "crates/jig-core" }`, `Cargo.toml:96`). No `jig-protocol-*`, `jig-shell-*`, `jig-test-service`/`jig-test` member exists yet.
- **Where crates live, by kind:** step crates in `crates/` (jig-charter, jig-client*, jig-core*, jig-host) and `examples/ops/*`; test kits in `testing/` (incl. the step-code roots `jig-test-domain`, `jig-test-connector`, `jig-test-client-domain`, `jig-test-client-view`); worlds in `tests/<name>` (package `jig-<name>-world`). service.md 7 puts `jig-test-service` "among jig's test kits but written as step code" (`docs/design/protocol/service.md:234-244`) → `testing/jig-test-service` with `[lints] workspace = true`, like `testing/jig-test-domain`.
- **Workspace deps** `Cargo.toml:60-101`: upstream only `skein-lib`, `skein-json`, `skein-world` (`:67-69`), `smith-domain-run`, `smith-charter`, `smith-transcript` (`:70-72`), `sha2 =0.10.9` (`:73`); all git URLs already GitHub (`https://github.com/forgetic/...`), so the workspace AGENTS.md line "jig's and temper's Cargo git URLs still name git.ekanayaka.io" is stale for jig (moved in 0e952778). `jig-core-world` is not a workspace dep: `tests/client/Cargo.toml:26` uses `{ path = "../core" }`.
- **To add for the plan:** `skein-channel`, `skein-codec` (+ the generator crate smith uses, `skein-codegen`, see smith `Cargo.toml:17`), `skein-http`, `skein-kv`, `skein-oauth-accounts`, `skein-io`, `skein-shell`, `skein-sim` (worlds), `skein-llm`'s fake peer, and smith's `smith-host-domain`, `smith-host-protocol`, `smith-protocol-channel`, `smith-agent-shell` (+ smith's scripted agent kit for worlds). jig must resolve one revision per git source shared with smith's crates (development.md 5.7, step 1).
- **Pins** (`Cargo.lock`): skein `30dc6f7d` for all 8 skein packages (`skein-codec, -heap, -io, -json, -lib, -shell, -sim, -world`), smith `2a800ea2` (= smith `origin/main`). smith's own lock pins skein at the same `30dc6f7`. skein `origin/main` 5c9eedd is 8 commits ahead of 30dc6f7 and **all 8 are Markdown only** (`git diff --stat 30dc6f7 origin/main`: `docs/design/{STATUS,codec,http,io,kernel,lib,oauth,shell,testing}.md`, `docs/foundation/programming-model.md`, `scripts/dev/workspace/AGENTS.md`): joined sockets/in-process addresses (f6c3e82), private startup read/base64/`Secret`/wasm32 (72462f7), `CryptoRng` (dec0075), the shell's seed (6d92015), `OpenWrite`/`Cut` (5c9eedd) are designs, no code. So jig's pin = skein's code on main; every upstream piece the protocol design asks of skein beyond "what it has, as it is" (`protocol/README.md:283-326`) is unbuilt (see section 4).
- **Profiles** `Cargo.toml:105-111`: `panic = "abort"`, `overflow-checks = true` in dev and release (test harness ignores `panic`). A process-layer test that spawns the `jig-test` binary gets an aborting binary: a panic in it shows as a signal/exit, not an unwind.
- **Lints for step crates** `Cargo.toml:116-187`: `[workspace.lints.rust]` (`unsafe_code` deny, `missing_debug_implementations` deny, …) and `[workspace.lints.clippy]` (pedantic warn; the 10.3 table: `wildcard_enum_match_arm`, `indexing_slicing`, `arithmetic_side_effects`, `as_conversions`, `float_arithmetic`, `unwrap_used`, `todo`, `impl_trait_in_params`, `disallowed_types/macros/methods`, `allow_attributes`, `panic`, `print_stdout/stderr`, `use_debug`, `host_endian_bytes`, `default_numeric_fallback`, `std_instead_of_core`/`alloc`, `partial_pub_fields`, `mod_module_files`, `tests_outside_test_module`, `iter_over_hash_type`, all deny). Every step crate opts in with `[lints] workspace = true` and `#![cfg_attr(not(test), no_std)]` + `forbid(unsafe_code)` at its root (all 20 crates under `crates/` and `examples/ops/*`, and `testing/jig-test-{domain,connector,client-domain,client-view}`).

### 5.2 clippy.toml and the two lint regimes

- **`clippy.toml`** (root, applies to every crate without its own, `clippy.toml:1-5`): `allow-{unwrap,expect,panic,indexing-slicing}-in-tests = true` (`:17-20`); `disallowed-types` `:22-79` (no `Vec`, `VecDeque`, `String`, `Cow`, `Rc/Arc`, cells, atomics, `Pin`, `Future/Poll/Context/Waker`, `Fn*`, `Any`, `PhantomData`, floats, `core::fmt::Write`, `core::time::Duration`, `Instant`, `SystemTime`, `HashMap/Set`); `disallowed-macros` `:81-102` (`format!`, `vec!`, `matches!`, `write!`, `debug_assert*`, `Default`, `include*`, `env!`, `thread_local!`, `dbg!`, `print*`); `disallowed-methods` `:104-210` (every closure combinator, `collect`, `sum`, `to_vec`, `to_owned`, `to_string`, `split_at`, `copy_from_slice`, **`core::str::from_utf8` and `str::parse` (`:203-205`, "the protocol layer validates text/parses")**, `mem::take`, `Box::leak`). Note for the plan: protocol crates are step code under this file too, so they cannot call `str::from_utf8`/`str::parse` directly either; text validation and number parsing go through skein-lib/skein-json/skein-codec's own readers (skein's are exempt as they are skein's).
- **`tests/clippy.toml`** (worlds, ordinary Rust, `tests/clippy.toml:1-13`): only `HashMap/HashSet`, `Instant`, `SystemTime`, and `std::thread::spawn` banned ("one thread, so that a run replays"). A process-layer test under `tests/` that runs a loopback server on a thread would breach `:12`; spawning `jig-test` and skein's fake LLM peer as child processes (`std::process::Command`) does not.
- **Test kits** (`testing/*` but the four step roots): own `[lints.rust]`/`[lints.clippy]` tables. Two patterns: (a) `testing/jig-fake-workers/Cargo.toml:12-28` (also fake-parties, fake-store, scripted-engine, conformance, test-client-kit): step lints subset + `disallowed_types/methods/macros = "allow"` against the root clippy.toml; (b) `testing/jig-test-system/Cargo.toml:15-30` + its own `clippy.toml:1-11` (like `tests/clippy.toml`), also `testing/jig-ops-fake-production/clippy.toml`, `tests/ops/clippy.toml`. `jig-shell-native` and the `jig-test` binary are "ordinary Rust like skein's shell" (jig `AGENTS.md:23-30`): they need pattern (b) (own `clippy.toml` banning only replay/time hazards), since the root clippy.toml would otherwise ban `Vec`, `String`, `format!`, `print*` in them.
- **`rustfmt.toml`:** `max_width = 120`, `use_small_heuristics = "Max"`.

### 5.3 tests/boundary (`jig-boundary`): what it enforces

`tests/boundary/src/tests.rs` (1009 lines, over the 800 limit), `names.rs` (211), `fixtures/names.txt`. It walks every `Cargo.toml` and `.rs` under the root but `target/` and dot-directories (`tests.rs:17-38`).

| Test | Line | Rule |
|---|---|---|
| `no_kit_crate_depends_on_an_application_crate_even_when_empty` | `:210` (`breaches` `:169-181`) | no manifest depends on a crate starting with the first application's name |
| `core_and_children_use_only_the_kit_and_foundation_even_when_empty` | `:216-241` | `jig-core` → only `skein-lib` + `jig-core-*`; each `jig-core-*` → only `skein-lib` |
| `hosts_and_translation_use_no_core_crates_even_when_empty` | `:243-258` | `jig-host`, **`jig-inline-agent` (stale: crate dropped in 91f8bfec)**, `jig-charter` → no `jig-core*` |
| `host_hub_links_only_skein` | `:260-274` | `jig-host` → only `skein-lib` |
| `kit_sources_*_keep_the_kit_vocabulary` ×64 | `:276-614` | words of `forbidden_words` (`:187-203`: the first application's name, "forge", "forgejo", "repository/ies", "branch/es", "pull request", "git", "wiki") as whole words (`names.rs:114-140`: not preceded by an alphanumeric, not followed by a lowercase alphanumeric) in every `.rs` and in every crate `Cargo.toml` (the workspace manifest's dependency sources are stripped, `:129-155`); plus identifier rules on `.rs`: no name ending `V<digit>`/`_v<digit>`, no part `Typed`/`Untyped`/`Legacy`, no "legacy" in comments (`names.rs:9-37`, `:150-200`) |
| `client_view_links_only_its_domain_and_foundation` | `:708-725` | `jig-client-view` → `skein-lib`, `jig-client` |
| `client_domain_depends_on_foundation_alone` | `:727-741` | `jig-client` → `skein-lib` |
| `client_step` (helper) | `:743-755` | client step = `jig-client`, `jig-client-*` (not `-world`), test/ops client domains and views |
| `client_and_engine_step_crates_share_no_dependencies` | `:757-775` | no client step crate links `jig-core*`, no `jig-core*` links a client step crate |
| `client_transcript_is_the_client_only_smith_link` | `:791-817` | among client step crates only `jig-client-transcript` links smith, exactly `[jig-client, skein-lib, smith-transcript]` |
| `fake_person_…`, `scripted_engine_…`, `test_client_domain_…`, `test_connector_…`, `test_client_view_…` | `:819-894` | exact dependency lists of those kits; the two client roots `no_std` + `forbid(unsafe_code)` by source text |
| `client_world_joins_only_the_client_and_test_roles` | `:896-939` | allowlist of `jig-client-world`'s deps (a protocol world that reuses it must extend this list) |
| `ops_*` | `:941-977` | ops' client crates' exact deps |

Consequences for the plan:
- **Names in sources.** skein's generator is a CLI whose output is committed under `src/generated/v<N>.rs` (section 4.2), so jig's generated codecs are scanned like hand-written code. The generator writes doc comments naming each field (smith's `crates/smith-channel/src/generated/v2.rs:94`, "The `git` field of `Directory`", would breach jig's word list), so jig's schemas' record and field names must avoid the forbidden words; smith's generated v2/v3 files have no identifier ending `V<digit>`/`_v<digit>` (grep: 0 hits) and the module name `v1` passes `forbidden_name` (`names.rs:9-12` needs `V` or `_v` before the digits). Beyond that, a record named `CharterV2`, a fn `decode_v3`, or a comment naming "git" fails the gate. Wire-version constants must avoid `_v<digit>` identifiers (e.g. `CHANNEL_VERSION: u16 = 1`, not `CHANNEL_V1`). A new crate's `Cargo.toml` must take upstream deps by `.workspace = true` (a git URL in a member manifest contains "git" → breach).
- **Renames (README 15) break string-named rules:** `:221-223`, `:246`, `:263`, `:710`, `:731`, `:743-754`, `:777-788`, `:795-806`, `:822`, `:837`, `:851`, `:869-870`, `:885`, `:899-934`, `:944-967` all name crates as built; the rename session edits each, and `client_step`'s prefix test (`starts_with("jig-client-")`) would no longer match `jig-protocol-transcript`/`jig-view`/`jig-domain-client`.
- **Missing rules the design states** (none exists today): only `jig-charter` and the client's turn decoder link smith (`protocol/README.md:171-175`; jig `AGENTS.md:35-42`) — `jig-core*` and `jig-host` are covered (`:216`, `:260`), the client's step crates too (`:791`), but nothing stops a future `jig-protocol-*` (engine-workers, worker-link, tools, store, accounts, config) from linking smith; no protocol crate depends on an application crate other than the first application (`protocol/README.md:166-170`); client-side protocol crates (`jig-protocol-client-link`, `-client-wire`, `jig-shell-native`) link no `jig-core*`; engine-side ones link no client domain; `jig-protocol-engine-workers`/`-worker-link` link the hub's vocabulary (`jig-host`) and not smith (`protocol/README.md:435-441`).

### 5.4 .config/nextest.toml and scripts/check.sh

- `.config/nextest.toml:7-10` default profile: `default-filter = "not binary(/^fuzzy_/)"`, `global-timeout = "15s"`, `slow-timeout = { period = "500ms" }`; `:14-16` override `kind(test)` priority 10 (worlds first); `:18-21` `fuzzy`: `binary(/^fuzzy_/)`, `60s`, slow `5s`; `:26-28` `measure`: `1h`, slow `60s`, no budget. The 15 s budget is the whole workspace's default suite; every new protocol world's focused tests share it (workflow.md `docs/development/workflow.md:36-41`, `:68-73`: raising a budget is a decision, not a fix). A process-layer proof (`jig-test` binary + fake LLM peer, "about a minute", development.md 5.5) cannot sit in the default suite: it is a script in the run's `logs/` or, once needed twice, a script under `scripts/` (development.md 5.5, "A script needed twice is kept").
- `scripts/check.sh:7-13`: `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo nextest run --workspace`; `cargo nextest run --workspace --profile fuzzy`. No `cargo doc`, no wasm32 build, no per-crate target check, no process test. `scripts/` holds only `check.sh`.
- Gate rules: jig `AGENTS.md:3-8` (land only via skein's `land`; tests of touched crates through `heavy`), `:9-12` (15 s / 60 s budgets).

### 5.5 Files over 800 lines (size limit: development.md 5.2, "a file of at most 800 lines … unless an increment says otherwise")

Among crates the plan touches (the core's boundary and routing for store/stopping/grants/accounts/hand-out; the hub; the fleet; accounts; the charter; the client domain for signing out; the test application; the worlds a protocol world reuses):
- `crates/jig-core`: `src/tests.rs` 2936, `src/reads/rows.rs` 1744, `src/effects.rs` 1441, `src/routing/tasks.rs` 1414, `src/documents/task_tree.rs` 1018, `src/tests/acts.rs` 973, `src/documents/objects.rs` 933, `src/documents/inbox_deployment.rs` 830. (`src/routing/vocabulary.rs` 649, `src/routing/runs.rs` 784, `src/lib.rs` 707 are close.)
- `crates/jig-host/src/hosted.rs` 1340 (`boundary.rs` 675).
- `crates/jig-core-fleet`: `src/tests.rs` 1268, `src/attempt.rs` 1110.
- `crates/jig-core-tasks`: `src/boundary.rs` 1776, `src/domain.rs` 1251, `src/stored.rs` 1049, `src/tests/revisions.rs` 960, `src/proposals.rs` 874.
- `crates/jig-core-people`: `src/domain.rs` 1873, `src/tests.rs` 1592.
- `crates/jig-core-authority`: `src/tests_policy.rs` 1339, `src/check.rs` 1006.
- `crates/jig-client`: `src/tests/pages.rs` 2587, `src/domain.rs` 1238, `src/tests/conversations.rs` 1171, `src/contract/documents.rs` 1161, `src/objects.rs` 1132, `src/pages/routing.rs` 1111, `src/tests/confirmation.rs` 802.
- `crates/jig-client-view`: `src/phrases/jig.rs` 2994, `src/tests/cards_objects.rs` 1978, `src/phrases/english.rs` 1495, `src/tests/owner_project.rs` 1226, `src/pages/project/owner.rs` 1171, `src/owner_deployment_words.rs` 1062, `src/tests/turns.rs` 935, `src/tests/page_capacity.rs` 823 (a native shell renders through it).
- `testing/jig-test-domain`: `src/lib.rs` 1154, `src/routes.rs` 1005 (the reference root; hand-out of smith payloads and host tools lands here).
- `testing/jig-scripted-engine`: `src/tests.rs` 2295, `src/payload.rs` 1331.
- `testing/jig-test-client-domain`: `src/tests/root.rs` 1037, `src/first.rs` 1011; `testing/jig-test-client-view/src/tests.rs` 820.
- `testing/jig-conformance/src/referee.rs` 1088 (the protocol world's referee "is the conformance world's, seen through bytes", service.md 8).
- Worlds: `tests/client/src/joint/session.rs` 1553, `joint/packets.rs` 1104, `joint/wire/cursor.rs` 1036, `joint/wire/owner.rs` 881, `stories.rs` 878, `joint/wire/documents.rs` 865, `harness/world.rs` 840; `tests/core/tests/stories.rs` 1886, `src/effects.rs` 1289, `src/observations.rs` 1080, `src/world.rs` 810; `tests/host/src/world.rs` 1048; `tests/fleet/src/world.rs` 1320; `tests/boundary/src/tests.rs` 1009.
- An increment that adds to any of these either splits it first or says it exceeds the limit (development.md 5.2). tech-debt.md section 3/7 already names "files of 1,600 to 3,000 lines" as debt (`docs/design/STATUS.md:122-126`).

### 5.6 Fixtures modules (development.md 5.2: one per crate, `src/tests/fixtures.rs` or the world's own)

**No crate has `src/tests/fixtures.rs`.** What exists:
- `tests/host/src/fixtures.rs` (36 lines): event builders for the hub's world: `assign(ReplyTo, RunAssignment) -> Event` `:6`, `inbound(run, attempt, name, words) -> Event` `:10`, `called(owner, call, Ask) -> Event` `:14`, `withdrawn` `:18`, `finished(owner, Finish)` `:22`, `deliver(title) -> Ask` `:26`, `relay(input) -> Ask` `:30`, `callback(name) -> Token` `:34`. The worker-link/engine-workers worlds can reuse these for the hub's side.
- `tests/client/src/joint/fixture.rs` (139): `clock(now) -> Wall` `:11`, `turn() -> Box<[u8]>` `:15` (a whole committed turn's bytes), `world(body)` `:54`, `configured_world(body, fn(&mut Joint))` `:59`, `bounded_world` `:64`, `seeded_world` `:73`, `seeded_world_with_receiving` `:82` — the joint world's constructors; the client protocol world would start from these.
- `tests/ops/src/joint/fixture.rs` (272): ops' joint world constructors.
- `tests/client/src/harness/{world,faults,observation,referee}.rs`: the client world's harness (scripted engine).
- `testing/jig-conformance/src/harness.rs` (533): the conformance harness (referee, scenarios, trace).
- `tests/boundary/fixtures/names.txt`: the name-rule fixture.
- In-crate helpers inside `tests.rs` (no module of their own): `crates/jig-host/src/tests.rs:78` `assignment(run) -> RunAssignment`; `crates/jig-client-transcript/src/tests.rs:4-67` (`limits`, `text`, `call`, `result`, `message`, `source`, `encode` building smith turns) and goldens `crates/jig-client-transcript/tests/data/record_turn_{smallest,full}.bin` (copied from smith 2a800ea2 `crates/smith-transcript/golden/v3/`, README there); `crates/jig-charter/src/tests.rs` one test, no helpers.
- Each new protocol crate's first session creates `src/tests/fixtures.rs`; note `mod_module_files = "deny"` (`Cargo.toml:183`) means `src/tests.rs` + `src/tests/fixtures.rs` (not `src/tests/mod.rs`), as `crates/jig-host/src/tests.rs` + `src/tests/progress.rs` do.

## 6. The old draft plan

Paths without a prefix are under `/srv/data/git/runner/src/plans/jig-protocol-implementation/`. The prefix `jig:` means `/srv/data/git/runner/src/jig/`, at `main` 2fae240a. The plan was drafted on 2026-10-10 against jig `0679f66` and skein's and smith's `design/jig-standalone` branches. Since then: jig dropped its inline agent (91f8bfec). It repinned skein to 30dc6f7 and smith to 2a800ea2, which is smith's origin/main, through GitHub URLs (0e952778; jig:Cargo.toml:67-72). It revised the design to use only smith's agents (c135dc24), designed the store (9a9aed35), and added handed-in LLM accounts and excluded the browser (2fae240a). jig has also gained `scripts/check.sh`, and `land` now pushes (9af61c4c; jig:AGENTS.md:3-6).

### 6.1 README, code organization, decisions, sessions 01–11

#### README.md: the session graph, the rules, and done-when

**Stale.**
- **Home and run directory** (README.md:24-26): the plan now lives in `docs/development/plans/jig-protocol/` (jig:docs/design/STATUS.md:11).
- **Start commits and inputs** (README.md:4-8, 39-66):
  - skein's designs are on its main (f6c3e82, 72462f7, dec0075, 6d92015, 5c9eedd) but not built.
  - smith's `design/jig-standalone` is moot. The design no longer asks for the completion contract, text in flight, the channel's text, or the LLM close (jig:docs/design/protocol/README.md:344-359).
- **Session tables:**
  - The smith sessions 02, 03, 04 and 06 (README.md:103-107) are not needed for the first build. smith 01 and 05 (repin onto skein's `Addr` and joined sockets) remain, but only after skein builds them.
  - jig 02 is obsolete (README.md:114).
  - jig 10, 27 and 28 are out of scope: they cover agents in the engine (README.md:128, 145-146; jig:docs/design/STATUS.md:76-80).
  - jig 35 is excluded: no browser and no wasm32 (README.md:159; STATUS.md:103-108).
  - jig 34 is no longer a placeholder: store.md is designed and the store is in scope (README.md:158; STATUS.md:55).
- **Rules:**
  - The preconditions (Chromium, wasm32, wasm-bindgen) no longer apply (README.md:195-197).
  - `land --local` with an ad-hoc `--gate` is replaced by plain `land` with `scripts/check.sh` (README.md:214-220).
  - The repins by smith 01/05 and jig 02/11 (README.md:221-223) shrink to "skein builds, smith repins, then jig repins".
  - Of the six reference-service ports (README.md:232-243): the store port, the secrets port, the accounts-driver port and the scripted-agents port are stale. Keep only the parties port and the system port.
  - The "ops debts" list (README.md:250-255) still names the inline agent and `Content::Call`.
- **Done-when:**
  - "skein … build for wasm32" (README.md:301-302) is stale.
  - smith's line (README.md:303-307) is stale.
  - "browser's among them" (README.md:315-316) is stale.
  - "inline runs against fake LLM peer" (README.md:320-321) is stale.
  - "browser tier" (README.md:328-330) is stale.
  - The binary `jig-test` and its process layer are missing (jig:docs/design/protocol/service.md:244-256, 301-308).

**Keep.**
- The table convention of "Needs at start" and "Increments that need more", and dependencies stated by capability (README.md:82-86).
- jig 01 runs alone and first (README.md:206-208). Every debt is checked first (README.md:258-261).
- The shared hotspots list (README.md:265-271). It needs `tests/store` and `jig-test` added.
- "Whichever lands second keeps the other's" (README.md:272-276).
- The rule for budgets measured twice (README.md:277-286).
- Version owners (README.md:224-231). Drop smith 04 and jig 32's `STORE_VERSION` wording; add the store's format row.
- These chains survive as the spine: jig 01 → 07 → 22 → 23 / 24 → 25 → 29 → 26, and 30 → 31 / 32 → 33.

#### code-organization.md

**Crate layout against protocol/README.md §5 and service.md §7.**

Matches:
- The new `jig-protocol-*` crates and their kinds (code-organization.md:54-63). These are jig:docs/design/protocol/README.md:140-152.
- `jig-shell-native` (code-organization.md:64).
- `jig-test-service` as step code among the test kits (code-organization.md:67).
- The world packages `tests/channel` and `tests/client-protocol` (code-organization.md:71, 73). The design uses the same names (jig:docs/design/protocol/channel.md:600, client.md:390).

Stale:
- The `jig-inline-agent` row (code-organization.md:34): the crate is gone.
- `jig-shell-web`, `jig-test-web` and `jig-fake-person-browser` (code-organization.md:65, 69-70): excluded.
- "store as its design says" with "jig 34" (code-organization.md:63): the store is now designed and in scope.
- `jig-test-service` holding "smith's LLM component" and "ports" (code-organization.md:67, 117-122).
  - service.md §7 puts the LLM component only for `smith-inline-agent`, which is out of scope. The STATUS row excludes llm.md's in-process calls.
  - The design adds that the test worker composes smith's host half (`smith-host-protocol`) and agent processes (jig:docs/design/protocol/service.md:189-196, 234-243).
- `jig-test-worker` holding "scripted agents" answered through a port (code-organization.md:66). The design now has it hold smith's scripted agent processes and a scripted workspace (jig:docs/design/protocol/channel.md:603-606).

Missing:
- The reference binary `jig-test`, with its `agent` command run through `smith-agent-shell` (jig:docs/design/protocol/service.md:244-256; jig:docs/design/protocol/README.md:183).
- `tests/store` (jig:docs/design/protocol/store.md:577).
- The process layer: skein's fake LLM peer as a loopback process (service.md:301-308).

**Roles** (code-organization.md:83-124).
- Still fit: the codec crates depend only on skein-codec and lib. The channel's components depend on `jig-domain-host`, never the core. Only charter and transcript link smith. These match jig:docs/design/protocol/README.md:163-175.
- Stale: `jig-test-service` → `smith-protocol-llm` (code-organization.md:119-120). In its place it needs `smith-host-domain`, `smith-host-protocol` and `smith-protocol-channel` on the worker side. Under the hand-out design, the service's routing links smith (jig:docs/design/domain/hosts.md:521-527).

**Modules** (code-organization.md:128-185). Reusable as is:
- The component layout (`boundary`, `component`, `connection`, `translate`, `limits`).
- The codec crate layout (schema, generated, sizes, golden, with drift and fuzz in the world).
- The reference service's module tree (code-organization.md:166-178). Drop "the ports" from `boundary.rs`. Add a hand-out module for smith's payloads and the run's identity (jig:docs/design/protocol/channel.md, §7: SHA-256 of the deployment id, the task and its first attempt).

**Names and ports** (code-organization.md:195-205).
- Stale: the store, secrets, agents and driver port pairs.
- Still fits: `system_*` and `parties_*`.
- Fits: `lib::Secret` everywhere (code-organization.md:206-212). It is designed in skein but not built (skein STATUS: lib.md 6 not built).

**Owners table** (code-organization.md:216-255).
- Stale rows:
  - 225-228: smith 02, 03, 04 and 06.
  - 242: the inline agent's text.
  - 250: the test root's inline agent.
  - 222: wasm32.
  - 252-255: the store's port, now replaced by `jig_protocol_store` owning its boundary.
- Reusable: the jig 03–09 and jig 20–33 rows.

#### decisions.md

**Still fits:**
- §1 scope for jig (decisions.md:37-43). Drop "but the store".
- §5 renames (decisions.md:81-96).
- §6 (decisions.md:98-103).
- §7 Addr in two steps (decisions.md:105-111). This is skein's own business.
- §13 new formats at v1 and codecs from schemas (decisions.md:227-235). Drop smith 04 at decisions.md:222-226.
- §14 step code versus ordinary Rust (decisions.md:237-245). Drop the web crates.
- §15 host tools outside the root (decisions.md:247-257).
- §18 the printed link (decisions.md:285-291).
- §19 grants naming their table (decisions.md:293-298). This matches jig:docs/design/domain/engine.md:794-816.

**Stale:**
- §2 (decisions.md:48-52): the home.
- §3 (decisions.md:59-67): `land --local`, and jig having no gate.
- §4 (decisions.md:71-74): the client plan has closed.
- §8 (decisions.md:113-131): two skein revisions through smith 01/05.
  - jig is already on smith's main.
  - The remaining skein builds (joined sockets, `read_private_file`, `Secret`, `CryptoRng`, `OpenWrite` and `Cut`) need skein sessions, then a smith repin, then a jig repin.
- §9 (decisions.md:133-170): a port for the store and secrets, `jig-fake-store`, a driver port, and "No binary is built". All are stale.
- §10 (decisions.md:172-179): text asked on every run. Agent processes give no text (jig:docs/design/domain/hosts.md:444-450).
- §11 (decisions.md:181-187): no live services. It still fits, but a handed-in account now enables a read-only live check (jig:docs/design/protocol/llm.md:229-239).
- §12 budgets (decisions.md:189-216): re-measure. The gate at 0e952778 recorded 2173 tests in 9.0 s and 86 tests in 12.0 s.
- §16 (decisions.md:259-277): the test worker. The worker's layer now composes smith's host half and real or scripted agent processes.
- §17 (decisions.md:279-283): first records by file. It is superseded by handed-in accounts (llm.md 5.4).
- §20 (decisions.md:300-312): the browser's client.
- "For the user" 1-4 (decisions.md:316-339) are all resolved or moot. Item 6, the ops CLI proof (decisions.md:349-357), is still "not planned".

#### later.md

It still fits as a list. Items that changed:
- Row 6 ("jig's store, durable"; later.md:13) and row 7 (later.md:14) are now in scope through store.md.
- "Signing in through the client" stays later.
- "API keys" (later.md:20) is answered by handed-in accounts (llm.md 5.4).
- "Fairness among inline runs" (later.md:21) is moot.
- The ops-standalone row (later.md:10) still fits. The drafts in `findings/ops-drafts/40-44*.md` are kept for that plan.

#### findings/

**code-debts.md**, checked again on jig main:

Paid:
- 1, 3 and 9 (paid then).
- 4, chats forgotten. It appears built: `core.people.chat_seen`/`results_seen`, then `ForgetChat` with the ended row erased (jig:crates/jig-core/src/inbox.rs:497-515).
- The people timers in 15. `Timer::People` exists and fires (jig:crates/jig-core/src/routing/vocabulary.rs:636-638; jig:crates/jig-core/src/routing/mod.rs:519).
- The bounced half of "Also found". A bounce stays unread (jig:crates/jig-core/src/routing/children.rs:284-286, from 91f8bfec).

Obsolete:
- 13, the inline completion contract.
- The part of 6 about the inline agent's content. ops now takes `Event::AgentContent` and computes the turn and offset itself (jig:examples/ops/domain/src/route.rs:200, 1533-1538).

Present:
- 2: escalations refused as `Unknown` (jig:crates/jig-core/src/routing/party_decisions.rs:331-347).
- 5: no stop. `views::End::Stopping` exists (jig:crates/jig-core-views/src/boundary.rs:33), but nothing makes it.
- 7: one charter (jig:crates/jig-core/src/lib.rs:107-127, `Settings.charter`/`run`/`account`).
- 8: a gap is ignored (jig:crates/jig-core-views/src/text.rs:114). `TurnName` already has `retry` (jig:crates/jig-core-views/src/boundary.rs:48-52).
- 10: `named_propose` lacks the `unproposable` check (jig:crates/jig-core/src/routing/runs.rs:262-290; jig:crates/jig-core/src/acts/bounds.rs:384).
- 11: ops decodes host tools in its root (jig:examples/ops/domain/src/translate.rs:878).
- 12: grants are never pushed. `fleet::Request::{Grant,Rejected,Exhausted,Told}` reach `unreachable!` (jig:crates/jig-core/src/routing/children.rs:287-290).
- 14: `signing_in` (jig:crates/jig-core/src/routing/people.rs:80, 131, 169; jig:crates/jig-core/src/lib.rs:267, 636).
- `decode_result` in the ops root (jig:examples/ops/domain/src/assemble.rs:260).
- `jig-core` is not an explicit member (jig:Cargo.toml:4-50 has no `crates/jig-core`).

**client-boundary.md** (a pre-jig-45 survey).
- Item 5 is paid: `people::fire` is now reached through `Timer::People`.
- Still open: item 4 (`signing_in`), item 6 (no signed-out read; jig:crates/jig-core/src/reads.rs:150), item 7 (no stop), item 11 (the separate `Read::Turn`; jig:crates/jig-core/src/reads.rs:56) and item 20 (no sign-out act).
- Its other items were settled or translated by the joint world, which has now landed (45.3 and 45.4 commits). jig 30 should re-survey against `tests/client/src/joint/wire/*` rather than trust this file.

#### Sessions 01–11

- **01-renames: not landed. It fits, minus one row.**
  - All crates keep their as-built names (jig:Cargo.toml:65-109; the `name =` lines in each manifest). Its table (01-renames.md:123-137) matches jig:docs/design/README.md §15 exactly, except the `jig-inline-agent` row (01-renames.md:128; also 01-renames.md:29-30, 56-57, 67, 74, 80-81), which is gone.
  - 19 crate renames remain (the 12 rows of README 15's table):
    - `jig-core` → `jig-domain-core`;
    - the eight `jig-core-*` → `jig-domain-*`;
    - `jig-host` → `jig-domain-host`;
    - `jig-client` → `jig-domain-client`;
    - `jig-client-view` → `jig-view`;
    - `jig-charter` → `jig-protocol-charter`;
    - `jig-client-transcript` → `jig-protocol-transcript`;
    - `jig-test-client-domain` → `jig-test-domain-client`;
    - `jig-test-client-view` → `jig-test-view`;
    - `jig-ops-client-domain` → `jig-ops-domain-client`;
    - `jig-ops-client-view` → `jig-ops-view`;
    - `jig-ops-connectors` → `jig-ops-domain-connectors`.
  - The `jig-core` membership debt also remains.
  - Reuse: the three-group split (01-renames.md:62-119), the grep check (01-renames.md:146-151; drop `inline[-_]agent`), and the boundary-role note (01-renames.md:55-58).
  - `jig-ops-fake-production` and `jig-test-client-kit` keep their names.
- **02-repin-smith: obsolete.**
  - 0e952778 already pins smith at 2a800ea2 (origin/main) through GitHub URLs, with charter v2 and transcript v3.
  - Everything it adapts no longer exists or is no longer asked: the inline agent (02-repin-smith.md:30-31, 92-95), the text option and contract (02-repin-smith.md:39-46), and smith 02/03/04/06.
  - Nothing reusable but the rule "jig's lock names smith's skein" (02-repin-smith.md:48-49).
  - jig's lock has skein 30dc6f7. smith's lock must agree when skein's new work lands.
- **03-people-and-tasks: not landed. It fits.** STATUS still lists people.md 3, 5.1 and 6 and tasks.md 9 (jig:docs/design/STATUS.md:33, 38).
  - Partly paid: 03.3's timer (`Timer::People` exists and fires, at the paths above). `Now::SignInEnded` is still missing; nothing named so is in jig-core. 03.4's forgetting looks built (inbox.rs:497-515); verify it, then land the missing test.
  - Still needed: 03.1 (party_decisions.rs:331-347), 03.2 (people.rs:80/131/169), 03.5 (runs.rs:262-290) and 03.6. The core has no `next_deadline`, only `effect_deadline` (jig:crates/jig-core/src/lib.rs:354-356). service.md §4 needs it for `next_policy_deadline`.
  - Reuse: the sketches as is (03-people-and-tasks.md:223-253).
- **04-stopping: not landed. It fits.**
  - Nothing exists in code for `Event::Stopping`, `Held::Drained` or `workers_in_process` (grep of jig:crates/jig-core/src finds none). This is engine.md 6.1 and §6 step 3 (jig:docs/design/domain/engine.md:355-365, 382-402).
  - It needs only 07.1's `Relayed::Busy`.
  - Stale: the mentions of jig 29/26/31 as users (04-stopping.md:190) and `jig-protocol-config` (04-stopping.md:114-115). These are fine as forward references.
  - Reuse: the sketch and the input/answer table (04-stopping.md:187-237) whole.
- **05-charters: not landed. Still needed, but incomplete for the revised design.**
  - The code still has one charter (jig:crates/jig-core/src/lib.rs:112-127).
  - Reuse: 05.1-05.3 and the sketch (05-charters.md:164-217).
  - Missing, from engine.md 7.1/7.2 and STATUS.md:36:
    - the start's messages;
    - the conversation's first attempt for the run's identity;
    - windows, and a resume from the current window;
    - the reserve with one most costly completion before a run is placed (authority.md 7; STATUS.md:34).
  - Charter v2's window, output and reserve are already in the core's `RunPolicy` (0e952778).
- **06-accounts-and-grants: not landed. It fits, with additions.**
  - The accounts child still refreshes on its own: `Request::Refresh`, `Event::Refreshed`, `Refreshing` (jig:crates/jig-core-accounts/src/boundary.rs:19, 60, 73).
  - Grants are unrouted (children.rs:287-290).
  - Reuse: the vocabulary sketch (06-accounts-and-grants.md:157-197), and 06.2 whole.
  - Add:
    - an `Expiring` event for a handed-in account (jig:docs/design/protocol/llm.md:188, 229-239). skein's driver has `HandIn`/`Expiring` (skein:crates/skein-oauth-accounts/src/boundary.rs:18, 44).
    - an application's credentials kept, granted and resent like accounts (engine.md 12.1, jig:docs/design/domain/engine.md:794-816). 06 has only the `GrantTable` naming.
- **07-hub-link-and-run-text: not landed. Mostly fits, but needs rework.**
  - The hub still has `Report`, `Unacknowledged` and `Facts` and no `Hello` (jig:crates/jig-host/src/boundary.rs:43, 91, 96, 227).
  - Reuse:
    - 07.1 (07-hub-link-and-run-text.md:107-134), with its hello, answer kept and `Relayed { Answer | Busy }`.
    - The grant with its table (07-hub-link-and-run-text.md:271-274).
    - The notices in 07.2.
    - 07.4, the views by retry. `TurnName.retry` already exists, so only the gap/cut work remains.
  - Stale:
    - 07.2's `Activity::{ToolBegun{call,tool,input}, CheckBegun/Ended}` taken from smith's typed facts (07-hub-link-and-run-text.md:300-312). Activity is now content-free: phase, long operations with span, and counts decoded from smith's channel facts by hand-out (hosts.md 6.5, jig:docs/design/domain/hosts.md:434-443).
    - 07.5's ops `Content::Call` / `inline_report` paths (07-hub-link-and-run-text.md:220-226, 254-256).
  - Missing from 07:
    - the hello's agent bounds, which the fleet checks (hosts.md §2, jig:docs/design/domain/hosts.md:83-91; engine.md §8);
    - refusals riding with turns, and the unread fence (hosts.md 6.4, §2);
    - the window flag on turns;
    - smith's payloads by hand-out (hosts.md 6.8): charter, transcript and call names decoded down; turns, call names, results and facts encoded or decoded up.
  - That missing work is new domain work that the hub and both roots (ops, jig-test-domain) need before jig 22.
- **08-client-owed: not landed. It fits.**
  - `reads.rs` has no `SignedOut` (`refused_application`, jig:crates/jig-core/src/reads.rs:150).
  - The separate `Read::Turn` still exists (jig:crates/jig-core/src/reads.rs:56).
  - There is no client `SignOut`.
  - Reuse: 08.1-08.3 and the sketch (08-client-owed.md:142-170).
  - 08.2 touches `jig-view`'s frame and the fake person. This is native-only now; drop any browser mention.
- **09-host-tools: not landed. It fits the design** (channel.md 6.1, 6.3 and §7; STATUS root.md 11-12 and hosts.md 8).
  - `jig-charter` exposes only `charter` and `encode` (jig:crates/jig-charter/src/charter.rs:86; jig:crates/jig-charter/src/encode.rs:172). It has no result reader and no call key.
  - ops decodes results in its root (jig:examples/ops/domain/src/assemble.rs:260-263, through `smith_charter::RunResult::decode` with `v2::CEILINGS`) and decodes tools there too (jig:examples/ops/domain/src/translate.rs:878).
  - Reuse:
    - the `jig_protocol_tools` sketch (`Declaration`, `decode`, `Decoded::{Action,Delegate,Problem,NotMine}`, `render`; 09-host-tools.md:239-259);
    - `Event::Undecodable` (09-host-tools.md:265-271);
    - `Now::Result`/`ResultRead`/`ResultUnread` (09-host-tools.md:279-290).
  - Change 09.3's `RunResult` sketch (09-host-tools.md:275-276) to smith v2's result form.
  - Add the run's identity derivation (channel.md §7). It is engine protocol-layer work, and could go here or in jig 25.
- **10-inline-agent: obsolete.**
  - `jig-inline-agent` was deleted in 91f8bfec. Agents in the engine are out of scope (STATUS.md:76-80).
  - Nothing to reuse. The idea of mapping smith's facts to the hub's activity (10-inline-agent.md:212-220) moves to the worker's hand-out routing (channel.md 6.4), in a different form: decode smith's channel facts.
- **11-repin-skein: not landed. Still needed, in a new form.**
  - jig's lock has skein 30dc6f7. skein's origin/main 5c9eedd adds only designs since then: joined sockets, in-process addresses, `read_private_file`, base64, `Secret`, `CryptoRng`, `OpenWrite`/`Cut`. None of these are built (skein:docs/design/STATUS.md:28, 30, 32).
  - So a jig repin follows skein's build sessions, and smith's repin onto them. It is one repin of all of skein's new work, not "the second revision".
  - Reuse: 11.2 (11-repin-skein.md:70-90), in-process names in `jig-protocol-config` setting `workers_in_process`.
  - Stale: "the cryptographic generator came with the first, through jig 02" (11-repin-skein.md:5-6).

### 6.2 Sessions 20–35

Paths below are relative to `/srv/data/git/runner/src/plans/jig-protocol-implementation/` (the old plan), jig's `docs/design/` (design files named bare, e.g. `channel.md`), or the named repository.

**Stale in every session 20–35 (say once, fix everywhere):**
- **"paths as after jig 01's renames"**: the renames have not landed. jig's `README.md:633-654` still lists them as owed, and `STATUS.md:11` puts README 15 in jig-protocol's scope.
- **"jig 02" as the source of skein's first revision** (`Addr`, `lib::Secret`/`secret N`, `lib::base64`, `CryptoRng`):
  - jig is already repinned at GitHub's `main` (jig `0e952778`; `Cargo.toml` git URLs name `github.com/forgetic`).
  - None of those skein pieces is built. skein's `main` has only their designs (`f6c3e82`, `72462f7`, `dec0075`, `6d92015`, `5c9eedd`). `grep` finds no `InProcess`, `read_private_file`, `struct Secret`, `CryptoRng` or `OpenWrite` in `skein/crates`, and `pub mod base64` exists only in `testing/skein-browser/src/wire.rs`.
  - So every "needs jig 02" in 20–33 really needs a skein build plus a repin.
- **"jig 07's link vocabulary"** (`host::FromHub`/`ToHub` placeholders in 23, 24, 25):
  - It is still owed (`protocol/README.md:375-378`).
  - The hub today still bounces messages: `crates/jig-host/src/boundary.rs:118,219,338,545` (`Bounced`, `Bounce`). The design replaces bounces with refusals carried with turns and answers (`channel.md:220-229`).
- **"jig 11" for in-process addresses** (23, 24, 26) waits on skein's joined sockets, which are designed but not built.
- **Landing** with `land --local` / `land --gate` (`README.md:214-220`; `decisions.md:57-67`) is stale:
  - jig `AGENTS.md:3-5` says `land` pushes, with `scripts/check.sh` as the gate.
  - smith has a gate script (smith `564d4408`).
- **The reference service's ports** for the store, the secrets and the accounts driver (`decisions.md:133-170`; `code-organization.md:195-205`) are stale. See 25, 28, 29 and 34.
- **`code-organization.md`:**
  - Still usable: the component layout (`:130-145`), the codec-crate layout (`:147-160`), the reference service's layout (`:166-178`), the port and secret-value naming (`:189-212`) and the crate-graph roles (`:88-124`).
  - Stale:
    - `jig-test-service` depending on `smith-protocol-llm` (`:117-122`);
    - `jig-domain-inline-agent` (`:34`);
    - the rows for `jig-shell-web`, `jig-test-web` and `jig-fake-person-browser` (`:65,69-70`);
    - the browser tier (`:271-272`);
    - the interface owners tied to smith 02/03/04/06 and jig 10/27/28 (`:225-228,242,250`).

**20-config: still fits** (`configuration.md` has the same shape).
- **Stale:**
  - 20.2's `hosts` member carries "the engine's own slots; the inline agent's limits" (`20-config.md:155-156`). Agents in the engine are not planned (`STATUS.md:76-80`), so the parser only refuses engine slots.
  - `llm` accounts (`:153-157`) lack the handed-in account's private file (`configuration.md:73,154`; `llm.md:229-238`).
  - Models give "most output tokens, prices" (`:153`). They must give window, output, and input/cached/output/cache-write prices (`configuration.md:73`).
  - Charters lack the reserve and the budget check, which needs a reserve plus one most costly completion (`configuration.md:71,134-137`).
  - `deployment` (`:114-115`) lacks the retention horizons and the store, transcript and secrets directory (`configuration.md:66`).
  - The worker's `agent` (`:195-196`) is now smith's agent configuration, whose models' windows the hello states (`configuration.md:99`).
  - "jig 10's start" (`:197`) is dead.
  - `secret()` returns `skein_lib::Secret` (`:254`), which is not built.
- **Reuse:**
  - the choices at `:77-105`: parsing over skein-json `Document` with paths, bounded `Problems`, files parsed but never read, components' limits as plain structs;
  - the whole sketch at `:212-255` (`Engine`, `Worker`, `Profile`, `Address`, `Problem`/`What`, `fits`, `stop_bound`, `digest`, `secret`);
  - the test lists at `:140-145,174-182,204-206`.

**21-accounts-component: fits, with one reversal.**
- **Stale:**
  - The reversal: `Expiring` is asserted unreachable because "no account is handed in" (`21-accounts-component.md:44,61-62`). Handed-in accounts now exist:
    - the driver's *expiring* maps to the child's *expiring* (`llm.md:188`);
    - `jig-protocol-accounts` reads the handed-in file at start and hands it in (`llm.md:232-234`);
    - skein already has `Request::HandIn` and `Expiring` (skein `crates/skein-oauth-accounts/src/boundary.rs:18` `HandIn`, `:44` `Expiring`, `:85` the `HandIn` request kind in refusals; commit `4eb3634`).
  - "Fill smith's LLM component's grants (jig 28)" (`:9,160-161`): the table now feeds only the workers' grants.
  - The child's `ToComponent`/`FromComponent` "as jig 06 lands it" (`:130-143`): `jig-core-accounts` still speaks `Refresh`/`Keep`/`Unsaved`/`Availability` (`crates/jig-core-accounts/src/boundary.rs:12-68`), so the domain's move onto skein's driver (`protocol/README.md:372-374`) comes first.
  - `Secret` (`:70-77`) is not built in skein.
- **Reuse:**
  - the translation table (`:33-49`) and the refusals by name (`:63-69`);
  - the `Up` enum and the `Table` API (`:128-157`);
  - the tests at `:98-102,117-122`, including the search of `Debug` output for a value's bytes.
- **Add:** an increment for reading and handing in the handed-in file.

**22-worker-wire: the crate fits; the records changed.**
- **Stale:**
  - "Sixteen kinds" and `bounced` (`22-worker-wire.md:4,35,162-170`). The design has 7 up kinds (hello, turn, call, answer, fact, rejected, exhausted) and 8 down kinds (`channel.md:203-242`), with refusals riding on turns and answers (`:220-229`).
  - Activity as "a tool begun with its call and input; checks begun and ended" and "a run's text" (`:42-47,75-81,127,179`). It is now:
    - a phase;
    - long operations (checks, an LLM completion, a command) with their spans;
    - counts from smith's channel facts.

    Text and "tool begun" come only from hosts that give them (`channel.md:209,357-373`).
  - The hello gains the agent's bounds (models with window and output; turns, spend, time, waiting; checks' timeout; message bound and inbox; turns kept) (`channel.md:205`).
  - The turn gains the last message read, refusals and opens-a-window (`:206`).
  - The answer gains the unread fence and refusals (`:208`).
  - The assignment gains the 16-byte run identity, the current window, calls committed since the last turn, and the start's messages (`:235`).
  - "retry and offset as smith 03 reports" (`:73`): smith 03 is not on smith's `main`.
  - The shape to copy (`:25-27`) is now smith's v2:
    - `crates/smith-channel/schema/channel-v2.schema`;
    - `kinds.rs` with `V2` at `:55`, `schema` at `:78`, and `peer_terms_gap(schema, role, version, terms)` at `:116`;
    - drift at `tests/codecs/tests/drift.rs`.
  - `Sizes` (`:176-184`) lacks the models a hello may declare and the refusals per record (`channel.md:566-569`).
  - `secret N` needs skein's generator, which is not built.
- **Reuse:**
  - magic `jigw`, the kind ranges and `Reason` 256–261 (`:65-70`);
  - the widths (`:71-74`);
  - `kinds.rs`/`terms_gap`;
  - the `Sizes`, `Cap`, `largest_body`, `engine_cap` and `worker_cap` API (`:176-189`);
  - the tests (`:122-130,150-152`);
  - the `tests/channel` drift and fuzz layout (`:112-121`).

**23-engine-workers: fits well** (`channel.md` 3, 4, 9 and 10.2 are unchanged in substance).
- **Stale:**
  - `NotAdmitted` (`23-engine-workers.md:211`) lacks "the agent's bounds cannot run the engine's charters" (`channel.md:185-188`).
  - The fleet's `Request::Refuse { channel }` still has no reason (`crates/jig-core-fleet/src/boundary.rs:387-389`), so 23.2's change (`:76-79`) stands.
  - In-process "with jig 11" (`:67`) and `Secret` (`:51-52,69-75`) wait on skein.
- **Reuse:**
  - the whole boundary sketch (`:186-234`);
  - the choices:
    - `net.rs` over `skein_world::stream::Wire` with `skein-fake-channel` (`:82-85`; both exist: skein `testing/skein-world/src/stream.rs:16`, `testing/skein-fake-channel`);
    - channel tokens from 1 (`:80-81`);
    - the constant-time compare (`:86-87`);
    - *framing* refusals (`:88-90`);
  - increments 23.1–23.3 and their tests, nearly as written.

**24-worker-link: fits** (`channel.md` 5.2, 10.1 and 10.3 are unchanged).
- **Stale:** only the shared items above. The hello now carries the agent's bounds, which the worker's root supplies (`channel.md:177-181`). The worker's table feeding smith's host half (`24-worker-link.md:101-103`) matches `channel.md:435-442`.
- **Reuse:**
  - the choices (`:77-106`): every refusal is a loss; seeded jitter; the busy-turn table; a busy call is its relay's answer; relays resent after the hello; the worker's own table;
  - the sketch (`:208-249`);
  - increments 24.1–24.3 and their tests.

**25-channel-world: the spine fits; the worker half and the ports are stale.**
- **Stale:**
  - The store's port answered by `jig-fake-store` (`25-channel-world.md:70-72,111-116,279-283,470-472`). It is now `jig-protocol-store`, with the real store over skein-kv's in-memory mode in the worlds (`store.md:563-576`; `service.md:292-293`).
  - The driver's port played by the worlds "until jig 28" (`:117-118,475-476`). skein's driver is a child of the engine's layer (`service.md:48-49`), and handed-in accounts give the worlds a file (`llm.md:229-238`).
  - The agents' port answered in the hub's own vocabulary by a `jig-test-worker` kit moved from `tests/host/src/parent.rs` (`:98-103,124-128,149-153,161-181,481-483,525-534`). Now:
    - the worker's root composes `jig-host` with smith's host domain;
    - its protocol layer holds `smith-host-protocol` and agent processes, each in its own process group (`service.md:189-198,202-209`);
    - the channel world runs `testing/jig-test-worker`'s smith scripted agent processes (`channel.md:600-607`), so the agents' "port" becomes io (process spawn, pipes);
    - the hand-out belongs to the worker's routing: charters decoded against the agent configuration's endpoints, turns and results encoded, facts decoded into activity (`service.md:210-213`; `channel.md:274-315`).
  - Scripted agents "write text by turn, retry and offset" (`:152-153`): agent processes give none.
  - "The charter encoded with jig-protocol-charter" in the test root (`:189`) is now handed out to the protocol layer (`protocol/README.md:34-40`).
  - The refusal of engine slots (`:215-216`) stays, but its reason is now "not planned", not "until jig 27/28".
  - 25.11 (`:430-451`) needs a result reader and a call's key that `jig-charter` lacks (`crates/jig-charter/src/` has only `encode` at `encode.rs:172` and `charter` at `charter.rs:86`).
- **Reuse:**
  - the layer/`Service` split and routing by token (`:84-97,487-523`);
  - the two startup calls, `named` and `startup` (`:104-110,456-466`);
  - the system and parties' ports (`:116-123`);
  - the flow of values, minus the driver (`:129-137`);
  - host tools and results in the routing (`:138-144`, 25.11);
  - one `link.rs` translation (`:145-148,536-539`);
  - increments 25.3–25.6: startup, `iterate`, timers, a failed commit, and `tests/service` on skein's `World::spawn_restartable` (`testing/skein-world/src/world.rs:267`);
  - the referee rules and stories (`:390-409`).

**26-one-process: fits the design's first shape** (`service.md:320-359`).
- **Stale:**
  - The fake store, the secrets' table and the driver's port played until jig 28 (`26-one-process.md:80-87,157-158`).
  - `jig-test-worker`'s scripted agents behind the agents' port (`:32-33,83-86`). It should be smith's scripted agent processes on skein's fake machine (`service.md:295-296,354-356`).
  - "Reads its text by turn, retry and offset" (`:146-147`): no text comes from agent processes (`client.md:231-232`).
  - The session needs skein's joined sockets (`:75,111`), which are not built.
  - It misses the process layer: `jig-test` on a scratch directory, skein's fake LLM peer as a loopback process, smith's real agent processes, killed and restarted (`service.md:244-252,301-308`). The old "No binary is built" (`decisions.md:149`) is overturned.
- **Reuse:**
  - the composition (`:73-79`);
  - drain the engine, then abort the workers (`:92-102`);
  - the `shutdown`, `end` and `Worker::abort` sketch (`:186-205`);
  - the stories (`:124-129,166-178`).

**27-test-root-agents: DROPPED.**
- The engine's own hub with `jig-domain-inline-agent` (`27-test-root-agents.md:3-15,106-117,172-197`): jig's inline agent was removed (jig `91f8bfec`), and agents in the engine are `smith-inline-agent`'s and not planned (`STATUS.md:76-80`).
- **Reuse:** only the note that `link.rs` serves the engine's own hub too (`:72-77`), for a later shape-2 plan.

**28-service-llm: SPLIT. 28.2 and 28.3 are dropped; 28.1 survives.**
- **Dropped:** completions and inline text (`28-service-llm.md:150-195`), since `llm.md` 1–4 are not planned (`STATUS.md:53`).
- **Survives:** 28.1, the accounts driver as the service's child (`:114-148`), is now jig-protocol's (`service.md:48-49`; `llm.md` 5).
- **Stale in 28.1:**
  - The driver's 32-byte `CryptoRng` seed (`:108-110,221`). skein's driver takes `seed: u64` today (skein `crates/skein-oauth-accounts/src/component.rs:124`).
  - The `llm.grant(...)` step (`:207`) goes.
  - "A revoked account's record is dropped" (`:138-139`) needs skein 02, which is not built (see below).
- **Reuse:** "the driver replaces the port whole" (`:82-86`), `route_accounts` and `values` (`:204-212`), `Roots { accounts }` (`:220`), and the stories at `:133-143`.
- **Add:** handed-in stories (`llm.md` 5.4).

**29-service-clients: fits, apart from its ports and browser framing.**
- **Stale:**
  - The secrets' port with an in-memory table (`29-service-clients.md:6,80-83,110-112,203-205`). It is now the store's secrets' records (`store.md` 9; `client.md:251-258`).
  - The fake store (`:160`).
  - Closing smith's LLM component in the drain, and every jig 28 reference (`:91-94,177-178,192-193,220-221`).
  - Run text "by turn, retry and offset through bytes" (`:123-125`).
  - `Seeds::secrets` for `CryptoRng` (`:84-86,228`), which is not built.
  - "A browser reaches the route" (`:7,122-123`) is now the native shell.
- **Reuse:**
  - the clients' component taking the parties' port's place (`:69-72`);
  - the engine end in `family.rs` (`:73-76,230-232`);
  - the `links` queue (`:77-79,207`);
  - the drain's order (`:87-97`) and the `End` enum (`:219-227`);
  - scripted parties as world code (`:98-101`);
  - the stories (`:119-128,145-150,166-170,187-194`).

**30-client-wire: fits** (`client.md` 3, 9 and 10).
- **Stale:**
  - `lib::base64` "through jig 02" (`30-client-wire.md:101-103,175-176`) is not built in skein.
  - The retry width "jig 07 reports" (`:94-95,209`): the contract keeps run text (7.3), but nothing in jig-protocol produces it.
- The map it follows exists: `tests/client/src/joint/wire/` (`asks.rs`, `answers.rs`, `reads.rs`, `watches.rs`, `cursor.rs`, `documents.rs`, `turns.rs`, `family.rs`, …).
- **Reuse:**
  - every choice (`:79-108`): one family; `Keyed`/`Owned`; a final answer names its owner; `u16` versions; no range in the frame; one base64 `StreamEvent`;
  - the schema sketch (`:202-214`);
  - increments 30.1–30.4.

**31-engine-clients: fits, minus the browser parts and the port.**
- **Stale:**
  - The secrets' port (`31-engine-clients.md:10-11,94-97,275,283,288-290`), which becomes the store's secrets' records (`store.md:449-484`).
  - 31.4, the client's files under a wasm content security policy (`:75-79,235-248,323`): it serves the browser only, which is excluded (`STATUS.md:103-108`).
  - Cookie attributes and the "Secure on loopback" question (`:57-60,152,307-309`) are browser-only. The native bearer (`:106-107`) becomes the main path.
  - `CryptoRng` (`:98-102,294`) is not built.
  - Run text with its call (`:207-208,269-270`): the record stays, but nothing produces it.
- **Reuse:**
  - `Settings` as plain values (`:83-90`);
  - the digest by `skein_io::digest::digest` (exists: skein `crates/skein-io/src/digest.rs:153`), compared in constant time (`:103-106`);
  - the cursor (`:108-110`);
  - watches admitted at the end of the step (`:111-114`);
  - the sign-in route (`:115-121`);
  - the boundary sketch (`:254-297`);
  - increments 31.1–31.3.

**32-client-link: fits** (`client.md` 2, 5–7 and 10).
- **Stale:**
  - 32.4, wasm32 (`32-client-link.md:3-4,14-15,69,197-214,279`; `decisions.md:300-312`), is excluded. `client.md:85-86` still says "built for the host and for wasm32".
  - base64 through jig 02 (`:85-87`).
- **Reuse:**
  - identifiers of its own (`:77-80`);
  - the reader inside the component (`:81-87`);
  - the out-of-date event and the frame's range removed (`:90-96,149-153,172-174`);
  - the small store's writes leading each step (`:101-104`);
  - the boundary sketch (`:219-257`);
  - increments 32.1–32.3.

**33-native-shell: fits** (`client.md:362-365,390-397`).
- **Stale:**
  - The world answers the secrets' port with an in-memory table (`33-native-shell.md:69-70`).
  - "`jig-test-web` (jig 35) uses it" (`:86`).
  - A native client signing in by the printed link "as a browser would… the cookie's secret sent after as a bearer" (`:88-91`). The design does not say how a native client takes the deployment's link: `client.md:266-269` names a device's flow or a browser; the link is `:270-279`. The plan must settle it, or ask.
  - Uses skein-io `OpenPrivate`/`Load`/`Store` (`:35-36,102-105`); `OpenPrivate` exists (skein `crates/skein-io/src/file.rs:40`).
- **Reuse:**
  - the shell's machines apart from io (`:61-65`);
  - the world as a `Front` of the `tests/client` harness (`:74-77`);
  - the `jig-test-protocol-client-wire` crate (`:78-87`);
  - the sketches (`:162-178,183-189`);
  - the stories (`:145-153`).

**34-store: REPLACED.**
- It was a placeholder (`34-store.md:3-6`) for a design that has since landed (jig `9a9aed35`; `store.md`, 639 lines). It must be planned in full:
  - skein-kv with two stores (`store.md:13-22`);
  - keys and tags (4); codecs and the format row (5); commits (6);
  - loads, parties' reads and indexes (7);
  - transcript files synced before their commit (8);
  - the secrets' store (9); opening (10); retention and fill (11);
  - the application's part (12);
  - the real store in every world, which retires `jig-fake-store` (`store.md:563-576`);
  - `tests/store` (`:577-583`);
  - the domain's owed items (`:595-616`).
- Its dependency is skein's `OpenWrite` and `Cut` (`store.md:617-619`), which are designed only (skein `5c9eedd`).
- **Reuse:** only `:20-28` ("what it replaces"), which a new plan never builds.

**35-web-shell: DROPPED** by the user's exclusion (`STATUS.md:103-108`).
- It takes with it:
  - `jig-shell-web`, `jig-test-web` and `jig-fake-person-browser` (`code-organization.md:65,69-70`);
  - the browser tier and its profile;
  - wasm-bindgen, web-sys and js-sys (`decisions.md:310-312`);
  - the Chromium and wasm-bindgen preconditions (`README.md:195-197`; `decisions.md:333-339`).
- The design still says "a browser on loopback" and "a real browser" for `jig-test` (`service.md:249,309-311`; `README.md:133-134`). The plan narrows these to the native shell.

#### decisions.md: what the revised design overturned
- **1 Scope** (`decisions.md:37-44`): the store is now in scope (`store.md`); the browser's shells are out (`STATUS.md:103-108`). smith's asks (`:27-36`) are mostly gone:
  - the completion's contract is "not asked" (`protocol/README.md:344-346`);
  - run text, the next channel version and the LLM close are "later" (`:347-359`).
- **3 Landing**: `land --local`, and jig and smith without a gate (`:59-67`), are replaced by `land` pushing through `scripts/check.sh` (jig `AGENTS.md:3-5`; smith `564d4408`).
- **5 Renames**: the `jig-inline-agent` row (`code-organization.md:34`) is gone. The rest stand (`README.md:637-650`).
- **8 Repins in two skein revisions** (`:115-131`): jig is now on GitHub's `main`. The skein work they assumed is still unbuilt (see above), so the repin sequence must be re-derived.
- **9 Reference service and its ports** (`:135-170`) is overturned:
  - the store's port and the secrets' port, answered by `jig-fake-store` and a table, give way to `jig-protocol-store`;
  - the driver's port "until jig 28" gives way to the driver as a child, with handed-in accounts;
  - the test root's "engine's own hub and inline agent" (`:146-147`) is gone;
  - "No binary is built" (`:149`) is reversed: `jig-test` is in scope (`service.md:244-252`).
- **10 Text asked on every run** (`:172-179`) is overturned. Agent processes give no text or named calls until smith's channel carries them (`channel.md:366-373`; `protocol/README.md:347-355`).
- **11 No live services** (`:183-187`) is softened. A live layer exists (`service.md:309-311`), and handed-in accounts serve "live checks that borrow a login read-only" (`llm.md:229-238`; jig `2fae240a`). The browser part is excluded.
- **13 Formats** (`:222-226`): smith's channel "next version" (smith 04) is not needed. jig builds on smith's channel v2 as it stands.
- **16 The test worker** (`:272-277`): its kit no longer holds a scripted agent capability behind a hub-vocabulary port. It holds smith's scripted agent processes, spawned by the worker's protocol layer (`channel.md:600-607`; `service.md:295-296`).
- **17 LLM accounts by file only** (`:281-283`), together with 21's "no account is handed in", is extended by handed-in accounts (`llm.md` 5.4).
- **20 The browser's client** (`:300-312`) and "For the user" 4 (`:333-339`) are dropped.
- **"For the user" 1, the store waits** (`:316-322`), is overturned (jig `9a9aed35`).
- **Second round 3, closing smith's LLM component** (`:431-434`), is later (`protocol/README.md:356-359`).
- **Still holding:**
  - 12, the budgets;
  - 14, step code against ordinary Rust (with `jig-test-worker` redefined);
  - 15, host tools decoded outside the root and results read by `jig-charter` (`channel.md:330-350,298-304`; not built in `jig-charter`);
  - 18, sign-in by the printed link (native now);
  - 19, grants naming their table (`channel.md:445-454`).

#### skein/ and smith/ sessions: did they land?
- **skein 01, in-process addresses:** designed (`f6c3e82`), not built. There is no `InProcess` in `skein/crates`.
- **skein 02, record dropped:** not built. On `Expired` the driver keeps `State::Record` and only clears `held` (`crates/skein-oauth-accounts/src/component.rs:1092-1095`).
- **skein 03, private startup read:** designed (`72462f7`), not built. There is no `read_private_file`.
- **skein 04, base64, `Secret`, wasm32:** designed (`72462f7`), not built.
- **skein 05, `CryptoRng`:** designed (`dec0075`, with the shell's seed in `6d92015`, docs only), not built. `skein-shell/src/seed.rs` still gives a `u64` `Rng` seed.
- **smith 01, skein addresses:** not done; it waits on skein 01–05.
- **smith 02, the completion's contract:** dropped ("not asked", `protocol/README.md:344-346`).
- **smith 03, text in flight:** not on `main`; "later" (`protocol/README.md:348-355`).
- **smith 04, channel text:** not on `main`. smith has channel v2 (`crates/smith-channel/schema/channel-v2.schema`) without text; "later".
- **smith 05, skein joined sockets:** not done; it waits on skein 01.
- **smith 06, the LLM component's close:** "later" (`protocol/README.md:356-359`; `STATUS.md:81-86`).
