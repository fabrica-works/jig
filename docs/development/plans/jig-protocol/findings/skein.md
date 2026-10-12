# skein code audit for jig-protocol

This audit was done read-only on skein `5c9eedd`, smith `2a800ea2` and jig `2fae240a`, each on `main` and equal to `origin/main`. smith's and jig's `Cargo.lock` both pin skein at `30dc6f7`, whose code is the same as `cbabb0c`.

## 0. At a glance

| # | Piece | Built today? | Prod. lines (est.) | Increments | Breaks smith's build at repin? |
|---|---|---|---|---|---|
| 1 | io `Addr` enum, `Name`, `Limits::names`, joined sockets | no (`Addr = SocketAddr` alias) | 900–1300 | 4 (1a type+names+migration, 1b listener/connect/accept, 1c moves/ends/close, 1d worlds) | **yes**: 14 address sites + 10 `io::Limits` literals |
| 2 | `read_private_file`, crypto seed | no | 170–220 | 1 | no |
| 3 | lib `base64`, `Secret`; codec `secret N` | no | ~280 lib, ~−150 net in copies, 100–150 generator | 2 (lib+copies; generator) | no, if the generator's output for existing types is byte-identical |
| 4 | lib `CryptoRng`; OAuth states/verifiers from it | no (`Rng`) | 150–190 | 1 | no (smith never calls `skein_oauth_accounts::Component::new`) |
| 5 | drop record on "sign in again"; handed-in | drop: **no**; handed-in: **yes** | 15–30 | 1 (small) | no |
| 6 | `OpenWrite`/`Cut`, `OpenHow::Write`/`Truncate`, crash keeps old length | no | 300–350 | 1–2 | **yes**: 8 exhaustive-match sites |

Total skein production code is about 2,000–2,600 lines, in about 10–12 increments.

**Gates**
- **skein:** `scripts/check.sh` runs fmt, `clippy -D warnings`, `scripts/taste.sh crates/skein-tls tests/tls` and `cargo check -p skein-tls`, then nextest with the default, `fuzzy` and `browser` profiles.
  - Its last landed code commit is `cbabb0c`: 1942 tests / 4.024 s; 97 tests / 11.329 s; 6 tests / 4.692 s; 42 s in all.
  - `30dc6f7`, which changed scripts only, gives 1942 / 4.252 s; 97 / 11.382 s; 6 / 4.450 s; 41 s.
- **smith:** smith now has a `scripts/check.sh` (`564d4408`), so the workspace AGENTS.md's "smith has no check.sh yet" is stale. It runs fmt, clippy, and nextest with the default and fuzzy profiles.
  - At `2a800ea2`: 2212 tests / 6.024 s; 34 tests / 14.513 s; 77 s in all.
- **jig:** at `0e952778`: 2173 tests / 9.007 s; 86 tests / 12.010 s; 136 s.

---

## 1. io's `Addr::{Ip, InProcess}`, `Limits::names`, joined sockets

**Design.** In io.md §2 (sketch), `pub enum Addr { Ip(SocketAddr), InProcess(Name) }`.
- **§3.5 (io.md:428–512):** a listener on a name accepts sockets joined in memory, through the same requests, events and orders as TCP. There is no kernel and no TLS. `Limits::names` bounds the names table, and an end costs one slot of `Limits::sockets`.
- **§4 (io.md:514–532):** `Name` holds at most `Name::MAX` bytes, inline, so `Addr` allocates nothing. The empty name belongs to a connector.
- **§8 (io.md:961–981):** the worlds to build. §9 lists the work as remaining.
- **kernel.md §3:** the kernel's records keep socket addresses only (`Done::Accepted{peer: Addr}`, `Bound(Addr)`, `Op::Bind/Connect{addr}`), so the kernel needs no change.
- **simulator.md:** says nothing new. io.md §3.5 says joined pairs never reach the simulator or its trace, and a cut drops both ends.
- **shell.md §6, step 5 (shell.md:138–140):** startup hands an in-process name on unresolved.

**Code today.**
- `crates/skein-io/src/kernel.rs:286–291` has `pub type Addr = SocketAddr`, and `Family::of(&Addr)` is at `:303`.
- io's records use that alias directly:
  - `records.rs:9`, `:18` (`Listen{addr}`), `:22` (`Connect{addr}`), `:71` (`Listening{addr}`), `:79` (`Accepted{peer}`).
- So every user passes a `SocketAddr` straight in.
- There is no `Name`, no names table and no joined code.
- `Limits` (`limits.rs:13–45`) is a struct with all-public fields and no `names`. `worst_case` is at `limits.rs:107–130` and `is_usable` at `:63–72`.
- The entity enum is at `layer.rs:27–33` (`Listener`, `Stream`, `Pipe`, `Child`, `Signals`).
- The stream machine (`stream.rs`, 1715 lines) is built around `fd` and `Flight`s (states at `stream.rs:36–68`). A joined end is best a new machine (for example `joined.rs`) that reuses `Intake`/`Output`/`Demand`/`Reservation`.

**Types and functions that change or are added.**
- **New in skein-io:**
  - `Addr` (enum, Copy) and `Name` (inline bytes, `MAX`, `Copy`/`Eq`/`Ord`/`Hash`), exported from `lib.rs:48–50`.
  - `Limits::names`, with `worst_case` and maybe `is_usable` taking it into account.
  - The names table in `Tables` (`layer.rs:38–51`) and a joined entity in `Entity`.
