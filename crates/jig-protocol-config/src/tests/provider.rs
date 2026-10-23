use super::fixtures;
use crate::{Path, Problem, Segment, What, clients::provider_key};
const PROVIDER: &[u8] = br#""provider":{"key":"secrets/provider.pem","clients":[{"id":"app","secret":"secrets/app","redirect_uri":"https://app.example.com/callback"}],"addresses":[3]},"link":true"#;
fn bytes() -> Box<[u8]> {
    let bytes = fixtures::with(fixtures::ENGINE_SMALL, br#""link": true"#, PROVIDER);
    fixtures::with(
        &bytes,
        br#"{"number": 2, "kind": "deployment"}"#,
        br#"{"number": 2, "kind": "deployment"},{"number":3,"kind":"application"}"#,
    )
}
fn provider_path() -> Path {
    fixtures::path(&[b"clients", b"provider"])
}
fn client_path() -> Path {
    provider_path().member(b"clients").child(Segment::Index(0))
}
fn refuses(from: &[u8], to: &[u8], path: Path, what: What) {
    let document = fixtures::with(&bytes(), from, to);
    let found = fixtures::engine(&document).unwrap_err();
    assert_eq!(found.problems.as_slice(), &[Problem { path, what }]);
}
#[test]
fn provider_is_optional_and_its_private_paths_are_data_paths() {
    assert!(fixtures::engine(fixtures::ENGINE_SMALL).unwrap().clients.provider.is_none());
    let null = fixtures::with(fixtures::ENGINE_SMALL, br#""link": true"#, br#""link":true,"provider":null"#);
    assert!(fixtures::engine(&null).unwrap().clients.provider.is_none());
    let bytes = bytes();
    let document = crate::document(&bytes, &crate::DocumentLimits { bytes: 8192, tokens: 1536, depth: 16 }).unwrap();
    let dirs = crate::Directories::given(b"/config", b"/data", b"/cache").unwrap();
    let engine = crate::engine(&document, 0, &dirs).unwrap();
    let provider = engine.clients.provider.unwrap();
    assert_eq!(provider.key.as_ref(), b"/data/secrets/provider.pem");
    assert_eq!(provider.clients[0].secret.as_ref(), b"/data/secrets/app");
    assert_eq!(provider.addresses.as_ref(), &[3]);
}
#[test]
fn client_ids_are_visible_ascii_and_unique_and_there_is_at_least_one_client() {
    for (id, what) in [
        ("", What::Bound),
        ("a b", What::Form),
        ("a\\t", What::Form),
        ("a\\u007f", What::Form),
        ("\\u00e9", What::Form),
    ] {
        let id = json_text(b"id", id.as_bytes());
        refuses(br#""id":"app""#, &id, client_path().member(b"id"), what);
    }
    for size in [1, 255, 256] {
        let value = json_text(b"id", &repeated(if size == 255 { b'~' } else { b'!' }, size));
        let document = fixtures::with(&bytes(), br#""id":"app""#, &value);
        if size <= 255 {
            fixtures::engine(&document).expect("valid configuration");
        } else {
            assert_eq!(fixtures::engine(&document).unwrap_err().problems.get(0).unwrap().what, What::Bound);
        }
    }
    refuses(
        br#""clients":[{"id":"app","secret":"secrets/app","redirect_uri":"https://app.example.com/callback"}]"#,
        br#""clients":[]"#,
        provider_path().member(b"clients"),
        What::Bound,
    );
    refuses(br#""clients":[{"id":"app","secret":"secrets/app","redirect_uri":"https://app.example.com/callback"}]"#, br#""clients":[{"id":"app","secret":"a","redirect_uri":"https://app.example.com"},{"id":"app","secret":"b","redirect_uri":"https://app.example.com"}]"#, provider_path().member(b"clients").child(Segment::Index(1)).member(b"id"), What::Unique);
}
#[test]
fn provider_redirects_require_https_or_loopback_http_with_a_port_and_no_fragment() {
    for uri in [
        "https://app.example.com",
        "https://app.example.com:443/cb?x=1",
        "http://127.0.0.1:8080/cb",
        "http://[::1]:80/cb",
        "http://localhost:1/cb",
    ] {
        let value = json_text(b"redirect_uri", uri.as_bytes());
        fixtures::engine(&fixtures::with(&bytes(), br#""redirect_uri":"https://app.example.com/callback""#, &value))
            .expect("valid redirect URI");
    }
    for uri in [
        "http://app.example.com:80/cb",
        "http://localhost/cb",
        "http://127.0.0.1/cb",
        "http://[::1]/cb",
        "http://localhost:/cb",
        "http://localhost:0/cb",
        "http://localhost:65536/cb",
        "http://localhost:80.evil/cb",
        "http://localhost.evil:80/cb",
        "https://app.example.com/cb#frag",
        "https://",
        "https:///cb",
        "ftp://localhost:80/cb",
        "https://user@app.example.com/cb",
        "https://app.example.com/has space",
    ] {
        let value = json_text(b"redirect_uri", uri.as_bytes());
        refuses(
            br#""redirect_uri":"https://app.example.com/callback""#,
            &value,
            client_path().member(b"redirect_uri"),
            What::Form,
        );
    }
    for size in [2048_usize, 2049] {
        let uri = joined(&[b"https://app.example.com/", &repeated(b'a', size.checked_sub(24).unwrap())]);
        assert_eq!(uri.len(), size);
        let value = json_text(b"redirect_uri", &uri);
        let document = fixtures::with(&bytes(), br#""redirect_uri":"https://app.example.com/callback""#, &value);
        if size == 2048 {
            fixtures::engine(&document).expect("valid configuration");
        } else {
            assert_eq!(
                fixtures::engine(&document).unwrap_err().problems.get(0).unwrap(),
                &Problem { path: client_path().member(b"redirect_uri"), what: What::Bound }
            );
        }
    }
}
#[test]
fn address_providers_must_be_of_kind_application_and_paths_must_be_data_paths() {
    for value in [br#""addresses":[0]"#.as_slice(), br#""addresses":[1]"#, br#""addresses":[2]"#, br#""addresses":[4]"#]
    {
        refuses(
            br#""addresses":[3]"#,
            value,
            provider_path().member(b"addresses").child(Segment::Index(0)),
            What::Refers,
        );
    }
    refuses(
        br#""key":"secrets/provider.pem""#,
        br#""key":"../provider.pem""#,
        provider_path().member(b"key"),
        What::Relative,
    );
    refuses(br#""secret":"secrets/app""#, br#""secret":"../app""#, client_path().member(b"secret"), What::Relative);
}
#[test]
fn provider_limits_are_positive_named_overrides_with_profile_defaults() {
    for (profile, codes, tokens) in [(crate::Profile::Small, 16, 64), (crate::Profile::Standard, 256, 1024)] {
        let mut limits = crate::profile(profile);
        assert_eq!((limits.clients.provider_codes, limits.clients.provider_tokens), (codes, tokens));
        for name in [b"clients.provider_codes".as_slice(), b"clients.provider_tokens"] {
            assert_eq!(crate::overrides::apply(&mut limits, name, 0), Err(What::Override));
            crate::overrides::apply(&mut limits, name, 7).unwrap();
            assert_eq!(crate::overrides::apply(&mut limits, name, u64::MAX), Err(What::Override));
        }
        assert_eq!((limits.clients.provider_codes, limits.clients.provider_tokens), (7, 7));
        limits.clients.provider_codes = 0;
        assert!(jig_protocol_engine_clients::worst_case(&limits.clients).is_none());
        limits.clients.provider_codes = 1;
        limits.clients.provider_tokens = 0;
        assert!(jig_protocol_engine_clients::worst_case(&limits.clients).is_none());
    }
}
#[test]
fn provider_key_reads_only_pkcs8_armor_and_base64_without_interpreting_der() {
    let arbitrary = b"this is arbitrary DER-shaped test input, not a key";
    let size =
        skein_lib::base64::encoded_len(skein_lib::base64::Alphabet::Standard, u32::try_from(arbitrary.len()).unwrap())
            .unwrap();
    let mut writer = skein_lib::Writer::new(usize::try_from(size).unwrap());
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Standard, arbitrary, &mut writer).unwrap();
    let encoded = writer.finish();
    for eol in [b"\n".as_slice(), b"\r\n", b"\r"] {
        let pem = joined(&[b"-----BEGIN PRIVATE KEY-----", eol, &encoded, eol, b"-----END PRIVATE KEY-----", eol]);
        assert_eq!(provider_key(&pem).unwrap().expose(), arbitrary);
    }
    for pem in [
        b"-----BEGIN RSA PRIVATE KEY-----\nYWJj\n-----END RSA PRIVATE KEY-----\n".as_slice(),
        b"-----BEGIN ENCRYPTED PRIVATE KEY-----\nYWJj\n-----END ENCRYPTED PRIVATE KEY-----\n",
        b"-----BEGIN CERTIFICATE-----\nYWJj\n-----END CERTIFICATE-----\n",
        b"-----BEGIN PRIVATE KEY-----\n!!!!\n-----END PRIVATE KEY-----\n",
        b"-----BEGIN PRIVATE KEY-----\n\n-----END PRIVATE KEY-----\n",
        b"prefix\n-----BEGIN PRIVATE KEY-----\nYWJj\n-----END PRIVATE KEY-----\n",
        b"-----BEGIN PRIVATE KEY-----\nYWJj\n-----END PRIVATE KEY-----\nsuffix",
        b"-----BEGIN PRIVATE KEY-----\nYWJj-----END PRIVATE KEY-----\n",
    ] {
        let problem = provider_key(pem).unwrap_err();
        assert_eq!(problem.path, provider_path().member(b"key"));
        assert_eq!(
            problem.what,
            What::ProviderKey { generate: "openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:3072" }
        );
    }
}

fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).unwrap();
    }
    let mut writer = skein_lib::Writer::new(length);
    for part in parts {
        writer.put(part).unwrap();
    }
    writer.finish()
}
fn json_text(name: &[u8], value: &[u8]) -> Box<[u8]> {
    joined(&[b"\"", name, b"\":\"", value, b"\""])
}
fn repeated(byte: u8, count: usize) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(count);
    for _ in 0..count {
        writer.put(&[byte]).unwrap();
    }
    writer.finish()
}
