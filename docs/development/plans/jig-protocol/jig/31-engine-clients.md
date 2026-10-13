# Session 31: `jig-protocol-engine-clients`, the engine's side of the client's protocol

A component of the engine's protocol layer over skein's HTTP server and
server-sent events writer. It signs a person in by the deployment's
printed link, trading the link's secret for a bearer token whose digest
it keeps in the store's secrets' records; it checks each exchange's
token; it turns exchanges into the core's vocabulary and the core's
answers, reads and documents back into `jig-protocol-client-wire`'s
records; it keeps one stream per client, paced by its connection; and it
routes an application's records and sign-in providers to the protocol
layer's routing. Its translation follows the client's joint world's map
(`tests/client/src/joint/wire/`), ported to step code and aimed at the
wire's records. No browser: no cookie, no page, no client's files.

## Read first

- jig's `protocol/client.md`, sections 1, 2, 4 to 9, 11 and 13.
- jig's `protocol/README.md`, section 8; `protocol/store.md`, section 9;
  `domain/people.md`, section 3; `domain/engine.md`, 5.2, 6.1 and
  section 11; `client/domain/contract.md`, sections 3.3, 4, 6, 7 and 8;
  `client/domain/testing.md`, section 5 (the joint world's translation).
- skein's `http.md`, sections 1, 2, 4 and 5; skein's `lib.md`, sections
  6, 6.1 and 8 (`Secret`, `base64`, `CryptoRng`).
- `decisions.md`, 2, 3 and 20; `code-organization.md`, sections 2 to 6.
- This plan's `jig/03-people-and-tasks.md` (03.2: a reply's own sign-in,
  no output naming an ended one), `jig/13-store-keys.md` (the keys'
  encoders), `jig/17-store-files.md` (the secrets' store) and
  `jig/20-config.md` (the profiles).
- findings/protocol.md, items 1.6 (the core's client-facing vocabulary),
  1.7 (the joint world's translation, module by module) and 4.3
  (skein-http).
- The code: `crates/jig-domain-core/src/routing/vocabulary.rs` (`Event`,
  `Held`, `Now`), `crates/jig-domain-core/src/reads/vocabulary.rs`,
  `crates/jig-domain-core/src/documents.rs`,
  `crates/jig-domain-people/src/boundary.rs` (`Identity`, `Reply`,
  `Event::Ask`, `Event::SignOut`); `tests/client/src/joint/wire/*.rs` and
  `tests/client/src/joint/{transport.rs, requests.rs, packets.rs}`;
  skein's `crates/skein-http/src/{server.rs, sse/writer.rs}` and
  `crates/skein-oauth-accounts/src/listener.rs` (a step-code owner of a
  listener and its HTTP servers).

## What the design asks

- **Exchanges, one record each way** (`client.md`, section 4): POSTs to
  the client's routes, each carrying its sign-in's secret and its owner's
  version.
- **Keyed requests** (section 5): "The engine's side checks the secret;
  hands the core the sign-in's number, the key and the request, decoded
  into the core's vocabulary; and, for an application's request, the
  digest of the record's bytes as they came (SHA-256)". The answer is
  one record: done, refused, busy, not ready, signed out.
- **Reads** (section 6): one exchange, answered once, held until durable.
- **Watches and the stream** (sections 4 and 7): one GET per client; the
  hello first; a delivery written when released, "and tells the core it
  was taken once the event has gone to the connection. While the
  connection does not drain, nothing more is written for that stream";
  heartbeats; endings; "A stream that ends ends its watches."
- **Signing in by the link, natively** (section 8): "A native client on
  the same machine signs in by the same link: it sends the link's secret
  to the sign-in route in an exchange, and takes back its bearer token in
  the answer." One link live at a time; a fresh one printed when the
  route is reached without a live link's secret; "only on a loopback
  listener".
- **Kept before used** (section 8; `store.md`, section 9): the core
  commits the sign-in and numbers it; the digest of a fresh secret is
  kept under that number; only then is the secret handed out; "the
  secret a client holds is the sign-in's number and its random bytes";
  secrets from a cryptographic generator seeded at start; digests erased
  when the core says a sign-in ended or expired.
- **An application's providers** (section 8): the provider's sign-in
  route is routed "to that component, through the protocol layer's
  routing, and takes back an identity ... or a refusal".
- **Translation, exhaustive both ways** (section 3): "a module of small
  total functions".
- **Stopping** (`domain/engine.md`, 6.1; `service.md`, section 5): new
  exchanges answered not ready; watches ended as stopping.

## Choices this plan makes

- **Routes** (each a `POST` but the stream; bodies `application/octet-stream`,
  one record each way, status 200 for any record answer):
  `/jig/v1/sign-in/link` (`LinkSignIn` → `SignInAnswer`),
  `/jig/v1/sign-out` (`SignOut` → `SignOutAnswer`), `/jig/v1/request`
  (`Keyed` → `Answer`), `/jig/v1/read` (`Owned` → `ReadAnswer`),
  `/jig/v1/watch` (`Open` → `OpenAnswer`), `/jig/v1/unwatch` (`Close` →
  `CloseAnswer`), `/jig/v1/stream-gone` (`Gone` → empty), `GET
  /jig/v1/stream`, and `/jig/v1/sign-in/provider/<number>` for an
  application's provider. A route not served: 404. A body past its bound:
  413. A record that does not decode: 400.
- **The bearer token** is the sign-in's number (8 bytes, big-endian) and
  32 random bytes, sent as `Authorization: Bearer <base64url, unpadded>`.
  Its digest is `skein_io::digest::digest` of the 40 bytes, compared in
  constant time.
- **The printed link** is
  `http://<listener address>/jig/v1/sign-in/link?secret=<base64url of 32
  random bytes>`, handed to the routing as `Request::Link`, for the shell
  to print. A link stays live for `Limits::link_span`; a native client
  reaching the route with no live link's secret is answered `refused`,
  and a fresh link is printed if none is live, as `client.md`, section 8
  has it for a browser.
- **Digests live in jig 17's secrets' store,** reached through the
  routing, never through a root: `Request::Secrets` asks to keep a
  sign-in's digest and expiry, to erase it, or to get it by the
  sign-in's number (jig 17's `Secrets::{Keep, Erase, Get}` under
  `SecretOwner::SignIn`); `Event::Secrets` answers `Kept`, `Erased`,
  `Got` (in the same iterate, as jig 17 answers a get in the step) or
  `Failed`. The component keeps no table of digests: each exchange's
  token is checked against the digest got by its number. The restart's
  sweep of digests no sign-in names is the store's (jig 17).
- **A sign-in's end is read from its commit:** jig 03 adds no output
  naming an ended sign-in ("the erase in the decision is what the
  protocol layer reads", jig 03.2); the routing hands the component
  `Event::SignInEnded` once a commit that erases `people::Key::SignIn(n)`
  is durable, and the component erases the digest and ends the stream.
- **A sign-in refused above the store's fill mark** (decision 20) comes as
  `Now::SignInRefused`, answered `SignInAnswer::refused`.
- **Cursors are the store's keys:** a page's cursor, the core's
  `reads::Cursor`, is written as its variant and its fields, each
  `core::Key` inside a position by jig 13's `jig_protocol_store::keys::encode`
  and read back by `keys::decode`; the component has no key encoding of
  its own, so a key's change reaches the wire through that one function.
- **The component's machine world is a world crate,** `tests/engine-clients`
  (`jig-engine-clients-world`, ordinary Rust), as jig 23's
  `tests/engine-workers` is: a `Peer` there plays io for the component
  and writes and reads HTTP bytes. The translations' tests are the
  crate's own step tests.
- **The component's limits are in jig 20's profiles:** 31.1 adds them to
  `jig_protocol_config`'s `Limits` and both profiles, so the reference
  service's startup (jig 33) finds them there.
- **The generator** is skein-lib's `CryptoRng` (skein 02, through jig
  02), keyed by the `Secret` seed the shell draws and a stream number the
  settings give.
- **Opening a watch is answered admitted** when the routing reports the
  core's step that took it refused nothing (`Event::Admitted`); a refusal
  in that step comes as `Event::WatchRefused`.
- **Settings are plain values** the routing builds from
  `jig-protocol-config` (jig 20); the component does not depend on it.
- **Stream names and watch tokens are the component's:** a stream is
  named by 16 drawn bytes; each opened watch gets a core watcher token
  from the component's own counter, mapped to (stream, the client's watch
  name).
- **A service's credential is not issued here:** the contract's
  `Made::Service` carries no credential (see Questions).

## Rules that bind it

- "**`jig-protocol-*` crates** depend on no application's crate. Only
  `jig-protocol-charter` and `jig-protocol-transcript` link smith"
  (`code-organization.md`, section 3). This crate links neither smith nor
  any client crate but `jig-protocol-client-wire`.
- "**Step code** follows skein's `programming-model.md`, section 10, and
  jig's `AGENTS.md`: no traits, no generic items, no `dyn`, no
  closures." (`code-organization.md`, section 2)
- "No increment of this plan adds an `unreachable!` arm for a variant the
  reference service can reach; an arm the service reaches is routed."
  (`code-organization.md`, section 6)
- **No secret reaches a domain:** no `core::Event` the component sends
  holds a token, a link's secret or a digest; the test
  `no_core_event_holds_a_token_or_digest` (31.1) checks it.

## Increments

### 31.1 The component, and signing in by the printed link

Scope: the crate `crates/jig-protocol-engine-clients`: `boundary.rs`
(the interface sketch), `component.rs` (`Component::new`, `down`, `up`,
`fire`, `next_deadline`, `reclaim`, `worst_case`), `limits.rs`,
`connection.rs` (one `skein_http::server::Server` per accepted socket,
over io's socket stream records), `routes.rs` (dispatch by method and
target), `sign_in.rs` (the link, the bearer token's checks).
The listener on `Settings::listener`; a start refused when the link is
enabled and the address is not loopback. At construction, the first link
drawn and handed out as `Request::Link`. The link's route: the secret
compared in constant time; spent; `core::Event::SignIn { reply_to,
identity }` with the first owner's identity; on `Event::Reply` with
`people::Reply::SignedIn { person, sign_in, expires }` (jig 03), a fresh
token drawn, its digest kept (`Request::Secrets(Keep)`), and only on
`Event::Secrets(Kept)` the answer `SignInAnswer::signed_in`; on
`Now::SignInRefused`, `refused`. Each later exchange's bearer: its
number's digest got (`Request::Secrets(Get)`) and compared in constant
time; an unknown, absent or unmatched one answered signed out in its
route's answer record. The component's `Limits` added to
`jig_protocol_config::Limits` and its two profiles (jig 20). jig 30's `SignedIn.token` changed from
`bytes 48` to `secret 48`, the code and goldens regenerated. The crate's
rules in `tests/boundary`. Routes not yet served answer 404 until 31.2,
31.4, 31.5 and 31.6 serve them.
Needs: nothing beyond the session's start.
Files: the new crate; `tests/engine-clients/{Cargo.toml, clippy.toml,
src/{lib.rs, peer.rs, fixtures.rs}, tests/sign_in.rs}`;
`crates/jig-protocol-client-wire/{schema/client-v1.schema,
src/generated/v1.rs, golden/v1/}`; `tests/client-wire/tests/drift.rs`;
`crates/jig-protocol-config/src/{profile.rs, profile/small.rs,
profile/standard.rs}` and each literal of `jig_protocol_config::Limits`
(`git grep -n 'Limits {' -- crates/jig-protocol-config testing/jig-test-service`);
the workspace `Cargo.toml`;
`tests/boundary/src/tests.rs`. Shared hotspots: the workspace
`Cargo.toml`, `tests/boundary/src/tests.rs`, jig 30's schema, jig 20's
profiles.
Shape: as the interface sketch; `sign_in::Link { secret: [u8; 32],
until: Time }`; `sign_in::Pending` over `skein_lib::Map<Token, Waiting>`
(exchanges waiting for a digest's get or keep);
`sign_in::constant_time_eq(&[u8], &[u8]) -> bool`.
Test shape: the machine world `tests/engine-clients`, its tests in
`tests/sign_in.rs`, fixtures in `src/fixtures.rs`: `settings()`,
`limits()`, `first_owner()` (a `people::Identity`, provider 0, subject
`b"owner"`), `FIRST_SIGN_IN = 41`, `FIRST_PERSON = 7`; and `src/peer.rs`'s
`Peer`, which plays io for the component (answers `Listen`, makes a
socket per `connect`, carries socket stream records both ways), the core
(scripted answers) and the secrets' store (an in-memory map), and writes
HTTP requests as bytes, reading answers with a
`skein_http::client::Client`. The world's clippy is
`tests/clippy.toml`'s. Tests:
`the_printed_link_signs_the_first_owner_in`,
`a_spent_link_is_refused_and_a_fresh_one_printed`,
`the_token_leaves_only_after_its_digest_is_kept`,
`a_failed_keep_hands_no_token_out`,
`an_unknown_bearer_is_answered_signed_out`,
`two_tokens_drawn_in_turn_differ`,
`a_link_on_a_non_loopback_listener_refuses_the_start`,
`a_sign_in_refused_above_the_fill_mark_is_answered_refused`,
`no_core_event_holds_a_token_or_digest`. Ceilings: each under 20 ms;
the world aims for at most 0.3 s focused for the session.
Reference test, in full:

```rust
#[test]
fn the_token_leaves_only_after_its_digest_is_kept() {
    let mut peer = fixtures::Peer::started(&fixtures::settings());
    let link = peer.printed_link();
    let exchange = peer.post_link(&link);
    peer.answer_signed_in(fixtures::FIRST_SIGN_IN, fixtures::FIRST_PERSON);
    assert_eq!(peer.answered(exchange), None);
    assert_eq!(peer.digest_kept(), Some(fixtures::FIRST_SIGN_IN));
    peer.answer_kept(fixtures::FIRST_SIGN_IN);
    let answer = peer.sign_in_answer(exchange);
    assert_eq!(answer.party, fixtures::FIRST_PERSON);
    assert_eq!(answer.sign_in, fixtures::FIRST_SIGN_IN);
}
```

Proof: world: `the_token_leaves_only_after_its_digest_is_kept`, run once
with `heavy cargo nextest run -p jig-engine-clients-world --test
sign_in`, passes, showing a native client's token is handed out only
once its digest is durable; the joint layer is jig 33's, whose world
needs jig 32. Under 0.2 s for the file.
Don't: set a cookie; serve a page or a file; hand a token, a link secret
or a digest to the core.
Size: at most 600 lines.
Removes: nothing.

### 31.2 Keyed requests: jig's asks

Scope: `/jig/v1/request`: the bearer checked; `Keyed` decoded; jig's
owner: the inner `Ask` record decoded and translated to `people::Ask`
(`translate/asks.rs`), a port of the joint world's `wire/asks.rs`,
`wire/request_arguments.rs` and `wire/offers.rs`, exhaustive over the
wire's asks; `core::Event::People(people::Event::Ask { reply_to,
sign_in, key, ask })`. A record that does not decode, or does not
translate, answered `400` with no core event. An application's owner:
`Request::Application(ToApplication::Request { .. })` with the SHA-256
digest of the record's bytes as they came. Exchanges in flight bounded
per client and for the component (`Limits::exchanges_per_client`,
`Limits::exchanges`); past either, `Answer::busy`.
Needs: nothing beyond 31.1.
Files: `src/{translate.rs, translate/asks.rs, routes.rs, component.rs,
boundary.rs}`; `src/tests.rs`, `src/tests/{asks.rs, fixtures.rs}` (the
translation's step tests); `tests/engine-clients/tests/requests.rs` (the
exchange's).
Shape: `translate::asks::ask(record: v1::Ask) -> Result<people::Ask,
Untranslatable>`; `Untranslatable { path: v1::Path }` naming the field.
Test shape: step tests, one per ask family:
`a_start_chat_record_becomes_the_core_s_start_chat`,
`a_decision_record_keeps_its_revision`,
`an_amendment_record_keeps_its_release_and_reason`,
`an_ask_that_does_not_translate_reaches_no_core`,
`an_application_record_is_routed_with_its_digest`,
`a_third_exchange_past_the_client_s_bound_is_busy`. Expected values
written as the joint world's tests have them for the same values
(`tests/client/src/joint/wire/asks.rs`'s tests). Under 20 ms each.
Proof: unit, `heavy cargo nextest run -p jig-protocol-engine-clients`,
run once: each ask family's record reaches the core as its ask, one that
does not translate reaches no core, and a third exchange past the
client's bound is answered busy (the six tests); under 0.2 s.
Don't: compute a digest over anything but the record's bytes as they
came.
Size: at most 600 lines.
Removes: nothing.

### 31.3 Answers

Scope: the core's answers to a keyed request, translated to the wire
(`translate/answers.rs`): `Held::PeopleReply` with `people::Reply::Outcome`
to `Answer::final` holding `Done` with its `Outcome`, `Made` and
`OwnerOutcome`, or `Refused` with its facts; `Reply::Again` to `busy` or
`not_ready` as its `Again` says; a sign-in ended or absent to
`signed_out`; `Now::SignInRefused` to `SignInAnswer::refused`. A port of
the joint world's `wire/answers.rs`, `wire/outcomes.rs`,
`wire/refusals.rs` and `wire/results.rs`, exhaustive over `people::Reply`,
`Outcome` and `Refusal`. An application's answer from the routing
(`Event::Application(FromApplication::Answered)`) carried as `Owned`
under its family.
Needs: nothing beyond 31.2.
Files: `src/{translate/answers.rs, translate/refusals.rs, routes.rs,
component.rs}`; `src/tests/{answers.rs, fixtures.rs}`;
`tests/engine-clients/tests/requests.rs`.
Shape: `translate::answers::answer(reply: &people::Reply, limits:
&v1::Limits) -> Result<v1::Answer, v1::Problem>`.
Test shape: `a_made_chat_is_answered_done_with_its_task`,
`a_moved_hold_is_refused_with_its_facts`,
`again_busy_is_answered_busy_and_not_kept`,
`an_ended_sign_in_is_answered_signed_out`,
`an_application_outcome_is_carried_under_its_family`,
`a_lost_connection_drops_the_answer_and_nothing_else`. Under 20 ms
each.
Proof: unit, `heavy cargo nextest run -p jig-protocol-engine-clients`,
run once: a made chat is answered done with its task, a moved hold
refused with its facts, busy answered and not kept, and an ended sign-in
answered signed out (the six tests); under 0.2 s.
Don't: answer before `Held::PeopleReply` is released.
Size: at most 600 lines.
Removes: nothing.

### 31.s1 A service's credential

A step not in the plan (decisions.md, ruling 228), from the designer's
answer on jig 31.3 (jig `2dfb78a5`, `protocol/client.md`, 8;
`client/domain/client.md`, 12; `ee79fa5f`, decision 36). Its text is the
designer's.
Scope: an owner who makes a service is answered its credential once; the
service signs in with it at the deployment provider's sign-in route, as
a native client signs in with the link's secret, and makes every
exchange with the sign-in's secret; removing the service ends both.
Needs: 31.3.
Files: `crates/jig-protocol-engine-clients/src/{boundary.rs, sign_in.rs,
component.rs, translate/answers.rs}` and its tests;
`crates/jig-protocol-client-wire/schema/client-v5*.schema`, its
generated code and goldens; `crates/jig-protocol-client-link/src/` where
it hands an answer to its root; the test roots' secrets routing, where a
`SecretsRequest` reaches the store (whose `SecretOwner::Service`,
`crates/jig-protocol-store/src/secrets.rs:46`, is there already).
Shape:
- `jig-protocol-engine-clients`, answering `MakeService` once it is
  committed, draws the credential (`sign_in::token` with the service's
  number: its party, 8 bytes, beside 32 random bytes). It keeps its
  digest in the secrets' store under the service's number, durable
  before the answer goes, and answers `MadeService { party, credential
  }`. `SecretsRequest`'s `Keep`, `Erase` and `Get` name whose secret
  they are, a sign-in's or a service's.
- The deployment provider's sign-in route takes a service's credential
  as a bearer, checks its digest by number in constant time, and signs
  the service in as a sign-in of provider the deployment, subject the
  service (people.md, 3); the core commits it as any other.
- Removing a service erases its digest and ends its sign-ins.
- The wire's `MadeService` gains `credential: secret 40`, in client v5
  (pre-release, development.md 5.1; the version this plan moved).
- `jig-protocol-client-link` hands the credential to its root as a shell
  request beside the answer; the domain sees `Made::Service { party }`
  alone, as the contract has it.
Test shape: `a_made_service_answers_its_credential_once_with_its_digest_kept`,
`a_service_signs_in_with_its_credential_and_exchanges_with_its_sign_in`,
`a_removed_services_credential_is_refused_and_its_sign_ins_end`,
`the_client_domain_never_holds_a_service_credential`. Under 20 ms each;
cases as rows (README, ruling 219).
Proof: the gate.
Don't: put the credential in a contract type, a domain state, a record of
the main store or a log; check an exchange by the credential; answer the
credential twice.
Size: at most 400 lines of production code, generated code excepted.
Removes: nothing.

### 31.s2 A refusal's ending travels as a fact

A step not in the plan (decisions.md, ruling 233): a defect jig 34 found
in 31.3's landed `translate::refusals` (jig `a280d09b`).
Scope: `ending` (`crates/jig-protocol-engine-clients/src/translate/refusals.rs:202-230`)
puts a length-prefixed `w::EndingRow` in `w::Ending.rows`, a table of
facts that the client link splits and decodes as `w::Fact`
(`crates/jig-protocol-client-link/src/translate/facts.rs`, `decode`); a
Done ending's piece `[0, 1]` decodes there as `Fact::Chip`. Each piece
is a `w::Fact::Ending(row)`, measured and encoded as the fact; the
`EndingRows` bound and every owner fact stay.
Needs: 31.3. Run by jig 34's session while it waits on 03.s2, or by
31.s1's before its own work; the coordinator's choice.
Files: `crates/jig-protocol-engine-clients/src/translate/refusals.rs`,
`src/tests/answers.rs` (or the refusals' table); in jig 34, its local
ending helper, removed once this lands.
Test shape: a row in the refusals' table: a keyed `Refusal::Ended`
with a Done ending, encoded by `refusals::refusal` and decoded by the
client link's facts' `decode`, reads back `Fact::Ending` with its
fields. Under 20 ms.
Proof: the gate.
Don't: change the wire schema or the `EndingRows` bound; keep two
ending encoders once this lands.
Size: at most 20 lines of production code.

### 31.4 Reads and cursors

Scope: `/jig/v1/read`: the `Owned` read decoded; jig's read translated
to `reads::Read` (`translate/reads.rs`, a port of the joint world's
`wire/reads.rs`), with its cursor decoded (`translate/cursor.rs`: the
core's `reads::Cursor` written as its variant and fields, each
`core::Key` by jig 13's `jig_protocol_store::keys::encode` and read by
`keys::decode`, as the joint world's `wire/cursor.rs` lays the variants
out; one that does not decode answered `none: unknown`);
`core::Event::Read { reply, sign_in, read }`; the answer from
`Held::Read` through 34.1 to 34.4's functions, or `none` with the core's
`Why` (signed out included, jig 03). An application's read routed
(`ToApplication::Read`) and answered from the routing. 34.3's cursor
stub replaced by the encoder.
Needs: jig 34's pages with their cursor stub (34.3) and single reads
(34.4), beyond the session's start; jig 13's `jig_protocol_store::keys`
is on `main` at the session's start (jig 17 needs it).
Files: `src/{translate/reads.rs, translate/cursor.rs, translate/pages.rs,
routes.rs, component.rs}`; the crate's `Cargo.toml`
(`jig-protocol-store`, for its keys alone); `src/tests/{reads.rs,
fixtures.rs}`; `tests/engine-clients/tests/reads.rs`.
Shape: `translate::cursor::{encode(cursor: &reads::Cursor, limit: u32)
-> Option<Box<[u8]>>, decode(bytes: &[u8]) -> Option<reads::Cursor>}`.
Test shape: `a_read_is_answered_only_once_held_is_released`,
`a_cursor_round_trips_to_the_same_position`,
`a_cursor_that_does_not_decode_is_answered_unknown`,
`a_read_signed_out_is_answered_signed_out`,
`an_application_read_is_routed_and_answered_under_its_family`. Under
20 ms each.
Proof: unit, `heavy cargo nextest run -p jig-protocol-engine-clients`,
run once: a read is answered only once `Held::Read` is released, a
cursor round-trips to its position, and one that does not decode is
answered unknown (the five tests); under 0.2 s.
Don't: answer a read before `Held::Read`.
Size: at most 600 lines.
Removes: 34.3's cursor stub.

### 31.5 Watches and the stream

Scope: `/jig/v1/watch`: `Open` decoded, its subject translated
(`translate/watches.rs`, a port of the joint world's `wire/watches.rs`)
to `core::Event::Watch` or `WatchRun`; answered admitted on
`Event::Admitted`, refused on `Event::WatchRefused`. `/jig/v1/unwatch`:
`core::Event::Unwatch`, answered once its ending comes.
`/jig/v1/stream-gone`: the stream's watches unwatched. `GET
/jig/v1/stream` (`stream.rs`): one per sign-in; a
`skein_http::sse::writer::Writer` over the response body; the hello first
(the stream's name, the served ranges, the heartbeat); `Held::Delivery`
written as a `delivery` event, then `core::Event::Delivered`;
`Now::RunText` written as `text`, then `core::Event::StreamDelivered`;
`Held::WatchEnded` written as `ended`; while the writer waits for room,
nothing more written for that stream; a heartbeat comment after
`Settings::heartbeat` of silence; each event's data the record in base64
(`skein_lib::base64`, `Alphabet::Standard`). A stream that ends ends its
watches with `core::Event::Unwatch`; a watch ended signed out ends its
stream.
Needs: nothing beyond 31.4.
Files: `src/{stream.rs, translate/watches.rs, routes.rs, component.rs,
connection.rs, limits.rs}`; `src/tests/{watches.rs, fixtures.rs}`;
`tests/engine-clients/{src/peer.rs, tests/stream.rs}`.
Shape: `stream::Stream { name: [u8; 16], writer: Writer, watches:
Set<Token>, waiting: bool }`; `Limits::{streams, watches_per_stream,
queued_bytes}`.
Test shape: `a_stream_opens_with_its_hello`,
`a_slow_reader_gets_the_next_delivery_only_after_the_last`,
`a_delivery_is_marked_taken_only_once_written`,
`a_stream_cut_mid_event_ends_its_watches`,
`a_silent_stream_carries_a_heartbeat_comment`,
`run_text_leaves_without_waiting_for_a_commit`,
`a_watch_refused_in_its_step_is_answered_refused`. Under 20 ms each.
Reference test of this kind, in full:

```rust
#[test]
fn a_delivery_is_marked_taken_only_once_written() {
    let mut peer = fixtures::Peer::signed_in(&fixtures::settings(), fixtures::FIRST_SIGN_IN);
    let stream = peer.open_stream();
    let watch = peer.open_watch(stream, fixtures::frame_subject());
    peer.stop_reading(stream);
    peer.release_delivery(watch, fixtures::frame_document());
    assert_eq!(peer.taken(watch), 0);
    peer.read_all(stream);
    assert_eq!(peer.taken(watch), 1);
}
```

Proof: world, `heavy cargo nextest run -p jig-engine-clients-world
--test stream`, run once: a stream opens with its hello, a slow reader
gets the next delivery only after the last, and a stream cut mid-event
ends its watches (the seven tests); under 0.3 s.
Don't: keep a backlog: one delivery per watch waits at most, and the
core coalesces the rest.
Size: at most 600 lines.
Removes: nothing.

### 31.6 Signing out, providers, stopping and closing

Scope: `/jig/v1/sign-out`: `people::Event::SignOut`, the digest erased
(`Request::Secrets(Erase)`), answered `signed_out`; `Event::SignInEnded`
(the routing's word, read from a durable commit that erases
`people::Key::SignIn(n)`) erases its digest and ends its stream. The
restart's sweep is the store's (jig 17), not the component's. `/jig/v1/sign-in/provider/<n>`:
`Request::Provider { exchange, provider, call }` to the routing, the
answer `Event::Provider { exchange, verdict }`: an identity signs in as
the link does, a refusal answers `SignInAnswer::refused`; an unconfigured
provider 404. `Event::Stop`: the listener stops admitting, new exchanges
answered `not_ready`, open streams go on; `Event::Close`: each stream
ended, each connection closed once its last bytes are written, then
`Request::Closed`; `Event::Abort`: dropped at once, then
`Request::Closed`.
Needs: nothing beyond 31.5.
Files: `src/{sign_in.rs, routes.rs, component.rs, boundary.rs}`;
`tests/engine-clients/tests/{sign_in.rs, closing.rs}`.
Shape: as the interface sketch.
Test shape: `signing_out_erases_the_digest_and_refuses_the_token`,
`a_sign_in_whose_record_is_erased_ends_its_stream`,
`a_provider_identity_signs_in_as_the_link_does`,
`a_provider_refusal_signs_nobody_in`,
`after_stop_a_new_exchange_is_answered_not_ready`,
`close_ends_each_stream_then_says_closed`. Under 20 ms each.
Proof: world, `heavy cargo nextest run -p jig-engine-clients-world
--test closing`, run once: signing out erases the digest and refuses the
token, a provider's identity signs in as the link does, and a close ends
each stream, then says closed (the six tests); under 0.2 s. The joint
layer runs in jig 33's `tests/client-protocol`.
Don't: clear anything on the client's side (the shell forgets its
token).
Size: at most 500 lines.
Removes: nothing.

## Interface sketches

```rust
// jig_protocol_engine_clients::boundary: what the protocol layer routes in and out.
use jig_domain_core as core;
use jig_domain_core::documents::Document;
use jig_domain_core::reads::Answer as ReadAnswer;
use jig_domain_people as people;
use jig_domain_views as views;
use skein_lib::{Duration, List, ReplyTo, Secret, Token};

/// Into the component.
pub enum Event {
    /// io's events for the listener and the connections.
    Io(skein_io::Event),
    /// `Held::PeopleReply`, released: answers a keyed request, a sign-in, a sign-out.
    Reply { to: ReplyTo, reply: people::Reply },
    /// `Now::SignInRefused`: the sign-in's exchange is answered refused.
    SignInRefused { to: ReplyTo },
    /// The routing: a durable commit erased `people::Key::SignIn(sign_in)`;
    /// its digest is erased and its stream ended.
    SignInEnded { sign_in: u64 },
    /// `Held::Read`, released: a read's one answer.
    Read { reply: ReplyTo, answer: ReadAnswer },
    /// The routing: the core's step that took this watch refused nothing.
    Admitted { watcher: Token },
    /// `Now::WatchRefused`.
    WatchRefused { watcher: Token, why: core::WatchRefusal },
    /// `Held::Delivery`, released; `Delivered` goes back once written.
    Delivery { watcher: Token, document: Box<Document> },
    /// `Held::WatchEnded`.
    WatchEnded { watcher: Token, why: views::End },
    /// `Now::RunText`: provisional; `StreamDelivered` goes back once written.
    RunText { watcher: Token, turn: views::TurnName, from: u32, to: u32, text: Box<[u8]>, cut: bool, call: Option<views::Call> },
    /// The routing's answer about an application's record.
    Application(FromApplication),
    /// The routing's answer from the store's secrets' records.
    Secrets(SecretsAnswer),
    /// The routing's answer from an application's provider.
    Provider { exchange: Token, verdict: Verdict },
    /// Stop admitting; close draining; abort at once. Each ends with `Request::Closed` but Stop.
    Stop,
    Close,
    Abort,
}

/// Out of the component.
pub enum Request {
    Io(skein_io::Request),
    /// To the core, in its vocabulary: asks, reads, watches, taken, sign-ins, sign-outs.
    Core(core::Event),
    /// An application's record as bytes, for the routing to decode with its wire.
    Application(ToApplication),
    Secrets(SecretsRequest),
    /// An application's provider's sign-in exchange.
    Provider { exchange: Token, provider: u16, call: ProviderCall },
    /// A sign-in link for the shell to print: at start, and a fresh one when needed.
    Link { link: Box<[u8]> },
    /// After Close or Abort: nothing of the component is left in io.
    Closed,
}

pub enum ToApplication {
    Request { reply_to: ReplyTo, sign_in: u64, key: [u8; 16], family: u16, version: u32, record: Box<[u8]>, digest: [u8; 32] },
    Read { reply: ReplyTo, sign_in: u64, family: u16, version: u32, record: Box<[u8]> },
    Watch { watcher: Token, sign_in: u64, family: u16, version: u32, record: Box<[u8]> },
}
pub enum FromApplication {
    /// A final answer, the application's record encoded by its wire.
    Answered { to: ReplyTo, family: u16, record: Box<[u8]> },
    Read { reply: ReplyTo, family: u16, record: Option<Box<[u8]>> },
    Delivery { watcher: Token, family: u16, record: Box<[u8]> },
    /// The record did not decode: answered 400, nothing reached the root.
    Undecodable { to: ReplyTo },
}
/// To jig 17's secrets' store, under `SecretOwner::SignIn`, through the routing.
pub enum SecretsRequest { Keep { sign_in: u64, value: Kept }, Erase { sign_in: u64 }, Get { sign_in: u64 } }
/// Its answers; `Got` comes in the iterate that asked.
pub enum SecretsAnswer { Kept { sign_in: u64 }, Erased { sign_in: u64 }, Got { sign_in: u64, value: Option<Kept> }, Failed }
/// A sign-in's kept value: its token's digest and its expiry.
pub struct Kept { pub digest: Secret, pub expires: skein_lib::Wall }
pub struct ProviderCall { pub target: Box<[u8]>, pub body: Box<[u8]> }
pub enum Verdict { Identity(people::Identity), Refused }

pub struct Settings {
    /// io's address (skein 01): a loopback socket address when the link is enabled.
    pub listener: skein_io::Addr,
    /// The first owner the printed link signs in; None: no link.
    pub link: Option<people::Identity>,
    pub heartbeat: Duration,
    /// jig's served range, and each family's, by number.
    pub served: List<Served>,
    /// The application's providers by number; their routes 404 otherwise.
    pub providers: List<u16>,
    pub seed: Secret,
    pub stream: u64,
}
pub struct Served { pub family: Option<u16>, pub first: u32, pub last: u32 }
```

- **Who owns what:** the component owns its listener, connections,
  streams and the live link; the secrets' store owns the digests; the
  core owns sign-ins,
  their numbers and their ends; the routing owns an application's codecs,
  its providers and the secrets' store's commits.
- **What ends what:** an exchange ends with its one answer or its
  connection; a stream with its connection, a stream-gone exchange, its
  sign-in's end, or `Close`; the component with `Request::Closed`.

## Questions to raise when reached

- **A service's credential:** settled by the designer (jig `2dfb78a5`;
  `client.md`, section 8). The contract's `Made::Service { party }` stays
  as it is: the credential crosses beside it on the wire, to the owner's
  shell, never the client's domain; a service signs in with it at the
  deployment provider's sign-in route. A step of the supervisor's after
  31.3 builds it; 31.3 answers `MakeService` with no credential until
  then.
- **The link's span and the heartbeat** are limits: report the numbers
  chosen.

## Done when

- The component's tests pass: `heavy cargo nextest run -p
  jig-protocol-engine-clients -p jig-engine-clients-world` passes.
- It links no smith and no client domain: `git grep -n -E
  '^(smith-|jig-domain-client|jig-view)' --
  crates/jig-protocol-engine-clients/Cargo.toml` prints nothing.
- No cookie and no page: `git grep -n -i -E 'set-cookie|text/html' --
  crates/jig-protocol-engine-clients/src` prints nothing.
- The translation has no wildcard: `heavy cargo clippy -p
  jig-protocol-engine-clients --all-targets -- -D warnings` passes with
  the workspace's `wildcard_enum_match_arm` denied.
- The gate passed on this session's last landing: `git -C ~/src/rust/jig
  log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step:.jig-31\.' origin/main | head -1` names
  `scripts/check.sh passed`.