- **Changed in skein-io:**
  - `records.rs:18,22,71,79` take io's `Addr`.
  - `layer.rs:508–509` and `listen`/`connect` at `:531–547` dispatch `Ip` to the kernel path and `InProcess` to the joined path.
  - `listener.rs:577` and `:599` wrap `Addr::Ip` when they emit. Internal state keeps `SocketAddr` (`listener.rs:9,26,35,93,111,132,347,411–416,476–497,542–577`; `stream.rs:11,42,47,265,380–382,467,803–804,873–874,1554`).
- **Keep `kernel::Addr = SocketAddr`.** Renaming it would add about 54 lines in 9 files:
  - `skein-shell/src/ring.rs:226,1495,1529`
  - `skein-sim/src/net.rs` (5), `sim.rs` (7) and `trace.rs` (3)
  - `testing/skein-conformance/src/run.rs` (9) and `scenarios.rs` (12)
  - `tests/ring/src/lib.rs` (5), `tests/sim/src/exchange.rs` (4) and `support.rs` (6)

**Users that must change in skein.** Matches with `..` need no edit: about 60 in skein and 43 in smith.

**A. Where io's address flows as a value (builders and matches): 27 sites.** An `impl From<SocketAddr> for Addr` keeps each edit to one token.
- skein-io's own tests (10): `src/tests.rs:46–48` (`local()` serves both kernel `Done::Bound` and the records, so split it) and `:262–282`; `tests/layer.rs:15–17`; `tests/listener.rs:20,34,45,57,76,208,239`; `tests/stream.rs:81,95,358,608,628`.
- `crates/skein-oauth-accounts/src/component/sign_in.rs:118–121`: `Listen{addr: oauth::redirect_address(..)}` returns a `core::net::SocketAddr`.
- `examples/echo`:
  - `protocol/src/layer.rs:9,24,58,98,186,429,434` and `protocol/src/tests.rs:17,36`
  - `service/src/lib.rs:46,185,279` and `service/src/tests.rs:13,44` (one helper serves kernel and io)
  - `shell/src/lib.rs:5,20` and `shell/src/main.rs:36,99` (parses a `SocketAddr`)
- `testing/skein-echo-client/src/conn.rs:9,151`, `lib.rs:30,184–185,266` and `tests.rs:13,53`.
- `testing/skein-fake-peers/src/face.rs:49` (`Listen{addr: address}`), `llm.rs:146–149` and `oauth.rs:161–164` (store `Listening.addr` in `Option<SocketAddr>`). Keep `address()` returning `SocketAddr` so that smith (`tests/local-process`, `agent-process`) does not churn.
- `tests/io/src`:
  - `owner.rs:10,17,92,130,178,352,362,369,396,570–571` (`nowhere()` uses `.ip()`)
  - `scenarios.rs:13,50,189`
  - `output.rs:132`
- `tests/io/tests/memory.rs:282,285,355–356,375,467,469`.
- `tests/browser/src/serve.rs:46,60,110–112` and `tests/browser/tests/page_server.rs:18,21`.
- `tests/echo/src/proc.rs:7,21`, `referee.rs:10,170` and `scenarios.rs:18,101`; `tests/echo/tests/end_to_end.rs:7,79` and `real.rs:16,33` (`"127.0.0.1:0".parse()`).
- `tests/tls/src/native.rs:177`; `tests/tls/tests/fake_peer.rs:50,97`.
- `tests/oauth-accounts/src/sign_in/browser.rs:167`.

**B. `Endpoint.address`: 2 type definitions, 2 loopback checks and 18 constructions.**
- Type definitions:
  - `crates/skein-llm-connection/src/endpoint.rs:5,21`
  - `crates/skein-oauth-accounts/src/accounts.rs:4,25`
- Loopback checks, which need a rule for `InProcess` (see risks):
  - `crates/skein-llm-connection/src/component.rs:157`
  - `crates/skein-oauth-accounts/src/component.rs:141`
- Constructions:
  - `crates/skein-llm-connection/src/tests.rs:3,40,331,351`
  - `crates/skein-oauth-accounts/src/private_tests.rs:33`; `crates/skein-oauth-accounts/src/tests.rs:380` (`refresh_account`)
  - `tests/llm-connection/src/simulated.rs:76,109`; `tests/llm-connection/src/world.rs:328`
  - `tests/llm-connection/tests/failures.rs:5,110`, `fake_protocol.rs:8,89`, `memory.rs:6,51,259` and `protocol.rs:5,69,243`
  - `tests/oauth-accounts/src/sign_in.rs:188`, `world.rs:311`, and `tests/memory.rs:134,198,301`
- These pass through and need no edit: `llm-connection/src/component.rs:939` and `oauth-accounts/src/component.rs:706`.

