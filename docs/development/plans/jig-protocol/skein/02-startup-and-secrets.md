# Session 02: secrets, base64, the cryptographic seed and generator

skein builds the pieces a service needs to hold secrets: lib's `Secret`,
whose `Debug` shows only its length, and lib's `base64`, which skein's
own copies move onto; the generator's `secret N`, held as a `Secret`;
the shell's `read_private_file` and its 32-byte cryptographic seed;
lib's `CryptoRng`, keyed by that seed and a stream number, from which
skein's OAuth driver draws its states and PKCE verifiers; and the
driver's rule that a record failed as "sign in again" is dropped, with
the world stories oauth.md asks for. jig's protocol layer is the first
user of each (jig's `protocol/README.md`, section 8; `configuration.md`,
section 7; `llm.md`, 5.4; `client.md`, sections 1 and 8).

## Read first

- skein's `lib.md`: section 2 (no 64-bit `usize` assumed), 6 (`Secret`),
  6.1 (base64), 8 (`Rng` and `CryptoRng`, revised 2026-10-12: "Each
  generator is made from the seed and a stream number its owner gives")
  and 13.
- skein's `codec.md`, section 3 (`secret N`, "A credential is a `secret
  N`") and section 4 (a `secret` field's `Debug`); section 6 (drift).
- skein's `shell.md`, section 5 (the cryptographic seed) and 6.1
  (`read_private_file`); `io.md`, 5.3 (the private-file checks, and the
  path-only open that names a denied file).
- skein's `oauth.md`: section 2 (the verifier from the caller's
  `CryptoRng`), 6.1 (the owner seeds the generator), 6.3 (hand in,
  expiring), 6.4 ("Loading again") and 6.8 (the protocol world's
  stories).
- `programming-model.md`, sections 9 (randomness is injected state) and
  10.2 (what lib may use beyond the subset).
- `findings/skein.md`, sections 2 to 5 and 7.

## What the design asks

- **`Secret`** (lib.md, 6): "Its `Debug` shows its length only, never its
  bytes"; "Its owner reads the bytes with `expose`"; "Clone, equality and
  `Hash` are over its bytes"; it neither zeroes nor locks memory.
- **`base64`** (lib.md, 6.1): RFC 4648; `Alphabet::Standard` padded,
  `Alphabet::Url` unpadded; "`encoded_len` gives the text's length before
  anything is set aside, and `encode` writes into a `Writer`"; `decode`
  "takes a maximum of decoded bytes and refuses past it before setting
  anything aside", and refuses "a symbol outside the alphabet; padding
  where the alphabet takes none, or anywhere but the end; a length no
  encoding has; trailing bits that are not zero"; a `Decoder` takes text
  in pieces. "skein's crates use it and keep no copy of their own"
  (lib.md, 13: "`skein-oauth`'s PKCE and claims, `skein-shell`'s PEM and
  `skein-browser`'s screenshots").
- **`secret N`** (codec.md, 3): encoded as `bytes N`, generated as
  `lib::Secret`; a record's `Debug` shows a `secret` field's length.
- **`read_private_file`** (shell.md, 6.1): read as `read_file` reads,
  after io's checks on what it opened: a regular file reached through no
  link in any component of its path, no group or other bits, one link,
  owned by the effective user; "Anything else is refused unread"; the
  refusal "says what was unsafe, as io's `Unsafe` names it"; a denied
  open is named "by a path-only open whose stat decides nothing".
- **The cryptographic seed** (shell.md, 5): "32 bytes, drawn from
  `getrandom` once at startup, beside the seed and never derived from
  it, and handed to the service's startup as a `Secret`"; "a world draws
  it from its own seed instead".
- **`CryptoRng`** (lib.md, 8): ChaCha20 keyed by the 32-byte seed; "`fill`
  writes fresh bytes into a bounded buffer"; "Whatever must not be
  guessed is drawn from it: secrets, OAuth states and PKCE verifiers";
  each generator made from the seed and a stream number, "distinct for
  every generator of a process".
- **OAuth** (oauth.md, 2 and 6.1): the verifier "is supplied by the
  caller's seeded cryptographic generator"; the owner "seeds its
  cryptographic generator ..., from which each sign-in's state and
  verifier are drawn".
- **Loading again** (oauth.md, 6.4): "A refresh that fails as sign in
  again drops the account's record, whose refresh token is then dead, so
  the account holds none and loads again before its next grant."
- **Handed in** (oauth.md, 6.8): the protocol world holds "an
  access-only record expiring and handed in again".

## Choices this plan makes

- `Secret` is `skein_lib::Secret(Box<[u8]>)`, a one-field newtype; its
  `Debug` prints `Secret { len: N }`.
- `base64` adds `decoded_len`, which checks a text and measures it
  without setting anything aside, for JWT's header and signature, which
  `jwt.rs:27-29` checks and never decodes.
- `base64::Decoder` holds its own buffer, at most its maximum, and
  `finish` hands the decoded bytes over; PEM's reader keeps its own line
  and whitespace handling (`startup.rs:157-176`) and feeds the decoder
  the runs between.
- Lengths in `base64` and `CryptoRng` are `u32`, converted with
  `try_from`, and ChaCha20 runs on `u32` words (lib.md, 2;
  findings/skein.md, section 7), so the later wasm32 build needs no
  change. Nothing here builds for wasm32.
- `read_private_file(path, max, user)` takes the expected owner from its
  caller, who reads `effective_user()` once at startup, and answers a
  `Secret`; its refusal is a new `Unread::Unsafe(skein_io::file::Unsafe)`.
- The cryptographic seed is `skein_shell::crypto_seed() -> Result<Secret,
  i32>`; a world's is `skein_world::crypto_seed(seed: u64) -> Secret`,
  32 bytes drawn from `Rng::new(seed)`.
- ChaCha20's state: words 0–3 the constant, 4–11 the seed, 12–13 a 64-bit
  block counter from 0, 14–15 the stream number, little-endian (the
  original ChaCha layout; RFC 8439's 96-bit nonce is words 13–15). So
  RFC 8439's test vectors with a zero first nonce word apply.
- `CryptoRng::new(seed: &Secret, stream: u64) -> Option<CryptoRng>`:
  `None` unless the seed is 32 bytes.
- `skein_oauth_accounts::Component::new(accounts, limits, crypto:
  CryptoRng)`: the owner makes the generator, with its own stream number.
  A sign-in's verifier is 32 drawn bytes in base64url, 43 characters
  (oauth.md, 2: "43–128"); its state is `state_bytes.min(32) * 3 / 4`
  drawn bytes in base64url, at most `state_bytes` characters.
- The record is dropped where a refresh's failure is handled
  (`fail_exchange`, `component.rs:827-880`), for a refresh whose failure
  is `Failure::Exchange(skein_oauth::Failure::Refused)`; a rejected token
  rejected again (`Expired`) keeps its record, as today.

## Rules that bind it

- "**No browser, no wasm32, no web shell,** in any session" (README,
  "Rules"). 02.1 edits skein's existing `testing/skein-browser` only to
  drop its base64 copy (lib.md, 13): it adds no browser and builds
  nothing for wasm32; skein's gate already runs the browser suite.
- "**No increment of this plan** ... computes an expected value with the
  code under test" (code-organization.md, section 6): base64's and
  ChaCha20's expected values are the RFCs' literals.
- "**Shared hotspots with smith's reliability plan** ...
  `crates/skein-shell/` (its terminal modes, skein 02.6 of that plan).
  `land` rebases each landing on the other's; no session waits for it"
  (README, "Rules"). Within this plan, skein 01.1 edits
  `crates/skein-oauth-accounts/src/component.rs:706` and
  `component/sign_in.rs:118-121`, which 02.4 and 02.5 edit too.
- Files already past 800 lines (`crates/skein-oauth-accounts/src/component.rs`,
  1,116; `crates/skein-shell/src/ring.rs`, 1,731;
  `tests/oauth-accounts/src/world.rs`, 1,153;
  `tests/oauth-accounts/src/sign_in.rs`, 883) grow by at most what each
  increment names and are not split here; this replaces
  code-organization.md, section 7's split for them. New code goes in new
  modules.

## Increments

### 02.1 `Secret`, `base64`, and skein's copies on it
Scope: lib.md, 6 and 6.1; lib.md, 13's copies moved: PKCE's challenge
  (`skein-oauth`), JWT's claims (`skein-oauth`), PEM (`skein-shell`), the
  browser kit's screenshots, and `tests/tls`'s test encoder.
Needs: nothing.
Files: new `crates/skein-lib/src/secret.rs` and `src/base64.rs`;
  `crates/skein-lib/src/lib.rs:18-60` (`pub mod base64;`, `pub use
  secret::Secret;`); `src/tests.rs` (module list), new
  `src/tests/base64.rs`, `src/tests/bytes.rs` (`Secret`'s tests);
  `tests/lib/tests/fuzzy_bytes.rs` (one round-trip test);
  `crates/skein-oauth/src/pkce.rs:73,84-118` (the alphabet and the encode
  loop go); `crates/skein-oauth/src/jwt.rs:27-29,57-134` (`sextet`,
  `decoded_len`, `decode` go; `base64::Error::TooLong` is
  `DecodeError::TooLarge`, any other `DecodeError::Malformed`);
  `crates/skein-shell/src/startup.rs:125-151,198-262` (the `Decoder`
  goes; `pem()` feeds `base64::Decoder`, keeping its counts,
  `RootsRefusal::TooLong { index, bound }` from `TooLong`, and a block
  malformed on any other error); `crates/skein-shell/Cargo.toml` (skein-lib,
  if not there); `testing/skein-browser/src/wire/base64.rs` (removed),
  `src/wire.rs:3`, `src/browser.rs:1676`; `tests/tls/tests/startup.rs:11-24`.
Shape: as the sketches below. `Secret` derives `Clone`, `PartialEq`,
  `Eq`, `Hash`, and implements `Debug` by hand (lib may,
  programming-model.md, 10.2). `base64::encode` writes `encoded_len`
  bytes or refuses with `Overflow`, writing nothing. `Decoder::push`
  carries what a piece leaves of a group; `finish` refuses a text ended
  mid-group or without its padding.
Test shape: `crates/skein-lib/src/tests/base64.rs`, beside the bytes
  tests (lib.md, 10). Reference test:

  ```rust
  #[test]
  fn rfc_4648_vectors_encode_in_the_standard_alphabet_with_padding() {
      let vectors: [(&[u8], &[u8]); 7] = [
          (b"", b""),
          (b"f", b"Zg=="),
          (b"fo", b"Zm8="),
          (b"foo", b"Zm9v"),
          (b"foob", b"Zm9vYg=="),
          (b"fooba", b"Zm9vYmE="),
          (b"foobar", b"Zm9vYmFy"),
      ];
      for (bytes, text) in vectors {
          let len = base64::encoded_len(Alphabet::Standard, u32::try_from(bytes.len()).expect("short"));
          let mut writer = Writer::new(usize::try_from(len.expect("fits a u32")).expect("fits a usize"));
          base64::encode(Alphabet::Standard, bytes, &mut writer).expect("room measured first");
          assert_eq!(writer.finish().as_ref(), text, "RFC 4648, section 10");
      }
  }
  ```

  The others: `rfc_4648_vectors_decode_back_in_the_standard_alphabet`,
  `the_url_alphabet_encodes_without_padding` (`[0xfb, 0xff]` is `-_8`),
  `decode_refuses_a_symbol_outside_the_alphabet_at_its_offset`,
  `decode_refuses_padding_in_the_url_alphabet`,
  `decode_refuses_a_length_no_encoding_has`,
  `decode_refuses_trailing_bits_that_are_not_zero`,
  `decode_refuses_a_text_past_its_maximum`,
  `pieces_cut_at_each_offset_decode_as_one_text`; in
  `src/tests/bytes.rs`, `a_secrets_debug_shows_its_length_and_not_its_bytes`
  and `secrets_compare_and_hash_over_their_bytes`; each under 20 ms. In
  `tests/lib/tests/fuzzy_bytes.rs`,
  `random_bytes_round_trip_through_both_alphabets`, at most 1 s. The
  moved users' tests keep their names and expected values:
  `crates/skein-oauth/src/pkce.rs:174-196`,
  `crates/skein-oauth/src/tests.rs:193`
  (`base64url_mutations_and_claim_shape_errors_are_refused_without_authentication`),
  `tests/ring/tests/files/startup.rs`, `tests/tls/tests/startup.rs`, the
  browser suite.
Proof: unit and real filesystem, run once with `heavy cargo nextest run
  -p skein-oauth -p skein-ring-tests`: the RFC 7636 example's challenge
  (`rfc_7636_example_uses_s256_without_padding`) and the PEM bundles'
  certificates (`pem_takes_der_in_order_and_skips_other_blocks_without_decoding_them`,
  `malformed_certificate_blocks_recover_at_the_next_begin_and_do_not_consume_the_count_bound`)
  come out as the literals they held before the move. Under 30 s. The
  browser suite runs in `land`'s gate only.
Don't: change `skein_oauth::SavedToken`, `Granted.token` or any other
  public record to `Secret` (smith's ~15 record sites would break,
  findings/skein.md, section 3); change PEM's accepted inputs (its
  whitespace, `0x0b` included, stays skipped).
Size: at most 300 lines of production code added; the copies' removal
  takes about 230.
Removes: `pkce.rs`'s `ALPHABET` and encode loop; `jwt.rs`'s `sextet`,
  `decoded_len` and `decode`; `startup.rs`'s `Decoder`;
  `testing/skein-browser/src/wire/base64.rs`; `tests/tls`'s encoder.

### 02.2 The generator's `secret N`
Scope: codec.md, 3 and 4: `secret N` encoded as `bytes N`, its field a
  `lib::Secret`, a record's derived `Debug` showing its length.
Needs: nothing beyond 02.1 (`Secret`).
Files: `testing/skein-codegen/src/lib.rs:68-81,99,123` (`Type::Secret`),
  `src/parse.rs:118`, `src/check.rs:123`,
  `src/emit_codec.rs:25,36,76,109,167`, `src/emit_limits.rs:83`,
  `src/emit_tests.rs:56,98,177,227`, `src/emit_types.rs:16,26,41,60,69`,
  `src/emit_worst.rs:16,30,46` (each `Type::Bytes` arm gains its
  `Type::Secret` twin); new `tests/codec/schema/secrets-v1.schema`, its
  generated `tests/codec/src/secrets/v1.rs` and goldens
  `tests/codec/golden/v1/record_credential_*.bin`;
  `tests/codec/src/lib.rs` (`#[path = "secrets/v1.rs"] pub mod
  secrets;`); `tests/codec/tests/drift.rs` (one test);
  `tests/codec/tests/wire.rs`.
Shape: the schema:

  ```
  family secrets 1

  record Credential {
      name: text 64
      value: secret 64
  }
  ```

  The generated `Credential` takes `value: Secret` in `CredentialParts`,
  answers `fn value(&self) -> &Secret`, encodes `value.expose()` as a
  `bytes 64`, and decodes into `Secret::new`; its worst heap is a `bytes
  64`'s. Output for a schema without a `secret` field is unchanged, byte
  for byte.
Test shape: `tests/codec/tests/drift.rs` gains
  `secret_schema_code_and_goldens_have_no_drift`, as its three others
  (`drift.rs:40-54`); the generated bounds tests cover `value` at its
  limit and past it. Reference test, in `tests/codec/tests/wire.rs`:

  ```rust
  #[test]
  fn a_credentials_debug_shows_its_secrets_length_and_not_its_bytes() {
      let parts = CredentialParts { name: Box::from(&b"llm"[..]), value: Secret::new(Box::from(&b"hunter2"[..])) };
      let credential = Credential::new(&CEILINGS, parts).expect("within its limits");
      let shown = format!("{credential:?}");
      assert!(!shown.contains("hunter2"), "the secret's bytes are not shown: {shown}");
      assert!(shown.contains("len: 7"), "its length is: {shown}");
  }
  ```

  And `a_credential_encodes_its_secret_as_bytes_and_decodes_back`, the
  bytes written as a literal (`0, 0, 0, 3, b'l', b'l', b'm', 0, 0, 0, 7,`
  then `hunter2`). Each under 20 ms.
Proof: unit, run once with `heavy cargo nextest run -p skein-codec-tests`:
  the three existing schemas' drift tests pass with their committed code
  and goldens untouched, and `git diff --stat origin/main --
  tests/codec/src/generated tests/codec/src/scalars tests/codec/src/lints`
  prints nothing. Under 30 s. smith's own drift tests run at smith 01.
Don't: change the generator's output for any schema without a `secret`
  field; edit `sample-v1.schema`, whose goldens are frozen.
Size: at most 150 lines of production code; the generated file and the
  goldens are not counted (README, "Rules").
Removes: nothing.

### 02.3 The private read and the cryptographic seed
Scope: shell.md, 6.1 (`read_private_file`) and 5 (the cryptographic
  seed).
Needs: nothing beyond 02.1 (`Secret`).
Files: `crates/skein-shell/src/startup.rs:18-35` (`Unread::Unsafe`,
  `read_private_file`); `src/ring/startup.rs` (the read: `openat2` with
  `RESOLVE_NO_SYMLINKS` from `AT_FDCWD`, as `open_append` calls
  `openat2` at `ring.rs:1609-1633`; the checks; on `EACCES`, the
  path-only `O_PATH | O_NOFOLLOW` open and its stat; at most 200 lines,
  so `ring.rs` does not grow); `src/seed.rs` (`crypto_seed`);
  `src/ring.rs:1654-1666` (`random` fills a caller's buffer; at most 15
  lines changed); `src/lib.rs:34-41` (exports);
  `tests/ring/tests/files/startup.rs`; `tests/ring/tests/ring.rs:195-204`.
Shape: `pub fn read_private_file(path: &Path, max: u32, user: u32) ->
  Result<Secret, Unread>`. The checks, and their order, are io's for a
  private file (`crates/skein-io/src/file_layer/private.rs`), on the
  descriptor it then reads: so startup and io name one file alike. A
  link anywhere in the path (`ELOOP` under `RESOLVE_NO_SYMLINKS`) is
  `Unsafe::Link`. `pub fn crypto_seed() -> Result<Secret, i32>`: 32
  bytes from one `getrandom`. `read_file` and `seed` are unchanged.
Test shape: `tests/ring/tests/files/startup.rs` (real files in a
  `Scratch`, as its `read_file` tests at `:20-40`). Reference test:

  ```rust
  #[test]
  fn a_file_with_group_or_other_bits_is_refused_unread_as_mode() {
      let scratch = Scratch::new("private");
      let path = scratch.path().join("token");
      fs::write(&path, b"secret").expect("plant the token file");
      fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).expect("make it group-readable");
      let user = skein_shell::effective_user();
      assert_eq!(read_private_file(&path, 64, user), Err(Unread::Unsafe(Unsafe::Mode(0o040))));
  }
  ```

  The others: `a_private_file_is_read_within_its_bound`,
  `a_symbolic_link_at_the_file_is_refused_as_link`,
  `a_symbolic_link_in_a_parent_is_refused_as_link`,
  `a_file_with_two_links_is_refused_as_links`,
  `a_directory_and_a_fifo_are_refused_as_their_kind`,
  `a_file_owned_by_another_expected_user_is_refused_as_owner` (the
  expected user passed as `effective_user() + 1`),
  `a_denied_own_file_is_named_by_its_path_only_stat` (skipped, saying
  so, when the process has `CAP_DAC_OVERRIDE`, read as
  `tests/io/tests/ring.rs:148-151` reads it),
  `private_reads_leave_no_descriptor_open` (the `/proc/self/fd` count, as
  `:28,37`); in `tests/ring/tests/ring.rs`,
  `two_cryptographic_seeds_differ_and_hold_32_bytes`. Each under 20 ms.
Proof: real filesystem, run once with `heavy cargo nextest run -p
  skein-ring-tests --test files`: a token file that is group-readable,
  reached through a link, or linked twice is refused unread with the
  cause named, and a private one is read, its bytes returned. Under 30 s.
Don't: read `effective_user()` inside `read_private_file`; follow a
  link; read a byte before the checks pass.
Size: at most 220 lines of production code.
Removes: nothing.

### 02.4 `CryptoRng`, and OAuth's states and verifiers drawn from it
Scope: lib.md, 8 (`CryptoRng` by stream number); oauth.md, 2 and 6.1;
  shell.md, 5's world seed.
Needs: nothing beyond 02.1 (`Secret`, `base64`).
Files: new `crates/skein-lib/src/crypto_rng.rs`; `crates/skein-lib/src/lib.rs`
  (`pub use crypto_rng::CryptoRng;`); new `src/tests/crypto_rng.rs`;
  `crates/skein-oauth-accounts/src/component.rs:117,124,199` (`random:
  Rng` becomes `crypto: CryptoRng`; at most 10 lines);
  `src/component/sign_in.rs:109,111,187-195` (`draw` fills and encodes);
  the users of `Component::new` (12): `src/private_tests.rs:46`,
  `src/tests.rs:70,415,449,497,508`, `tests/oauth-accounts/src/sign_in.rs:202`,
  `src/world.rs:331`, `tests/oauth-accounts/tests/memory.rs:71,151,206,311`;
  `testing/skein-world/src/lib.rs` (`crypto_seed`). Shared hotspot:
  `component.rs` and `sign_in.rs` (skein 01.1).
Shape: the sketch below. `fill` keeps the unused rest of its last block
  for the next call, so fills in pieces give one stream. In
  `crates/skein-oauth-accounts/src/tests.rs` (the crate's fixtures),
  `fn crypto(n: u8) -> CryptoRng`, the seed 32 bytes of `n`, stream 0;
  the worlds take `skein_world::crypto_seed(seed)`, stream 0.
Test shape: `crates/skein-lib/src/tests/crypto_rng.rs`. Reference test,
  RFC 8439, appendix A.1, test vector #1:

  ```rust
  #[test]
  fn a_zero_seed_on_stream_zero_gives_rfc_8439_vector_one() {
      let seed = Secret::new(bytes::copy_of(&[0; 32]));
      let mut rng = CryptoRng::new(&seed, 0).expect("a 32-byte seed");
      let mut block = [0_u8; 64];
      rng.fill(&mut block);
      assert_eq!(
          block,
          [
              0x76, 0xb8, 0xe0, 0xad, 0xa0, 0xf1, 0x3d, 0x90, 0x40, 0x5d, 0x6a, 0xe5, 0x53, 0x86, 0xbd, 0x28,
              0xbd, 0xd2, 0x19, 0xb8, 0xa0, 0x8d, 0xed, 0x1a, 0xa8, 0x36, 0xef, 0xcc, 0x8b, 0x77, 0x0d, 0xc7,
              0xda, 0x41, 0x59, 0x7c, 0x51, 0x57, 0x48, 0x8d, 0x77, 0x24, 0xe0, 0x3f, 0xb8, 0xd8, 0x4a, 0x37,
              0x6a, 0x43, 0xb8, 0xf4, 0x15, 0x18, 0xa1, 0x1c, 0xc3, 0x87, 0xb6, 0x69, 0xb2, 0xee, 0x65, 0x86,
          ],
          "RFC 8439, A.1, test vector #1"
      );
  }
  ```

  The others: `the_stream_number_is_the_nonce_as_rfc_8439_vector_five_shows`
  (stream `2 << 56`; its first block, as a literal, starts `0xc2, 0xc6,
  0x4d, 0x37, 0x8c, 0xd5, 0x36, 0x37`: copy the 64 bytes from RFC 8439,
  A.1, #5, or from the words of `rand_chacha` 0.9.0's `test_chacha_nonce`
  in the cargo registry, written little-endian);
  `the_second_block_follows_as_rfc_8439_vector_two_shows` (A.1, #2: the
  second 64 bytes of the zero seed's stream 0; the words are the second
  half of `rand_chacha` 0.9.0's `test_chacha_true_values_a`);
  `fills_in_pieces_across_blocks_give_the_same_bytes` (fills of 7 bytes
  from a zero seed on stream 0, 128 bytes across the pieces, compared with
  vectors #1 and #2 as literals);
  `two_streams_of_one_seed_start_with_different_blocks`;
  `a_seed_that_is_not_32_bytes_makes_no_generator`; in
  `crates/skein-oauth-accounts/src/tests.rs`,
  `two_sign_ins_draw_different_states_and_verifiers` and
  `a_verifier_is_43_url_safe_characters`. Each under 20 ms.
Proof: world, run once with `heavy cargo nextest run -p
  skein-oauth-accounts-world --test sign_in`: a public client's sign-in,
  its state and verifier drawn from a `CryptoRng`, is accepted by the
  fake issuer, which checks the state and S256 over the verifier
  (`a_public_client_signs_in_through_its_loopback_listener`). Under 30 s.
Don't: use `Rng` for a secret anywhere this increment touches; change
  `Rng`'s other holders (retry jitter, the simulator, io's temporary
  names: findings/skein.md, section 4).
Size: at most 220 lines of production code.
Removes: the accounts driver's `Rng` and its `u64` seed; `draw`'s own
  alphabet (`sign_in.rs:188`).

### 02.5 A record failed as "sign in again" dropped; handed-in accounts in the world
Scope: oauth.md, 6.4's "Loading again" and 6.8's "an access-only record
  expiring and handed in again".
Needs: nothing beyond the session's start.
Files: `crates/skein-oauth-accounts/src/component.rs:827-880`
  (`fail_exchange`; at most 15 lines); `src/tests.rs`;
  `src/private_tests.rs`; `tests/oauth-accounts/src/private.rs` (a
  `Story`), `tests/oauth-accounts/tests/private.rs`;
  `tests/oauth-accounts/src/world.rs` (`Story::HandedIn`,
  `Fact::Expiring`, and `start` making an `Account::HandedIn`; at most
  80 lines), `tests/oauth-accounts/tests/world.rs`. Shared hotspot:
  `component.rs` (skein 01.1 and 02.4).
Shape: in `fail_exchange`, an exchange that is a refresh
  (`!exchange.signing_in()`) and fails with
  `Failure::Exchange(oauth::Failure::Refused)` sets the account's slot to
  `State::Empty`, after the failure is told. An owner-kept account then
  fails its next grant as `Expired` until a sign-in; a privately kept one
  loads its file again at its next grant (`component/private.rs:16-35`).
  The fake issuer's refusal is `testing/skein-fake-oauth/src/lib.rs:392-399`'s
  `invalid_grant`.
Test shape:
  - Step, in `src/tests.rs`: `a_refresh_refused_as_sign_in_again_drops_an_owner_kept_record`;
    in `src/private_tests.rs`:
    `a_refresh_refused_as_sign_in_again_loads_the_file_at_the_next_grant`.
  - World, `tests/oauth-accounts/src/private.rs` gains `Story::SignInAgain`:
    the issuer refuses the refresh as `invalid_grant`; the file is then
    replaced with a newer record, as a sign-in from another process
    writes it; the next grant loads it and lends its token. Test in
    `tests/oauth-accounts/tests/private.rs`:
    `a_record_refused_as_sign_in_again_is_dropped_and_its_replacement_lent`.
  - World, `tests/oauth-accounts/src/world.rs` gains `Story::HandedIn`:
    an access-only record handed in, granted, heard `Expiring` once at
    its lead, a newer one handed in, granted again, then closed. Test in
    `tests/oauth-accounts/tests/world.rs`:
    `an_access_only_record_expiring_is_handed_in_again_and_lent`, its
    facts compared with the literal `[Granted(0), Expiring, Granted(1),
    Closed]`.
  - The existing `Story::Repeated` (a second rejection fails as
    `Expired`, the record kept) runs unchanged. Each test under 0.5 s.
Proof: world, run once with `heavy cargo nextest run -p
  skein-oauth-accounts-world --test private`:
  `a_record_refused_as_sign_in_again_is_dropped_and_its_replacement_lent`
  shows a revoked refresh token's record dropped, and a record a person
  wrote meanwhile lent at the next grant, without a restart. Under 30 s.
Don't: erase or rewrite the private file (it is the person's); drop a
  record on any failure other than `Refused` from a refresh.
Size: at most 30 lines of production code.
Removes: nothing.

### 02.s1 The generator's `boxed` variant, and output the lints accept

A step not in the plan (decisions.md, ruling 138), from the designer's
answer to jig 19.3 (skein `482691a7`, `docs/design/codec.md`, 3, 4, 6).
Scope: codec.md, 3: "**A variant marked `boxed`** (`report: boxed
  Report`) holds its record in a `Box`, and its bytes are the record's as
  they would be"; 6: "the generator writes what the lints accept
  (functions short enough, no nesting to collapse)".
Needs: 02.2.
Files: `testing/skein-codegen/src/*.rs`; `tests/codec/schema/lints-v1.schema`,
  its generated `tests/codec/src/lints/v1.rs` and goldens;
  `tests/codec/tests/{drift.rs, wire.rs}`.
Shape:
- `boxed` after a variant's colon: the enum holds `Box<Record>`; its
  constructor, accessor, encoding, decoding and heap worst case (the
  record "counting its own size", section 4) follow; its bytes equal the
  unboxed variant's.
- A bound checked inside an `option` is one `if let ... && ...`
  condition, as clippy's `collapsible_if` suggests.
- A generated function that would pass 100 lines (clippy's
  `too_many_lines`), such as a family's `worst_case_bytes` and
  `worst_case_heap`, is split into functions of at most 100 lines.
- Output for a schema whose generated code passes clippy on `main` today
  is unchanged, byte for byte.
Test shape: `lints-v1.schema` gains an enumeration with one `boxed`
  variant and an `option bytes N` field; `wire.rs` gains
  `a_boxed_variant_encodes_as_its_unboxed_twin`, the bytes a literal; a
  unit test in `testing/skein-codegen` generates a family of 120 records
  and finds no function past 100 lines. Each under 20 ms.
Proof: unit, run once with `heavy cargo nextest run -p skein-codegen -p
  skein-codec-tests`; and `git diff --stat --diff-filter=MD origin/main
  -- tests/codec/src/generated tests/codec/src/scalars
  tests/codec/src/secrets tests/codec/golden` prints nothing: no existing
  generated file or golden is changed or removed. The lint fixture's new
  goldens are added, and the commit body lists them (ruling 142). The
  gate's clippy checks the lint fixture.
Don't: guess a type's size; box a variant not marked; add an allowance
  to generated code; move golden bytes; change `sample-v1.schema`.
Size: at most 250 lines of production code; generated files and goldens
  are not counted.

### 02.s2 A list of fixed-width items generates no unused loop

A step not in the plan (decisions.md, ruling 152), after jig 22.1 found
it: for a `list N u64` field (`Hello.workstreams`), skein-codegen at
`e723e1db` writes an empty check loop and a measure loop whose `item_0`
is unused, which `-D warnings` refuses.
Scope: codec.md, 6: generated files "pass the workspace's lints with no
  allowances: the generator writes what the lints accept".
Needs: 02.s1.
Files: `testing/skein-codegen/src/*.rs`; `tests/codec/schema/lints-v1.schema`,
  its generated `tests/codec/src/lints/v1.rs` and goldens.
Shape: a list whose items need no check writes no check loop; a list of
  fixed-width items measures as its count times the item's width, with no
  loop. And (ruling 155) a record's generated functions (constructor
  check, measure, encode, decode) that would pass 100 lines, as jig
  30.1's 23-field `OwnerChipDetailsFields::encode` (107) does, are split
  into helpers of at most 100 lines, as 02.s1 split the family's worst
  cases. Output for a schema with neither is unchanged, byte for byte.
Test shape: `lints-v1.schema` gains a `list 4 u64` field, a `list 4 u32`
  one, and a record of 30 `bytes 4` fields; the gate's clippy over the
  fixture is the check.
Proof: unit, `heavy cargo nextest run -p skein-codegen -p
  skein-codec-tests` once; `git diff --stat --diff-filter=MD origin/main
  -- tests/codec/src/generated tests/codec/src/scalars
  tests/codec/src/secrets tests/codec/golden` prints nothing.
Don't: change the bytes of any encoding; add an allowance.
Size: at most 120 lines of production code.

### 02.s3 A long record's generated functions are split

A step not in the plan (decisions.md, rulings 155 and 157): 02.s2 landed
(skein `be47b1b5`) before ruling 155 added this to it.
Scope: codec.md, 6: "the generator writes what the lints accept
  (functions short enough ...)".
Needs: 02.s2.
Files: `testing/skein-codegen/src/*.rs`; `tests/codec/schema/lints-v1.schema`,
  its generated `tests/codec/src/lints/v1.rs` and goldens.
Shape: a record's generated functions (constructor check, measure,
  encode, decode) that would pass 100 lines, as jig 30.1's 23-field
  `OwnerChipDetailsFields::encode` (107) does, are split into helpers of
  at most 100 lines. Output for a schema whose functions are all within
  100 lines is unchanged, byte for byte.
Test shape: `lints-v1.schema` gains a record with enough `bytes 4`
  fields that each of its four functions would pass 100 lines unsplit
  (at most 100 fields; ruling 158); the gate's clippy over the fixture
  is the check.
Proof: as 02.s2's.
Don't: change the bytes of any encoding; add an allowance.
Size: at most 120 lines of production code (ruling 158).

### 02.s7 skein-codec's piece helpers

A step not in the plan (decisions.md, ruling 182), from the designer's
answer on jig 30.2 (skein `58cc905a`, `docs/design/codec.md`, 2 and 3: "A
sum crosses as one piece"); its text is the designer's.
Shape: write items into one bounded byte string, each after a `u32`
  length; split one back, refusing lengths that pass the end or do not end
  exactly at it, with `skein-codec`'s reason for malformed.
Test shape: unit tests of both, at a small bound.
Proof: skein's gate.
Don't: change an existing encoding.
Size: at most 80 lines of production code.

## Interface sketches

```rust
// skein_lib::Secret (crates/skein-lib/src/secret.rs)

/// A secret's owned bytes (lib.md, 6): a credential, a token, a key. Its
/// owner reads them with `expose`; `Debug` shows its length only.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Secret(Box<[u8]>);

impl Secret {
    /// Takes `bytes` as a secret.
    #[must_use]
    pub fn new(bytes: Box<[u8]>) -> Secret {
        todo!()
    }

    /// The bytes, for their one use: each call is visible in review.
    #[must_use]
    pub fn expose(&self) -> &[u8] {
        todo!()
    }

    /// How many bytes it holds.
    #[must_use]
    pub fn len(&self) -> usize {
        todo!()
    }

    /// Whether it holds none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        todo!()
    }
}

impl core::fmt::Debug for Secret {
    /// `Secret { len: N }`.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}
```

```rust
// skein_lib::base64 (crates/skein-lib/src/base64.rs)
use skein_lib::{Overflow, Writer};

/// Which alphabet, and its padding (lib.md, 6.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Alphabet {
    /// `+` and `/`, padded with `=` to whole groups of four.
    Standard,
    /// `-` and `_`, unpadded (RFC 7515, RFC 7636).
    Url,
}

/// Why a text was refused; nothing was set aside.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    /// A byte outside the alphabet, at this offset of the text.
    Symbol { at: u32 },
    /// Padding where the alphabet takes none, or not at the end.
    Padding { at: u32 },
    /// A length no encoding has.
    Length,
    /// Trailing bits that are not zero.
    Bits,
    /// More decoded bytes than the maximum.
    TooLong { max: u32 },
}

/// The text's length for `bytes` bytes, or `None` past a `u32`.
#[must_use]
pub fn encoded_len(alphabet: Alphabet, bytes: u32) -> Option<u32> {
    todo!()
}

/// Writes `bytes`' text into `out`, which has `encoded_len` room left, or
/// writes nothing.
pub fn encode(alphabet: Alphabet, bytes: &[u8], out: &mut Writer) -> Result<(), Overflow> {
    todo!()
}

/// Checks `text` and gives its decoded length, setting nothing aside.
pub fn decoded_len(alphabet: Alphabet, text: &[u8]) -> Result<u32, Error> {
    todo!()
}

/// Decodes `text`, refusing past `max` decoded bytes before setting anything aside.
pub fn decode(alphabet: Alphabet, text: &[u8], max: u32) -> Result<Box<[u8]>, Error> {
    todo!()
}

/// Text taken in pieces, under `decode`'s rules and maximum.
#[derive(Debug)]
pub struct Decoder {/* the alphabet, the maximum, what a piece left of a group, the bytes so far */}