**C. Full `io::Limits` literals that `Limits::names` breaks: 31 in skein.**
- `crates/skein-io/src/tests.rs:30–42`
- `crates/skein-llm-connection/src/tests.rs:16`
- `crates/skein-oauth-accounts/src/tests.rs:21`
- `examples/echo/service/src/tests.rs:20`
- `examples/echo/shell/src/lib.rs:89` (production)
- `testing/skein-echo-client/src/tests.rs:19`
- `testing/skein-fake-peers/src/transport/tests.rs:16`, `tests/llm.rs:12` and `tests/oauth.rs:23`
- `tests/browser/tests/browser_real.rs:97`
- `tests/echo/src/scenarios.rs:54,82`
- `tests/io/src/append.rs:60`, `output.rs:107` and `scenarios.rs:27`
- `tests/io/tests/memory.rs:418,430,442,456` and `process_memory.rs:177,213` (the lines of their `backlog` fields)
- `tests/llm-connection/src/simulated.rs:29` and `world.rs:93`; `tests/llm-connection/tests/failures.rs:23`, `fake_protocol.rs:26`, `memory.rs:22` and `protocol.rs:21`
- `tests/oauth-accounts/src/world.rs:90`; `tests/oauth-accounts/tests/memory.rs:36`
- `tests/tls/src/native.rs:157`; `tests/tls/tests/fake_peer.rs:19`

`crates/skein-io/src/tests/layer.rs:93` asserts that `worst_case` does not vary with some fields. Recheck it once the names table counts.

**Not users.** skein-http, skein-tls, skein-channel, skein-kv, skein-llm, skein-json, skein-codec, skein-world and skein-browser name no address (checked with `git grep SocketAddr|Addr`).

**smith users.**
- Address sites (14):
  - `crates/smith-agent-shell/src/config.rs:420` (`connection::Endpoint{address}` from `resolve()` at `:371`; its `.ip()` check at `:448` stays on `SocketAddr`)
  - `crates/smith-local-shell/src/local_auth.rs:123` and `:214`
  - `crates/smith-local-shell/src/local_auth_http.rs:15,112,372`
  - `tests/agent-process/src/lib.rs:189`
  - `tests/local-process/src/oauth.rs:50,213`; `tests/local-process/src/process.rs:200`
  - `tests/protocol-llm/src/pool_peer.rs:98` (`.parse()`); `tests/protocol-llm/tests/component_boundary.rs:3,88`, `component_peer.rs:8,131` and `endpoints_grants.rs:3,26`
- These pass through and need no edit: `crates/smith-agent-service/src/service.rs:1143–1144,1928–1930`.
- `io::Limits` literals (10):
  - `crates/smith-agent-service/src/profile.rs:254` (production)
  - `crates/smith-agent-service/src/service.rs:2251`
  - `crates/smith-local-service/src/tests.rs:38`
  - `tests/hosts/src/lib.rs:120`
  - `tests/local-process/src/oauth.rs:287` and `terminal.rs:76`
  - `tests/machine/src/lib.rs:42`
  - `tests/protocol-llm/src/pool_peer.rs:34`; `tests/protocol-llm/tests/component_boundary.rs:36` and `component_peer.rs:38`

**jig users.** None. jig never names skein-io.

**Tests and fixtures today.**
- Step tests: `crates/skein-io/src/tests/{layer,listener,stream,output,kernel}.rs`.
- io worlds:
  - `tests/io/src/{world,owner,referee,ledger,scenarios,census}.rs`
  - `tests/io/tests/{scenarios,fuzzy_scenarios,output,fuzzy_output,memory,ring}.rs`
- The world's `busy()` already counts `io.is_ready()` (`tests/io/src/world.rs:142–148`), so progress driven only by the ready list works.

**The world that tests the new piece.**
- New step tests, for example `crates/skein-io/src/tests/joined.rs`: each cell of the §3.5 table, and "a joined end submits nothing" at every call.
- io worlds in `tests/io` with one process, a `Serve` on `Addr::InProcess` and `Dial`s to it. The owner already holds both. New referee scenarios cover:
  - refusals: an unbound name, and a closing listener
  - connectors waiting with no room or no slot, and an abort while waiting
  - two listeners under one batch; a name taken again after `Closed`; every name held
  - small intakes and backpressure
  - the mid-upload refusal; abort, heard as reset; the close deadline; random closes
- Plus `tests/io/tests/joined.rs` and `fuzzy_joined.rs`, and the "same owners over either" calm world.
- Memory, in `tests/io/tests/memory.rs`: every name held, every slot an end.
- A one-process real-ring test in `tests/io/tests/ring.rs`, via `skein_world::real`, with a loopback listener and a name listener in one `drive`.

**Size and split.**
- **1a:** `Addr`/`Name`, `Limits::names` and the names table; `Listen` on a name takes it and `Closed` frees it; `Connect` to an unbound name is refused; plus all of A–C. About 150–250 production lines in skein-io, but the "Files:" list spans about 45 skein files.
- **1b:** the joined listener, with waiting connectors, accept arming under batch, room and slot, bind/reject and refusing at close. About 300–400 lines.
- **1c:** joined ends: moves, demand and room, independent output, half-close, the four-step close, abort/reset propagation. About 400–550 lines.
- **1d:** the worlds, memory and the ring one-process test. Test code only, about 600+ lines.

**Risks.**
- **Design gap: TLS or plaintext to an in-process name.** llm-connection.md §3 (`:79–81`) and oauth.md §6.2 allow plaintext only on loopback, and io.md §3.5 says there is no TLS on a joined socket. Neither says how `component.rs:157` (llm-connection) and `component.rs:141` (oauth-accounts) treat `InProcess`. Suggested rule: admit plaintext, and refuse TLS at configuration. jig's one-process shape needs this.
- **Conveniences decide the churn.** `From<SocketAddr>`, and possibly `FromStr` and `From<(IpAddr, u16)>`, matter because tests use `.parse()` (skein `tests/oauth-accounts/tests/memory.rs:134,198`, `llm-connection/src/tests.rs:331`; smith `pool_peer.rs:98`) and tuple `.into()` (`tests/llm-connection/src/world.rs:328`, `simulated.rs:76`, `tests/oauth-accounts/tests/memory.rs:301`).
- **`Addr` must stay `Copy`.** `tests/io/src/output.rs:79–81` copies through `*addr` and `*peer`. `Name::MAX` is unset; its size drives the size of `Addr`, which is 32 B today.
- **Every host must count `io.is_ready()` in `Host::work_pending`.** echo does (`examples/echo/service/src/lib.rs:236–239`). Otherwise `drive` (`crates/skein-shell/src/drive.rs:63–70`) blocks on the kernel while joined bytes wait. jig's host must do the same.
- **Trace evidence is missing for joined pairs.** The io worlds' census of the trace (`tests/io/src/census.rs`) never sees them, so a scenario's evidence must come from the ledger or the owner. io.md §10 leaves faults between joined ends open, so worlds see only owner behaviour.
- **Coordination with smith's reliability plan.** STATUS.md marks it ready (02.6): io `Resized`/`Window` and kernel `Window`, in the same files (`records.rs`, `kernel.rs`, `ring.rs`, the sim). Expect rebase conflicts.

---

## 2. `read_private_file`, the cryptographic seed, `skein_shell::seed()`

**Design.**
- **shell.md §6.1 (`:152–191`):** `read_private_file` reads like `read_file`, then makes io's §5.3 checks on what it opened before it reads a byte. It requires a regular file; no link in any path component; no group or other bits; one link; and the process's effective user as owner. Anything else is refused unread, naming the cause as io's `Unsafe` does. A denied open is named through a path-only stat. shell.md §11 lists it as not built.
- **shell.md §5 (`:117–121`):** a 32-byte crypto seed from `getrandom`, beside the `u64` seed and never derived from it, handed to startup as a `Secret`. "The service keys each `CryptoRng` from it."

**Code today.**
- `crates/skein-shell/src/startup.rs:33–35` (`read_file`) calls `ring/startup.rs:16–27`, which uses `libc::open` with `O_NONBLOCK|O_NOCTTY|O_CLOEXEC`. `read_opened` (`:29–66`) checks the kind and size with `fstat`; `kernel_error` is at `:68–78`; `Unread` is at `startup.rs:18–29`.
- There is no `read_private_file`.
- `effective_user()` is at `ring.rs:1669–1674`.
- `seed()` (`crates/skein-shell/src/seed.rs:11–13`) calls `ring::random()` (`ring.rs:1654–1666`), which draws 8 bytes.
- `Unsafe` already exists at `crates/skein-io/src/file.rs:155–168` (Link, Kind, Mode, Links, Owner) and can be reused.
- Exports are at `crates/skein-shell/src/lib.rs:34–41`.

**Changes and additions.**
- A new `read_private_file(path, max, user?) -> Result<Secret | Box<[u8]>, PrivateUnread>`. The signature is not designed.
- Implementation: `openat2(AT_FDCWD, …, RESOLVE_NO_SYMLINKS)` (the syscall pattern is `open_append` at `ring.rs:1609–1633`), then `fstat` checks. On `EACCES`, reopen with `O_PATH|O_NOFOLLOW`, stat, and name the refusal.
- A new `crypto_seed() -> Result<Secret, i32>` (name not designed), from generalising `ring::random` to fill a 32-byte buffer.

**Users.**
- `seed()` has 5 users and needs no change:
  - skein: `examples/echo/shell/src/main.rs:89`, `tests/ring/tests/ring.rs:202`
  - smith: `crates/smith-agent-shell/src/agent_shell.rs:372`, `crates/smith-local-shell/src/local_shell.rs:122,179`
- smith's `oauth_entropy()` (`local_shell.rs:175–183`) builds 32 bytes from 4 `seed()` calls. It could move to the crypto seed later; that is optional.
- `read_private_file` has no user yet. jig's protocol is the first (jig `docs/design/protocol/README.md:305`, `llm.md` §5.4).

**Tests and world.**
- `tests/ring/tests/files/startup.rs` (real filesystem, scratch dir) holds the `read_file` cases. Add:
  - a `0o644` file refused as Mode
  - a symlink in a parent, and at the leaf, refused as Link
  - a hard link refused as Links
  - a directory or FIFO refused as Kind
  - another owner refused as Owner: pass a wrong expected user, or take a root-owned `0o600` file
  - every descriptor closed
- Mirror the `CAP_DAC_OVERRIDE` skip in `tests/io/tests/ring.rs:164–169`.
- For the crypto seed: two draws differ, as `tests/ring/tests/ring.rs:202` does.

**Size.** About 150–180 lines for the private read and 20–30 for the seed: one increment.

**Risks.**
- The signature, the refusal type and the seed function's name are not designed.
- New `unsafe` code (`openat2`, `O_PATH`) goes in the ring adapter, so it needs review.
- The owner check runs differently under root.

---

## 3. lib `base64`, lib `Secret`, codec `secret N`