impl Decoder {
    /// A decoder of at most `max` bytes.
    #[must_use]
    pub fn new(alphabet: Alphabet, max: u32) -> Decoder {
        todo!()
    }

    /// The next piece of text.
    pub fn push(&mut self, piece: &[u8]) -> Result<(), Error> {
        todo!()
    }

    /// The bytes decoded so far.
    #[must_use]
    pub fn len(&self) -> u32 {
        todo!()
    }

    /// Whether none are.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// The decoded bytes, once the text ended where a text may.
    pub fn finish(self) -> Result<Box<[u8]>, Error> {
        todo!()
    }
}
```

```rust
// skein_lib::CryptoRng (crates/skein-lib/src/crypto_rng.rs)
use skein_lib::Secret;

/// `ChaCha20` keyed by a 32-byte seed and a stream number (lib.md, 8): held
/// in a layer's state, for whatever must not be guessed. No heap.
#[derive(Clone)]
pub struct CryptoRng {/* the key's words, the stream, the block counter, the unused rest of the last block */}

impl CryptoRng {
    /// The generator of `seed` on `stream`, or `None` unless `seed` holds 32 bytes.
    #[must_use]
    pub fn new(seed: &Secret, stream: u64) -> Option<CryptoRng> {
        todo!()
    }

    /// Fresh bytes into `out`.
    pub fn fill(&mut self, out: &mut [u8]) {
        todo!()
    }
}