**Design.**
- **lib.md §6 (`:101–106`):** `Secret`'s `Debug` shows only its length; `expose()` reads it; Clone, Eq and Hash are over its bytes.
- **lib.md §6.1 (`:109–134`):**
  - two alphabets: Standard (padded) and Url (unpadded)
  - `encoded_len` and `encode` into a `Writer`
  - a bounded, canonical `decode(max)` with `base64::Error`
  - a piecewise `Decoder`
- lib.md §13 (`:478–484`) lists the copies to move.
- **codec.md §3 (`:91`, `:112–117`):** `secret N` is encoded as `bytes N` and generated as `lib::Secret`. In §4 (`:139`), the `Debug` of a `secret` field is its length.

**Code today.** There is no `base64` module and no `Secret` in `crates/skein-lib/src/lib.rs:18–60`. The copies to replace are:
- **skein-oauth PKCE encode:** `crates/skein-oauth/src/pkce.rs:73` (alphabet) and `:84–118` (the encode loop in `challenge`). `sha256` is at `:120+`.
- **skein-oauth JWT decode:** `crates/skein-oauth/src/jwt.rs:57–66` (`sextet`), `:67–96` (`decoded_len`) and `:97–134` (`decode`), used at `:27–29`. Header and signature are only validated (`decoded_len`), never decoded.
- **skein-shell PEM decoder:** `crates/skein-shell/src/startup.rs:198–262` (`Decoder`, `push`, `valid`), driven by `pem()` at `:125–151`. It has lax line handling (`whitespace`, including 0x0b, at `:157–176`) and returns `RootsRefusal::TooLong{index, bound}` per certificate.
- **skein-browser screenshots:** `testing/skein-browser/src/wire/base64.rs:1–73` (92 lines with its tests), declared at `wire.rs:3` and used at `testing/skein-browser/src/browser.rs:1676`.
- **Test-only encoder:** `tests/tls/tests/startup.rs:11–24`.
- **Alphabet copy for the OAuth state:** `crates/skein-oauth-accounts/src/component/sign_in.rs:187–195` (`draw`). It moves with item 4.
- smith's `benchmarks` use the crates.io `base64` crate. That is not skein's and stays out of scope.

**Codec generator.**
- `testing/skein-codegen/src/lib.rs:68–81` holds `enum Type` and `parse.rs:118` parses `"bytes"`.
- `Type::Bytes` has 22 match sites that need a `Secret` arm:
  - `check.rs:123`
  - `emit_codec.rs:25,36,76,109,167`
  - `emit_limits.rs:83`
  - `emit_tests.rs:56,98,177,227`
  - `emit_types.rs:16,26,41,60,69`
  - `emit_worst.rs:16,30,46`
  - `lib.rs:99,123`
  - `parse.rs:118`
- Fixtures: `tests/codec/schema/{sample,scalars,lints}-v1.schema`, `tests/codec/src/generated`, `tests/codec/golden/v1`, and `tests/codec/tests/{drift,wire,bounds,worst,memory,fuzzy_decode}.rs`.

**Users of the changing types.**
- `Secret` and `base64` are new, so nothing breaks. If `skein_oauth::SavedToken` or `Granted.token` moved to `Secret`, smith would break at about 15 `encode_record`/`decode_record` sites, so leave them.
- smith calls no `jwt`, `pkce`, `read_trust_roots` or browser base64 (`git grep` is empty).
- jig uses none of these.

**The world that tests them.**
- For `base64`: lib tests (`crates/skein-lib/src/tests/bytes.rs`, plus a new `base64.rs`) with RFC 4648 vectors, every refusal, and pieces at every split. `tests/lib/tests/fuzzy_bytes.rs` takes a round-trip.
- The existing tests then prove the moves:
  - `crates/skein-oauth/src/pkce.rs:174–196` and `tests.rs:193` (base64url mutations)
  - `tests/ring/tests/files/startup.rs` (PEM) and `tests/tls/tests/startup.rs`
  - skein-browser's `wire/base64` tests, moved, and `tests/browser`
- For `secret N`: add a field in a new test schema. Do not edit `sample-v1`, whose goldens are frozen. Add a test that `Debug` shows only the length.

**Size.**
- lib `base64` plus `Secret`: about 250–300 production lines. Moving the copies removes about 230 lines and adds about 50.
- The generator: about 100–150 lines.
- Two increments: (a) lib `base64` and `Secret`, with the copies moved; (b) codec `secret N`.

**Risks.**
- **smith's drift tests** (`smith/tests/codecs/tests/drift.rs`) compare generator output byte for byte. Emission for existing types must not change.
- **smith's channel schema breaks codec.md §3's credential rule** (`crates/smith-channel/schema/channel-v2.schema:16`, `GrantValue{credential: bytes 8192}`). That is smith's to change, not this plan's.
- **JWT validation without decoding.** lib's API (decode with a max, and a `Decoder`) must still validate header and signature without keeping them, or accept a bounded allocation.
- **PEM semantics.** The PEM reader needs the `Decoder`'s per-block state to keep its malformed and other-block counts and `TooLong{index}`.
- **wasm32-safe lengths.** Length arithmetic must stay checked (see §7).

---

## 4. lib `CryptoRng`; OAuth states and verifiers drawn from it

**Design.**
- **lib.md §8 (`:308–312`):** ChaCha20 keyed by a 32-byte seed, with `fill` into a bounded buffer. Whatever must not be guessed is drawn from it.
- **oauth.md §2 (`:74–79`):** the verifier comes from the caller's `CryptoRng`.
- **oauth.md §6.1 (`:214–218`):** the owner seeds the component's generator.
- **programming-model.md §9 (`:822–825`).**

**Code today.**
- `crates/skein-lib/src/rng.rs` holds only `Rng` (SplitMix64, "not for secrets"). lib has no dependencies (`crates/skein-lib/Cargo.toml`), so ChaCha20 must be hand-written.
- `skein_oauth_accounts::Component` has the field `random: Rng` (`crates/skein-oauth-accounts/src/component.rs:117`).
- The constructor is `pub fn new(accounts, limits, seed: u64)` (`:124`) and seeds with `Rng::new(seed)` (`:199`).
- The state and verifier are drawn at `component/sign_in.rs:109,111` through `draw()` (`:187–195`), which calls `Rng::below(64)` over the url alphabet.
- `skein_oauth`'s machine takes the state and verifier as input and draws nothing itself.

**Users of `Component::new`: 12, all in skein.**
- `crates/skein-oauth-accounts/src/private_tests.rs:46`
- `crates/skein-oauth-accounts/src/tests.rs:70,415,449,497,508`
- `tests/oauth-accounts/src/sign_in.rs:202` and `world.rs:331`
- `tests/oauth-accounts/tests/memory.rs:71,151,206,311`

smith and jig never call it. smith builds its own OAuth nonce (`crates/smith-local-shell/src/local_auth.rs:196–209`, a digest over its entropy).

**Changes.**
- A new `skein_lib::CryptoRng`.
- `Component::new(.., seed: u64)` becomes `(.., crypto: CryptoRng)` or takes the crypto seed.
- `draw` becomes a CryptoRng fill plus base64url (32 bytes give a 43-character verifier), or `byte & 63` over the alphabet if item 3 has not landed.
- Other `Rng` holders draw nothing secret and stay: `skein-lib/src/requests.rs:186` (retry jitter), `skein-llm/src/translate.rs:851`, `skein-sim/src/sim.rs:124`, and io's temp-file suffix.

**Tests and world.**
- lib tests with RFC 8439 ChaCha20 block and keystream vectors, and `fill` across block boundaries.
- `crates/skein-oauth-accounts/src/tests.rs`.
- `tests/oauth-accounts/tests/{sign_in,fuzzy_sign_in,real,world}.rs`, with each world keying its generator from its own seed. No test pins a state value (only the fake-peers fixture at `testing/skein-fake-peers/src/oauth.rs:648`, which is not drawn).

**Size.** About 120–150 lines for ChaCha20 and `CryptoRng`, and 20–40 for oauth-accounts: one increment.

**Risks.**
- **Design gap.** "keys **each** `CryptoRng` from it" (shell.md:120) needs a per-generator stream or nonce, or two layers keyed from one seed draw identical streams. Decide on something like `CryptoRng::new(&Secret, stream: u64)` with ChaCha20's nonce.
- The cipher is hand-rolled, so it needs vectors and review.

---

## 5. oauth-accounts: drop a record failed as "sign in again"; handed-in accounts

**Design.**
- **oauth.md §6.4 "Loading again" (`:318–321`):** a refresh that fails as sign in again drops the account's record, so the account holds none and loads again before its next grant. The design commit `f6c3e82` added this on 2026-10-10.
- **oauth.md §6.2 and §6.3:** a "handed in" source, the `hand in` request and the `expiring` event.

**Is the drop built?** **No.**
- The code is from `ebbeca4` (2026-10-09), which is older than the rule.
- In code, "sign in again" is `skein_oauth::Failure::Refused`, which `classify` gives for `invalid_grant` (`crates/skein-oauth/src/common.rs:65–66`).
- `fail_exchange` (`crates/skein-oauth-accounts/src/component.rs:827–880`) fails the grant and clears `held`, but never resets the slot to `State::Empty`.
- `git grep 'Failure::Refused' crates/skein-oauth-accounts tests/oauth-accounts` is empty.
- STATUS.md says oauth.md has only §7 left, which is stale for this rule.

**Is handed-in built?** **Yes.**
- `Account::HandedIn` is at `accounts.rs:17` (and `Source::HandedIn` at `:50`).
- `Request::HandIn` is at `boundary.rs:18`, with `Asked::HandIn` at `:85` and the refusals `Source`, `NotAccessOnly` and `Record` at `:118–129`.
- `Event::Expiring` is at `boundary.rs:44`.
- Handling: `component.rs:243` (`hand_in` at `:375–392`); a rejection gives `Expired` at `:257–260`; `Expiring` is announced once at `:556–560`.
- Step tests: `crates/skein-oauth-accounts/src/tests.rs:62–168,262–277,330,376`. Memory: `tests/oauth-accounts/tests/memory.rs:69,77`.
- **Gap:** no world story covers it. oauth.md §6.8 asks for "an access-only record expiring and handed in again" in the protocol world, and the `Story` enum at `tests/oauth-accounts/src/world.rs:21–46` has none.

**Change.**
- In `fail_exchange`, or where a refresh's `oauth::Request::Failed{Refused}` arrives (`component.rs:719`), set the slot to `State::Empty` when the exchange is a refresh.
- A private keeper then reloads (`component/private.rs:18,34`). An owner keeper fails as `Expired` until a sign-in.