impl core::fmt::Debug for CryptoRng {
    /// `CryptoRng { stream: N }`: never the key or the block.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}
```

```rust
// skein_shell (crates/skein-shell/src/startup.rs and seed.rs)
use std::path::Path;

use skein_io::file::Unsafe;
use skein_io::kernel;
use skein_lib::Secret;

/// Why startup's read returned no bytes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unread {
    /// The requested path was absent.
    Absent,
    /// The opened entry was not a regular file.
    NotAFile,
    /// The file exceeded startup's byte bound.
    TooLarge { max: u32 },
    /// The adapter's kernel call failed.
    Kernel(kernel::Error),
    /// A private read found this unsafe, and read nothing (shell.md, 6.1).
    Unsafe(Unsafe),
}

/// A secret configuration names, read within `max` bytes once io's
/// private-file checks pass for `user`, the effective user startup read.
pub fn read_private_file(path: &Path, max: u32, user: u32) -> Result<Secret, Unread> {
    todo!()
}

/// 32 bytes from `getrandom`, beside the seed and never derived from it (shell.md, 5).
pub fn crypto_seed() -> Result<Secret, i32> {
    todo!()
}
```

`skein_oauth_accounts::Component::new` becomes `pub fn new(accounts:
List<Account>, limits: &Limits, crypto: CryptoRng) -> Result<Component,
Unusable>`, documented "Validates configuration before the loop; the
generator is the owner's, keyed by its process's seed on a stream of its
own". Its other entry points are unchanged.

Who owns what: the shell draws the seed once; the service's startup
makes each layer's `CryptoRng` from it with a stream number of its own;
a world makes its seed from its own `u64`. The accounts driver owns its
generator from `new` until it is dropped.

## Questions to raise when reached

- **Stream numbers across a process's layers.** lib.md, 8 asks that they
  be "distinct for every generator of a process", but no document says
  who hands them out. This session's users take stream 0 (one generator
  each). A service with two (jig's engine: OAuth's driver and the
  clients' sign-ins) needs a rule; raise it before a second generator
  lands in one process.
- **A private file's path through a system link.** shell.md, 6.1 refuses
  a link in any component of the path, so a configured path under a
  directory the system links (`/var/run` on some systems) is refused.
  This session implements the rule as written.

## Done when

1. lib holds the pieces: `git -C ~/src/rust/skein grep -n -E '^pub (struct
   Secret|mod base64|struct CryptoRng)' origin/main -- crates/skein-lib/src`
   prints three lines.
2. No base64 copy is left: `git -C ~/src/rust/skein grep -n -E
   "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789|b'A'..=b'Z'"
   origin/main -- crates testing tests examples ':!crates/skein-lib'` prints nothing
   (at the session's start it prints the six copies' lines).
3. The accounts driver holds no `Rng`: `git -C ~/src/rust/skein grep -n 'Rng'
   origin/main -- crates/skein-oauth-accounts/src` prints only `CryptoRng` lines.
4. The five increments landed through `land`: `git -C ~/src/rust/skein
   log --format='%(trailers:key=Step,valueonly)' origin/main` lists
   `skein-02.1` to `skein-02.5`, each commit's `Gate-Result` naming
   `scripts/check.sh passed`.