**Users.** None. No type changes.

**Tests and world.**
- A step test in `crates/skein-oauth-accounts/src/tests.rs` (owner keeper).
- `private_tests.rs`: private keeper, file reloaded at the next grant.
- A world story in `tests/oauth-accounts/src/world.rs`: the fake issuer's `refuse()` already answers `invalid_grant` (`testing/skein-fake-oauth/src/lib.rs:392–399`).
- In `tests/oauth-accounts/src/private.rs`: a sign-in from another process writes the file, then the next grant loads it.
- Optionally, the missing handed-in story.

**Size.** 15–30 production lines and about 150 test lines: one small increment, which could join item 4.

**Risk.** The distinction between `Refused`, which drops, and a second rejected token (`Expired`), which does not, must stay.

---

## 6. io `OpenWrite`/`Cut`; kernel `OpenHow::Write` and `Truncate`; the crashing disk keeping the old length

**Design.**
- **io.md §5:** the sketch at `:552–557` has `OpenWrite`, `OpenWriteNoFollow` and `Cut{owner, file, length}`. The rule at `:590–597`: `OpenWrite` opens an existing regular file to read and write and never creates it ("`Missing`" if absent); `Cut` sets the length, durable once synced.
- **kernel.md §6.1 (`:233–243`):** `OpenHow::Write`, and `Truncate` (`ftruncate`) on a descriptor opened to write. Like a write it is durable once synced: "the simulator's crashing disk keeps the old length past a crash until then (simulator.md, 3.1)".
- The design commit `5c9eedd` did not update simulator.md §3.1.

**Code today.**
- `OpenHow` (`crates/skein-io/src/kernel.rs:591–610`) has no `Write`, and `Op` (`:364–470`) has no `Truncate`.
- `file::Request` (`crates/skein-io/src/file.rs:31–148`) has no `OpenWrite` or `Cut`, and there is no `Event::Cut`.
- FileIo dispatch is at `crates/skein-io/src/file_layer.rs:440–565`. `open_read` and `Sync` (`:543`) are the models.
- The simulator's machine seam `Ask` (`crates/skein-sim/src/machine.rs:57–95`) has no truncate.
- The fake machine: `testing/skein-fake-machine/src/fs.rs:60–80` (`How`, no `Write`); `write`, `sync` and `crash` at `:440–534` (`durable_nodes` and `pending_writes`); `face.rs:35–60,139–147` (the Ask→fs and OpenHow→How mappings).
- **The ring:** io-uring 0.7.15 has `opcode::Ftruncate` (`~/.cargo/registry/.../io-uring-0.7.15/src/opcode.rs:2134`). `IORING_OP_FTRUNCATE` arrived in 6.9, below the 6.12 floor (shell.md §4), so Truncate can be a ring operation. Add it to the probe list at `crates/skein-shell/src/ring.rs:383–401`.

**Changes (types).**
- `kernel::OpenHow::{Write, WriteNoFollow}` and `kernel::Op::Truncate{fd, length}`, with `Done::Nothing`.
- `file::Request::{OpenWrite, OpenWriteNoFollow, Cut}` and `file::Event::Cut{owner}`.
- `skein_sim::machine::Ask::Truncate{file, length}`.
- `skein_fake_machine::How::Write` and `Machine::truncate`.

**Exhaustive matches that break in skein.**

`kernel::Op`, about 43 sites:
- **skein-io:**
  - `file_layer.rs:827–832`; `file_layer/store.rs:671–676,703–708`
  - `kernel.rs:931/953` (`is_valid`), `:995/1012` (`shape`), `:1023–1028` (`files`), `:1331–1336` (`fits`), `:1377–1382` (`binds`) and `:1402–1423` (`counts`), plus the error table in the doc at `:205–213`
  - `pipe.rs:523–540`; `stream.rs:682–687,711–716`
  - `tests.rs:71–76,131–139`
  - `tests/kernel.rs:81–98,297–319,368–378`, `tests/output.rs:329–334` and `tests/store.rs:380–382`
- **skein-shell:** `ring.rs:807–829` (`prepare`), `:918–957` (`decode`), `:980–985` (`error`), `:1071–1076` (`operation_error`), `:1113–1146` (`file_error`) and `:1211–1219` (`check_path_access`).
- **skein-sim:** `sim.rs:275–280` (`cut_with`), `:571–576` (`submit_one`) and `:1161–1166` (`withdraw`); `sim/files.rs:117–121,172–187,246–275,344–358`; `trace.rs:219–228`, plus a `Summary` variant.
- **Elsewhere:**
  - `examples/echo/service/src/tests.rs:149–154`
  - `testing/skein-conformance/src/run.rs:116–121,751–760,792–807,842–847`
  - `testing/skein-echo-client/src/tests.rs:167–172`
  - `testing/skein-world/src/real.rs:609–614`
  - `tests/io/tests/memory.rs:221–265`
  - `tests/ring/src/lib.rs:172–177`

`OpenHow`: `kernel.rs:936–945`; `ring.rs:1282–1299` (`open`, flags `O_RDWR`, no `O_CREAT`); `sim/files.rs:172–181` and `sim.rs:717–773` (access); `testing/skein-conformance/src/files.rs:773–777`; `testing/skein-fake-machine/src/face.rs:139–147`.

`Ask`: `sim/files.rs:153–160,246–275`; `testing/skein-fake-machine/src/face.rs:35–60`.

`file::Request`: `file_layer.rs:441` (`down_inner`); `crates/skein-oauth-accounts/src/private_tests.rs:105–115,331–339`; `testing/skein-world/src/files.rs:214–224`.

`file::Event`: `file.rs:262–275` (the owner accessor); `crates/skein-oauth-accounts/src/component/private.rs:195–200,236–241,286–290,377–381`. kv's tuple matches have catch-alls.

**smith sites that break: 8.**
- `kernel::Op`: `benchmarks/src/process.rs:331–350`, `crates/smith-local-shell/src/local_host.rs:465–490` and `tests/agent-process/src/world.rs:190–210`.
- `Ask`: `tests/machine/src/lib.rs:422–433`.
- `file::Event`: `crates/smith-local-protocol/src/markers.rs:105–115`, `plain.rs:122–133` and `crates/smith-protocol-machine/src/files.rs:129–136`.
- `file::Request`: `crates/smith-protocol-machine/src/tests.rs:579–594`.

**jig:** none.

**Tests and world.**
- **Conformance:** `testing/skein-conformance/src/files.rs`, run on both backends through `tests/conformance/{ring,sim}`. Cover:
  - a `Write` open of an absent file gives `NotFound` and creates nothing
  - it opens an existing file read and write, links and kinds refused as for `Read`
  - `Truncate` shortens and lengthens
  - `Truncate` on a `Read` descriptor is refused
- **Simulator:** `tests/sim/tests/{files,cuts,fuzzy_files}.rs`.
- **Fake machine:** `testing/skein-fake-machine/src/tests.rs`: truncate, then crash before sync keeps the old length and contents; after sync keeps the new length.
- **io steps:** `crates/skein-io/src/tests/file.rs` (OpenWrite and Cut, deadlines and cancels).
- Real ring through conformance, and `tests/ring/tests/files.rs`.

**Size.**
- Kernel, ring, sim, fake machine and conformance: about 220–270 production lines.
- io FileIo: about 60–80 lines.
- One increment of about 330, or two (kernel and backends; then io's requests).

**Risks.**
- **Cancelling Truncate is not designed.** kernel.md:304 lists only `Open`, `Read`, `Write`, `Append` and `Sync` as cancellable, while io.md §5 cancels the operation in flight at a request's deadline.
- **`WriteNoFollow` is not in kernel.md.** It is implied by io's `OpenWriteNoFollow` and by the other NoFollow variants.
- **The crash rule is unwritten in simulator.md §3.1.** Define how `pending_writes` past the cut land at a crash (`fs.rs:487–534` replays chosen writes onto the durable node).
- **"`Missing`" is not an event.** It should be `Failed{NotFound}`.
- **The error set for Truncate** (`ReadOnly`, `NoSpace`, `InvalidArgument`, `EFBIG`) must go into the tables at `tests/kernel.rs:297–319` and `ring.rs:1113`.

---

## 7. wasm32: not in this plan

The pieces that later join the wasm build all live in lib or codec: `base64`, `Secret`, `CryptoRng` and the generated `secret` fields. To stay buildable later:
- Keep them `no_std` and free of any platform API.
- Use no 64-bit `usize` arithmetic: return `encoded_len` as `u32` or `u64`, and convert with `try_from`.
- Keep ChaCha20 to `u32` arithmetic.

io's `Addr` and joined sockets, `read_private_file`, the seeds and `OpenWrite`/`Cut` sit in io or the shell, outside the wasm set (lib.md §2). A browser draws its own crypto seed, so nothing above needs wasm work now.

---

## smith's repin onto skein with items 1–6

Only items 1 and 6 break smith's build: 14 address sites, 10 `io::Limits` literals and 8 exhaustive matches, about **32 edits of one or two lines in about 22 files** (sites in §1 and §6).
- Items 2–5 need no change, as long as `seed()`, `skein_oauth`'s public types (`SavedToken`, `encode_record`, `decode_record`, `client_worst_case`) and the generator's output for existing schemas do not change.
- smith never listens on a name, so `names: 0` suffices.
- Estimate: about 60–80 lines of diff plus the `Cargo.lock` bump. It is one increment, gated by smith's `scripts/check.sh` (77 s).
- It is not urgent for jig.

## Does jig use these types directly?

No.
- jig's code names only `skein_lib` (List, Token, Wall, Duration, Time, Rng, bytes, Map, Set, Slab, Queue, Writer, Reader, Decimal, Journal*, Request*, stream, Env, Id), `skein_json` and `skein_world::domain`.
- jig has no skein-io, skein-shell, skein-oauth or skein-codec dependency, and no base64, Secret or CryptoRng of its own.
- jig's lock pins skein at `30dc6f7`. Its smith dependencies (smith-charter, smith-transcript, smith-domain-run) use only skein-lib and skein-codec at build time; `smith-machine-world` is a dev-dependency. So bumping skein in jig does not wait on smith's repin.
- jig's protocol design is the first user of every item: jig `docs/design/protocol/README.md:283–325`, `configuration.md:72,95,162,207`, `llm.md` §5.4 and `store.md:394,617`.
